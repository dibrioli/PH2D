# 116 — CICLO 10 · O CARIMBO NO DISPOSITIVO

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

### §4.3 — ✅ O RELÓGIO: a escada do encode — e a CURA, que custa ZERO nitidez

⛔⛔ **A 1.ª redacção desta secção media OUTRO PROGRAMA, e o número dela está corrigido em baixo.**
A sonda construía uma `VectorScene::new()` por medição; a shell **reaproveita** a cena de um quadro
para o seguinte (`fase_frame_open.rs`: `vector_scene.reset()`) e o `reset` do Vello limpa os fluxos
**mantendo a capacidade** ⇒ um quadro em regime **não paga o crescimento dos buffers**, e a sonda
pagava-o em toda leitura. *É a mesma família do sucedâneo que a `line/components` registou — «uma
sonda que mede um sucedâneo para sempre mede outro programa» —, e ela mordeu aqui.* A `10⁶` cópias
a diferença é de `93,7` (frio) para `56,2 ms` (quente); a `102 400` é de `10,0` para `5,66`.

⚠️ **E as três sondas de relógio deste ficheiro mediam-se UMAS ÀS OUTRAS**: o `libtest` corre os
testes de um binário em paralelo, e a mesma célula leu `9,79`, `11,89` e `23,39 ms` nas três da
MESMA corrida (`2,4×` que não é lei nenhuma). Hoje há uma **fatia** (`Mutex`) que cada sonda toma —
⛔ a cura não podia ser pedir `--test-threads=1` no cabeçalho, que é a nota que o `CLAUDE.md` §2
mede a morrer.

#### A cura: **a forma é encodada UMA vez e CARIMBADA `N` vezes**

⭐⭐⭐ **A ordem do dono (2026-09-20) foi «manter a nitidez e ir procurar uma cura que não a custe»**,
depois de RECUSAR assar a forma numa imagem acima de um tecto. Ela existe, e é exacta.

⭐ **A lei que a torna possível:** no Vello a pose de uma forma **não entra no caminho** — o
`Scene::fill` escreve a transformação num fluxo, o estilo noutro, o caminho em mais dois e o pincel
noutros dois. ⇒ *os bytes do caminho são os mesmos para todas as cópias*, e o que muda por cópia é
a **pose** e a **cor**. A porta nova
([`ph2d_vector::scene_prepared`](../../crates/ph2d-vector/src/scene_prepared.rs)) guarda os bytes do
caminho e reproduz o `Scene::fill` passo a passo — `encode_transform` · `encode_fill_style` · o
caminho · `encode_brush` —, só que o terceiro passo é um `extend_from_slice` em vez de um percurso
elemento a elemento.

⭐⭐ **A prova não é esse parágrafo: é o gate.** `o_carimbo_preparado_escreve_os_MESMOS_bytes`
encoda `N` cópias pelas duas rotas — **com uma tinta diferente em cada cópia** — e compara os
**seis** fluxos do Vello mais os dois contadores, em três formas e nas duas regras de
preenchimento. *Um desvio de um byte num fluxo do Vello não devolve erro nenhum: devolve outro
desenho, e num quadro de `102 400` cópias ninguém o vê a olho.* **7 de 7 mutações sangram.**

⇒ **nitidez: ZERO de custo.** Não há bake, não há tecto, não há resolução: é o MESMO vector, com os
MESMOS bytes, encodado mais depressa.

| cópias | `fill` por cópia (o de antes) | carimbo PREPARADO | % de um quadro | razão |
|---:|---:|---:|---:|---:|
| `1 000` | `0,05 ms` | `0,02`–`0,03 ms` | `~0 %` | `~2×` |
| `10 000` | `0,55 ms` | `0,17 ms` | `1 %` | `3,2×` |
| **`102 400`** | **`5,66 ms`** | **`1,76 ms`** | **`34 % → 11 %`** | **`3,2×`** |
| `1 000 000` | `56,2 ms` | `17,4 ms` | `337 % → 104 %` | `3,2×` |

*(`--release`, quadro QUENTE, mínimo de três por linha, duas corridas independentes com `73`–`85 %`
de CPU ociosa; `PH2D_CARIMBO_PREPARADO=0` devolve a coluna da esquerda — as duas rotas são
byte-idênticas, logo a diferença entre as colunas é **só** relógio.)*

⭐⭐⭐ **O custo por cópia é PLANO e passou de `0,055` para `0,017 µs`** sobre um intervalo de
`1000×` ⇒ o encode continua **linear em `N`**, como a
[ADR-0154](../architecture/decisions/0154-motion-shapes-are-live-gpu-vector-not-baked-tiles.md)
declara — o que mudou foi a constante. ⇒ **o tecto de contagem de uma forma VIVA subiu `3,2×`**: a
um quarto de um quadro de 60 fps cabiam `~76 000` cópias e cabem **`~245 000`**; num quadro inteiro
cabiam `~303 000` e cabem **`~980 000`**.

