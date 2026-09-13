# HANDOFF DE INTEGRAÇÃO — `line/render-bodies` (2026-09-13)

> **Leitor:** o agente integrador e a próxima LLM que tocar no quadro. Denso de propósito.
> **Estado:** FECHADA — não integrada, não enviada (CLAUDE.md §0.7). Worktree `Worktrees/line-render-bodies`, ramo
> `line/render-bodies`, merge-base `29ff6576e`, 23 (com o deste handoff) commits.
> ⛔ **Esta linha não muda produto:** nenhum pixel, texto ou comportamento, e a ORDEM do quadro é a de sempre. Sem janela o
> quadro acaba no primeiro `gfx` (hoje o `let gfx = self.gfx.as_mut()?;` da `fase_frame_canvas`), logo a prova de
> COMPORTAMENTO é o smoke do dono (§12); a prova de que o CÓDIGO é o mesmo é a de movimento de cada peça (§5).

## 1. O que a linha fez

Tirou das listas NUMERADAS dos tectos de LOC da shell (`tests/it/fn_loc_caps.rs::FN_OVERAGE_OK`, 200 L por função;
`tests/it/file_loc_caps.rs::FILE_OVERAGE_OK`, 600 L por ficheiro) as **22 entradas** do território `render_loop/` +
`hero_intents/` — 18 funções e 4 ficheiros —, movendo os corpos para funções e ficheiros menores pela ordem de sempre:
UM commit por peça, cada um a compilar e a passar, com a entrada a sair no mesmo commit. ⭐ O `run_render_frame` (984 → 71)
e o `fase_bus_drain` (2401 → 80) deixaram de ter entrada: nenhum corpo do quadro vive hoje acima do tecto. Depois, o fecho
curou o clippy da workspace sem supressões, e as três lentes de auditoria deram quatro commits de cura (§6.9–§6.12).

## 2. Superfície de colisão (`bash scripts/collision-surface.sh`, sobre o ramo final)

