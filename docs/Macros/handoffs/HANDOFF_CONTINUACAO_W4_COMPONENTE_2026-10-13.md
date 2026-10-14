# Continuação — line/Macros-Automação, W4 itens 1–4 e 5.1–5.5 (A+B+C+D) fechados → a prova do bloco (cenas de smoke + medição)

> Para a PRÓXIMA janela. Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`, base
> `0aab2c1cb6` (reaberta a 14/10 depois da integração; ver «Reaberta» abaixo). Leia antes de agir:
> - o [ADR-0186 §Emenda W4](../../architecture/decisions/0186-a-macro-grava-comandos-resolvidos-no-relogio-da-macro.md) (e a W3);
> - o [handoff da W3](HANDOFF_CONTINUACAO_W3_MOTION_2026-10-13.md) e o [plano da onda 1](../01_plano_onda1.md).

## Estado (commits desde a W3)

| commit | o quê |
|---|---|
| `9ebdf44f13` | o palco não guia o painel do editor nem é guiado por ele |
| `2e8e4403eb` | o palco recebe a placa e o mundo; o fiel confere as marcas da bomba |
| `02d083459a` | o alvo: a trilha abre com o objecto em edição; o palco coze só o dono |
| `53b6fd7cff` | o componente `Macro`; `PROJECT_SCHEMA` 188 → 189 |
| `75afc54516` | os vermelhos da suíte (contagens dos registadores, censo do Inspector) |
| `5b0328fada` | **14/10, item 5.1–5.2**: `PropKind::MacroTime = 18` → `MacroTempo` (runtime), limpo por quadro |
| `1f4e85547b` | **14/10, item 5.3–5.4**: `add_macro_clip` (a porta do Gravar ▸ Parar) e `MacroEmCena` |
| `412aba04da` | **14/10, item 5.5 A+B+C**: palcos do Motion por entidade, o editor larga o dono, N fontes no quadro |
| `4efd97012b` | prova de mutação do A+B+C: 10/10 |
| `dbf89c60db` | **14/10, item 5.5 D**: o Painter em cena (prova 14/14) |
| `86192e1559` | **14/10 (fim)**: o palco edita o objecto da trilha (nós criados na gravação já desenham) |
| `11c5c20ef1` | as cenas `PH2D_MACRO_SMOKE=1` (Splash) e `=2` (Motion nó a nó) |
| `037a08b75b` | `fotografa_cena.sh`: cache de shaders real, a crase que corria, rajada `FOTO_RAJADA` |
| `b2c5ba08b3` | sondas do custo do quadro com N palcos (`#[ignore]`, placa) |
| `819e04a6bb` | gate `o_splash_tocado_pinta_a_tela_e_o_inicio_e_branco` |

### 14/10 — itens 5.1 a 5.4 (o que existe; não reconstruir)

- **O canal** (`5b0328fada`): `PropKind::MacroTime = 18` (fio; `from_target`, `i18n_suffix` `macro_time`,
  `from_expr_name`, LINEAR, `Sum`); fora de `ALL`/`AUTOKEYED`; **no «+ Track»** («Macro Time»: re-cronometrar
  a macro ou recriar a faixa). `write_prop` insere `ph2d_ecs::MacroTempo { segundos: f64 }` (sem `Serialize`,
  fora do save/undo; o ledger `ph2d-timeline-preview` não o vê, e não tem de ver) **só numa entidade com
  `Macro`**; `read_prop_kind` devolve `MacroTempo` ou `0` (o `rest`), `None` sem `Macro`.
  `ph2d_timeline::clear_macro_clocks(world)` corre em `render_loop/timeline_bridge.rs` antes dos três ramos do
  apply e do `toca_os_objectos`. ⚠️ No Arrange um canal que nenhum strip cobre NÃO se escreve (`eval_frame`
  devolve `touched.then_some`), logo a macro só toca onde um clipe a conduz. K: conduzida = o instante que
  mostra; sem faixa = `key_time` (identidade, a lei do Time); sem `Macro` = recusa
  (`timeline_bridge_keys.rs::key_value_for`). Gates: `macro_time_conduz` (5), `o_tempo_da_macro_chega_a_lista`,
  `o_tempo_da_macro_some_quando_a_faixa_deixa_de_conduzir`, `k_no_tempo_da_macro_cai_no_relogio_que_ela_mostra`;
  `prop_readback`/`clip_stack_autokey` cobrem o canal. **Mutações 6/6 sangram** (id 18, valor, cerca do `Macro`,
  a limpeza, a cerca do K, a linha do «+ Track»).
