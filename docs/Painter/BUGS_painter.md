# Bugs do módulo Painter — registro + soluções

> **O que este doc é:** o registro dos bugs do Painter cuja **causa enganava** — aqueles em que a
> aparência levou a vários rounds na pista errada. Não é o log de todo fix (isso o git já faz).
>
> **O que está VIVO aqui:** só o que ainda está **ABERTO** — os Bugs **#15**, **#11**, a **tinta
> EMPURRADA** do #14, e os dois achados abertos da varredura do #13. Tudo mais está **FECHADO**, e o
> post-mortem inteiro (sintoma → causa → tentativas que falharam → lições) foi movido **verbatim** para
> [`docs/archive/docs-2026-08-18/Painter/BUGS_painter.md`](../archive/docs-2026-08-18/Painter/BUGS_painter.md)
> em 2026-08-18. A tabela abaixo é o índice: **uma linha por bug fechado, com o MECANISMO** — leia-a
> antes de caçar o próximo, e vá ao arquivo quando a linha do índice bater com o que você está vendo.
>
> ⛔ **Nada aqui foi resumido.** As duas metades remontam o original byte-a-byte (sha256).

## Índice dos FECHADOS — o mecanismo de cada um, em uma linha

> Post-mortem completo: [arquivo](../archive/docs-2026-08-18/Painter/BUGS_painter.md), na seção `## Bug #N`.

| # | O MECANISMO (é isto que se repete, não o sintoma) | Data |
|---|---|---|
| 1 | Offset de curva: as quinas não ficavam paralelas — deslocar os **pontos de controle** não é offset; a forma certa é **offset-then-trim** (padrão CAD). | 2026-06-29 |
| 2 | Per-Layer Color: artefatos retangulares = **buffer GPU sem clear-on-alloc** (memória não-inicializada). *"Primeira vez, depois nunca"* aponta direto para leitura não-inicializada. | 2026-06-29 |
| 3 | Queda de FPS em todo arraste: o preview **recompunha a seleção inteira** por evento em vez de compor a região suja. | 2026-07-04 |
| 4 | Simplify Curve degenerava: o **fit de Schneider não fecha loops** — DP fechado + Catmull-Rom corner-aware. | 2026-07-05 |
| 5 | Offset amontoava os pontos após Convert: o offset **movia os pontos de controle**; virou DRAWING-ONLY (modelo da Seleção). | 2026-07-05 |
| 6 | Simplify "quase bom" + quinas arredondadas: refit com **corner-split** + vértice reconstruído por **interseção de bordas**. | 2026-07-05 |
| 7 | Aquarela "grave queda de FPS": era **build profile** (debug) + composite 2×/frame + loops seriais — **não** os algoritmos. Meça antes de culpar a matemática. | 2026-07-07 |
| 8 | Borda dura nas junções, **6 fixes verdes sem efeito**: o harness reproduzia o **MECANISMO**, não o **CONTEXTO**. Pare o harness em 1-2 tentativas e **instrumente o app**. | 2026-07-09 |
| 9 | "Retângulo" na união de traços úmidos: o `pour` **re-molhava o vizinho dentro do BBOX**; a cura é pour **por-footprint-dona** (o blur do véu foi a tentativa errada). | 2026-07-11 |
| 10 | Borda dura ao mudar params de Wash: params **por-dono discretos** degrauavam na junção; campo suavizado (`build_style_field`, grad 118→13). | 2026-07-11 |
| 12 | **PANIC/SIGSEGV** ao trocar de Shape no meio do traço: o guard de reuso pergunta *"existe?"* quando devia perguntar *"que FORMA tem?"*. Guard de forma, **num ponto só**. | 2026-07-12 |
| 13 | **Varredura da espécie do #12** (3 fixes): *um choke point só protege quem se registra nele* — 3 subsistemas nasceram depois de `reset_transient_edit_state` e nunca se registraram. ⚠️ O caso que corrompe **em silêncio** é o **sprite do mesmo tamanho**. | 2026-07-12 |
| 14 | Impasto "a tinta extravasa o relevo": o gate ficava verde porque media **suporte** (*onde há tinta*), e o sintoma é de **ÁREA/CONTRASTE** (*quanta tinta é neblina*). Cura: o FILME + opacidade Beer-Lambert. | 2026-07-12 |
| 16 | Aquarela "borda dura pixelada": o **AA alimentado na DENSIDADE** era comido pela saturação óptica. Split clássico de rasterizador: **forma × sombreamento** (a fração entra como ALPHA linear). | 2026-07-20 |
| 17 | Tinta atravessando a máscara saía **CRAQUELADA**: a força da proteção era um fato sobre o **MOUSE**, e na 2ª rodada um **teto que ERODIA**. | 2026-07-25 |
| 18 | A lavagem reconstruía por **EVENTO de ponteiro** e o doc dela afirmava **QUADRO** (+ pen-down alocando 268 MB). | 2026-08-02 |
| 19 | O **Smudge** forkava o canvas do DOCUMENTO em todo evento (67 MB): `Arc::make_mut` com dois donos — e o gate por **ENDEREÇO** lia *"não moveu"*. | 2026-08-02 |
| 20 | O **véu de umidade** custava 42,6 ms/quadro **no shell** e era invisível a toda sonda de bancada: densidade de **construção** ≠ densidade de **exibição**. | 2026-08-02 |
| 21 | A **secagem** custava 10-16 ms em TODO quadro, e três curas byte-idênticas mediram **1,00×**: o custo era **CAMINHAR** o canvas, não a conta. Row-parallel: 9,3× e 19,8×. | 2026-08-02 |
| 22 | **Composite Brush**: a sessão de smear nunca era encerrada — a guarda que a fechava era uma **ENUMERAÇÃO** de modos, e a pilha era o terceiro membro da família. | 2026-08-09 |
| 23 | A **FITA** divergiu e o processo comeu **90,2 GB**: um teto que limitava a **RESOLUÇÃO**, não o **TRABALHO** (a assinatura foi a suíte parar sem `ok` e sem falha). | 2026-08-14 |

