# 20 — A barra de rolagem do tempo com alças de zoom (dono 16/11)

> Pedido: «Precisamos de uma barra de scroll horizontal na timeline com alças de zoom na própria barra.
> Godot faz assim. Mas veja se alguém faz melhor.» Escolha pelo estado da arte, sem perguntar
> ([feedback](../../project-memory/feedback_finish_choices_go_to_the_state_of_the_art.md)).
> Código: [`barra_tempo.rs`](../../crates/ph2d-panel-timeline/src/barra_tempo.rs).

## Referências (pesquisa 16/11)

| App | O que a barra faz | Fonte |
|---|---|---|
| **After Effects** — Time Navigator | Duas chavetas = início e fim visíveis; puxar uma para a outra dá zoom, o meio rola. Duplo clique = zoom aos quadros e volta; Shift+duplo clique = composição inteira e volta. | [manual AE](https://www.manualsdir.com/manuals/753844/adobe-after-effects.html?page=201) · [Adobe community](https://community.adobe.com/t5/after-effects-discussions/request-for-documentation-timeline-elements/m-p/14877811) |
| **Premiere Pro** — zoom scroll bar | Pontas arrastáveis = zoom, meio = rolar, duplo clique = a sequência inteira. Defeito relatado: a ponta clicada fica «escolhida» e engole o espaço. | [Adobe community](https://community.adobe.com/t5/premiere-pro-discussions/premiere-pro-2024-timeline-scroll-zoom-bar-function-changed/m-p/14686647) |
| **Blender** — View2D scrollers | Pontas de zoom só quando o eixo pode dar zoom e o polegar é largo o bastante; o polegar tem tamanho mínimo e dessincroniza do rato nos extremos. | [commit 5a1603374c](https://projects.blender.org/archive/blender-archive/commit/5a1603374c768d7a9c7d9cb4b05ca4098ca32537.diff) · [bug #65740](https://developer.blender.org/T65740) |
| **Godot 4** — AnimationPlayer | Barra de rolagem horizontal e um **slider de zoom separado** («Fit to panel» ao lado); a barra em si não tem alças. | [fórum Godot](https://forum.godotengine.org/t/help-zooming-animation-track-y-axis-view-without-ctrl-alt-mouse-or-trackpad/144492) |

## A escolha (o melhor de cada)

1. **Polegar = janela visível** sobre `0 → max(fim do clipe, conteúdo da aba, fim da vista)`.
2. **Pontas = zoom; a OUTRA ponta fica parada** (AE/Premiere/Blender). Melhor que o Godot: um gesto só.
3. **Meio = rolar**; **calha = salta** (centra a janela no clique e o mesmo arrasto continua a rolar —
   o «clicar para posicionar» moderno, em vez do salto por página).
4. **Duplo clique = caber o clipe todo; outro duplo clique VOLTA** à vista de antes (a alternância do AE,
   sem precisar de Shift). Usa o mesmo enquadrar de abrir a fita (`apply_fit_clip`).
5. **Roda sobre a barra rola o tempo** (sem Shift); Ctrl+roda dá zoom pelo meio da vista.
6. **Marcas das chaves e o playhead na calha** (minimapa à Resolve): com zoom perto, vê-se onde está o resto.
7. **Mapa congelado durante o arrasto** (o defeito do Blender #65740: o «tempo todo» cresce quando a
   vista passa do fim, e o polegar fugia do rato). Sem estado «escolhido» depois de largar (o defeito do
   Premiere): a barra não guarda foco.

## Desenho

- **Uma vista só:** a barra escreve `view_start_s`/`px_per_s` do `TimelinePanelState` — a MESMA que a régua,
  as chaves e os gráficos expandidos lêem (`paint.rs`, `graph::TimeView`). Nada de segundo estado de tempo.
- **Lugar:** `geom::resolve` tira `TIMELINE_BARRA_TEMPO_H_PX` (14) do pé da região; a régua, o playhead e as
  linhas param por cima; a barra fica só sob a coluna do tempo.
- **Alvos:** `TimelineHitKind::TimeBar { part }` (0/1 pontas, 2 meio, 3 calha), registados calha → meio →
  pontas (o último ganha); `wants_double_click` inclui a barra; cursor `↔` nas pontas, mão no meio.
- **Medidas:** `ph2d_tokens::chrome::TIMELINE_BARRA_TEMPO_*` (altura 14, recuo 2, ponta 10, polegar mín. 36). As pontas são
  pintadas inteiras mais escuras, sem risco (dono 16/11).

## Provas

`barra_tempo_seam` (cartões, despacho real: lugar e alvos, pontas, meio, calha, duplo clique e volta, roda) ·
`barra_tempo::tests` (mapa, polegar mínimo, limites das pontas, meio) · `exactly_the_container_bar_and_the_time_bar_want_a_double_click` ·
`as_pontas_esticam_o_corpo_agarra_o_fundo_e_a_seta`.
