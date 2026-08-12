import { useEffect, useMemo, useState } from "react";
import * as api from "../api/commands";
import type {
  ActionKind,
  BattleConfig,
  ConfigDataDTO,
  MemospriteStepReq,
  RotationRequest,
  RotationResultDTO,
  RotationStepReq,
  Team,
} from "../types";
import styles from "./RotationPage.module.css";
import { formatNumber } from "../utils/format";

interface Props {
  team: Team;
  addToast: (msg: string, type?: "success" | "error" | "info") => void;
}

const ACTION_LABEL: Record<ActionKind, string> = {
  basic: "普攻",
  skill: "战技",
  ult: "终结技",
  wait: "等待",
};

const DEFAULT_BATTLE: BattleConfig = { base_sp_cap: 5, start_sp: 3, start_energy: 0 };

export default function RotationPage({ team, addToast }: Props) {
  const [config, setConfig] = useState<ConfigDataDTO | null>(null);
  const [enemyId, setEnemyId] = useState("");
  const [sequence, setSequence] = useState<RotationStepReq[]>([]);
  const [memoSteps, setMemoSteps] = useState<MemospriteStepReq[]>([]);
  const [battle, setBattle] = useState<BattleConfig>(DEFAULT_BATTLE);
  const [cycles, setCycles] = useState(1);
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

  const teamChars = useMemo(
    () =>
      team.members
        .map((m) => config?.characters.find((c) => c.id === m.char_id))
        .filter(Boolean) as NonNullable<ConfigDataDTO["characters"]>,
    [team, config]
  );

  function addStep(charId: string, action: ActionKind) {
    const ability = config?.characters
      .find((c) => c.id === charId)
      ?.abilities.find((a) => a.kind === action);
    const needsTarget = ability?.buff?.target === "ally" || ability?.immediate_action;
    const defaultTarget = needsTarget
      ? team.members.find((m) => m.char_id !== charId)?.char_id ?? null
      : null;
    setSequence((s) => [...s, { char_id: charId, action, target: defaultTarget }]);
  }

  function addMemoStep(ownerId: string, abilityIndex: number) {
    setMemoSteps((s) => [...s, { owner_id: ownerId, ability_index: abilityIndex, target: null }]);
  }
  function removeMemoStep(index: number) {
    setMemoSteps((s) => s.filter((_, i) => i !== index));
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

  function patchStep(index: number, patch: Partial<RotationStepReq>) {
    setSequence((s) => s.map((x, i) => (i === index ? { ...x, ...patch } : x)));
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
    if (team.members.length === 0) {
      addToast("请先在上方添加队伍成员", "error");
      return;
    }
    setRunning(true);
    try {
      const req: RotationRequest = {
        config,
        team,
        enemy,
        coefficient: { def_const: 200, broken_multiplier: 0.9, break_multiplier: 1.0 },
        battle,
        steps: sequence,
        memosprite_steps: memoSteps,
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
          <h2 className={styles.sectionTitle}>队伍行动</h2>
          {teamChars.map((c) => (
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
                      title={`SP ${a.skill_point > 0 ? "+" + a.skill_point : a.skill_point} · 能量 ${a.energy_gain}`}
                    >
                      {a.name}
                    </button>
                  ))}
              </div>
            </div>
          ))}
          {teamChars.length === 0 && (
            <div className={styles.empty}>先在上方添加队伍成员</div>
          )}
        </div>

        <div className={styles.panel}>
          <h2 className={styles.sectionTitle}>忆灵行动</h2>
          {teamChars
            .map((c) => ({ c, memos: c.abilities.filter((a) => a.kind === "memosprite") }))
            .filter(({ memos }) => memos.length > 0)
            .map(({ c, memos }) => (
              <div key={c.id} className={styles.charRow}>
                <span className={styles.charName}>{c.name}·忆灵</span>
                <div className={styles.actionBtns}>
                  {memos.map((a, idx) => (
                    <button
                      key={a.name + idx}
                      className={styles.actionBtn}
                      disabled={a.forced}
                      title={a.forced ? "强制触发，不可选" : "加入忆灵行动序列"}
                      onClick={() => addMemoStep(c.id, idx)}
                    >
                      {a.name}
                      {a.forced ? " (强制)" : ""}
                    </button>
                  ))}
                </div>
              </div>
            ))}
          {memoSteps.length > 0 && (
            <div className={styles.seqList}>
              {memoSteps.map((m, i) => {
                const c = teamChars.find((x) => x.id === m.owner_id);
                const ab = c?.abilities.filter((a) => a.kind === "memosprite")[m.ability_index];
                return (
                  <li key={i} className={styles.seqItem}>
                    <span className={styles.seqChar}>{c?.name}·{ab?.name}</span>
                    <button className={styles.removeBtn} onClick={() => removeMemoStep(i)}>×</button>
                  </li>
                );
              })}
            </div>
          )}
          {teamChars.filter((c) => c.abilities.some((a) => a.kind === "memosprite")).length === 0 && (
            <div className={styles.empty}>队伍无忆灵角色</div>
          )}
        </div>

        <div className={styles.panel}>
          <h2 className={styles.sectionTitle}>敌方与战斗环境</h2>
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
          <div className={styles.grid2}>
            <label className={styles.field}>
              <span>战技点上限</span>
              <input
                type="number"
                value={battle.base_sp_cap}
                onChange={(e) => setBattle((b) => ({ ...b, base_sp_cap: Math.max(1, Number(e.target.value)) }))}
              />
            </label>
            <label className={styles.field}>
              <span>开局战技点</span>
              <input
                type="number"
                value={battle.start_sp}
                onChange={(e) => setBattle((b) => ({ ...b, start_sp: Math.max(0, Number(e.target.value)) }))}
              />
            </label>
            <label className={styles.field}>
              <span>开局能量</span>
              <input
                type="number"
                value={battle.start_energy}
                onChange={(e) => setBattle((b) => ({ ...b, start_energy: Number(e.target.value) }))}
              />
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
          </div>
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
                const c = teamChars.find((x) => x.id === step.char_id);
                return (
                  <li key={i} className={styles.seqItem}>
                    <span className={styles.seqIndex}>{i + 1}</span>
                    <span className={styles.seqChar}>{c?.name ?? step.char_id}</span>
                    <span className={styles.seqAction}>{ACTION_LABEL[step.action]}</span>
                    {step.action !== "wait" && team.members.length > 1 && (
                      <select
                        className={styles.targetSel}
                        value={step.target ?? ""}
                        onChange={(e) => patchStep(i, { target: e.target.value || null })}
                        title="目标"
                      >
                        <option value="">—</option>
                        {team.members
                          .filter((m) => m.char_id !== step.char_id)
                          .map((m) => (
                            <option key={m.char_id} value={m.char_id}>
                              {teamChars.find((x) => x.id === m.char_id)?.name}
                            </option>
                          ))}
                      </select>
                    )}
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
                    <th>单位</th>
                    <th>动作</th>
                    <th>伤害</th>
                    <th>能量</th>
                    <th>战技点</th>
                  </tr>
                </thead>
                <tbody>
                  {result.steps.map((s, i) => (
                    <tr key={i} className={s.is_enemy ? styles.enemyRow : undefined}>
                      <td>{s.av.toFixed(1)}</td>
                      <td>{s.is_enemy ? `${s.char_name}·${s.enemy_ability}` : s.char_name}</td>
                      <td>{s.is_enemy ? "敌方行动" : ACTION_LABEL[s.action]}</td>
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
