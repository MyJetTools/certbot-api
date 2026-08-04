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

    result.register_post_action(Arc::new(
        crate::http::controllers::certbot::StartRenewAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(
        crate::http::controllers::certbot::CheckRenewAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        crate::http::controllers::certbot::FixSymlinksAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        crate::http::controllers::http01::InitHttp01Action::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        crate::http::controllers::http01::ReissueHttp01Action::new(app.clone()),
    ));

    result.register_get_action(Arc::new(
        crate::http::controllers::http01::Http01StatusAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(
        crate::http::controllers::tasks::ListTasksAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(
        crate::http::controllers::certbot::GetLogAction::new(app.clone()),
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

    result.register_get_action(Arc::new(
        crate::http::controllers::certificates::GetDomainsListAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(
        crate::http::controllers::certificates::GetDomainsWithExpirationAction::new(app.clone()),
    ));

    result
}
