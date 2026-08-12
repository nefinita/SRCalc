//! rotation — 四角色战斗模拟器
//!
//! 依据 HSR 回合流程（Speed / Damage Wiki）：
//! - 行动值 AV = 10000/SPD，玩家角色与敌方交错行动
//! - 战技点：全队共享、上限默认 5，可被在场被动/光锥/大招临时提升
//! - 终结技：插入不占行动值、不重置施放者 AV（在下一行动前结算）
//! - buff：OnUse 施放应用 / OnSpConsume 消耗战技点触发，按回合递减
//! - 敌方行动：回我方能量 / 特殊机制回/扣战技点与能量

use sr_api::{
    action_value, AbilityKind, ActionKind, BuffTarget, Build, Character, Effect, LightCone,
    RotationRequest, RotationResult, RotationStep, RotationStepReq, Trigger,
};
use std::collections::HashMap;

use super::damage::{compute_ability_damage_for, compute_final_stats, AbilityContext, FinalStats, StatMods};

#[derive(Debug, Clone, PartialEq)]
enum Carrier {
    Team,
    Owner,
    Ally(String),
}

impl Carrier {
    fn applies_to(&self, id: &str) -> bool {
        match self {
            Carrier::Team => true,
            Carrier::Owner => true, // 由调用方按 source==id 再判断
            Carrier::Ally(a) => a == id,
        }
    }
}

#[derive(Debug, Clone)]
struct ActiveBuff {
    source: String,
    carrier: Carrier,
    mods: StatMods,
    turns_remaining: u32,
    stacks: u32,
    sp_on_basic: i32,
    cap_bonus: i32,
    max_stacks: u32,
}

#[derive(Debug, Clone)]
struct UnitState {
    av: f64,
    energy: f64,
    max_energy: f64,
}

#[derive(Debug, Clone)]
struct SpPool {
    current: i32,
    cap: i32,
}

impl SpPool {
    fn add(&mut self, v: i32) {
        self.current = (self.current + v).clamp(0, self.cap);
    }
    fn clamp(&mut self) {
        self.current = self.current.clamp(0, self.cap);
    }
}

struct Sim<'a> {
    req: &'a RotationRequest,
    by_id: HashMap<&'a str, &'a Character>,
    builds: HashMap<&'a str, &'a Build>,
    base_stats: HashMap<&'a str, FinalStats>,
    unit: HashMap<&'a str, UnitState>,
    on_sp_consume: Vec<(&'a str, &'a Effect)>,
    active_buffs: Vec<ActiveBuff>,
    sp_pool: SpPool,
    enemy_av: f64,
    enemy_idx: usize,
    total_av: f64,
    total_damage: f64,
    steps_out: Vec<RotationStep>,
}

impl<'a> Sim<'a> {
    fn new(req: &'a RotationRequest) -> Result<Self, String> {
        let mut by_id: HashMap<&str, &Character> = HashMap::new();
        for c in &req.config.characters {
            by_id.insert(c.id.as_str(), c);
        }
        let cones: HashMap<&str, &LightCone> = req
            .config
            .light_cones
            .iter()
            .map(|c| (c.id.as_str(), c))
            .collect();
        let mut builds: HashMap<&str, &Build> = HashMap::new();
        for m in &req.team.members {
            builds.insert(m.char_id.as_str(), &m.build);
        }

        let mut perm_map: HashMap<&str, StatMods> = HashMap::new();
        let mut base_stats: HashMap<&str, FinalStats> = HashMap::new();
        let mut unit: HashMap<&str, UnitState> = HashMap::new();
        let mut on_sp_consume: Vec<(&str, &Effect)> = Vec::new();
        let mut sp_pool = SpPool {
            current: req.battle.start_sp,
            cap: req.battle.base_sp_cap,
        };

        for m in &req.team.members {
            let id = m.char_id.as_str();
            let Some(character) = by_id.get(id) else {
                return Err(format!("未找到角色: {}", m.char_id));
            };
            let cone = m.build.light_cone.as_deref().and_then(|cid| cones.get(cid)).copied();
            let mut perm = StatMods::default();
            if let Some(c) = cone {
                for e in &c.effects {
                    perm.add(&StatMods::from_effect(e, 1));
                    sp_pool.cap += e.cap_bonus;
                }
            }
            for e in &character.team_effects {
                match e.trigger {
                    Trigger::OnUse => {
                        perm.add(&StatMods::from_effect(e, 1));
                        sp_pool.cap += e.cap_bonus;
                    }
                    Trigger::OnSpConsume => on_sp_consume.push((id, e)),
                }
            }
            perm_map.insert(id, perm.clone());
            let stats = compute_final_stats(character, cone, &m.build, &perm);
            let max_energy = character
                .abilities
                .iter()
                .map(|a| a.max_energy)
                .fold(0.0_f64, f64::max);
            unit.insert(
                id,
                UnitState {
                    av: action_value(stats.spd),
                    energy: req.battle.start_energy,
                    max_energy,
                },
            );
            base_stats.insert(id, stats);
        }

        Ok(Sim {
            req,
            by_id,
            builds,
            base_stats,
            unit,
            on_sp_consume,
            active_buffs: Vec::new(),
            sp_pool,
            enemy_av: action_value(req.enemy.spd.max(1.0)),
            enemy_idx: 0,
            total_av: 0.0,
            total_damage: 0.0,
            steps_out: Vec::new(),
        })
    }

