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
  wait: "—",
};

const DEFAULT_BATTLE: BattleConfig = { base_sp_cap: 5, start_sp: 3, start_energy: 0 };

export default function RotationPage({ team, addToast }: Props) {
  const [config, setConfig] = useState<ConfigDataDTO | null>(null);
  const [enemyId, setEnemyId] = useState("");
  const [battle, setBattle] = useState<BattleConfig>(DEFAULT_BATTLE);
  const [naturalAv, setNaturalAv] = useState(3000);
  const [generated, setGenerated] = useState<RotationStepReq[]>([]);
  const [memoSteps, setMemoSteps] = useState<MemospriteStepReq[]>([]);
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

  async function run(reqSteps: RotationStepReq[], natural: number) {
    if (!config || !enemy) {
      addToast("请先选择敌方", "error");
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
        steps: reqSteps,
        memosprite_steps: memoSteps,
        natural_until_av: natural,
        cycles: 1,
      };
      const r = await api.calculateRotation(req);
      setResult(r);
      if (natural > 0 && r.generated_steps.length) {
        setGenerated(r.generated_steps);
        addToast(`已生成全普攻轴（${r.generated_steps.length} 个行动）`, "success");
      }
    } catch (e) {
      addToast(String(e), "error");
    } finally {
      setRunning(false);
    }
  }

  function generate() {
    setGenerated([]);
    run([], naturalAv);
  }

  function changeAction(index: number, action: ActionKind) {
    const next = generated.map((s, i) => (i === index ? { ...s, action } : s));
    setGenerated(next);
    run(next, 0); // 脚本模式重跑，动态重排后续
  }

  function addMemoStep(ownerId: string, abilityIndex: number) {
    setMemoSteps((s) => [...s, { owner_id: ownerId, ability_index: abilityIndex, target: null }]);
  }
  function removeMemoStep(index: number) {
    setMemoSteps((s) => s.filter((_, i) => i !== index));
  }

  // 时间轴中的玩家行动下标 → generated 下标
  const playerIndex = useMemo(() => {
    const map = new Map<number, number>();
    let gi = 0;
    result?.steps.forEach((s, i) => {
      if (!s.is_enemy && !s.buffs.includes("忆灵攻击")) {
        map.set(i, gi);
        gi++;
      }
    });
    return map;
  }, [result]);

  const memoRows = useMemo(
    () =>
      teamChars
        .map((c) => ({ c, memos: c.abilities.filter((a) => a.kind === "memosprite") }))
        .filter(({ memos }) => memos.length > 0),
    [teamChars]
  );

  return (
    <div className={styles.page}>
      <div className={styles.left}>
        <div className={styles.panel}>
          <h2 className={styles.sectionTitle}>自动排轴</h2>
          <p className={styles.hint}>按角色速度动态生成全普攻轴，点击时间轴上的行动可换成其他技能，后续自动重排。</p>
          <label className={styles.field}>
            <span>生成到 AV</span>
            <input
              type="number"
              min={100}
              value={naturalAv}
              onChange={(e) => setNaturalAv(Math.max(100, Number(e.target.value)))}
            />
          </label>
          <button className={styles.primaryBtn} onClick={generate} disabled={running}>
            {running ? "生成中…" : "生成全普攻轴"}
          </button>
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
          </div>
        </div>

        {memoRows.length > 0 && (
          <div className={styles.panel}>
            <h2 className={styles.sectionTitle}>忆灵行动</h2>
            {memoRows.map(({ c, memos }) => (
              <div key={c.id} className={styles.charRow}>
                <span className={styles.charName}>{c.name}·忆灵</span>
                <div className={styles.actionBtns}>
                  {memos.map((a, idx) => (
                    <button
                      key={a.name + idx}
                      className={styles.actionBtn}
                      disabled={a.forced}
                      title={a.forced ? "强制触发" : "加入忆灵行动序列"}
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
          </div>
        )}
      </div>

      <div className={styles.center}>
        <div className={styles.panel}>
          <h2 className={styles.sectionTitle}>时间轴（点击行动更换技能）</h2>
          {!result ? (
            <div className={styles.empty}>点击「生成全普攻轴」开始</div>
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
                  {result.steps.map((s, i) => {
                    const isPlayer = !s.is_enemy && !s.buffs.includes("忆灵攻击");
                    const gi = playerIndex.get(i);
                    return (
                      <tr key={i} className={s.is_enemy ? styles.enemyRow : undefined}>
                        <td>{s.av.toFixed(1)}</td>
                        <td>{s.is_enemy ? `${s.char_name}·${s.enemy_ability}` : s.char_name}</td>
                        <td>
                          {isPlayer && gi !== undefined ? (
                            <select
                              className={styles.targetSel}
                              value={generated[gi]?.action ?? "basic"}
                              onChange={(e) => changeAction(gi, e.target.value as ActionKind)}
                            >
                              {teamChars
                                .find((c) => c.id === generated[gi]?.char_id)
                                ?.abilities.filter((a) => a.kind !== "talent")
                                .map((a) => (
                                  <option key={a.kind} value={a.kind}>
                                    {a.name}
                                  </option>
                                ))}
                            </select>
                          ) : s.is_enemy ? (
                            "敌方行动"
                          ) : (
                            ACTION_LABEL[s.action]
                          )}
                        </td>
                        <td className={styles.strong}>{formatNumber(s.damage)}</td>
                        <td>{s.energy.toFixed(0)}</td>
                        <td>{s.skill_point}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
              <div className={styles.totalRow}>
                总伤害 <b>{formatNumber(result.total_damage)}</b> · 总行动值{" "}
                <b>{result.total_av.toFixed(1)}</b> · 玩家行动{" "}
                <b>{generated.length}</b>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
