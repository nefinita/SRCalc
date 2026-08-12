# SRCalc · 崩坏星穹铁道 排轴 + 伤害计算器

> 精确计算每一发伤害，科学规划行动轴，一键优选配装。

**技术栈**: Tauri v2 + Rust 核心（`sr_core`）+ React 19 / Vite 前端
**许可证**: AGPL-3.0（见 `LICENSE`）

## 架构

```
Cargo workspace
├── crates/sr_api/     纯 DTO 契约 crate（serde 单一来源，双端 JSON 对齐）
├── crates/sr_core/    计算核心：engine(伤害/排轴/配装) → store(数据) → host(契约) → ffi(C ABI)
├── crates/sr_const/   版本常量 + 击破等级乘数表（1~90）
├── app/               Tauri 壳（src-tauri）+ React 前端（src）
└── data/              TOML 游戏数据（角色/光锥/遗器套装/敌方）
```

核心分层（借鉴 jpcg 工作区模式）：

```
type_set(sr_api DTO) → engine(纯计算) → store(文件/内置) → host(契约方法) → ffi(句柄+JSON)
```

## 伤害公式（依据 HSR Fandom Wiki）

```
DMG = Base × 增伤 × 防御 × 抗性 × 易伤 × 减伤 × 韧性乘区 × 暴击乘区
Base = 倍率 × 攻击(或生命%/防御%) + 固定伤害
防御乘区 = 1 − DEF'/(DEF' + 200 + 10×攻方等级)，DEF' = DEF×(1−无视防御)
抗性乘区 = 1 − (目标抗性 − 穿透)，范围 10%~200%
韧性乘区 = 0.9(未破韧) / 1.0(已破韧)
```

另含完整击破伤害（类型系数 × 等级乘数 × 最大韧性乘数 × (1+击破特攻) × 防御×抗性×易伤×韧性）。

## 行动值（排轴）

```
基础行动值 AV = 10000 / 速度
行动提前/推迟：AV新 = max(0, AV旧 − 基础AV × (提前%−推迟%))
速度变更：AV新 = AV旧 × SPD旧 / SPD新
```

## 功能

| 页面 | 说明 |
|------|------|
| 🛡️ 队伍 | ≤4 角色（角色+光锥+等级+副词条），保存/加载 |
| ⚔️ 伤害计算 | 全队成员各自技能伤害明细 + 击破伤害 + 敌方/全局增益 |
| ⏱️ 排轴 | 四角色行动序列 → AV 时间轴：敌方交错行动、终结技插入、共享战技点轨迹、buff 覆盖 |
| 🎯 配装优化 | 队伍上下文枚举四部位主词条，以期望伤害排序输出 Top 8 |
| 📝 数据编辑 | 技能 buff/额外SP/行动提前、team_effects、敌方行动（回能/回SP/扣能） |

## 战斗机制

- **行动值**：`AV = 10000/速度`，玩家与敌方交错；终结技插入不占行动值
- **战技点**：全队共享、上限 5（可被在场被动/光锥/特殊模式提升）；按技能消耗（饮月强化普攻扣点/刃不耗）
- **buff**：施放应用（全队/单体定向/自身）、消耗战技点触发叠层、按回合递减；行动提前/立即行动
- **敌方**：时间轴交错、攻击回能、特殊机制（回战技点/扣能量）

## 开发

```sh
# Rust 核心
cargo test --workspace
cargo clippy --workspace

# 前端
cd app && npm install && npm run dev    # Vite dev server (1420)
npm run build

# 完整 Tauri 应用（需 npm deps 已装）
cd app && npx tauri dev
```

## 数据

`data/` 下为 TOML 文件，随应用打包（`resources`）。运行时优先读磁盘数据，
缺失时回退内置占位数据。环境变量 `SR_DATA_DIR` 可覆盖数据目录。

**数据来源**：游戏数值（角色基础属性/技能倍率/光锥/遗器套装）来自
[Mar-7th/StarRailRes](https://github.com/Mar-7th/StarRailRes)（AGPL-3.0，
源自 Dimbreath/StarRailData 数据挖掘）。重新生成：

```sh
python3 scripts/import_starrailres.py --out data
```

敌方数据（防御/抗性/韧性）不在该数据源中，请使用应用内「数据编辑」页维护。
游戏内容版权归 HoYoverse 所有。
