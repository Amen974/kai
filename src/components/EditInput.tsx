import { useEffect, useRef } from "react";


const MAX_HEIGHT = 200

type EditInputProps = {
  editingMessage: string;
  setEditingMessage: (message: string) => void;
  editMode: boolean;
};

const EditInput = ({ editingMessage, setEditingMessage, editMode }: EditInputProps) => {
  const inputRef = useRef<HTMLTextAreaElement>(null);
  
  const resize = () => {
    const el = inputRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, MAX_HEIGHT)}px`;
  };
  
  useEffect(() => {
    const input = inputRef.current;
    console.log("[EditInput] edit mode changed", {
      editMode,
      value: editingMessage,
    });
    if (!input || !editMode) return;

    input.focus();
  
    const length = input.value.length;
    input.setSelectionRange(length, length);
  
  }, [editMode]);


  if (!editMode) return null

  return (
    <div className="highlight border rounded-sm px-6 py-1 absolute shadow-[0_2px_10px_rgba(43,42,40,0.12)] w-[55vw] min-h-[9vh] flex items-center z-999 bottom-10 left-1/2 -translate-x-1/2">
      <textarea
        rows={1}
        value={editingMessage}
        onChange={(e) => {
          console.log("[EditInput] text changed", {
            value: e.target.value,
            length: e.target.value.length,
          });
          setEditingMessage(e.target.value);
          resize();
        }}
        onKeyDown={(e) => {
          console.log("[EditInput] key pressed", {
            key: e.key,
            shiftKey: e.shiftKey,
            value: editingMessage,
          });
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
          }
        }}
        ref={inputRef}
        className="no-scrollbar overflow-y-auto resize-none outline-none disabled:opacity-50 text-[1.2vw] leading-6 tracking-widest placeholder:text-[#858078] w-full"
        placeholder="Type something..."
      />
    </div>
  );
}

export default EditInput