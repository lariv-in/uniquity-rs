use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonLink, ButtonSubmit, FieldText, FieldTitle, FormOpts,
        ManyToManyItem, ObjectList, PaginationPage, ShellChrome,
        SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter, TableColumnHeader,
        TablePagination, TableRow, button_clear, button_delete, button_link,
        button_submit, container_column, container_row, data_table_list,
        detail, field_text, field_title, form, form_hx_get_route, form_hx_post_main,
        label_inline, pagination_pages,
        row_attr_navigate_route, row_attr_select,
        table_button_filter, table_pagination,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    picker::RenderPickerSelect,
};

use uniquity_finance_accounts::accounting_detail_menu::{
    DetailMenuNavItem, detail_sidebar_menu,
};
use uniquity_finance_accounts::templates::{
    app_scaffold, app_scaffold_with_sidebar, layout_main_content, layout_with_entity_sidebar,
    layout_with_sidebar,
};

use super::forms::{
    ProductFilterForm, ProductFilterFormField, ProductForm, ProductFormField,
};
use super::keys::{ProductSelectModalKey, ProductSelectTableKey, ProductTableKey};
use super::routes::{
    ProductCreateGetRouteTag, ProductCreatePostRouteTag, ProductDefaultRouteTag,
    ProductDeletePostRouteTag, ProductDetailRouteTag,
    ProductEditGetRouteTag, ProductEditPostRouteTag,
};

fn product_detail_menu(id: i64, name: &str, active: &str, can_edit: bool) -> Markup {
    let menu_title = format!("Product: {name}");
    let detail_url = ProductDetailRouteTag::new(id).url();
    let mut nav = vec![DetailMenuNavItem {
        title: "Product Detail",
        url: detail_url,
        active: active == "detail",
    }];
    if can_edit {
        nav.push(DetailMenuNavItem {
            title: "Edit Product",
            url: ProductEditGetRouteTag::new(id).url(),
            active: active == "edit",
        });
    }
    detail_sidebar_menu(
        menu_title,
        "Back to Products",
        ProductDefaultRouteTag.url(),
        &nav,
        None,
        html! {},
    )
}

lariv_rs::define_register_items! {
    plugin: UniquityFinanceProductsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ProductListIdx: ProductListPageTag => ProductListPage,
        ProductDetailIdx: ProductDetailPageTag => ProductDetailPage,
        ProductFormIdx: ProductFormPageTag => ProductFormPage,
        ProductSelectIdx: ProductSelectPageTag => ProductSelectPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: UniquityFinanceProductsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

fn product_filter_form(name: &str, reference: &str) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<ProductTableKey, ProductDefaultRouteTag>(
            ProductDefaultRouteTag,
        ),
        inputs: ProductFilterForm::render_inputs(
            &FormCtx::form::<ProductFilterForm>()
                .value(ProductFilterFormField::Name, name)
                .value(ProductFilterFormField::Reference, reference),
        ),
        actions: html! {
            (container_row("flex gap-2", html! {
                (button_submit(ButtonSubmit { label: "Apply Filters", ..Default::default() }))
                (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
            }))
        },
        ..Default::default()
    })
}

fn render_pagination<K: SwapKey>(path_and_query: &str, number: u32, num_pages: u32) -> Markup {
    let owned = pagination_pages(path_and_query, number, num_pages, true);
    let pages: Vec<PaginationPage<'_>> = owned
        .iter()
        .map(|(ellipsis, url, push_url, active, label)| PaginationPage {
            ellipsis: *ellipsis,
            url: url.as_str(),
            push_url: *push_url,
            active: *active,
            label: label.as_str(),
        })
        .collect();
    table_pagination(TablePagination {
        pages: &pages,
        hx_target: K::SELECTOR,
    })
}

#[derive(Clone)]
pub struct ProductRow {
    pub id: i64,
    pub product_type: String,
    pub reference: String,
    pub name: String,
    pub base_cost: String,
    pub sales_price: String,
    pub hsn_code: String,
}

#[derive(Generic)]
pub struct ProductListPage {
    pub products: ObjectList<ProductRow>,
    pub filter_name: String,
    pub filter_reference: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl ProductListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Type", sort_url: None, push_url: true },
            TableColumnHeader { label: "Reference", sort_url: None, push_url: true },
            TableColumnHeader { label: "Name", sort_url: None, push_url: true },
            TableColumnHeader { label: "Base cost", sort_url: None, push_url: true },
            TableColumnHeader { label: "Sales price", sort_url: None, push_url: true },
            TableColumnHeader { label: "HSN", sort_url: None, push_url: true },
        ];
        let rows: Vec<TableRow> = self
            .products
            .items
            .iter()
            .map(|p| TableRow {
                attrs: row_attr_navigate_route(ProductDetailRouteTag::new(p.id)),
                cells: vec![
                    field_text(FieldText { value: &p.product_type, classes: "" }),
                    field_text(FieldText { value: &p.reference, classes: "" }),
                    field_text(FieldText { value: &p.name, classes: "" }),
                    field_text(FieldText {
                        value: &p.base_cost,
                        classes: "text-end tabular-nums",
                    }),
                    field_text(FieldText {
                        value: &p.sales_price,
                        classes: "text-end tabular-nums",
                    }),
                    field_text(FieldText { value: &p.hsn_code, classes: "" }),
                ],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: product_filter_form(&self.filter_name, &self.filter_reference),
                ..Default::default()
            }))
        };
        if self.can_edit {
            actions = html! {
                (actions)
                (button_link(ButtonLink {
                    href: &ProductCreateGetRouteTag.url(),
                    icon_name: Some("plus"),
                    classes: "btn-square btn-outline btn-sm",
                    ..Default::default()
                }))
            };
        }
        let pagination = render_pagination::<ProductTableKey>(
            &self.path_and_query,
            self.products.number,
            self.products.num_pages,
        );
        data_table_list::<ProductTableKey>(
            "Products",
            actions,
            &headers,
            &rows,
            pagination,
        )
    }

    fn body(&self) -> Markup {
        self.render_table()
    }
}

