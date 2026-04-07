-- Create account type enum
CREATE TYPE belvo_account_type AS ENUM ('checking', 'savings', 'credit_card', 'loan', 'other');

-- Create belvo_accounts table to store bank accounts from Belvo
CREATE TABLE belvo_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    link_id UUID NOT NULL REFERENCES belvo_links(id) ON DELETE CASCADE,
    belvo_account_id UUID NOT NULL UNIQUE,
    name VARCHAR(255),
    number_masked VARCHAR(50),
    type belvo_account_type NOT NULL DEFAULT 'other',
    currency VARCHAR(10) NOT NULL DEFAULT 'COP',
    balance_current NUMERIC(15, 2),
    balance_available NUMERIC(15, 2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_belvo_accounts_link_id ON belvo_accounts(link_id);
CREATE INDEX idx_belvo_accounts_belvo_account_id ON belvo_accounts(belvo_account_id);
