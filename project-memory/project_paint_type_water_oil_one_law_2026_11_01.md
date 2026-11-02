---
name: project_paint_type_water_oil_one_law_2026_11_01
description: "Dono 01/11: óleo diluído TEM de escorrer; UMA lei (sem 2.º líquido), o diluente do óleo é líquido mais grosso e lento; o Medium vira seletor «Paint type: Water · Oil»"
metadata:
  node_type: memory
  type: project
  originSessionId: 846a4977-a04a-49ef-b2ab-c92b84f7cb8c
  modified: 2026-11-01T15:43:51.015Z
---

Smoke da E10 (01/11): o dono gostou do painel novo e perguntou porque o óleo muito diluído nunca escorre. Causa
medida: a receita pousa água `(1−φ)(1−m)h` — com `m = 1` o diluente do óleo SOME (não há líquido que corra).

Decisão do dono (depois de pedir avaliação séria, «não quero que concorde comigo»):
1. **Física = UMA lei** (ADR-0197): NADA de segundo líquido / imiscibilidade (complexidade de pesquisa, ganho
   pequeno). O diluente é SEMPRE líquido; o meio só muda as propriedades dele: viscosidade (`log η` misturado), secagem
   `E·(1−m)`, cedência, endurecimento com o tempo (o óleo não evapora — sem isso o solver corre para sempre).
2. **Controlo = seletor «Paint type: Water · Oil»** no lugar do slider Medium: só os dois pontos reais são testados e
   garantidos (o «60 % óleo» não existe na vida real). Acrílica = Water com Re-wet 0. Fan Brush (0,5) migra para um
   dos dois.
3. Rejeitado também: «uma mídia só de APARÊNCIA sem física» — foi o que deu as ondas/crateras e os 177 knobs.

**Why:** menos regras especiais e menos superfície de teste; o risco conhecido é cada efeito da aquarela (orla,
backrun, grão, capilar, bloom) vazar para o óleo diluído — conferir CADA UM no óleo pela grelha antes de mostrar.

**How to apply:** a cura vai no MESMO bloco da regressão do Bingham que trava a água sobre o óleo (E10e2 §E3) — a
mesma família. Relacionado: [[project_painter_panel_e10_owner_decisions_2026_11_01]],
[[project_wetpaint_refactor_and_illustrated_help_2026_10_31]].
