import { useEffect } from "react";

function useKeyboardShortcut(
    shortcut: string,
    callback: () => void
) {
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            const keys = shortcut.toLowerCase().split("+");

            const key = keys[keys.length - 1];

            const ctrlRequired = keys.includes("ctrl");
            const shiftRequired = keys.includes("shift");
            const altRequired = keys.includes("alt");

            if (
                e.key.toLowerCase() === key &&
                e.ctrlKey === ctrlRequired &&
                e.shiftKey === shiftRequired &&
                e.altKey === altRequired
            ) {
                callback();
            }
        };

        window.addEventListener("keydown", handleKeyDown);

        return () => {
            window.removeEventListener("keydown", handleKeyDown);
        };
    }, [shortcut, callback]);
}

export default useKeyboardShortcut;