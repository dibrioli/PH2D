---
name: project_color_mixing_tuned_for_our_engine_not_csp_2026_11_08
description: Dono 08/11 — pincéis com Color Mixing (Wet blend) afinam-se pelo bom resultado no NOSSO motor, não pelo Clip Studio; Round Mixing = Amount 0,25 + Hardness 0,5
metadata:
  type: project
---

Dono (08/11, depois da revisão v3): com os valores casados ao Clip Studio, a maioria dos pincéis com Color
mixing: Wet blend **não mistura** — o Round Mixing com Amount 0,9 sai «vermelho fosco em qualquer traço».
Ele achou à mão: **Round Mixing = Wet blend Amount 0,25 + Shape Hardness 0,5** (mescla suave, borda sem
serrilhado).

**Why:** o Clip Studio deixa de ser a referência para a MISTURA; casar números com ele matou o efeito no
nosso motor. Ver [[project_rebelle_no_longer_the_reference_2026_10_31]] (mesma lógica, outro app).

**How to apply:** Amount, Hardness e os outros campos do Wet blend de todo pincel com mistura afinam-se pelo
resultado no PH2D (a mistura tem de se VER), com o Round Mixing do dono como âncora do «bom». Amount < 0,25
deixa de cobrir a camada vazia (`AMOUNT_QUE_COBRE`). Watercolor Splash / Wet Wash melhoram-se com imagens
gratuitas (licença conferida). Pointillism: o CSP continua superior — investigar porquê.
Revisão v3: `docs/Painter/REVISAO_CSP_V3_2026-11-08.md`; [[feedback_brush_oracle_must_include_blank_canvas]].

**Decisões do dono 08/11 (depois da auditoria mistura × papel/grão/composite/camadas):**
(a) esbatedores: APROVADA a cura que pesa pela quantidade (Amount 0 nunca traz a cor do pincel), mesmo mudando a cor
a meio da carga em todos os pincéis com mistura — fotos antes de ficar. (b) Composite: cada parte mistura só a tela
de ANTES do traço (estado da mistura por posição da pilha). (c) SIM a uma opção de amostra «This layer» / «All
visible layers»; «All» de fábrica nos esbatedores. Material: `docs/Painter/handoffs/material_2026-11-08_auditoria_*`.
