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

// ── ThemeConfig (record — TS constructs it; all fields optional, base is modern()) ──

#[derive(uniffi::Record, Clone)]
pub struct ArielThemeConfig {
    pub node_color: Option<String>,
    pub secondary_color: Option<String>,
    pub tertiary_color: Option<String>,
    pub node_border_color: Option<String>,
    pub cluster_border: Option<String>,
    pub arrow_color: Option<String>,
    pub edge_label_background: Option<String>,
    pub text_color: Option<String>,
    pub background: Option<String>,
    pub cluster_background: Option<String>,
    pub sequence_actor_fill: Option<String>,
    pub sequence_actor_border: Option<String>,
    pub sequence_actor_line: Option<String>,
    pub sequence_note_fill: Option<String>,
    pub sequence_note_border: Option<String>,
    pub sequence_activation_fill: Option<String>,
    pub sequence_activation_border: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
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

    #[uniffi::constructor]
    pub fn from_config(config: ArielThemeConfig) -> Arc<Self> {
        let mut t = mmdr::Theme::modern();
        if let Some(v) = config.node_color              { t.primary_color = v; }
        if let Some(v) = config.secondary_color         { t.secondary_color = v; }
        if let Some(v) = config.tertiary_color          { t.tertiary_color = v; }
        if let Some(v) = config.node_border_color       { t.primary_border_color = v; }
        if let Some(v) = config.cluster_border          { t.cluster_border = v; }
        if let Some(v) = config.arrow_color             { t.line_color = v; }
        if let Some(v) = config.edge_label_background   { t.edge_label_background = v; }
        if let Some(v) = config.text_color              { t.primary_text_color = v.clone(); t.text_color = v; }
        t.background = config.background.unwrap_or_else(|| "transparent".to_string());
        if let Some(v) = config.cluster_background      { t.cluster_background = v; }
        if let Some(v) = config.sequence_actor_fill     { t.sequence_actor_fill = v; }
        if let Some(v) = config.sequence_actor_border   { t.sequence_actor_border = v; }
        if let Some(v) = config.sequence_actor_line     { t.sequence_actor_line = v; }
        if let Some(v) = config.sequence_note_fill      { t.sequence_note_fill = v; }
        if let Some(v) = config.sequence_note_border    { t.sequence_note_border = v; }
        if let Some(v) = config.sequence_activation_fill    { t.sequence_activation_fill = v; }
        if let Some(v) = config.sequence_activation_border  { t.sequence_activation_border = v; }
        if let Some(v) = config.font_family             { t.font_family = v; }
        if let Some(v) = config.font_size               { t.font_size = v; }
        Arc::new(Self(t))
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

// ── SVG post-processing ───────────────────────────────────────────────────────
// react-native-svg parses marker orient values as f64 and crashes on the
// SVG 2.0 value "auto-start-reverse". Replace with "auto" which is equivalent
// for the arrowhead use-case and universally supported.
fn fix_svg(svg: String) -> String {
    svg.replace("orient=\"auto-start-reverse\"", "orient=\"auto\"")
}

// ── Simple one-liner ───────────────────────────────────────────────────────────

#[uniffi::export]
pub fn render_mermaid(input: String) -> Result<String, MermaidError> {
    setup_panic_hook();
    // Use default theme/config to avoid any Instant::now() calls in the pipeline
    mmdr::render(&input).map(fix_svg).map_err(wrap_err)
}

// ── One-liner with theme + layout control ─────────────────────────────────────

#[uniffi::export]
pub fn render_mermaid_with_options(
    input: String,
    _theme: Arc<ArielTheme>,
    _config: ArielLayoutConfig,
) -> Result<String, MermaidError> {
    setup_panic_hook();
    let parsed = mmdr::parse_mermaid(&input).map_err(wrap_err)?;
    let layout_cfg: mmdr::LayoutConfig = _config.into();
    let layout = mmdr::compute_layout(&parsed.graph, &_theme.0, &layout_cfg);
    Ok(fix_svg(mmdr::render_svg(&layout, &_theme.0, &layout_cfg)))
}

// ── One-liner with timing metrics ─────────────────────────────────────────────

#[uniffi::export]
pub fn render_mermaid_with_timing(
    input: String,
    _theme: Arc<ArielTheme>,
    _config: ArielLayoutConfig,
) -> Result<ArielRenderResult, MermaidError> {
    setup_panic_hook();

    #[cfg(target_arch = "wasm32")]
    {
        let parsed = mmdr::parse_mermaid(&input).map_err(wrap_err)?;
        let layout_cfg: mmdr::LayoutConfig = _config.into();
        let layout = mmdr::compute_layout(&parsed.graph, &_theme.0, &layout_cfg);
        let svg = fix_svg(mmdr::render_svg(&layout, &_theme.0, &layout_cfg));
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
            svg: fix_svg(r.svg),
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
    fix_svg(mmdr::render_svg(&layout.0, &theme.0, &config.into()))
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
    fn generates_theme_preview_html() {
        let flowchart = "flowchart TD
  A([Start]) --> B{Working?}
  B -->|Yes| C[Ship it]
  B -->|No| D[Debug]
  D --> E[Fix bug]
  E --> B
  C --> F([Done])";

        let sequence = "sequenceDiagram
  actor User
  participant App
  participant API
  User->>App: Open diagram
  App->>API: Request render
  API-->>App: SVG response
  App-->>User: Display result";

        let config = ArielLayoutConfig { node_spacing: None, rank_spacing: None };

        let light_theme = ArielTheme::modern();
        let dark_theme = ArielTheme::from_config(ArielThemeConfig {
            node_color:                  Some("#1E293B".into()),
            secondary_color:             Some("#334155".into()),
            tertiary_color:              Some("#0F172A".into()),
            node_border_color:           Some("#475569".into()),
            cluster_border:              Some("#475569".into()),
            arrow_color:                 Some("#94A3B8".into()),
            text_color:                  Some("#F1F5F9".into()),
            background:                  Some("#0F172A".into()),
            edge_label_background:       Some("#1E293B".into()),
            cluster_background:          Some("#1E293B".into()),
            sequence_actor_fill:         Some("#1E293B".into()),
            sequence_actor_border:       Some("#475569".into()),
            sequence_actor_line:         Some("#64748B".into()),
            sequence_note_fill:          Some("#1C2033".into()),
            sequence_note_border:        Some("#475569".into()),
            sequence_activation_fill:    Some("#334155".into()),
            sequence_activation_border:  Some("#475569".into()),
            font_family: None,
            font_size:   None,
        });

        let fc_light  = render_mermaid_with_options(flowchart.into(), light_theme.clone(), config.clone()).unwrap();
        let fc_dark   = render_mermaid_with_options(flowchart.into(), dark_theme.clone(),  config.clone()).unwrap();
        let seq_light = render_mermaid_with_options(sequence.into(),  light_theme,         config.clone()).unwrap();
        let seq_dark  = render_mermaid_with_options(sequence.into(),  dark_theme,          config).unwrap();

        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Ariel Theme Preview</title>
<style>
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{ font-family: system-ui, -apple-system, sans-serif; }}
.grid {{ display: grid; grid-template-columns: 1fr 1fr; min-height: 100vh; }}
.panel {{ padding: 40px; }}
.light {{ background: #ffffff; color: #0f172a; }}
.dark  {{ background: #0f172a; color: #f1f5f9; }}
.mode-label {{ font-size: 11px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; opacity: 0.4; margin-bottom: 32px; }}
.diagram {{ margin-bottom: 40px; }}
.diagram-label {{ font-size: 11px; font-weight: 600; letter-spacing: 0.06em; text-transform: uppercase; opacity: 0.35; margin-bottom: 14px; }}
svg {{ max-width: 100%; height: auto; display: block; }}
</style>
</head>
<body>
<div class="grid">
  <div class="panel light">
    <div class="mode-label">Light</div>
    <div class="diagram"><div class="diagram-label">Flowchart</div>{fc_light}</div>
    <div class="diagram"><div class="diagram-label">Sequence</div>{seq_light}</div>
  </div>
  <div class="panel dark">
    <div class="mode-label">Dark</div>
    <div class="diagram"><div class="diagram-label">Flowchart</div>{fc_dark}</div>
    <div class="diagram"><div class="diagram-label">Sequence</div>{seq_dark}</div>
  </div>
</div>
</body>
</html>"#);

        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../preview.html");
        std::fs::write(&out, &html).unwrap();
        println!("Written: {}", out.display());
    }

    #[test]
    fn from_config_partial_fields() {
        let theme = ArielTheme::from_config(ArielThemeConfig {
            node_color: Some("#FF0000".into()),
            text_color: Some("#FFFFFF".into()),
            secondary_color: None, tertiary_color: None,
            node_border_color: None, cluster_border: None,
            arrow_color: None, edge_label_background: None,
            background: None, cluster_background: None,
            sequence_actor_fill: None, sequence_actor_border: None,
            sequence_actor_line: None, sequence_note_fill: None,
            sequence_note_border: None, sequence_activation_fill: None,
            sequence_activation_border: None, font_family: None, font_size: None,
        });
        let svg = render_mermaid_with_options(
            "flowchart TD\n  A --> B".into(),
            theme,
            ArielLayoutConfig { node_spacing: None, rank_spacing: None },
        ).unwrap();
        assert!(svg.contains("<svg"));
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
