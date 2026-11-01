import React from "react";
import { Message } from "../types";
import ThinkingBlock from "./ThinkingBlock";
import MarkdownMessage from "./MarkdownMessage";
import Copy from "./Copy";

export interface MessageViewProps {
  message: Message;
  index: number;
  isLast: boolean;
  isGenerating: boolean;
  isEditingThis: boolean;
  setMessageRef: (index: number, el: HTMLDivElement | null) => void;
}

export const MessageView = React.memo(function MessageView({
  message,
  index,
  isLast,
  isGenerating,
  isEditingThis,
  setMessageRef,
}: MessageViewProps) {
  const setRef = React.useCallback(
    (el: HTMLDivElement | null) => {
      setMessageRef(index, el);
    },
    [index, setMessageRef]
  );

  if (message.role === "user") {
    return (
      <div
        ref={setRef}
        onClick={async()=> await navigator.clipboard.writeText(message.content)}
        className={`flex justify-end text-right transition-all duration-200 cursor-pointer ${
          isEditingThis ? "panel-card p-4 -m-4" : ""
        }`}
      >
        <p className="w-[70%]">{message.content}</p>
      </div>
    );
  }

  const isProcessing = isGenerating && isLast && !message.content && !message.thinking;
  const hasThinking = !!message.thinking;
  const hasContent = !!message.content;

  return (
    <div ref={setRef}>
      {isProcessing && (
        <span className="opacity-40 text-sm tracking-widest animate-pulse">
          Processing…
        </span>
      )}

      {hasThinking && (
        <ThinkingBlock
          thinking={message.thinking!}
          hasContent={hasContent}
          index={index}
        />
      )}

      {hasContent && (
        <div className="markdown-content">
          <MarkdownMessage content={message.content} />
          <Copy text={message.content} />
        </div>
      )}
    </div>
  );
});

export default MessageView;
