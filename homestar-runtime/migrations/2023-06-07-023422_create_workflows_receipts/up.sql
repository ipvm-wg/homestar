CREATE TABLE workflows_receipts (
  workflow_cid TEXT NOT NULL REFERENCES workflows(cid),
  receipt_cid TEXT NOT NULL REFERENCES receipts(cid),
  PRIMARY KEY(workflow_cid, receipt_cid)
);
