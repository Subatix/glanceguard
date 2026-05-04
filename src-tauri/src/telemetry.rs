//! Opt-in native crash reporting (DECISIONS D8). DSN: `GLANCEGUARD_SENTRY_DSN` at runtime.

use std::sync::Mutex;

use sentry::ClientOptions;

static GUARD: Mutex<Option<sentry::ClientInitGuard>> = Mutex::new(None);

pub fn sync_rust_sentry(enabled: bool) {
    let mut slot = GUARD.lock().expect("telemetry mutex poisoned");
    *slot = None;
    if !enabled {
        return;
    }
    let Some(dsn) = std::env::var("GLANCEGUARD_SENTRY_DSN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    let opts = ClientOptions {
        release: Some(std::borrow::Cow::Owned(format!(
            "glanceguard@{}",
            env!("CARGO_PKG_VERSION")
        ))),
        ..Default::default()
    };
    *slot = Some(sentry::init((dsn, opts)));
}
