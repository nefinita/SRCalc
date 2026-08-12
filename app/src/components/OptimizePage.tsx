import { useEffect, useMemo, useState } from "react";
import * as api from "../api/commands";
import type {
  BuffConfig,
  BuildConfig,
  ConfigDataDTO,
  OptimizeResultDTO,
} from "../types";
import styles from "./OptimizePage.module.css";
import { formatNumber } from "../utils/format";

interface Props {
  addToast: (msg: string, type?: "success" | "error" | "info") => void;
}

const EMPTY_BUFF: BuffConfig = {
  atk_pct: 0,
  dmg_pct: 0,
  crit_rate: 0,
  crit_dmg: 0,
  def_ignore: 0,
  res_pen: 0,
  vuln_pct: 0,
  break_effect: 0,
  weakness_break_eff: 0,
};

export default function OptimizePage({ addToast }: Props) {
  const [config, setConfig] = useState<ConfigDataDTO | null>(null);
  const [charId, setCharId] = useState("");
  const [coneId, setConeId] = useState("");
  const [enemyId, setEnemyId] = useState("");
  const [result, setResult] = useState<OptimizeResultDTO | null>(null);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    api.loadConfig().then((cfg) => {
      setConfig(cfg);
      if (cfg.characters.length) setCharId(cfg.characters[0].id);
      if (cfg.enemies.length) setEnemyId(cfg.enemies[0].id);
      if (cfg.light_cones.length) setConeId(cfg.light_cones[0].id);
    });
  }, []);

  const character = useMemo(
    () => config?.characters.find((c) => c.id === charId),
    [config, charId]
  );

  function buildBuild(): BuildConfig {
    return {
      level: 80,
      light_cone: config?.light_cones.find((c) => c.id === coneId)?.id ?? null,
      relic_sets: [],
      main_stats: [],
      substats: {},
      traces: {},
    };
  }

  async function handleRun() {
    if (!character || !config) {
      addToast("请先选择角色", "error");
      return;
    }
    const enemy = config.enemies.find((e) => e.id === enemyId);
    if (!enemy) {
      addToast("请先选择敌方", "error");
      return;
    }
    setRunning(true);
    try {
      const req = {
        config,
        char_id: charId,
        build: buildBuild(),
        enemy,
        buff: EMPTY_BUFF,
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
        <h2 className={styles.sectionTitle}>优化目标</h2>
        <label className={styles.field}>
          <span>角色</span>
          <select value={charId} onChange={(e) => setCharId(e.target.value)}>
            {config?.characters.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        </label>
        <label className={styles.field}>
          <span>光锥</span>
          <select value={coneId} onChange={(e) => setConeId(e.target.value)}>
            {config?.light_cones.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
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
          枚举 身体×脚部×位面球×连接绳 主词条组合，以期望伤害排序。
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
