-- Fix unique constraint to include statement_id
-- This allows legitimate duplicate transactions (same amount/date/description)
-- from different statements, while preventing re-upload of same file
DROP INDEX IF EXISTS idx_transactions_unique;

CREATE UNIQUE INDEX idx_transactions_unique
ON transactions(user_id, statement_id, date, description, amount);
