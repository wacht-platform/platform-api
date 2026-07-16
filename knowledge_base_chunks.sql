CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS knowledge_base_chunks (
    id BIGSERIAL PRIMARY KEY,
    knowledge_base_id BIGINT NOT NULL,
    document_id BIGINT NOT NULL,
    chunk_index INTEGER NOT NULL,
    path TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    content TEXT NOT NULL,
    embedding vector(1536),
    embedding_768 vector(768),
    search_vector TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', content)) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT knowledge_base_chunks_document_chunk_key UNIQUE (document_id, chunk_index),
    CONSTRAINT knowledge_base_chunks_knowledge_base_fk
        FOREIGN KEY (knowledge_base_id) REFERENCES ai_knowledge_bases(id) ON DELETE CASCADE,
    CONSTRAINT knowledge_base_chunks_document_fk
        FOREIGN KEY (document_id) REFERENCES ai_knowledge_base_documents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS knowledge_base_chunks_knowledge_base_idx
    ON knowledge_base_chunks (knowledge_base_id);

CREATE INDEX IF NOT EXISTS knowledge_base_chunks_document_idx
    ON knowledge_base_chunks (document_id, chunk_index);

CREATE INDEX IF NOT EXISTS knowledge_base_chunks_search_vector_idx
    ON knowledge_base_chunks USING GIN (search_vector);

CREATE INDEX IF NOT EXISTS knowledge_base_chunks_embedding_hnsw_idx
    ON knowledge_base_chunks USING hnsw (embedding vector_cosine_ops)
    WHERE embedding IS NOT NULL;

CREATE INDEX IF NOT EXISTS knowledge_base_chunks_embedding_768_hnsw_idx
    ON knowledge_base_chunks USING hnsw (embedding_768 vector_cosine_ops)
    WHERE embedding_768 IS NOT NULL;
