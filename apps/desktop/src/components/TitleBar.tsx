import { useCallback, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

const win = getCurrentWindow();

function MinusIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden>
      <rect x="0.5" y="5" width="11" height="1.6" rx="0.8" fill="currentColor" />
    </svg>
  );
}

function MaximizeIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden>
      <rect x="1" y="1" width="10" height="10" rx="1.2" fill="none" stroke="currentColor" strokeWidth="1.3" />
    </svg>
  );
}

function RestoreIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden>
      <rect x="1" y="2.5" width="7.5" height="7.5" rx="0.8" fill="none" stroke="currentColor" strokeWidth="1.2" />
      <rect x="3" y="0.5" width="7.5" height="7.5" rx="0.8" fill="var(--surface)" stroke="currentColor" strokeWidth="1.2" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden>
      <path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
    </svg>
  );
}

export default function TitleBar() {
  const [maximized, setMaximized] = useState(false);
  const [focused, setFocused] = useState(true);
  const [changingWindowState, setChangingWindowState] = useState(false);

  const syncMaximizedState = useCallback(async () => {
    setMaximized(await win.isMaximized());
  }, []);

  useEffect(() => {
    let disposed = false;
    const unlisten: (() => void)[] = [];
    (async () => {
      const [initialMaximized, initialFocused, stopFocusListener, stopResizeListener] = await Promise.all([
        win.isMaximized(),
        win.isFocused(),
        win.onFocusChanged(({ payload }) => {
          if (!disposed) setFocused(payload);
        }),
        win.onResized(() => {
          if (!disposed) void syncMaximizedState();
        }),
      ]);

      if (disposed) {
        stopFocusListener();
        stopResizeListener();
        return;
      }

      setMaximized(initialMaximized);
      setFocused(initialFocused);
      unlisten.push(stopFocusListener, stopResizeListener);
    })().catch((error) => console.error("无法初始化窗口控制", error));

    return () => {
      disposed = true;
      unlisten.forEach((fn) => fn());
    };
  }, [syncMaximizedState]);

  const handleToggleMaximize = async () => {
    if (changingWindowState) return;
    setChangingWindowState(true);
    try {
      await win.toggleMaximize();
      await syncMaximizedState();
    } catch (error) {
      console.error("窗口最大化/还原失败", error);
    } finally {
      setChangingWindowState(false);
    }
  };

  const handleDragMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    if (event.detail === 2) {
      handleToggleMaximize();
      return;
    }
    void win.startDragging();
  };

  return (
    <header className={`titlebar ${maximized ? "is-maximized" : ""} ${focused ? "is-focused" : "is-blurred"}`}>
      <div className="titlebar-drag-region" onMouseDown={handleDragMouseDown}>
        <span className="titlebar-title">投了吗</span>
        <span className="titlebar-subtitle">Applied Yet?</span>
      </div>
      <div className="titlebar-controls">
        <button type="button" className="tb-btn" onClick={() => void win.minimize()} aria-label="最小化" title="最小化">
          <MinusIcon />
        </button>
        <button type="button" className="tb-btn" onClick={handleToggleMaximize} disabled={changingWindowState} aria-label={maximized ? "还原" : "最大化"} title={maximized ? "还原" : "最大化"}>
          {maximized ? <RestoreIcon /> : <MaximizeIcon />}
        </button>
        <button type="button" className="tb-btn tb-close" onClick={() => void win.close()} aria-label="关闭" title="关闭">
          <CloseIcon />
        </button>
      </div>
    </header>
  );
}
