import { useEffect, useRef, useState } from "react";
import { run, useMessages } from "../store/messages";
import { useUI } from "../store/ui";
import { Conversation, SessionSnapshot } from "../types";

const HistoryPanel = () => {
  const isOpen = useUI((state) => state.overlay === "history");
  const setOverlay = useUI((state) => state.setOverlay);
  const history = useUI((state) => state.historyList);
  const selectedIndex = useUI((state) => state.historyIndex);
  const armedDeleteId = useUI((state) => state.historyArmedId);
  const searchQuery = useUI((state) => state.historySearchQuery);
  const setSearchQuery = useUI((state) => state.setHistorySearchQuery);

  const historyId = useMessages((state) => state.historyId);
  const historyVersion = useMessages((state) => state.historyVersion);
  const dispatch = useMessages((state) => state.dispatch);

  const itemRefs = useRef<(HTMLLIElement | null)[]>([]);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [hasAnyHistory, setHasAnyHistory] = useState(
    () => useUI.getState().historyList.length > 0
  );

  const fetchHistory = async () => {
    try {
      const result = await run<Conversation[]>("get_history");
      useUI.getState().setHistoryList(result);
      setHasAnyHistory(result.length > 0);
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
      setSearchQuery("");
      fetchHistory().then(() => {
        const currentList = useUI.getState().historyList;
        if (currentList.length > 0) {
          const idx = currentList.findIndex((c) => c.id === historyId);
          useUI.getState().setHistoryIndex(idx >= 0 ? idx : 0);
        }
      });
    }
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;

    const timer = setTimeout(async () => {
      const trimmed = searchQuery.trim();
      try {
        if (!trimmed) {
          const result = await run<Conversation[]>("get_history");
          useUI.getState().setHistoryList(result);
          setHasAnyHistory(result.length > 0);
        } else {
          const result = await run<Conversation[]>("search_history", { query: trimmed });
          useUI.getState().setHistoryList(result);
          useUI.getState().setHistoryIndex(0);
        }
      } catch (error) {
        console.error("Failed to search history:", error);
      }
    }, 150);

    return () => clearTimeout(timer);
  }, [searchQuery, isOpen]);

  useEffect(() => {
    if (isOpen && itemRefs.current[selectedIndex]) {
      searchInputRef.current?.focus();
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
        {(hasAnyHistory || history.length > 0) && (
          <div className="flex items-center px-6 py-3 border-b border-(--color-border)">
            <input
              ref={searchInputRef}
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search history or messages..."
              className="w-full bg-transparent outline-none text-sm tracking-wide placeholder:text-(--color-text-muted)"
            />
            {searchQuery && (
              <button
                type="button"
                onClick={() => setSearchQuery("")}
                className="text-xs text-(--color-text-muted) hover:text-(--color-text-main) tracking-wider font-mono ml-3 cursor-pointer select-none"
              >
                Clear
              </button>
            )}
          </div>
        )}

        <ul
          role="listbox"
          tabIndex={-1}
          className="overflow-y-auto no-scrollbar flex-1 outline-none"
        >
          {history.length === 0 ? (
            <li className="px-6 py-4 text-sm tracking-wide opacity-50">
              {searchQuery.trim()
                ? "No matching conversations."
                : "No history yet."}
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
