# 116 — CICLO 10 · O CARIMBO NO DISPOSITIVO

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, nesta ordem. ⚠️ **Este ciclo
> não é um GRUPO de nós**, é o primeiro dos três `⚡` do fim da fila (doc 103 §5.1): *o que tem de
> existir para uma cadeia com CARIMBO continuar no dispositivo.*
>
> **Estado (2026-09-20):** passos **1** (o item, §1) e **2** (a auditoria, §2–§4) FECHADOS. O plano
> está na §5. ⏳ Uma coluna de relógio (§4.3) espera máquina calma.

---

## §1 — O ITEM (passo 1), e por que ele não é um grupo

O pedido é do dono e chegou **duas** vezes, com dois anos de fila entre elas:

- **2026-09-06:** *«no primeiro grafo tentei colocar 1000×1000 no grid e pesou muito. Retirando
  Shape e Duplicator fica um pouco melhor.»* — ele bisseccionou sozinho e acertou nos dois nós.
- **2026-09-14:** *«usando shape (exemplo: star) fps cai para 27»* (cena `=116`, medido em
  [doc 110 §7](110_ciclo_6_valor_e_pulso.md)).

⚠️ **O doc 103 §5.1 já partia isto em DUAS perguntas**, e a auditoria confirma a partição e
**corrige a primeira**:

1. **A CONTAGEM** — uma cadeia com carimbo continuar no dispositivo. O `motion.duplicator` muda a
   contagem (`formas × pontos`), que é **estrutural** e não um mapa por-elemento.
2. **A FORMA DESENHÁVEL** — `N` cópias de uma forma vectorial são `N` caminhos **encodados a cada
   quadro**. É a *Fase 3* que o
   [ADR-0154](../architecture/decisions/0154-motion-shapes-are-live-gpu-vector-not-baked-tiles.md)
   deixou escrita: *«bake fallback para contagens de instância extremas — **se** for medido, não por
   default»*.

⛔ **Este ciclo não tem tutorial nem cena própria** (doc 103 §1 passos 6 e 7): ele não acrescenta
vocabulário nenhum ao artista. O que ele muda é o **preço** de cenas que já existem, e é por isso
que o passo 5 (a medição) é o entregável e não uma tabela de apoio.

---

## §2 — A ROTA (passo 2, primeira metade): **onde a cadeia PARA, e porquê**

Sonda `audit_the_stamp_route`
([`motion_carimbo_probe.rs`](../../crates/ph2d-app-motion/src/motion_carimbo_probe.rs)), grade
`320 × 320` = `102 400` pontos — a população do report da estrela. ⭐ **A tabela é IMUNE À CARGA:**
uma rota é uma decisão do planeador, não um relógio.

| cadeia | rota | etapas de GPU | fronteira (CPU) | recusa da ponte |
|---|---|---:|---|---|
| `grade` | **PLACA** 🟢 | 1 | — | — |
| `grade + escala` | **PLACA** 🟢 | 2 | — | — |
| `grade + carimbo` | CPU 🔴 | 0 | `motion.duplicator` | vector VIVO 🔴 |
| `grade + clone` | CPU 🔴 | 0 | `motion.clone` | — |
| `só a forma` | CPU 🔴 | 0 | `source.shape` | vector VIVO 🔴 |

⭐⭐⭐ **O achado que MUDA a wave está na última linha: a `source.shape` SOZINHA já é uma fronteira.**
O doc 103 §5.1 escreve *«a resposta provável é uma contagem derivada no planeador»* e nomeia **um**
nó; a cadeia do report tem **três** cercas, em três camadas:

1. o **carimbo** não tem kernel (muda a contagem — estrutural);
2. a **fonte** não tem kernel (ela emite `geometry_id`, e o cozimento residente não tem rota para
   geometria viva);
3. a **ponte** recusa o grafo inteiro por ele trazer um vector vivo (ADR-0154).

⇒ **curar só a metade 1 não põe a cadeia do report na placa.** ⚠️ E isso não é uma correcção da
fila, é uma correcção da *frase*: o `motion.clone` — que não traz forma nenhuma — é 🔴 **só** pela
contagem, e é ele o caso puro da metade 1.

### §2.1 — ⛔⛔ E as duas metades são ACOPLADAS, medido no código e não inferido

