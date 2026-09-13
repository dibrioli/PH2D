# HANDOFF de INTEGRAÇÃO — `line/loc-caps` (o resto das listas de LOC)

> Modo L. Branch `line/loc-caps`, worktree `Worktrees/line-loc-caps`. **Nada integrado, nada enviado.**
> Leitor: o agente INTEGRADOR (e a próxima LLM que tocar no `main.rs`, no `app_state.rs` ou numa das
> listas numeradas de LOC).
> Bloco de abertura: [`BLOCOS_ABERTURA_REFATORACAO_FINAL_2026-09-13.md`](BLOCOS_ABERTURA_REFATORACAO_FINAL_2026-09-13.md) §5.
> ⭐ **Zero mudança de produto** — todo commit MOVE código (prova por `git show --color-moved`), e as
> únicas linhas novas são assinaturas, chamadas, declarações de módulo, re-exports e cabeçalhos. A
> auditoria de correção (§10) conferiu-o commit a commit.
> ✅ **Smoke do dono OK** (13/09, sobre `cffed737d`: os gestos do §12 — *«smoke parece ok»*).
> ⛔ **Leia o §7.8–§7.12 antes de bissecar esta linha:** quatro commits do meio têm um gate vermelho
> ou enfraquecido que só o fecho apanhou, cada um curado num commit próprio no fim.

## §1 · Identidade

| | |
|---|---|
| branch | `line/loc-caps` |
| HEAD | `4c0b5f2be` (os achados da Lente 1) — o handoff entra no commit seguinte |
| merge-base com `main` | `29ff6576e` (o `main` andou 1 commit de docs, `0bfee712e`, que não toca nada daqui) |
| commits da linha | 35 (+ o do handoff): 31 cortes · 4 curas do fecho (`3fc051eec` · `52ea2e752` · `9bb1cbf90` · `4c0b5f2be`) |

## §2 · O veredito, em números

| lista / grandeza | antes | depois |
|---|---|---|
| `architecture_workspace_file_loc_cap.rs` — `FILE_OVERAGE_OK` (700 L/ficheiro) | 16 entradas | **0** — a lista esvaziou (fica a nota «Retired 2026-09-13») |
| `architecture_panel_loc_cap.rs` — `FN_OVERAGE_OK` (200 L/fn de painel) | 6 entradas | **0** — a lista esvaziou (fica a nota «✅ A LISTA ESVAZIOU») |
| `fn_loc_caps.rs` (shell, 200 L/fn) — as entradas DESTA linha | 8 | **1**: `main.rs::new` 263 → **237** (razão medida, §5) |
| `file_loc_caps.rs` (shell, 600 L/ficheiro) — as entradas DESTA linha | 2 | **2**, as duas descidas: `app_state.rs` 1 551 → **1 019** · `main.rs` 1 289 → **1 118** (razões medidas, §5) |
| linhas `.rs` em `shells/desktop` (base `193 205`, quota da linha 800) | 193 205 | **193 645** (**+440**: cabeçalhos dos irmãos, re-exports, e os gates re-apontados do §6, que moram em `tests/it`) |
| `nextest list --workspace` (ci-test) | 22 723 | **22 723** — `ONLY-A 0` · `ONLY-B 0` · `MOVED 0` |
| schemas · registos · contrato congelado · ADR · `Cargo.lock` | — | **intocados** (§3) |

## §3 · Superfície de colisão (`bash scripts/collision-surface.sh`, colado)

Corrida em `a7707b752`; os quatro commits seguintes são as curas do §7.8–§7.12 (ficheiros de teste,
`row.rs`, `init_subsystems.rs` e três prosas) e não mexem em nada que esta sonda mede.

```
SUPERFÍCIE DE COLISÃO — line/loc-caps contra main
  merge-base 29ff6576e   ·   31 commit(s)   ·   66 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

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
  ✗  1019 / 600   shells/desktop/src/app_state.rs
  ✗  1118 / 600   shells/desktop/src/main.rs
───────────────────────────────────────────────────────────────────────────────
```

⚠️ O `project*.rs` que a linha toca é o `project_load.rs` — **nenhum degrau de migração nem a tripla
`PROJECT_SCHEMA` mudou** (o corte partiu o `project_load_from` em fases, §4.1). Os dois `✗` são as
duas entradas NUMERADAS que ficam, com a razão medida (§5).

## §4 · Os cortes — ficheiro/função → para onde → linhas

