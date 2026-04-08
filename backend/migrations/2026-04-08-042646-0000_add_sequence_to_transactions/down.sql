-- Remove sequence column and revert unique constraint
DROP INDEX IF EXISTS idx_transactions_unique;

ALTER TABLE transactions DROP COLUMN sequence;

CREATE UNIQUE INDEX idx_transactions_unique
ON transactions(user_id, statement_id, date, description, amount);
