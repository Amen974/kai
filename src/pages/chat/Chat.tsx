import { useCallback, useEffect, useLayoutEffect, useRef } from "react";
import MessageView from "../../components/MessageView";
import { useMessages } from "../../store/messages";
import { useUI } from "../../store/ui";

const SCROLL_THRESHOLD = 40;

const Chat = () => {
  const messages = useMessages((state) => state.messages);
  const isGenerating = useMessages((state) => state.isGenerating);

  const mode = useUI((state) => state.mode);
  const overlay = useUI((state) => state.overlay);
  const editIndex = useUI((state) => state.editIndex);
  const scrollPinSignal = useUI((state) => state.scrollPinSignal);

  const scrollerRef = useRef<HTMLDivElement>(null);
  const isAtBottomRef = useRef(true);
  const messageRefs = useRef<(HTMLDivElement | null)[]>([]);

  const setMessageRef = useCallback((index: number, el: HTMLDivElement | null) => {
    messageRefs.current[index] = el;
  }, []);

  // Focus scroller when active layer returns to idle so native keys (PageUp/Down/Home/End/Arrows) scroll
  useEffect(() => {
    if (mode === "idle" && overlay === null) {
      scrollerRef.current?.focus({ preventScroll: true });
    }
  }, [mode, overlay]);

  // Handle scroll events to detect if user is near bottom
  const handleScroll = () => {
    const el = scrollerRef.current;
    if (!el) return;
    const distanceToBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    isAtBottomRef.current = distanceToBottom <= SCROLL_THRESHOLD;
  };

  // Auto-scroll on messages change if pinned to bottom
  useLayoutEffect(() => {
    const el = scrollerRef.current;
    if (!el) return;
    if (isAtBottomRef.current) {
      el.scrollTop = el.scrollHeight;
    }
  }, [messages]);

  // ResizeObserver on the scroller: re-pin when InputBar opens, closes, or grows
  useEffect(() => {
    const el = scrollerRef.current;
    if (!el) return;

    const observer = new ResizeObserver(() => {
      if (isAtBottomRef.current) {
        el.scrollTop = el.scrollHeight;
      }
    });

    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // Force-pin on send, resend, or edit commit
  useEffect(() => {
    if (scrollPinSignal > 0) {
      isAtBottomRef.current = true;
      const el = scrollerRef.current;
      if (el) {
        el.scrollTop = el.scrollHeight;
      }
    }
  }, [scrollPinSignal]);

  // Edit mode: scroll target message into view with block: "nearest"
  useEffect(() => {
    if (mode === "edit" && editIndex !== null) {
      const targetEl = messageRefs.current[editIndex];
      targetEl?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [mode, editIndex]);

  return (
    <div
      ref={scrollerRef}
      tabIndex={-1}
      onScroll={handleScroll}
      className="flex-1 min-h-0 w-full overflow-y-auto no-scrollbar outline-none"
    >
      <div className="mx-auto w-(--column) flex flex-col gap-10 py-10">
        {messages.map((message, i) => (
          <MessageView
            key={i}
            message={message}
            index={i}
            isLast={i === messages.length - 1}
            isGenerating={isGenerating}
            isEditingThis={mode === "edit" && editIndex === i}
            setMessageRef={setMessageRef}
          />
        ))}
      </div>
    </div>
  );
};

export default Chat;
