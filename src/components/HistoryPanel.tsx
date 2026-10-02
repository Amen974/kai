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
  const currentId = useMessages().currentId;
  const setMessages = useMessages().setMessages;
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

  const handleDelete = async (id: number) => {
    try {
      await invoke("delete_history", { id });
      setHistory((currentHistory) => currentHistory.filter((conv) => conv.id !== id));

      if (currentId === id) {
        setCurrentId(null);
        setMessages([]);
        navigate("/");
      }
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
              <li key={conv.id} className="flex items-center hover:bg-black/5 transition-colors">
                <button
                  onClick={() => handleSelect(conv.id)}
                  className="px-6 py-3 text-sm tracking-wide text-left flex-1"
                >
                  {conv.title}
                </button>
                <button
                  onClick={() => handleDelete(conv.id)}
                  className="px-6 py-3 text-xs tracking-wide opacity-50 hover:opacity-100"
                >
                  Delete
                </button>
              </li>
            ))
          )}
        </ul>
      </div>
    </div>
  );
};

export default HistoryPanel;
