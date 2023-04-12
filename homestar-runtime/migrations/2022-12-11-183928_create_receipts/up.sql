CREATE TABLE receipts (
  cid         TEXT NOT NULL PRIMARY KEY,
  ran         TEXT NOT NULL,
  instruction TEXT NOT NULL,
  out         BLOB NOT NULL,
  meta        BLOB NOT NULL,
  issuer      TEXT,
  prf         BLOB NOT NULL,
  version     TEXT NOT NULL
);

CREATE INDEX instruction_index ON receipts (instruction);