    fn mods_for(&self, id: &str) -> StatMods {
        let mut m = StatMods::default();
        for b in &self.active_buffs {
            let apply = match &b.carrier {
                Carrier::Team => true,
                Carrier::Owner => b.source == id,
                Carrier::Ally(a) => a == id,
            };
            if apply {
                m.add(&b.mods);
            }
        }
        m
    }

    fn spd_for(&self, id: &str) -> f64 {
        let base = self.base_stats.get(id).map(|s| s.spd).unwrap_or(100.0);
        base * (1.0 + self.mods_for(id).spd_pct)
    }

    fn apply_buff(&mut self, source: &str, eff: &Effect, target: Option<&str>) {
        let carrier = match eff.target {
            BuffTarget::Self_ => Carrier::Owner,
            BuffTarget::Team => Carrier::Team,
            BuffTarget::Ally => Carrier::Ally(target.unwrap_or(source).to_string()),
        };
        let cap = eff.cap_bonus;
        if cap != 0 {
            self.sp_pool.cap += cap;
            self.sp_pool.clamp();
        }
        self.active_buffs.push(ActiveBuff {
            source: source.to_string(),
            carrier,
            mods: StatMods::from_effect(eff, 1),
            turns_remaining: eff.turns,
            stacks: 1,
            sp_on_basic: eff.sp_on_basic,
            cap_bonus: cap,
            max_stacks: eff.max_stacks,
        });
    }

    fn stack_team_buff(&mut self, source: &str, eff: &Effect) {
        if let Some(b) = self
            .active_buffs
            .iter_mut()
            .find(|b| b.source == source && b.carrier == Carrier::Team)
        {
            b.stacks = (b.stacks + 1).min(b.max_stacks.max(1));
            b.mods = StatMods::from_effect(eff, b.stacks);
            b.turns_remaining = eff.turns;
        } else {
            self.apply_buff(source, eff, None);
        }
    }

    fn tick_buffs(&mut self, actor: &str) {
        let mut i = 0;
        while i < self.active_buffs.len() {
            let tick = match &self.active_buffs[i].carrier {
                Carrier::Team => self.active_buffs[i].source == actor,
                Carrier::Owner => self.active_buffs[i].source == actor,
                Carrier::Ally(a) => a == actor,
            };
            if tick && self.active_buffs[i].turns_remaining > 0 {
                self.active_buffs[i].turns_remaining -= 1;
                if self.active_buffs[i].turns_remaining == 0 {
                    let cap = self.active_buffs[i].cap_bonus;
                    if cap != 0 {
                        self.sp_pool.cap -= cap;
                        self.sp_pool.clamp();
                    }
                    self.active_buffs.remove(i);
                    continue;
                }
            }
            i += 1;
        }
    }

    fn ability_of(char: &Character, action: ActionKind) -> Option<&sr_api::AbilityData> {
        let kind = match action {
            ActionKind::Basic => AbilityKind::Basic,
            ActionKind::Skill => AbilityKind::Skill,
            ActionKind::Ult => AbilityKind::Ult,
            ActionKind::Wait => AbilityKind::Talent,
        };
        char.abilities.iter().find(|a| a.kind == kind)
    }

