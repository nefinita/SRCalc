//! damage — 星铁伤害计算引擎
//!
//! 公式依据 HSR Fandom Wiki（Damage / Damage RES / Toughness / Speed）：
//!
//! ```text
//! DMG = Base × 增伤 × 防御 × 抗性 × 易伤 × 减伤 × 韧性乘区 × 暴击乘区
//! ```

use sr_api::{
    AbilityData, BuffStat, BuffTarget, Build, Character, CoefficientConfig, DmgType, Effect,
    Element, Enemy, LightCone, RelicSet, RelicSlot, Scaling, Trigger,
};
use std::collections::HashMap;

/// 可叠加的百分比属性修正（buffs / 光锥效果 / 在场被动 折叠到此）
#[derive(Debug, Clone, Default)]
pub struct StatMods {
    pub hp_pct: f64,
    pub atk_pct: f64,
    pub def_pct: f64,
    pub spd_pct: f64,
    pub crit_rate: f64,
    pub crit_dmg: f64,
    pub dmg_pct: f64,
    pub def_ignore: f64,
    pub res_pen: f64,
    pub vuln_pct: f64,
    pub break_effect: f64,
    pub energy_regen: f64,
    pub ult_dmg_pct: f64,
    pub skill_dmg_pct: f64,
    pub basic_dmg_pct: f64,
    pub followup_dmg_pct: f64,
}

impl StatMods {
    pub fn from_buff(b: &sr_api::BuffConfig) -> StatMods {
        StatMods {
            atk_pct: b.atk_pct,
            dmg_pct: b.dmg_pct,
            crit_rate: b.crit_rate,
            crit_dmg: b.crit_dmg,
            def_ignore: b.def_ignore,
            res_pen: b.res_pen,
            vuln_pct: b.vuln_pct,
            break_effect: b.break_effect,
            ..Default::default()
        }
    }

    pub fn from_effect(e: &Effect, stacks: u32) -> StatMods {
        let v = e.value * stacks.max(1) as f64;
        let mut m = StatMods::default();
        match e.stat {
            BuffStat::AtkPct => m.atk_pct = v,
            BuffStat::HpPct => m.hp_pct = v,
            BuffStat::DefPct => m.def_pct = v,
            BuffStat::SpeedPct => m.spd_pct = v,
            BuffStat::CritRate => m.crit_rate = v,
            BuffStat::CritDmg => m.crit_dmg = v,
            BuffStat::DmgPct => m.dmg_pct = v,
            BuffStat::DefIgnore => m.def_ignore = v,
            BuffStat::ResPen => m.res_pen = v,
            BuffStat::VulnPct => m.vuln_pct = v,
            BuffStat::BreakEffect => m.break_effect = v,
            BuffStat::EnergyRegen => m.energy_regen = v,
            BuffStat::UltDmgPct => m.ult_dmg_pct = v,
            BuffStat::SkillDmgPct => m.skill_dmg_pct = v,
            BuffStat::BasicDmgPct => m.basic_dmg_pct = v,
            BuffStat::FollowUpDmgPct => m.followup_dmg_pct = v,
        }
        m
    }

    pub fn add(&mut self, o: &StatMods) {
        self.hp_pct += o.hp_pct;
        self.atk_pct += o.atk_pct;
        self.def_pct += o.def_pct;
        self.spd_pct += o.spd_pct;
        self.crit_rate += o.crit_rate;
        self.crit_dmg += o.crit_dmg;
        self.dmg_pct += o.dmg_pct;
        self.def_ignore += o.def_ignore;
        self.res_pen += o.res_pen;
        self.vuln_pct += o.vuln_pct;
        self.break_effect += o.break_effect;
        self.energy_regen += o.energy_regen;
        self.ult_dmg_pct += o.ult_dmg_pct;
        self.skill_dmg_pct += o.skill_dmg_pct;
        self.basic_dmg_pct += o.basic_dmg_pct;
        self.followup_dmg_pct += o.followup_dmg_pct;
    }
}

