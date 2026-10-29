# Plano — partir `ph2d-app-motion` em crates por família (item M da comissão)

> Origem: [COMISSAO_VELOCIDADE_2026-10-29.md](COMISSAO_VELOCIDADE_2026-10-29.md) item M. Leitor: a LLM que
> executa. Molde e armadilhas: [HOWTO_partir_uma_familia_da_shell.md](../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md)
> — **leia a §2 inteira (§2.1–§2.21) antes do primeiro ficheiro**. Catraca: `architecture_the_shell_only_shrinks.rs`,
> teste `the_shell_only_shrinks_and_every_app_crate_too` (tabela `TETOS_DAS_CRATES_APP`).

## 1. O que foi medido (29/10, `a0fcfc1f19`)

| grandeza | valor | como |
|---|---|---|
| linhas `.rs` da crate (src + tests) | **193 644** | a `conta()` da catraca |
| ficheiros `.rs` | 733 em `src` (planos) + 5 em `tests` | `find -name '*.rs'` |
| `lib.rs` | 474 linhas | `wc -l` |
| `#[path]` | 592 | `grep -rh '^\s*#\[path' src` |
| ficheiros com `#[cfg(test)]` | 222 | `grep -rl` |
| tecto na catraca | 197 516 (medido + 2 %) | `TETOS_DAS_CRATES_APP` |

Linhas por prefixo (sem o `motion_`): `state_conferencia` 33 633 · `shape` 11 971 · `state_gpu` 9 633 ·
`state_demo` 9 063 · `macro_track` ~9 k · `bridge_gpu` 5 573 · `state_pilha` 5 180 · `object` 4 800 ·
`bridge_params` 3 770 · `carimbo` 3 208 · `lsystem` 3 144 · `demo` 2 757 · `bridge_objects` 2 564 ·
`state_rig` 2 441 · `path` ~1,8 k · `flip` ~1,3 k; o resto ~35 k. ⚠️ §2.15: o PREFIXO não é a família —
confira o grafo de `use` de cada ficheiro antes de o atribuir.

## 2. Por que a Onda 0 vem antes de mover

Há CICLOS no núcleo (conferido por grep): `motion_state.rs` ↔ `motion_bridge.rs` (10 e 4 menções);
`motion_state` → `motion_shape_placa`/`_gen`/`_bake`, `motion_objeto`, `macro_track`, `motion_particulas`,
`motion_mundo`; `macro_track` ↔ `motion_bridge_*`; `shape` ↔ resto. Uma crate não pode depender de quem
depende dela — mover uma folha que o `MotionState` nomeia obriga a crate nova a importar o núcleo e o
núcleo a importá-la. ⇒ primeiro cortar o núcleo (ADR-0075: componentes + eventos/recursos; systems não
se chamam), depois mover.

## 3. Ondas (série: cada uma muda a fronteira da seguinte)

| onda | o quê | tamanho | crate(s) alvo |
|---|---|---|---|
| **0** | núcleo: o agregado `MotionState` vira COMPOSIÇÃO — cada família tem o seu estado (struct própria, campo no agregado), e as chamadas cruzadas viram eventos/recursos ou um trait de host (HOWTO §1, as portas). `motion_state` ↔ `motion_bridge` deixa de ser ciclo. Nada muda de crate. | 0 linhas movidas | — |
| **1** | `path` + `flip` — folhas, ~3 k. Valida o molde e o lote de gates partidos com pouco risco. | ~3 k | `ph2d-app-motion-path`, `-flip` |
| **2** | `shape` (+ `shape_placa`/`_gen`/`_bake`) | ~12 k | `ph2d-app-motion-shape` |
| **3** | `carimbo`, `lsystem`, `object`/`objeto` | ~13 k | uma por família |
| **4** | `state_conferencia` + `demo`/`state_demo` — o TOPO (dependem de quase tudo), por último | ~45 k | `ph2d-app-motion-conferencia`, `-demo` |

`state_gpu`/`bridge_gpu`/`macro_track` ficam no núcleo até a Onda 0 dizer, pelo grafo medido, se saem.
Ganho esperado (comissão): ~−50 % na unidade da motion; precedente medido, o corte da shell: gate 93 → 60 s.

## 4. Os gates que leem `ph2d-app-motion/src` por CAMINHO (partem mudos — §2.7, §2.9, §2.14)

Cada um tem de passar a varrer a crate nova **e** ganhar piso de população (varre > 0 nela):