- **A porta do clipe** (`1f4e85547b`): `TimelineDoc::add_macro_clip(name, owner, D) -> Result<usize,
  ClipRefusal>` (`doc_owner.rs`): clip do dono, chaves LINEARES `(0,0)→(D,D)`, `length_override = D`, clip
  activo intacto, D ≤ 0 = uma chave. Laço (`keys_loop_range` do clip) e Autoplay ficam ao artista. Ninguém a
  chama ainda: o Gravar ▸ Parar é W5.
- **A macro em cena** (`1f4e85547b`): `ph2d_app_motion::macro_track::em_cena::MacroEmCena { de(bytes, modo),
  segue(segundos, &MundoDaCena, placa), palco, tempo, erro }` + `instante(s) -> MacroTime` (µs). Só a 1.ª
  trilha `"motion"`; recuar = palco novo (`da_trilha`) + cursor novo + avançar (a lei do `MacroPlayer::seek`, à
  mão: o `seek(0)` do núcleo despacharia os passos do instante 0 com um aplicador vazio). Erro pára e fica
  guardado; recuar limpa-o. `ErroDaMacro::Ficheiro(CodecError)` novo. Gate
  `gate_tests::a_macro_em_cena_segue_o_relogio_e_recuar_e_tocar_do_zero_ate_la` (150 quadros ao bit; recuo ==
  tocar do zero até lá, com o controlo fim ≠ meio). Mutação `recomeca_se` nunca recua: sangra (14/10).
- ⚠️ **Controlo por agora sem efeito visível:** o «Macro Time» escreve o `MacroTempo` e ninguém em cena o lê
  até ao 5.5. A linha não fecha antes do 5.5.

## O que existe (não reconstruir)

- **Painel na instância** (`9ebdf44f13`): `MotionState.ao_painel: PedidosAoPainel { selecao, params }`
  (`motion_state_ao_painel.rs`; `pede_selecao`, `encaminha_param`). 12 sítios `request_graph_selection`
  (motion_bridge_{edit,rewire,heal,intents,subgraph,adapt}) e 5 `push_param_intent` (intents 3, choices 2;
  `write_source`/`write_channel` recebem `&mut MotionState`; `step_choice`/`pick_choice` passam a `&mut`).
  Só o editor publica: `intents::publica_no_painel(motion)` no fim do `apply_graph_intents` e no `dispatch`
  depois do `subgraph::clamp_level`. Leituras: `quadro::MaoDoEditor { selecionado, arrasto }` é argumento de
  `o_quadro(motion, gpu, playhead, dt, mao)`; `warp_gizmo::taps_for(motion, selecionado)` + `warp_de`,
  `pivot_gizmo::taps_for(motion, arrasto)` + `forma_com_pivot`. Censo: flash e `set_card_*` NÃO são
  alcançáveis do palco. Gate red-first `macro_track::painel_tests` (3, vistos vermelhos).
- **Mundo e placa** (`2e8e4403eb`): `motion_mundo.rs::publica_o_mundo(motion, Cena{…})` (saiu da shell
  `fase_motion_bridge.rs`, que encolheu ~25 linhas: três chamadas viraram uma) guarda
  `motion.mundo: MundoDaCena { externos, jogo, donos }`. `MotionState::recebe_o_mundo` e
  `TocadorDoMotion::recebe_o_mundo`. `TocadorDoMotion::aplica(carga, placa: Option<&GpuContext>)`; o palco
  nasce com o `gpu_enabled` do produto.
- **Marcas da bomba**: `MotionCookPump::sujidades()` (contador) no irmão novo
  `crates/ph2d-eval-motion/src/sujidade.rs` (mark_dirty / mark_dirty_keeping_ring / is_dirty movidos;
  `lib.rs` 697 → 665; `isola_na_camada` marca via keeping_ring). O passo `Quadro` renomeia `sujo` → `sujou`
  (etiqueta 5 mantida; «marcada desde o quadro anterior»; nenhum `MacroDoc` persistido existia). A
  testemunha grava `Toque.sujou` pela mesma contagem. Gates: `motion_mundo::tests` (2, cada um com o controlo
  sem o mundo; a cena usa duplicator+source.shape porque «uma corrente só de posições não desenha»,
  lower.rs); `macro_track::placa_tests` (2, `#[ignore]` placa): a sessão do gate 3 na placa toca no palco na
  placa AO BIT (`retrato_visto` lê `read_instances`/`read_formas`); o fiel atravessa placa↔CPU (vermelho
  visto: `Divergiu`). Mutações sangram: fiel que nunca acusa; testemunha sem a marca.
