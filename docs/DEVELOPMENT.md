# Development Setup

Moonships is a Windows-first Tauri 2 desktop application. This repository is under active construction; the starter shell is not yet connected to operational tools or persistent application data.

## Prerequisites

- Windows 11 (primary development and CI target).
- Node.js 24.x (LTS) and npm 11.x (the npm major bundled with the selected Node LTS line). `.nvmrc` declares the Node major; `package.json` engines and `.npmrc` reject unsupported Node/npm majors.
- Rust stable via rustup, including the MSVC toolchain. `rust-toolchain.toml` declares stable plus rustfmt, clippy and the Windows MSVC target.
- Microsoft C++ Build Tools and the WebView2 Runtime, as required by Tauri on Windows.

Verify the installed toolchain:

```powershell
node --version
npm --version
rustc --version
cargo --version
rustup show
```

## Install and run the frontend

From the repository root:

```powershell
npm install
npm run dev
```

The Vite server binds to `127.0.0.1:1420` and fails rather than silently selecting a different port.

## Run the desktop shell

After installing the Windows prerequisites:

```powershell
npm install
npm run tauri -- dev
```

## Standard root scripts

- `npm run dev`: start the Vite development server.
- `npm run format:check`: verify Biome formatting for frontend TypeScript/TSX, Vite config and HTML entry files.
- `npm run lint`: run Biome frontend lint rules and syntax-check the repository's Node-based sync tool.
- `npm test`: run the Node regression tests for backlog issue synchronization.
- `npm run typecheck`: run strict TypeScript project checks.
- `npm run build`: type-check and build the frontend production bundle.
- `npm run check`: run format, lint, test, typecheck and production build gates.
- `npm run tauri -- dev`: run the Tauri desktop shell.

## Checks

```powershell
npm run format:check
npm run lint
npm test
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

The Rust commands assume the Rust toolchain and Tauri's Windows native prerequisites are installed. CI will be the source of truth for clean-run verification. Do not mark a task complete merely because files exist.

## Project structure

- `src/`: React user interface.
- `src-tauri/`: Tauri configuration and Rust entry point.
- `docs/`: product, UX, security, design and implementation contracts.

## Security baseline

The webview receives only Tauri's default core permissions. Do not add filesystem, shell, process, network, or plugin permissions without a scoped task and security review. UI status in this starter screen is descriptive only; it does not imply operational checks are running.
