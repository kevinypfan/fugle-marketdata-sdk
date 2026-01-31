---
created: 2026-01-31T14:30
title: Fix WebSocket client shutdown blocking
area: uniffi
files:
  - uniffi/src/websocket.rs
  - bindings/go/marketdata/channel_wrapper.go
---

## Problem

WebSocket client's `Close()` / `Disconnect()` blocks indefinitely when called, preventing graceful shutdown. This affects the Go binding where `client.Close()` hangs after receiving Ctrl+C signal.

Current workaround in Go examples uses a 3-second timeout with `os.Exit(0)` to force termination.

Root cause is likely in `uniffi/src/websocket.rs` - the shutdown signal (`AtomicBool`) may not be properly terminating the message loop, or there's a blocking call that doesn't respect the shutdown flag.

Symptoms:
- Go: `StreamingClient.Close()` hangs
- Ctrl+C doesn't cleanly exit without timeout workaround

## Solution

TBD - Investigate:
1. Check if `shutdown.store(true, Ordering::SeqCst)` is being called in `disconnect()`
2. Verify `receive_timeout` loop checks shutdown flag before blocking again
3. May need to close the underlying channel/connection to unblock the receiver