```
SUPERFÍCIE DE COLISÃO — line/render-bodies contra main
  merge-base 29ff6576e   ·   22 commit(s)   ·   66 arquivo(s)
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)
▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0169   próximo livre: 0170
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo
▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha
▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
    nenhum arquivo da linha passa do teto
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

## 3. Corpo → peças → linhas

| # | Corpo (antes → depois, L) | Peças | Commit |
|---|---|---|---|
| 1 | `hero_intents/image_edit/equalize_sizes.rs::drain_equalize_sizes` 204 → 132 | `commit_per_entity` (a fase 3) | `e0a2900c7` |
| 2 | `hero_intents/image_edit/bgremoval.rs::drain_bgremoval` 237 → 185 | `spawn_islands` (as ilhas `[1..]`) | `5e492eff4` |
| 3 | `hero_intents/hierarchy.rs::drain_reparent` 297 → 194 | a ordem entre raízes · a ordem entre irmãos | `01eddb458` |
| 4 | `render_loop/push_look_probe.rs::probe_push_render_and_look` 314 → 161 | `blob_cross_and_smear` (cenas 10–13) | `e8b5dfa12` |
| 5 | `render_loop/autokey_pass.rs::apply_samples` 317 → 165 | `diff_each_sample` | `c70b121b7` |
| 6 | `render_loop/bgremoval_preview.rs::dispatch` 333 → 196 | filho `bgremoval_preview_gpu.rs` (`upload_preview`, `draw_overlays`) | `624392e03` |
| 7 | `render_loop/image_edit.rs::dispatch` 483 → 133 | bakes por sprite · Apply de ferramenta · filho `image_edit_import.rs` | `c668b9b12` |
| 8 | `render_loop/hierarchy.rs::dispatch` 414 → 180 | câmera + interruptores · verbos de instância · filho `hierarchy_select.rs` | `e0d9aa207` |
| 9 | `render_loop/inspector_commits.rs::dispatch` 383 → 187 | `drain_section_edits` · `drain_signal_names` · testes → `inspector_commits_sprite_field_tests.rs` | `47eb8e6f6` |
| 10 | `hero_intents/sprite_merge.rs::drain_merge_sprites` 439 → 173 | `order_selection` · `read_sources` · filho `sprite_merge_warp.rs` (`merge_grid`, `composite`) | `1868e1330` |
| 11 | `render_loop/fase_snapshots_publish.rs::fase_snapshots_publish` 216 → 146 | fase-filha `fase_snapshot_readouts` (`SnapshotReadouts`) | `2207ee0d5` |
| 12 | `render_loop/fase_vector_bands.rs::fase_vector_bands` 209 → 171 | fase-filha `fase_authored_panel_rows` | `ee077da4c` |
| 13 | `render_loop/fase_audio_panels.rs::fase_audio_panels` 373 → 20 | fases-filhas `fase_audio_mixer` · `fase_audio_editor_{io,rack,prep}` | `ca4e18b7e` |
| 14 | `render_loop/sim_extract.rs::run` 491 → 155 · **ficheiro 1155 → 533** | filho `sim_extract_emit.rs` · testes → 4 `sim_extract_*_tests.rs` | `cd2bb602f` |
| 15 | `render_loop/present.rs::run_present_phase` 484 → 131 | 3 métodos `present_*` · filho `present_title.rs` | `ec21e8e79` |
| 16 | `render_loop/snapshots.rs::publish` 1068 → 73 · **ficheiro 1398 → 566** | `publish_hierarchy` · `publish_gizmo` · filhos `snapshots_{hud,gizmo,inspector,inspector_sprite}.rs` · testes → `snapshots_sheet_authorship_tests.rs` | `337dddb1f` |
| 17 | `render_loop/fase_bus_drain.rs::fase_bus_drain` 2401 → 80 · **ficheiro 2654 → 579** | `DrainOut: Default` · 11 sub-drenos `fase_bus_*` + 6 pedaços das cadeias do clique/campo, em 5 filhos | `4869918e7` |
| 18 | `render_loop/mod.rs::run_render_frame` 984 → 71 · **ficheiro 1604 → 569** | `fase_frame_{simulation,canvas}` · `fase_hero_{frame,document_verbs,tools,scene,commits}` · `frame_prof.rs` | `0ec55e7ed` |
| — | o clippy da workspace sem supressões novas (§6.9) | tuplos → `MergeGrid`/`Identity`/`PhysicsSections`/`LateSections`; `let x = …; x` → a expressão; um `&*` a menos | `4414bddd0` |
| — | as garantias que o `match` único dava (§6.10) | 3 gates novos | `e3da48c13` |
| — | o extract volta a segurar a entidade que gerou (§6.11) | `EntityWorldMut` até ao `spawn`; `drop` → `into_world_mut` | `e561bfb63` |
| — | os gates de caminho leem os filhos `#[path]` (§6.12) | `rust_src::{path_children, with_path_children}` + 1 gate; a agulha da captura do pivô | `21d3b3217` |

Os filhos declaram-se no PAI por `#[path]`; no `render_loop/mod.rs` só entram três `mod` novos (`fase_frame_open`,
`fase_hero_frame`, `frame_prof`). Nenhum `mod` novo no `main.rs`, nenhum campo novo na `App`.

## 4. O que o `frame_text::render_frame` vê

- Só se emendam chamadas `self.fase_*(` (recursivamente). **30 fases novas**, todas chamadas pelo quadro (o gate das órfãs
  passa): `fase_snapshot_readouts` · `fase_authored_panel_rows` · `fase_audio_mixer` · `fase_audio_editor_{io,rack,prep}` ·
  `fase_bus_{tool_panel,tool_panel_forward,click_live,click_layout,click_document,value_bones_and_text,value_live,
  timeline_panel,tool_requests,topbar,hierarchy_tree,hierarchy_rows,sprite_ops,inspector_sections,inspector_instance,
  physics_sections,inspector_identity}` · `fase_frame_{simulation,canvas}` · `fase_hero_{frame,document_verbs,tools,scene,commits}`.
  A auditoria de integridade reconstruiu o texto emendado em Python, byte a byte: 125 fases na base, 155 no ramo, zero órfãs.