- **Alvo** (`02d083459a`): `MotionState::gravacao_abre(rec)` = arma + `open_track(FAMILIA,
  objeto.editing.unwrap_or(0), inicial)`; `TocadorDoMotion::da_trilha(track, modo)` →
  `palco.objeto.so_o_dono`; `sinks_visiveis` filtra pelo dono. Gate red-first `macro_track::alvo_tests` (2).
- **Componente** (`53b6fd7cff`): `ph2d_ecs::Macro { doc: Vec<u8>, modo: ModoDaMacro { Fiel, Explicativo } }`
  em `crates/ph2d-ecs/src/macro_doc.rs`; o `doc` é o `MacroDoc` pelo `ph2d_macro::codec` (cabeçalho
  `PH2DMACRO` major/minor). Descritor INTRÍNSECO em `ph2d-component-desc` `catalog/logic.rs` (`C::Animation`
  vizinho; nasce do Gravar▸Parar, a paleta não o oferece), campo 2 = modo (`K::Enum`). i18n
  `component.macro.name`, `component.field.macro_fields.2`, `ph2d-i18n/src/inspector_macro.rs`
  (`newer_version`, `damaged`). Modelo do cartão: `ph2d-editor-core/src/macro_edits.rs` (`MACRO_MODOS`,
  `MacroFieldEdit::Modo`, `MacroGravacao {Vazia, Lida{trilhas, duracao_us}, Ilegivel(chave)}`,
  `InspectorMacroInfo`) e `ph2d-app-components/src/macro_inspector.rs` (`build_info/apply/apply_all/
  gravacao`; dep nova `ph2d-macro`). Gate 8 verde; o descritor apagado reprova-o (visto).
- **Vermelhos** (`75afc54516`): contagens `ph2d-render` e `ph2d-script` `registry.rs` 108 → 109; censo das
  palavras do Inspector (`inspector_macro.rs` na lista; os nomes dos modos saíram — chave sem leitor — até a
  W5 os pintar).

## Foundational / partilhado tocado (para o integrador)

`ph2d-eval-motion` (`sujidade.rs` novo, campo `sujidades`) · `ph2d-ecs` (`Macro`, `ModoDaMacro`, registo) ·
`ph2d-component-desc` (descritor) · `ph2d-i18n` (chaves + ficheiro) · `ph2d-editor-core` (`macro_edits.rs`) ·
`ph2d-app-components` (`macro_inspector` + dep `ph2d-macro`) · `ph2d-panel-inspector` (só a lista do teste de
censo) · `ph2d-render`/`ph2d-script` (contagens) · `shells/desktop`: `fase_motion_bridge.rs` encolheu (delta
negativo), `project_schema(.rs/_tests.rs)`. 5.5 D: `ph2d-macro` (`MacroTime::from_secs_f64`) · `ph2d-tool-painter` (`runtime.rs::composto_da_cena`, `macro_track/em_cena.rs` novo) · `shells/desktop` (`App.painter_macros`, `fase_macros_do_painter`; +≈35 linhas com o gate). Contratos congelados encostados: **nenhum**.

**Contadores como delta:** `PROJECT_SCHEMA` 188 → 189 (escada + tripla `(189,13,22)`); registo ECS +1
(107 → 108; render/script 108 → 109). ⚠️ `python3 scripts/schema-recount.py` NÃO conta: só renumera um
degrau DENTRO de um conflito de rebase (lê `:1/:2/:3`). A conta da linha é o delta (+1 registado); o
integrador reconta com o script.

**O que um merge pode partir:** outra linha que some um componente ECS ou suba o schema (contagens de
`registry.rs`, escada); `fase_motion_bridge.rs` (as 3 chamadas viraram `publica_o_mundo`); quem chame
`request_graph_selection`/`push_param_intent` novos fora das portas da instância (o palco guiaria o painel);
quem escreva `Quadro.sujo`.

## Medido

**Suíte** (`PH2D_SUITE=5-commits`, HEAD `53b6fd7cff`): `20 839/20 843` em `659 s`, loadavg 33–59 (outra
linha corria a suíte). 3 vermelhos reais, curados em `75afc54516`. O 4.º, `ph2d-light2d-gpu
g4_o_custo_do_rc_nao_depende_do_numero_de_luzes` (8,25 % > 5 %), é flake de carga ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)):
verde sozinho a load 25.

