import ReactMarkdown from "react-markdown";
import useMessages from "../../store/messages";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import useKeyboardShortcut from "../../hooks/useKeyboardShortcut";
import EditInput from "../../components/EditInput";
import { useNavigate } from "react-router";

const ThinkingBlock = ({ thinking, hasContent }: { thinking: string; hasContent: boolean }) => {
  const [open, setOpen] = useState(true);

  useEffect(() => {
    if (hasContent) setOpen(false);
  }, [hasContent]);

  return (
    <div className="mb-3">
      <button
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-2 text-xs tracking-widest opacity-50 hover:opacity-80 transition-opacity"
      >
        <span
          className={`inline-block transition-transform duration-200 ${open ? "rotate-90" : "rotate-0"}`}
        >
          ▶
        </span>
        {hasContent ? "Thought" : "Thinking…"}
      </button>

      {open && (
        <div className="mt-2 pl-4 border-l border-current opacity-40 text-sm leading-relaxed whitespace-pre-wrap">
          {thinking}
        </div>
      )}
    </div>
  );
};

const Chat = () => {
  const messages = useMessages().messages;
  const isLoading = useMessages().isLoading;
  const bottomRef = useRef<HTMLDivElement>(null);
  const setLoading = useMessages().setIsloading;
  const currentId = useMessages().currentId;
  const [editIndex, setEditIndex] = useState(0);
  const [editMode, setEditMode] = useState(false);
  const [editingMessage, setEditingMessage] = useState("");
  const navigate = useNavigate();

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ block: "end" });
  }, [messages]);

  useKeyboardShortcut("ctrl+s",() => {
    cancelMessage();
    setLoading(false);
  })

  useKeyboardShortcut("ctrl+e",() => {
    let nextEditIndex
    if(messages[messages.length - 1].role === 'user') {
      nextEditIndex = messages.length - 1;
    } else {
      nextEditIndex = messages.length - 2;
    }
    if (nextEditIndex < 0) {
      return;
    }

    setEditIndex(nextEditIndex);
    setEditingMessage(messages[nextEditIndex].content);
    setEditMode(true);
  })

  useKeyboardShortcut("ctrl+ArrowUp",() => {
    if (!editMode || editIndex - 2 < 0) {
      return;
    }

    const nextEditIndex = editIndex - 2;
    setEditIndex(nextEditIndex);
    setEditingMessage(messages[nextEditIndex].content);
  })

  useKeyboardShortcut("ctrl+ArrowDown",() => {
    if (!editMode || editIndex + 2 > messages.length - 2) {
      return;
    }

    const nextEditIndex = editIndex + 2;
    setEditIndex(nextEditIndex);
    setEditingMessage(messages[nextEditIndex].content);
  })

  useKeyboardShortcut("Enter",() => {
    if (!editMode) {
      return;
    }

    setEditMode(false);
    setEditIndex(messages.length - 1);
    editMessage();
  })

    useKeyboardShortcut("ctrl+n",() => {
      navigate('/');
    })

  const cancelMessage = async () => {
    try {
      await invoke("cancel_token");
    } catch (error) {
      console.log(
        error instanceof Error ? error.message : `Something went wrong ${error}.`,
      );
    }
  };

  const editMessage = async () => {
    const payload = { message: editingMessage, index: editIndex };

    try {
      await invoke("edit_message", payload);
      await invoke("send_message", { message: editingMessage });
    } catch (error) {
      console.log(
        error instanceof Error ? error.message : `Something went wrong ${error}.`,
      );
    }
  };

  const resendMessage = async (message: string) => {
    if (currentId === null) return;

    try {
      await invoke("recend_message", { message, id: currentId });
    } catch (error) {
      setLoading(false);
      console.log(
        error instanceof Error ? error.message : `Something went wrong ${error}.`,
      );
    }
  };

  return (
    <div className="h-screen w-[60vw] absolute left-1/2 -translate-x-1/2 flex flex-col gap-10 overflow-y-scroll no-scrollbar pt-20">
      {messages.map((message, i) => {
        const isLast = i === messages.length - 1;

        if (message.role === 'user') {
          return (
            <div key={i} className="flex justify-end text-right">
              <p className="w-[70%]">{message.content}</p>
            </div>
          );
        }

        const isProcessing = isLoading && isLast && !message.content && !message.thinking;
        const hasThinking  = !!message.thinking;
        const hasContent   = !!message.content;

        return (
          <div key={i}>
            {isProcessing && (
              <span className="opacity-40 text-sm tracking-widest animate-pulse">
                Processing…
              </span>
            )}

            {hasThinking && (
              <ThinkingBlock
                thinking={message.thinking!}
                hasContent={hasContent}
              />
            )}

            {hasContent && (
              <ReactMarkdown>{message.content}</ReactMarkdown>
            )}

            {currentId !== null &&
              !isLoading &&
              isLast &&
              i > 0 &&
              messages[i - 1].role === "user" && (
                <button
                  onClick={() => resendMessage(messages[i - 1].content)}
                  className="mt-3 text-xs tracking-widest opacity-40 hover:opacity-80 transition-opacity"
                >
                  Resend
                </button>
              )}
          </div>
        );
      })}
      <div ref={bottomRef} className="h-10"></div>
      <EditInput
        editingMessage={editingMessage}
        setEditingMessage={setEditingMessage}
        editMode={editMode}
      />
    </div>
  );
};

export default Chat;
