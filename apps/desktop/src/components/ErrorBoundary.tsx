import { Component, type ErrorInfo, type ReactNode } from "react";

interface ErrorBoundaryState {
  error?: Error;
}

export default class ErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  state: ErrorBoundaryState = {};

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("界面渲染失败", error, info.componentStack);
  }

  render() {
    if (!this.state.error) return this.props.children;
    return <main className="fatal-error" role="alert">
      <div>
        <span>应用界面遇到问题</span>
        <h1>当前页面无法继续显示</h1>
        <p>本地数据不会因此丢失。重新载入后仍有问题时，请保留下面的错误信息。</p>
        <pre>{this.state.error.message || this.state.error.name}</pre>
        <button className="button button--primary" onClick={() => window.location.reload()}>重新载入</button>
      </div>
    </main>;
  }
}
