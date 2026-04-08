-- Add sequence number to transactions for deduplication
-- This distinguishes identical transactions within the same statement
ALTER TABLE transactions ADD COLUMN sequence INTEGER NOT NULL DEFAULT 0;

-- Update unique constraint to include sequence
DROP INDEX IF EXISTS idx_transactions_unique;

CREATE UNIQUE INDEX idx_transactions_unique
ON transactions(user_id, statement_id, date, description, amount, sequence);
