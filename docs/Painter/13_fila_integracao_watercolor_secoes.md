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

## ✅ #1 — Shape "Automatic" (LANDOU 2026-07-07, `5000decc`)

Implementado conforme a spec abaixo: checkbox na Shape (modo aquarela, default marcado =
byte-idêntico, provado por teste de continuidade via o seam real), `Falloff::Watercolor` novo
(wire 10, bit-idêntico ao feather — avaliado no `t` original, round-trip `1−p` perdia bits),
engine `WetShapeStamp` nos dois splats com RNG replay-sync entre os passes. Hardness participa via
`falloff_weight`. Deferidos anotados: Shape Tone ramp no stamp (item #7) e `dab_flatten`.
Jitter Rotate (#6) resolvido de brinde. **Round 2 (`1d4ecc36`, pós-smoke):** Flatten/Rotate
integrados ao envelope (fp.falloff_t quando não-identity; identity mantém dn bit-exato) +
**normalização da ponta** (1/max_lum por traço): cobertura watercolor é geometria de molhado
max-blend que precisa SATURAR (cw→1 corpo, inner→1 rim) — luminância tonal crua deixava centro
pálido e rim morto. Ponta cinza uniforme == branca byte-a-byte; textura relativa sobrevive.
**Round 3 (`57639e65`):** ponta TEXTURIZADA mantém a aquarela típica — split molhado/pigmento:
wet = envelope saturado (imagem via rampa TIP_WET 0.03→0.20; procedural = só falloff) + density =
a textura, acumulada em `stroke_density` (per-stroke, max-blend) e multiplicada no fill do
composite. Corpo molhado + rim no contorno EXTERNO + textura como variação de pigmento; pigmento
0 = "só água". `watercolor_render` re-split pro teto LOC (RewetFields+consts → field).

## 🎯 spec original — Shape "Automático" (Enio, 2026-07-07)

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

> ✅ **RESOLVIDO 2026-07-09 (rota a+b combinadas):** tabela de estilos por-traço da sessão
> (`WetSessionStyles` em `watercolor_field.rs` — fill/depth/edge_gain/wet/granulation/warp/
> pigment_mix/cor-fallback, capturados com os clamps EXATOS do composite no pen-down) + mapa u8
> de DONO por-pixel (recência, casa com o source-over da cor; splat no coverage pass). O composite
> resolve os params do dono por pixel (leitura warpada; warp usa o dono pré-warp); geometria/
> campos (pad, rewet build, soaked) usam MÁXIMOS da sessão (conservador). Owner 0 / sem estilos =
> caminho antigo bit-idêntico (491/491 verdes). Fica global por composite (documentado): core_r/
> spread do blur, fonte da textura de grain, spread_thin, paper. Teste refutável:
> `watercolor_session_keeps_each_strokes_style` (wash 1 byte-exato pós-união; FAIL sem o fix).
> Bloco de granulação extraído p/ `granulation_factor` no field (teto LOC).
>
> ~~**1º DA FILA (ordem do Enio, 2026-07-09): BUG da sessão molhada — parâmetros por-traço.**~~
> Traço 1 com Concentration 1.0 + traço 2 com 0.3 na mesma sessão ⇒ no pen-up o re-bake da
> união re-estiliza o traço 1 com 0.3 (o composite lê os params CORRENTES do brush pro conjunto;
> era o caveat documentado do EDGE-1 take 2 — confirmado no smoke, não é aceitável). Rotas a
> avaliar amanhã: (a) dobrar Concentration (e candidatos: fill/depth) no **mapa por-pixel de
> reserva** no splat (o `stroke_deplete` já multiplica fill+edge — carrier natural, cuidado com o
> caminho byte-idêntico do default); (b) mapa u8 de índice-de-traço + snapshot de params por
> traço da sessão (geral, mais estado); (c) params divergentes ⇒ encerra sessão (barato, mas
> perde a fusão — último recurso). Escolher a que preserva o caráter POR TRAÇO mantendo a fusão.
>
> ~~**Sintoma adicional (Enio 2026-07-09):** QUALQUER mudança no brush com poças úmidas propaga
> pelas poças dentro da área RETANGULAR de ação do brush~~ — **RESOLVIDO 2026-07-09**, junto com
> o retângulo-que-clareia-sem-mudar-params (raiz mais funda que os params do #1: o composite não
> era função PURA do estado da sessão — campos de rewet lidos do base per-stroke envenenado,
> settle na flag do frame, soak zerado por traço, core_r/spread_thin do brush vivo, px de água
> sem dono). Doc 12 §"Reprodutibilidade da sessão". Testes:
> `watercolor_session_rerender_reproduces_the_bake_byte_exact` +
> `watercolor_session_brush_changes_do_not_touch_baked_washes`.

| # | Item | Estado hoje | Nota |
|---|---|---|---|
| 2 | **Tiling** | ignorado (a replicação `tiled_dabs` vive em `stamp_dabs_routed`, depois do desvio) | replicar os dabs antes do accumulate; o composite/dirty-rect precisa do wrap também |
| 3 | **Stroke shape-editors** (Curve/Circle/Polygon/Free Hand) | deliberadamente plain (stampam sem lifecycle → sem base congelada) | dar lifecycle/óptica aos bakes dos editors |
| 4 | **Blend dropdown** | nunca consultado (depósito source-over + óptica própria) | decisão de design: suportar × esconder/dim em modo aquarela (honestidade da UI) |
| 5 | ~~**Composite Brush**~~ | ✅ escondido em modo aquarela (`a7712f45`, decisão Enio); Strength não some junto | — |
| 6 | ~~**Jitter Rotate**~~ | ✅ resolvido com o #1 (silhueta orientável, `5000decc`) | — |
| 7 | **Shape Tone ramp / Per-Layer Color** | ignorados | avaliar semântica em aquarela (tone da silhueta?) |
| 8 | **Alpha-lock da camada** | não aplicado no bake | mesma família do fix de Seleção; avaliar keep = alpha existente |
| 9 | **Botão "Dry" (secar rápido)** (Enio 2026-07-09) | — | zera o `canvas_wet` + encerra a sessão molhada na hora (bake congelado vira definitivo); equivalente ao "Dry the layer" do Rebelle. UI: painel Watercolor |
| 10 | **Botão "Wet" (molhar canvas)** (Enio 2026-07-09) | — | re-molha o papel (pour manual no `canvas_wet`, canvas todo ou região) SEM depositar pigmento — próximo traço funde/bloom sobre a pintura existente; é o "Wet the layer" do Rebelle (doc 11 §6 tinha excluído — pedido explícito agora) |
| 11 | **Slider de tempo de secagem** (Enio 2026-07-09) | const `CANVAS_WET_DRY_PER_S = 30` (~8,5 s) | expor como slider (ex.: 2 s–60 s, ou "∞ = nunca seca sozinho"); vira o knob de calibração da janela de fusão do EDGE-1 |
| 12 | **Preview de umidade + secagem paulatina** (Enio 2026-07-09) | mapa u8 já existe e decai por byte | (a) overlay on-canvas do `canvas_wet` (véu/brilho de umidade, estilo Rebelle "show wetness"); (b) a secagem GRADUAL influenciando a mescla — hoje a fusão da sessão é binária (molhado enquanto `rect` vivo); usar o valor local do mapa pra atenuar progressivamente a fusão/derretimento do rim conforme seca (meio-seco = meio-rim), casando com o settle do GRAN-1 |

**Blur do Wet Mix: exposto (`a7712f45`) e REVERTIDO no smoke** (Enio: "funcionava melhor quando
ele não era configurável") — o pickup do mixer fica FIXO em r×0,5 (cerca de Chesterton anotada no
`sample_surface`; não re-expor sem novo smoke).

> Perf/cor (outra dimensão, não-UI): waves W-A..W-D da auditoria em
> [`12_aquarela_auditoria_pos_f123_padrao_ouro.md`](12_aquarela_auditoria_pos_f123_padrao_ouro.md).
