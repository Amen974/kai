import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router";
import useKeyboardShortcut from "../hooks/useKeyboardShortcut";
import useMessages from "../store/messages";
import { Conversation } from "../types";

const HistoryPanel = () => {
  const [open, setOpen] = useState(false);
  const [history, setHistory] = useState<Conversation[]>([]);

  const setCurrentId = useMessages().setCurrentId;
  const navigate = useNavigate();

  useKeyboardShortcut("ctrl+h", async () => {
    if (open) {
      setOpen(false);
      return;
    }

    try {
      const result = await invoke<Conversation[]>("get_history");
      setHistory(result);
      setOpen(true);
    } catch (error) {
      console.log(error);
    }
  });

  useKeyboardShortcut("escape", () => {
    setOpen(false);
  });

  const handleSelect = async (id: number) => {
    try {
      await invoke("get_messages", { id });
      setCurrentId(id);
      navigate("/chat");
      setOpen(false);
    } catch (error) {
      console.log(error);
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div
        className="absolute inset-0 bg-black/30"
        onClick={() => setOpen(false)}
      />

      <div className="highlight relative border rounded-sm shadow-[0_2px_10px_rgba(43,42,40,0.12)] w-[50vw] max-h-[60vh] flex flex-col z-10 overflow-hidden">
        <p className="px-6 py-4 text-xs tracking-widest uppercase border-b">
          History
        </p>

        <ul className="overflow-y-auto no-scrollbar flex-1">
          {history.length === 0 ? (
            <li className="px-6 py-4 text-sm tracking-wide opacity-50">
              No history yet.
            </li>
          ) : (
            history.map((conv) => (
              <li
                key={conv.id}
                onClick={() => handleSelect(conv.id)}
                className="px-6 py-3 text-sm tracking-wide cursor-pointer hover:bg-black/5 transition-colors"
              >
                {conv.title}
              </li>
            ))
          )}
        </ul>
      </div>
    </div>
  );
};

export default HistoryPanel;