- ⚠️ **Um ajudante SEM o prefixo é invisível ao texto do quadro** (`present_*`, `publish_hierarchy`, `publish_gizmo`,
  `sim_extract_emit::sprite`, os ajudantes dos `hero_intents`…). Nada se perdeu por isso: os PAIS deles
  (`run_present_phase`, `snapshots::publish`, `sim_extract::run`, os `drain_*`) já eram chamados pelo nome e nunca foram
  emendados, e os gates deles leem por CAMINHO — e desde §6.12 leem também os filhos `#[path]`.
- A ordem: o dreno aparece braço a braço pela ordem do `match` antigo (a cadeia `and_then` dos sub-drenos), e o ramo do
  `HeroScreen` bloco a bloco. A auditoria de correção comparou os 88 padrões (87 braços + o `_`) da base com os do ramo pela
  ordem da cadeia: idênticos, as duas guardas incluídas; e as 40 saídas antecipadas da base mapeiam uma-a-uma em 40 `?`.
- ⚠️ A janela do braço `InspectorSignalLeaveEdit` no `every_inspector_verb_declares_its_bulk_behaviour` (do nome do braço até
  ao próximo `EditorAction::`) atravessa agora o fim do sub-dreno dele e 466 B do seguinte. Esse verbo é uma excepção
  declarada, logo o alargamento só pode dar um VERMELHO falso, nunca um verde falso.
- Os censos por NOME de ficheiro (`mod.rs` ou `fase_*.rs`: `architecture_no_per_tool_branch_in_render_loop`,
  `the_skeleton_panel_only_opens_where_it_has_a_subject`, `morph_arrow_seam_tests`, `modal_tests`, `bone_limit_gizmo_tests`,
  `shell_frame_tests`) veem todo o código de quadro que se mudou: os ficheiros novos do quadro chamam-se `fase_*`, salvo o
  `frame_prof.rs` (contadores e duas notas do perfilador, sem nenhuma agulha desses censos).

## 5. As trocas declaradas, e as provas

Cada peça MUDA corpos; as únicas diferenças de texto são as declaradas na mensagem do commit, e a prova de movimento corre
contra o HEAD antes do commit (normalização: espaço em branco, vírgula final, chavetas de fecho/braço de expressão única).

- **Peças 1–16:** trocas por linha (`super::` → `crate::render_loop::` num filho, um `let x = …;` devolvido como expressão,
  `&T` onde era `&&T`), provadas por `moved_proof.py`.
- **Peça 17 (o dreno):** os 227 `let mut pending_* = None/false/Vec::new()` viram os campos do `DrainOut` (ordem igual,
  assert; o `Default` é o valor da declaração, assert contra o tipo; o comentário da declaração mora no campo); as três
  LEITURAS do quadro (`osso_selecionado`, `selecao_bits`, `inspector_selection`) ficam no `let mut pd`. No corpo dos braços a
  única troca é `pd.<pedido>`, só em código, por um lexer (`rustlex.rename`, zero saltos); a prova tira os `pd.` e devolve o
  trecho de HEAD byte a byte. A fila sai por `mem::take` durante o laço e volta. As cadeias `if … else if` partidas: cada
  pedaço devolve se tomou o evento, e o seguinte só corre se não. 7 mutações.
- **Peça 18 (o quadro):** `let Some(p) = f() else { return; };` → `let p = f()?;` (37×) e o `is_none()`/`return` do
  `fase_vec_expand` → `?` (1×); o destructure do `DrainOut` (232 L) sai, e cada pedido que uma intenção levava por atalho sai
  do dreno (`x: take(&mut pd.x)`, 219×; o `osso_selecionado`, lido duas vezes, copia-se; as leituras `Copy` em expressão
  leem `pd.x`; o `hierarchy_select_intent` move-se por `take`). Os três nomes que um bloco LIGA de novo
  (`pending_vec_text_axis`, `reparent_intent`, `hierarchy_select_intent`) ficam intocados a partir do `let`. Cada pedido é
  consumido exatamente uma vez (assert do gerador). A prova tira as três formas e devolve o trecho só com a troca do `?`.
  3 mutações.
