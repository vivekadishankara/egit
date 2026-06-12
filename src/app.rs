use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Meta, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::navbar::Navbar;
use crate::pages::{
    auth::{login::LoginPage, register::RegisterPage},
    home::HomePage,
    repo::{
        blob::BlobPage,
        commit::CommitPage,
        commits::CommitsPage,
        create::CreateRepoPage,
        overview::RepoOverviewPage,
        pulls::{detail::PullDetailPage, list::PullListPage, new::NewPullPage},
        tree::TreePage,
    },
    user::profile::ProfilePage,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Meta charset="utf-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Title text="eGit"/>
        <Stylesheet id="leptos" href="/pkg/egit.css"/>
        <Link rel="icon" href="/favicon.ico"/>

        <Router>
            <Navbar/>
            <main class="min-h-screen bg-surface">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/login") view=LoginPage/>
                    <Route path=path!("/register") view=RegisterPage/>
                    <Route path=path!("/repos/new") view=CreateRepoPage/>
                    <Route path=path!("/:username") view=ProfilePage/>
                    <Route path=path!("/:username/:reponame") view=RepoOverviewPage/>
                    <Route path=path!("/:username/:reponame/tree/:branch") view=TreePage/>
                    <Route path=path!("/:username/:reponame/tree/:branch/*path") view=TreePage/>
                    <Route path=path!("/:username/:reponame/blob/:branch/*path") view=BlobPage/>
                    <Route path=path!("/:username/:reponame/commits") view=CommitsPage/>
                    <Route path=path!("/:username/:reponame/commits/:branch") view=CommitsPage/>
                    <Route path=path!("/:username/:reponame/commit/:id") view=CommitPage/>
                    <Route path=path!("/:username/:reponame/pulls") view=PullListPage/>
                    <Route path=path!("/:username/:reponame/pulls/new") view=NewPullPage/>
                    <Route path=path!("/:username/:reponame/pulls/:pr_id") view=PullDetailPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-[60vh] gap-4">
            <h1 class="text-6xl font-bold text-accent">"404"</h1>
            <p class="text-muted text-lg">"Page not found"</p>
            <a href="/" class="btn-primary">"Go home"</a>
        </div>
    }
}
