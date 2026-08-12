import { useEffect, useState } from "react";
import * as api from "../api/commands";
import type { BuildConfig, ConfigDataDTO, Team, TeamMember } from "../types";
import { mainStatOptions, SLOT_KEYS, SUBSTAT_KEYS, type MainSlot } from "../utils/constants";
import styles from "./TeamPanel.module.css";

interface Props {
  team: Team;
  setTeam: (t: Team) => void;
  addToast: (msg: string, type?: "success" | "error" | "info") => void;
}

const MAX_MEMBERS = 4;

export default function TeamPanel({ team, setTeam, addToast }: Props) {
  const [config, setConfig] = useState<ConfigDataDTO | null>(null);
  const [saveName, setSaveName] = useState("");
  const [teams, setTeams] = useState<string[]>([]);

  useEffect(() => {
    api.loadConfig().then(setConfig).catch(() => {});
    api.listTeams().then(setTeams).catch(() => {});
  }, []);

  function patchMember(index: number, patch: Partial<TeamMember>) {
    const members = team.members.map((m, i) => (i === index ? { ...m, ...patch } : m));
    setTeam({ members });
  }

  function patchBuild(index: number, patch: Partial<BuildConfig>) {
    patchMember(index, { build: { ...team.members[index].build, ...patch } });
  }

  function setMainStat(index: number, slot: MainSlot, stat: string, value: number) {
    const main_stats = team.members[index].build.main_stats.filter((m) => m.slot !== slot);
    main_stats.push({ slot, stat, value });
    patchBuild(index, { main_stats });
  }

  function setRelic(index: number, relicIndex: number, setId: string) {
    const relic_sets = [...team.members[index].build.relic_sets];
    const count = relic_sets[relicIndex]?.count ?? (relicIndex === 2 ? 2 : 2);
    relic_sets[relicIndex] = { set_id: setId, count: relicIndex === 2 ? 2 : count };
    patchBuild(index, { relic_sets: relic_sets.filter((r) => r.set_id) });
  }

  function setRelicCount(index: number, relicIndex: number, count: number) {
    const relic_sets = [...team.members[index].build.relic_sets];
    if (relic_sets[relicIndex]) relic_sets[relicIndex].count = count;
    if (relicIndex === 0 && count === 4) {
      // 整套：清空散件2
      relic_sets[1] = { set_id: "", count: 0 };
    }
    patchBuild(index, { relic_sets: relic_sets.filter((r) => r.set_id) });
  }

  function addMember() {
    if (team.members.length >= MAX_MEMBERS) {
      addToast("队伍最多 4 人", "error");
      return;
    }
    const unused = config?.characters.find(
      (c) => !team.members.some((m) => m.char_id === c.id)
    );
    if (!unused) {
      addToast("没有更多角色", "error");
      return;
    }
    const member: TeamMember = {
      char_id: unused.id,
      build: {
        level: 80,
        light_cone: null,
        relic_sets: [],
        main_stats: [],
        substats: { crit_rate: 0.3, crit_dmg: 0.8, atk_pct: 0.3 },
        traces: {},
      },
    };
    setTeam({ members: [...team.members, member] });
  }

  function removeMember(index: number) {
    setTeam({ members: team.members.filter((_, i) => i !== index) });
  }

  async function handleSave() {
    if (!saveName.trim()) {
      addToast("请输入队伍名", "error");
      return;
    }
    if (team.members.length === 0) {
      addToast("队伍为空", "error");
      return;
    }
    try {
      await api.saveTeam(saveName.trim(), team);
      setTeams(await api.listTeams());
      addToast("队伍已保存", "success");
    } catch (e) {
      addToast(String(e), "error");
    }
  }

  async function handleLoad(name: string) {
    try {
      const t = await api.loadTeam(name);
      if (t) {
        setTeam(t);
        addToast(`已加载 ${name}`, "success");
      }
    } catch (e) {
      addToast(String(e), "error");
    }
  }

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <span className={styles.title}>队伍</span>
        <span className={styles.count}>
          {team.members.length}/{MAX_MEMBERS}
        </span>
        <button className={styles.addBtn} onClick={addMember}>
          + 添加角色
        </button>
      </div>

      <div className={styles.members}>
        {team.members.map((m, i) => {
          const char = config?.characters.find((c) => c.id === m.char_id);
          const opts = char ? mainStatOptions(char.element) : null;
          const currentMain = (slot: MainSlot) =>
            m.build.main_stats.find((x) => x.slot === slot);
          return (
            <div key={i} className={styles.member}>
              <div className={styles.memberHead}>
                <select
                  className={styles.charSelect}
                  value={m.char_id}
                  onChange={(e) => patchMember(i, { char_id: e.target.value })}
                >
                  {config?.characters.map((c) => (
                    <option
                      key={c.id}
                      value={c.id}
                      disabled={team.members.some((x, j) => j !== i && x.char_id === c.id)}
                    >
                      {c.name}
                    </option>
                  ))}
                </select>
                <button className={styles.removeBtn} onClick={() => removeMember(i)} title="移除">
                  ×
                </button>
              </div>

              <div className={styles.row}>
                <label className={styles.field}>
                  <span>光锥</span>
                  <select
                    value={m.build.light_cone ?? ""}
                    onChange={(e) => patchBuild(i, { light_cone: e.target.value || null })}
                  >
                    <option value="">无</option>
                    {config?.light_cones
                      .filter((c) => char && c.path === char.path)
                      .map((c) => (
                        <option key={c.id} value={c.id}>
                          {c.name}
                        </option>
                      ))}
                  </select>
                </label>
                <label className={styles.field}>
                  <span>等级</span>
                  <input
                    type="number"
                    min={1}
                    max={80}
                    value={m.build.level}
                    onChange={(e) => patchBuild(i, { level: Number(e.target.value) })}
                  />
                </label>
              </div>

              <div className={styles.subTitle}>主词条</div>
              {opts &&
                SLOT_KEYS.map((slot) => {
                  const cur = currentMain(slot);
                  return (
                    <label key={slot} className={styles.field}>
                      <span>{slotLabel(slot)}</span>
                      <select
                        value={cur?.stat ?? ""}
                        onChange={(e) => {
                          const opt = opts[slot].find((o) => o.stat === e.target.value);
                          if (opt) setMainStat(i, slot, opt.stat, opt.value);
                        }}
                      >
                        <option value="">未选择</option>
                        {opts[slot].map((o) => (
                          <option key={o.stat} value={o.stat}>
                            {o.label}
                          </option>
                        ))}
                      </select>
                    </label>
                  );
                })}

              <div className={styles.subTitle}>副词条</div>
              <div className={styles.row}>
                {SUBSTAT_KEYS.map((s) => (
                  <label key={s.key} className={styles.field}>
                    <span>{s.label}</span>
                    <input
                      type="number"
                      step={0.01}
                      value={m.build.substats[s.key] ?? 0}
                      onChange={(e) =>
                        patchBuild(i, {
                          substats: { ...m.build.substats, [s.key]: Number(e.target.value) },
                        })
                      }
                    />
                  </label>
                ))}
              </div>

              <div className={styles.subTitle}>遗器套装（整套4件 / 2+2散件 · 饰品2件）</div>
              <div className={styles.row}>
                {/* 遗器套装1（含件数；4件=整套） */}
                <div className={styles.relicGroup}>
                  <label className={styles.field}>
                    <span>遗器套装1</span>
                    <select
                      value={m.build.relic_sets[0]?.set_id ?? ""}
                      onChange={(e) => setRelic(i, 0, e.target.value)}
                    >
                      <option value="">无</option>
                      {config?.relic_sets.map((r) => (
                        <option key={r.id} value={r.id}>
                          {r.name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <select
                    className={styles.relicCount}
                    value={m.build.relic_sets[0]?.count ?? 2}
                    onChange={(e) => setRelicCount(i, 0, Number(e.target.value))}
                  >
                    <option value={2}>2件</option>
                    <option value={4}>4件(整套)</option>
                  </select>
                </div>
                {/* 遗器套装2：仅当套装1为 2 件（2+2 散件） */}
                {m.build.relic_sets[0]?.count !== 4 && (
                  <div className={styles.relicGroup}>
                    <label className={styles.field}>
                      <span>遗器套装2</span>
                      <select
                        value={m.build.relic_sets[1]?.set_id ?? ""}
                        onChange={(e) => setRelic(i, 1, e.target.value)}
                      >
                        <option value="">无</option>
                        {config?.relic_sets.map((r) => (
                          <option key={r.id} value={r.id}>
                            {r.name}
                          </option>
                        ))}
                      </select>
                    </label>
                    <span className={styles.relicCount}>2件</span>
                  </div>
                )}
                {/* 饰品套装（固定 2 件） */}
                <div className={styles.relicGroup}>
                  <label className={styles.field}>
                    <span>饰品套装</span>
                    <select
                      value={m.build.relic_sets[2]?.set_id ?? ""}
                      onChange={(e) => setRelic(i, 2, e.target.value)}
                    >
                      <option value="">无</option>
                      {config?.relic_sets.map((r) => (
                        <option key={r.id} value={r.id}>
                          {r.name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <span className={styles.relicCount}>2件</span>
                </div>
              </div>
            </div>
          );
        })}
        {team.members.length === 0 && <div className={styles.empty}>尚未添加角色</div>}
      </div>

      <div className={styles.footer}>
        <input
          className={styles.nameInput}
          placeholder="队伍名"
          value={saveName}
          onChange={(e) => setSaveName(e.target.value)}
        />
        <button className={styles.footerBtn} onClick={handleSave}>
          保存
        </button>
        {teams.map((t) => (
          <button key={t} className={styles.footerBtn} onClick={() => handleLoad(t)}>
            {t}
          </button>
        ))}
      </div>
    </div>
  );
}

function slotLabel(slot: MainSlot): string {
  switch (slot) {
    case "body":
      return "身体";
    case "feet":
      return "脚部";
    case "sphere":
      return "位面球";
    case "rope":
      return "连接绳";
  }
}
