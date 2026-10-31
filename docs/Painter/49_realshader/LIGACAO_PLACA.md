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

## Ronda 2 (31/10) — gates corridos, defeitos 1–4, medida

**Gate (1) neutro** (`PH2D_WET_GPU` ausente, por nome): `ph2d-wet-paint` — `flow_bit_tests` (2) · `advect_jacobi_bits` ·
`assenta_bit` (2) · `historia_cow` (4) · `a_borda_graduada` (it, 2): verdes. `ph2d-tool-painter` — `undo_tinta_viva`,
`redo_nao_mata_a_tinta_viva`, `oleo_por_traco`: 9 verdes, 1 vermelho = `o_traco_seguinte_usa_o_depth_o_gloss_e_a_caixa_novos`,
**vermelho ANTERIOR** (já listado em `HANDOFF_REBELLE_MEDIDO.md` e `R3_PINTAR_RELATORIO.md`; sem `ligada()` nada da placa corre).
`por_quadro_tests` não casou nenhum teste em `ph2d-wet-paint`.

**Gate (2)** `crates/ph2d-wet-gpu/tests/passo.rs::o_passo_placa_sobre_o_engine_casa_com_a_cpu_na_cor` (`#[ignore]`):
`Engine` 512×320, três traços de aguada (Cobre, sem Glaze), `PassoPlaca` sobre o próprio Engine × `sim_step` numa cópia,
60 passos, cor sobre branco + **devolução** (`desce` e +12 passos de CPU dos dois lados). Afirma relógio, `dry_every`
(o vmax), máx ≤ 1 e p99.9 ≤ 0,5 níveis.

| placa | máx | p99.9 | Δ>0,5 | devolvido +12 |
|---|---|---|---|---|
| Radeon 610M (Granite Ridge) RADV (Vulkan) | 0,4384 | 0,0002 | 0 | 0,4384 / 0,0002 |
| Radeon radeonsi (GL) | 0,4366 | 0,0001 | 0 | — |
| **RTX 5060 Ti** | **não medido**: o Vulkan da NVIDIA não inicializa nesta sessão (`vulkaninfo`: `ERROR_INITIALIZATION_FAILED`, `nvidia-smi` responde) — correr de novo quando o driver voltar | | | |

⚠️ **Achado que manda na ligação**: `placa_sabe` exige `!p.paredes`, e o `Engine` liga `sim.paredes = paredes_valem()` =
**`true` com o pincel normal** (só Blow/Wet/Smear/Blend o desligam). O gate força `e.sim.paredes = false`. Ou seja: com
`PH2D_WET_GPU=1` o worker **só** vai à placa nas ferramentas que movem tinta; no traço de pincel corre a CPU. Para a placa
servir o produto falta a parede entre traços na placa (`solver::parede_do_traco`) — é o próximo degrau, não um defeito do gate.

**Defeitos**:
1. ✅ Secagem sem ida à CPU: `separa`/`junta` (compute, `st` ⇄ `film`/`susp` do `SecaGpu`) + cópias de `rgb`/`sett`, tudo num
   codificador (`passo.rs::seca_na_placa`). Gate igual ao dígito.
2. ✅ `vmax` reduzido na placa: passada `vmax_red` (`atomicMax` dos bits f32 ≥ 0 em `caixa[15]`, `PLANO_VMAX`), 64 B lidos por
   passo em vez de `uv`+`active` inteiros. `dry_every` igual ao da CPU no gate.
3. ✅ Faixas vivas depois do `desce`: `row_lo = 0`, `row_hi = s−1` (superconjunto declarado; o próximo rebuild aperta). A
   secagem na caixa inteira não divergiu no gate (máx 0,44).
4. ✅ Lido: a `drying.rs` da CPU **não** escreve `wet` — não há o que copiar; `wet ≠ 0` células no gate.
5. Aberto (ver o achado acima). 6. Contornado (o gate chama `PassoPlaca` direto, como recomendado).

**Medida (4)** `tests/passo.rs::mede_o_passo_placa_a_4k` (`#[ignore]`, 4096×2560, aguada de 3 traços, caixa
3340×900, mínimo de 3 rondas de 12 passos, ⚠️ loadavg ~15 — relógio de CPU pouco fiável):

| | placa (Radeon 610M (Granite Ridge)) | CPU (`sim_step`) |
|---|---|---|
| ms/passo, antes dos defeitos 1+2 | 100,2 | 10,1 |
| ms/passo, depois | **80,0** | **10,6** |
| subir a grade (por traço) | 200 ms | — |
| descer a grade (por traço) | 242 ms | — |

A CPU anda nas faixas vivas (as bandas finas dos traços); a placa anda a caixa inteira e as passadas do rebuild/bordas na
folha inteira, com 2–3 esperas por passo (`step_plano` + `readback_caixa` + `readback_vmax`). RTX por medir.

