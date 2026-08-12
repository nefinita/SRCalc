export type Element = "physical" | "fire" | "ice" | "lightning" | "wind" | "quantum" | "imaginary";

export type Path =
  | "destruction"
  | "the_hunt"
  | "erudition"
  | "harmony"
  | "nihility"
  | "preservation"
  | "abundance"
  | "remembrance"
  | "elation";

export type AbilityKind = "basic" | "skill" | "ult" | "talent";

export type ActionKind = "basic" | "skill" | "ult" | "wait";

export type BuffStat =
  | "atk_pct"
  | "hp_pct"
  | "def_pct"
  | "speed_pct"
  | "crit_rate"
  | "crit_dmg"
  | "dmg_pct"
  | "def_ignore"
  | "res_pen"
  | "vuln_pct"
  | "break_effect";

export type BuffTarget = "self" | "team" | "ally";

export type Trigger =
  | "on_use"
  | "on_sp_consume"
  | "battle_start"
  | "on_ult"
  | "on_skill"
  | "on_basic"
  | "on_hit"
  | "turn_start";

export interface Effect {
  trigger: Trigger;
  stat: BuffStat;
  value: number;
  turns: number;
  target: BuffTarget;
  cap_bonus: number;
  sp_on_basic: number;
  max_stacks: number;
}

export interface AbilityData {
  name: string;
  kind: AbilityKind;
  multiplier: number;
  multipliers: number[];
  skill_level: number;
  scaling: "atk" | "hp" | "def";
  flat_damage: number;
  dmg_type: "normal" | "followup" | "dot";
  can_crit: boolean;
  toughness_reduction: number;
  hits: number;
  hit_split: number[];
  energy_gain: number;
  max_energy: number;
  skill_point: number;
  bonus_sp: number;
  target: "single" | "all" | "adjacent" | "random";
  buff: Effect | null;
  immediate_action: boolean;
  action_advance_pct: number;
  self_advance_pct: number;
}

export interface CharacterDTO {
  id: string;
  name: string;
  path: Path;
  element: Element;
  base_hp: number;
  base_atk: number;
  base_def: number;
  base_spd: number;
  abilities: AbilityData[];
  team_effects: Effect[];
}

export interface LightConeDTO {
  id: string;
  name: string;
  path: Path;
  rarity: number;
  base_hp: number;
  base_atk: number;
  base_def: number;
  superimposition: number;
  passive?: string;
  effects: Effect[];
}

export interface RelicSetDTO {
  id: string;
  name: string;
  two_piece?: string;
  four_piece?: string;
  two_piece_effects: Effect[];
  four_piece_effects: Effect[];
}

export interface RelicSetPiece {
  set_id: string;
  count: number;
}

export interface MainStat {
  slot: "head" | "hands" | "body" | "feet" | "sphere" | "rope";
  stat: string;
  value: number;
}

export interface BuildConfig {
  level: number;
  light_cone: string | null;
  relic_sets: RelicSetPiece[];
  main_stats: MainStat[];
  substats: Record<string, number>;
  traces: Record<string, boolean>;
}

export interface TeamMember {
  char_id: string;
  build: BuildConfig;
}

export interface Team {
  members: TeamMember[];
}

export interface BattleConfig {
  base_sp_cap: number;
  start_sp: number;
  start_energy: number;
}

export interface EnemyAbility {
  name: string;
  energy_gain_players: number;
  sp_delta: number;
  energy_drain: number;
}

export interface EnemyDTO {
  id: string;
  name: string;
  level: number;
  def: number;
  max_toughness: number;
  broken: boolean;
  res: Record<Element, number>;
  spd: number;
  actions: EnemyAbility[];
  weaknesses: Element[];
}

export interface BuffConfig {
  atk_pct: number;
  dmg_pct: number;
  crit_rate: number;
  crit_dmg: number;
  def_ignore: number;
  res_pen: number;
  vuln_pct: number;
  break_effect: number;
  weakness_break_eff: number;
}

export interface CoefficientConfig {
  def_const: number;
  broken_multiplier: number;
  break_multiplier: number;
}

export interface ConfigDataDTO {
  characters: CharacterDTO[];
  light_cones: LightConeDTO[];
  relic_sets: RelicSetDTO[];
  enemies: EnemyDTO[];
}

export interface SkillResultDTO {
  char_name: string;
  ability: string;
  base: number;
  crit_rate: number;
  crit_dmg: number;
  non_crit: number;
  crit: number;
  expected: number;
}

export interface DamageRequest {
  config: ConfigDataDTO;
  team: Team;
  focus: string;
  enemy: EnemyDTO;
  buff: BuffConfig;
  coefficient: CoefficientConfig;
}

export interface RotationStepReq {
  char_id: string;
  action: ActionKind;
  target: string | null;
}

export interface RotationStepDTO {
  char_id: string;
  char_name: string;
  action: ActionKind;
  is_enemy: boolean;
  enemy_ability: string | null;
  av: number;
  damage: number;
  energy: number;
  skill_point: number;
  buffs: string[];
}

export interface RotationRequest {
  config: ConfigDataDTO;
  team: Team;
  enemy: EnemyDTO;
  coefficient: CoefficientConfig;
  battle: BattleConfig;
  steps: RotationStepReq[];
  cycles: number;
}

export interface RotationResultDTO {
  steps: RotationStepDTO[];
  total_damage: number;
  total_av: number;
}

export interface OptimizeRequest {
  config: ConfigDataDTO;
  team: Team;
  focus: string;
  enemy: EnemyDTO;
  coefficient: CoefficientConfig;
}

export interface OptimizeResultDTO {
  best: { body: string; feet: string; sphere: string; rope: string; expected: number }[];
}
