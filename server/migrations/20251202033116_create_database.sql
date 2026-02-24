CREATE TABLE IF NOT EXISTS users (
    id         BIGSERIAL PRIMARY KEY,
    anonymous  BOOLEAN NOT NULL DEFAULT true,
    username   TEXT NOT NULL UNIQUE CHECK (username <> ''),
    password   TEXT NOT NULL CHECK (password <> ''),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_permissions (
    user_id BIGSERIAL NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token   TEXT NOT NULL
);

INSERT INTO users (id, username, password, anonymous)
    VALUES (1, 'Guest', 'guest', true)
    ON CONFLICT (id)
    DO NOTHING;
