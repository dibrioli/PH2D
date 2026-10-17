# CONTINUAÇÃO — `line/motion-value`: o custo fixo da rota híbrida (16/10) — UM passe de computação para os despachos consecutivos

> Continua o [CARIMBO](HANDOFF_CONTINUACAO_line_motion_value_2026-10-16_CARIMBO.md) (cujo aberto 1 fecha aqui;
> os abertos 2–6 dele seguem abaixo). Mecanismo, tabelas e recusas: [doc 120 §8.3.2](../120_ciclo_12_os_tectos_confortaveis.md).
> **Este ficheiro SUBSTITUI o CARIMBO como fila viva.** Regras R1–R9 do `…_SO_O_ESTADO.md` valem.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, base = `main` `0aab2c1cb6` (não andou; rebase vazio). **Linha NÃO integrada**; só integra por ordem do Enio.
- Esta onda, sobre `5b4e570f60`: `09af314f76` (as três curas como variantes em execução + a sonda) · `8901099e16` (o passe único vira a lei; curas 1a/1b retiradas; o gate) · `c2d8fb243e` (o gate cobre a leitura dos cartões e vários cozimentos por quadro; o controlo de leitura da varredura de placa sem relógio; `lib.rs` abaixo do tecto) · docs deste handoff.
- **Superfície de colisão:** `crates/ph2d-gpu-cook/src/` — `gravador.rs` NOVO; ~20 ficheiros trocaram `encoder: &mut wgpu::CommandEncoder` por `&mut crate::gravador::Gravador` e `let mut pass = encoder.begin_compute_pass(` por `let pass = encoder.passe(` (`encode`, `formas`, `instances`, `stream_op*`, `reduce_stage`, `saidas`, `tap`, `tap_voo`, `voronoi`); `lib.rs` (o `cook_many` cria o `Gravador`, conta `passes`, grupos do stream-op e buffers alocados); `fases.rs` (3 campos novos em `FasesDoCozimento` + `um_passe_por_despacho`); `estado.rs` (`passe_por_despacho`). `crates/ph2d-app-motion/src/`: `motion_state_demo_curas_custo_fixo_tests.rs` NOVO, `motion_state_demo_carimbo_escala_tests.rs` (helpers `pub(super)`), `motion_state_demo_rota_anunciada_tests.rs` (o `mod`). Doc 120 §8.3.1 (uma frase) e §8.3.2. `project-memory/reference_topic_gate_discipline.md` (+1) e o contador no `MEMORY.md`.
- **Um merge pode partir:** quem acrescente um sítio de despacho no `ph2d-gpu-cook` com `encoder.begin_compute_pass` directo sobre o `Gravador` (compila pelo `DerefMut`, que FECHA o passe — funciona, mas parte o passe único: use `encoder.passe(&desc)`); quem mude a assinatura de uma função interna que recebe o codificador; quem construa `FasesDoCozimento` por literal (três campos novos).

## O que mudou (não reconstruir)

- **`Gravador`** ([`gravador.rs`](../../../crates/ph2d-gpu-cook/src/gravador.rs)): o codificador do cozimento com o passe aberto. `passe(&desc)` reaproveita o passe aberto (ou abre um, sempre, com `um_passe = false`); o `DerefMut` fecha-o antes de cópia/limpeza; `finish` fecha-o. O campo `aberto` vem ANTES do `encoder` (ordem de `drop`).
- Ficam com passe próprio, de propósito (API pública com codificador cru, usada pelos testes): `scan`, `reduce`, `grid`, `voronoi::*` públicos, `dono`, `passe_na_placa`. O `Gravador` cede-lhes o codificador já fechado (coerção pelo `DerefMut`).
- **Lei por omissão = um passe**; `GpuCook::um_passe_por_despacho(true)` (doc-hidden) devolve o codificador de antes pelo MESMO código — só a sonda e o gate.
- `FasesDoCozimento`: `passes`, `grupos_de_stream_op_criados/_reaproveitados`, `buffers_alocados`.
- Sonda `sonda_as_curas_do_custo_fixo`: A · `B·antes` · `B·antes·gémeo` (controlo da régua e da paridade) · B, blocos rodados, mínimo + mediana, saída contra `B·antes` ao bit; envs `PH2D_SONDA_CENAS`, `PH2D_SONDA_BLOCO`, `PH2D_SONDA_RODADAS`. Correr com `__GL_SHADER_DISK_CACHE=0 PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke --lib sonda_as_curas_do_custo_fixo -- --ignored --nocapture --test-threads=1` (21 s).

## Resultado (tabela completa: [doc 120 §8.3.2](../120_ciclo_12_os_tectos_confortaveis.md))

- Quadro inteiro, µs, `B·antes → B`: =120 650 → 458 · =132 666 → 485 · =119 771 → 472 · =136 383 → 243 · =112 229 → 154 · =125 238 → 173 · =111 647 → 488 · =132 ×11 1 121 → 750 · =136 ×11 579 → 343. Saída ao bit em todas.
- Cada passe custava ~3,5 µs de espera na placa + ~2–3 µs de `finish`. Agora a híbrida empata ou ganha à rota A em =119/=112/=125/=111. Sobram três lacunas: =132 +48 µs, =136 +36 µs e =120 +325 µs (a A da =120 é toda CPU).
- **Recusadas pelo kill-criterion declarado antes** (⛔ Recusas MEDIDAS do §8.3.2):
  - cura 1 (LUT por conteúdo + pool por ordem; código em `09af314f76`): 3–9 % da lacuna;
  - cura 3 (fusão): não construída, porque o resto por despacho é 2–3 µs, abaixo dos 5 µs.

