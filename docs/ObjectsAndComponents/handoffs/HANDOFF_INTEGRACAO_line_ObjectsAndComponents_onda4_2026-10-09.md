# HANDOFF DE INTEGRAÇÃO — `line/ObjectsAndComponents`, onda 4 (09/10): tudo o que ficou aberto (clipe de entrada, espaço do objecto Motion, cemitério, gizmo com cisalhamento)

> Leitor: o agente integrador (e a próxima janela desta linha). Plano e leis:
> [`01_objeto_e_componente.md`](../01_objeto_e_componente.md) §14. Ondas anteriores:
> [onda 1](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_2026-10-08.md) ·
> [onda 2](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda2_2026-10-08.md) ·
> [onda 3](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda3_2026-10-08.md). **As QUATRO ondas vão
> juntas na mesma branch, nenhuma está no `main`** — integre-as num só passo. ⚠️ A onda 4 toca
> ficheiros da `line/motion-value` (viva): §4.

## 1. Identidade

- Branch `line/ObjectsAndComponents` · worktree `Worktrees/line-ObjectsAndComponents` · base (merge-base)
  `2a925859d` (= o `main`; não andou, sem rebase). Ondas 1–3 já na branch (36 commits); onda 4 por cima de `ad434318e`.
- Commits da onda 4: `efff5a92e` (item 2) · `a24eac157` (4) · `74960e98c` (3 + 6) · `65de09a64` (fixtura v19,
  gravada ANTES da mudança) · `d224861fb` (7) · `3a98cde37` (8) · `192433d41` (1, núcleo) · `56b034d51` (saem
  `Dono.pose`/`pai_em_edicao`) · `f6ea120c2` (9) · `e34f9d17c` (5) · `86a8b248d` (10a) · `3ae8e50ac` (10b) ·
  `9ff6974e7` (1, interruptor Autoplay) · `9e32e6388` (smoke `=3`) · `32db9e454` (1, verbo Play Clip) ·
  `a6ec3803f` (1, curas) · `57cf9c357` (fmt) · `5f25256c7` (`pose_de_mundo` + gate) + os commits deste handoff.
- Diff da onda 4 (`ad434318e..HEAD`): 152+ ficheiros, ~`+5100 −656`. **Shell: `+126 −95`**
  (`the_shell_only_shrinks` verde; shell 185 744 de 189 041).

## 2. O que entrou

Escolhas do DONO (09/10, doc 01 §14): **4.1** clipe de entrada por Animação (o *Autoplay* do Godot) · **4.2**
o mundo é convertido sozinho para o espaço do objecto Motion · **4.3** caminhos ficam onde estão (nada a construir).

