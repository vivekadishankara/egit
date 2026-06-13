use leptos::prelude::*;

fn get_http_url(owner: &str, name: &str) -> String {
    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        if let Ok(origin) = window.location().origin() {
            return format!("{}/{}/{}", origin, owner, name);
        }
    }
    format!("http://localhost:3000/{}/{}", owner, name)
}

fn get_ssh_url(owner: &str, name: &str) -> String {
    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        if let Ok(hostname) = window.location().hostname() {
            return format!("git@{}:{}/{}.git", hostname, owner, name);
        }
    }
    format!("git@localhost:{}/{}.git", owner, name)
}

fn copy_to_clipboard(url: &str) {
    #[cfg(feature = "hydrate")]
    if let Some(window) = web_sys::window() {
        let clipboard = window.navigator().clipboard();
        let _ = clipboard.write_text(url);
    }
}

#[component]
pub fn CloneButton(owner: String, name: String) -> impl IntoView {
    let active_tab = RwSignal::new("http");
    let http_url = RwSignal::new(String::new());
    let ssh_url = RwSignal::new(String::new());

    let ensure_urls = move || {
        if http_url.get().is_empty() {
            http_url.set(get_http_url(&owner, &name));
            ssh_url.set(get_ssh_url(&owner, &name));
        }
    };

    view! {
        <div class="relative inline-block">
        <details class="relative inline-block">
            <summary
                class="text-sm px-3 py-1.5 rounded-md border border-theme bg-surface-secondary text-text cursor-pointer list-none"
                on:click=move |_| ensure_urls()
            >
                "Clone"
                <span class="ml-1 text-xs text-muted">"▼"</span>
            </summary>
            <div class="absolute top-full left-0 mt-0.5 rounded-md border border-theme bg-surface shadow-lg z-10 min-w-[320px]">
                <div class="flex border-b border-theme">
                    <button
                        class="flex-1 px-3 py-2 text-sm cursor-pointer transition-colors"
                        class:border-b-2={true}
                        class:border-accent={move || active_tab.get() == "http"}
                        class:border-transparent={move || active_tab.get() != "http"}
                        class:text-accent={move || active_tab.get() == "http"}
                        class:text-muted={move || active_tab.get() != "http"}
                        on:click=move |_| active_tab.set("http")
                    >
                        "HTTP"
                    </button>
                    <button
                        class="flex-1 px-3 py-2 text-sm cursor-pointer transition-colors"
                        class:border-b-2={true}
                        class:border-accent={move || active_tab.get() == "ssh"}
                        class:border-transparent={move || active_tab.get() != "ssh"}
                        class:text-accent={move || active_tab.get() == "ssh"}
                        class:text-muted={move || active_tab.get() != "ssh"}
                        on:click=move |_| active_tab.set("ssh")
                    >
                        "SSH"
                    </button>
                </div>
                <div class="p-3">
                    <div class="flex items-center gap-2">
                        <input
                            type="text"
                            class="input flex-1"
                            readonly
                            value={move || if active_tab.get() == "http" { http_url.get() } else { ssh_url.get() }}
                        />
                        <button
                            class="btn-secondary text-sm !px-3 !py-1.5"
                            on:click=move |_| {
                                let url = if active_tab.get() == "http" { http_url.get() } else { ssh_url.get() };
                                copy_to_clipboard(&url);
                            }
                        >
                            "Copy"
                        </button>
                    </div>
                </div>
            </div>
        </details>
        </div>
    }
}
