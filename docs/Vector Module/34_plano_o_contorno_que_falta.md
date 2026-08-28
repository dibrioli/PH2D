# Plano 34 — **o contorno que uma forma não consegue ganhar**

> Nasceu do report do Enio de 2026-08-27 (*"o contorno funciona com as shapes que eu desejo, mas
> não funcionam com os teus desenhos"*), que foi diagnosticado no
> [plano 33 §6-quinquies](33_plano_texture_pattern.md). A cena do smoke foi curada lá; **o buraco de
> produto que ela destapou é este**.

---

## §0 — O que foi MEDIDO antes de desenhar (na `line/Vector`, worktree, 2026-08-27)

| Facto | Endereço |
|---|---|
| Toda forma AUTORADA nasce com traço: `path.stroke = Some(self.style.stroke_spec(stroke_w))`, **incondicional** | [`ph2d-vec-edit/src/shape.rs:253`](../../crates/ph2d-vec-edit/src/shape.rs) |
| O `restyle_selected_strokes` **recusa** quem não tem traço: `let Some(old) = path.stroke else { continue }` | [`vector_bridge_style.rs:98`](../../shells/desktop/src/render_loop/vector_bridge_style.rs) |
| ⭐ E a recusa **está CERTA**: ele corre **por quadro** sobre a selecção, sempre que o estilo da tool difere | [`vector_bridge.rs:339`](../../shells/desktop/src/render_loop/vector_bridge.rs) |
| A largura de mundo já é derivada num sítio só: `tool.stroke_width_px() * px_to_world` | [`vector_bridge.rs:290`](../../shells/desktop/src/render_loop/vector_bridge.rs) |
| A porta de *"que traço uma coisa nova recebe"* já existe: `PenStyle::stroke_spec(width)` | [`pen_support.rs:71`](../../crates/ph2d-vec-edit/src/pen_support.rs) |
| A resposta *"esta forma TEM traço?"* **já é publicada** — mas escondida dentro do `TokenBindings` | [`vec_bindings.rs:234`](../../shells/desktop/src/vec_bindings.rs) |
| … e só **dois** sítios a lêem (as duas rows de token) | [`paint_sections_stroke.rs:48,93`](../../crates/ph2d-panel-vector/src/paint_sections_stroke.rs) |
| A secção *Stroke* do painel é a ficha da **TOOL** (`snap: &VectorStyleSnapshot`), espelhada na selecção | [`paint_sections_stroke.rs:13`](../../crates/ph2d-panel-vector/src/paint_sections_stroke.rs) |
| O precedente completo de checkbox (paint · populate · click · drain) | `VECTOR_TRANSFORM_RESIZE_BOX` |

### §0.1 — ⭐⭐ Por que isto NÃO se cura no `restyle_selected_strokes`

A tentação óbvia — *"se não há traço, cria um"* — é **errada, e a medição diz porquê**: aquela
função corre **a cada quadro** em que o estilo da tool difere do da selecção
([`vector_bridge.rs:339`](../../shells/desktop/src/render_loop/vector_bridge.rs)). Criar ali daria
traço a **toda** forma sem traço que estivesse selecionada, **sem ninguém pedir** — e o comentário
que está lá (*"ganhar um traço do nada seria a UI inventando geometria"*) é a cerca certa pelo
motivo certo.

⇒ **A criação tem de ser um GESTO explícito**, nunca um efeito colateral de um espelhamento.

### §0.2 — ⛔ Schema: **zero**

`VecPath::stroke` já é `Option<StrokeSpec>`. Pôr `Some`/`None` usa bytes que o formato **já tem** —
não há campo novo, não há degrau, não há tripla a mexer. ⚠️ E não é um detalhe: é o que faz este
plano ser uma wave e não uma jornada (contraste: o *padrão no traço* do
[plano 33 §7](33_plano_texture_pattern.md), medido em 287 menções e 13 crates).

### §0.3 — Contrato congelado (§6): **intocado**

Nada aqui chega ao `NodeOp`/`OpResolver`/`NodeManifest` nem ao `Tool`/`RasterEditTool`/`PanelEvent`.
Os ids novos são um bloco **append-only** no ficheiro de ids do painel, e o clique viaja pelo
`PanelEvent::Click` que já existe.

---

## §1 — Pesquisa: o estado da arte, e o que cada um deles ABANDONOU

| | Como diz *"esta forma não tem traço"* | O que isso custou |
|---|---|---|
| **SVG** | `stroke="none"` — uma **propriedade** com o valor *nenhum*. Nunca um objecto ausente. | — (é o modelo de que todos descendem) |
| **Illustrator** | O traço é um *paint* cujo valor pode ser `None`; a **largura persiste** por baixo. `/` põe o swatch activo em None. | ⭐ o ida-e-volta `None → cor → None` **não perde a largura**. ⚠️ Mas o estado fica **invisível**: uma forma com `stroke: None` e largura 12 desenha-se igual a uma com largura 1. |
| **Inkscape** | *Fill & Stroke* tem botões de TIPO de tinta, com um **✗ explícito**; clicar em "flat colour" **cria**. | ⭐ o estado é legível de relance. ⚠️ Três cliques para o caso comum. |
| **Figma** | A secção *Stroke* mostra um **`+`** quando não há nenhum, e um `−` na linha. | ⚠️ o `+` cria com os **defaults**, perdendo o que lá estava — mas o Figma não tem o conceito de *estilo da ferramenta*, então não há nada melhor a herdar. |

⭐⭐ **A síntese, e é ela que decide o nosso desenho:** *todos* tratam **"sem traço" como um VALOR**,
e o nosso `Option<StrokeSpec>` trata-o como um **buraco**. Um valor tem widget; um buraco não tem —
e é exactamente por isso que este app não tem por onde preencher o buraco.

---

## §2 — O DESENHO, com a porta ÚNICA de cada pergunta

### §2.1 — Uma CAIXA DE MARCAR, e a lei já estava escrita nesta casa

> *"um `segmented` é uma escolha entre MODOS nomeados, e um checkbox é uma PROPRIEDADE que o objeto
> tem ou não tem"* — [`paint_rows.rs:79`](../../crates/ph2d-panel-vector/src/paint_rows.rs)

*Ter traço* é uma **propriedade que o objeto tem ou não tem** ⇒ **checkbox**, primeira linha da
secção *Stroke*, com o rótulo **"Stroke"**. ⚠️ A redundância com o título da secção **é o ponto**:
a linha diz *"o assunto desta secção existe nesta forma"* — é o que a caixa de painel do Blender faz.

⛔ **Rótulo "Outline" RECUSADO:** já existe `VECTOR_EXPAND_OUTLINE_STROKE` (*Outline Stroke*), que é
outra coisa — converter o traço em forma preenchida. Duas palavras para um conceito e uma palavra
para dois é como um painel passa a mentir.

### §2.2 — As duas portas únicas

| Pergunta | Porta | Porquê UMA |
|---|---|---|
| *Esta forma tem traço?* | uma publicação só, `state::stroke_present() -> Option<bool>` | ⚠️ Ela **substitui** o `TokenBindings::stroke_exists`, que responde à MESMA pergunta noutro sítio. Duas respostas divergem no dia em que uma delas ganhar uma condição. |
| *Que traço uma coisa nova recebe?* | `PenStyle::stroke_spec(w)`, com `w = tool.stroke_width_px() * px_to_world` | É **exactamente** a porta que a ferramenta de forma usa ao criar. Um default escrito no painel seria uma segunda resposta, e a forma criada pela caixa sairia diferente da desenhada. |

### §2.3 — A semântica, e o que ela NÃO promete

- **Marcar** ⇒ `path.stroke = Some(pen.style().stroke_spec(w))`.
- **Desmarcar** ⇒ `path.stroke = None`.
- ⚠️ **Desmarcar DESTRÓI a ficha** — é o modelo do Figma, não o do Illustrator. O ida-e-volta é
  aceitável **porque a ficha da tool é o que a própria secção mostra**: as rows *Width*/*Color*
  continuam visíveis com o estilo da ferramenta, então re-marcar devolve o que se está a ver.
  ⛔ Guardar a ficha removida **no documento** seria estado invisível a envenenar o undo; guardá-la
  na shell seria estado de sessão que o save não leva. Nomeado, não construído.
- **Um passo de undo por clique**, e um clique que não muda nada não grava passo.

### §2.4 — ⛔ O que NÃO se esconde, e é contra-intuitivo

As rows *Width*, *Color*, *Cap/Join*, *Dash* e as pontas **ficam visíveis mesmo sem traço**.

⚠️ **Elas não estão mortas** — a secção é a ficha da **TOOL** (§0), então cada uma autora *o traço da
próxima forma que se desenhar*. Escondê-las tiraria ao artista o único sítio onde se afina o traço
de desenho. ⚠️ Isto é o **oposto** da lei que o Enio pediu para a secção *Pattern* (*"os parâmetros
que um modo não usa não devem aparecer"*), e a diferença é real: lá os knobs **não têm leitor
nenhum** no modo `Clamp`; aqui eles têm um leitor — a ferramenta.

⇒ o que a caixa cura não é *"o controlo está morto"*, é *"o controlo não diz que não alcança ESTA
forma"*.

---

## §3 — A UI: as QUATRO condições, cada uma independente

1. **O componente existe** — `checkbox_row`, o primitivo do design system (o mesmo do Inspector e do
   Painter). Sem widget novo.
2. **É pintado e REGISTADO** — `store.register(id, InteractiveState::Button{..})` no `populate`.
   ⛔ Sem isto ele fica pintado, com hit-rect, e **morto sob o rato**: a checagem de focabilidade
   mora no store, e este painel já pagou esse defeito **cinco** vezes.
3. **O clique chega ao barramento** — o id entra no `forwards_plain_click`.
4. **A SEQUÊNCIA leva a algum lado** — o dreno da shell escreve no documento e regista o undo.

⚠️ A linha só aparece quando há **exactamente uma** forma selecionada (`Option<bool>` = `None`
esconde) — a mesma lei da secção *Pattern* e da row *Resize Box*: *uma caixa que não sabe o que
mostrar não se mostra*.

---

## §4 — Os gates, red-first

| # | Gate | O defeito que ele mata |
|---|---|---|
| 1 | `a_shape_can_gain_a_stroke_it_was_not_born_with` | o buraco inteiro |
| 2 | `the_new_stroke_comes_from_the_tool_style_not_from_a_panel_default` | a forma criada pela caixa sair diferente da desenhada |
| 3 | `unchecking_removes_the_stroke_and_rechecking_brings_one_back` | uma porta só de ida |
| 4 | `a_repeated_click_records_no_undo_step` | cada quadro virar um passo |
| 5 | `the_checkbox_row_is_wired_at_all_four_sites` (fonte) | pintado e morto sob o rato |
| 6 | `only_one_publication_answers_whether_the_stroke_exists` (fonte) | as duas respostas voltarem a divergir |
| 7 | `the_stroke_rows_stay_visible_without_a_stroke` | esconder o que autora a próxima forma (§2.4) |

⚠️ **A fixtura tem de conter o fenómeno**: uma forma nascida de `..VecPath::default()` (sem traço) **e**
uma nascida como a ferramenta a faz (com traço). Um gate só sobre a segunda passa com o bug inteiro
de pé — foi **exactamente** assim que este defeito sobreviveu a uma wave inteira de gates verdes.

---

## §5 — O que este plano NÃO faz

- ⛔ **Não mexe no `restyle_selected_strokes`** (§0.1).
- ⛔ **Não guarda a ficha removida** (§2.3).
- ⏸️ **Não trata selecção múltipla** — a caixa exige uma forma só, como a *Resize Box*. Marcar N
  formas de uma vez é uma pergunta diferente (*o que mostrar quando elas discordam?*).
- ⏸️ **Não toca no preenchimento.** O `fill` tem o mesmo buraco (`Option<Paint>`), mas tem uma saída
  que o traço não tem: a fileira *Fill Type* já cria um ao clicar num tipo. Se um smoke pedir uma
  caixa simétrica, ela é a irmã desta.

---

## §6 — O ESTADO (2026-08-27) — **fechado numa wave**

| | |
|---|---|
| **A caixa** | `VECTOR_STROKE_PRESENT`, primeira linha da secção *Stroke*, rótulo **"Stroke"** |
| **A porta** | [`vec_stroke_present.rs`](../../shells/desktop/src/vec_stroke_present.rs) — `selected_stroke_present` + `toggle` |
| **A publicação** | `state::stroke_present()` — e o `TokenBindings::stroke_exists` **foi apagado**, com gate a impedir que volte |
| **Schema** | **zero** (§0.2) |
| **Gates** | 5 de unidade + 3 de seam (gesto REAL) + 4 de fonte |
| **Mutações** | **8/8 mortas**, com os três controlos no arnês |

### ⚠️ O que uma leitura rápida do diff entende ao CONTRÁRIO

1. **O `stroke_exists` não foi «removido», mudou de dono.** A pergunta continua a existir e continua
   a gatear as duas rows de token — ela só deixou de ter **duas** respostas.
2. **O `selected_bindings` perdeu o argumento `scene`** porque a única coisa que o lia era o
   `stroke_exists`. *Um argumento que ninguém usa é o próximo a ser preenchido com a árvore errada.*
3. **As rows do traço continuarem visíveis sem traço não é um esquecimento** — tem gate com o nome
   inteiro (`the_stroke_rows_stay_visible_without_a_stroke`) e o motivo está no §2.4.
4. **A recusa do `restyle_selected_strokes` FICA**, e há gate de fonte a exigi-la: mover a criação
   para lá vestiria toda forma selecionada sem traço, por quadro, sem ninguém pedir.

### ⚠️ A armadilha que esta wave pagou TRÊS vezes

Um gate de fonte que procura um nome **conta a própria prosa que explica esse nome**. Aconteceu no
gate da cena do smoke (o doc-comment citava `..VecPath::default()`) e outra vez aqui (o comentário
que diz que o `stroke_exists` saiu **cita** o `stroke_exists`). ⇒ *todo* gate de fonte deste repo lê
o ficheiro **sem linhas de comentário**, e o helper chama-se `code()` de propósito.
