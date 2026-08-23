# Uniquity Ventures (Rust)

Rust port of the Uniquity Ventures deployment using [lariv-rs](../../lariv-rs/).

## Quick start

```bash
cd deployments/uniquity_ventures-rs
createdb -U postgres uniquity 2>/dev/null || true
cargo run -- migrate               # schema + finance seed migrations
cargo run -- seed                  # admin user bootstrap
cargo run                          # serve on :42070 (default bind)
```

Ensure PostgreSQL is running and `database_url` in `config.toml` points at the `uniquity` database.

Finance seed data (chart of accounts, GST, default preferences) runs inside **migrations**, not `seed`. The `seed` command only bootstraps the admin user and roles via the users plugin.

## Environment overrides

| Variable | Overrides |
|----------|-----------|
| `DATABASE_URL` | `database_url` in config |
| `BIND` | `bind` in config |
| `RUST_LOG` | tracing filter (default: `warn`) |

## External tools

| Tool | Required for |
|------|--------------|
| PostgreSQL | Runtime and integration tests |

Invoice PDF templates in Accounting preferences use [Minijinja](https://github.com/mitsuhiko/minijinja) syntax (not Go `text/template`). PDFs are compiled with the [Typst Rust crate](https://docs.rs/typst/latest/typst/) (no external `typst` CLI required). When preferences are empty, the built-in example template is used (`uniquity-finance-accounts/templates/example_invoice_pdf_template.typ.tmpl`). Custom functions: `num2words`, `num2wordsAnd`, `num2wordsRupees`, `invoiceGrandTotalWords()`, `urlImage('https://…')`. Date fields: `DatetimeDisplay` (`DD/MM/YYYY`), `DatetimeYear`, `DatetimeMonth`, `DatetimeDay`; payments expose `DatetimeDisplay`.

## Installed plugins

| Plugin | URL prefix | Notes |
|--------|------------|-------|
| `uniquity-finance-accounts` | `/finance/` | Chart of accounts, journals, preferences |
| `customer` | `/customers/` | Customer CRUD |
| `crm` | `/crm/leads` | Leads, companies, contacts, deals |
| `uniquity-finance-creditnotes` | `/finance-creditnotes/` | Credit notes with auto-reversing JE |
| `uniquity-finance-taxes` | `/finance-taxes/` | Tax configuration |
| `uniquity-finance-products` | `/finance-products/` | Products with M2M taxes |
| `uniquity-finance-invoices` | `/finance-invoices/` | Invoices, payments, posting |
| `uniquity-finance-indian` | — | Migrations only (GST seed) |
| `uniquity-employees` | `/employees/` | Staff + points ledger |
| `uniquity-video` | `/video/` | Raw/edited/published pipeline |

Core lariv-rs plugins: `users`, `filesystem`, `llm_assistant`, `otp`, `pwa`, `dashboard`.

## Config

| Key | Purpose |
|-----|---------|
| `database_url` | PostgreSQL DSN |
| `bind` | Listen address (default `0.0.0.0:42070`) |
| `[users].adminEmail` / `adminPassword` | Bootstrap admin via `seed` |
| `[filesystem].localDir` | Upload storage |
| `[uniquity_video].youtubeApiKey` | YouTube metadata for published videos |

## Tests

```bash
cargo test                                    # mount smoke (sqlite memory)
DATABASE_URL=postgres://... cargo test -- --ignored  # Postgres integration
```
