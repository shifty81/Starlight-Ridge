use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StableId(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("content validation failed: {0}")]
    Validation(String),
}
