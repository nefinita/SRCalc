# SRCalc 数据目录

数据以**结构化 TOML** 存储，由 `sr_core::store` 模块直接读取（单一数据源），引擎不解析文本描述。

- `characters/<id>.toml` — 角色（基础属性 + 技能倍率 + 每级倍率 + 战技点/回能/buff/行动提前 + 在场被动 team_effects）
- `light_cones/<id>.toml` — 光锥（基础属性 + 被动效果 effects）
- `relic_sets/<id>.toml` — 遗器/饰品套装（二件套/四件套效果：常驻 `battle_start` + 触发式 `on_ult/on_skill/...`）
- `enemies/<id>.toml` — 敌方（防御/抗性/韧性/弱点/行动）

## 维护方式

- **优先**：应用内「数据编辑」页直接修改并保存（权威工具）
- **重新生成**：`python3 ../scripts/import_starrailres.py --out .`（一次性引导，来自 StarRailRes AGPL-3.0）
  - 结构化字段直接映射；无法结构化的（触发式套装被动）用**策展表**写入，不依赖文本解析
- 敌方数据请手动维护

## 数据来源

角色/光锥/遗器套装数值来自 [Mar-7th/StarRailRes](https://github.com/Mar-7th/StarRailRes)（AGPL-3.0，源自 Dimbreath/StarRailData）。
游戏内容版权归 HoYoverse 所有。