- **Item 1 — clipe de entrada (Autoplay).** `NamedClip.autoplay` (apêndice ⇒ `DOC_VERSION` 19→20);
  `doc_v19.rs` = imagem congelada + migração (nenhuma marcada ⇒ a 1.ª), provada contra
  `crates/ph2d-timeline/tests/fixtures/timeline_doc_v19.{bin,json}` (tests/it/clip_entry.rs). API em
  `crates/ph2d-timeline/src/doc_entry.rs`: `entry_clip_of`, `set_entry_clip` (rádio; o Duplicate não rouba),
  `arranged_owners` (a Cena dirige quem arranjou, recursivo nos containers), `autoplay_entries`,
  `clip_local_time` (loop do Keys dobrado por `ClipStrip::fold`; corte de duração vale).
  `ph2d_ecs::ClipPlayback{clip,desde}` = execução, **NÃO registado** (não grava, não entra no undo).
  `ph2d_timeline::apply_autoplay` + `apply_one_clip_except` (respeita o skip/a mão do artista): pedido > entrada;
  pedido do futuro (rewind) LARGADO; clipe inexistente largado e DITO (`ClipRefusal::UnknownClip`, toast).
  Shell: `fase_sequences::toca_os_objectos` (ex-`toca_as_cutscenes`) toca a Animação de cada objecto e depois
  as cutscenes, com UM censo do ledger do undo. ⭐ **A Animação de cada objecto só toca no PLAY ALL**
  (`acd61634c`, dono no smoke de 09/10: *«se o Arrange ainda não tem nada, como pode tocar alguma coisa? Um
  segundo Play na Timeline para Play All»*): o Play de sempre toca o que a aba mostra (Keys: o clipe; Arrange:
  só o arranjado + cutscenes, como antes da onda 4); o Play All toca a cena inteira no relógio da CENA, em
  qualquer aba, e acaba quando esse relógio pára. `TimelineState.play_all` · `TimelineIntent::PlayAll` drenado
  por `ph2d_timeline::take_play_all` ANTES de o dreno escolher o relógio da vista (com ele ligado a vista
  avaliada é a da cena) · `toca_os_objectos(.., (t, refusals, play_all), ..)` · botão «Play All» ao lado do
  transporte (`transport_row.rs`, saído do `transport.rs` que estava em 599/600; acende `Accent`) · linha
  «Play All» no menu Run (`TOPBAR_PLAY_ALL` → `TransportCmd::PlayAll` → a shell entrega o intent; `OWN_ROWS`
  8→9). A barra de espaço continua o Play normal. UI: interruptor «Autoplay» no grupo de clipes (`transport_entry.rs`, id
  `TIMELINE_CLIP_AUTOPLAY` em `ids/timeline_entry.rs`, i18n `panel.timeline.autoplay`): rádio — no clipe que já
  é a entrada pinta-se ligado e MORTO (Disabled, sem registo de clique); nos outros emite
  `TimelineIntent::SetEntryClip` (um passo do undo da timeline); só na vista de um OBJECTO
  (`AnimationHeader.entry`). `toggle_vivo` generaliza o pintor de toggles; o rótulo entra na família medida por
  `toggle_label_w`. Verbo de sinal `SignalVerb::PlayClip` (apendado, tag 14, `ArgKind::ClipName`; postcard lê
  enum apendado ⇒ `PROJECT_SCHEMA` NÃO sobe; tag fixada na tabela GRAVADAS 14→15): põe/substitui
  `ClipPlayback{clip, desde: t}` nos alvos; `signal_actions_bridge::apply` ganhou `t: f64` (a shell passa
  `self.playhead.time()`, gate textual em `fase_signal_outbox_tests`); i18n `ecs.signal_verb.play_clip`,
  `panel.inspector.actions.clip_name`. `ph2d-app-components` ganhou dev-dependency `ph2d-timeline`
  (aresta interna no `Cargo.lock`).
- **Item 2 — nomes de clip únicos POR animação** (`doc_owner.rs:37 same_animation`; `clip_named(owner,name)`);
  o osso inteligente resolve dentro de `animation_of` (`smart.rs clip_of`); osso cuja animação não tem o nome
  fica inerte (nunca o homónimo de outro dono); a cópia de Prefab mantém «andar».
- **Item 3 — cemitério.** `ph2d_ecs::Graveyard<D>` (`crates/ph2d-ecs/src/graveyard.rs`, `GRAVEYARD_DEPTH = 256`
  = `UNDO_CAP`, gate `the_graveyard_is_as_deep_as_the_global_undo` em `shells/desktop/src/undo_tests.rs`).
  A purga (timeline `purge_the_dead`, Motion `mortes`) empurra o passo na fila própria e ENTERRA-o com os
  `StableId` dos donos; quando o undo global devolve os ids e esse passo ainda é o TOPO da fila (nada editado
  depois), a fila desfá-lo sozinha (`take_top_if`, sem redo) no mesmo quadro ⇒ UM Ctrl+Z devolve objecto +
  clipes + pistas (+ o documento que o reset do último objecto animado reiniciou) + nós. Editado entretanto ⇒
  a cova fecha-se e ficam os dois passos de sempre. Timeline: `TimelineHistory.graveyard`, `ressuscita` antes
  da cura no upkeep; Motion: `ObjetoState.cemiterio`, `ressuscita` antes das `mortes`, `Manutencao.ressuscitados`.
  Recusado: «passo ligado» entre filas (parte quando uma fila é desfeita fora de ordem).
  ⭐ **E o Ctrl+Z que encontra a purga enterrada no topo é do undo GLOBAL** (`8d249c881`): com a timeline
  aberta a fila dela consumia-o (repunha os clipes de um dono morto, a purga tirava-os — laço) e, com a
  ferramenta Motion, a fila do Motion repunha os nós e a migração dava-os a OUTRO objecto — o objecto nunca
  voltava (defeito anterior à onda 4). `Graveyard::holds`, `TimelineHistory::top_is_buried` (o
  `keyboard_timeline` larga a tecla), `MotionHistory::top` + `motion_objeto::undo_e_do_global` (o
  `keyboard_cadeia` manda-a DIRETO ao `undo_or_redo`, sem passar pela timeline). Portão dos ganchos:
  `a_buried_purge_sends_ctrl_z_to_the_global_undo`.
