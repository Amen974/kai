import React from "react";
import { Message } from "../types";
import ThinkingBlock from "./ThinkingBlock";
import MarkdownMessage from "./MarkdownMessage";
import Copy from "./Copy";
import BranchNavigator from "./BranchNavigator";
import { AttachmentChip } from "./AttachmentChip";
import { parentKey, useMessages } from "../store/messages";

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

  const parentKeyStr = parentKey(message.parent_message_id);
  const variants = useMessages(
    (state) => message.variants ?? state.variantsByParentId[parentKeyStr]
  );
  const currentIndex =
    variants && message.id
      ? Math.max(0, variants.findIndex((v) => v.id === message.id))
      : (message.current_variant_index ?? 0);

  if (message.role === "user") {
    return (
      <div ref={setRef} className="flex flex-col items-end gap-2">
        <div
          onClick={async () => await navigator.clipboard.writeText(message.content)}
          className={`flex justify-end text-right transition-all duration-200 cursor-pointer w-full ${
            isEditingThis ? "panel-card p-4 -m-4" : ""
          }`}
        >
          <p className="w-[70%]">{message.content}</p>
        </div>
        {message.attachments && message.attachments.length > 0 && (
          <div className="flex flex-wrap justify-end gap-2">
            {message.attachments.map((attachment) => (
              <AttachmentChip key={attachment.id ?? attachment.path} attachment={attachment} />
            ))}
          </div>
        )}
        <BranchNavigator
          variants={variants}
          currentIndex={currentIndex}
        />
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
        <div className="markdown-content group">
          <MarkdownMessage content={message.content} />
          <div className="flex items-center gap-3 opacity-0 transition-opacity duration-150 group-hover:opacity-100 focus-within:opacity-100">
            <Copy text={message.content} />
          </div>
        </div>
      )}
    </div>
  );
});

export default MessageView;
