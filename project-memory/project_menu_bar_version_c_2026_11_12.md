---
name: project_menu_bar_version_c_2026_11_12
description: Dono 12/11 — barra de menus = versão C da crítica (faixa que responde): menus no W, busca no centro; revoga a D2 «menus à vista»; layouts Scene·Draw·… saem; lições fora da busca
metadata:
  type: project
---

Dono, 12/11/2026, sobre `~/Documentos/GameEngineSamples/critica_ph2d.html#menubar` (linha `line/BarraDeMenus`):

1. **Versão C** — uma faixa só; File/Edit/View/Window/Help dentro do botão **W** (rotulado «Menu», Alt abre);
   no centro o campo «Find anything» (Ctrl+K) em LISTA, com caminho e atalho por linha; «Alt+Enter: mostre
   onde mora»; o chip que ensina o atalho depois da 3.ª vez pelo menu. **Revoga a D2** (30/08, «menus do app
   à vista no topo») — confirmado pelo dono.
2. Tirar a barra do sistema (faixa única, botões de janela desenhados pelo app): **decisão técnica delegada**
   — vai por último, testada no KWin antes. **Feito 12/11 (ondas 4–5):** smoke do dono OK com rato e caneta; é a
   omissão (`PH2D_FAIXA_UNICA=0` volta à barra do sistema).
3. **Lições fora da busca** por enquanto (não existem no app; tutoriais são PDF).
4. **Os layouts Scene · Draw · Vector · Flip · Animate · Nodes SAEM da faixa** (confirmado; não estava anotado).

A arrumação comum às três (título = projeto, File 8 linhas com New/Import/Export/Open Recent, Kanban/Planner/Som
para Window, separadores, atalho em coluna, Help + Developer, Input Map em Window, cinzento-nunca-some, Play
com nome) é a onda 1.

5. **Acabamento = a maquete C em repouso** (dono 12/11, depois das fotos lado a lado): faixa com a ALTURA da maquete
   (30 px), a barra de ferramentas logo abaixo FINA (26 px, glifos 13 px, texto apagado, como `.c15tl`), o logo
   **«PH2D»** no lugar do W (o «Menu» continua), Play pílula verde, busca bela como a maquete (campo 22 px com a tecla
   em chip; a lista: linha escolhida cheia, termo em negrito/acento, chip do tipo à direita, rodapé com as teclas).
   **Cores do app** (não o azul-marinho da maquete). ⛔ **Contraste:** todo controlo ≥ 3:1 contra o chão da faixa
   (WCAG 1.4.11), texto ≥ 4,5:1 — o campo estava 1,1:1 (medido na foto 12/11).

6. **Smoke 13/11 da onda 6 não aprovado:** «não é bela o suficiente, não bem alinhada, barras um pouco mais largas,
   ícones melhores e mais nítidos; busque o estado da arte; acabamento e polimento intenso» → onda 7.

7. **Onda 7 (13/11) — feita, à espera do smoke:** referências MEDIDAS (VS Code 35/24, Godot ~38/39, Blender 27/28; ícones
   14–16 com o contraste do texto). Faixa 36 com controlos 26, fila 32 com chips 24, glifos de 16 nítidos (traço 1,5 px
   de ecrã), uma altura e um centro para tudo, busca abaixo da faixa. Handoff `HANDOFF_CONTINUACAO_…_ONDA7.md`.

8. **Smoke 13/11 da onda 7: aprovado («ótimo! Muito bom!»)**, com dois pedidos: (a) BUG — menu aberto não fecha ao
   clicar fora (agente em curso); (b) **onda 8 — os menus como a maquete** (`critica_ph2d.html#menubar`, `.c15dd`/`.c15it`):
   ícone à esquerda de cada item, divisórias entre grupos (`.c15sep`), atalho em coluna à direita, o item que abriu o
   submenu aceso no acento. Hoje: sem ícones e sem divisórias na raiz («ainda não estão tão bonitos»).

9. **Onda 8 (13/11) — smoke do dono OK (13/11):** menus na cara da maquete (ícone de 16 em cada linha, risco curto,
   atalho em coluna, caminho aceso no acento, largura 240, linha 24; contraste nos 12 temas). Handoff
   `HANDOFF_CONTINUACAO_…_ONDA8.md`.

10. **Fecho 13/11:** Pause/Stop **sempre à vista** (dono); texto dos menus **mais leve, nítido em 1080p** = peso exato 550
    (medido). Linha FECHADA, à espera da ordem de integrar: `HANDOFF_INTEGRACAO_line_BarraDeMenus_2026-11-13.md`.

**Why:** a C é a única que ensina (onde mora + atalho); a crítica recomendava a B, o dono escolheu a C.
**How to apply:** não reponha menus à vista na faixa nem os layouts; mitigar o «menu escondido» com o rótulo
e o Alt, nunca voltando à B. Ver [[feedback_app_ui_english_only]].
