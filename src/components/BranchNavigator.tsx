import React from "react";
import { switchBranch } from "../store/messages";
import { MessageVariant } from "../types";

export interface BranchNavigatorProps {
  variants?: MessageVariant[];
  currentIndex?: number;
}

export const BranchNavigator = React.memo(function BranchNavigator({
  variants,
  currentIndex = 0,
}: BranchNavigatorProps) {
  if (!variants || variants.length <= 1) {
    return null;
  }

  const total = variants.length;
  const prevBranchId = currentIndex > 0 ? variants[currentIndex - 1]?.branch_id : null;
  const nextBranchId =
    currentIndex < total - 1 ? variants[currentIndex + 1]?.branch_id : null;

  const handlePrev = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (prevBranchId !== null && prevBranchId !== undefined) {
      switchBranch(prevBranchId);
    }
  };

  const handleNext = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (nextBranchId !== null && nextBranchId !== undefined) {
      switchBranch(nextBranchId);
    }
  };

  return (
    <div
      onClick={(e) => e.stopPropagation()}
      className="inline-flex items-center gap-1 text-xs label text-(--color-text-muted) select-none pt-1"
    >
      <button
        type="button"
        disabled={currentIndex <= 0}
        onClick={handlePrev}
        aria-label="Previous version"
        className="p-1 rounded transition-colors hover:text-(--color-text-main) disabled:opacity-25 disabled:cursor-not-allowed cursor-pointer"
      >
        <svg
          aria-hidden="true"
          viewBox="0 0 24 24"
          className="size-3.5"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M15 19l-7-7 7-7" />
        </svg>
      </button>

      <span className="tabular-nums tracking-widest px-0.5">
        {currentIndex + 1} / {total}
      </span>

      <button
        type="button"
        disabled={currentIndex >= total - 1}
        onClick={handleNext}
        aria-label="Next version"
        className="p-1 rounded transition-colors hover:text-(--color-text-main) disabled:opacity-25 disabled:cursor-default cursor-pointer"
      >
        <svg
          aria-hidden="true"
          viewBox="0 0 24 24"
          className="size-3.5"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
        </svg>
      </button>
    </div>
  );
});

export default BranchNavigator;
