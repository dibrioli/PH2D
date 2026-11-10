# Continuação — `line/TimelinePrincipal`, onda 3 (Containers e Arrange) — fecha o plano

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · commit da onda `4060362f15` sobre `bc4a6ce930` (= main 10/11).
- Plano: [`15_plano_redesenho_molde_rive.md`](../15_plano_redesenho_molde_rive.md) §Onda 3 (versão B). Spec medida: [`16_onda3_containers_arrange.md`](../16_onda3_containers_arrange.md).
  Onda anterior: [`..._ONDA2.md`](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-09_ONDA2.md). Cena: [`smoke_nest.md`](../smoke_nest.md).
- Fotos: [`15_onda3_cena4.png`](../fotos/15_onda3_cena4.png) (Arrange) · [`15_onda3_cena5.png`](../fotos/15_onda3_cena5.png) (Containers) ·
  [`15_onda3_lado_a_lado_arrange.png`](../fotos/15_onda3_lado_a_lado_arrange.png) (maquete B4 sobre o app) ·
  [`15_onda3_lado_a_lado_containers.png`](../fotos/15_onda3_lado_a_lado_containers.png) (A3 sobre o app).

## O que a onda fez (só cartões, mesmos ids e eventos; `PH2D_CARDS=0` intacto)

Maquete B3/B4 + a lista de A3, capturada em chrome headless a 1300 px, escala 1, `.ph` isolado por script; rects por `getBoundingClientRect`
+ estilos computados + amostra de pixel; variáveis `--c13-*` exatas (tabela completa no doc 16).

1. **Dado novo, só leitura:** `TimelineDoc::container_uses(i)` (`ph2d-timeline/src/doc_extent.rs`) → `ContainerView::uses` (`snapshot.rs`):
   colocações diretas na pilha da cena + em cada pilha de container; faixa muda conta; `0` = «not placed». Derivado no `rebuild`, não gravado
   (`DOC_VERSION` e `PROJECT_SCHEMA` iguais). Teste `o_container_conta_onde_esta_colocado`.
2. **Tokens** (`ph2d-tokens`): `TimelineTinta::{PecaA..PecaE, PecaContainer, PesoPilula, OlhoMudo}` + `PECAS` (reexportado);
   `chrome.rs`: `TIMELINE_FAIXA_H_PX` 30, `_PECA_MARGEM_PX` 4, `_PECA_{TOPO 0.58, PE 0.34, BORDA 0.8, MUDA 0.4}_ALPHA`, `_MISTURA_ALPHA` 0.34,
   `_MISTURA_BORDA_ALPHA` 0.55, `_PECA_RAIO_PX` 5, `_PESO_RAIO_PX` 4, `_LISTA_BARRA_PX` 3. `color.rs` intocado (700/700).
   Gate `as_cores_das_pecas_leem_se_diferentes_em_todo_tema` (≥ 0,10 OKLab, `Theme::ALL`).
3. **Faixas (lanes)** — `faixas_coluna.rs` (novo; a coluna clássica foi movida sem mudança): altura 30 nos cartões pela porta única
   `geom::faixa_h()` (clássico `ROW_H_PX`); `geom::peca_margem()`; `geom::min_label_w_medido` = piso para que nomes de faixa e de container
   + «used 2×» nunca elidam (teste `os_nomes_das_faixas_e_da_lista_saem_inteiros_a_1290`). Peso em pílula, olho (mudo = `OlhoMudo`).
4. **Peças** — `pecas_cartoes.rs` (novo, `pinta_peca`): cor do clipe = índice do `clip_name` na lista de clipes do snapshot % 5; container =
   `PecaContainer` (laranja); muda × 0,4; listras na mistura. `stack_lane_paint.rs` 590→372; `strip_paint.rs` 591.
5. **Lista de containers** — `lista_coluna.rs` (novo): linha = altura da faixa, nome + uso à direita antes do lápis e da lixeira, barra do
   tempo pintada como peça; `uso.rs` (`rotulo(uses)`, «used 2×» / «not placed»).
