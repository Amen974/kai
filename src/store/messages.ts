import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { ChatEvent, Message, MessageVariant, SessionSnapshot } from "../types";
import { useUI } from "./ui";

export interface ChatState {
  messages: Message[];
  historyId: number | null;
  activeBranchId: number | null;
  variantsByParentId: Record<string, MessageVariant[]>;
  isGenerating: boolean;
  error: string | null;
  historyVersion: number;
  dispatch: (event: ChatEvent) => void;
  setError: (error: string | null) => void;
}

export function appendToAssistant(
  messages: Message[],
  field: "content" | "thinking",
  text: string
): Message[] {
  const lastIndex = messages.length - 1;
  const last = messages[lastIndex];
  const updated = [...messages];
  updated[lastIndex] = { ...last, [field]: (last[field] ?? "") + text };
  return updated;
}

export function chatReducer(state: ChatState, event: ChatEvent): Partial<ChatState> {
  switch (event.type) {
    case "Snapshot": {
      const updates: Partial<ChatState> = {
        messages: event.messages,
        error: null,
      };
      if (event.history_id !== undefined) {
        updates.historyId = event.history_id;
      }
      if (event.active_branch_id !== undefined) {
        updates.activeBranchId = event.active_branch_id;
      }
      if (event.generating !== undefined) {
        updates.isGenerating = event.generating;
      }
      return updates;
    }

    case "ContentDelta": {
      return {
        messages: appendToAssistant(state.messages, "content", event.content),
        isGenerating: true,
      };
    }

    case "ThinkingDelta": {
      return {
        messages: appendToAssistant(state.messages, "thinking", event.thinking),
        isGenerating: true,
      };
    }

    case "GenerationFinished": {
      return {
        isGenerating: false,
      };
    }

    case "Error": {
      return {
        isGenerating: false,
        error: event.message,
      };
    }

    case "HistoryChanged": {
      return {
        historyVersion: state.historyVersion + 1,
      };
    }
  }
}

export const useMessages = create<ChatState>((set) => ({
  messages: [],
  historyId: null,
  activeBranchId: null,
  variantsByParentId: {},
  isGenerating: false,
  error: null,
  historyVersion: 0,
  dispatch: (event: ChatEvent) => {
    set((state) => chatReducer(state, event));
    if (event.type === "Snapshot" || event.type === "GenerationFinished") {
      refreshVariants().catch(() => {});
    }
  },
  setError: (error: string | null) => set({ error }),
}));

export function parentKey(parentId: number | null | undefined): string {
  return parentId === null || parentId === undefined ? "root" : String(parentId);
}

export async function fetchVariants(parentMessageId: number | null): Promise<void> {
  const { historyId } = useMessages.getState();
  if (historyId === null) return;
  const key = parentKey(parentMessageId);

  try {
    const variants = await run<MessageVariant[]>("get_variants", {
      historyId,
      parentMessageId,
      history_id: historyId,
      parent_message_id: parentMessageId,
    });
    useMessages.setState((state) => ({
      variantsByParentId: {
        ...state.variantsByParentId,
        [key]: variants,
      },
    }));
  } catch (err) {
    console.error("Failed to load variants:", err);
  }
}

export async function refreshVariants(): Promise<void> {
  const { historyId, messages } = useMessages.getState();
  if (historyId === null || messages.length === 0) return;

  const parentIds = Array.from(
    new Set(messages.map((m) => (m.parent_message_id === undefined ? null : m.parent_message_id)))
  );

  await Promise.all(parentIds.map((pid) => fetchVariants(pid)));
}

export async function switchBranch(branchId: number): Promise<void> {
  const { isGenerating } = useMessages.getState();
  if (isGenerating) return;
  useMessages.getState().setError(null);
  try {
    const snapshot = await run<SessionSnapshot>("switch_branch", {
      branchId,
      branch_id: branchId,
    });
    useMessages.getState().dispatch({ type: "Snapshot", ...snapshot });
    useUI.getState().forceScrollPin();
  } catch (err) {
    console.error("Failed to switch branch:", err);
  }
}

export async function run<T = void>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err: unknown) {
    useMessages.getState().setError(String(err));
    throw err;
  }
}