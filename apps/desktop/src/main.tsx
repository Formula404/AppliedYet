import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { ThemeProvider } from "./hooks/useTheme";
import { InterviewFlowProvider } from "./hooks/useInterviewFlow";
import App from "./app/App";
import ErrorBoundary from "./components/ErrorBoundary";
import "./styles/index.css";

const root = document.getElementById("root");
if (!root) throw new Error("应用根节点不存在");

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <ErrorBoundary>
      <ThemeProvider>
        <InterviewFlowProvider>
          <BrowserRouter>
            <App />
          </BrowserRouter>
        </InterviewFlowProvider>
      </ThemeProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);
