export type Message = {
  role: "user" | "assistant";
  content: string;
  thinking?: string | null;
};

export type SessionSnapshot = {
  history_id: number | null;
  messages: Message[];
  generating: boolean;
};

export type ChatEvent =
  | {
      type: "Snapshot";
      history_id?: number | null;
      messages: Message[];
      generating?: boolean;
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
    };

export type Conversation = {
  id: number;
  title: string;
};