- **As curas do fecho (§6.9–§6.12):** não são movimentos; cada uma diz na mensagem o que muda e traz as suas mutações.
- Ferramentas: `.cauda-render-bodies/` na worktree — `move_block.py`, `moved_proof{,2,3}.py`, `verbatim_norm.py`,
  `rustlex.py`, `frame_xform.py`, `gen_drain.py`, `gen_frame.py`, `fix_*.py`, `commit4.sh`, `mutlib.sh`, `mut_*.sh`.
  ⚠️ Estão excluídas pelo `.git/info/exclude` do primário: **não viajam no ramo**.

## 6. Construído · medido · REVERTIDO

1. As primeiras fases-filhas nasceram com `mod x;` novos no `render_loop/mod.rs` — que estava NO tecto numerado (1610 > 1604):
   **REVERTIDO** (`reset --mixed` para `c70b121b7`) e refeito com filhos `#[path]` declarados no pai.
2. O `inspector::publish` saiu da 1.ª redacção com 202 L → partido outra vez (`identity`).
3. O renomeador por lexer olhava para trás no texto cru e leu o `.` final de um COMENTÁRIO como acesso a campo — o assert de
   zero saltos apanhou-o na 1.ª corrida; passou a olhar só para CÓDIGO.
4. A troca `let … else` → `?` casou `if let Some(bits) = osso_selecionado {` na 1.ª corrida — o assert de
   `else { return; };` apanhou-o; um `if`/`while`/`&&` à esquerda deixam de contar.
5. A mensagem do commit do dreno saiu com o `{MUT}` por preencher (aspas num heredoc) → `--amend` só da mensagem.
6. `commit3.sh` (a entrada de uma linha) partiu na entrada multi-linha do `sprite_merge` → `commit4.sh` com as duas formas.
7. Na spec do áudio, o `mod_decl` do 1.º spec deslocava as linhas do 2.º → passou para o último.
8. O gate dos braços do dreno (§6.10) nasceu com o piso de **88** escrito de memória e reprovou: o censo mediu **87** — o 88
   contava o `_` da barra do topo. Corrigido para o medido.
9. ⚠️ **O clippy da workspace reprovou no fecho com três lints de peças ANTERIORES** (um `let_and_return`, um tuplo de 9 em
   `type_complexity`, um `&*` sobre uma referência partilhada), e a linha tinha acrescentado **quatro supressões de espécies que
   a base não tinha** (`type_complexity` ×3, `let_and_return` ×1; a base tem 0 de cada). Curados SEM supressão — a regra do
   dono, «silenciar um diagnóstico é armengo» — em `4414bddd0`. ⚠️ Os `#[allow(clippy::too_many_arguments)]` ficam: a base
   tem **61**, o ramo tem **81**, e são o molde da abertura («os locais por parâmetro») a esbarrar no limiar de 7.
   Os `drop_non_drop` (2), `float_cmp` (3) e `cast_possible_truncation` (27) são os da base, mudados de sítio — e os dois
   `drop_non_drop` saem em §6.11.
10. **A lente de CORREÇÃO e a de CUSTO acharam a mesma coisa: o corte do dreno tirou ao compilador três garantias** que o
    `for action in hero.bus.drain()` dava por construção (o `Drain` emprestava a fila inteira; o `rustc` avisava de um braço
    inalcançável; o laço não tinha para onde devolver um `?`). Nenhuma falha hoje, nenhum gate as impedia amanhã →
    `e3da48c13`: `no_sub_drain_writes_the_bus_it_borrowed`, `every_request_has_one_arm_across_the_chain`,
    `no_arm_body_ends_the_drain_early` (censo de CÓDIGO, pisos de 5 ficheiros / 11 sub-drenos / 87 braços), 3 mutações.
11. **A lente de CUSTO achou uma procura a mais por sprite emitida, todo quadro:** o corte `cd2bb602f` guardou só o
    `Entity` do builder e o `sim_extract_emit::spawn` re-obtinha-o (`present.entity_mut(builder_id)`). Não alocava, e passou
    a prova de movimento porque o texto movido era o mesmo — o que mudou foi o CAMINHO entre o `spawn` e as inserções.
    `e561bfb63`: o `EntityWorldMut` viaja até ao `spawn`, e o `drop(builder)` que devolvia o `present` vira
    `builder.into_world_mut()` (os dois `#[allow(clippy::drop_non_drop)]` saem). ⚠️ O custo removido não foi medido — a
    cura repõe o caminho da base, que é a barra.
