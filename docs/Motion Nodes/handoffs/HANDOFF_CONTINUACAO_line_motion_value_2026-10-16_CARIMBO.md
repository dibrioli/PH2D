# CONTINUAÇÃO — `line/motion-value`: o CARIMBO na placa (16/10) — `point_scale > 0` e pontos com `rot` deixam de ser recusas do Duplicator

> Continua o [LÍNGUA](HANDOFF_CONTINUACAO_line_motion_value_2026-10-15_LINGUA.md) (a fila dele vale para o que não é
> o aberto 1). Mecanismo, medição e tabela: [doc 120 §8.3.1](../120_ciclo_12_os_tectos_confortaveis.md).
> **Este ficheiro SUBSTITUI o LÍNGUA como fila viva.** Regras R1–R9 do `…_SO_O_ESTADO.md` valem.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, HEAD `a2d913a44b`, base = `main` `0aab2c1cb6` (não andou; rebase vazio). **40 commits por integrar** (`git rev-list --count main..HEAD`), 41 com o de docs deste handoff. **Linha NÃO integrada**; só integra por ordem do Enio.
- Esta onda, sobre `9b04d9891b`: `32782190a4` (feat: `point_scale > 0` e pontos com `rot` deixam de ser recusas do Duplicator) · `762c7a6df6` (test: a saída derivada tem o `size` e a `rot` dos PONTOS — `output_columns` público) · `898754bc80` (docs doc 120 §8.3.1) · `a2d913a44b` (fmt).
- **Superfície de colisão:** `crates/ph2d-node-motion-duplicator/src/kernel.rs` · `crates/ph2d-gpu-cook/src/plan_forma.rs` (+ `plan::output_columns` público) · `crates/ph2d-gpu-cook/tests/it/gpu_cpu_parity_duplicator.rs` · `crates/ph2d-app-motion/src/` (`motion_campos_probe.rs`, `motion_state_demo_carimbo_escala_tests.rs` novo, `motion_state_demo_rota_medida_tests.rs`, `motion_state_demo_announce.rs`, `motion_state_colisor_nome_demo.rs`) · tutoriais 03 e 10 (HTML + PDF) · doc 120 §8.3.1.
- **Um merge pode partir:** quem toque o kernel do Duplicator ou o `plan_forma.rs` (escrita condicional por porta); quem acrescente cena que dependa da recusa antiga (`point_scale <= 0`, pontos com `rot`) — o gate `as_recusas_do_carimbo_entregam_o_no_a_cpu` e `toda_frase_de_rota_confere_com_a_rota_medida` re-medem; quem mexa em anúncios/tutoriais com frase de rota (=111, =112, =137, tutoriais 03 e 10); quem amplie `output_columns`.

## O que mudou (não reconstruir)

- **Kernel** (`ph2d-node-motion-duplicator/src/kernel.rs`): variante escolhida por `variant_by_param` (`COM_ESCALA_DO_PONTO` quando `t > 0`, `SEM_ESCALA_DO_PONTO`); `size` = `apply_point_scale` da CPU em f32 na MESMA ordem; `size` e `rot` do PONTO são `SourceReadWriteExisting` na porta 1 (escritos sse os pontos os trazem; o gather leva os da forma); guarda `np > 0` antes de ler o ponto. **Só `point_scale = NaN` (param dirigido) continua recusa.**
- **Plano** (`ph2d-gpu-cook/src/plan_forma.rs`): escrita condicional numa porta ≠ 0 julga a presença na porta DELA. `ph2d_gpu_cook::plan::output_columns` é público (o gate sem placa o usa).
- **Frases de rota** pela rota medida (gate `toda_frase_de_rota_confere_com_a_rota_medida` verde): anúncios =111, =112, =137; tutoriais 03 e 10 (HTML + PDF por `scripts/tutorial-pdf.sh`). Dois controlos falsos que viraram verdade trocados: «O `Duplicator` corre no processador.» (=111) e «HIBRIDO: o `Falloff` corre no processador.» (=112). `the_focus_scene_builds_and_stays_on_the_device` (`motion_campos_probe.rs`): a =112 só deixa `source.shape` na CPU, carimbo na placa.
- **Sonda A×B** `sonda_o_carimbo_com_escala` (`motion_state_demo_carimbo_escala_tests.rs`): A = kernel registado sem a variante, com a recusa antiga, trocado no `m.registry` (sem dependência nova); env `PH2D_SONDA_ESCALAS=1,4,11`, `PH2D_SONDA_CENAS`, `PH2D_SONDA_BLOCO`, `PH2D_SONDA_RODADAS`. `monta_demo` em `motion_state_demo_rota_medida_tests.rs`.

