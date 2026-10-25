# Testing and Release Gates

## Local checks

Run the supported toolchain from the repository root:

```sh
npm install
npm run format:check
npm run lint
npm run test:repo
npm test
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Windows installer generation must be validated on Windows because Tauri uses the Windows MSVC toolchain and Windows bundlers.

## CI expectations

- Frontend component smoke tests run separately from repository backlog-sync regression tests.
- Rust unit tests run as a separate job from Rust formatting and Clippy.
- The Windows packaging job must produce both an MSI and an NSIS setup executable. A successful bare executable build alone is not an installer release.
- Release artifacts are previews until core application features and manual installation/runtime checks are completed.

## Alpha release scope

The initial desktop shell is a foundation preview. Do not describe planned or disconnected features as operational. A release must identify its version, platform, installer formats, and known limitations.
