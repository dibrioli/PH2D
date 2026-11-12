# Barra de Menus — plano da linha `line/BarraDeMenus` (12/11/2026)

Fonte: crítica do dono `~/Documentos/GameEngineSamples/critica_ph2d.html#menubar` (fora do repo).
Decisão do dono 12/11: **versão C** («a faixa que responde») — memória
`project-memory/project_menu_bar_version_c_2026_11_12.md`. Revoga a D2 «menus à vista».
Layouts Scene·Draw·Vector·Flip·Animate·Nodes **saem**. Lições **fora** da busca.

## O que existe (medido no código, 12/11)

| peça | onde | estado |
|---|---|---|
| tabelas dos menus | `ph2d-editor-core/src/screens/hero/menu_tables.rs` (`MENU_BAR_*_ROWS`) | `&[MenuRow]` = `(NodeId, TextKey, swatch)`; sem divisor |
| pintor dos menus | `screens/hero/context_menu_overlay.rs:73` | já pinta `MenuRow::Divider` (contribuído) e linha indisponível (`menu_row_is_unavailable`) |
| linhas ocultas | `interaction/state/store_hierarchy.rs:213` `menu_row_is_hidden` | as 6 Pixel do File SOMEM |
| comandos de família | `doc_surface.rs:83` `DocMenu{File,Edit,Tab}`; kanban `surface.rs:156`, planner `:608`, sfxlab `:133` | caem no File |
| barra | `screens/hero/menu_bar.rs:359` `paint_menu_bar` (títulos · abas de doc · transporte · layout_tabs) | |
| cascata | `chrome/menu_bar.rs` + `chrome::cascade_anchor` | Preferences/Theme/Scenes |
| paleta Ctrl+K | `screens/hero/global_palette.rs` (`keywords` vazio), desenho `widget/command_palette/` | nuvem de chips |
| título | `shells/desktop/src/render_loop/present_title.rs:38` | diagnóstico de obra |
| janela | `shells/desktop/src/init.rs:89` | decorações do SO; nada de arrasto pelo cliente |

## Ondas

1. **Arrumação** (vale para A/B/C): título = projeto; atalho em coluna à direita (derivado do rótulo,
   sem segunda tabela); grupos com divisor; File = New ▸ · Open · Scenes · Save · Save As · Import ▸ ·
   Export ▸ (sempre as seis, cinzentas sem Pixel); comandos de família Kanban/Planner/Som → Window ▸ Plan
   / Audio; Window agrupado (Pixel ▸, Audio ▸, Plan ▸) e Input Map nele; Help nasce (Commands…, Developer ▸
   com Design Tokens · Authored UI · Widget Gallery · Widget Lab); layouts saem da faixa; «›» só onde há
   submenu.
2. **A faixa C**: botão W («Menu») abre os cinco menus em cascata; Alt abre-o; no centro o campo
   «Find anything» (abre a paleta); a paleta em LISTA com caminho («File · Export») e atalho por linha,
   recentes em cima; Play com nome.
3. **A busca que responde**: ações do Input Map com as teclas, objetos da cena (seleciona), campos dos
   cartões do objeto; «Alt+Enter: show me where it lives» abre o menu de verdade e acende a linha;
   loja de preferências por utilizador → contador «3.ª vez pelo menu» e o chip que ensina o atalho;
   Open Recent.
4. **Faixa única**: sem decoração do SO, min/max/fechar desenhados, arrasto e redimensionar pelo app —
   testado no KWin (Wayland) antes de virar omissão.
