pub mod app;
pub mod components;
pub mod diff;
pub mod pages;

#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod syntax;

#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod git;
#[cfg(feature = "ssr")]
pub mod git_routes;
#[cfg(feature = "ssr")]
pub mod error;

pub mod server;

#[cfg(feature = "ssr")]
pub use server::prs;
