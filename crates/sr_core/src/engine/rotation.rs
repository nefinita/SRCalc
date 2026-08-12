//! rotation — 四角色战斗模拟器
//!
//! 依据 HSR 回合流程（Speed / Damage Wiki）：
//! - 行动值 AV = 10000/SPD，玩家角色与敌方交错行动
//! - 战技点：全队共享、上限默认 5，可被在场被动/光锥/大招临时提升
//! - 终结技：插入不占行动值、不重置施放者 AV（在下一行动前结算）
//! - buff：OnUse 施放应用 / OnSpConsume 消耗战技点触发，按回合递减
//! - 敌方行动：回我方能量 / 特殊机制回/扣战技点与能量

use sr_api::{
    action_value, AbilityKind, ActionKind, BuffTarget, Build, Character, Effect, Element,
    LightCone, RotationRequest, RotationResult, RotationStep, RotationStepReq, Trigger,
};
use std::collections::{HashMap, VecDeque};

use super::damage::{
    compute_ability_damage_for, compute_break_damage, compute_final_stats, relic_set_conditional,
    relic_set_permanent, AbilityContext, FinalStats, StatMods,
};

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
    /// 效果归属角色（tick/作用判断）；source 为去重标识
    owner: String,
    /// 当回合施放 → 本回合末不递减（"持续N回合"从下一回合算）
    skip_first_tick: bool,
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

/// 忆灵：独立行动单位（自有速度/AV，回合到自动攻击）
#[derive(Debug, Clone)]
struct MemoState {
    owner: String,
    av: f64,
    spd: f64,
    /// 忆灵行动队列（ability_index, target）；空 = 默认第 0 个忆灵技能
    queue: VecDeque<(u32, Option<String>)>,
}

#[derive(Debug, Clone)]
struct SpPool {
    current: i32,
    cap: i32,
    /// 溢出记录（花火大招等：溢出至多 10 点，消耗后补回）
    overflow: i32,
}

