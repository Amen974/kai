import { useEffect, useLayoutEffect, useRef } from "react";
import { detachAttachment, pickAttachments, run, useMessages } from "../store/messages";
import { useUI } from "../store/ui";
import { AttachmentChip } from "./AttachmentChip";

const MAX_HEIGHT = 200;

const PaperclipIcon = () => (
  <svg
    width="14"
    height="14"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.6"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <path d="M21.44 11.05 12.25 20.24a5.5 5.5 0 0 1-7.78-7.78l8.49-8.49a3.5 3.5 0 0 1 4.95 4.95l-8.49 8.49a1.5 1.5 0 0 1-2.12-2.12l7.78-7.78" />
  </svg>
);

type InputBarProps = {
  isHome: boolean;
};

const InputBar = ({ isHome }: InputBarProps) => {
  const mode = useUI((state) => state.mode);
  const draft = useUI((state) => state.draft);
  const editDraft = useUI((state) => state.editDraft);
  const editIndex = useUI((state) => state.editIndex);
  const setDraft = useUI((state) => state.setDraft);
  const setEditDraft = useUI((state) => state.setEditDraft);
  const setMode = useUI((state) => state.setMode);

  const isGenerating = useMessages((state) => state.isGenerating);
  const attachments = useMessages((state) => state.attachments);
  const error = useMessages((state) => state.error);
  const setError = useMessages((state) => state.setError);

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const isEdit = mode === "edit";
  const isCompose = mode === "compose";
  const isOpen = isEdit || isCompose;
  const currentValue = isEdit ? editDraft : draft;
  const pendingAttachments = isEdit ? [] : attachments;

  const resize = () => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, MAX_HEIGHT)}px`;
  };

  useLayoutEffect(() => {
    if (isOpen) {
      resize();
    }
  }, [currentValue, isOpen, mode]);

  useEffect(() => {
    if (isOpen) {
      const el = textareaRef.current;
      if (el) {
        el.focus();
        const len = el.value.length;
        el.setSelectionRange(len, len);
      }
    }
  }, [isOpen, mode]);

  const handleSend = async () => {
    const trimmed = draft.trim();
    if ((!trimmed && attachments.length === 0) || isGenerating) return;

    setError(null);
    const savedDraft = draft;
    setDraft("");
    setMode("idle");
    useUI.getState().forceScrollPin();

    try {
      await run("send_message", { message: trimmed });
    } catch {
      setDraft(savedDraft);
      setMode("compose");
    }
  };

  const handleSaveEdit = async () => {
    const trimmed = editDraft.trim();
    if (!trimmed || editIndex === null || isGenerating) return;

    setError(null);
    const savedDraft = editDraft;
    const targetIndex = editIndex;
    setMode("idle");
    useUI.getState().forceScrollPin();

    try {
      await run("edit_message", { index: targetIndex, message: trimmed });
    } catch {
      setEditDraft(savedDraft);
      setMode("edit");
    }
  };

  if (!isOpen) {
    if (!error) return null;
    return (
      <div className="flex-none w-full pb-6 pt-2">
        <div className="mx-auto w-(--column)">
          <div className="w-full mb-2 text-xs tracking-wider text-(--color-error) px-1 truncate">
            {error}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={
      isHome
      ? "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full"
      : "flex-none w-full pb-6 pt-2"
    }>
      <div className="mx-auto w-(--column) flex flex-col">
        {error && (
          <div className="w-full mb-2 text-xs tracking-wider text-(--color-error) px-1 truncate">
            {error}
          </div>
        )}

        <div className="panel-card px-6 py-3 min-h-[9vh] flex flex-col justify-center gap-2 w-full">
          {pendingAttachments.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {pendingAttachments.map((attachment) => (
                <AttachmentChip
                  key={attachment.path}
                  attachment={attachment}
                  onRemove={() => detachAttachment(attachment)}
                />
              ))}
            </div>
          )}

          <div className="relative flex flex-col">
            <textarea
              ref={textareaRef}
              rows={1}
              value={currentValue}
              onChange={(e) => {
                if (isEdit) {
                  setEditDraft(e.target.value);
                } else {
                  setDraft(e.target.value);
                }
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
                  e.preventDefault();
                  if (isEdit) {
                    handleSaveEdit();
                  } else {
                    handleSend();
                  }
                }
              }}
              className={`no-scrollbar overflow-y-auto max-h-50 resize-none outline-none text-base leading-6 tracking-widest placeholder:text-(--color-text-muted) w-full field-sizing-content${isCompose ? " pr-9" : ""}`}
              placeholder={isEdit ? "Edit message" : undefined}
            />

            {isCompose && (
              <button
                type="button"
                onClick={() => void pickAttachments()}
                title="Attach a .txt or image"
                aria-label="Attach a file"
                className="absolute right-0 bottom-0 inline-flex size-6 cursor-pointer items-center justify-center text-(--color-text-muted) transition-colors hover:text-(--color-text-main) focus-visible:text-(--color-text-main) focus-visible:outline-none"
              >
                <PaperclipIcon />
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default InputBar;
