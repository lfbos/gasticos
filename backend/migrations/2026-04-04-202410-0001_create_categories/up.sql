-- Create categories table
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    key VARCHAR(50),
    icon VARCHAR(50),
    color VARCHAR(7),
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_category_name UNIQUE (user_id, name),
    CONSTRAINT unique_system_key UNIQUE (key)
);

CREATE INDEX idx_categories_user_id ON categories(user_id);

-- Seed system categories
INSERT INTO categories (key, name, icon, color, is_system) VALUES
    ('housing', 'Housing', 'home', '#8B5CF6', true),
    ('groceries', 'Groceries', 'shopping-cart', '#10B981', true),
    ('restaurants', 'Restaurants', 'utensils', '#F59E0B', true),
    ('transportation', 'Transportation', 'car', '#3B82F6', true),
    ('health', 'Health', 'heart-pulse', '#EF4444', true),
    ('education', 'Education', 'graduation-cap', '#6366F1', true),
    ('entertainment', 'Entertainment', 'gamepad', '#EC4899', true),
    ('clothing', 'Clothing', 'shirt', '#F97316', true),
    ('financial', 'Financial', 'landmark', '#14B8A6', true),
    ('technology', 'Technology', 'laptop', '#6B7280', true),
    ('subscriptions', 'Subscriptions', 'repeat', '#8B5CF6', true),
    ('other', 'Other', 'circle-dot', '#9CA3AF', true);