12. **A lente de INTEGRIDADE DOS GATES achou dois enfraquecimentos MUDOS** (`21d3b3217`, 5 mutações):
    - a agulha `let joint_pivot_commit =` (`joint_anchor_gizmo`) passou a casar a CHAMADA da fase que o §5 escreveu
      (`let joint_pivot_commit = self.fase_inspector_commits(…)?;`, que no texto emendado vem antes do corpo) → a agulha é a
      captura inteira, `… = transform_edit.map(`;
    - três gates liam o `snapshots.rs`/`sim_extract.rs` por CAMINHO (`the_hover_pick_happens_exactly_once`,
      `the_hierarchy_does_not_pick_the_canvas`, as duas ausências do `an_empty_object_is_reachable`), e ~1 350 linhas
      desses ficheiros foram para filhos `#[path]` → UMA porta nova, `tests/it/rust_src.rs::{path_children,
      with_path_children}` (recursiva, com o gate `the_path_lens_reads_the_children`).

## 7. Premissas REFUTADAS

1. **O cabeçalho do `fase_bus_drain.rs` justificava a entrada numerada:** *«partir por braço obrigaria o corpo a escrever
   `*pedido = …` através de referências, e isso já não é o corpo verbatim»*. Falso: um contexto NOMEADO
   (`pd: &mut DrainOut`) pede um prefixo, que um lexer põe só em código e a prova tira.
2. **`convert_to_curves_asks_one_question` estava VERDE sobre um COMENTÁRIO** desde a OBRA 2 da `line/render-loop`: lia o
   `render_loop/mod.rs` inteiro, e a porta `vec_convert::is_convertible` só lá estava na prosa — a chamada mudara-se para a
   `fase_selection_mirror_convert_envelope`. Hoje lê o quadro SEM comentários e exige a CHAMADA.
3. **A ausência do rig** (`!block.contains("&inspector_selection")`) deixaria passar `&pd.inspector_selection` → alargada ao nome.
4. *«Cortar o `run_render_frame` em sub-índices basta para o `mod.rs` caber em 600»* — não bastava: ficavam as 451 L de
   declarações de módulo e o perfilador (125 L). O perfilador foi a única saída de código não-quadro do `mod.rs`.
5. (peça 16) *«Partir a lista em lets mudaria a ordem de avaliação»* — verdadeiro no texto (o bloco dos segundos de corrida
   passa a correr antes das secções do Inspector) e INOBSERVÁVEL: dos dois lados só há leituras, e as escritas no painel
   continuam no fim, pela ordem de sempre.
6. *«O `frame_text` só emenda `self.fase_*(`»* — confirmado, com a consequência que a abertura não nomeava (§4, 2.º ponto).
7. ⚠️ ***«A prova de movimento prova que nada mudou»*** — prova o TEXTO. Uma procura a mais por sprite (§6.11) passou todas
   as provas de todas as peças, porque o que se moveu foi o caminho entre duas linhas idênticas; só a lente de custo a viu.
8. ⚠️ ***«Um gate de caminho acompanha o ficheiro»*** — acompanha o PAI. Um corte por `#[path]` deixa as ausências e as
   contagens exactas cegas ao filho (§6.12), e nenhuma reprova.
9. ⚠️ **O `Drain` emprestado era parte do contrato do dreno e ninguém o tinha escrito:** partir o `match` em funções troca
   três impossibilidades do compilador por três convenções (§6.10). *Uma partição que remove uma garantia de tipo tem de
   deixar um gate no lugar dela, no mesmo commit* — aqui ficou para o fecho, que é onde a auditoria a encontrou.

## 8. Quota da shell (`the_shell_only_shrinks`, TETO_LOC 196 990 intocado)

`target/prova/shell_antes.txt` = **193 205**; agora **194 378** (**+1 173**; quota da linha 1 500). As peças 1–16
somaram +1 220; o dreno −226; o quadro −117; as curas do fecho +296 (a maior parte é o gate novo do dreno e a lente dos filhos).

## 9. Portão do fecho (medido sobre o ramo final)

