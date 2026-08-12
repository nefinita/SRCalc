#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""从 StarRailRes（github.com/Mar-7th/StarRailRes，AGPL-3.0）导入全量数据。

下载 cn 索引，生成 data/{characters,light_cones,relic_sets}/*.toml，
映射到 SRCalc 的 sr_api DTO 结构。

用法:
    python3 scripts/import_starrailres.py [--out data] [--lang cn]
"""

import argparse
import json
import os
import re
import sys
import time
import urllib.request

BASE = "https://raw.githubusercontent.com/Mar-7th/StarRailRes/master/index_new/{lang}/"

FILES = [
    "characters",
    "character_skills",
    "character_promotions",
    "light_cones",
    "light_cone_promotions",
    "light_cone_ranks",
    "relic_sets",
]

PATH_MAP = {
    "Warrior": "destruction",
    "Rogue": "the_hunt",
    "Mage": "erudition",
    "Shaman": "harmony",
    "Warlock": "nihility",
    "Knight": "preservation",
    "Priest": "abundance",
    "Memory": "remembrance",
    "Elation": "elation",
}

ELEMENT_MAP = {
    "Physical": "physical",
    "Fire": "fire",
    "Ice": "ice",
    "Thunder": "lightning",
    "Wind": "wind",
    "Quantum": "quantum",
    "Imaginary": "imaginary",
}

TYPE_KIND = {"Normal": "basic", "BPSkill": "skill", "Ultra": "ult", "Talent": "talent"}

ATTACK_EFFECTS = {"SingleAttack", "AoEAttack", "Blast", "Bounce"}

TARGET_MAP = {
    "SingleAttack": "single",
    "AoEAttack": "all",
    "Blast": "adjacent",
    "Bounce": "random",
}


def fetch(lang: str, name: str) -> dict:
    url = BASE.format(lang=lang) + name + ".json"
    for attempt in range(5):
        try:
            with urllib.request.urlopen(url, timeout=60) as r:
                return json.loads(r.read().decode("utf-8"))
        except Exception as e:  # noqa: BLE001 网络抖动重试
            if attempt == 4:
                raise
            print(f"  重试 {name}.json ({e}); 第 {attempt + 2} 次")
            time.sleep(2)
    raise RuntimeError(f"unreachable: {name}")


def toml_str(s: str) -> str:
    s = (
        s.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", "\\n")
        .replace("\t", "\\t")
        .replace("\r", "")
    )
    return '"' + s + '"'


def stat_at(values, level: int) -> dict:
    """按 promotion 表计算 level 级基础属性（最后一个条目覆盖 70->80）。"""
    starts = [1, 20, 30, 40, 50, 60, 70]
    last = values[-1]
    start = starts[-1]
    return {k: round(v["base"] + v["step"] * (level - start), 3) for k, v in last.items()}


def resolve_param(desc: str, params: list, lvl: int, key: int):
    """解析 desc 中 #N[i] 或 #N[f1] 占位符对应的参数值。"""
    if not params:
        return None
    if not re.search(r"#" + str(key) + r"\[(?:i|f\d+)\]", desc):
        return None
    row = params[lvl] if lvl < len(params) else params[-1]
    if key - 1 < len(row):
        return row[key - 1]
    return None


SP_PATTERNS = [
    ("造成的伤害提高", "dmg_pct"),
    ("攻击力提高", "atk_pct"),
    ("暴击伤害提高", "crit_dmg"),
    ("暴击率提高", "crit_rate"),
    ("速度提高", "speed_pct"),
    ("防御力提高", "def_pct"),
]


def find_stat_key(desc: str):
    for phrase, stat in SP_PATTERNS:
        if phrase in desc:
            return stat
    return None


def parse_sp(desc: str, kind: str, params: list) -> tuple:
    """按描述解析战技点：返回 (skill_point, bonus_sp)。"""
    base = {"basic": 1, "skill": -1, "ult": 0, "talent": 0}[kind]
    m = re.search(r"消耗#(\d+)\[i\]点战技点", desc)
    if m:
        val = resolve_param(desc, params, 5, int(m.group(1)))
        return -int(val or 0), 0
    m = re.search(r"恢复#(\d+)\[i\]个战技点", desc)
    if m:
        val = resolve_param(desc, params, 5, int(m.group(1)))
        if kind in ("ult", "talent"):
            return 0, int(val or 0)
        return int(val or 0), 0
    if "不消耗战技点" in desc:
        return 0, 0
    return base, 0


def parse_buff(desc: str, params: list, kind: str):
    """从描述解析施放时的 buff（增益）。"""
    stat = find_stat_key(desc)
    if stat is None:
        return None
    idx = desc.find(SP_PATTERNS[[x[1] for x in SP_PATTERNS].index(stat)][0])
    m = re.search(r"#(\d+)\[", desc[idx:])
    if not m:
        return None
    key = int(m.group(1))
    value = resolve_param(desc, params, 5, key)
    if value is None:
        return None
    tm = re.search(r"持续#(\d+)\[i\]回合", desc)
    turns = int(resolve_param(desc, params, 5, int(tm.group(1))) or 1) if tm else 1
    target = {"basic": "self", "skill": "ally", "ult": "team", "talent": "self"}[kind]
    return {
        "trigger": "on_use",
        "stat": stat,
        "value": value,
        "turns": turns,
        "target": target,
        "cap_bonus": 0,
        "sp_on_basic": 0,
        "max_stacks": 0,
    }


def parse_advance(desc: str, params: list, kind: str):
    """返回 (immediate_action, action_advance_pct)。"""
    imm = "立即行动" in desc and kind == "skill"
    adv = 0.0
    m = re.search(r"行动提前#(\d+)\[i\]%", desc)
    if m:
        v = resolve_param(desc, params, 5, int(m.group(1))) or 0
        adv = round(v / 100.0, 4)
    return imm, adv


def parse_team_effects(desc: str, params: list, kind: str):
    """天赋：在场被动（战技点上限 / 消耗SP触发）。"""
    if kind != "talent":
        return []
    effects = []
    m = re.search(r"战技点上限额外增加#(\d+)\[i\]点", desc)
    if m:
        cap = resolve_param(desc, params, 5, int(m.group(1)))
        effects.append({
            "trigger": "on_use", "stat": "atk_pct", "value": 0.0, "turns": 0,
            "target": "team", "cap_bonus": int(cap or 0),
            "sp_on_basic": 0, "max_stacks": 0,
        })
    if "每消耗1点战技点" in desc:
        dm = re.search(r"伤害提高#(\d+)\[", desc)
        sm = re.search(r"最多可叠加#(\d+)\[i\]层", desc)
        tm = re.search(r"持续#(\d+)\[i\]回合", desc)
        value = resolve_param(desc, params, 5, int(dm.group(1))) if dm else 0.0
        stacks = int(resolve_param(desc, params, 5, int(sm.group(1))) or 1) if sm else 1
        turns = int(resolve_param(desc, params, 5, int(tm.group(1))) or 1) if tm else 1
        effects.append({
            "trigger": "on_sp_consume", "stat": "dmg_pct", "value": value,
            "turns": turns, "target": "team", "cap_bonus": 0,
            "sp_on_basic": 0, "max_stacks": stacks,
        })
    return effects


def parse_cone_effects(desc: str, params: list):
    """光锥效果：战技点上限 / 常驻增益。"""
    effects = []
    m = re.search(r"战技点上限提高#(\d+)\[i\]点", desc)
    if m:
        cap = resolve_param(desc, params, 0, int(m.group(1)))
        effects.append({
            "trigger": "on_use", "stat": "atk_pct", "value": 0.0, "turns": 0,
            "target": "team", "cap_bonus": int(cap or 0),
            "sp_on_basic": 0, "max_stacks": 0,
        })
    for phrase, stat in SP_PATTERNS:
        idx = desc.find(phrase)
        if idx < 0:
            continue
        m = re.search(r"#(\d+)\[", desc[idx:])
        if not m:
            continue
        value = resolve_param(desc, params, 0, int(m.group(1)))
        if value:
            effects.append({
                "trigger": "on_use", "stat": stat, "value": value, "turns": 0,
                "target": "team", "cap_bonus": 0, "sp_on_basic": 0, "max_stacks": 0,
            })
    return effects


PATH_CN = {
    "destruction": "毁灭", "the_hunt": "巡猎", "erudition": "智识", "harmony": "同谐",
    "nihility": "虚无", "preservation": "存护", "abundance": "丰饶",
    "remembrance": "记忆", "elation": "欢愉",
}
ELEMENT_CN = {
    "physical": "物理", "fire": "火", "ice": "冰", "lightning": "雷",
    "wind": "风", "quantum": "量子", "imaginary": "虚数",
}


def clean_name(name: str, path: str = "", element: str = "") -> str:
    """主角名字为 {NICKNAME} 占位符 → 按 命途·属性 区分。"""
    if "{NICKNAME}" in name:
        return f"开拓者·{PATH_CN.get(path, path)}·{ELEMENT_CN.get(element, element)}"
    return name


def detect_scaling(desc: str) -> str:
    if "生命" in desc:
        return "hp"
    if "防御" in desc:
        return "def"
    return "atk"


def build_character(cid: str, c: dict, skills: dict, promotions: dict) -> list:
    stats = stat_at(promotions[cid]["values"], 80)
    abilities = []
    team_effects = []
    for sid in c.get("skills", []):
        s = skills.get(sid)
        if not s:
            continue
        kind = TYPE_KIND.get(s["type"])
        if kind is None:
            continue  # 跳过秘技/位面等
        params = s.get("params") or []
        multipliers = [row[0] for row in params] if params else []
        mult = multipliers[0] if multipliers else 0.0
        effect = s.get("effect", "")
        desc = s.get("desc", "")
        sp, bonus_sp = parse_sp(desc, kind, params)
        imm, adv = parse_advance(desc, params, kind)
        ability = {
            "name": clean_name(s.get("name") or kind),
            "kind": kind,
            "multiplier": mult,
            "multipliers": multipliers,
            "skill_level": 6,
            "scaling": detect_scaling(desc),
            "flat_damage": 0.0,
            "dmg_type": "dot" if ("持续伤害" in desc or "DoT" in desc) else "normal",
            "can_crit": effect in ATTACK_EFFECTS,
            "toughness_reduction": {"basic": 10.0, "skill": 20.0, "ult": 30.0, "talent": 0.0}[kind],
            "hits": 1,
            "hit_split": [1.0],
            "energy_gain": {"basic": 20.0, "skill": 30.0, "ult": 5.0, "talent": 0.0}[kind],
            "max_energy": float(c.get("max_sp") or 100),
            "skill_point": sp,
            "bonus_sp": bonus_sp,
            "target": TARGET_MAP.get(effect, "single"),
            "buff": parse_buff(desc, params, kind),
            "immediate_action": imm,
            "action_advance_pct": adv,
            "self_advance_pct": 0.0,
        }
        abilities.append(ability)
        team_effects.extend(parse_team_effects(desc, params, kind))

    # 去重 team_effects（强化/普通变体会重复天赋）
    seen = set()
    dedup = []
    for e in team_effects:
        key = (e["trigger"], e["stat"], e["target"], e["cap_bonus"], round(e["value"], 6), e["turns"], e["max_stacks"])
        if key not in seen:
            seen.add(key)
            dedup.append(e)

    character = {
        "id": cid,
        "name": clean_name(c["name"], PATH_MAP[c["path"]], ELEMENT_MAP[c["element"]]),
        "path": PATH_MAP[c["path"]],
        "element": ELEMENT_MAP[c["element"]],
        "base_hp": stats["hp"],
        "base_atk": stats["atk"],
        "base_def": stats["def"],
        "base_spd": stats["spd"],
        "abilities": abilities,
        "team_effects": dedup,
    }
    return [character]


def build_light_cone(lcid: str, lc: dict, promotions: dict, ranks: dict) -> dict:
    stats = stat_at(promotions[lcid]["values"], 80)
    rank = ranks.get(lcid) or {}
    passive = rank.get("desc", "") if isinstance(rank, dict) else ""
    params = rank.get("params") if isinstance(rank, dict) else None
    if passive and params:
        passive += " | 叠影1: " + str(params[0])
    return {
        "id": lcid,
        "name": lc["name"],
        "path": PATH_MAP[lc["path"]],
        "rarity": lc["rarity"],
        "base_hp": stats["hp"],
        "base_atk": stats["atk"],
        "base_def": stats["def"],
        "superimposition": 1,
        "passive": passive or None,
        "effects": parse_cone_effects(passive or "", params or []),
    }


PROP_STAT = {
    "AttackAddedRatio": "atk_pct",
    "HPAddedRatio": "hp_pct",
    "DefenceAddedRatio": "def_pct",
    "SpeedAddedRatio": "speed_pct",
    "CriticalChanceBase": "crit_rate",
    "CriticalDamageBase": "crit_dmg",
    "BreakDamageAddedRatioBase": "break_effect",
    "SPRatioBase": "energy_regen",
    # 元素伤害近似为通用增伤（配装通常匹配本角色元素）
    "FireAddedRatio": "dmg_pct",
    "IceAddedRatio": "dmg_pct",
    "ImaginaryAddedRatio": "dmg_pct",
    "PhysicalAddedRatio": "dmg_pct",
    "QuantumAddedRatio": "dmg_pct",
    "ThunderAddedRatio": "dmg_pct",
    "WindAddedRatio": "dmg_pct",
}


def effects_from_props(props: list) -> list:
    effects = []
    for p in props:
        stat = PROP_STAT.get(p.get("type", ""))
        if stat is None:
            continue
        effects.append({
            "trigger": "on_use", "stat": stat, "value": p["value"], "turns": 0,
            "target": "self", "cap_bonus": 0, "sp_on_basic": 0, "max_stacks": 0,
        })
    return effects


def build_relic_set(sid: str, s: dict) -> dict:
    desc = s.get("desc") or []
    props = s.get("properties") or []
    return {
        "id": sid,
        "name": s["name"],
        "two_piece": desc[0] if len(desc) > 0 else None,
        "four_piece": desc[1] if len(desc) > 1 else None,
        "two_piece_effects": effects_from_props(props[0]) if len(props) > 0 else [],
        "four_piece_effects": effects_from_props(props[1]) if len(props) > 1 else [],
    }


def inline_table(d: dict) -> str:
    parts = []
    for k, v in d.items():
        if isinstance(v, bool):
            parts.append(f"{k} = {'true' if v else 'false'}")
        elif isinstance(v, (int, float)):
            parts.append(f"{k} = {v}")
        elif isinstance(v, list):
            parts.append(f"{k} = [{', '.join(str(x) for x in v)}]")
        else:
            parts.append(f"{k} = {toml_str(str(v))}")
    return "{ " + ", ".join(parts) + " }"


def emit_field(lines: list, k: str, v) -> None:
    if v is None:
        return
    if isinstance(v, bool):
        lines.append(f"{k} = {'true' if v else 'false'}")
    elif isinstance(v, (int, float)):
        lines.append(f"{k} = {v}")
    elif isinstance(v, dict):
        lines.append(f"{k} = {inline_table(v)}")
    elif isinstance(v, list):
        if not v:
            lines.append(f"{k} = []")
        elif all(isinstance(x, (int, float)) for x in v):
            lines.append(f"{k} = [{', '.join(str(x) for x in v)}]")
        elif all(isinstance(x, dict) for x in v):
            lines.append(f"{k} = [{', '.join(inline_table(x) for x in v)}]")
        else:
            lines.append(f"{k} = [{', '.join(str(x) for x in v)}]")
    else:
        lines.append(f"{k} = {toml_str(str(v))}")


def emit_toml(obj: dict, path: str) -> None:
    lines = []
    for k, v in obj.items():
        if k == "abilities":
            continue
        emit_field(lines, k, v)

    if isinstance(obj.get("abilities"), list):
        for ab in obj["abilities"]:
            lines.append("\n[[abilities]]")
            for k, v in ab.items():
                emit_field(lines, k, v)

    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="data")
    ap.add_argument("--lang", default="cn")
    args = ap.parse_args()

    print("下载索引（lang=%s）..." % args.lang)
    data = {name: fetch(args.lang, name) for name in FILES}

    chars, skills, proms = data["characters"], data["character_skills"], data["character_promotions"]
    lcs, lcp, lcr = data["light_cones"], data["light_cone_promotions"], data["light_cone_ranks"]
    sets = data["relic_sets"]

    import shutil
    for kind in ("characters", "light_cones", "relic_sets"):
        shutil.rmtree(os.path.join(args.out, kind), ignore_errors=True)
    n_char = n_lc = n_set = 0
    seen_tb = set()  # 主角成对重复（同命途同属性只留一个）
    for cid, c in chars.items():
        if "{NICKNAME}" in c["name"]:
            key = (c["path"], c["element"])
            if key in seen_tb:
                continue
            seen_tb.add(key)
        for character in build_character(cid, c, skills, proms):
            emit_toml(character, os.path.join(args.out, "characters", f"{cid}.toml"))
            n_char += 1
    for lcid, lc in lcs.items():
        cone = build_light_cone(lcid, lc, lcp, lcr)
        emit_toml(cone, os.path.join(args.out, "light_cones", f"{lcid}.toml"))
        n_lc += 1
    for sid, s in sets.items():
        emit_toml(build_relic_set(sid, s), os.path.join(args.out, "relic_sets", f"{sid}.toml"))
        n_set += 1

    print(f"完成: 角色 {n_char} / 光锥 {n_lc} / 遗器套装 {n_set} → {args.out}/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
