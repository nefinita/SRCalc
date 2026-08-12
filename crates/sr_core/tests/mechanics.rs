//! 机制集成测试：共享战技点 / 动态上限 / 定向buff / 触发 / 大招插入 / 敌方机制

use sr_api::*;

fn enemy() -> Enemy {
    let mut res = std::collections::HashMap::new();
    for e in [
        Element::Physical, Element::Fire, Element::Ice, Element::Lightning, Element::Wind,
        Element::Imaginary,
    ] {
        res.insert(e, 0.2);
    }
    res.insert(Element::Quantum, 0.0);
    Enemy {
        id: "e".into(),
        name: "木桩".into(),
        level: 80,
        def: 1000.0,
        max_toughness: 120.0,
        broken: false,
        res,
        spd: 1.0,
        actions: vec![],
        weaknesses: vec![],
        hp: 0.0,
    }
}

fn ability(name: &str, kind: AbilityKind, mult: f64, sp: i32, energy: f64) -> AbilityData {
    AbilityData {
        name: name.into(),
        kind,
        multiplier: mult,
        multipliers: vec![],
        skill_level: 6,
        scaling: Scaling::Atk,
        flat_damage: 0.0,
        dmg_type: DmgType::Normal,
        can_crit: true,
        toughness_reduction: 10.0,
        hits: 1,
        hit_split: vec![1.0],
        energy_gain: energy,
        max_energy: 100.0,
        skill_point: sp,
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
    }
}

fn character(id: &str, name: &str, spd: f64, abilities: Vec<AbilityData>) -> Character {
    Character {
        id: id.into(),
        name: name.into(),
        path: Path::TheHunt,
        element: Element::Quantum,
        base_hp: 1000.0,
        base_atk: 500.0,
        base_def: 300.0,
        base_spd: spd,
        abilities,
        team_effects: vec![],
        has_memosprite: false,
        memosprite_spd: 0.0,
        memosprite_multiplier: 0.0,
            memosprite_explode_pct: 0.0,
            summon_at_battle_start: false,
    }
}

fn eff(stat: BuffStat, value: f64, turns: u32, target: BuffTarget, cap: i32, sp_on_basic: i32) -> Effect {
    Effect {
        trigger: Trigger::OnUse,
        stat,
        value,
        turns,
        target,
        cap_bonus: cap,
        sp_on_basic,
        max_stacks: 0,
    }
}

fn config(chars: Vec<Character>) -> ConfigData {
    ConfigData {
        characters: chars,
        light_cones: vec![],
        relic_sets: vec![],
        enemies: vec![enemy()],
    }
}

fn team(ids: &[&str]) -> Team {
    Team {
        members: ids
            .iter()
            .map(|id| TeamMember {
                char_id: id.to_string(),
                build: Build::default(),
            })
            .collect(),
    }
}

fn rotate(
    chars: Vec<Character>,
    ids: &[&str],
    steps: Vec<RotationStepReq>,
    battle: BattleConfig,
) -> RotationResult {
    let cfg = config(chars);
    let e = cfg.enemies[0].clone();
    sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: team(ids),
        enemy: e,
        coefficient: Default::default(),
        battle,
        steps,
        memosprite_steps: vec![],
        cycles: 1,
    })
    .expect("rotation")
}

fn basic(id: &str) -> RotationStepReq {
    RotationStepReq { char_id: id.into(), action: ActionKind::Basic, target: None }
}
fn skill(id: &str) -> RotationStepReq {
    RotationStepReq { char_id: id.into(), action: ActionKind::Skill, target: None }
}
fn ult(id: &str) -> RotationStepReq {
    RotationStepReq { char_id: id.into(), action: ActionKind::Ult, target: None }
}
fn skill_target(id: &str, target: &str) -> RotationStepReq {
    RotationStepReq { char_id: id.into(), action: ActionKind::Skill, target: Some(target.into()) }
}

#[test]
fn shared_sp_pool_basic_skill() {
    // 3 起步：普攻+1 / 战技−1 / 普攻+1 → 4,3,4
    let a = character("a", "A", 115.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        ability("战技", AbilityKind::Skill, 2.0, -1, 30.0),
    ]);
    let out = rotate(vec![a], &["a"], vec![basic("a"), skill("a"), basic("a")], BattleConfig::default());
    let sps: Vec<i32> = out.steps.iter().filter(|s| !s.is_enemy).map(|s| s.skill_point).collect();
    assert_eq!(sps, vec![4, 3, 4]);
}

