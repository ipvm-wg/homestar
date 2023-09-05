export type Task = {
  id: number;
  workflowId: "one" | "two";
  operation: TaskOperation;
  selected: boolean;
  status: TaskStatus;
  message: string;
  receipt?: Receipt;
};

export type Receipt = {
  iss: string | null;
  meta: Meta | null;
  out: ["ok" | "error", string];
  prf: string[];
  ran: string;
};

export type Meta = {
  op: string;
  workflow: string;
};

export type TaskStatus = "waiting" | "replayed" | "executed" | "failure";

export type TaskOperation = "crop" | "rotate90" | "blur" | "grayscale";