---

## Bug #15 — Impasto: os chips do rig de luzes pintam e não clicam (ABERTO)

**Área:** seam da UI (painel `ph2d-panel-painter-layers` ↔ `ph2d-tool-painter`). **Não** é a matemática
do rig — essa tem 6 gates e 3 mutações vermelhas (`16_impasto_plano_implementacao.md` §18).
**Estado:** 🔎 **ABERTO** — fila de amanhã, por ordem do Enio.

### Sintoma (Enio, 2026-07-12, print)

*"UI não funciona, nem o checkbox nem se pode selecionar outra luz."*

Os chips `1 2 3 4` do card **Lighting** **pintam** (o print mostra `1` selecionado e `2· 3· 4·`
apagados — os pontinhos são a marca de "desligada", então **o snapshot chega certo no painel**) e
**não respondem ao clique**. O checkbox **Enable** também não; mas isso pode ser *consequência*: ele só
é pintado quando a lâmpada selecionada é ≠ 1, e não dá pra selecionar outra.

### Causa — NÃO IDENTIFICADA (e não vou adivinhar)

Duas hipóteses levantadas e **descartadas na leitura**:

1. **Colisão de id**: passei `PAINTER_IMPASTO_LIGHT_1` como `group_id` do segmented **e** como id da
   opção 1. → **Descartada**: `paint_segmented_adaptive` **ignora** o `group_id` (só mapeia
   `widget.options` para `paint_segmented_group_adaptive`).
2. **Falta de `store.register` em `populate.rs`** ([[feedback_panel_populate_register]]). →
   **Descartada**: os segmentos de **Depth Source** / **Draw To** também não estão em `populate.rs` e
   funcionam.

**Candidatos ainda NÃO checados:**

- **A altura do `card_frame`.** O segmented **reflui** (4 chips num painel estreito podem virar 2
  linhas), mas eu dimensionei o card por uma contagem **fixa** de linhas (`rows = 6`, ou 7 com o
  Enable). Se o conteúdo estoura o card, o **card seguinte é pintado por cima** — e os hit-rects dele
  ganham. O print reforça: o card parece **curto demais**, terminando logo abaixo dos chips.
- A **ordem dos arms** em `event.rs::handle_event`.

### A LIÇÃO — e é a terceira vez que ela cobra

Gatei a **MATEMÁTICA** do rig com 6 gates e 3 mutações vermelhas, e escrevi **ZERO gates no seam da
UI**. O `ph2d-ui-testkit` existe exatamente para isso: um teste headless que **clica no chip 2** e
afirma que `impasto_rig.selected == 1` teria saído **vermelho antes de o Enio abrir o app**.

É [[feedback_painted_is_not_populated_paint_gate]] (*pintado ≠ populado: teste a PINTURA... e o
CLIQUE*) e [[feedback_tool_unit_green_integration_dead]] (*unit-verde ≠ funciona no produto*) outra vez.
**Um widget novo não está pronto quando pinta — está pronto quando um teste clica nele.**

### Ordem de amanhã (não negociável)

1. **Escrever o gate do seam PRIMEIRO.** Headless: clica o chip 2 → `selected == 1`; clica Enable →
   `lights[1].on`. **Ele nasce VERMELHO.** Sem ele, qualquer fix é chute.
2. Só então diagnosticar (candidatos acima).
3. Consertar. É **UI pura**: não toca a matemática, e nenhum dos 6 gates do rig deve se mexer.

---

## Bug #14 (fechado) — o que dele ficou ABERTO

### ⚠️ ABERTO (adiado por ordem do Enio, 2026-07-12) — **a tinta EMPURRADA**

