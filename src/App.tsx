import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function useChatStream(
  setResponse: React.Dispatch<React.SetStateAction<string>>,
  setIsLoading: React.Dispatch<React.SetStateAction<boolean>>,
) {
  useEffect(() => {
    const chunkListener = listen<string>("chat-chunk", (event) => {
      setResponse((current) => current + event.payload);
    });

    const doneListener = listen("chat-done", () => {
      setIsLoading(false);
    });

    return () => {
      chunkListener.then((cleanup) => cleanup());
      doneListener.then((cleanup) => cleanup());
    };
  }, [setResponse, setIsLoading]);
}

function App() {
  const inputRef = useRef<HTMLInputElement>(null);

  const [response, setResponse] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  useChatStream(setResponse, setIsLoading);

  const sendMessage = async () => {
    const message = inputRef.current?.value.trim();

    if (!message || isLoading) return;

    setIsLoading(true);
    setError("");
    setResponse("");

    try {
      await invoke("send_message", { message });
    } catch (error) {
      setError(
        error instanceof Error ? error.message : `Something went wrong ${error}.`,
      );
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

        <div className="flex flex-1 items-center overflow-auto py-10">
          {!response && !error && !isLoading && (
            <div>
              <p className="font-serif text-2xl">Ask something to begin.</p>

              <p className="mt-2 text-sm text-white/50">
                Your prompt will be sent to qwen3:4b locally.
              </p>
            </div>
          )}

          {isLoading && !response && (
            <p className="text-[#b3d84c]">Thinking...</p>
          )}

          {error && <p className="text-red-400">{error}</p>}

          {response && (
            <p className="w-full whitespace-pre-wrap border-l-2 border-[#b3d84c] px-6 py-4 leading-7">
              {response}
            </p>
          )}
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
            onClick={sendMessage}
            disabled={isLoading}
            className="bg-[#b3d84c] px-5 font-bold text-[#151912] disabled:opacity-50"
          >
            {isLoading ? "..." : "Send →"}
          </button>
        </div>
      </section>
    </main>
  );
}

export default App;
