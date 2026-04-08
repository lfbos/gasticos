-- Remove income category
DELETE FROM categories WHERE key = 'income' AND is_system = true;