impl SpPool {
    /// 基础增减（普攻/战技）：钳制；消耗时优先从溢出补回
    fn add(&mut self, v: i32) {
        self.current = (self.current + v).clamp(0, self.cap);
        if v < 0 {
            let refill = self.overflow.min(self.cap - self.current);
            self.current += refill;
            self.overflow -= refill;
        }
    }
    /// 效果恢复（大招/敌方/罚恶）：溢出可记录至多 10 点
    fn add_recover(&mut self, v: i32) {
        if v > 0 {
            let room = self.cap - self.current;
            if v > room {
                self.overflow = (self.overflow + (v - room)).min(10);
                self.current = self.cap;
            } else {
                self.current += v;
            }
        } else if v < 0 {
            self.current = (self.current + v).max(0);
            let refill = self.overflow.min(self.cap - self.current);
            self.current += refill;
            self.overflow -= refill;
        }
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
    permanent: HashMap<&'a str, StatMods>,
    unit: HashMap<&'a str, UnitState>,
    on_sp_consume: Vec<(&'a str, &'a Effect)>,
    set_conditionals: HashMap<&'a str, Vec<(Trigger, Effect)>>,
    active_buffs: Vec<ActiveBuff>,
    memos: HashMap<String, MemoState>,
    sp_pool: SpPool,
    enemy_av: f64,
    enemy_idx: usize,
    enemy_toughness: f64,
    enemy_broken: bool,
    enemy_hp: f64,
    enemy_killed: bool,
    sp_consumed_turn: i32,
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
        let sets: Vec<&sr_api::RelicSet> = req.config.relic_sets.iter().collect();
        let mut builds: HashMap<&str, &Build> = HashMap::new();
        for m in &req.team.members {
            builds.insert(m.char_id.as_str(), &m.build);
        }

        let mut perm_map: HashMap<&str, StatMods> = HashMap::new();
        let mut base_stats: HashMap<&str, FinalStats> = HashMap::new();
        let mut unit: HashMap<&str, UnitState> = HashMap::new();
        let mut on_sp_consume: Vec<(&str, &Effect)> = Vec::new();
        let mut set_conditionals: HashMap<&str, Vec<(Trigger, Effect)>> = HashMap::new();
        let mut sp_pool = SpPool {
            current: req.battle.start_sp,
            cap: req.battle.base_sp_cap,
            overflow: 0,
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
                    Trigger::OnUse | Trigger::BattleStart => {
                        perm.add(&StatMods::from_effect(e, 1));
                        sp_pool.cap += e.cap_bonus;
                    }
                    Trigger::OnSpConsume => on_sp_consume.push((id, e)),
                    Trigger::OnUlt | Trigger::OnSkill | Trigger::OnBasic | Trigger::OnHit
                    | Trigger::TurnStart | Trigger::OnFollowUp | Trigger::OnAttack
                    | Trigger::OnApplyDebuff | Trigger::OnHeal | Trigger::OnKill
                    | Trigger::OnTargeted | Trigger::OnMemospriteAttack => {}
                }
            }
            perm.add(&relic_set_permanent(&m.build, &sets));
            let conds = relic_set_conditional(&m.build, &sets);
            if !conds.is_empty() {
                set_conditionals.insert(id, conds);
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

        let mut memo_queues: HashMap<String, VecDeque<(u32, Option<String>)>> = HashMap::new();
        for ms in &req.memosprite_steps {
            memo_queues
                .entry(ms.owner_id.clone())
                .or_default()
                .push_back((ms.ability_index, ms.target.clone()));
        }
        let mut memos: HashMap<String, MemoState> = HashMap::new();
        for m in &req.team.members {
            let mid = m.char_id.as_str();
            if let Some(c) = by_id.get(mid)
                && c.has_memosprite
                && c.memosprite_spd > 0.0
            {
                memos.insert(
                    mid.to_string(),
                    MemoState {
                        owner: mid.to_string(),
                        av: action_value(c.memosprite_spd),
                        spd: c.memosprite_spd,
                        queue: memo_queues.remove(mid).unwrap_or_default(),
                    },
                );
            }
        }
        Ok(Sim {
            req,
            by_id,
            builds,
            base_stats,
            permanent: perm_map,
            unit,
            on_sp_consume,
            set_conditionals,
            active_buffs: Vec::new(),
            memos,
            sp_pool,
            enemy_av: action_value(req.enemy.spd.max(1.0)),
            enemy_idx: 0,
            enemy_toughness: if req.enemy.broken { 0.0 } else { req.enemy.max_toughness },
            enemy_broken: req.enemy.broken,
            enemy_hp: req.enemy.hp,
            enemy_killed: false,
            sp_consumed_turn: 0,
            total_av: 0.0,
            total_damage: 0.0,
            steps_out: Vec::new(),
        })
    }

