import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import useMessages from "../store/messages";
import { Message } from "../types";

export const useMessageSync = () => {
  const addMessage = useMessages().addMessage;
  const addChunk = useMessages().addChunk;
  const addThinkingChunk = useMessages().addThinkingChunk;
  const setMessages = useMessages().setMessages;
  const setLoading = useMessages().setIsloading;

  useEffect(() => {
    let active = true;

    const setup = async () => {
      const unlistenNew = await listen<Message>("update_message", (event) => {
        if (active) {
            addMessage(event.payload);
            setLoading(true);
        }
      });

      const unlistenContent = await listen<string>(
        "update_message_content",
        (event) => {
          if (active) addChunk(event.payload);
        },
      );

      const unlistenThinking = await listen<string>(
        "update_thinking_content",
        (event) => {
          if (active) addThinkingChunk(event.payload);
        },
      );

      const unlistenMessages = await listen<Message[]>(
        "update_messages",
        (event) => {
          if (active) {
            setMessages(event.payload);
          }
        },
      );

      const unlistenDone = await listen<boolean>("emit_done", () => {
        if (active) setLoading(false);
      });

      return () => {
        unlistenNew();
        unlistenContent();
        unlistenThinking();
        unlistenMessages();
        unlistenDone();
      };
    };

    const cleanupPromise = setup();

    return () => {
      active = false;
      void cleanupPromise.then((cleanup) => cleanup());
    };
  }, [addMessage, addChunk, addThinkingChunk, setMessages, setLoading]);
};
