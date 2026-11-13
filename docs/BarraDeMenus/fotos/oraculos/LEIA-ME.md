# Oráculos da onda 7 (13/11) — o topo de três editores, medido

Corrida: `PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/BarraDeMenus/ferramentas/oraculos_onda7.sh`
(Xvfb própria 1930×1040, escala 1, HOME de faz de conta, só a SAÍDA dos apps). Régua:
`python3 docs/BarraDeMenus/ferramentas/regua_faixa.py <foto> --y0 A --y1 B`.

| app (pacote, licença) | barra de cima | barra de baixo | ícone desenhado | pico do ícone / texto |
|---|---|---|---|---|
| VS Code 1.145.2 (`visual-studio-code-bin`, comercial) | 35 (campo 24) | 29 | 14–16 | 0,595 / 0,595 |
| Godot 4.7.2 (MIT) | ~38 | 39 | 14 | 0,536 / ~0,55 |
| Blender 5.2.3 (GPL…) | 27 | 28 | 14–15 | 0,73–0,75 / 0,74 |
| PH2D onda 6 | 30 (controlos 22/24/28) | 26 | 9–10 | 0,16–0,40 / 0,50 |
| PH2D onda 7 | 36 (controlos 26, segmentos 22) | 32 (chip 24) | 16 | 0,50 / 0,50 |

## Onda 8 — os MENUS abertos (13/11, `ferramentas/oraculos_onda8.sh`)

| | linha | divisória (contraste) | rótulo | atalho | ícones |
|---|---|---|---|---|---|
| VS Code 1.145 (`vscode_menu.png`) | 24 px | 1 px, 1,7:1 | 10,3:1 | 5,7:1 | não (coluna de visto reservada) |
| Godot 4.7 (`godot_menu.png`) | 23 px | 2 px, 1,2:1 | 10,7:1 | 3,2:1 | não no menu principal |
| Maquete C (`.c15it`) | 23 px | 1 px, ~1,3:1 | — | — | 16 px, vão 7 |
| **PH2D onda 8** | **24** (`chrome.menu-row-h`) | **≥ 1,5:1** | **≥ 4,5** | **≥ 4,5** | **16 px nítidos, ≥ 3:1** |
