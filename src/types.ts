export type Message = {
  role: "user" | "assistant";
  content: string;
  thinking?: string;
};

export type Conversation = {
  id: number;
  title: string;
};

export type Invoke = {
  sendMessage: "send_message",
  cancelToken: "cancel_token",
  editMessage: "edit_message",
}

export type Listen = {
  updateMessage: "update_message",
  updateMessageContent: "update_message_content",
  updateMessages: "update_messages",
  emitDone: "emit_done",
}