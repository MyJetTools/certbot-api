use std::path::Path;

const CERT_FILES: [&str; 4] = ["cert", "chain", "fullchain", "privkey"];

pub struct FixedFile {
    pub file: String,
    pub action: String,
    pub backup_path: Option<String>,
}

pub struct FixSymlinksResult {
    pub domain: String,
    pub archive_version: u32,
    pub files: Vec<FixedFile>,
}

// Certbot requires every *.pem in /etc/letsencrypt/live/<name>/ to be a symlink
// into ../../archive/<name>/. If they were replaced by regular files (copy without -a,
// scp, docker cp, etc.) certbot refuses to renew with "expected ... to be a symlink".
// This repairs the live dir: regular files are moved aside to a backup dir and
// symlinks to the latest archive version are recreated.
pub async fn fix_cert_symlinks(domain: String) -> Result<FixSymlinksResult, String> {
    // Normalize "*.example.com" -> "example.com": certbot stores the cert under
    // the apex name regardless of whether the original request was wildcard.
    let cert_name = domain
        .strip_prefix("*.")
        .map(|s| s.to_string())
        .unwrap_or(domain);

    let archive_dir = format!("/etc/letsencrypt/archive/{}", cert_name);
    let live_dir = format!("/etc/letsencrypt/live/{}", cert_name);

    let version = find_latest_archive_version(&archive_dir).await?;

    // All four files of that version must exist before we touch anything in live/
    for kind in CERT_FILES {
        let archive_file = format!("{}/{}{}.pem", archive_dir, kind, version);
        if tokio::fs::symlink_metadata(&archive_file).await.is_err() {
            return Err(format!(
                "Archive file not found: {}. Archive is incomplete — re-issue the certificate instead of fixing symlinks.",
                archive_file
            ));
        }
    }

    tokio::fs::create_dir_all(&live_dir)
        .await
        .map_err(|e| format!("Failed to create live directory {}: {}", live_dir, e))?;

    let backup_dir = format!("/etc/letsencrypt/broken-live-backup/{}", cert_name);

    let mut files = Vec::new();

    for kind in CERT_FILES {
        let file_name = format!("{}.pem", kind);
        let live_path = format!("{}/{}", live_dir, file_name);
        let link_target = format!("../../archive/{}/{}{}.pem", cert_name, kind, version);

        let fixed = match tokio::fs::symlink_metadata(&live_path).await {
            Ok(meta) if meta.file_type().is_symlink() => {
                let current_target = tokio::fs::read_link(&live_path)
                    .await
                    .map_err(|e| format!("Failed to read symlink {}: {}", live_path, e))?;

                if current_target == Path::new(&link_target) {
                    FixedFile {
                        file: file_name,
                        action: "already_valid".to_string(),
                        backup_path: None,
                    }
                } else {
                    tokio::fs::remove_file(&live_path)
                        .await
                        .map_err(|e| format!("Failed to remove {}: {}", live_path, e))?;
                    create_symlink(&link_target, &live_path).await?;
                    FixedFile {
                        file: file_name,
                        action: "relinked".to_string(),
                        backup_path: None,
                    }
                }
            }
            Ok(_) => {
                // Regular file — the broken case. Move it aside, never delete:
                // it may hold a newer certificate than the archive.
                let backup_path = backup_file(&backup_dir, &live_path, &file_name).await?;
                create_symlink(&link_target, &live_path).await?;
                FixedFile {
                    file: file_name,
                    action: "backed_up_and_linked".to_string(),
                    backup_path: Some(backup_path),
                }
            }
            Err(_) => {
                create_symlink(&link_target, &live_path).await?;
                FixedFile {
                    file: file_name,
                    action: "created".to_string(),
                    backup_path: None,
                }
            }
        };

        files.push(fixed);
    }

    Ok(FixSymlinksResult {
        domain: cert_name,
        archive_version: version,
        files,
    })
}

async fn find_latest_archive_version(archive_dir: &str) -> Result<u32, String> {
    let mut entries = tokio::fs::read_dir(archive_dir).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!(
                "Archive directory not found: {}. Nothing to restore symlinks from — re-issue the certificate.",
                archive_dir
            )
        } else {
            format!("Failed to read directory {}: {}", archive_dir, e)
        }
    })?;

    let mut latest: Option<u32> = None;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };

        if let Some(version) = name
            .strip_prefix("cert")
            .and_then(|s| s.strip_suffix(".pem"))
            .and_then(|s| s.parse::<u32>().ok())
        {
            latest = Some(latest.map_or(version, |current| current.max(version)));
        }
    }

    latest.ok_or_else(|| {
        format!(
            "No certN.pem files found in {}. Nothing to restore symlinks from — re-issue the certificate.",
            archive_dir
        )
    })
}

async fn create_symlink(target: &str, link_path: &str) -> Result<(), String> {
    tokio::fs::symlink(target, link_path)
        .await
        .map_err(|e| format!("Failed to create symlink {} -> {}: {}", link_path, target, e))
}

async fn backup_file(backup_dir: &str, src: &str, file_name: &str) -> Result<String, String> {
    tokio::fs::create_dir_all(backup_dir)
        .await
        .map_err(|e| format!("Failed to create backup directory {}: {}", backup_dir, e))?;

    // Never overwrite an earlier backup — pick a free name with a numeric suffix.
    let mut backup_path = format!("{}/{}", backup_dir, file_name);
    let mut suffix = 1u32;
    while tokio::fs::symlink_metadata(&backup_path).await.is_ok() {
        backup_path = format!("{}/{}.{}", backup_dir, file_name, suffix);
        suffix += 1;
    }

    tokio::fs::rename(src, &backup_path)
        .await
        .map_err(|e| format!("Failed to move {} to {}: {}", src, backup_path, e))?;

    Ok(backup_path)
}
