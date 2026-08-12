import type {
  ConfigDataDTO,
  DamageRequest,
  SkillResultDTO,
  RotationRequest,
  RotationResultDTO,
  OptimizeRequest,
  OptimizeResultDTO,
  CharacterDTO,
  EnemyDTO,
  Team,
} from "../types";

let _invoke:
  | ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>)
  | null = null;

function getInvoke() {
  if (_invoke !== null) return _invoke;
  const api = window.__TAURI__?.core;
  if (api?.invoke) {
    _invoke = api.invoke.bind(api);
    return _invoke;
  }
  _invoke = async () => {
    throw new Error("not in Tauri");
  };
  return _invoke;
}

function isTauri() {
  return !!window.__TAURI__?.core?.invoke;
}

async function invoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  const fn = getInvoke();
  if (!isTauri()) {
    return mockResponse(command, args) as Promise<T>;
  }
  try {
    return (await fn(command, args)) as T;
  } catch (err) {
    throw new Error(String(err));
  }
}

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

async function mockResponse(
  command: string,
  args?: Record<string, unknown>
): Promise<unknown> {
  await delay(200);
  switch (command) {
    case "load_config_cmd":
      return mockConfig();
    case "calculate_damage_cmd":
      return mockCalculate(args);
    case "calculate_rotation_cmd":
      return mockRotation(args);
    case "run_optimize_cmd":
      return mockOptimize(args);
    case "save_character_cmd":
    case "delete_character_cmd":
    case "save_enemy_cmd":
    case "save_team_cmd":
    case "load_team_cmd":
    case "delete_team_cmd":
      return null;
    case "list_teams_cmd":
      return [];
    case "get_module_versions":
      return { core: "0.1.0", const_: "0.1.0" };
    default:
      throw new Error(`未知命令: ${command}`);
  }
}

let cachedConfig: ConfigDataDTO | null = null;

