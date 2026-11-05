import { useUI, UILayer, getActiveLayer } from "./store/ui";
import { useMessages, run } from "./store/messages";
import { SessionSnapshot } from "./types";
import { getCurrentWindow } from "@tauri-apps/api/window";

export interface KeyBinding {
  keys: string;
  layers: UILayer[];
  run: () => void | Promise<void>;
}

export const keymap: KeyBinding[] = [
  {
    keys: "ctrl+m",
    layers: ["idle"],
    run: () => {
      const { isGenerating } = useMessages.getState();
      if (isGenerating) return;
      useUI.getState().startCompose();
    },
  },
  {
    keys: "ctrl+e",
    layers: ["idle"],
    run: () => {
      const { isGenerating, messages } = useMessages.getState();
      if (isGenerating || messages.length === 0) return;

      let lastUserIndex = -1;
      for (let i = messages.length - 1; i >= 0; i--) {
        if (messages[i].role === "user") {
          lastUserIndex = i;
          break;
        }
      }
      if (lastUserIndex >= 0) {
        useUI.getState().startEdit(lastUserIndex, messages[lastUserIndex].content);
      }
    },
  },
  {
    keys: "ctrl+ArrowUp",
    layers: ["edit"],
    run: () => {
      const { editIndex } = useUI.getState();
      const { messages } = useMessages.getState();
      if (editIndex === null) return;

      for (let i = editIndex - 1; i >= 0; i--) {
        if (messages[i].role === "user") {
          useUI.getState().startEdit(i, messages[i].content);
          break;
        }
      }
    },
  },
  {
    keys: "ctrl+ArrowDown",
    layers: ["edit"],
    run: () => {
      const { editIndex } = useUI.getState();
      const { messages } = useMessages.getState();
      if (editIndex === null) return;

      for (let i = editIndex + 1; i < messages.length; i++) {
        if (messages[i].role === "user") {
          useUI.getState().startEdit(i, messages[i].content);
          break;
        }
      }
    },
  },
  {
    keys: "ctrl+w",
    layers: ["idle", "compose", "edit", "history"],
    run: () => getCurrentWindow().close(),
  },
  {
    keys: "ctrl+shift+m",
    layers: ["idle", "compose", "edit", "history"],
    run: () => getCurrentWindow().minimize(),
  },
  {
    keys: "F11",
    layers: ["idle", "compose", "edit", "history"],
    run: async () => {
      const win = getCurrentWindow();
      await win.setFullscreen(!(await win.isFullscreen()));
    },
  },
  {
    keys: "Escape",
    layers: ["compose", "edit", "history"],
    run: () => {
      useMessages.getState().setError(null);
      useUI.getState().closeTopmost();
    },
  },
  {
    keys: "ctrl+r",
    layers: ["idle"],
    run: async () => {
      const { isGenerating } = useMessages.getState();
      if (isGenerating) return;
      useMessages.getState().setError(null);
      try {
        await run("resend_message");
      } catch (err) {
        console.error("Failed to resend message:", err);
      }
    },
  },
  {
    keys: "ctrl+s",
    layers: ["idle", "compose", "edit"],
    run: async () => {
      const { isGenerating } = useMessages.getState();
      if (!isGenerating) return;
      try {
        await run("cancel_generation");
      } catch (err) {
        console.error("Failed to cancel generation:", err);
      }
    },
  },
  {
    keys: "ctrl+n",
    layers: ["idle"],
    run: async () => {
      useUI.getState().closeTopmost();
      useUI.setState({ thinkingIndex: null });
      try {
        const snapshot = await run<SessionSnapshot>("new_chat");
        useMessages.getState().dispatch({ type: "Snapshot", ...snapshot });
      } catch (err) {
        console.error("Failed to start new chat:", err);
      }
    },
  },
  {
    keys: "ctrl+h",
    layers: ["idle", "history"],
    run: () => {
      const { overlay, mode, historyList } = useUI.getState();
      if (overlay === "history") {
        useUI.getState().setOverlay(null);
      } else if (mode === "idle" && overlay === null) {
        const currentActiveId = useMessages.getState().historyId;
        const idx = historyList.findIndex((c) => c.id === currentActiveId);
        useUI.setState({
          overlay: "history",
          historyIndex: idx >= 0 ? idx : 0,
          historyArmedId: null,
        });
      }
    },
  },
  {
    keys: "ArrowUp",
    layers: ["history"],
    run: () => {
      const { historyIndex } = useUI.getState();
      useUI.setState({
        historyIndex: Math.max(0, historyIndex - 1),
        historyArmedId: null,
      });
    },
  },
  {
    keys: "ArrowDown",
    layers: ["history"],
    run: () => {
      const { historyIndex, historyList } = useUI.getState();
      useUI.setState({
        historyIndex: Math.min(Math.max(0, historyList.length - 1), historyIndex + 1),
        historyArmedId: null,
      });
    },
  },
  {
    keys: "Enter",
    layers: ["history"],
    run: async () => {
      const { historyIndex, historyList, setOverlay } = useUI.getState();
      const { dispatch } = useMessages.getState();
      const selected = historyList[historyIndex];
      if (!selected) return;

      useUI.setState({ historyArmedId: null });
      try {
        const snapshot = await run<SessionSnapshot>("load_chat", { id: selected.id });
        dispatch({ type: "Snapshot", ...snapshot });
        useUI.getState().forceScrollPin();
        useUI.setState({ thinkingIndex: null });
        setOverlay(null);
      } catch (error) {
        console.error("Failed to load chat:", error);
      }
    },
  },
  {
    keys: "Delete",
    layers: ["history"],
    run: handleDeleteHistory,
  },
  {
    keys: "Backspace",
    layers: ["history"],
    run: handleDeleteHistory,
  },
];

