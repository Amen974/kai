import { useEffect, useRef } from "react";
import { loadModels, selectModel, useMessages } from "../store/messages";
import { useUI } from "../store/ui";
import { formatSize } from "../format";

const ModelPanel = () => {
  const isOpen = useUI((state) => state.overlay === "models");
  const setOverlay = useUI((state) => state.setOverlay);
  const selectedIndex = useUI((state) => state.modelIndex);

  const models = useMessages((state) => state.models);
  const modelsVersion = useMessages((state) => state.modelsVersion);

  const itemRefs = useRef<(HTMLLIElement | null)[]>([]);

  useEffect(() => {
    loadModels();
  }, [modelsVersion]);

  useEffect(() => {
    if (isOpen) {
      itemRefs.current[selectedIndex]?.scrollIntoView({ block: "nearest" });
    }
  }, [isOpen, selectedIndex]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-layer-overlay flex items-center justify-center">
      <div className="absolute inset-0 bg-black/30" onClick={() => setOverlay(null)} />

      <div className="panel-card relative w-[50vw] max-h-[60vh] flex flex-col z-layer-modal overflow-hidden">
        <ul
          role="listbox"
          tabIndex={-1}
          className="overflow-y-auto no-scrollbar flex-1 outline-none"
        >
          {models.length === 0 ? (
            <li className="px-6 py-4 text-sm tracking-wide opacity-50">No models.</li>
          ) : (
            models.map((model, idx) => {
              const isSelected = selectedIndex === idx;

              return (
                <li
                  key={model.name}
                  ref={(el) => {
                    itemRefs.current[idx] = el;
                  }}
                  role="option"
                  aria-selected={isSelected}
                  onClick={() => {
                    selectModel(idx);
                  }}
                  className={`flex items-center gap-4 px-6 py-3 cursor-pointer transition-colors ${
                    isSelected ? "bg-black/10" : "hover:bg-black/5"
                  }`}
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-baseline gap-2">
                      <span className="text-sm tracking-wide truncate">{model.name}</span>
                      {model.active && (
                        <span className="label text-[0.55rem] tracking-[0.2em] text-(--color-text-muted)">
                          ACTIVE
                        </span>
                      )}
                    </div>
                    <div className="text-xs tracking-wide text-(--color-text-muted) truncate">
                      {model.family} · {model.parameter_size} · {model.quantization_level} ·{" "}
                      {formatSize(model.size_bytes)} · {model.modified_at.slice(0, 10)}
                    </div>
                  </div>

                  <span
                    className={`shrink-0 font-mono text-[0.6rem] tracking-[0.15em] ${
                      model.supports_vision
                        ? "text-(--color-text-secondary)"
                        : "text-(--color-text-muted)"
                    }`}
                  >
                    {model.supports_vision ? "READS IMAGES" : "TEXT ONLY"}
                  </span>
                </li>
              );
            })
          )}
        </ul>
      </div>
    </div>
  );
};

export default ModelPanel;
