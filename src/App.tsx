import { useEffect } from "react";
import "./App.css";
import InputBar from "./components/InputBar";
import Home from "./pages/Home";
import Chat from "./pages/chat/Chat";
import SideLabels from "./components/Sidelabels";
import HistoryPanel from "./components/HistoryPanel";
import { setupChatEventListener } from "./events/chatListener";
import { run, useMessages } from "./store/messages";
import { handleKeydown } from "./keymap";
import { SessionSnapshot } from "./types";

function App() {
  const dispatch = useMessages((state) => state.dispatch);
  const messages = useMessages((state) => state.messages);

  useEffect(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  }, []);

  useEffect(() => {
    let isMounted = true;
    let cleanupListener: (() => void) | null = null;

    setupChatEventListener().then((cleanup) => {
      if (!isMounted) {
        cleanup();
      } else {
        cleanupListener = cleanup;
      }
    });

    run<SessionSnapshot>("get_session")
      .then((session) => {
        if (isMounted) {
          console.debug("[mount]", session.messages.map((m) => !!m.thinking));
          dispatch({
            type: "Snapshot",
            ...session,
          });
        }
      })
      .catch((err) => {
        console.error("Failed to recover session on mount:", err);
      });

    return () => {
      isMounted = false;
      if (cleanupListener) {
        cleanupListener();
      }
    };
  }, [dispatch]);

  return (
    <main className="h-screen w-full flex flex-col overflow-hidden relative">
      <SideLabels />
      <HistoryPanel />
      {messages.length ? <Chat /> : <Home />}
      <InputBar />
    </main>
  );
}

export default App;