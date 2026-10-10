import { create } from "zustand";

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
  scrollPinSignal: number;
  setMode: (mode: UIMode) => void;
  setOverlay: (overlay: UIOverlay) => void;
  startCompose: () => void;
  startEdit: (index: number, content: string) => void;
  setDraft: (draft: string) => void;
  setEditDraft: (editDraft: string) => void;
  toggleThinking: (index: number) => void;
  forceScrollPin: () => void;
  closeTopmost: () => void;
}

export const useUI = create<UIState>(set => ({
  mode: "idle",
  overlay: null,
  editIndex: null,
  draft: "",
  editDraft: "",
  showThinking: false,
  scrollPinSignal: 0,
  thinkingIndex: null,

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

  forceScrollPin: () => set((state) => ({ scrollPinSignal: state.scrollPinSignal + 1 })),


  closeTopmost: () => {
      set({ overlay: null });
      set({ mode: "idle", editIndex: null, editDraft: "" });
      return;
  },
}));

export function getActiveLayer(state: UIState): UILayer {
  return state.overlay ?? state.mode;
}
