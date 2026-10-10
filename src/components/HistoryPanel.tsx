import { useEffect, useRef, useState } from "react";
import { run, useMessages } from "../store/messages";
import { useUI } from "../store/ui";
import { Conversation, SessionSnapshot } from "../types";

const HistoryPanel = () => {
  const isOpen = useUI((state) => state.overlay === "history");
  const setOverlay = useUI((state) => state.setOverlay);
  const [history, setHistory] = useState<Conversation[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [armedDeleteId, setArmedDeleteId] = useState<number | null>(null);

  const historyId = useMessages((state) => state.historyId);
  const historyVersion = useMessages((state) => state.historyVersion);
  const dispatch = useMessages((state) => state.dispatch);

  const listRef = useRef<HTMLUListElement>(null);
  const itemRefs = useRef<(HTMLLIElement | null)[]>([]);

  const fetchHistory = async () => {
    try {
      const result = await run<Conversation[]>("get_history");
      setHistory(result);
    } catch (error) {
      console.error("Failed to fetch history:", error);
    }
  };

  useEffect(() => {
    fetchHistory();
  }, [historyVersion]);

  useEffect(() => {
    if (isOpen) {
      setArmedDeleteId(null);
      if (history.length > 0) {
        const idx = history.findIndex((c) => c.id === historyId);
        setSelectedIndex(idx >= 0 ? idx : 0);
      }
    }
  }, [isOpen]);

  useEffect(() => {
    if (isOpen && itemRefs.current[selectedIndex]) {
      itemRefs.current[selectedIndex]?.scrollIntoView({ block: "nearest" });
    }
  }, [isOpen, selectedIndex]);

  const handleSelect = async (id: number) => {
    try {
      const snapshot = await run<SessionSnapshot>("load_chat", { id });
      console.debug("[load_chat]", snapshot.messages.map((m) => !!m.thinking));
      dispatch({ type: "Snapshot", ...snapshot });
      useUI.getState().forceScrollPin();
      useUI.setState({ thinkingIndex: null });
      setOverlay(null);
    } catch (error) {
      console.error("Failed to load chat:", error);
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await run("delete_history", { id });
      setHistory((currentHistory) => {
        const next = currentHistory.filter((conv) => conv.id !== id);
        setSelectedIndex((prev) => Math.min(prev, Math.max(0, next.length - 1)));
        return next;
      });

      if (historyId === id) {
        const session = await run<SessionSnapshot>("get_session");
        dispatch({ type: "Snapshot", ...session });
      }
    } catch (error) {
      console.error("Failed to delete history:", error);
    }
  };

  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.isComposing || e.keyCode === 229) return;

      if (e.key === "ArrowUp") {
        e.preventDefault();
        setArmedDeleteId(null);
        setSelectedIndex((prev) => Math.max(0, prev - 1));
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setArmedDeleteId(null);
        setSelectedIndex((prev) => Math.min(Math.max(0, history.length - 1), prev + 1));
      } else if (e.key === "Enter") {
        e.preventDefault();
        setArmedDeleteId(null);
        if (history[selectedIndex]) {
          handleSelect(history[selectedIndex].id);
        }
      } else if (e.key === "Delete" || e.key === "Backspace") {
        e.preventDefault();
        const selected = history[selectedIndex];
        if (!selected) return;

        if (armedDeleteId === selected.id) {
          handleDelete(selected.id);
          setArmedDeleteId(null);
        } else {
          setArmedDeleteId(selected.id);
        }
      } else if (
        e.key !== "Control" &&
        e.key !== "Meta" &&
        e.key !== "Alt" &&
        e.key !== "Shift" &&
        e.key !== "Escape"
      ) {
        setArmedDeleteId(null);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, history, selectedIndex, armedDeleteId]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-layer-overlay flex items-center justify-center">
      <div
        className="absolute inset-0 bg-black/30"
        onClick={() => setOverlay(null)}
      />

      <div className="panel-card relative w-[50vw] max-h-[60vh] flex flex-col z-layer-modal overflow-hidden">
        <ul
          ref={listRef}
          role="listbox"
          tabIndex={-1}
          className="overflow-y-auto no-scrollbar flex-1 outline-none"
        >
          {history.length === 0 ? (
            <li className="px-6 py-4 text-sm tracking-wide opacity-50">
              No history yet.
            </li>
          ) : (
            history.map((conv, idx) => {
              const isSelected = selectedIndex === idx;
              const isArmed = armedDeleteId === conv.id;

              return (
                <li
                  key={conv.id}
                  ref={(el) => {
                    itemRefs.current[idx] = el;
                  }}
                  role="option"
                  aria-selected={isSelected}
                  onClick={() => {
                    setSelectedIndex(idx);
                    setArmedDeleteId(null);
                    handleSelect(conv.id);
                  }}
                  className={`flex items-center px-6 py-3 text-sm tracking-wide cursor-pointer transition-colors ${
                    isSelected ? "bg-black/10 font-medium" : "hover:bg-black/5"
                  }`}
                >
                  <span className="truncate flex-1">
                    {conv.title}
                  </span>
                  {isArmed && (
                    <span className="text-xs text-(--color-error) tracking-wider font-mono shrink-0 ml-3 animate-pulse">
                      Delete again to confirm
                    </span>
                  )}
                </li>
              );
            })
          )}
        </ul>
      </div>
    </div>
  );
};

export default HistoryPanel;
