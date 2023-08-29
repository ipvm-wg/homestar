import { get as getStore } from "svelte/store";

import type { Receipt, TaskOperation, TaskStatus, Meta } from "$lib/task";

import {
  activeWorkflowStore,
  channelStore,
  taskStore,
  workflowStore,
} from "../stores";
import { connect, type Channel } from "$lib/channel";
import type { Maybe } from "$lib";

export type Workflow = {
  id: WorkflowId;
  status: "waiting" | "working";
};

export type WorkflowState = {
  id: WorkflowId;
  tasks: TaskOperation[];
  step: number;
  failedPingCount: number;
};

export type WorkflowId = "one" | "two";

// RUN
export async function run(workflowId: WorkflowId) {
  let channel = getStore(channelStore);
  const tasks = getStore(taskStore);

  if (!channel) {
    await connect();
    channel = getStore(channelStore);
  }

  // Reset workflow UI and state
  reset(workflowId);

  // Initialize active workflow
  activeWorkflowStore.set({
    id: workflowId,
    tasks: tasks[workflowId].map((task) => task.operation),
    step: 0,
    failedPingCount: 0,
  });

  // Set workflow status to working
  workflowStore.update((workflows) => ({
    ...workflows,
    [workflowId]: { ...workflows[workflowId], status: "working" },
  }));

  // Send run command to server
  if (workflowId === "one") {
    channel?.send(
      JSON.stringify({
        action: "run",
        name: workflowId,
        workflow: workflowOneJson,
      })
    );
  } else if (workflowId === "two") {
    channel?.send(
      JSON.stringify({
        action: "run",
        name: workflowId,
        workflow: workflowTwoJson,
      })
    );
  }

  if (import.meta.env.VITE_EMULATION_MODE === "true") {
    // Emulate with an echo server
    emulate(workflowId, channel);
  }
}

/**
 * Reset tasks to waiting and workflow to starting state
 *
 * @param workflowId Workflow to reset
 */
function reset(workflowId: WorkflowId) {
  const status: TaskStatus = "waiting";

  taskStore.update((store) => {
    const updatedTasks = store[workflowId].map((t) => ({
      ...t,
      status,
      message: getTaskMessage(status),
      receipt: null,
    }));
    return { ...store, [workflowId]: updatedTasks };
  });
}

/**
 * Fail any tasks that have not completed
 *
 * @param workflowId Workflow to fail
 */
export function fail(workflowId: WorkflowId) {
  taskStore.update((store) => {
    const updatedTasks = store[workflowId].map((t) => {
      if (t.status !== "executed" && t.status !== "replayed") {
        const status: TaskStatus = "failure";

        return { ...t, status };
      } else {
        return t;
      }
    });

    return { ...store, [workflowId]: updatedTasks };
  });

  // Set workflow status to waiting
  workflowStore.update((workflows) => ({
    ...workflows,
    [workflowId]: { ...workflows[workflowId], status: "waiting" },
  }));
}

// HANDLER

export async function handleMessage(event: MessageEvent) {
  const data = await event.data.text();

  console.log("Received message from server: ", data);

  // Reset ping count on echoed ping or pong from server
  if (data === "ping" || data === "pong") {
    activeWorkflowStore.update((store) =>
      store ? { ...store, failedPingCount: 0 } : null
    );

    return;
  }

  const message = JSON.parse(data);
  if (message.receipt !== undefined && message.receipt.meta !== undefined) {
    const activeWorkflow = getStore(activeWorkflowStore);

    if (!activeWorkflow) {
      console.error("Received a receipt but workflow was not initiated");
      return;
    }

    const taskId = activeWorkflow.step + 1;
    const status = message.metadata.replayed ? "replayed" : "executed";
    const receipt = parseReceipt(message.receipt);

    // Update task in UI
    taskStore.update((store) => {
      const updatedTasks = store[activeWorkflow.id].map((t) =>
        t.id === taskId
          ? {
            ...t,
            status,
            message: getTaskMessage(status),
            receipt,
          }
          : t
      );

      return { ...store, [activeWorkflow.id]: updatedTasks };
    });

    // Log receipt
    console.table(receipt);

    if (activeWorkflow.step === activeWorkflow.tasks.length - 1) {
      // Workflow is done. Reset workflow status to waiting.
      workflowStore.update((workflows) => ({
        ...workflows,
        [activeWorkflow.id]: {
          ...workflows[activeWorkflow.id],
          status: "waiting",
        },
      }));

      // Deactivate workflow
      activeWorkflowStore.set(null);
    } else {
      // Increment workflow step
      activeWorkflowStore.update((store) =>
        store ? { ...store, step: store.step + 1 } : null
      );
    }
  } else {
    console.warn("Received an unexpected message", message);
  }
}

function parseReceipt(raw: {
  iss: string | null;
  meta: Meta | null;
  out: ["ok" | "error", Record<"/", Record<"bytes", string>>];
  prf: string[];
  ran: Record<"/", string>;
}): Receipt {
  return {
    iss: raw.iss,
    meta: raw.meta,
    out: [raw.out[0], raw.out[1]["/"].bytes],
    prf: raw.prf,
    ran: raw.ran["/"],
  };
}

function getTaskMessage(status: TaskStatus) {
  switch (status) {
    case "waiting":
      return "Waiting for task to complete.";

    case "executed":
      return "Task executed.";

    case "failure":
      return "Task failed.";

    case "replayed":
      return "Task replayed.";
  }
}

// JSON WORKFLOWS

