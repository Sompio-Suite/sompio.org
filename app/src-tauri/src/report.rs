//! "Report a bug": sends what the person wrote to sompio.org, which emails it to us. Runs only
//! when they press Send, and sends only what the window showed them: their message, the version
//! and system, and the problems log lines and screenshot only if they ticked or chose them.

use crate::{problems, CmdResult};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

const REPORT_URL: &str = "https://sompio.org/report.php";
const MAX_SHOT: u64 = 4 * 1024 * 1024;

#[derive(Deserialize)]
pub struct BugReport {
    area: String,
    kind: String,
    message: String,
    contact: String,
    /// Add the last lines of the problems log
    details: bool,
    /// A picture the person chose to attach
    screenshot: Option<String>,
}

#[derive(Serialize)]
pub struct Sent {
    message: String,
    id: String,
}

#[derive(Deserialize)]
struct Answer {
    ok: bool,
    message: String,
    #[serde(default)]
    id: String,
}

#[tauri::command]
pub async fn send_bug_report(app: AppHandle, report: BugReport) -> CmdResult<Sent> {
    let d = problems::app_details(app)?;
    let details = if report.details { d.recent.join("\n") } else { String::new() };
    let screenshot = match &report.screenshot {
        Some(path) => Some(read_picture(path)?),
        None => None,
    };
    let body = serde_json::json!({
        "version": d.version,
        "os": d.os,
        "area": report.area,
        "kind": report.kind,
        "message": report.message,
        "contact": report.contact,
        "details": details,
        "screenshot": screenshot,
    })
    .to_string();
    let agent_name = format!("Sompio/{}", d.version);
    tauri::async_runtime::spawn_blocking(move || {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(60)))
            .http_status_as_error(false)
            .build()
            .into();
        let mut res = agent
            .post(REPORT_URL)
            .header("User-Agent", &agent_name)
            .header("Content-Type", "application/json")
            .send(body)
            .map_err(|e| format!("Couldn't reach sompio.org ({e}). Check your internet connection, or email hello@sompio.org."))?;
        let text = res.body_mut().read_to_string().unwrap_or_default();
        match serde_json::from_str::<Answer>(&text) {
            Ok(a) if a.ok => Ok(Sent { message: a.message, id: a.id }),
            Ok(a) => Err(a.message),
            Err(_) => Err(format!("sompio.org didn't accept the report (error {}). Please try again later, or email hello@sompio.org.", res.status().as_u16())),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The chosen screenshot, checked to be a small PNG, JPG or WebP picture.
fn read_picture(path: &str) -> CmdResult<serde_json::Value> {
    let size = std::fs::metadata(path).map_err(|e| format!("Couldn't open the screenshot: {e}"))?.len();
    if size > MAX_SHOT {
        return Err("The screenshot is bigger than 4 MB. Choose a smaller one, or crop it.".into());
    }
    let data = std::fs::read(path).map_err(|e| format!("Couldn't open the screenshot: {e}"))?;
    let ext = match data.get(..12) {
        Some(h) if h.starts_with(b"\x89PNG") => "png",
        Some(h) if h.starts_with(b"\xFF\xD8\xFF") => "jpg",
        Some(h) if h.starts_with(b"RIFF") && &h[8..12] == b"WEBP" => "webp",
        _ => return Err("The screenshot needs to be a PNG or JPG picture.".into()),
    };
    Ok(serde_json::json!({ "name": format!("screenshot.{ext}"), "data": B64.encode(data) }))
}