**Recomendação (o coordenador decide)**: **NÃO inverter a omissão.** (a) a RTX não foi medida; (b) na Radeon a placa é
~7,5× MAIS LENTA que a CPU nesta cena e o subir+descer custa ~0,45 s por traço — uma pausa visível ao pousar e ao levantar;
(c) com o pincel normal a placa nem corre (paredes). Antes de pensar na omissão: paredes na placa; subir/descer só a caixa
suja (não a grade inteira); juntar as esperas do passo num único submit com a leitura da caixa e do vmax num só mapa;
e medir na RTX numa cena de aguada que encha a folha.

## Ronda 3 (31/10) — paredes na placa, só as linhas sujas, uma espera por passo

Commits `2fe3400624` (paredes) · `0f3fa93d0c` (uma espera) · `c2460bc455` (linhas sujas). Só a Radeon 610M (RADV):
a RTX continua sem Vulkan (`VK_ICD_FILENAMES=…/nvidia_icd.json vulkaninfo` → `ERROR_INITIALIZATION_FAILED`).

1. **Paredes** (defeito 5 fechado): `advect.wgsl` lê `traco` (binding 18) e `K.isola` (= `p.paredes && !traco.is_empty()`);
   `parede()` = `solver::parede_do_traco` no `campo` (nivelamento + escrita final) e na `suaviza` (escrita final).
   `placa_sabe` já não exige `!paredes` ⇒ o pincel normal vai à placa com `PH2D_WET_GPU=1`.
   ⭐ **Mecanismo** da 1.ª divergência (máx 9,5 níveis): o retro-traço com `d = −1e-10` arredondava a fração a `1` em f32
   e o canto de baixo perdia o peso `1e-10` que a CPU (f64) lhe dá ⇒ resíduo `susp > 0` na CPU, `0` na placa ⇒ a parede
   (`susp > 0` dos dois lados) liga/desliga. Cura: `gx = (fl+1) − d` (complemento exacto), sem o embrulho `f ≥ 1 ⇒ base+1`.
   Gate `o_passo_placa_com_paredes_casa_com_a_cpu_na_cor` (6 528 células com vizinho separado, 60 passos):
   **máx 0,4009 · p99.9 0,0002 · Δ>0,5 = 0**, devolvido +12 igual. Os gates antigos (`passo`, `paridade`) verdes.
2. **Só as linhas sujas**: espelho por **hash de linha** (fina e de fluxo) da CPU quando placa = CPU. `sobe_sujo` sobe a faixa
   cujo hash mudou (dab, undo, troca de camada — sem depender de marcas de dirty); `desce_suja` desce a união das caixas desde
   a subida + 2 linhas de borda de cada lado. O worker usa os dois. Gate `a_subida_e_a_descida_so_das_linhas_sujas_casam_com_a_cpu`
   (paredes, traço novo na CPU entre duas tandas de 30 passos, faixa 185..263 de 322): máx 0,4064 · p99.9 0,0002.
   ⚠️ Faixa = linhas inteiras (não o rectângulo): uma escrita por buffer. A `momentum` da placa zera `vel` nas células inactivas
   da folha toda e essas linhas fora da faixa não descem — a CPU fica com a `vel` antiga onde não há água (igual ao que a placa
   já fazia; o gate devolvido não o vê).
3. **Uma espera por passo**: o rebuild e o resto num submit; a passada `porta` escreve os grupos indirectos (os da CPU, ou 0 se
   `caixa[14] = 0`); caixa + `vmax` numa cópia de 64 B. Excepção: passo par COM secagem (`n % 6`) espera 2× (a secagem precisa
   da caixa na CPU para as faixas e os grupos — próxima cura: faixas/grupos da secagem também por indirecto).

**Medida** (`mede_o_passo_placa_a_4k`, 4096×2560, caixa 3340×900, mínimo de 3, loadavg 9–16):

| | antes (ronda 2 + paredes) | depois |
|---|---|---|
| placa ms/passo | 73,9 | 78,6 (CPU no mesmo processo 8,7 → 6,7: ruído de carga; a espera não era o gargalo na 610M) |
| subir por traço | 205,6 ms (grade) | **17,9 ms** (traço de 64 linhas; o grosso é o hash paralelo das 2 560 linhas) |
| descer por traço | 234–288 ms (grade) | **109 ms** (900 linhas da caixa, 11 leituras × 3 faixas, uma espera cada) |

**Recomendação**: continua **NÃO inverter a omissão** — na 610M a placa anda ~10× mais lenta que a CPU por passo (o
cálculo, não a espera). Falta: medir na RTX (degrau 3: 6,6 ms o quadro de campo 4K); a descida num codificador só (uma espera
em vez de 33); a secagem por indirecto.
