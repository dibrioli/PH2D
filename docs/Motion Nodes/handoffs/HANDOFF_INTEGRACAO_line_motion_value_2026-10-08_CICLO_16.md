# HANDOFF de integração — `line/motion-value`, CICLO 16 (2026-10-08)

> Ordem do dono de 08/10, depois do smoke da `=129` aprovado: *«vamos corrigir tudo»* — os cinco
> abertos do [handoff de continuação](HANDOFF_CONTINUACAO_line_motion_value_2026-10-08_CICLO_16.md).
> Doc do ciclo: [125](../125_ciclo_16_a_corda_que_nao_se_atravessa_e_o_collide_que_diz_o_que_fazer.md)
> — estado medido, desenho e kill-criteria, o oráculo, a rodada de medição, as provas de mutação. A
> linha NÃO integra e NÃO pusha: integrar é por ordem do dono.

## §1 — Base e commits

- Base: `main` `2a925859d` (o merge-base; o `main` não andou). A linha leva os ciclos 13, 14 e 15
  (handoffs [CICLO_13](HANDOFF_INTEGRACAO_line_motion_value_2026-10-07_CICLO_13.md),
  [CICLO_14](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_14.md),
  [CICLO_15](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_15.md), por integrar) e, por cima, os
  commits do ciclo 16 (`git log --oneline 2a925859d..line/motion-value`).

## §2 — O que fechou

| # | item | veredito | gate / prova |
|---|---|---|---|
| 1 | **Bug #16** — a corda com colisor atravessa-se num chicote | ✅ a corda ganhou a porta `shape` (`Shape → Verlet Rope`, peças em `Cycle`) e LÊ o colisor que o `Collide` do Shape declara, com o contacto DENTRO das iterações, o passo seguro CFL (meio alcance) e a âncora a andar ao longo dos sub-passos; o relógio do grafo continua intocado. Oráculo Godot (uma corrente, `godot_oracle_corrente/`): `0,9957·D` | `the_rope_reads_its_pieces_collider_and_does_not_cross` (`0,0138 → 0,9598`, comprimento `1,04×`, lê a `saida.txt`) · `the_left_rope_crosses_and_the_right_one_holds` · `turning_on_the_left_shapes_collide_makes_the_rope_hold` · `todo_leitor_do_colisor_declarado_se_regista` (a corda na tabela) · cena `=130` |
| 2 | o `Collide` do Shape DEPOIS do corpo (o botão mudo) | ✅ o grafo diz o que fazer: `Deficit::ColisorMudo("shape"|"output"|"collide")` no diagnóstico, a frase no cartão; o registo ganhou quem DECLARA (`register_collider_switch`) e quem ARMA o passe (`register_contact_pass_switch`). ⭐ E o passe do fim deixou de inchar um corpo de peças (lê `rest`/`body`) | `every_collide_alone_has_an_effect_or_an_advice` (cinco montagens, pela bomba) · `a_working_collide_is_never_advised` · `the_shape_after_the_rope_advises_and_rewiring_it_holds` · `the_end_pass_does_not_swell_a_body_of_pieces` · `no_conference_scene_ships_a_setup_hole` (a lista `ENSINAM_A_SEGUNDA_CHAVE`) · cena `=131` |
| 3 | o `scatter` com o `count` por fio | ✅ o nó é LINEAR desde a `NearGrid` (`23,8 ms` a `32 768`, release) — o «quadrático» e o tecto de `3 000` eram de antes dela; o digitável passou ao da casa. A família medida: `34` params de contagem a `1e9` por fio, só o `motion.voronoi` (a CPU de referência de um nó de placa, já isento) passa de `0,11 s` | `the_typed_ceiling_is_where_the_eval_clamps` · `the_grid_keeps_the_work_per_point_flat` (distâncias contadas) · `the_scatter_ceiling_is_the_house_ceiling_now_that_it_is_linear` (era `the_quadratic_node_keeps_a_tight_ceiling`) · `sonda_cada_contagem_a_1e9_por_fio` |
| 4 | o SIGSEGV do `ph2d-render` | ✅ NÃO reproduz: `0/32` (binário inteiro e `nextest`, faixa normal e de placa) | doc 125 §4 |
| 5 | a sonda `FIELD_DOC_VERSION` | ✅ já fechada no `main` (`69b28daa5`); as referências dos handoffs 13–15 fecham | doc 125 §5 |
| — | ⭐ achado pela FOTO do smoke: a ficha da casa cortava as últimas letras de TODA ficha | ✅ `paint_chip` media-se com uma folga e pintava com o orçamento de um rótulo; agora pinta com o da mesma folga (nenhuma ficha muda de tamanho) | `a_chip_paints_the_whole_text_it_measured` (doc 125 §8.1) |

