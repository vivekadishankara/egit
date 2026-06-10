use leptos::prelude::*;

use crate::pages::auth::{set_theme, SetTheme, THEMES};

/// Dropdown that lets the logged-in user change their theme.
/// Immediately updates `document.documentElement.dataset.theme` on the client
/// so the change is visible before the next page load.
#[component]
pub fn ThemeSwitcher(
    /// The user's currently-active theme (e.g. "dark").
    current_theme: String,
) -> impl IntoView {
    let set_theme_action = ServerAction::<SetTheme>::new();

    // Track the optimistically-selected theme in local state.
    let (selected, set_selected) = signal(current_theme);

    // Whenever the action settles successfully, do nothing extra —
    // the signal is already updated. On error, we could revert, but
    // for simplicity we leave the optimistic value in place.

    view! {
        <div class="theme-switcher-wrapper">
            <ActionForm action=set_theme_action>
                <label for="theme-select" class="sr-only">"Theme"</label>
                <select
                    id="theme-select"
                    name="theme"
                    class="theme-select"
                    // Optimistic update: flip the DOM attribute immediately on change,
                    // then submit the form so the server persists the preference.
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        set_selected.set(val.clone());
                        // 1. Update <html data-theme="..."> instantly (no flicker).
                        #[cfg(feature = "hydrate")]
                        {
                            use wasm_bindgen::JsCast;
                            let target = ev.target();
                            // Update the theme token on <html>.
                            if let Some(html) = web_sys::window()
                                .and_then(|w| w.document())
                                .and_then(|d| d.document_element())
                                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                            {
                                let _ = html.dataset().set("theme", &val);
                            }
                            // 2. Auto-submit the parent <form> so the server persists it.
                            if let Some(select_el) = target
                                .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                            {
                                if let Some(form) = select_el.form() {
                                    let _ = form.request_submit();
                                }
                            }
                        }
                    }
                >
                    {THEMES.iter().map(|(id, label)| {
                        let id = *id;
                        let label = *label;
                        view! {
                            <option
                                value=id
                                selected=move || selected.get() == id
                            >
                                {label}
                            </option>
                        }
                    }).collect_view()}
                </select>
                // Auto-submit when the select changes via a hidden button triggered by JS,
                // OR let the browser submit naturally via the form.
                // We trigger submission programmatically in the on:change handler above,
                // but the ActionForm also needs a submit button to work without JS.
                <button type="submit" class="theme-submit-btn" aria-label="Apply theme">
                    // Paint-brush icon
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                         stroke="currentColor" stroke-width="2" stroke-linecap="round"
                         stroke-linejoin="round">
                        <path d="M18.37 2.63 14 7l-1.59-1.59a2 2 0 0 0-2.82 0L8 7l9 9 1.59-1.59a2 2 0 0 0 0-2.82L17 10l4.37-4.37a2.12 2.12 0 1 0-3-3Z"/>
                        <path d="M9 8c-2 3-4 3.5-7 4l8 8c1-.5 3.5-2 4-7"/>
                        <path d="M14.5 17.5 4.5 15"/>
                    </svg>
                </button>
            </ActionForm>
        </div>
    }
}
