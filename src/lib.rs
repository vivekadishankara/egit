pub mod app;
pub mod components;
pub mod pages;

#[cfg(feature = "ssr")]
pub mod db;

#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod git;
#[cfg(feature = "ssr")]
pub mod git_routes;
#[cfg(feature = "ssr")]
pub mod error;

#[cfg(feature = "ssr")]
pub mod server;
