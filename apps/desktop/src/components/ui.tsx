import { forwardRef, type CSSProperties, type ReactNode } from "react";
import { ChevronRight } from "lucide-react";
import type { StatusTone } from "../types";

export function Badge({ children, tone = "gray" }: { children: ReactNode; tone?: StatusTone }) {
  return <span className={`badge badge--${tone}`}>{children}</span>;
}

export const Card = forwardRef<HTMLElement, { children: ReactNode; className?: string; style?: CSSProperties }>(
  function Card({ children, className = "", style }, ref) {
    return <section ref={ref} className={`card ${className}`} style={style}>{children}</section>;
  },
);

export function CardHeader({ title, action, subtitle }: { title: string; action?: string; subtitle?: string }) {
  return (
    <div className="card-header">
      <div><h2>{title}</h2>{subtitle && <span>{subtitle}</span>}</div>
      {action && <button className="text-button">{action}<ChevronRight size={14} /></button>}
    </div>
  );
}

export function PageHeader({ title, description, action }: { title: string; description: string; action?: ReactNode }) {
  return <div className="page-header"><div><h1>{title}</h1><p>{description}</p></div>{action}</div>;
}

export function EmptyState({ icon, title, description, action }: { icon: ReactNode; title: string; description: string; action: string }) {
  return <div className="empty-state"><div className="empty-icon">{icon}</div><h3>{title}</h3><p>{description}</p><button className="button button--secondary">{action}</button></div>;
}
