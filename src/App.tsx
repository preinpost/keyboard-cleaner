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
      <h3>{isRunning === true? "키보드 입력 방지 상태 입니다." : "키보드 입력 가능 상태 입니다."}</h3>

      <div className="row">

        {isRunning === false ? (
          <button onClick={start}>입력 차단</button>
        ) : (
            <button onClick={stop}>입력 차단 해제</button>
        )}
      </div>
    </main>
  );
}

export default App;