- **Item 4 — recusa visível** de Add Clip/Duplicate acima do tecto (`ClipRefusal::{AnimationFull, DocumentFull}`,
  i18n `timeline.clip.animation_full`/`document_full` com `{max}`; `TimelineState.refusals` drenado por
  `ph2d_panel_timeline::speak_clip_refusals`); verificada ANTES da edição (nenhum undo vazio); o `+` não fica
  desabilitado (botão apagado que despacha é defeito; um que não despacha calaria a recusa).
- **Item 5 — ▾ do Arrange** lista os clips de TODAS as animações agrupados por dono (cabeçalhos não
  clicáveis), com rolagem (`transport_sources.rs`, `AnimationHeader.sources: Vec<SourceGroup>`, faixa de ids
  `timeline_source_opt(i)`, `TIMELINE_SOURCE_ROWS = 131 072` = 65 536 clips (u16 de um strip) + um cabeçalho
  por dono; tabela da medição ao lado da constante, gate de unicidade). Escolher um clip de outro dono escolhe-o
  como fonte da lane; o `+` da lane põe-no no Arrange.
- **Item 6 — mundo → espaço do objecto Motion.** O cozido é global e o nó não sabe o dono ⇒ porta única
  `ctx.external`: `Cook::set_spaces` (nó → `StableId` do dono fora da origem; `cook_espacos.rs`),
  `external::in_space(nome, sid)` (separador U+001F), `EvalCtx.space`; o nó de um espaço lê a cópia e regista
  as DUAS chaves (a cópia aparecer recozinha); `Fingerprint.space` (mudar de dono recozinha). Sem dono
  deslocado: mapa vazio, nada copiado, um nome lido — o caminho de sempre ao bit.
  `motion_objeto::entradas::publica` no fim de `motion_externals::publish_all`: pontos (`$cursor`, `$at:`,
  `$curve:`) por A⁻¹; vista (`$camera`: P, size/s, zoom·s, s = √|det L|); pose (`$pose`: M = L⁻¹·R·diag
  decomposta). `position_of`/`pose_of` publicam a pose de MUNDO (`pose_de_mundo` = `world_transform`; antes o
  Transform local). Aparência/colisores da aparência/tabelas não dependem do espaço (lidos crus). Uma conversão
  serve CPU e placa: todo leitor de externo é nó CPU; censo `a_placa_nao_le_o_mundo_por_outra_porta` prova que
  o `ph2d-gpu-cook` não lê a tabela. Smoke `PH2D_MOTION_OBJETO_SMOKE=3`: o íman do Enxame segue o rato.
- **Item 7 — gizmo de campo exacto.** `GizmoView.model: [f32;6]` (identidade ⇒ byte a byte o de antes); pintura
  câmara·model·p, hit pelos cantos pintados; arrasto pela inversa (`GizmoCamera::screen_in_model`), motor no
  espaço local; Down movido da shell para a crate (`field_gizmo_down.rs`); cantos unificados em
  `gizmo/paint_geom.rs` (`paint.rs` 695→508). 10 construtores de `GizmoView` actualizados (flip 3, vec 1,
  editor-core testes 2, shell 3, campo 1). Saíram `Dono.pose` e `motion_objeto::pai_em_edicao`.
