ALTER TABLE workflows ADD COLUMN status TEXT CHECK(
    status IN ('pending', 'completed', 'running', 'stuck')) NOT NULL DEFAULT
            'pending';
ALTER TABLE workflows ADD COLUMN retries INTEGER NOT NULL DEFAULT 0;
