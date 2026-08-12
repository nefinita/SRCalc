import { useEffect, useMemo, useState } from "react";
import * as api from "../api/commands";
import type {
  BuffConfig,
  ConfigDataDTO,
  DamageRequest,
  SkillResultDTO,
  Team,
} from "../types";
import styles from "./CalcPage.module.css";
import { formatNumber, formatPercent } from "../utils/format";

interface Props {
  team: Team;
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

export default function CalcPage({ team, addToast }: Props) {
  const [config, setConfig] = useState<ConfigDataDTO | null>(null);
  const [enemyId, setEnemyId] = useState("");
  const [buff, setBuff] = useState<BuffConfig>(EMPTY_BUFF);
  const [results, setResults] = useState<Record<string, SkillResultDTO[]>>({});
  const [calculating, setCalculating] = useState(false);

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

  async function handleCalculate() {
    if (!enemy) {
      addToast("请先选择敌方", "error");
      return;
    }
    if (team.members.length === 0) {
      addToast("请先在上方添加队伍成员", "error");
      return;
    }
    setCalculating(true);
    const out: Record<string, SkillResultDTO[]> = {};
    try {
      for (const member of team.members) {
        const req: DamageRequest = {
          config: config!,
          team,
          focus: member.char_id,
          enemy,
          buff,
          coefficient: { def_const: 200, broken_multiplier: 0.9, break_multiplier: 1.0 },
        };
        out[member.char_id] = await api.calculateDamage(req);
      }
      setResults(out);
      addToast("计算完成", "success");
    } catch (e) {
      addToast(String(e), "error");
    } finally {
      setCalculating(false);
    }
  }

  function setBuffKey(key: keyof BuffConfig, value: number) {
    setBuff((b) => ({ ...b, [key]: value }));
  }

  return (
    <div className={styles.page}>
      <div className={styles.panel}>
        <h2 className={styles.sectionTitle}>敌方与全局增益</h2>
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
        <div className={styles.grid}>
          <NumBuff label="攻击力%" value={buff.atk_pct} onChange={(v) => setBuffKey("atk_pct", v)} />
          <NumBuff label="增伤%" value={buff.dmg_pct} onChange={(v) => setBuffKey("dmg_pct", v)} />
          <NumBuff label="暴击率%" value={buff.crit_rate} onChange={(v) => setBuffKey("crit_rate", v)} />
          <NumBuff label="暴击伤害%" value={buff.crit_dmg} onChange={(v) => setBuffKey("crit_dmg", v)} />
          <NumBuff label="无视防御%" value={buff.def_ignore} onChange={(v) => setBuffKey("def_ignore", v)} />
          <NumBuff label="抗性穿透%" value={buff.res_pen} onChange={(v) => setBuffKey("res_pen", v)} />
          <NumBuff label="易伤%" value={buff.vuln_pct} onChange={(v) => setBuffKey("vuln_pct", v)} />
          <NumBuff label="击破特攻%" value={buff.break_effect} onChange={(v) => setBuffKey("break_effect", v)} />
        </div>

        <button className={styles.primaryBtn} onClick={handleCalculate} disabled={calculating}>
          {calculating ? "计算中…" : "计算全队伤害"}
        </button>
      </div>

      <div className={styles.results}>
        {Object.keys(results).length === 0 ? (
          <div className={styles.empty}>配置队伍并点击「计算全队伤害」。</div>
        ) : (
          team.members.map((member) => {
            const char = config?.characters.find((c) => c.id === member.char_id);
            const rows = results[member.char_id] ?? [];
            return (
              <div key={member.char_id} className={styles.panel}>
                <h2 className={styles.sectionTitle}>
                  {char?.name ?? member.char_id}（{member.build.level} 级）
                </h2>
                <table className={styles.table}>
                  <thead>
                    <tr>
                      <th>技能</th>
                      <th>基础</th>
                      <th>不暴击</th>
                      <th>暴击</th>
                      <th>期望</th>
                      <th>暴击率</th>
                    </tr>
                  </thead>
                  <tbody>
                    {rows.map((r, i) => (
                      <tr key={i}>
                        <td>{r.ability}</td>
                        <td>{formatNumber(r.base)}</td>
                        <td>{formatNumber(r.non_crit)}</td>
                        <td>{formatNumber(r.crit)}</td>
                        <td className={styles.strong}>{formatNumber(r.expected)}</td>
                        <td>{formatPercent(r.crit_rate)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}

function NumBuff({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
}) {
  return (
    <label className={styles.field}>
      <span>{label}</span>
      <input
        type="number"
        step={0.01}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  );
}
