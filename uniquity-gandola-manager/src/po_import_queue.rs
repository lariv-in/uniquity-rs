//! Ephemeral in-memory queue for purchase-order PDF imports.
//!
//! Jobs live only in process memory (lost on restart). Total active jobs are
//! capped; Gemini extraction runs with a separate concurrency limit.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sea_orm::DatabaseConnection;
use tokio::sync::{Mutex, Semaphore};

use lariv_rs::plugins::filesystem::state::FilesystemState;

use crate::po_from_pdf::{extract_purchase_order_from_pdf, form_from_extracted, store_purchase_order_pdf};

/// Maximum queued + running jobs across all sites.
pub const MAX_PO_IMPORT_JOBS: usize = 32;

/// Maximum concurrent Gemini extractions.
pub const MAX_PO_IMPORT_CONCURRENCY: usize = 4;

const TERMINAL_RETENTION: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoImportJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl PoImportJobStatus {
    pub fn is_active(self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Reading PDF…",
            Self::Succeeded => "Created",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoImportJobSnapshot {
    pub id: u64,
    pub site_id: i64,
    pub filename: String,
    pub status: PoImportJobStatus,
    pub error: Option<String>,
    pub purchase_order_id: Option<i64>,
    pub purchase_order_number: Option<String>,
}

impl PoImportJobSnapshot {
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }
}

#[derive(Clone)]
pub struct PoImportEnqueue {
    pub site_id: i64,
    pub customer_id: i64,
    pub filename: String,
    pub pdf_bytes: Vec<u8>,
    pub timezone: String,
}

#[derive(Clone)]
struct WorkerDeps {
    db: DatabaseConnection,
    fs: FilesystemState,
    api_key: String,
    model: String,
}

struct JobRecord {
    id: u64,
    site_id: i64,
    customer_id: i64,
    filename: String,
    pdf_bytes: Arc<Vec<u8>>,
    timezone: String,
    status: PoImportJobStatus,
    error: Option<String>,
    purchase_order_id: Option<i64>,
    purchase_order_number: Option<String>,
    updated_at: Instant,
}

impl JobRecord {
    fn snapshot(&self) -> PoImportJobSnapshot {
        PoImportJobSnapshot {
            id: self.id,
            site_id: self.site_id,
            filename: self.filename.clone(),
            status: self.status,
            error: self.error.clone(),
            purchase_order_id: self.purchase_order_id,
            purchase_order_number: self.purchase_order_number.clone(),
        }
    }

    fn is_active(&self) -> bool {
        self.status.is_active()
    }
}

struct Inner {
    next_id: AtomicU64,
    jobs: Mutex<HashMap<u64, JobRecord>>,
    slots: Arc<Semaphore>,
}

#[derive(Clone)]
pub struct PoImportQueue {
    inner: Arc<Inner>,
}

impl Default for PoImportQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl PoImportQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                next_id: AtomicU64::new(1),
                jobs: Mutex::new(HashMap::new()),
                slots: Arc::new(Semaphore::new(MAX_PO_IMPORT_CONCURRENCY)),
            }),
        }
    }

    pub async fn jobs_for_site(&self, site_id: i64) -> Vec<PoImportJobSnapshot> {
        let jobs = self.inner.jobs.lock().await;
        let mut out: Vec<_> = jobs
            .values()
            .filter(|j| j.site_id == site_id)
            .map(JobRecord::snapshot)
            .collect();
        out.sort_by_key(|j| j.id);
        out
    }

    pub async fn site_has_active_jobs(&self, site_id: i64) -> bool {
        let jobs = self.inner.jobs.lock().await;
        jobs.values()
            .any(|j| j.site_id == site_id && j.is_active())
    }

    /// Drop finished jobs for a site (e.g. after the user dismisses the panel).
    pub async fn clear_terminal_for_site(&self, site_id: i64) {
        let mut jobs = self.inner.jobs.lock().await;
        jobs.retain(|_, j| j.site_id != site_id || j.is_active());
    }

    /// Enqueue PDF imports and spawn workers. Fails if the active job cap would be exceeded.
    pub async fn enqueue_many(
        &self,
        items: Vec<PoImportEnqueue>,
        db: DatabaseConnection,
        fs: FilesystemState,
        api_key: String,
        model: String,
    ) -> Result<Vec<u64>, String> {
        if items.is_empty() {
            return Err("Upload at least one PDF file".into());
        }

        let deps = WorkerDeps {
            db,
            fs,
            api_key,
            model,
        };

        let mut ids = Vec::with_capacity(items.len());
        {
            let mut jobs = self.inner.jobs.lock().await;
            prune_stale_terminal(&mut jobs);

            let active = jobs.values().filter(|j| j.is_active()).count();
            if active + items.len() > MAX_PO_IMPORT_JOBS {
                return Err(format!(
                    "Import queue is full ({active} active, max {MAX_PO_IMPORT_JOBS}). Wait for current imports to finish."
                ));
            }

            // Replace prior finished results for this site with the new batch.
            if let Some(site_id) = items.first().map(|i| i.site_id) {
                jobs.retain(|_, j| j.site_id != site_id || j.is_active());
            }

            for item in items {
                let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
                jobs.insert(
                    id,
                    JobRecord {
                        id,
                        site_id: item.site_id,
                        customer_id: item.customer_id,
                        filename: item.filename,
                        pdf_bytes: Arc::new(item.pdf_bytes),
                        timezone: item.timezone,
                        status: PoImportJobStatus::Queued,
                        error: None,
                        purchase_order_id: None,
                        purchase_order_number: None,
                        updated_at: Instant::now(),
                    },
                );
                ids.push(id);
            }
        }

        for id in ids.iter().copied() {
            let queue = self.clone();
            let deps = deps.clone();
            tokio::spawn(async move {
                queue.run_job(id, deps).await;
            });
        }

        Ok(ids)
    }

    async fn run_job(&self, id: u64, deps: WorkerDeps) {
        let Some((filename, pdf_bytes, site_id, customer_id, timezone)) =
            self.take_job_payload(id).await
        else {
            return;
        };

        let Ok(_permit) = self.inner.slots.acquire().await else {
            self.fail_job(id, "Import worker unavailable".into()).await;
            return;
        };

        self.set_status(id, PoImportJobStatus::Running).await;

        let extracted =
            match extract_purchase_order_from_pdf(&deps.api_key, &deps.model, pdf_bytes.as_slice())
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    self.fail_job(id, e).await;
                    return;
                }
            };

        let file_id =
            match store_purchase_order_pdf(&deps.fs, &filename, pdf_bytes.as_slice().to_vec()).await
            {
                Ok(fid) => fid,
                Err(e) => {
                    self.fail_job(id, format!("Could not store the PDF: {e}"))
                        .await;
                    return;
                }
            };

        let form = form_from_extracted(&extracted, customer_id, site_id, file_id);
        match crate::po_persist::persist_new_purchase_order(&deps.db, &form, &timezone).await {
            Ok(saved) => {
                self.succeed_job(id, saved.id, saved.number).await;
            }
            Err(e) => {
                self.fail_job(id, e).await;
            }
        }
    }

    async fn take_job_payload(
        &self,
        id: u64,
    ) -> Option<(String, Arc<Vec<u8>>, i64, i64, String)> {
        let jobs = self.inner.jobs.lock().await;
        let job = jobs.get(&id)?;
        Some((
            job.filename.clone(),
            job.pdf_bytes.clone(),
            job.site_id,
            job.customer_id,
            job.timezone.clone(),
        ))
    }

    async fn set_status(&self, id: u64, status: PoImportJobStatus) {
        let mut jobs = self.inner.jobs.lock().await;
        if let Some(job) = jobs.get_mut(&id) {
            job.status = status;
            job.updated_at = Instant::now();
        }
    }

    async fn fail_job(&self, id: u64, message: String) {
        let mut jobs = self.inner.jobs.lock().await;
        if let Some(job) = jobs.get_mut(&id) {
            job.status = PoImportJobStatus::Failed;
            job.error = Some(message);
            job.pdf_bytes = Arc::new(Vec::new());
            job.updated_at = Instant::now();
        }
    }

    async fn succeed_job(&self, id: u64, po_id: i64, number: String) {
        let mut jobs = self.inner.jobs.lock().await;
        if let Some(job) = jobs.get_mut(&id) {
            job.status = PoImportJobStatus::Succeeded;
            job.purchase_order_id = Some(po_id);
            job.purchase_order_number = Some(number);
            job.pdf_bytes = Arc::new(Vec::new());
            job.updated_at = Instant::now();
        }
    }
}

fn prune_stale_terminal(jobs: &mut HashMap<u64, JobRecord>) {
    let now = Instant::now();
    jobs.retain(|_, j| j.is_active() || now.duration_since(j.updated_at) < TERMINAL_RETENTION);
}