#[derive(Debug, Clone)]
pub struct FinalStats {
    pub hp: f64,
    pub atk: f64,
    pub def: f64,
    pub spd: f64,
    pub crit_rate: f64,
    pub crit_dmg: f64,
    pub dmg_pct: f64,
    /// 能量回复效率（ERR，充能绳/副词条）→ 能量获取 ×(1+ERR)
    pub energy_regen: f64,
}

/// 进场生效的永久修正：自身光锥效果 + 自身在场被动 + 队友"全队"在场被动
pub fn presence_mods(
    character: &Character,
    cone: Option<&LightCone>,
    allies: &[&Character],
) -> StatMods {
    let mut m = StatMods::default();
    if let Some(c) = cone {
        for e in &c.effects {
            m.add(&StatMods::from_effect(e, 1));
        }
    }
    for e in &character.team_effects {
        if e.trigger == Trigger::OnUse {
            m.add(&StatMods::from_effect(e, 1));
        }
    }
    for ally in allies {
        if ally.id == character.id {
            continue;
        }
        for e in &ally.team_effects {
            if e.trigger == Trigger::OnUse && e.target == BuffTarget::Team {
                m.add(&StatMods::from_effect(e, 1));
            }
        }
    }
    m
}

/// 遗器套装数值效果 → 永久修正（2件/4件；饰品位固定 2 件）
pub fn relic_set_mods(build: &Build, sets: &[&RelicSet]) -> StatMods {
    let mut m = StatMods::default();
    for piece in &build.relic_sets {
        let Some(set) = sets.iter().find(|s| s.id == piece.set_id) else {
            continue;
        };
        if piece.count >= 4 {
            for e in &set.four_piece_effects {
                m.add(&StatMods::from_effect(e, 1));
            }
        }
        if piece.count >= 2 {
            for e in &set.two_piece_effects {
                m.add(&StatMods::from_effect(e, 1));
            }
        }
    }
    m
}

/// 仅常驻（BattleStart）套装效果 → 排轴永久修正
pub fn relic_set_permanent(build: &Build, sets: &[&RelicSet]) -> StatMods {
    let mut m = StatMods::default();
    for piece in &build.relic_sets {
        let Some(set) = sets.iter().find(|s| s.id == piece.set_id) else {
            continue;
        };
        if piece.count >= 4 {
            for e in &set.four_piece_effects {
                if e.trigger == Trigger::BattleStart {
                    m.add(&StatMods::from_effect(e, 1));
                }
            }
        }
        if piece.count >= 2 {
            for e in &set.two_piece_effects {
                if e.trigger == Trigger::BattleStart {
                    m.add(&StatMods::from_effect(e, 1));
                }
            }
        }
    }
    m
}

/// 触发式套装被动（OnUlt/OnSkill/OnBasic/OnHit/TurnStart）→ 排轴触发列表
pub fn relic_set_conditional(build: &Build, sets: &[&RelicSet]) -> Vec<(Trigger, Effect)> {
    let mut out = Vec::new();
    for piece in &build.relic_sets {
        let Some(set) = sets.iter().find(|s| s.id == piece.set_id) else {
            continue;
        };
        let mut check = |es: &[Effect]| {
            for e in es {
                if e.trigger != Trigger::BattleStart {
                    out.push((e.trigger, e.clone()));
                }
            }
        };
        if piece.count >= 4 {
            check(&set.four_piece_effects);
        }
        if piece.count >= 2 {
            check(&set.two_piece_effects);
        }
    }
    out
}