6. **Barra de chips** — `barra_clips.rs` 334→500: dois grupos rotulados na MESMA fileira («Clips» + «+», depois «Containers»), uma regra de
   transbordo «+N more»; o chip de container leva o texto de uso.
7. **Pedido de aba:** `state::request_containers_tab()` (`state_requests.rs`), consumido em `paint.rs` depois do pedido do Arrange
   (`pop_to_depth(0)` + `Tab::Containers`).
8. **editor-core (append-only):** `banda::degrade_vertical_em` (`cards_banda_tons.rs`, degradê vertical recortado ao retângulo arredondado);
   `text_elide::largura_do_titulo`.
9. **i18n:** novas `panel.timeline.card.{used_times,not_placed,containers}`; removida `card.sources` (sem uso).

## Alvos de clique (medidos ANTES, cartões 1930; nenhum encolheu)

Peso 38×16 → 38×22 · mudo/`+` 16×16 → 16×22 · rótulo da faixa 92×22 → 92×30 · lápis/lixeira 16×16 → 16×22 · add-lane/add-container 174×22 (igual) ·
chips: clipe 56/61×28, container 101/85×28, «⋯» 28×28, «+» 28×28, «+N» 89×28 — todos ≥. Gates `os_alvos_das_faixas_nao_encolheram`,
`os_alvos_dos_chips_nao_encolheram`. Nota: a barra do container na lista tem largura = tempo × zoom, por isso o gate só afirma a ALTURA (≥ 16);
a largura caiu 89→85 px na vista de teste sem enquadrar porque a coluna cresceu 4 px para nomes inteiros.

Pisos de cor medidos (12 temas): nome sobre a peça ≥ 0,33 (pior 0,335 Forge `PecaB`) · muda × viva ≥ 0,15 (0,152 Sunstone `PecaC`) · listra de mistura ≥ 0,17 (0,178).

## Superfície de colisão

`ph2d-panel-timeline`: `barra_clips.rs`, `container_list.rs`, `faixas_coluna.rs` (+`_tests`), `geom.rs` (+`_tests`), `lib.rs`, `lista_coluna.rs`,
`paint.rs`, `pecas_cartoes.rs` (+`_tests`), `stack_lane_paint.rs`, `state.rs`, `state_requests.rs`, `strip_drag.rs` (+`_tests`), `strip_paint.rs`,
`transport_clips.rs`, `uso.rs`; testes `tests/it/{as_faixas_nos_cartoes,os_chips_da_barra,main}.rs`. `ph2d-timeline`: `doc_extent.rs`, `snapshot.rs`
(+`_tests`). `ph2d-tokens`: `chrome.rs`, `lib.rs`, `timeline_tinta.rs`. `ph2d-editor-core`: `cards_banda_tons.rs`, `text_elide.rs`. `ph2d-i18n/src/timeline.rs`.
Shell: `shells/desktop/src/nest_smoke.rs` (+`_tests`). Docs: `16_onda3_containers_arrange.md`, `smoke_nest.md`, 4 fotos.
Sem `PROJECT_SCHEMA`, sem contrato §6, sem formato gravado. Contadores (DELTA): shell LOC 189016 → ~189012 (teto 189041; os doc comments das
cenas 2/3 foram movidos verbatim para `smoke_nest.md`); chaves i18n +3 −1; campos de estado: nenhum; `color.rs` 700/700.

## O que um merge pode partir

- `paint.rs`, `state.rs`, `geom.rs`, `strip_paint.rs`, `lib.rs` e `i18n/timeline.rs` são tocados também por outras ondas/linhas da Timeline: conferir o
  `collision-surface.sh` contra o main.
