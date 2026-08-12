//! optimize — 配装优化器
//!
//! 枚举 身体×脚部×位面球×连接绳 四部位主词条组合（标准 5★ Lv15 数值），
//! 以角色战技（缺省时终结技/普攻）的期望伤害为目标排序，输出 Top N。

use sr_api::{
    AbilityKind, Build, BuildOption, MainStat, OptimizeRequest, OptimizeResult, RelicSlot,
};

use super::damage::{
    compute_ability_damage_for, compute_final_stats, main_stat_options, presence_mods,
    AbilityContext, StatMods,
};

pub fn run(req: &OptimizeRequest) -> Result<OptimizeResult, String> {
    let Some(character) = req.config.characters.iter().find(|c| c.id == req.focus) else {
        return Err(format!("未找到角色: {}", req.focus));
    };
    let Some(member) = req.team.members.iter().find(|m| m.char_id == req.focus) else {
        return Err(format!("角色 {} 不在队伍中", req.focus));
    };

    let allies: Vec<&sr_api::Character> = req
        .team
        .members
        .iter()
        .filter_map(|m| req.config.characters.iter().find(|c| c.id == m.char_id))
        .collect();
    let cone = req
        .config
        .light_cones
        .iter()
        .find(|c| Some(c.id.as_str()) == member.build.light_cone.as_deref());
    let permanent = presence_mods(character, cone, &allies);

    // 目标技能：优先战技 → 终结技 → 普攻
    let ability = character
        .abilities
        .iter()
        .find(|a| a.kind == AbilityKind::Skill)
        .or_else(|| character.abilities.iter().find(|a| a.kind == AbilityKind::Ult))
        .or_else(|| character.abilities.iter().find(|a| a.kind == AbilityKind::Basic))
        .ok_or_else(|| "角色没有可用技能".to_string())?;

    let opts = main_stat_options(character.element);
    let attacker_level = member.build.level.max(1);
    let mut best: Vec<(f64, BuildOption)> = Vec::new();

    for (body_label, body_key, body_val) in &opts.body {
        for (feet_label, feet_key, feet_val) in &opts.feet {
            for (sphere_label, sphere_key, sphere_val) in &opts.sphere {
                for (rope_label, rope_key, rope_val) in &opts.rope {
                    let mut build = member.build.clone();
                    build.main_stats.retain(|m| m.slot != RelicSlot::Body);
                    build.main_stats.push(MainStat {
                        slot: RelicSlot::Body,
                        stat: body_key.clone(),
                        value: *body_val,
                    });
                    push_main(&mut build, RelicSlot::Feet, feet_key, *feet_val);
                    push_main(&mut build, RelicSlot::Sphere, sphere_key, *sphere_val);
                    push_main(&mut build, RelicSlot::Rope, rope_key, *rope_val);

                    let stats = compute_final_stats(character, cone, &build, &permanent);
                    let mods = StatMods::default();
                    let result = compute_ability_damage_for(AbilityContext {
                        stats: &stats,
                        ability,
                        element: character.element,
                        attacker_level,
                        enemy: &req.enemy,
                        mods: &mods,
                        coeff: &req.coefficient,
                        broken: req.enemy.broken,
                    });

                    best.push((
                        result.expected,
                        BuildOption {
                            body: body_label.to_string(),
                            feet: feet_label.to_string(),
                            sphere: sphere_label.to_string(),
                            rope: rope_label.to_string(),
                            expected: result.expected,
                        },
                    ));
                }
            }
        }
    }

    best.sort_by(|a, b| b.0.total_cmp(&a.0));
    let best: Vec<BuildOption> = best.into_iter().take(8).map(|(_, o)| o).collect();

    Ok(OptimizeResult { best })
}

fn push_main(build: &mut Build, slot: RelicSlot, key: &str, value: f64) {
    build.main_stats.retain(|m| m.slot != slot);
    build.main_stats.push(MainStat {
        slot,
        stat: key.to_string(),
        value,
    });
}
