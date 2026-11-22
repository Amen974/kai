export type MessageVariant = {
  id: number;
  branch_id: number;
};

export type AttachmentKind = "image" | "text";

export type Attachment = {
  id?: number | null;
  name: string;
  kind: AttachmentKind;
  mime_type: string;
  path: string;
  size_bytes: number;
};

export type Message = {
  id?: number | null;
  parent_message_id?: number | null;
  role: "user" | "assistant";
  content: string;
  thinking?: string | null;
  branch_id?: number | null;
  attachments?: Attachment[];
  variants?: MessageVariant[];
  current_variant_index?: number;
};

export type SessionSnapshot = {
  history_id: number | null;
  messages: Message[];
  generating: boolean;
  active_branch_id?: number | null;
};

export type ChatEvent =
  | {
      type: "Snapshot";
      history_id?: number | null;
      messages: Message[];
      generating?: boolean;
      active_branch_id?: number | null;
    }
  | {
      type: "ContentDelta";
      content: string;
    }
  | {
      type: "ThinkingDelta";
      thinking: string;
    }
  | {
      type: "GenerationFinished";
      reason: string;
    }
  | {
      type: "Error";
      message: string;
    }
  | {
      type: "HistoryChanged";
    }
  | {
      type: "ModelsChanged";
    }
  | ({ type: "ModelDownloadProgress" } & DownloadProgress);

export type Conversation = {
  id: number;
  title: string;
};

export type DownloadProgress = {
  name: string;
  status: string;
  completed: number;
  total: number;
};

export type ModelInfo = {
  name: string;
  family: string;
  parameter_size: string;
  quantization_level: string;
  size_bytes: number;
  modified_at: string;
  supports_vision: boolean;
  active: boolean;
};