#[test]
fn sp_pool_caps_at_five() {
    let a = character("a", "A", 115.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let out = rotate(vec![a], &["a"], vec![basic("a"); 6], BattleConfig::default());
    let sps: Vec<i32> = out.steps.iter().map(|s| s.skill_point).collect();
    assert_eq!(*sps.last().unwrap(), 5);
    assert!(sps.iter().all(|&s| s <= 5));
}

#[test]
fn sparkle_talent_raises_sp_cap_on_entry() {
    // 花火在场：上限 5→7，进队即生效（无需开大）
    let mut s = character("s", "花火", 99.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
    ]);
    s.team_effects = vec![eff(BuffStat::AtkPct, 0.0, 0, BuffTarget::Team, 2, 0)];
    let a = character("a", "A", 115.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let out = rotate(vec![s, a], &["s", "a"], vec![basic("a"); 6], BattleConfig::default());
    let sps: Vec<i32> = out.steps.iter().map(|x| x.skill_point).collect();
    assert_eq!(*sps.last().unwrap(), 7);
    assert!(sps.iter().all(|&s| s <= 7));
}

#[test]
fn ult_bonus_sp_and_insertion_av() {
    // 大招：恢复 bonus_sp，不占行动值，AV 与下一行动一致
    let mut s = character("s", "花火", 99.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        AbilityData {
            name: "一人千役".into(),
            kind: AbilityKind::Ult,
            multiplier: 2.0,
            multipliers: vec![],
            skill_level: 6,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 10.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 5.0,
            max_energy: 100.0,
            skill_point: 0,
            bonus_sp: 4,
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
    ]);
    s.team_effects = vec![eff(BuffStat::AtkPct, 0.0, 0, BuffTarget::Team, 2, 0)];
    let a = character("a", "A", 115.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let steps = vec![basic("a"), ult("s"), basic("a")];
    let out = rotate(
        vec![s, a],
        &["s", "a"],
        steps,
        BattleConfig { start_energy: 100.0, ..Default::default() },
    );
    assert_eq!(out.steps.len(), 3);
    // 大招 AV 与下一个普攻一致（下一行动前结算）
    assert!((out.steps[1].av - out.steps[2].av).abs() < 1e-6);
    // 大招 SP：3 + 4 = 7（上限 7）
    assert_eq!(out.steps[1].skill_point, 7);
}

#[test]
fn ally_buff_targets_and_immediate_action() {
    // 布洛妮娅战技→A：立即行动（AV 归0）+ 增伤 buff
    let b = character("b", "布洛妮娅", 99.0, vec![AbilityData {
        name: "战技".into(),
        kind: AbilityKind::Skill,
        multiplier: 0.0,
        multipliers: vec![],
        skill_level: 6,
        scaling: Scaling::Atk,
        flat_damage: 0.0,
        dmg_type: DmgType::Normal,
        can_crit: false,
        toughness_reduction: 0.0,
        hits: 1,
        hit_split: vec![1.0],
        energy_gain: 30.0,
        max_energy: 120.0,
        skill_point: -1,
        bonus_sp: 0,
        target: Target::Single,
        buff: Some(eff(BuffStat::DmgPct, 0.33, 1, BuffTarget::Ally, 0, 0)),
        immediate_action: true,
        action_advance_pct: 0.0,
        self_advance_pct: 0.0,
                applies_debuff: false,
                heals: false,
                forced: false,
                repeat: 1,
            hp_cost_pct: 0.0,
            on_deplete: false,
            summons_memo: false,
    }]);
    let a = character("a", "A", 115.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
    ]);
    let steps = vec![basic("a"), skill_target("b", "a"), basic("a")];
    let out = rotate(vec![b, a], &["b", "a"], steps, BattleConfig::default());
    // A 被立即行动后，下一次 A 行动 AV 与布洛妮娅战技相同
    assert!((out.steps[1].av - out.steps[2].av).abs() < 1e-6);
    // 增伤 buff 生效：第二次普攻 > 第一次
    assert!(out.steps[2].damage > out.steps[0].damage);
}

#[test]
fn enemy_actions_energy_sp_drain() {
    // 敌方：快、攻击回能、特殊行动回SP/扣能
    let mut e = enemy();
    e.spd = 200.0; // av 50
    e.actions = vec![
        EnemyAbility { name: "攻击".into(), energy_gain_players: 10.0, sp_delta: 0, energy_drain: 0.0 },
        EnemyAbility { name: "回能回SP".into(), energy_gain_players: 5.0, sp_delta: 2, energy_drain: 0.0 },
        EnemyAbility { name: "扣能".into(), energy_gain_players: 0.0, sp_delta: 0, energy_drain: 30.0 },
    ];
    let a = character("a", "A", 50.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![],
        enemies: vec![e.clone()],
    };
    // A av=200，敌方 av=50 → 敌方先行动 3 次（150），A 再行动
    let steps = vec![basic("a")];
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: team(&["a"]),
        enemy: e,
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        steps,
        memosprite_steps: vec![],
        cycles: 1,
    })
    .expect("rotation");
    let enemy_steps: Vec<_> = out.steps.iter().filter(|s| s.is_enemy).collect();
    assert_eq!(enemy_steps.len(), 3);
    assert_eq!(enemy_steps[0].enemy_ability.as_deref(), Some("攻击"));
    assert_eq!(enemy_steps[1].enemy_ability.as_deref(), Some("回能回SP"));
    assert_eq!(enemy_steps[2].enemy_ability.as_deref(), Some("扣能"));
    // 能量：0 +10 +5 +20(扣能到0) → 结束 +20 普攻 = 20
    let last_player = out.steps.last().unwrap();
    assert!((last_player.energy - 20.0).abs() < 1e-6);
    // SP：3 +2(回SP) +1(普攻) = 6 → 但 cap=5 → 5
    assert_eq!(last_player.skill_point, 5);
}

#[test]
fn sp_on_basic_brand() {
    // 寒鸦"罚恶"：目标普攻时全队+1 SP
    let h = character("h", "寒鸦", 99.0, vec![AbilityData {
        name: "战技".into(),
        kind: AbilityKind::Skill,
        multiplier: 0.0,
        multipliers: vec![],
        skill_level: 6,
        scaling: Scaling::Atk,
        flat_damage: 0.0,
        dmg_type: DmgType::Normal,
        can_crit: false,
        toughness_reduction: 0.0,
        hits: 1,
        hit_split: vec![1.0],
        energy_gain: 30.0,
        max_energy: 100.0,
        skill_point: -1,
        bonus_sp: 0,
        target: Target::Single,
        buff: Some(Effect {
            trigger: Trigger::OnUse,
            stat: BuffStat::AtkPct,
            value: 0.0,
            turns: 2,
            target: BuffTarget::Ally,
            cap_bonus: 0,
            sp_on_basic: 1,
            max_stacks: 0,
        }),
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
    }]);
    let a = character("a", "A", 115.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    // 寒鸦战技(-1)→3-1=2；A 普攻 +1(普攻)+1(罚恶)=4
    let steps = vec![skill_target("h", "a"), basic("a"), basic("a")];
    let out = rotate(vec![h, a], &["h", "a"], steps, BattleConfig::default());
    let sps: Vec<i32> = out.steps.iter().map(|s| s.skill_point).collect();
    assert_eq!(sps, vec![2, 4, 5]);
}

#[test]
fn per_skill_sp_cost() {
    // 强化普攻扣 2 点 SP（饮月类）
    let a = character("a", "A", 115.0, vec![AbilityData {
        name: "强化普攻".into(),
        kind: AbilityKind::Basic,
        multiplier: 1.0,
        multipliers: vec![],
        skill_level: 6,
        scaling: Scaling::Atk,
        flat_damage: 0.0,
        dmg_type: DmgType::Normal,
        can_crit: true,
        toughness_reduction: 10.0,
        hits: 1,
        hit_split: vec![1.0],
        energy_gain: 20.0,
        max_energy: 100.0,
        skill_point: -2,
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
    }]);
    let out = rotate(vec![a], &["a"], vec![basic("a")], BattleConfig::default());
    assert_eq!(out.steps[0].skill_point, 1); // 3 - 2
}

#[test]
fn on_sp_consume_stacks_team_damage() {
    // 花火天赋：消耗SP → 全队伤害+3% 叠3层
    let mut s = character("s", "花火", 99.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
    ]);
    s.team_effects = vec![Effect {
        trigger: Trigger::OnSpConsume,
        stat: BuffStat::DmgPct,
        value: 0.03,
        turns: 2,
        target: BuffTarget::Team,
        cap_bonus: 0,
        sp_on_basic: 0,
        max_stacks: 3,
    }];
    let a = character("a", "A", 115.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        ability("战技", AbilityKind::Skill, 2.0, -1, 30.0),
    ]);
    // A 普攻（无buff）→ 战技（触发1层）→ 战技（触发2层）→ 普攻（+6%）
    let steps = vec![basic("a"), skill("a"), skill("a"), basic("a")];
    let out = rotate(vec![s, a], &["s", "a"], steps, BattleConfig::default());
    let dmg: Vec<f64> = out.steps.iter().filter(|x| !x.is_enemy).map(|x| x.damage).collect();
    assert!(dmg[3] > dmg[0] * 1.05, "期望 dmg3≈1.06×dmg0, got {:.2} vs {:.2}", dmg[3], dmg[0]);
}

