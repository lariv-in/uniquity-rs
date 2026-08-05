use lariv_rs::html_form::{
    html_form,
    widgets::Text,
};

#[html_form]
pub struct CustomerForm {
    #[form(label = "Name", required, widget = Text)]
    pub name: String,

    #[form(label = "Address line 1", widget = Text)]
    pub address_line_1: String,

    #[form(label = "Address line 2", widget = Text)]
    pub address_line_2: String,

    #[form(label = "City", widget = Text)]
    pub city: String,

    #[form(label = "Pincode", widget = Text)]
    pub pincode: String,

    #[form(label = "State", widget = Text)]
    pub state: String,

    #[form(label = "GSTIN", widget = Text)]
    pub gstin: String,

    #[form(label = "PAN", widget = Text)]
    pub pan: String,

    #[form(label = "Phone", widget = Text)]
    pub phone: String,

    #[form(label = "Email", widget = Text)]
    pub email: String,

    #[form(label = "Website", widget = Text)]
    pub website: String,
}

#[html_form]
pub struct CustomerFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,

    #[form(label = "Email", widget = Text)]
    pub email: String,
}
