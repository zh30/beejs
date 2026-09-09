//! Beejs AI Engine - Native embeddings, tensor ops, and local inference

pub mod embedding;

pub use embedding::{
    cosine_similarity, embed_batch, embed_text, EmbedOptions, DEFAULT_EMBEDDING_DIM,
};
