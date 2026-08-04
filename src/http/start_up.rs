use std::{net::SocketAddr, sync::Arc};

use mcp_server_middleware::McpMiddleware;
use my_http_server::controllers::swagger::SwaggerMiddleware;
use my_http_server::{MyHttpServer, StaticFilesMiddleware};

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

    mcp.register_tool_call(Arc::new(AddDomainHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(RenewCertificateHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetRenewStatusHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(UpdateCloudflareConfigHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetPrivateKeyHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetFullchainHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetCertInfoHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetDomainsListHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetDomainsWithExpirationHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(FixCertSymlinksHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetLetsEncryptLogHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(InitHttp01Handler::new(app.clone())));
    mcp.register_tool_call(Arc::new(ReissueHttp01Handler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetHttp01StatusHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(ListTasksHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(ListAcmeChallengesHandler::new(app.clone())));

    http_server.add_middleware(Arc::new(mcp));

    http_server.add_middleware(controllers);

    // Order is load-bearing: the static-files fallback answers every unmatched
    // path, so it sits last or it would swallow /api, /swagger and /mcp. Serves
    // the Dioxus admin UI from ./wwwroot (built by ./build-ui.sh, committed).
    http_server.add_middleware(Arc::new(
        StaticFilesMiddleware::new()
            .add_index_file("index.html")
            // The UI is a SPA: a deep link must return index.html, not a 404.
            .set_not_found_file("index.html".to_string())
            // Without ETags a redeployed wwwroot can be served from a stale
            // browser cache; dx content-hashes the wasm/js bundles but not
            // app.css or index.html.
            .with_etag(),
    ));

    http_server.start(app.app_states.clone(), my_logger::LOGGER.clone());
}