**Integrada no main a 14/10** (`foundational-integrate.sh`, 4.ª corrida): suíte da árvore combinada
**`20 874 / 20 874`** (HEAD `c5c074ff92`) — esta é a última suíte VERDE da linha. As três primeiras reprovaram
no G4 da holográfica (curado na linha em `c5c074ff92`: a régua de um solver por contagem media o sítio dos
buffers desde a W5h `f9c8110287`; ver FLAKES_DE_CARGA) e num gate de placa da luz com a placa ocupada.

## Reaberta a 14/10 sobre o main novo (`0aab2c1cb6`, `reset --keep`: tudo já estava no main)

- **Regras novas do main** (vivem no `CLAUDE.md §2`, carregado — não se copiam para aqui): suíte só a cada
  10 commits ou `PH2D_SUITE=fecho`; no meio do trabalho, teste POR NOME (guarda R5); clippy só no fecho (R7);
  edição por script tem de provar que casou (R6).
- ⚠️ **Muda o desenho do item 5:** a aba Keys sem pilha passou a dirigir o relógio do EDITOR e o Motion segue-o
  (ADR-0115 R8 emenda 3; `fix(timeline)` + handoff ZERAR_132 do Motion). Ler isso ANTES de desenhar o
  `PropKind::MacroTime` e a limpeza por quadro abaixo — o relógio que a macro segue na Keys pode já ser este.

