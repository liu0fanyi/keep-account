//! Mobile bottom navigation component.

use leptos::prelude::*;
#[derive(Clone, Copy, PartialEq)]
pub enum MobileView {
    List, Form, Categories, CategoryForm, Installments, InstallmentForm, Summary,
}

/// 底部导航栏
#[component]
pub fn MobileBottomNav(
    current_view: RwSignal<MobileView>,
) -> impl IntoView {
    view! {
        <div class="mobile-bottom-nav">
            <button
                class=move || if current_view.get() == MobileView::List { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::List)
            >
                <div class="mobile-nav-icon">"📝"</div>
                <div class="mobile-nav-label">"记账"</div>
            </button>
            
            <button
                class=move || if current_view.get() == MobileView::Categories { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::Categories)
            >
                <div class="mobile-nav-icon">"📂"</div>
                <div class="mobile-nav-label">"项目"</div>
            </button>
            
            <button
                class=move || if current_view.get() == MobileView::Installments { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::Installments)
            >
                <div class="mobile-nav-icon">"💳"</div>
                <div class="mobile-nav-label">"分期"</div>
            </button>
            
            <button
                class=move || if current_view.get() == MobileView::Summary { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::Summary)
            >
                <div class="mobile-nav-icon">"📊"</div>
                <div class="mobile-nav-label">"汇总"</div>
            </button>
        </div>
    }
}
