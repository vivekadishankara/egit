use leptos::prelude::*;

use crate::server::repos::DeleteRepo;

#[component]
pub fn DeleteRepoButton(
    owner: String,
    reponame: String,
    delete_action: ServerAction<DeleteRepo>,
) -> impl IntoView {
    view! {
        <ActionForm action=delete_action>
            <input type="hidden" name="username" value=owner/>
            <input type="hidden" name="reponame" value=reponame/>
            <button
                type="submit"
                class="btn-danger text-sm"
                onclick="return confirm('Are you sure you want to delete this repository? This action cannot be undone.')"
            >
                "Delete"
            </button>
        </ActionForm>
    }
}
