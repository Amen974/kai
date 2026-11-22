import { Attachment } from "../types";

type AttachmentChipProps = {
  attachment: Attachment;
  onRemove?: () => void;
};

const CHIP =
  "inline-flex items-center gap-1.5 label text-[0.55rem] tracking-[0.15em] uppercase border border-(--color-border) px-2 py-0.5 max-w-52";

export const AttachmentChip = ({ attachment, onRemove }: AttachmentChipProps) => {
  const label = (
    <span className="truncate">
      {attachment.kind === "image" ? "Image" : "Text"} · {attachment.name}
    </span>
  );

  if (!onRemove) {
    return <span className={`${CHIP} text-(--color-text-muted)`}>{label}</span>;
  }

  return (
    <button
      type="button"
      onClick={onRemove}
      title={attachment.mime_type}
      aria-label={`Remove ${attachment.name}`}
      className={`group ${CHIP} text-(--color-text-secondary) cursor-pointer transition-colors hover:text-(--color-text-main) hover:border-(--color-text-muted)`}
    >
      {label}
      <span aria-hidden="true" className="shrink-0 opacity-50 group-hover:opacity-100">
        ×
      </span>
    </button>
  );
};