Smokes da placa (comando inteiro):
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke -- --ignored macro_track::placa`
(o resto: `bash scripts/cargo-test-narrow.sh ph2d-app-motion`). Smoke do Enio: ainda nenhum — o cartão pintado é W5.

## Premissas do briefing derrubadas pela medição

- O `TimelineDoc` não é componente: é `ProjectFile.timeline: Vec<u8>` com `DOC_VERSION` próprio; o «molde» é o blob com versão no cabeçalho.
- Autoplay/laço/duração NÃO entram no componente (do clipe na Timeline; duas verdades divergiriam).
- O `TocadorDoPainter` vive em `crates/ph2d-tool-painter` (o plano dizia `ph2d-app-painter`).
- Crates `ph2d-app-*` não dependem umas das outras (só `ph2d-app-registry-init`) ⇒ a «ponte do Play» não pode ser uma crate que chame Motion e Painter: cada família toca a SUA trilha lendo o componente.
- O cartão PINTADO no Inspector (`ph2d-panel-inspector` sections/sync/populate/event/ids + editor-core `action_bus` + shell `fase_bus_inspector`…) é W5 (molde `a777f2f541`); a W4 entregou o modelo.

## 14/10 (tarde) — 5.5 A+B+C FEITOS (`412aba04da`; não reconstruir)

- **A** `crates/ph2d-app-motion/src/macro_track/palcos.rs::PalcosDasMacros` (campo `palcos` do
  `MotionShellState`): `segue(world, &motion.mundo, Some(gpu)) -> BTreeSet<u64>` corre em
  `fase_motion_bridge.rs` entre `publica_o_mundo` e `dispatch`. Palco por entidade (bits), recriado só
  quando `get_change_ticks::<Macro>().changed` muda; a leitura faz `world.increment_change_tick()`
  (a shell não corre `Schedule` por quadro: sem isto dois quadros partilham o tique). `Macro.doc`
  vazio = `Ok(None)`; erro de leitura fica em `Lido.palco = Err` (sem leitor até à W5). Sem
  `MacroTempo` = em cache, não toca.
- **B** `ObjetoState.tocados_por_macro` (`0` = todos) em `sinks_visiveis`;
  `motion_objeto::tocados_por_macro` faz `mark_dirty_keeping_ring` quando o conjunto muda (a
  testemunha grava-o como `Toque`).
- **C** cada palco pelas portas do editor: `fase_vector_fx_recook` (os 3 assados), `fase_vector_bands`
  (`palcos.decide`, OR nas faixas do documento), `fase_vector_overlays` (`palcos.codifica` na cena da
  camada, `MotionFiltro::cena_da_camada_sempre`), `present.rs` (`palcos.liga()` =
  `MotionFiltro::liga_em_camada` + lente; `banded |= palcos`; `palcos.desenha` antes do halo),
  `present_fx` (halo por fonte: o bloco saiu para `ph2d_app_motion::motion_glow_passe::passe`),
  `present_chrome` (`palcos.pousa` = `filtra_a_camada` com pilha vazia, depois do editor e antes do
  chrome; `palcos.lentes` depois da lente do editor), `present_luz` (`palcos.com_oclusoes`). Uma camada
  por palco. Gizmos: só o editor.
- Gates: `macro_track::palcos_tests` (2), `motion_glow_passe::tests` (o gate de texto do halo, saiu de
  `present_placa_tests`), `present_bands::placa_tests` reescrito para N fontes +
  `os_palcos_das_macros_passam_por_cada_sitio_do_quadro`. Verdes também os gates que leem os ficheiros
  tocados (`skeleton_shell_seam`, `as_barras_de_vida`, `o_cerebro_fala`, `o_lod_da_forma`,
  `the_frame_partition`, `the_light_table`, `the_sink_style`, `queixas_cabem`, `grid_tests`) e
  `the_shell_only_shrinks`. Shell: +20 linhas líquidas (o código encolheu, o gate cresceu).
- ✅ **Prova de mutação 10/10 sangram** (14/10, HEAD `bb2584e678`; controlos verdes antes: F1
  `macro_track::palcos_tests` 2, F2 `a_macro_em_cena_segue…` 1, F3 `ph2d-host-desktop`
  `render_loop::present_bands::placa_tests` 4, F4 `motion_glow_passe::tests` 1 — este não guardou
  nenhuma). F1 apanha: tirar `!tocado` do filtro; tirar o `mark_dirty_keeping_ring`; tirar o
  `increment_change_tick`; `fresco` sempre; tirar o `retain`; `toca = true`. F2: `recomeca_se` nunca
  recua (fecha também a prova do gate do 5.4). F3: o laço do `present_fx` sem `passe`; tirar
  `palcos.lentes`; `palcos.segue` + `tocados_por_macro` depois do `dispatch`. Âncoras 1× cada (pré-voo),
  restauro `checkout` + `touch`, árvore limpa no fim.
- ⚠️ Não medido: custo do quadro com N palcos (placa); nenhuma cena de smoke ainda.

## 14/10 (noite) — 5.5 D FEITO (`dbf89c60db`; não reconstruir)

- `ph2d_tool_painter::macro_track::em_cena::MacroEmCena` (irmã da do Motion): só a 1.ª trilha
  `"painter"`; `segue(segundos)` (recuar = `TocadorDoPainter::novo` + `MacroPlayer::new` + avançar; erro
  guardado, recuar limpa); `alvo()` = `target` (`StableId` da sprite; `0` = a entidade do próprio `Macro`).
  `PainterTool::macro_abre(rec, alvo)` = irmã do `gravacao_abre` (sem chamador de produto: W5).
  `ErroDaMacro::Ficheiro(CodecError)` novo. `MacroTime::from_secs_f64` (ph2d-macro) é a conta única; o
  `instante` do Motion delega.
- **A tela da cena** = a MESMA composição do editor: `PainterTool::composto_da_cena` (pub(crate)) =
  `preview_dirty = true` + `take_preview_arc_corpo` (sem a porta, sem o portão). Medido: no editor uma
  pilha de várias camadas vai à PLACA (`painter_gpu_preview::try_drive` drena `take_preview_dirty`), logo
  a gravação pode não levar imagem nenhuma — copiar a composição (papel, vidro, relevo, máscara) seria uma
  2.ª lei. `TocadorDoPainter::toma_a_tela` recompõe só quando `mudou`: os drenos gravados `Preview`
  (CPU) e `PreviewSujo` (placa) e o `Para` (o fim assenta a água e nenhum dreno vem depois — o gate viu
  978 texels a faltar sem ele). Nasce `true` (a tela inicial mostra-se).
- `ph2d_app_painter::macro_track::PalcosDoPainter` (campo `App.painter_macros`, +2 linhas no
  `app_state.rs`): palco por entidade com `Macro`, renasce pelo tique (`increment_change_tick` como o do
  Motion), sprite resolvida por `entity_with_stable_id` a cada quadro em que toca; ranhura própria
  (`RanhurasDaCena`: o `SpriteRenderer`, um duplo nos gates), upload premultiplicado só quando a tela mudou,
  ranhuras de palcos que morrem/renascem devolvidas no `sobe` seguinte; falha de upload = toast
  `app.painter.painter_bridge_upload.preview_upload_failed`. Shell: `fase_macros_do_painter`
  (`fase_extract_inputs.rs`) chamada em `fase_frame_open.rs` DEPOIS do `fase_timeline_drain` e antes do
  `fase_sim_extract`, com as trocas À FRENTE das do editor (o extract usa o 1.º override por sprite: uma
  sprite que a macro toca mostra a macro).
- Gates: `ph2d-tool-painter` `macro_em_cena` (3: ler a cena não muda a reprodução, ao bit; recuar ==
  tocar do zero; a pista da placa acorda a cena e pinta o mesmo que a CPU), `ph2d-app-painter`
  `macro_track::palcos_tests` (3), `ph2d-host-desktop` `placa_tests::as_macros_do_painter_trocam_a_sprite_
  antes_do_extract_e_a_frente_do_editor`. Verdes também `the_shell_only_shrinks`,
  `architecture_workspace_file_loc_cap`, `architecture_no_orphan_source_file`, `file_loc_caps`.
  **Mutações 14/14 sangram** (os 3 sinais de `mudou`, o `preview_dirty` forçado, `toma_a_tela` sempre,
  nunca recuar, `toca = true`, as duas devoluções de ranhura, o alvo `0`, o tique, adquirir em vez de
  substituir, a ordem da junção e a da fase).
- Deps: `ph2d-app-painter` dev-dep `ph2d-macro` (Cargo.lock +1 linha).
- ⚠️ Não medido: custo do upload a 1024²/2048² por quadro com N palcos (só sobe quando muda; a
  composição 1024² ≈ 2,4 ms está no plano §6); a sprite resolve-se por varredura O(entidades) por palco
  que toca.

## 14/10 (fim) — a prova do bloco FEITA (não reconstruir)

Fecho (`0e11ec111d`): `PH2D_SUITE=fecho BASE=0aab2c1cb6 nextest-impacted` 18 470/18 474 (508 s, load
23–43). Os 4 vermelhos, curados no `0e11ec111d` e re-corridos verdes por nome: `fn_loc_caps`
(`main.rs::new` 212 e `fase_vector_overlays` 207 — do 5.5 C/D), `the_app_only_sheds_fields` (173
campos — o `App.painter_macros` do 5.5 D foi para o `AppGfx`: as ranhuras são do `renderer`),
`the_frame_hands_the_project_filter_to_the_motion_encoder` (o encode do editor recebe
`hero.project.image_filter` por extenso; os palcos ficam com `filtro`, que o gate
`os_palcos_das_macros_passam_por_cada_sitio_do_quadro` lê), `macro_portas::toda_porta_do_painter_tem_decisao`
(`macro_abre` nas INTERNAS). Clippy `--all-targets -D warnings` verde nas 4 crates (a MACRO do gate 3
voltou ao `gate_tests`: só os testes a leem); `cargo fmt` aplicado. Os dois de custo da máscara do
`ph2d-tool-painter` são da [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md): 3/3 sozinhos a load 11.
⚠️ A suíte INTEIRA não correu depois do `0e11ec111d` (só os vermelhos por nome) — o integrador corre-a.
Fotos refeitas depois do `0e11ec111d`: as duas cenas iguais; numa rajada a load ~25 a cena =2 parou ~5 s
em 6,48 s e a =1 1–1,5 s, e com a máquina calma (load 3) a mesma rajada avança regular do início à volta
do laço ⇒ carga, não defeito.

**Cenas** (`11c5c20ef1`): `PH2D_MACRO_SMOKE=1` (Splash, Painter) e `=2` (Motion nó a nó). As famílias montam o
objecto e gravam a macro por script pelas portas do editor; `shells/desktop/src/render_loop/macro_smoke.rs`
(chamado de `fase_app_scene_smokes.rs` depois de `animacao_smoke`) monta o clipe: `add_macro_clip`,
`set_active`, `set_active_loop_for(keys,[0,D])`, `set_autoplay`, `TimelineIntent::RunScene`; cada volta do
laço recua a macro (palco novo + avançar). A env var é roteada pela shell e NÃO está em nenhuma FAMILY (como
`PH2D_ANIMACAO_SMOKE`: os dois níveis são de duas famílias e `no_two_families_claim_the_same_router` proíbe
partilhar).
- `=1`: `ph2d_app_painter::macro_track::smoke::monta` (tela branca 1024², bg 2 como `PH2D_WETPAINT_SMOKE` — o
  plano dizia transparente; branco = o fundo da aguarela) + `ph2d_tool_painter::macro_track::roteiro::grava_o_splash(lado, alvo)`
  (4 pinceladas Wet Paint; meio e relógio da água fixo definidos ANTES de armar — `set_paint_media`/`set_wet_relogio_fixo`
  são portas INTERNAS, só dentro de `ph2d-tool-painter`, gate `nenhuma_porta_interna_tem_chamador_de_fora`; cor/tamanho
  pelas portas gravadas). Fiel, 4,47 s.
- `=2`: `ph2d_app_motion::macro_track::smoke` (objecto «Wind tutorial» sem nós do editor, Macro Explicativo, 23
  gestos de `roteiro::CENA`, um a cada 0,5 s, 14,5 s). `roteiro.rs`: o `MACRO` do gate 3 saiu de `gate_tests` (uma
  fonte; `gate_tests` re-exporta) e `CENA` é uma ordem visível à parte (a ordem do fixture `MACRO` deixava a tela
  vazia ~6,5 de 10,25 s: saída ligada aos 3,25 s e o vento levava tudo antes do arrasto).
- `motion_bridge_params_edit.rs::aplica_lote_de_params` (o dreno de params e a gravação por script partilham-na; o
  grafo deixa `SetParam` em `ao_painel.params`, que no editor vão pela fila global). Sem ela a cena gravava zero
  params (medido: grelha 3×3 por omissão = 9 cópias; tamanho da forma 1,0).
- Gates: `smoke_tests::a_macro_da_cena_coze_formas_e_o_vento_as_move` (os dois modos; 60 cópias; o vento move-as);
  `roteiro::tests::o_splash_tocado_pinta_a_tela_e_o_inicio_e_branco` (`819e04a6bb`).

**Defeito achado pela cena (5.5)** (`86192e1559`): `TocadorDoMotion::da_trilha`
(`crates/ph2d-app-motion/src/macro_track/tocar.rs`) só punha `so_o_dono`; sem `objeto.editing` o `adopt_new` do
palco deixava órfão todo nó CRIADO durante a gravação e o filtro de dono (`sinks_visiveis`) largava-o — uma macro
que constrói o grafo de um objecto não desenhava nada (cena `=2`: `sinks=0`). O gate do alvo só usava nós
existentes ao armar. Gate novo, visto vermelho primeiro: `alvo_tests::o_no_criado_durante_a_gravacao_e_do_objecto_da_trilha_no_palco`
(`None ≠ Some(11)`). `motion_bridge_intents.rs`: `aplica_e_grava` (lote de grafo aplicado+gravado) saiu de `apply_graph_intents`.

**fotografa_cena.sh** (`037a08b75b`): (a) o HOME isolado levava o cache de shaders da NVIDIA para um tmp vazio —
cada foto recompilava tudo (o 1.º quadro de um palco do Motion parava 25 s em `libnvidia-gpucomp`, visto com gdb;
as fotos pareciam congeladas); agora `__GL_SHADER_DISK_CACHE_PATH`/`MESA_SHADER_CACHE_DIR` = o cache real do runner;
(b) um comentário DENTRO do heredoc tinha crases e executava `[env]` e `.cargo/config.toml` a cada foto (o guarda
só vê o script expandido); (c) `FOTO_RAJADA=<n> FOTO_INTERVALO=<s>`: n fotos na mesma sessão (`<saida>_02.png`…).

**Custo com N palcos** (`b2c5ba08b3`, `#[ignore]`, placa; método MEDIR_VELOCIDADE: 7 rondas × blocos de 20
intercalados, MÍNIMO com a mediana ao lado, GPU sincronizada no fim do quadro; o Painter repete o MESMO segmento
por bloco com recuo não cronometrado porque a água gravada chega em lotes — o mínimo por quadro dava ~0). Sondas:
`crates/ph2d-app-motion/src/macro_track/palcos_custo_tests.rs::sonda_n_palcos_do_motion`,
`crates/ph2d-app-painter/src/macro_track/palcos_custo_tests.rs::{sonda_n_palcos_do_painter,sonda_o_quadro_mais_caro_do_splash}`.
Comando (por crate, `cd` na worktree): `PH2D_GPU=1 __GL_SHADER_DISK_CACHE=0 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke --lib -- --ignored --nocapture macro_track::palcos_custo` (e `-p ph2d-app-painter`).
Medido 14/10, perfil smoke, load 11–55:

