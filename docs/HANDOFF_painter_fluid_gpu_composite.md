# HANDOFF — W15.3 GPU composite (próximo agente, START HERE)

> **W15.3 GPU composite está IMPLEMENTADO end-to-end** ([ADR-0049](architecture/decisions/0049-fluid-brushes.md)
> + amendment-1). O shader K–M foi provado bit-a-bit no GPU, o CPU virou single-source, e o shell
> agora roda o composite no GPU lendo o pigmento residente (sem readback de pigmento, sem composite
> CPU). **Falta:** validação VISUAL do Enio no app (`--features fluid`) + 1 otimização opcional
> (preview-texture fully-async). Leia `CLAUDE.md`. Você atua como **Coordenador sozinho**.

---

## §0 — STATUS

**Tudo commitado local, sem push** (fast mode). Quatro commits desta linha:

| Commit | Conteúdo | Prova |
|---|---|---|
| `2d0f5d3` | Shader composite (K–M WGSL) + gate + CPU single-source (`wet_composite.rs`) | `composite_parity` GPU↔CPU = **0 LSB** |
| `5e535d5` | `composite_buffer` lê `pig_a` residente (seam) | `gpu_step_then_composite_resident_matches_cpu` = **0 LSB** |
| `2cbc823` | **Shell resident-composite drive** (remove readback de pigmento + composite CPU) | tool fluid 12/12; clippy clean |

### ✅ Como roda agora (caminho GPU, `--features fluid`)
`drive_fluid_gpu` (`shells/desktop/src/render_loop/painter_fluid_bridge.rs`), por frame:
1. pigmento **GPU-residente** em `solver.pig_a`; os dabs deste frame (grid pigment) sobem como
   `deposit` aditivo (`cs_deposit`) + diffuse/advect no GPU (sem `cs_evaporate`, sem readback).
2. **água = espelho CPU** (sobe pro gate; a CPU evapora + faz o dry-check → sem readback de água).
3. compositor lê `pig_a` + backdrop → **só a faixa de linhas molhada** volta pra `canvas_rgba`
   (a camada canônica que o upload de preview existente + Apply/undo consomem).

### ✅ Testado (headless / Metal)
- `cargo test -p ph2d-painter-fluid --features fluid -- --ignored` → **7/7** (gpu_parity 4 +
  composite_parity 3... na verdade 4 cada agora): composite-only 0 LSB; step+composite residente
  0 LSB; resident-vs-classic 0 Δ; rows==full-band; discriminantes K–M no GPU.
- `cargo test -p ph2d-tool-painter --lib -- fluid` → 12/12 (inclui `gpu_fluid_driven_skips_cpu`).
- naga valida `composite.wgsl` + `fluid.wgsl`; caps intactos; clippy limpo (tool + shell).
- `cargo check -p ph2d-host-desktop --features fluid` ✓.

### ▶︎ COMO O ENIO TESTA (importante — `run-shell.sh` NÃO liga fluid)
```
cargo run -p ph2d-host-desktop --release --features fluid
```
(Pinte com um brush fluid — Brush Studio → fluid enabled. O wash deve bloomar/secar igual ao CPU,
amarelo-sobre-azul→verde, sem franja preta. Em `--release` pra sentir a perf; dev é opt0.)

## §1 — O QUE FALTA

### (A) Validação visual do Enio — BLOQUEADOR de "fechar"
A paridade é 0 LSB no headless, mas o caminho do app (deposit/residência/dry-check/faixa de linhas)
só foi exercido por testes, não na tela. Rode o comando acima e confira o wash ao vivo.
**Possíveis pontos de atenção** (se algo parecer errado):
- **Transient do frame 1:** `gpu_fluid_driven` só liga no `drive_fluid_gpu` (após o `on_tick`), então
  o 1º frame pós-pointer-down pode rodar 1 CPU-tick + 1 GPU-step (duplo processa 1 frame). Imperceptível,
  mas se houver um flash, é aqui (pré-existente no fluxo antigo também).
- **Região = water_bbox(1e-3) ∪ anterior:** se o bloom passar da bbox da água, apareceria corte. O
  composite curto-circuita pixel seco, então deveria cobrir; se cortar, aumente o pad / baixe o threshold.

