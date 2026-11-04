# 17 — Varredura final da nova UI: o que ficou fora dos cartões

Leitor: quem continua a linha. O censo inicial viu os 29 `impl Panel` + chrome ([15](15_CHROME_CENSO.md)); aqui, o que é aba, modo ou sobreposição. Tudo olhado em foto a `1930 × 1040` (04/11), salvo onde a linha diz «não olhado». Regra: nas áreas de trabalho (canvas do Quadro, texto do editor) o canvas não muda, só o que o rodeia.

Custo: **P** até 30 min · **M** uma fatia · **G** mais de uma fatia. Ordem = mais visto e mais barato primeiro.

| # | Peça | Cena | No padrão? | O que falta | Custo | Estado |
|---|---|---|---|---|---|---|
| 1 | Área do documento (Quadro, Kanban, Planner, Editor) | `PH2D_BOARD_SMOKE=13`, `KANBAN=1`, `PLANNER=1` | **Não** | O documento passa por baixo do Inspector: os painéis Live, Voting, Private e Timer ficam tapados («QBM-G» cortado), «Share» e «+ Event» sob a aba | P | ⬜ |
| 2 | Faixa cinza atrás da Hierarchy numa aba de documento | `BOARD=1`, `CODE=5` | **Não** | Cena a aparecer à esquerda do documento; acaba a meio da altura no Editor | P | ⬜ |
| 3 | Avisos (toast) | `MACRO=1`, `MACRO=8` | **Não** | Filete e faixa de cor do clássico, cantos vivos; o aviso da 1.ª linha sai sem corpo, texto solto sobre a barra | P | ⬜ |
| 4 | Painéis do Quadro: Live, Voting, Private, Timer, Comments, Frames, Minimap | `BOARD=9`, `10`, `12`, `13`, `14`, `29` | **Não** | Chapa, botões e linhas pretos; mesma porta `board_nav::panel` | M | ⬜ |
| 5 | Barra de estilo do Quadro | `BOARD=28`, `29` | Parcial | Pílula preta sem a superfície da lista | P | ⬜ |
| 6 | Cluster do canto (Live, Present, Comments, Nav) e gavetas da barra | `BOARD=1`, `29` | Parcial | Fundo preto; deve ter a superfície da lista | P | ⬜ |
| 7 | Aviso de menção do Quadro | `BOARD=28` | Parcial | Chapa preta, cantos vivos | P | ⬜ |
| 8 | Pílula de acções sobre o objecto (Vector) | `VEC_CARTOES=1`, `OBJECT_MODE=6` | **Não** | Bloco preto, dois andares de ícones; sem superfície nem raio da lista | P | ⬜ |
| 9 | Barra do Planner | `PLANNER=1` | Parcial | «Calendar» em bloco de acento; deve ser segmento (`Seg`) como as abas | P | ⬜ |
| 10 | Cartão Constraints do Inspector | `CONSTRAINT=9`, `13` | **Não** | Título grande dentro do cartão, barra azul de Influence, unidade dentro da caixa (`-150 px`), rótulo à esquerda, setas | M | ⬜ |
| 11 | Cartão Animation do Inspector | `GAME=2` | Parcial | «x Remove Animation» com letra, «Playback» e «Idle» fora da tipografia de rótulo | P | ⬜ |
| 12 | Editor de scripts: Console e Problems | `CODE=5` | Parcial | HUD por cima da faixa, Timeline a meio; título «Scripts» fora do idioma da lista | P | ⬜ |
| 13 | Timeline, modos Containers e Arrange | `NEST=2`, `CODE=1` | Sim (bloco I) | «Summary» e linhas da folha são canvas; «+ Track ▾» com seta em letra | P | ⬜ |
| 14 | Kanban (referência aprovada) | `KANBAN=1`, `4`, `8`, `9`; `BOARD=30`, `31` | Sim | Só divergência do item 1; o relatório de importação tapa o HUD e mistura títulos de 14 px no corpo | — | — |
| 15 | Balão do tutorial «fazer» (Macros) | `screens/hero/chrome/tutorial_balao.rs` | **Não olhado** | Lê `BgElev`, moldura e `paint_button` antigos | P | ⬜ |
| 16 | Luz 2D | `LIGHT2D=1`, `8` | Sim | Só gizmos no canvas e Inspector já em cartões | — | — |
| 17 | Macros, painel | `MACRO=8` | Sim | Na foto a 5 fps a lista ainda abre; sem defeito | — | — |
| 18 | Game: janela Play e HUD do jogo | `GAME=1`, foto de 2.ª janela | **Não olhado** | Janela do jogo e HUD no palco | ? | ⬜ |
| 19 | Ajuda ilustrada (tooltip) e páginas de ajuda do Editor | `CODE=5` com passo | **Não olhado** | `widget/illustration` | ? | ⬜ |
| 20 | Menus de contexto por família | `CROMO=1` | Sim (M1) | Mesma moldura; famílias não fotografadas uma a uma | — | — |
| 21 | Pintores velhos fora de `PH2D_CARDS=0` | `grep paint_panel_` | Sim, salvo 22 e 23 | Ramo antigo atrás de `cards_on()` | — | — |
| 22 | `app-sprite-anim` (editor e Region) | `editor/paint.rs`, `region/paint.rs` | **Não olhado** | Lê `Bg*` e `Border` direto | M | ⬜ |
| 23 | Audio Editor, topo; barra do Motion (`paint_chrome`, `paint_breadcrumb`, `paint_menu`) | `audio_overlay.rs` | **Não olhado** | Lê `Bg*`, `Border` direto | M | ⬜ |