- `cargo nextest list --workspace --cargo-profile ci-test` × `target/prova/antes.txt`: **22 723 → 22 727**
  testes, `MOVED 0`, `ONLY-A 0`, **`ONLY-B 4`** — os quatro gates novos, nomeados: `every_request_has_one_arm_across_the_chain`, `no_arm_body_ends_the_drain_early`, `no_sub_drain_writes_the_bus_it_borrowed`, `the_path_lens_reads_the_children`
  (§6.10 e §6.12). Nenhum nome de teste mudou: os testes que se mudaram de ficheiro foram por `#[path]`.
- `cargo test -p ph2d-host-desktop --test it`: **807 passed**, 0 failed, 6 ignored.
- `cargo nextest run -p ph2d-host-desktop`: **2230/2230**. · `bash scripts/nextest-impacted.sh`: **2236/2236**.
- `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings`: **exit 0** (a 1.ª corrida do fecho
  reprovou com três lints — §6.9).
- `cargo check --workspace --all-targets`: **0 avisos**.
- `scripts/check-standalone-optional.sh` ✓ (10 crates com dependência interna opcional) · `scripts/check-workflow-packages.sh`
  ✓ (32 nomes citados pelos workflows contra 359 membros) — corridos sobre `0ec55e7ed`; as curas seguintes não tocam
  `Cargo.toml` nem features.
- `cargo machete`: nenhuma dependência morta. · `typos --force-exclude` sobre os 66 ficheiros da linha: limpo.
- Gates de família em `crates/` que leem a shell pelo caminho: 10/10 (os três do `field3d`, o do esqueleto, o
  `the_intent_drain_reaches_every_variant`, os quatro do `architecture_no_orphan_source_file`, o `the_shell_only_shrinks`).
- Provas de mutação: 7 (dreno) + 3 (quadro) + 3 (as garantias do dreno) + 5 (as lentes de caminho) + as das peças 1–16
  (nas mensagens de commit) — cada uma reprova, restaura byte a byte com `touch`, e o controlo passa.
- Carga da máquina durante o portão final: `load` 2.55–2.71 (nenhum gate de relógio reprovou).

## 10. Auditoria (três lentes independentes, sem cargo, sobre o ramo)

- **Correção e ordem:** nenhum defeito. Verificou o destino de cada pedido (88 padrões idênticos pela ordem da cadeia, as
  guardas incluídas), a semântica do `else if` em cada corte, a devolução da fila, que nenhum campo do `DrainOut` é lido
  depois do `take`, os três nomes religados, as 40 saídas antecipadas → 40 `?`, o `_paint_frame_timer` a cair no fim do
  quadro e a ordem das chamadas. Dois riscos LATENTES (a fila e o braço repetido) → §6.10.
- **Custo e alocações:** nenhuma alocação, clone, `Box` ou `format!` novo por quadro; o `mem::take` e o `Default` não alocam;
  cada pedido consumido uma vez. Achou a procura a mais por sprite → §6.11, e o mesmo risco da fila → §6.10. ⚠️ Não
  verificou, e fica registado: os dois reordenamentos DECLARADOS de leituras (as leituras da física antes do `gfx` da fase-mãe
  em `2207ee0d5`; o player antes das secções do Inspector em `337dddb1f`) — os dois lados são leituras (as assinaturas o
  dizem), mas a preparação de `QueryState` do áudio/câmara não foi seguida até ao fim.
- **Integridade dos gates:** reconstruiu o texto emendado da base e do ramo e mediu cada agulha e cada janela dos dois
  lados; nenhum gate vermelho, dois enfraquecimentos mudos → §6.12, e a janela do `InspectorSignalLeaveEdit` (§4). Os
  nomes dos testes mudados por `#[path]` são os mesmos (contagens de `#[test]` iguais).
- Lacuna nomeada por duas lentes e que esta linha NÃO fecha: a afirmação «zero alocações por quadro» assenta em provas de
  TEXTO e na leitura das lentes — não há contador de alocações sobre um quadro real (sem janela, o quadro acaba cedo).

## 11. O que a fusão pode partir

