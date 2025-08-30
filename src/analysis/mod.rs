//! Analysis module for ccusage-rs
//! 
//! This module provides cost calculation, usage analysis, and statistical
//! functions for processing usage data.

pub mod calculator;
pub mod statistics;
pub mod trends;
pub mod insights;

pub use calculator::*;
pub use statistics::*;
pub use trends::*;
pub use insights::*;