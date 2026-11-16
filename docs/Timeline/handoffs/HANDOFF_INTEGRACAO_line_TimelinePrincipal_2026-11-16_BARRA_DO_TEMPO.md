# Integração — `line/TimelinePrincipal`: barra do tempo com alças de zoom + acabamento das curvas prontas (dono 16/11)

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · merge-base = `main` = `dc3d4f9fb8` (o rebase foi
  no-op: o `main` não andou). Sem push, nada integrado por esta linha.
- **10 commits** `38e92274e8..c24de103fb` (`git log --oneline --reverse dc3d4f9fb8..HEAD`). O [handoff anterior](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-15_RETIME_POR_TRECHO.md)
  (23 commits até `ae78635856`/`87bbadf620`) JÁ está no `main` (cadeia `dc3d4f9fb8`); este cobre só o que veio depois.
- Mecanismo e pesquisa da barra: [`20_barra_do_tempo.md`](../20_barra_do_tempo.md).
- **Smoke do dono 16/11: OK de tudo** — pontos/undo/conversão (até `169fab7570`), barra, pontas escuras.

## O que entra

1. `38e92274e8` **curva pronta → Custom** (`ph2d-timeline/graph_pontos_pronta.rs`): cantos primeiro (Bounce In-out exato), alças
   afinadas por trecho, nós ≥ `ESPACO_MIN` 5 %. Medido: Bounce/Quad/Cubic/Back exatos, os outros ≤ 0,8 %, Elastic In-out 2,8 % (era 4,1).
   `SPLINE_MAX_POINTS` não subiu.
2. `236bb7f6f0` **undo/redo mantém a seleção da chave** (`keep_living_selection`, antes `clear()`, `ph2d-timeline/state.rs`);
   **alças só do ponto escolhido** no gráfico e no cartão (`mostra_alcas`; regra Illustrator/Inkscape/AE), clique partido na mediana;
   braços de velocidade dos pontos movidos para `graph_pontos_velocidade.rs` (tecto de LOC). Gates `a_curva_pronta_convertida_…`,
   `undo_and_redo_keep_…`, `so_o_ponto_escolhido_…`; 8 mutações vermelhas.
3. `169fab7570` o 1.º toque numa curva pronta só a converte; o 2.º cria o ponto (`o_primeiro_toque_numa_curva_pronta_converte_e_o_segundo_cria_o_ponto`).
4. `813feac76c` + `b901b85a76` **barra de rolagem do tempo com alças de zoom** (`ph2d-panel-timeline/src/barra_tempo.rs` + `barra_tempo_tests.rs`):
   pontas = zoom (a outra fica), meio rola, calha salta e continua a rolar, duplo clique = caber o clipe (`apply_fit_clip`) e outro
   volta, roda sobre a barra rola o tempo (Ctrl = zoom pelo meio), marcas das chaves + playhead na calha, mapa congelado no arrasto;
   pontas inteiras mais escuras. **Uma vista só:** escreve `view_start_s`/`px_per_s` do `TimelinePanelState`; chaves e gráfico lêem o
   mesmo `graph::TimeView`. Estado `TimelinePanelState.barra: BarraEstado` (nunca gravado).
5. `22c21dffb1`, `58f779df2b` docs: smoke do dono; os três abertos de teclado conferidos no app com
   [`fotos_teclado.sh`](../ferramentas/fotos_teclado.sh) (Ctrl+Z tira o ponto / Ctrl+Shift+Z repõe; valor digitado no Inspector
   regrava a chave, −300 → −150, fica após ±1 quadro; Auto Key na sessão: arrastar regrava, «+» grava no Clip 2 sem Done).
   Nota do roteiro: xdotool `--window` é sintético e o app ignora; XTEST + `windowfocus` chega; `type` não entra no campo, `key` sim.
   Fotos `../fotos/teclado_{1,2,3}_*.png`, `barra_sequencia.png` (+ b0..b5), roteiro `fotos_barra_do_tempo.sh`.
6. `c24de103fb` fecho: clippy assign-op no seam, tecto 600 do `stack_lane_paint_tests.rs` (cortado o doc), `LITERAL-PX-OK` no terço da barra.