#[test]
fn weakness_break_system() {
    // 韧性 30，量子弱点；希儿类普攻削韧 10 → 第 3 下破韧
    let mut e = enemy();
    e.max_toughness = 30.0;
    e.weaknesses = vec![Element::Quantum];
    e.broken = false;
    let a = character("a", "A", 115.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![],
        enemies: vec![e.clone()],
    };
    let mut build = Build::default();
    build.level = 80;
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: e,
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        memosprite_steps: vec![],
        steps: vec![basic("a"); 4],
        cycles: 1,
    })
    .expect("rotation");
    let dmg: Vec<f64> = out.steps.iter().map(|s| s.damage).collect();
    // 前两下未破韧（×0.9），第三下破韧（含破韧伤害），第四下破韧后（×1.0）
    assert!((dmg[0] - dmg[1]).abs() < 1e-6, "d0={:.2} d1={:.2}", dmg[0], dmg[1]);
    assert!(dmg[2] > dmg[1] * 3.0, "破韧伤害未计入 d2={:.2}", dmg[2]);
    assert!(dmg[3] > dmg[0], "破韧后伤害应提升 d3={:.2} d0={:.2}", dmg[3], dmg[0]);
    assert!(out.steps[2].buffs.contains(&"破韧".to_string()));
}

#[test]
fn non_weakness_no_break() {
    // 敌人弱风，角色量子 → 不削韧不破韧
    let mut e = enemy();
    e.max_toughness = 30.0;
    e.weaknesses = vec![Element::Wind];
    let a = character("a", "A", 115.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![],
        enemies: vec![e.clone()],
    };
    let mut build = Build::default();
    build.level = 80;
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: e,
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        memosprite_steps: vec![],
        steps: vec![basic("a"); 4],
        cycles: 1,
    })
    .expect("rotation");
    let dmg: Vec<f64> = out.steps.iter().map(|s| s.damage).collect();
    assert!((dmg[3] - dmg[0]).abs() < 1e-6);
}

#[test]
fn sp_overflow_recording() {
    // 花火：上限+2=7，开局满SP；大招回4 → 溢出记录4；战技-1 → 从溢出补回
    let mut s = character("s", "花火", 99.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        ability("战技", AbilityKind::Skill, 2.0, -1, 30.0),
        AbilityData {
            name: "一人千役".into(),
            kind: AbilityKind::Ult,
            multiplier: 2.0,
            multipliers: vec![],
            skill_level: 6,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 10.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 5.0,
            max_energy: 100.0,
            skill_point: 0,
            bonus_sp: 4,
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
    ]);
    s.team_effects = vec![eff(BuffStat::AtkPct, 0.0, 0, BuffTarget::Team, 2, 0)];
    let out = rotate(
        vec![s],
        &["s"],
        vec![ult("s"), skill("s")],
        BattleConfig { start_sp: 7, start_energy: 100.0, ..Default::default() },
    );
    // 大招：7 + 4 → 溢出记4，仍 7
    assert_eq!(out.steps[0].skill_point, 7);
    // 战技：-1 → 6 → 从溢出补回 1 → 7
    assert_eq!(out.steps[1].skill_point, 7);
}

#[test]
fn relic_set_effects_apply() {
    // 太空封印站 2件：攻击+12%
    let set = sr_api::RelicSet {
        id: "301".into(),
        name: "太空封印站".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![Effect {
            trigger: Trigger::OnUse,
            stat: BuffStat::AtkPct,
            value: 0.12,
            turns: 0,
            target: BuffTarget::Self_,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 0,
        }],
        four_piece_effects: vec![],
    };
    let mut a = character("a", "A", 115.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    a.base_atk = 500.0;
    let mut build = Build::default();
    build.level = 80;
    build.relic_sets = vec![sr_api::RelicSetPiece { set_id: "301".into(), count: 2 }];
    let cfg = ConfigData {
        characters: vec![a.clone()],
        light_cones: vec![],
        relic_sets: vec![set],
        enemies: vec![enemy()],
    };
    // 有套装：攻击 500→560
    let stats = engine_stats(&cfg, &a, &build);
    assert!((stats.atk - 560.0).abs() < 1e-6, "atk={}", stats.atk);
}

fn engine_stats(
    cfg: &ConfigData,
    character: &Character,
    build: &Build,
) -> sr_core::engine::FinalStats {
    let cone = None;
    let allies: Vec<&Character> = vec![];
    let sets: Vec<&sr_api::RelicSet> = cfg.relic_sets.iter().collect();
    let mut permanent = sr_core::engine::presence_mods(character, cone, &allies);
    permanent.add(&sr_core::engine::relic_set_mods(build, &sets));
    sr_core::engine::compute_final_stats(character, cone, build, &permanent)
}

#[test]
fn conditional_set_effect_on_ult_expires() {
    // 密林卧雪 4件：终结技后 暴伤+25%·2回合
    let set = sr_api::RelicSet {
        id: "104".into(),
        name: "密林卧雪".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![Effect {
            trigger: Trigger::OnUlt,
            stat: BuffStat::CritDmg,
            value: 0.25,
            turns: 2,
            target: BuffTarget::Self_,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 0,
        }],
    };
    let a = character("a", "A", 115.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        AbilityData {
            name: "终结技".into(),
            kind: AbilityKind::Ult,
            multiplier: 2.0,
            multipliers: vec![],
            skill_level: 6,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 10.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 5.0,
            max_energy: 100.0,
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
    ]);
    let mut build = Build::default();
    build.level = 80;
    build.relic_sets = vec![sr_api::RelicSetPiece { set_id: "104".into(), count: 4 }];
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![set],
        enemies: vec![enemy()],
    };
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: enemy(),
        coefficient: Default::default(),
        battle: BattleConfig { start_energy: 100.0, ..Default::default() },
        memosprite_steps: vec![],
        steps: vec![ult("a"), basic("a"), basic("a"), basic("a")],
        cycles: 1,
    })
    .expect("rotation");
    let dmg: Vec<f64> = out.steps.iter().map(|s| s.damage).collect();
    // 终结技后两下普攻带暴伤+25%，第三下普攻 buff 已过期
    assert!(dmg[1] > dmg[3], "buff 生效期应更高 d1={:.3} d3={:.3}", dmg[1], dmg[3]);
    assert!(dmg[2] > dmg[3], "buff 第二回合应更高 d2={:.3} d3={:.3}", dmg[2], dmg[3]);
    assert!((dmg[1] - dmg[2]).abs() < 1e-6);
}

