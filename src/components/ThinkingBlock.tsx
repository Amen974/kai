import { useUI } from "../store/ui";

interface ThinkingBlockProps {
  thinking: string;
  hasContent: boolean;
}

export const ThinkingBlock = ({ thinking, hasContent }: ThinkingBlockProps) => {
  const showThinking = useUI((state) => state.showThinking);
  const toggleThinking = useUI((state) => state.toggleThinking);
  const open = showThinking || !hasContent;

  return (
    <div className="mb-3">
      <button
        onClick={toggleThinking}
        type="button"
        className="flex items-center gap-2 text-xs tracking-widest opacity-50 hover:opacity-80 transition-opacity cursor-pointer select-none"
      >
        <span
          className={`inline-block transition-transform duration-200 ${open ? "rotate-90" : "rotate-0"}`}
        >
          ▶
        </span>
        {hasContent ? "Thought" : "Thinking…"}
      </button>

      {open && (
        <div className="mt-2 pl-4 border-l border-current opacity-40 text-sm leading-relaxed whitespace-pre-wrap font-mono">
          {thinking}
        </div>
      )}
    </div>
  );
};

export default ThinkingBlock;
