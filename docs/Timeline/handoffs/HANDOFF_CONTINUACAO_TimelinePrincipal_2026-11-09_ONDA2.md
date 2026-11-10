# Continuação — `line/TimelinePrincipal`, onda 2 (a fita) → onda 3 (Containers e Arrange)

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · commit da onda `8694c376c4` sobre `a5d54be996`.
- Plano: [`15_plano_redesenho_molde_rive.md`](../15_plano_redesenho_molde_rive.md) (versão B; «Keys | Curves» FORA; sem função nova).
  Onda anterior: [`..._ONDA1.md`](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-09_ONDA1.md).
- Fotos: [`15_onda2_depois.png`](../fotos/15_onda2_depois.png) (app, `PH2D_ANIMACAO_SMOKE=1`, 1930×1040) ·
  [`15_onda2_lado_a_lado_maquete_B1.png`](../fotos/15_onda2_lado_a_lado_maquete_B1.png) (em cima a maquete B1 capturada de novo
  em chrome headless — a variante com Auto Key ligado —, em baixo o app com chaves e barras à vista). Pendências fechadas na
  2.ª rodada: secção **2b** abaixo.

## O que a onda 2 fez (só cartões; `PH2D_CARDS=0` intacto)

1. **Nomes em árvore** (`linhas::layout`): nos cartões entra `Linha::Objeto(entity)` — uma linha de cabeçalho por objecto,
   nome em negrito (`paint_text_title_elided`) e um losango pequeno (`OBJ_DIAMOND_H`, `Text3`) por instante com chave.
   As faixas de um objecto juntam-se sob ele pela ordem da 1.ª aparição (o smoke trazia Herói, Espada, Herói intercalados).
   A faixa diz só a propriedade (`tracks_label::propriedade_snap`), inteira, com o valor à direita como antes; o cabeçalho
   de grupo idem. Unidade «(px)», «(°)» só quando o canal tem grandeza (`tracks_value::sufixo_da_unidade`, `Field`).
   O cabeçalho é só pintura: sem alvo, `keys_in_rect` ignora-o.
2. **Barra entre chaves** (`linhas::paint_barras`, `cor_da_barra`): por baixo dos losangos, 3 px, na cor da interpolação da
   chave de saída — `Hold` tracejada, `Linear`, o resto (eased/bézier/weighted) = curva. Segue o `preview_dx` do arrasto.
3. **Tokens novos** (apelidos, `ph2d-tokens`): `TimelineInterpCurve`→`Info`, `TimelineInterpLinear`→`Accent`,
   `TimelineInterpHold`→`Text3`, `TimelineRecord`→`Danger`. `color.rs` estava em 697/700: os 4 entram com `#[doc]` na mesma
   linha e o comentário da secção encolheu (698).
4. **Auto Key**: com `snapshot.auto_key` a folha ganha borda `TimelineRecord` (2×`FIELD_BORDER`), em `paint.rs` depois da régua.
5. **Depois do fim**: o `BgScrim` de `ruler::beyond_end_shade` ganha listras a 45° (`Border`) — peça genérica nova
   `ph2d-editor-core/src/panel/cards_banda.rs::listras_em` + `LISTRA_PASSO` (append-only).
6. **Enquadrar**: `state.enquadrado: Option<usize>`; trocar `active_clip` nos cartões faz o `apply_fit` (o `F`) sozinho.
   O 1.º quadro só anota (ver divergências).

## Testes (por nome) e fecho

- Novos em `linhas_tests.rs`: `nos_cartoes_o_objeto_e_o_cabecalho_das_suas_faixas`, `a_faixa_diz_so_a_propriedade_e_o_cabecalho_o_objeto`,
  `a_barra_veste_a_interpolacao_da_chave_de_saida`.
- Testes de disposição PLANA (`linhas` ×3, `geom` ×3, `box_select` ×4) passaram a forçar o clássico
  (`force_cards_on_this_thread(Some(false))`): medem a aritmética das faixas, não a árvore.
- Resultado: `linhas` 15 ✓ · `geom` 12 ✓ · `box_select` 7 ✓ · `tracks` 10 ✓ · `summary` 12 ✓ · `interact` 12 ✓ · `key_drag` 11 ✓ ·
  `os_cartoes` 21 ✓ (inclui o gate de altura estrito, sem mudança) · `_seam` 81 ✓ · `ph2d-tokens color` 17 ✓.
- `cargo fmt --all`; clippy `-D warnings --all-targets` limpo em `ph2d-panel-timeline`, `ph2d-editor-core`, `ph2d-tokens`;
  smoke `--profile smoke` do `ph2d-host-desktop` compilado.
- Nenhum controlo mudou de sítio nesta onda (sem teste `onde_mora` novo); ids e eventos os mesmos.

## Divergências / interpretação da maquete B1

- As propriedades dizem a palavra da Timeline («Translate X»), não a do Inspector («Position X»); a unidade só aparece em
  canais `Field` com grandeza — para `TranslationX/Y`/`Rotation` embutidos não confirmei a unidade do valor mostrado.
