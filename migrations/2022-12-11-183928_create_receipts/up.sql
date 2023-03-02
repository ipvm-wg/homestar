CREATE TABLE receipts (
  cid         TEXT NOT NULL PRIMARY KEY,
  ran         TEXT NOT NULL,
  out         BLOB  NOT NULL,
  meta        BLOB  NOT NULL,
  iss         TEXT,
  prf         BLOB NOT NULL
);

CREATE INDEX ran_index ON receipts (ran);