- `shells/desktop/tests/it/`: `every_demo_scene_ends_in_an_output_node.rs:30,44` ·
  `every_membrane_resolves_the_driven_param.rs:137` · `the_camera_pan_has_exactly_one_door.rs:39` ·
  `a_layout_never_commands_a_panel_a_bridge_owns.rs:54` · `the_warp_gizmo_is_wired_to_the_pointer.rs:22` ·
  `the_device_marches_the_island_rate.rs:16,99` · `the_gpu_cook_recusal_placement.rs:31,109` ·
  `no_two_object_smoke_modes_claim_the_same_number.rs:20` · `the_graph_only_shouts_while_the_clock_plays_forward.rs:28,31` ·
  `the_spacing_scene_arms_what_its_message_promises.rs:17` · `the_global_palette_is_wired.rs:91` ·
  `the_bridge_reverts_a_param_by_clearing_both_channels.rs:13` · `the_sink_style_is_one_answer_for_both_routes.rs:26`
- `crates/ph2d-editor-core/tests/it/node_id_collisions.rs:97,103,169` ·
  `architecture_as_chaves_partilhadas_nao_ganham_leitor_partilhado.rs:211` ·
  `architecture_no_dependency_climbs_a_layer.rs:324` · `crates/ph2d-editor-core/src/toast_reading_census.rs:43`
- `crates/ph2d-app-constraints/tests/it/as_cenas_de_smoke.rs:698,700` — `include_str!` (§2.6: falha alto)
- internos com `CARGO_MANIFEST_DIR` (§2.6, o gémeo em runtime — mudo): `src/motion_ignore_census.rs:27` ·
  `motion_param_reach.rs:311` · `motion_objeto_entradas_tests.rs:229` · `motion_state_cor_demo_tests.rs:116` ·
  `motion_state_jogo_demo_tests.rs:369` · `motion_state_valor_demo_tests.rs:287` ·
  `motion_state_pilha_demo_discos_diag.rs:391`
- `scripts/motion-gate-placa.sh:18,63-64` · `.config/testes-lentos.tsv` (chave por `binary_id`: o teste
  movido muda de binário e sai da faixa lenta calado) · a feature `panel-motion-graph` (125 usos, §2.4:
  declare-a em cada crate nova que tenha código sob ela).

⚠️ Esta lista é de 29/10 — reconte-a no início de cada onda (`git grep -n 'ph2d-app-motion'` em
`*.rs`, `*.sh`, `*.tsv`, `*.toml`), sem `head` (§2.1).

## 5. Critério VERDE de cada onda (todos, nesta ordem)

1. `cargo check --workspace --all-targets` (via `scripts/ph2d-run.sh`).
2. Cada gate da §4 corrido **por nome** (`scripts/cargo-test-narrow.sh <crate> <nome>`), com piso de
   população na crate nova — um verde que varre zero é vermelho.
3. `architecture_no_orphan_source_file` (§2.14) e `architecture_no_dependency_climbs_a_layer` (a crate
   nova é `ph2d-app-*` ⇒ espécie Familia; família não chama família, §2.20).
4. `cargo machete` (§2.18: a dependência que fica para trás).
5. A catraca: `the_shell_only_shrinks_and_every_app_crate_too` — a crate nova entra com o seu medido, a
   `ph2d-app-motion` baixa o tecto (o censo de obsolescência reprova se não baixar), e **a soma das
   crates motion só desce** (o corte paga prelúdio/`use`; se a soma subir mais que a folga, é fragmentação,
   não partição).
6. Medir o ganho: `touch` de um `.rs` da família × `check`, antes/depois, variantes intercaladas, o
   MÍNIMO ([MEDIR_VELOCIDADE.md](MEDIR_VELOCIDADE.md)) — ⚠️ `touch` não mede uma edição (memória
   `feedback_a_touch_does_not_measure_an_edit…`): edite uma linha de verdade.
7. Só no fim do M (não por onda): a suíte completa 1× — imprescindível, código movido entre crates
   (CLAUDE.md §2).

## 6. O que NÃO fazer

- Mover antes da Onda 0 (o ciclo obriga uma crate a depender da outra nos dois sentidos).
- Subir um tecto da catraca para caber a partição: a cura é mover/cortar.
- Grepar por prefixo para decidir a família (§2.15) ou trocar `crate::foo` sem `\b` (§2.2).
