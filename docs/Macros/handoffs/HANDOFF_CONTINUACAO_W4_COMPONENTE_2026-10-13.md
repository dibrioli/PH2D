# Continuação — line/Macros-Automação, W4 itens 1–4 e 5.1–5.4 fechados → item 5.5 (a cena: palcos, exclusão, present de N fontes, Painter)

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
  tocar do zero até lá, com o controlo fim ≠ meio). ⚠️ Prova de mutação deste gate por fazer (no fecho).
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
negativo), `project_schema(.rs/_tests.rs)`. Contratos congelados encostados: **nenhum**.

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
- ⚠️ **Prova de mutação por fazer** (controlos verdes: F1 `palcos_tests` 2, F2 `a_macro_em_cena_segue`
  1, F3 `present_bands::placa_tests` 4). As 10 planeadas: tirar `!tocado` do filtro; tirar o
  `mark_dirty_keeping_ring`; tirar o `increment_change_tick`; `fresco = true`; tirar o `retain`;
  `toca = true`; `recomeca_se` nunca recua (F2); tirar o laço de palcos do `present_fx`; tirar
  `palcos.lentes`; mover `palcos.segue` para depois do `dispatch` (F1/F3). O subagente `mutacao` teve
  a ferramenta Edit recusada pelas permissões e não a contornou.
- ⚠️ Não medido: custo do quadro com N palcos (placa); nenhuma cena de smoke ainda.

## ABERTO — item 5.5 D (o Painter em cena), a prova do bloco, e 5.6

**Mapa do D medido a 14/10 (explorador, endereços do HEAD `412aba04da`)** — o desenho D abaixo fica
corrigido por isto:
- `TocadorDoPainter` (`ph2d-tool-painter/src/macro_track/tocar.rs:30`): `novo(inicial, cargas)` `:41`,
  `sobre_tela(inicial, cargas, rgba, w, h)` `:54`, `aplica(carga)` `:106` (um passo, sem relógio, só
  avança). Resultado: `tool().canvas_rgba` (straight; `take_preview_arc`, `tool/runtime.rs:338-401`).
  Visibilidade de `canvas_rgba` fora da crate e dependência do worker da água: por confirmar.
- Gravação: `PainterTool::macro_grava() -> Result<(Vec<u8>, (u32,u32)), Recusa>` (`mod.rs:61`) sem alvo
  e sem chamador de produto (só testes); falta o irmão de `MotionState::gravacao_abre`
  (`ph2d-app-motion/src/macro_track/mod.rs:126`) com o `StableId` do sprite.
- ⭐ **O ponto de troca JÁ EXISTE**: `PreviewOverride { entity_bits, texture_id, premultiplied }`
  (`shells/desktop/src/render_loop/sim_extract.rs:175`, aplicado em `:331-334`); a lista monta-se em
  `fase_extract_inputs.rs:77-115` ao lado de `painter_preview_override` (`:90`, o traço em curso:
  `painter_bridge_upload.rs:76` sobe para um `PainterPreviewGpu`) e do `bgremoval_preview_override`.
  ⚠️ `find` apanha o primeiro override da entidade.
- Renderer: `acquire_individual(w,h,rgba)` (`renderer_individual.rs:22`), `replace_individual_pixels`
  (`:198`), `individual_mut().release(id)`; `Rgba8UnormSrgb`, pixels PREMULTIPLICADOS.
- Casa do sistema: `crates/ph2d-app-painter/src/macro_track/` (novo; deps já lá: `ph2d-tool-painter`,
  `ph2d-ecs`, `ph2d-render`, `ph2d-preview-slot`; falta `ph2d-macro`). Não há `PainterShellState`: o
  estado do Painter vive em parâmetros de `fase_painter_dispatch.rs`. Corre antes do
  `fase_extract_inputs` (a Timeline já escreveu o `MacroTempo`, `ph2d-timeline/src/apply_prop.rs:112`).

O desenho abaixo foi medido por leitura a 14/10 (dois exploradores; endereços do HEAD `1f4e85547b`).
⛔ **Excluir o dono sem desenhar o palco é pior que não existir** (o objecto sumiria no Play): A, B, C e D
entram juntos.