Unidade das funções: **linhas do `fn` à chaveta que o fecha** (a do `fn_loc_caps` da shell; o gate
dos painéis descarta linhas vazias e de comentário, e por isso os títulos de alguns commits de painel
dizem números menores que esta tabela).

### §4.1 · Shell — as 8 funções

| função | antes | depois | o que saiu, e para onde | commit |
|---|---:|---:|---|---|
| `envelope_smoke::frame` | 247 | **6** | `frame_cage` 159 (`=11`/`=12`) · `frame_star` 94 (`=27`) — o `frame` despacha `27 => star, _ => cage` | `170cacef2` |
| `build_smoke_router::route` | 441 | **6** | quatro sub-roteadores ligados por `\|\|`, **pela mesma ordem**, cada um acaba em `false`: `route_deformers_fx_and_text` 121 · `route_drawing_and_layout` 131 · `route_ui_system` 111 · `route_paints_edits_and_assets` 87 | `069456fe6` |
| `blend_smoke` | 205 | **167** | o braço `8 if level == 3` → `blend_smoke_arc_spine` 41 | `f27a6b09b` |
| `LayoutLive::lay_out` | 206 | **167** | a recolha dos filhos em largura → `layout_live_collect.rs::collect_flow_children` 69 (uma linha nova: `let w = sim.world();`) | `c52895917` |
| `build_initial_state` | 499 | **160** | por SUBSISTEMA, pela ordem do arranque → `init_subsystems.rs`: `boot_assets_and_renderer` 61 · `boot_sim_world` 98 · `boot_script_host` 20 · `boot_editor_layer` 52 · `boot_rt_pipeline` 53 · `boot_imageio_registries` 21 · `boot_vec_scene` 27. O `boot_hero_screen` 104 **fica no `init.rs`**: o gate `the_registry_is_installed_before_the_hero` lê lá a ordem | `172e47e8e` · cura `9bb1cbf90` |
| `project_load_from` | 484 | **194** | as duas fases que o cabeçalho do ficheiro já nomeava: `project_forget_previous` 135 (a sessão esquece o anterior) · `project_install_accepted` 185 (o aceite entra). Fica no pai o decode e as recusas | `decacf746` · gates `52ea2e752` |
| `build_smoke` | 416 | **139** | `build_smoke_blend_scenes` 188 (níveis 7–9) · `build_smoke_object_scenes` 109 (10, 15–19), entregues logo depois do roteador | `037224ca5` · nota `4c0b5f2be` |
| `App::new` | 263 | **237** ⛔ FICA | a abertura do comando e do áudio → `app_state_devices.rs::init_gamepads` 23 / `init_audio` 10 | `a7707b752` |

### §4.2 · Shell — os 2 ficheiros

| ficheiro | antes | depois | o que saiu, e para onde | commit |
|---|---:|---:|---|---|
| `app_state.rs` | 1 551 | **1 019** ⛔ FICA | `app_state_gfx.rs` 384 (o `AppGfx` inteiro) · `app_state_image_tools.rs` 203 (a transação de undo de um Apply, os 6 aliases de preview, o predicado da paleta e — no 2.º commit — as duas portas do undo) · `app_state_hero_live.rs` 33 (o `HeroLive`) · e declara o `app_state_devices.rs` | `de9be8bcf` · `a7707b752` |
| `main.rs` | 1 289 | **1 118** ⛔ FICA | `app_state_devices.rs` 121 (abertura do comando/áudio + `pump_gamepad` · `convert_modifiers` · `timestamp_ns` · `push_input_to_script`) · as duas portas do undo de imagem → `app_state_image_tools.rs` · `mod theme_env_tests` → `main_theme_env_tests.rs` 40 (mesmo nome de módulo ⇒ mesmos nomes de teste) | `a7707b752` |

Os outros ficheiros da shell que a linha partiu ficaram todos abaixo de 600: `envelope_smoke.rs` 284 ·
`build_smoke_router.rs` 497 · `blend_smoke.rs` 270 · `layout_live.rs` 531 (+83) · `init.rs` 352 (+366) ·
`project_load.rs` 558 · `build_smoke.rs` 566.

