import { useEffect, useRef } from "react";
import { run, useMessages } from "../store/messages";
import { useUI } from "../store/ui";
import { Conversation, SessionSnapshot } from "../types";

const HistoryPanel = () => {
  const isOpen = useUI((state) => state.overlay === "history");
  const setOverlay = useUI((state) => state.setOverlay);
  const history = useUI((state) => state.historyList);
  const selectedIndex = useUI((state) => state.historyIndex);
  const armedDeleteId = useUI((state) => state.historyArmedId);

  const historyId = useMessages((state) => state.historyId);
  const historyVersion = useMessages((state) => state.historyVersion);
  const dispatch = useMessages((state) => state.dispatch);

  const itemRefs = useRef<(HTMLLIElement | null)[]>([]);

  const fetchHistory = async () => {
    try {
      const result = await run<Conversation[]>("get_history");
      useUI.getState().setHistoryList(result);
    } catch (error) {
      console.error("Failed to fetch history:", error);
    }
  };

  useEffect(() => {
    fetchHistory();
  }, [historyVersion]);

  useEffect(() => {
    if (isOpen) {
      useUI.getState().setHistoryArmedId(null);
      if (history.length > 0) {
        const idx = history.findIndex((c) => c.id === historyId);
        useUI.getState().setHistoryIndex(idx >= 0 ? idx : 0);
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
      dispatch({ type: "Snapshot", ...snapshot });
      useUI.getState().forceScrollPin();
      useUI.setState({ thinkingIndex: null });
      setOverlay(null);
    } catch (error) {
      console.error("Failed to load chat:", error);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-layer-overlay flex items-center justify-center">
      <div
        className="absolute inset-0 bg-black/30"
        onClick={() => setOverlay(null)}
      />

      <div className="panel-card relative w-[50vw] max-h-[60vh] flex flex-col z-layer-modal overflow-hidden">
        <ul
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
                    useUI.getState().setHistoryIndex(idx);
                    useUI.getState().setHistoryArmedId(null);
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