    fn resolve_action(&mut self, step: &RotationStepReq) -> Result<(), String> {
        let id = step.char_id.as_str();
        let char_name = self
            .by_id
            .get(id)
            .map(|c| c.name.clone())
            .ok_or_else(|| format!("未找到角色: {}", step.char_id))?;
        let element = self.by_id.get(id).map(|c| c.element).unwrap_or_default();
        let default_build = Build::default();
        let build = self.builds.get(id).copied().unwrap_or(&default_build);
        let ability = self
            .by_id
            .get(id)
            .and_then(|c| Self::ability_of(c, step.action))
            .cloned();

        let mut damage = 0.0_f64;
        let mut labels = Vec::new();

        if let Some(ability) = &ability {
            if step.action != ActionKind::Wait {
                let mods = self.mods_for(id);
                let ctx = AbilityContext {
                    stats: &self.base_stats[id],
                    ability,
                    element,
                    attacker_level: build.level.max(1),
                    enemy: &self.req.enemy,
                    mods: &mods,
                    coeff: &self.req.coefficient,
                };
                damage = compute_ability_damage_for(ctx).expected;
                if mods.atk_pct > 0.0 || mods.dmg_pct > 0.0 || mods.crit_rate > 0.0 {
                    labels.push("增益覆盖".to_string());
                }
            }

            // 战技点：按技能消耗/恢复
            self.sp_pool.add(ability.skill_point);
            if ability.skill_point < 0 {
                for (src, eff) in self.on_sp_consume.clone() {
                    self.stack_team_buff(src, eff);
                }
            }
            self.sp_pool.add(ability.bonus_sp);

            // 目标普攻时额外战技点（寒鸦"罚恶"）
            if step.action == ActionKind::Basic {
                let sp_on = self
                    .active_buffs
                    .iter()
                    .filter(|b| b.sp_on_basic > 0 && b.carrier.applies_to(id) && b.source != id)
                    .count();
                for _ in 0..sp_on {
                    self.sp_pool.add(1);
                }
            }

            // 施放时应用 buff
            if let Some(eff) = &ability.buff {
                self.apply_buff(id, eff, step.target.as_deref());
            }

            // 行动提前 / 立即行动（目标）
            if let Some(t) = step.target.as_deref() {
                if ability.immediate_action
                    && let Some(s) = self.unit.get_mut(t)
                {
                    s.av = 0.0;
                }
                if ability.action_advance_pct > 0.0 {
                    let spd_t = self.spd_for(t);
                    if let Some(s) = self.unit.get_mut(t) {
                        s.av =
                            (s.av - action_value(spd_t) * ability.action_advance_pct).max(0.0);
                    }
                }
            }

            // 能量
            if let Some(s) = self.unit.get_mut(id) {
                s.energy = (s.energy + ability.energy_gain).min(s.max_energy);
            }
        }

        self.steps_out.push(RotationStep {
            char_id: step.char_id.clone(),
            char_name,
            action: step.action,
            is_enemy: false,
            enemy_ability: None,
            av: self.total_av,
            damage,
            energy: self.unit.get(id).map(|s| s.energy).unwrap_or(0.0),
            skill_point: self.sp_pool.current,
            buffs: labels,
        });
        self.total_damage += damage;
        Ok(())
    }

    fn resolve_ult(&mut self, step: &RotationStepReq) -> Result<(), String> {
        let id = step.char_id.as_str();
        let char_name = self
            .by_id
            .get(id)
            .map(|c| c.name.clone())
            .ok_or_else(|| format!("未找到角色: {}", step.char_id))?;
        let element = self.by_id.get(id).map(|c| c.element).unwrap_or_default();
        let energy = self.unit.get(id).map(|s| s.energy).unwrap_or(0.0);
        let max_energy = self.unit.get(id).map(|s| s.max_energy).unwrap_or(0.0);
        if energy + 1e-6 < max_energy {
            return Err(format!(
                "{} 能量不足，无法施放终结技（{:.0}/{:.0}）",
                char_name, energy, max_energy
            ));
        }
        let default_build = Build::default();
        let build = self.builds.get(id).copied().unwrap_or(&default_build);
        let ability = self
            .by_id
            .get(id)
            .and_then(|c| Self::ability_of(c, ActionKind::Ult))
            .cloned();

        let mut damage = 0.0_f64;
        if let Some(ability) = &ability {
            let mods = self.mods_for(id);
            let ctx = AbilityContext {
                stats: &self.base_stats[id],
                ability,
                element,
                attacker_level: build.level.max(1),
                enemy: &self.req.enemy,
                mods: &mods,
                coeff: &self.req.coefficient,
            };
            damage = compute_ability_damage_for(ctx).expected;
            self.sp_pool.add(ability.skill_point);
            self.sp_pool.add(ability.bonus_sp);
            if let Some(eff) = &ability.buff {
                self.apply_buff(id, eff, step.target.as_deref());
            }
        }
        if let Some(s) = self.unit.get_mut(id) {
            s.energy = 0.0;
        }
        self.steps_out.push(RotationStep {
            char_id: step.char_id.clone(),
            char_name,
            action: ActionKind::Ult,
            is_enemy: false,
            enemy_ability: None,
            av: self.total_av,
            damage,
            energy: 0.0,
            skill_point: self.sp_pool.current,
            buffs: Vec::new(),
        });
        self.total_damage += damage;
        Ok(())
    }

