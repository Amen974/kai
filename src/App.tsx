import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

type Message = {
  role: "user" | "assistant";
  content: string;
};

function useChatStream(
  setMessages: React.Dispatch<React.SetStateAction<Message[]>>,
  setIsLoading: React.Dispatch<React.SetStateAction<boolean>>,
) {
  useEffect(() => {
    const messagesListener = listen<Message[]>("update_messages", (event) => {
      setMessages(event.payload);
    });

    const chunkListener = listen<string>("update_message", (event) => {
      setMessages((current) => {
        const messages = [...current];
        const lastMessage = messages[messages.length - 1];

        if (lastMessage?.role === "assistant") {
          messages[messages.length - 1] = {
            ...lastMessage,
            content: lastMessage.content + event.payload,
          };
        } else {
          messages.push({ role: "assistant", content: event.payload });
        }

        return messages;
      });
    });

    const doneListener = listen("emit_done", () => {
      setIsLoading(false);
    });

    return () => {
      messagesListener.then((cleanup) => cleanup());
      chunkListener.then((cleanup) => cleanup());
      doneListener.then((cleanup) => cleanup());
    };
  }, [setMessages, setIsLoading]);
}

function App() {
  const inputRef = useRef<HTMLInputElement>(null);

  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editingMessage, setEditingMessage] = useState("");

  useChatStream(setMessages, setIsLoading);

  const sendMessage = async () => {
    const message = inputRef.current?.value.trim();

    if (!message || isLoading) return;

    setIsLoading(true);
    setError("");
    if (inputRef.current) {
      inputRef.current.value = "";
    }

    try {
      await invoke("send_message", { message });
    } catch (error) {
      setError(
        error instanceof Error ? error.message : `Something went wrong ${error}.`,
      );
      setIsLoading(false);
    }
  };

  const cancelMessage = async () => {
    try {
      await invoke("cancel_token");
    } catch (error) {
      setError(
        error instanceof Error ? error.message : `Something went wrong ${error}.`,
      );
    } finally {
      setIsLoading(false);
    }
  };

  const startEditing = (index: number, message: string) => {
    setEditingIndex(index);
    setEditingMessage(message);
    setError("");
  };

  const saveEditedMessage = async () => {
    const message = editingMessage.trim();

    if (!message || editingIndex === null || isLoading) return;

    setIsLoading(true);
    setError("");

    try {
      await invoke("edit_message", { message, index: editingIndex });
      setEditingIndex(null);
      setEditingMessage("");
      await invoke("send_message", { message });
    } catch (error) {
      setError(
        error instanceof Error ? error.message : `Something went wrong ${error}.`,
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      sendMessage();
    }
  };

  return (
    <main className="min-h-screen bg-[#101312] p-8 text-[#f2f1eb]">
      <section className="mx-auto flex min-h-[calc(100vh-4rem)] max-w-190 flex-col">
        <header className="border-b border-white/10 pb-6">
          <p className="text-xs uppercase tracking-widest text-[#b3d84c]">
            LOCAL MODEL / READY
          </p>

          <h1 className="mt-2 font-serif text-5xl">AI chat</h1>
        </header>

        <div className="flex flex-1 flex-col overflow-auto py-10">
          {messages.length === 0 && !error && !isLoading && (
            <div>
              <p className="font-serif text-2xl">Ask something to begin.</p>

              <p className="mt-2 text-sm text-white/50">
                Your prompt will be sent to qwen3:4b locally.
              </p>
            </div>
          )}

          {isLoading && (
            <p className="text-[#b3d84c]">Thinking...</p>
          )}

          {error && <p className="text-red-400">{error}</p>}

          {messages.map((message, index) => (
            <div
              key={`${message.role}-${index}`}
              className={`mb-6 whitespace-pre-wrap px-6 py-4 leading-7 ${
                message.role === "user"
                  ? "border-l-2 border-white/30"
                  : "border-l-2 border-[#b3d84c]"
              }`}
            >
              <p className="mb-2 text-xs uppercase tracking-widest text-white/40">
                {message.role === "user" ? "You" : "AI"}
              </p>
              {editingIndex === index ? (
                <div className="flex gap-2">
                  <input
                    value={editingMessage}
                    onChange={(event) => setEditingMessage(event.target.value)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter") saveEditedMessage();
                    }}
                    className="min-w-0 flex-1 border border-white/20 bg-black/20 px-3 py-2 outline-none"
                    autoFocus
                  />
                  <button
                    type="button"
                    onClick={saveEditedMessage}
                    className="bg-[#b3d84c] px-3 font-bold text-[#151912]"
                  >
                    Save
                  </button>
                </div>
              ) : (
                <>
                  {message.content || (isLoading && index === messages.length - 1
                    ? "Thinking..."
                    : "")}
                  {message.role === "user" && !isLoading && (
                    <button
                      type="button"
                      onClick={() => startEditing(index, message.content)}
                      className="ml-4 text-xs text-white/50 underline underline-offset-4 hover:text-white"
                    >
                      Edit
                    </button>
                  )}
                </>
              )}
            </div>
          ))}
        </div>

        <div className="flex gap-2 border border-white/15 bg-black/20 p-2">
          <input
            ref={inputRef}
            type="text"
            placeholder="Ask a question..."
            className="min-w-0 flex-1 bg-transparent px-3 py-2 outline-none"
            onKeyDown={handleKeyDown}
            disabled={isLoading}
          />

          <button
            type="button"
            onClick={isLoading ? cancelMessage : sendMessage}
            className="bg-[#b3d84c] px-5 font-bold text-[#151912] disabled:opacity-50"
          >
            {isLoading ? "Cancel" : "Send →"}
          </button>
        </div>
      </section>
    </main>
  );
}

export default App;