⛔⛔ **RECUSA MEDIDA — o `Scene::append` de um fragmento NÃO é a cura.** Foi a 1.ª candidata (ela
lia `4,1×` contra a rota de então, e foi ela que abriu a investigação). Contra o carimbo preparado
ela lê `1,10×` a `102 400` e **PERDE** a `10⁶` (`26,7` contra `18,3 ms`) — e um fragmento carrega o
`brush` **ASSADO**, logo só serve cópias da MESMA cor. *Duas razões independentes, e qualquer uma
chega.* A sonda `audit_the_stamp_encode_routes` fica, para a recusa ser uma medição e não uma
opinião.

#### E o mesmo quadro pelas PORTAS DO PRODUTO — o encode era `86 %` do custo de CPU

⭐⭐⭐ **A escada mede o encode SOZINHO; esta mede o quadro do report** (`audit_the_stamp_frame_split`:
a cadeia `grade 320×320 + source.shape(Star) + motion.duplicator`, cozida pelo `pump` e desenhada
pelo `motion_shape_gen::encode`; `12` segmentos por cópia, mínimo de três por coluna, as duas rotas
corridas de seguida):

| rota | cozer | desenho | (pose) | CPU do quadro |
|---|---:|---:|---:|---:|
| `fill` por cópia | `0,94`–`1,04 ms` | **`6,37 ms`** | `0,13 ms` | `44 %` |
| carimbo PREPARADO | `0,64`–`1,36 ms` | **`2,25 ms`** | `0,18 ms` | **`17`–`22 %`** |

⇒ **o quadro passa a menos de metade**, e a cura caiu exactamente na metade que manda. ⚠️ O `cozer`
**não** muda entre as duas rotas (é o mesmo cozimento), e é isso que prova que a diferença é lei e
não máquina.

⛔⛔⛔ **E esta secção só existe com estes números porque uma FIXTURA mentia: toda esta auditoria
mediu um CÍRCULO julgando medir uma estrela.** O `monta` escolhia o `kind` procurando o **rótulo**
`"Star"` na lista de opções do nó — e os `KIND_LABELS` da `source.shape` passaram a ser **chaves de
i18n** (`"node.opts.node_motion_shape.kind_labels.7"`) quando a fronteira dos motores fechou, no
mesmo dia. A procura devolvia `None` e o `if let Some(k)` caía **calado** na forma de omissão.
⭐ **Quem o denunciou foi a contagem de SEGMENTOS que esta sonda passou a imprimir** (`4` — que é um
círculo — onde uma estrela de cinco pontas tem `12`). ⇒ o índice passa a derivar do **próprio enum**
(`ALL_KINDS`, alinhado ao `KIND_LABELS` por gate na crate do nó) e a função **falha alto**; o
`indice_do_quadrado` do `motion_custo_do_quadro_probe` tinha o mesmo defeito e foi curado com ela.
*Um censo que classifica por string tem de provar que a string existe* — e uma tabela sem a
contagem da fixtura ao lado não deixa ninguém ver que ela mudou.

⛔⛔ **E uma hipótese MINHA caiu aqui, medida:** eu escrevera que o que sobra do desenho é o
`instance_pose` (compor base · tamanho · âncora · câmera por instância) e que ele valia ~`40 %`. A
coluna `(pose)` mede-o isolado, pela porta do produto: **`0,13`–`0,19 ms`**, ou seja **`~7 %`**.
*O resto é o encode propriamente dito, e a porta do produto já está no chão da escada* ⇒ **não há
degrau seguinte do lado do CPU**; o que sobra é a RASTERIZAÇÃO.

⚠️ **E o que NENHUMA destas colunas mede é a PLACA.** A lei já estava escrita no
`motion_custo_do_quadro_probe`: *se o encode for barato, o que sobra é a placa, e o que a governa
não é o número de formas — é quantos PIXEIS elas cobrem*. A `102 400` cópias minúsculas o encode
era `44 %` do orçamento de um quadro e passou a `~20 %`; quem quiser o degrau seguinte mede a
rasterização, não o encode.

⛔⛔ **E a W2 tem uma peça que a §5.6 não podia ver, achada a ler o código da ponte:** a marca de
vector vivo é **por TIPO de nó** (`NodeRegistry::register_live_vector_source`, um conjunto de
`NodeTypeId`), e a recusa pergunta *«este grafo contém um nó DESSE TIPO?»*. ⇒ um *bake fallback* não
pode «desligar a marca»: a pergunta da ponte tem de passar de **tipo** para **documento** (*«este
tique desenha geometria viva?»*). *Uma cura que assa a forma e deixa a pergunta onde está entrega
uma cena assada que continua a recusar.*

