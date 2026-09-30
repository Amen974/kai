import { useEffect, useRef, useState } from "react";
import useKeyboardShortcut from "../hooks/useKeyboardShortcut";
import { useLocation, useNavigate } from "react-router";
import { invoke } from "@tauri-apps/api/core";
import useMessages from "../store/messages";

const MAX_HEIGHT = 200

const ChatInput = () => {
  const [open, setOpen] = useState(false);
  const [inputValue, setInputValue] = useState("");
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const currentId = useMessages().currentId;


  const location = useLocation();
  const navigate = useNavigate();

  useKeyboardShortcut("ctrl+m",() => {
    setOpen(true);
  })

  useKeyboardShortcut("escape",() => {
    setOpen(false);
  })

  const handleSend = async (message: string) => {
    if (!open || !message.trim()) return;

    setOpen(false);
    setInputValue("");

    if (location.pathname === "/") navigate("/chat");

    try {
      await invoke("send_message", { message, id: currentId ?? undefined });
    } catch (error) {
      console.log(error);
    }
  };

  const resize = () => {
    const el = inputRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, MAX_HEIGHT)}px`;
  };

  useEffect(() => {
    const input = inputRef.current
  
    if (open) input?.focus();

    const length = input?.value.length ?? 0

    input?.setSelectionRange(length, length);

  },[open]);


  if (!open) return null

  return (
    <div className={`highlight border rounded-sm px-6 py-1 absolute shadow-[0_2px_10px_rgba(43,42,40,0.12)] w-[55vw] min-h-[9vh] flex items-center z-999 ${location.pathname === '/' ? 'top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2' : 'bottom-10 left-1/2 -translate-x-1/2'}`}>
      <textarea
        rows={1}
        value={inputValue}
        onChange={(e) => {
          setInputValue(e.target.value);
          resize();
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            handleSend(inputValue);
          }
        }}
        ref={inputRef}
        className="no-scrollbar overflow-y-auto resize-none outline-none disabled:opacity-50 text-[1.2vw] leading-6 tracking-widest placeholder:text-[#858078] w-full"
        placeholder="Type something..."
      />
    </div>
  );
};

export default ChatInput;