/// 汇总角色最终面板：基础属性(角色+光锥) × (1+%+permanent%) + 固定值
pub fn compute_final_stats(
    character: &Character,
    cone: Option<&LightCone>,
    build: &Build,
    permanent: &StatMods,
) -> FinalStats {
    let cone_hp = cone.map(|c| c.base_hp).unwrap_or(0.0);
    let cone_atk = cone.map(|c| c.base_atk).unwrap_or(0.0);
    let cone_def = cone.map(|c| c.base_def).unwrap_or(0.0);

    let mut flat = HashMap::new();
    let mut pct = HashMap::new();
    flat.insert("hp".to_string(), cone_hp);
    flat.insert("atk".to_string(), cone_atk);
    flat.insert("def".to_string(), cone_def);

    for m in &build.main_stats {
        match m.slot {
            RelicSlot::Head => {
                flat.entry("hp".to_string()).and_modify(|v| *v += m.value);
            }
            RelicSlot::Hands => {
                flat.entry("atk".to_string()).and_modify(|v| *v += m.value);
            }
            _ => {
                if m.stat == "spd" {
                    flat.entry("spd".to_string()).and_modify(|v| *v += m.value);
                } else {
                    pct.insert(m.stat.clone(), m.value);
                }
            }
        }
    }
    for (k, v) in &build.substats {
        if k.ends_with("_pct") || k == "crit_rate" || k == "crit_dmg" || k == "dmg_pct" {
            pct.entry(k.clone()).and_modify(|x| *x += v).or_insert(*v);
        } else {
            flat.entry(k.clone()).and_modify(|x| *x += v).or_insert(*v);
        }
    }

    let atk_pct = pct.get("atk_pct").copied().unwrap_or(0.0) + permanent.atk_pct;
    let hp_pct = pct.get("hp_pct").copied().unwrap_or(0.0) + permanent.hp_pct;
    let def_pct = pct.get("def_pct").copied().unwrap_or(0.0) + permanent.def_pct;

    let atk = character.base_atk * (1.0 + atk_pct) + flat.get("atk").copied().unwrap_or(0.0);
    let hp = character.base_hp * (1.0 + hp_pct) + flat.get("hp").copied().unwrap_or(0.0);
    let def = character.base_def * (1.0 + def_pct) + flat.get("def").copied().unwrap_or(0.0);
    let spd_pct = pct.get("spd_pct").copied().unwrap_or(0.0) + permanent.spd_pct;
    let spd = character.base_spd * (1.0 + spd_pct) + flat.get("spd").copied().unwrap_or(0.0);

    let crit_rate = (0.05 + pct.get("crit_rate").copied().unwrap_or(0.0) + permanent.crit_rate)
        .clamp(0.0, 1.0);
    let crit_dmg = 0.5 + pct.get("crit_dmg").copied().unwrap_or(0.0) + permanent.crit_dmg;

    let mut dmg_pct = pct.get("dmg_pct").copied().unwrap_or(0.0) + permanent.dmg_pct;
    // 元素伤害加成（球位 + 副词条）
    let element_key = format!("{}_dmg", element_str(character.element));
    if let Some(v) = pct.get(&element_key) {
        dmg_pct += v;
    }

    let energy_regen =
        pct.get("energy_regen").copied().unwrap_or(0.0) + permanent.energy_regen;

    FinalStats {
        hp,
        atk,
        def,
        spd,
        crit_rate,
        crit_dmg,
        dmg_pct,
        energy_regen,
    }
}

pub fn element_str(e: Element) -> &'static str {
    match e {
        Element::Physical => "physical",
        Element::Fire => "fire",
        Element::Ice => "ice",
        Element::Lightning => "lightning",
        Element::Wind => "wind",
        Element::Quantum => "quantum",
        Element::Imaginary => "imaginary",
    }
}

/// 防御乘区：1 − DEF'/(DEF' + def_const + 10×攻方等级)，DEF' = DEF×max(0,1−无视防御)
pub fn def_multiplier(
    attacker_level: u32,
    enemy: &Enemy,
    def_ignore: f64,
    coeff: &CoefficientConfig,
) -> f64 {
    let effective = enemy.def * (1.0 - def_ignore).max(0.0);
    1.0 - effective / (effective + coeff.def_const + 10.0 * attacker_level as f64)
}

