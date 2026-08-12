import { useEffect, useMemo, useState } from "react";
import * as api from "../api/commands";
import type {
  BuildConfig,
  BuffConfig,
  ConfigDataDTO,
  SkillResultDTO,
} from "../types";
import styles from "./CalcPage.module.css";
import { formatNumber, formatPercent } from "../utils/format";

interface Props {
  addToast: (msg: string, type?: "success" | "error" | "info") => void;
  onResultChange: (r: SkillResultDTO[]) => void;
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

export default function CalcPage({ addToast, onResultChange }: Props) {
  const [config, setConfig] = useState<ConfigDataDTO | null>(null);
  const [charId, setCharId] = useState("");
  const [coneId, setConeId] = useState("");
  const [level, setLevel] = useState(80);
  const [enemyId, setEnemyId] = useState("");
  const [buff, setBuff] = useState<BuffConfig>(EMPTY_BUFF);
  const [substats, setSubstats] = useState({ crit_rate: 0.3, crit_dmg: 0.8, atk_pct: 0.3 });
  const [results, setResults] = useState<SkillResultDTO[]>([]);
  const [calculating, setCalculating] = useState(false);

  useEffect(() => {
    api
      .loadConfig()
      .then((cfg) => {
        setConfig(cfg);
        if (cfg.characters.length) setCharId(cfg.characters[0].id);
        if (cfg.enemies.length) setEnemyId(cfg.enemies[0].id);
        if (cfg.light_cones.length) setConeId(cfg.light_cones[0].id);
      })
      .catch((e) => addToast(String(e), "error"));
  }, [addToast]);

  const character = useMemo(
    () => config?.characters.find((c) => c.id === charId),
    [config, charId]
  );
  const enemy = useMemo(
    () => config?.enemies.find((e) => e.id === enemyId),
    [config, enemyId]
  );

  function buildBuild(): BuildConfig {
    return {
      level,
      light_cone: config?.light_cones.find((c) => c.id === coneId)?.id ?? null,
      relic_sets: [],
      main_stats: [],
      substats: {
        atk_pct: substats.atk_pct,
        crit_rate: substats.crit_rate,
        crit_dmg: substats.crit_dmg,
      },
      traces: {},
    };
  }

  async function handleCalculate() {
    if (!character || !enemy) {
      addToast("请先选择角色与敌方", "error");
      return;
    }
    setCalculating(true);
    try {
      const req = {
        config,
        char_id: charId,
        ability_kind: "basic",
        build: buildBuild(),
        enemy,
        buff,
        coefficient: { def_const: 200, broken_multiplier: 0.9, break_multiplier: 1.0 },
      };
      const r = await api.calculateDamage(req);
      setResults(r);
      onResultChange(r);
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
        <h2 className={styles.sectionTitle}>角色与配装</h2>
        <div className={styles.grid}>
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
            <span>角色等级</span>
            <input
              type="number"
              min={1}
              max={90}
              value={level}
              onChange={(e) => setLevel(Number(e.target.value))}
            />
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
        </div>

        <h2 className={styles.sectionTitle}>副词条</h2>
        <div className={styles.grid}>
          <label className={styles.field}>
            <span>暴击率</span>
            <input
              type="number"
              step={0.01}
              value={substats.crit_rate}
              onChange={(e) =>
                setSubstats((s) => ({ ...s, crit_rate: Number(e.target.value) }))
              }
            />
          </label>
          <label className={styles.field}>
            <span>暴击伤害</span>
            <input
              type="number"
              step={0.01}
              value={substats.crit_dmg}
              onChange={(e) =>
                setSubstats((s) => ({ ...s, crit_dmg: Number(e.target.value) }))
              }
            />
          </label>
          <label className={styles.field}>
            <span>攻击力%</span>
            <input
              type="number"
              step={0.01}
              value={substats.atk_pct}
              onChange={(e) =>
                setSubstats((s) => ({ ...s, atk_pct: Number(e.target.value) }))
              }
            />
          </label>
        </div>

        <h2 className={styles.sectionTitle}>增益 / 减益（对全局生效）</h2>
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
          {calculating ? "计算中…" : "计算伤害"}
        </button>
      </div>

      <div className={styles.panel}>
        <h2 className={styles.sectionTitle}>伤害结果</h2>
        {results.length === 0 ? (
          <div className={styles.empty}>尚未计算。配置后点击「计算伤害」。</div>
        ) : (
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
              {results.map((r, i) => (
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
