//! calc — 伤害计算契约方法

use sr_api::{ConfigData, DamageRequest, LightCone, SkillResult};

use crate::engine::damage::{
    compute_ability_damage_for, compute_break_damage, compute_final_stats, presence_mods,
    relic_set_mods, AbilityContext, StatMods,
};

pub fn find_cone<'a>(config: &'a ConfigData, id: Option<&str>) -> Option<&'a LightCone> {
    id.and_then(|id| config.light_cones.iter().find(|c| c.id == id))
}

fn allies<'a>(config: &'a ConfigData, team: &'a sr_api::Team) -> Vec<&'a sr_api::Character> {
    team.members
        .iter()
        .filter_map(|m| config.characters.iter().find(|c| c.id == m.char_id))
        .collect()
}

pub fn calculate_damage(req: DamageRequest) -> Result<Vec<SkillResult>, String> {
    let Some(character) = req.config.characters.iter().find(|c| c.id == req.focus) else {
        return Err(format!("未找到角色: {}", req.focus));
    };
    let Some(member) = req.team.members.iter().find(|m| m.char_id == req.focus) else {
        return Err(format!("角色 {} 不在队伍中", req.focus));
    };

    let allies = allies(&req.config, &req.team);
    let cone = find_cone(&req.config, member.build.light_cone.as_deref());
    let sets: Vec<&sr_api::RelicSet> = req.config.relic_sets.iter().collect();
    let mut permanent = presence_mods(character, cone, &allies);
    permanent.add(&relic_set_mods(&member.build, &sets));
    let stats = compute_final_stats(character, cone, &member.build, &permanent);
    let mods = StatMods::from_buff(&req.buff);
    let attacker_level = member.build.level.max(1);

    let mut out = Vec::new();
    for ability in &character.abilities {
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
        out.push(SkillResult {
            char_name: character.name.clone(),
            ability: ability.name.clone(),
            base: result.base,
            crit_rate: result.crit_rate,
            crit_dmg: result.crit_dmg,
            non_crit: result.non_crit,
            crit: result.crit,
            expected: result.expected,
        });
    }

    // 击破伤害（可作为独立条目）
    let break_dmg = compute_break_damage(
        character.element,
        attacker_level,
        &req.enemy,
        &mods,
        &req.coefficient,
        req.enemy.broken,
    );
    out.push(SkillResult {
        char_name: character.name.clone(),
        ability: "击破伤害".into(),
        base: break_dmg,
        crit_rate: 0.0,
        crit_dmg: 0.0,
        non_crit: break_dmg,
        crit: break_dmg,
        expected: break_dmg,
    });

    Ok(out)
}
