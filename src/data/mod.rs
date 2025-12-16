//! Data reading and representation.
//!
//! This module handles reading NetCDF files and representing their structure
//! as a tree of nodes.

mod dataset;
mod node;
mod reader;
mod variable_data;

pub use dataset::DatasetInfo;
pub use node::{DataNode, NodeType};
pub use reader::DataReader;
pub use variable_data::{read_variable, LoadedVariable};
