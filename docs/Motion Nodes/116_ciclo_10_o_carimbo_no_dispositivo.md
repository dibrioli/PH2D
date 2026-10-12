# 116 — CICLO 10 · O CARIMBO NO DISPOSITIVO

> História verbatim: [arquivo](../archive/motion-nodes-2026-10-12/116_ciclo_10_o_carimbo_no_dispositivo.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| `Scene::append` de um fragmento como cura do encode | Recusa medida: a sonda `audit_the_stamp_encode_routes` mostra que não é a cura; a escada do encode cai a `11 %` de um quadro sem assar | [arquivo](../archive/motion-nodes-2026-10-12/116_ciclo_10_o_carimbo_no_dispositivo.md) §4.3 |
| Bake fallback (W2, ADR-0154 Fase 3) | RETIRADA: o dono recusou (2026-09-20, «manter a nitidez») e o encode a 102 400 caiu de `34 %` para `11 %` com bytes idênticos | vivo §5 W2 / [arquivo](../archive/motion-nodes-2026-10-12/116_ciclo_10_o_carimbo_no_dispositivo.md) §4.3 |
| `--test-threads=1` no cabeçalho como cura do relógio | A nota do `CLAUDE.md` §2 manda o contrário; a 1.ª redacção media OUTRO programa | [arquivo](../archive/motion-nodes-2026-10-12/116_ciclo_10_o_carimbo_no_dispositivo.md) §4.3 |
| Kernel para o `motion.duplicator` (W1(b)) como entregável | Muda ZERO cartões: `36/36` trazem vector vivo e a ponte recusa o grafo inteiro (ADR-0154) | vivo §5.6 |
| W2 como ganho nas cenas do produto | A pior cena desenha `190` linhas e custa `0,1 %` de um quadro | vivo §5.7 |
| Passo/`k` do kernel como trig no dispositivo | HR-5: a seno parabólica da casa portada não seria a mesma; derivam-se no hospedeiro, sem custo à placa | [arquivo](../archive/motion-nodes-2026-10-12/116_ciclo_10_o_carimbo_no_dispositivo.md) §5.5 |
| Derivar a população do ciclo a partir das portas do produto | Errou por `~3×`; a população `320×320 = 102 400` do report não serve para a cena | [arquivo](../archive/motion-nodes-2026-10-12/116_ciclo_10_o_carimbo_no_dispositivo.md) §5.8 |

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, nesta ordem. ⚠️ **Este ciclo
> não é um GRUPO de nós**, é o primeiro dos três `⚡` do fim da fila (doc 103 §5.1): *o que tem de
> existir para uma cadeia com CARIMBO continuar no dispositivo.*
>
> **Estado (2026-09-20):** passos **1** (o item, §1) e **2** (a auditoria, §2–§4) FECHADOS. O plano
> está na §5.

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

## §5 — O PLANO (passos 3 a 7)

| wave | o que é | porquê nesta ordem |
|---|---|---|
| **W0** ✅ | A **auditoria** (§2–§4), com as três sondas versionadas | A frase da fila nomeava **um** nó e a cadeia tem **três** cercas (§2) |
| **W1(a)** ✅ | **A CONTAGEM** no `motion.clone` — o verbo estrutural, os uniformes derivados e a paridade (§5.5) | É a metade que a [auditoria 98](98_auditoria_de_performance_2026-09-01.md) mede em `50,9×`, e o `clone` é o caso **puro** dela — mede-se sem forma nenhuma no caminho |
| **W1(b)** ✅ (ciclo 12) | O mesmo verbo no `motion.duplicator`, **mais** a partição por textura a sobreviver a ele (§2.1) | ⭐ **FECHADA no ciclo 12 ([doc 120 §8](120_ciclo_12_os_tectos_confortaveis.md)), e quem a reabriu foi o RUNTIME e não o catálogo:** num jogo o que se carimba são IMAGENS do átlas, e para essas a partição é vazia por construção. ⛔⛔ **A §5.2 disse *«é o ENTREGÁVEL (34), e as duas metades fecham juntas»* e a §5.6 REFUTOU-O com o número que ela encomendou:** `36` de `36` cartões trazem um vector VIVO, logo a ponte recusa-os uma camada acima e o kernel move **zero**. *Ela volta a ser obrigatória no dia em que a W2 aterrar* |
| **W2** ⛔ **RETIRADA** | **A FORMA DESENHÁVEL:** o *bake fallback* do ADR-0154 Fase 3. ⛔ O dono **RECUSOU-O** em 2026-09-20 (*«manter a nitidez»*), e a §4.3 tirou-lhe o motivo: o encode a `102 400` passou de `34 %` de um quadro para **`11 %`** sem assar nada e com os bytes idênticos | O que a justificava era o relógio, e ele mudou. ⇒ o que fica é a **cerca** que ela ia atravessar, e essa é uma pergunta de PONTE (de *tipo* para *documento*), não de bake |
| **W3** ✅ (ciclo 12) | A **MEDIÇÃO** do ciclo (passo 5) — a mesma bancada, depois das curas | §0.0 — feita no [doc 120 §8.5–§8.6](120_ciclo_12_os_tectos_confortaveis.md): com o carimbo na placa o Motion das IMAGENS deixou de crescer com o número (proxy de telemóvel a `32 768`: `7,30 → 0,72 ms`), depois de curadas as duas contas escondidas que o anulavam e a leitura dos cartões que esperava pela placa |
| **W4** ✅ (2026-09-24) | O **smoke do dono** (passo 7) | **Enio** — aprovado nas duas rondas (as imagens e as marcas das posições; os cartões a `0,56 ms` de Motion na máquina dele) |

⭐ **O CICLO FECHA assim:** a cadeia das IMAGENS vai inteira à placa (fonte na CPU, o resto no
dispositivo); a da ESTRELA fica na CPU **por decisão do dono** (*«manter a nitidez»*, a W2 retirada),
e o que a destravaria é a pergunta da ponte sobre o vector vivo — decisão de desenho, fora deste
ciclo.

⛔ **A ORDEM ERA W1 ANTES DE W2, e a razão era medida:** a W1 é a única que se pode provar **sem**
tocar no desenho (o `motion.clone` fecha a cadeia inteira na placa sem uma forma no caminho), e a W2
é uma **troca de produto** (nitidez contra contagem) que o ADR-0154 condiciona a uma medição que a
W1 não precisava de esperar.

⚠️⚠️ **Isso continua VERDADE para a W1(a) e ficou FALSO para a W1(b)** (a §5.6: `36/36` recusados
pela ponte antes de planear). ⛔⛔ **E a frase que aqui estava — *«a §4.3 justifica a W2»* — MORREU
no dia seguinte ao em que foi escrita**, por duas razões independentes: o dono recusou a troca
(*«manter a nitidez»*) e a mesma §4.3, re-medida, tirou-lhe o motivo (`44 % → ~20 %` do quadro **sem
assar nada**). *Quem move o número que tornava uma wave necessária tem de reconferir a nota que a
declarava necessária* (`CLAUDE.md` §0.0).

⇒ a ordem que fica é **W1(a) ✅ → W3 → W4**, e a W1(b) deixa de esperar pela W2: o que a destrava é
a **pergunta da PONTE** (a marca de vector vivo passar de *tipo* para *documento*), que é uma
decisão de desenho e não uma wave de bake.

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

### §5.4 — ✅ A PEÇA EM FALTA EXISTE: `ColumnAccess::SourceReadWriteExisting`

Construída no mesmo dia em que a §5.3 a nomeou. Ela é a **cruza** do `SourceRead` (lê a porta
template num índice que o corpo calcula) com o `ReadWriteExisting` (escreve só quando a entrada
carrega a coluna) — e nenhuma das nove variantes que existiam tinha as duas metades.

⚠️ **Append-only** no `ph2d-nodegraph` (side-metadata do ADR-0136; ⛔ **não** o contrato congelado
do `CLAUDE.md` §6.2), e o alcance é pequeno de propósito: a variante, três predicados
(`reads` · `writes` · `is_source_read`) e **um** braço no `plan_bindings`. *Nenhum kernel que não a
declare muda um byte.*

⭐⭐ **E o alcance ser pequeno é uma propriedade do desenho, não sorte:** todo consumidor do enum
passa por predicados (`reads()`, `writes(present)`, `consumes()`, `broadcasts()`, `refuses()`,
`is_gather_key()`, `is_source_read()`) em vez de um `match` sobre o verbo — medido, há **cinco**
sítios em toda a `ph2d-gpu-cook` + `ph2d-nodegraph` que nomeiam uma variante, e um deles é o braço
que esta wave escreveu.

⛔⛔ **A variante compilou sem quebrar um único `match`, e isso é o sinal de alarme e não o de
sucesso** — é a forma exacta do *dreno de um braço só* que o `CLAUDE.md` §5.0 nomeia: um verbo novo
que cai num `_ =>` responde plausivelmente e em silêncio. ⇒ o gate que fica é um **`match`
EXAUSTIVO** (`indice`) de que a lista de variantes é **derivada**: quem acrescentar um verbo é
obrigado a dar-lhe um índice (erro de compilação) e a lista não pode ficar para trás sem reprovar.
*As listas escritas à mão que já existiam nos gates deste enum varriam seis nomes de nove.*

⚠️ **E as duas populações condicionais passam a ser afirmadas** (quem escreve só-se-presente ·
quem lê num índice que o corpo calcula), porque um verbo que caia numa delas por acidente muda o
produto sem tocar numa linha de gate.

**Gates:** `toda_variante_esta_na_lista_e_na_ordem_do_indice` ·
`a_cruza_le_na_fonte_e_escreve_so_o_que_existe` (com o CONTROLO dos dois irmãos, cada um com uma
metade) · `as_duas_populacoes_condicionais_sao_as_que_a_casa_declara` — os três na
`ph2d-nodegraph`; e na `ph2d-gpu-cook`, `a_cruza_escreve_so_quando_a_coluna_existe` (o PLANO de
bindings: `ReadBuffer`+`WriteBuffer` presente, `ReadIdentity`+`WriteDropped` ausente) e
`o_acessor_de_escrita_existe_mesmo_quando_a_coluna_falta`.

⛔ **Os dois níveis são obrigatórios e o de cima não prova o de baixo:** o do enum afirma o que os
predicados respondem, o do codegen afirma que ele os HONRA — e as duas coisas já divergiram nesta
casa (o `ReadBroadcast` nasceu com os predicados certos e um `match` que o tratava como escritor;
quem o apanhou foi o naga a recusar `redefinition of out_v`).

**Mutação: 6 de 6 sangram**, com os dois controlos verdes — quatro sobre a LEI (ela cunha · ela
nunca escreve · ela deixa de ler na fonte · ela deixa de ler), uma sobre o CODEGEN (o `write_`
no-op esquecido) e uma sobre a DERIVAÇÃO. ⚠️ **E a da derivação teve de ser reescrita:** *tirar* a
variante da lista não compila (o array é `[ColumnAccess; 10]`), e uma mutação que não compila
lê-se no relatório exactamente como uma que sobreviveu — a forma que este repo já pagou. A que
mede é **trocar a ORDEM** de duas entradas, que compila e reprova; a remoção fica nomeada como o
caso **mais forte** (erro de compilação, não teste vermelho).

⚠️ **Promoção pedida à lista de flakes de fan-out** (`CLAUDE.md` §5.0 — a linha pede, o integrador
escreve): **`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`**
(`ph2d-tool-painter`, módulo `measure_input_cost`) — único ✗ de `18 433` no portão da W1 do verbo,
com **zero** linhas do diff naquela crate, e **3 de 3 verde sozinho a `load 38`–`41`**.

⭐⭐ **E ele REPETIU na W1(a), que é o que fecha a assinatura:** único ✗ de **`18 443`**, outra vez
com **zero** linhas de diff em `ph2d-tool-painter`, e **3 de 3 verde sozinho a `load 13,13`**. ⇒
*duas corridas independentes, dois diffs sem uma linha naquela crate, e o teste verde sozinho nas
duas — gate de custo medido, a assinatura exacta da família do `CLAUDE.md` §5.0.*

### §5.6 — ⛔⛔⛔ E A W1(b) NÃO É O ENTREGÁVEL: `36` de `36` cartões trazem um VECTOR VIVO

O ⏳ que a §5.2 deixou por medir — *«dos 36 cartões, quantos têm uma `source.shape` na porta 0
contra um objecto com textura»* — foi medido (`audit_the_stamp_shape_side`, a sonda que conta o
**PAR** e não os dois nós por cena), subindo a montante de cada porta:

| o lado da FORMA (porta 0) de cada cartão | cartões |
|---|---:|
| com `source.shape` a montante (**vector VIVO**) | **`36`** |
| com `source.object` a montante (textura) | `0` |
| com os dois | `0` |
| com nenhum dos dois | `0` |
| *(controlo)* o lado dos PONTOS com `source.shape` | `0` |
| com `transfer ≠ Shape Wins` | `6` |

⛔⛔⛔ **⇒ dar um kernel ao `motion.duplicator` muda ZERO cartões no produto.** A recusa que manda
não é a do planeador, é a da PONTE, e ela corre **antes de planear**:

```rust
if graph_has_live_vector_source(&motion.doc.graph, &motion.registry) {
    return fell(motion, "CPU: o grafo traz uma FORMA vectorial viva (source.shape)");
}
```

*Ela olha o GRAFO inteiro*, não a fronteira: com o carimbo a ter verbo estrutural, aqueles 36
cartões continuam a cair na CPU pela **mesma linha**, sem uma etapa de GPU a correr.

⚠️⚠️ **Isto refuta a conclusão da §5.2 com o número que ela própria encomendou.** Ela escreveu *«as
duas metades fecham na MESMA wave: a lei prova-se no clone e **ATERRA** no duplicador (34)»* — e o
aterrar era uma suposição sobre uma contagem que não estava feita. *A §5.2 apanhou-me a escolher uma
wave pela pureza da prova; esta apanha-me a escolher a seguinte pela população do NÓ quando o que
decide é a população da CADEIA.*

⭐ **E ela ilibou a cerca §2.1 para esta população, com número:** `0` dos 36 tem `source.object` no
lado da forma, logo o par `graph_has_object_source && suffix_changes_count` **não é o que bloqueia
aqui**. ⛔ Ele continua a ser uma dívida real do dia em que a W2 assar a forma — porque uma forma
assada **é** uma textura —, e a §2.1 fica de pé como preço da W2, não da W1(b).

⇒ **A ordem muda, e a razão é medida:**

| wave | estado | porquê |
|---|---|---|
| **W1(a)** `motion.clone` | ✅ **fechada** | a BANCADA da lei de contagem — `2` cartões, e prova-se sem forma no caminho |
| **W1(b)** `motion.duplicator` | ✅ **FECHADA no ciclo 12** ([doc 120 §8](120_ciclo_12_os_tectos_confortaveis.md)) — a nota que segue é a de 2026-09, e a premissa dela era sobre o CATÁLOGO | inerte até a W2: `36/36` recusados uma camada acima. Construí-la agora seria a **segunda** bancada seguida |
| **W2** o *bake fallback* (ADR-0154 Fase 3) | ⭐ **a seguinte** | é ela que dissolve a cerca que gateia **100 %** dos cartões |

⚠️ **E a W1(b) volta a ser obrigatória no dia em que a W2 aterrar:** com a forma assada, a cadeia
fica `forma(fronteira) → carimbo(fronteira) → …` e **não sobra etapa nenhuma para o dispositivo** —
a W2 sozinha tira a recusa e não põe nada na placa. *As duas são necessárias e a ORDEM entre elas
inverteu-se.*

### §5.7 — ⛔⛔⛔ E A W2 TAMBÉM NÃO ATERRA NAS CENAS DO PRODUTO: o pior cartão desenha `190` linhas

A §4.3 é uma **função**, não um veredito: ela diz quanto custam `N` cópias e **não** diz qual é o
`N` que existe. Cozidas as onze cenas com carimbo pela porta do produto (o `pump`, três tiques —
`audit_the_stamp_live_vector_population`):

| cena | linhas vectoriais | geometrias | encode previsto |
|---:|---:|---:|---:|
| `=62` | `14` | `1` | `0,001 ms` |
| `=98` | `64` | `1` | `0,006 ms` |
| `=110` | `73` | `4` | `0,006 ms` |
| `=114` | `50` | `1` | `0,004 ms` |
| `=115` | `36` | `2` | `0,003 ms` |
| **`=120`** | **`190`** | `4` | **`0,017 ms`** |
| `=121` · `=122` · `=123` · `=124` · `=125` | `24` · `32` · `16` · `40` · `22` | `1` | `≤ 0,004 ms` |

⛔⛔⛔ **A pior cena do produto inteiro desenha `190` linhas e custa `0,1 %` de um quadro.** O
critério que o ADR-0154 escreve para o *bake fallback* é **`100 k+` instâncias estáticas de uma
forma só**, e o produto está **`540×`** abaixo dele. ⇒ *a W2 nunca dispararia em cena nenhuma que
hoje ship.*

⭐⭐⭐ **E é isto que nomeia a população verdadeira deste ciclo: ela não está nas cenas — está nos
DOCUMENTOS DO DONO.** Os dois reports são sobre grafos que ele montou (*«1000 × 1000 no grid»* =
`10⁶`; a estrela a `102 400`), e a escada do §4.3 diz exactamente o que isso custa — hoje `17,4` e
**`1,76 ms`** de encode, contra os `56,2` e `5,66` de antes do carimbo preparado. As cenas de
demonstração nunca foram lentas.

⚠️⚠️ **Três medições seguidas disseram a mesma coisa por eixos diferentes, e é essa a lição da
jornada:** a §5.2 mediu a população do NÓ (`2` contra `34`), a §5.6 a da CADEIA (`36/36` recusados
uma camada acima) e esta a do DESENHO (`190` contra `100 000`). *De cada vez eu tinha uma wave
escolhida por um argumento de mecanismo, e de cada vez a contagem mudou-a.*

⛔ **Consequência prática que não é opcional:** enquanto não existir uma cena **na população do
report**, nem o dono vê a cura nem um gate a mede — as onze que existem não contêm o fenómeno, e
uma delas a ficar `0,017 ms` mais rápida é ruído. *Uma cena que não contém o fenómeno não aprova
nem reprova a cura dele* (a mesma lei que a `=50` da escultura pagou, e a `=45` antes dela).

⇒ **DECISÃO DO DONO, com os números ao lado:** as duas waves que sobram (W2 e W1(b)) curam os
documentos DELE e não as cenas que ship, e o primeiro passo delas é uma **cena nova na população do
report** (`320 × 320` = `102 400` carimbos de uma estrela) — que é, ela própria, a reprodução do que
ele fotografou.

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


## ⭐ A cena `=126` ganhou SIMULAÇÃO com campos (2026-09-23)

Pela regra do dono de 23/09 (doc 103 §1: *toda cena de smoke do Motion tem FORMAS e SIMULAÇÃO com
campos*), as `32 761` estrelas passam a girar como uma galáxia — `grid → integrate ← (vortex →
attractor → curl → drag)`, e só depois o carimbo da estrela. ⚠️ **O gate que o proibia teve a
premissa MORTA à vista** (`a_cena_e_a_cadeia_do_report_mais_a_simulacao_e_nada_mais`): ele dizia
que uma simulação a mais faria a cena *«medir outra coisa»*, e o roteiro compara uma DIFERENÇA de
`raw` entre duas corridas, onde um custo igual nas duas se cancela. Medido: a simulação custa
**`2,02 ms`** p50 por tique em `--release` (a `load 29,6`; a leitura calma fica por fazer, como o
resto dos relógios desta linha). O roteiro deixou de prometer *«os fps ficam nos 60»* (não
medido) e manda olhar só o `raw`. Gates: `a_galaxia_gira_e_fica_do_tamanho_do_campo` (hoje `a_galaxia_gira_e_nao_colapsa`, ver abaixo) (mexe
`> 0,3 m` e o extremo fica abaixo de `1,25 ×` o meio-lado do campo de partida — medido `13,74`
contra `14,6 m`, senão o passo de AFASTAR até caber tudo deixava de ser possível). Mutação 3 de 3.

### ⛔⛔ E ESSA GALÁXIA COLAPSAVA — o report do `Corner Radius` (2026-09-23, no mesmo dia)

O dono devolveu *«estrelas com corner radius de 1 provoca queda de raw»*. O custo dos cantos é o
que a §5.9.1 já mediu (a estrela passa de `12` para `30` vértices, e o quadro segue os segmentos das
estrelas **desenhadas**). ⛔ **O que era novo era a população desenhada, e o defeito era meu:** a
galáxia tinha UMA órbita de equilíbrio (`r* = (s_v/k)²/s_a ≈ 1,7 m`, vortex + ímã + arrasto em
aceleração) e o arrasto levava o campo inteiro até ela. Medido pela sonda
`sonda_a_galaxia_a_longo_prazo` (janela de arranque `21,8 × 6,8`):

| t | 0 s | 10 s | 20 s | 40 s | 60 s |
|---|---:|---:|---:|---:|---:|
| estrelas na janela (antes) | `8 957` | `19 712` | `28 180` | **`32 757`** | `32 761` |
| estrelas na janela (hoje) | `8 957` | `8 849` | `8 931` | `8 959` | `9 048` |

⇒ aos 40 s o recorte por câmara já não tinha o que cortar (`3,7×` as estrelas desenhadas), e os
cantos redondos multiplicavam isso por `2,5×`: o `raw` caía com o TEMPO, e mais com os cantos.
⚠️ **O gate da galáxia olhava 10 s e media a FUGA** (o extremo), e um colapso é o contrário de uma
fuga — ficava verde por cima.

⭐ **A cura é um disco em que todo raio é equilíbrio** (rotação rígida): um `motion.falloff`
circular, linear e invertido (o NÚCLEO, `f = d/17`) à frente das forças; o redemoinho em modo
VELOCIDADE-ALVO (`v = s_v·f`, e o modo alvo também amortece a velocidade radial — o arrasto saiu,
seria um nó morto); e o ímã em `s_a = s_v²/núcleo`, que é a condição de órbita circular a TODO raio.
Uma volta a cada `~53 s`. O gate passou a `a_galaxia_gira_e_nao_colapsa`, com a metade nova a ser uma
**INCLINAÇÃO** (a população na janela não muda mais de `1 %` entre os 2 s e os 10 s — o requisito de
`±10 %` numa sessão de 2 min, com a deriva medida recta). ⚠️ A 1.ª redacção dessa metade (`10 %`
absolutos) deixou **sobreviver** o ímã velho, que com o núcleo colapsa DEVAGAR; o vale medido é
`+0,3 %` (o que shipa) contra `+1,7 %` / `−2,0 %` / `+7,5 %` dos vizinhos. Mutação **4 de 4**.

⭐ **Instrumento novo: `PH2D_DEMO_ALTURA=<metros>`** abre uma cena do `PH2D_GPU_COOK_DEMO` com a
câmara nessa altura — o zoom é um gesto de roda que nenhuma corrida sem interface alcança. Ausente
⇒ o arranque de sempre, ao bit.

✅ **Smoke do dono (2026-09-24), na máquina dele: `raw 132` com cantos a `0` contra `128` com cantos a `1`** — `3 %`, com a galáxia já estável. ⇒ o que ele reportou era o COLAPSO; os cantos, sozinhos e com o recorte por câmara a funcionar, quase não custam.

⚠️ **O relógio desta resposta NÃO está medido:** a máquina esteve a `load 21`–`65` e a mesma cena leu
`43` e `113` de `raw` em duas corridas. O que está medido é a CONTAGEM de estrelas desenhadas, que
não depende da carga. ⏳ Fica **aberta, e é decisão do dono**, a outra alavanca: a barra do LOD da
forma (`LADO_MAXIMO_PX = 4`) — a tile da estrela erra `≤ 1,1` tons até `32 px`, logo subir a barra
faria as estrelas do arranque (`~6 px`) irem a tile e os cantos deixarem de custar; o preço é o que
o doc da constante já escreve (uma tile desce no z para trás dos sprites).
