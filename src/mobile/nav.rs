//! Mobile bottom navigation component.

use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api::{invoke_safe, JsValue};

#[derive(Clone, Copy, PartialEq)]
pub enum MobileView {
    List, Form, Categories, CategoryForm, Installments, InstallmentForm, Summary, Settings,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SyncState {
    Idle,      // ☁️
    Syncing,   // 🔄 spinning
    Success,   // ✓
    Error,     // ❌
}

/// 底部导航栏
#[component]
pub fn MobileBottomNav(
    current_view: RwSignal<MobileView>,
) -> impl IntoView {
    let sync_state = RwSignal::new(SyncState::Idle);
    
    let do_sync = move |_| {
        if sync_state.get() == SyncState::Syncing {
            return; // Prevent double-click
        }
        
        sync_state.set(SyncState::Syncing);
        
        spawn_local(async move {
            match invoke_safe("sync_database", JsValue::NULL).await {
                Ok(_) => {
                    sync_state.set(SyncState::Success);
                    // Reset to idle after 2 seconds
                    set_timeout(move || sync_state.set(SyncState::Idle), std::time::Duration::from_secs(2));
                }
                Err(_) => {
                    sync_state.set(SyncState::Error);
                    // Reset to idle after 2 seconds
                    set_timeout(move || sync_state.set(SyncState::Idle), std::time::Duration::from_secs(2));
                }
            }
        });
    };
    
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
                class="mobile-nav-item"
                on:click=do_sync
                disabled=move || sync_state.get() == SyncState::Syncing
            >
                <div class=move || format!("mobile-nav-icon{}", if sync_state.get() == SyncState::Syncing { " spinning" } else { "" })>
                    {move || match sync_state.get() {
                        SyncState::Idle => "☁️",
                        SyncState::Syncing => "🔄",
                        SyncState::Success => "✅",
                        SyncState::Error => "❌",
                    }}
                </div>
                <div class="mobile-nav-label">"同步"</div>
            </button>
            
            <button
                class=move || if current_view.get() == MobileView::Summary { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::Summary)
            >
                <div class="mobile-nav-icon">"📊"</div>
                <div class="mobile-nav-label">"汇总"</div>
            </button>
            
            <button
                class=move || if current_view.get() == MobileView::Settings { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::Settings)
            >
                <div class="mobile-nav-icon">"⚙️"</div>
                <div class="mobile-nav-label">"设置"</div>
            </button>
        </div>
    }
}
