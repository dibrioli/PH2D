# HANDOFF DE CONTINUAÇÃO — `line/MiroClone`, W4 fechada → smoke do dono / W5 (2026-10-07)

> Para a PRÓXIMA JANELA desta linha (não é handoff de integração). Antecessor:
> [`HANDOFF_CONTINUACAO_W3_2026-10-06.md`](HANDOFF_CONTINUACAO_W3_2026-10-06.md).
> Plano: [`../02_plano.md`](../02_plano.md) — §3 W4 (estado), §6 (recusas medidas, quatro novas).
> Leis da caneta do Miro: [`../ferramentas/miro_caneta_notas.txt`](../ferramentas/miro_caneta_notas.txt).

## §0 — Identidade

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`, base `a46c4c200` (o `main` não andou).
- Commits da W4: `76cc80019` (a porta + o oráculo + a fixture do formato 3), `92e22c360` (caneta,
  borrachas, laser, rascunho), `87b9e70eb` (cena 5 e testes) + os do fecho. Diff da linha
  `a46c4c200..HEAD`.
- ⏳ **Smoke do dono por fazer** (§5).

## §1 — As leis e de onde vêm

| lei | fonte |
|---|---|
| `P` caneta, `E` borracha; caneta, borracha (e laço) FICAM na mão | help «Pen», «Keyboard shortcuts», «Toolbars» |
| caneta e marcador, TRÊS predefinições de cor e espessura cada, editáveis, não se apagam | help «Pen» |
| a transparência do marcador não se ajusta | help «Pen», FAQ — **o valor não é publicado** (`HIGHLIGHTER_ALPHA` 0,4, por medir) |
| a borracha apaga SÓ desenho da caneta, o traço que tocar; a de PRECISÃO só o pedaço por onde passa | help «Pen» |
| um desenho selecciona-se, move-se e apaga-se como um objecto | help «Pen» |
| a largura NÃO varia com a pressão | comunidade (3.ª fonte): ideia 7583 «Open» |
| laser `K` | Excalidraw (`HelpDialog`); o help do Miro não tem laser (só marketing do Engage) |
| rascunho (rough) | Excalidraw; o Miro não tem (ideia 11993 «Open») |

Recolha: agente, `help.miro.com/api/v2/help_center/en-us/articles.json?page=1..7` (652 artigos; o
endpoint de busca com `/en-us/` no caminho dá **404** desde a W3 — usar `…/help_center/articles/search.json`).

## §2 — As peças

| crate | o quê |
|---|---|
| `ph2d-board-rough` (nova) | `rough` (line/rectangle/ellipse/polygon/linear_path/curve/path, `Random` Park–Miller do rough.js, `FillStyle::{Hachure, Solid, CrossHatch}`), `fill` (hachure-fill, com a rotação NO SÍTIO dos polígonos que a 2.ª passada lê), `curve_points` (normalize Q→C, pointsOnPath, RDP), `freehand` (`stroke_points` + `outline`). Sem dependências; `oracle_tests` contra `saidas/portas_*.json` (29 + 11 casos, 1e-9). Avisos MIT em `LICENSE-THIRD-PARTY.md` |
| `ph2d-board-model` | **formato 4**: `Style::sketch`, `Board::sketch`, `ElementKind::Ink(Box<Ink>)` (`Ink { style, pen, points: Vec<[f32;3]>, base, pressure }`, pontos relativos à caixa: mover/redimensionar é a caixa), `Pen::{Pen, Highlighter}`, `Element::seed` (splitmix64 do id), `new_ink`/`ink()`. `legacy.rs` lê 1–3 por structs GENÉRICAS (`ElementOld<K>`…), `StyleV3`/`ConnectorV3`; fixture `fixtures/format_v3.bin` gravada pelo build de `ea38b5e61` |
| `ph2d-board-edit` | `ink.rs`: `Tool::{Pen(Pen), Eraser{precise}, Laser}`, `PenBox`/`PenPreset`/`PEN_WIDTHS`, os gestos (`ink_down/move/up/cancel`, estado próprio `Editor::ink`, uma linha de desvio em cada porta do `gesture.rs`), `erase_inverse` (o passo: tira os pedaços criados, repõe os originais), `cut` (subdivide a `r/2` antes de cortar), `laser_trail(now)`, `eraser_ring`, `set_ink_width`; `toggle_sketch` + `sketchable` em `lib.rs`; `Metrics::eraser`; `Overlay::{laser, eraser}`; a moldura e o mover/rodar/redimensionar aceitam traços (`connector().is_none()`) |
| `ph2d-board-render` | `sketch.rs`: `Rough` em cache por hash da geometria (`SketchCache`), forma (elipse pelo `ellipse`, o resto pelo `path` do contorno) e seta (linha + pontas) à mão; `ink_outline` (perfect-freehand, quadráticas pelos pontos médios); `paint_laser`, o anel da borracha; `cached_outline` passou a função sobre o CAMPO (o desenho escreve noutro campo da cache ao mesmo tempo) |
| `ph2d-editor-core` | `board_bar_pen.rs` (filho): `PenItem`, o painel (caneta, marcador, 2 borrachas, laser; 3 predefinições; 10 cores; 5 espessuras), `pen_box(theme)`, a barra de estilo dos traços (`Selected::Ink`); `Item::Sketch` na barra curta (aceso = selecção toda à mão, ou o modo do quadro); `paint_panel` mudou para `board_bar_look.rs` (tecto); `board_view::{set_tool, metrics().eraser}`, `ed.style.sketch = board.sketch` antes de cada clique; `P`/`E`/`K` |
| `ph2d-app-board` | `smoke_pen.rs` (filho): a cena 5 |
| `ph2d-text` + `ph2d-board-layout` | a `Virgil` embutida (`fonts/Virgil-Regular.ttf` + `Virgil-LICENSE.txt`, registada como `HAND_FAMILY` = «PH2D Virgil»); `shape(…, hand)`, `TextCache::get{,_rich}(…, hand)` (a chave inclui-a), `TextEdit::hand(true)`; `ph2d_board_edit::hand_lettered(el)` = a ÚNICA porta que decide (desenho e editor) |

## §3 — Prova à saída

- Oráculo: `ph2d-board-rough::oracle_tests` — 29 casos do rough.js op a op com a MESMA semente, 11 do
  perfect-freehand nos DOIS passos; controlo de sangria feito à mão (`0.2→0.2000001` no `diverge`,
  `1e-4→1.0001e-4` no `FIXED_PI`: os dois testes vermelhos, reposto).
- Testes novos: 2 do documento (formato 3 abre e fica final; rascunho + traço ida e volta), 9 da
  edição (`ink_tests.rs`), 3 do desenho (`sketch_tests.rs`), 5 pelo caminho real do ecrã
  (`board_bar_pen_tests.rs`).
- Cena 5 fotografada 3×: a 1.ª com riscas (a letra sumia — §6), a 2.ª cheia, a 3.ª em FINAL por troca
  temporária (A/B: o rascunho lê-se como tal).
- ⏳ Portão batched, prova de mutação, binário do smoke quente: preenchidos no fecho (abaixo).

## §4 — ⏳ O que fica aberto (por ordem)

1. **Smoke do dono da W4** (§5). Perguntas de produto para ele: (a) a pressão da mesa (abaixo);
   (b) a transparência do marcador e as espessuras (o Miro não as publica). ✅ A letra à mão no
   rascunho foi pedida por ele (07/10) e está feita: a `Virgil` (OFL; a Excalifont foi recusada na
   triagem — §6 do plano).
2. ⚠️ **A pressão da mesa digitalizadora não chega ao app** (`winit 0.30.13`: `force: None` nos três
   backends de desktop; `shells/desktop/src/vec_app_bridge.rs`). O traço guarda a pressão e passa a
   afinar quando `Ink::pressure` vier `true` — o que falta é a dependência (decisão do dono).
3. Capturas do Miro ainda pedidas (W3 §4.1): P/G das notas, vão do `Tab`, sombra, pega de quatro
   pontos; e da W4: um traço da caneta e um do marcador sobre branco (a transparência e a espessura).
4. Não feitos da caneta do Miro: o **laço** (seleccionar à mão — W5?) e o **smart drawing** (rabisco →
   forma); o duplo-clique para editar uma predefinição (aqui a cor e a espessura estão à vista no
   painel); o tamanho da borracha (fixo em `Spacing::Md`).
5. Herdados: W3 §4.3–4.4.

## §5 — Smoke do dono (W4)

`cd ~/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=5 ./target/smoke/ph2d-host-desktop`

1. Abre o **Board 1** com um fluxograma desenhado À MÃO (contornos tremidos), uma volta vermelha à
   roda de «Worth it?», um risco de marcador amarelo debaixo de «An idea», um visto verde, e a caneta
   já na mão (o painel dela aberto ao lado da barra da esquerda).
2. **Rascunho ↔ Final:** com nada seleccionado, clique no botão ONDULADO, o último da barra da
   esquerda — o diagrama fica limpo (final); clique outra vez — volta à mão. `Ctrl+Z` desfaz.
3. **Caneta:** arraste no quadro — desenha. No painel: a 2.ª linha são as três predefinições; as
   cores e as espessuras de baixo mudam a predefinição escolhida. O 2.º botão da 1.ª linha é o
   marcador (translúcido).
4. **Borracha:** `E` (ou o 3.º botão) e passe por cima de um traço — sai inteiro; as formas não. O
   4.º botão é a borracha de precisão: corta só por onde passa.
5. **Laser:** `K` (ou o último botão da 1.ª linha) e arraste — um rasto vermelho que some em ~1 s e
   não fica no quadro.
6. **Seleccionar um traço:** `V`, clique no traço — moldura, mover, rodar; a barra por cima muda a
   cor, a espessura e a opacidade dele.
7. **Errado se:** o botão ondulado não trocar o diagrama todo, a borracha apagar uma forma, a
   borracha de precisão apagar o traço inteiro, o laser deixar marca, ou `Ctrl+Z` não desfizer cada
   gesto.

## §6 — O que custou e não se repete

- ⛔ Uma variante nova num enum lido por ACESSORES (`shape()`, `connector()` com `_ => None`) compila em
  todo o lado e é IGNORADA em silêncio — o traço da caneta não se desenhava nem se clicava, e o
  compilador não disse nada. Procurar os leitores de cada acessor (render, hit, moldura, barra) foi o
  que o pôs à vista.
- ⛔ Comparar o documento inteiro depois de desfazer falha por desenho: o desfazer repõe o CONTEÚDO, a
  versão e as lápides só andam (colaboração). Os testes comparam o que está vivo sem a versão.
- ⛔ As riscas no rascunho (§6 do plano): a 1.ª foto mostrou a letra a sumir.
- Dois cliques seguidos no MESMO botão no relógio do teste são um duplo-clique (o `click_bar` exige
  um `Click`): pôr outro clique entre eles.