Nenhum caminho mudou: `crate::AppGfx`, `crate::HeroLive`, `crate::ImageEditTransaction`,
`crate::commit_image_edit_transaction`, `crate::app_state::UpscalePreview`, … resolvem pelos re-exports
do `app_state.rs` e do `main.rs`. Nenhum campo da `App` mudou (catraca `the_app_only_sheds_fields`, 187).
Nenhum `mod` novo no `main.rs` — os irmãos declaram-se DENTRO do `app_state.rs` (o `theme_env_tests`
continua a ser o mesmo módulo, só com o corpo noutro ficheiro).

### §4.3 · Crates — os 16 ficheiros do tecto de 700

| ficheiro | antes | depois | irmãos | commit |
|---|---:|---:|---|---|
| `ph2d-render/src/compressed_pipeline.rs` | 993 | 618 | `_tests.rs` 379 | `3340f6cca` |
| `ph2d-tool-bgremoval/src/algorithm/compose.rs` | 931 | 407 | `compose_tests.rs` 528 | `4a56c9b81` |
| `ph2d-vector/src/vector_network.rs` | 780 | 487 | `_tests.rs` 296 | `514321589` |
| `ph2d-tool-equalize-sizes/src/algorithm.rs` | 755 | 534 | `_tests.rs` 225 | `1214b0bd2` |
| `ph2d-tool-color-equalization/src/gpu/auto_wb.rs` | 748 | 590 | `_tests.rs` 162 | `a1ee76e6f` |
| `ph2d-tool-color-equalization/src/gpu/tonal_batch.rs` | 744 | 468 | `_tests.rs` 280 | `c505f3364` |
| `ph2d-imageio-ph2d-native/src/schema.rs` | 746 | 695 | `schema_tests.rs` 55 (⚠️ formato serializado: nenhum campo nem ORDEM de campo mexeu — um hunk só, a apagar o `mod tests`) | `f250a1bd2` |
| `ph2d-tool-rasterize/src/algorithm.rs` | 734 | 470 | `_tests.rs` 268 | `720b8ae91` |
| `ph2d-render/src/renderer.rs` | 932 | 595 | `renderer_tests.rs` 110 · `renderer_individual.rs` 244 (a fachada da loja individual) | `84e0d4bfa` |
| `ph2d-tool-color-equalization/src/params.rs` | 888 | 514 | `params_tests.rs` 110 · `params_edit.rs` 278 (`pub use edit::{ColorEqualizationUiEdit, apply_ui_edit}`) | `d53f947b2` |
| `ph2d-render/src/layer_compositor/mod.rs` | 882 | 652 | `flatten.rs` 174 (`pub use flatten::{GpuOpScratch, flatten_layer_ops}`) · `readback.rs` 79 | `d00cbf803` |
| `ph2d-painter-effects/src/adjustments/mod.rs` | 863 | 652 | `params_queries.rs` 178 · `psd_export.rs` 51 (`pub use psd_export::PsdExport`) | `2c6ac0774` |
| `ph2d-painter-effects/src/adjustments/spatial.rs` | 856 | 601 | `spatial_tonal.rs` 265 (`pub use tonal::{apply_bloom, apply_shadows_highlights}`) | `3eda64964` |
| `ph2d-editor-core/src/grid_snap/state.rs` | 796 | 556 | `state_nonuniform.rs` 255 | `28525cadf` |
| `ph2d-render/src/individual.rs` | 708 | 621 | `individual_error.rs` 95 (`pub use error::IndividualTextureError`) | `d47547703` |
| `ph2d-tool-bgremoval/src/algorithm/chroma/mod.rs` | 704 | 611 | `kmeans.rs` 101 | `023b86de1` |

API pública das crates: **nenhum caminho mudou** (cada item que saiu volta por `pub use` no sítio antigo).

### §4.4 · Painéis — as 6 funções do tecto de 200

| função | antes (gate) | depois (fn→`}`) | irmã | commit |
|---|---:|---:|---|---|
| `ph2d-panel-color-equalization` `populate` | 203 | 64 | `register_slider_rows` 142 | `f2ae7ebb1` |
| `ph2d-panel-hierarchy` `paint_hierarchy_row` | 226 | 178 | `paint_row_locks` 83 (devolve o `right_x`) | `51b3e63a9` · cura `3fc051eec` |
| `ph2d-panel-inspector` `apply_event_impl` | 230 | 160 | `color_tint_click` 76 (devolve `bool`; chamada logo depois dos `SINGLE_ID_CLICKS`) | `014afaf63` |
| `ph2d-panel-equalize-sizes` `paint_body_sections` | 237 | 137 | `paint_mode_rows` 136 (devolve o `y`) | `99ead9bda` |
| `ph2d-panel-audio-mixer` `paint` | 203 | 184 | `paint_scrollbar_and_publish` 29 | `6ac8fac89` |
| `ph2d-panel-hierarchy` `paint_hierarchy_body` | 252 | 180 | `paint_head.rs::paint_hierarchy_head` 92 (devolve `(body_pad, search_rect, search_text)`) | `6cb94c950` · gate `4c0b5f2be` |