impl RenderAppPane for ProductListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_sidebar(self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for ProductListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold("Products — Uniquity", chrome, self.body())
    }
}

#[derive(Generic)]
pub struct ProductDetailPage {
    pub id: i64,
    pub name: String,
    pub product_type: String,
    pub reference: String,
    pub remarks: String,
    pub base_cost: String,
    pub sales_price: String,
    pub hsn_code: String,
    pub taxes: String,
    pub can_edit: bool,
}

impl ProductDetailPage {
    fn body(&self) -> Markup {
        html! {
            (detail(html! {
                (container_column("", html! {
                    (field_title(FieldTitle { value: &self.name, classes: "" }))
                    (label_inline("Type", field_text(FieldText { value: &self.product_type, classes: "" })))
                    (label_inline("Reference", field_text(FieldText { value: &self.reference, classes: "" })))
                    (label_inline("Remarks", field_text(FieldText { value: &self.remarks, classes: "" })))
                    (label_inline("Taxes", field_text(FieldText { value: &self.taxes, classes: "" })))
                    (label_inline("Base cost", field_text(FieldText { value: &self.base_cost, classes: "" })))
                    (label_inline("Sales price", field_text(FieldText { value: &self.sales_price, classes: "" })))
                    (label_inline("HSN code", field_text(FieldText { value: &self.hsn_code, classes: "" })))
                }))
            }))
        }
    }

    fn menu(&self) -> Markup {
        product_detail_menu(self.id, &self.name, "detail", self.can_edit)
    }
}

impl RenderAppPane for ProductDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.menu(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for ProductDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Product — Uniquity", chrome, self.menu(), self.body())
    }
}

#[derive(Generic)]
pub struct ProductFormPage {
    pub id: i64,
    pub name: String,
    pub product_type: String,
    pub reference: String,
    pub remarks: String,
    pub base_cost: String,
    pub sales_price: String,
    pub hsn_code: i64,
    pub tax_items: Vec<ManyToManyItem>,
    pub is_edit: bool,
    pub error: String,
}

impl ProductFormPage {
    fn body(&self) -> Markup {
        let title = if self.is_edit {
            "Edit Product"
        } else {
            "Create Product"
        };
        let choices = ProductForm::product_type_choices();
        html! {
            (container_column("@container", html! {
                (field_title(FieldTitle { value: title, classes: "" }))
                (form(FormOpts {
                    attrs: if self.is_edit {
                        form_hx_post_main(ProductEditPostRouteTag::new(self.id))
                    } else {
                        form_hx_post_main(ProductCreatePostRouteTag)
                    },
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: ProductForm::render_inputs(
                        &FormCtx::form::<ProductForm>()
                            .value(ProductFormField::Name, &self.name)
                            .value(ProductFormField::ProductType, &self.product_type)
                            .value(ProductFormField::Reference, &self.reference)
                            .value(ProductFormField::Remarks, &self.remarks)
                            .value(ProductFormField::BaseCost, &self.base_cost)
                            .value(ProductFormField::SalesPrice, &self.sales_price)
                            .value(ProductFormField::HsnCode, self.hsn_code.to_string())
                            .m2m(ProductFormField::TaxIds, &self.tax_items)
                            .choices(
                                ProductFormField::ProductType,
                                &choices
                                    .iter()
                                    .map(|(k, v)| (k.to_string(), v.to_string()))
                                    .collect::<Vec<_>>(),
                            ),
                    ),
                    actions: html! {
                        (container_row("flex gap-2 mt-2", html! {
                            (button_submit(ButtonSubmit {
                                label: "Save Product",
                                classes: "btn-primary",
                                ..Default::default()
                            }))
                            @if self.is_edit {
                                (button_delete(
                                    ProductDeletePostRouteTag::new(self.id),
                                    "Delete Product",
                                    "Permanently delete this product?",
                                ))
                            }
                        }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }

    fn sidebar(&self) -> Markup {
        if self.is_edit {
            product_detail_menu(self.id, &self.name, "edit", true)
        } else {
            uniquity_finance_accounts::accounting_sidebar::accounting_sidebar()
        }
    }
}

impl RenderAppPane for ProductFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        layout_with_entity_sidebar(self.sidebar(), self.body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        layout_main_content(self.body())
    }
}

impl RenderTemplate for ProductFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold_with_sidebar("Product Form — Uniquity", chrome, self.sidebar(), self.body())
    }
}

#[derive(Generic)]
pub struct ProductSelectPage {
    pub products: ObjectList<ProductRow>,
    pub target_input: String,
    pub path_and_query: String,
}

impl RenderPickerSelect<ProductSelectTableKey, ProductSelectModalKey> for ProductSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { label: "Reference", sort_url: None, push_url: false },
            TableColumnHeader { label: "Name", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .products
            .items
            .iter()
            .map(|p| TableRow {
                attrs: row_attr_select(&self.target_input, &p.id.to_string(), &p.name),
                cells: vec![
                    field_text(FieldText { value: &p.reference, classes: "" }),
                    field_text(FieldText { value: &p.name, classes: "" }),
                ],
            })
            .collect();
        let pagination = render_pagination::<ProductSelectTableKey>(
            &self.path_and_query,
            self.products.number,
            self.products.num_pages,
        );
        data_table_list::<ProductSelectTableKey>(
            "Select Product",
            html! {},
            &headers,
            &rows,
            pagination,
        )
    }
}

impl RenderTemplate for ProductSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}