*"a tinta empurrada ainda não resolveu. Adiar para o final de toda essa implementação. Fim da fila."*

O **Push** (conservação de volume, §13 do plano) é real-time, conservativo, vivo e idempotente — a crista
sobe sob o pincel e a soma fecha em zero. Mas o **desenho** da tinta deslocada ainda não convence. Não
foi diagnosticado: **fica no fim da fila**, depois de todo o resto do Impasto.

---

## Bug #13 (fechado) — o que dele ficou ABERTO

> As linhas ~~riscadas~~ da tabela original (13 achados **fechados**) estão no
> [arquivo](../archive/docs-2026-08-18/Painter/BUGS_painter.md#-abertos-na-varredura-nenhum-é-crash--precisam-de-decisão-ou-fila).
> Restaram estes dois:

### ⚠️ ABERTOS na varredura (nenhum é crash) — precisam de decisão ou fila

| Achado | Gravidade | Nota |
|---|---|---|
| **Watercolor OFF→ON no meio do traço** | 🔎 **ABERTO** | Mesmo mecanismo suspeito (o `watercolor_base` é congelado no pen-down). **NÃO corrigido de propósito:** não consegui construir um RED — o dab plano nem chega a pintar no harness, então não sei o que estou corrigindo. Regra do projeto (e ordem do Enio: *não ferir a aquarela*): **sem RED refutável, não se mexe**. O fix tentado (re-congelar o ground no toggle) foi **revertido**. |
| **Gates de paridade banda-vs-serial dependem da máquina** | cobertura | Num runner de 1 core os gates "bit-identical to sequential" comparam serial contra serial — verdes e vazios. Nenhum gate força a contagem de bandas. |

---

## Bug #11 — Per-Layer Color: linhas retangulares intermitentes (ABERTO)

> **Estado: ABERTO e DORMENTE.** Nada foi corrigido. A caçada de 2026-07-11 **não achou a causa**, mas
> **eliminou quase todo o espaço de busca** e deixou uma **armadilha re-ativável** (§Armadilha). Leia a
> tabela de descartados ANTES de tentar de novo — ela economiza rounds inteiros.

**Sintoma (Enio 2026-07-11, smoke em `--release` LIMPO):** ao usar **Per-Layer Color** com **shapes
dinâmicas** (Free Hand / Ellipse / Polygon), aparecem **linhas nas bordas de retângulos**, **nas cores do
próprio brush** (não em cor de chrome). Enio: *"parecem os retângulos da umidade que foram resolvidos
(Bug #9), mas aparecem como linhas nas bordas dos retângulos."* Na screenshot: um pretzel free-hand já
desenhado + um editor de **Ellipse ativo por cima**, sendo editado, com um **círculo-fantasma deslocado**
à direita.

**O fato que domina tudo: é INTERMITENTE.** Apareceu; depois **3 runs seguidas sem reproduzir** (inclusive
COM Free Hand, o método que o Enio suspeitava ser o gatilho). Isso mata a abordagem "reproduz e bissecta"
e é a assinatura clássica de **memória não-inicializada** (Bug #2 lição #4) *ou* de uma condição de
timing/ordem (a troca de produtor CPU↔GPU).

### O que foi DESCARTADO (com o método — não repita)

| Suspeito | Veredito | Como foi descartado |
|---|---|---|
| **Composite CPU** (canvas + cache `composited`) | ❌ **DESCARTADO** | **9 testes** (`per_layer_*` em `tool/paint/tests.rs`): o cache parcial (`composite_region`+`blit_region`) é **byte-idêntico** a um recompose CHEIO em shrink, forma que se move, multi-shape, Free Hand auto-sobreposto, **multi-move-por-frame**, parked-freehand+ellipse-ativa, caminhos **cached E dinâmico** (Randomize Color) |
| **Upload parcial GPU** (`preview_upload_bbox`) | ❌ DESCARTADO | `PH2D_PAINT_FULL_UPLOAD=1` → o artefato **PERSISTIU** |
| **Tiling / Repeat Image** (`draw_repeat_image`) | ❌ DESCARTADO | Enio confirmou **Tiling OFF** (a função faz early-return) |
| **Slot GPU não-inicializado** | ❌ Já corrigido (Bug #2) | `clear_all_mips_transparent` presente em `individual.rs::create_entry_empty` |
| **Upload de camada por versão (GPU)** | ❌ DESCARTADO | `pixel_clock` **incrementa** a cada `bump_layer_pixels`; `ensure_slice` sobe a camada **inteira** quando a versão muda |
| **Resíduo no canvas** (restore/recomposite) | ❌ DESCARTADO | `dab_bbox` e a footprint do accumulate usam a **mesma** fórmula (`floor(c−r)..ceil(c+r)+1`); `restore_region` **marca dirty** |
| **Produtor GPU** (`painter_gpu_preview::try_drive`) | ⚠️ **RESTA** | Intestável no harness CPU; **o `FULL_UPLOAD` não o toca** |
| **Overlay** desenhado por cima | ⚠️ **RESTA** | Não passa pelo composite nem pelo upload. Candidatos: `draw_overlays` (symmetry / ellipse / polygon / **stencil**), `draw_selection_overlay` |
| **Tamanho do canvas** | ⚠️ **Condição provável** | Quando apareceu, os dirty bboxes chegaram a `(227,56,635,893)` ⇒ canvas **≥ ~862×949**. As 3 runs limpas foram em **512×512** |

### A pista mais forte que sobrou (leia antes de tudo)

O `PH2D_PREVIEW_DIAG` provou que **as edições de shape rodam no produtor CPU** (`gpu_owns=false`), MAS o
log tinha um bloco de **~2710 frames `gpu_owns=true`** no meio (um **arraste de slider** — o produtor GPU
assume o slot para sliders rápidos). Ou seja: **o preview ALTERNA de produtor** durante a sessão. A
troca CPU↔GPU é o único caminho que (a) o harness headless não alcança, (b) o `FULL_UPLOAD` não cobre, e
(c) depende de timing/ordem — casando com a intermitência. **Comece por aí.**

### Armadilha (re-ativável — já commitada, custo ZERO desligada)

Duas metades em [`painter_bridge.rs`](../../shells/desktop/src/render_loop/painter_bridge.rs):

```bash
# 1) Qual produtor tem o slot + o bbox do upload parcial, por frame:
PH2D_PREVIEW_DIAG=1 ./target/release/ph2d-host-desktop 2>/tmp/diag.log

# 2) O composite CPU exato que vai subir (ANTES de qualquer overlay), 1 PNG por frame:
mkdir -p /tmp/dump && PH2D_PREVIEW_DUMP=/tmp/dump ./target/release/ph2d-host-desktop
```

**Como usar quando o artefato reaparecer:** reproduza **no sprite GRANDE** com o dump ligado e **feche o
app no instante em que o retângulo aparecer**. Então:
- **Retângulo NOS PNGs** ⇒ está no composite ⇒ os 9 testes estão errando alguma condição do gesto real;
  compare o frame ruim contra o que o teste gera.
- **PNGs LIMPOS enquanto o artefato está na tela** ⇒ o composite é inocente ⇒ é **overlay** ou o
  **produtor GPU**. (Este é o desfecho que a evidência atual favorece.)

### Lições (já pagas — não repita)

1. **9 verdes no harness ≠ bug inexistente.** É a [[feedback_harness_reproduces_mechanism_not_context]] de
   novo: gastei 9 tentativas headless reproduzindo o *mecanismo* (restore/recomposite) sem o *contexto*
   (produtor GPU, canvas grande, timing). O doc já mandava parar em 1-2 e **instrumentar o app** — e foi a
   instrumentação (`gpu_owns`) que produziu a única pista real. **Pare o harness mais cedo.**
2. **Bug intermitente: a NÃO-reprodução não é prova de correção.** Enio: *"alguma coisa que vc fez deve ter
   resolvido"* — o `git diff` provou o contrário: **+21 linhas, todas dentro de `if env::var_os(...)`**, zero
   mudança de comportamento. É o falso-negativo do Bug #2 **invertido**: lá um binário stale fez um fix certo
   parecer morto; aqui a não-reprodução faz um bug vivo parecer morto. **Sempre cheque o diff antes de
   aceitar "resolveu".**
3. **Eliminar tem valor mesmo sem resolver.** Esta entrada não tem solução — tem um **espaço de busca
   reduzido a 2 suspeitos** e uma armadilha armada. Registrar isso é o que evita o próximo round começar do
   zero (é literalmente para isso que este doc existe).
4. **Compare contra o ORÁCULO certo.** Comparar gesto-vs-gesto **cancela** um bug geometria-dependente (os
   dois lados passam pela mesma via parcial). O oráculo que vale é **cache parcial vs recompose CHEIO** do
   mesmo estado — é exatamente a diferença que o `FULL_UPLOAD` **não** consegue corrigir.

---

## Como adicionar um bug aqui

Uma seção `## Bug #N — <título>` + linha na tabela do topo. Foque nos bugs cuja **causa enganou** (vários rounds
na pista errada); fix trivial fica só no git. Sempre termine em **lições generalizáveis**.

⚠️ **Quando ele FECHAR, ele não fica aqui inteiro.** O post-mortem vai para o
[arquivo](../archive/docs-2026-08-18/Painter/BUGS_painter.md) e sobra **uma linha no índice, com o
MECANISMO** — o que se repete é o mecanismo, não o sintoma. Este doc vivo só carrega o que está ABERTO.
