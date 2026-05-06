use std::{net::SocketAddr, sync::Arc};

use mcp_server_middleware::McpMiddleware;
use my_http_server::controllers::swagger::SwaggerMiddleware;
use my_http_server::MyHttpServer;

use crate::app::AppContext;
use crate::mcp::*;

pub async fn start(app: &Arc<AppContext>) {
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 8000)));

    let controllers = Arc::new(super::builder::build_controllers(&app));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        crate::app::APP_NAME.to_string(),
        crate::app::APP_VERSION.to_string(),
    );

    http_server.add_middleware(Arc::new(swagger_middleware));

    let mut mcp = McpMiddleware::new(
        "/mcp",
        crate::app::APP_NAME,
        crate::app::APP_VERSION,
        "Certbot API — manage Let's Encrypt certificates over MCP",
    );

    mcp.register_tool_call(Arc::new(AddDomainHandler::new(app.clone())))
        .await;
    mcp.register_tool_call(Arc::new(RenewCertificateHandler::new(app.clone())))
        .await;
    mcp.register_tool_call(Arc::new(GetRenewStatusHandler::new(app.clone())))
        .await;
    mcp.register_tool_call(Arc::new(UpdateCloudflareConfigHandler::new(app.clone())))
        .await;
    mcp.register_tool_call(Arc::new(GetPrivateKeyHandler::new(app.clone())))
        .await;
    mcp.register_tool_call(Arc::new(GetFullchainHandler::new(app.clone())))
        .await;
    mcp.register_tool_call(Arc::new(GetCertInfoHandler::new(app.clone())))
        .await;
    mcp.register_tool_call(Arc::new(GetDomainsListHandler::new(app.clone())))
        .await;
    mcp.register_tool_call(Arc::new(GetDomainsWithExpirationHandler::new(app.clone())))
        .await;

    http_server.add_middleware(Arc::new(mcp));

    http_server.add_middleware(controllers);
    http_server.start(app.app_states.clone(), my_logger::LOGGER.clone());
}