/// 抗性乘区：1 − (目标抗性 − 穿透)，范围 10%~200%
pub fn res_multiplier(enemy: &Enemy, element: Element, res_pen: f64) -> f64 {
    let res = enemy.res.get(&element).copied().unwrap_or_else(|| element.default_res());
    (1.0 - (res - res_pen)).clamp(0.1, 2.0)
}

/// 韧性乘区：未破韧 0.9，已破韧 1.0
pub fn broken_multiplier(broken: bool, coeff: &CoefficientConfig) -> f64 {
    if broken {
        coeff.break_multiplier
    } else {
        coeff.broken_multiplier
    }
}

#[derive(Debug, Clone)]
pub struct DamageBreakdown {
    pub base: f64,
    pub non_crit: f64,
    pub crit: f64,
    pub expected: f64,
    pub crit_rate: f64,
    pub crit_dmg: f64,
}

pub struct AbilityContext<'a> {
    pub stats: &'a FinalStats,
    pub ability: &'a AbilityData,
    pub element: Element,
    pub attacker_level: u32,
    pub enemy: &'a Enemy,
    /// 当前生效（时变）修正：主动 buff + 全局 buff
    pub mods: &'a StatMods,
    pub coeff: &'a CoefficientConfig,
    /// 韧性是否已破（动态，覆盖 enemy.broken）
    pub broken: bool,
}

/// 按技能等级取倍率：multipliers 非空时取 `skill_level` 档，否则回退 `multiplier`
pub fn ability_multiplier(ability: &AbilityData) -> f64 {
    if !ability.multipliers.is_empty() {
        let idx = ability.skill_level.saturating_sub(1) as usize;
        ability
            .multipliers
            .get(idx)
            .copied()
            .unwrap_or(ability.multiplier)
    } else {
        ability.multiplier
    }
}

pub fn compute_ability_damage_for(ctx: AbilityContext) -> DamageBreakdown {
    let AbilityContext { stats, ability, element, attacker_level, enemy, mods, coeff, broken: _ } = ctx;
    let scaling_stat = match ability.scaling {
        Scaling::Atk => stats.atk * (1.0 + mods.atk_pct),
        Scaling::Hp => stats.hp * (1.0 + mods.hp_pct),
        Scaling::Def => stats.def * (1.0 + mods.def_pct),
    };
    let mult = ability_multiplier(ability);
    let base = mult * scaling_stat + ability.flat_damage;

    let type_dmg = match ability.kind {
        sr_api::AbilityKind::Basic => mods.basic_dmg_pct,
        sr_api::AbilityKind::Skill => mods.skill_dmg_pct,
        sr_api::AbilityKind::Ult => mods.ult_dmg_pct,
        sr_api::AbilityKind::Talent | sr_api::AbilityKind::Memosprite => mods.followup_dmg_pct,
    };
    let boost = 1.0 + stats.dmg_pct + mods.dmg_pct + type_dmg;
    let def_m = def_multiplier(attacker_level, enemy, mods.def_ignore, coeff);
    let res_m = res_multiplier(enemy, element, mods.res_pen);
    let vuln = 1.0 + mods.vuln_pct;
    let broken = broken_multiplier(ctx.broken, coeff);

    let pre_crit = base * boost * def_m * res_m * vuln * broken;

    let can_crit = ability.can_crit && ability.dmg_type != DmgType::Dot;
    let (crit_rate, crit_mult) = if can_crit {
        let cr = (stats.crit_rate + mods.crit_rate).clamp(0.0, 1.0);
        (cr, 1.0 + stats.crit_dmg + mods.crit_dmg)
    } else {
        (0.0, 1.0)
    };

    let non_crit = pre_crit;
    let crit = pre_crit * crit_mult;
    let expected = non_crit * (1.0 - crit_rate) + crit * crit_rate;

    DamageBreakdown {
        base,
        non_crit,
        crit,
        expected,
        crit_rate,
        crit_dmg: stats.crit_dmg + mods.crit_dmg,
    }
}

