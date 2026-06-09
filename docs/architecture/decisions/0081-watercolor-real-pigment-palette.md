# ADR-0081 — Watercolor: real artist-pigment palette (granulation · staining/lift · transparency)

**Status:** Accepted (2026-06-09) — pedido pelo Enio após o smoke OK do K–M ([ADR-0080](0080-watercolor-km-multipigment-field.md)): "paleta de pigmentos reais — ultramarine granula+levanta, phthalo mancha". Feature #1 da fila de 3 (loop autônomo; #2 = MoXi/LBM franja, #3 = 4K residency).
**Decisor(es):** Enio (dono/decisor) + Claude.
**Estende:** [ADR-0080](0080-watercolor-km-multipigment-field.md) (campo K–M multi-pigmento, 28 canais), [ADR-0079](0079-watercolor-params-per-brush-exposure.md) (`WatercolorParams`), [ADR-0078](0078-watercolor-gold-standard-resident-tiled-shallow-water.md) (deposição S3 / granulação).
**Tags:** painter, watercolor, pigment, granulation, staining, lift, contract-surface, gpu-parity

---

## 1. Contexto

O campo K–M (ADR-0080) já mistura cor subtrativamente. Mas todo pigmento se comporta IGUAL:
a **granulação** é um único slider per-brush (`WatercolorParams.granulation`, uniforme no campo) e
não há **staining** (fixação) nem **lift** (re-molhar levanta tinta seca). Pigmentos reais diferem
fisicamente: French Ultramarine **granula** (sedimenta nos vales do papel) e **levanta** (não-
fixador); Phthalo **mancha** (fixa o papel, não levanta) e é transparente; Cerulean granula e é
semi-opaco. Hoje nenhum desses traços existe — e quando dois pigmentos se misturam no campo, o
comportamento físico não viaja com eles.

## 2. Decisão

### 2.1 `Pigment` + paleta curada (`pigment_palette.rs`, `ph2d-painter-brush`)
Um tipo `Pigment { name, srgb (masstone), granulation, staining, transparency }` + uma `PALETTE`
curada de ~16 pigmentos reais (caracterizações-padrão de aquarela; não dados medidos). O masstone
alimenta o K/S (a diluição emerge do `mass` do campo — masstone↔undertone de graça). `granulation`/
`staining`/`transparency` ∈ [0,1] são as PROPRIEDADES FÍSICAS.

### 2.2 As propriedades viajam NO CAMPO (per-célula, ponderadas por massa) — o ponto-chave
`granulation` e `staining` são **acumuladores ponderados por massa**, exatamente como `ks`/`err`:
`gran_acc = Σ mass_i·gran_i`, `stain_acc = Σ mass_i·stain_i`. Logo **transportam LINEARMENTE** (os
mesmos stencils diffuse/advect/transfer/capillary) e a propriedade do MIX é `gran_acc/mass` —
ultramarine continua granulando mesmo depois de misturado com phthalo. O campo cresce de **28→32
canais** (`PIG_CH=32`, `PV=8` vec4): `[ks 0..24][err 24..27][mass 27][gran 28][stain 29][pad 30..32]`.
O custo é ~14% de memória low-res (irrelevante) e o transporte é **grátis** (os passos já iteram `PV`
vec4 idênticos por canal). A redução do composite IGNORA gran/stain (são comportamento, não cor) →
**a cor composta é inalterada**.

### 2.3 Granulação per-célula no `transfer_pigment`
`transfer_pigment` (deposição S3) passa a ler a granulação **da célula** (`gran_acc/mass`) no lugar
do `params.granulation` uniforme. O slider per-brush `granulation` vira o **default do dab** (um traço
SEM pigmento selecionado deposita com a granulação do brush — comportamento atual preservado).

### 2.4 Lift (re-molhar levanta tinta) — NOVO passo, **opt-in (default 0)**
Novo `WatercolorParams.lift` ∈ [0, 1] (default **0**). Um novo passo `lift_pigment` move
deposited→flowing por `lift · smoothstep(w_lo,w_hi,water) · (1 − stain_acc_dep/mass_dep)` por célula:
área molhada re-mobiliza a tinta seca NÃO-fixadora (levanta), e o staining (per-célula, do pigmento
depositado) **resiste**. `lift = 0` ⇒ passo dormante ⇒ **bit-idêntico ao caminho ADR-0080** (o
deposited fica congelado como hoje). Conservativo (massa sai de deposited, entra em flowing).

### 2.5 Não-destrutivo (inegociável, ADR-0078 discipline)
Tudo defaulta para a IDENTIDADE: `lift = 0` (sem lift), dab sem pigmento ⇒ `stain = 0` + `gran =
brush.granulation` (o slider de hoje). Um traço de cor crua com os params validados é **bit-idêntico**
ao ADR-0080. A paleta + lift só AGEM quando o usuário escolhe um pigmento / sobe o `lift`.

### 2.6 Seleção + UI
v1: a UI de paleta seleciona um `Pigment` ativo → o pick seta a cor do brush (masstone → OKLCH via
`srgb8_to_painter_oklch`) + `WatercolorParams.granulation` (do pigmento) e marca o pigmento ativo
(seu `staining`/`gran` vão nos dabs). Sem pigmento ativo = cor crua (comportamento atual).

## 3. Impacto em contratos

- **`PIG_CH = 28 → 32`** (`PV = 7 → 8`): campo CPU + mirror GPU + readbacks + parity (gran/stain
  comparados além de cor+mass; transporte 0-ULP mantido). `SPECTRAL_BANDS=24` intacto.
- **`WatercolorParams ≤ 18` → 18/18** com `+lift` (ADR-0079-amendment-2). `CONTROLS` += "Lift"
  (APPEND — índice de contrato). Gate `architecture_painter_contract_surface` substring atualizado.
- **`pigment_palette.rs`**: tipo + paleta novos (`ph2d-painter-brush`), sem ABI serializada nova.
- **HR-5:** gran/stain/lift são aritmética pura; defaults identidade ⇒ single-color byte-idêntico.

## 4. Consequências

Pigmentos reais nomeados que se COMPORTAM distinto — granulam, mancham/levantam, glazeiam — e o
comportamento sobrevive à mistura no campo (per-célula). Tudo opt-in: o look validado é preservado
por construção. Próximos da fila: **#2 MoXi/LBM franja ramificada** (ADR-0082, opt-in) e **#3 4K
full-res residency** do campo (ADR-0083). [feedback-no-industrial-claims]: as propriedades dos
pigmentos são caracterizações-padrão de aquarela, não dados medidos citados.
