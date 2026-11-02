---
name: project_hierarchy_order_rules_drawing_and_click_2026_11_01
description: "Dono 01/11 — a Hierarchy manda no desenho de TODAS as famílias (Flip, Pixel, Motion incluídos) e o 1.º clique é do que se vê por cima; ícones especiais ganham o clique; ciclo = um clique por degrau"
metadata:
  node_type: memory
  type: project
  originSessionId: ba4e337a-d9fe-4c85-99c3-393e097bcf8f
  modified: 2026-11-01T12:34:27.949Z
---

Dono, 01/11/2026 (linha GameEngineFuncional, onda 19):
1. **Mais acima na Hierarchy = desenha por BAIXO, para TODAS as famílias.** Achado: só sprites e vetor obedeciam
   (bandas por rank); Flip, Pixel Sprites e formas do Motion desenhavam sempre por cima. O dono escolheu
   «desenho e clique» (a cura completa), não só o clique.
2. **O 1.º clique é do objeto que aparece por cima** — o pick segue a MESMA ordem do desenho.
3. **Gizmos de objetos especiais (câmera, lâmpada, macro) têm a preferência do clique** sobre a arte.
4. **Ciclo de sobrepostos: UM clique por degrau** (antes eram dois); carregar e arrastar move o já selecionado
   (idioma do Blender).
5. A **Macro ganha ícone próprio** no canvas, como a câmera.
6. **Todos os tipos de objeto têm o Z-index no seu Inspector** (dono, 01/11 — já era lei de 30/08 para a §7
   Ordering, gate `the_ordering_section_belongs_to_every_object`, mas há tipos onde o dono não a vê).

**Why:** o dono vê o que está por cima e espera que o clique e a Hierarchy digam o mesmo.
**How to apply:** qualquer família nova que desenhe no mundo entra na ordem por rank e no pick por rank; nunca
uma camada fixa por cima. Ver [[project_game_engine_plan_owner_decisions_2026_10_25]].
