import { useEffect, useMemo, useState } from "react";
import * as api from "../api/commands";
import type { AbilityData, CharacterDTO, ConfigDataDTO, EnemyDTO } from "../types";
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
        setEditingChar(cfg.characters[0]);
      }
      if (cfg.enemies.length) {
        setEnemyId(cfg.enemies[0].id);
        setEditingEnemy(cfg.enemies[0]);
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

  function patchEnemy(patch: Partial<EnemyDTO>) {
    setEditingEnemy((e) => (e ? { ...e, ...patch } : e));
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
                    <label className={styles.field}>
                      <span>技能等级</span>
                      <input
                        type="number"
                        min={1}
                        value={a.skill_level}
                        onChange={(e) => patchAbility(i, { skill_level: Math.max(1, Number(e.target.value)) })}
                      />
                    </label>
                    <label className={styles.field}>
                      <span>每级倍率(逗号分隔)</span>
                      <input
                        value={a.multipliers.join(",")}
                        onChange={(e) =>
                          patchAbility(i, {
                            multipliers: e.target.value
                              .split(",")
                              .map((x) => Number(x.trim()))
                              .filter((x) => Number.isFinite(x)),
                          })
                        }
                      />
                    </label>
                    <Num label="削韧" value={a.toughness_reduction} onChange={(v) => patchAbility(i, { toughness_reduction: v })} />
                    <Num label="回能" value={a.energy_gain} onChange={(v) => patchAbility(i, { energy_gain: v })} />
                    <Num label="最大能量" value={a.max_energy} onChange={(v) => patchAbility(i, { max_energy: v })} />
                    <Num label="战技点" value={a.skill_point} onChange={(v) => patchAbility(i, { skill_point: v })} />
                  </div>
                </div>
              ))}

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
