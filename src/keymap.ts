import { useUI, UILayer, getActiveLayer } from "./store/ui";
import { useMessages, run } from "./store/messages";
import { SessionSnapshot } from "./types";

export interface KeyBinding {
  keys: string;
  layers: UILayer[];
  label: string;
  run: () => void | Promise<void>;
}

export const keymap: KeyBinding[] = [
  {
    keys: "ctrl+m",
    layers: ["idle"],
    label: "Compose message",
    run: () => {
      const { isGenerating } = useMessages.getState();
      if (isGenerating) return;
      useUI.getState().startCompose();
    },
  },
  {
    keys: "ctrl+e",
    layers: ["idle"],
    label: "Edit last user message",
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
    label: "Previous user message",
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
    label: "Next user message",
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
    keys: "Escape",
    layers: ["compose", "edit", "history", "help"],
    label: "Close topmost layer",
    run: () => {
      useMessages.getState().setError(null);
      useUI.getState().closeTopmost();
    },
  },
  {
    keys: "ctrl+r",
    layers: ["idle"],
    label: "Resend",
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
    label: "Cancel generation",
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
    label: "New chat",
    run: async () => {
      useUI.getState().setMode("idle");
      useUI.getState().setOverlay(null);
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
    label: "Toggle history",
    run: () => {
      const { overlay, mode } = useUI.getState();
      if (overlay === "history") {
        useUI.getState().setOverlay(null);
      } else if (mode === "idle" && overlay === null) {
        useUI.getState().setOverlay("history");
      }
    },
  },
  {
    keys: "ctrl+t",
    layers: ["idle"],
    label: "Toggle thinking",
    run: () => {
      useUI.getState().toggleThinking();
    },
  },
  {
    keys: "ctrl+/",
    layers: ["idle", "help"],
    label: "Toggle shortcut help",
    run: () => {
      const { overlay, mode } = useUI.getState();
      if (overlay === "help") {
        useUI.getState().setOverlay(null);
      } else if (mode === "idle" && overlay === null) {
        useUI.getState().setOverlay("help");
      }
    },
  },
];

export function handleKeydown(e: KeyboardEvent): boolean {
  if (e.isComposing || e.keyCode === 229) {
    return false;
  }

  if (e.repeat) {
    return false;
  }

  const activeLayer = getActiveLayer(useUI.getState());
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
