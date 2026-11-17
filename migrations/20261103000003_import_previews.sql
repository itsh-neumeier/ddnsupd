CREATE TABLE provider_discovery_limits (
    provider_id uuid PRIMARY KEY REFERENCES providers(id) ON DELETE CASCADE,
    window_start timestamptz NOT NULL DEFAULT now(),
    requests integer NOT NULL DEFAULT 1
);
CREATE TABLE import_previews (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_id uuid NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    credential_version bigint NOT NULL,
    plan jsonb NOT NULL,
    plan_digest text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL DEFAULT now()+interval '10 minutes',
    consumed_at timestamptz
);
CREATE INDEX import_previews_expiry ON import_previews(expires_at);
