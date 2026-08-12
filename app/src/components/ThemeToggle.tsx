import styles from "./ThemeToggle.module.css";

interface Props {
  theme: "dark" | "light";
  onToggle: () => void;
}

export default function ThemeToggle({ theme, onToggle }: Props) {
  return (
    <button
      className={styles.btn}
      onClick={onToggle}
      title={theme === "dark" ? "切换亮色" : "切换暗色"}
    >
      {theme === "dark" ? "☀️" : "🌙"}
    </button>
  );
}