## Resultado (tabela completa: [doc 120 §8.3.1](../120_ciclo_12_os_tectos_confortaveis.md))

- =132 ×11: 9,344 → 1,016 ms (**9×**) · =136 ×11: 2,733 → 0,478 ms (**5,7×**).
- **Custo fixo** +0,06 a +0,6 ms por quadro nas cenas de demo (ver aberto 1).
- K2 (declarado antes) disparou a ×1 e foi **recusado pelo dono (16/10) como régua do tecto** — mede o custo fixo do despacho. Está em ⛔ Recusas MEDIDAS do doc 120 §8.3.1; não reabrir.

## Gates e mutações

- `gpu_cpu_parity_duplicator.rs`: `a_escala_do_ponto_concorda_com_a_cpu` (barra `ULP_DO_SIZE = 2`, pior medido **1,67 ULP**; controlo `t=0` 0,80 ULP vindo do `motion.scale` a montante — FMA) · `a_rotacao_dos_pontos_concorda_com_a_cpu` (incl. sem orçamento) · `a_saida_derivada_do_carimbo_tem_o_size_e_a_rot_dos_pontos` (sem placa) · `as_recusas_do_carimbo_entregam_o_no_a_cpu`. Motivos `#[ignore]` no formato `placa:`.
- **Prova de mutação: 9/9 sangraram**, cada uma no gate previsto: M1 clamp `min` perdido · M2 `t` ignorado · M3 índice `dp_s`↔`dp_p` do size · M4 size da porta 1 sem escrita · M5 leitura incondicional da `rot` do ponto sem orçamento · M6 `rot` do ponto perdida · M7 `variant_by_param` fixo · M8 recusa antiga `point_scale <= 0` · M9 presença da escrita condicional na porta 0 em `plan_forma.rs`.

## Fecho

Verde em `898754bc80` (+ fmt `a2d913a44b`): `PH2D_SUITE=fecho` nextest-impacted **17 767/17 767** · `motion-gate-placa` **63/63** · paridade do carimbo na placa **4/4** · clippy na forma do `ship.sh` ✓ · check warnings=deny ✓ · fmt ✓ (após `a2d913a44b`).

## Aberto (ordem)

