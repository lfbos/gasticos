-- Create sync status enum
CREATE TYPE belvo_sync_status AS ENUM ('pending', 'in_progress', 'completed', 'failed');

-- Create belvo_sync_logs table to track sync history and errors
CREATE TABLE belvo_sync_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    link_id UUID NOT NULL REFERENCES belvo_links(id) ON DELETE CASCADE,
    status belvo_sync_status NOT NULL DEFAULT 'pending',
    transactions_fetched INTEGER DEFAULT 0,
    transactions_created INTEGER DEFAULT 0,
    transactions_updated INTEGER DEFAULT 0,
    date_from DATE,
    date_to DATE,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_belvo_sync_logs_link_id ON belvo_sync_logs(link_id);
CREATE INDEX idx_belvo_sync_logs_status ON belvo_sync_logs(status);
CREATE INDEX idx_belvo_sync_logs_started_at ON belvo_sync_logs(started_at DESC);
