import { useState } from "react";
import { useTheme } from "./hooks/useTheme";
import { useToast } from "./hooks/useToast";
import styles from "./App.module.css";
import ThemeToggle from "./components/ThemeToggle";
import ActivityBar, { PageKey } from "./components/ActivityBar";
import Toast from "./components/Toast";
import CalcPage from "./components/CalcPage";
import RotationPage from "./components/RotationPage";
import OptimizePage from "./components/OptimizePage";
import DataEditorPage from "./components/DataEditorPage";
import type { SkillResultDTO } from "./types";

export default function App() {
  const { theme, toggleTheme } = useTheme();
  const { toasts, addToast, removeToast } = useToast();
  const [curPage, setCurPage] = useState<PageKey>("calc");
  const [, setLastResults] = useState<SkillResultDTO[]>([]);

  return (
    <div className={styles.container}>
      <div className={styles.body}>
        <ActivityBar currentPage={curPage} onNavigate={setCurPage} />
        <div className={styles.main}>
          <header className={styles.header}>
            <div className={styles.logo}>星铁排轴计算器</div>
            <div className={styles.headerActions}>
              <ThemeToggle theme={theme} onToggle={toggleTheme} />
            </div>
          </header>
          <div className={styles.page}>
            {curPage === "calc" && (
              <CalcPage addToast={addToast} onResultChange={setLastResults} />
            )}
            {curPage === "rotation" && <RotationPage addToast={addToast} />}
            {curPage === "optimize" && <OptimizePage addToast={addToast} />}
            {curPage === "editor" && <DataEditorPage addToast={addToast} />}
          </div>
        </div>
      </div>
      <Toast toasts={toasts} onRemove={removeToast} />
    </div>
  );
}
