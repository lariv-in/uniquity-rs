use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        FieldText, ObjectList, ShellChrome, TableColumnHeader, TableRow, data_table_list,
        field_text, row_attr_select,
    },
    picker::RenderPickerSelect,
    template::RenderTemplate,
};

use crate::keys::{SourceDocSelectModalKey, SourceDocSelectTableKey};

use super::common::render_picker_pagination;

#[derive(Clone)]
pub struct SourceDocRow {
    pub id: i64,
    pub source_doc_type: String,
    pub source_doc_id: i64,
    pub label: String,
}

#[derive(Generic)]
pub struct SourceDocSelectPage {
    pub docs: ObjectList<SourceDocRow>,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<SourceDocSelectTableKey, SourceDocSelectModalKey> for SourceDocSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Type", sort_url: None, push_url: false },
            TableColumnHeader { label: "Reference", sort_url: None, push_url: false },
            TableColumnHeader { label: "Label", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .docs
            .items
            .iter()
            .map(|d| TableRow {
                attrs: row_attr_select(&self.target_input, &d.id.to_string(), &d.label),
                cells: vec![
                    field_text(FieldText { value: &d.source_doc_type, classes: "" }),
                    field_text(FieldText {
                        value: &d.source_doc_id.to_string(),
                        classes: "",
                    }),
                    field_text(FieldText { value: &d.label, classes: "" }),
                ],
            })
            .collect();
        let pagination = render_picker_pagination::<SourceDocSelectModalKey>(
            &self.path_and_query,
            self.docs.number,
            self.docs.num_pages,
        );
        data_table_list::<SourceDocSelectTableKey>(
            "Select Source Document",
            html! {},
            &headers,
            &rows,
            pagination,
        )
    }
}

impl RenderTemplate for SourceDocSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}