async function handleDeleteHistory() {
  const { historyIndex, historyList, historyArmedId } = useUI.getState();
  const selected = historyList[historyIndex];
  if (!selected) return;

  if (historyArmedId === selected.id) {
    try {
      await run("delete_history", { id: selected.id });
      const nextList = historyList.filter((conv) => conv.id !== selected.id);
      const nextIndex = Math.min(historyIndex, Math.max(0, nextList.length - 1));
      useUI.setState({
        historyList: nextList,
        historyIndex: nextIndex,
        historyArmedId: null,
      });

      const { historyId, dispatch } = useMessages.getState();
      if (historyId === selected.id) {
        const session = await run<SessionSnapshot>("get_session");
        dispatch({ type: "Snapshot", ...session });
      }
    } catch (error) {
      console.error("Failed to delete history:", error);
    }
  } else {
    useUI.setState({ historyArmedId: selected.id });
  }
}

export function handleKeydown(e: KeyboardEvent): boolean {
  if (e.isComposing || e.keyCode === 229) {
    return false;
  }

  if (e.repeat) {
    return false;
  }

  const activeLayer = getActiveLayer(useUI.getState());
  if (activeLayer === "history") {
    if (
      e.key !== "Control" &&
      e.key !== "Meta" &&
      e.key !== "Alt" &&
      e.key !== "Shift" &&
      e.key !== "Escape" &&
      e.key !== "Delete" &&
      e.key !== "Backspace"
    ) {
      useUI.getState().setHistoryArmedId(null);
    }
  }
  const hasCtrl = e.ctrlKey || e.metaKey;
  const keyName = e.key;

  for (const binding of keymap) {
    if (!binding.layers.includes(activeLayer)) {
      continue;
    }

    const parts = binding.keys.toLowerCase().split("+");
    const targetKey = parts[parts.length - 1];
    const targetCtrl = parts.includes("ctrl");
    const targetShift = parts.includes("shift");
    const targetAlt = parts.includes("alt");

    const keyMatches =
      keyName.toLowerCase() === targetKey.toLowerCase() ||
      (targetKey === "arrowup" && keyName === "ArrowUp") ||
      (targetKey === "arrowdown" && keyName === "ArrowDown") ||
      (targetKey === "arrowleft" && keyName === "ArrowLeft") ||
      (targetKey === "arrowright" && keyName === "ArrowRight") ||
      (targetKey === "/" && (keyName === "/" || keyName === "?"));

    if (
      keyMatches &&
      hasCtrl === targetCtrl &&
      e.shiftKey === targetShift &&
      e.altKey === targetAlt
    ) {
      e.preventDefault();
      binding.run();
      return true;
    }
  }

  return false;
}
