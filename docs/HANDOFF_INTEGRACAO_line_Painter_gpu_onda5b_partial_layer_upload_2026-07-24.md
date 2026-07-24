# Handoff de integração — `line/Painter` · Onda 5b (o compositor GPU re-envia só a região suja)

**Para:** o agente integrador (DIRETRIZ §1.5.9). **Data:** 2026-07-24.

> ⚠️ Esta branch empilha, em ordem: Ondas 1-2 (compositor GPU), a transferência sRGB do
> Wet Paint, a Onda 5a (a pintura para de copiar o canvas por movimento) e esta (5b).
> Todas integram juntas; handoffs próprios de cada uma ao lado deste.

## 1. Identidade

| | |
|---|---|
| branch | `line/Painter` |
| HEAD | `abe0123ec` (+ 1 commit de docs) |
| commit desta onda | **1** (`abe0123ec`) + 1 docs |

## 2. O que muda, em uma frase

A Onda 5a consertou a pintura trivial (CPU). As três cenas de MÁSCARA (pintar a máscara,
pintar com máscara, pintar após limpar) tornam a pilha **não-trivial** ⇒ vão pelo
**produtor GPU**, que re-enviava a camada **inteira** por movimento (`ensure_slice` →
`write_texture` cheio, 64 MiB @ 4096² por dab — a cópia de staging na CPU). Agora o
compositor re-envia **só a sub-região suja** de uma fatia residente.

Medido (wgpu real, traço com máscara): **5,789 → 0,470 ms/move @ 4096² (~12×)** ·
1,001 → 0,213 @ 2048². Composite continua cheio (GPU, rápido) ⇒ **imagem inalterada**.

## 3. Foundational / contrato

- **`ph2d-render::LayerPixels` ganhou 1 campo:** `pub dirty: Option<Region>`. **5 sítios
  de construção** atualizados — o provider real (`painter_gpu_preview`), o `flip_pass`, e
  3 testes (`layer_compositor_gpu`, `composite_blend`, o test provider) põem `None`.
  ⚠️ Se outra linha construir um `LayerPixels`, o merge **compila-quebrado** (`E0063
  missing field \`dirty\``), não conflita — a correção é `dirty: None`.
- **`ph2d-tool-painter`: superfície pública alterada** — `preview_layer_pixels` retorna
  `Option<(u64, &[u8], Option<(u32,u32,u32,u32)>)>` (era `Option<(u64, &[u8])>`); o
  3º elemento é a região suja da camada ativa. Um chamador só: o provider do shell.
- **Nenhum contrato congelado**, **nenhum schema** (`PROJECT_SCHEMA` fica **29**).
- `LayerCompositor`/`LayerOp`/`flatten_for_gpu` intactos (só o cache de fatias mudou).

## 4. Arquivos

`ph2d-render`: `layer_compositor/mod.rs` (`LayerPixels.dirty`) ·
`layer_compositor/compositor/buffers.rs` (`ensure_slice` partial + `upload_slice_region`) ·
`tests/layer_compositor_gpu.rs` (o gate + `dirty: None` no provider).
`ph2d-tool-painter`: `tool/mod.rs` (campo `preview_dirty_region`) ·
`tool/layers/preview.rs` (`preview_layer_pixels` + `take_preview_dirty`).
`ph2d-flip-render`: `tests/composite_blend.rs` (`dirty: None`).
`shells/desktop`: `render_loop/painter_gpu_preview.rs` (provider mapeia a região) ·
`render_loop/flip_pass.rs` (`dirty: None`) ·
`render_loop/painter_preview_handoff_tests.rs` (a medição `measure_the_masked_stroke...`).

## 5. Símbolos que podem COLIDIR

Nada numerado. O ponto estrutural: qualquer linha que construa um `LayerPixels`
(compila-quebrado, não conflita — `dirty: None`).

## 6. O que rodei

- `cargo fmt --all --check` · `clippy --all-targets` limpos nas 4 crates tocadas · `typos`
  limpo · sem dep nova, `Cargo.toml`/`Cargo.lock` intocados.
- `cargo nextest --workspace --cargo-profile ci-test`: **8899/8899** (excl. a flake
  conhecida `the_cost_of_depth_is_linear_not_explosive`, passa isolada).
- LOC caps ✓.
- ⚠️ **GPU-adapter (`#[ignore]`, `ship.sh` NÃO roda) — rodei aqui, RODE na integração:**
  - `ph2d-render --test layer_compositor_gpu -- --ignored` → **37/37** (inclui o gate novo
    `gpu_partial_layer_upload_patches_only_the_dirty_region`, mutação-provado).
  - `ph2d-host-desktop ... painter_preview_handoff -- --ignored` → **3/3** (o produtor real
    agora com regiões sujas; byte-exato contra a CPU).
  - `measure_the_masked_stroke_on_the_gpu_producer -- --ignored` → 0,213/0,470 ms/move.

## 7. O que smoke-testar

Sprite **2048²** (ou 4096²), Painter aberto:

1. Adicione uma **máscara** e **pinte NELA** — deve estar fluido (antes caía).
2. **Pinte a camada COM a máscara** presente — fluido.
3. **Limpe/remova a máscara e pinte** — fluido.
4. ⚠️ **A aparência NÃO pode mudar** — o composite continua cheio; só o upload ficou
   parcial. Se alguma cor/borda mudar é regressão (e a paridade está gateada).

**Não smokado por mim:** só gates headless (device real); nenhuma janela aberta.

## 8. Aberto (nomeado)

- **Composite parcial** (o resíduo de 0,47 ms/move @ 4096², razão 2,2×): o composite +
  premul + slot-copy seguem cheios (GPU compute, rápido). Fica como a próxima alavanca
  se algum dia incomodar — exige rastrear a fatia "seeded" na sessão GPU (o análogo do
  `plan_upload` da CPU). Doc 25 §12.2.
- **Onda 5 (residência de canvas na GPU)** segue por fazer e **não é necessária** para
  este problema (o upload parcial removeu a cópia O(canvas); o composite é GPU rápido).