- (Cores, legenda, enquadrar ao abrir e a fita vazia do smoke: fechados na secção 2b.)
- O smoke só tem chaves lineares: a barra de curva e a segura aparecem na legenda, não na cena.

## 2b — pendências fechadas (coordenador, mesma onda)

1. **Abrir enquadrado**: `state.enquadrado` começa `None` e volta a `None` quando o painel se esconde
   (`release_everything_a_hidden_panel_must_not_hold`) ⇒ o 1.º quadro visível e cada troca de clipe chamam
   `view::apply_fit_clip`: o clipe inteiro `0 → view_length_seconds` unido às chaves (o `F` continua a enquadrar só as
   chaves). E nos cartões o seguir-o-cursor durante o Play **não persegue o cursor depois do fim do clipe** (`alem_do_fim`
   em `paint.rs`): a cena do smoke toca com o relógio em 9 s sobre um clipe de 2 s e arrastava a vista para a zona morta.
   Estrago medido (crate inteira, `PH2D_SUITE=motivo`): **0** testes por causa da vista; 12 de `duration_chip_gesture`
   estavam vermelhos desde a onda 1 (o «Length» foi para a janelinha) — curados abrindo `opcoes_abertas` antes de pintar;
   1 (`the_short_id_is_formatted_in_exactly_one_place`) era meu (`objeto_snap` montava `#nnnn`) — passa pela porta `compoe`.
   Teste novo `tests/it/a_fita_abre_enquadrada.rs` (cartões: 0 e 2 s à vista a tocar em 9 s; clássico: segue o cursor);
   duas mutações sangram (tirar `!alem_do_fim`; trocar `apply_fit_clip` por `apply_fit`).
2. **Cores medidas** (OKLCH dos slots gerais nos 8 temas): nenhum é ciano constante; `Info` 236–264° (o mais perto do ciano
   da maquete), `CodeKeyword` 305° em todos (violeta, claridade por tema), `Text3` quase sem croma. `Accent` recusado: muda
   de matiz por tema (340/206/42/257…). ⇒ curva `Info`, **reta `CodeKeyword`**, segura `Text3` tracejada. Gate
   `as_tres_barras_leem_se_diferentes_nos_oito_temas` (distância OKLab ≥ 0,10, a barra do `BoneHandle`): pior caso **0,107**.
3. **Legenda no pé** (`src/legenda.rs`): pílula `BgElev` no canto de baixo à direita da área das faixas, «Curve · Linear ·
   Hold» com a amostra de cada barra (`linhas::pinta_barra`, partilhada); i18n `panel.timeline.card.legend_{curve,linear,hold}`.
   Só pintura, sem alvo.
4. **Coluna de nomes**: os ▶ das propriedades FICAM — são o `timeline_twirl_id` (`TimelineHitKind::Twirl`, abre a curva).
   Unidades conferidas no código: `ph2d_ecs::Transform` guarda translação em **metros** e rotação em **radianos** ⇒ nos
   cartões `Translate X (px)` mostra `-300px` (o mesmo do Inspector; antes `-3.00`) e `Rotation (°)` converte de radianos
   (`tracks_value::grandeza`). O clássico continua com o número nu.
5. **A barra azul vertical** era o polegar da barra de rolagem das faixas (aparece porque a árvore alongou a lista). Nos
   cartões passou a fio de meia largura, sem trilho pintado; o alvo é o trilho inteiro como antes (`scrollbar.rs`).
6. **Maquete capturada** (chrome headless, `--virtual-time-budget=30000`, o elemento isolado por script): linha 22 px (o app
   usa `ROW_H_PX` = 22), coluna de nomes ~170 px (app ~186), losango de propriedade ~6 px de meia diagonal (app
   `DIAMOND_H` 4,5), do cabeçalho ~4 px (app `OBJ_DIAMOND_H` 2,7). Diferenças que ficam: a maquete pinta ícone por
   propriedade e o losango com contorno; a palavra é «Translate X» (a da Timeline), não «Position X».

Testes por nome desta rodada: `linhas` 16 ✓ · `tracks_value` 6 ✓ · `scrollbar` 6 ✓ · `scroll` 19 ✓ · `view::` 20 ✓ ·
`a_fita_abre` 1 ✓ · `os_cartoes` 22 ✓ · `_seam` 81 ✓ · `duration_chip_gesture` 15 ✓ · `every_word` 3 ✓ · `one_door` 2 ✓ ·
`ph2d-tokens color` 17 ✓ · i18n `ordenad`/`known_keys`/`a_text_key`/`honesto` ✓; crate inteira 300 + 161 ✓. Clippy
`-D warnings` limpo em `ph2d-panel-timeline`, `ph2d-editor-core`, `ph2d-tokens`, `ph2d-i18n`; fmt; smoke compilado.

