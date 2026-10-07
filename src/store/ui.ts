import { create } from "zustand";

export type UIMode = "idle" | "compose" | "edit";
export type UIOverlay = "history" | "help" | null;
export type UILayer = "history" | "help" | "compose" | "edit" | "idle";

export interface UIState {
  mode: UIMode;
  overlay: UIOverlay;
  editIndex: number | null;
  draft: string;
  editDraft: string;
  showThinking: boolean;
  scrollPinSignal: number;
  setMode: (mode: UIMode) => void;
  setOverlay: (overlay: UIOverlay) => void;
  startCompose: () => void;
  startEdit: (index: number, content: string) => void;
  setDraft: (draft: string) => void;
  setEditDraft: (editDraft: string) => void;
  toggleThinking: () => void;
  forceScrollPin: () => void;
  closeTopmost: () => void;
}

export const useUI = create<UIState>((set, get) => ({
  mode: "idle",
  overlay: null,
  editIndex: null,
  draft: "",
  editDraft: "",
  showThinking: false,
  scrollPinSignal: 0,

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

  toggleThinking: () => set((state) => ({ showThinking: !state.showThinking })),

  forceScrollPin: () => set((state) => ({ scrollPinSignal: state.scrollPinSignal + 1 })),


  closeTopmost: () => {
    const { overlay, mode } = get();
    if (overlay !== null) {
      set({ overlay: null });
      return;
    }
    if (mode === "edit") {
      // Edit cancels: discard edit draft and index
      set({ mode: "idle", editIndex: null, editDraft: "" });
      return;
    }
    if (mode === "compose") {
      // Compose closes: keep draft in store
      set({ mode: "idle" });
      return;
    }
  },
}));

export function getActiveLayer(state: UIState): UILayer {
  return state.overlay ?? state.mode;
}
