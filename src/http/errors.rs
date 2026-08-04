use my_http_server::HttpFailResult;

use crate::scripts::CertbotPrepareError;

/// Map a typed prepare failure to the right HTTP status: missing lineage → 404,
/// bad request → 400, server-side IO/parse failure → 500.
pub fn prepare_error_to_fail(error: CertbotPrepareError) -> HttpFailResult {
    match error {
        CertbotPrepareError::NotFound(message) => HttpFailResult::as_not_found(message, false),
        CertbotPrepareError::Invalid(message) => HttpFailResult::as_validation_error(message),
        CertbotPrepareError::Io(message) => HttpFailResult::as_fatal_error(message),
    }
}
