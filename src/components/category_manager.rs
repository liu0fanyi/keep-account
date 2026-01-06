//! Category Manager component for desktop view.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::types::Category;
use crate::shared::{COMMON_ICONS, DEFAULT_ICON, fetch_categories, create_category, delete_category};

#[component]
pub fn CategoryManager(
    categories: ReadSignal<Vec<Category>>,
    set_categories: WriteSignal<Vec<Category>>,
) -> impl IntoView {
    let (show_add_form, set_show_add_form) = create_signal(false);
    let (new_category_name, set_new_category_name) = create_signal(String::new());
    let (new_category_icon, set_new_category_icon) = create_signal(String::new());

    let add_category = move |_| {
        let name = new_category_name.get();
        let icon = new_category_icon.get();
        let set_categories = set_categories.clone();
        let set_show_add_form = set_show_add_form.clone();
        let set_new_category_name = set_new_category_name.clone();
        let set_new_category_icon = set_new_category_icon.clone();

        if name.is_empty() { return; }

        let icon_val = if icon.is_empty() { DEFAULT_ICON.to_string() } else { icon };

        spawn_local(async move {
            if let Ok(_) = create_category(&name, &icon_val).await {
                if let Ok(cats) = fetch_categories().await {
                    set_categories.set(cats);
                }
                set_new_category_name.set(String::new());
                set_new_category_icon.set(String::new());
                set_show_add_form.set(false);
            }
        });
    };

    let on_delete = move |cat_id: i64| {
        let set_categories = set_categories.clone();
        spawn_local(async move {
            let _ = delete_category(cat_id).await;
            if let Ok(cats) = fetch_categories().await {
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
                            {COMMON_ICONS.iter().map(|&icon| {
                                let icon_str = icon.to_string();
                                let is_selected = new_category_icon.get() == icon_str;
                                view! {
                                    <button class=move || format!("icon-btn {}", if is_selected { "selected" } else { "" })
                                        on:click=move |_| set_new_category_icon.set(icon.to_string())>
                                        {icon}
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
                        <span class="category-icon">{category.icon.clone().unwrap_or_else(|| DEFAULT_ICON.to_string())}</span>
                        <span class="category-name">{category.name}</span>
                        <button class="btn-danger" on:click=move |_| on_delete(category.id)>"删除"</button>
                    </div>
                </For>
            </div>
        </div>
    }
}
