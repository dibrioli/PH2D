# Watercolor v2 — backup (2026-06-12)

Snapshot **fiel** do sistema watercolor (ADR-0078..0085) imediatamente **antes** da
super-simplificação planejada pelo Enio (2026-06-12).

Motivo do backup (Enio): *"Apesar de ter recursos interessantes, o sistema watercolor
é instável, inconsistente e de difícil manutenção."* Vamos tentar uma reescrita
drasticamente mais simples; este é o ponto de retorno se algo se perder.

## Estado capturado

- **Commit:** `9fa573bf` (branch `main`).
- **Tag git:** `watercolor-v2-backup-2026-06-12` → ponteiro autoritativo (restauração exata).
- Esta pasta é uma **cópia de conveniência** (apenas fonte; sem `target/`/`target-slots/`)
  para consulta lado-a-lado durante a reescrita. A restauração exata vem da tag/commit.

## O que está incluído

| Parte | Arquivos |
|---|---|
| **Motor GPU** (coração) | `crates/ph2d-painter-fluid/` inteira — `solver.rs`, `composite.rs`, `sim.rs`, `budget.rs`, `params.rs` + todos os shaders `src/shader/*.wgsl` + `tests/` |
| **Pigmento / params (brush)** | `crates/ph2d-painter-brush/src/{watercolor,diffusion,pigment,pigment_mix,pigment_palette,wet_composite,wet_mix}.rs` |
| **Integração (tool)** | `crates/ph2d-tool-painter/src/{lib,params}.rs` + `src/tool/{lifecycle,mod,runtime,tests,trait_impls}.rs` |
| **Contrato + ADRs** | `architecture_painter_contract_surface.rs` + `docs/architecture/decisions/0078..0085-*.md` |

## Como restaurar

**Exato (recomendado), via git:**
```sh
# inspecionar
git show watercolor-v2-backup-2026-06-12
git checkout watercolor-v2-backup-2026-06-12 -- crates/ph2d-painter-fluid   # um caminho
# ou voltar a crate inteira a esse estado e seguir dali
```

**Da pasta (cópia de arquivos):** copiar de volta os arquivos desta pasta para os
caminhos `crates/...` correspondentes (os caminhos aqui espelham a raiz do repo).

## Última arquitetura (resumo do que será simplificado)

GPU-resident, single-submit, K–M espectral 24 bandas (PIG_CH=32), passes por substep:
`splat → lift → diffuse → advect → transfer(deposit) → evaporate → capillary → combine`.
Controles per-brush em `WatercolorParams::CONTROLS` (21). Último mecanismo de bounding:
**Bleed Limit** = set-timer `gel` que congela o wick (ADR-0079-amendment-2).
