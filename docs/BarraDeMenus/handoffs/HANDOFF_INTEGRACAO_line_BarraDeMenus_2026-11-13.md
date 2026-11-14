# HANDOFF de INTEGRAÇÃO — line/BarraDeMenus (12–13/11): a faixa única da maquete C, a busca e os menus

Linha FECHADA (13/11), 37 commits sobre o merge-base `4f46a3573c` (= o `main` em 13/11: rebase trivial). Smokes do
dono: ondas 1–8 aprovadas (a 6 refeita na 7) e o do fecho (texto 550, `PH2D_GAME_SMOKE=2` ▸ Menu ▸ File) **OK 13/11**.
Árvore limpa. **Não integrada** — o integrador entra só por ordem explícita do dono.
Plano: [`01_plano.md`](../01_plano.md) · decisões: `project-memory/project_menu_bar_version_c_2026_11_12.md` (itens 1–10).

## O que a linha entrega (por onda; o detalhe está no handoff de cada uma)
1. Arrumação dos menus (File 8 linhas, grupos, cascatas, Help ▸ Developer, Input Map em Window) — [ONDA1](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA1.md)
2. Versão C: «Menu» com os cinco, Alt abre, busca «Find anything» no centro, seta do Play — [ONDA2](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA2.md)
3. A busca que responde (Input Map, objetos, campos), Alt+Enter, Open Recent, chip que ensina — [ONDA3](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA3.md)
4–5. Faixa única (botões de janela do app, KWin; caneta move/redimensiona) — omissão, `PH2D_FAIXA_UNICA=0` volta — [ONDA4](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA4.md) · [ONDA5](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA5.md)
6–7. Acabamento medido (faixa 36, fila 32, glifos nítidos de 16, contraste nos 12 temas, busca bela) — [ONDA6](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-13_ONDA6.md) · [ONDA7](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-13_ONDA7.md)
8. Menus na cara da maquete (ícone, risco, atalho em coluna, caminho aceso; texto 550 exato) — [ONDA8](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-13_ONDA8.md)

## Decisões do dono no fecho (13/11)
- Pause/Stop **sempre à vista** (a maquete só os mostrava a jogar): fica como está.
- Texto dos menus **mais leve sem perder definição em 1920×1080**: peso EXATO 550 (medido 400…600 contra VS Code e
  Godot, `ferramentas/regua_texto.py`, tabela em `fotos/oraculos/LEIA-ME.md`).

## Fecho da linha (feito)
- **clippy/fmt** das 7 crates tocadas (ph2d-editor-core, -tokens, -pen, -app-kanban, -app-planner, -app-sfxlab, -app-code):
  zero avisos. A `ph2d-pen` sem a feature `winit` (a `ph2d-pixel-ponte`) tinha o arrasto da janela como código morto:
  agora `#[cfg(feature = "winit")]`; testes da pen 29 verdes com a feature.
- **Mutação** (12): contraste dos menus ×3, ícone em toda folha, grelha ×2, uma altura e um centro, traço do token,
  origem no meio do pixel x e y (sobrevivia — o portão `o_glifo_das_barras_tem_o_traco_do_token_em_qualquer_lado` passou
  a conferi-la), clicar fora fecha. Todas RED.
- Fotos na UI de antes (`PH2D_CARDS=0`): `fotos/onda8c_ui_classica_*`.

## Para o integrador
- **Contratos §6:** nenhum tocado. **Schema:** nenhum. **Tokens novos** (`docs/design/tokens.json` → gerados):
  `chrome.menu-bar-h` 36, `menu-bar-ctl-h` 26, `tool-row-*`, `bar-icon` 16, `bar-icon-stroke` 1,5, `menu-w` 240,
  `menu-row-h` 24 (+ `design_token_sync`).
- **Campo novo** `DocCommand::icon` (ph2d-editor-core) — todo produtor fora desta linha que construa `DocCommand` precisa
  dele (nesta árvore: kanban, planner, sfxlab, code e um teste).
- **CLAUDE.md §5.1:** não há entrada da barra de menus; sugerida (≤ 700 B): «**Barra de menus** — faixa única da
  maquete C: «Menu» com File/Edit/View/Window/Help (Alt), busca «Find anything» (Ctrl+K) que acha comandos, ações,
  objetos e campos, Play com a seta; menus com ícone e atalho. `PH2D_FAIXA_UNICA=0` = barra do sistema. Último: este handoff.»
- Memórias do dono escritas no PRIMÁRIO (não rastreadas lá): `project_menu_bar_version_c_2026_11_12.md` (+ ponteiro no
  `MEMORY.md`) — entram com a rodada.
- Portões por nome que a linha tocou: ver os handoffs 7 e 8 (secção «Portões»); a suíte completa é da integração.

## Aberto (nada bloqueia)
- Nenhum item do dono por fazer nesta linha.
