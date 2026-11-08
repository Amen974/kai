import { create } from "zustand";
import { Conversation } from "../types";

export type UIMode = "idle" | "compose" | "edit";
export type UIOverlay = "history" | null;
export type UILayer = "history" | "compose" | "edit" | "idle";

export interface UIState {
  mode: UIMode;
  overlay: UIOverlay;
  editIndex: number | null;
  draft: string;
  editDraft: string;
  thinkingIndex: number | null;
  historyList: Conversation[];
  historyIndex: number;
  historyArmedId: number | null;
  historySearchQuery: string;
  scrollPinSignal: number;
  setMode: (mode: UIMode) => void;
  setOverlay: (overlay: UIOverlay) => void;
  startCompose: () => void;
  startEdit: (index: number, content: string) => void;
  setDraft: (draft: string) => void;
  setEditDraft: (editDraft: string) => void;
  toggleThinking: (index: number) => void;
  setHistoryList: (list: Conversation[]) => void;
  setHistoryIndex: (index: number) => void;
  setHistoryArmedId: (id: number | null) => void;
  setHistorySearchQuery: (query: string) => void;
  forceScrollPin: () => void;
  closeTopmost: () => void;
}

export const useUI = create<UIState>(set => ({
  mode: "idle",
  overlay: null,
  editIndex: null,
  draft: "",
  editDraft: "",
  scrollPinSignal: 0,
  thinkingIndex: null,
  historyList: [],
  historyIndex: 0,
  historyArmedId: null,
  historySearchQuery: "",

  setMode: (mode) => set({ mode }),
  setOverlay: (overlay) => set({ overlay }),

  startCompose: () =>
    set({
      mode: "compose",
      overlay: null,
    }),

  startEdit: (index, content) =>
    set({
      mode: "edit",
      editIndex: index,
      editDraft: content,
      overlay: null,
    }),

  setDraft: (draft) => set({ draft }),
  setEditDraft: (editDraft) => set({ editDraft }),

  toggleThinking: (index) =>
    set((state) => ({
      thinkingIndex: state.thinkingIndex === index ? null : index,
    })),

  setHistoryList: (historyList) => set({ historyList }),
  setHistoryIndex: (historyIndex) => set({ historyIndex }),
  setHistoryArmedId: (historyArmedId) => set({ historyArmedId }),
  setHistorySearchQuery: (historySearchQuery) => set({ historySearchQuery }),

  forceScrollPin: () => set((state) => ({ scrollPinSignal: state.scrollPinSignal + 1 })),


  closeTopmost: () => {
      set({ overlay: null, mode: "idle", editIndex: null, editDraft: "", historyArmedId: null, historySearchQuery: "" });
      return;
  },
}));

export function getActiveLayer(state: UIState): UILayer {
  return state.overlay ?? state.mode;
}
