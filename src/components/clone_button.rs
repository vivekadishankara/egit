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

    let style_css = format!(
        "#{h}:checked ~ .clone-http-content {{ display: flex; }} \
         #{h}:checked ~ .clone-ssh-content {{ display: none; }} \
         #{h}:checked ~ .clone-tab-http {{ border-bottom-color: var(--color-accent); color: var(--color-accent); }} \
         #{h}:checked ~ .clone-tab-ssh {{ border-bottom-color: transparent; color: var(--color-muted); }} \
         #{s}:checked ~ .clone-http-content {{ display: none; }} \
         #{s}:checked ~ .clone-ssh-content {{ display: flex; }} \
         #{s}:checked ~ .clone-tab-http {{ border-bottom-color: transparent; color: var(--color-muted); }} \
         #{s}:checked ~ .clone-tab-ssh {{ border-bottom-color: var(--color-accent); color: var(--color-accent); }}",
        h = http_id, s = ssh_id
    );

    let copy_script = "var i=this.previousElementSibling;i.select();i.setSelectionRange(0,99999);document.execCommand('copy');var t=this.textContent;this.textContent='Copied!';setTimeout(function(){this.textContent=t}.bind(this),2000)";

    let close_script = "if(!window.__egitClose){window.__egitClose=true;document.addEventListener('click',function(e){document.querySelectorAll('details').forEach(function(d){if(!d.contains(e.target))d.removeAttribute('open')})})}";

    view! {
        <script>{close_script}</script>
        <details class="relative inline-block">
            <summary class="text-sm px-3 py-1.5 rounded-md border border-theme bg-surface-secondary text-text cursor-pointer list-none">
                "Clone"
                <span class="ml-1 text-xs text-muted">"▼"</span>
            </summary>
            <div class="absolute top-full left-0 mt-0.5 rounded-md border border-theme bg-surface shadow-lg z-10 min-w-[320px] bg-surface flex flex-wrap">
                <style>{style_css}</style>
                <input type="radio" name="clone-tab" id={http_id.clone()} checked style="display:none" />
                <input type="radio" name="clone-tab" id={ssh_id.clone()} style="display:none" />
                <label for={http_id.clone()} class="clone-tab-http flex-1 px-3 py-2 text-sm cursor-pointer border-b-2 text-center">
                    "HTTP"
                </label>
                <label for={ssh_id.clone()} class="clone-tab-ssh flex-1 px-3 py-2 text-sm cursor-pointer border-b-2 text-center">
                    "SSH"
                </label>
                <div class="p-3 clone-http-content w-full">
                    <div class="flex flex-col gap-2 w-full">
                        <input type="text" class="input w-full" readonly value=http_url />
                        <button class="self-start text-sm px-3 py-1 rounded-md bg-accent text-white cursor-pointer border-none" onclick={copy_script}>"Copy"</button>
                    </div>
                </div>
                <div class="p-3 clone-ssh-content w-full">
                    <div class="flex flex-col gap-2 w-full">
                        <input type="text" class="input w-full" readonly value=ssh_url />
                        <button class="self-start text-sm px-3 py-1 rounded-md bg-accent text-white cursor-pointer border-none" onclick={copy_script}>"Copy"</button>
                    </div>
                </div>
            </div>
        </details>
    }
}
