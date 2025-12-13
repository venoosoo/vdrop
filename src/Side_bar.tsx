import React from "react";

interface SidebarProps {
  activeTab: "send" | "receive";
  setActiveTab: React.Dispatch<React.SetStateAction<"send" | "receive">>;
}

function Sidebar({ activeTab, setActiveTab }: SidebarProps) {
  return (
    <aside className="bg-[#1e293b] h-full w-48 flex flex-col gap-1 text-white">
      <div
        className={`ml-5 mt-5 p-2 cursor-pointer font-bold transition ${
          activeTab === "send" ? "text-blue-400" : "hover:text-blue-400"
        }`}
        onClick={() => setActiveTab("send")}
      >
        Send File
      </div>

      <div
        className={`ml-5 p-2 cursor-pointer font-bold transition ${
          activeTab === "receive" ? "text-blue-400" : "hover:text-blue-400"
        }`}
        onClick={() => setActiveTab("receive")}
      >
        Received Files
      </div>
    </aside>
  );
}

export default Sidebar;
