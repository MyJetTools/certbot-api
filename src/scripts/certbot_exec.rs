use std::process::Stdio;
use std::time::Duration;

use tokio::sync::Mutex;

/// Hard cap on a single certbot invocation. ACME validation + issuance normally
/// takes seconds, but a stuck network or a wedged challenge can hang the process
/// indefinitely, so every certbot run is killed after this long.
const CERTBOT_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// certbot takes exclusive host-wide fcntl locks (over /etc/letsencrypt,
/// /var/lib/letsencrypt, /var/log/letsencrypt) for its whole run and *refuses*
/// (does not wait) if another instance holds them — "Another instance of Certbot
/// is already running." So two certbot processes for two DIFFERENT lineages would
/// otherwise collide and the loser would fail for a reason unrelated to its cert.
///
/// This process-wide lock serialises every certbot invocation we make (issue,
/// reissue, renew, add-domain) so they never race that global lock.
static CERTBOT_LOCK: Mutex<()> = Mutex::const_new(());

/// Run a prepared certbot command under the global lock and the timeout, and
/// normalize the result. `fail_prefix` is prepended to certbot's stderr on a
/// non-zero exit (e.g. "Certbot HTTP-01 issuance failed").
///
/// `kill_on_drop`: on timeout the `output()` future is dropped, which kills the
/// child so a wedged certbot never keeps holding the global certbot lock.
pub async fn run_certbot(
    mut cmd: tokio::process::Command,
    fail_prefix: &str,
) -> Result<String, String> {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let _guard = CERTBOT_LOCK.lock().await;

    let output = match tokio::time::timeout(CERTBOT_TIMEOUT, cmd.output()).await {
        Ok(result) => result.map_err(|e| format!("Failed to execute certbot: {}", e))?,
        Err(_) => {
            return Err(format!(
                "{} timed out after {} minutes",
                fail_prefix,
                CERTBOT_TIMEOUT.as_secs() / 60
            ))
        }
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("{}: {}", fail_prefix, stderr))
    }
}