## Formato, contratos, superfície de colisão

- Nenhum formato gravado muda; **`PROJECT_SCHEMA` intocado** (nada a recontar). Nenhum contrato §6 tocado.
- Foundational aditivo (Modo L): `ph2d-editor-core` `TimelineHitKind::TimeBar { part: u8 }` (variante nova ⇒ `match` exaustivo
  fora desta árvore); `wants_double_click` passa a incluir `TimeBar` (gate renomeado `exactly_the_container_bar_wants_a_double_click`
  → `exactly_the_container_bar_and_the_time_bar_want_a_double_click`); `pointer_look` (pontas ResizeEw, meio mão, calha seta).
  `ph2d-tokens::chrome`: 5 consts `TIMELINE_BARRA_TEMPO_{H_PX=14, RECUO_PX=2, ALCA_PX=10, POLEGAR_MIN_PX=36, PONTA_ESCURA=0.4}`.
  Ids novos `ids/timeline_rolagem.rs` `TIMELINE_TIME_BAR[4]`.
- ⚠️ **O que um merge pode partir:** `geom::resolve` tira 14 px do pé (`Geom.barra_tempo`): régua, playhead e linhas param por cima;
  quem mede `rows.h` vê 14 px a menos. O seam `no_lugar_lanes_seam::o_renome_da_lane_tem_a_altura_da_linha_pintada` passou a rolar
  até à lane B (`scroll_y = f32::MAX`) — teste de outra linha que dependa da altura da área das lanes reconfere-se.
  Visibilidade: `view::content_extent` e `alca_estado::mistura` viraram `pub(crate)`. `shells/` intocado.
- Superfície de colisão: sem ✗; `Cargo.lock` sem pacote novo.
- Ficheiros partilhados: `crates/ph2d-panel-timeline/src/{geom,event,populate,state,lib,ids,view,alca_estado,graph*}.rs` ·
  `crates/ph2d-editor-core/src/…` (hit kind, `wants_double_click`, `pointer_look`, `ids/`) · `crates/ph2d-tokens/src/chrome*` ·
  `crates/ph2d-timeline/src/{state,graph_pontos*}.rs`.

## Provas

- `barra_tempo_seam` 6 (cartões, despacho real: lugar e alvos, pontas, meio, calha, duplo clique e volta, roda) + `barra_tempo::tests` 5.
- Mutação 10/10 vermelhas: troca das pontas · sem mínimo do polegar · sem salto da calha · duplo clique sem volta · roda inerte ·
  meio com zoom · geom sem a faixa · `TimeBar` fora de `wants_double_click` · calha registada por cima · ponta direita no alvo esquerdo.
- Fecho 16/11 (`PH2D_SUITE=fecho`): fmt ✓ nas 4 crates (panel-timeline, editor-core, tokens, timeline); clippy `-D warnings
  --all-targets` ✓ nas 4; testes por nome ✓ (panel-timeline 298 + barra 11 + 2; editor-core 2 + `no_magic_numeric` 3 + LOC/literal 15;
  timeline 2); `panel_files_under_loc_cap`, `panel_functions_under_loc_cap`, `workspace_src_files_under_loc_cap`,
  `widget_primitives_under_loc_cap` ✓.
- **NÃO corridos:** a suíte completa e o `nextest-impacted` (regra do dono: só na integração).

## Smoke (o dono já fez; para reconferir)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal
PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop
```
Barra no pé da Timeline: arrastar uma ponta muda o zoom (a outra fica); o meio rola; clique na calha salta; duplo clique cabe o clipe
e outro volta; roda sobre a barra rola (Ctrl = zoom). Errado: a barra rola mas as chaves/gráfico ficam noutra vista.
Fotos: [`barra_sequencia.png`](../fotos/barra_sequencia.png). Binário compilado no HEAD `c24de103fb`, 2.ª corrida 0 linhas `Compiling`.

## Aberto

- Nada do dono. Para o integrador: a suíte inteira; o §Aberto do handoff RETIME_POR_TRECHO tem os ✅ de 16/11.
- `CLAUDE.md` §5.1 Timeline: esta árvore já aponta para este handoff («Barra do tempo com alças de zoom, 16/11»).
