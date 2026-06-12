use leptos::prelude::*;
use crate::components::repo_tab_bar::url_encode_branch;

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub name: String,
    pub is_dir: bool,
}

#[component]
pub fn FileTree(
    entries: Vec<TreeEntry>,
    #[prop(into)]
    username: String,
    #[prop(into)]
    reponame: String,
    #[prop(into)]
    branch: String,
    #[prop(into)]
    current_path: String,
) -> impl IntoView {
    let rows = entries
        .into_iter()
        .map(|entry| {
            let path = if current_path.is_empty() {
                entry.name.clone()
            } else {
                format!("{}/{}", current_path, entry.name)
            };

            if entry.is_dir {
                let href = format!("/{username}/{reponame}/tree/{}/{path}", url_encode_branch(&branch));
                view! {
                    <tr class="border-b border-theme hover:bg-surface-secondary">
                        <td class="px-3 py-1.5">
                            <a
                                href=href
                                class="flex items-center gap-2 no-underline text-accent text-sm"
                            >
                                <span class="text-muted select-none">"▸"</span>
                                <span>{entry.name}</span>
                            </a>
                        </td>
                    </tr>
                }
            } else {
                let href = format!("/{username}/{reponame}/blob/{}/{path}", url_encode_branch(&branch));
                view! {
                    <tr class="border-b border-theme hover:bg-surface-secondary">
                        <td class="px-3 py-1.5">
                            <a
                                href=href
                                class="flex items-center gap-2 no-underline text-sm"
                            >
                                <span class="text-muted select-none">" " </span>
                                <span>{entry.name}</span>
                            </a>
                        </td>
                    </tr>
                }
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="card overflow-x-auto">
            <table class="w-full">
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}
