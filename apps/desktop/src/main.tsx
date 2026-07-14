import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { ThemeProvider } from "./hooks/useTheme";
import { InterviewFlowProvider } from "./hooks/useInterviewFlow";
import App from "./app/App";
import "./styles/index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider>
      <InterviewFlowProvider>
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </InterviewFlowProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
