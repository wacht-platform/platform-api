CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS agent_memories (
    id BIGINT PRIMARY KEY,
    deployment_id BIGINT NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    actor_id BIGINT,
    project_id BIGINT,
    thread_id BIGINT,
    execution_run_id BIGINT,
    owner_agent_id BIGINT,
    recorded_by_agent_id BIGINT,
    memory_scope TEXT NOT NULL CHECK (memory_scope IN ('actor', 'project', 'thread')),
    content TEXT NOT NULL,
    embedding vector NOT NULL,
    memory_category TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    search_vector tsvector GENERATED ALWAYS AS (to_tsvector('english', content)) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT agent_memories_actor_scope CHECK (memory_scope <> 'actor' OR actor_id IS NOT NULL),
    CONSTRAINT agent_memories_project_scope CHECK (memory_scope <> 'project' OR project_id IS NOT NULL),
    CONSTRAINT agent_memories_thread_scope CHECK (memory_scope <> 'thread' OR thread_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS agent_memories_deployment_created_idx
    ON agent_memories (deployment_id, created_at DESC);

CREATE INDEX IF NOT EXISTS agent_memories_actor_scope_idx
    ON agent_memories (deployment_id, actor_id, memory_scope, created_at DESC)
    WHERE actor_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS agent_memories_project_scope_idx
    ON agent_memories (deployment_id, project_id, memory_scope, created_at DESC)
    WHERE project_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS agent_memories_thread_scope_idx
    ON agent_memories (deployment_id, thread_id, memory_scope, created_at DESC)
    WHERE thread_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS agent_memories_category_idx
    ON agent_memories (deployment_id, memory_category);

CREATE INDEX IF NOT EXISTS agent_memories_search_vector_idx
    ON agent_memories USING GIN (search_vector);
