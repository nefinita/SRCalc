import { useEffect, useMemo, useState } from "react";
import * as api from "../api/commands";
import type { ConfigDataDTO, OptimizeRequest, OptimizeResultDTO, Team } from "../types";
import styles from "./OptimizePage.module.css";
import { formatNumber } from "../utils/format";

interface Props {
  team: Team;
  addToast: (msg: string, type?: "success" | "error" | "info") => void;
}

export default function OptimizePage({ team, addToast }: Props) {
  const [config, setConfig] = useState<ConfigDataDTO | null>(null);
  const [enemyId, setEnemyId] = useState("");
  const [focus, setFocus] = useState("");
  const [result, setResult] = useState<OptimizeResultDTO | null>(null);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    api.loadConfig().then((cfg) => {
      setConfig(cfg);
      if (cfg.enemies.length) setEnemyId(cfg.enemies[0].id);
    });
  }, []);

  useEffect(() => {
    if (!focus && team.members.length > 0) setFocus(team.members[0].char_id);
  }, [team, focus]);

  const focusName = useMemo(
    () => config?.characters.find((c) => c.id === focus)?.name ?? "",
    [config, focus]
  );

  async function handleRun() {
    const enemy = config?.enemies.find((e) => e.id === enemyId);
    if (!enemy) {
      addToast("请先选择敌方", "error");
      return;
    }
    if (!focus || !team.members.some((m) => m.char_id === focus)) {
      addToast("请先在上方添加要优化的角色", "error");
      return;
    }
    setRunning(true);
    try {
      const req: OptimizeRequest = {
        config: config!,
        team,
        focus,
        enemy,
        coefficient: { def_const: 200, broken_multiplier: 0.9, break_multiplier: 1.0 },
      };
      const r = await api.runOptimize(req);
      setResult(r);
      addToast("配装优化完成", "success");
    } catch (e) {
      addToast(String(e), "error");
    } finally {
      setRunning(false);
    }
  }

  return (
    <div className={styles.page}>
      <div className={styles.panel}>
        <h2 className={styles.sectionTitle}>优化目标（队伍上下文中）</h2>
        <label className={styles.field}>
          <span>角色</span>
          <select value={focus} onChange={(e) => setFocus(e.target.value)}>
            {team.members.map((m) => (
              <option key={m.char_id} value={m.char_id}>
                {config?.characters.find((c) => c.id === m.char_id)?.name ?? m.char_id}
              </option>
            ))}
          </select>
        </label>
        <label className={styles.field}>
          <span>敌方</span>
          <select value={enemyId} onChange={(e) => setEnemyId(e.target.value)}>
            {config?.enemies.map((e) => (
              <option key={e.id} value={e.id}>
                {e.name}
              </option>
            ))}
          </select>
        </label>
        <button className={styles.primaryBtn} onClick={handleRun} disabled={running}>
          {running ? "优化中…" : "开始优化"}
        </button>
        <p className={styles.hint}>
          枚举 身体×脚部×位面球×连接绳 主词条组合，以{focusName}期望伤害排序（含队伍在场被动）。
        </p>
      </div>

      <div className={styles.panel}>
        <h2 className={styles.sectionTitle}>推荐配装（Top 8）</h2>
        {!result ? (
          <div className={styles.empty}>点击「开始优化」</div>
        ) : (
          <table className={styles.table}>
            <thead>
              <tr>
                <th>#</th>
                <th>身体</th>
                <th>脚部</th>
                <th>位面球</th>
                <th>连接绳</th>
                <th>期望伤害</th>
              </tr>
            </thead>
            <tbody>
              {result.best.map((b, i) => (
                <tr key={i}>
                  <td>{i + 1}</td>
                  <td>{b.body}</td>
                  <td>{b.feet}</td>
                  <td>{b.sphere}</td>
                  <td>{b.rope}</td>
                  <td className={styles.strong}>{formatNumber(b.expected)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