- **Item 8 — legendas das demos.** `Caption.no: Option<NodeId>` (dono resolvido por quadro via `owner_of`; a
  mesma afim dos sinks — `afim_do_no`/`dono_do_no` em `motion_objeto_origem.rs`); `monta_cena` é a porta única
  (roteador, testes, smoke); o olho do dono esconde as legendas dele.
- **Item 9 — ▾ do filtro da Hierarquia rola.** Porta nova da fundação
  `ph2d_editor_core::panel::paint_dropdown_popover_scrolled_with_hits` (`panel/dropdown_popover.rs`; em
  `widget/` reprovava `the_foundation_modules_form_a_dag` 43→45), `WidgetStore::publish_dropdown_popover`,
  `DROPDOWN_SCROLLBAR_ID` (sem id novo); o Inspector passou a usar a mesma porta; `MockPanelHost::wheel_at`
  no testkit.
- **Item 10.** (a) `InspectorReveal` casa pela ENTIDADE (`entity: u64` bits; o Inspector não tem `StableId`
  ao pintar); `HierarchyEntity.entity: Option<u64>` cheio em `shells/desktop/src/hero_bridge.rs`.
  (b) `pending_layer` largado no `leave()` e quando `bind_document` liga outra entidade; medido: um Paint
  RECUSADO não chega a escrever a camada (`mode_drive` só chama `open_content` com o modo entrado).

## 3. Versões e contadores (DELTA da onda 4)

- `DOC_VERSION` da timeline **19 → 20** (contra a base `2a925859d`: **18 → 20**). `PROJECT_SCHEMA` **187, sem diff**.
- Registos de componentes: **SEM delta nesta onda** (`ph2d-ecs` 106 / `ph2d-render` 107 / `ph2d-script` 107
  são da onda 3; `ClipPlayback` não é registado). `ObjectKind` sem diff. Contratos congelados intocados.
  Nenhum ADR. `SignalVerb` +1 (tag 14), tabela GRAVADAS 14→15.

## 4. Superfície de colisão — o que um merge pode partir

| sítio | mudança |
|---|---|
| `GizmoView` | campo `model: [f32;6]` — literal sem `..Default` reprova |
| `NamedClip` | campo `autoplay` |
| `Fingerprint` / `EvalCtx` | privados, campo `space` |
| `Cook` | `set_spaces` / `clear_spaced_externals` |
| `TimelineHistory.graveyard` · `ObjetoState.cemiterio` | campos novos |
| `Dono` | perdeu `pose` · `motion_objeto::pai_em_edicao` saiu |
| `Caption.no: Option<NodeId>` | campo novo |
| `TimelineIntent::SetEntryClip` · `ClipRefusal::UnknownClip` (+`AnimationFull`/`DocumentFull`) | variantes novas |
| `signal_actions_bridge::apply(.., t)` · `SignalVerb::PlayClip` | assinatura / variante |
| `toca_os_objectos(world, doc, (t, refusals, play_all), drive, skip)` · `apply_autoplay(.., refusals)` · `TimelineState.play_all` · `TimelineIntent::PlayAll` · `TransportCmd::PlayAll` (variante nova — `match` exaustivo noutra linha reprova) · `TimelineViewSnapshot.play_all` | assinaturas |
| `InspectorReveal.entity` · `HierarchyEntity.entity` | campos novos |
| `transport_clips::width/cluster` | levam a coluna de rótulos |

- **Colisão com a `line/motion-value` (viva, `e050e96d5`, mesmo merge-base):** na onda 4 só `Cargo.lock`
  (regenerar, **nunca à mão**) e `crates/ph2d-app-motion/src/motion_state_demo_router.rs` (**687/700 LOC
  aqui** — atenção ao tecto SOMADO na fusão). As da onda 3 continuam (handoff da onda 3 §4).
- **Shell (composição):** `+126 −95` na onda 4; a catraca soma entre linhas (ondas 1 +35, 2 +194, 3 +61).
- **Ids/i18n novos:** `TIMELINE_CLIP_AUTOPLAY`, `timeline_source_opt(i)`; `panel.timeline.autoplay`,
  `timeline.clip.{unknown_request,animation_full,document_full}`, `ecs.signal_verb.play_clip`,
  `panel.inspector.actions.clip_name`.

