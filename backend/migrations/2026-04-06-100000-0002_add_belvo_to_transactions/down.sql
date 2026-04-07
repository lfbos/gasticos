DROP INDEX IF EXISTS idx_transactions_belvo_account_id;
DROP INDEX IF EXISTS idx_transactions_belvo_transaction_id;

ALTER TABLE transactions
    DROP COLUMN IF EXISTS belvo_account_id,
    DROP COLUMN IF EXISTS belvo_transaction_id;
