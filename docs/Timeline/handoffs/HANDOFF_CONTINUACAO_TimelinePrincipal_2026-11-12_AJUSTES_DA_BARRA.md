# HANDOFF CONTINUAÇÃO — line/TimelinePrincipal, ajustes da barra (dono 12/11)

Leitor: a próxima LLM. A linha estava FECHADA para integração ([handoff de integração](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-12_PLANO17_ONDA5.md),
commit `79810cba51`); depois disso o dono pediu ajustes, feitos no commit `aabe9342da`. Sem rebase, sem push, sem integração.

## Feito em `aabe9342da` (verde: clippy -D warnings em panel-timeline/editor-core/i18n/host-desktop; por nome os_cartoes 35, clipes_em_lista 2, seam 131, transport 24, icons 9, card 145, shrinks 2, file_loc_cap 2, hr15 1, host the_motion_path 13, timeline 86; 2 mutações sangraram)
1. Caixa de marcar VAZIA da barra: contorno `TonsDaBarra::borda_da_caixa` = `TimelineTinta::TextoPropriedade` (6,2:1 contra o `CardField`; era `CardLabel` 3,06). Porta nova `paint_card_check_com_borda` (`widget/checkbox/card_box.rs`); só `emoldurada_discreta_em` (só a Timeline) a usa. Teste `transport::cartoes::tests::a_caixa_vazia_da_barra_tem_contraste_com_o_fundo` (todos os temas).
2. «Record while playing» saiu da barra para «Timeline options» (`barra_menus::corpo_das_opcoes`); `caixa_de_gravar` só com Auto Key.
3. «Show clips as a list» (`TIMELINE_CLIPS_AS_LIST`, `state.clips_as_list`, sessão, nunca gravado): `barra_clips::como_lista` pinta a lista de sempre (`TIMELINE_CLIP_DD`) com o nome do escolhido, o «⋯» (`pinta_reticencias`) e o «+». Teste `tests/it/clipes_em_lista.rs`.
4. «Edit curves on canvas» → **«Edit Path»** (i18n `panel.timeline.curves_on_canvas`; a dica continua a explicar).
5. Time e Frame: `CardCtx::campo_compacto` (`cards_banda.rs`, `CAMPO_COMPACTO_W` = 0,8 × 82 = 65,6 px), sem rótulo ao lado; ícones `Timer` e **`FilmFrame`** (novo, `docs/design/icons/film-frame.svg`); dicas `panel.timeline.tip.time|frame`. O Flip continua com `campo_em_fila`.
6. Transporte com traço `StrokeToken::Thick` (`icones_em_fila_com_traco`; `icones_em_fila` continua `Default`).
- Fotos: [barra](../fotos/17_ajustes_12_11_barra.png) · [engrenagem](../fotos/17_ajustes_12_11_opcoes.png) · [clips em lista](../fotos/17_ajustes_12_11_clips_lista.png).
- Shell intocada; `PROJECT_SCHEMA` 237 e `DOC_VERSION` 25 intocados.

## ABERTO → estado em 12/11 (2.ª janela)
Os itens 1, 2 e 4 foram FEITOS: ver o «Acréscimo» do [handoff de integração](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-12_PLANO17_ONDA5.md). O item 2 era defeito (o Done soltava as faixas e trocava o clip). O item 3 continua sendo decisão do dono.

### Texto original
1. **Os ícones DENTRO da lista suspensa de clipes têm de ser os mesmos de fora** (o chip usa `barra_clips::glifo_do_tipo`: clipe = `Film`, container = `Cube`; a lista aberta de `TIMELINE_CLIP_DD` usa o `DropdownOption::icon` de `transport_clips::source_options`). Cura provável: uma porta só para o glifo do tipo, lida pelo chip, pela lista em modo «as a list» e pela lista aberta. Foto antes/depois.
2. **«Antes de apertar Done, as curvas Bezier e de velocidade não aparecem na Timeline»** — reproduzir no app (`PH2D_ANIMACAO_SMOKE=1`, fotos antes/depois do Done). Leitura do código (explorador): nada liga `animating` à pintura das curvas; a banda do grafo só pinta com a linha EXPANDIDA (`linhas.rs:163`, `tracks.rs:302`, `state.expanded`, ▶). Medir se o Done (restauro/`apply_project_with`) expande linhas ou se outra coisa muda; se for defeito, `/pd-bug-causa`.
3. O dono ainda não vê o porquê do Done (explicado: «terminei de animar» — pára, rebobina, desliga Auto Key, devolve a pose; a prévia não entra no undo). Se ele decidir mudar, é decisão de produto dele.
4. Ao terminar: acrescentar estes ajustes ao [handoff de integração](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-12_PLANO17_ONDA5.md) (superfície: `ph2d-editor-core` cards_banda/cards_banda_tons/checkbox/icons + ícone novo — ⚠️ o discriminante do `IconId` desloca-se uma casa depois de `film`; ninguém grava o número, só o slug) e re-correr a bateria por nome das crates tocadas.
