use crate::error::AppError;
use llm::builder::{LLMBackend, LLMBuilder};

#[derive(Debug, Clone)]
pub struct EmbeddingService {
    api_key: String,
    model: String,
}

impl EmbeddingService {
    pub fn new() -> Result<Self, AppError> {
        let api_key = std::env::var("GEMINI_API_KEY").map_err(|_| {
            AppError::Internal("GEMINI_API_KEY environment variable not set".to_string())
        })?;

        let model = std::env::var("GEMINI_EMBEDDING_MODEL")
            .unwrap_or_else(|_| "text-embedding-004".to_string());

        Ok(Self { api_key, model })
    }

    pub async fn generate_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, AppError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Build LLM for each request (since we can't store it due to Clone issues)
        let llm = LLMBuilder::new()
            .backend(LLMBackend::Google)
            .api_key(&self.api_key)
            .model(&self.model)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to initialize Gemini LLM: {}", e)))?;

        // Use the llm crate to generate embeddings
        let embeddings = llm
            .embed(texts)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to generate embeddings: {}", e)))?;

        Ok(embeddings)
    }

    pub async fn generate_embedding(&self, text: String) -> Result<Vec<f32>, AppError> {
        let embeddings = self.generate_embeddings(vec![text]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("No embedding returned".to_string()))
    }
}
