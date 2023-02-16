CREATE TABLE receipts (
  cid         VARCHAR NOT NULL PRIMARY KEY,
  closure_cid VARCHAR NOT NULL,
  nonce       VARCHAR NOT NULL,
  out         BINARY  NOT NULL
);

CREATE INDEX closure_cid_index ON receipts (closure_cid);