Dar um verbo estrutural ao carimbo faz `GpuPlan::suffix_changes_count` passar a responder `true`
naquela cadeia — e a ponte tem, desde a wave do `source.object`, esta cerca
([`motion_bridge_gpu.rs`](../../crates/ph2d-app-motion/src/motion_bridge_gpu.rs)):

```rust
if graph_has_object_source(..) && plan.suffix_changes_count(..) {
    return fell(motion, "CPU: sob um source.object, o sufixo de GPU muda a contagem");
}
```

⇒ **a metade 1, sozinha, move o problema do planeador para a ponte**: um grafo de OBJECTOS que hoje
chega ao dispositivo passaria a recusar, porque a partição por textura assume *«a posição `i` da
fronteira é a posição `i` do sink»* — verdade só enquanto o sufixo preservar a posição. *A wave da
contagem tem de ensinar a partição a sobreviver ao produto, ou ela paga com uma regressão noutra
cena.*

---

## §3 — O DESENHO (passo 2, segunda metade): **`N` cópias de UMA geometria**

Sonda `audit_the_stamp_draw`, a mesma grade, pela porta do produto (o `pump`, três tiques):

| cadeia | linhas `quad` | linhas `vector` | geometrias **distintas** |
|---|---:|---:|---:|
| `grade` | 0 | 0 | 0 |
| `grade + carimbo` | 0 | **102 400** | **1** |
| `só a forma` | 0 | 1 | 1 |

⭐⭐⭐ **A coluna que nomeia a cura é a última:** `102 400` linhas vectoriais e **uma** geometria. O
lote já poupa a metade cara — a `ph2d_vec_render::draw_shared_instances` tessela cada handle
**DISTINTO** uma vez (*«o congelamento das 160k estrelas»*, ADR-0154) —, logo o que sobra é o
**encode**: `N` comandos `fill` no Vello, cada um a repor a mesma geometria com outro transform.

⚠️ **A primeira linha é a lei de 19/09 a funcionar, não um defeito:** uma grade de POSIÇÕES sem
forma desenha **nada** (`tem_aparencia`), e é por isso que ela custa `0` dos dois lados.

---

## §4 — A LEITURA DE CONJUNTO (o que decide este ciclo)

### §4.1 — ⭐⭐ `42` dos `134` nós não têm rota NENHUMA

Sonda `audit_the_stream_op_family`, derivada do registo (⛔ nunca de uma lista escrita à mão):

| canal | nós |
|---|---:|
| verbo estrutural ([`StreamOp`](../../crates/ph2d-nodegraph/src/stream_op_meta.rs)) | **10** |
| kernel por-elemento | 82 |
| **sem rota nenhuma** | **42** |
| total (sem fixturas) | 134 |

Os dez com verbo: `sim.lifetime` e `motion.cull` (`Compact`) · `sim.spawn`, `motion.mirror`,
`motion.kaleidoscope`, `fx.drop_shadow`, `fx.rgb_split` (`SourceRows`) · `motion.combine`
(`Concat`) · `motion.trail` (`Carry`) · `value.attribute` (`Project`).

⇒ **o carimbo entra numa família que já tem cinco verbos e nove anos de maquinaria** (ADR-0136): o
sequenciador já sabe compactar, semear por gather de template, concatenar, projectar e carregar
estado. ⛔ O que ele **não** sabe é o **produto cartesiano**, que é a única coisa que o duplicador
pede.

### §4.2 — O que o duplicador de facto faz, lido do `eval`

Da [`ph2d-node-motion-duplicator`](../../crates/ph2d-node-motion-duplicator/src/lib.rs), modo `Off`
(o valor de fábrica):

- **contagem** `= ns · np` — **calculável no hospedeiro**, como o `Concat` e o `SourceRows`, e ⛔
  **nunca** como o `Compact`, cuja contagem é um facto dos dados e exige leitura de volta;
- pares em ordem **shape-major**: `(i / np, i % np)`;
- `P` e `rot` **somam** os dois lados; `Index`/`Count` renumeram contínuo;
- toda outra coluna da FORMA **replica** no índice da forma;
- as colunas do PONTO entram sob o `transfer`, e o `point_scale` multiplica o `size`.

⭐ Traduzido para o vocabulário que o sequenciador já fala: **dois gathers com índice derivado do
`i` de saída, mais um kernel que soma duas colunas** — a mesma maquinaria do `SourceRows`, com
**duas** portas em vez de uma e com as linhas a virem de uma divisão em vez de uma coluna escrita
por um kernel.

