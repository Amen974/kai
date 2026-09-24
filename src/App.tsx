import { Route, Routes } from "react-router";
import "./App.css";
import ChatInput from "./components/ChatInput";
import Home from "./pages/Home";
import Chat from "./pages/chat/Chat";
import SideLabels from "./components/Sidelabels";
import { useMessageSync } from "./hooks/useMessageSync";

function App() {
  useMessageSync();

  return (
    <main className="overflow-hidden min-h-screen w-full">
      <ChatInput />
      <SideLabels />
      
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/chat" element={<Chat />} />
      </Routes>
    </main>
  );
}

export default App;