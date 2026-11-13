# HANDOFF de continuação — line/BarraDeMenus, onda 8 (13/11): os menus na cara da maquete C

Antes: [onda 7](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-13_ONDA7.md). Pedido do dono (smoke da onda 7 aprovado):
*«nossos menus sem ícones e sem divisórias ainda não estão tão bonitos»*.

## Medido antes (oráculos corridos, `ferramentas/oraculos_onda8.sh`)
VS Code: linha 24, divisória 1,7:1, atalho 5,7:1. Godot: linha 23, divisória 1,2:1, atalho 3,2:1. Maquete C: linha 23,
grelha `16 | rótulo | atalho | 10`, vão 7, largura 240. Tabela: [`fotos/oraculos/LEIA-ME.md`](../fotos/oraculos/LEIA-ME.md).

## Feito (um bloco)
- **Cara da barra** (`menu_barra_linha::veste`): os cinco menus, a raiz, a seta do Play e todo submenu aberto a partir
  deles (Preferences ▸ …, Theme). Os outros menus do app ficam como estavam.
- `menu_barra_nivel.rs` pinta o nível: largura `chrome.menu-w` (240), linha `chrome.menu-row-h` (24), risco curto com
  `Xs` de cada lado (era uma banda de 17 px). `menu_barra_linha.rs` pinta a linha: ícone nítido de 16
  (`paint_icon_nitido`), rótulo com «…», atalho numa coluna, «›» na última coluna reservada em toda linha.
- **Aceso no acento**: a linha que abriu o submenu e a linha sob o rato (`.c15it.on`), texto na tinta do acento.
- **Ícones**: tabela `menu_icones::ICONES` (raiz, File, Edit, View, Window, Help, Developer, Play, as categorias das
  Preferências). Linhas de família: campo novo `DocCommand::icon` (Kanban, Planner, SFX Lab, Code); projetos recentes:
  `File`; contribuídas por módulo: o `ToolRailEntry::Icon`. Listas de VALORES sem ícone: o visto na coluna do glifo.
  Itens ligados com ícone (Hierarchy, Rulers, «Play in a Window»): o ícone fica, o visto vai à direita.
- **Tons** (`menu_tons.rs`): rótulo/atalho ≥ 4,5, ícone ≥ 3, aceso ≥ 3 com texto ≥ 4,5, risco ≥ 1,5 (decorativo,
  acima dos oráculos), nas 12×2 (cartões e antes). Plumber Red/Green na UI de antes: nenhum tom do tema passava de 4,1
  sobre o BgElev ⇒ o fundo do menu escolhe o primeiro `Bg*` do tema onde o texto lê.
- Dica do botão (a seta do Play) não se pinta com um menu da barra aberto (tapava as linhas).
- Tokens novos `chrome.menu-w` 240, `chrome.menu-row-h` 24 (+ `design_token_sync`).

## Portões
Novos: `os_menus_da_barra_tem_contraste_em_todos_os_temas`, `toda_folha_dos_menus_da_barra_tem_icone`,
`o_glifo_e_o_rotulo_tem_uma_coluna_so`. Mexido: `a_diagonal_para_o_filho_nao_troca_de_submenu` — o alvo passa à borda
de entrada do Save (com 240 a recta até ao centro entrava no filho pela Open Recent, que abre o dela; o que se mede é a
irmã da RAIZ não trocar). Verdes por nome: menu_cascade, context_menu_overlay, menu_faixa, menu_bar, global_palette,
menu_teach, menu_reveal, menu_row, document_tabs (96), design_token_sync (9).

## Fotos (antes do smoke)
`fotos/onda8_lado_a_lado_file_export.png` (antes · maquete · agora, mesma escala 2×) · `onda8_x3_file_export.png` ·
`onda8_x3_view.png` · `onda8_{1..7}_*.png` · antes: `onda8_antes_*`. Script: `ferramentas/fotos_onda8.sh <prefixo>`.

## Smoke do dono (13/11): APROVADO.

## Aberto
- O texto do app é mais pesado que o da maquete (peso da fonte é preferência do app, Edit ▸ Preferences ▸ Font weight).
- Mutação dos portões novos e clippy/fmt das crates tocadas: no fecho da linha.
