//! Ampel worker library
//!
//! This crate provides background job processing for the Ampel application.
//! It includes jobs for polling repositories, calculating health scores,
//! collecting metrics, and cleaning up stale data.

pub mod jobs;
