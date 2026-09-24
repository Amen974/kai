import ReactMarkdown from "react-markdown";
import useMessages from "../../store/messages";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import useKeyboardShortcut from "../../hooks/useKeyboardShortcut";
import EditInput from "../../components/EditInput";

const Chat = () => {
  const messages = useMessages().messages;
  const bottomRef = useRef<HTMLDivElement>(null);
  const setLoading = useMessages().setIsloading;
  const [editIndex, setEditIndex] = useState(0);
  const [editMode, setEditMode] = useState(false);
  const [editingMessage, setEditingMessage] = useState("");

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ block: "end" });
  }, [messages]);

  useKeyboardShortcut("ctrl+s",() => {
    cancelMessage();
    setLoading(false);
  })

  useKeyboardShortcut("ctrl+e",() => {
    const nextEditIndex = messages.length - 2;
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

  return (
    <div ref={bottomRef} className="h-screen w-[60vw] absolute left-1/2 -translate-x-1/2 flex flex-col gap-10 overflow-y-scroll no-scrollbar pt-20">
      {messages.map((message, i) => {
        return message.role === 'user' ? (
          <div key={i} className="flex justify-end text-right">
            <p className="w-[70%]">{message.content}</p>
          </div>
        ) : (
          <div key={i}>
              <ReactMarkdown>
                {message.content}
              </ReactMarkdown>
          </div>
        )
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