| | N=1 | N=4 | N=16 | por palco |
|---|---|---|---|---|
| Motion (cena `=2` desde 11 s), ms mín. (med.) | 0,336 (0,423) | 1,100 (1,338) | 4,282 (5,719) | ≈ 0,27, linear |
| Painter 1024², ms mín. · med. | 4,264 · 5,809 | 16,125 · 21,241 | 61,983 · 89,802 | 4,26 · 4,03 · 3,87 |
| Painter 2048², ms mín. · med. | 11,059 · 16,959 | 42,814 · 57,593 | 170,652 · 222,852 | 11,06 · 10,70 · 10,67 |

- Painter, segue+sobe ao mínimo: 1024² N=1 2,378+1,887 · N=16 39,564+22,419; 2048² N=1 5,006+6,054 · N=16 78,066+92,586.
  Só custa enquanto a tela muda.
- Motion: 1.º quadro a saltar 0→11 s = 338 ms (N=1), 2210 ms (N=16) — o explicativo re-simula da semente.
- Gravar: 1024² 1111 ms (doc 4114 KiB); 2048² 2362 ms (16402 KiB). Pior quadro do Splash 1024²: 30–45 ms
  (t=3,783 s 44,8 ms, segue 40,0) — repetição da água em lote; recuo do laço 13,5 ms.
