-- Revert to original unique constraint without statement_id
DROP INDEX IF EXISTS idx_transactions_unique;

CREATE UNIQUE INDEX idx_transactions_unique
ON transactions(user_id, date, description, amount);
