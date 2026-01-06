use std::{net::SocketAddr, sync::Arc};

use my_http_server::controllers::swagger::SwaggerMiddleware;
use my_http_server::MyHttpServer;

use crate::app::AppContext;

pub fn start(app: &Arc<AppContext>) {
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 8000)));

    let controllers = Arc::new(super::builder::build_controllers(&app));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        crate::app::APP_NAME.to_string(),
        crate::app::APP_VERSION.to_string(),
    );

    http_server.add_middleware(Arc::new(swagger_middleware));

    http_server.add_middleware(controllers);
    http_server.start(app.app_states.clone(), my_logger::LOGGER.clone());
}
