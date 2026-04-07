-- Create link status enum
CREATE TYPE belvo_link_status AS ENUM ('valid', 'invalid', 'token_required', 'unconfirmed');

-- Create belvo_links table to store bank connections via Belvo
CREATE TABLE belvo_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    belvo_link_id UUID NOT NULL UNIQUE,
    institution VARCHAR(100) NOT NULL,
    institution_name VARCHAR(255) NOT NULL,
    access_mode VARCHAR(50) NOT NULL DEFAULT 'recurrent',
    status belvo_link_status NOT NULL DEFAULT 'valid',
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_belvo_links_user_id ON belvo_links(user_id);
CREATE INDEX idx_belvo_links_belvo_link_id ON belvo_links(belvo_link_id);
CREATE INDEX idx_belvo_links_status ON belvo_links(status);
