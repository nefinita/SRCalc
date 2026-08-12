//! sr_api — 星铁计算器纯 DTO 契约 crate（serde 单一来源，零业务依赖）
//!
//! 所有 IPC / FFI / 持久化共享的数据类型都在这里定义，字段为 snake_case，
//! 与前端 `src/types/index.ts` 一一对应（JSON 直接对齐）。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    #[default]
    Physical,
    Fire,
    Ice,
    Lightning,
    Wind,
    Quantum,
    Imaginary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Path {
    #[default]
    Destruction,
    TheHunt,
    Erudition,
    Harmony,
    Nihility,
    Preservation,
    Abundance,
    Remembrance,
    Elation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbilityKind {
    #[default]
    Basic,
    Skill,
    Ult,
    Talent,
    /// 忆灵技能（忆灵回合可选用）
    Memosprite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scaling {
    #[default]
    Atk,
    Hp,
    Def,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmgType {
    #[default]
    Normal,
    Followup,
    Dot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    #[default]
    Single,
    All,
    Adjacent,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    #[default]
    Basic,
    Skill,
    Ult,
    Wait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelicSlot {
    #[default]
    Head,
    Hands,
    Body,
    Feet,
    Sphere,
    Rope,
}

/// 增益作用属性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuffStat {
    #[default]
    AtkPct,
    HpPct,
    DefPct,
    SpeedPct,
    CritRate,
    CritDmg,
    DmgPct,
    DefIgnore,
    ResPen,
    VulnPct,
    BreakEffect,
    EnergyRegen,
    /// 终结技伤害%（风举云飞 4件）
    UltDmgPct,
    /// 战技伤害%
    SkillDmgPct,
    /// 普攻伤害%
    BasicDmgPct,
    /// 追加攻击伤害%
    FollowUpDmgPct,
}

/// 增益作用目标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuffTarget {
    #[default]
    #[serde(rename = "self")]
    Self_,
    Team,
    Ally,
}

/// 效果触发时机
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    /// 施放技能时应用（技能 buff）
    #[default]
    OnUse,
    /// 我方消耗战技点时触发/叠层（花火天赋）
    OnSpConsume,
    /// 进场即生效（套装常驻效果 / 在场被动）
    BattleStart,
    /// 施放终结技后触发（套装被动）
    OnUlt,
    /// 施放战技后触发
    OnSkill,
    /// 施放普攻后触发
    OnBasic,
    /// 受击时触发
    OnHit,
    /// 回合开始时触发
    TurnStart,
    /// 施放追加攻击时触发
    OnFollowUp,
    /// 攻击命中时触发（劫火铸炼宫）
    OnAttack,
    /// 对敌方施加负面时触发（名冶/死水深潜）
    OnApplyDebuff,
    /// 治疗时触发（烈阳女武神）
    OnHeal,
    /// 消灭敌人时触发（千星）
    OnKill,
    /// 成为其他我方目标技能目标时触发（船长）
    OnTargeted,
    /// 忆灵攻击时触发（凯歌英豪；角色行动时近似触发）
    OnMemospriteAttack,
}

/// 通用效果：buff / 光锥被动 / 角色在场被动
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Effect {
    pub trigger: Trigger,
    pub stat: BuffStat,
    pub value: f64,
    /// 持续回合（0 = 永久/进场生效）
    pub turns: u32,
    pub target: BuffTarget,
    /// 战技点上限加成（进队/在场生效）
    pub cap_bonus: i32,
    /// 持有该效果的角色普攻时，全队额外战技点（寒鸦"罚恶"）
    pub sp_on_basic: i32,
    /// 最大叠层（OnSpConsume 类）
    pub max_stacks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AbilityData {
    pub name: String,
    pub kind: AbilityKind,
    /// 基础倍率（multipliers 为空时的回退值）
    pub multiplier: f64,
    /// 每级倍率（下标 0 = 1级），非空时引擎按 skill_level 取档
    pub multipliers: Vec<f64>,
    /// 技能等级（1..=max_level），决定从 multipliers 取哪一档
    pub skill_level: u32,
    pub scaling: Scaling,
    pub flat_damage: f64,
    pub dmg_type: DmgType,
    pub can_crit: bool,
    pub toughness_reduction: f64,
    pub hits: u32,
    pub hit_split: Vec<f64>,
    pub energy_gain: f64,
    pub max_energy: f64,
    /// 战技点变化（按技能：普攻+1/战技−1/强化普攻−N/刃0）
    pub skill_point: i32,
    /// 施放时额外战技点（花火大招 +4）
    pub bonus_sp: i32,
    pub target: Target,
    /// 施放时应用的 buff（角色给人上buff）
    pub buff: Option<Effect>,
    /// 立即行动（目标 AV 归 0，如布洛妮娅/花火战技）
    pub immediate_action: bool,
    /// 目标行动提前比例（按目标基础 AV 的 %）
    pub action_advance_pct: f64,
    /// 施放者自身行动提前比例
    pub self_advance_pct: f64,
    /// 对敌方施加负面（触发 OnApplyDebuff 套装）
    pub applies_debuff: bool,
    /// 治疗（触发 OnHeal 套装）
    pub heals: bool,
    /// 忆灵技能强制触发（死龙/长夜月：回合到必放该技能）
    pub forced: bool,
    /// 一次行动重复施放次数（死龙"本回合不会结束"，1=单次）
    pub repeat: u32,
    /// 施放消耗自身生命上限%（死龙燎尽；忆灵 HP）
    pub hp_cost_pct: f64,
    /// 忆灵生命耗尽时触发（爆炸技能，如死龙灼掠幽墟的晦翼）
    pub on_deplete: bool,
    /// 施放该技能召唤忆灵/召唤物（不在场则召唤，在场则恢复生命）
    pub summons_memo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub path: Path,
    pub element: Element,
    pub base_hp: f64,
    pub base_atk: f64,
    pub base_def: f64,
    pub base_spd: f64,
    pub abilities: Vec<AbilityData>,
    /// 在场被动（进队/在场即生效，如花火天赋 +战技点上限）
    pub team_effects: Vec<Effect>,
    /// 拥有忆灵/召唤物（凯歌英豪 忆灵在场条件）
    pub has_memosprite: bool,
    /// 忆灵速度（独立行动单位，AV=10000/速度；0=不调度）
    pub memosprite_spd: f64,
    /// 忆灵自动攻击倍率（忆灵技能）
    pub memosprite_multiplier: f64,
    /// 忆灵生命≤此比例时施放技能触发爆炸（死龙 5%）
    pub memosprite_explode_pct: f64,
    /// 开战即召唤（神君类普通召唤物）；false=由 summons_memo 技能召唤
    pub summon_at_battle_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LightCone {
    pub id: String,
    pub name: String,
    pub path: Path,
    pub rarity: u32,
    pub base_hp: f64,
    pub base_atk: f64,
    pub base_def: f64,
    pub superimposition: u32,
    pub passive: Option<String>,
    /// 光锥结构化效果（如"花花世界迷人眼" 战技点上限+3）
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RelicSet {
    pub id: String,
    pub name: String,
    pub two_piece: Option<String>,
    pub four_piece: Option<String>,
    /// 二件套数值效果（遗器二件套 / 饰品二件套）
    pub two_piece_effects: Vec<Effect>,
    /// 四件套数值效果
    pub four_piece_effects: Vec<Effect>,
}

/// 装备的遗器套装：set_id + 件数（2 或 4；饰品位固定 2）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RelicSetPiece {
    pub set_id: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MainStat {
    pub slot: RelicSlot,
    pub stat: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Build {
    pub level: u32,
    pub light_cone: Option<String>,
    pub relic_sets: Vec<RelicSetPiece>,
    pub main_stats: Vec<MainStat>,
    pub substats: HashMap<String, f64>,
    pub traces: HashMap<String, bool>,
}

/// 敌方行动（特殊机制）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EnemyAbility {
    pub name: String,
    /// 被击中的我方角色回能量
    pub energy_gain_players: f64,
    /// 战技点变化（Boss 特殊机制：+SP / −SP）
    pub sp_delta: i32,
    /// 扣除我方能量
    pub energy_drain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Enemy {
    pub id: String,
    pub name: String,
    pub level: u32,
    pub def: f64,
    pub max_toughness: f64,
    pub broken: bool,
    pub res: HashMap<Element, f64>,
    pub spd: f64,
    pub actions: Vec<EnemyAbility>,
    /// 弱点属性（仅弱点属性攻击削韧；空 = 不限制）
    pub weaknesses: Vec<Element>,
    /// 生命值（>0 时启用击杀判定；0 = 不判定）
    pub hp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BuffConfig {
    pub atk_pct: f64,
    pub dmg_pct: f64,
    pub crit_rate: f64,
    pub crit_dmg: f64,
    pub def_ignore: f64,
    pub res_pen: f64,
    pub vuln_pct: f64,
    pub break_effect: f64,
    pub weakness_break_eff: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoefficientConfig {
    pub def_const: f64,
    pub broken_multiplier: f64,
    pub break_multiplier: f64,
}

impl Default for CoefficientConfig {
    fn default() -> Self {
        Self {
            def_const: 200.0,
            broken_multiplier: 0.9,
            break_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigData {
    pub characters: Vec<Character>,
    pub light_cones: Vec<LightCone>,
    pub relic_sets: Vec<RelicSet>,
    pub enemies: Vec<Enemy>,
}

// ---------- 队伍 ----------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TeamMember {
    pub char_id: String,
    pub build: Build,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Team {
    pub members: Vec<TeamMember>,
}

/// 战斗环境（特殊模式基线）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BattleConfig {
    /// 战技点上限基线（默认 5；模拟宇宙/特殊模式可提高）
    pub base_sp_cap: i32,
    /// 开局战技点（默认 3）
    pub start_sp: i32,
    /// 开局能量（默认 0）
    pub start_energy: f64,
}

impl Default for BattleConfig {
    fn default() -> Self {
        Self {
            base_sp_cap: 5,
            start_sp: 3,
            start_energy: 0.0,
        }
    }
}

// ---------- 伤害计算 ----------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkillResult {
    pub char_name: String,
    pub ability: String,
    pub base: f64,
    pub crit_rate: f64,
    pub crit_dmg: f64,
    pub non_crit: f64,
    pub crit: f64,
    pub expected: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DamageRequest {
    pub config: ConfigData,
    pub team: Team,
    pub focus: String,
    pub enemy: Enemy,
    pub buff: BuffConfig,
    pub coefficient: CoefficientConfig,
}

// ---------- 排轴 ----------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RotationStepReq {
    pub char_id: String,
    pub action: ActionKind,
    /// 单体 buff/立即行动的目标（布洛妮娅战技→希儿）
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MemospriteStepReq {
    pub owner_id: String,
    /// 忆灵技能下标（角色 abilities 中 kind=memosprite 的序号）
    pub ability_index: u32,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RotationRequest {
    pub config: ConfigData,
    pub team: Team,
    pub enemy: Enemy,
    pub coefficient: CoefficientConfig,
    pub battle: BattleConfig,
    pub steps: Vec<RotationStepReq>,
    /// 忆灵行动序列（忆灵回合按序消耗；空 = 默认第 0 个忆灵技能）
    pub memosprite_steps: Vec<MemospriteStepReq>,
    pub cycles: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RotationStep {
    pub char_id: String,
    pub char_name: String,
    pub action: ActionKind,
    /// 是否敌方行动
    pub is_enemy: bool,
    pub enemy_ability: Option<String>,
    pub av: f64,
    pub damage: f64,
    pub energy: f64,
    /// 全队当前战技点
    pub skill_point: i32,
    pub buffs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RotationResult {
    pub steps: Vec<RotationStep>,
    pub total_damage: f64,
    pub total_av: f64,
}

// ---------- 配装优化 ----------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OptimizeRequest {
    pub config: ConfigData,
    pub team: Team,
    pub focus: String,
    pub enemy: Enemy,
    pub coefficient: CoefficientConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BuildOption {
    pub body: String,
    pub feet: String,
    pub sphere: String,
    pub rope: String,
    pub expected: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OptimizeResult {
    pub best: Vec<BuildOption>,
}

// ---------- 基础换算 ----------

impl Element {
    /// 标准敌方抗性：弱点属性 0%，其余 20%
    pub fn default_res(self) -> f64 {
        0.2
    }
}

impl AbilityData {
    /// 单段命中占比校验：总和应约等于 1
    pub fn validated_split(&self) -> Vec<f64> {
        if self.hit_split.is_empty() {
            vec![1.0]
        } else {
            let sum: f64 = self.hit_split.iter().sum();
            if sum > 0.0 {
                self.hit_split.iter().map(|x| x / sum).collect()
            } else {
                vec![1.0]
            }
        }
    }
}

/// 行动值：10000 / SPD
pub fn action_value(spd: f64) -> f64 {
    if spd <= 0.0 {
        10_000.0
    } else {
        10_000.0 / spd
    }
}
