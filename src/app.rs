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
        <Link rel="icon" type_="image/svg+xml" href="data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%20viewBox%3D%220%200%201024%201024%22%3E%3Cdefs%3E%3ClinearGradient%20id%3D%22ringBlue%22%20x1%3D%220%25%22%20y1%3D%220%25%22%20x2%3D%22100%25%22%20y2%3D%22100%25%22%3E%3Cstop%20offset%3D%220%25%22%20stop-color%3D%22%234d95ff%22/%3E%3Cstop%20offset%3D%22100%25%22%20stop-color%3D%22%232270ea%22/%3E%3C/linearGradient%3E%3C/defs%3E%3Crect%20width%3D%221024%22%20height%3D%221024%22%20fill%3D%22transparent%22/%3E%3Ccircle%20cx%3D%22512%22%20cy%3D%22512%22%20r%3D%22350%22%20fill%3D%22none%22%20stroke%3D%22url%28%23ringBlue%29%22%20stroke-width%3D%22110%22/%3E%3Cpath%20d%3D%22M286%20409%20L370%20378%20H615%20C623%20378%20626%20382%20626%20389%20V419%20H753%20C744%20460%20707%20486%20630%20497%20C588%20503%20570%20530%20570%20590%20H648%20V646%20H582%20C568%20603%20462%20603%20448%20646%20H362%20V590%20C362%20590%20445%20571%20445%20526%20C445%20487%20389%20459%20286%20451%20Z%22%20fill%3D%22%233f87f5%22%20stroke%3D%22%2375adff%22%20stroke-width%3D%222%22/%3E%3Crect%20x%3D%22375%22%20y%3D%22392%22%20width%3D%2240%22%20height%3D%2216%22%20fill%3D%22%2382b7ff%22%20opacity%3D%220.8%22/%3E%3C/svg%3E"/>

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
