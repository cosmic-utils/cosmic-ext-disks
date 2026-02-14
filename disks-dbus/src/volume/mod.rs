//! Volume hierarchy types
//!
//! This module provides types for representing volume hierarchies:
//! - VolumeNode (tree structure)
 //! - BlockIndex (device tracking)

pub mod node;

pub use node::{VolumeNode, BlockIndex};

