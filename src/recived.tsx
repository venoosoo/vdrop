import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ReceivedFile {
  name: string;
  preview: string; // Base64 string for image previews or empty string for others
}

function Receive() {
  const [files, setFiles] = useState<ReceivedFile[]>([]);
  const [loading, setLoading] = useState(true);

  const handle_reload = async () => {
    setFiles([]);
    setLoading(true);
    fetchFiles();
  };

  async function fetchFiles() {
    try {
      const result: ReceivedFile[] = await invoke("get_received");
      setFiles(result);
    } catch (error) {
      console.error("Failed to fetch received files:", error);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    fetchFiles();
  }, []);

  return (
    <main className="flex-1 p-10 text-white">
      <div className="flex items-center mb-4">
        <h1 className="text-2xl font-bold">Received Files</h1>
        <button 
          onClick={handle_reload} 
          className="ml-auto p-1 rounded hover:bg-gray-700 transition"
        >
          ⟳
        </button>
      </div>

      {loading ? (
        <div className="flex flex-col items-center justify-center h-64 opacity-70">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-400"></div>
          <p className="mt-4 text-sm text-gray-400">Loading files...</p>
        </div>
      ) : (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {files.length === 0 ? (
            <p className="text-gray-400 col-span-full">No files received.</p>
          ) : (
            files.map((file, index) => (
              <div 
                key={index} 
                className="bg-gray-800 p-4 rounded-3xl hover:bg-gray-700 cursor-pointer transition flex flex-col items-center"
              >
                {file.preview ? (
                  <img 
                    src={`data:image/png;base64,${file.preview}`} 
                    alt={file.name} 
                    className="h-24 w-24 object-cover rounded-lg mb-2"
                  />
                ) : (
                  <div className="h-24 w-24 bg-gray-600 flex items-center justify-center rounded-lg mb-2 text-gray-400 text-xs">
                    No Preview
                  </div>
                )}
                <span className="font-bold text-center break-words">{file.name}</span>
              </div>
            ))
          )}
        </div>
      )}
    </main>
  );
}

export default Receive;
