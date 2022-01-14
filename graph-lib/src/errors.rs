use std::io;
use thiserror::Error;
use super::*;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("vertex with id \"{0}\" already exists in the graph")]
    VertexAlreadyExist(DefaultGraphIdType),
    #[error("vertex id \"{0}\" not found in graph")]
    VertexNotFound(DefaultGraphIdType),
    #[error("{0}")]
    SerializeGraph(#[from] io::Error),
    #[error("vertex id in \"{0}\" not set")]
    ParseVertexId(String),
    #[error("wrong vertex id type in \"{0}\"")]
    WrongVertexIdType(String),
}