//! Volume hierarchy types
//!
//! This module provides types for representing volume hierarchies:
//! - VolumeNode (tree structure, internal only)
//! - BlockIndex (device tracking)

pub(crate) mod node;

// BlockIndex is still needed publicly for device lookups
pub use node::BlockIndex;