## §5 · As entradas que FICARAM, com a razão MEDIDA — e a proposta

⛔ O bloco proibia REAGRUPAR campos da `App` nesta rodada (as linhas paralelas compilam contra
`self.<campo>`). Com essa cerca, as três entradas não chegam ao tecto — medido:

| entrada | valor | porque não desce mais |
|---|---:|---|
| `file_loc_caps` `app_state.rs` | 1 019 | a `struct App` sozinha são **937 linhas** (187 campos, cada um com o doc do dono) |
| `fn_loc_caps` `main.rs::new` | 237 | o literal `Self { … }` são **233 linhas** — um inicializador por campo (+ os `#[cfg]` e os comentários de semeadura) |
| `file_loc_caps` `main.rs` | 1 118 | as primeiras **658 linhas** são a raiz: **246** declarações de `mod`, **35** aliases `pub(crate) use … as …` e **314** linhas de comentário que dizem porque cada um existe. Mover um `mod` para dentro de outro muda o `super::` de quem o escreve — **71 dos 245** módulos de topo com ficheiro escrevem `super::` (= a raiz) — e o `crate::x` de todo citador |

**Proposta — uma linha própria, depois desta integração** (as três entradas caem juntas):

1. **Agrupar os campos da `App` por assunto**, contados por prefixo/sufixo do nome:
   `*_smoke_done` **33** (um `SmokeLatches`) · `*_preview*` **14** (as caches das ferramentas de
   imagem — o tipo já mora no `app_state_image_tools.rs`) · `*_live` **13** · `last_*` 10 ·
   `pending_*` 7 · `timeline_*` 7 · `ui_*` 5 · `prefab_*` 5 · `cycle_*` 5 · `undo_*` 5. Cada grupo
   com `Default` encolhe a struct **e** o literal do `App::new` de uma vez; a catraca
   `the_app_only_sheds_fields` desce no mesmo commit.
2. **Os 95 módulos `*smoke*` da raiz** num directório `smoke/` (declarado por `#[path]`) com os
   re-exports que mantêm `crate::x_smoke` — medir antes quantos deles escrevem `super::`.

## §6 · Leitores de TEXTO re-apontados (cada um lê o que lia antes, nem mais nem menos)

| leitor | o que mudou | commit |
|---|---|---|
| `ph2d-editor-core` `hr15_no_hardcoded_ui_strings` | a entrada `("ph2d-panel-hierarchy/src/paint.rs", 1)` passa a `paint_head.rs` (o `.placeholder("Search…")` mudou-se) | `6cb94c950` |
| `ph2d-panel-hierarchy` `hierarchy_selection_by_identity` | o grep da selecção por NOME lê `paint.rs` **+** `paint_head.rs` (as três asserções são AUSÊNCIAS, logo juntar é legítimo). Prova de mutação: `// entity.name == sel_label` no `paint_head.rs` ⇒ vermelho; restaurado ⇒ verde | `4c0b5f2be` |
| `project_input_map_tests::the_authored_map_has_exactly_one_holder` | lê as QUATRO structs que moravam no ficheiro único: `app_state.rs` + `_gfx` + `_image_tools` + `_hero_live` | `de9be8bcf` (2) · `a7707b752` (4) |
| `scripts/fecho-da-familia.py` (o censo de campos `RE_CAMPO`) | lê os quatro **pela ordem do ficheiro único** — o `setdefault` fica com a 1.ª declaração, e `physics` existe no `AppGfx` e na `App`. Prova: `PYTHONHASHSEED=0` sobre `render_loop`, árvore de ANTES contra a de agora ⇒ só mudam a contagem de ficheiros (444 → 449) e o LOC do `app_state.rs` | `de9be8bcf` · `a7707b752` |
| `a_baked_object_outlives_the_3d_module::loading_forgets_the_baked_objects_of_the_previous_document` | a ordem «esquecer antes de devolver os canais» em TRÊS metades: no pai, `self.project_forget_previous(` antes de `self.project_install_accepted(`; no 1.º irmão, `forget_live_producers()`; no 2.º, `restore_baked_forms(` | `52ea2e752` |
| `the_sculpt_document_is_wired::the_refusal_precedes_every_mutation_of_the_session` | em DUAS metades: no pai, `decode_doc(` antes de `self.project_forget_previous(`; no irmão, `forget_live_producers()` | `52ea2e752` |

