//! Shared types and helpers for Agent Toast — used by both the desktop app
//! and the remote `agent-toast-send` CLI.

pub mod hook_config;
pub mod wire;

pub use wire::NotifyRequest;
