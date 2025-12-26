-- Seed demo OAuth2 client and user

-- Insert demo client (matches .env credentials)
INSERT INTO clients (client_id, client_secret, redirect_uri)
VALUES (
    'demo_client',
    'demo_secret',
    'http://127.0.0.1:8081/callback'
);

-- Insert demo user
-- Username: demo
-- Password: demo123
-- Hash generated with argon2
INSERT INTO users (user_id, username, password_hash)
VALUES (
    'user_001',
    'demo',
    '$argon2id$v=19$m=19456,t=2,p=1$JhvuLV9U8crvYEKOD3jmjw$pE+rVQN+1BvVj0q+fwIXixV8JbNpz4Q2TimSnEJDuGo'
);