### (B) Otimização opcional — preview-texture fully-async (tira o ÚLTIMO readback)
Hoje sobra **1 readback da faixa de linhas** por frame (pequeno, mas é um `device.poll`). Pra zerar:
composite → **textura de preview GPU** (não buffer) → premul → copy pro slot `PainterPreviewGpu`
(GPU→GPU, sem readback); readback RGBA só no pen-up pra bakear `canvas_rgba`.
- Blocker estrutural: `drive_fluid_gpu` (render_loop `mod.rs` L242) só tem `tools`+`gpu`; o slot de
  preview vive no `painter_bridge::dispatch` (L1059, com `renderer`). Precisa passar o renderer/slot
  pro drive (ou mover o drive pro contexto do bridge). Estude `painter_gpu_preview.rs` (`PreviewPremul`
  + `copy_texture_into_individual`) — é o padrão a espelhar.
- Adicione `FluidCompositor::composite_to_texture` (saída storage texture `rgba8unorm` em vez de
  `array<u32>`), e no pen-up um `composite_buffer_rows` (já existe) pra bakear `canvas_rgba`.
- **Só vale se a faixa-de-linhas mostrar stall real** (meça em `--release` com wash grande). Pode não
  valer a complexidade — decida no padrão-ouro com número na mão.

## §2 — MAPA DOS ARQUIVOS (o que cada peça faz)
- `ph2d-painter-brush/src/pigment_mix.rs` — `spectral_basis()`, `PreparedPigment::{color,ks,err}`,
  `mix_prepared_exact` (sem LUT, o ground-truth da paridade).
- `ph2d-painter-brush/src/wet_composite.rs` — CPU reference (single source) + bicúbico/bbox +
  `prepare_wet_composite[_from_stroke]`. O tool `composite_wet_field` delega pra cá (fallback CPU).
- `ph2d-painter-brush/src/diffusion.rs` — `clear_pigment`/`evaporate`/`max_water`/`water_bbox` (resident).
- `ph2d-painter-fluid/src/shader/{fluid,composite}.wgsl` — solver (+`cs_deposit`) + K–M composite.
- `ph2d-painter-fluid/src/solver.rs` — `pigment_buffer()`, `step_resident`, `upload_paper`,
  `clear_resident_pigment` (residência, sem readback de pigmento).
- `ph2d-painter-fluid/src/composite.rs` — `FluidCompositor`: `composite_buffer` (lê buffer externo) +
  `composite_buffer_rows` (faixa) + `composite_to_rgba` (upload CPU, conveniência).
- `ph2d-tool-painter/src/tool/{mod,lifecycle}.rs` — `FluidFrameInputs`, `fluid_stroke_epoch`, e os
  hooks `fluid_frame_step_inputs` / `fluid_apply_gpu_composite_rows` / `fluid_dry_check_and_drop` etc.
- `shells/desktop/src/render_loop/painter_fluid_bridge.rs` — o `drive_fluid_gpu` resident-composite.

## §3 — GOTCHAS / INVARIANTES
- **Paridade 0 LSB** no Metal — o shader está certo. Não mexa no shader; só no wiring/perf.
- **pcol vem da COR DO STROKE** (`prepare_wet_composite_from_stroke`) no caminho GPU — é igual ao
  total do grid pra stroke de 1 cor (o `Σamount` cancela; tem teste).
- **NÃO re-suba pigmento por frame** (resetaria o bloom) — só o `deposit` aditivo. Água SIM (espelho CPU).
- **`composite_wet_field` (CPU) continua o fallback** — quando `--features fluid` está OFF ou device
  incapaz. Não o quebre.
- **`run-shell.sh` não liga fluid** — use `--features fluid` no `cargo run` (acima). Considere adicionar
  um arg `fluid` no script (mexe em `scripts/`, coordene).
- **Det/§2.11**: composite é frame-driven, fora do replay HR-5. O solver det fallback é o CPU `diffusion`.
- **NÃO pusha** (fast mode — Enio valida visual antes).

— deixado por Claude (sessão brush-overhaul + W15.3 GPU composite — gate + integração, 2026-06-07).
