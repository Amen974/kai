import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { ChatEvent, Message } from "../types";
import { useMessages } from "../store/messages";

export async function setupChatEventListener(): Promise<() => void> {
  const unlistens: UnlistenFn[] = [];
  const dispatch = (event: ChatEvent) => {
    useMessages.getState().dispatch(event);
  };

  const unlistenChatEvent = await listen<ChatEvent>("chat_event", (event) => {
    dispatch(event.payload);
  });
  unlistens.push(unlistenChatEvent);

  const unlistenMessages = await listen<Message[]>("update_messages", (event) => {
    console.debug("[update_messages]", event.payload.map((m) => !!m.thinking));
    dispatch({
      type: "Snapshot",
      messages: event.payload,
      generating: true,
    });
  });
  unlistens.push(unlistenMessages);

  const unlistenContent = await listen<string>("update_message_content", (event) => {
    dispatch({
      type: "ContentDelta",
      content: event.payload,
    });
  });
  unlistens.push(unlistenContent);

  const unlistenThinking = await listen<string>("update_thinking_content", (event) => {
    dispatch({
      type: "ThinkingDelta",
      thinking: event.payload,
    });
  });
  unlistens.push(unlistenThinking);

  const unlistenDone = await listen<void>("emit_done", () => {
    dispatch({
      type: "GenerationFinished",
      reason: "done",
    });
  });
  unlistens.push(unlistenDone);

  const unlistenError = await listen<string>("chat_error", (event) => {
    dispatch({
      type: "Error",
      message: event.payload,
    });
  });
  unlistens.push(unlistenError);

  const unlistenHistory = await listen<void>("history_changed", () => {
    dispatch({
      type: "HistoryChanged",
    });
  });
  unlistens.push(unlistenHistory);

  return () => {
    for (const unlisten of unlistens) {
      unlisten();
    }
  };
}
