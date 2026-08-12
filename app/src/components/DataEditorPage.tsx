import { useEffect, useMemo, useState } from "react";
import * as api from "../api/commands";
import type {
  AbilityData,
  CharacterDTO,
  ConfigDataDTO,
  Effect,
  EnemyAbility,
  EnemyDTO,
} from "../types";
import styles from "./DataEditorPage.module.css";

interface Props {
  addToast: (msg: string, type?: "success" | "error" | "info") => void;
}

type Tab = "character" | "enemy";

export default function DataEditorPage({ addToast }: Props) {
  const [config, setConfig] = useState<ConfigDataDTO | null>(null);
  const [tab, setTab] = useState<Tab>("character");
  const [charId, setCharId] = useState("");
  const [enemyId, setEnemyId] = useState("");
  const [editingChar, setEditingChar] = useState<CharacterDTO | null>(null);
  const [editingEnemy, setEditingEnemy] = useState<EnemyDTO | null>(null);

  useEffect(() => {
    api.loadConfig().then((cfg) => {
      setConfig(cfg);
      if (cfg.characters.length) {
        setCharId(cfg.characters[0].id);
        setEditingChar(structuredClone(cfg.characters[0]));
      }
      if (cfg.enemies.length) {
        setEnemyId(cfg.enemies[0].id);
        setEditingEnemy(structuredClone(cfg.enemies[0]));
      }
    });
  }, []);

  const characters = useMemo(() => config?.characters ?? [], [config]);
  const enemies = useMemo(() => config?.enemies ?? [], [config]);

  function selectCharacter(id: string) {
    setCharId(id);
    const c = characters.find((x) => x.id === id);
    if (c) setEditingChar(structuredClone(c));
  }

  function selectEnemy(id: string) {
    setEnemyId(id);
    const e = enemies.find((x) => x.id === id);
    if (e) setEditingEnemy(structuredClone(e));
  }

  function patchChar(patch: Partial<CharacterDTO>) {
    setEditingChar((c) => (c ? { ...c, ...patch } : c));
  }

  function patchAbility(index: number, patch: Partial<AbilityData>) {
    setEditingChar((c) => {
      if (!c) return c;
      const abilities = c.abilities.map((a, i) => (i === index ? { ...a, ...patch } : a));
      return { ...c, abilities };
    });
  }

  function patchEffect(index: number, patch: Partial<Effect>) {
    setEditingChar((c) => {
      if (!c) return c;
      const team_effects = c.team_effects.map((e, i) => (i === index ? { ...e, ...patch } : e));
      return { ...c, team_effects };
    });
  }

  function patchEnemy(patch: Partial<EnemyDTO>) {
    setEditingEnemy((e) => (e ? { ...e, ...patch } : e));
  }

  function patchEnemyAction(index: number, patch: Partial<EnemyAbility>) {
    setEditingEnemy((e) => {
      if (!e) return e;
      const actions = e.actions.map((a, i) => (i === index ? { ...a, ...patch } : a));
      return { ...e, actions };
    });
  }

  async function handleSaveChar() {
    if (!editingChar) return;
    try {
      await api.saveCharacter(editingChar);
      addToast(`已保存角色 ${editingChar.name}`, "success");
    } catch (e) {
      addToast(String(e), "error");
    }
  }

  async function handleSaveEnemy() {
    if (!editingEnemy) return;
    try {
      await api.saveEnemy(editingEnemy);
      addToast(`已保存敌方 ${editingEnemy.name}`, "success");
    } catch (e) {
      addToast(String(e), "error");
    }
  }

  return (
    <div className={styles.page}>
      <div className={styles.tabs}>
        <button className={tab === "character" ? styles.tabActive : styles.tab} onClick={() => setTab("character")}>
          角色
        </button>
        <button className={tab === "enemy" ? styles.tabActive : styles.tab} onClick={() => setTab("enemy")}>
          敌方
        </button>
      </div>

      {tab === "character" && (
        <div className={styles.body}>
          <div className={styles.list}>
            {characters.map((c) => (
              <button
                key={c.id}
                className={c.id === charId ? styles.listItemActive : styles.listItem}
                onClick={() => selectCharacter(c.id)}
              >
                {c.name}
              </button>
            ))}
          </div>
          {editingChar && (
            <div className={styles.editor}>
              <div className={styles.row}>
                <Field label="ID" value={editingChar.id} onChange={(v) => patchChar({ id: v })} />
                <Field label="名称" value={editingChar.name} onChange={(v) => patchChar({ name: v })} />
                <label className={styles.field}>
                  <span>命途</span>
                  <select value={editingChar.path} onChange={(e) => patchChar({ path: e.target.value as CharacterDTO["path"] })}>
                    <option value="destruction">毁灭</option>
                    <option value="the_hunt">巡猎</option>
                    <option value="erudition">智识</option>
                    <option value="harmony">同谐</option>
                    <option value="nihility">虚无</option>
                    <option value="preservation">存护</option>
                    <option value="abundance">丰饶</option>
                    <option value="remembrance">记忆</option>
                    <option value="elation">欢愉</option>
                  </select>
                </label>
                <label className={styles.field}>
                  <span>属性</span>
                  <select value={editingChar.element} onChange={(e) => patchChar({ element: e.target.value as CharacterDTO["element"] })}>
                    <option value="physical">物理</option>
                    <option value="fire">火</option>
                    <option value="ice">冰</option>
                    <option value="lightning">雷</option>
                    <option value="wind">风</option>
                    <option value="quantum">量子</option>
                    <option value="imaginary">虚数</option>
                  </select>
                </label>
                <Num label="生命" value={editingChar.base_hp} onChange={(v) => patchChar({ base_hp: v })} />
                <Num label="攻击" value={editingChar.base_atk} onChange={(v) => patchChar({ base_atk: v })} />
                <Num label="防御" value={editingChar.base_def} onChange={(v) => patchChar({ base_def: v })} />
                <Num label="速度" value={editingChar.base_spd} onChange={(v) => patchChar({ base_spd: v })} />
                <label className={styles.field}>
                  <span>拥有忆灵</span>
                  <input type="checkbox" checked={editingChar.has_memosprite} onChange={(e) => patchChar({ has_memosprite: e.target.checked })} />
                </label>
                <Num label="忆灵速度" value={editingChar.memosprite_spd} onChange={(v) => patchChar({ memosprite_spd: v })} />
                <Num label="忆灵倍率" value={editingChar.memosprite_multiplier} onChange={(v) => patchChar({ memosprite_multiplier: v })} />
                <Num label="低血爆炸%" value={editingChar.memosprite_explode_pct * 100} onChange={(v) => patchChar({ memosprite_explode_pct: v / 100 })} />
                <label className={styles.field}>
                  <span>开战召唤</span>
                  <input type="checkbox" checked={editingChar.summon_at_battle_start} onChange={(e) => patchChar({ summon_at_battle_start: e.target.checked })} />
                </label>
              </div>

              <h3 className={styles.subTitle}>技能</h3>
              {editingChar.abilities.map((a, i) => (
                <div key={i} className={styles.abilityCard}>
                  <div className={styles.row}>
                    <Field label="名称" value={a.name} onChange={(v) => patchAbility(i, { name: v })} />
                    <label className={styles.field}>
                      <span>类型</span>
                      <select value={a.kind} onChange={(e) => patchAbility(i, { kind: e.target.value as AbilityData["kind"] })}>
                        <option value="basic">普攻</option>
                        <option value="skill">战技</option>
                        <option value="ult">终结技</option>
                        <option value="talent">天赋</option>
                      </select>
                    </label>
                    <Num label="倍率" value={a.multiplier} onChange={(v) => patchAbility(i, { multiplier: v })} />
                    <Num label="技能等级" value={a.skill_level} onChange={(v) => patchAbility(i, { skill_level: v })} />
                    <Field label="每级倍率(逗号)" value={a.multipliers.join(",")} onChange={(v) => patchAbility(i, { multipliers: v.split(",").map((x) => Number(x.trim())).filter((x) => Number.isFinite(x)) })} />
                    <Num label="削韧" value={a.toughness_reduction} onChange={(v) => patchAbility(i, { toughness_reduction: v })} />
                    <Num label="回能" value={a.energy_gain} onChange={(v) => patchAbility(i, { energy_gain: v })} />
                    <Num label="最大能量" value={a.max_energy} onChange={(v) => patchAbility(i, { max_energy: v })} />
                    <Num label="战技点" value={a.skill_point} onChange={(v) => patchAbility(i, { skill_point: v })} />
                    <Num label="额外战技点" value={a.bonus_sp} onChange={(v) => patchAbility(i, { bonus_sp: v })} />
                    <Num label="行动提前%" value={a.action_advance_pct * 100} onChange={(v) => patchAbility(i, { action_advance_pct: v / 100 })} />
                    <Num label="自身提前%" value={a.self_advance_pct * 100} onChange={(v) => patchAbility(i, { self_advance_pct: v / 100 })} />
                    <label className={styles.field}>
                      <span>立即行动</span>
                      <input type="checkbox" checked={a.immediate_action} onChange={(e) => patchAbility(i, { immediate_action: e.target.checked })} />
                    </label>
                    <label className={styles.field}>
                      <span>施加负面</span>
                      <input type="checkbox" checked={a.applies_debuff} onChange={(e) => patchAbility(i, { applies_debuff: e.target.checked })} />
                    </label>
                    <label className={styles.field}>
                      <span>治疗</span>
                      <input type="checkbox" checked={a.heals} onChange={(e) => patchAbility(i, { heals: e.target.checked })} />
                    </label>
                    <label className={styles.field}>
                      <span>忆灵强制</span>
                      <input type="checkbox" checked={a.forced} onChange={(e) => patchAbility(i, { forced: e.target.checked })} />
                    </label>
                    <Num label="重复施放" value={a.repeat} onChange={(v) => patchAbility(i, { repeat: v })} />
                    <Num label="耗血%" value={a.hp_cost_pct * 100} onChange={(v) => patchAbility(i, { hp_cost_pct: v / 100 })} />
                    <label className={styles.field}>
                      <span>耗尽爆炸</span>
                      <input type="checkbox" checked={a.on_deplete} onChange={(e) => patchAbility(i, { on_deplete: e.target.checked })} />
                    </label>
                    <label className={styles.field}>
                      <span>召唤忆灵</span>
                      <input type="checkbox" checked={a.summons_memo} onChange={(e) => patchAbility(i, { summons_memo: e.target.checked })} />
                    </label>
                  </div>
                  {a.buff && (
                    <div className={styles.effectBox}>
                      <span className={styles.effectTitle}>施放buff</span>
                      <EffectFields
                        eff={a.buff}
                        onChange={(p) => patchAbility(i, { buff: { ...a.buff!, ...p } })}
                        onRemove={() => patchAbility(i, { buff: null })}
                      />
                    </div>
                  )}
                  {!a.buff && (
                    <button className={styles.miniBtn} onClick={() => patchAbility(i, { buff: emptyEffect() })}>
                      + 添加 buff
                    </button>
                  )}
                </div>
              ))}

              <h3 className={styles.subTitle}>在场被动（team_effects）</h3>
              {editingChar.team_effects.map((e, i) => (
                <div key={i} className={styles.effectBox}>
                  <EffectFields
                    eff={e}
                    onChange={(p) => patchEffect(i, p)}
                    onRemove={() => patchChar({ team_effects: editingChar.team_effects.filter((_, j) => j !== i) })}
                  />
                </div>
              ))}
              <button className={styles.miniBtn} onClick={() => patchChar({ team_effects: [...editingChar.team_effects, emptyEffect()] })}>
                + 添加在场被动
              </button>

              <button className={styles.saveBtn} onClick={handleSaveChar}>
                保存角色
              </button>
            </div>
          )}
        </div>
      )}

      {tab === "enemy" && (
        <div className={styles.body}>
          <div className={styles.list}>
            {enemies.map((e) => (
              <button
                key={e.id}
                className={e.id === enemyId ? styles.listItemActive : styles.listItem}
                onClick={() => selectEnemy(e.id)}
              >
                {e.name}
              </button>
            ))}
          </div>
          {editingEnemy && (
            <div className={styles.editor}>
              <div className={styles.row}>
                <Field label="ID" value={editingEnemy.id} onChange={(v) => patchEnemy({ id: v })} />
                <Field label="名称" value={editingEnemy.name} onChange={(v) => patchEnemy({ name: v })} />
                <Num label="等级" value={editingEnemy.level} onChange={(v) => patchEnemy({ level: v })} />
                <Num label="防御" value={editingEnemy.def} onChange={(v) => patchEnemy({ def: v })} />
                <Num label="最大韧性" value={editingEnemy.max_toughness} onChange={(v) => patchEnemy({ max_toughness: v })} />
                <Num label="速度" value={editingEnemy.spd} onChange={(v) => patchEnemy({ spd: v })} />
                <Num label="生命(>0启用击杀)" value={editingEnemy.hp} onChange={(v) => patchEnemy({ hp: v })} />
              </div>
              <h3 className={styles.subTitle}>弱点属性（仅弱点攻击削韧）</h3>
              <div className={styles.weakRow}>
                {(["physical", "fire", "ice", "lightning", "wind", "quantum", "imaginary"] as const).map((el) => (
                  <label key={el} className={styles.weakItem}>
                    <input
                      type="checkbox"
                      checked={editingEnemy.weaknesses.includes(el)}
                      onChange={(e) => {
                        const set = new Set(editingEnemy.weaknesses);
                        if (e.target.checked) set.add(el);
                        else set.delete(el);
                        patchEnemy({ weaknesses: [...set] });
                      }}
                    />
                    {el}
                  </label>
                ))}
              </div>
              <h3 className={styles.subTitle}>抗性</h3>
              <div className={styles.row}>
                {(Object.keys(editingEnemy.res) as (keyof typeof editingEnemy.res)[]).map((k) => (
                  <Num
                    key={k}
                    label={k}
                    value={editingEnemy.res[k]}
                    onChange={(v) => patchEnemy({ res: { ...editingEnemy.res, [k]: v } })}
                  />
                ))}
              </div>
              <h3 className={styles.subTitle}>行动（回能/回SP/扣能）</h3>
              {editingEnemy.actions.map((a, i) => (
                <div key={i} className={styles.abilityCard}>
                  <div className={styles.row}>
                    <Field label="名称" value={a.name} onChange={(v) => patchEnemyAction(i, { name: v })} />
                    <Num label="我方回能" value={a.energy_gain_players} onChange={(v) => patchEnemyAction(i, { energy_gain_players: v })} />
                    <Num label="战技点变化" value={a.sp_delta} onChange={(v) => patchEnemyAction(i, { sp_delta: v })} />
                    <Num label="扣能" value={a.energy_drain} onChange={(v) => patchEnemyAction(i, { energy_drain: v })} />
                  </div>
                </div>
              ))}
              <button
                className={styles.miniBtn}
                onClick={() =>
                  patchEnemy({
                    actions: [...editingEnemy.actions, { name: "新行动", energy_gain_players: 10, sp_delta: 0, energy_drain: 0 }],
                  })
                }
              >
                + 添加行动
              </button>
              <button className={styles.saveBtn} onClick={handleSaveEnemy}>
                保存敌方
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function emptyEffect(): Effect {
  return {
    trigger: "on_use",
    stat: "atk_pct",
    value: 0,
    turns: 1,
    target: "team",
    cap_bonus: 0,
    sp_on_basic: 0,
    max_stacks: 0,
  };
}

function EffectFields({
  eff,
  onChange,
  onRemove,
}: {
  eff: Effect;
  onChange: (p: Partial<Effect>) => void;
  onRemove: () => void;
}) {
  return (
    <div className={styles.effectRow}>
      <label className={styles.field}>
        <span>触发</span>
        <select value={eff.trigger} onChange={(e) => onChange({ trigger: e.target.value as Effect["trigger"] })}>
          <option value="on_use">施放</option>
          <option value="on_sp_consume">消耗战技点</option>
          <option value="battle_start">进场常驻</option>
          <option value="on_ult">终结技后</option>
          <option value="on_skill">战技后</option>
          <option value="on_basic">普攻后</option>
          <option value="on_hit">受击</option>
          <option value="turn_start">回合开始</option>
          <option value="on_follow_up">追加攻击后</option>
          <option value="on_attack">攻击命中</option>
          <option value="on_apply_debuff">施加负面</option>
          <option value="on_heal">治疗</option>
          <option value="on_kill">消灭敌人</option>
          <option value="on_targeted">成为技能目标</option>
          <option value="on_memosprite_attack">忆灵攻击</option>
        </select>
      </label>
      <label className={styles.field}>
        <span>属性</span>
        <select value={eff.stat} onChange={(e) => onChange({ stat: e.target.value as Effect["stat"] })}>
          <option value="atk_pct">攻击%</option>
          <option value="hp_pct">生命%</option>
          <option value="def_pct">防御%</option>
          <option value="speed_pct">速度%</option>
          <option value="crit_rate">暴击率</option>
          <option value="crit_dmg">暴伤</option>
          <option value="dmg_pct">增伤</option>
          <option value="def_ignore">无视防御</option>
          <option value="res_pen">抗穿</option>
          <option value="vuln_pct">易伤</option>
          <option value="break_effect">击破特攻</option>
          <option value="energy_regen">充能效率</option>
          <option value="ult_dmg_pct">终结技伤害</option>
          <option value="skill_dmg_pct">战技伤害</option>
          <option value="basic_dmg_pct">普攻伤害</option>
          <option value="follow_up_dmg_pct">追加攻击伤害</option>
        </select>
      </label>
      <Num label="值" value={eff.value} onChange={(v) => onChange({ value: v })} />
      <Num label="回合" value={eff.turns} onChange={(v) => onChange({ turns: v })} />
      <label className={styles.field}>
        <span>目标</span>
        <select value={eff.target} onChange={(e) => onChange({ target: e.target.value as Effect["target"] })}>
          <option value="self">自己</option>
          <option value="team">全队</option>
          <option value="ally">单体</option>
        </select>
      </label>
      <Num label="SP上限+" value={eff.cap_bonus} onChange={(v) => onChange({ cap_bonus: v })} />
      <Num label="目标普攻+SP" value={eff.sp_on_basic} onChange={(v) => onChange({ sp_on_basic: v })} />
      <Num label="叠层上限" value={eff.max_stacks} onChange={(v) => onChange({ max_stacks: v })} />
      <button className={styles.miniBtn} onClick={onRemove}>
        删除
      </button>
    </div>
  );
}

function Field({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <label className={styles.field}>
      <span>{label}</span>
      <input value={value} onChange={(e) => onChange(e.target.value)} />
    </label>
  );
}

function Num({
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
      <input type="number" step={0.01} value={value} onChange={(e) => onChange(Number(e.target.value))} />
    </label>
  );
}
