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

1. ⭐ **O CUSTO FIXO da rota híbrida:** +0,06–0,6 ms por quadro nas cenas pequenas; a =120 fica 3× mais lenta mesmo a ×11 (0,249 → 0,774 ms) por ter muitas costuras CPU→placa (Shape, Bones, Skin, Wave); =111/=112/=119/=125 também mais lentas em todas as escalas medidas. Hipótese POR MEDIR: subárvores ESTÁTICAS (Shape, Grid) recozidas/reenviadas a cada quadro — mesma família do aberto 2 do LÍNGUA (=116, ~0,12 ms). Meça antes de desenhar.
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