## 5. Portão de fecho (batched sobre o diff acumulado, 09/10)

- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` ✓ 52 s · `clippy --workspace
  --all-targets -D warnings` ✓ 46 s · `fmt` ✗ (ordem de um `#[path]` em `doc.rs`) curado em `57cf9c357` ✓ ·
  `machete` ✓ · `check-standalone-optional` ✓ · `check-workflow-packages` ✓ (32/372).
- Censos da árvore combinada: 12/12 censos, 114/114 testes ✓. `nextest-impacted` **18974/18974** ✓ (104,7 s,
  load 23→36). `PH2D_GPU=1 cargo test -p ph2d-gpu-cook dono -- --ignored` 1/1 ✓.

- Depois do Play All (`acd61634c`): `check --workspace --all-targets` ✓ · clippy `--workspace --all-targets -D warnings` ✓ · fmt ✓ · `nextest-impacted` **18980/18980** ✓ (108 s); 4/4 mutações do Play All sangraram (o toggle sem `pause`, o fim pelo relógio, o `play_all &&` da fase, a ponte bus→timeline).

## 6. Provas de mutação (27/27 SANGRARAM nas leis da janela principal + as dos agentes)

- Janela principal (27 — as 4 últimas: `holds`, `top_is_buried`, `undo_e_do_global`, o gancho do `keyboard_timeline`): espaço no `EvalCtx` · push da chave espaçada · `Fingerprint.space` · `publica` no
  `publish_all` · Pontos · `pose_em` · zoom · `pose_de_mundo` · `ressuscita` (timeline) · `take_top_if` ·
  `ressuscita` (Motion) · `Graveyard` all→any · `entry_clip_of` · `arranged_owners` · `clip_local_time` ·
  pedido · futuro · `UnknownClip` · despacho v19 · «Duplicate não rouba» · `toca_os_objectos` · `push_intent`
  do Autoplay · interruptor morto.
- Agentes: itens 7/8 (paint.rs, model.rs, field_gizmo*.rs, motion_demo_legend_dono.rs), itens 2/4/5 (7), Play Clip (4).
- Itens 9/10: testes nasceram RED-first contra o código antigo; mutações **candidatas, NÃO corridas**:
  `dropdown_popover.rs:53-55/60`, `panel_ops.rs:305-306`, hierarchy `paint.rs:100`, `reveal.rs:121`,
  `outline.rs:54`, `paint_mode.rs:98`.

## 7. Testes novos (principais)

`ph2d-timeline` tests/it/`clip_entry.rs` (6) e `clip_names_per_owner.rs` (3) · `ph2d-ecs` `graveyard_tests` (5) ·
`ph2d-app-motion` `motion_objeto_entradas_tests` (7, incl. o censo da placa), `field_gizmo_down_tests` (3),
`motion_demo_legend_dono_tests` (3) · `ph2d-editor-core` `gizmo/model_tests` (3) · `ph2d-panel-timeline`
`autoplay_seam`/`add_clip_full_seam`/`arrange_sources_seam` · `ph2d-panel-hierarchy` `a_long_filter_list_scrolls` (4) ·
`ph2d-app-components` `signal_actions_play_clip_tests` (3) · shell `fase_sequences_tests`, `undo_tests`.

## 8. Premissas que caíram

1. «O Play toca a animação do objecto» — só tocava o Arrange da Cena; seguiu-se o molde das cutscenes (relógio por objecto).
2. «Converter as entradas num ponto por dono» — o cozido é global e os externos uma tabela; a porta única é a leitura do nó.
3. «A placa precisa da conversão própria» — nenhum kernel lê externos (censo).
4. «O `$at:` é a pose de mundo» — publicava o Transform LOCAL.
5. «O pedido ao Inspector pode levar `StableId`» — o Inspector só tem os bits da entidade.
6. «A recusa do Paint deixa a camada pendente» — a recusa não chega a escrever; os casos rançosos eram `leave` e outra imagem.
7. «Um undo composto entre filas» — recusado; o cemitério dispara o 2.º Ctrl+Z de sempre só quando é seguro.
8. «Ctrl+Z global e depois o da timeline» (o caminho documentado de recuperação) — com a timeline ABERTA o
   1.º Ctrl+Z nunca chegava ao global: a fila da timeline consumia-o sempre que tinha passo (laço com a purga).