export function mockConfig(): ConfigDataDTO {
  if (cachedConfig) return cachedConfig;
  cachedConfig = {
    characters: [
      {
        id: "1102",
        name: "希儿",
        path: "the_hunt",
        element: "quantum",
        base_hp: 494,
        base_atk: 340,
        base_def: 193,
        base_spd: 115,
        team_effects: [],
        abilities: [
          {
            name: "强袭",
            kind: "basic",
            multiplier: 1.0,
            multipliers: [0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 10,
            hits: 1,
            hit_split: [1],
            energy_gain: 20,
            max_energy: 120,
            skill_point: 1,
            bonus_sp: 0,
            target: "single",
            buff: null,
            immediate_action: false,
            action_advance_pct: 0,
            self_advance_pct: 0,
          },
          {
            name: "归刃",
            kind: "skill",
            multiplier: 3.0,
            multipliers: [2.2, 2.4, 2.6, 2.8, 3.0, 3.2, 3.4, 3.6, 3.8, 4.0],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 20,
            hits: 1,
            hit_split: [1],
            energy_gain: 30,
            max_energy: 120,
            skill_point: -1,
            bonus_sp: 0,
            target: "single",
            buff: null,
            immediate_action: false,
            action_advance_pct: 0,
            self_advance_pct: 0,
          },
          {
            name: "乱蝶",
            kind: "ult",
            multiplier: 5.0,
            multipliers: [4.0, 4.4, 4.8, 5.2, 5.6, 6.0, 6.4, 6.8, 7.2, 7.6],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 30,
            hits: 1,
            hit_split: [1],
            energy_gain: 5,
            max_energy: 120,
            skill_point: 0,
            bonus_sp: 0,
            target: "single",
            buff: null,
            immediate_action: false,
            action_advance_pct: 0,
            self_advance_pct: 0,
          },
        ],
      },
      {
        id: "1306",
        name: "花火",
        path: "harmony",
        element: "quantum",
        base_hp: 700,
        base_atk: 250,
        base_def: 250,
        base_spd: 99,
        team_effects: [
          {
            trigger: "on_use",
            stat: "atk_pct",
            value: 0,
            turns: 0,
            target: "team",
            cap_bonus: 2,
            sp_on_basic: 0,
            max_stacks: 0,
          },
        ],
        abilities: [
          {
            name: "独角戏",
            kind: "basic",
            multiplier: 1.0,
            multipliers: [],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 10,
            hits: 1,
            hit_split: [1],
            energy_gain: 20,
            max_energy: 110,
            skill_point: 1,
            bonus_sp: 0,
            target: "single",
            buff: null,
            immediate_action: false,
            action_advance_pct: 0,
            self_advance_pct: 0,
          },
          {
            name: "梦游鱼",
            kind: "skill",
            multiplier: 0,
            multipliers: [],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: false,
            toughness_reduction: 0,
            hits: 1,
            hit_split: [1],
            energy_gain: 30,
            max_energy: 110,
            skill_point: -1,
            bonus_sp: 0,
            target: "single",
            buff: {
              trigger: "on_use",
              stat: "crit_dmg",
              value: 0.18,
              turns: 2,
              target: "ally",
              cap_bonus: 0,
              sp_on_basic: 0,
              max_stacks: 0,
            },
            immediate_action: false,
            action_advance_pct: 0.5,
            self_advance_pct: 0,
          },
          {
            name: "一人千役",
            kind: "ult",
            multiplier: 2.0,
            multipliers: [],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 20,
            hits: 1,
            hit_split: [1],
            energy_gain: 5,
            max_energy: 110,
            skill_point: 0,
            bonus_sp: 4,
            target: "all",
            buff: null,
            immediate_action: false,
            action_advance_pct: 0,
            self_advance_pct: 0,
          },
        ],
      },
      {
        id: "1101",
        name: "布洛妮娅",
        path: "harmony",
        element: "wind",
        base_hp: 900,
        base_atk: 260,
        base_def: 300,
        base_spd: 99,
        team_effects: [],
        abilities: [
          {
            name: "普攻",
            kind: "basic",
            multiplier: 1.0,
            multipliers: [],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 10,
            hits: 1,
            hit_split: [1],
            energy_gain: 20,
            max_energy: 120,
            skill_point: 1,
            bonus_sp: 0,
            target: "single",
            buff: null,
            immediate_action: false,
            action_advance_pct: 0,
            self_advance_pct: 0,
          },
          {
            name: "战技·归途",
            kind: "skill",
            multiplier: 0,
            multipliers: [],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: false,
            toughness_reduction: 0,
            hits: 1,
            hit_split: [1],
            energy_gain: 30,
            max_energy: 120,
            skill_point: -1,
            bonus_sp: 0,
            target: "single",
            buff: {
              trigger: "on_use",
              stat: "dmg_pct",
              value: 0.33,
              turns: 1,
              target: "ally",
              cap_bonus: 0,
              sp_on_basic: 0,
              max_stacks: 0,
            },
            immediate_action: true,
            action_advance_pct: 0,
            self_advance_pct: 0,
          },
        ],
      },
      {
        id: "1215",
        name: "寒鸦",
        path: "harmony",
        element: "physical",
        base_hp: 800,
        base_atk: 280,
        base_def: 260,
        base_spd: 110,
        team_effects: [],
        abilities: [
          {
            name: "普攻",
            kind: "basic",
            multiplier: 1.0,
            multipliers: [],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 10,
            hits: 1,
            hit_split: [1],
            energy_gain: 20,
            max_energy: 100,
            skill_point: 1,
            bonus_sp: 0,
            target: "single",
            buff: null,
            immediate_action: false,
            action_advance_pct: 0,
            self_advance_pct: 0,
          },
          {
            name: "战技·罚恶",
            kind: "skill",
            multiplier: 0,
            multipliers: [],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: false,
            toughness_reduction: 0,
            hits: 1,
            hit_split: [1],
            energy_gain: 30,
            max_energy: 100,
            skill_point: -1,
            bonus_sp: 0,
            target: "single",
            buff: {
              trigger: "on_use",
              stat: "atk_pct",
              value: 0,
              turns: 2,
              target: "ally",
              cap_bonus: 0,
              sp_on_basic: 1,
              max_stacks: 0,
            },
            immediate_action: false,
            action_advance_pct: 0,
            self_advance_pct: 0,
          },
        ],
      },
    ],
    light_cones: [
      {
        id: "23000",
        name: "于夜色中",
        path: "the_hunt",
        rarity: 5,
        base_hp: 1058,
        base_atk: 582,
        base_def: 396,
        superimposition: 1,
        effects: [],
      },
    ],
    relic_sets: [{ id: "101", name: "云无留迹的过客", two_piece: "治疗量+10%", four_piece: "" }],
    enemies: [
      {
        id: "9000",
        name: "测试木桩",
        level: 80,
        def: 1000,
        max_toughness: 120,
        broken: false,
        spd: 100,
        weaknesses: ["quantum"],
        actions: [
          { name: "普通攻击", energy_gain_players: 10, sp_delta: 0, energy_drain: 0 },
        ],
        res: {
          physical: 0.2,
          fire: 0.2,
          ice: 0.2,
          lightning: 0.2,
          wind: 0.2,
          quantum: 0,
          imaginary: 0.2,
        },
      },
    ],
  };
  return cachedConfig;
}

function mockCalculate(args?: Record<string, unknown>): SkillResultDTO[] {
  const req = args?.req as { team?: Team; focus?: string } | undefined;
  const focus = req?.focus ?? "1102";
  const name = focus === "1306" ? "花火" : "希儿";
  return [
    {
      char_name: name,
      ability: "普攻",
      base: 340,
      crit_rate: 0.3,
      crit_dmg: 1.0,
      non_crit: 170,
      crit: 340,
      expected: 221,
    },
    {
      char_name: name,
      ability: "战技",
      base: 1020,
      crit_rate: 0.3,
      crit_dmg: 1.0,
      non_crit: 510,
      crit: 1020,
      expected: 663,
    },
  ];
}

function mockRotation(args?: Record<string, unknown>): RotationResultDTO {
  const req = args?.req as RotationRequest | undefined;
  const steps = (req?.steps ?? []).map((s, i) => ({
    char_id: s.char_id,
    char_name: "希儿",
    action: s.action,
    is_enemy: false,
    enemy_ability: null,
    av: i * 87,
    damage: s.action === "ult" ? 2000 : 500,
    energy: 100,
    skill_point: Math.max(0, 3 - i),
    buffs: [],
  }));
  return {
    steps,
    total_damage: steps.reduce((a, s) => a + s.damage, 0),
    total_av: steps[steps.length - 1]?.av ?? 0,
  };
}

function mockOptimize(_args?: Record<string, unknown>): OptimizeResultDTO {
  return {
    best: [
      { body: "暴伤", feet: "速度", sphere: "量子伤害", rope: "攻击", expected: 8580 },
      { body: "暴率", feet: "速度", sphere: "量子伤害", rope: "攻击", expected: 8420 },
    ],
  };
}

export function loadConfig(): Promise<ConfigDataDTO> {
  return invoke("load_config_cmd");
}

export function calculateDamage(req: DamageRequest): Promise<SkillResultDTO[]> {
  return invoke("calculate_damage_cmd", { req });
}

export function calculateRotation(req: RotationRequest): Promise<RotationResultDTO> {
  return invoke("calculate_rotation_cmd", { req });
}

export function runOptimize(req: OptimizeRequest): Promise<OptimizeResultDTO> {
  return invoke("run_optimize_cmd", { req });
}

export function saveCharacter(character: CharacterDTO): Promise<void> {
  return invoke("save_character_cmd", { character });
}

export function deleteCharacter(id: string): Promise<void> {
  return invoke("delete_character_cmd", { id });
}

export function saveEnemy(enemy: EnemyDTO): Promise<void> {
  return invoke("save_enemy_cmd", { enemy });
}

export function saveTeam(name: string, team: Team): Promise<void> {
  return invoke("save_team_cmd", { name, team });
}

export function loadTeam(name: string): Promise<Team | null> {
  return invoke("load_team_cmd", { name });
}

export function listTeams(): Promise<string[]> {
  return invoke("list_teams_cmd");
}

export function deleteTeam(name: string): Promise<void> {
  return invoke("delete_team_cmd", { name });
}

export function getModuleVersions(): Promise<{ core: string; const_: string }> {
  return invoke("get_module_versions");
}
