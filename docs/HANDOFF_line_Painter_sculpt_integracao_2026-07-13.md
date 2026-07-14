# HANDOFF do SCULPT — detalhe técnico (W1 + W2 + W3, 2026-07-13)

> ⛔ **HISTÓRICO — a linha foi INTEGRADA na `main` em 2026-07-13.**
> O documento vivo é [`HANDOFF_line_Painter_continuacao_2026-07-14.md`](HANDOFF_line_Painter_continuacao_2026-07-14.md).
> **O smoke do Enio segue PENDENTE** — é o item nº 1, antes de W4.

> **Detalhe técnico do SCULPT.** O documento operacional do integrador é [`HANDOFF_line_Painter_INTEGRACAO.md`](HANDOFF_line_Painter_INTEGRACAO.md) — comece por lá.
> A linha está **fechada e parada**. Não integrei, não
> pushei, não rodei `ship.sh` — isso é ordem explícita do Enio (CLAUDE.md §0.7).
>
> Plano: [`docs/Painter/18_plano_sculpt_relevo.md`](Painter/18_plano_sculpt_relevo.md).
> Handoff de continuação anterior: [`HANDOFF_line_Painter_sculpt_2026-07-13.md`](HANDOFF_line_Painter_sculpt_2026-07-13.md).

---

## 1. O que entrou

**W1 do plano: o modo Sculpt + o motor por-traço + os dois primeiros verbos.**

O Enio pediu **Smooth**. Entregou-se **Smooth e Sharpen** — porque são *um* kernel com o sinal trocado
(o próprio plano diz, em W3: *"Sharpen = o kernel do Smooth com sinal invertido. Cai de graça."*), e
porque um segmented de **um chip só** é um cheiro de design. Custou ~10 linhas. Nada mais de W2/W3 entrou.

| | |
|---|---|
| **Base** | `4cd8ef13` (main) — a linha foi rebaseada; 8 commits anteriores + 1 novo |
| **Commits** | 1 (o W1 inteiro; ver `git log`) |
| **Gates** | `cargo test --workspace` → **6634 passed, 0 failed** · clippy `--all-targets --all-features` → **0 warnings** · `cargo fmt --all --check` → limpo |
| **Perf** | **3,04 ms/move @2048² · 2,96 ms @4096²** (alvo ≤4, kill 8) — e **plano entre os dois tamanhos**, que é a prova de que o custo é O(traço), não O(canvas) |
| **Smoke** | **PENDENTE — é o próximo passo do Enio** (§7) |

**Auditoria de 2 lentes rodou e achou 7 defeitos reais — todos corrigidos, todos gateados** (§6). Três eram
sérios e nenhum deles aparecia em teste nenhum: **undo dentro de um shape aplicava o kernel duas vezes**,
uma **sessão parkeada seguia o artista pro sprite seguinte**, e a **seleção com Feather** compunha uma vez
por evento de ponteiro.

---

## 2. As 3 decisões de desenho, e onde elas DIVERGEM do plano

O plano §3 manda *"espelhar `PaintMode::Deform` linha por linha"*. Duas das três decisões abaixo se
afastam disso — **de propósito, e o §10.1 é quem manda afastar.** Estão escritas aqui porque um
integrador que ler §3 e o código vai notar a diferença e precisa saber que ela foi escolhida.

### D1 — O sculpt NÃO tem geometria própria. (§10.1, o invariante)

Ele pendura no **mesmo choke point da cor** (`stamp_dabs_inner`, ao lado do `stamp_dabs_height`) e
**retorna antes das rotas de cor** (§5: não escreve pigmento). Consequência: Symmetry, Tiling, os shape
editors, pressão, Jitter, falloff, Shape e **Grain** chegam de graça — e continuam chegando quando alguém
mexer neles. É o que o §10.1 chama de *"um passe com geometria própria é como nasce 'Tiling não funciona
no Sculpt' daqui a seis meses"*.

### D2 — A seção do painel é **ADITIVA**, não mode-exclusive.

O Deform é exclusivo porque **tem** geometria própria. O sculpt não tem: os controles do **pincel** (Size,
Spacing, Falloff, Shape, Grain) *são* a espátula. Escondê-los deixaria o artista com os ajustes de uma
ferramenta que ele não consegue mais mirar. Então `is_sculpt` entra na lista `paints_no_color()` — a
família Smear/Blur/Clone, que mantém o corpo do pincel e esconde só a **cor** — e o card acrescenta apenas
o que o pincel não sabe dizer: **qual verbo** e **em que escala** (Radius).

O gate `seam_sculpt::sculpt_keeps_the_brush_controls_and_drops_the_colour_ones` **prende essa decisão**:
ele falha se alguém "consertar" o painel pra ficar exclusivo como o vizinho.

### D3 — NÃO existe "Sculpt Strength". O Strength é o do **pincel**.

O pincel já tem um. Dois competindo pelo mesmo número é bug de design com fantasia de feature
([[feedback_ergonomics_verdict_is_a_design_bug]]). O `Dab::coverage` já carrega o Strength, e o kernel
aplica o **mesmo fold** que o depósito e a cor aplicam — o que me levou ao achado do §5.

---

## 3. Superfície FOUNDATIONAL tocada (ADR-0107)

