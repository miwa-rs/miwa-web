use axum::{body::Body, Router};
use miwa::core::{Extension, SystemContext, SystemResult};
use miwa::derive::extension;
use tracing::info;

use crate::service::{RouterKind, WebService, WebServiceState};

pub struct WebServiceExtension(WebService);

#[async_trait::async_trait]
impl Extension for WebServiceExtension {
    async fn start(&self) -> SystemResult<()> {
        for (ctx, routers) in self.0.services.lock().unwrap().iter() {
            let server = self
                .0
                .servers
                .lock()
                .unwrap()
                .get(ctx)
                .cloned()
                .unwrap_or_default();

            let ctx_inner = ctx.clone();
            let routers_cloned = routers.clone();
            tokio::task::spawn(async move {
                let app: Router<WebServiceState, _> = routers_cloned.into_iter().fold(
                    Router::<WebServiceState, Body>::new(),
                    |acc: Router<WebServiceState, _>, router| match router.kind {
                        RouterKind::Nest(nest) => acc.nest(&nest, router.router),
                        RouterKind::Merge => acc.merge(router.router),
                    },
                );

                let app = app.with_state(WebServiceState);

                let addr = format!("127.0.0.1:{}", server.port).parse().unwrap();

                info!("Starting server with context {} at: {} ", ctx_inner, addr);

                axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .await
                    .unwrap();
            });
        }
        Ok(())
    }

    async fn shutdown(&self) -> SystemResult<()> {
        Ok(())
    }
}

#[extension(name = "WebService extension", provides(WebService))]
pub async fn web_service_extension(ctx: &SystemContext) -> SystemResult<WebServiceExtension> {
    let service = WebService::new();
    ctx.register(service.clone());
    Ok(WebServiceExtension(service))
}
