# Handoff de integração — `line/anim`: SINAIS DA TIMELINE (ADR-0143) + `Interp::Nearest`

**Data:** 2026-07-25 · **Linha:** `line/anim` · **Estado:** FECHADA, pendente de smoke + ordem de
integração. **NÃO integrei nem pushei** (CLAUDE §0.7).

> A linha entrega **três** coisas: (1) os SINAIS da timeline (ADR-0143), (2) a autoria do marker pelo
> **menu do botão direito** (W3), (3) o **`Interp::Nearest`** (o *Interpolate Mode: Nearest* do Godot).
> As três são aditivas e não tocam contrato congelado; nenhum `PROJECT_SCHEMA`. Só o `DOC_VERSION` da
> timeline bumpou (12→13, pelos sinais).

## O que é

O **GAP 1** da pesquisa de padrão-ouro (Enio: *"o essencial para o estado da arte"*) — **eventos de
timeline**, a única feature que Unity/Unreal/Godot/AE convergem em ter. Um **marker pode carregar um
sinal nomeado** que **dispara ao ser cruzado no play**, como **evento desacoplado** (ADR-0075), nunca
uma chamada — pela **lei do CAMINHO** (`(prev, now]` + wrap de loop), não igualdade de frame (o bug do
Godot), **play-only**.

3 waves + smoke, todas gated e mutação-provadas:

- **W0** (`ph2d-timeline`, headless): `Marker.signal: Option<String>` + `signal::signals_crossed` (a lei).
- **W1** (`ph2d-core` + shell): `Playhead::is_advancing_forward()` + `SignalEmitter` no `timeline_bridge`
  (enche o outbox no branch Arrange, play-only; scrub/reverse/salto re-baselizam).
- **W2** (`ph2d-timeline` + `ph2d-panel-timeline` + shell): autoria + **glifo** (furo no galhardete)
  + **consumidor visível** (toast por sinal cruzado).