---

## §5 — O PLANO (passos 3 a 7)

| wave | o que é | porquê nesta ordem |
|---|---|---|
| **W0** ✅ | A **auditoria** (§2–§4), com as três sondas versionadas | A frase da fila nomeava **um** nó e a cadeia tem **três** cercas (§2) |
| **W1(a)** ✅ | **A CONTAGEM** no `motion.clone` — o verbo estrutural, os uniformes derivados e a paridade (§5.5) | É a metade que a [auditoria 98](98_auditoria_de_performance_2026-09-01.md) mede em `50,9×`, e o `clone` é o caso **puro** dela — mede-se sem forma nenhuma no caminho |
| **W1(b)** ⏸️ | O mesmo verbo no `motion.duplicator`, **mais** a partição por textura a sobreviver a ele (§2.1) | ⛔⛔ **A §5.2 disse *«é o ENTREGÁVEL (34), e as duas metades fecham juntas»* e a §5.6 REFUTOU-O com o número que ela encomendou:** `36` de `36` cartões trazem um vector VIVO, logo a ponte recusa-os uma camada acima e o kernel move **zero**. *Ela volta a ser obrigatória no dia em que a W2 aterrar* |
| **W2** ⛔ **RETIRADA** | **A FORMA DESENHÁVEL:** o *bake fallback* do ADR-0154 Fase 3. ⛔ O dono **RECUSOU-O** em 2026-09-20 (*«manter a nitidez»*), e a §4.3 tirou-lhe o motivo: o encode a `102 400` passou de `34 %` de um quadro para **`11 %`** sem assar nada e com os bytes idênticos | O que a justificava era o relógio, e ele mudou. ⇒ o que fica é a **cerca** que ela ia atravessar, e essa é uma pergunta de PONTE (de *tipo* para *documento*), não de bake |
| **W3** | A **MEDIÇÃO** do ciclo (passo 5) — a mesma bancada, depois das curas | §0.0 |
| **W4** | O **smoke do dono** (passo 7) | **Enio** |

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

### §5.5 — ✅ A W1(a) FECHOU: o `motion.clone` CORRE NO DISPOSITIVO

O kernel é o molde do §5.3 ponto 1 com uma **translação** no lugar da rotação — *slice-major*
(`copia = i / n`, `linha = i % n`), `write_cp_rows`, `read_P(linha)`, `write_P`, e o sequenciador a
colher todas as outras colunas do template nesses índices, que é exactamente o `replicate` da CPU.

**Três números saem do hospedeiro por `DerivedUniform`, calculados pelas MESMAS funções do `eval`:**

| uniform | a lei | quem mais a lê |
|---|---|---|
| `cl_k` | `copies_within_budget(param_as_count(count), n, RECOMMENDED_MAX_ELEMENTS)` | a `count_law` deste kernel |
| `cl_step_x` / `cl_step_y` | `radial::linear_step(angle, distance)` | o `Placement::of` da CPU |

⛔ **O passo NÃO podia ser trig no dispositivo** (HR-5: a seno parabólica da casa portada seria a
segunda cópia de uma lei, e o `sin` do WGSL é outra curva) — e ele é um número **por NÓ**, logo
derivá-lo não custa um ciclo à placa. ⛔ **E o `k` também não:** reescrito em WGSL ele seria a
segunda resposta a *«quantas cópias?»*, e duas respostas que discordem **desenham um número
diferente de coisas**. ⭐ O `linear_step` nasceu desta wave como **porta com dois leitores** — o
`eval` multiplica-o pelo posto, o uniform entrega-o resolvido.

#### ⛔⛔ E a §5.4 estava INCOMPLETA por UMA variante — quem o disse foi um gate que já existia

A `Count` da CPU vale `total` em toda a linha: *ela não depende do que entrou*, logo o corpo
escreve-a e **nunca a lê**. Ligada com a cruza — a única variante que então tinha as duas metades
de que ela precisa —, o módulo declarava `in_Count`, o corpo não chamava `read_Count`, **a naga
apagava esse buffer do layout derivado** e o bind group do sequenciador ficava com uma entrada a
mais: `create_bind_group` estoura, e **só no tique em que a coluna nasce**.

⭐⭐ **Quem o apanhou foi o `every_registered_kernel_validates_across_the_whole_presence_space`**,
que varre as `2ⁿ` máscaras de presença de todo kernel registado **sem placa nenhuma** — e cuja
mensagem já nomeava o mecanismo inteiro, escrita por outra wave. *Eu tinha considerado este risco e
dispensado-o por raciocínio; o gate mediu-o em sete segundos.*

