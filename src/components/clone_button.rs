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

#[component]
pub fn CloneButton(owner: String, name: String) -> impl IntoView {
    let http_url = get_http_url(&owner, &name);
    let ssh_url = get_ssh_url(&owner, &name);
    let http_id = format!("clone-http-{}-{}", owner, name);
    let ssh_id = format!("clone-ssh-{}-{}", owner, name);

    // CSS to show/hide tab content based on radio state
    let style_css = format!(
        "#{h}:checked ~ .clone-http-content {{ display: flex; }} \
         #{h}:checked ~ .clone-ssh-content {{ display: none; }} \
         #{s}:checked ~ .clone-http-content {{ display: none; }} \
         #{s}:checked ~ .clone-ssh-content {{ display: flex; }}",
        h = http_id, s = ssh_id
    );

    view! {
        <details class="relative inline-block">
            <summary class="text-sm px-3 py-1.5 rounded-md border border-theme bg-surface-secondary text-text cursor-pointer list-none">
                "Clone"
                <span class="ml-1 text-xs text-muted">"▼"</span>
            </summary>
            <div class="absolute top-full left-0 mt-0.5 rounded-md border border-theme bg-surface shadow-lg z-10 min-w-[320px] bg-surface">
                <style>{style_css}</style>
                <input type="radio" name="clone-tab" id={http_id.clone()} checked style="display:none" />
                <input type="radio" name="clone-tab" id={ssh_id.clone()} style="display:none" />
                <div class="flex border-b border-theme">
                    <label for={http_id.clone()} class="flex-1 px-3 py-2 text-sm cursor-pointer border-b-2 border-accent text-accent text-center">
                        "HTTP"
                    </label>
                    <label for={ssh_id.clone()} class="flex-1 px-3 py-2 text-sm cursor-pointer border-b-2 border-transparent text-muted text-center">
                        "SSH"
                    </label>
                </div>
                <div class="p-3 clone-http-content" style="display:flex">
                    <div class="flex items-center gap-2 w-full">
                        <input type="text" class="input flex-1" readonly value=http_url />
                    </div>
                </div>
                <div class="p-3 clone-ssh-content" style="display:none">
                    <div class="flex items-center gap-2 w-full">
                        <input type="text" class="input flex-1" readonly value=ssh_url />
                    </div>
                </div>
            </div>
        </details>
    }
}
