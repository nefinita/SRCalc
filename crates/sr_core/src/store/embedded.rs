//! embedded — 内置占位数据
//!
//! MVP 阶段内置 1 个演示角色（希儿）+ 光锥 + 遗器套装 + 敌方木桩，
//! 磁盘上的 `data/` TOML 文件存在时优先覆盖。

use sr_api::{
    AbilityData, AbilityKind, Character, ConfigData, DmgType, Element, Enemy, LightCone,
    Path, RelicSet, Scaling, Target,
};

pub fn default_config() -> ConfigData {
    ConfigData {
        characters: vec![default_seele()],
        light_cones: vec![default_cone()],
        relic_sets: vec![default_set()],
        enemies: vec![default_dummy()],
    }
}

fn default_seele() -> Character {
    Character {
        id: "1101".into(),
        name: "希儿".into(),
        path: Path::TheHunt,
        element: Element::Quantum,
        base_hp: 1041.0,
        base_atk: 563.0,
        base_def: 330.0,
        base_spd: 115.0,
        abilities: vec![
            AbilityData {
                name: "强袭".into(),
                kind: AbilityKind::Basic,
                multiplier: 1.1,
                multipliers: vec![1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0],
                skill_level: 6,
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
                repeat: 1,
                hp_cost_pct: 0.0,
                on_deplete: false,
                summons_memo: false,
            },
            AbilityData {
                name: "归刃".into(),
                kind: AbilityKind::Skill,
                multiplier: 2.2,
                multipliers: vec![2.2, 2.4, 2.6, 2.8, 3.0, 3.2, 3.4, 3.6, 3.8, 4.0],
                skill_level: 6,
                scaling: Scaling::Atk,
                flat_damage: 0.0,
                dmg_type: DmgType::Normal,
                can_crit: true,
                toughness_reduction: 60.0,
                hits: 1,
                hit_split: vec![1.0],
                energy_gain: 30.0,
                max_energy: 120.0,
                skill_point: -1,
                bonus_sp: 0,
                target: Target::Single,
                buff: None,
                immediate_action: false,
                action_advance_pct: 0.0,
                self_advance_pct: 0.0,
                applies_debuff: false,
                heals: false,
                forced: false,
                repeat: 1,
                hp_cost_pct: 0.0,
                on_deplete: false,
                summons_memo: false,
            },
            AbilityData {
                name: "乱蝶".into(),
                kind: AbilityKind::Ult,
                multiplier: 4.0,
                multipliers: vec![4.0, 4.4, 4.8, 5.2, 5.6, 6.0, 6.4, 6.8, 7.2, 7.6],
                skill_level: 6,
                scaling: Scaling::Atk,
                flat_damage: 0.0,
                dmg_type: DmgType::Normal,
                can_crit: true,
                toughness_reduction: 90.0,
                hits: 1,
                hit_split: vec![1.0],
                energy_gain: 5.0,
                max_energy: 120.0,
                skill_point: 0,
                bonus_sp: 0,
                target: Target::Single,
                buff: None,
                immediate_action: false,
                action_advance_pct: 0.0,
                self_advance_pct: 0.0,
                applies_debuff: false,
                heals: false,
                forced: false,
                repeat: 1,
                hp_cost_pct: 0.0,
                on_deplete: false,
                summons_memo: false,
            },
        ],
        team_effects: vec![],
        has_memosprite: false,
        memosprite_spd: 0.0,
        memosprite_multiplier: 0.0,
            memosprite_explode_pct: 0.0,
            summon_at_battle_start: false,
    }
}

fn default_cone() -> LightCone {
    LightCone {
        id: "23000".into(),
        name: "于夜色中".into(),
        path: Path::TheHunt,
        rarity: 5,
        base_hp: 1058.0,
        base_atk: 582.0,
        base_def: 396.0,
        superimposition: 1,
        passive: None,
        effects: vec![],
    }
}

fn default_set() -> RelicSet {
    RelicSet {
        id: "101".into(),
        name: "快枪手（演示）".into(),
        two_piece: Some("攻击力+12%".into()),
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![],
    }
}

fn default_dummy() -> Enemy {
    let mut res = std::collections::HashMap::new();
    for e in [
        Element::Physical,
        Element::Fire,
        Element::Ice,
        Element::Lightning,
        Element::Wind,
        Element::Imaginary,
    ] {
        res.insert(e, 0.2);
    }
    res.insert(Element::Quantum, 0.0);
    Enemy {
        id: "9000".into(),
        name: "测试木桩".into(),
        level: 80,
        def: 1000.0,
        max_toughness: 120.0,
        broken: false,
        res,
        spd: 100.0,
        actions: vec![],
        weaknesses: vec![Element::Quantum],
        hp: 0.0,
    }
}
