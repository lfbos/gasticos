-- Add unique constraint to prevent duplicate transactions
-- A transaction is unique per user, date, description, and amount
CREATE UNIQUE INDEX idx_transactions_unique
ON transactions(user_id, date, description, amount);
