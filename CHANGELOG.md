# Changelog

## [0.1.0-alpha] - 2026-03-10
### Added
- Initial core implementation of the `SolanaStateSyncer` in Rust.
- Support for raw Base64-to-Borsh account reconstruction.
- Concurrent state management via `DashMap` to bypass Python's GIL.
- Python FFI bindings using `PyO3` for sub-microsecond state lookups.
- Slot-tracking and sequence validation logic to prevent stale data updates.

### Technical Specifications
- **Runtime:** Tokio (Async)
- **Serialization:** Borsh 0.10
- **Concurrency:** Lock-free HashMaps
- **FFI:** Python 3.8+ compatible