1. ⭐ **O CUSTO FIXO da rota híbrida — DIAGNÓSTICO MEDIDO** (`6cdc8d1895`; sonda `sonda_as_fases_do_quadro_hibrido` em `crates/ph2d-app-motion/src/motion_state_demo_carimbo_escala_tests.rs`; relógio `MotionState.fases` + `FasesDoCozimento` / `fases_do_ultimo_cozimento()` em `ph2d-gpu-cook`, sempre ligado, ~0,5 µs/quadro; load 2,6→2,4). Sintoma: +0,06–0,6 ms por quadro nas cenas pequenas; a =120 fica 3× mais lenta mesmo a ×11 (0,249 → 0,774 ms); =111/=112/=119/=125 mais lentas em todas as escalas.
   - **Hipótese «subárvore estática recozida/reenviada» REFUTADA:** os Shapes estáticos atravessam a costura a 0 B (reaproveitam o buffer), a marcha do prefixo fica em 2,7–15 µs, o plano refeito a cada quadro custa 2–11 µs (não é o culpado). Exceção: a =120 reenvia 3 fronteiras animadas (ossos, onda, pele) — 27 µs para 4,9 KB, ~9 µs fixos por envio.
   - **O custo é POR DESPACHO:** cada estágio na placa paga ~3,7 µs a montar na CPU com o grupo de ligação da cache (=120) e ~5,8 µs quando é criado (=132), mais ~4–5 µs no `encoder.finish()` por passe, e ~5–12 µs de espera na placa. Cada estágio abre o SEU `begin_compute_pass` (13 sítios no `ph2d-gpu-cook`, ex. `encode.rs:283`).
   - **Tabela B−A (µs):** =120 +485 (cook_many +268: finish 96, estágios 66, saídas 45, costuras 27, submit 15; espera +210; 18 despachos) · =112 +49 (3→6 despachos) · =125 +36 · =119 +182 (12→22) · =111 +51 · =132 ×1 +210 (2→24; marcha −122) · =136 ×1 +103 (1→12). A ×11 B ganha (=132 6,2 → 0,9 ms; =136 2,1 → 0,4 ms): A paga a marcha do carimbo na CPU (5,1 / 1,6 ms) e 4,2 / 2 MB de envio por quadro.
   - **Rota A NÃO é «toda-CPU»** exceto na =120: nas outras A também é híbrida, com o carimbo como fronteira.
   - **Surpresas:** (a) a cache de grupos de ligação NUNCA acerta na cadeia do carimbo (=132 B: 25 criados / 3 da cache por quadro; =136 B: 12 / 0; =120 B: 0 / 18) — causa por investigar (suspeita: buffers do pool trocam de identidade a cada quadro no `SourceRows`/gather); (b) o `finish()` pesa como montar os estágios e não acompanha o nº de despachos (=111 A: 98 µs com 5); (c) a espera da placa é ~40 % do quadro de B a ×1; (d) o preparo do `coze` custa 25–50 µs na =120/=119 nas DUAS rotas.
   - **CURAS candidatas, a MEDIR como variantes em execução no mesmo processo ([MEDIR_VELOCIDADE](../../DevOps/MEDIR_VELOCIDADE.md)), por ordem de custo:** (1) a cache de grupos de ligação acertar na cadeia do carimbo (identidade estável dos buffers por estágio); (2) UM `ComputePass` para os estágios consecutivos em vez de um por estágio (o wgpu sincroniza o uso entre despachos dentro do passe; o `pass_profiler` precisa de rótulos por despacho); (3) fusão de kernels consecutivos por elemento (a mais cara — o `wgsl_lib` já prevê «a future fused module»). Kill-criterion por declarar ANTES do build. ⛔ Leia antes o [doc 131 C.7](../131_zerar_o_ultimo_bloco.md) (linhas 166–209): a cache de ligações e uniformes do passe de GRUPO de formas já foi recusada (−0,03 ms) — é outro passe, mas é a mesma ideia.
2. =139 continua com a simulação na CPU por outra fronteira: `Integrate`.
3. =137 a ×11: 1,44 s por quadro nas DUAS rotas — outro gargalo, por achar (×4 já 40–45 ms).
4. Outras fronteiras, outro tecto (não o carimbo): =98 `transfer` = 1 e 3 · =134 Time Remap ⇒ cena toda na CPU · =51 Boids e Soft Body · =131 Verlet Rope · =120/=125 Bones/Skin · =119 Table/Text/Camera.
5. `size` Scalar nas duas portas: a CPU (`size_at`) aceita-o, todo kernel da família liga `size` como Vec2 e nenhum confere a dimensão na placa — convenção da família, não medida nesta onda.
6. Os abertos 2–8 do [LÍNGUA](HANDOFF_CONTINUACAO_line_motion_value_2026-10-15_LINGUA.md#aberto-ordem) continuam (por referência; o aberto 1 dele — o tecto do carimbo — está FECHADO aqui).

FECHADO nesta onda: o tecto `point_scale > 0` / `rot` do ponto (ex-aberto 1 do LÍNGUA).

## Smoke ao dono

Nenhum ainda, de propósito: no tamanho das cenas a IMAGEM é a mesma nas duas rotas (a paridade é o
gate) e o ganho só aparece com o pano a escalar (doc 120 §8.3.1). O que muda e se vê é a linha
`[motion-route]` do terminal — já conferida pelo `toda_frase_de_rota_confere_com_a_rota_medida`. Um
smoke ao dono desta onda pede uma cena com MUITAS estrelas (ex.: a =132 com o `Grid` a 176²),
fotografada por `docs/Components/ferramentas/fotografa_cena.sh` antes de ir ao Enio (CLAUDE.md §0.8).
