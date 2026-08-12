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
  target: "single" | "all" | "adjacent" | "random";
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
}

export interface RelicSetDTO {
  id: string;
  name: string;
  two_piece?: string;
  four_piece?: string;
}

export interface MainStat {
  slot: "body" | "feet" | "sphere" | "rope";
  stat: string;
  value: number;
}

export interface BuildConfig {
  level: number;
  light_cone: string | null;
  relic_sets: string[];
  main_stats: MainStat[];
  substats: Record<string, number>;
  traces: Record<string, boolean>;
}

export interface EnemyDTO {
  id: string;
  name: string;
  level: number;
  def: number;
  max_toughness: number;
  broken: boolean;
  res: Record<Element, number>;
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

export interface RotationStepDTO {
  char_id: string;
  char_name: string;
  action: ActionKind;
  av: number;
  damage: number;
  energy: number;
  skill_point: number;
  buffs: string[];
}

export interface RotationStepReq {
  char_id: string;
  action: ActionKind;
}

export interface RotationRequest {
  config: ConfigDataDTO;
  builds: Record<string, BuildConfig>;
  enemy: EnemyDTO;
  buff: BuffConfig;
  coefficient: CoefficientConfig;
  steps: RotationStepReq[];
  cycles: number;
}

export interface RotationResultDTO {
  steps: RotationStepDTO[];
  total_damage: number;
  total_av: number;
}

export interface OptimizeRequest {
  char_id: string;
  config: ConfigDataDTO;
  build: BuildConfig;
  enemy: EnemyDTO;
  buff: BuffConfig;
}

export interface OptimizeResultDTO {
  best: { body: string; feet: string; sphere: string; rope: string; expected: number }[];
}