#[test]
fn on_hit_set_stacks_crit_rate() {
    // 莳者 4件：受击 → 暴率+8%·2回合，叠2层
    let set = sr_api::RelicSet {
        id: "113".into(),
        name: "莳者".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![Effect {
            trigger: Trigger::OnHit,
            stat: BuffStat::CritRate,
            value: 0.08,
            turns: 2,
            target: BuffTarget::Self_,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 2,
        }],
    };
    let a = character("a", "A", 200.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let mut build = Build::default();
    build.level = 80;
    build.relic_sets = vec![sr_api::RelicSetPiece { set_id: "113".into(), count: 4 }];
    let mut e = enemy();
    e.spd = 100.0; // 与角色交错
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![set],
        enemies: vec![e.clone()],
    };
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: e,
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        memosprite_steps: vec![],
        steps: vec![basic("a"); 6],
        cycles: 1,
    })
    .expect("rotation");
    let dmg: Vec<f64> = out.steps.iter().filter(|s| !s.is_enemy).map(|s| s.damage).collect();
    assert!(dmg[dmg.len() - 1] > dmg[0], "受击叠层后暴率应提升 last={:.2} first={:.2}", dmg[dmg.len()-1], dmg[0]);
}

#[test]
fn ally_target_set_buff_applies_and_expires() {
    // 司铎 4件：对目标施放战技/终结技 → 目标暴伤+18%·2回合·叠2
    let set = sr_api::RelicSet {
        id: "121".into(),
        name: "司铎".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![Effect {
            trigger: Trigger::OnSkill,
            stat: BuffStat::CritDmg,
            value: 0.18,
            turns: 2,
            target: BuffTarget::Ally,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 2,
        }],
    };
    let a = character("a", "A", 200.0, vec![AbilityData {
        name: "战技".into(),
        kind: AbilityKind::Skill,
        multiplier: 0.0,
        multipliers: vec![],
        skill_level: 6,
        scaling: Scaling::Atk,
        flat_damage: 0.0,
        dmg_type: DmgType::Normal,
        can_crit: false,
        toughness_reduction: 0.0,
        hits: 1,
        hit_split: vec![1.0],
        energy_gain: 30.0,
        max_energy: 100.0,
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
    }]);
    let b = character("b", "B", 200.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let mut build = Build::default();
    build.level = 80;
    build.relic_sets = vec![sr_api::RelicSetPiece { set_id: "121".into(), count: 4 }];
    let cfg = ConfigData {
        characters: vec![a, b],
        light_cones: vec![],
        relic_sets: vec![set],
        enemies: vec![enemy()],
    };
    let steps = vec![
        RotationStepReq { char_id: "a".into(), action: ActionKind::Skill, target: Some("b".into()) },
        basic("b"),
        basic("b"),
        basic("b"),
    ];
    let team = Team {
        members: vec![
            TeamMember { char_id: "a".into(), build },
            TeamMember { char_id: "b".into(), build: Build { level: 80, ..Default::default() } },
        ],
    };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team,
        enemy: enemy(),
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        steps,
        memosprite_steps: vec![],
        cycles: 1,
    })
    .expect("rotation");
    let dmg: Vec<f64> = out.steps.iter().filter(|s| !s.is_enemy && s.char_id == "b").map(|s| s.damage).collect();
    // B 两下普攻带暴伤+18%，第三下过期
    assert!(dmg[0] > dmg[2], "buff 期应更高 b0={:.3} b2={:.3}", dmg[0], dmg[2]);
    assert!((dmg[0] - dmg[1]).abs() < 1e-6);
}

#[test]
fn sp_consume_threshold_set() {
    // 天国直播间 2件：同回合消耗≥3战技点 → 暴伤+32%·3回合
    let set = sr_api::RelicSet {
        id: "324".into(),
        name: "天国直播间".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![Effect {
            trigger: Trigger::OnSpConsume,
            stat: BuffStat::CritDmg,
            value: 0.32,
            turns: 3,
            target: BuffTarget::Self_,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 0,
        }],
        four_piece_effects: vec![],
    };
    let a = character("a", "A", 200.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        AbilityData {
            name: "强化战技".into(),
            kind: AbilityKind::Skill,
            multiplier: 2.0,
            multipliers: vec![],
            skill_level: 6,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 10.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 30.0,
            max_energy: 100.0,
            skill_point: -3,
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
    ]);
    let run = |with_set: bool| {
        let mut b = Build::default();
        b.level = 80;
        if with_set {
            b.relic_sets = vec![sr_api::RelicSetPiece { set_id: "324".into(), count: 2 }];
        }
        let cfg = ConfigData {
            characters: vec![a.clone()],
            light_cones: vec![],
            relic_sets: vec![set.clone()],
            enemies: vec![enemy()],
        };
        let tm = TeamMember { char_id: "a".into(), build: b };
        let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
            config: cfg,
            team: Team { members: vec![tm] },
            enemy: enemy(),
            coefficient: Default::default(),
            battle: BattleConfig { start_sp: 5, ..Default::default() },
            memosprite_steps: vec![],
        steps: vec![skill("a"), basic("a")],
            cycles: 1,
        })
        .expect("rotation");
        out.steps[1].damage
    };
    // 强化战技一次消耗 3 点 → 触发暴伤+32%，下个普攻受益
    let without = run(false);
    let with = run(true);
    assert!(with > without, "阈值触发后普攻应提升 with={:.2} without={:.2}", with, without);
}