| Crate | O quê | Isolamento |
|---|---|---|
| `ph2d-painter-brush` | **módulo NOVO `sculpt.rs`** (irmão de `height.rs`) + `height::grain_groove()` exposto `pub(crate)` + `pub mod sculpt` no `lib.rs` | **Append-only.** `height.rs` ganhou uma fn e trocou 1 linha por uma chamada a ela; `HeightDab`/`HeightFields`/`accumulate_dab_height` **intactos**. Uma linha concorrente editando `height.rs` não colide com este arquivo. |
| `ph2d-editor-core` | **arquivo NOVO `ids/chrome/painter_sculpt.rs`** + `PAINTER_RAIL_SCULPT` + `PAINTER_RAIL_TOOL_IDS` **11 → 12** + a tabela do rail **10 → 11** + a string `"sculpt"` no dispatch | Os **dois arrays com contagem** são o risco de merge: se outra linha também apendou no rail, o valor certo **não existe em nenhum dos dois lados** — conte, não escolha ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). |
| `ph2d-editor-core/tests` | 1 entrada nova em `RECONCILES_VIA` (`arch_mode_has_reconcile.rs`) | Lista append-only. |

**Contratos congelados: NENHUM foi tocado.** `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/
`CanvasPaintTool`/`PanelEvent`, e a superfície do `ph2d-vector-doc` estão todos intactos — o sculpt é um
`PaintMode` a mais e um módulo irmão, nada mais.

**`PaintMode` foi de 9 para 10 variantes** (`PAINT_MODE_COUNT` 9 → 10) — é um enum `pub(crate)` do
`ph2d-tool-painter`, não um contrato. Mas ele dimensiona o array `brush_by_mode`, então **é uma contagem
que soma entre linhas**, igual ao rail.

---

## 4. Os gates, e o VERMELHO de cada um

15 gates + 1 perf (tool) + 6 de seam (painel). **Todos tiveram o vermelho provado por mutação** — o código foi quebrado de
propósito, um de cada vez, e o gate caiu. A tabela é o que o próximo agente precisa pra confiar neles:

| Gate | Mutação que o mata |
|---|---|
| `the_relief_is_a_function_of_the_shape_not_of_how_it_was_dragged_there` (§4.1) | tirar o `restamp_reset_sculpt()` do `stamp_drag_preview` |
| `a_faster_mouse_does_not_sculpt_deeper` (§4.2) | `render_sculpt` ler `target[i]` (o relevo vivo) em vez de `pre[i]` (a fonte congelada) |
| `an_applied_shape_keeps_its_carving_when_the_next_shape_starts` | tirar o `end_sculpt_session()` do `commit_drag_preview` |
| `the_tile_memo_is_byte_identical_to_a_whole_canvas_blur` | crescer a janela de leitura por `r−1` em vez de `r` |
| `the_sculpt_writes_the_relief_and_nothing_else` (§5) | tirar o `return` depois do `stamp_dabs_sculpt` |
| `the_sculpt_does_not_light_bare_paper` (§5) | fazer o sculpt depositar `covers` |
| `strength_zero_never_touches_the_relief` | tirar o early-out de `coverage <= 0` |
| `smooth_lowers_the_gradient_and_sharpen_raises_it` | tirar o braço `sharpen` (os dois verbos viram Smooth) |
| `the_sculpt_respects_the_selection` | tirar o bloco `restrict` |
| `the_sculpt_knobs_do_not_touch_a_finished_stroke` (§11) | **parkear a sessão no commit** — i.e. o que esta onda tinha shipado |
| `the_sculpt_knobs_re_render_an_open_shape` (§11, irmão de PRESENÇA) | fazer o `refresh_live_sculpt` retornar na hora |
| `cancelling_a_shape_un_carves_it` (§11) | tirar o `cancel_sculpt_session()` do `cancel_open_shape` |
| `the_sculpt_session_costs_twelve_bytes_per_pixel_and_nothing_once_committed` | parkear a sessão no commit |
| `no_other_paint_mode_touches_the_relief` | tirar o guard de modo do choke point (o **Mask** re-entra nele com o canvas trocado — é a rota sorrateira) |
| `undo_and_redo_inside_a_shape_do_not_sculpt_twice` (§6.1) | trocar o `restore_sculpt` por `end_sculpt_session` no `restore_model` |
| `a_session_does_not_follow_the_artist_to_the_next_sprite` (§6.2) | tirar o `end_sculpt_session()` do `reset_transient_edit_state` |
| `a_feathered_selection_does_not_attenuate_once_per_pointer_batch` (§6.3) | dobrar a máscara no total acumulado em vez de no dab |

E o **seam** (`crates/ph2d-panel-painter-layers/tests/seam_sculpt.rs`, 6 testes — incl. o do chip em px,
§6.7), com as 3 armadilhas do handoff anterior provadas uma a uma:

* **não registrar** no `WidgetStore` → o chip pinta, registra hit-rect, e está **morto** sob o mouse ✗
* **registrar como `Checkbox`** → emite `Toggled`, que o `event.rs` não encaminha → *registrado e ainda morto* ✗
* **sem arm `"sculpt"`** no `set_paint_tool_mode` → o botão do rail cai em `_ => PaintMode::Paint`, silenciosamente ✗

---

## 5. ⚠️ O ACHADO QUE VIROU DO AVESSO — leia isto antes de "consertar" o fold

Durante a auditoria eu **"descobri"** que o impasto era quadrático no Strength enquanto a cor era linear:

* `Dab::coverage` é documentado como *"brush strength × pressure × space-attenuation"* — **já tem o Strength**;
* `height.rs:369` multiplica por `spec.strength` **de novo**;
* e o comentário lá diz *"the same fold the colour kernel applies"* — o que, lendo `stamp.rs`, parecia falso.

Eu ia escrever isso aqui como bug do impasto pra sua decisão. **Antes, medi.** No produto:

```
Strength 1.0 → relevo 1.0000, pigmento 255
Strength 0.5 → relevo 0.2500, pigmento  64
  razão do relevo   (0.5/1.0) = 0.250   ← quadrático
  razão do pigmento (0.5/1.0) = 0.251   ← quadrático TAMBÉM
