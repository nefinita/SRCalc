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


def detect_scaling(desc: str) -> str:
    if "生命" in desc:
        return "hp"
    if "防御" in desc:
        return "def"
    return "atk"


def build_character(cid: str, c: dict, skills: dict, promotions: dict) -> list:
    stats = stat_at(promotions[cid]["values"], 80)
    abilities = []
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
        ability = {
            "name": s.get("name") or kind,
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
            "skill_point": {"basic": 1, "skill": -1, "ult": 0, "talent": 0}[kind],
            "target": TARGET_MAP.get(effect, "single"),
        }
        abilities.append(ability)

    character = {
        "id": cid,
        "name": c["name"],
        "path": PATH_MAP[c["path"]],
        "element": ELEMENT_MAP[c["element"]],
        "base_hp": stats["hp"],
        "base_atk": stats["atk"],
        "base_def": stats["def"],
        "base_spd": stats["spd"],
        "abilities": abilities,
    }
    return [character]


def build_light_cone(lcid: str, lc: dict, promotions: dict, ranks: dict) -> dict:
    stats = stat_at(promotions[lcid]["values"], 80)
    rank = ranks.get(lcid) or {}
    passive = rank.get("desc", "") if isinstance(rank, dict) else ""
    if passive and isinstance(rank.get("params"), list) and rank["params"]:
        passive += " | 叠影1: " + str(rank["params"][0])
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
    }


def build_relic_set(sid: str, s: dict) -> dict:
    desc = s.get("desc") or []
    return {
        "id": sid,
        "name": s["name"],
        "two_piece": desc[0] if len(desc) > 0 else None,
        "four_piece": desc[1] if len(desc) > 1 else None,
    }


def emit_toml(obj: dict, path: str) -> None:
    lines = []
    scalar = []
    for k, v in obj.items():
        if k == "abilities":
            continue
        if isinstance(v, bool):
            scalar.append(f"{k} = {'true' if v else 'false'}")
        elif isinstance(v, (int, float)):
            scalar.append(f"{k} = {v}")
        elif v is None:
            pass
        elif isinstance(v, list):
            scalar.append(f"{k} = [{', '.join(str(x) for x in v)}]")
        else:
            scalar.append(f"{k} = {toml_str(str(v))}")
    lines.extend(scalar)

    if isinstance(obj.get("abilities"), list):
        for ab in obj["abilities"]:
            lines.append("\n[[abilities]]")
            for k, v in ab.items():
                if isinstance(v, bool):
                    lines.append(f"{k} = {'true' if v else 'false'}")
                elif isinstance(v, (int, float)):
                    lines.append(f"{k} = {v}")
                elif isinstance(v, list):
                    lines.append(f"{k} = [{', '.join(str(x) for x in v)}]")
                else:
                    lines.append(f"{k} = {toml_str(str(v))}")

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

    n_char = n_lc = n_set = 0
    for cid, c in chars.items():
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