#[test]
fn on_attack_break_buff() {
    // 劫火 2件：攻击命中 → 击破特攻+40%·1回合（下一次击破受益）
    let set = sr_api::RelicSet {
        id: "316".into(),
        name: "劫火铸炼宫".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![Effect {
            trigger: Trigger::OnAttack,
            stat: BuffStat::BreakEffect,
            value: 0.40,
            turns: 1,
            target: BuffTarget::Self_,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 0,
        }],
        four_piece_effects: vec![],
    };
    let a = character("a", "A", 200.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let mut e = enemy();
    e.max_toughness = 20.0; // 两下普攻破韧，第二次破韧时带击破加成
    e.weaknesses = vec![Element::Quantum];
    let run = |with_set: bool| {
        let mut build = Build::default();
        build.level = 80;
        if with_set {
            build.relic_sets = vec![sr_api::RelicSetPiece { set_id: "316".into(), count: 2 }];
        }
        let cfg = ConfigData {
            characters: vec![a.clone()],
            light_cones: vec![],
            relic_sets: vec![set.clone()],
            enemies: vec![e.clone()],
        };
        let tm = TeamMember { char_id: "a".into(), build };
        let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
            config: cfg,
            team: Team { members: vec![tm] },
            enemy: e.clone(),
            coefficient: Default::default(),
            battle: BattleConfig::default(),
            memosprite_steps: vec![],
        steps: vec![basic("a"), basic("a")],
            cycles: 1,
        })
        .expect("rotation");
        out.steps[1].damage
    };
    let without = run(false);
    let with = run(true);
    assert!(with > without, "击破加成后破韧伤害应更高 with={:.2} without={:.2}", with, without);
}

#[test]
fn ult_dmg_type_stat() {
    // 终结技伤害% 常驻效果 → 终结技伤害提升（普攻不受影响）
    let set = sr_api::RelicSet {
        id: "t".into(),
        name: "测试".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![Effect {
            trigger: Trigger::BattleStart,
            stat: BuffStat::UltDmgPct,
            value: 0.5,
            turns: 0,
            target: BuffTarget::Self_,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 0,
        }],
    };
    let a = character("a", "A", 200.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        AbilityData {
            name: "终结技".into(),
            kind: AbilityKind::Ult,
            multiplier: 2.0,
            multipliers: vec![],
            skill_level: 6,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 10.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 5.0,
            max_energy: 100.0,
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
    ]);
    let run = |with_set: bool| {
        let mut build = Build::default();
        build.level = 80;
        if with_set {
            build.relic_sets = vec![sr_api::RelicSetPiece { set_id: "t".into(), count: 4 }];
        }
        let cfg = ConfigData {
            characters: vec![a.clone()],
            light_cones: vec![],
            relic_sets: vec![set.clone()],
            enemies: vec![enemy()],
        };
        let tm = TeamMember { char_id: "a".into(), build };
        let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
            config: cfg,
            team: Team { members: vec![tm] },
            enemy: enemy(),
            coefficient: Default::default(),
            battle: BattleConfig { start_energy: 100.0, ..Default::default() },
            memosprite_steps: vec![],
        steps: vec![ult("a"), basic("a")],
            cycles: 1,
        })
        .expect("rotation");
        (out.steps[0].damage, out.steps[1].damage)
    };
    let (u0, b0) = run(false);
    let (u1, b1) = run(true);
    assert!(u1 > u0 * 1.4, "终结技伤害应+50% u1={:.2} u0={:.2}", u1, u0);
    assert!((b1 - b0).abs() < 1e-6, "普攻不受终结技增伤影响");
}

#[test]
fn enemy_kill_detection_and_on_kill() {
    // 千星 2件：消灭敌人 → 全队暴伤+12%（本场永久）
    let set = sr_api::RelicSet {
        id: "326".into(),
        name: "千星".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![Effect {
            trigger: Trigger::OnKill,
            stat: BuffStat::CritDmg,
            value: 0.12,
            turns: 0,
            target: BuffTarget::Team,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 0,
        }],
        four_piece_effects: vec![],
    };
    let a = character("a", "A", 200.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    let mut build = Build::default();
    build.level = 80;
    build.relic_sets = vec![sr_api::RelicSetPiece { set_id: "326".into(), count: 2 }];
    let mut e = enemy();
    e.hp = 500.0; // 约 2 下半血，第 3 下击杀
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![set],
        enemies: vec![e.clone()],
    };
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: e,
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        memosprite_steps: vec![],
        steps: vec![basic("a"); 4],
        cycles: 1,
    })
    .expect("rotation");
    assert!(out.steps[2].buffs.contains(&"击杀".to_string()), "第3下应击杀: {:?}", out.steps[2].buffs);
    // 击杀后全队暴伤+12% 永久生效 → 后续普攻提升
    assert!(out.steps[3].damage > out.steps[0].damage, "击杀后伤害应提升 d3={:.3} d0={:.3}", out.steps[3].damage, out.steps[0].damage);
}

#[test]
fn on_apply_debuff_set() {
    // 名冶 4件：施加负面 → 全队伤害+15%·2回合
    let set = sr_api::RelicSet {
        id: "132".into(),
        name: "名冶".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![Effect {
            trigger: Trigger::OnApplyDebuff,
            stat: BuffStat::DmgPct,
            value: 0.15,
            turns: 2,
            target: BuffTarget::Team,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 0,
        }],
    };
    let a = character("a", "A", 200.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        AbilityData {
            name: "战技·施放负面".into(),
            kind: AbilityKind::Skill,
            multiplier: 2.0,
            multipliers: vec![],
            skill_level: 6,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 10.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 30.0,
            max_energy: 100.0,
            skill_point: -1,
            bonus_sp: 0,
            target: Target::Single,
            buff: None,
            immediate_action: false,
            action_advance_pct: 0.0,
            self_advance_pct: 0.0,
            applies_debuff: true,
            heals: false,
            forced: false,
                repeat: 1,
            hp_cost_pct: 0.0,
            on_deplete: false,
            summons_memo: false,
        },
    ]);
    let run = |with_set: bool| {
        let mut b = Build::default();
        b.level = 80;
        if with_set {
            b.relic_sets = vec![sr_api::RelicSetPiece { set_id: "132".into(), count: 4 }];
        }
        let cfg = ConfigData {
            characters: vec![a.clone()],
            light_cones: vec![],
            relic_sets: vec![set.clone()],
            enemies: vec![enemy()],
        };
        let tm = TeamMember { char_id: "a".into(), build: b };
        let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
            config: cfg,
            team: Team { members: vec![tm] },
            enemy: enemy(),
            coefficient: Default::default(),
            battle: BattleConfig::default(),
            memosprite_steps: vec![],
        steps: vec![skill("a"), basic("a")],
            cycles: 1,
        })
        .expect("rotation");
        out.steps[1].damage
    };
    assert!(run(true) > run(false), "施加负面后全队增伤应提升");
}

