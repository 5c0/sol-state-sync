use borsh::{BorshDeserialize, BorshSerialize};
use dashmap::DashMap;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info, warn};

#[derive(Error, Debug)]
pub enum NexusError {
    #[error("Base64 decoding failed: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("Borsh deserialization failed: {0}")]
    Borsh(std::io::Error),
    #[error("Stale data update: current slot {current}, update slot {update}")]
    StaleData { current: u64, update: u64 },
    #[error("Missing symbol state for {0}")]
    MissingSymbol(String),
}

impl From<NexusError> for PyErr {
    fn from(err: NexusError) -> Self {
        pyo3::exceptions::PyRuntimeError::new_err(err.to_string())
    }
}

/// A single price level in the orderbook.
/// Note: Real Phoenix/Drift implementations often use fixed-point u64. 
/// Using f64 for Python-layer interoperability.
#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct Level {
    pub price: f64,
    pub size: f64,
}

/// Reconstructed Orderbook state from Solana account data.
#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct Orderbook {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub slot: u64,
    pub last_updated_ts: u64,
}

#[pyclass]
pub struct SolanaStateSyncer {
    /// Concurrent state storage using DashMap to minimize lock-contention.
    state: Arc<DashMap<String, Orderbook>>,
}

#[pymethods]
impl SolanaStateSyncer {
    #[new]
    pub fn new() -> Self {
        Self {
            state: Arc::new(DashMap::new()),
        }
    }

    /// High-performance account reconstruction logic.
    /// Handles decoding, sequence validation, and lock-free memory updates.
    pub fn update_from_raw(&self, symbol: String, raw_base64: String, slot: u64) -> PyResult<()> {
        // 1. Decode & Deserialize
        let bytes = base64::decode(raw_base64).map_err(NexusError::Base64)?;
        
        // Use generic try_from_slice. In production, we'd map to specific program IDL.
        let mut book = Orderbook::try_from_slice(&bytes)
            .map_err(NexusError::Borsh)?;

        // 2. Slot Verification (Crucial for HFT: Never update with stale data)
        if let Some(current) = self.state.get(&symbol) {
            if slot <= current.slot {
                return Err(NexusError::StaleData { 
                    current: current.slot, 
                    update: slot 
                }.into());
            }
        }

        // 3. Update state
        book.slot = slot;
        book.last_updated_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.state.insert(symbol, book);
        Ok(())
    }

    /// Sub-microsecond read from the concurrent state map.
    pub fn get_book(&self, symbol: String) -> PyResult<Option<Orderbook>> {
        Ok(self.state.get(&symbol).map(|r| r.clone()))
    }

    /// Fast-path for Best Bid/Offer (BBO) extraction.
    pub fn get_bbo(&self, symbol: String) -> PyResult<Option<(f64, f64)>> {
        match self.state.get(&symbol) {
            Some(book) => {
                let bid = book.bids.first().map(|l| l.price).unwrap_or(0.0);
                let ask = book.asks.first().map(|l| l.price).unwrap_or(0.0);
                Ok(Some((bid, ask)))
            },
            None => Ok(None)
        }
    }

    /// Health check for the syncer.
    pub fn get_metrics(&self) -> PyResult<std::collections::HashMap<String, u64>> {
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("total_symbols".to_string(), self.state.len() as u64);
        Ok(metrics)
    }
}

#[pymodule]
fn sol_state(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<SolanaStateSyncer>()?;
    Ok(())
}
