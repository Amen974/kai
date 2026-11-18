import { useEffect, useRef } from "react";
import { useMessages } from "../store/messages";
import { useUI } from "../store/ui";
import { formatSize } from "../format";

const DownloadPanel = () => {
  const isOpen = useUI((state) => state.overlay === "downloads");
  const setOverlay = useUI((state) => state.setOverlay);
  const query = useUI((state) => state.downloadQuery);
  const setQuery = useUI((state) => state.setDownloadQuery);

  const download = useMessages((state) => state.download);
  const error = useMessages((state) => state.error);

  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      inputRef.current?.focus();
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const percent =
    download && download.total > 0
      ? Math.min(100, (download.completed / download.total) * 100)
      : 0;

  return (
    <div className="fixed inset-0 z-layer-overlay flex items-center justify-center">
      <div className="absolute inset-0 bg-black/30" onClick={() => setOverlay(null)} />

      <div className="panel-card relative w-[50vw] flex flex-col z-layer-modal overflow-hidden">
        <div className="px-6 py-3 border-b border-(--color-border)">
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Model name"
            className="w-full bg-transparent outline-none text-sm tracking-wide placeholder:text-(--color-text-muted)"
          />
        </div>

        {download && (
          <div className="border-t border-(--color-border) px-6 py-3">
            <div className="flex items-baseline justify-between gap-4">
              <span className="text-xs tracking-wide truncate">{download.name}</span>
              <span className="shrink-0 font-mono text-[0.55rem] tracking-[0.15em] text-(--color-text-muted)">
                {download.total > 0 ? `${percent.toFixed(0)}%` : download.status.toUpperCase()}
              </span>
            </div>
            <div className="mt-2 h-0.5 w-full bg-black/10">
              <div className="h-full bg-(--color-text-main)" style={{ width: `${percent}%` }} />
            </div>
            {download.total > 0 && (
              <div className="mt-1 font-mono text-[0.6rem] tracking-widest text-(--color-text-muted)">
                {formatSize(download.completed)} / {formatSize(download.total)}
              </div>
            )}
          </div>
        )}

        {error && (
          <div className="border-t border-(--color-border) px-6 py-3 text-xs tracking-wide text-(--color-error)">
            {error}
          </div>
        )}
      </div>
    </div>
  );
};

export default DownloadPanel;