### §4.3 — ⏳ O RELÓGIO (a escada do encode) espera máquina calma

A sonda `audit_the_stamp_encode_cost` existe e mede o que `N` cópias da mesma estrela custam a
encodar pela **porta do produto** (`draw_shared_instances`). ⛔ Ela **é** um relógio, e nesta
jornada a máquina não desceu de `load 10` — a tabela entra aqui quando puder ser tirada abaixo de
`load ~5` (`CLAUDE.md` §5.0). O que já se sabe, do report de 14/09 (doc 110 §7): a cena das estrelas
lia **`39,80 ms` de `cpu-encode`** com `4,49` de cozimento, ou seja **`~35 ms` fora do cook**.

---

## §5 — O PLANO (passos 3 a 7)

| wave | o que é | porquê nesta ordem |
|---|---|---|
| **W0** ✅ | A **auditoria** (§2–§4), com as três sondas versionadas | A frase da fila nomeava **um** nó e a cadeia tem **três** cercas (§2) |
| **W1** | **A CONTAGEM:** um verbo estrutural para o produto cartesiano (`motion.duplicator` + `motion.clone`), **mais** a partição por textura a sobreviver a ele (§2.1) | É a metade que a [auditoria 98](98_auditoria_de_performance_2026-09-01.md) mede em `50,9×`, e o `motion.clone` é o caso **puro** dela — mede-se sem forma nenhuma no caminho. ⚠️ **A §5.2 mediu a população e ela reordena a wave por dentro:** o `clone` é a BANCADA (`2` cartões no produto) e o `duplicator` é o ENTREGÁVEL (`34`) — as duas metades fecham juntas |
| **W2** | **A FORMA DESENHÁVEL:** o *bake fallback* do ADR-0154 Fase 3, **se** a escada do §4.3 o justificar | Sem ela a cadeia do report continua 🔴 pela cerca 2 e 3, mesmo com a W1 fechada |
| **W3** | A **MEDIÇÃO** do ciclo (passo 5) — a mesma bancada, depois das curas | §0.0 |
| **W4** | O **smoke do dono** (passo 7) | **Enio** |

⛔ **A ORDEM É W1 ANTES DE W2, e a razão é medida:** a W1 é a única que se pode provar **sem** tocar
no desenho (o `motion.clone` fecha a cadeia inteira na placa sem uma forma no caminho), e a W2 é uma
**troca de produto** (nitidez contra contagem) que o ADR-0154 condiciona a uma medição que a W1 não
precisa de esperar.

⚠️ **E nenhuma das duas é «tornar o duplicador mais rápido na CPU»** — o §5.1 da fila já o escreve,
e a tabela do §2 diz porquê: o que se perde não são os milissegundos do nó, é o **dispositivo
inteiro** para tudo o que vem antes dele.

### §5.1 — ⭐⭐ O que a composição JÁ exprime (a lei do `CLAUDE.md` §5.0, aplicada à W1)

Antes de desenhar um verbo novo, medido contra a API que existe:

| a peça | já existe? | onde |
|---|---|---|
| a contagem `ns · np` **no hospedeiro** | ✅ **sim** | `CountLawCtx::inputs` dá *«a contagem de cada porta, em ordem de porta»* — uma lei pode perguntar quão LARGAS são as entradas dela |
| herdar **toda** coluna de UMA porta numa linha calculada | ✅ **sim** | `StreamOp::SourceRows` — o kernel escreve `cp_rows` e o sequenciador colhe o resto |
| ler uma coluna de OUTRA porta **noutro índice** | ⛔ ~~não~~ **SIM** | ⚠️ **REFUTADO em 2026-09-20 — ver §5.3.** O `ColumnAccess::SourceRead` lê a porta template num índice que o corpo calcula, e o `motion.kaleidoscope` já o faz (`i % src_n`) |

⇒ **a W1 parte-se em duas, e a fronteira está medida:**

- **(a) o caso de UMA porta** (`motion.clone`: `saída = entrada × k`) é exprimível **com os verbos
  que já existem** — `SourceRows` na porta 0 mais uma `count_law`. ⚠️ Com **uma** reserva já lida: a
  lei dele tem um canto que é de COZIMENTO e não de kernel (o leque de relógios do `time_offset`,
  que re-cozinha a entrada em N instantes) — esse fica na CPU, declarado.