export const workflowOneJson = {
  tasks: [
    {
      cause: null,
      meta: {
        memory: 4294967296,
        time: 100000,
      },
      prf: [],
      run: {
        input: {
          args: [
            {
              "/": "bafybeiejevluvtoevgk66plh5t6xiy3ikyuuxg3vgofuvpeckb6eadresm",
            },
            150,
            350,
            500,
            500,
          ],
          func: "crop",
        },
        nnc: "",
        op: "wasm/run",
        rsc: "https://ipfs.io/ipfs/bafybeiabbxwf2vn4j3zm7bbojr6rt6k7o6cg6xcbhqkllubmsnvocpv7y4",
      },
    },
    {
      cause: null,
      meta: {
        memory: 4294967296,
        time: 100000,
      },
      prf: [],
      run: {
        input: {
          args: [
            {
              "await/ok": {
                "/": "bafyrmiaqme3vvgunr5outkbd4qrldbwmo23hdefvu2lbckhjykhgo6dlsm",
              },
            },
          ],
          func: "rotate90",
        },
        nnc: "",
        op: "wasm/run",
        rsc: "https://ipfs.io/ipfs/bafybeiabbxwf2vn4j3zm7bbojr6rt6k7o6cg6xcbhqkllubmsnvocpv7y4",
      },
    },
    {
      cause: null,
      meta: {
        memory: 4294967296,
        time: 100000,
      },
      prf: [],
      run: {
        input: {
          args: [
            {
              "await/ok": {
                "/": "bafyrmidmuwz4tkjrck6wipx2qi7y5ke7cny5ovp3latt7mo2b7njt6g7si",
              },
            },
            20.2,
          ],
          func: "blur",
        },
        nnc: "",
        op: "wasm/run",
        rsc: "https://ipfs.io/ipfs/bafybeiabbxwf2vn4j3zm7bbojr6rt6k7o6cg6xcbhqkllubmsnvocpv7y4",
      },
    },
  ],
};

export const workflowTwoJson = {
  tasks: [
    {
      cause: null,
      meta: {
        memory: 4294967296,
        time: 100000,
      },
      prf: [],
      run: {
        input: {
          args: [
            {
              "/": "bafybeiejevluvtoevgk66plh5t6xiy3ikyuuxg3vgofuvpeckb6eadresm",
            },
            150,
            350,
            500,
            500,
          ],
          func: "crop",
        },
        nnc: "",
        op: "wasm/run",
        rsc: "https://ipfs.io/ipfs/bafybeiabbxwf2vn4j3zm7bbojr6rt6k7o6cg6xcbhqkllubmsnvocpv7y4",
      },
    },
    {
      cause: null,
      meta: {
        memory: 4294967296,
        time: 100000,
      },
      prf: [],
      run: {
        input: {
          args: [
            {
              "await/ok": {
                "/": "bafyrmiaqme3vvgunr5outkbd4qrldbwmo23hdefvu2lbckhjykhgo6dlsm",
              },
            },
          ],
          func: "rotate90",
        },
        nnc: "",
        op: "wasm/run",
        rsc: "https://ipfs.io/ipfs/bafybeiabbxwf2vn4j3zm7bbojr6rt6k7o6cg6xcbhqkllubmsnvocpv7y4",
      },
    },
    {
      cause: null,
      meta: {
        memory: 4294967296,
        time: 100000,
      },
      prf: [],
      run: {
        input: {
          args: [
            {
              "await/ok": {
                "/": "bafyrmidmuwz4tkjrck6wipx2qi7y5ke7cny5ovp3latt7mo2b7njt6g7si",
              },
            },
          ],
          func: "grayscale",
        },
        nnc: "",
        op: "wasm/run",
        rsc: "https://ipfs.io/ipfs/bafybeiabbxwf2vn4j3zm7bbojr6rt6k7o6cg6xcbhqkllubmsnvocpv7y4",
      },
    },
  ],
};

// EMULATION

function emulate(workflowId: string, channel: Maybe<Channel>) {
  if (!channel) {
    console.error("Cannot emulate. Channel has not been set.");
    return;
  }

  if (workflowId === "one") {
    Promise.resolve()
      .then(() => sendEmulated("executed", "one", "crop", channel, 500))
      .then(() => sendEmulated("executed", "one", "rotate90", channel, 1500))
      .then(() => sendEmulated("executed", "one", "blur", channel, 20000));
  } else if (workflowId === "two") {
    Promise.resolve()
      .then(() => sendEmulated("replayed", "two", "crop", channel, 200))
      .then(() => sendEmulated("replayed", "two", "rotate90", channel, 200))
      .then(() => sendEmulated("executed", "two", "grayscale", channel, 1500));
  }
}

function sendEmulated(
  status: TaskStatus,
  workflowId: string,
  op: TaskOperation,
  channel: Channel,
  delay: number
) {
  return new Promise((resolve) => {
    setTimeout(() => {
      const message = JSON.stringify(sampleReceipt(status, workflowId, op));

      channel.send(message);

      resolve(null);
    }, delay);
  });
}

const catResponse = await fetch("./spacecat");
const base64Cat = await catResponse.text();

function sampleReceipt(
  status: TaskStatus,
  workflowId: string,
  op: TaskOperation
) {
  return {
    metadata: {
      name: workflowId,
      replayed: status == "executed" ? false : true,
      receipt_cid: {
        "/": "bafyrmiczrugtx6jj42qbwd2ctlmj766th2nwzfsqmvathjdxk63rwkkvpi",
      },
    },
    receipt: {
      iss: null,
      meta: {
        op: op,
        workflow: "bafyrmiczrugtx6jj42qbwd2ctlmj766th2nwzfsqmvathjdxk63rwkkvpd",
      },
      out: ["ok", { "/": { bytes: `${base64Cat}` } }],
      prf: [],
      ran: {
        "/": "bafkr4ickinozehpaz72vtgpbhhqpf6v2fi67rvr6uis52bwsesoss6vinq",
      },
    },
  };
}