/// 击破伤害：类型系数 × 等级乘数 × 最大韧性乘数 × (1+击破特攻) × DEF × RES × 易伤 × 韧性乘区
pub fn compute_break_damage(
    element: Element,
    attacker_level: u32,
    enemy: &Enemy,
    mods: &StatMods,
    coeff: &CoefficientConfig,
    broken: bool,
) -> f64 {
    let type_coeff = match element {
        Element::Physical | Element::Fire => 2.0,
        Element::Ice | Element::Lightning => 1.0,
        Element::Wind => 1.5,
        Element::Quantum | Element::Imaginary => 0.5,
    };
    let max_toughness_mult = 0.5 + enemy.max_toughness / 40.0;
    let base = type_coeff * sr_const::level_multiplier(attacker_level) * max_toughness_mult;

    let def_m = def_multiplier(attacker_level, enemy, mods.def_ignore, coeff);
    let res_m = res_multiplier(enemy, element, mods.res_pen);
    let vuln = 1.0 + mods.vuln_pct;
    let broken_m = broken_multiplier(broken, coeff);

    base * (1.0 + mods.break_effect) * def_m * res_m * vuln * broken_m
}

/// 主词条标准值（5★ Lv15），供配装优化器使用
pub fn main_stat_options(element: Element) -> MainStatOptions {
    let element_key = format!("{}_dmg", element_str(element));
    MainStatOptions {
        body: vec![
            ("暴伤".into(), "crit_dmg".into(), 0.648),
            ("暴率".into(), "crit_rate".into(), 0.323),
            ("攻击".into(), "atk_pct".into(), 0.432),
            ("生命".into(), "hp_pct".into(), 0.432),
            ("防御".into(), "def_pct".into(), 0.540),
        ],
        feet: vec![
            ("速度".into(), "spd".into(), 25.03),
            ("攻击".into(), "atk_pct".into(), 0.432),
            ("生命".into(), "hp_pct".into(), 0.432),
            ("防御".into(), "def_pct".into(), 0.540),
        ],
        sphere: vec![
            ("元素伤害".into(), element_key, 0.388),
            ("攻击".into(), "atk_pct".into(), 0.432),
            ("生命".into(), "hp_pct".into(), 0.432),
            ("防御".into(), "def_pct".into(), 0.540),
        ],
        rope: vec![
            ("攻击".into(), "atk_pct".into(), 0.432),
            ("生命".into(), "hp_pct".into(), 0.432),
            ("防御".into(), "def_pct".into(), 0.540),
            ("击破".into(), "break_effect".into(), 0.648),
            ("充能".into(), "energy_regen".into(), 0.194),
        ],
    }
}

