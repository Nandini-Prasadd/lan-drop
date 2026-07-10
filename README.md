# lan-drop

`lan-drop` is a privacy-focused desktop app for encrypted, direct file sharing between paired devices on the same local network.

The project uses Rust and Tauri for native capabilities, with a Svelte and TypeScript interface. It deliberately has no cloud backend, account system, or telemetry.

## Development

Install Node.js, Rust, and the platform prerequisites listed by Tauri. Then run:

```powershell
npm install
npm run tauri dev
```

Useful validation commands:

```powershell
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

Feature, security, architecture, and troubleshooting documentation will grow with the implementation.
