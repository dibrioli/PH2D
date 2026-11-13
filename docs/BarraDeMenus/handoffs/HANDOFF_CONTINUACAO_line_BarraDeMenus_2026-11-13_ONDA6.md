# HANDOFF de continuação — line/BarraDeMenus, onda 6 (12–13/11): o acabamento da maquete C

Antes: [onda 5](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA5.md). Pedido do dono com as fotos lado a lado:
fica a **C** com o acabamento da maquete, **cores do app**, «PH2D» no lugar do W, a altura da maquete, a fila de baixo
fina, a busca «muitíssimo mais bela», e **contraste suficiente entre widgets e fundo** (memória
`project_menu_bar_version_c_2026_11_12.md` §5).

## O que mudou (3 commits)
- **Faixa** (`fc5a8bef52` + o da plaquinha): token `chrome.menu-bar-h` = 30; controlos de 22 px (`ROW_H_PX`) centrados
  (`menu_faixa::caixa`, `transport_bar::controlo`); marca «PH2D» (pílula no acento) + «Menu» + seta; campo com filete e
  «Ctrl+K» em plaquinha; Play pílula cheia no tom de sucesso. Tons em `screens/hero/faixa_tons.rs` — portão
  `a_faixa_tem_contraste_em_todos_os_temas` (controlo ≥ 3:1, texto ≥ 4,5:1, plaquinha ≥ 1,5:1 contra o campo; 12 temas;
  mínimo 3,08 / 4,50). Onde o tom do tema não chega, puxa-se o mínimo (sem hex novo).
- **Fila de ferramentas fina** (`f56974ecd1`, agente): `widget/tool_rail/fila_fina.rs`, tokens `chrome.tool-row-*` (26 px,
  glifo 13, sem legendas, o nome na dica); portão `a_fila_fina_tem_contraste_em_todo_tema` (mín. 4,64). A barra de um
  módulo com campos numéricos (Vector) segue alta.
- **Busca** (`2140049c23`, agente): parte casada a negrito na cor da chave (`busca::match_ranges`), «onde mora · atalho»,
  plaquinha do tipo (Action · Setting · Object · Command), escolhida cheia, «N kinds» no campo, rodapé de duas dicas;
  portão `a_busca_tem_contraste_em_todos_os_temas` (mín. 4,54). Sem «Lesson» (não existem). `fotos_onda6.sh`.
- **Portões do tablet** (`the_chrome_never_eats_more_of_a_tablet_than_this`): perguntam a altura da fila ao produto
  (`tool_bar_h_da_loja`), não à fórmula alta; o saldo mediu 51,6 · 50,9 · 50,4 % e o piso subiu para 51,5 · 50,8 · 50,3.
  O controlo de transbordo do pincel vale só para a fila alta (a fina leva as 18 numa linha).
- O convite do campo ficou «Find anything» (o longo da maquete é cortado em fontes grandes: `nenhum_rotulo_medido_do_chrome_e_cortado`).

## Aberto
- Glifos da fila fina desenham ~10 px (o pintor de ícone recua dentro da caixa de 13).
- Pause/Stop seguem à vista em repouso (a maquete C só os mostra a jogar) — não mexido.
- Os chips «Hierarchy»/«Inspector» das colunas dividem a fila com as ferramentas (layout de colunas, não desta linha).
- `PH2D_CARDS=0` (saída de referência) fora dos portões de contraste.

## Fotos
`fotos/onda6_app_inteiro.png` · `onda6_faixa_maquete_em_cima_app_embaixo.png` · `onda6_maquete_em_cima_app_embaixo.png`
(fila) · `onda6_busca_lado_a_lado.png`.