- **(b) o PRODUTO de duas portas** (`motion.duplicator`) precisa do **segundo gather**, e é essa a
  única maquinaria nova do ciclo: `P` e `rot` somam a forma em `i / np` com o ponto em `i % np`, e
  nenhum dos dois índices é `i`.

⭐ *Metade da wave era composição, e sabê-lo antes de a escrever é o que a §5.0 compra.*

### §5.2 — ⛔⛔ A POPULAÇÃO, medida — e ela REORDENA a W1

A §5.1 escolheu a ordem `(a)` → `(b)` por **provabilidade** (*«o `motion.clone` é o caso puro: mede-se
sem forma nenhuma no caminho»*), e isso continua verdade. O que faltava era a outra pergunta —
**quantas cenas do produto caem em cada caso** —, e ela tem agora duas sondas versionadas
(`audit_the_stamp_clone_population` · `audit_the_stamp_duplicator_population`), varrendo os
`MAX_DEMO_LEVEL` níveis do roteador:

| nó | cenas que o usam | cartões | no regime que a wave alcança |
|---|---|---|---|
| `motion.clone` | **3** (`=64` · `=90` · `=106`) | `6` | **`2`** (linear · sem leque · sem taper) |
| `motion.duplicator` | **11** (`=62` · `=98` · `=110` · `=114` · `=115` · `=120`..`=125`) | `36` | **`34`** com `pick = Off`, que é o produto cartesiano |
| `source.shape` | **15** | — | — |

⛔⛔ **⇒ a W1(a) alcança DOIS cartões em todo o produto.** Ela continua a ser a wave certa para
**PRIMEIRA**, e a razão muda de nome: ela é a **BANCADA** da lei de contagem — o sítio onde a
maquinaria se prova sem uma forma no caminho —, e **não** um entregável que o dono sinta. *Uma
wave escolhida pela pureza da prova e não pela população é uma wave que ninguém sente*, e as duas
frases dos reports dele nomeiam o outro nó à letra (*«Retirando Shape e Duplicator fica um pouco
melhor»*).

⇒ **as duas metades fecham na MESMA wave**: a lei de contagem prova-se no `motion.clone` (2
cartões, grátis) e **aterra** no `motion.duplicator` (34), que é onde os `50,9×` da
[auditoria 98](98_auditoria_de_performance_2026-09-01.md) esperam. Fechar `(a)` e parar seria
entregar a máquina sem o consumidor dela.

⚠️ **E o `pick` confirma a cerca §6.4 com número:** `34` de `36` estão em `Off`. Os outros `2`
não são um caso de canto a ignorar — são a metade da lei que uma contagem ingénua entrega errada,
e eles existem no produto.

⏳ **Por medir, e é o que decide a W2:** dos `36` cartões, quantos têm uma `source.shape` na porta
`0` (a cerca do vector VIVO, que é a 2.ª das três do §2) contra um objecto com textura. A sonda de
hoje conta os dois nós por cena, não o par.

### §5.3 — ⭐⭐⭐ O DESENHO DA W1 ESTÁ FECHADO, e falta UMA peça nomeada

Antes de escrever o kernel, a maquinaria foi **lida** e a §5.1 saiu com uma linha refutada e três
peças achadas. *Metade desta wave era um molde que já ship.*

**1. O molde EXISTE, e é o `motion.kaleidoscope`.** Ele já é um kernel `SourceRows` que **muda a
contagem** (`segments · n`), é *slice-major* (saída `i` é a fatia `i / n`, linha fonte `i % n`), lê
a fonte nesse índice e escreve um `P` transformado; o sequenciador faz o gather de todas as outras
colunas em `cp_rows`. ⇒ **o `motion.clone` é a mesma forma com uma translação no lugar da rotação**
— `copy = i / n`, `row = i % n`, `P[i] = P_src[row] + rank(copy)·(dx, dy)`.

⛔⛔ **E é isto que refuta a linha da §5.1**: o `ColumnAccess::SourceRead` lê a porta template num
índice que o CORPO calcula, e a length-decouple dele é o que torna a leitura presente. ⇒ **o
`motion.duplicator` também é exprimível** — `shape_row = i / np` na porta `0` e `point_row = i % np`
na `1`, dois `SourceRead` em portas diferentes —, e a única maquinaria que a §5.1 dava por
inexistente já shipa há um ciclo.

