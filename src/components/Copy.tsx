const Copy = ({text}: {text: string}) => {

  return (
    <button
      type="button"
      onClick={async()=> await navigator.clipboard.writeText(text)}
      className="muted-text inline-flex items-center justify-center p-1 transition-colors hover:text-(--color-text-main) focus-visible:outline-1 focus-visible:outline-(--color-text-muted) cursor-pointer"
    >
      <svg
        aria-hidden="true"
        viewBox="0 0 24 24"
        className="size-4"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
      >
        <rect x="8" y="8" width="11" height="11" rx="1" />
        <path d="M16 8V5a1 1 0 0 0-1-1H5a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h3" />
      </svg>
    </button>
  );
};

export default Copy;