## §3 — Superfície de colisão (para o integrador)

`collision-surface.sh`: `PROJECT_SCHEMA 187` (base `187`) · registos de componentes iguais · contratos
congelados INTOCADOS · nenhum ADR · nenhum pacote externo novo no `Cargo.lock` (a aresta interna
`verlet-rope → ph2d-contact`).

- **`ph2d-node-registry`** (foundational, append-only): dois campos novos (`collider_switches`,
  `contact_pass_switches`) e o módulo irmão `colisor_declarado.rs` com os seis acessores (os dois do
  leitor MUDARAM-SE do `lib.rs` para lá, pelo tecto de LOC — a API pública não muda).
- **`ph2d-editor-core`** (foundational, UI): `readout.rs` — `paint_chip` pinta com
  `chip_text_budget` (novo, público) em vez do `label_budget`; os consumidores (`ph2d-app-vec`
  `snap_labels.rs`, as leituras do `hero/paint_canvas_overlays.rs`, a legenda das cenas na shell)
  passam a mostrar o texto inteiro, no MESMO tamanho de ficha. Gate em `readout_tests.rs`. ⚠️ Linha
  UI/UX: é a ficha de vocês — a mudança é a do orçamento do texto, mais nada.
- **`ph2d-contact`** (motor partilhado): `passe.rs` lê `rest`/`body` (`Pecas::com_ligados`); sem as
  duas colunas, byte-idêntico.
- **`ph2d-motion-diagnose`**: `Deficit::ColisorMudo` (+3 em `Deficit::ALL`), o módulo `colisor.rs`, a
  chamada no `diagnose` (⚠️ ela entra em `diagnose_setup` — um `Collide` mudo é buraco de setup).
- **Nós:** `ph2d-node-motion-verlet-rope` (a porta `shape` APENDADA — `anchor_x`/`anchor_y`/`state`
  ficam em `0/1/2`; `contacto.rs` novo; dependência nova `ph2d-contact`; `register_declared_collider_reader`;
  `step`/`simulate` com o argumento `contacto`; sondas `contacto_probe.rs`) · `ph2d-node-motion-shape`
  e `ph2d-node-motion-output` (uma linha de registo cada) · `ph2d-node-motion-scatter` (o
  `PARAM_HARD_MAX` e dois gates).
- **`ph2d-app-motion/src`:** cenas novas `motion_state_corda_com_colisor_demo.rs` (+`_tests`) e o censo
  `motion_colisor_botoes_tests.rs`; `mod` em `motion_state.rs`; `"130"`/`"131"` na tabela das cenas de
  ciclo (`motion_state_demo_router_ciclos.rs`) e `MAX_DEMO_LEVEL 129 → 131` (⚠️ outra linha que
  acrescente uma cena reconta o número); `motion_bridge_heal.rs` (três braços do `explain`) e `motion_bridge_heal_advisory_tests.rs` (o gate do
  selo da `=131`);
  `motion_state_demo_router_tests.rs` (`ENSINAM_A_SEGUNDA_CHAVE`); `motion_bridge_gpu_collider_tests.rs`
  (a corda na tabela); `motion_bridge_range_tests.rs` (o gate do `scatter` reescrito);
  `motion_count_ceiling_tests.rs`; `motion_tecto_de_instancias.rs` (a nota da atribuição) e a sonda
  `motion_tecto_por_fio_probe.rs`.
