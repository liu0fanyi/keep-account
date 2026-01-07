use leptos::prelude::*;
use leptos::task::spawn_local;
use chrono::Datelike;

// Import shared types and API
use crate::types::Category;
use crate::api::{invoke, JsValue};
use crate::components::{CategoryManager, DesktopTransactionView, InstallmentManager};


// ============================================================================
// Main App Component
// ============================================================================

#[component]
pub fn App() -> impl IntoView {
    // Categories state
    let (categories, set_categories) = create_signal(Vec::<Category>::new());

    // Current view: "categories", "transactions", "installments"
    let (current_view, set_current_view) = create_signal("transactions".to_string());

    // Selected month for transaction view - use current date
    let now = chrono::Local::now();
    let current_year = now.year();
    let current_month = now.month();

    let (selected_year, set_selected_year) = create_signal(current_year);
    let (selected_month, set_selected_month) = create_signal(current_month as i32);

    // Load categories on mount
    let load_categories = {
        let set_categories = set_categories.clone();
        move || {
            let set_categories = set_categories.clone();
            spawn_local(async move {
                let result = invoke("get_categories", JsValue::NULL).await;
                if let Ok(cats) = serde_wasm_bindgen::from_value::<Vec<Category>>(result) {
                    set_categories.set(cats);
                }
            });
        }
    };

    // Load categories on mount
    create_effect(move |_| {
        load_categories();
    });

    // Navigation
    let show_categories = move |_| set_current_view.set("categories".to_string());
    let show_transactions = move |_| set_current_view.set("transactions".to_string());
    let show_installments = move |_| set_current_view.set("installments".to_string());
    let show_summary = move |_| set_current_view.set("summary".to_string());

    view! {
        <main class="container">
            <div class="header">
                <h1>"简易记账"</h1>
                <nav class="nav">
                    <button
                        class=move || format!("nav-btn {}", if current_view.get() == "transactions" { "active" } else { "" })
                        on:click=show_transactions
                    >
                        "记账"
                    </button>
                    <button
                        class=move || format!("nav-btn {}", if current_view.get() == "categories" { "active" } else { "" })
                        on:click=show_categories
                    >
                        "消费项目"
                    </button>
                    <button
                        class=move || format!("nav-btn {}", if current_view.get() == "installments" { "active" } else { "" })
                        on:click=show_installments
                    >
                        "分期管理"
                    </button>
                    <button
                        class=move || format!("nav-btn {}", if current_view.get() == "summary" { "active" } else { "" })
                        on:click=show_summary
                    >
                        "汇总"
                    </button>
                </nav>
            </div>

            <div class="content">
                <Show when=move || current_view.get() == "categories">
                    <CategoryManager categories=categories set_categories=set_categories />
                </Show>

                <Show when=move || current_view.get() == "transactions">
                    <TransactionView
                        categories=categories
                        set_categories=set_categories
                        selected_year=selected_year
                        selected_month=selected_month
                        set_selected_year=set_selected_year
                        set_selected_month=set_selected_month
                    />
                </Show>


                <Show when=move || current_view.get() == "installments">
                    <InstallmentManager categories=categories />
                </Show>

                <Show when=move || current_view.get() == "summary">
                    <crate::summary::SummaryView categories=categories />
                </Show>
            </div>
        </main>
    }
}

// ============================================================================
// Transaction View Component
// ============================================================================

#[component]
fn TransactionView(
    categories: ReadSignal<Vec<Category>>,
    set_categories: WriteSignal<Vec<Category>>,
    selected_year: ReadSignal<i32>,
    selected_month: ReadSignal<i32>,
    set_selected_year: WriteSignal<i32>,
    set_selected_month: WriteSignal<i32>,
) -> impl IntoView {
    view! {
        <div class="transaction-view-wrapper">
            // 移动端视图 - 通过CSS媒体查询控制显示
            <div class="mobile-only">
                <crate::mobile::MobileTransactionView
                    categories=categories
                    set_categories=set_categories
                    selected_year=selected_year
                    selected_month=selected_month
                    set_selected_year=set_selected_year
                    set_selected_month=set_selected_month
                />
            </div>

            
            // 桌面端视图 - 通过CSS媒体查询控制显示
            <div class="desktop-only">
                <DesktopTransactionView
                    categories=categories
                    selected_year=selected_year
                    selected_month=selected_month
                    set_selected_year=set_selected_year
                    set_selected_month=set_selected_month
                />
            </div>
        </div>
    }
}
