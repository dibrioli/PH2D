# HANDOFF de integração — `line/motion-value`, CICLO 15 (2026-10-08)

> As duas ordens do dono de 08/10: *«o nó de corpo mole deve ter limite de 32 768»* e *«o defeito
> pode consertar conforme padrão ouro»* (o Bug #15). Doc do ciclo:
> [124](../124_ciclo_15_o_tecto_dos_criadores_e_o_colisor_do_corpo_mole.md) — estado medido, desenho e
> kill-criteria escritos ANTES; o oráculo, a rodada de medição e as provas de mutação estão lá. A linha
> NÃO integra e NÃO pusha: integrar é por ordem do dono.

## §1 — Base e commits

- Base: `main` `2a925859d`. A linha leva os ciclos 13 e 14 (handoffs
  [CICLO_13](HANDOFF_INTEGRACAO_line_motion_value_2026-10-07_CICLO_13.md) e
  [CICLO_14](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_14.md), por integrar) e, por cima, os
  commits do ciclo 15 (`git log --oneline 2a925859d..line/motion-value`).

## §2 — O que fechou

| # | item | veredito | gate / prova |
|---|---|---|---|
| 1 | o tecto de `32 768` em TODO criador | o censo achou CINCO acima: `motion.soft_body` (`262 144`), `motion.emitter` (`4 M`), `motion.path` (`2^24`), `motion.distribute_poisson` (`262 144`), `source.table` (sem tecto) — todos cortados pela constante da casa; os multiplicadores (`trail`/`mirror`/`combine`/`rgb_split`/`drop_shadow`) ficam, já decididos e medidos por recurso | `nenhum_criador_passa_o_tecto_com_a_contagem_no_extremo` (censo comportamental, `0,9 s`) · um gate por crate (`the_mesh_never_passes_the_per_node_ceiling`, `neither_mode_passes_the_per_node_ceiling`, `the_radius_floor_keeps_every_rectangle_under_the_ceiling`, `a_table_above_the_ceiling_delivers_its_first_rows`) |
| 2 | Bug #15 | ✅ pelo padrão-ouro, com o oráculo (Godot `4.7.2`, MIT) corrido sobre a NOSSA fixtura: os pares que o repouso sobrepõe não colidem (filtro do FleX; `rest` + `body` publicados pelo corpo mole e pela corda, lidos pelo colisor na CPU e no WGSL), o contacto entra na velocidade (`sb_pos`/`sb_dt`), e o corpo declara `substeps` ao relógio do grafo. ⭐ O corpo NÃO incha — o «inchar» do ciclo 14 era o defeito; e a corda gémea passava o gate ESTICADA `37 %` | `the_collider_holds_the_diameter_and_the_body_does_not_swell` (lê o número do oráculo da fixtura `ferramentas/godot_oracle_corpo_mole/saida.txt`) · `a_pair_the_rest_pose_overlaps_is_held_by_the_body_not_the_collider` · `without_the_body_columns_the_packing_is_bit_identical` · `the_body_filter_packs_the_same_on_both_routes` (placa) · `rope_thickness` (agora exige o comprimento) · cena `=129` |

| 3 | o smoke do dono: *«o colisor estava desligado nas duas. ao ligar não vi diferença»* | ✅ medido: os dois botões `Collide` visíveis (Shape, Output) não faziam nada sozinhos — a cena segurava com o `motion.collide` fora do catálogo. **Decisão do dono (08/10): o `Collide` do Shape segura o corpo mole.** O `motion.soft_body` passou a LER o colisor declarado das peças que entram pela porta `shape` (dentro de cada passo, o que o `sim.step` faz), passa adiante a aparência delas, e o `ph2d_contact` ganhou o filtro opcional `Pecas::com_ligados` (pares que já se SOBREPÕEM no repouso). A cena `=129` refeita: a única diferença entre as duas colunas é o `Collide` do Shape | `turning_on_the_left_shapes_collide_makes_it_hold` (o gesto do dono) · `the_left_body_crosses_and_the_right_one_holds` · `a_body_of_pieces_holds_the_diameter_and_does_not_swell` (os MESMOS números da rota do `motion.collide`) · `a_pair_the_body_holds_is_left_alone` · `the_rows_the_tutorial_names_are_on_the_cards` · `todo_leitor_do_colisor_declarado_se_regista` (o corpo mole na tabela) |

## §3 — Superfície de colisão (para o integrador)

- **`ph2d-contact`** (motor partilhado — `sim.step`, o passe do Output, o `motion.collide`): `Pecas` ganha o campo `ligados: Option<Ligados>` (`Pecas::novas` põe `None`; os quatro literais dos testes ganharam `ligados: None`), o filtro em `varredura::corrigida`, `ligados_tests.rs`. Ausente ⇒ idêntico (a suíte inteira verde).
- **Nós:** `ph2d-node-motion-soft-body` (+ `contacto.rs`: o leitor do colisor declarado e a «veste»; dependência nova `ph2d-contact`; `register_declared_collider_reader`) (`lib.rs`: `MAX_SIDE` derivado, as colunas `rest`/`body`/`sb_pos`/`sb_dt`, `fold_correction`, o param `substeps` APENDADO no fim do manifesto; `params_ui.rs`; os testes ganharam o argumento `body` em `simulate`) · `ph2d-node-motion-collide` (`lib.rs` `Estrutura` + o argumento novo de `push_apart`; `gpu.rs` duas ligações `rest`/`body` — **muda o WGSL**; os testes) · `ph2d-node-motion-verlet-rope` (`lib.rs`: `rest`/`body` no `eval`) · `ph2d-node-motion-emitter` (`MAX_ALIVE`) · `ph2d-node-motion-distribute-poisson` (`floor_by_ceiling`) · `ph2d-node-motion-path` · `ph2d-node-source-table`.
- **`ph2d-app-motion/src`:** `motion_tecto_de_instancias.rs` (o censo); cena nova `motion_state_corpo_com_colisor_demo.rs` (+`_tests`), `mod` em `motion_state.rs`, braço `Some("129")` e `MAX_DEMO_LEVEL 128 → 129` em `motion_state_demo_router.rs` (⚠️ outra linha que acrescente uma cena reconta o número); a cena `=5` (`motion_state_gpu_demos.rs` + o teste dela) ao tecto; escadas de sondas paradas no tecto (`motion_rig_relogio.rs`, `motion_bridge_fontes_costura.rs`); comentários de `1,2 M` (`motion_bridge_edit*.rs`, `motion_state_conferencia_demos_drizzle.rs`); `motion_state_rig_tutorial_tests.rs` (`2,26 → 0,25`).
- **`ph2d-gpu-cook/tests/it`:** `gpu_collide_corpos.rs` novo (+`mod` no `main.rs`), escadas paradas no tecto em `gpu_collide.rs` e `gpu_cpu_parity_sim.rs`.
- **`ph2d-node-registry-init/tests/it`:** `soft_body_radius.rs` (gate novo + sondas), `rope_thickness.rs` (gate com comprimento + sonda + um nó de teste `teste.sem_corpo`).
- **`ph2d-i18n`:** `node.motion.soft_body.param.substeps`.
- **Censos que mudaram de lista (com o motivo escrito ao lado):** `ph2d-node-registry-init/tests/it/substeps.rs`
  (`CLOCK_DECLARERS` ganha o `motion.soft_body`) e `ph2d-app-motion/src/motion_bridge_params_stream.rs`
  (`INTERNAL` ganha `rest`/`body`/`sb_pos`/`sb_dt`). O `soft-body` ganhou o ficheiro `cap_gates.rs`
  (corte de LOC, `#[path]`).
- **Docs:** doc 124 novo, nota no doc 103 §5, Bug #15 fechado e Bug #16 aberto no BUGS, `README.md` do módulo (regenerado), tutoriais `08`/`09` (src + PDF), o oráculo em `ferramentas/godot_oracle_corpo_mole/`, memória (três famílias + o vigia do `pgrep`), `CLAUDE.md` §5.1.
- ⚠️ Nenhum contrato congelado tocado. Nenhum `PROJECT_SCHEMA` mexido (o param novo é APENDADO e tem default `1` = byte-idêntico).

## §4 — Gate batched (sobre `2a925859d..` a árvore)

| passo | resultado | carga |
|---|---|---|
| `nextest-impacted.sh` (`BASE=2a925859d`) | 1.ª corrida `17031/17034` — três vermelhos REAIS, curados: o censo dos donos do relógio (`substeps.rs`, o corpo mole entrou em `CLOCK_DECLARERS` com o motivo), o tecto de LOC (`soft-body/lib.rs` `766` → `645`, os gates do tecto num irmão `cap_gates.rs`), e o seletor de colunas (`rest`/`body`/`sb_pos`/`sb_dt` no `INTERNAL`) ⇒ **`17034/17034`** | `35 → 4` |
| `ph2d-app-motion` inteiro depois da última receita da cena | ✅ `1379/1379` | `16` |
| `RUSTFLAGS=-D warnings cargo check --all-targets` (as onze crates tocadas) | ✅ | `17` |
| `cargo clippy --all-targets -D warnings` (as onze) | ✅ (dois lints curados: `neg_cmp_op_on_partial_ord`, `collapsible_if`) | `14` |
| `cargo fmt --all --check` | ✅ | — |
| `censos-da-arvore-combinada.sh` | ✅ `114/114`, `12/12` censos | — |
| `doc-index.sh --check` | ✅ `17` índices | — |
| placa `scripts/motion-gate-placa.sh` | ✅ `30/30` (duas vezes, antes e depois da receita final) | `4` |
| placa `ph2d-gpu-cook --test it --release -- gpu_collide` | ✅ `9/9` (inclui `the_body_filter_packs_the_same_on_both_routes`) | `4` |

## §5 — Provas de mutação

`10/10` sangram — a tabela está no [doc 124 §5](../124_ciclo_15_o_tecto_dos_criadores_e_o_colisor_do_corpo_mole.md).

## §6 — Conhecidos que NÃO são deste ciclo

- O `ph2d-render` com o desenho do SIGSEGV (`~20` `OnceLock` de contexto nos `tests/it`) e a sonda
  `FIELD_DOC_VERSION` do `scripts/collision-surface.sh` — herdados do ciclo 14, fora do módulo.
- O `motion.scatter` honra no `eval` o tecto da casa mas é `O(count²)`: o digitável dele é `3 000`
  (de recurso); um `count` dirigido por fio acima disso custa segundos. Medido no censo, não é deste
  ciclo.

## §7 — O que fica ABERTO

- **Bug #16** — a corda com colisor atravessa-se durante um chicote a `60 Hz` (`0,0497` na 2.ª metade):
  com sub-passos do grafo segura `0,81`, mas a corda não declara o ritmo do grafo (o `solver_substeps`
  é LOCAL por uma decisão medida). O `motion.collide` está fora do catálogo. Decisão para a próxima
  linha que tocar na corda.

## §8 — Smoke e binário

✅ **Smoke do dono APROVADO (2026-10-08)** — a cena `=129` com o gesto do `Collide` do Shape.

Smoke ao dono (em `--release`): a cena `=129` refeita pela decisão dele (a diferença entre as duas
gelatinas é SÓ o `Collide` do Shape; o gesto é ligar o da esquerda), fotografada a `5`, `9` e `12 s`
(`target/prova/c15/fotos/`), e o tutorial `09` (a tabela do custo e a nota «bolinhas que não se
atravessam», PDF refeito). Binário `ph2d-host-desktop --release` compilado; a 2.ª corrida: `0`
`Compiling`, `Finished … in 0.25s`. Gate final: `nextest-impacted` **`17099/17099`**; `clippy -D
warnings` e `fmt` limpos nas crates tocadas; placa `30/30`.
