CREATE TABLE tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    code VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    namespace VARCHAR NOT NULL,
    creation_timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    permission_read BOOLEAN NOT NULL DEFAULT 0,
    permission_write BOOLEAN NOT NULL DEFAULT 0,
    permission_share_share BOOLEAN NOT NULL DEFAULT 0,
    permission_share_read BOOLEAN NOT NULL DEFAULT 0,
    permission_share_write BOOLEAN NOT NULL DEFAULT 0,
    parent INTEGER,
    FOREIGN KEY (parent) REFERENCES tokens(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_tokens_code ON tokens(code);
CREATE INDEX idx_tokens_namespace ON tokens(namespace);
CREATE INDEX idx_tokens_parent ON tokens(parent);

