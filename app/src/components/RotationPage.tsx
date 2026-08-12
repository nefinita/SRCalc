import { useEffect, useMemo, useState } from "react";
import * as api from "../api/commands";
import type {
  ActionKind,
  BuffConfig,
  BuildConfig,
  ConfigDataDTO,
  RotationResultDTO,
  RotationStepReq,
} from "../types";
import styles from "./RotationPage.module.css";
import { formatNumber } from "../utils/format";

interface Props {
  addToast: (msg: string, type?: "success" | "error" | "info") => void;
}

const ACTION_LABEL: Record<ActionKind, string> = {
  basic: "普攻",
  skill: "战技",
  ult: "终结技",
  wait: "等待",
};

const ACTION_SP: Record<ActionKind, string> = {
  basic: "+1",
  skill: "-1",
  ult: "0",
  wait: "0",
};

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

export default function RotationPage({ addToast }: Props) {
  const [config, setConfig] = useState<ConfigDataDTO | null>(null);
  const [enemyId, setEnemyId] = useState("");
  const [sequence, setSequence] = useState<RotationStepReq[]>([]);
  const [cycles, setCycles] = useState(1);
  const [buff, setBuff] = useState<BuffConfig>(EMPTY_BUFF);
  const [result, setResult] = useState<RotationResultDTO | null>(null);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    api.loadConfig().then((cfg) => {
      setConfig(cfg);
      if (cfg.enemies.length) setEnemyId(cfg.enemies[0].id);
    });
  }, []);

  const enemy = useMemo(
    () => config?.enemies.find((e) => e.id === enemyId) ?? null,
    [config, enemyId]
  );

  function addStep(charId: string, action: ActionKind) {
    setSequence((s) => [...s, { char_id: charId, action }]);
  }

  function removeStep(index: number) {
    setSequence((s) => s.filter((_, i) => i !== index));
  }

  function moveStep(index: number, dir: -1 | 1) {
    const target = index + dir;
    if (target < 0 || target >= sequence.length) return;
    setSequence((s) => {
      const copy = [...s];
      [copy[index], copy[target]] = [copy[target], copy[index]];
      return copy;
    });
  }

  async function handleRun() {
    if (!config || !enemy) {
      addToast("请先选择敌方", "error");
      return;
    }
    if (sequence.length === 0) {
      addToast("请先编排行动序列", "error");
      return;
    }
    setRunning(true);
    try {
      const builds: Record<string, BuildConfig> = {};
      for (const id of new Set(sequence.map((s) => s.char_id))) {
        builds[id] = {
          level: 80,
          light_cone: null,
          relic_sets: [],
          main_stats: [],
          substats: {},
          traces: {},
        };
      }
      const req = {
        config,
        builds,
        enemy,
        buff,
        coefficient: { def_const: 200, broken_multiplier: 0.9, break_multiplier: 1.0 },
        steps: sequence,
        cycles,
      };
      const r = await api.calculateRotation(req);
      setResult(r);
      addToast("排轴完成", "success");
    } catch (e) {
      addToast(String(e), "error");
    } finally {
      setRunning(false);
    }
  }

  return (
    <div className={styles.page}>
      <div className={styles.left}>
        <div className={styles.panel}>
          <h2 className={styles.sectionTitle}>角色行动</h2>
          {config?.characters.map((c) => (
            <div key={c.id} className={styles.charRow}>
              <span className={styles.charName}>{c.name}</span>
              <div className={styles.actionBtns}>
                {c.abilities
                  .filter((a) => a.kind !== "talent")
                  .map((a) => (
                    <button
                      key={a.kind}
                      className={styles.actionBtn}
                      onClick={() => addStep(c.id, a.kind as ActionKind)}
                      title={`SP ${ACTION_SP[a.kind as ActionKind]}`}
                    >
                      {a.name}
                    </button>
                  ))}
              </div>
            </div>
          ))}
        </div>

        <div className={styles.panel}>
          <h2 className={styles.sectionTitle}>敌方与循环</h2>
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
          <label className={styles.field}>
            <span>循环次数</span>
            <input
              type="number"
              min={1}
              value={cycles}
              onChange={(e) => setCycles(Math.max(1, Number(e.target.value)))}
            />
          </label>
          <label className={styles.field}>
            <span>增益攻击%</span>
            <input
              type="number"
              step={0.01}
              value={buff.atk_pct}
              onChange={(e) => setBuff((b) => ({ ...b, atk_pct: Number(e.target.value) }))}
            />
          </label>
        </div>
      </div>

      <div className={styles.center}>
        <div className={styles.panel}>
          <h2 className={styles.sectionTitle}>行动序列</h2>
          {sequence.length === 0 ? (
            <div className={styles.empty}>从左侧添加行动</div>
          ) : (
            <ol className={styles.seqList}>
              {sequence.map((step, i) => {
                const c = config?.characters.find((x) => x.id === step.char_id);
                return (
                  <li key={i} className={styles.seqItem}>
                    <span className={styles.seqIndex}>{i + 1}</span>
                    <span className={styles.seqChar}>{c?.name ?? step.char_id}</span>
                    <span className={styles.seqAction}>{ACTION_LABEL[step.action]}</span>
                    <span className={styles.seqSp}>SP {ACTION_SP[step.action]}</span>
                    <div className={styles.seqBtns}>
                      <button onClick={() => moveStep(i, -1)}>↑</button>
                      <button onClick={() => moveStep(i, 1)}>↓</button>
                      <button onClick={() => removeStep(i)}>×</button>
                    </div>
                  </li>
                );
              })}
            </ol>
          )}
          <button
            className={styles.primaryBtn}
            onClick={handleRun}
            disabled={running || sequence.length === 0}
          >
            {running ? "模拟中…" : "运行排轴"}
          </button>
        </div>

        <div className={styles.panel}>
          <h2 className={styles.sectionTitle}>时间轴与伤害</h2>
          {!result ? (
            <div className={styles.empty}>运行后展示时间轴</div>
          ) : (
            <>
              <table className={styles.table}>
                <thead>
                  <tr>
                    <th>AV</th>
                    <th>角色</th>
                    <th>动作</th>
                    <th>伤害</th>
                    <th>能量</th>
                    <th>战技点</th>
                  </tr>
                </thead>
                <tbody>
                  {result.steps.map((s, i) => (
                    <tr key={i}>
                      <td>{s.av.toFixed(1)}</td>
                      <td>{s.char_name}</td>
                      <td>{ACTION_LABEL[s.action]}</td>
                      <td className={styles.strong}>{formatNumber(s.damage)}</td>
                      <td>{s.energy.toFixed(0)}</td>
                      <td>{s.skill_point}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <div className={styles.totalRow}>
                总伤害 <b>{formatNumber(result.total_damage)}</b> · 总行动值{" "}
                <b>{result.total_av.toFixed(1)}</b>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