## 9. Smokes (binário `smoke` compilado na worktree; fotografados com `fotografa_cena.sh` em 09/10)

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-ObjectsAndComponents
PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop       # item 1: Autoplay
PH2D_MOTION_OBJETO_SMOKE=1 ./target/smoke/ph2d-host-desktop  # itens 6/8: Pilha + Enxame, legendas
PH2D_MOTION_OBJETO_SMOKE=3 ./target/smoke/ph2d-host-desktop  # item 6: o íman segue o rato
PH2D_VIDA_SMOKE=1 ./target/smoke/ph2d-host-desktop           # item 9: ▾ «All» ao lado da busca
```

- `PH2D_ANIMACAO_SMOKE=1`: interruptor «Autoplay» ligado no «andar» do Herói; botão «Play All» ao lado do
  transporte. Com **Play All** (em qualquer aba) o Herói anda sozinho enquanto o Inimigo salta pela Animação da
  Cena. Errado: o Herói parado em Play All, ou a mexer-se num Arrange vazio com o Play normal, ou o interruptor
  clicável no clipe que já é a entrada.
- `=1` (Motion): Pilha + Enxame; legendas sob as taças (item 8). `=3`: sem rato na tela virtual o cursor fica
  na origem do MUNDO e o enxame correu para lá — não para a origem do objecto em (400,150), como antes desta onda.
- `PH2D_VIDA_SMOKE=1`: ▾ «All» ao lado da busca.
- O resto (3, 4, 5, 7, 10) prova-se por testes de costura: a tela virtual não aceita eventos sintéticos.

## 10. O que fica ABERTO

- (a) O gizmo de SPRITE (`snapshots_gizmo.rs`) ainda usa `model` identidade — filho de pai com escala não
  uniforme mostra caixa sem cisalhamento (o mesmo `model` cura; fora da lista).
- (b) Cinco painéis com cópia própria da lista com rolagem (painter-layers, authored, flip, vector,
  skeleton) — migrar para `paint_dropdown_popover_scrolled_with_hits`.
- (c) O argumento do Play Clip é texto livre (como o `TimerName`): o Inspector não tem o documento da timeline
  para oferecer um ▾ de clipes; o nome errado é DITO em toast no Play.
- (d) A guarda do reset da timeline (`nomeados` em `timeline-persist`) procura nomes de ossos inteligentes no
  documento inteiro — só mais conservadora.
- (e) ~~O autoplay corre só na aba Arrange~~ — substituído pelo **Play All** (`acd61634c`, pedido do dono).
- (f) `ph2d-panel-registry-init` com `-p` simples mostra 76 falhas por features que só o workspace liga (com
  `--features panel-flip,panel-flip-frames,panel-painter-layers,panel-wet-tuning` 124/124).
- (g) Flakes de carga vistos e confirmados sozinhos: `ph2d-tool-painter::the_mask_stroke_cost_does_not_follow_the_canvas`
  (load ~14; **já** na tabela de `FLAKES_DE_CARGA.md`) e `nesting_clock::the_cost_of_depth_is_linear_not_explosive`
  (load ~8–10, 3/3 sozinho; **já** na tabela, `ph2d-timeline`). Nada a promover.
- **Próxima onda: decisão do DONO**; o integrador não a abre.

## 11. Fecho

- `rm -rf target/*/incremental` feito; `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 1.ª `31.22s`, 2.ª corrida:

```text
    Finished `smoke` profile [optimized] target(s) in 0.23s
```

- `bash scripts/agent-loop-profile.sh`:

```text
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                248   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                562 : 278   alvo: <= 1,0  razao 2.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  35%   alvo: >= 80%  (1083 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         416 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

- Máquina limpa do lado desta linha (o único `ph2d-host-desktop` vivo no fecho era o da `line-PainterWatercolor`, no ecrã real — não tocado).
- Integrar não é aprovar: smoke é do Enio; integração e ship só por ordem dele.
