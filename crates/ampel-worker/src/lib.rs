//! Ampel worker library
//!
//! This crate provides background job processing for the Ampel application.
//! It includes jobs for polling repositories, calculating health scores,
//! collecting metrics, and cleaning up stale data.

rust_i18n::i18n!("locales", fallback = "en");

pub mod jobs;