#[test]
fn on_heal_set() {
    // 烈阳女武神 4件：治疗 → 全队暴伤+15%
    let set = sr_api::RelicSet {
        id: "125".into(),
        name: "女武神".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![Effect {
            trigger: Trigger::OnHeal,
            stat: BuffStat::CritDmg,
            value: 0.15,
            turns: 2,
            target: BuffTarget::Team,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 0,
        }],
    };
    let a = character("a", "A", 200.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        AbilityData {
            name: "战技·治疗".into(),
            kind: AbilityKind::Skill,
            multiplier: 0.0,
            multipliers: vec![],
            skill_level: 6,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: false,
            toughness_reduction: 0.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 30.0,
            max_energy: 100.0,
            skill_point: -1,
            bonus_sp: 0,
            target: Target::Single,
            buff: None,
            immediate_action: false,
            action_advance_pct: 0.0,
            self_advance_pct: 0.0,
            applies_debuff: false,
            heals: true,
            forced: false,
                repeat: 1,
            hp_cost_pct: 0.0,
            on_deplete: false,
            summons_memo: false,
        },
    ]);
    let run = |with_set: bool| {
        let mut b = Build::default();
        b.level = 80;
        if with_set {
            b.relic_sets = vec![sr_api::RelicSetPiece { set_id: "125".into(), count: 4 }];
        }
        let cfg = ConfigData {
            characters: vec![a.clone()],
            light_cones: vec![],
            relic_sets: vec![set.clone()],
            enemies: vec![enemy()],
        };
        let tm = TeamMember { char_id: "a".into(), build: b };
        let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
            config: cfg,
            team: Team { members: vec![tm] },
            enemy: enemy(),
            coefficient: Default::default(),
            battle: BattleConfig::default(),
            memosprite_steps: vec![],
        steps: vec![skill("a"), basic("a")],
            cycles: 1,
        })
        .expect("rotation");
        out.steps[1].damage
    };
    assert!(run(true) > run(false), "治疗后全队暴伤应提升");
}

#[test]
#[test]
fn captain_charge_ult_buff() {
    // 船长 4件：成为目标 2 次(助力2) → 终结技消耗 → 攻+48%·1回合
    let set = sr_api::RelicSet {
        id: "126".into(),
        name: "船长".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![
            Effect {
                trigger: Trigger::OnTargeted,
                stat: BuffStat::AtkPct,
                value: 0.0,
                turns: 0,
                target: BuffTarget::Self_,
                cap_bonus: 0,
                sp_on_basic: 0,
                max_stacks: 2,
            },
            Effect {
                trigger: Trigger::OnUlt,
                stat: BuffStat::AtkPct,
                value: 0.48,
                turns: 1,
                target: BuffTarget::Self_,
                cap_bonus: 0,
                sp_on_basic: 0,
                max_stacks: 0,
            },
        ],
    };
    let a = character("a", "A", 200.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        AbilityData {
        name: "终结技".into(),
        kind: AbilityKind::Ult,
        multiplier: 2.0,
        multipliers: vec![],
        skill_level: 6,
        scaling: Scaling::Atk,
        flat_damage: 0.0,
        dmg_type: DmgType::Normal,
        can_crit: true,
        toughness_reduction: 10.0,
        hits: 1,
        hit_split: vec![1.0],
        energy_gain: 5.0,
        max_energy: 100.0,
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
    ]);
    let b = character("b", "B", 200.0, vec![ability("战技", AbilityKind::Skill, 0.0, -1, 30.0)]);
    let run = |team: Team, steps: Vec<RotationStepReq>| {
        let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
            config: ConfigData {
                characters: vec![a.clone(), b.clone()],
                light_cones: vec![],
                relic_sets: vec![set.clone()],
                enemies: vec![enemy()],
            },
            team,
            enemy: enemy(),
            coefficient: Default::default(),
            battle: BattleConfig { start_energy: 100.0, ..Default::default() },
            steps,
            memosprite_steps: vec![],
            cycles: 1,
        })
        .expect("rotation");
        out.steps[out.steps.len() - 1].damage
    };
    let team_a = Team {
        members: vec![TeamMember {
            char_id: "a".into(),
            build: Build { level: 80, relic_sets: vec![sr_api::RelicSetPiece { set_id: "126".into(), count: 4 }], ..Default::default() },
        }],
    };
    let team_ab = Team {
        members: vec![
            TeamMember { char_id: "a".into(), build: Build { level: 80, relic_sets: vec![sr_api::RelicSetPiece { set_id: "126".into(), count: 4 }], ..Default::default() } },
            TeamMember { char_id: "b".into(), build: Build { level: 80, ..Default::default() } },
        ],
    };
    // 助力未满：终结技后普攻无加成
    let no_charge = run(team_a, vec![ult("a"), basic("a")]);
    // 助力满（B 战技指向 A 两次）：终结技消耗助力 → 攻+48% → 后续普攻提升
    let full_charge = run(
        team_ab,
        vec![
            RotationStepReq { char_id: "b".into(), action: ActionKind::Skill, target: Some("a".into()) },
            RotationStepReq { char_id: "b".into(), action: ActionKind::Skill, target: Some("a".into()) },
            ult("a"),
            basic("a"),
        ],
    );
    assert!(full_charge > no_charge * 1.4, "助力满终结技后普攻应+48% full={:.2} none={:.2}", full_charge, no_charge);
}

#[test]
fn death_water_amplify_on_debuff() {
    // 死水深潜 4件：对负面敌人暴伤+8%（常驻）；施加负面后翻倍(+8%·1回合)
    let set = sr_api::RelicSet {
        id: "117".into(),
        name: "死水深潜".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![
            Effect {
                trigger: Trigger::BattleStart,
                stat: BuffStat::CritDmg,
                value: 0.08,
                turns: 0,
                target: BuffTarget::Self_,
                cap_bonus: 0,
                sp_on_basic: 0,
                max_stacks: 0,
            },
            Effect {
                trigger: Trigger::OnApplyDebuff,
                stat: BuffStat::CritDmg,
                value: 0.08,
                turns: 1,
                target: BuffTarget::Self_,
                cap_bonus: 0,
                sp_on_basic: 0,
                max_stacks: 0,
            },
        ],
    };
    let a = character("a", "A", 200.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        AbilityData {
            name: "战技·施放负面".into(),
            kind: AbilityKind::Skill,
            multiplier: 2.0,
            multipliers: vec![],
            skill_level: 6,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 10.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 30.0,
            max_energy: 100.0,
            skill_point: -1,
            bonus_sp: 0,
            target: Target::Single,
            buff: None,
            immediate_action: false,
            action_advance_pct: 0.0,
            self_advance_pct: 0.0,
            applies_debuff: true,
            heals: false,
            forced: false,
                repeat: 1,
            hp_cost_pct: 0.0,
            on_deplete: false,
            summons_memo: false,
        },
    ]);
    let run = |steps: Vec<RotationStepReq>| {
        let mut b = Build::default();
        b.level = 80;
        b.relic_sets = vec![sr_api::RelicSetPiece { set_id: "117".into(), count: 4 }];
        let cfg = ConfigData {
            characters: vec![a.clone()],
            light_cones: vec![],
            relic_sets: vec![set.clone()],
            enemies: vec![enemy()],
        };
        let tm = TeamMember { char_id: "a".into(), build: b };
        let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
            config: cfg,
            team: Team { members: vec![tm] },
            enemy: enemy(),
            coefficient: Default::default(),
            battle: BattleConfig::default(),
            steps,
            memosprite_steps: vec![],
            cycles: 1,
        })
        .expect("rotation");
        out.steps[out.steps.len() - 1].damage
    };
    // 无施加负面：仅常驻 +8% 暴伤
    let no_debuff = run(vec![basic("a"), basic("a")]);
    // 施加负面后翻倍：常驻 +8% + 1回合额外 +8% = 16%
    let with_debuff = run(vec![skill("a"), basic("a")]);
    assert!(with_debuff > no_debuff, "施加负面后暴伤翻倍应更高 amp={:.2} base={:.2}", with_debuff, no_debuff);
}

