//! Self-update against GitHub Releases.
//!
//! On startup the app asks GitHub for the latest release; if it is newer than
//! the running version a banner is shown. Installing downloads the artifact for
//! the current platform, replaces the running binary and asks the user to
//! restart. All network and disk work happens on background threads so the UI
//! never blocks.

use std::sync::{Arc, Mutex};

const REPO_OWNER: &str = "micferna";
const REPO_NAME: &str = "rust-viewer-pro";
const BIN_NAME: &str = "rust-viewer-pro";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// State machine for the update flow, polled by the UI each frame.
#[derive(Clone, Debug, Default)]
pub enum UpdateStatus {
    /// Nothing has happened yet.
    #[default]
    Idle,
    /// A release check is in flight.
    Checking,
    /// The running version is the latest.
    UpToDate,
    /// A newer release is available.
    Available { version: String, notes: String },
    /// The new version is being downloaded and installed.
    Installing,
    /// Installed successfully; a restart is required to apply it.
    Installed { version: String },
    /// Something went wrong (network, permissions, ...).
    Failed(String),
}

/// Cheap-to-clone handle shared between the UI and worker threads.
#[derive(Clone)]
pub struct Updater {
    status: Arc<Mutex<UpdateStatus>>,
    ctx: egui::Context,
}

impl Updater {
    pub fn new(ctx: egui::Context) -> Self {
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
            ctx,
        }
    }

    /// Current status snapshot.
    pub fn status(&self) -> UpdateStatus {
        self.status.lock().expect("update status poisoned").clone()
    }

    pub fn current_version() -> &'static str {
        CURRENT_VERSION
    }

    fn set(&self, status: UpdateStatus) {
        *self.status.lock().expect("update status poisoned") = status;
        self.ctx.request_repaint();
    }

    /// Check GitHub for a newer release, in the background.
    pub fn check(&self) {
        // Don't start a second check while one is running or already resolved.
        if !matches!(self.status(), UpdateStatus::Idle) {
            return;
        }
        let this = self.clone();
        self.set(UpdateStatus::Checking);
        std::thread::Builder::new()
            .name("update-check".to_owned())
            .spawn(move || match latest_release() {
                Ok(Some((version, notes))) => this.set(UpdateStatus::Available { version, notes }),
                Ok(None) => this.set(UpdateStatus::UpToDate),
                Err(e) => {
                    log::warn!("update check failed: {e}");
                    this.set(UpdateStatus::Failed(e));
                }
            })
            .expect("failed to spawn update-check thread");
    }

    /// Download and install the latest release, in the background.
    pub fn install(&self) {
        if matches!(self.status(), UpdateStatus::Installing) {
            return;
        }
        let this = self.clone();
        self.set(UpdateStatus::Installing);
        std::thread::Builder::new()
            .name("update-install".to_owned())
            .spawn(move || match install_latest() {
                Ok(version) => this.set(UpdateStatus::Installed { version }),
                Err(e) => {
                    log::error!("update install failed: {e}");
                    this.set(UpdateStatus::Failed(e));
                }
            })
            .expect("failed to spawn update-install thread");
    }
}

/// Returns `Some((version, notes))` if a newer release exists, else `None`.
fn latest_release() -> Result<Option<(String, String)>, String> {
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()
        .map_err(|e| e.to_string())?
        .fetch()
        .map_err(|e| e.to_string())?;

    let Some(latest) = releases.into_iter().next() else {
        return Ok(None);
    };

    let newer = self_update::version::bump_is_greater(CURRENT_VERSION, &latest.version)
        .map_err(|e| e.to_string())?;

    if newer {
        let notes = latest.body.unwrap_or_default();
        Ok(Some((latest.version, notes)))
    } else {
        Ok(None)
    }
}

/// Replace the running binary with the latest release. Returns the new version.
fn install_latest() -> Result<String, String> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(CURRENT_VERSION)
        .show_download_progress(false)
        .no_confirm(true)
        .build()
        .map_err(|e| e.to_string())?
        .update()
        .map_err(|e| e.to_string())?;

    Ok(status.version().to_owned())
}

/// Restart the application from the (possibly just-updated) binary on disk.
pub fn restart() -> std::io::Error {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return e,
    };
    let err = std::process::Command::new(exe)
        .args(std::env::args_os().skip(1))
        .spawn();
    match err {
        Ok(_) => std::process::exit(0),
        Err(e) => e,
    }
}