- ⇒ **NENHUM tecto escrito** (CLAUDE.md §0.0): o recurso é a CPU da thread principal (repetir a água +
  premultiplicar/subir); a 60 Hz ≈ 3 palcos Painter 1024² ou 1 a 2048² a pintar; Motion ≈ 50 palcos de cook. O
  desenho de cada palco do Motion (camada, filtros, present) NÃO está na sonda — o relógio da app: cena `=2` a
  61 fps com 1 palco (load ~66).

**Prova de mutação 14/10: 7/7 sangram** (controlos verdes: F1 `ph2d-app-motion` alvo_tests+smoke_tests+gate_tests,
10 testes; F2 `o_splash_tocado`, 1; âncoras pré-voadas 1× cada): M1 `editing` fora do `tocar` → alvo_tests:116 +
smoke_tests:32 · M2 lote de params saltado na cena → smoke_tests:34 (9≠60) · M3 `macro_param` fora de
`aplica_lote_de_params` → 5 gate_tests + smoke_tests (gate_tests:502, o grafo perdeu as linhas de params) · M4
`macro_lote_de_grafo` fora de `aplica_e_grava` → 8 testes · M5 linhas de `CENA` 6→3 → smoke_tests:34 (30≠60) · M6
cor do pincel fora → roteiro.rs:147 (azul 0, vermelho 0) · M7 sem passos gravados → roteiro.rs:147 (pintou 0).

