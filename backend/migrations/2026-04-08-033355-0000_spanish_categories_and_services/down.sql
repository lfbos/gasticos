-- Revert category names to English
UPDATE categories SET name = 'Income' WHERE key = 'income' AND is_system = true;
UPDATE categories SET name = 'Housing' WHERE key = 'housing' AND is_system = true;
UPDATE categories SET name = 'Groceries' WHERE key = 'groceries' AND is_system = true;
UPDATE categories SET name = 'Restaurants' WHERE key = 'restaurants' AND is_system = true;
UPDATE categories SET name = 'Transportation' WHERE key = 'transportation' AND is_system = true;
UPDATE categories SET name = 'Health' WHERE key = 'health' AND is_system = true;
UPDATE categories SET name = 'Education' WHERE key = 'education' AND is_system = true;
UPDATE categories SET name = 'Entertainment' WHERE key = 'entertainment' AND is_system = true;
UPDATE categories SET name = 'Clothing' WHERE key = 'clothing' AND is_system = true;
UPDATE categories SET name = 'Financial' WHERE key = 'financial' AND is_system = true;
UPDATE categories SET name = 'Technology' WHERE key = 'technology' AND is_system = true;
UPDATE categories SET name = 'Subscriptions' WHERE key = 'subscriptions' AND is_system = true;
UPDATE categories SET name = 'Other' WHERE key = 'other' AND is_system = true;

-- Remove Services category
DELETE FROM categories WHERE key = 'services' AND is_system = true;