⛔ **Nos dois últimos, concatenar os corpos para medir a posição seria FRAUDE** — tudo o que está num
irmão viria depois de tudo o que está no pai, e a asserção passaria por construção. A ordem entre as
CHAMADAS é também imposta pelos tipos: o `sculpt` decodificado é argumento do `project_forget_previous`,
e o `file` é MOVIDO para o `project_install_accepted`. Os dois leem pelo `sculpt_source::project_family_fn`,
que procura a função na família `project*.rs` em vez de num endereço.
**Prova de mutação:** apagar `self.forget_live_producers();` do `project_forget_previous` ⇒ os **dois**
reprovam; restaurado (sem diff contra HEAD) ⇒ os dois verdes (`target/prova/repoint_{green,mut,green2}.txt`).

Leitores conferidos e que NÃO precisaram de mudar: `ph2d-app-components::component_registry_for_tests`
(lê `init.rs::build_component_registry`, que não saiu) · `the_global_palette_is_wired` ·
`the_tree_settles_before_the_capture` · `the_math_is_installed_at_boot` ·
`the_open_recipe_comes_to_the_artist` (as agulhas estão no `render_frame` e no `fn main`, que ficaram) ·
`the_app_only_sheds_fields` · `the_sculpture_bytes_cross_a_build_that_never_reads_them` ·
`the_shape_art_picker_is_wired` · `the_ui_burst_is_wired` (campos da `App`, que ficaram) ·
`no_two_smoke_scenes_claim_the_same_level` · `project_settings_tests`. Os outros leitores da shell que
vivem em crates (`ph2d-app-field3d`, `-motion`, `-skeleton`, `-flip`, `-registry-init`, `ph2d-ecs`)
leem ficheiros que esta linha não tocou.

## §7 · Construído, medido e REVERTIDO (ou corrigido no caminho)

1. **Painel da Hierarquia:** a cabeça como função-irmã NO MESMO ficheiro deixava o `paint.rs` entre
   597 e 603 linhas contra o tecto de 600 ⇒ irmã em ficheiro próprio (`paint_head.rs`).
2. **`layout_live.rs`:** a recolha como função-irmã no mesmo ficheiro dava 604 ⇒ `layout_live_collect.rs`.
3. **`blend_smoke`:** a 1.ª inserção caiu DENTRO da função (a âncora `}` era ambígua) ⇒ não compilou
   (`self` fora de função associada) ⇒ refeito com a âncora conferida em três linhas.
4. **`main.rs`:** o import `use ph2d_render::SpriteRenderer;` parecia só das portas movidas e foi
   retirado ⇒ `E0425` em `render_loop/frame_gfx.rs`, que o alcança pela cadeia de `use super::*` até à
   raiz ⇒ o import FICA. (Os outros dois — `log_input_event`, `Modifiers` — saíram de facto.)
5. **`app_state.rs` cresceu para 1 040** com as declarações dos `devices`, contra a entrada que o commit
   anterior descera a 1 037 e que só desce ⇒ o gate reprovou ⇒ o `HeroLive` saiu no mesmo commit (1 019).
6. **O `fecho-da-familia.py`** lido na ordem `app_state.rs` → `_gfx` trocava o tipo de `.physics` (de
   `PhysicsBridge` para `PhysicsState`) ⇒ reordenado; o resto da diferença era a ORDEM de empates, que
   varia com o hash de `set` do Python ⇒ provado com `PYTHONHASHSEED=0`.
7. **A agulha do Input Map** em `de9be8bcf` lia só `app_state.rs` + `_gfx` — as duas structs da
   transação de undo tinham saído da varredura ⇒ corrigido em `a7707b752` (lê as quatro).
