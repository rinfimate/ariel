import {
  ArielTheme,
  type ArielThemeLike,
  type ArielThemeConfig,
  type ArielLayoutConfig,
  type ArielRenderResult,
  renderMermaidWithOptions,
  renderMermaidWithTiming,
} from './generated/web/mermaid_wrapper';

export class ArielThemeRegistry {
  private readonly themes = new Map<string, ArielThemeLike>();
  private currentName: string;

  constructor(configs: Record<string, ArielThemeConfig>, defaultTheme?: string) {
    for (const [name, config] of Object.entries(configs)) {
      this.themes.set(name, ArielTheme.fromConfig(config));
    }
    const first = Object.keys(configs)[0];
    this.currentName = defaultTheme ?? first ?? 'light';
  }

  applyTheme(name: string): void {
    if (!this.themes.has(name)) throw new Error(`Theme "${name}" is not defined`);
    this.currentName = name;
  }

  getTheme(name?: string): ArielThemeLike {
    const key = name ?? this.currentName;
    const theme = this.themes.get(key);
    if (!theme) throw new Error(`Theme "${key}" is not defined`);
    return theme;
  }

  get currentTheme(): ArielThemeLike {
    return this.getTheme();
  }

  render(input: string, config: ArielLayoutConfig = {}): string {
    return renderMermaidWithOptions(input, this.currentTheme, config);
  }

  renderWithTiming(input: string, config: ArielLayoutConfig = {}): ArielRenderResult {
    return renderMermaidWithTiming(input, this.currentTheme, config);
  }
}