pub struct MainStatOptions {
    pub body: Vec<(String, String, f64)>,
    pub feet: Vec<(String, String, f64)>,
    pub sphere: Vec<(String, String, f64)>,
    pub rope: Vec<(String, String, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sr_api::{AbilityData, AbilityKind, Build, Character, DmgType, Enemy, Element, Path, Scaling, Target};

    fn dummy_enemy(level: u32, def: f64, broken: bool) -> Enemy {
        let mut res = std::collections::HashMap::new();
        for e in [Element::Physical, Element::Fire, Element::Ice, Element::Lightning, Element::Wind, Element::Imaginary] {
            res.insert(e, 0.2);
        }
        res.insert(Element::Quantum, 0.0);
        Enemy {
            id: "dummy".into(),
            name: "木桩".into(),
            level,
            def,
            max_toughness: 120.0,
            broken,
            res,
            spd: 100.0,
            actions: vec![],
            weaknesses: vec![],
            hp: 0.0,
        }
    }

    fn seele() -> Character {
        Character {
            id: "1101".into(),
            name: "希儿".into(),
            path: Path::TheHunt,
            element: Element::Quantum,
            base_hp: 1041.0,
            base_atk: 563.0,
            base_def: 330.0,
            base_spd: 115.0,
            abilities: vec![AbilityData {
                name: "普攻".into(),
                kind: AbilityKind::Basic,
                multiplier: 1.1,
                multipliers: vec![1.1],
                skill_level: 1,
                scaling: Scaling::Atk,
                flat_damage: 0.0,
                dmg_type: DmgType::Normal,
                can_crit: true,
                toughness_reduction: 30.0,
                hits: 1,
                hit_split: vec![1.0],
                energy_gain: 20.0,
                max_energy: 120.0,
                skill_point: 1,
                bonus_sp: 0,
                target: Target::Single,
                buff: None,
                immediate_action: false,
                action_advance_pct: 0.0,
                self_advance_pct: 0.0,
                applies_debuff: false,
                heals: false,
                forced: false,
            }],
            team_effects: vec![],
            has_memosprite: false,
        memosprite_spd: 0.0,
        memosprite_multiplier: 0.0,
            
        }
    }

    #[test]
    fn def_multiplier_level80_vs_level80() {
        let enemy = dummy_enemy(80, 1000.0, true);
        let coeff = CoefficientConfig::default();
        assert!((def_multiplier(80, &enemy, 0.0, &coeff) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn def_multiplier_with_ignore() {
        let enemy = dummy_enemy(80, 1000.0, true);
        let coeff = CoefficientConfig::default();
        // DEF 削减 20% → 有效防御 800 → 1 − 800/1800 = 0.5555...
        let got = def_multiplier(80, &enemy, 0.2, &coeff);
        assert!((got - (1.0 - 800.0 / 1800.0)).abs() < 1e-9);
    }

    #[test]
    fn res_multiplier_weak_zero() {
        let enemy = dummy_enemy(80, 1000.0, true);
        assert!((res_multiplier(&enemy, Element::Quantum, 0.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn golden_seele_basic_no_buff() {
        let enemy = dummy_enemy(80, 1000.0, false);
        let character = seele();
        let build = Build::default();
        let mods = StatMods::default();
        let coeff = CoefficientConfig::default();
        let stats = compute_final_stats(&character, None, &build, &mods);
        let result = compute_ability_damage_for(AbilityContext {
            stats: &stats,
            ability: &character.abilities[0],
            element: character.element,
            attacker_level: 80,
            enemy: &enemy,
            mods: &mods,
            coeff: &coeff,
            broken: enemy.broken,
        });
        // base = 1.1 × 563 = 619.3；def=0.5；res=1；broken=0.9
        // pre_crit = 619.3 × 0.5 × 0.9 = 278.685
        // crit_rate=0.05, crit_dmg=0.5 → expected = 278.685×0.95 + 278.685×1.5×0.05
        let pre_crit = 619.3 * 0.5 * 0.9;
        let expected = pre_crit * 0.95 + pre_crit * 1.5 * 0.05;
        assert!((result.base - 619.3).abs() < 1e-6);
        assert!((result.expected - expected).abs() < 1e-6);
        assert!((result.crit_rate - 0.05).abs() < 1e-9);
    }

    #[test]
    fn break_damage_sanity() {
        let enemy = dummy_enemy(80, 1000.0, false);
        let mods = StatMods::default();
        let coeff = CoefficientConfig::default();
        let dmg = compute_break_damage(Element::Quantum, 80, &enemy, &mods, &coeff, enemy.broken);
        // type_coeff=0.5, LM(80)=3767.5533, MTM=0.5+120/40=3.5
        // base=0.5×3767.5533×3.5=6593.218275；×def(0.5)×res(1)×broken(0.9)
        let expected = 0.5 * sr_const::level_multiplier(80) * 3.5 * 0.5 * 0.9;
        assert!((dmg - expected).abs() < 1e-6);
    }

    #[test]
    fn level_multiplier_table() {
        assert_eq!(sr_const::level_multiplier(1), 54.0);
        assert!((sr_const::level_multiplier(80) - 3767.5533).abs() < 1e-4);
        assert!((sr_const::level_multiplier(90) - 6020.8836).abs() < 1e-4);
        assert_eq!(sr_const::level_multiplier(999), 6020.8836);
    }
}
