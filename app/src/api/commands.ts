import type {
  ConfigDataDTO,
  SkillResultDTO,
  RotationRequest,
  RotationResultDTO,
  OptimizeRequest,
  OptimizeResultDTO,
  CharacterDTO,
  EnemyDTO,
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
      return null;
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
        id: "1101",
        name: "希儿",
        path: "the_hunt",
        element: "quantum",
        base_hp: 1000,
        base_atk: 600,
        base_def: 300,
        base_spd: 115,
        abilities: [
          {
            name: "普攻",
            kind: "basic",
            multiplier: 1.1,
            multipliers: [1.1],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 30,
            hits: 1,
            hit_split: [1],
            energy_gain: 20,
            max_energy: 100,
            skill_point: 1,
            target: "single",
          },
          {
            name: "战技",
            kind: "skill",
            multiplier: 2.2,
            multipliers: [2.2],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 60,
            hits: 1,
            hit_split: [1],
            energy_gain: 30,
            max_energy: 100,
            skill_point: -1,
            target: "single",
          },
          {
            name: "终结技",
            kind: "ult",
            multiplier: 4.0,
            multipliers: [4.0],
            skill_level: 6,
            scaling: "atk",
            flat_damage: 0,
            dmg_type: "normal",
            can_crit: true,
            toughness_reduction: 90,
            hits: 1,
            hit_split: [1],
            energy_gain: 5,
            max_energy: 100,
            skill_point: 0,
            target: "single",
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
        base_hp: 1000,
        base_atk: 582,
        base_def: 300,
        superimposition: 1,
      },
    ],
    relic_sets: [{ id: "101", name: "巡猎套装", two_piece: "攻击力+12%", four_piece: "" }],
    enemies: [
      {
        id: "9000",
        name: "测试木桩",
        level: 80,
        def: 1000,
        max_toughness: 120,
        broken: false,
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

function mockCalculate(_args?: Record<string, unknown>): SkillResultDTO[] {
  return [
    {
      char_name: "希儿",
      ability: "普攻",
      base: 660,
      crit_rate: 0.3,
      crit_dmg: 1.0,
      non_crit: 330,
      crit: 660,
      expected: 429,
    },
    {
      char_name: "希儿",
      ability: "战技",
      base: 1320,
      crit_rate: 0.3,
      crit_dmg: 1.0,
      non_crit: 660,
      crit: 1320,
      expected: 858,
    },
  ];
}

function mockRotation(args?: Record<string, unknown>): RotationResultDTO {
  const req = args?.req as RotationRequest | undefined;
  const steps = (req?.steps ?? [{ char_id: "1101", action: "basic" }]).map(
    (s, i) => ({
      char_id: s.char_id,
      char_name: "希儿",
      action: s.action,
      av: i * 87,
      damage: s.action === "ult" ? 2000 : 500,
      energy: 100,
      skill_point: 3 - i,
      buffs: [],
    })
  );
  return {
    steps,
    total_damage: steps.reduce((a, s) => a + s.damage, 0),
    total_av: steps[steps.length - 1]?.av ?? 0,
  };
}

function mockOptimize(_args?: Record<string, unknown>): OptimizeResultDTO {
  return {
    best: [
      {
        body: "暴伤",
        feet: "速度",
        sphere: "量子伤害",
        rope: "攻击",
        expected: 8580,
      },
      {
        body: "暴率",
        feet: "速度",
        sphere: "量子伤害",
        rope: "攻击",
        expected: 8420,
      },
    ],
  };
}

export function loadConfig(): Promise<ConfigDataDTO> {
  return invoke("load_config_cmd");
}

export function calculateDamage(
  req: Record<string, unknown>
): Promise<SkillResultDTO[]> {
  return invoke("calculate_damage_cmd", { req });
}

export function calculateRotation(
  req: RotationRequest
): Promise<RotationResultDTO> {
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

export function getModuleVersions(): Promise<{ core: string; const_: string }> {
  return invoke("get_module_versions");
}
