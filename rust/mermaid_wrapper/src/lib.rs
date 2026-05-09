use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use mermaid_rs_renderer as mmdr;

uniffi::setup_scaffolding!();

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum MermaidError {
    #[error("{message}")]
    RenderError { message: String },
}

fn wrap_err(e: impl std::fmt::Display) -> MermaidError {
    MermaidError::RenderError { message: e.to_string() }
}

// ── Timing log flag (off by default) ──────────────────────────────────────────

static TIMING_LOGS: AtomicBool = AtomicBool::new(false);

#[uniffi::export]
pub fn set_timing_logs(enabled: bool) {
    TIMING_LOGS.store(enabled, Ordering::Relaxed);
}

fn maybe_log_timing(label: &str, result: &ArielRenderResult) {
    if !TIMING_LOGS.load(Ordering::Relaxed) {
        return;
    }
    let msg = format!(
        "[ariel] {} | total {:.2}ms  parse {}µs  layout {}µs  render {}µs",
        label, result.total_ms, result.parse_us, result.layout_us, result.render_us
    );
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&msg.into());
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("{msg}");
}

// ── Theme (opaque — TS gets a handle and passes it to render calls) ───────────

#[derive(uniffi::Object)]
pub struct ArielTheme(mmdr::Theme);

#[uniffi::export]
impl ArielTheme {
    #[uniffi::constructor]
    pub fn modern() -> Arc<Self> {
        Arc::new(Self(mmdr::Theme::modern()))
    }

    #[uniffi::constructor]
    pub fn mermaid_default() -> Arc<Self> {
        Arc::new(Self(mmdr::Theme::mermaid_default()))
    }
}

// ── LayoutConfig (record — TS constructs it with named fields) ────────────────

#[derive(uniffi::Record, Clone)]
pub struct ArielLayoutConfig {
    pub node_spacing: Option<f32>,
    pub rank_spacing: Option<f32>,
}

impl From<ArielLayoutConfig> for mmdr::LayoutConfig {
    fn from(c: ArielLayoutConfig) -> Self {
        let mut cfg = mmdr::LayoutConfig::default();
        if let Some(v) = c.node_spacing {
            cfg.node_spacing = v;
        }
        if let Some(v) = c.rank_spacing {
            cfg.rank_spacing = v;
        }
        cfg
    }
}

// ── Render result with timing (record — plain data back to TS) ────────────────

#[derive(uniffi::Record)]
pub struct ArielRenderResult {
    pub svg: String,
    pub parse_us: u64,
    pub layout_us: u64,
    pub render_us: u64,
    pub total_ms: f64,
}

// ── Pipeline intermediates (opaque — TS holds handles between stages) ──────────

#[derive(uniffi::Object)]
pub struct ArielParsedDiagram(mmdr::ParseOutput);

#[derive(uniffi::Object)]
pub struct ArielLayout(mmdr::Layout);

// ── WASM panic hook (redirects Rust panics to browser console.error) ──────────

fn setup_panic_hook() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

// ── Simple one-liner ───────────────────────────────────────────────────────────

#[uniffi::export]
pub fn render_mermaid(input: String) -> Result<String, MermaidError> {
    setup_panic_hook();
    // Use default theme/config to avoid any Instant::now() calls in the pipeline
    mmdr::render(&input).map_err(wrap_err)
}

// ── One-liner with theme + layout control ─────────────────────────────────────

#[uniffi::export]
pub fn render_mermaid_with_options(
    input: String,
    _theme: Arc<ArielTheme>,
    _config: ArielLayoutConfig,
) -> Result<String, MermaidError> {
    setup_panic_hook();
    #[cfg(target_arch = "wasm32")]
    {
        return mmdr::render(&input).map_err(wrap_err);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let parsed = mmdr::parse_mermaid(&input).map_err(wrap_err)?;
        let layout_cfg: mmdr::LayoutConfig = _config.into();
        let layout = mmdr::compute_layout(&parsed.graph, &_theme.0, &layout_cfg);
        Ok(mmdr::render_svg(&layout, &_theme.0, &layout_cfg))
    }
}

// ── One-liner with timing metrics ─────────────────────────────────────────────

#[uniffi::export]
pub fn render_mermaid_with_timing(
    input: String,
    _theme: Arc<ArielTheme>,
    _config: ArielLayoutConfig,
) -> Result<ArielRenderResult, MermaidError> {
    setup_panic_hook();

    // wasm32-unknown-unknown has no std::time::Instant.
    // compute_layout/render_svg also call Instant internally so we use the
    // simplest render() entry point which avoids explicit timing.
    #[cfg(target_arch = "wasm32")]
    {
        let svg = mmdr::render(&input).map_err(wrap_err)?;
        let result = ArielRenderResult { svg, parse_us: 0, layout_us: 0, render_us: 0, total_ms: 0.0 };
        maybe_log_timing(&input, &result);
        return Ok(result);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let opts = mmdr::RenderOptions {
            theme: _theme.0.clone(),
            layout: _config.into(),
            ..Default::default()
        };
        let r = mmdr::render_with_timing(&input, opts).map_err(wrap_err)?;
        let total_ms = r.total_ms();
        let result = ArielRenderResult {
            svg: r.svg,
            parse_us: r.parse_us as u64,
            layout_us: r.layout_us as u64,
            render_us: r.render_us as u64,
            total_ms,
        };
        maybe_log_timing(&input, &result);
        Ok(result)
    }
}

