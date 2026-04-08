-- Update category names to Spanish
UPDATE categories SET name = 'Ingresos' WHERE key = 'income' AND is_system = true;
UPDATE categories SET name = 'Vivienda' WHERE key = 'housing' AND is_system = true;
UPDATE categories SET name = 'Mercado' WHERE key = 'groceries' AND is_system = true;
UPDATE categories SET name = 'Restaurantes' WHERE key = 'restaurants' AND is_system = true;
UPDATE categories SET name = 'Transporte' WHERE key = 'transportation' AND is_system = true;
UPDATE categories SET name = 'Salud' WHERE key = 'health' AND is_system = true;
UPDATE categories SET name = 'Educación' WHERE key = 'education' AND is_system = true;
UPDATE categories SET name = 'Entretenimiento' WHERE key = 'entertainment' AND is_system = true;
UPDATE categories SET name = 'Ropa' WHERE key = 'clothing' AND is_system = true;
UPDATE categories SET name = 'Financiero' WHERE key = 'financial' AND is_system = true;
UPDATE categories SET name = 'Tecnología' WHERE key = 'technology' AND is_system = true;
UPDATE categories SET name = 'Suscripciones' WHERE key = 'subscriptions' AND is_system = true;
UPDATE categories SET name = 'Otros' WHERE key = 'other' AND is_system = true;

-- Add Services category for utilities (Luz, Gas, Agua, Internet, etc.)
INSERT INTO categories (key, name, icon, color, is_system) VALUES
    ('services', 'Servicios', 'zap', '#0EA5E9', true);
