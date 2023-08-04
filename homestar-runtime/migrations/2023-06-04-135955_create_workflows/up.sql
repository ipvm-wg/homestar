CREATE TABLE workflows (
  cid           TEXT NOT NULL PRIMARY KEY,
  num_tasks     INTEGER NOT NULL,
  resources     BLOB NOT NULL,
  created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  completed_at  TIMESTAMP
);
