//! Python iterator for WebSocket messages
//!
//! Provides Python iterator protocol (__iter__ and __next__) for streaming messages.
//! Uses blocking receive with optional timeout for message consumption.

use pyo3::prelude::*;
use std::sync::Arc;
use std::time::Duration;

use crate::websocket::message_to_dict;

/// Python iterator for WebSocket messages
///
/// Implements Python's iterator protocol (__iter__ and __next__).
/// Bridges marketdata_core::MessageReceiver to Python iteration.
///
/// # Example (Python)
///
/// ```python
/// for msg in ws.stock.messages():
///     print(msg)
///
/// # With timeout (returns None on timeout instead of blocking forever)
/// for msg in ws.stock.messages(timeout_ms=1000):
///     if msg is None:
///         print("Timeout, no message received")
///         continue
///     print(msg)
/// ```
///
/// # Note
///
/// The `unsendable` attribute is required because `MessageReceiver` contains
/// `std::sync::mpsc::Receiver` which is not `Sync`. This means the iterator
/// can only be used from the thread that created it.
#[pyclass(unsendable)]
pub struct MessageIterator {
    receiver: Arc<marketdata_core::MessageReceiver>,
    timeout: Option<Duration>,
}

impl MessageIterator {
    /// Create a new message iterator
    pub fn new(
        receiver: Arc<marketdata_core::MessageReceiver>,
        timeout: Option<Duration>,
    ) -> Self {
        Self { receiver, timeout }
    }
}

#[pymethods]
impl MessageIterator {
    /// Return self as iterator (required for Python iteration protocol)
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Get next message from stream
    ///
    /// Returns:
    ///     dict: Message data containing event, channel, symbol, data fields
    ///     None: If timeout specified and no message received within timeout
    ///
    /// Raises:
    ///     StopIteration: When channel is closed (connection disconnected)
    ///
    /// Note: This method blocks the current thread while waiting for messages.
    /// The GIL is NOT released during blocking because MessageReceiver is not Sync.
    /// For long-running message consumption, consider using a separate thread
    /// or async patterns in Python.
    fn __next__(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        // Note: We cannot use py.allow_threads() here because MessageReceiver
        // contains std::sync::mpsc::Receiver which is not Sync.
        // The blocking call happens with GIL held, which may impact Python
        // responsiveness. For production use, consider using Python's
        // concurrent.futures or asyncio patterns.

        let result = if let Some(timeout) = self.timeout {
            self.receiver.receive_timeout(timeout)
        } else {
            self.receiver.receive().map(Some)
        };

        match result {
            Ok(Some(msg)) => {
                let dict = message_to_dict(py, &msg)?;
                Ok(Some(dict.into_any()))
            }
            Ok(None) => {
                // Timeout with no message - return None but don't stop iteration
                Ok(None)
            }
            Err(_) => {
                // Channel closed, stop iteration
                Err(pyo3::exceptions::PyStopIteration::new_err(
                    "Message channel closed",
                ))
            }
        }
    }

    /// Try to receive a message without blocking
    ///
    /// Returns:
    ///     dict: Message data if available
    ///     None: If no message available
    fn try_recv(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        match self.receiver.try_receive() {
            Some(msg) => {
                let dict = message_to_dict(py, &msg)?;
                Ok(Some(dict.into_any()))
            }
            None => Ok(None),
        }
    }

    /// Receive a message with timeout
    ///
    /// Args:
    ///     timeout_ms: Timeout in milliseconds
    ///
    /// Returns:
    ///     dict: Message data if received within timeout
    ///     None: If timeout elapsed with no message
    ///
    /// Raises:
    ///     MarketDataError: If channel is closed
    fn recv_timeout(&self, py: Python<'_>, timeout_ms: u64) -> PyResult<Option<Py<PyAny>>> {
        let timeout = Duration::from_millis(timeout_ms);
        match self.receiver.receive_timeout(timeout) {
            Ok(Some(msg)) => {
                let dict = message_to_dict(py, &msg)?;
                Ok(Some(dict.into_any()))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(crate::errors::to_py_err(e)),
        }
    }
}

// Note: Tests for MessageIterator require Python runtime and must be run via maturin develop + pytest
// See test_websocket.py for Python-side iterator tests