- `stack_lane_paint.rs` perdeu ~220 linhas (movidas para `faixas_coluna.rs`/`pecas_cartoes.rs`): um rebase com edição concorrente ali conflita.
- Gate `a_montagem_poe_cada_container_onde_o_eprintln_diz` e o teto de LOC da shell soma entre linhas.
- Gates de `no_orphan_source_file`/`panel_loc_cap` varrem os 5 ficheiros novos.
- Duas cenas de smoke novas (`NEST_SMOKE=4|5`): se outra linha também numerar cenas do nest, recontar.

## Testes e fecho

`cargo fmt`; clippy `-D warnings` limpo em `ph2d-panel-timeline`, `ph2d-timeline`, `ph2d-tokens`, `ph2d-i18n`, `ph2d-editor-core`, `ph2d-host-desktop`
(dois tipos complexos em `build_montagem` factorados em `Clipe`/`Faixa<'a>`). 44 corridas por nome, todas verdes: `os_chips_da_barra`,
`as_faixas_nos_cartoes`, `faixas_coluna`, `lista_coluna`, `pecas_cartoes`, `os_cartoes`, `containers_tab_seam`, `arrange_sources_seam`,
`nesting_seam`, `strip_ease_grip_seam`, `strip_paint`, `stack_lane_paint`, `strip_drag`, `geom`, `box_select`, `linhas`, `scroll`, `_seam`,
`a_folha_nao_perde_linhas_para_o_cromo` (estrito), `every_key_of_this_crate`, `a_fita_abre`, `duration_chip_gesture`, `one_door`; editor-core:
corner/frame/field theme doors, `no_magic_numeric`, `no_literal_color`, hr15, `panel_loc_cap`, `workspace_file_loc_cap`, `no_orphan_source_file`,
`banda`, `the_shell_only_shrinks`; timeline `o_container_conta` + `snapshot`; tokens `timeline_tinta` + `an_alias`; i18n
`known_keys`/`ordenad`/`a_text_key`/`honesto`; shell `nest_smoke` + `file_loc_caps`. Smoke compilado `--profile smoke`.
**Sem suíte completa e sem `nextest-impacted`** (do integrador). A falha do teste «+N more» foi do ajudante de teste, que repintava a 1930 dentro de
um teste de 1100 px (curado com `textos_em`); a contagem `pintados + n == total` foi restaurada, o código não mudou.

## Fica de fora (função ou dado novo)

- Arrastar o chip para a faixa (gesto da B4): colocar continua pelo `+` da faixa.
- Marcas das voltas de loop dentro da peça: o `StripView` não traz modo de loop nem o tamanho do clipe.
- Anel da peça selecionada: ligado, mas hoje nada seleciona uma strip.
- A 1290 px de largura acoplada o grupo de containers dobra em «+2 more» no Arrange (Idle pack/Step só na lista).
- Região de loop na régua segue a banda de altura inteira existente (maquete: fio teal de 5 px) — igual à aba Keys, não tocada.
- Marcas de canto da strip (verde/vermelho) mantidas.

## Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_NEST_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. A Timeline em baixo abre em «Arrange»: faixas Base 1.00, Upper body 0.60 e FX com o olho fechado; peças coloridas (idle verde, Run & jump laranja,
   swing azul, shake apagada), sobreposições listradas; chips «Clips … Containers  Run & jump used 2×».
3. Clicar no olho de FX: as peças ganham vida. Clicar na aba «Containers»: lista com Run & jump used 2×, Idle pack not placed, Step used 1×;
   duplo clique numa barra laranja entra no container. A cena `PH2D_NEST_SMOKE=5` abre direto na lista.
4. Errado se: um nome aparece cortado com «…», um botão deixou de responder, ou `PH2D_CARDS=0` ficou diferente de antes.

## Próximo

- O plano 15 fecha com esta onda. Falta: smoke do dono (não feito) e a decisão pendente «Translate X» × «Position X» (perguntada de novo no fim da onda 3).
- Depois: `/pd-linha-fechar` (handoff de integração; §5.1 do CLAUDE.md: Timeline passa a `PH2D_NEST_SMOKE=1..5`). Sem push nem integração.