- **Os gates editados** (só agulhas/lentes do `render_loop`/`hero_intents`, cada troca com mutação):
  `architecture_no_downcast_to_concrete_tool_in_shell`, `convert_to_curves_asks_one_question`,
  `every_inspector_verb_declares_its_bulk_behaviour`, `selection_gestures_are_not_fanned_out`,
  `the_canvas_backdrop_has_one_door`, `the_paste_is_the_one_joint_edit_that_fans_out`, `the_shape_art_picker_is_wired`,
  `the_skeleton_speaks_when_it_has_no_subject`, `the_stroke_checkbox_is_wired`, `the_stroke_paint_row_is_wired`,
  `joint_anchor_gizmo`, `the_highlight_has_one_source`, `an_empty_object_is_reachable`, e os das peças 1–16
  (`git diff --name-only 29ff6576e..HEAD -- shells/desktop/tests`). ⚠️ A `line/input-dispatch` re-aponta agulhas nos mesmos
  gates partilhados — no `the_highlight_has_one_source` esta linha só ACRESCENTOU linhas depois do laço e trocou a leitura do
  `snapshots.rs` do `the_hierarchy_does_not_pick_the_canvas`; um conflito textual aqui é de AGULHA, e as duas edições coexistem.
- **Ficheiros de teste partilhados, só acrescentos:** `tests/it/main.rs` (um `mod the_bus_drain_keeps_what_the_single_match_guaranteed;`)
  e `tests/it/rust_src.rs` (duas funções e um gate no fim).
- **As listas `FN_OVERAGE_OK`/`FILE_OVERAGE_OK`:** esta linha só APAGOU as 22 entradas dela; a `line/loc-caps` mexe nas
  mesmas listas → conflito de linhas vizinhas provável; resolva mantendo as entradas das outras linhas.
- ⚠️ **Código que aterre no dreno ou no ramo do `HeroScreen`:** um braço que escrevia `pending_x = …` escreve
  `pd.pending_x = …` (o local não existe); um braço NOVO entra no sub-dreno do assunto dele (ou num novo, na cadeia
  `and_then` pela ordem do `match` antigo — e o `every_request_has_one_arm_across_the_chain` reprova um pedido com dois
  braços); um pedido NOVO é um campo do `DrainOut` (com o comentário e o `Default`), e a fase que o consome lê-o por
  `take(&mut pd.x)`. Uma fase consumidora nova entra no bloco `fase_hero_*` do assunto dela.
- Nenhum contrato congelado, schema, registo, ADR ou pacote externo tocado (§2).
- O `CLAUDE.md §5` não foi tocado (edita-se na integração): uma linha sugerida para o *Editor / shell* — *«o
  `run_render_frame` (71 L) e o dreno do barramento (`DrainOut` + 11 sub-drenos) cabem nos tectos sem entrada numerada —
  [handoff](docs/IntegracaoMultiAgente/HANDOFF_INTEGRACAO_line_render_bodies_2026-09-13.md)»*.

## 12. Smoke do dono — os gestos

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-render-bodies && cargo run -p ph2d-host-desktop --profile smoke
```
O critério é um só: **tudo igual a antes**. Qualquer diferença (um painel que não atualiza, um clique que não faz nada, uma
ordem diferente, um Ctrl+Z que não desfaz) é defeito desta linha.
1. Abrir cada módulo pela barra do topo (os pills) e voltar.
2. Criar DOIS objetos e selecionar os dois: o Inspector mostra-os (campos «Mixed» onde diferem); mudar a Opacidade muda os dois.
3. Hierarquia: renomear (duplo clique), o olho, o cadeado, apagar, duplicar, arrastar uma linha para outro pai — e Ctrl+Z
   desfaz cada um, um passo por gesto.
4. Com imagens: Remoção de fundo (aplicar, com ilhas separadas), Equalize Sizes (duas imagens), Color Equalization, e
   «fundir sprites» (duas imagens, botão direito na Hierarquia → Merge Sprites).
5. Os painéis de áudio (mixer e editor) abrem e mexem.
6. Selecionar objetos no canvas e na Hierarquia: o Inspector, a Hierarquia e o gizmo acompanham NO MESMO quadro (sem
   atraso de um quadro).

## 13. Smoke compilado (`cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida)

```
    Finished `smoke` profile [optimized] target(s) in 0.19s
```
