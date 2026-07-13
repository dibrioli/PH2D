# HANDOFF de INTEGRAÇÃO — `line/Painter`: o SCULPT do relevo, W1 (2026-07-13)

> **Para o agente INTEGRADOR** (DIRETRIZ §1.5.9). A linha está **fechada e parada**. Não integrei, não
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
| `an_applied_shape_keeps_its_carving_when_the_next_shape_starts` | tirar o `commit_stroke_sculpt()` do `commit_drag_preview` |
| `the_tile_memo_is_byte_identical_to_a_whole_canvas_blur` | crescer a janela de leitura por `r−1` em vez de `r` |
| `the_sculpt_writes_the_relief_and_nothing_else` (§5) | tirar o `return` depois do `stamp_dabs_sculpt` |
| `the_sculpt_does_not_light_bare_paper` (§5) | fazer o sculpt depositar `covers` |
| `strength_zero_never_touches_the_relief` | tirar o early-out de `coverage <= 0` |
| `smooth_lowers_the_gradient_and_sharpen_raises_it` | tirar o braço `sharpen` (os dois verbos viram Smooth) |
| `the_sculpt_respects_the_selection` | tirar o bloco `restrict` |
| `the_sculpt_knobs_re_render_the_finished_stroke` | fazer o `commit_stroke_sculpt` matar a sessão em vez de parkeá-la |
| `the_sculpt_session_costs_twelve_bytes_per_pixel_and_parks_at_eight` | manter o memo parkeado no pen-up |
| `no_other_paint_mode_touches_the_relief` | tirar o guard de modo do choke point (o **Mask** re-entra nele com o canvas trocado — é a rota sorrateira) |
| `undo_and_redo_inside_a_shape_do_not_sculpt_twice` (§6.1) | trocar o `restore_sculpt` por `end_sculpt_session` no `restore_model` |
| `a_parked_session_does_not_follow_the_artist_to_the_next_sprite` (§6.2) | tirar o `end_sculpt_session()` do `reset_transient_edit_state` |
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
4. Depois do traço, gire o **Radius** e alterne Smooth/Sharpen — o traço que você acabou de fazer
   **re-renderiza ao vivo** (como os knobs do card Body).

**Nada aqui é armado por código — você clica o rail você mesmo, de propósito.** O smoke que arma estado
por baixo do pano pula exatamente o seam que ele deveria provar, e esta linha tem a cicatriz: o
`PH2D_IMPASTO_SMOKE` pré-marcava o Enable, e foi assim que o botão mestre embarcou **morto** e ninguém viu
por uma semana.

---

## 8. O que ficou ABERTO

* **W2 — a espátula** (Scrape · Fill · Flatten + Multi-plane): plano §7. **Um** kernel (ajuste de plano por
  mínimos quadrados, inclinado, 3 acumuladores, zero transcendental), quatro verbos. O `SculptMode` e o
  `PAINTER_SCULPT_MODE_IDS` são **append-only** — a ordem nunca muda, e o sweep do seam cresce junto.
* **W3** — Clay · Clay Strips · Layer · Draw Sharp · Inflate (composições dos kernels de W1/W2).
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

**Fechada. Não integrada, não pushada.** `cargo test --workspace` verde (6629), clippy limpo, fmt limpo,
perf dentro do alvo. **Falta o smoke do Enio** (§7).

Aguardando ordem explícita.
