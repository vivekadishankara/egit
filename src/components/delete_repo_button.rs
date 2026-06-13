use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::server::repos::DeleteRepo;

#[component]
pub fn DeleteRepoButton(
    owner: String,
    reponame: String,
) -> impl IntoView {
    let delete_action = ServerAction::<DeleteRepo>::new();
    let delete_action_for_effect = delete_action.clone();
    let navigate = use_navigate();
    let navigate_delete = navigate.clone();
    let owner_for_effect = owner.clone();
    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action_for_effect.value().get() {
            navigate_delete(&format!("/{}", owner_for_effect), Default::default());
        }
    });

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
