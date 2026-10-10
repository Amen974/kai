import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { ChatEvent, Message } from "../types";

export interface ChatState {
  messages: Message[];
  historyId: number | null;
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
  isGenerating: false,
  error: null,
  historyVersion: 0,
  seq: 0,
  dispatch: (event: ChatEvent) => set((state) => chatReducer(state, event)),
  setError: (error: string | null) => set({ error }),
}));

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