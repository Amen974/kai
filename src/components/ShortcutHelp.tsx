import { keymap } from "../keymap";
import { useUI } from "../store/ui";

const ShortcutHelp = () => {
  const overlay = useUI((state) => state.overlay);
  const setOverlay = useUI((state) => state.setOverlay);

  if (overlay !== "help") return null;

  return (
    <div className="fixed inset-0 z-layer-overlay flex items-center justify-center">
      <div
        className="absolute inset-0 bg-black/30"
        onClick={() => setOverlay(null)}
      />

      <div className="panel-card relative w-[45vw] max-h-[70vh] flex flex-col z-layer-modal overflow-hidden">
        <div className="px-6 py-4 flex items-center justify-between border-b">
          <p className="text-xs tracking-widest uppercase">Keyboard Shortcuts</p>
          <button
            onClick={() => setOverlay(null)}
            type="button"
            className="text-xs opacity-50 hover:opacity-100 cursor-pointer"
          >
            Esc
          </button>
        </div>

        <div className="p-6 overflow-y-auto no-scrollbar space-y-3">
          {keymap.map((item, idx) => (
            <div
              key={idx}
              className="flex items-center justify-between text-sm py-1 border-b border-black/5 last:border-0"
            >
              <span className="opacity-80">{item.label}</span>
              <kbd className="px-2 py-0.5 text-xs font-mono bg-black/5 rounded border border-black/10">
                {item.keys}
              </kbd>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default ShortcutHelp;
