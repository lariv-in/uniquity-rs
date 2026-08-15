use lariv_rs::components::{attrs::escape_attr, text::icon};
use lariv_rs::html_form::{FieldRender, FormCtx, FormWidget};
use maud::{Markup, html};

const PO_LINES_ALPINE_METHODS: &str = r#"addLine() {
	this.lines.push({
		item_code: '',
		description: '',
		unit: '',
		delivery_date: '',
		quantity: '1',
		rate: '',
	});
},
removeLine(idx) {
	if (!Array.isArray(this.lines) || this.lines.length <= 1) return;
	this.lines.splice(idx, 1);
},
deliveryDateIso(line) {
	const s = String(line.delivery_date ?? '').trim();
	const dmy = s.match(/^(\d{2})\/(\d{2})\/(\d{4})$/);
	if (dmy) return dmy[3] + '-' + dmy[2] + '-' + dmy[1];
	const iso = s.match(/^(\d{4})-(\d{2})-(\d{2})$/);
	return iso ? s : '';
},
setDeliveryDateFromIso(line, iso) {
	if (!iso) { line.delivery_date = ''; return; }
	line.delivery_date = String(iso).split('-').reverse().join('/');
},
openDeliveryDatePicker(line, ev) {
	const wrap = ev.currentTarget.closest('[data-lariv-date-wrap]');
	if (!wrap) return;
	const picker = wrap.querySelector('[data-lariv-picker]');
	if (!picker) return;
	picker.value = this.deliveryDateIso(line);
	try { picker.showPicker(); } catch (err) { picker.click(); }
}"#;

pub struct PurchaseOrderLinesDraft;

impl FormWidget for PurchaseOrderLinesDraft {
    fn render(_ctx: &FormCtx<'_>, field: &FieldRender<'_>) -> Markup {
        input_purchase_order_lines(field.name, field.value)
    }
}

pub fn input_purchase_order_lines(name: &str, defaults: &str) -> Markup {
    let defaults = if defaults.trim().is_empty() {
        crate::po_lines::default_po_lines_json()
    } else {
        defaults.trim().to_string()
    };

    let alpine_data = format!(
        "{{ lines: {defaults}, {methods} }}",
        methods = PO_LINES_ALPINE_METHODS.trim_end_matches(',')
    );
    let name_escaped = escape_attr(name);
    let init_js = format!(
        r#"
(function () {{
	const d = Alpine.$data($el);
	if (!d || !Array.isArray(d.lines) || d.lines.length === 0) {{
		d.lines = {defaults};
	}}
}})();
$el.closest('form').addEventListener('submit', (ev) => {{
	const d = Alpine.$data($el);
	if (!d || !Array.isArray(d.lines)) return;
	const h = $el.querySelector('input[type="hidden"][name={name_q}]');
	if (!h) return;
	const strip = (l) => ({{
		item_code: l.item_code || '',
		description: l.description || '',
		unit: l.unit || '',
		delivery_date: l.delivery_date || '',
		quantity: l.quantity || '',
		rate: l.rate || '',
	}});
	h.value = JSON.stringify(d.lines.map(strip));
}}, true);"#,
        name_q = serde_json::to_string(name).unwrap_or_else(|_| "\"PoLinesJson\"".into())
    );

    html! {
        div class="w-full" x-data=(alpine_data) x-init=(init_js) {
            input type="hidden" name=(name_escaped) value="" {}
            div class="overflow-x-auto" {
                table class="table table-sm w-full" {
                    thead {
                        tr {
                            th { "Item code" }
                            th { "Description" }
                            th { "Unit" }
                            th { "Delivery date" }
                            th { "Quantity" }
                            th { "Rate" }
                            th class="w-12" {}
                        }
                    }
                    tbody {
                        template x-for="(line, idx) in lines" x-bind:key="idx" {
                            tr {
                                td {
                                    input class="input input-bordered input-sm w-full min-w-24"
                                        type="text"
                                        x-model="line.item_code" {}
                                }
                                td {
                                    input class="input input-bordered input-sm w-full min-w-40"
                                        type="text"
                                        x-model="line.description" {}
                                }
                                td {
                                    input class="input input-bordered input-sm w-full min-w-20"
                                        type="text"
                                        x-model="line.unit" {}
                                }
                                td {
                                    div class="join relative w-full min-w-36" data-lariv-date-wrap="" {
                                        input class="input input-bordered input-sm join-item min-w-0 flex-1"
                                            type="text"
                                            placeholder="DD/MM/YYYY"
                                            autocomplete="off"
                                            x-model="line.delivery_date" {}
                                        button type="button" class="btn btn-square btn-sm join-item"
                                            x-on:click="openDeliveryDatePicker(line, $event)"
                                            aria-label="Open date picker" {
                                            (icon("calendar", "heroicon-sm"))
                                        }
                                        input class="pointer-events-none absolute right-0 top-0 bottom-0 w-10 opacity-0"
                                            type="date"
                                            tabindex="-1"
                                            aria-hidden="true"
                                            data-lariv-picker=""
                                            x-bind:value="deliveryDateIso(line)"
                                            x-on:change="setDeliveryDateFromIso(line, $event.target.value)" {}
                                    }
                                }
                                td {
                                    input class="input input-bordered input-sm w-full min-w-20"
                                        type="text"
                                        x-model="line.quantity" {}
                                }
                                td {
                                    input class="input input-bordered input-sm w-full min-w-20"
                                        type="text"
                                        x-model="line.rate" {}
                                }
                                td {
                                    button type="button" class="btn btn-ghost btn-sm"
                                        x-on:click="removeLine(idx)"
                                        x-show="lines.length > 1"
                                        title="Remove line" {
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            button type="button" class="btn btn-sm btn-outline mt-2" x-on:click="addLine()" {
                "+ Add line"
            }
        }
    }
}
