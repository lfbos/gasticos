-- Add Transfers system category
INSERT INTO categories (id, user_id, name, key, icon, color, is_system, created_at)
VALUES (
    gen_random_uuid(),
    NULL,
    'Transferencias',
    'transfers',
    'arrow-right-left',
    '#8B5CF6',
    true,
    NOW()
);
