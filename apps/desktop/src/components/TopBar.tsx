import type { ReactNode } from "react";

type TopBarProps = {
  title: string;
  subtitle: string;
  action?: ReactNode;
};

export function TopBar({ title, subtitle, action }: TopBarProps) {
  return (
    <header className="topbar">
      <div>
        <p className="eyebrow">{subtitle}</p>
        <h2>{title}</h2>
      </div>
      {action}
    </header>
  );
}
