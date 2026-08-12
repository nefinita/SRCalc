//! 机制集成测试：共享战技点 / 动态上限 / 定向buff / 触发 / 大招插入 / 敌方机制

use sr_api::*;
use sr_core::engine::StatMods;

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