    fn resolve_enemy(&mut self) {
        let act = self.req.enemy.actions.get(self.enemy_idx);
        let (name, gain, sp_delta, drain) = match act {
            Some(a) => (a.name.clone(), a.energy_gain_players, a.sp_delta, a.energy_drain),
            None => ("普通攻击".to_string(), 0.0, 0, 0.0),
        };
        for s in self.unit.values_mut() {
            s.energy = (s.energy + gain).min(s.max_energy);
        }
        self.sp_pool.add(sp_delta);
        for s in self.unit.values_mut() {
            s.energy = (s.energy - drain).max(0.0);
        }
        if !self.req.enemy.actions.is_empty() {
            self.enemy_idx = (self.enemy_idx + 1) % self.req.enemy.actions.len();
        }
        self.steps_out.push(RotationStep {
            char_id: String::new(),
            char_name: self.req.enemy.name.clone(),
            action: ActionKind::Wait,
            is_enemy: true,
            enemy_ability: Some(name),
            av: self.total_av,
            damage: 0.0,
            energy: 0.0,
            skill_point: self.sp_pool.current,
            buffs: Vec::new(),
        });
    }
}

pub fn simulate(req: &RotationRequest) -> Result<RotationResult, String> {
    let mut sim = Sim::new(req)?;
    let mut cycle = 0;
    let mut pending_ults: Vec<RotationStepReq> = Vec::new();
    let mut remaining: Vec<RotationStepReq> = req.steps.clone();

    while !remaining.is_empty() {
        let step = remaining.remove(0);
        if step.action == ActionKind::Ult {
            // 终结技：不占行动值，挂起到下一行动前结算
            pending_ults.push(step);
            continue;
        }

        let id = step.char_id.as_str();
        if !sim.unit.contains_key(id) {
            return Err(format!("角色 {} 不在队伍中", step.char_id));
        }

        // 敌方在施放者行动前交错插入
        loop {
            let actor_av = sim.unit.get(id).map(|s| s.av).unwrap_or_default();
            if sim.enemy_av >= actor_av {
                break;
            }
            let eav = sim.enemy_av;
            sim.total_av += eav;
            for u in sim.unit.values_mut() {
                u.av = (u.av - eav).max(0.0);
            }
            sim.resolve_enemy();
            sim.enemy_av = action_value(sim.req.enemy.spd.max(1.0));
        }

        // 推进到该行动回合点
        let dt = sim.unit.get(id).map(|s| s.av).unwrap_or_default();
        sim.total_av += dt;
        for u in sim.unit.values_mut() {
            u.av = (u.av - dt).max(0.0);
        }
        sim.enemy_av = (sim.enemy_av - dt).max(0.0);

        // 待放终结技在"下一行动前"结算
        for u in pending_ults.drain(..) {
            sim.resolve_ult(&u)?;
        }

        sim.resolve_action(&step)?;

        // 重置施放者行动值（含自身行动提前）
        let spd = sim.spd_for(id);
        let self_adv = sim
            .by_id
            .get(id)
            .and_then(|c| Sim::ability_of(c, step.action))
            .map(|a| a.self_advance_pct)
            .unwrap_or(0.0);
        if let Some(s) = sim.unit.get_mut(id) {
            s.av = action_value(spd);
            if self_adv > 0.0 {
                s.av = (s.av - action_value(spd) * self_adv).max(0.0);
            }
        }

        sim.tick_buffs(id);

        if remaining.is_empty() && cycle + 1 < req.cycles {
            cycle += 1;
            remaining = req.steps.clone();
        }
    }

    // 末尾孤立终结技
    for u in pending_ults {
        sim.resolve_ult(&u)?;
    }

    Ok(RotationResult {
        steps: sim.steps_out,
        total_damage: sim.total_damage,
        total_av: sim.total_av,
    })
}
