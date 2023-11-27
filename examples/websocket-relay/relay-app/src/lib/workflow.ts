import { base64 } from "iso-base/rfc4648";
import { get as getStore } from "svelte/store";
import type { MaybeResult } from "@fission-codes/homestar/codecs/types";
import type {
  Receipt as RawReceipt,
  WorkflowNotification,
  WorkflowNotificationError,
} from "@fission-codes/homestar";
import * as WorkflowBuilder from "@fission-codes/homestar/workflow";

import type { Receipt, TaskOperation, TaskStatus, Meta } from "$lib/task";

import {
  activeWorkflowStore,
  firstWorkflowToRunStore,
  homestarStore,
  taskStore,
  workflowStore,
} from "../stores";

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
  const firstWorkflowToRun = getStore(firstWorkflowToRunStore);
  const homestar = getStore(homestarStore);
  const tasks = getStore(taskStore);

  // Reset workflow UI and state
  reset(workflowId);

  // Initialize active workflow
  activeWorkflowStore.set({
    id: workflowId,
    tasks: tasks[workflowId].map((task) => task.operation),
    step: 0,
    failedPingCount: 0,
  });

  // Record the first workflow that ran
  if (!firstWorkflowToRun) {
    firstWorkflowToRunStore.set(workflowId);
  }

  // Set workflow status to working
  workflowStore.update((workflows) => ({
    ...workflows,
    [workflowId]: { ...workflows[workflowId], status: "working" },
  }));

  // Send run command to server
  if (workflowId === "one") {
    const workflowOne = await workflowOnePromised;
    homestar.runWorkflow(workflowOne, handleMessage);
  } else if (workflowId === "two") {
    const workflowTwo = await workflowTwoPromised;
    homestar.runWorkflow(workflowTwo, handleMessage);
  }

  checkHealth();
}

/**
 * Check health and fail workflow when the Homestar node does not respond in time.
 */
function checkHealth() {
  const homestar = getStore(homestarStore);

  let interval = setInterval(async () => {
    const activeWorkflow = getStore(activeWorkflowStore);

    if (activeWorkflow) {
      if (activeWorkflow.step === activeWorkflow.tasks.length - 1) {
        // Workflow completed
        clearInterval(interval);
      }

      if (
        activeWorkflow.failedPingCount >= import.meta.env.VITE_MAX_PING_RETRIES
      ) {
        // Fail the workflow
        fail(activeWorkflow.id);
        clearInterval(interval);
      }

      const health = (await homestar.health()).result;
      if (health?.healthy) {
        activeWorkflowStore.update((store) =>
          store ? { ...store, failedPingCount: 0 } : null
        );
      } else {
        activeWorkflowStore.update((store) =>
          store
            ? { ...store, failedPingCount: activeWorkflow.failedPingCount + 1 }
            : null
        );
      }
    } else {
      // No workflow active
      clearInterval(interval);
    }
  }, import.meta.env.VITE_PING_INTERVAL);
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

export async function handleMessage(
  data: MaybeResult<WorkflowNotification, WorkflowNotificationError>
) {
  console.log("Received message from server: ", data);

  if (data.error) {
    throw data.error;
  }

  const activeWorkflow = getStore(activeWorkflowStore);

  if (!activeWorkflow) {
    console.error("Received a receipt but workflow was not initiated");
    return;
  }

  const taskId = activeWorkflow.step + 1;
  const status = data.result.metadata.replayed ? "replayed" : "executed";
  const receipt = parseReceipt(data.result.receipt);

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
}

const parseReceipt = (raw: RawReceipt<Uint8Array>): Receipt => {
  return {
    iss: raw.iss ?? null,
    meta: raw.meta as Meta,
    out: [raw.out[0], base64.encode(raw.out[1])],
    prf: raw.prf.map(toString),
    ran: raw.ran.toString(),
  };
};

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

// WORKFLOWS

export const workflowOnePromised = WorkflowBuilder.workflow({
  name: "one",
  workflow: {
    tasks: [
      WorkflowBuilder.crop({
        name: "crop",
        resource:
          "ipfs://bafybeiczefaiu7464ehupezpzulnti5jvcwnvdalqrdliugnnwcdz6ljia",
        args: {
          data: "{{ cid:bafybeiejevluvtoevgk66plh5t6xiy3ikyuuxg3vgofuvpeckb6eadresm }}",
          x: 150,
          y: 350,
          height: 500,
          width: 500,
        },
      }),
      WorkflowBuilder.rotate90({
        name: "rotate90",
        resource:
          "ipfs://bafybeiczefaiu7464ehupezpzulnti5jvcwnvdalqrdliugnnwcdz6ljia",
        args: {
          data: "{{needs.crop.output}}",
        },
      }),
      WorkflowBuilder.blur({
        name: "blur",
        resource:
          "ipfs://bafybeiczefaiu7464ehupezpzulnti5jvcwnvdalqrdliugnnwcdz6ljia",
        args: {
          data: "{{needs.rotate90.output}}",
          sigma: 20.2,
        },
      }),
    ],
  },
});

export const workflowTwoPromised = WorkflowBuilder.workflow({
  name: "two",
  workflow: {
    tasks: [
      WorkflowBuilder.crop({
        name: "crop",
        resource:
          "ipfs://bafybeiczefaiu7464ehupezpzulnti5jvcwnvdalqrdliugnnwcdz6ljia",
        args: {
          data: "{{ cid:bafybeiejevluvtoevgk66plh5t6xiy3ikyuuxg3vgofuvpeckb6eadresm }}",
          x: 150,
          y: 350,
          height: 500,
          width: 500,
        },
      }),
      WorkflowBuilder.rotate90({
        name: "rotate90",
        resource:
          "ipfs://bafybeiczefaiu7464ehupezpzulnti5jvcwnvdalqrdliugnnwcdz6ljia",
        args: {
          data: "{{needs.crop.output}}",
        },
      }),
      WorkflowBuilder.grayscale({
        name: "grayscale",
        resource:
          "ipfs://bafybeiczefaiu7464ehupezpzulnti5jvcwnvdalqrdliugnnwcdz6ljia",
        args: {
          data: "{{needs.rotate90.output}}",
        },
      }),
    ],
  },
});
