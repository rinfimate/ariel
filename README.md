# Ariel

> **Mermaid diagrams for React Native and React Native Web. No DOM. No WebView. Pure Rust.**

[![npm](https://img.shields.io/npm/v/react-native-ariel)](https://www.npmjs.com/package/react-native-ariel)
[![license](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
![platforms](https://img.shields.io/badge/platforms-iOS%20%7C%20Android%20%7C%20Web-brightgreen)

## Why Ariel?

Every other Mermaid solution for React Native requires a DOM, a WebView, or a headless browser. That means slow startup, heavy bundles, and broken native builds.

**Ariel** wraps [mermaid-rs-renderer](https://github.com/1jehuang/mermaid-rs-renderer) — a pure Rust Mermaid engine 500–2000× faster than mermaid-cli — as a React Native Turbo Module. The Rust code compiles natively for iOS and Android, and to WASM for the browser. No DOM. No WebView. Works fully offline.

| | Ariel | mermaid.js | mermaid-cli |
|---|:---:|:---:|:---:|
| iOS native | ✅ | ❌ | ❌ |
| Android native | ✅ | ❌ | ❌ |
| Web (WASM) | ✅ | ✅ | ❌ |
| No DOM | ✅ | ❌ | ❌ |
| No WebView | ✅ | ❌ | ❌ |
| Offline | ✅ | ⚠️ | ❌ |

---

## Installation

```sh
# React Native (iOS + Android)
yarn add react-native-ariel react-native-svg

# Web only (no react-native-svg needed)
yarn add react-native-ariel
```

> **Requirements:**
> - **Android:** React Native ≥ 0.73, New Architecture enabled. On RN 0.73–0.75 set `newArchEnabled=true` in `android/gradle.properties` — it is the default from RN 0.76+.
> - **iOS:** React Native ≥ 0.76. Add the following to your `Podfile` **before** the `target` block:
>   ```ruby
>   ENV['RCT_NEW_ARCH_ENABLED'] = '1'
>   ```
> - **Web:** any bundler supporting WebAssembly. **Vite note:** `uniffi-bindgen-react-native` incorrectly declares `"type": "module"` in its `package.json` while shipping CommonJS code. After `npm install`, remove that field:
>   ```sh
>   node -e "const fs=require('fs'),f='node_modules/uniffi-bindgen-react-native/package.json',p=JSON.parse(fs.readFileSync(f,'utf8'));delete p.type;fs.writeFileSync(f,JSON.stringify(p,null,2))"
>   ```
>   Then clear Vite's dep cache (`node_modules/.vite/`) and restart the dev server.

---

## Usage

### Simple render

```typescript
import { useState, useEffect } from 'react';
import { View } from 'react-native';
import { SvgXml } from 'react-native-svg';
import { renderMermaid, uniffiInitAsync } from 'react-native-ariel';

export default function DiagramView() {
  const [svg, setSvg] = useState<string | null>(null);

  useEffect(() => {
    async function run() {
      // uniffiInitAsync is a no-op on native and loads the WASM module on web.
      // Always await it before rendering so the same code works on both platforms.
      await uniffiInitAsync();
      try {
        setSvg(renderMermaid('flowchart TD\n  A[Hello] --> B[World]'));
      } catch (e) {
        console.error('Render failed:', e);
      }
    }
    run();
  }, []);

  return <View>{svg && <SvgXml xml={svg} width="100%" />}</View>;
}
```

`renderMermaid` is **synchronous** on native — no async overhead after init.

### With themes

> **Web note:** `ArielThemeRegistry` calls `ArielTheme.fromConfig()` in its constructor, which invokes the WASM module. On web you must construct the registry **after** `uniffiInitAsync()` resolves — typically inside a `useEffect`. On native this is not a concern since the module is available synchronously.

Define your theme set once using pre-shipped configs or fully custom values:

```typescript
import {
  ArielThemeRegistry,
  ArielLightTheme,
  ArielDarkTheme,
  uniffiInitAsync,
} from 'react-native-ariel';

// Option 1 — use pre-shipped themes as-is
const themes = new ArielThemeRegistry({ light: ArielLightTheme, dark: ArielDarkTheme }, 'light');

// Option 2 — extend a pre-shipped theme
const themes = new ArielThemeRegistry({
  light: ArielLightTheme,
  dark: { ...ArielDarkTheme, fontFamily: 'MyFont' },
});

// Option 3 — fully custom
const themes = new ArielThemeRegistry({
  brand: { nodeColor: '#7C3AED', arrowColor: '#A78BFA', textColor: '#F5F3FF', background: '#1E1B4B' },
});
```

Switching and rendering:

```typescript
useEffect(() => {
  async function run() {
    await uniffiInitAsync();
    themes.applyTheme('dark');
    setSvg(themes.render('flowchart TD\n  A --> B'));
  }
  run();
}, []);
```

### With theme and layout control

```typescript
import { ArielThemeRegistry, ArielLightTheme, ArielDarkTheme, renderMermaidWithOptions, uniffiInitAsync } from 'react-native-ariel';

const themes = new ArielThemeRegistry({ light: ArielLightTheme, dark: ArielDarkTheme }, 'light');

useEffect(() => {
  async function run() {
    await uniffiInitAsync();
    const svg = renderMermaidWithOptions(diagram, themes.currentTheme, { nodeSpacing: 50, rankSpacing: 80 });
    setSvg(svg);
  }
  run();
}, []);
```

### With timing metrics

```typescript
import { ArielThemeRegistry, ArielDarkTheme, uniffiInitAsync } from 'react-native-ariel';

const themes = new ArielThemeRegistry({ dark: ArielDarkTheme }, 'dark');

useEffect(() => {
  async function run() {
    await uniffiInitAsync();
    const result = themes.renderWithTiming(diagram, { nodeSpacing: 50 });
    console.log(`Total: ${result.totalMs.toFixed(2)}ms`);
    // parseUs / layoutUs / renderUs are bigint (µs); totalMs is number (ms)
    // On web, timing values are always 0 — the SVG is still correctly themed
  }
  run();
}, []);
```

### React hook — auto dark/light

```typescript
import { ArielThemeRegistry, ArielLightTheme, ArielDarkTheme, useArielTheme, renderMermaidWithOptions } from 'react-native-ariel';

const themes = new ArielThemeRegistry({ light: ArielLightTheme, dark: ArielDarkTheme }, 'light');

export default function DiagramView({ input }: { input: string }) {
  // Automatically picks 'dark' or 'light' from the registry based on system color scheme
  const theme = useArielTheme(themes);
  const [svg, setSvg] = useState<string | null>(null);

  useEffect(() => {
    async function run() {
      await uniffiInitAsync();
      setSvg(renderMermaidWithOptions(input, theme, {}));
    }
    run();
  }, [input, theme]);

  return <View>{svg && <SvgXml xml={svg} width="100%" />}</View>;
}
```

### Full pipeline (parse once, render with multiple themes)

```typescript
import {
  ArielThemeRegistry,
  ArielLightTheme,
  ArielDarkTheme,
  parseDiagram,
  computeDiagramLayout,
  renderSvgFromLayout,
  uniffiInitAsync,
} from 'react-native-ariel';

const themes = new ArielThemeRegistry({ light: ArielLightTheme, dark: ArielDarkTheme }, 'light');

useEffect(() => {
  async function run() {
    await uniffiInitAsync();
    const config = { nodeSpacing: 50, rankSpacing: 80 };
    const parsed = parseDiagram('flowchart LR\n  A --> B --> C');
    const layout = computeDiagramLayout(parsed, themes.currentTheme, config);
    setSvg(renderSvgFromLayout(layout, themes.currentTheme, config));
  }
  run();
}, []);
```

### Debug timing logs

```typescript
import { setTimingLogs } from 'react-native-ariel';

setTimingLogs(true); // off by default
// → [ariel] flowchart TD… | total 1.23ms  parse 420µs  layout 610µs  render 200µs
```

Logs are written to **Android Logcat**, the **Xcode console** on iOS, and the **browser DevTools console** on web (WASM timings are always zero).

> **Note:** When timing is on, every render call allocates a format string regardless of whether you can see the output. This is cheap but not free. The flag is intended for debugging — avoid shipping with `setTimingLogs(true)` hardcoded.

---

## API

| Function / Class | Description |
|---|---|
| `renderMermaid(input)` | Render to SVG with default theme |
| `renderMermaidWithOptions(input, theme, config)` | Render with explicit theme and layout control |
| `renderMermaidWithTiming(input, theme, config)` | Render and return SVG + timing metrics |
| `parseDiagram(input)` | Parse only — returns opaque `ArielParsedDiagram` |
| `computeDiagramLayout(parsed, theme, config)` | Layout stage — returns opaque `ArielLayout` |
| `renderSvgFromLayout(layout, theme, config)` | SVG stage from pre-computed layout |
| `setTimingLogs(enabled)` | Toggle timing output to console (default: off) |
| `uniffiInitAsync()` | Load WASM module on web; no-op on native. Call once before rendering |
| `ArielTheme.modern()` | Built-in modern theme |
| `ArielTheme.mermaidDefault()` | Built-in classic Mermaid theme |
| `ArielTheme.fromConfig(config)` | Create a theme from a partial `ArielThemeConfig` object |
| `ArielLightTheme` | Pre-shipped light theme config (use with registry or `fromConfig`) |
| `ArielDarkTheme` | Pre-shipped dark theme config (use with registry or `fromConfig`) |
| `new ArielThemeRegistry(configs, default?)` | Register named themes and manage the active selection |
| `registry.applyTheme(name)` | Switch the active theme |
| `registry.getTheme(name?)` | Get a theme by name, or the current theme if omitted |
| `registry.currentTheme` | The active `ArielThemeLike` handle |
| `registry.render(input, config?)` | Render using the active theme |
| `registry.renderWithTiming(input, config?)` | Render with timing using the active theme |
| `useArielTheme(registry?)` | React hook — auto-picks 'dark' or 'light' from registry based on system color scheme |

### Themes

#### Registry (recommended)

Define any number of named themes and switch between them at runtime:

```typescript
import { ArielThemeRegistry } from 'react-native-ariel';

const themes = new ArielThemeRegistry({
  light: { nodeColor: '#F8FAFC', textColor: '#0F172A', background: '#FFFFFF' },
  dark:  { nodeColor: '#1E293B', textColor: '#F1F5F9', background: '#0F172A' },
  ocean: { nodeColor: '#1e3a5f', arrowColor: '#0ea5e9', textColor: '#e0f2fe' },
}, 'light');

themes.applyTheme('dark');
const svg = themes.render(diagram);
```

All fields are optional — unset fields fall back to the `modern` preset. See [Theme fields](#theme-fields) for the full reference.

#### Pre-shipped theme configs

Two ready-made `ArielThemeConfig` objects are exported for use with the registry or `fromConfig`:

```typescript
import { ArielLightTheme, ArielDarkTheme, ArielThemeRegistry } from 'react-native-ariel';

// Use as-is
const themes = new ArielThemeRegistry({ light: ArielLightTheme, dark: ArielDarkTheme });

// Or spread and override just what you need
const themes = new ArielThemeRegistry({
  dark: { ...ArielDarkTheme, fontFamily: 'MyFont' },
});
```

#### Built-in opaque presets

For one-off renders without a registry:

```typescript
import { ArielTheme } from 'react-native-ariel';

ArielTheme.modern()         // clean light theme (default for renderMermaid)
ArielTheme.mermaidDefault() // classic Mermaid look
```

**`ArielTheme.modern()`** — a clean, contemporary look. Good default for light-background UIs.

**`ArielTheme.mermaidDefault()`** — reproduces the classic mermaid.js appearance. Useful when you need visual parity with existing mermaid.js output.

---

### Theme fields

All color fields accept any valid CSS color string (`#hex`, `rgb()`, `hsl()`, named colors).

#### Nodes

| Field | Affects |
|---|---|
| `nodeColor` | Fill color of all nodes (rectangles, rounded boxes, circles, etc.) |
| `secondaryColor` | Fill color of secondary / alternate node variants |
| `tertiaryColor` | Fill color of tertiary node variants |

#### Node border

| Field | Affects |
|---|---|
| `nodeBorderColor` | Border stroke of all nodes |
| `clusterBorder` | Border stroke of subgraph / cluster boxes |

#### Arrows

| Field | Affects |
|---|---|
| `arrowColor` | Stroke color of all edges and arrowheads |
| `edgeLabelBackground` | Background fill behind edge label text |

#### Text

| Field | Affects |
|---|---|
| `textColor` | Primary text color inside nodes and on labels |

#### Background

| Field | Affects |
|---|---|
| `background` | Canvas background color of the diagram. **Defaults to `transparent` when omitted** — the SVG floats on whatever background the container has. Pass an explicit color (e.g. `'#FFFFFF'`) for a solid canvas. |
| `clusterBackground` | Fill color inside subgraph / cluster boxes |

#### Sequence diagrams

| Field | Affects |
|---|---|
| `sequenceActorFill` | Fill color of actor boxes |
| `sequenceActorBorder` | Border color of actor boxes |
| `sequenceActorLine` | Color of the vertical lifeline |
| `sequenceNoteFill` | Fill color of note boxes |
| `sequenceNoteBorder` | Border color of note boxes |
| `sequenceActivationFill` | Fill color of activation bars |
| `sequenceActivationBorder` | Border color of activation bars |

#### Typography

| Field | Type | Affects |
|---|---|---|
| `fontFamily` | `string` | CSS font stack for all text. Rendered with fonts available on the device — bundle the font in your app to guarantee a specific typeface. |
| `fontSize` | `number` | Base font size in px |

### Error handling

`renderMermaid`, `renderMermaidWithOptions`, `renderMermaidWithTiming`, and `parseDiagram` throw `MermaidError.RenderError` on invalid input:

```typescript
import { renderMermaid, MermaidError } from 'react-native-ariel';

try {
  const svg = renderMermaid(input);
} catch (e) {
  if (MermaidError.instanceOf(e)) {
    console.error('Mermaid error:', e.inner.message);
  }
}
```

### `ArielLayoutConfig`

```typescript
{
  nodeSpacing?: number;
  rankSpacing?: number;
}
```

### `ArielRenderResult`

```typescript
{
  svg: string;
  parseUs: bigint;   // microseconds (u64)
  layoutUs: bigint;  // microseconds (u64)
  renderUs: bigint;  // microseconds (u64)
  totalMs: number;   // milliseconds (f64)
}
```

---

## Dev Setup

### Prerequisites

| Tool | Purpose | Install |
|---|---|---|
| [Rust](https://rustup.rs) | Compile the Rust crate | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| [Node.js](https://nodejs.org) ≥ 22 | JS toolchain | nodejs.org |
| Yarn | Package manager | `npm i -g yarn` |
| [Android Studio](https://developer.android.com/studio) | Android SDK + NDK | See below |
| Xcode ≥ 15 | iOS builds (macOS only) | Mac App Store |
| [cargo-ndk](https://github.com/bbqsrc/cargo-ndk) | Android cross-compilation | `cargo install cargo-ndk` |
| [wasm-pack](https://rustwasm.github.io/wasm-pack/) | WASM builds | `cargo install wasm-pack` |

**Android NDK:** Open Android Studio → SDK Manager → SDK Tools tab → check **NDK (Side by side)** → Apply.

**Rust targets** — run once after installing Rust:

```sh
# Android
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# iOS
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

# Web
rustup target add wasm32-unknown-unknown
```

### Clone and install

```sh
git clone https://github.com/rinfimate/ariel.git
cd ariel
yarn
```

---

## Building

### Generate TypeScript bindings

Before building for any platform you need to generate the TypeScript and C++ bindings from the Rust source. This compiles the crate for your **host OS** (not a cross-compile target) so the code-generator can introspect it.

```sh
yarn ubrn:generate
```

This works on **Windows**, **macOS**, and **Linux** and writes to `src/generated/rn/` and `cpp/`. Re-run it whenever you change the Rust API.

> **Windows note:** `ubrn build android --and-generate` tries to invoke `prettier` as a bare executable, which fails on Windows (error 193 — the Unix shell script can't run as a Win32 app). `yarn ubrn:generate` bypasses this by building the host DLL directly and passing `--no-format`.

### Android

Compiles the Rust crate for all Android ABIs and places `.a` files under `android/src/main/jniLibs/`. Run `yarn ubrn:generate` first if you haven't already.

```sh
yarn ubrn:generate   # generates src/generated/rn/ and cpp/
yarn ubrn:android    # compiles for arm64-v8a, armeabi-v7a, x86_64, x86
yarn bob build       # compiles TypeScript → lib/
```

### iOS (macOS only)

Compiles for device + simulator. Run `yarn ubrn:generate` first.

```sh
yarn ubrn:generate   # generates src/generated/rn/ and cpp/
yarn ubrn:ios        # compiles for aarch64-apple-ios, x86_64-apple-ios, aarch64-apple-ios-sim
yarn bob build       # compiles TypeScript → lib/
```

### Web (WASM)

Compiles to `wasm32-unknown-unknown` and produces a `.wasm` bundle. Run `yarn ubrn:generate` first.

```sh
yarn ubrn:generate   # generates src/generated/web/ (WASM TypeScript bindings)
yarn ubrn:web        # compiles the WASM crate with wasm-pack
yarn bob build       # compiles TypeScript → lib/ and copies WASM into lib/module/
```

### Building the npm package

To produce a publishable tarball locally (mirrors what CI does, skipping iOS):

```sh
yarn ubrn:generate
yarn ubrn:android
yarn ubrn:web
yarn bob build
npm pack --ignore-scripts
```

### Rust unit tests

```sh
cargo test --manifest-path rust/mermaid_wrapper/Cargo.toml
```

### TypeScript tests

```sh
yarn test
```

### Clean generated files

Removes all `ubrn`-generated native code so you can rebuild from scratch.

```sh
yarn ubrn:clean
```

---

## CI

On every push to `main` and on pull requests, `ci.yml` runs:

| Job | Runner | What it does |
|---|---|---|
| `test-rust` | `ubuntu-latest` | Rust unit tests |
| `build-android` | `ubuntu-latest` | Cross-compiles Rust for all Android ABIs |
| `build-ios` | `macos-latest` | Builds iOS xcframework |
| `build-web` | `ubuntu-latest` | Builds WASM bundle |
| `publish` | `ubuntu-latest` | Publishes to npm (push to main or release only) |

iOS uses GitHub's free `macos-latest` runner (unlimited minutes on public repos).

---

## License

MIT © 2026 [Rochanglien Infimate](https://github.com/rinfimate)

---

Built on [mermaid-rs-renderer](https://github.com/1jehuang/mermaid-rs-renderer) and [uniffi-bindgen-react-native](https://github.com/jhugman/uniffi-bindgen-react-native).
