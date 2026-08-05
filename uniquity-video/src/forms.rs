use lariv_rs::html_form::{
    html_form,
    widgets::{ForeignKey, ManyToMany, Text},
};

#[html_form]
pub struct RawFootageForm {
    #[form(label = "Title", required, widget = Text)]
    pub title: String,

    #[form(
        label = "Files",
        widget = ManyToMany,
        url = "/filesystem/file-select/",
        swap_key = "fk-raw-files",
        placeholder = "Select files…"
    )]
    pub files: Vec<i64>,

    #[form(
        label = "Assigned to",
        required,
        widget = ForeignKey,
        url = "/video/raw/select-employee/",
        swap_key = "fk-assigned-employee",
        display = "assigned_display",
        placeholder = "Select employee…"
    )]
    pub assigned_to_id: i64,
}

#[html_form]
pub struct RawFootageFilterForm {
    #[form(label = "Title", widget = Text)]
    pub title: String,
}

#[html_form]
pub struct EditedVideoForm {
    #[form(
        label = "Raw footage",
        required,
        widget = ForeignKey,
        url = "/video/raw/select/",
        swap_key = "fk-raw-footage",
        display = "raw_display",
        placeholder = "Select raw footage…"
    )]
    pub raw_footage_id: i64,

    #[form(
        label = "Output file",
        required,
        widget = ForeignKey,
        url = "/filesystem/file-select/",
        swap_key = "fk-edited-vnode",
        display = "vnode_display",
        placeholder = "Select output file…"
    )]
    pub edited_v_node_id: i64,
}

#[html_form]
pub struct PublishedVideoForm {
    #[form(
        label = "Edited video",
        required,
        widget = ForeignKey,
        url = "/video/edited/select/",
        swap_key = "fk-edited-video",
        display = "edited_display",
        placeholder = "Select edited cut…"
    )]
    pub edited_video_id: i64,

    #[form(label = "YouTube link or video ID", required, widget = Text)]
    pub you_tube_video_id: String,
}

#[html_form]
pub struct EditorPointsForm {
    #[form(label = "Points", required, widget = Text)]
    pub points: String,
}