## Rodada 3b (dono 10/11): Timeline flutua + listras por baixo + sombra das alças

- **Flutua** (cartões só): `frame_layout.rs` deixa de reservar a faixa de baixo para a Timeline quando `cromo_em_cartoes()`;
  só ergue a pílula do rodapé (`HeroLayout::lift_hud_over`). O painel publica `set_floating_panel_rect` (`paint.rs`), a declaração
  D1/D24 em runtime. Flip/Pixel timeline e `PH2D_CARDS=0` continuam a reservar a faixa.
- **Causa da peça toda listrada:** a cena 4 não autorava o fim da cena (fim = 4 s) e tudo depois dele lia-se como mistura. A cena
  autora agora 6 s; `a_mistura_de_uma_peca_cobre_so_a_sobreposicao` prende `[4, 4.25)` = 25 px.
- **Ordem de pintura** (cartões): `paint_lane` pinta em duas passadas (`Fase::Base` corpos + misturas de todas as peças, depois
  `Fase::Topo` rótulo → barras de mudança → cantos → alças de fade). O clássico continua `Fase::Tudo`.
- **Sombra das alças** = lei da câmera/contorno: a mesma forma `Thin` mais larga de cada lado, `BgScrim` a
  `ph2d_tokens::OUTLINE_SHADOW_ALPHA` (`paint_flight::sombra_de_alca`). UMA porta na timeline: `strip_paint::pinta_sombra_de_alca`
  (rects + triângulos), usada pelos cantos, alça de fade, chaveta e linha de mover do loop (`ruler.rs`) e a «⟷» da duração
  (`ruler_veil.rs`). Hit rects intactos. Alças das strips foram para `strip_paint_alcas.rs` (tecto 600 LOC; a isenção de raio viaja).
- **Testes novos:** `nos_cartoes_a_timeline_flutua_e_o_desenho_chega_ao_fundo` (registry-init; mutação: sem `&& !flutua` ⇒ vermelho) ·
  `nos_cartoes_a_mistura_sai_antes_do_rotulo_e_das_alcas` (stack_lane_paint_tests; mutação: `Fase::Tudo` ⇒ vermelho). As listras
  saem por `inner_mut().stroke` (não se gravam): o teste delimita-as entre o FUNDO da mistura e o `Pop` do recorte; o texto também
  não se grava, o rótulo conta pelo ícone na cor `cores().texto`.
- Fotos: `docs/Timeline/fotos/16_timeline_flutua.png`, `16_alcas_com_sombra_zoom.png` (2×).
- **Aberto:** as listras além do fim da cena (véu) parecem listras de mistura — o dono pode querer outra textura para «fora da cena».

## Rodada 3c (dono 10/11): estados das alças

- Porta única `crates/ph2d-panel-timeline/src/alca_estado.rs` (`EstadoAlca::{Repouso,Hover,Apertada,Arrasto}`), só nos cartões
  (`PH2D_CARDS=0` byte a byte: o clássico mantém o Accent no hover dos cantos/fade).
- Estado sai do store: `hot_id` (hover só sem nada apertado), `active_id` (aperto) e a folga da captura
  (`WidgetStore::timeline_dragging` → `PressSlop::moved`, novos, só leitura) = arrasto. Sem tween: o `hover_live` só é publicado pelo hero.
- Hover = mistura com `TimelineTinta::Chave` (`TIMELINE_ALCA_HOVER_MIX` 0,5); na chave da folha acende o ANEL.
  Aperto = `ChaveEscolhida` + forma `Thin` maior. Arrasto = + halo de acento (`TIMELINE_ALCA_HALO_ALPHA` 0,35, `Heavy` mais largo) sob a sombra.
- Guia de acento (`TIMELINE_ALCA_GUIA_ALPHA`) na altura da strip + etiqueta de leitura (segundos `{:.2}`, `zero_sem_sinal`):
  cantos = tempo da borda, fade = comprimento do fade, loop = início/fim, linha de mover = início, ⟷ = duração, chave = tempo ao vivo.