⇒ **`ColumnAccess::SourceWriteExisting`** (append-only, o 11.º verbo): escreve só se presente ·
porta length-decoupled · **não lê**. As três ligações da família passam a ser

| verbo | lê? | escreve? | porta |
|---|---|---|---|
| `SourceRead` | na fonte | ⛔ nunca | template |
| `SourceReadWriteExisting` | na fonte | só se presente | template |
| **`SourceWriteExisting`** | ⛔ **não** | só se presente | template |

⚠️ **E o predicado mudou de NOME porque a pergunta dele nunca foi sobre LER:** o `column_present`
pergunta *«com que comprimento julgo esta porta?»* ⇒ `is_source_read` → **`is_source_mapped`**.
*Enquanto ele se chamou pelo que dois dos três faziam, o terceiro não cabia no nome sem o tornar
falso.*

⭐⭐ **Duas decisões do codegen deixaram de ser LISTAS e passaram a ser DERIVAÇÕES**, e as duas por
o verbo novo não caber nelas:

1. o braço do `WriteDropped` enumerava `ReadWriteExisting | SourceReadWriteExisting` — *a forma
   exacta que o comentário três linhas acima condena por escrito*; hoje pergunta
   `writes(true) && !writes(false)`, que é **ser condicional**;
2. a **assinatura de presença** tinha o bit preso a `reads()`. Uma escrita condicional sem leitura
   produz módulos diferentes (`WriteBuffer` contra `WriteDropped`) **com o mesmo conjunto de
   ligações declaradas** ⇒ presos à mesma entrada da cache, o wgpu valida um bind group contra o
   layout errado — o crash que o doc daquela função descreve. ⭐ Para todo acesso que já existia a
   troca é **byte-idêntica**: os condicionais de então também liam.

#### A BANCADA: a renumeração não aparece na POSIÇÃO

⛔⛔ `Index`/`Count` são colunas que o desenho não carrega, e a cópia `c` põe as peças exactamente
onde a translação manda **quer o `Index` tenha sido renumerado quer não**. ⇒ a cadeia da bancada
projecta **`Index / Count`** num campo (`value.attribute` × 2 + `value.math ▸ Divide`) e pinta-o com
o `motion.color_ramp`: `t` varre `[0,1)` **uma vez sobre o conjunto multiplicado**, e a TINTA passa
a ser a renumeração à vista. ⭐ O discriminador é o par `(j, j + n)` — o mesmo elemento da fonte em
duas cópias vizinhas —, afirmado no lado da CPU **antes** de se comparar seja o que for.

| caso | pior \|Δpos\| | pior \|Δtint\| |
|---|---|---|
| `count 6` · dist 2 · 0° | `0` | `5,8826e-3` |
| `count 4` · dist 1,25 · 37° | `4,77e-7` | `5,8824e-3` |
| `count 5` · dist 0,9 · −110° · centrado | `0` | `5,8824e-3` |
| **`count 1`** (o cloner em PASSAGEM) | `0` | **`5,8824e-3`** |

⭐⭐ **A última linha é o CONTROLO, e é ela que torna a tabela legível:** com uma cópia só o
multiplicador não multiplica nada e a tinta lê **o mesmo** desvio ⇒ *ele é a quantização da LUT da
rampa* — a `LUT_RESOLUTION` dela é **`256`**, logo a célula mede `1/255 = 3,92e-3` e o canto de uma
parada a cair dentro de uma célula vale `1,5/255 = 5,88e-3`, que é o número medido; tudo isto
pré-existente —, **não** uma deriva do kernel novo. ⛔ E a barra (`1e-2`)
continua a discriminar por duas ordens de grandeza: a avaria que o gate existe para apanhar — a
renumeração a evaporar-se — desloca `t` em `1/k`, que a `k = 6` é `17×` a barra.

⚠️ **A cadeia inteira é reclamada pelo dispositivo** (`grelha → clone → 2× atributo → divisão →
rampa → saída`), com o `plan.is_fully_gpu()` a estourar se não for — *uma fronteira de CPU faria a
bancada comparar a CPU consigo própria*.

#### As RECUSAS, com a população ao lado

Dos `6` cartões de `motion.clone` no produto (§5.2) este kernel alcança **`2`**; as outras quatro
são `applicable` a devolver `false`, **com gate de PLANO a prová-lo e o controlo dos valores de
fábrica na mesma corrida**:

| recusa | cartões | mecanismo |
|---|---|---|
| `time_offset ≠ 0` | `1` | **COZIMENTO e não kernel** — ele re-cozinha a entrada em N instantes (`TimeFans`, ADR-0163). O que muda não é a conta, é quantas vezes o grafo a montante corre |
| `mode = Radial` | `1` | uma **variante** por escrever (o `wgsl_lib` do kaleidoscope já tem a seno parabólica portada), não um bloqueio |
| taper (`scale`/`rot`) | `3` | o único que **CUNHA**: com o knob ligado e a coluna ausente o `eval` cria `size`/`rot`, e uma escrita `…Existing` é por definição a que não cunha. ⚠️ O `applicable` só vê PARAMS, logo a recusa é pelo knob — inclusive nas cadeias em que a coluna existe |

