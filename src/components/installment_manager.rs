//! Installment Manager component.

use leptos::prelude::*;
use leptos::task::spawn_local;
use chrono::Datelike;

use crate::types::{Category, InstallmentWithCategory, InstallmentDetail};
use crate::api::{invoke, JsValue};

#[component]
pub fn InstallmentManager(
    categories: ReadSignal<Vec<Category>>,
) -> impl IntoView {
    let (installments, set_installments) = create_signal(Vec::<InstallmentWithCategory>::new());
    let (due_installments, set_due_installments) = create_signal(Vec::<InstallmentDetail>::new());
    let (selected_installment, set_selected_installment) = create_signal(None::<i64>);
    let (installment_details, set_installment_details) = create_signal(Vec::<InstallmentDetail>::new());

    // Form state
    let (show_add_form, set_show_add_form) = create_signal(false);
    let (selected_category_id, set_selected_category_id) = create_signal(0i64);
    let (total_amount, set_total_amount) = create_signal(String::new());
    let (installment_count, set_installment_count) = create_signal(3i32);
    let (start_date, set_start_date) = create_signal({
        let now = chrono::Local::now();
        now.format("%Y-%m-%d").to_string()
    });
    let (note, set_note) = create_signal(String::new());

    // Load all installments on mount
    let load_installments = {
        let set_installments = set_installments.clone();
        let set_due_installments = set_due_installments.clone();
        move || {
            let set_installments = set_installments.clone();
            let set_due_installments = set_due_installments.clone();
            spawn_local(async move {
                // Load all installments
                let result = invoke("get_installments", JsValue::NULL).await;
                if let Ok(insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentWithCategory>>(result) {
                    set_installments.set(insts);
                }

                // Load due installments for current month
                let now = chrono::Local::now();
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "year": now.year(),
                    "month": now.month(),
                })).unwrap();

                let due_result = invoke("get_due_installments_by_month", args).await;
                if let Ok(due_insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(due_result) {
                    set_due_installments.set(due_insts);
                }
            });
        }
    };

    // Load installments (delayed to avoid startup issues)
    load_installments();

    // Load installment details
    let load_installment_details = move |installment_id: i64| {
        let set_installment_details = set_installment_details.clone();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "installmentId": installment_id })).unwrap();
            let result = invoke("get_installment_details", args).await;
            if let Ok(details) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(result) {
                set_installment_details.set(details);
            }
        });
    };

    // Add installment
    let add_installment = move |_| {
        let cat_id = selected_category_id.get();
        let amt_str = total_amount.get();
        let count = installment_count.get();
        let dt = start_date.get();
        let nt = note.get();
        let set_total_amount = set_total_amount.clone();
        let set_note = set_note.clone();
        let set_show_add_form = set_show_add_form.clone();

        if cat_id == 0 || amt_str.is_empty() {
            return;
        }

        let amt: f64 = match amt_str.parse() {
            Ok(a) => a,
            Err(_) => {
                eprintln!("Invalid amount");
                return;
            }
        };

        let note_val = if nt.is_empty() { None } else { Some(nt) };

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "categoryId": cat_id,
                "totalAmount": amt,
                "installmentCount": count,
                "startDate": dt,
                "note": note_val,
            })).unwrap();

            let _result = invoke("create_installment", args).await;

            // Reload installments
            let result = invoke("get_installments", JsValue::NULL).await;
            if let Ok(insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentWithCategory>>(result) {
                set_installments.set(insts);
            }

            // Reset form
            set_total_amount.set(String::new());
            set_note.set(String::new());
            set_show_add_form.set(false);
        });
    };

    // Mark installment as paid
    let mark_paid = move |detail_id: i64| {
        let set_installment_details = set_installment_details.clone();
        spawn_local(async move {
            let now = chrono::Local::now();
            let paid_date = now.format("%Y-%m-%d").to_string();

            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "detailId": detail_id,
                "paidDate": paid_date,
            })).unwrap();

            let _result = invoke("mark_installment_paid", args).await;

            // Reload details
            let installment_id = selected_installment.get().unwrap_or(0);
            if installment_id > 0 {
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "installmentId": installment_id })).unwrap();
                let result = invoke("get_installment_details", args).await;
                if let Ok(details) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(result) {
                    set_installment_details.set(details);
                }
            }

            // Reload due installments
            let now = chrono::Local::now();
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "year": now.year(),
                "month": now.month(),
            })).unwrap();

            let due_result = invoke("get_due_installments_by_month", args).await;
            if let Ok(due_insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(due_result) {
                set_due_installments.set(due_insts);
            }
        });
    };

    // Delete installment
    let delete_installment = move |inst_id: i64| {
        let set_installments = set_installments.clone();
        let set_installment_details = set_installment_details.clone();
        let set_selected_installment = set_selected_installment.clone();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "id": inst_id })).unwrap();
            let _result = invoke("delete_installment", args).await;

            let result = invoke("get_installments", JsValue::NULL).await;
            if let Ok(insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentWithCategory>>(result) {
                set_installments.set(insts);
            }

            set_installment_details.set(Vec::new());
            set_selected_installment.set(None);
        });
    };

    // Show installment details
    let show_details = move |inst_id: i64| {
        set_selected_installment.set(Some(inst_id));
        load_installment_details(inst_id);
    };

    view! {
        <div class="installment-manager">
            <div class="section-header">
                <h2>"分期管理"</h2>
                <button class="btn-primary" on:click=move |_| {
                    let is_showing = show_add_form.get();
                    if !is_showing {
                        // Reset to today's date when opening the form
                        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                        set_start_date.set(today);
                    }
                    set_show_add_form.set(!is_showing);
                }>
                    {move || if show_add_form.get() { "取消" } else { "+ 新建分期" }}
                </button>
            </div>

            {move || if show_add_form.get() {
                Some(view! {
                    <div class="add-form">
                        <div class="icon-selector">
                            {categories.get().into_iter().map(|cat| {
                                let cat_id = cat.id;
                                let is_selected = selected_category_id.get() == cat_id;
                                view! {
                                    <button
                                        class=move || {
                                            if is_selected { "icon-btn selected" } else { "icon-btn" }
                                        }
                                        on:click=move |_| set_selected_category_id.set(cat_id)
                                        title=cat.name.clone()
                                    >
                                        {cat.icon.clone().unwrap_or("".to_string())}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <input
                            type="number"
                            placeholder="总金额"
                            value=total_amount
                            on:input=move |ev| set_total_amount.set(event_target_value(&ev))
                        />
                        <div class="input-group">
                            <label>"分期期数:"</label>
                            <select
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_installment_count.set(val.parse().unwrap_or(3));
                                }
                            >
                                <option value="3">"3期"</option>
                                <option value="6">"6期"</option>
                                <option value="12">"12期"</option>
                                <option value="24">"24期"</option>
                            </select>
                        </div>
                        <input
                            type="date"
                            value=start_date
                            on:input=move |ev| set_start_date.set(event_target_value(&ev))
                        />
                        <input
                            type="text"
                            placeholder="备注（可选）"
                            value=note
                            on:input=move |ev| set_note.set(event_target_value(&ev))
                        />
                        <button
                            class="btn-primary"
                            on:click=add_installment
                        >
                            "保存"
                        </button>
                    </div>
                })
            } else {
                None
            }}

            // Installment list
            <div class="installment-list">
                <For
                    each=move || installments.get()
                    key=|inst| inst.id
                    let:installment
                >
                    {
                        let inst_id = installment.id;
                        let monthly_amount = installment.total_amount / installment.installment_count as f64;
                        view! {
                            <div class="installment-item">
                                <span class="installment-icon">
                                    {installment.category_icon.clone().unwrap_or_else(|| "📦".to_string())}
                                </span>
                                <div class="installment-info">
                                    <span class="installment-category">{installment.category_name}</span>
                                    <span class="installment-dates">
                                        {format!("{}起 · {}期", installment.start_date, installment.installment_count)}
                                    </span>
                                </div>
                                <div class="installment-amount">
                                    <span class="total-amount">{format!("{:.2}", installment.total_amount)}</span>
                                    <span class="monthly-amount">{format!("每期 {:.2}", monthly_amount)}</span>
                                </div>
                                <button
                                    class="btn-delete"
                                    on:click=move |_| delete_installment(inst_id)
                                >
                                    "删除"
                                </button>
                            </div>
                        }
                    }
                </For>
            </div>
        </div>
    }
}

