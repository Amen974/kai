import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <div
      data-tauri-drag-region
      className="fixed inset-x-0 top-0 z-50 h-6"
    />
    <App />
  </React.StrictMode>,
);