- Alças cobertas: cantos e fade das strips, chaves e linha de mover do loop, ⟷ da duração, chaves da folha (não as de grupo).
  ⛔ Não feitos: grip do playhead, bandeiras dos marcadores, cursor do rato (o app não tem API de cursor no painel), «lift» da strip em hover.
- Pisos MEDIDOS em `Theme::ALL` (OKLab): hover ≥ 0,054 (fade/duração, pior tema); acento ≥ 0,017 — num tema o playhead JÁ é
  azul, aí o tamanho e o halo dizem o estado. Portão `os_estados_das_alcas_leem_se_em_todo_tema` (pisos 0,05/0,015).
- Portões novos (`alca_estado_tests.rs`): estados no store; strip REAL (canto 5 e fade-in) hover/aperto/arrasto com os hits
  idênticos ao repouso (seam); loop e chave nos quatro estados; leitura. `paint_diamond` e triângulos passaram à porta que grava
  (`fill_shape`/`stroke_path`, a mesma codificação).
- Isenção de quina do `ruler_veil.rs` viajou para `alca_estado.rs` (o véu já não escreve raio). `stack_lane_paint_tests.rs` estava a 605 LOC no HEAD: comentários encurtados (599).
- Foto offscreen `17_alcas_estados.png`: não feita (o painel não tem rasterizador CPU; só GPU) — falta.

## Rodada 3d (10/11): o resto das alças + o cursor do rato

- Porta `alca_estado` estendida: playhead (`playhead`/`estado_do_playhead` — a régua inteira é a pega; arrasto = slider `TIMELINE_RULER` em `Dragging`),
  bandeiras dos marcadores (`ruler::paint_markers`, leitura à esquerda), losango do Summary (`losango_mestre`) e chaves de grupo (`chave_da_folha`).
  Repouso nos cartões e `PH2D_CARDS=0` pintam o mesmo de antes (a porta só entra fora do repouso).
- Cursor: o mecanismo JÁ existia (`timeline_resize_cursor` na shell, única `set_cursor`, por `TimelineHitKind` do hit). A tabela mudou-se para
  `TimelineHitKind::pointer_look(apertado)` (`crates/ph2d-editor-core/src/interaction/timeline_cursor.rs`): `↔` nas pontas/fades/stretch, chaves do loop,
  ⟷, splitter, escala; mão aberta/fechada no corpo da strip, linha do loop, chaves, marcadores, âncoras; `None` no fundo. Enquanto há aperto vale o alvo
  APERTADO (o `grabbing` segue o arrasto fora do hit). Shell: `icone_de(PointerLook)` partilhado com a aba — shell líquido −5 LOC.
- Portões: `o_playhead_os_marcadores_e_as_chaves_mestras_mudam_de_estado` (3 estados × 4 alças, hits idênticos), `o_cursor_da_faixa_sai_do_alvo_registado`
  (faixa REAL), `as_pontas_esticam_o_corpo_agarra_o_fundo_e_a_seta` (editor-core). Corrigido o `clippy::unnecessary_operation` do `nest_smoke.rs:258`.

## Rodada 3e (10/11): a linha do fim e «além do fim»

