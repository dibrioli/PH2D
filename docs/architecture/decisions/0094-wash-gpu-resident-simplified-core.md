# ADR-0094 — Wash GPU-residente (núcleo simplificado, tempo-real-only)

- **Status:** ACEITO (Enio, 2026-06-14).
- **Contexto-plano:** [`docs/plans/2026-06-14-wash-gpu-resident.md`](../../plans/2026-06-14-wash-gpu-resident.md).
- **Supersede (parcial):** a TOPOLOGIA de execução dos ADR-0086–0092 (storage-buffers, multi-submit,
  janela móvel). **Mantém** a FÍSICA validada (Curtis-1997 mínimo: difusão gated + FlowOutward +
  evaporação, gather conservativo) e a COR (Mixbox residual, ADR-0091). Reassenta tudo sobre
  [ADR-0093](0093-gpu-resident-painter-canvas.md) (canvas GPU-residente, single-submit).

## 1. Decisão

Reimplementar o modo **Wash** (aquarela) como um motor **GPU-residente, single-submit, tempo-real-only**,
escrevendo nas MESMAS texturas residentes que o canvas-GPU (ADR-0093) já entrega ao preview e ao
`LayerCompositor`. Portar a física e a cor **já depuradas** do backup `ph2d-painter-wash` (bugs B1–B9 de
[`wash_solucao_de_erros.md`](../../Painter_projeto/wash_solucao_de_erros.md)) — **portar, não reinventar**.

## 2. Princípio (inegociável)

O Painter é ferramenta de **runtime em tempo real com parâmetros animáveis** (game engine 2D). Portanto
**GPU-first, tempo-real-only: ZERO fallback CPU.** Se a CPU não sustenta o recurso em tempo real, o recurso
não existe nessa forma. Isto supersede a estratégia "cai pro CPU" cogitada na avaliação de gaps GPU.

## 3. Estado (campos do solver) — em TEXTURA, não storage-buffer

A v-antiga usava 8 storage-buffers e teve de **remover `paper` do step** para caber. Migrar para textura
dá headroom de binding (o `paper` volta), amostragem bilinear grátis no composite, e alinha com a
topologia do canvas-GPU. Cada campo dinâmico é ping-pong `_a/_b`; `paper` é estático.

| Campo | Formato | Papel |
|---|---|---|
| `water` | `r32float` | água — transporte + gate de secagem |
| `pig`   | `rgba32float` | pigmento `(absorb.rgb, mass)` — Beer–Lambert, massa conservada |
| `dye`   | `rgba32float` | corante dissolvido (ADR-0089) |
| `res`   | `rgba32float` | residual Mixbox (ADR-0091 — cor fiel) |
| `paper` | `r32float` | tooth/permeabilidade (estático, fora do gate — B5) |

`pig/dye/res` são **rgba32float** (não half): o backup usa buffers `f32`, e o gather conservativo acumula
massa por ~100 substeps — half-float quebraria a conservação (gate). f32 dá paridade ULP apertada com o
oráculo. Storage write-only + leitura amostrada (ping-pong `_in`/`_out`) ⇒ não precisa de `read_write`.
A migração buffer→textura muda a numérica; valida-se com **banda ULP** contra o backup, **um campo por vez**.

## 4. Pipeline (um encoder / submit por frame)

```
splat (deposita pig+water dos gpu_stamps no envelope monotônico)
  → cs_step ×N substeps  (ping-pong nas texturas, dentro do MESMO encoder)
  → composite            (campo → canvas premul + canvas_straight para o inject do compositor)
```

Single-submit; trabalho restrito ao **envelope molhado monotônico** (resolve a costura de região, B3);
**zero readback no hot path** — só no pen-up (como ADR-0093). O tool produz stamps (scheduler CPU
determinístico, HR-5); o shell é dono das texturas + dispatcher (tool sem dep de GPU).

## 5. Cor e composite (portados, não reinventados)

- `composite`: `MASS_MAX` + **saturação suave** `eff = MASS_MAX·(1−e^{−mass/MASS_MAX})` (B1/B5b) +
  **anti-alias gaussiano** `BLUR_RADIUS=2, σ=1.2` (B6). Nunca `min()` duro.
- Cor: **residual Mixbox** (`km.rs`) — `c=unmix(rgb)`, `r=rgb−mix(c)`; decode `mix(c̄)+r̄`. Cor sozinha =
  identidade EXATA; só a mistura wet-on-wet mostra o pigmento espectral (B9).

## 6. Estabilidade (matemática dura — não calibração)

- **CFL único:** difusão + advecção compartilham UM orçamento — `4·(D_MAX+V_MAX)=4·(0.20+0.03)=0.92<1`
  ⇒ nenhuma célula vai negativa ⇒ sem xadrez, por construção (B2).
- **Gather conservativo:** massa de pigmento conservada (gate `inv_mass_conserved_under_diffusion`).
- **Recessão de borda viesada** `EDGE_EVAP_FLOOR·(1−w)` ⇒ rim macio mesmo em **evap=0** (pior caso —
  testado como primário, lição 4).

## 7. Undo = estado de solver (integrado ao histórico transacional)

O undo do wash é **estado**, não controle (B7/B8). `FieldSnap` captura `pig`+`dye`+`water`+`res` (TODO
estado dinâmico; `paper` é estático) do envelope, integrado ao enum `crate::undo` (`Stroke`/`Structural`
+ um braço de estado-de-solver). Regra dura: todo `upload_*` parcial escreve os **DOIS** gêmeos `_a/_b`
(senão o copy-back full do próximo step de região ressuscita o stale — B7).

## 8. Consequências

- **+** Aquarela perfeita (B1–B9 já vencidos) em tempo real, residente, single-submit; substrato para
  params animáveis em runtime; sem o débito submit/copy-bound da v-antiga.
- **−** Migração buffer→textura exige re-validação ULP campo a campo; coexistência temporária com o
  `cs_wash` trivial (ADR-0093) até o motor cobrir o caminho default.
- **Gates:** portados de `backups/wash_2026-06-14/crates/ph2d-painter-wash/tests/` (invariants +
  artifact-repro), rodados headless (Metal, `--ignored`) a cada fase Wn do plano.
