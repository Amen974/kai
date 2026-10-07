import { useEffect, useLayoutEffect, useRef } from "react";
import { run, useMessages } from "../store/messages";
import { useUI } from "../store/ui";

const MAX_HEIGHT = 200;

const InputBar = () => {
  const mode = useUI((state) => state.mode);
  const draft = useUI((state) => state.draft);
  const editDraft = useUI((state) => state.editDraft);
  const editIndex = useUI((state) => state.editIndex);
  const setDraft = useUI((state) => state.setDraft);
  const setEditDraft = useUI((state) => state.setEditDraft);
  const setMode = useUI((state) => state.setMode);

  const isGenerating = useMessages((state) => state.isGenerating);
  const error = useMessages((state) => state.error);
  const setError = useMessages((state) => state.setError);

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const isEdit = mode === "edit";
  const isCompose = mode === "compose";
  const isOpen = isEdit || isCompose;
  const currentValue = isEdit ? editDraft : draft;

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
    if (!trimmed || isGenerating) return;

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
    <div className="flex-none w-full pb-6 pt-2">
      <div className="mx-auto w-(--column) flex flex-col">
        {error && (
          <div className="w-full mb-2 text-xs tracking-wider text-(--color-error) px-1 truncate">
            {error}
          </div>
        )}

        <div className="text-[11px] tracking-widest uppercase opacity-40 mb-1 px-1">
          {isEdit ? "Edit · Enter save · Esc cancel" : "Enter send"}
        </div>

        <div className="panel-card px-6 py-1 min-h-[9vh] flex items-center w-full">
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
              if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing && e.keyCode !== 229) {
                e.preventDefault();
                if (isEdit) {
                  handleSaveEdit();
                } else {
                  handleSend();
                }
              }
            }}
            className="no-scrollbar overflow-y-auto max-h-50 resize-none outline-none text-base leading-6 tracking-widest placeholder:text-(--color-text-muted) w-full field-sizing-content"
            placeholder={isEdit ? "Edit message..." : "Type something..."}
          />
        </div>
      </div>
    </div>
  );
};

export default InputBar;