## Gates e mutações

- `um_passe_da_a_mesma_saida_com_menos_passes` (`#[ignore = "placa: …"]`): =132/=120/=119/=136/=112 × escalas 1 e 4, 40 quadros, saída ao bit (instâncias + formas), `hoje × 3 ≤ antes` em passes, **zero erros de validação num `push_error_scope` próprio**.
- **Prova de mutação 4/4 sangraram:**
  - M1 `passe()` abre sempre → passes 36 = 36;
  - M2 `DerefMut` sem fechar → `Validation Error`;
  - M3 lei invertida no `cook_many` → 36 contra 6;
  - M4 `finish` sem fechar → `Validation Error`.
  
  ⛔ Sem o âmbito de erros, M2 e M4 SOBREVIVIAM: o produto só escreve os erros do dispositivo no terminal, e os dois lados passam pelo mesmo `Gravador` (lição em `reference_topic_gate_discipline`).

## Fecho

Verde em `c2d8fb243e`:
- `PH2D_SUITE=fecho` nextest-impacted: **17 765/17 767**. Os 2 vermelhos são relógio do `ph2d-tool-painter`
  (crate sem diff nesta linha), a load 66: `the_mask_stroke_cost_does_not_follow_the_canvas` e
  `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs`. Os dois passaram 3/3 sozinhos a
  load 44–48, portanto são flakes de carga e não estão na tabela do `FLAKES_DE_CARGA.md`.
- `motion-gate-placa` **64/64** (o 64.º é o gate novo).
- Paridade GPU/CPU do `ph2d-gpu-cook` com `--ignored` **189/189**.
- Tecto de LOC ✓ (`lib.rs` 695).
- Clippy na forma do `ship.sh` ✓ · check `--workspace --all-targets` warnings=deny ✓ · fmt ✓.

⛔ **O 1.º fecho, sobre `8901099e16`, apanhou três coisas:**
- **(a)** o tecto de LOC: `lib.rs` 703 > 700. Cura: `conta_o_cozimento` passou para o `fases.rs`.
- **(b)** o fmt.
- **(c)** `toda_cena_de_ciclo_coze_pela_placa`: «leitura nunca chegou» na =138/=139.
  - Não era do passe único. Com a lei invertida no MESMO binário, o passe por despacho reprovou por igual:
    3/3 de cada lado, a load ~20.
  - O laço apertado exigia que a leitura em voo caísse entre dois quadros.
  - Cura: se a cena correu na placa e não leu, o gate espera UMA vez pela placa e recolhe. 3/3 a load 15.
  - Ao investigar, o gate do passe único mostrou que não pedia a leitura dos cartões. Passou a pedi-la,
    como o `dispatch` faz.

Mutação refeita sobre o gate final: 4/4 (a =138 sangra primeiro).
`target/*/incremental` reclamado (6,9 GB). Binário do smoke (`cargo build -p ph2d-host-desktop --profile
smoke`, 2.ª corrida): `Finished `smoke` profile [optimized] target(s) in 0.21s` — zero `Compiling`.

## Aberto (ordem)

1. **O que sobra do custo fixo, por tipo:**
   - =120: ir à placa custa +325 µs contra a rota toda-CPU. É uma DECISÃO DE ROTA (cruzamento medido por tamanho), não custo de despacho. Só com régua por cena.
   - =132 e =136: sobram +48/+36 µs. O pior ainda é a ligação guardada que não acerta (=132: 25 criados por quadro). A cura 1 foi recusada pelo número; não reabrir sem medir de novo.
   - =111 (24 → 9 passes) e as outras com mais de 2 passes: o passe parte-se em cada sítio que o fecha (cópia ou limpeza no codificador — o `v[0]` do condutor, concat/project, a grelha —, a compactação com submit próprio, e o `scan`/`reduce`/`grid` com passe próprio). Ainda não está medido qual pesa em cada cena: conte os fechos por sítio antes de propor outra organização.
2. =139 continua com a simulação na CPU por outra fronteira: `Integrate`.
3. =137 a ×11: 1,44 s por quadro nas DUAS rotas — outro gargalo, por achar.
4. Outras fronteiras, outro tecto: =98 `transfer` = 1 e 3 · =134 Time Remap ⇒ cena toda na CPU · =51 Boids e Soft Body · =131 Verlet Rope · =120/=125 Bones/Skin · =119 Table/Text/Camera.
5. `size` Scalar nas duas portas (convenção da família, não medida).
6. O cabeçalho do [`handoffs/README.md`](README.md) diz «90 handoffs»: a pasta tem 113 `HANDOFF_*.md` e a
   tabela tem 80 linhas (contado a 16/10, já antes desta onda). É um número que se CONTA; nenhum gate o vê.
7. Os abertos 2–8 do [LÍNGUA](HANDOFF_CONTINUACAO_line_motion_value_2026-10-15_LINGUA.md#aberto-ordem) continuam.

FECHADO nesta onda: o aberto 1 do CARIMBO (o custo fixo da rota híbrida era o passe por despacho).

## Smoke ao dono

Nenhum ainda: a imagem é a mesma (o gate prova-o ao bit) e o ganho é de 0,07–0,37 ms por quadro. Isso
fica invisível a 60 fps nas cenas de demo. O smoke desta onda é o da onda do carimbo: a =132 com MUITAS
estrelas, fotografada antes por `docs/Components/ferramentas/fotografa_cena.sh`.
