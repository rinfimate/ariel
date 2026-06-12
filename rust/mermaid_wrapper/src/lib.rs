use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ariel_rs::theme::Theme;

uniffi::setup_scaffolding!();

// ── Error ──────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum MermaidError {
    #[error("{message}")]
    RenderError { message: String },
}

fn wrap_err(e: impl std::fmt::Display) -> MermaidError {
    MermaidError::RenderError { message: e.to_string() }
}

// ── Debug timing toggle ────────────────────────────────────────────────────────

static TIMING_LOGS: AtomicBool = AtomicBool::new(false);

#[uniffi::export]
pub fn set_timing_logs(enabled: bool) {
    TIMING_LOGS.store(enabled, Ordering::Relaxed);
}

// ── Theme (opaque — TS gets a handle and passes it to render calls) ────────────

#[derive(uniffi::Object)]
pub struct ArielTheme(Theme);

#[uniffi::export]
impl ArielTheme {
    /// Standard Mermaid light theme (white background, purple accents).
    #[uniffi::constructor]
    pub fn light() -> Arc<Self> {
        Arc::new(Self(Theme::Default))
    }

    /// Dark theme for dark-mode UIs.
    #[uniffi::constructor]
    pub fn dark() -> Arc<Self> {
        Arc::new(Self(Theme::Dark))
    }

    /// Forest/green-tinted light theme.
    #[uniffi::constructor]
    pub fn forest() -> Arc<Self> {
        Arc::new(Self(Theme::Forest))
    }

    /// Neutral greyscale theme.
    #[uniffi::constructor]
    pub fn neutral() -> Arc<Self> {
        Arc::new(Self(Theme::Neutral))
    }
}

// ── WASM panic hook ────────────────────────────────────────────────────────────

fn setup_panic_hook() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

// ── SVG post-processing ────────────────────────────────────────────────────────
// react-native-svg parses marker orient values as f64 and crashes on the
// SVG 2.0 value "auto-start-reverse". Replace with "auto" which is equivalent
// for the arrowhead use-case and universally supported.
fn fix_svg(svg: String) -> String {
    svg.replace("orient=\"auto-start-reverse\"", "orient=\"auto\"")
}

// ── Render with default (light) theme ─────────────────────────────────────────

#[uniffi::export]
pub fn render_mermaid(input: String) -> Result<String, MermaidError> {
    setup_panic_hook();
    ariel_rs::try_render(&input, Theme::Default).map(fix_svg).map_err(wrap_err)
}

// ── Render with explicit theme ─────────────────────────────────────────────────

#[uniffi::export]
pub fn render_mermaid_with_theme(
    input: String,
    theme: Arc<ArielTheme>,
) -> Result<String, MermaidError> {
    setup_panic_hook();
    ariel_rs::try_render(&input, theme.0).map(fix_svg).map_err(wrap_err)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_flowchart() {
        let svg = render_mermaid("flowchart TD\n  A --> B".into()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn renders_sequence_diagram() {
        let svg = render_mermaid("sequenceDiagram\n  Alice->>Bob: Hello".into()).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn renders_pie_chart() {
        let svg = render_mermaid("pie\n  title Pets\n  \"Dogs\" : 386\n  \"Cats\" : 85".into()).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn renders_with_light_theme() {
        let svg = render_mermaid_with_theme(
            "flowchart LR\n  A --> B --> C".into(),
            ArielTheme::light(),
        ).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn renders_with_dark_theme() {
        let svg = render_mermaid_with_theme(
            "flowchart TD\n  A --> B".into(),
            ArielTheme::dark(),
        ).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn renders_with_forest_theme() {
        let svg = render_mermaid_with_theme(
            "flowchart TD\n  A --> B".into(),
            ArielTheme::forest(),
        ).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn renders_with_neutral_theme() {
        let svg = render_mermaid_with_theme(
            "flowchart TD\n  A --> B".into(),
            ArielTheme::neutral(),
        ).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn unknown_input_returns_error() {
        let result = render_mermaid("not valid mermaid %%%".into());
        assert!(result.is_err());
    }

    #[test]
    fn set_timing_logs_toggles_without_panic() {
        set_timing_logs(true);
        set_timing_logs(false);
    }
}
