-- Remove Transfers system category
DELETE FROM categories WHERE key = 'transfers' AND is_system = true;
