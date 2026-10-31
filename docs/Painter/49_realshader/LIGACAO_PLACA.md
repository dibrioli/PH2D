# A água na placa LIGADA ao produto — degraus A+B (31/10, onde parou: prazo de 45 min)

Continuação de [PLACA_degrau5.md](PLACA_degrau5.md).

## Feito (compila; ⛔ nenhum gate corrido)

- **A**: `ph2d-tool-painter` depende de `ph2d-wet-gpu` (opcional, na feature `gpu`, que o desktop e agora
  o `ph2d-app-painter` ligam). `GpuContext` é `#[derive(Clone)]` com `Arc` por dentro ⇒ vai ao worker.
  O device do app chega por `PainterTool::instala_placa_do_wet(renderer.gpu())` (um `OnceLock` no tool),
  chamado nos dois `PainterGpuPreview::new` de `crates/ph2d-app-painter/src/painter_gpu_preview.rs`.
  `PH2D_WET_GPU` lido 1× (`offthread/placa.rs::ligada`). ⚠️ **Desvio do briefing**: por ora só `=1`
  liga a placa; a omissão continua CPU até o gate (2) estar verde (código não medido não vai por omissão).
- **B**: `crates/ph2d-wet-gpu/src/passo.rs` — `placa_sabe` (Cobre, sem K–M, sem paredes/gota/óleo/ladeira,
  sem fingering/backrun/difusão, Jacobi, sem passo em voo) e `PassoPlaca` (o laço de `sim_step`:
  rebuild → lê a caixa (64 B) e aborta sem fluido · secagem 1 em `dry_every` · campo/suave · sedimenta ·
  bordas · projeção · `vmax` lido de volta para a cadência).
  No worker (`wetpaint/offthread/placa.rs`, ligado em `worker_loop`): **ao receber** o motor sobe a grade
  INTEIRA (cobre toda escrita da CPU: dab, deposit_mask, acções — tudo passa por `bring_home`); **ao
  devolver** desce a grade INTEIRA (cobre composite, undo, save, drain). Decide-se ao receber ⇒ nunca troca
  a meio. A história marca a grade inteira (`active_grid_mut`) porque `marca_passo` é `pub(crate)`.

## Defeitos conhecidos ANTES de medir (o próximo agente começa aqui)

1. **Secagem ida-e-volta**: o `st` entrelaçado (film, susp) passa pela CPU em cada secagem (lento);
   `rgb`/`sett` copiam-se na placa. Cura: o `SecaGpu` partilhar os buffers do `AdvectGpu`.
2. **`vmax`** lê `uv` + `active` inteiros por passo (125 MB a 4K) — falta uma redução na placa.
   Usa o `uv` já em f32 (a CPU usa o f64 antes de arredondar; só importa no limiar 0,5).
3. **Faixas vivas**: a placa não as tem; a secagem corre na caixa inteira (`faixas_da_caixa`). Se uma
   célula fora das faixas mas dentro da caixa mudar na `dry_cell`, diverge — o gate (2) dirá.
   Depois do `desce`, `row_lo/row_hi` ficam os da CPU de antes (o composite pode lê-los).
4. A secagem da placa NÃO escreve `wet` (o `SecaGpu` não tem o buffer) — conferir se a `dry_cell` da CPU escreve.
5. `sim.paredes` vem do último passo da CPU (`paredes_valem` é privado); `agua_nova` não é zerada (privado).
6. O worker RITMADO (`offthread_ritmo`, o dos gates) não tem a placa — o gate (2) precisa dela lá, ou
   chamar `PassoPlaca` direto sobre o `Engine` depois de um traço (mais simples, recomendado).

## Gates por correr

(1) `PH2D_WET_GPU` ausente/`0` ⇒ `ligada()` é falso e nada muda (por construção); correr por nome
`flow_bit_tests advect_jacobi_bits assenta_bit historia_cow por_quadro_tests`. (2) `#[ignore]` de placa:
aguada sem Glaze + 60 passos, cor final máx ≤ 1 · p99.9 ≤ 0,5. (3) ms do tick 4096×2560 placa × CPU.
