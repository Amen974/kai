import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { ChatEvent, Message } from "../types";

export interface ChatState {
  messages: Message[];
  historyId: number | null;
  isGenerating: boolean;
  error: string | null;
  historyVersion: number;
  seq: number;
  dispatch: (event: ChatEvent) => void;
  setError: (error: string | null) => void;
}

export function appendToAssistant(
  messages: Message[],
  field: "content" | "thinking",
  text: string
): Message[] {
  if (messages.length === 0) {
    return [
      {
        role: "assistant",
        content: field === "content" ? text : "",
        thinking: field === "thinking" ? text : null,
      },
    ];
  }

  const lastIndex = messages.length - 1;
  const lastMessage = messages[lastIndex];

  if (lastMessage.role === "assistant") {
    const updated = [...messages];
    updated[lastIndex] = {
      ...lastMessage,
      [field]: (lastMessage[field] ?? "") + text,
    };
    return updated;
  }

  return [
    ...messages,
    {
      role: "assistant",
      content: field === "content" ? text : "",
      thinking: field === "thinking" ? text : null,
    },
  ];
}

export function chatReducer(state: ChatState, event: ChatEvent): Partial<ChatState> {
  switch (event.type) {
    case "Snapshot": {
      const updates: Partial<ChatState> = {
        messages: event.messages,
        error: null,
        seq: state.seq + 1,
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
        seq: state.seq + 1,
      };
    }

    case "ThinkingDelta": {
      return {
        messages: appendToAssistant(state.messages, "thinking", event.thinking),
        isGenerating: true,
        seq: state.seq + 1,
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
    const message =
      typeof err === "string"
        ? err
        : err instanceof Error
        ? err.message
        : typeof err === "object" && err !== null && "message" in err
        ? String((err as { message: unknown }).message)
        : String(err);
    useMessages.getState().setError(message);
    throw err;
  }
}