    fn mods_for(&self, id: &str) -> StatMods {
        let mut m = StatMods::default();
        // 常驻伤害类修正（未被 compute_final_stats 消化的字段）
        if let Some(p) = self.permanent.get(id) {
            m.def_ignore = p.def_ignore;
            m.res_pen = p.res_pen;
            m.vuln_pct = p.vuln_pct;
            m.break_effect = p.break_effect;
            m.ult_dmg_pct = p.ult_dmg_pct;
            m.skill_dmg_pct = p.skill_dmg_pct;
            m.basic_dmg_pct = p.basic_dmg_pct;
            m.followup_dmg_pct = p.followup_dmg_pct;
        }
        for b in &self.active_buffs {
            let apply = match &b.carrier {
                Carrier::Team => true,
                Carrier::Owner => b.owner == id,
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

    fn apply_buff(&mut self, source: &str, eff: &Effect, target: Option<&str>, skip: bool) {
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
        let owner = match &carrier {
            Carrier::Owner | Carrier::Team => source.to_string(),
            Carrier::Ally(a) => a.clone(),
        };
        self.active_buffs.push(ActiveBuff {
            source: source.to_string(),
            carrier,
            owner: owner.clone(),
            skip_first_tick: skip && owner == source,
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
            self.active_buffs.push(ActiveBuff {
                source: source.to_string(),
                carrier: Carrier::Team,
                owner: source.to_string(),
                skip_first_tick: true,
                mods: StatMods::from_effect(eff, 1),
                turns_remaining: eff.turns,
                stacks: 1,
                sp_on_basic: eff.sp_on_basic,
                cap_bonus: eff.cap_bonus,
                max_stacks: eff.max_stacks,
            });
        }
    }

    /// 触发式套装被动：按触发类型应用（刷新或叠层），持续 eff.turns 回合；
    /// target=ally 时挂到技能目标（如司铎4件）
    fn apply_set_conditional(
        &mut self,
        id: &str,
        trigger: Trigger,
        target: Option<&str>,
        skip: bool,
    ) {
        let Some(conds) = self.set_conditionals.get(id) else {
            return;
        };
        let conds = conds.clone();
        for (t, eff) in conds {
            if t != trigger {
                continue;
            }
            let is_ally = eff.target == BuffTarget::Ally;
            let owner = if is_ally {
                target.unwrap_or(id).to_string()
            } else {
                id.to_string()
            };
            let marker = if is_ally {
                format!("set:{id}:{:?}:{owner}", eff.stat)
            } else if eff.trigger == Trigger::OnTargeted && eff.value == 0.0 {
                format!("set:{id}:charge")
            } else {
                format!("set:{id}:{:?}", eff.stat)
            };
            if let Some(b) = self
                .active_buffs
                .iter_mut()
                .find(|b| b.source == marker)
            {
                if eff.max_stacks > 0 {
                    b.stacks = (b.stacks + 1).min(eff.max_stacks);
                    b.mods = StatMods::from_effect(&eff, b.stacks);
                }
                b.turns_remaining = if eff.turns == 0 { u32::MAX } else { eff.turns.max(1) };
            } else {
                self.active_buffs.push(ActiveBuff {
                    source: marker,
                    carrier: if is_ally {
                        Carrier::Ally(owner.clone())
                    } else {
                        Carrier::Owner
                    },
                    owner: owner.clone(),
                    skip_first_tick: skip && owner == id,
                    mods: StatMods::from_effect(&eff, 1),
                    turns_remaining: if eff.turns == 0 { u32::MAX } else { eff.turns.max(1) },
                    stacks: 1,
                    sp_on_basic: eff.sp_on_basic,
                    cap_bonus: eff.cap_bonus,
                    max_stacks: eff.max_stacks,
                });
            }
        }
    }

    fn tick_buffs(&mut self, actor: &str) {
        let mut i = 0;
        while i < self.active_buffs.len() {
            let tick = match &self.active_buffs[i].carrier {
                Carrier::Team => self.active_buffs[i].owner == actor,
                Carrier::Owner => self.active_buffs[i].owner == actor,
                Carrier::Ally(a) => a == actor,
            };
            if tick {
                if self.active_buffs[i].skip_first_tick {
                    self.active_buffs[i].skip_first_tick = false;
                } else if self.active_buffs[i].turns_remaining < u32::MAX {
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

    /// 敌方受击：扣血并检测击杀（启用 hp>0 时）
    fn apply_enemy_damage(&mut self, amount: f64) {
        if self.req.enemy.hp <= 0.0 || self.enemy_killed {
            return;
        }
        self.enemy_hp -= amount;
        if self.enemy_hp <= 0.0 {
            self.enemy_killed = true;
            let ids: Vec<&str> = self.base_stats.keys().copied().collect();
            for id in &ids {
                self.apply_set_conditional(id, Trigger::OnKill, None, true);
            }
        }
    }

    /// 削韧：仅弱点属性；破韧时返回破韧伤害并延迟敌方行动 25%
    fn apply_toughness(
        &mut self,
        element: Element,
        amount: f64,
        mods: &StatMods,
        attacker_level: u32,
    ) -> Option<f64> {
        if amount <= 0.0 {
            return None;
        }
        let weak = self.req.enemy.weaknesses.is_empty()
            || self.req.enemy.weaknesses.contains(&element);
        if !weak {
            return None;
        }
        self.enemy_toughness -= amount;
        if self.enemy_toughness <= 0.0 && !self.enemy_broken {
            self.enemy_broken = true;
            let bd = compute_break_damage(
                element,
                attacker_level,
                &self.req.enemy,
                mods,
                &self.req.coefficient,
                false, // 破韧一击按未破韧 ×0.9
            );
            self.enemy_av += action_value(self.req.enemy.spd.max(1.0)) * 0.25;
            Some(bd)
        } else {
            None
        }
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

        // 回合开始触发式套装被动；重置本回合消耗战技点计数
        self.sp_consumed_turn = 0;
        self.apply_set_conditional(id, Trigger::TurnStart, None, true);

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
                    broken: self.enemy_broken,
                };
                damage = compute_ability_damage_for(ctx).expected;
                if mods.atk_pct > 0.0 || mods.dmg_pct > 0.0 || mods.crit_rate > 0.0 {
                    labels.push("增益覆盖".to_string());
                }
                // 削韧（弱点属性）→ 破韧 + 破韧伤害 + 敌方行动延迟
                if let Some(bd) = self.apply_toughness(element, ability.toughness_reduction, &mods, build.level.max(1)) {
                    damage += bd;
                    labels.push("破韧".to_string());
                }
            }

            // 战技点：按技能消耗/恢复
            self.sp_pool.add(ability.skill_point);
            if ability.skill_point < 0 {
                self.sp_consumed_turn += -ability.skill_point;
                for (src, eff) in self.on_sp_consume.clone() {
                    self.stack_team_buff(src, eff);
                }
                if self.sp_consumed_turn >= 3 {
                    self.apply_set_conditional(id, Trigger::OnSpConsume, None, true);
                }
            }
            self.sp_pool.add_recover(ability.bonus_sp);

            // 目标普攻时额外战技点（寒鸦"罚恶"）→ 溢出记录
            if step.action == ActionKind::Basic {
                let sp_on = self
                    .active_buffs
                    .iter()
                    .filter(|b| b.sp_on_basic > 0 && b.carrier.applies_to(id) && b.source != id)
                    .count();
                for _ in 0..sp_on {
                    self.sp_pool.add_recover(1);
                }
            }

            // 施放时应用 buff
            if let Some(eff) = &ability.buff {
                self.apply_buff(id, eff, step.target.as_deref(), true);
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

            // 能量（×ERR 能量回复效率）
            if let Some(s) = self.unit.get_mut(id) {
                let err = self.base_stats[id].energy_regen;
                s.energy = (s.energy + ability.energy_gain * (1.0 + err)).min(s.max_energy);
            }
            // 施加负面 / 治疗 → 触发套装被动
            if ability.applies_debuff {
                self.apply_set_conditional(id, Trigger::OnApplyDebuff, None, true);
            }
            if ability.heals {
                self.apply_set_conditional(id, Trigger::OnHeal, None, true);
            }
        }

        // 套装触发式被动（按动作类型；定向型传目标）
        match step.action {
            ActionKind::Ult => {
                self.apply_set_conditional(id, Trigger::OnUlt, step.target.as_deref(), true);
                self.apply_set_conditional(id, Trigger::OnAttack, None, true);
            }
            ActionKind::Skill => {
                self.apply_set_conditional(id, Trigger::OnSkill, step.target.as_deref(), true);
                self.apply_set_conditional(id, Trigger::OnAttack, None, true);
            }
            ActionKind::Basic => {
                self.apply_set_conditional(id, Trigger::OnBasic, None, true);
                self.apply_set_conditional(id, Trigger::OnAttack, None, true);
            }
            ActionKind::Wait => {}
        }
        // 成为其他我方目标技能目标（船长"助力"）
        if let Some(t) = step.target.as_deref()
            && t != id
        {
            self.apply_set_conditional(t, Trigger::OnTargeted, None, true);
        }


        if damage > 0.0 {
            self.apply_enemy_damage(damage);
            if self.enemy_killed {
                labels.push("击杀".to_string());
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
                broken: self.enemy_broken,
            };
            damage = compute_ability_damage_for(ctx).expected;
            if let Some(bd) = self.apply_toughness(element, ability.toughness_reduction, &mods, build.level.max(1)) {
                damage += bd;
            }
            self.sp_pool.add(ability.skill_point);
            self.sp_pool.add_recover(ability.bonus_sp);
            if let Some(eff) = &ability.buff {
                self.apply_buff(id, eff, step.target.as_deref(), true);
            }
        }
        // 船长"助力"门控：仅当该角色装备了 OnTargeted 助力机制时才消耗触发；
        // 否则按普通 OnUlt 套装触发（密林卧雪等）
        let has_charge_mech = self
            .set_conditionals
            .get(id)
            .map(|c| c.iter().any(|(t, _)| *t == Trigger::OnTargeted))
            .unwrap_or(false);
        if has_charge_mech {
            let charge_marker = format!("set:{id}:charge");
            let charge_ready = self
                .active_buffs
                .iter()
                .any(|b| b.source == charge_marker && b.stacks >= b.max_stacks.max(1));
            if charge_ready {
                self.apply_set_conditional(id, Trigger::OnUlt, step.target.as_deref(), false);
                self.active_buffs.retain(|b| b.source != charge_marker);
            }
        } else {
            self.apply_set_conditional(id, Trigger::OnUlt, step.target.as_deref(), false);
        }
        if let Some(s) = self.unit.get_mut(id) {
            s.energy = 0.0;
        }
        if damage > 0.0 {
            self.apply_enemy_damage(damage);
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

    /// 忆灵行动：触发 OnMemospriteAttack + 使用选中的忆灵技能攻击（继承忆主面板）
    fn resolve_memo(&mut self, owner: &str, ability_index: u32, target: Option<&str>) {
        self.apply_set_conditional(owner, Trigger::OnMemospriteAttack, None, false);
        let Some(stats) = self.base_stats.get(owner).cloned() else {
            return;
        };
        let element = self.by_id.get(owner).map(|c| c.element).unwrap_or_default();
        let attacker_level = self
            .builds
            .get(owner)
            .map(|b| b.level.max(1))
            .unwrap_or(80);
        let mods = self.mods_for(owner);
        // 选中的忆灵技能（kind=memosprite 下标）；缺省用 memosprite_multiplier 合成
        let memo_abilities: Vec<sr_api::AbilityData> = self
            .by_id
            .get(owner)
            .map(|c| {
                c.abilities
                    .iter()
                    .filter(|a| a.kind == sr_api::AbilityKind::Memosprite)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        // 强制触发技能（死龙/长夜月）：回合到必放，忽略选择；否则用序列/默认
        let forced = memo_abilities.iter().find(|a| a.forced).cloned();
        let ability = forced
            .or_else(|| memo_abilities.get(ability_index as usize).cloned())
            .unwrap_or_else(|| sr_api::AbilityData {
                name: "忆灵攻击".into(),
                kind: sr_api::AbilityKind::Memosprite,
                multiplier: self.by_id.get(owner).map(|c| c.memosprite_multiplier).unwrap_or(0.0),
                can_crit: true,
                ..Default::default()
            });
        if let Some(eff) = &ability.buff {
            self.apply_buff(owner, eff, target, false);
        }
        let ctx = AbilityContext {
            stats: &stats,
            ability: &ability,
            element,
            attacker_level,
            enemy: &self.req.enemy,
            mods: &mods,
            coeff: &self.req.coefficient,
            broken: self.enemy_broken,
        };
        let dmg = compute_ability_damage_for(ctx).expected;
        if dmg > 0.0 {
            self.apply_enemy_damage(dmg);
        }
        let owner_name = self.by_id.get(owner).map(|c| c.name.clone()).unwrap_or_default();
        self.steps_out.push(RotationStep {
            char_id: owner.to_string(),
            char_name: format!("{owner_name}·{}", ability.name),
            action: sr_api::ActionKind::Wait,
            is_enemy: false,
            enemy_ability: None,
            av: self.total_av,
            damage: dmg,
            energy: self.unit.get(owner).map(|s| s.energy).unwrap_or(0.0),
            skill_point: self.sp_pool.current,
            buffs: vec!["忆灵攻击".to_string()],
        });
        self.total_damage += dmg;
    }

    fn resolve_enemy(&mut self) {
        let act = self.req.enemy.actions.get(self.enemy_idx);
        let (name, gain, sp_delta, drain) = match act {
            Some(a) => (a.name.clone(), a.energy_gain_players, a.sp_delta, a.energy_drain),
            None => ("普通攻击".to_string(), 0.0, 0, 0.0),
        };
        let hit_ids: Vec<&str> = self.base_stats.keys().copied().collect();
        for id in &hit_ids {
            self.apply_set_conditional(id, Trigger::OnHit, None, true);
        }
        for (id, base) in &self.base_stats {
            if let Some(s) = self.unit.get_mut(id) {
                s.energy = (s.energy + gain * (1.0 + base.energy_regen)).min(s.max_energy);
            }
        }
        self.sp_pool.add_recover(sp_delta);
        // 敌方回合：破韧恢复（韧性回满、解除击破）
        if self.enemy_broken {
            self.enemy_broken = false;
            self.enemy_toughness = self.req.enemy.max_toughness;
        }
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

        // 敌方/忆灵在施放者行动前交错插入（取最早事件）
        loop {
            let actor_av = sim.unit.get(id).map(|s| s.av).unwrap_or_default();
            let mut min_av = sim.enemy_av;
            let mut kind = 0usize;
            let mut memo_id: Option<String> = None;
            for (mid, memo) in &sim.memos {
                if memo.av < min_av {
                    min_av = memo.av;
                    kind = 1;
                    memo_id = Some(mid.clone());
                }
            }
            if min_av >= actor_av {
                break;
            }
            sim.total_av += min_av;
            for u in sim.unit.values_mut() {
                u.av = (u.av - min_av).max(0.0);
            }
            if kind == 0 {
                sim.resolve_enemy();
                sim.enemy_av = action_value(sim.req.enemy.spd.max(1.0));
            } else if let Some(mid) = memo_id {
                let (owner, next_action) = {
                    let m = sim.memos.get_mut(&mid).expect("memo");
                    (m.owner.clone(), m.queue.pop_front())
                };
                let (idx, tgt) = next_action.unwrap_or((0, None));
                sim.resolve_memo(&owner, idx, tgt.as_deref());
                if let Some(m) = sim.memos.get_mut(&mid) {
                    m.av = action_value(m.spd.max(1.0));
                }
            }
        }

        // 推进到该行动回合点
        let dt = sim.unit.get(id).map(|s| s.av).unwrap_or_default();
        sim.total_av += dt;
        for u in sim.unit.values_mut() {
            u.av = (u.av - dt).max(0.0);
        }
        sim.enemy_av = (sim.enemy_av - dt).max(0.0);
        for m in sim.memos.values_mut() {
            m.av = (m.av - dt).max(0.0);
        }

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
