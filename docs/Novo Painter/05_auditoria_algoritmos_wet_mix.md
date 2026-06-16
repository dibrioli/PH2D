# 05 — Auditoria: cada algoritmo e cada parâmetro do Wet Mix vs padrão da indústria

> Pedido do Enio (2026-06-16): "Pull parece um carimbo ruim. Confira se os algoritmos usados são padrão da
> indústria. Confira cada algoritmo e cada parâmetro." Confronto da nossa implementação (`cpu_render`, W7)
> contra as fontes canônicas — Krita Color Smudge (source + manual), MyPaint `mypaint-brush.c`, Baxter
> **DAB**/**IMPaSTo**, Photoshop Mixer/Smudge, Procreate Handbook. Veredicto + correção por item.

## Modelo-padrão consolidado (referência)

Mixer-brush = **reservatório** que evolui por dab: **pickup** (amostra a área do canvas vivo sob o dab),
**carry** (EMA do reservatório), **deposit** (rate × load), **depleção de load** (∝ depositado), **dilution**
(transparência). Citações nas linhas de cada parâmetro. Fontes ao final.

---

## Veredictos por parâmetro

| Parâmetro | Nossa impl (antes) | Padrão da indústria | Veredicto |
|---|---|---|---|
| **Pull** | 1 pixel do **backdrop estático** → reservatório | **área** do **canvas vivo** (Krita Dulling/Smudge Radius; MyPaint Gaussian) | ❌→✅ **CORRIGIDO** |
| **Carry (reservatório)** | EMA `r←(1−pull)·r+pull·sample` | EMA `r←L·r+(1−L)·α·sample` (MyPaint) | ✅ é EMA (L=1−pull) |
| **Charge / depleção** | `load −= deposit·K` (∝ depositado) | ∝ volume depositado (IMPaSTo §4.5; PS Load) | ✅ padrão |
| **Attack** | escala a taxa de depósito | deposit-rate (Krita Color Rate / PS Flow) | ✅ padrão (conceito) |
| **Dilution** | `(1−dilution)` reduz cobertura → transparente | reduzir carga/opacidade sobre branco; **não** lerp p/ branco | ✅ padrão |
| **Wetness Jitter** | randomiza `dilution` por-dab | "randomize water mix" (Procreate) | ✅ |
| **Grade** | contraste dos vales da textura (pivota em 1) | "chunkiness/contrast da textura" (Procreate, proprietário) | 🟡 interpretação |
| **Blur** | compõe sobre **backdrop** box-blurred | "blur the laid paint + spread" (Procreate, proprietário) | 🟡 aproximação |
| **Espaço de cor** | linear + Mixbox (pigment) | linear ≥ mínimo; K-M/Mixbox = ouro (IMPaSTo, MyPaint) | ✅ ouro |
| **Spacing do smudge** | usa o spacing do brush (~0.25×∅ default) | **0.05–0.10×∅** (Krita), até 1px (losingfight) | ⚠️ gap (scheduler) |

---

## 1. Pull (smudge) — ❌ era não-padrão → ✅ CORRIGIDO

**Era:** `picked = backdrop[centro]` — **um pixel** do **snapshot pré-stroke**. Dois desvios:
- O padrão amostra a **área** sob o dab (Krita *Smudge Radius* = % do tamanho; MyPaint amostra Gaussiano de
  raio = raio do dab). Um pixel = chapado/ruidoso.
- O padrão lê o **canvas vivo** (a tinta acumulada no traço), não um snapshot. Krita:
  *"as soon as the interstroke data is reset, the paint is considered as 'dried-out'"* — ele amostra o paint
  device vivo. Photoshop Smudge: *"picks up color where the stroke begins and pushes it in the direction you
  drag."* Lendo o backdrop estático, o Pull **re-carimbava o conteúdo antigo de baixo** = o "carimbo ruim".

**Agora (Krita "Dulling"):** `sample = box_average_linear(canvas, cx, cy, smudge_r)`, `smudge_r =
footprint·0.5` (cap 16px) sobre o **canvas vivo**; reservatório = EMA `r←(1−pull)·r+pull·sample`; deposita o
reservatório. O composite continua usando o backdrop pré-stroke (estabilidade do wash) — pickup-source e
composite-source são **independentes**, exatamente como o Krita (que amostra o paint device vivo e compõe
separadamente). Teste `wet_pull_smears_live_canvas_across_boundary` prova que arrasta vermelho através de uma
fronteira. *(Krita docs Color Smudge "Dulling"; MyPaint `mypaint_surface_get_color`.)*

## 2. Carry do reservatório (EMA) — ✅

Padrão MyPaint (`mypaint-brush.c`, verbatim): `reservoir' = L·reservoir + (1−L)·α·sample`, `L` = *smudge
length* (default 0.5). É o que **mata o artefato de discos chapados**: sem histórico, cada dab é fill
constante. Nossa `r←(1−pull)·r+pull·sample` é a mesma EMA com `L=1−pull` (Pull = taxa de atualização do
reservatório). ✅ estruturalmente padrão.

## 3. Charge / depleção de load — ✅

Padrão: **proporcional ao depositado**. IMPaSTo (Baxter §4.5): volume que sai ∝ volume na camada; PS Mixer
Load: *"runs out of paint as you paint with it… at low load rates, paint strokes dry out more quickly."*
Nossa: `load −= deposit_scale·K`, e `deposit_scale ∝ load` → depleção ∝ load. ✅ padrão. `K=0.06` é tuning.

## 4. Attack — ✅ (conceito)

Padrão: deposit-rate (Krita *Color Rate*, PS *Flow*). Nossa: escala a taxa de depósito. ✅. Nuance Procreate
("evenly along your whole stroke" — Attack contrabalançaria a depleção) não modelada; aceitável.
**Nota:** Krita aplica Color Rate **quadrático** (`colorRate²`); o nosso Attack é linear (escolha — Attack ≠
Color Rate exatamente, pois não re-injeta cor de brush; o Procreate carrega 1× via Charge e esfrega).

## 5. Dilution — ✅

Padrão (aquarela/Procreate): **transparência** = reduzir carga de pigmento sobre o branco do papel, **não**
lerp da cor para branco (resultado diferente sob mistura espectral), e **desacoplado** do termo de smear.
Nossa: `(1−dilution)` reduz a cobertura → mais backdrop aparece = transparente; não toca a cor; independente
de Pull. ✅ exatamente o padrão.

## 6. Wetness Jitter — ✅

Procreate: *"Randomize how much water mixes in… at any point."* Nossa: randomiza `dilution` por-dab (hash
determinístico, HR-5). ✅.

## 7. Grade — 🟡 interpretação (Procreate-proprietário)

Não há algoritmo publicado (Grade é específico do Procreate: "chunkiness/contrast da textura"). Nossa:
escala os **vales** da textura (grain×paper) abaixo da cobertura cheia, pivotando em `tex=1` (não escurece
brush liso), `grade=0.5` = identidade. É uma operação de contraste defensável; documentada como
interpretação, não como cópia de um padrão.

## 8. Blur — 🟡 aproximação (Procreate-proprietário)

Procreate: "blur the paint on canvas + how much spreads." O ideal seria **borrar a tinta depositada** (pass
de vizinhança sobre os buffers do traço). A nossa atual **compõe sobre um backdrop box-blurred** (suaviza o
*seam* paint↔canvas), que é uma aproximação barata e por-dab, não um blur do depósito. Documentado como
aproximação; um blur fiel da tinta é um follow-up (pass sobre `wash_color`/`coverage` no dirty-rect).

## 9. Espaço de cor — ✅ ouro

Padrão: misturar em **linear** no mínimo (sRGB lerp = lama); K-M/**Mixbox** é o ouro (IMPaSTo, MyPaint
Pigment on-by-default, Rebelle). Nós compomos em **linear** e, com `pigment_mode`, usamos **Mixbox**
(`pigment_mix`, ADR-0091). ✅.

---

## Gap aberto (recomendação, não corrigido aqui)

- **Spacing do smudge (scheduler).** Brushes de smudge usam spacing **0.05–0.10×∅** (Krita), até **1px**
  (losingfight: *"Smudge has to be spaced by 1 pixel or we get jaggies"*), bem mais apertado que o default
  ~0.25×∅. Com o spacing largo, mesmo o Dulling correto pode mostrar leve disco entre dabs. Fix = quando
  Pull/Wet ativo, o **`stamp_scheduler`** apertar o spacing (ou o brush padrão de Wet Mix nascer com spacing
  baixo). É mudança de geometria (W1/scheduler), fora do `cpu_render`; recomendada como próximo passo se o
  smear ainda parecer descontínuo após o fix do pickup.

## Tuning a validar visualmente (Enio)

`WET_DEPLETE_K=0.06` (vida da carga), `SMUDGE_RADIUS_FRAC=0.5` + `MAX_SMUDGE_RADIUS=16` (raio do pickup),
`WET_GRADE_K=1.6` (força do Grade), `MAX_WET_BLUR_RADIUS=3` (raio do Blur). Todos em
[`cpu_render/mod.rs`](../../crates/ph2d-painter-brush/src/cpu_render/mod.rs).

## Fontes

- Krita Color Smudge (manual + source `KisColorSmudgeStrategyBase.cpp` / `KisColorSmudgeSampleUtils.h`):
  https://docs.krita.org/en/reference_manual/brushes/brush_engines/color_smudge_engine.html
- MyPaint `mypaint-brush.c` (EMA + deposit, verbatim): https://github.com/mypaint/libmypaint
- Baxter **IMPaSTo** (NPAR 2004) + dissertação (DAB/IMPaSTo, K-M): https://www.billbaxter.com/dissertation/Baxter-dissertation.pdf
- Photoshop Mixer Brush / Smudge: https://helpx.adobe.com/photoshop/using/painting-mixer-brush.html
- losingfight (smudge 1px spacing): https://losingfight.com/blog/2007/09/05/how-to-implement-smudge-and-stamp-tools/
- Procreate Handbook (Wet Mix): https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings
- Mixbox (Sochorová & Jamriška, SIGGRAPH Asia 2021): https://scrtwpns.com/mixbox/
