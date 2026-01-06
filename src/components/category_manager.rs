//! Category Manager component for desktop view.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::types::Category;
use crate::api::{invoke, JsValue};

#[component]
pub fn CategoryManager(
    categories: ReadSignal<Vec<Category>>,
    set_categories: WriteSignal<Vec<Category>>,
) -> impl IntoView {
    let (show_add_form, set_show_add_form) = create_signal(false);
    let (new_category_name, set_new_category_name) = create_signal(String::new());
    let (new_category_icon, set_new_category_icon) = create_signal(String::new());

    let available_icons = vec![
        "🍔", "🍕", "🍜", "🍰", "☕", "🍺", "🥤",
        "🛒", "🏠", "🚗", "✈️", "🎮", "📱", "💻",
        "👕", "👟", "💄", "💊", "🏥", "📚", "✏️",
        "🎬", "🎵", "🎨", "💡", "🔧", "📦", "💰",
    ];

    let add_category = move |_| {
        let name = new_category_name.get();
        let icon = new_category_icon.get();
        let set_categories = set_categories.clone();
        let set_show_add_form = set_show_add_form.clone();
        let set_new_category_name = set_new_category_name.clone();
        let set_new_category_icon = set_new_category_icon.clone();

        if name.is_empty() { return; }

        let icon_val = if icon.is_empty() { Some("📦".to_string()) } else { Some(icon) };

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "name": name, "icon": icon_val,
            })).unwrap();

            let result = invoke("create_category", args).await;
            if let Ok(_) = serde_wasm_bindgen::from_value::<Category>(result) {
                let reload_result = invoke("get_categories", JsValue::NULL).await;
                if let Ok(cats) = serde_wasm_bindgen::from_value::<Vec<Category>>(reload_result) {
                    set_categories.set(cats);
                }
                set_new_category_name.set(String::new());
                set_new_category_icon.set(String::new());
                set_show_add_form.set(false);
            }
        });
    };

    let delete_category = move |cat_id: i64| {
        let set_categories = set_categories.clone();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "id": cat_id })).unwrap();
            let _ = invoke("delete_category", args).await;
            let reload_result = invoke("get_categories", JsValue::NULL).await;
            if let Ok(cats) = serde_wasm_bindgen::from_value::<Vec<Category>>(reload_result) {
                set_categories.set(cats);
            }
        });
    };

    view! {
        <div class="category-manager">
            <div class="section-header">
                <h2>"消费项目"</h2>
                <button class="btn-primary" on:click=move |_| set_show_add_form.set(!show_add_form.get())>
                    {move || if show_add_form.get() { "取消" } else { "+ 新建项目" }}
                </button>
            </div>

            {move || if show_add_form.get() {
                Some(view! {
                    <div class="add-form">
                        <input type="text" placeholder="项目名称" value=new_category_name
                            on:input=move |ev| set_new_category_name.set(event_target_value(&ev)) />
                        <div class="icon-selector">
                            <span>"选择图标:"</span>
                            {available_icons.iter().map(|icon| {
                                let icon = icon.to_string();
                                let is_selected = new_category_icon.get() == icon;
                                let display_icon = icon.clone();
                                view! {
                                    <button class=move || format!("icon-btn {}", if is_selected { "selected" } else { "" })
                                        on:click=move |_| set_new_category_icon.set(icon.clone())>
                                        {display_icon}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <button class="btn-primary" on:click=add_category>"保存"</button>
                    </div>
                })
            } else { None }}

            <div class="category-list">
                <For each=move || categories.get() key=|category| category.id let:category>
                    <div class="category-item">
                        <span class="category-icon">{category.icon.clone().unwrap_or_else(|| "📦".to_string())}</span>
                        <span class="category-name">{category.name}</span>
                        <button class="btn-danger" on:click=move |_| delete_category(category.id)>"删除"</button>
                    </div>
                </For>
            </div>
        </div>
    }
}