**2. O `k` não se recalcula em WGSL — ele vem por `DerivedUniform`.** A contagem de cópias passa
pelo `copies_within_budget` (o clamp de orçamento do §6.3), e o `DerivedUniform` deriva um param
**no host, com o mesmo `CountLawCtx` da lei de contagem** ⇒ a lei e o kernel leem **o mesmo
número**, calculado uma vez em Rust. *Isso mata a classe inteira de divergências que um `clamp`
reescrito em WGSL traria* — e é onde o kaleidoscope ainda re-deriva o dele (`k_seg`).

**3. A renumeração é a peça que falta, e a medição diz porquê.** A lei da CPU renumera
(`Index += cópia · n`, `Count = total`) **só quando a coluna existe** — é um braço de `match`. Num
kernel `SourceRows` o que o corpo não escreve chega por GATHER (uma cópia), logo renumerar obriga a
escrever; e escrever uma coluna ausente **cunha-a**, que muda a forma do stream. Medido nas cenas,
porta a porta de todo multiplicador:

| | portas |
|---|---|
| trazem `Index` **e** `Count` | **`40`** |
| não trazem pelo menos uma | **`38`** |

⇒ *nem «escrever sempre» (cunha em 38) nem «recuar sempre» (perde tudo)*. O verbo que exprime o
braço do `match` **já existe** — `ColumnAccess::ReadWriteExisting`, *«escreve só quando a entrada
carrega a coluna»* — e ele lê no índice `i`.

⛔ **A peça em falta é a CRUZA das duas:** ler no índice da FONTE (`i % n`) **e** escrever só se
presente. Hoje há `SourceRead` (lê na fonte, nunca escreve) e `ReadWriteExisting` (escreve
condicionalmente, lê em `i`), e nenhuma faz as duas. É **uma variante append-only** no
`ColumnAccess` (`ph2d-nodegraph`, side-metadata do ADR-0136 — ⛔ **não** o contrato congelado do
§6.2) mais o braço dela no `writes()` e no codegen.

⚠️ **E é isso, e não o kernel, que era a incógnita desta wave.** O kaleidoscope contornou-a
**recuando** (`applicable: Some(|p| p(REINDEX) < 0.5)`), o que ali é aceitável porque a renumeração
dele é um knob opcional; no `motion.clone` ela é **incondicional**, logo o mesmo contorno recusaria
o nó sempre.

**4. As recusas declaradas da 1.ª wave**, com a população ao lado (§5.2): o **leque de relógios**
(`time_offset ≠ 0`, `1` cartão) é COZIMENTO e não kernel — ele re-cozinha a entrada em N instantes;
o modo **`Radial`** (`1` cartão) carrega trig por cópia, e o kaleidoscope já tem a seno parabólica
portada, logo é uma **variante** e não um bloqueio; o **taper** (`3` cartões) é o único que *cunha*
`size`/`rot` quando o knob está ligado e a coluna falta — o mesmo problema do ponto 3, no mesmo
verbo novo.

---

## §6 — CERCAS que este ciclo herda (lidas, não lembradas)

1. ⛔ **A contagem de um `Compact` vem do dispositivo por leitura de volta; a deste NÃO.**
   `ns · np` é host-computável (§4.2) — usar o caminho do `Compact` aqui pagaria um *readback* por
   quadro sem necessidade nenhuma.
2. ⛔ **O contrato está CONGELADO** (`CLAUDE.md` §6): `NodeOp = 2` · `OpResolver = 1` ·
   `NodeManifest = 8`. Um verbo novo entra como **side-metadata no registo** (ADR-0136), como os
   cinco que já existem — nunca como campo do manifesto.
3. ⚠️ **O orçamento do produto já existe e é lido no `eval`** (`points_within_budget` contra
   `RECOMMENDED_MAX_ELEMENTS`): a rota do dispositivo tem de o honrar **com o mesmo número**, senão
   as duas rotas emitem contagens diferentes para o mesmo documento.
4. ⚠️ **O `pick` do duplicador tem três modos** (`Off` · `Cycle` · `Random`) e só o `Off` é o
   produto cartesiano — os outros dois emitem exactamente `np` linhas. *Uma lei de contagem que
   ignore o modo entrega a contagem errada em dois dos três.*
