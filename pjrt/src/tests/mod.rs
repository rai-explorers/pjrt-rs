//! Integration and unit tests for the pjrt crate.
//!
//! This module contains tests that are organized into submodules:
//! - `async_transfer_tests`: Unit tests for async transfer types (no plugin required)
//! - `buffer_ref_count`: Tests for buffer reference counting
//! - `core_types_tests`: Unit tests for core types (no plugin required)
//! - `event_tests`: Unit tests for event module (no plugin required)
//! - `executable_tests`: Unit tests for executable module (no plugin required)
//! - `execute_tests`: Unit tests for execute module (no plugin required)
//! - `extension_tests`: Tests for extension discovery and usage
//! - `memory_tests`: Unit tests for memory module (no plugin required)

mod async_transfer_tests;
mod buffer_ref_count;
mod core_types_tests;
mod event_tests;
mod executable_tests;
mod execute_tests;
mod extension_tests;
mod memory_tests;
