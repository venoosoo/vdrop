import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Pc {
  ip: string;
  name: string;
}

function Send() {
  const [devices, setDevices] = useState<Pc[]>([]);
  const [loading, setLoading] = useState(true);


  const handle_reload = async () => {
    setDevices([]);
    setLoading(true);
    fetchDevices();
  }

  const handle_send = async (ip: string) => {
    try {
      // Open file picker dialog
      const selected = await open({
        multiple: false,
        directory: false,
      });

      // Check if user selected a file
      if (!selected) {
        console.log("No file selected");
        return;
      }

      // Extract file name from path
      const filePath = selected as string;
      const fileName = filePath.split(/[\\/]/).pop() || "unknown_file";

      console.log("Sending file:", fileName, "to", ip);

      // Call the backend command
      await invoke('send_file', {
        ip: ip,
        filePath: filePath,
        fileName: fileName,
      });

      console.log("File sent successfully!");
    } catch (error) {
      console.error("Failed to send file:", error);
    }
  };

  async function fetchDevices() {
      try {
        console.log("Starting scan...");
        const result: Pc[] = await invoke("scan_network");
        setDevices(result);
      } catch (error) {
        console.error("Scan failed:", error);
      } finally {
        setLoading(false);
      }
    } 

  useEffect(() => {
    fetchDevices();
  }, []);

  return (
    <main className="flex-1 p-10 text-white">
        <div className="flex items-center mb-2">
            <h1 className="text-2xl font-bold">Select a Device</h1>

            <div className="ml-auto flex items-center space-x-2">
                <button onClick={handle_reload} className="p-1 rounded hover:bg-gray-700 transition">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-6 h-6">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99" />
                </svg>
                </button>
            </div>
        </div>
      {loading ? (
        <div className="flex flex-col items-center justify-center h-64 opacity-70">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-400"></div>
          <p className="mt-4 text-sm text-gray-400">Scanning network...</p>
        </div>
      ) : (
        <div className="space-y-2">
          {devices.length === 0 ? (
            <p className="text-gray-400">No devices found.</p>
          ) : (
            devices.map((device, index) => (
              <button 
                key={index} 
                className="w-full bg-gray-800 p-4 rounded-3xl hover:bg-gray-700 cursor-pointer transition flex justify-between items-center"
                onClick={() => handle_send(device.ip)}
              >
                <span className="font-bold">{device.name}</span>
                <span className="text-sm text-gray-400">{device.ip}</span>
              </button>
            ))
          )}
        </div>
      )}
    </main>
  );
}

export default Send;