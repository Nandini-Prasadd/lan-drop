# lan-drop

Private desktop file sharing for devices on the same local network. `lan-drop` uses a Rust/Tauri backend and a Svelte/TypeScript interface. It has no account system, cloud relay, telemetry, or hard-coded credentials.

## Features

- QR and expiring short-code pairing invitations
- Local UDP discovery with a manual `IP:port` fallback
- Noise XX authenticated sessions using X25519, ChaChaPoly, and SHA-256 through the maintained `snow` library
- Bounded file hashing and SHA-256 integrity verification before a received file is published
- Safe receive writes: path-traversal rejection, temporary files, integrity checks, and no silent overwrite
- Accessible file queue, cancellation, retryable failure messaging, transfer history, and privacy settings
- SQLite-only local settings, identity, and transfer history

## Architecture

```text
Svelte UI → Tauri commands → Rust domain/storage/network modules
                         ├─ SQLite: settings, identity, history
                         ├─ UDP: local discovery
                         ├─ Noise: authenticated encrypted session primitives
                         └─ Streaming SHA-256: transfer integrity
```

The backend is deliberately modular: `domain` validates metadata, `pairing` creates invitations, `discovery` handles LAN advertisements, `session` wraps Noise, `transfer` hashes streams, `receiver` publishes verified files, and `storage` owns SQLite persistence.

## Local development

Install Node.js 22+, Rust stable, and Tauri's [platform prerequisites](https://v2.tauri.app/start/prerequisites/). On Windows, the MSVC C++ Build Tools workload is required for native Rust linking.

```powershell
npm install
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

The GitHub workflow runs the frontend checks, Rust formatting, Rust tests, and production frontend build on Ubuntu. See [the two-instance test flow](docs/LOCAL_TWO_INSTANCE_TEST.md) for manual local-network validation.

## Security model and limitations

- Pairing invitations contain only an expiring short code, device name, and public key—never file contents or private keys.
- Noise session primitives come from `snow`; `lan-drop` does not implement cryptographic primitives itself.
- All application state stays on the local device. The app does not provide a cloud backup, relay, remote-access feature, or telemetry.
- Local discovery is broadcast on the current LAN. Treat pairing codes and unknown LAN peers carefully, especially on shared networks.
- SQLite identity material is local but is not hardware-backed key storage. A device compromise can expose local application data.
- The current UI contains queue and pairing workflows; production transfer orchestration should be exercised with the documented two-instance flow before relying on it for irreplaceable files.

## Troubleshooting

- **Cargo cannot link on Windows:** install Visual Studio Build Tools with the C++ workload, then restart the terminal.
- **No peers appear:** enable local discovery in Settings, allow the app through the firewall, and use the manual `IP:port` fallback.
- **Pairing code expired:** create a fresh QR invitation; codes expire after five minutes.
- **Receive failed verification:** the temporary file is removed. Retry after confirming both devices are paired and connected to the same LAN.
- **Saved file already exists:** choose a new filename or directory; lan-drop does not overwrite existing files silently.

## Development history

The repository was built through 15 tracked issues and 15 focused pull requests covering the workspace, validation, SQLite, pairing, discovery, crypto sessions, integrity, UI, settings, testing, and release hygiene. See the closed issues and merged pull requests for the implementation record.

## License

MIT. See [LICENSE](LICENSE).
