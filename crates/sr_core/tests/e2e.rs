//! 端到端集成测试：内置数据 → 队伍 → 伤害 → 排轴 → 配装

use sr_api::{
    ActionKind, BattleConfig, Build, BuffConfig, ConfigData, DamageRequest,
    OptimizeRequest, RotationRequest, RotationStepReq, Team, TeamMember,
};

fn full_config() -> ConfigData {
    sr_core::host::config::load_config()
}

fn seele_team() -> Team {
    Team {
        members: vec![TeamMember {
            char_id: "1101".into(),
            build: Build::default(),
        }],
    }
}

#[test]
fn load_config_has_seele() {
    let cfg = full_config();
    assert!(!cfg.characters.is_empty());
    assert!(cfg.characters.iter().any(|c| c.id == "1101"));
}

#[test]
fn calculate_damage_pipeline() {
    let cfg = full_config();
    let enemy = cfg.enemies.first().cloned().unwrap();
    let req = DamageRequest {
        config: cfg,
        team: seele_team(),
        focus: "1101".into(),
        enemy,
        buff: BuffConfig::default(),
        coefficient: Default::default(),
    };
    let out = sr_core::host::calc::calculate_damage(req).expect("calculate_damage");
    assert!(!out.is_empty());
    assert!(out.iter().all(|r| r.expected >= 0.0));
}

#[test]
fn rotation_pipeline() {
    let cfg = full_config();
    let enemy = cfg.enemies.first().cloned().unwrap();
    let req = RotationRequest {
        config: cfg,
        team: seele_team(),
        enemy,
        coefficient: Default::default(),
        battle: BattleConfig {
            start_energy: 120.0,
            ..Default::default()
        },
        memosprite_steps: vec![],
        steps: vec![
            RotationStepReq {
                char_id: "1101".into(),
                action: ActionKind::Basic,
                target: None,
            },
            RotationStepReq {
                char_id: "1101".into(),
                action: ActionKind::Skill,
                target: None,
            },
            RotationStepReq {
                char_id: "1101".into(),
                action: ActionKind::Ult,
                target: None,
            },
        ],
        natural_until_av: 0.0,
        cycles: 1,
    };
    let out = sr_core::host::rotation::calculate_rotation(req).expect("rotation");
    let player: Vec<_> = out.steps.iter().filter(|s| !s.is_enemy).collect();
    assert_eq!(player.len(), 3);
    assert!(out.total_av > 0.0);
    for pair in player.windows(2) {
        assert!(pair[1].av >= pair[0].av);
    }
}

#[test]
fn optimize_pipeline() {
    let cfg = full_config();
    let enemy = cfg.enemies.first().cloned().unwrap();
    let req = OptimizeRequest {
        config: cfg,
        team: seele_team(),
        focus: "1101".into(),
        enemy,
        coefficient: Default::default(),
    };
    let out = sr_core::host::optimize::run_optimize(req).expect("optimize");
    assert_eq!(out.best.len(), 8);
    for pair in out.best.windows(2) {
        assert!(pair[0].expected >= pair[1].expected);
    }
}
