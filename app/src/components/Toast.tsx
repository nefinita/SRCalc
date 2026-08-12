import { ToastItem } from "../hooks/useToast";
import styles from "./Toast.module.css";
import { clsx } from "../utils/clsx";

interface Props {
  toasts: ToastItem[];
  onRemove: (id: number) => void;
}

const TYPE_ICON: Record<string, string> = {
  success: "✅",
  error: "❌",
  info: "ℹ️",
};

export default function Toast({ toasts, onRemove }: Props) {
  return (
    <div className={styles.stack}>
      {toasts.map((t) => (
        <div key={t.id} className={clsx(styles.toast, styles[t.type])}>
          <span>{TYPE_ICON[t.type]}</span>
          <span className={styles.msg}>{t.message}</span>
          <button className={styles.close} onClick={() => onRemove(t.id)}>
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