- Causa: nos cartões nunca houve linha do fim — o clássico marca o fim pela borda do véu `BgScrim`; a onda 2g trocou o véu por listras `Listra` (escuras, a 135°, o mesmo ângulo da mistura) sobre a fita, e a borda sumiu. Sobre peça, listra escura em laranja ≈ mistura.
- Novo `ruler_fim.rs` (filho de `ruler`): `pinta_alem` = véu `FitaPropriedade`×`TIMELINE_ALEM_VEU_ALPHA` (0,65) + listras claras a 45° (`\`, `TextoCabecalho`×0,42, 2 px, passo 12); `pinta_linha_do_fim` = linha de 2 px na tinta da ⟷ (`TimelinePlayhead`), régua+faixas inteiras, estado e sombra da ⟷ (`alca_estado::pinta`, id `TIMELINE_DUR_HANDLE`), pintada depois das ticks e antes do playhead/chavetas. Clássico intacto (véu `BgScrim`, sem linha).
- Tokens novos em `chrome.rs` (`TIMELINE_ALEM_*`, `TIMELINE_FIM_LINHA_PX`); mortos removidos: `TimelineTinta::Listra`, `banda::listras_em`/`LISTRA_PASSO`.
- Pisos medidos (12 temas × 6 peças + fita): listra × véu 0,262 (Sunstone `PecaB`, piso 0,25 ≥ 0,17 da mistura); peça velada × viva 0,166 (Sunstone `PecaC`, piso 0,16). Portões: `alem_do_fim_le_se_sobre_toda_peca_e_apaga_a_peca`, `nos_cartoes_a_linha_do_fim_sai_depois_das_pecas`.
- Semântica igual (o fim autorado da vista). ⚠️ A cena 4 autora o fim em 6 s e abre enquadrada em ~0–5,7 s: nada passa do fim nela. A foto `18_fim_e_alem.png` foi tirada com um patch LOCAL não comitado (fim 5 s, 150 px/s) para mostrar a peça a cruzar o fim; zoom `18_fim_e_alem_zoom.png`.
- ABERTO: o enquadrar ao abrir o Arrange não mostra o fim inteiro (corta ~5,7 de 6 s) — verificar `apply_fit_clip` com o Arrange.

## Rodada 3f (10/11): laço fino na régua, enquadrar ao abrir, chips

- **Laço na régua** (maquete B `.c13lp`, cartões): `ruler::faixa_do_laco` = faixa de `TIMELINE_LACO_REGUA_PX` (5) no topo, `TimelineTinta::LacoDaRegua` (= `PecaA`, [73,224,174]) × `TIMELINE_LACO_REGUA_ALPHA` (0,7). Nos cartões a faixa É a linha de mover: em repouso não se repinta; hover/aperto/arrasto pintam-na na altura dela por `alca_estado::pinta`. Chavetas, ⟷, hits e sombras intactos; clássico = régua inteira em `TimelineLoopRegion`.
- Pisos medidos (`Theme::ALL`, OKLab): laço × régua pior 0,367 (Sunstone) → piso 0,33; números × régua 0,332 (inalterado, os números ficam abaixo da faixa — conferido na foto). A moldura de 1 px da área cobre a 1.ª linha da faixa.
- **Enquadrar ao abrir**: causa = a área de tempo muda de largura nos primeiros quadros (cartão flutuante a assentar) e o fit do 1.º quadro ficava. `view::enquadra_ao_abrir` reenquadra enquanto a vista é a que o fit deixou (`state.enquadre`). Folga = `TIMELINE_ENQUADRE_MARGEM` (0,05/lado; ancorado em 0 sobra ~9 % à direita). Cena 4 abre com o fim (6 s) e «além do fim» à vista; Keys idem.
- **Chips a 1290 px**: fileira inteira precisa 897 px, há 857 (−40). Não há regra de compactar «used 2×»; o único ganho seria apertar os vãos de 6 px dos chips para ~2 px (desenho pior) — não feito; a dobra única «+2 more» fica.
- Testes: `nos_cartoes_o_laco_e_uma_faixa_fina_no_topo_da_regua`, `a_faixa_do_laco_le_se_sobre_a_regua_em_todo_tema`, `a_fita_abre_com_o_fim_e_a_folga_a_vista_mesmo_se_a_area_muda` (mutações: altura `RULER_H`/sem o reenquadrar ⇒ vermelho).
- §5 do CLAUDE.md: `PH2D_NEST_SMOKE=1..5`. Fotos `docs/Timeline/fotos/19_regua_e_chips.png`, `19_keys.png`.
