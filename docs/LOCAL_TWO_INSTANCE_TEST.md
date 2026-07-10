# Local two-instance transfer test

Use two computers on the same trusted LAN, or two desktop sessions with routed local networking.

1. Run `npm install` and `npm run tauri dev` on both devices.
2. In **Settings**, give each device a distinctive name, choose an empty download directory, and enable discovery.
3. On device A, open **Pair a device**, create a QR invitation, and scan it on device B. If discovery is unavailable, validate and use A's shown `IP:port` address instead.
4. Confirm that the short code matches on both devices before trusting the peer.
5. Queue a non-empty test file on A. Confirm its name and size are shown, then cancel it once to verify the cancellation UI.
6. Re-queue the file and send it through the authenticated session. The receiver must only show completion after integrity verification.
7. Verify the output is inside B's configured download directory, has the expected SHA-256 digest, and appears in B's local history.
8. Repeat with a modified payload or mismatched digest and verify that the receive operation fails without publishing a final file.

Expected privacy behavior: no account, cloud relay, telemetry, or file contents leave the local network.