8. ⛔ **Dois gates de `shells/desktop/tests/it` ficaram VERMELHOS de `decacf746` até `52ea2e752`**
   (`the_refusal_precedes_every_mutation_of_the_session` e
   `loading_forgets_the_baked_objects_of_the_previous_document`): liam o corpo do `project_load_from`
   por nome, e o filtro de gates que corri no commit não os incluía. Quem os apanhou foi a suíte
   `--test it` INTEIRA do fecho. *Um filtro escrito à mão por nomes de ficheiro não vê um gate que lê
   uma FUNÇÃO pelo nome.*
9. ⛔ **Clippy vermelho em `ph2d-panel-hierarchy` de `51b3e63a9` até `3fc051eec`**
   (`needless_option_as_deref`: `hit_index.as_deref_mut()` passado à irmã, sem uso depois). O
   `cargo check -p … --all-targets` por crate estava verde; só o clippy da workspace o viu.
10. **`vector_network_tests.rs`:** o `rustfmt` juntou linhas depois de desindentar ⇒ o `movecheck`
    lista-as como novas; são as mesmas expressões numa linha.
11. ⛔ **Clippy vermelho na shell de `172e47e8e` até `9bb1cbf90`** (`let_and_return` duas vezes): o
    corte do arranque fez do ScriptHost e da cena vetorial funções que acabavam em `let x = match …;`
    seguido de `x`. Só apareceu na 2.ª corrida do clippy da workspace — a 1.ª parou na crate do §7.9
    antes de chegar à shell. *Um clippy que falha numa crate não diz nada sobre as que vêm depois.*
12. ⛔ **Gate ENFRAQUECIDO, verde, de `6cb94c950` até `4c0b5f2be`:** o `hierarchy_selection_by_identity`
    lia só `paint.rs` depois de a cabeça do painel sair para `paint_head.rs` — uma comparação da
    selecção por nome escrita na cabeça passaria muda. Nenhuma suíte o via (uma ausência num ficheiro
    menor continua ausente); quem o apanhou foi a Lente 1 (§10). No commit era verdade que o
    `paint_head.rs` não tinha nenhuma das três agulhas.

## §8 · Premissas DESTE bloco que a medição derrubou

1. *«No `App::new`/`app_state.rs`/`main.rs`, mover impl, tipos auxiliares e docs chega ao tecto»* — não
   chega: 233 linhas de literal, 937 de struct, 658 de raiz (§5). O bloco já previa esta saída.
2. *«`cargo check -p <crate> --all-targets` sozinho prova a crate»* — prova a COMPILAÇÃO. O clippy da
   workspace, a suíte `--test it` inteira e a auditoria apanharam o §7.8, §7.9, §7.11 e §7.12, que os
   checks por crate não viam.
3. **A build sem `sculpt3d` NÃO compila na BASE** (antes desta linha, em ficheiros fora do território:
   `render_loop/audio_pieces.rs`, `render_loop/fase_bus_drain.rs`, `render_loop/mod.rs:727` `surface`, e
   imports de escultura em `tests/it`). ⇒ o ramo `#[cfg(not(feature = "sculpt3d"))]` do corte do load
   (`self.project_forget_previous(&mut file);`, o parâmetro e o `solo`) foi verificado por LEITURA — por
   mim e pela Lente 1 —, não por build.
4. Os títulos de alguns commits trazem estimativas com `~` que a medição corrigiu: `build_initial_state`
   «~140» é **160**; `lay_out` «~159» é **167**; `project_load_from` «~190» é **194**; e os de painel
   usam a unidade do gate dos painéis (sem linhas vazias/comentário), menor que a desta tabela.

## §9 · Provas do fecho

| prova | resultado |
|---|---|
| `cargo nextest list --workspace --cargo-profile ci-test` + `nextest-list-diff.py antes depois` | 22 723 = 22 723 · **ONLY-A 0** · ONLY-B 0 · MOVED 0 |
| `cargo nextest run -p ph2d-host-desktop -p ph2d-editor-core` (ci-test) | 1.ª: 3 805 / 3 807 (os 2 do §7.8) · depois da cura, com `-p ph2d-panel-hierarchy`: **3 844 / 3 844** |
| `cargo test -p ph2d-host-desktop --test it` (à parte) | 1.ª: 801 / 803 (os mesmos 2) · depois da cura: **803 / 803** |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | 1.ª: 1 erro (§7.9) · 2.ª: 2 erros (§7.11) · 3.ª: **verde** |
| `cargo check --workspace --all-targets` | verde, **0 warnings** |
| `bash scripts/check-standalone-optional.sh` · `bash scripts/check-workflow-packages.sh` | verdes (10 crates com dependência interna opcional · 32 nomes contra 359 membros) |
| `cargo check -p <crate> --all-targets`, cada crate tocado SOZINHO (14 crates + a shell) | 15 × verde, 0 warnings |
| `cargo machete` · `cargo fmt --all -- --check` · `typos` nos ficheiros da linha | verdes (fmt e typos re-corridos depois de cada cura) |
| depois das curas da Lente 1: `ph2d-panel-hierarchy` inteira · as listas de LOC, hr15, cenas de smoke, órfãos e tecto da shell | **37 / 37** · **19 / 19** · `cargo check -p ph2d-host-desktop` verde |
| delta da shell contra `target/prova/shell_antes.txt` | **+440** (quota 800) |
| suítes das crates da FRENTE 2 (corridas no corte) | 9 crates 2 415 / 2 415 · `ph2d-editor-core` + 5 painéis 1 916 / 1 916 |

