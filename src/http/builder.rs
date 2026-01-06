use std::sync::Arc;

use my_http_server::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build_controllers(app: &Arc<AppContext>) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new(None, None);

    result.register_post_action(Arc::new(
        crate::http::controllers::cloudflare::UpdateCloudflareConfigAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        crate::http::controllers::certbot::AddDomainAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(
        crate::http::controllers::certificates::GetPrivateKeyAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(
        crate::http::controllers::certificates::GetFullchainAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(
        crate::http::controllers::certificates::GetCertInfoAction::new(app.clone()),
    ));

    result
}
