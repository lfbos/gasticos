-- Create statement status enum
CREATE TYPE statement_status AS ENUM ('pending', 'processing', 'completed', 'failed');

-- Create statements table
CREATE TABLE statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bank VARCHAR(50) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500),
    file_size BIGINT NOT NULL DEFAULT 0,
    status statement_status NOT NULL DEFAULT 'pending',
    transaction_count INTEGER,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

CREATE INDEX idx_statements_user_id ON statements(user_id);
CREATE INDEX idx_statements_status ON statements(status);
CREATE INDEX idx_statements_created_at ON statements(created_at DESC);