- **W3 — a autoria virou o MENU do botão direito** (Enio, 2026-07-25: *"todas as opções de marker
  no menu do botão direito"*): o Shift+duplo-clique SAIU; o pennant ganhou um menu de contexto
  **Rename Marker / Set Signal / Delete Marker** (`ContextMenuKind::TimelineMarker`, tabela
  `TIMELINE_MARKER_MENU`, roteado por `marker_menu::route`). Clique = busca · arrasto = move
  (manipulação direta, fora do menu); o marker saiu de `wants_double_click`.

### `Interp::Nearest` — o *Interpolate Mode: Nearest* do Godot (2026-07-25)

Enio pediu paridade com o **Update Mode: Discrete** e o **Interpolate Mode: Nearest** do Godot. A
avaliação (reportada antes de implementar) concluiu: **Discrete JÁ existe** — é o `Interp::Hold`, no
menu de presets (step no key FAR, `u=1`); **Nearest** era o único buraco, e cabe no sistema de easing
como um variant irmão do Hold — um STEP no **MEIO** do segmento (`u≥0.5`), a lei do "fica com o key
mais próximo". Ours é **por-key** (Godot é por-track) ⇒ mais expressivo (mistura com Hold/Bezier).

- **`ph2d-anim`** (`curve.rs`): `Interp::Nearest`, **apendado por ÚLTIMO** (índices postcard estáveis,
  precedente do `BezierW`) ⇒ **SEM bump de `DOC_VERSION`** (saves antigos seguem legíveis). `remap`,
  `slope`=0, `reversed`=self, `handles`=None; `value`/`value_slope`/`interpolate` caem nas vias
  genéricas. curve.rs ficou **no cap de 700 LOC** ⇒ o gate foi pro sibling `curve_slope_tests.rs`.
- **Menu + shell:** `CTX_MENU_TL_NEAREST` em `TIMELINE_SEGMENT_MENU` (7→**8**, abaixo de "Hold") ·
  `Preset::Nearest` + `preset_for`/`absolute` em `timeline_presets.rs` · `path_convert` conta Nearest
  como step (não ease) junto de Hold/Linear.
- **Gates mutação-provados:** `nearest_steps_at_the_midpoint_not_the_far_key` (mut `remap`→Hold: RED) ·
  o par `(NEAREST, Interp::Nearest)` no gate de pick do shell (mut `absolute`→Hold: RED) · o
  anti-item-morto `every_published_menu_row_resolves_to_a_preset` cobre a row nova de graça.
- ⚠️ **BUG DE COSTURA FECHADO PÓS-SMOKE (`c6a5a4003`):** o clique real de "Nearest" no menu **não
  fazia nada** (Enio: *"não muda o gráfico da animação em nada"*). Os gates acima testavam
  `intents_for_pick` **direto**, pulando o passo `timeline_segment::apply` que transforma o clique da
  linha num `TimelineInterpPick` — e esse passo **enumerava os ids "folha" à mão**
  (`Hold|Linear|Custom|Rove`), sem Nearest ⇒ o clique caía no `return false`, nenhum pick era
  estacionado, nenhum `SetInterp` saía. O gate de seam (`timeline_segment_menu_parks_each_leaf_pick_for_the_shell`)
  enumerava a MESMA lista incompleta (nem cobria Rove) ⇒ **verde sobre produto quebrado**.
  [[feedback_a_condition_that_enumerates_its_readers_rots]]. **Fix rot-proof:** `is_top_level_leaf`
  agora DERIVA da tabela `TIMELINE_SEGMENT_MENU` (menos as 3 cascatas), e o gate caminha a mesma tabela
  (5 folhas: Hold/Nearest/Linear/Custom/Rove, guard de contagem) — mutação (lista hardcoded sem
  Nearest) → RED em "pick parked". Lição: **um gate que testa a função por baixo da UI não prova a
  UI**; o seam do clique real era o que faltava.
- **Sem smoke env-gated** (é um preset de menu): a checagem manual é botão-direito num key →
  **Nearest** → o valor salta no meio do segmento (Hold salta no fim).

## Commits (tip = `c6a5a4003`)

```
c6a5a4003 fix(anim): liga a UI do Nearest -- o clique do menu estava caindo no vazio
58dc14eb2 docs(handoff): registra o Interp::Nearest na linha line/anim
af65ba67e feat(anim): Interp::Nearest -- o Interpolate Mode: Nearest do Godot
1b4f6af2c feat(timeline): autoria de sinal do marker vira o menu do botao direito (W3)
8d4169783 chore(timeline): fechamento -- fmt, elisao, split doc.rs, LITERAL-PX-OK
edfb135aa feat(timeline): smoke dos sinais -- PH2D_SIGNAL_SMOKE=1
26ef3ea34 feat(timeline): W2 -- autoria + glifo + consumidor visivel
ebcbc97cf feat(timeline): W1 -- o canal desacoplado no bridge
f1006949a feat(timeline): W0 -- Marker.signal + a lei de cruzamento
e06d22387 docs(adr): 0143 -- sinais da timeline
```

## ⚠️ Números PROVISÓRIOS que o integrador RECONCILIA (o número se CONTA, não se escolhe)

- **ADR-0143** — o maior no main hoje é **0142**. Se outra linha reivindicou 0143 nesta janela,
  **renumere** o arquivo **e todos os `[ADR-0143]`/`ADR-0143` nos doc-comments** — enumerar os arquivos
  aqui apodrece (a W3 acrescentou mais: `menus_timeline.rs`, `marker_menu.rs`, `types.rs`), então use
  `git grep -l 'ADR-0143'` como fonte. 4ª colisão potencial no repo.
- **`DOC_VERSION` 12 → 13** (`crates/ph2d-timeline/src/doc.rs`). Se **physics/vector/qualquer linha**
  também bumpou o `DOC_VERSION` da timeline nesta janela, o valor final **se conta** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
  Pinado em 3 gates (`doc_roundtrip.rs`, `nesting_data.rs::the_schema_is_thirteen...`).
- **`PROJECT_SCHEMA` NÃO mudou** (fica 29): o `TimelineDoc` viaja como blob versionado dentro do
  `ProjectFile`. Foi isso que manteve esta linha FORA da disputa de número de `PROJECT_SCHEMA`.
- **Contrato congelado (§6): INTACTO** — `NodeOp`/`OpResolver`/`NodeManifest` não tocados; nenhum
  gate de contrato afetado. Sinal é dado de documento + um canal de shell.

## Superfície nova (toda ADITIVA)

- **Foundational** `ph2d-core`: `Playhead::is_advancing_forward()` — método `pub` append-only, isolado
  (nenhum campo novo). Projetado para isolamento.
- `ph2d-timeline`: `Marker.signal` (campo apendado) · `signal.rs` (`TimelineSignal`, `signals_crossed`,
  re-exportados) · `TimelineIntent::SetMarkerSignal` (enum **não serializado** ⇒ append livre) ·
  `TimelineViewSnapshot.markers` virou `(f64, String, Option<String>)` (snapshot runtime, não salvo) ·
  `doc_markers.rs` (módulo-filho novo, só code-motion dos 6 métodos de marker).
- `ph2d-panel-timeline`: `MarkerRename.editing_signal` · glifo no `ruler.rs` · editor de sinal no
  `marker_rename.rs` · **`marker_menu.rs`** (router novo do menu, `route(state, host, id)`). O editor
  inline **reusa** `TIMELINE_MARKER_RENAME_INPUT` (nenhum IconId novo); os ids das linhas do menu vêm do
  editor-core.
- **W3 (editor-core, foundational + append-only):** `ContextMenuKind::TimelineMarker { index }` (variante
  apendada) · `TIMELINE_MARKER_MENU` + `CTX_MENU_TL_{RENAME_MARKER,SET_SIGNAL,DELETE_MARKER}` em
  `ids/menus_timeline.rs` · o arm do resolver em `pointer_down_menus.rs` · o arm do overlay em
  `context_menu_overlay.rs` · o registro em `pre_populate.rs` · `wants_double_click` perdeu o `Marker`.
  O `timeline_hit_kind_tests` saiu de `types.rs` (716>700) para `types_tests.rs` (`#[path]`, code-motion).
- shell: `SignalEmitter` (`App.timeline_signals`) + drain→toast no `render_loop/mod.rs` + `signal_smoke.rs`.
  **W3 não tocou o shell** (o menu é editor-core + painel).

## Gates (todos verdes; mutação-provados)

- **W0** `ph2d-timeline/tests/signals_crossed.rs` (7): mut **igualdade-de-frame → RED**, mut **drop-wrap → RED**.
- **W1** `shells/desktop/src/render_loop/timeline_bridge_signal_tests.rs` (6): mut **drop-forward-guard →
  paused+reverse RED** · `shells/desktop/tests/timeline_signal_emits_in_arrange_only.rs` (arch-gate).
- **W2** `marker_rename_tests.rs` (+2 signal) · `snapshot`/`ruler` compilados.
- **W3 (o menu, mutação-provado):** `ph2d-panel-timeline/tests/marker_menu_seam.rs` (4 — cada linha CLICA
  o seam real: Rename arma label-mode, Set Signal arma signal-mode, Delete levanta `RemoveMarker`, e o
  anti-item-morto varre `TIMELINE_MARKER_MENU`; **mut** neutralizar o arm do `event.rs` → 4 RED · **mut**
  Rename em signal-mode → label RED) · `dispatch/tests/timeline.rs`: **`right_clicking_a_marker_opens_the_marker_menu`**
  (o arm do resolver — **mut** removê-lo → RED) + `double_clicking_a_marker_is_now_a_plain_click` +
  `double_clicking_a_key_stays_a_click` · `marker_drag_tests.rs`: `a_double_click_no_longer_arms_a_rename`
  · `types_tests.rs`: `exactly_the_container_bar_wants_a_double_click` (pin do double-click list).
- **smoke** `signal_smoke_tests.rs` (2): a cena dispara exatamente os 2 sinais numa volta.
- Fechamento: `cargo fmt` · `clippy --all-targets` limpo (core/timeline/panel/shell) ·
  `architecture_workspace_file_loc_cap` + `no_magic_numeric` + `file_loc_caps` + `architecture_panel_loc_cap`
  verdes · **`nextest-impacted` verde** (após os 2 fixes de fechamento).

## Smoke (pendente de aprovação do Enio)

```
env PH2D_SIGNAL_SMOKE=1 cargo run -p ph2d-host-desktop
```
- **Toast a cada volta do loop** para `footstep` (@1,0 s) e `beat` (@2,5 s); a anotação `chapter`
  (@3,5 s, sem sinal) **não** dispara. ⚠️ Se a linha `[signal-smoke]` não aparecer, PARE.
- **Pause (Space):** nada dispara. **Scrub** na régua: nada dispara.
- **Abra a timeline (`L`):** os markers com sinal têm um **FURO** no galhardete; a anotação não.
- **Autoria (o MENU):** **botão direito num marker** → **Rename Marker** (edita o label) · **Set Signal**
  (edita o sinal; borda na cor do marker, nome em branco LIMPA) · **Delete Marker**. Confira que o menu
  abre no pennant, que cada linha faz o que promete, e que **clique simples ainda busca** e **arrasto
  ainda move** (não estão no menu, de propósito). O Shift+duplo-clique **não existe mais**.

## Aberto / nomeado (não é bug — decisão)

- **O consumidor REAL é cross-line, decisão sua** (a mesma cerca da física W-ContactEvents): o **cue de
  áudio** (footstep → `ph2d-audio` one-shot), **gameplay**, **Luau/MCP** são consumidores do canal
  `App.timeline_signals.out` / `ph2d_timeline::TimelineSignal`. O v1 entrega o canal + o toast (prova
  visível). A timeline **nunca** chama nenhum deles (ADR-0075). Próximo consumidor barato e in-domain:
  um sinal `stop` que pausa o transporte (o *stop marker* de editor de vídeo).
- ~~**Descobribilidade da autoria:** Shift+duplo-clique é v1; um item de menu de contexto no marker
  é o v1.1 natural.~~ **FEITO na W3 (2026-07-25):** o menu do botão direito (Rename / Set Signal /
  Delete) substituiu o Shift+duplo-clique.
- **Range/Repeater** (o marker vira faixa — o Trigger×Repeater do Unreal) · **fire-on-scrub toggle** ·
  **multi-lap num único dispatch** subdispara o meio (limite documentado no `signal.rs`).

## Para quem integra

- Os gates de `shells/desktop/tests/` **só correm na varredura impactada** — o ship do integrador os
  pega; o `cargo test -p` por-crate não. Rodei `file_loc_caps` + o arch-gate + os dois de editor-core
  à mão, verdes.
- `git rebase main` antes; reconcilie **ADR-0143** e **`DOC_VERSION` 13** se colidiram.
