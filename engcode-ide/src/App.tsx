import { useState } from "react";
import Editor, { Monaco } from "@monaco-editor/react";
import { invoke } from "@tauri-apps/api/core";
import { englishCodeLanguage, englishCodeConfig, englishCodeTokens } from "./englishcode-language";
import "./App.css";

function App() {
  const [code, setCode] = useState(`create a database called "MyApp"

create these collections in it
  users
  posts

insert into users with name "Alice" and age 30

select all from users

show "Hello from EnglishCode!"`);

  const [output, setOutput] = useState("Ready to run EnglishCode...");
  const [running, setRunning] = useState(false);

  const handleEditorChange = (value: string | undefined) => {
    setCode(value || "");
  };

  const handleEditorMount = (_editor: any, monaco: Monaco) => {
    // Register EnglishCode language
    monaco.languages.register(englishCodeLanguage);
    monaco.languages.setMonarchTokensProvider("englishcode", englishCodeTokens as any);
    monaco.languages.setLanguageConfiguration("englishcode", englishCodeConfig);
  };

  const runCode = async () => {
    setRunning(true);
    setOutput("Running...");

    try {
      const result = await invoke<string>("run_engcode", { code });
      setOutput(result);
    } catch (error) {
      setOutput(`Error: ${error}`);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="ide-container">
      <div className="toolbar">
        <h1>EnglishCode IDE</h1>
        <button onClick={runCode} className="run-button" disabled={running}>
          {running ? "⏳ Running..." : "▶ Run"}
        </button>
      </div>

      <div className="editor-container">
        <Editor
          height="100%"
          defaultLanguage="englishcode"
          value={code}
          onChange={handleEditorChange}
          onMount={handleEditorMount}
          theme="vs-dark"
          options={{
            fontSize: 14,
            minimap: { enabled: false },
            lineNumbers: "on",
            scrollBeyondLastLine: false,
            wordWrap: "on",
            automaticLayout: true,
          }}
        />
      </div>

      <div className="output-container">
        <div className="output-header">Output</div>
        <div className="output-content">
          <pre>{output}</pre>
        </div>
      </div>
    </div>
  );
}

export default App;
