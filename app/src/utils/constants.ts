import type { Element } from "../types";

export interface MainStatOption {
  label: string;
  stat: string;
  value: number;
}

export const SLOT_KEYS = ["body", "feet", "sphere", "rope"] as const;
export type MainSlot = (typeof SLOT_KEYS)[number];

/** 标准 5★ Lv15 主词条数值（与引擎 main_stat_options 同源） */
export function mainStatOptions(element: Element): Record<MainSlot, MainStatOption[]> {
  return {
    body: [
      { label: "暴伤", stat: "crit_dmg", value: 0.648 },
      { label: "暴率", stat: "crit_rate", value: 0.323 },
      { label: "攻击", stat: "atk_pct", value: 0.432 },
      { label: "生命", stat: "hp_pct", value: 0.432 },
      { label: "防御", stat: "def_pct", value: 0.54 },
    ],
    feet: [
      { label: "速度", stat: "spd", value: 25.03 },
      { label: "攻击", stat: "atk_pct", value: 0.432 },
      { label: "生命", stat: "hp_pct", value: 0.432 },
      { label: "防御", stat: "def_pct", value: 0.54 },
    ],
    sphere: [
      { label: "元素伤害", stat: `${element}_dmg`, value: 0.388 },
      { label: "攻击", stat: "atk_pct", value: 0.432 },
      { label: "生命", stat: "hp_pct", value: 0.432 },
      { label: "防御", stat: "def_pct", value: 0.54 },
    ],
    rope: [
      { label: "攻击", stat: "atk_pct", value: 0.432 },
      { label: "生命", stat: "hp_pct", value: 0.432 },
      { label: "防御", stat: "def_pct", value: 0.54 },
      { label: "击破", stat: "break_effect", value: 0.648 },
      { label: "充能", stat: "energy_regen", value: 0.194 },
    ],
  };
}

export const SUBSTAT_KEYS: { label: string; key: string }[] = [
  { label: "暴击率", key: "crit_rate" },
  { label: "暴伤", key: "crit_dmg" },
  { label: "攻击%", key: "atk_pct" },
  { label: "生命%", key: "hp_pct" },
  { label: "防御%", key: "def_pct" },
  { label: "速度", key: "spd" },
  { label: "击破特攻", key: "break_effect" },
  { label: "充能效率", key: "energy_regen" },
];