- **2c (dono 09/11, cabeçalho do objecto):** fundo próprio de ponta a ponta (token `TimelineObjectRow` → `Bg3`, via `linhas::fundo_da_linha`), fio `BorderStrong` antes de cada objecto, ▾ SÓ PINTADO (não há id/evento de recolher objecto), ícone do tipo lido do `entry.icon` da Hierarquia em `hier::declare_in_row` (mapa `hier::publica_icone`/`icone_do_objeto`, sem linha nova na shell; `Cube` até a Hierarquia declarar), ícone por propriedade (`Move`/`Rotate`/`Scale`/`CardOpacity`), piso da coluna nos cartões `geom::min_arvore_label_w` (nomes inteiros); teste `o_cabecalho_do_objeto_tem_fundo_e_icones_proprios`; crate 301 + 162 ✓; `color.rs` 699/700.

- **2d (dono 09/11, Summary com cor própria):** `summary_paint::fundo` = `TimelineObjectRow` + `TimelineSummaryRow` (→ `Accent`) a `TIMELINE_SUMMARY_TINT_ALPHA` 0,15 (`ph2d-tokens/src/chrome.rs`); medido nos 8 temas: pior 0,049 contra cabeçalho/zebra/fundo (degraus existentes 0,020–0,112; âmbar a 0,20 dava 0,045 e o losango a 0,14 no Light); gate `o_summary_tem_cor_propria_nos_oito_temas` (≥ 0,04; texto e losango ≥ 0,15). Clássico continua `Bg3`. `color.rs` 700/700.

- **2e (dono 09/11, «capriche»):** chave nos cartões = métricas `ph2d_tokens::TIMELINE_KEY_HALF_PX` 6,5 + `_RING_PX` 1,5 (contorno `Bg0`; maquete medida: ~11 px com contorno ~1) e alvo `TIMELINE_KEY_HIT_HALF_PX` 8 (`tracks::paint_chave/chave_h/chave_hit_hw`; seam `o_alvo_da_chave_cobre_o_losango_com_contorno`, mutação sangra); ponto do objecto `TIMELINE_OBJECT_DOT_HALF_PX` 4 em `Text2`. Summary: 3 candidatos por `PH2D_SUMMARY_TOM` em execução ([comparação](../fotos/15_onda2_summary_candidatos.png)) — A faixa funda (`Bg0` + destaque a 0,25) com filete de 2 px: fundo 0,045 · texto/losango 0,235; B âmbar: losango a 0,148 (reprovado); C célula do destaque a 0,45: 0,045 · 0,189 e compete com o chip de clipe. **A escolhido**; override removido; `TIMELINE_SUMMARY_TINT_ALPHA` 0,25; gate apertado a 0,044 / 0,23. Zoom: [`15_onda2_losangos_zoom.png`](../fotos/15_onda2_losangos_zoom.png).

## Superfície de colisão

`ph2d-panel-timeline`: `linhas.rs`, `tracks.rs`, `tracks_label.rs`, `tracks_value.rs`, `paint.rs`, `ruler.rs`, `state.rs`,
`view.rs`, `scrollbar.rs`, `legenda.rs` (novo), `lib.rs`; testes `linhas_tests.rs`, `geom_tests.rs`, `box_select.rs`,
`tests/it/a_fita_abre_enquadrada.rs` (novo), `tests/it/duration_chip_gesture.rs`, `tests/it/main.rs`. `ph2d-i18n/src/timeline.rs` (+3 chaves). `ph2d-tokens`: `color.rs` (698/700), `color_alias.rs`, `skin_body.rs`
(+4 tokens no `ColorToken::ALL`). `ph2d-editor-core`: `cards_banda.rs` (append). Sem `PROJECT_SCHEMA`, contrato §6 nem formato gravado.
Campo de estado +1 (`enquadrado`).

## Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Na Timeline em baixo, coluna da esquerda: «Herói» em negrito e por baixo «Translate X» e «Translate Y» com o número à
   direita; depois «Espada» e «Rotation». Nada cortado com «…».
3. Ao abrir, a régua já mostra o clipe inteiro (0 → 2 s) com as chaves e, entre elas, barras violetas (reta); no canto de baixo
   a legenda Curve · Linear · Hold. À direita do fim do clipe a fita fica listrada. Clicar «pular» enquadra o outro clipe.
4. Ligar «Auto Key»: a borda da folha fica vermelha; desligar: some.
5. Errado se: um nome aparece cortado, um objecto aparece duas vezes, clicar numa chave já não a seleciona, ou nada encaixa
   ao trocar de clipe.

## Próximo

- Pendências desta onda: nenhuma aberta; a palavra do Inspector («Position X») nos canais embutidos fica por decidir com o dono.
- **Onda 3 do plano**: Containers e Arrange — chips das peças («used 2×» / «not placed»), lanes com peso e olho na coluna,
  o mesmo desenho de strips; sem gesto novo.
- Smoke do dono NÃO feito; sem push nem integração.
