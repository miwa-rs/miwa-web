use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::Router;
use derive_builder::Builder;
use miwa::derive::Injectable;

#[derive(Injectable, Clone)]
pub struct WebService {
    pub(crate) services: Arc<Mutex<HashMap<String, Vec<WebServiceRouter>>>>,
    pub(crate) servers: Arc<Mutex<HashMap<String, WebServiceConfig>>>,
}

#[derive(Clone)]
pub enum RouterKind {
    Nest(String),
    Merge,
}

#[derive(Clone)]
pub struct WebServiceRouter {
    pub router: Router<WebServiceState>,
    pub kind: RouterKind,
}

impl WebServiceRouter {
    pub fn new(router: Router<WebServiceState>, kind: RouterKind) -> Self {
        Self { router, kind }
    }
}

impl WebService {
    pub fn new() -> Self {
        WebService {
            services: Arc::default(),
            servers: Arc::default(),
        }
    }
}

#[derive(Clone)]
pub struct WebServiceState;

static DEFAULT_CTX: &str = "default";

#[derive(Clone, Builder)]
pub struct WebServiceConfig {
    #[builder(default = "DEFAULT_CTX.to_string()")]
    pub ctx: String,
    pub port: u16,
}

impl Default for WebServiceConfig {
    fn default() -> Self {
        Self {
            ctx: "default".to_owned(),
            port: 3000,
        }
    }
}

impl WebService {
    pub fn nesting(&mut self, ctx: &str, nest: &str, router: Router<WebServiceState>) {
        let mut guard = self.services.lock().unwrap();
        let r = guard.entry(ctx.to_string()).or_insert_with(Vec::new);
        r.push(WebServiceRouter::new(
            router,
            RouterKind::Nest(nest.to_string()),
        ));
    }

    pub fn merging(&self, ctx: &str, router: Router<WebServiceState>) {
        let mut guard = self.services.lock().unwrap();
        let r = guard.entry(ctx.to_string()).or_insert_with(Vec::new);
        r.push(WebServiceRouter::new(router, RouterKind::Merge));
    }

    pub fn add_server(&self, server: WebServiceConfig) {
        self.servers
            .lock()
            .unwrap()
            .insert(server.ctx.clone(), server);
    }
}
