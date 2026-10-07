# HANDOFF DE CONTINUAÇÃO — `line/MiroClone`, W3 fechada → smoke do dono / W4 (2026-10-06)

> Para a PRÓXIMA JANELA desta linha (não é handoff de integração). Antecessor:
> [`HANDOFF_CONTINUACAO_W2_2026-10-06.md`](HANDOFF_CONTINUACAO_W2_2026-10-06.md).
> Plano: [`../02_plano.md`](../02_plano.md) — §3 W3 (estado), §6 (recusas medidas, sete novas).
> Fonte das leis do Miro: [`../ferramentas/miro_api_notas.txt`](../ferramentas/miro_api_notas.txt).

## §0 — Identidade

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`, base `a46c4c200` (o `main` não andou).
- Commits da W3: `770fda0d2` (documento + texto + editor), `7a022b524` (interface), `729538c3a`
  (cena 4) + os do fecho. Diff da linha `a46c4c200..HEAD`.
- ⏳ **Smoke do dono por fazer** (passos no §5). Não houve smoke intermédio nesta onda.

## §1 — As leis do Miro e de onde vêm (memória `feedback_the_owners_reference_product_decides_the_law`)

Pesquisa (agente, help.miro.com pela API pública de artigos + developers.miro.com, 06/10):

| lei | fonte |
|---|---|
| `N` = nota; clicar põe, arrastar desenha; a ÚLTIMA cor vale para a próxima | help «Sticky notes», «Colors» |
| 16 cores fixas, `light_yellow` de nascença — os hex | developers.miro.com `fillColor` (verificado por `curl`+`grep`, guardado em `miro_api_notas.txt`) |
| quadrada 199 / larga 350, mesma altura (228 com a sombra) | developers.miro.com |
| menu de tamanho da nota existe (P/M/G) | moderador, 09/11/2024 — **medidas não publicadas** |
| escrever com a nota seleccionada começa a escrever | help: «select it and start typing» (acrescentar × substituir: não dito → ACRESCENTA, §6) |
| `Tab`/`Ctrl+D` a escrever = a seguinte à direita, mesma cor e tamanho, a escrever | staff do Miro (2020) + 3.º — o help não o documenta |
| modo em massa: uma ideia por linha, `Done` cria em fila | help «Bulk mode» |
| pilha: arrastar dela tira notas | help «Sticky Stack» |
| colar células = uma nota por célula | help «Paste as sticky notes» |
| pega de quatro pontos arruma em grelha | só fonte de 3.º (facilitator.school, 2024) — **a confirmar no smoke** |
| B/I/U nas notas; **cor da letra NÃO** | help «Fonts» / FAQ |
| a nota CRESCE (não encolhe a letra) | **decisão 2 do dono** (o FigJam), contra o *Auto font size* do Miro |

## §2 — As peças

| crate | o quê |
|---|---|
| `ph2d-board-model` | `RichText { text, spans }` (`rich.rs`: `replace`/`restyle`/`toggle`/`all`/`color`/`typing_marks`, invariante reposta a cada mudança); `Shape.text: RichText`; `ShapeType::{Sticky, StickyStack, StickyWide}` no FIM, `is_note`, `note_aspect`; `STICKY_COLORS` (16), `STICKY_SIDE` 199, `STICKY_WIDE` 350; **`FORMAT_VERSION` 3**, `legacy::read_v2` com a fixture `fixtures/format_v2.bin` GRAVADA pelo build de `d8a331ec3` (memória nova `feedback_a_byte_fixture_is_the_old_builds_file_never_a_hand_copy`) |
| `ph2d-board-layout` | moldar com trechos (`Layout<Ink>`, pincel = cor do trecho, contexto de moldar PRÓPRIO — o do `ph2d-text` é `LayoutContext<()>`); desenhar com o itálico sintético (`glyph_transform` skew), sublinhado e riscado (métricas do run); `TextEdit` próprio sobre a `Selection` do parley, PREGUIÇOSO (abre e muda de estilo sem o moldador, molda no 1.º uso); `text_height` |
| `ph2d-board-edit` | `notes.rs` (nota nova, `note_box`, `type_into_note`, `next_note`, `begin_bulk`/`finish_bulk`, `paste_cells`, `set_note_{color,size,wide}`, `toggle_mark`/`set_text_color`/`has_mark`/`text_color`, `grid_handle`, `arrange`, `peel`, `fit_later`); `notes_layout.rs` (`grid_layout`, `reading_order`, `parse_cells` — com aspas e `\r\n`); `resize.rs` (a geometria pura que saiu do `gesture.rs` no tecto); `Gesture::{Grid, Peel}`; redimensionar uma nota = uniforme + a letra escala; `Command::Mark`; a cópia guarda `copied_text` e `is_foreign_paste`; `unfitted` + `resync` — a altura ajusta-se no próximo `overlay` (quem mudou não tinha o moldador: um clique na barra, trocar de aba a meio do modo em massa) |
| `ph2d-board-render` | a sombra da nota (`draw_blurred_rounded_rect`), as folhas da pilha, a pega de quatro pontos (`Overlay::grid` + `grid_handle_offset`) |
| `ph2d-editor-core` | `board_bar_notes.rs` (filho): o painel da ferramenta Nota (16 cores 4×4, P/M/G, quadrada/larga, pilha, em massa), os grupos `Selected::{Notes, TextNote, TextShape}`; grupo de > 8 em DUAS linhas; a barra fica a escrever; `board_keys`: `Tab` (sempre do quadro), `Ctrl+B/I/U`, `Ctrl+Shift+X`, escrever numa nota seleccionada, `Ctrl+V` de fora, `Ctrl+C` dá o texto ao sistema; `board_view::fit_later`; o `overlay` corre ANTES do `paint` |
| `ph2d-app-board` | `PH2D_BOARD_SMOKE=4` (fotografada, §5) |
| shell | `KeyCode::Tab → BoardKey::Tab` (+1 linha) |

## §3 — Prova à saída

⏳ (preenchido no fecho)

## §4 — ⏳ O que fica aberto (por ordem)

1. **Smoke do dono** (§5) — e, nele, PEDIR as capturas do Miro que fecham o que está por medir:
   (a) três notas P/M/G lado a lado, (b) duas notas feitas com `Tab`, (c) uma nota sobre fundo branco
   (a sombra), (d) a pega de quatro pontos de uma selecção. Medir pela régua de `ferramentas/`, nunca
   a olho (§6 do plano).
2. **W4 — caneta e «Rascunho ↔ Final»** (plano §3).
3. Da W3, não pedidos: o rascunho do modo em massa não mostra um texto-guia («uma ideia por linha»);
   a pilha não mostra quantas notas «tem» (no Miro também não); a altura das notas coladas de uma
   planilha usa a nota M de nascença como célula (uma linha alta empurra só a sua linha).
4. Herdados da W2: arrastar o SEGMENTO do cotovelo; rótulo ao longo da linha; as 3 pontas de UML na
   barra; transbordo das abas; desfazer das operações de ABA; cursor sobre as pegas; trackpad; menu do
   botão direito numa forma.

## §5 — Smoke do dono (W3)

`cd ~/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=4 ./target/smoke/ph2d-host-desktop`

(passos no relatório ao dono; os mesmos aqui no fecho)

## §6 — O que custou e não se repete

- ⛔ Copiar 251 bytes à mão: duas cópias erradas (253, 257); a 1.ª «lia». A fixture é o ficheiro do
  build antigo (§2).
- ⛔ O `PlainEditor` do parley tem UM estilo: medir antes de desenhar (plano §6) poupou construir o
  texto rico em cima dele.
- ⛔ Os botões da barra chegam ao editor SEM o `TextSystem` (pré-despacho): cada operação de texto que um
  botão pede tem de funcionar sem o moldador — a cura foi o `TextEdit` preguiçoso e a altura ajustada no
  `overlay` seguinte, uma porta para o teclado e para a barra.
- ⛔ A 1.ª foto da cena 4 mostrou a nota «que cresceu» SEM crescer (o texto cabia): a cena ensinava o
  contrário. Texto mais comprido.
- ⛔ Um teste da pega da grelha apontou ao meio da 2.ª coluna com `1,5 × largura` (sem o vão) e caiu
  na 1.ª: a lei conta a BORDA DIREITA mais perto — o teste estava errado, não a pega.
- `gesture.rs` passou o tecto (788/700) — cortado (`resize.rs`), nunca isento.
