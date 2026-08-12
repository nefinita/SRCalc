import styles from "./ActivityBar.module.css";
import { clsx } from "../utils/clsx";

export type PageKey = "calc" | "rotation" | "optimize" | "editor";

const PAGES: { key: PageKey; icon: string; label: string }[] = [
  { key: "calc", icon: "⚔️", label: "伤害计算" },
  { key: "rotation", icon: "⏱️", label: "排轴" },
  { key: "optimize", icon: "🎯", label: "配装优化" },
  { key: "editor", icon: "📝", label: "数据编辑" },
];

interface Props {
  currentPage: PageKey;
  onNavigate: (page: PageKey) => void;
}

export default function ActivityBar({ currentPage, onNavigate }: Props) {
  return (
    <nav className={styles.bar}>
      {PAGES.map((p) => (
        <button
          key={p.key}
          className={clsx(
            styles.item,
            currentPage === p.key && styles.active
          )}
          onClick={() => onNavigate(p.key)}
          title={p.label}
        >
          <span className={styles.icon}>{p.icon}</span>
          <span className={styles.label}>{p.label}</span>
        </button>
      ))}
    </nav>
  );
}
