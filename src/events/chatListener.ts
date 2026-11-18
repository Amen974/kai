import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { ChatEvent, DownloadProgress, SessionSnapshot } from "../types";
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

  const unlistenSnapshot = await listen<SessionSnapshot>("update_snapshot", (event) => {
    dispatch({
      type: "Snapshot",
      ...event.payload,
    });
  });
  unlistens.push(unlistenSnapshot);

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

  const unlistenModels = await listen<void>("models_changed", () => {
    dispatch({
      type: "ModelsChanged",
    });
  });
  unlistens.push(unlistenModels);

  const unlistenDownloadProgress = await listen<DownloadProgress>(
    "model_download_progress",
    (event) => {
      dispatch({
        type: "ModelDownloadProgress",
        ...event.payload,
      });
    }
  );
  unlistens.push(unlistenDownloadProgress);

  const unlistenDownloadFinished = await listen<void>("model_download_finished", () => {
    dispatch({ type: "ModelDownloadFinished" });
  });
  unlistens.push(unlistenDownloadFinished);

  return () => {
    for (const unlisten of unlistens) {
      unlisten();
    }
  };
}
