import { useState } from "react";
import "./App.css";
import Sidebar from "./Side_bar";
import Send from "./send_file";
import Receive from "./recived";

function App() {
  // Track which tab is active
  const [activeTab, setActiveTab] = useState<"send" | "receive">("send");

  return (
    <main className="text-white flex h-screen w-screen bg-[#0f172a]">
      <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} />

      <div className="flex-1 overflow-auto">
        {activeTab === "send" && <Send />}
        {activeTab === "receive" && <Receive />}
      </div>
    </main>
  );
}

export default App;