## §10 · Auditoria (2 lentes, agentes independentes, só leitura)

**Lente 1 — CORREÇÃO (todos os commits): nenhuma mudança de comportamento.** Conferido commit a commit:
as linhas sem comentário do pai e do filho como conjuntos (nada perdido; o que entra são assinaturas,
`mod`/`#[path]`, `use super::*`, visibilidades, re-exports, chamadas e valores devolvidos — mais o
re-embrulho do `rustfmt` no `vector_network.rs`, as duas linhas adaptadas do `init.rs` e o `match`
devolvido do comando); saídas antecipadas (nenhum `return`/`?`/`break` nos irmãos que mude o fluxo do
chamador); o roteador (cortes entre blocos `if`, `||` preserva ordem e primeiro-que-casa); o envelope
(no pai, `27` era o único nível da estrela); o `build_smoke` (os braços que ficaram só aceitam níveis
≤ 6, e o `3 =>` sem guarda já era inalcançável para 7–10 e 15–19); o `project_load` (três hunks, ordem
igual; entre o sítio velho e o novo do `stable_id_seed` só há leituras de `file.*`, o `mem::take` da
escultura e uma cópia de `file.physics`); o `init` (cada `boot_*` reinserido no sítio da chamada dá o
corpo do pai); atributos órfãos em todo corte (nenhum); os números citados (conferem com o código); os
`pub use` das crates; o `schema.rs` (um hunk, só o `mod tests`). **Achados e o que lhes aconteceu:**

| # | achado | severidade | destino |
|---|---|---|---|
| 1 | `hierarchy_selection_by_identity` lia só `paint.rs` | gate enfraquecido | **curado** em `4c0b5f2be`, com prova de mutação (§6, §7.12) |
| 2 | `tools/ph2d-loc-trend` `CRITICAL_FILES` segue o `paint.rs` da Hierarquia e não o `paint_head.rs` | nit | **fica**: a lista é de ficheiros cujo crescimento já pediu intervenção («acrescentar pede justificação»); o orquestrador que cresceu continua lá, e a cabeça tem 105 linhas |
| 3 | em `de9be8bcf` a agulha do Input Map lia duas das quatro structs | gate enfraquecido num commit intermédio | já curado em `a7707b752` (§7.7) |
| 4 | o ramo `cfg(not(feature = "sculpt3d"))` do load não é compilado por nada | caminho não verificado | nomeado (§8.3) — a build sem a feature está partida na base |
| 5 | `architecture_panel_loc_cap.rs`: notas a falar de entradas que já não existem | prosa desactualizada | **curado** em `4c0b5f2be` |
| 6 | `envelope_smoke::frame` com doc «(11 ou 12)» por cima do `27` | prosa desactualizada (anterior à linha) | **curado** em `4c0b5f2be` |
| 7 | `build_smoke.rs`: «OBJETO vetorial (7-11)» e braços «acima» que saíram | prosa desactualizada | **curado** em `4c0b5f2be` |

**Lente 2 — COSTURA DE UI (os 6 cortes de painel): ZERO achados.** Cada id foi seguido do sítio que o
pinta/regista até ao consumidor (barramento ou handler); nenhum bloco movido tem `return`/`?`/`break`/
`continue`; os locais mutados (`y`, `right_x`) voltam por valor; a ordem de registo no hit index (quem
ganha na sobreposição) não mudou; a contagem do hr15 no `paint_head.rs` é 1 e no `paint.rs` é 0.
⚠️ **E nomeou três gates que NÃO apanhariam um corte errado** (pré-existentes, abertos — §12):
1. o registo do cadeado e do grupo de cada linha da Hierarquia — `hit_indexed_ids_are_registered` só lê
   ficheiros com `paint` no nome e só vê `ids::LITERAL` (estes ids são calculados); o `seam` usa `Click`
   sintético; `no_companion_target_leaves_its_own_row` tem piso `>= 8` que o olho e o ícone já cumprem;