#[test]
fn memosprite_attack_crit_buff() {
    // 凯歌英豪 4件：忆灵攻击（忆灵独立行动）→ 暴伤+30%·2回合
    let set = sr_api::RelicSet {
        id: "123".into(),
        name: "凯歌英豪".into(),
        two_piece: None,
        four_piece: None,
        two_piece_effects: vec![],
        four_piece_effects: vec![Effect {
            trigger: Trigger::OnMemospriteAttack,
            stat: BuffStat::CritDmg,
            value: 0.30,
            turns: 2,
            target: BuffTarget::Self_,
            cap_bonus: 0,
            sp_on_basic: 0,
            max_stacks: 0,
        }],
    };
    let mut a = character("a", "A", 200.0, vec![ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0)]);
    a.has_memosprite = true;
    a.summon_at_battle_start = true;
    a.memosprite_spd = 50.0;   // AV 200 → 主角动 4 次后忆灵行动一次
    a.memosprite_multiplier = 1.0;
    let mut build = Build::default();
    build.level = 80;
    build.relic_sets = vec![sr_api::RelicSetPiece { set_id: "123".into(), count: 4 }];
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![set],
        enemies: vec![enemy()],
    };
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: enemy(),
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        memosprite_steps: vec![],
        steps: vec![basic("a"); 8],
        cycles: 1,
    })
    .expect("rotation");
    // 忆灵独立行动出现在时间轴
    assert!(out.steps.iter().any(|s| s.buffs.contains(&"忆灵攻击".to_string())), "应有忆灵攻击步骤");
    let dmg: Vec<f64> = out.steps.iter().filter(|s| !s.is_enemy && !s.buffs.contains(&"忆灵攻击".to_string())).map(|s| s.damage).collect();
    // 忆灵行动后两下普攻提升，之后过期回落
    assert!(dmg[dmg.len() - 3] > dmg[0], "忆灵攻击后普攻应提升 d={:.3} base={:.3}", dmg[dmg.len()-3], dmg[0]);
    assert!(dmg[dmg.len() - 1] < dmg[dmg.len() - 3], "buff 过期应回落 d_last={:.3}", dmg[dmg.len()-1]);
}

#[test]
fn forced_memosprite_skill_overrides_queue() {
    // 死龙/长夜月类：忆灵强制技能无视队列选择，必放
    let mut a = character("a", "A", 200.0, vec![
        sr_api::AbilityData {
            name: "忆灵·普通".into(),
            kind: sr_api::AbilityKind::Memosprite,
            multiplier: 1.0,
            multipliers: vec![],
            skill_level: 1,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 0.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 0.0,
            max_energy: 0.0,
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
        sr_api::AbilityData {
            name: "忆灵·强制".into(),
            kind: sr_api::AbilityKind::Memosprite,
            multiplier: 3.0,
            multipliers: vec![],
            skill_level: 1,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 0.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 0.0,
            max_energy: 0.0,
            skill_point: 0,
            bonus_sp: 0,
            target: Target::Single,
            buff: None,
            immediate_action: false,
            action_advance_pct: 0.0,
            self_advance_pct: 0.0,
            applies_debuff: false,
            heals: false,
            forced: true,
            repeat: 1,
            hp_cost_pct: 0.0,
            on_deplete: false,
            summons_memo: false,
        },
    ]);
    a.has_memosprite = true;
    a.summon_at_battle_start = true;
    a.memosprite_spd = 100.0;
    let mut build = Build::default();
    build.level = 80;
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![],
        enemies: vec![enemy()],
    };
    let tm = TeamMember { char_id: "a".into(), build };
    // 队列选 index 0（倍率1.0），但强制技能应覆盖 → 用倍率 3.0
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: enemy(),
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        steps: vec![basic("a"); 4],
        memosprite_steps: vec![sr_api::MemospriteStepReq {
            owner_id: "a".into(),
            ability_index: 0,
            target: None,
        }],
        cycles: 1,
    })
    .expect("rotation");
    let memo = out.steps.iter().find(|s| s.buffs.contains(&"忆灵攻击".to_string())).expect("忆灵行动");
    // 倍率 3.0 的伤害应显著高于倍率 1.0（~3倍）
    assert!(memo.damage > 400.0, "强制技能应使用倍率3.0 dmg={:.1}", memo.damage);
}

#[test]
fn memosprite_repeat_multicast() {
    // 死龙：一次行动重复施放（repeat=4）→ 伤害 ×4
    let mut a = character("a", "A", 200.0, vec![sr_api::AbilityData {
        name: "燎尽黯泽的焰息".into(),
        kind: sr_api::AbilityKind::Memosprite,
        multiplier: 1.0,
        multipliers: vec![],
        skill_level: 1,
        scaling: Scaling::Atk,
        flat_damage: 0.0,
        dmg_type: DmgType::Normal,
        can_crit: true,
        toughness_reduction: 0.0,
        hits: 1,
        hit_split: vec![1.0],
        energy_gain: 0.0,
        max_energy: 0.0,
        skill_point: 0,
        bonus_sp: 0,
        target: Target::Single,
        buff: None,
        immediate_action: false,
        action_advance_pct: 0.0,
        self_advance_pct: 0.0,
        applies_debuff: false,
        heals: false,
        forced: true,
        repeat: 4,
            hp_cost_pct: 0.0,
            on_deplete: false,
            summons_memo: false,
    }]);
    a.has_memosprite = true;
    a.summon_at_battle_start = true;
    a.memosprite_spd = 100.0;
    let mut build = Build::default();
    build.level = 80;
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![],
        enemies: vec![enemy()],
    };
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: enemy(),
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        steps: vec![basic("a"); 4],
        memosprite_steps: vec![],
        cycles: 1,
    })
    .expect("rotation");
    let memo = out.steps.iter().find(|s| s.buffs.contains(&"忆灵攻击".to_string())).expect("忆灵行动");
    // 单发 ~230，repeat4 → ~920
    assert!(memo.damage > 800.0, "repeat4 应约4倍单发 dmg={:.1}", memo.damage);
}

