---
name: project_painter_volume_is_independent_of_dilution_2026_10_31
description: Dono 31/10 — Pigmento e Água definem só a DILUIÇÃO (concentração); o VOLUME que vai ao papel é outro controlo, independente; matar toda inconsistência do mesmo tipo
metadata:
  type: project
---

Dono 31/10 (noite), durante a refatoração do Painter: *«será preciso matar as inconsistências. Por exemplo: o volume
de tinta que vai ao papel não pode ser definido nem pelo pigmento nem pela água pois essas duas vão definir a diluição
da tinta e não o quanto da mistura vai pro papel. isso é só um exemplo que hoje está errado na nossa engine.»*

**Why:** hoje a carga depositada sai de `PigmentPerDab × Opacity × Intensity × PressureOpacity` e a água de
`Water × WaterPerDab` com a pressão invertida (censo F1, famílias D1 e D4) — mexer em pigmento/água muda QUANTO pinta,
não só a mistura; é um dos cantos do `dono_6`.

**How to apply:** lei ORTOGONAL (ADR-0197 §1.1): Pigmento + Água → concentração φ (a receita da tinta no pincel);
Volume/Carga (+ pressão → área de contacto) → espessura h depositada. Mudar um nunca mexe na grandeza do outro — e há
um portão que o prova. O exemplo é UM caso: cada par de controlos que hoje mexe na mesma grandeza (22 famílias do
censo) se resolve pela mesma regra — um controlo, uma grandeza. Ver [[project_wetpaint_refactor_and_illustrated_help_2026_10_31]].
