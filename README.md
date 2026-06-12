# react-native-ariel-rs

> **Mermaid diagrams for React Native and React Native Web. No DOM. No WebView. Pure Rust.**

[![npm](https://img.shields.io/npm/v/react-native-ariel-rs)](https://www.npmjs.com/package/react-native-ariel-rs)
[![license](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
![platforms](https://img.shields.io/badge/platforms-iOS%20%7C%20Android%20%7C%20Web-brightgreen)

## Why react-native-ariel-rs?

Every other Mermaid solution for React Native requires a DOM, a WebView, or a headless browser. That means slow startup, heavy bundles, and broken native builds.

**react-native-ariel-rs** wraps [ariel-rs](https://github.com/rinfimate/ariel-rs) — a pure Rust Mermaid engine — as a React Native Turbo Module. The Rust code compiles natively for iOS and Android, and to WASM for the browser. No DOM. No WebView. Works fully offline.

| | react-native-ariel-rs | mermaid.js | mermaid-cli |
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
yarn add react-native-ariel-rs react-native-svg

# Web only (no react-native-svg needed)
yarn add react-native-ariel-rs
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
import { renderMermaid, uniffiInitAsync } from 'react-native-ariel-rs';

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

### With a theme

Pass one of the four built-in themes to `renderMermaidWithTheme`:

```typescript
import { renderMermaidWithTheme, ArielTheme, uniffiInitAsync } from 'react-native-ariel-rs';

useEffect(() => {
  async function run() {
    await uniffiInitAsync();
    const svg = renderMermaidWithTheme(
      'flowchart TD\n  A --> B',
      ArielTheme.dark(),
    );
    setSvg(svg);
  }
  run();
}, []);
```

Available themes: `ArielTheme.light()`, `ArielTheme.dark()`, `ArielTheme.forest()`, `ArielTheme.neutral()`.

### React hook — auto dark/light

`useArielTheme` reads the system color scheme and returns `ArielTheme.dark()` or `ArielTheme.light()` automatically:

```typescript
import { renderMermaidWithTheme, useArielTheme, uniffiInitAsync } from 'react-native-ariel-rs';

export default function DiagramView({ input }: { input: string }) {
  const theme = useArielTheme();
  const [svg, setSvg] = useState<string | null>(null);

  useEffect(() => {
    async function run() {
      await uniffiInitAsync();
      setSvg(renderMermaidWithTheme(input, theme));
    }
    run();
  }, [input, theme]);

  return <View>{svg && <SvgXml xml={svg} width="100%" />}</View>;
}
```

### Error handling

Both render functions throw `MermaidError` on unrecognized input:

```typescript
import { renderMermaid, MermaidError } from 'react-native-ariel-rs';

try {
  const svg = renderMermaid(input);
} catch (e) {
  if (MermaidError.instanceOf(e)) {
    console.error('Mermaid error:', e.inner.message);
  }
}
```

---

## API

| Function / Class | Description |
|---|---|
| `renderMermaid(input)` | Render to SVG with the default light theme |
| `renderMermaidWithTheme(input, theme)` | Render with an explicit theme |
| `uniffiInitAsync()` | Load WASM module on web; no-op on native. Call once before rendering |
| `ArielTheme.light()` | Standard Mermaid light theme |
| `ArielTheme.dark()` | Dark theme for dark-mode UIs |
| `ArielTheme.forest()` | Forest/green-tinted light theme |
| `ArielTheme.neutral()` | Neutral greyscale theme |
| `useArielTheme()` | React hook — returns `dark` or `light` theme based on system color scheme |

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
git clone https://github.com/rinfimate/react-native-ariel-rs.git
cd react-native-ariel-rs
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

Built on [ariel-rs](https://github.com/rinfimate/ariel-rs) and [uniffi-bindgen-react-native](https://github.com/jhugman/uniffi-bindgen-react-native).
