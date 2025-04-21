import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {

  const [isRunning, setIsRunning] = useState(false);
  const [noAccess, setNoAccess] = useState(false);

  async function start() {
    let result = await invoke("start_command");
    
    if (result) {
      setIsRunning(true);
      setNoAccess(false);
    } else {
      setIsRunning(false);
      setNoAccess(true);
    }
  }
  async function stop() {
    let result = await invoke("stop_command");
    
    if (result) {
      setIsRunning(false);
    }
  }

  return (
    <main className="container">
      <h1 style={{ marginBottom: "-2px" }}>Keyboard Cleaner</h1>
      <h3>{isRunning === true ? "Keyboard input is blocked." : "Keyboard input is enabled."}</h3>
      {noAccess && (
        <div style={{ fontSize: "15px", marginBottom: "1em", fontWeight: "bold" }}>
          Accessibility permission is required to block keyboard input.<br />
        </div>
      )}
      <div className="row">
        {isRunning === false ? (
          <button onClick={start}>Block Input</button>
        ) : (
            <button onClick={stop}>Unblock Input</button>
        )}
      </div>
    </main>
  );
}

export default App;