```

As rotas de cor (`stamp_color_cache.rs:219`, `stamp_color_dynamic.rs:120/189`) fazem
`(d.coverage * brush.flow * brush.strength)`. **O comentário do `height.rs` está certo e a minha leitura
estava errada.** A resposta quadrática ao Strength é a **convenção da casa**, e cor e relevo concordam.

Quem estava fora do padrão era o **meu** kernel — a primeira versão "consertou" o double-count e teria
feito o Strength da espátula responder diferente do de todas as outras ferramentas. Corrigido: o sculpt
usa o **mesmo fold**.

**Se um dia essa curva for revista, ela é revista em todo o app de uma vez, não numa ferramenta.**
Não é decisão de uma onda de sculpt. ([[feedback_no_industrial_claims_without_verification]] pagou o
aluguel aqui: eu quase te entreguei um bug que não existe.)

---

## 6. Os 7 bugs que a linha encontrou (e corrigiu)

Cinco vieram de uma **auditoria de 2 lentes** (correção · integração) sobre o diff fechado. Os três
primeiros são a razão pela qual essa auditoria pagou o próprio custo: **nenhum deles aparecia em teste
nenhum**, e nos três os PIXELS ficavam perfeitos o tempo todo — só o relevo apodrecia.

Cada um tem gate, e cada gate tem o vermelho provado por mutação.

### 6.1 🔴 Undo dentro de um shape aplicava o kernel DUAS VEZES

O §10.4 do plano manda: *"a sessão entra no `ModelSnapshot`"*. **Eu li e não fiz.**

O sculpt escreve o relevo da camada **ao vivo**, então todo `begin_shape_txn()` captura um `heights` **já
esculpido**. Sem a sessão no snapshot, o restore deixava o re-stamp seguinte abrir uma sessão **nova**,
congelando aquele plano esculpido como fonte — e rodando o kernel em cima. `K(K(H₀,a),a)`. Undo, redo, e a
crista foi suavizada duas vezes; repita e ela derrete. **2354 texels**, e a cor perfeita o tempo inteiro.

Fix: `SculptSnap` no `ModelSnapshot` (espelho de `deform_disp`/`deform_pre`), `amount` virou `Arc` (a
captura é um refcount, não uma cópia de 64 MB por gesto). **E o `restore_sculpt` tem que vir ANTES do
`restore_shape_overlay`** — que RE-CARIMBA; restaurar depois dele é restaurar tarde demais. (Mesma razão do
comentário que já estava lá quatro linhas acima: *"restaure as SHAPES antes da máscara"*.)

### 6.2 🔴 Uma sessão parkeada seguia o artista pro sprite seguinte

O `reset_transient_edit_state` é o teardown do rebind, e **o comentário dele já descreve este bug** — foi
escrito quando mataram o gêmeo do Deform pelo mesmo motivo. As duas coisas entre o `pre` do sprite A e o
sprite B são exatamente as duas que não valem num rebind: o id da camada ativa (`LayerStack` reinicia o
`next_id` em 1, então os ids **colidem por construção**) e um check de comprimento (que casa sempre que os
dois sprites têm o mesmo tamanho).

Esculpa no sprite A → troque pro B → **sem pintar nada**, encoste no Radius. O relevo de A era escrito no
B. **2075 texels**, e sem undo (um knob não grava entrada).

### 6.3 🔴 Seleção com Feather compunha uma vez por evento de ponteiro

A seleção era aplicada ao `amount` **acumulado**, depois de cada batch. Mas o `amount` carrega todos os
batches anteriores — então um texel tocado por *k* batches tinha a primeira contribuição escalada *k*
vezes: `((a₁·s) + a₂)·s`, não `(a₁ + a₂)·s`.

Com seleção **dura** `s ∈ {0,1}` e a multiplicação é idempotente — **que é exatamente por que o meu gate
`the_sculpt_respects_the_selection` ficou verde o tempo todo.** Com **Feather** (um slider real, shipado),
a banda da borda tem `s` parcial, e ali a força da espátula virava função da taxa de polling do mouse.
É o bug do §4.2 com outro chapéu. **1292 texels.** Fix: a máscara é dobrada **dentro do kernel**, quando o
dab cai.

### 6.4 A sessão sobrevivia ao Apply do shape editor — e o próximo shape apagava a escultura

Os shape editors mantêm o traço **ABERTO** no pen-up, então o `close_stroke` nunca rodava pra eles. É a
**consequência nº 3** do doc-comment do `commit_drag_preview`, um canal ao lado — mas pior, porque o
sculpt escreve ao vivo: o primeiro frame de preview do shape seguinte restaurava por cima da escultura já
no canvas. Fix no mesmo ponto. (Achei perguntando ao canal NOVO a pergunta que o comentário já respondia
pro velho — [[feedback_ask_the_same_question_of_the_other_side]].)

### 6.5-6.7 Os três da lente de integração

* **O card do Composite Brush pintava em Sculpt** — inerte (`composite_active()` exige `PaintMode::Paint`)
  — **e ligá-lo ressuscitava a metade da cor**: `composite_enabled` é flag **tool-global**, então um
  Composite deixado marcado no Brush seguia o artista pro Sculpt e desligava o `paints_no_color()`,
  trazendo Blend / Color / Accumulate / Randomize de volta pra um modo que não escreve um pixel de cor.
* **O header do painel dizia "Brush"** com o card do Sculpt na tela.
* **O chip do Radius mostrava "0.50"** — o track cru — enquanto `sculpt_radius_px` era publicado,
  defaultado, documentado como *"o que o artista lê no chip"*… e lido por ninguém. Agora ele fala **px**, e
  o gate prende a **propriedade** (o mapeamento do chip e o kernel são a MESMA função do track), não a
  constante — pinar o número deixaria os dois livres pra divergir com o gate aplaudindo.

### Achados menores, também corrigidos

O memo **falhava aberto** (`unwrap_or(true)` = "tile desconhecida já foi borrada") → aquela tile ficava
`0.0` no memo, e o Smooth interpola **em direção ao memo**: o relevo seria puxado pra **ZERO**, achatando a
tinta em vez de mediá-la. Catástrofe visual silenciosa atrás de um default defensivo. Agora falha fechado.
Mais dois guards de índice (o `amount` sem check de tamanho; o `entry.len() == pre.len()` que um rebind
**transposto** — 64×128 → 128×64 — satisfaz enquanto o `rect` indexa pra fora).

---

## 7. Como o Enio faz o smoke (PENDENTE — o único passo que falta)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

1. Pinte dois ou três traços grossos (o smoke já arma o impasto). Cristas, com marcas de pincel.
2. Clique **SCULP** no rail esquerdo (entre Deform e Mask).
3. Arraste por cima das cristas. **Smooth** derruba; o chip **Sharpen** roda o mesmo kernel ao contrário.
4. **Alterne Smooth/Sharpen depois do traço: o traço que já está lá NÃO deve mudar** (§11 — foi o achado do
   1º smoke). O card arma o **próximo** traço. Pra mudar uma marca já feita, Ctrl+Z.
5. A exceção, que é a regra: com um **shape aberto** (Line/Curve — antes do Apply), gire o Radius e alterne
   o verbo — a curva **re-renderiza ao vivo**, porque um shape ainda é preview, não tela. Depois do Apply,
   gire de novo: nada acontece.

**Nada aqui é armado por código — você clica o rail você mesmo, de propósito.** O smoke que arma estado
por baixo do pano pula exatamente o seam que ele deveria provar, e esta linha tem a cicatriz: o
`PH2D_IMPASTO_SMOKE` pré-marcava o Enable, e foi assim que o botão mestre embarcou **morto** e ninguém viu
por uma semana.

---

## 8. O que ficou ABERTO

* ~~**W2 — a espátula**~~ ✅ **FECHADA no mesmo dia** (Flatten · Scrape · Fill) — ver §12.
* ~~**W3**~~ ✅ **FECHADA no mesmo dia** (Chisel · Layer · Inflate — e três da lista do Blender que **não
  são verbos**) — ver §13. O `SculptMode` e o `PAINTER_SCULPT_MODE_IDS` são **append-only**: a ordem nunca
  muda (o discriminante É o índice do segmented), e o sweep do seam cresce junto.
* **W4** — a família advectiva (Grab/Pinch/Nudge/Rotate/Thumb): fazer o motor do **Deform** carregar os
  planos do relevo, não construir motor novo (§8 W4).
* **W5** — Conserve (a *bow wave*, §6) + filtros de camada inteira.
* **A bifurcação do §6 segue adiada de propósito**: pra onde vai a tinta raspada. W1 não raspa nada, então
  ela não chegou. O critério de desempate está escrito no plano.
* Herdados, dormentes: Bug #11 (Per-Layer Color, listras) e o handoff de perf de camadas-como-brush.
* **A TINTA EMPURRADA (o Push)** — segue no fim da fila, como o Enio deixou.

### Dois gates que passam por NÃO OLHAR (pré-existentes — não são regressão desta onda)

A auditoria de integração encontrou dois, e os dois são **apodrecimento anterior**, não algo que o sculpt
quebrou. Ficam registrados porque um gate que passa por não olhar é pior que nenhum gate — ele dá a
sensação de cobertura:

* **`architecture_panel_wiring_parity`** lê o conjunto registrado **só de `src/populate.rs`**, e nunca abre
  os irmãos `populate_*.rs`. O lado de hits só coleta primeiros-argumentos literais de `.register(ids::X`,
  e o card do Sculpt registra via `paint_segmented_adaptive` / `paint_slider_with_chip_*`. **Os dois lados
  saem vazios ⇒ verde independentemente de qualquer coisa.** Vale igual pro `populate_deform` (o
  precedente). A cobertura real aqui é o `tests/seam_sculpt.rs`, que **clica de verdade**.
* **`node_id_collisions`** é uma lista **mantida à mão** (*"estender o chrome significa adicionar uma linha
  aqui"*). Ela não tem os 7 ids novos — e também não tem **nenhum** `PAINTER_DEFORM_*`, `PAINTER_IMPASTO_*`,
  `PAINTER_MASK_*` nem `PAINTER_SEL_*`. Só o legado `PAINTER_SIDEBAR_*`.

Nenhum dos dois é meu de consertar dentro de uma onda de sculpt (mexer neles muda o que **outras** linhas
veem), mas quem for capear a superfície de painel deve começar por aqui.

---

## 9. Armadilhas que ESTA onda pagou (pro próximo agente)

| Armadilha | O que aconteceu |
|---|---|
| **O harness reproduziu o mecanismo, não o CONTEXTO** | O gate de idempotência dirigia o Line editor com Down+Move. Mas o Line é um **construtor de polilinha**: um ponto só, e o `line_refill` **não carimba nada** (`path.len() < 2`). O gate ficou **verde contra um motor quebrado de propósito**. Dirija a ferramenta como o artista dirige. ([[feedback_harness_reproduces_mechanism_not_context]]) |
| **…e voltar exatamente à origem do grab é engolido** | `line_move` trata um retorno dentro do slop como **TAP**, não drag. "Arraste pra longe e volte" **nunca re-carimba**. O gate virou *"a mesma forma, alcançada por dois caminhos, deixa o mesmo relevo"* — que é mais forte e não briga com a ferramenta. |
| **`invalidate_composite()` no caminho quente** | Custou **148 ms/move** (37× o kill) contra uma baseline de 0,0 ms. O aviso está escrito, em letras garrafais, no `impasto::sync_relief_flags` — e eu andei direto nele. `mark_dirty` e **nada mais**. |
| **A mutação que não sangra pode estar apontando pro gate errado** | `pre` → `target` ficou verde no gate do shape editor — **corretamente**, porque o re-stamp restaura `pre` antes, então as duas leituras são a mesma leitura. A distinção só existe no traço **cumulativo**. Isso não era um gate frouxo: era um gate **faltando** (§4.2), e a mutação foi quem apontou. |
| **Backup por basename colide** | O script de mutação guardava `sculpt.rs` do tool e `sculpt.rs` da brush crate no **mesmo nome** — o restore escreveu um por cima do outro e as 11 mutações viraram "não compila". O `cp` (e não `git checkout`) salvou, mas **o nome do backup também precisa ser único**. (E um `cp` de backup **velho** depois reverteu meia correção: confira o ESTADO depois de restaurar, sempre.) |
| **O gate certo pode não existir ainda** | Uma mutação que **não sangra** tem três causas, não duas — e a terceira rende: *o gate que devia pegá-la não existe*. O `pre`→`target` ficou verde no gate do shape editor, e estava **certo** (o re-stamp restaura `pre` antes, então as duas leituras são a mesma). Explicar *por que* ela era inofensiva ALI nomeou o caminho onde ela não é. [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] |
| **A auditoria é o gate que você não sabe escrever** | Três dos sete bugs (§6.1-6.3) não apareciam em **teste nenhum**, e nos três a COR ficava perfeita — só o relevo apodrecia. Um canal novo que escreve num plano que ninguém olha não tem oráculo natural. Duas lentes independentes sobre o diff fechado acharam o que 15 gates verdes não acharam. |

---

## 10. Estado da linha

**Fechada. Não integrada, não pushada.** `cargo test --workspace` verde, clippy limpo, fmt limpo,
perf dentro do alvo (3,01 ms/move @2048² · 3,13 @4096²). **1º smoke do Enio FEITO** — o achado está na §11,
corrigido e gateado. Aguardando ordem explícita.

---

## 11. O que o 1º smoke do Enio derrubou (2026-07-13) — e é a lição mais cara da onda

> *"Ao trocar de Sharpen para Smooth e vice-versa, imediatamente o efeito é aplicado. […] Mas aqui não pode
> ser assim."*

A onda tinha shipado os knobs do card **vivos sobre o traço já feito** — a sessão era *parkeada* no pen-up e
o Radius / Smooth↔Sharpen re-renderizavam o último traço, pegando carona no **"Adjust Last Stroke"** do card
Body. Pegar o **Sharpen** (pra afiar em *outro lugar*) convertia o Smooth que você acabou de fazer no oposto
dele.

**A causa não é um bug de código. É uma affordance HERDADA POR ANALOGIA sem re-derivar.**

* Tinta é uma **substância**. Depth/Body são propriedades *da tinta que aquele traço depositou* — "me deixa
  continuar afinando" é uma oferta coerente, e o checkbox a faz.
* Um traço de sculpt é uma **operação**. Não deixa pra trás nada que tenha propriedades: só o relevo, como
  ele está agora. Não existe "o smoothing" parado ali pra ser re-parametrizado. Operações se **desfazem**.
* E o **Mode nem parâmetro é** — é *qual ferramenta*. Um verbo que reescreve o passado quando você o
  seleciona não é ajuste, é destruição que ninguém pediu.

**O motor por-traço (§4: `pre` + `amount` + re-render) sobrevive intacto** — as razões 1 e 2 (idempotência
sob re-stamp, e não virar difusão dependente do Spacing) o sustentam sozinhas. Morreu só o que eu fiz com
ele. O plano registrou a razão 3 como **riscada**, não apagada.

**A regra que ficou:** *a sessão vive exatamente enquanto o gesto não foi comitado.* Pen-up (freehand) ou
Apply (shape) a matam. O que segue re-renderizando ao vivo é um **shape aberto** — que tem botão Apply
justamente por ainda não ser tela; um card inerte *ali* deixaria a curva na tela discordando do card que a
descreve.

**Consequências no código:**

* `commit_stroke_sculpt` **morreu** — o commit chama `end_sculpt_session()`. Uma sessão, uma morte.
* O campo `SculptState.open` **morreu**: com a sessão morrendo no commit ele virou exatamente
  `layer.is_some()`, e um booleano redundante é um que um dia vai discordar do campo que sombreia. O guard
  `Some(layer)` que já existia **é** a checagem — o fix não precisou de gate novo no `refresh_live_sculpt`,
  ele caiu de graça.
* **Memória devolvida:** a sessão parkeada custava 8 B/px indefinidamente pra alimentar um recurso que era
  um bug. Agora um canvas que você terminou de esculpir custa **zero**.
* **E um TERCEIRO buraco da mesma família, achado ao varrer as saídas:** `cancel_open_shape` (Esc/Delete)
  descascava os pixels e **deixava o entalhe**. O comentário logo ali diz *"the pixels are peeled back to
  pristine — so the relief has to go with them"* — o depósito obedece de graça (ele estagia o relevo num
  envelope), o sculpt não podia, porque escreve o plano da camada **ao vivo**. Shape sumia, smoothing ficava,
  e **sem entrada de undo**, porque o shape nunca foi comitado. Fix: `cancel_sculpt_session()`.

**A lição que vale além do sculpt:** ⚠️ **o gate que pinava esse bug era VERDE, bem escrito, e tinha o
vermelho provado por mutação.** Ele se chamava `the_sculpt_knobs_re_render_the_finished_stroke` e fazia
exatamente o que dizia. Gates provam que o código faz o que você **disse**; nenhum gate te diz que o que
você disse está errado. O smoke do usuário é o único oráculo pra isso — e é por isso que ele não é opcional.
([[feedback_inherited_affordance_must_be_rederived]])

---

## 12. W2 — A ESPÁTULA (Flatten · Scrape · Fill), fechada 2026-07-13

**Cinco verbos, uma expressão.** `h = pre + k·Δ`, onde o verbo escolhe *de onde vem o alvo* e *qual sinal
de Δ passa*. Scrape e Fill não são motores novos — são o Flatten com metade do número jogada fora
(`delta.min(0.0)` / `delta.max(0.0)`), e custam um `min` cada.

O motor de W1 (`pre` + `amount` + re-render) **não foi tocado**. O plano ganhou um segundo alvo por-texel.

### 12.1 — As 3 decisões que o plano não previa

1. **O alvo do plano é uma MÉDIA PONDERADA por-texel, não um plano por-dab.**
   `plane_sum[i] += w · plano_d(i)` com o MESMO `w` que já soma em `amount[i]`; o render divide. Guardar
   "o plano" exigiria a lista de dabs no render — e os shape editors a jogam fora e reconstroem a cada
   frame. **E a divisão torna o alvo independente de Strength e Flow** (eles escalam numerador e
   denominador): Strength decide *quão longe* você viaja até o plano, não *onde o plano está*. Um Scrape
   que afundasse o plano quando você aperta seria um bug com cara de feature.
2. **O Offset NÃO entra na acumulação.** É deslocamento rígido:
   `Σ w·(plano + off) = plane_sum + off·amount`. O render soma no fim. Consequência: **o slider fica vivo
   num shape aberto sem re-carimbar um único dab.**
3. **`blurred` e `plane_sum` são mutuamente exclusivos** — um verbo pertence a UMA família. Então a sessão
   segue em **12 B/px** com cinco verbos (não 16). O preço: trocar de família num shape aberto tem que
   **RE-CARIMBAR** (`set_sculpt_mode` → `refill_open_shape`), porque `plane_sum` é função da LISTA DE DABS
   e não se reconstrói do `pre`. Sem isso, Smooth→Scrape dividia por um `plane_sum` zerado e puxava a tinta
   pro **chão do canvas** — um flatten-até-o-zero vestido de scrape. Gate:
   `switching_family_mid_shape_rebuilds_the_target`.

### 12.2 — ⚠️ A LIÇÃO DO GATE (leia antes de mexer no fit)

**O fit horizontal — o bug que o §7 inteiro existe pra evitar — é INVISÍVEL ao longo do traço.**

A intuição diz: plano horizontal na altura média ⇒ ele cava uma cratera na encosta. Verdade *dentro de um
footprint*. Mas o alvo por-texel é a **média ponderada de todos os planos que tocaram o texel**, e a média
móvel de planos horizontais **reconstrói a encosta por acidente**: cada plano fica na altura média do seu
próprio footprint, e essa média acompanha o morro.

Nada faz isso **perpendicular ao traço** — lá só existe UMA fileira de dabs, todos os planos na mesma
altura, e a inclinação transversal é simplesmente apagada (a espátula ara um vale nivelado na encosta).

Meu 1º gate de produto media a inclinação **ao longo** do traço e ficou **VERDE sob a mutação do fit
horizontal**. O comentário dele afirmava que pegaria. O comentário estava errado, não o código
([[feedback_mutate_the_code_not_just_the_test]]). Os dois gates certos:

* `flatten_on_a_pure_ramp_is_a_no_op` — rampa em **dois eixos**, Flatten tem que ser no-op (< 1e-3).
* `the_scrape_takes_the_marks_off_the_hillside_and_leaves_the_hill` — encosta **ATRAVESSANDO** o traço
  (`gy = 0.02`, cinco vezes a média das marcas). Correto: 0.020 loads/px. Fit horizontal: 0.005. Vermelho.

### 12.3 — Superfície nova

| Onde | O quê |
|---|---|
| `ph2d-painter-brush/src/plane.rs` **(NOVO, foundational, irmão append-only)** | `PlaneFit` (9 acumuladores `f64`, Cramer 3×3, **zero transcendental**) + `Plane`. Degenerado (1 texel, ou colinear) → **média plana**, não um tilt inventado: um sistema singular resolvido mesmo assim dá um gradiente que mora no último bit do acumulador — máquinas diferentes, quadros diferentes (HR-5) |
| `ph2d-painter-brush/src/sculpt.rs` | `accumulate_dab_plane` + `PlaneOut` + **`walk_dab` extraído**: o walk do footprint (corpo varrido, silhueta, Grain, Seleção) agora é UM, e os dois acumuladores o montam — uma mudança na forma do dab não pode alcançar um e esquecer o outro. O silhouette é amostrado **uma vez** (scratch `(índice, peso)`), senão o custo do dab dobrava |
| `tool/paint/sculpt.rs` | o MODELO: 5 verbos, `SculptFamily`, os 2 knobs, o roteamento |
| `tool/paint/sculpt_session.rs` **(NOVO — split por LOC cap)** | a SESSÃO: nascimento, o walk dos dabs, snapshot, cancel, re-stamp, `sculpt_displaced_volume` |
| `tool/paint/sculpt_blur.rs` | o KERNEL (uma expressão, cinco verbos) + o memo do blur |

`SculptSnap` ganhou `plane_sum` **no mesmo commit** — §10.4 do plano manda, e a cicatriz é o `mats`, que
ficou de fora do snapshot quando o material landou e só apareceu em tinta-sobre-tinta.

### 12.4 — Volume deslocado (§6)

`sculpt_displaced_volume()` — `Σ(h − pre)` sobre a janela do gesto, negativo quando a espátula tirou
material. **Computado, exposto, descartado de propósito, e GATEADO agora**
(`the_scrape_reports_the_volume_it_removed`): o Conserve de W5 (a *bow wave*) vira um **flag**, não uma
reescrita. Um número conferido só em W5, contra um kernel escrito em W2, seria um número que ninguém pode
checar.

### 12.5 — Multi-plane Scrape: NÃO entrou, e por quê

Não é cansaço. Ele precisa de (a) um **ângulo** — e o tilt é `tan(θ)`, transcendental (HR-5); (b) a
**direção do traço**, que é `[0,0]` no 1º dab; (c) um 4º knob no card. É **W3**, junto do Clay, e cabe no
mesmo fit (dois ajustes nas duas metades do footprint, partidas pelo eixo do traço).

### 12.6 — Gates de W2 (10 mutações, 10 mortas)

| Gate | Mutação que o mata |
|---|---|
| `an_exact_plane_is_recovered_exactly` (brush) | fit horizontal (`gx=gy=0` no `solve`) |
| `the_slope_survives_the_brush_marks` (brush) | idem |
| `a_collinear_footprint_falls_back_to_the_mean` (brush) | resolver o sistema singular mesmo assim |
| `flatten_on_a_pure_ramp_is_a_no_op` | **fit horizontal** — o gate do §7 |
| `the_scrape_takes_the_marks_off_the_hillside_and_leaves_the_hill` | idem (mede ATRAVESSANDO o traço) |
| `scrape_only_lowers_fill_only_raises_and_flatten_does_both` | tirar o `delta.min(0.0)` do Scrape |
| `the_plane_offset_gives_the_spatula_its_bite` | tirar o `+ offset` do alvo |
| `the_scrape_reports_the_volume_it_removed` | zerar (ou inverter o sinal de) `sculpt_displaced_volume` |
| `switching_family_mid_shape_rebuilds_the_target` | tirar o `refill_open_shape()` da troca de família |
| `a_plane_stroke_costs_twelve_bytes_per_pixel_too` | tirar o guard de família do memo (16 B/px) |
| `the_knob_row_swaps_with_the_verb_family` (seam) | pintar o Radius em todo verbo |
| `the_offset_slider_is_wired_to_the_tool` (seam) | tirar o arm do `route_sculpt_event` |

**Perf** (o kill criterion agora roda os DOIS motores — um orçamento medido num só é o orçamento de uma
ferramenta que o artista não tem):

```
SMOOTH @2048px: 3,12 ms/move   |  @4096px: 3,16   (alvo <=4, kill 8)
SCRAPE @2048px: 2,63 ms/move   |  @4096px: 2,57
```

O Scrape é **mais barato** que o Smooth (o fit é 9 FMAs/texel + um solve 3×3 por dab; sai mais barato que
um box blur de raio 16). Plano entre 2048² e 4096² nos dois ⇒ **O(traço), não O(canvas)**.

### 12.7 — Um warning de clippy que NÃO é meu

`tests/spike/src/bin/c11_flecs.rs:64` — *"casting to the same type is unnecessary"*. Está na **main**
(commit `cf62198e`), minha linha não toca `tests/`. Se o ship reclamar, é dele, não da espátula.

### 12.8 — Smoke de W2 (some ao smoke de W1)

Pinte uma crista → **SCULP** → escolha **Flatten** e passe **atravessando o FLANCO** dela: o flanco
continua um flanco. Uma espátula nivelada araria um vale ali — é o teste de um segundo pro §7.
Depois **Scrape** (só tira) e **Fill** (só põe), e gire o **Offset**: negativo = a faca crava, positivo =
o Fill amontoa.

---

## 13. W3 — Chisel · Layer · Inflate, fechada 2026-07-13

**Oito verbos, uma expressão.** `h = pre + k·Δ`. Chisel é Scrape com um `abs`. Layer é o kernel com alvo
**constante** — e é isso que o limita (`k ≤ 1` ⇒ nunca passa de `pre + Depth`). Inflate é o kernel com alvo
`pre + Depth·n_z`.

### 13.1 — ⚠️ A LIÇÃO (a mais importante de toda a linha, e o gate cobrou)

**Os dois eixos de um campo de altura NÃO são a mesma unidade.** `x` é texel; `h` é *carga de tinta*. Um
**ângulo** é razão de COMPRIMENTOS. Uma **normal** também. Então toda grandeza *geométrica* só significa
alguma coisa depois que a altura vira comprimento — e o conversor é o que a **LUZ** usa:
`DEPTH_UNIT_PX = 16` (uma carga de tinta tem 16 px de altura).

Eu acertei no **Inflate** porque fui procurar qual normal a luz usa (`impasto_shade::shade`). **Errei no
Chisel** porque não fui: o 1º corte usou `tan(36°)` cru, inclinando o plano em **0,73 load por texel** —
8,7 loads ao longo do footprint, **4× o `H_CEIL`**. O "ângulo" era um número num espaço sem geometria
dentro, e o V que ele cortava era um penhasco.

O gate `the_chisel_carves_a_crease` **pegou**: reportou 0,36 load "poupado" *no próprio eixo* — meio texel
de lado. O número denunciou a escala.

> **Regra pra W4/W5 e pra qualquer coisa que toque `h`:** *toda grandeza geométrica — normal, ângulo,
> inclinação que o artista VÊ — cruza `DEPTH_UNIT_PX` na entrada.*

Ambas as mutações estão gateadas (M4: `tan` cru no Chisel · M7: normal sem ganho no Inflate) — **uma lição
só gateada onde você já sabia olhar não está gateada.**

### 13.2 — Três da lista do Blender NÃO entraram, e cada ausência é um ACHADO

| Blender | Por que não é um verbo aqui |
|---|---|
| **Clay** | É **`Flatten` com Offset > 0**. O plano fica ACIMA da superfície: vales sobem até ele, cristas descem até ele — material adicionado, superfície achatada. *Isso É clay.* Os dois knobs já estão na tela; um chip seria **preset de outro chip**, e um card não sabe dizer qual de duas ferramentas idênticas você segura. |
| **Clay Strips** | É Clay + **dab quadrado**. A forma do dab é do **PINCEL** (10 falloffs, slot de Shape, flatten, ângulo). Um falloff quadrado é buraco do pincel, não verbo do sculpt. |
| **Draw Sharp** | **Colapsa no Layer.** O Blender precisa dele separado porque o Draw dele lê a malha *deformada* e arredonda a própria crista. **Nosso motor lê o `pre` CONGELADO e não consegue fazer diferente** (§4) — todo verbo aditivo já é "sharp" por construção. Não sobra nada pro segundo ser. |

### 13.3 — Famílias, alvos e custo

| família | verbos | alvo | reconstruível? | knobs | custo |
|---|---|---|---|---|---|
| Smooth | Smooth · Sharpen | `blur(pre)` | **sim** (memo, de `pre`) | Radius | 12 B/px |
| Plane | Flatten · Scrape · Fill · **Chisel** | `Σw·plano(i)` | **não** (função da lista de dabs → re-carimba) | Offset (+ **Angle** no Chisel) | 12 B/px |
| **Height** | **Layer** · **Inflate** | função de `pre` + knob, por texel | — (**sem buffer**) | Depth | **8 B/px** |

A família Height é a mais barata *e* a mais simples: o alvo do Layer é a constante `pre + Depth`; o do
Inflate é 4 leituras e uma raiz. Nenhum dos dois vale um plano do tamanho do canvas.

**O Chisel é o único verbo com DOIS knobs** (Offset *e* Angle — o plano precisa ser colocado antes do V ser
dobrado em torno dele). O gate do painel exige o **conjunto exato** de knobs por verbo, não "um sim, um
não": um verbo de W4 cuja família está certa mas cuja linha ninguém fiou passaria num check mais frouxo.

### 13.4 — O Chisel, em uma linha

O construtor de dois planos é o caminho longo. As duas faces passam pela **mesma linha** — o eixo do traço —
e sobem dela no mesmo ângulo, então a união delas é só:

```
plano(x,y)  +  tilt · |distância lateral ao eixo|
```

um plano e um valor absoluto. **`tilt = 0` é exatamente a faca chata** ⇒ Flatten/Scrape/Fill continuam
byte-idênticos ao kernel de W2, e o Angle no zero não é zona morta: é a lâmina plana (gateado:
`the_chisel_at_zero_degrees_is_byte_identical_to_scrape`).

**Um dab sem direção não crava** (`dir = [0,0]` no 1º dab e num Drag Dot) — sem eixo não há V, e ele deita
um scrape chato. Isso é honesto, não defensivo: inventar uma direção faria a marca depender de pra que lado
o ruído de float caiu.

### 13.5 — Gates de W3 (11 mutações, 11 mortas)

| Gate | Mutação que o mata |
|---|---|
| `the_chisel_at_zero_degrees_is_byte_identical_to_scrape` | pôr piso no tilt (`tan(a).max(0.05)`) |
| `the_chisel_carves_a_crease` | tirar o `+ v` do `plane_sum` · dobrar em torno de `dir` em vez de `perp` · **`tan` cru sem `DEPTH_UNIT_PX`** |
| `layer_lays_one_coat_however_long_you_dwell` | alvo `p + depth*a` (acumula em vez de limitar) |
| `inflate_rounds_the_crest_instead_of_translating_it` | `n_z = 1` · **normal sem o ganho `DEPTH_UNIT_PX`** |
| `a_layer_stroke_costs_eight_bytes_per_pixel` | alocar `plane_sum` pra família Height |
| `switching_into_the_height_family_frees_the_other_targets` | manter o alvo de plano ao entrar no Layer |
| `every_verb_paints_exactly_the_knobs_it_uses` (seam) | tirar a linha do Angle (o Chisel fica com 1 knob) |
| `the_depth_and_angle_sliders_are_wired_to_the_tool` (seam) | tirar qualquer um dos arms do `route_sculpt_event` |

**Perf** — os TRÊS motores (o Inflate é aritmética diferente: uma raiz e 4 leituras por texel, e nenhum
buffer):

```
SMOOTH  @2048px 3,12 ms/move  |  @4096px 3,09
SCRAPE  @2048px 2,62 ms/move  |  @4096px 2,93
INFLATE @2048px 2,32 ms/move  |  @4096px 2,31     (alvo <=4, kill 8)
```

Plano entre 2048² e 4096² nos três ⇒ **O(traço), não O(canvas)**. O Inflate é o mais barato — ele não
carrega buffer nenhum.

### 13.6 — Smoke de W3

**Chisel:** pinte uma crista, arraste ao longo dela. Ele **poupa os flancos e corta o eixo** ⇒ um sulco com
vinco no fundo. Ponha o Angle em 0: vira Scrape (é o mesmo kernel).
**Layer:** passe **dez vezes no mesmo lugar, dentro de um traço só**. A demão continua com uma espessura de
Depth. Nenhum outro verbo faz isso.
**Inflate:** ponha o pincel na borda de uma mancha grossa. O **topo sobe, a parede não** — a crista
*arredonda* em vez de subir. Depth negativo murcha.
**Clay:** não tem chip, de propósito. É **Flatten com Offset positivo** — experimente.
