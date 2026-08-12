import { useEffect, useState } from "react";
import * as api from "../api/commands";
import type { BuildConfig, ConfigDataDTO, Team, TeamMember } from "../types";
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
          return (
            <div key={i} className={styles.member}>
              <div className={styles.memberHead}>
                <select
                  className={styles.charSelect}
                  value={m.char_id}
                  onChange={(e) => patchMember(i, { char_id: e.target.value })}
                >
                  {config?.characters.map((c) => (
                    <option key={c.id} value={c.id} disabled={team.members.some((x, j) => j !== i && x.char_id === c.id)}>
                      {c.name}
                    </option>
                  ))}
                </select>
                <button className={styles.removeBtn} onClick={() => removeMember(i)} title="移除">
                  ×
                </button>
              </div>
              <div className={styles.buildRow}>
                <label className={styles.field}>
                  <span>光锥</span>
                  <select
                    value={m.build.light_cone ?? ""}
                    onChange={(e) =>
                      patchBuild(i, { light_cone: e.target.value || null })
                    }
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
                <label className={styles.field}>
                  <span>暴击率</span>
                  <input
                    type="number"
                    step={0.01}
                    value={m.build.substats.crit_rate ?? 0}
                    onChange={(e) =>
                      patchBuild(i, {
                        substats: { ...m.build.substats, crit_rate: Number(e.target.value) },
                      })
                    }
                  />
                </label>
                <label className={styles.field}>
                  <span>暴伤</span>
                  <input
                    type="number"
                    step={0.01}
                    value={m.build.substats.crit_dmg ?? 0}
                    onChange={(e) =>
                      patchBuild(i, {
                        substats: { ...m.build.substats, crit_dmg: Number(e.target.value) },
                      })
                    }
                  />
                </label>
                <label className={styles.field}>
                  <span>攻击%</span>
                  <input
                    type="number"
                    step={0.01}
                    value={m.build.substats.atk_pct ?? 0}
                    onChange={(e) =>
                      patchBuild(i, {
                        substats: { ...m.build.substats, atk_pct: Number(e.target.value) },
                      })
                    }
                  />
                </label>
              </div>
            </div>
          );
        })}
        {team.members.length === 0 && (
          <div className={styles.empty}>尚未添加角色</div>
        )}
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
