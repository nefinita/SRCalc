# SRCalc 数据目录

- `characters/<id>.toml` — 角色（基础属性 + 技能倍率）
- `light_cones/<id>.toml` — 光锥
- `relic_sets/<id>.toml` — 遗器套装
- `enemies/<id>.toml` — 敌方（防御/抗性/韧性）

应用内「数据编辑」页可直接增删改。目录不存在时回退内置占位数据。

## 数据来源与再生成

角色/光锥/遗器套装数值来自 [Mar-7th/StarRailRes](https://github.com/Mar-7th/StarRailRes)
（AGPL-3.0，源自 Dimbreath/StarRailData）。重新生成：

```sh
python3 ../scripts/import_starrailres.py --out .
```

敌方数据请手动维护。游戏内容版权归 HoYoverse 所有。