- **`ph2d-i18n`:** três frases do `app_motion.rs`.
- **Docs:** doc 125 novo, nota no doc 103 §5, Bug #16 fechado no BUGS, `README.md` do módulo
  (regenerado), tutoriais `09` (src + PDF) e `01` (a figura `fig/params.html` + PDF), o oráculo
  `ferramentas/godot_oracle_corrente/`, os handoffs 13–15 (as referências fechadas), memória (cinco
  famílias), `CLAUDE.md` §5.1.

## §4 — Gate batched (sobre `2a925859d..` a árvore)

Corrido DUAS vezes: a 1.ª sobre o diff do ciclo, a 2.ª sobre a árvore FINAL (depois da receita da cena,
dos gates do selo e da cura da ficha da casa).

| passo | 1.ª corrida | árvore final | carga |
|---|---|---|---|
| `nextest-impacted.sh` (`BASE=2a925859d`) | ✅ `18593/18593` | `18594/18595` — o único vermelho é `the_cost_of_sampling_a_path_is_flat_in_its_anchors` (`ph2d-timeline`, intocado; uma razão de relógios, membro REGISTADO da família de flakes de carga) a carga `20`–`60`, verde sozinho | `4` · `20` |
| `RUSTFLAGS=-D warnings cargo check --all-targets` (as crates tocadas, + `ph2d-editor-core`/`ph2d-app-vec`) | ✅ | ✅ | `24` · `48` |
| `cargo clippy --all-targets -D warnings` (as mesmas) | ✅ | ✅ | `20` · `35` |
| `cargo fmt --all --check` | ✅ | ✅ | — |
| `censos-da-arvore-combinada.sh` | ✅ `114/114`, `12/12` | ✅ `114/114`, `12/12` | — |
| `doc-index.sh --check` | ✅ `17` | ✅ `17` | — |
| placa `scripts/motion-gate-placa.sh` | ✅ `30/30` | ✅ `30/30` | `18` · `42` |
| `collision-surface.sh` (tectos de LOC) | ✅ (depois do corte da corda `731 → 697`, o `sub_passos` e a âncora para o `contacto.rs`) | ✅ | — |

## §5 — Provas de mutação

`12/12` sangram — a tabela está no [doc 125 §6](../125_ciclo_16_a_corda_que_nao_se_atravessa_e_o_collide_que_diz_o_que_fazer.md)
(pré-voo das âncoras, alvos verdes antes, restauro ao bit com `touch`, corrida limpa no fim).

## §6 — Conhecidos que NÃO são deste ciclo

- `ph2d-render`, faixa de placa (`--ignored`): os goldens `w2`–`w5` do `smoke_fixture_renderable` são
  `unimplemented!()` por desenho (as notas deles dizem que os PNG não existem), e o
  `the_pointwise_op_costs_much_less_than_a_blur` reprovou `1/8` a carga `28` (uma razão de tempos).
- O `motion.voronoi` na CPU de referência custa `~60 s` em debug a `32 768` (isento por medição no censo:
  quem manda no tecto dele é a placa).

## §7 — O que fica ABERTO

Nada dos cinco itens. Os conhecidos fora do módulo estão no §6.

## §8 — Smoke e binário

✅ **Smoke do dono APROVADO (2026-10-09)** — as cenas `=130` e `=131`.

Smoke ao dono (em `--release`): a cena `=130` (as duas cordas sacudidas de lado, sem gravidade; a única
diferença é o `Collide` do Shape ligado à porta `shape`; o gesto é ligar o da esquerda) e a `=131` (o
Shape depois da corda: o selo ⚠ no cartão, a frase, e o fio que o cala), fotografadas aos `9`/`12 s` e
`6 s` (`target/prova/c16/fotos/final_*`), e o tutorial `09` (as duas notas novas, PDF refeito; o `01`
refeito pela figura do `scatter`). Binário `ph2d-host-desktop --release`: a 2.ª compilação `0`
`Compiling`, `Finished … in 0.24s`.
