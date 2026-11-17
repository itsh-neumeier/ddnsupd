ALTER TABLE dns_records ADD COLUMN consecutive_failures integer NOT NULL DEFAULT 0;