**Gates:** `a_lei_de_contagem_e_a_do_eval` · `o_passo_derivado_e_o_da_cpu_ao_bit` ·
`o_k_derivado_e_o_do_orcamento` · `as_quatro_recusas_sao_as_medidas` ·
`a_renumeracao_entra_pela_cruza` · `o_corpo_chama_o_que_as_ligacoes_declaram` (na crate do nó) ·
`a_escrita_sem_leitura_e_de_porta_template_e_nao_cunha` (no enum) ·
`a_escrita_sem_leitura_nao_liga_buffer_de_leitura` · `a_presenca_de_uma_escrita_sem_leitura_entra_na_chave`
(no codegen) · `o_cloner_concorda_com_a_cpu_dentro_do_epsilon` ·
`as_recusas_do_cloner_entregam_o_no_a_cpu`.

**Mutação: 13 de 13 sangram**, com **seis** controlos verdes — quatro sobre o KERNEL pela paridade
na placa (o `Index` deixa de renumerar · a `Count` fica a do template · a fatia deixa de ser
slice-major · o posto ignora o `center`), uma sobre o **verbo** pelo naga (a `Count` volta a ser a
cruza — *a que reproduz o defeito que esta wave encontrou*), duas sobre os uniformes derivados, uma
sobre as recusas, três sobre o enum e duas sobre a derivação do codegen.

⏳ **O que a W1(a) deixa aberto:** o modo `Radial` como variante (a trig já está portada no irmão),
e o taper — que precisa de um verbo que **CUNHE** numa porta template, exactamente a pergunta que
esta wave respondeu do lado oposto.

#### ⛔⛔ E a pista `--ignored` da GPU tem DOIS vermelhos PRÉ-EXISTENTES, atribuídos por ABLAÇÃO

Correr a bateria de placa inteira (`226` testes que **nem o CI nem o `ship.sh` correm**) devolveu
três reprovados. A atribuição foi feita **antes** de olhar para o commit, ablando a única metade
desta wave que podia tocar noutro nó — o plano derivado do `codegen` e o bit da assinatura de
presença:

| teste | com a minha mudança | ABLADA | veredito |
|---|---|---|---|
| `both_routes_composite_the_same_document_in_the_same_blend` | vermelho | **vermelho** | pré-existente |
| `value_slope_kernel_matches_the_cpu_on_the_device` | vermelho | **vermelho** | pré-existente |
| `crossing_the_reach_boundary_does_not_step_the_cost` | **verde** | **verde** | flake de carga (reprovou no meio dos 226, passa sozinho) |

⚠️ **Os dois são de espécies diferentes e nenhum é um ε a derrapar:** o primeiro devolve **`0`
instâncias contra `64`** (uma avaria estrutural — o dispositivo não emite nada), o segundo lê
`max |d| = 1,05e-4` contra uma barra de `1e-4` (um fio acima). ⇒ *dívida NOMEADA, de quem os
escreveu; esta wave não os toca e não os afrouxa.*

⭐ **E o resto da mudança é inerte por construção, não por promessa:** o verbo novo é **append-only**
e **nenhum kernel existente o declara**; o `is_source_mapped` responde ao mesmo conjunto de sempre
mais ele; e o plano derivado coincide com a lista que substituiu para **todos** os dez verbos
anteriores, porque os únicos condicionais de então também LIAM (`here == presente`).

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
| **W1(b)** `motion.duplicator` | ⏸️ **ADIADA, com o número** | inerte até a W2: `36/36` recusados uma camada acima. Construí-la agora seria a **segunda** bancada seguida |
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

### §5.8 — ⭐⭐⭐ A CENA QUE MOSTRA (ordem do dono, 2026-09-20): `=126`, e a população NÃO é a do report

