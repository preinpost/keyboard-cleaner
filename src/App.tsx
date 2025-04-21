import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { listen } from "@tauri-apps/api/event";

function App() {

  const [isRunning, setIsRunning] = useState(false);

  async function start() {
    await invoke("start_command")
    setIsRunning(true)
  }
  async function stop() {
    await invoke("stop_command")
  }

  useEffect(() => {
    const unlisten = listen("keyboard_unblocked", () => {
      setIsRunning(false);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);


  return (
    <main className="container">
      <h1>Keyboard Cleaner</h1>
      <h3>{isRunning === true ? "Keyboard input is blocked." : "Keyboard input is enabled."}</h3>

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
