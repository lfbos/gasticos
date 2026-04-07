-- Add Belvo-related columns to transactions table
-- Note: Cannot add UNIQUE constraint on belvo_transaction_id since table is partitioned by date
-- Deduplication will be handled at the application level
ALTER TABLE transactions
    ADD COLUMN belvo_transaction_id UUID,
    ADD COLUMN belvo_account_id UUID REFERENCES belvo_accounts(id) ON DELETE SET NULL;

-- Indexes for efficient lookups (uniqueness enforced in application)
CREATE INDEX idx_transactions_belvo_transaction_id ON transactions(belvo_transaction_id) WHERE belvo_transaction_id IS NOT NULL;
CREATE INDEX idx_transactions_belvo_account_id ON transactions(belvo_account_id) WHERE belvo_account_id IS NOT NULL;