> ⛔⛔⛔ **LEIA A [§5.9](#59--o-report-do-corner-radius-e-a-segunda-derivação-errada-da-mesma-cena)
> ANTES DE CITAR QUALQUER NÚMERO DESTA SECÇÃO.** A tabela do «o que o app entrega»
> (`33 fps · 29.4 ms · 34 raw` pela rota antiga) **não reproduz**: ela foi obtida escalando por
> regra de três uma medição feita a `313 600` estrelas, e aquele regime tem um multiplicador — a
> dívida de tiques — que **não existe** abaixo da fronteira do vsync. Medido no app à população
> desta cena, a rota antiga custa `14,67 ms` e corre a `60` fps. *A secção fica inteira, com a
> refutação ao lado, porque o erro dela é o achado da §5.9.*

O dono mandou construí-la (*«construa uma cena de demonstração»*), e a §5.7 já dizia porquê: **as
onze cenas com carimbo não contêm o fenómeno** — a pior desenha `190` linhas e custa `0,1 %` de um
quadro. A cena vive em [`motion_state_carimbo_demo.rs`](../../crates/ph2d-app-motion/src/motion_state_carimbo_demo.rs)
e a cadeia é a do report e **nada mais**: `motion.grid → motion.duplicator ← source.shape (Star) →
motion.output`. *Um oscilador a mais entra na conta do quadro e a cena passa a medir outra coisa.*

⛔⛔ **E a população que a §5.7 encomendou — `320 × 320 = 102 400`, a do report — NÃO SERVE para o
olho do dono, e a razão é a `raw`:** pelas portas do produto (§4.3) aquela população custa
`7,36 ms` pela rota antiga e `3,24` pela de hoje, ou seja **as duas cabem num quadro de 60 fps** ⇒
as duas corridas mostram `60` no número que ele lê, e a diferença só aparece no terceiro campo da
barra (`raw = 1000/cpu`). *Ela prova a lei e não a mostra.*

⛔⛔⛔ **E a derivação a partir das portas do produto errou por `~3×` — o achado desta wave.** A
1.ª redacção desta cena escolheu `560 × 560 = 313 600` porque a recta dos `0,0719` / `0,0316 µs` por
cópia prometia `22,6` contra `9,9 ms`. Medido **no app**, com o perfilador e em `--release`:

| rota | quadro medido | ~fps | do qual o COZER | previsto pela recta |
|---|---:|---:|---:|---:|
| `fill` por cópia (ANTES) | `90,4 ms` | `11` | `18,0 ms` | `22,6 ms` |
| carimbo preparado (HOJE) | `53,6 ms` | `19` | `19,7 ms` | **`9,9 ms`** |

⚠️⚠️ **O `audit_the_stamp_frame_split` mede DUAS fases** (cozer · encode) **e o quadro do app tem
mais** — resolver, publicar, os gizmos, a moldura. *Uma derivação que só conta as fases que a sonda
mede prevê um quadro que o app não tem*, e as duas rotas saíram do mesmo lado da fronteira: `11` e
`19` fps, ou seja **as duas a engasgar**, que é exactamente a metade da janela que a cerca de
compilação existia para impedir — **e ela deixou passar, porque as constantes dela é que estavam
erradas**. ⇒ *uma cerca é tão boa como o número que lhe puseram dentro*.

⭐ **O que a mesma tabela ILIBA:** o **cozer é igual nas duas** (`18,0` contra `19,7`, a rota não lhe
toca) ⇒ a distância inteira é o DESENHO. E é por o cozer ser comum que a razão do **quadro**
(`1,69×`) é muito menor que a do **encode** (`3,2×`) — *é a do quadro que manda na população, porque
é o quadro que o dono lê*.

⇒ **`300 × 300` = `90 000`**, com as constantes da cerca re-escritas a partir do APP
(`0,288` / `0,171 µs` por cópia) e a janela a exigir que a rota de hoje caiba num quadro e a antiga
peça **mais de um quadro e meio**. O que o app entrega, com a máquina a `88 %` ociosa:

| rota | **a barra de baixo diz** | quadro | cozer |
|---|---|---:|---:|
| carimbo preparado (HOJE) | **`59 fps · 16.7 ms · 95 raw`** | `16,64 ms` (preso ao ecrã) | `1,8 ms` |
| `fill` por cópia (ANTES) | **`33 fps · 29.4 ms · 34 raw`** | `28,6`–`38,7 ms` | `6,7`–`9,0 ms` |

⭐⭐ **E o cozer caiu de `19,7` para `1,8 ms` sem ninguém lhe tocar:** acima de `16,67 ms` a shell
cozinha **um quadro por tique em dívida** (`MOTION = N × cozer + 1 × separar`, a aritmética que o
`motion.contact` deixou nomeada), logo a `313 600` o cozer já trazia `N ≈ 3` dentro. *A cura do
encode paga-se DUAS vezes: no encode, e no cozer que ela deixa de arrastar.*

⭐⭐⭐ **E as duas fotografias são a MESMA IMAGEM, pixel a pixel** — as duas rotas escrevem os mesmos
bytes, com gate na `ph2d-vector` — **e só o número da barra muda**. É isso que faz desta cena uma
demonstração, e não a comparação de duas coisas diferentes.

⛔ **E o campo é MAIOR que o ecrã por ARITMÉTICA, não por descuido:** uma estrela só se lê como
estrela com `~6 px` (`0,108` de mundo a `55,5 px`/unidade, a régua medida da `=124`) e a câmara de
arranque mostra `21,8 × 6,8` unidades ⇒ cabem **`~10 000`** estrelas legíveis no ecrã, e a cena
precisa de `90 000` para o relógio se mexer. *Encolher a estrela até tudo caber entrega um
rectângulo cinzento*, e uma cena onde o dono não vê estrelas não ensina que aquilo são estrelas —
é por isso que o passo (2) do roteiro manda **aproximar com a roda** até as pontas aparecerem. A
conta é paga pelas `90 000` estejam à vista ou não: **o desenho não tem recorte por câmara**, e é
isso que faz a cena medir o que ela diz medir.

⚠️ **O smoke é `--release`** (`CLAUDE.md` §5: o `smoke` não tem LTO), e por isso a ferramenta de
fotografia ganhou a porta `FOTO_PERFIL` — **por omissão continua `smoke`**, e `release` é só para
um smoke de PERFORMANCE, senão as duas colunas de um A/B medem o perfil de build.

⚠️⚠️ **E a MÁQUINA decidiu metade do dia:** a 1.ª tentativa de medir apanhou a placa **reservada
por outra linha** (o guarda recusou, e bem) e depois `2`–`16 %` de ociosidade; a corrida do
`audit_the_stamp_frame_split` feita nessa janela leu `cozer 4,75 ms` contra os `0,99` da máquina
calma — **`4,8×`**, e foi deitada fora. *Os números desta secção são todos de `88 %` ocioso, e a
sonda imprime a carga ao lado de cada um precisamente para que uma tabela dessas nunca entre num
doc por engano* (`CLAUDE.md` §5.0).

---

### §5.9 — O report do `Corner Radius`, e a SEGUNDA derivação errada da mesma cena

O dono correu o smoke da §5.8 e devolveu **uma frase com duas coisas dentro**: *«O caminho mais
novo é mais rápido mas nenhum dos dois tolerou modificar o corner radius das estrelas. travou»*.

#### §5.9.1 — O que o `Corner Radius` faz, medido

⭐ **A causa é de POPULAÇÃO DE SEGMENTOS e nada mais.** O
[`round_closed_corners`](../../crates/ph2d-vec-scene/src/corners.rs) troca **cada** vértice de quina
por dois — ou **três**, quando a quina vira mais de `90°`, que é o caso das pontas de uma estrela —
⇒ a estrela de `5` pontas passa de **`12` para `30`** vértices assim que o `corner` sai de zero, e
fica lá (a contagem é `30` em todo o resto do curso do slider).

Medido no app (`--release`, `1930×1040`, o `cpu-encode(raw)` do perfilador, que é o **quadro
inteiro de CPU**), sobre as `90 000` estrelas desta cena:

| rota | `corner = 0` (12 vértices) | `corner = 0,5` (30 vértices) | razão |
|---|---:|---:|---:|
| carimbo preparado | `9,99 ms` | **`19,53 ms`** | `1,96×` |
| `fill` por cópia | `13,36 ms` | **`30,30 ms`** | `2,27×` |

⇒ **o quadro dobra**, e as duas rotas passam o orçamento de `16,67 ms`. *A `90 000` cópias não há
rota que salve `2,5×` de segmentos.*

⚠️ **E o GESTO está ilibado com número:** o custo de um arrasto de slider é o `publish` com uma
chave de conteúdo nova (a forma reconstruída, internada e medida para o colisor), e ele mede
**`0,01 ms`** — dentro do ruído. *Arrastar não é pior do que o valor parado; o que trava é o que
fica desenhado.* O instrumento `PH2D_CARIMBO_CORNER=<f>` semeia o valor sem clicar, e é o que torna
isto medível (um gesto de painel **não é alcançável de um teste**).

⛔⛔ **E as sondas de encode NÃO continham o fenómeno, por uma razão que vale para toda a casa.** A
[`audit_the_corner_radius_cost`](../../crates/ph2d-app-motion/src/motion_carimbo_relogio_probe.rs)
mede, pelas portas do produto, `cozer + desenho`: ela lê `3,10 → 4,08 ms` (`+32 %`) pela rota de
hoje enquanto o app lê `+96 %`. O que falta na conta é a **preparação da cena no Vello**, que corre
depois do encode, escala com os segmentos e **nenhuma sonda deste módulo mede** — o cabeçalho do
`motion_carimbo_relogio_probe` já o dizia por escrito (*«o que sobra é a RASTERIZAÇÃO, que nenhuma
destas sondas mede»*) e eu li a frase como uma nota de rodapé em vez de uma fronteira de régua.

#### §5.9.2 — ⛔⛔⛔ A tabela da §5.8 não reproduz, e a causa é o REGIME

Ao reproduzir o report, o **controlo** contradisse a §5.8: a rota antiga com `corner = 0` lê
**`13,4`–`14,7 ms` de CPU e corre a `60` fps**, onde aquela secção promete `33 fps · 29,4 ms`. Duas
corridas independentes, a `91 %` de CPU ociosa, com o mesmo binário.

⭐⭐⭐ **A causa está na ESCADA, e ela é o achado:** com o knob `PH2D_CARIMBO_LADO` mediu-se a
população no app, nas duas rotas —

| lado | estrelas | carimbo preparado | `fill` por cópia | razão |
|---:|---:|---:|---:|---:|
| **`300`** | **`90 000`** | **`10,37 ms`** | **`14,67 ms`** | **`1,41×`** |
| `320` | `102 400` | `16,77` | `21,26` | `1,27×` |
| `400` | `160 000` | `25,64` | `33,40` | `1,30×` |
| `500` | `250 000` | `45,70` | `56,94` | `1,25×` |
| `600` | `360 000` | `72,58` | `88,89` | `1,22×` |

De `90 000` para `102 400` o relógio sobe **`1,62×`** para **`1,14×`** de população; de `102 400`
para `360 000` a inclinação é **`3,3×` menor**. ⇒ *a descontinuidade não é a população — é o
vsync*: acima de `16,67 ms` a shell cozinha **um quadro por tique em dívida**
(`MOTION = N × cozer + 1 × separar`) e esse multiplicador desaparece do outro lado.

⇒ **um custo medido acima da fronteira NÃO se extrapola para baixo dela**, e foi exactamente isso
que a §5.8 fez: ela mediu a `313 600` (onde o quadro já estava em dívida) e dividiu por `3,48`.

⚠️⚠️ **É a MESMA classe de erro da §5.8, um nível acima.** Lá a lição foi *«uma derivação que só
conta as fases que a sonda mede prevê um quadro que o app não tem»* e a cura foi **medir no app**.
Aqui a medição É do app e continua a não transferir, porque **o regime muda**. ⇒ a lei que fica:
*medir no sítio certo não basta — tem de ser no REGIME em que o produto vai correr*.

#### §5.9.3 — ⚠️⚠️ E a fronteira não é só abrupta: ela é INSTÁVEL

Três corridas na fronteira, a `305`, `310` e `315` de lado, pela **mesma** rota e com o mesmo
binário: `15,62`, `11,77` e `16,44 ms` — **não monótonas**. Ali o app cai de um lado ou do outro da
dívida conforme o ruído do arranque, e uma cena naquele ponto daria ao dono **um número diferente a
cada corrida**. ⇒ *nenhuma cena deste repo pode viver na fronteira do vsync*, e a cerca de
compilação da `=126` passou a exigi-lo.

#### §5.9.4 — O que a cena passa a ser

A `300 × 300` fica (a razão **cai** com a população, logo aumentar dilui a cura em vez de a
mostrar), e o que muda é **o que o roteiro manda ler**:

| rota | a barra de baixo diz | CPU do quadro |
|---|---|---:|
| carimbo preparado (HOJE) | `59 fps · 16.7 ms · **~96 raw**` | `10,37 ms` |
| `fill` por cópia (ANTES) | `59 fps · 16.7 ms · **~68 raw**` | `14,67 ms` |

⚠️ **Os `fps` são iguais nas duas e isso é o ECRÃ, não a cura** — as duas cabem no quadro e ficam
presas ao vsync. A coluna que se move é o **`raw`** (`1000 / cpu`), que é a folga. *Um roteiro que
mandasse comparar os `fps` ensinaria que a cura não faz nada* — a espécie que o `CLAUDE.md` §5.0
chama de **pior que uma cena ausente**, e que a §5.8 shipou sem saber.

A cerca de compilação passou de duas metades para **quatro**, e uma delas estava **invertida**: ela
exigia que a rota antiga custasse *mais de um quadro e meio* e passava só porque as constantes
vinham do regime com dívida. Hoje: *a de hoje cabe* · *a antiga também cabe* (a cena não vive na
zona instável) · *a antiga usa pelo menos três quartos do quadro* (abaixo disso o que é fixo domina
e a diferença dilui-se) · *e a distância entre as duas é grande o bastante para se ler*.
**10 mutações, 10 sangram**, três delas por **não compilar**.

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
medido) e manda olhar só o `raw`. Gates: `a_galaxia_gira_e_fica_do_tamanho_do_campo` (mexe
`> 0,3 m` e o extremo fica abaixo de `1,25 ×` o meio-lado do campo de partida — medido `13,74`
contra `14,6 m`, senão o passo de AFASTAR até caber tudo deixava de ser possível). Mutação 3 de 3.