**A — o sistema (crate `ph2d-app-motion`, nunca a shell).** `PalcosDasMacros` em `MotionShellState`
(`motion_shell_state.rs:51`, ao lado do `placa: PlacaDeFormas`, que é estado POR FONTE e hoje singular):
`BTreeMap<bits, PalcoEmCena { macro: MacroEmCena, placa: PlacaDeFormas, tick }>`. Por quadro, depois de
`publica_o_mundo` e antes do `dispatch` do editor (`fase_motion_bridge.rs:48`→`:72`): para cada entidade com
`Macro` + `MacroTempo`, (re)cria o palco quando o `Macro` mudou (o `ComponentTicks::changed` do bevy, O(1) —
nunca comparar os bytes por quadro), `segue(segundos, &motion.mundo, gpu)`; palco sem `MacroTempo` neste
quadro sai do conjunto de quem toca (mas pode ficar em cache para não recozer do zero). O dono do palco é o
`StableId` da entidade (o `target` da trilha é `StableId`, `macro_doc.rs:36`; `objeto.editing` também), logo a
macro sobrevive a salvar e reabrir.

**B — o editor exclui.** Hoje NÃO há mecanismo: `sinks_visiveis` (`motion_objeto.rs:475-488`) só filtra o olho
(`escondido`) e o `so_o_dono` do palco. Acrescentar `ObjetoState.tocados_por_macro: BTreeSet<u64>` (escrito por A
a cada quadro) e `!tocados.contains(dono)` no filtro. Gate red-first: com um palco a tocar, o editor não coze os
sinks do dono; sem `MacroTempo`, coze.

**C — o present desenha N fontes pelas MESMAS portas.** Cada palco É um `MotionState`, logo toda porta que
recebe `&MotionState` serve; o que é por fonte é o `PlacaDeFormas`. Costura: um iterador
`motion_shell.fontes(&editor) -> (editor, &placa) + palcos` e um laço em cada sítio (todos hoje globais):
`fase_vector_bands.rs:123` (`decide_o_quadro`), `present.rs:448` (`desenha_o_quadro`), `present_chrome`
(`cola_no_mundo`), `present_fx.rs:36,98` (`halo_do_quadro`; o glow lê `doc.graph` — por fonte), `present_luz.rs:39`
(`OclusaoDoMotion::do_quadro` — os bloqueadores SOMAM entre fontes), `fase_vector_fx_recook.rs:160-260`
(tiles/LOD sobre `pump.*`), `fase_vector_overlays.rs:63-71` (rota CPU, `motion_shape_gen::encode`), gizmos
`fase_motion_gizmos.rs:49-85` (só o editor — o palco não tem mão). ⚠️ Os gates `present_placa_tests.rs` são
`include_str!` sobre o fonte: as strings que afirmam (`plan.banded |= self.motion_shell.placa.ativa();`,
`decide_o_quadro(`, `doc_bands_of(frame_order, placa || luz_acesa)`, `if motion_tool_active &&
!self.motion_shell.placa.ativa() {`, `cola_no_mundo(placa, gpu, band_blit, alvo);`, `halo_do_quadro(g.motion,
g.formas.desenhou())`) mudam com o laço — reescrevê-las para a forma N, nunca apagá-las. ⚠️ Tectos: a shell só
encolhe (`the_shell_only_shrinks`, teto 189 041 + folga 4 000): o laço vive numa função da crate e a shell
troca uma chamada por outra; `app_state.rs` já tem 923 linhas (não lhe acrescente campos — o estado vai no
`MotionShellState`).

**D — o Painter em cena.** O `TocadorDoPainter` (`ph2d-tool-painter/src/macro_track/tocar.rs:41`) é CPU, só
avança (recuar = `novo` + reaplicar); o resultado é o `canvas_rgba` do `PainterTool` próprio. O objecto-imagem é
um `Sprite` com `SpriteSource::Individual { texture_id }`; override sem tocar no ECS = `replace_pixels` num id
PRÓPRIO do palco (`individual_read.rs:128`; `acquire` em `individual.rs:190`) e trocar o id na aparência do
quadro (`appearance_of`, `motion_bridge_objects_appearance.rs:~60-78`, ou o passe de sprites `present.rs:166`).
⚠️ Falta à trilha do Painter o `target`: `macro_grava` (`ph2d-tool-painter/src/macro_track/mod.rs:61`) não
recebe alvo (no produto ninguém o chama — W5); dar-lhe o `gravacao_abre` irmão do Motion. Custo: upload RGBA por
quadro só quando o tocador avançou; medir 1024²/2048² com a máquina calma (o número da W2 foi a load 61–66).

**Prova do bloco:** a cena de smoke (formas + simulação, `CLAUDE.md §0.8`) com uma macro gravada por script
(o `sessao()` dos gates) e um clipe `add_macro_clip` com Autoplay; fotografar Play e scrub para trás.
Medir o custo do quadro com N palcos (placa) antes de escrever qualquer limite.

**5.6** Da W3/W2: recusas/erros chegam ao artista pela UI da W5 (`MacroEmCena::erro`); os outros meios do
Painter; re-medir o custo do quadro a tocar com a máquina calma.

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