2. a barra de rolagem do mixer e a publicação das alturas — nenhum teste de comportamento a pinta;
3. os cliques de cor da sprite no Inspector — o `inspector_regression` usa `Stimulus::Picked`, e não
   ficou provado que passe pelo ramo `Click`.
(Protecção REAL, para contraste: o `y` do Equalize Sizes está coberto por
`the_mode_row_never_invades_the_row_below_it`.)

## §11 · Para o INTEGRADOR

1. **As duas listas partilhadas da shell** (`fn_loc_caps.rs`, `file_loc_caps.rs`) foram editadas SÓ nas
   linhas desta linha; as outras duas linhas da rodada editam as delas no mesmo ficheiro ⇒ conflito
   textual de vizinhança possível, e a resolução é **manter as duas**.
2. **`TETO_LOC`:** o delta desta linha é **+440** sobre `193 205`.
3. **`main.rs` / `app_state.rs`:** as outras linhas estão proibidas de acrescentar `mod` ao `main.rs` e
   campos à `App`. ⚠️ Quem acrescentar um consumidor de `crate::SpriteRenderer` (ou de outro import da
   raiz) por `use super::*` dentro do `render_loop` continua servido — o import ficou (§7.4).
4. Os quatro métodos de entrada (`pump_gamepad`, …) são agora `pub(crate)` em `app_state::devices`;
   um chamador novo escreve `self.pump_gamepad()` como antes.
5. ⚠️ **Bissecar dentro desta linha:** os commits entre `decacf746` e `52ea2e752` têm dois gates de
   texto vermelhos; os entre `51b3e63a9`/`172e47e8e` e `3fc051eec`/`9bb1cbf90` têm o clippy vermelho;
   os entre `6cb94c950` e `4c0b5f2be` têm um gate enfraquecido (§7.8–§7.12) — nenhum deles com
   mudança de produto.
6. Nada de schema, contrato, `Cargo.lock` nem ADR.

## §12 · Smoke do dono (gestos) · e o que fica ABERTO

**Gestos** (a árvore: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-loc-caps`):
1. Abrir o app: `cargo run -p ph2d-host-desktop --profile smoke` — o terminal diz o comando e o áudio como antes.
2. Gravar e abrir um projecto: `Ctrl+S`, fechar, `Ctrl+O` — a cena volta inteira (incluindo uma escultura/objeto assado, se houver).
3. As cenas de construção: `env PH2D_BUILD_SMOKE=7` (e `=8`, `=9`: blend), `=10` (morph), `=15`..`=19` (chamfer, quinas, expand e os dois retunes), `=11`/`=12` (envelope com gaiola), `=27` (envelope da estrela), `=50` (auto layout).
4. O blend vivo: `env PH2D_BLEND_SMOKE=3` (o spine curvo).
5. Hierarquia (olho, cadeado, grupo, `Add`, busca) e Inspector (as cores da sprite).
6. Grid Snap (as grelhas não-uniformes).
7. Ferramentas de imagem: Remoção de fundo, Color Equalization (os sliders), Equalize Sizes (as linhas de cada modo), Rasterize — e `Ctrl+Z` depois de um Apply.
8. Mixer de áudio (rolar a lista).
9. Abrir e gravar uma imagem `.ph2d`.

**Build do smoke** (depois de `rm -rf target/*/incremental`, que levou o `target/` de 23 G a 20 G, sobre
`4c0b5f2be`): 1.ª corrida `Finished smoke profile [optimized] target(s) in 9.31s`; **2.ª corrida:**

```
    Finished `smoke` profile [optimized] target(s) in 0.29s
```

**ABERTO:**
- A proposta do §5 (agrupar os campos da `App`; os módulos `*smoke*` num directório) — as três entradas que ficam só caem com ela.
- Os três gates que a Lente 2 nomeou (§10) — nenhum corte desta linha os tocou, mas nenhum o teria apanhado.
- A build sem `sculpt3d` partida na base (§8.3), fora deste território.