#[test]
fn netherwing_hp_cost_and_explosion() {
    // 死龙：施放燎尽扣血(25%)，低血触发灼掠爆炸(×6)，随后消失
    let mut a = character("a", "A", 50.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        sr_api::AbilityData {
            name: "燎尽黯泽的焰息".into(),
            kind: sr_api::AbilityKind::Memosprite,
            multiplier: 1.0,
            multipliers: vec![],
            skill_level: 1,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 0.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 0.0,
            max_energy: 0.0,
            skill_point: 0,
            bonus_sp: 0,
            target: Target::Single,
            buff: None,
            immediate_action: false,
            action_advance_pct: 0.0,
            self_advance_pct: 0.0,
            applies_debuff: false,
            heals: false,
            forced: true,
            repeat: 3,
            hp_cost_pct: 0.25,
            on_deplete: false,
            summons_memo: false,
        },
        sr_api::AbilityData {
            name: "灼掠幽墟的晦翼".into(),
            kind: sr_api::AbilityKind::Memosprite,
            multiplier: 2.0,
            multipliers: vec![],
            skill_level: 1,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 0.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 0.0,
            max_energy: 0.0,
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
            repeat: 6,
            hp_cost_pct: 0.0,
            on_deplete: true,
            summons_memo: false,
        },
    ]);
    a.base_hp = 1000.0;
    a.has_memosprite = true;
    a.summon_at_battle_start = true;
    a.memosprite_spd = 200.0;   // 忆主行动前连动 4 次
    a.memosprite_explode_pct = 0.05;
    let mut build = Build::default();
    build.level = 80;
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![],
        enemies: vec![enemy()],
    };
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: enemy(),
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        steps: vec![basic("a"), basic("a")],
        memosprite_steps: vec![],
        cycles: 1,
    })
    .expect("rotation");
    let memo_steps: Vec<_> = out.steps.iter().filter(|s| s.buffs.contains(&"忆灵攻击".to_string())).collect();
    // 3 次燎尽 + 1 次爆炸 = 4 次忆灵行动，随后消失（无第 5 次）
    assert_eq!(memo_steps.len(), 4, "应 3 次燎尽 + 1 次爆炸 = 4，实际 {}", memo_steps.len());
    // 爆炸那次伤害显著更高（灼掠 2.0×6 ＞ 燎尽 1.0×3）
    assert!(memo_steps[3].damage > memo_steps[0].damage * 3.0,
        "爆炸伤害应高 boom={:.1} liao={:.1}", memo_steps[3].damage, memo_steps[0].damage);
    // 前 3 次为燎尽（伤害一致）
    assert!((memo_steps[0].damage - memo_steps[1].damage).abs() < 1e-6);
}

#[test]
fn memosprite_summoned_by_skill() {
    // 阿格莱雅类：忆灵不随开战在场，由战技/大招召唤
    let mut a = character("a", "A", 50.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        AbilityData {
            name: "战技·召唤".into(),
            kind: AbilityKind::Skill,
            multiplier: 0.0,
            multipliers: vec![],
            skill_level: 6,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: false,
            toughness_reduction: 0.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 30.0,
            max_energy: 100.0,
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
            summons_memo: true,
        },
        sr_api::AbilityData {
            name: "忆灵攻击".into(),
            kind: sr_api::AbilityKind::Memosprite,
            multiplier: 1.0,
            multipliers: vec![],
            skill_level: 1,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 0.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 0.0,
            max_energy: 0.0,
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
    ]);
    a.has_memosprite = true;
    a.memosprite_spd = 200.0; // 忆主行动后忆灵行动
    a.summon_at_battle_start = false;
    let mut build = Build::default();
    build.level = 80;
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![],
        enemies: vec![enemy()],
    };
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: enemy(),
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        steps: vec![basic("a"), skill("a"), basic("a"), basic("a")],
        memosprite_steps: vec![],
        cycles: 1,
    })
    .expect("rotation");
    // 第一次普攻前无忆灵（未召唤）
    assert!(!out.steps[0].buffs.contains(&"忆灵攻击".to_string()), "召唤前不应有忆灵");
    // 战技召唤后，忆灵出现并行动
    let memo_av = out.steps.iter().filter(|s| s.buffs.contains(&"忆灵攻击".to_string())).map(|s| s.av).min_by(|a,b| a.partial_cmp(b).unwrap());
    let summon_av = out.steps[1].av;
    assert!(memo_av.is_some() && memo_av.unwrap() > summon_av, "召唤后忆灵应行动");
}

#[test]
fn summon_at_battle_start_lingyuan() {
    // 景元类（神君）：战斗开始即召唤，开战即有忆灵行动
    let mut a = character("a", "A", 50.0, vec![
        ability("普攻", AbilityKind::Basic, 1.0, 1, 20.0),
        sr_api::AbilityData {
            name: "神君".into(),
            kind: sr_api::AbilityKind::Memosprite,
            multiplier: 0.5,
            multipliers: vec![],
            skill_level: 1,
            scaling: Scaling::Atk,
            flat_damage: 0.0,
            dmg_type: DmgType::Normal,
            can_crit: true,
            toughness_reduction: 0.0,
            hits: 1,
            hit_split: vec![1.0],
            energy_gain: 0.0,
            max_energy: 0.0,
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
    ]);
    a.has_memosprite = true;
    a.memosprite_spd = 100.0; // AV 100 < 忆主 200 → 忆灵先行动
    a.summon_at_battle_start = true;
    let mut build = Build::default();
    build.level = 80;
    let cfg = ConfigData {
        characters: vec![a],
        light_cones: vec![],
        relic_sets: vec![],
        enemies: vec![enemy()],
    };
    let tm = TeamMember { char_id: "a".into(), build };
    let out = sr_core::host::rotation::calculate_rotation(RotationRequest {
        config: cfg,
        team: Team { members: vec![tm] },
        enemy: enemy(),
        coefficient: Default::default(),
        battle: BattleConfig::default(),
        steps: vec![basic("a")],
        memosprite_steps: vec![],
        cycles: 1,
    })
    .expect("rotation");
    // 第一个步骤就是忆灵行动（开战已召唤）
    assert!(out.steps[0].buffs.contains(&"忆灵攻击".to_string()), "开战召唤应首步行动");
}
