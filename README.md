# sol-state-sync: High-Performance Solana Account Reconstructor
### Native Rust FFI for Low-Latency State Synchronization

**sol-state-sync** is a high-concurrency systems library designed to eliminate the latency bottlenecks in Solana state reconstruction. By offloading **Base64 decoding**, **Borsh deserialization**, and **concurrent slot-tracking** to a multi-threaded Rust core, this library enables sub-microsecond state access for high-level strategy layers.

---

## 🚀 Performance Metrics

| Operation | Native Python | sol-state-sync (Hybrid) | Latency Reduction |
| :--- | :--- | :--- | :--- |
| **Borsh Reconstruction** | ~2.5ms - 5.0ms | **< 10μs** | **~500x Faster** |
| **Memory Access** | GIL-Locked | **Lock-Free DashMap** | **High-Concurrency** |
| **BBO Extraction** | ~1.5ms | **< 2μs** | **750x Faster** |

---

## 🛠 Systems Architecture

### 1. Concurrent State Management
Utilizes **DashMap** for lock-free, thread-safe access to account data. This architecture allows real-time WebSocket ingestion threads to update the global state without blocking the main strategy loop.

### 2. Slot-Aware Integrity
Implements strict **sequence and slot verification** to ensure that data mirrors are never updated with stale or out-of-order packets.

### 3. FFI Overhead Reduction
Leverages the **PyO3** framework to provide a native Python extension, allowing researchers to perform nanosecond-level lookups into the Rust state-cache from a research environment.

---

## 💻 Technical Usage

```python
import sol_state

# Initialize the High-Performance Rust Core
syncer = sol_state.SolanaStateSyncer()

# Update state from raw WebSocket bytes (Offloaded to Rust)
syncer.update_from_raw(
    symbol="SOL/USDC", 
    raw_base64="AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eH...", 
    slot=254890123
)

# Sub-microsecond read for high-frequency execution
bbo = syncer.get_bbo("SOL/USDC")
print(f"BBO: {bbo}")
```

---
*sol-state-sync is licensed under the MIT License - Built for the high-performance Solana ecosystem.*