**Fotografias** (`fotografa_cena.sh` com rajada; folhas de contacto da tela + campo Time da Timeline): `=1` certa
em todos os instantes, incl. depois da volta do laço (0,233 s: só o início da 1.ª pincelada). `=2`: vazia até
~3,5 s · grelha 3×3 a 4,3 s · 6×10 espalhada a 5,5 s · compacta a 6,2 s · ~5 s sem mudança visível (integrar/arrastar/caixa/vento
acrescentados) · a 11,7 s o vento leva só a coluna dentro da caixa e as formas empilham à direita · o último gesto
alarga a caixa · volta do laço → grafo vazio a 0,07 s. Numa rajada a load 54 viu-se uma paragem de ~2 s perto do fim do
laço de `=1` (Time preso em 4,033 durante 4 fotos), NÃO reproduzida sem janela (pior 45 ms) ⇒ atribuída a carga/outras
linhas na placa; não provado.

Binário do smoke JÁ compilado (2.ª corrida, sem `Compiling`):
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` →
«Finished `smoke` profile [optimized] target(s) in 0.47s». Smoke do dono:
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=2 ./target/smoke/ph2d-host-desktop`
(e `=1`).

## ABERTO — o que falta (5.6 e o que a medição abriu)

1. **5.6** Da W3/W2: recusas/erros chegam ao artista pela UI da W5 (`MacroEmCena::erro`); os outros meios do
   Painter; re-medir o custo do quadro com a máquina calma.
2. **Perf aberta pela medição (não é defeito):** picos de 30–45 ms na repetição da água do Painter quando a água
   gravada chega em lotes (compor UMA vez por quadro / espalhar a repetição); premultiplicar+subir ≈ 1,9 ms a
   1024² e 6 ms a 2048² por palco, na CPU (premultiplicar na GPU ou subir só o rect sujo); salto/scrub do
   explicativo do Motion = 338 ms até 11 s (instantâneos — o aberto do plano «Memória dos instantâneos para scrub
   para trás»).
3. **Cena `=2`:** ~5 s sem mudança visível enquanto a cadeia de forças se constrói — o editor de nós não mostra o
   grafo do palco (W5: o chrome do tutorial / cena `=3`).
4. **Compilação fria dos shaders** dos pipelines do palco do Motion: ~17–25 s com cache frio (UI parada) —
   propriedade antiga da família Motion, agora no cache real.

## Régua do laço do agente

`bash scripts/agent-loop-profile.sh` (20 sessões mais recentes, 13/10):

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                228   alvo: <= 800
  ✗ cargo test : cargo check                724 : 153   alvo: <= 1,0  razao 4.7x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  47%   alvo: >= 80%  (702 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         358 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```