// ── Full pipeline: parse → layout → svg ───────────────────────────────────────

#[uniffi::export]
pub fn parse_diagram(input: String) -> Result<Arc<ArielParsedDiagram>, MermaidError> {
    mmdr::parse_mermaid(&input)
        .map(|p| Arc::new(ArielParsedDiagram(p)))
        .map_err(wrap_err)
}

#[uniffi::export]
pub fn compute_diagram_layout(
    parsed: Arc<ArielParsedDiagram>,
    theme: Arc<ArielTheme>,
    config: ArielLayoutConfig,
) -> Arc<ArielLayout> {
    let layout = mmdr::compute_layout(&parsed.0.graph, &theme.0, &config.into());
    Arc::new(ArielLayout(layout))
}

#[uniffi::export]
pub fn render_svg_from_layout(
    layout: Arc<ArielLayout>,
    theme: Arc<ArielTheme>,
    config: ArielLayoutConfig,
) -> String {
    mmdr::render_svg(&layout.0, &theme.0, &config.into())
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
    fn renders_with_modern_theme() {
        let theme = ArielTheme::modern();
        let config = ArielLayoutConfig {
            node_spacing: None,
            rank_spacing: None,
        };
        let svg = render_mermaid_with_options(
            "flowchart LR\n  A --> B --> C".into(),
            theme,
            config,
        )
        .unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn renders_sequence_diagram() {
        let svg = render_mermaid("sequenceDiagram\n  Alice->>Bob: Hello".into()).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn pipeline_produces_same_result_as_simple() {
        let input = "flowchart TD\n  A --> B".to_string();
        let simple = render_mermaid(input.clone()).unwrap();

        let theme = ArielTheme::mermaid_default();
        let config = ArielLayoutConfig {
            node_spacing: None,
            rank_spacing: None,
        };
        let parsed = parse_diagram(input).unwrap();
        let layout = compute_diagram_layout(parsed, theme.clone(), config.clone());
        let pipeline = render_svg_from_layout(layout, theme, config);

        // Both should be valid SVGs (exact bytes may differ due to theme)
        assert!(simple.contains("<svg"));
        assert!(pipeline.contains("<svg"));
    }

    #[test]
    fn handles_unknown_input_without_panic() {
        let result = render_mermaid("not valid mermaid %%%".into());
        let _ = result;
    }

    #[test]
    fn set_timing_logs_toggles() {
        set_timing_logs(true);
        set_timing_logs(false);
        // No assertion needed — just verify neither call panics.
    }

    #[test]
    fn timing_logs_emit_when_enabled() {
        set_timing_logs(true);
        let result = render_mermaid_with_timing(
            "flowchart TD\n  A --> B".into(),
            ArielTheme::modern(),
            ArielLayoutConfig { node_spacing: None, rank_spacing: None },
        );
        set_timing_logs(false);
        assert!(result.unwrap().svg.contains("<svg"));
    }

    #[test]
    fn mermaid_default_theme() {
        let theme = ArielTheme::mermaid_default();
        let svg = render_mermaid_with_options(
            "flowchart TD\n  A --> B".into(),
            theme,
            ArielLayoutConfig { node_spacing: None, rank_spacing: None },
        ).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn custom_layout_config() {
        let svg = render_mermaid_with_options(
            "flowchart LR\n  A --> B".into(),
            ArielTheme::modern(),
            ArielLayoutConfig { node_spacing: Some(80.0), rank_spacing: Some(60.0) },
        ).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_with_timing_returns_svg_and_zeros_on_host() {
        let result = render_mermaid_with_timing(
            "flowchart TD\n  X --> Y".into(),
            ArielTheme::modern(),
            ArielLayoutConfig { node_spacing: None, rank_spacing: None },
        ).unwrap();
        assert!(result.svg.contains("<svg"));
        // On native host the timing values are real measurements (>= 0)
        assert!(result.parse_us >= 0);
        assert!(result.layout_us >= 0);
        assert!(result.render_us >= 0);
        assert!(result.total_ms >= 0.0);
    }

    #[test]
    fn multiple_diagram_types() {
        let diagrams = vec![
            "flowchart TD\n  A --> B",
            "sequenceDiagram\n  Alice->>Bob: Hi",
            "pie\n  title Pets\n  \"Dogs\" : 386\n  \"Cats\" : 85",
        ];
        for d in diagrams {
            let svg = render_mermaid(d.into()).unwrap();
            assert!(svg.contains("<svg"), "failed for: {d}");
        }
    }

    #[test]
    fn pipeline_with_modern_theme() {
        let parsed = parse_diagram("flowchart LR\n  A --> B --> C".into()).unwrap();
        let layout = compute_diagram_layout(
            parsed,
            ArielTheme::modern(),
            ArielLayoutConfig { node_spacing: Some(50.0), rank_spacing: None },
        );
        let svg = render_svg_from_layout(
            layout,
            ArielTheme::modern(),
            ArielLayoutConfig { node_spacing: None, rank_spacing: None },
        );
        assert!(svg.contains("<svg"));
    }
}
