# Continuação — `line/TimelinePrincipal`, onda 0 (estudo) → onda 1 (a barra)

- Worktree: `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · base `f254e42bf1`.
- Plano e decisões do dono: [`15_plano_redesenho_molde_rive.md`](../15_plano_redesenho_molde_rive.md) (versão **B**;
  «Keys | Curves» FORA; sem função nova).
- Foto de antes (`PH2D_ANIMACAO_SMOKE=1`, 1930×1040): [`fotos/15_antes_animacao_smoke_1.png`](../fotos/15_antes_animacao_smoke_1.png).
  Mostra: barra em 2 fileiras; trilhas «Herói · Translat…» ×2; régua 10–17 s com clipe de 2 s (o relógio da cena segue a
  andar, T=10.35); nenhuma chave visível.
- Maquete B (alvo): `~/Documentos/GameEngineSamples/critica_ph2d.html#timeline`, secção 6 «B · O molde do Rive».
  Para fotografar a maquete: extrair a `<section id="timeline">` com o `<head>` e o `<svg>` de símbolos e correr
  `google-chrome-stable --headless=new --screenshot` (feito na onda 0).

## Mapa do código (medido na onda 0)

- **A barra viva é a dos cartões**: `crates/ph2d-panel-timeline/src/transport_cartoes.rs` (`paint_bar` :336, `caixas` :36,
  `botoes_do_clip` :136, `fileira_do_transporte` :160, `pinta` :250). `paint.rs:119-141` escolhe; `cards_on()` em
  `paint.rs:182`. A clássica (`transport.rs`, `ITEMS` :94-117, 21 itens) só com `PH2D_CARDS=0` — NÃO mexer.
- Registo: `populate.rs` (botões :8-14, números :37-39, toggles :58-72, onion settings :76, clip :80-91).
  Despacho: `event.rs` (`is_button` :39, `is_chip` :56, `is_toggle` :63, `apply_event` :75; abas ~150; Speed :204 é
  estado de VISTA `speed_view`; Onion ~222). O shell traduz: `shells/desktop/src/render_loop/timeline_bridge.rs:266`
  (`intent_for_transport`).
- Opções de clipe por aba: `transport_clips::source_options(snap, tab)` / `selected_source`. Dono: `transport_animation::chip_text`.
  Autoplay: `transport_entry::ligado(snap)`, id `TIMELINE_CLIP_AUTOPLAY`.
- Nome da trilha: `tracks_label.rs:57` (`track_label`), `:88` (`compoe` = «{n} · {rotulo}»); grupos em `linhas.rs:215`;
  modelo `ph2d-timeline/src/channel_group.rs`; i18n `ph2d-i18n/src/timeline.rs` (`prop.translate_x` = «Translate X»).
- Losango `tracks.rs:494`; chaves `tracks.rs:266`; NÃO há barra entre chaves por interpolação. `Interp` em
  `ph2d-anim/src/curve.rs:20`. Menu de segmento (troca a interpolação, clique direito) vive na hero chrome
  `ph2d-editor-core/src/screens/hero/chrome/timeline_segment.rs`; menus em `ph2d-editor-core/src/ids/menus_timeline.rs`.
- Enquadrar: `F` → `request_fit()` (`state_requests.rs:35`), consumido em `paint.rs:227`, `view.rs:68` (`apply_fit`).
- Tokens da fita: `ph2d-tokens/src/color.rs:437-459` (`Timeline*`). Sem token de gravação nem por interpolação.
- Testes que tocam a barra: `tests/it/os_cartoes.rs` (:132 folha cabe, :375 janela estreita), `containers_tab_seam.rs`,
  `seam.rs`, `view_tabs_seam.rs`, `transport_*_seam.rs`, `a_coluna_do_toggle_mede_a_lista.rs`; i18n gate
  `every_word_this_panel_shows_comes_from_the_string_table.rs`.
- Smokes: `PH2D_ANIMACAO_SMOKE=1..6` (Herói, «andar», «pular»), `PH2D_NEST_SMOKE=1..3` (containers).

## Próximo passo (onda 1)

Reescrever `transport_cartoes::paint_bar` em três fileiras (título · clipes · tocar), o menu do clipe e o popover
«Timeline options», com os MESMOS ids e eventos; testes de costura por controlo que mudou de sítio; foto lado a lado
com a maquete B1 antes do smoke do dono.
