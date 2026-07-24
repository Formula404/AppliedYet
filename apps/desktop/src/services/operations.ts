export type OperationStatus = "running" | "success" | "error";

export interface OperationItem {
  id: string;
  label: string;
  detail?: string;
  status: OperationStatus;
  startedAt: number;
  finishedAt?: number;
}

export interface OperationHandle {
  id: string;
  update: (detail: string) => void;
  succeed: (detail?: string) => void;
  fail: (reason: unknown) => void;
}

const operations = new Map<string, OperationItem>();
const listeners = new Set<(items: OperationItem[]) => void>();
let operationSequence = 0;

function snapshot() {
  return Array.from(operations.values()).sort((left, right) => right.startedAt - left.startedAt);
}

function notify() {
  const items = snapshot();
  listeners.forEach((listener) => listener(items));
}

function finishOperation(id: string, status: "success" | "error", detail?: string) {
  const current = operations.get(id);
  if (!current) return;
  operations.set(id, { ...current, status, detail, finishedAt: Date.now() });
  notify();
  window.setTimeout(() => {
    if (operations.get(id)?.status === status) {
      operations.delete(id);
      notify();
    }
  }, status === "error" ? 8000 : 5000);
}

export function startOperation(label: string, detail?: string): OperationHandle {
  const id = `operation-${Date.now()}-${++operationSequence}`;
  operations.set(id, { id, label, detail, status: "running", startedAt: Date.now() });
  notify();
  return {
    id,
    update(nextDetail) {
      const current = operations.get(id);
      if (!current || current.status !== "running") return;
      operations.set(id, { ...current, detail: nextDetail });
      notify();
    },
    succeed(nextDetail) {
      finishOperation(id, "success", nextDetail ?? "已完成");
    },
    fail(reason) {
      finishOperation(id, "error", String(reason));
    },
  };
}

export async function trackOperation<T>(
  label: string,
  action: (operation: OperationHandle) => Promise<T>,
  detail?: string,
) {
  const operation = startOperation(label, detail);
  try {
    const result = await action(operation);
    operation.succeed();
    return result;
  } catch (reason) {
    operation.fail(reason);
    throw reason;
  }
}

export function subscribeOperations(listener: (items: OperationItem[]) => void) {
  listeners.add(listener);
  listener(snapshot());
  return () => {
    listeners.delete(listener);
  };
}
