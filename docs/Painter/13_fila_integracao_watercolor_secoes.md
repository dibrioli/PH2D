# 13 — Fila: integração das seções do painel ao sistema Watercolor

> **Origem (Enio, 2026-07-07):** pesquisa mapeou quais seções do painel Painter fluem (ou não) para o
> render-path watercolor. O ponto de bifurcação é `stamp_route.rs::stamp_dabs` — com watercolor ativo o
> batch desvia para `accumulate_wet_*` ANTES de todo o roteamento normal. O que sobrevive é o que já vem
> dentro do `Dab` (gerado pela engine de stroke antes do desvio): pressão (radius/coverage), Strength,
> Randomize Color, Jitter Scale, **Symmetry** (espelhada na engine), Stabilizer, métodos básicos de Stroke.
> Grain é re-propositado (vira o mapa de granulação). O resto é ignorado silenciosamente → esta fila.

## ✅ Feito

- **Seleção + máscara de proteção** (2026-07-07, `0eaa3501`): o watercolor pintava através de seleção
  ativa e região protegida. Fix em 3 camadas keyed no mesmo `keep` (`splat_keep`): splat gates (o wash
  não FORMA em texel gateado → rim/bleed reagem na borda, look de masking-fluid), keep-lerp no composite
  (semântica de restore exata, à prova do vazamento por warp) e snapshot/restore da base no Smudge.
  Teste: `watercolor_respects_selection_and_protection_masks` (com Ragged Edge ligado).

## 🎯 #1 na fila — Shape "Automático" (spec do Enio, 2026-07-07)

**Modo aquarela apenas**, na seção **Shape**: um checkbox **"Automatic"**, **marcado por default**.

- **Marcado (default):** exatamente o comportamento atual — silhueta procedural própria da aquarela
  (disco feather 2-segmentos 1,0→0,92@0,62→0, `watercolor_accum::feather`, + endurecimento SS0/SS1 +
  warp fractal). **Visual byte-idêntico ao de hoje — inegociável.**
- **Desmarcado:** abre os itens COMPATÍVEIS da seção Shape para configurar a silhueta da aquarela:
  - **Falloff** — como não existe preset equivalente ao feather da aquarela, **criar um Falloff novo
    nas opções** (ex.: "Watercolor" — a curva 2-segmentos 0,62/0,92 exata) que vira o selecionado ao
    desmarcar (transição contínua: desmarcar sem mexer em nada = mesmo visual). NOTA: `Falloff` é enum
    do brush (`MAX_FALLOFF` — verificar se há gate de contrato na superfície antes de estender).
  - **Shape image** (silhueta custom) + rotação **Angle/Rake/Random** + **Jitter Rotate** — via
    `sample_unit` + `dab_basis` por dab (o padrão da rota de Smear). Rake = pincel chato de aquarela.
  - **Hardness** — mapear para o endurecimento/feather se fizer sentido; senão fica inativo (dim).
- **Onde muda:** só o carimbo de cobertura (`accumulate_wet_coverage` + `accumulate_wet_color`, que
  compartilham o feather) — TODA a dinâmica (rim, bleed, thinning, granulação, rewet) vive rio abaixo
  da cobertura e segue qualquer silhueta automaticamente.
- **Cuidados mapeados na pesquisa:** endurecimento SS0/SS1 é calibrado pro range do feather (silhueta
  de máximo < 0,60 afinaria o wash → renormalizar); pontas esparsas fazem o edge-darkening contornar
  furos internos (provavelmente bonito, exige smoke); custo por-dab cacheável via `StampMask`
  (bake silhueta-only); o mixer (5-tap) e o depósito por prioridade seguem funcionando (amostram, não
  desenham).

## Fila (demais gaps, sem ordem decidida)

| # | Item | Estado hoje | Nota |
|---|---|---|---|
| 2 | **Tiling** | ignorado (a replicação `tiled_dabs` vive em `stamp_dabs_routed`, depois do desvio) | replicar os dabs antes do accumulate; o composite/dirty-rect precisa do wrap também |
| 3 | **Stroke shape-editors** (Curve/Circle/Polygon/Free Hand) | deliberadamente plain (stampam sem lifecycle → sem base congelada) | dar lifecycle/óptica aos bakes dos editors |
| 4 | **Blend dropdown** | nunca consultado (depósito source-over + óptica própria) | decisão de design: suportar × esconder/dim em modo aquarela (honestidade da UI) |
| 5 | **Composite Brush** | ignorado (desvio antes do `composite_active`) | idem: decidir semântica ou dim |
| 6 | **Jitter Rotate** | inerte (disco redondo) | resolve-se de graça com o #1 (silhueta orientável) |
| 7 | **Shape Tone ramp / Per-Layer Color** | ignorados | avaliar semântica em aquarela (tone da silhueta?) |
| 8 | **Alpha-lock da camada** | não aplicado no bake | mesma família do fix de Seleção; avaliar keep = alpha existente |

> Perf/cor (outra dimensão, não-UI): waves W-A..W-D da auditoria em
> [`12_aquarela_auditoria_pos_f123_padrao_ouro.md`](12_aquarela_auditoria_pos_f123_padrao_ouro.md).
