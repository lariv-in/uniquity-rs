use lariv_rs::html_form::{
    html_form,
    widgets::{ForeignKey, Text},
};
use lariv_rs::plugins::users::routes::UsersSelectRouteTag;

use super::routes::EmployeesSelectRouteTag;

#[html_form]
pub struct EmployeeForm {
    #[form(
        label = "User",
        required,
        widget = ForeignKey,
        route = UsersSelectRouteTag,
        swap_key = "fk-user",
        display = "user_display",
        placeholder = "Select user…"
    )]
    pub user_id: i64,
}

#[html_form]
pub struct EmployeeFilterForm {
    #[form(label = "User name", widget = Text)]
    pub name: String,

    #[form(label = "Email", widget = Text)]
    pub email: String,
}

#[html_form]
pub struct PointsForm {
    #[form(
        label = "Employee",
        required,
        widget = ForeignKey,
        route = EmployeesSelectRouteTag,
        swap_key = "fk-employee",
        display = "employee_display",
        placeholder = "Select employee…"
    )]
    pub to_employee_id: i64,

    #[form(label = "Points", required, widget = Text)]
    pub points: String,
}
