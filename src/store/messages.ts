import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Attachment,
  ChatEvent,
  DownloadProgress,
  Message,
  MessageVariant,
  ModelInfo,
  SessionSnapshot,
} from "../types";
import { useUI } from "./ui";

export interface ChatState {
  messages: Message[];
  historyId: number | null;
  activeBranchId: number | null;
  variantsByParentId: Record<string, MessageVariant[]>;
  attachments: Attachment[];
  models: ModelInfo[];
  download: DownloadProgress | null;
  isGenerating: boolean;
  error: string | null;
  historyVersion: number;
  modelsVersion: number;
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
        attachments: [],
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

    case "ModelsChanged": {
      return {
        modelsVersion: state.modelsVersion + 1,
      };
    }

    case "ModelDownloadProgress": {
      return {
        download: {
          name: event.name,
          status: event.status,
          completed: event.completed,
          total: event.total,
        },
      };
    }
  }
}

export const useMessages = create<ChatState>((set) => ({
  messages: [],
  historyId: null,
  activeBranchId: null,
  variantsByParentId: {},
  attachments: [],
  models: [],
  download: null,
  isGenerating: false,
  error: null,
  historyVersion: 0,
  modelsVersion: 0,
  dispatch: (event: ChatEvent) => {
    set((state) => chatReducer(state, event));
    if (event.type === "Snapshot" || event.type === "GenerationFinished") {
      refreshVariants().catch((err) => {
        console.error("Failed to refresh variants:", err);
      });
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

export async function pickAttachments(): Promise<void> {
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: "Text or image", extensions: ["txt", "png", "jpg", "jpeg", "webp", "gif"] }],
    });
    if (!selected) return;

    for (const path of Array.isArray(selected) ? selected : [selected]) {
      try {
        const attachment = await run<Attachment>("attach_file", { path });
        useMessages.setState((state) => ({ attachments: [...state.attachments, attachment] }));
      } catch {
        // run() already surfaced the rejection; keep attaching the remaining files.
      }
    }
  } catch (err) {
    useMessages.getState().setError(String(err));
  }
}

export async function detachAttachment(attachment: Attachment): Promise<void> {
  try {
    await run("remove_pending_attachment", { path: attachment.path });
    useMessages.setState((state) => ({
      attachments: state.attachments.filter((item) => item.path !== attachment.path),
    }));
  } catch (err) {
    console.error("Failed to detach attachment:", err);
  }
}

export async function loadModels(): Promise<void> {
  try {
    useMessages.setState({ models: await run<ModelInfo[]>("get_models") });
  } catch (err) {
    console.error("Failed to load models:", err);
  }
}

export async function selectModel(index: number): Promise<void> {
  const model = useMessages.getState().models[index];
  if (!model) return;

  useUI.getState().setModelIndex(index);
  if (!model.active && !(await switchModel(model.name))) return;
  useUI.getState().setOverlay(null);
}

async function switchModel(name: string): Promise<boolean> {
  try {
    await run("set_active_model", { name });
    return true;
  } catch (err) {
    console.error("Failed to switch model:", err);
    return false;
  }
}

export async function downloadModel(query: string): Promise<void> {
  const name = query.trim();
  if (!name || useMessages.getState().download) return;

  useMessages.setState({
    download: { name, status: "starting", completed: 0, total: 0 },
    error: null,
  });

  try {
    await run("download_model", { name });
    useMessages.setState({ download: null });
  } catch {
    useMessages.setState({ download: null });
  }
}

export async function cancelDownload(): Promise<void> {
  if (!useMessages.getState().download) return;

  try {
    await run("cancel_model_download");
  } catch (err) {
    console.error("Failed to cancel model download:", err);
  }
}

export async function syncModels(): Promise<void> {
  try {
    await run("sync_models");
  } catch (err) {
    console.error("Failed to sync models:", err);
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