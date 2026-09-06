# Forma **desenhada** contra forma de **fórmula** — o preço medido, e a lista do que dá para fazer

> **Perguntas do Enio, 2026-08-28:** *"um objeto criado através de desenho vetorial custa muito mais
> caro que um criado por fórmula. Isso é correto? […] quais objetos poderíamos criar com fórmulas.
> Tome como referência as shapes do módulo vetor e me traga uma lista extensiva e exaustiva."*
>
> Sonda: [`spike_formula_vs_profile.rs`](../../crates/ph2d-field-eval/tests/spike_formula_vs_profile.rs).

---

## §1 — ⭐ Sim, é correto — e agora tem número

⚠️ **A régua óbvia mente, e isso foi medido HOJE**, na wave do raio por aresta: `7,00×` os nós de
árvore são `1,21×` o relógio. Citar *«26 nós por aresta»* como preço seria repetir o erro na mesma
sessão. ⇒ o que está abaixo é **relógio**, mediana de 7, em série, a `load 5,81`.

| forma | nós | ns/ponto | × |
|---|---|---|---|
| esfera (fórmula) | 16 | `17,30` | `1,00×` |
| toro (fórmula) | 21 | `17,48` | `1,01×` |
| cilindro (fórmula) | 27 | `17,65` | `1,02×` |
| caixa (fórmula) | 28 | `17,66` | `1,02×` |
| caixa arredondada (fórmula) | 30 | `17,70` | `1,03×` |
| **desenhada, 6 lados** | 179 | `21,94` | **`1,27×`** |
| **desenhada, 12 lados** | 351 | `26,42` | **`1,53×`** |
| **desenhada, 24 lados** | 668 | `35,49` | **`2,06×`** |
| **desenhada, 48 lados** | 1 295 | `62,06` | **`3,60×`** |
| **desenhada, 96 lados** | 2 549 | `107,46` | **`6,23×`** |
| **desenhada, 192 lados** | 5 057 | `201,99` | **`11,72×`** |

### §1.1 — ⭐⭐⭐ Duas leituras, e a segunda muda o que se deve construir

**1. Todas as primitivas de fórmula custam o MESMO** — `17,3` a `17,7 ns`, de 16 a 30 nós. Abaixo de
~30 nós o relógio é **custo fixo**, não a forma.

> ⇒ **Uma primitiva de fórmula nova é DE GRAÇA no quadro.** O que decide se ela entra é quantas vezes
> o artista a quer, nunca o preço.

**2. O caro não é «desenhado» — é LISO.** Com 6 lados o desenho custa `1,27×`; a linha só fica cara
quando o contorno tem muitas arestas, e é exactamente isso que uma **curva** vira ao ser cozida. Um
rectângulo desenhado (4 segmentos) é barato; uma elipse desenhada não é.

⚠️ **E o número acima é o da fita CRUA.** Na produção o traçado especializa a árvore por região, e
isso já está medido noutro sítio como **`2×`–`8×`** a favor do perfil (§82.1/§87 do doc 06) ⇒ o
quadro real paga **menos** que esta tabela. A tabela serve para **comparar os dois caminhos**, que é
o que a pergunta pede.

### §1.2 — O mesmo cilindro pelos dois caminhos (o controlo)

Comparar uma esfera com uma extrusão de 96 lados mede a **complexidade da forma**, não o **caminho**.
A leitura honesta é a mesma forma dos dois modos:

| caminho | ns/ponto | × a fórmula |
|---|---|---|
| cilindro por **fórmula** | `17,78` | `1,00×` |
| desenhado, 24 lados | `36,08` | `2,03×` |
| desenhado, 96 lados | `107,47` | `6,04×` |
| desenhado, 192 lados | `199,41` | `11,22×` |

⚠️ E o desenhado com 96 lados **ainda não é um cilindro**: ele erra a flecha (`r·(1 − cos(π/n))`),
que a `04_resultados_perfis.md` já mede. *Paga-se 6× para ficar perto de uma coisa que a fórmula dá
exacta.*

---

## §2 — A lista, com as 47 formas do catálogo vetorial como referência

A fonte é [`ph2d_vec_scene::ShapeKind`](../../crates/ph2d-vec-scene/src/kind.rs) — **47** formas.
A classificação abaixo é por **confiança**, não por gosto:

| | o que quer dizer |
|---|---|
| **A** | há **SDF 2D exacta publicada** (Inigo Quilez, *2D distance functions* — a mesma fonte que o `ops.rs` já cita) ⇒ porte, não invente |
| **B** | **composição** de formas A com operações que **já temos** (união, subtracção, intersecção, espelho, matriz, radial) ⇒ certo por construção |
| **C** | precisa de **aproximação ou pesquisa** — a distância exacta não é fechada |
| **D** | ⛔ **quer mesmo um contorno desenhado** |

### §2.1 — Geometria base

| Forma do catálogo | Classe | Como |
|---|---|---|
| Rectangle | **A** | `sdBox` 2D — já temos em 3D |
| RoundRect | **A** | `sdRoundedBox`, e com **4 raios** (medido em 28/08, `1,21×`) |
| Ellipse | **A** | `sdEllipse` — ⚠️ exacta mas **pesada** (resolve uma quártica); medir antes de prometer |
| Polygon (3–128 lados) | **A** | `sdRegularPolygon` — fechada, com uma dobra angular |
| Star (n pontas, razão interna) | **A** | `sdStar(n, m)` |
| Line | **A** | `sdSegment` (é traço, não preenchimento) |
| Arc | **A** | `sdArc` |
| Pie | **A** | `sdPie` |
| Segment (fechado pela corda) | **B** | disco ∩ semiplano |
| Spiral | **C**/**D** | ⛔ a distância a uma espiral de Arquimedes **não é fechada**. Ou aproximação polar, ou fica desenhada |

### §2.2 — Fluxograma (ANSI/ISO 5807) — quase tudo é composição

| Forma | Classe | Como |
|---|---|---|
| Diamond | **A** | `sdRhombus` |
| Pill | **A** | `sdCapsule` / `sdUnevenCapsule` |
| Parallelogram | **A** | `sdParallelogram` |
| Trapezoid · TrapezoidFlip | **A** | `sdTrapezoid` (a segunda é a primeira espelhada) |
| HexagonFlat | **A** | `sdHexagon` |
| Cylinder (o símbolo) | **B** | rectângulo ∪ duas meias-elipses |
| Delay | **B** | rectângulo ∪ meio-disco |
| Display | **B** | idem, com a ponta |
| PredefinedProcess | **B** | rectângulo ∪ duas barras |
| OffPage | **B** | rectângulo ∩ semiplano inclinado |
| Junction | **B** | disco ∪ cruz |
| NoteBracket | **B** | três segmentos |
| Document | **C** | a base **ondulada** é uma senóide, e a distância a ela não é fechada |

### §2.3 — Setas, balões e símbolos

| Forma | Classe | Como |
|---|---|---|
| ArrowRight · ArrowDouble · Chevron | **B** | rectângulo ∪ triângulo(s) |
| ArrowBent | **B** | a mesma, com um arco no cotovelo |
| SpeechRect · SpeechOval | **B** | corpo ∪ triângulo da cauda |
| **Thought · Cloud** | **B** ⭐ | **união de discos** — e com o `Organic` (smooth-min) ela sai **melhor** que desenhada: é a forma que o SDF faz naturalmente |
| Burst | **A** | variante do `sdStar` com razão interna alta |
| Brace | **B** | quatro arcos + dois segmentos |
| Heart | **A** | `sdHeart` |
| Moon | **A** | `sdMoon` (disco menos disco deslocado, com distância exacta) |
| Drop | **B** | disco ∪ triângulo, com **smooth-min** — ou `sdEgg` (**A**) |
| Bolt · Check · Shield · Tag · Banner | **B** | polígonos de poucos vértices, ou composições |
| Cross | **A** | `sdCross` / `sdRoundedX` |
| **Gear** | **B** ⭐⭐ | **um dente + repetição radial** — e o modificador `Radial` **já existe** neste módulo. É onde o SDF humilha o desenho: um dente e um número |
| IsoCube · IsoCone · IsoPyramid | — ⭐ | são **desenhos isométricos de sólidos**. Em 3D deixam de fazer sentido: usa-se o sólido de verdade |

### §2.4 — ⭐ E as 3D que o catálogo vetorial nem podia ter

Todas **A** (mesma fonte), e nenhuma existe hoje no módulo:

**Cone** · **cone truncado** · **cápsula** · **elipsóide** (aproximada — a exacta não é fechada) ·
**octaedro** · **prisma hexagonal** · **prisma triangular** · **pirâmide** · **toro parcial** (com
ângulo) · **elo de corrente** · **cunha / ângulo sólido** · **esfera cortada** · **calota oca** ·
**moldura de caixa** (a "gaiola" de arestas) · **rombo 3D**.

### §2.5 — ⭐⭐ O multiplicador que já está pago

O módulo já tem `Mirror` (3 eixos), `Array`, `Radial`, `Shell` (ocar), `Offset` e `Taper`. **Cada
forma nova da lista multiplica por eles de graça.** Uma engrenagem é *um dente + `Radial`*; um flange
é *um cilindro + `Radial(furo)`*; uma coluna torcida é *um perfil + `Taper`*.

---

## §3 — Recomendação

1. ⭐ **As primitivas de fórmula são de graça no quadro (`1,00`–`1,03×`).** O critério de entrada é
   **quantas vezes o artista as quer**, e não o preço.
2. ⭐⭐ **O primeiro lote que eu escolheria** — as que aparecem em todo modelo e hoje obrigam a
   desenhar: **cone**, **cone truncado**, **cápsula/pill**, **prisma de N lados** (o `sdRegularPolygon`
   extrudado cobre hexágono, octógono e o resto de uma vez), **cunha** e **toro parcial**.
3. ⭐⭐⭐ **E o de maior alavanca por linha de código é a ENGRENAGEM**, que não é uma forma: é *um
   dente + o `Radial` que já existe*. O mesmo mecanismo dá flanges, discos de furos, rosetas.
4. ⛔ **O que fica desenhado, e está certo que fique:** a espiral, a base ondulada do `Document`, e
   qualquer contorno **autoral** — é para isso que o vínculo desenho→peça existe.

---

## ⛔ Recusas MEDIDAS

| Recusa | Motivo medido |
|---|---|
| Ler o preço de uma forma na **contagem de nós** | `7,00×` os nós foram `1,21×` o relógio na wave do raio por aresta; e aqui `16` contra `30` nós dão o **mesmo** tempo (§1.1) |
| Dizer que *«desenhado é caro»* sem o número de arestas | com 6 lados é `1,27×`. O caro é **liso**, não desenhado (§1.1) |
| Comparar uma esfera de fórmula com uma extrusão de 96 lados | mede a complexidade da FORMA, não o caminho — a leitura honesta é a mesma forma pelos dois modos (§1.2) |
| Espiral por fórmula | a distância a uma espiral de Arquimedes não tem forma fechada (§2.1) |

---

## §4 — ⭐⭐⭐ A AUDITORIA de 29/08: quantas das 47 o módulo JÁ exprime

O §5.0 do `CLAUDE.md` manda medir se a composição já exprime o item **antes** de o construir. Com o
lote da W101 no lugar (cone · tronco · cápsula · prisma de N lados), a lista encolhe muito.

### §4.1 — ⛔ O que NÃO se deve construir, porque já se faz hoje

| Forma do catálogo | Como se faz **hoje** |
|---|---|
| **Diamond / rombo** | **prisma de 4 lados** (o circunraio é a diagonal) |
| **HexagonFlat** | **prisma de 6 lados** |
| Polygon (3–128) | **prisma de N lados** (até 32 — acima disso é um cilindro, §MAX_PRISM_SIDES) |
| Rectangle · RoundRect (raio uniforme) | **caixa**, com `Fillet` |
| **Cross / plus** | duas caixas em **união** |
| **Moon** | cilindro **menos** cilindro deslocado |
| **Junction** | cilindro ∪ cruz |
| **Cloud · Thought** | ⭐ esferas com junta **Organic** — sai *melhor* que desenhada |
| **Drop / gota** | esfera ∪ cone, junta `Organic` |
| PredefinedProcess · NoteBracket · Banner · Tag | caixas em união/subtracção |
| ArrowRight · Chevron · Bolt · Check · Shield | caixa(s) ∪ **prisma de 3 lados** rodado |
| Cylinder (o símbolo) · Delay · Display | caixa ∪ cilindro |
| IsoCube · IsoCone · IsoPyramid | são **desenhos** de sólidos — em 3D usa-se o sólido |
| **Gear** | ⭐⭐ um dente (caixa ou tronco) + o modificador **`Radial`**, que já existe |

⚠️ **A engrenagem saiu da fila de construção por isto** — ela era o item que a §3 chamava de maior
alavanca, e a alavanca **já estava montada**: o `Radial` é do módulo desde a W12. *O que se perde ao
não reconferir não é tempo, é construir o que já existe.*

### §4.2 — ⭐ O que continua a faltar, e por que cada um não é composição

| Falta | Por que a composição não chega |
|---|---|
| **Pirâmide / tronco de pirâmide** | é o **prisma com o topo mais estreito** — não há como estreitar um prisma hoje (o `Taper` age em Y, é inexacto, e o `TAPER_FLOOR` impede o ápice) |
| **Cunha / rampa** | é uma caixa **cortada por um plano inclinado**, e não há primitiva de plano — cortar com uma caixa gigante rodada é um objecto a mais e um número sem sentido |
| **Arco de toro** | um sector angular precisa de dois semiplanos; hoje seriam **duas caixas gigantes** rodadas à mão |
| **Estrela de N pontas** (N ímpar) | uma estrela de 6 é dois triângulos; **uma de 5 não é união de polígonos nenhuns** |
| **Elipse / elipsóide** | a escala do módulo é **uniforme de propósito** (`‖∇f‖ = 1` é a fundação) ⇒ não há como achatar |
| **Moldura de caixa** (a gaiola) | caixa menos três caixas — faz-se, mas com **4 objectos** para uma forma que é 1 |
| **Espiral** | ⛔ classe **D**: a distância a uma espiral de Arquimedes não é fechada |

⇒ **A fila real é de ~7 itens, não de 47.** O resto ou já se faz, ou é desenho.

---

## §5 — ⛔⛔⛔ «A FILA FECHOU» — e ela fechou contra a LISTA ERRADA (corrigido em 30/08)

> ⚠️ **Esta seção afirmou, durante um dia, que a fila estava fechada. Estava errada, e o dono do
> produto foi quem a corrigiu:** *«Não finalizamos de construir as formas do catálogo. temos poucas
> formas»* (Enio, 30/08, com a foto da paleta a 19 itens e a família *Plates* com **uma**).
>
> **As duas causas, e as duas são instrutivas:**
>
> 1. ⛔ **Ela fechou contra a lista errada.** A [§4](#§4) auditou *«quantas das **47** do catálogo
>    vetorial»* — e o [§2.4](#§24) deste mesmo documento tem uma **segunda** lista, *«as 3D que o
>    catálogo vetorial nem podia ter»*, **quinze** formas, todas classe A, que **nunca foi
>    auditada**. O §5 fechou contra a resposta da §4 e deu a fila por terminada.
>    *Uma auditoria responde a lista que leu; fechar uma fila contra ela exige provar que era a fila toda.*
>
> 2. ⛔⛔ **O argumento que cortou 40 é do tipo errado para um MENU.** *«Já se faz por composição»*
>    responde **«o motor consegue exprimir?»**; a pergunta de uma paleta é **«a pessoa ACHA?»**.
>    Quem abre *Add Shape* à procura de uma engrenagem não quer descobrir que precisa de modelar um
>    dente e encontrar o modificador radial. ⚠️ **A lei já estava escrita neste módulo** — o gate
>    [`field3d_reach_tests`](../../shells/desktop/src/field3d_reach_tests.rs) afirma que *o painel
>    oferece exactamente o que o gesto faz* — e a auditoria passou ao lado dela.
>    ⚠️ E o critério de entrada estava escrito na **§3.1 deste mesmo doc**: *«o critério é quantas
>    vezes o artista as quer, e **não o preço**»* — três parágrafos antes de quarenta serem cortadas
>    pelo preço.
>
> ⇒ **a W106 acrescenta 14 formas** e leva a paleta de 19 para **33 itens**. O mecanismo de cada uma
> vive em [`ops_solids.rs`](../../crates/ph2d-field-eval/src/ops_solids.rs) e
> [`ops_plates.rs`](../../crates/ph2d-field-eval/src/ops_plates.rs); a wave está no
> [doc 06 §105](06_resultados_cena_e_gizmo.md).

## §5-hist — O que a W103 de facto fechou (29/08)

Os três itens que sobravam do §4.2 estão construídos, e o que fica de fora fica **com o motivo**:

| item do §4.2 | estado |
|---|---|
| Pirâmide / tronco de pirâmide | ✅ W102 — o prisma com o topo estreitado |
| Cunha / rampa | ✅ W102 |
| Arco de toro | ✅ W102 |
| **Estrela de N pontas** | ✅ **W103** — união do disco dos vales com uma pipa por ponta; `3..16` pontas |
| **Moldura de caixa** | ✅ **W103** — três caixas dobradas por `abs`, uma primitiva em vez de quatro objectos |
| **Elipse / elipsóide** | ✅ **W103** — ⚠️ a recusa respondia a outra pergunta (ver abaixo) |
| Espiral | ⛔ classe **D**, e continua: a distância a uma espiral de Arquimedes não é fechada. **Fica desenhada** |

### §5.1 — ⚠️ A recusa do elipsóide respondia a OUTRA pergunta

Ela dizia: *«a escala do módulo é uniforme de propósito (`‖∇f‖ = 1` é a fundação) ⇒ não há como
achatar»*. Isso é verdade sobre o [`Xform::scale`], e ali continua a valer — uma pose com escala por
eixo estraga a fundação em toda a árvore abaixo dela. **Uma primitiva com três raios não toca
nisso:** ela é uma folha, e a folha responde por si (`f(p/s)·min(s)` é 1-Lipschitz por construção).

*Uma recusa medida responde UMA pergunta; reconfira-a quando a sua for outra.*

### §5.2 — O que uma forma nova deste módulo custa hoje

Uma linha no catálogo (`field3d_shapes.rs`), uma variante no `Primitive` com os seus cinco braços
fechados (`dims` · `set_dim` · `round_limit` · `characteristic_size` · `bounding_radius` ·
`scale_primitive` · `fillet_inflates`), a fórmula em `ops.rs`, e o rótulo i18n. **O censo derivado
faz o resto**: as quatro perguntas do
[`the_census_of_every_primitive`](../../crates/ph2d-field-eval/tests/the_census_of_every_primitive.rs)
passam a valer para ela sem uma linha de mudança, e um `Primitive` novo é **erro de compilação** até
alguém dizer com que números ela se mede.

---

## ⛔ Recusas MEDIDAS (§5)

| Recusa | Motivo medido |
|---|---|
| Fórmula publicada do elipsóide (`k0·(k0−1)/k1`) | `‖∇f‖ = 1,86` (a marcha atravessa) e `f(centro) = −1` para **qualquer** elipsóide, por `0/0` na origem (doc 06 §101.5) |
| Teto de excentricidade do elipsóide | a forma está **correta**; só a marcha viva degrada (`324` passos a `1:64` contra `MAX_STEPS = 400`) — limitar a peça pelo previsualizador é o §0 ao contrário |
| Polígono dos vales **+ um triângulo por ponta** | é uma **partição**: `min` de peças que se tocam sem se sobrepor dá `0` no interior do sólido (doc 06 §101.5) |
| `MAX_STAR_POINTS` acima de 16 | `24` pontas custam `5,17×` o cilindro, contra o `3,80×` que o `MAX_PRISM_SIDES` fixou como preço aceite |
| **Ovo** por fórmula publicada (W125) | o knob de barriga degenera num **círculo** no valor natural (`x = 0` identicamente), é não-monótono de um lado e sem arco do outro; com ele fixo, `28` de `100` combinações deixam um **vinco** (doc 06 §126.2) |
| **Escada** por fórmula (W125, 4 construções) | a última passa marcha, chanfro e arestas — e o filete é **neutro em volume** (`20 139` amostras dentro com e sem), porque ela tem tantas quinas côncavas como convexas e do mesmo raio. ⛔ O `<` estrito de `the_biggest_fillet_still_leaves_a_body` não distingue *equilibrado* de *inerte*, e foi ele que apanhou o `round` inerte do cone e do prisma (doc 06 §126.4) |

| Filete que só é arco a **90°** | `(1 − 1/√2)·r/sin α` contra `r·(1/sin α − 1)`; as duas curas (canto exato · raio compensado) **pioram** a sonda de arestas (doc 06 §102.5) |
| Canto exato dobrado sobre expressão composta | a corda e o disco são globais: `0,0 %` → `1,3 %` de aresta viva no prisma (doc 06 §102.5) |
| Sonda de arestas semeada só por FORA | cega ao vinco **côncavo**: a mutação do vale da estrela sobrevive (`1,4 %` → `1,3 %`, ruído) — doc 06 §102.1 |

---

## §6 — ⭐⭐⭐ O LEVANTAMENTO de 04/09, e o LOTE 1 (W119)

> **Enio:** *«No plano original seriam mais de 40 shapes prontas […] Busque saber as que faltam
> implementar.»*

⚠️ **A fila conta-se contra as DUAS listas deste documento** — as 47 do catálogo vetorial (§2.1–§2.3)
**e** as 15 sólidas do §2.4 —, que é precisamente o erro que o §5 registou.

| | formas |
|---|---:|
| o plano (47 + 15) | **62** |
| já cobertas por uma porta da paleta | 36 |
| classe **D**, ficam desenhadas (espiral · a base ondulada do `Document`) | 2 |
| **por construir em 04/09** | **24** |

### §6.1 — O que a W119 fechou (lote 1): **9 portas**, `33 → 42` itens

**Setas (4):** Arrow · Double Arrow · Bent Arrow · Chevron ·
**Fluxograma (1):** Diamond (o losango de diagonais **diferentes** — ⛔ o prisma de 4 lados tem-nas
iguais, e é por isso que o §4.1 estava errado ao dá-lo por coberto) ·
**Redondas (2):** Circle Segment · Ring Arc ·
**Rings & tubes (2):** Tube · Washer — ⚠️ as duas **não** estavam em nenhuma das listas, e o nome da
família prometia-as desde a W100.

Mecanismo, as cinco medições que mudaram o desenho e as provas de mutação:
[doc 06 §120](06_resultados_cena_e_gizmo.md).

### §6.2 — ⏳ O que FICA (15), na ordem proposta

| lote | formas |
|---|---|
| ✅ **2** (10) | **FECHADO na W120** — os 4 balões (Speech Balloon · Speech Oval · Thought · Cloud) e os 6 símbolos (Lightning Bolt · Shield · Tag · Check Mark · Banner · Brace). Mecanismo: [doc 06 §121](06_resultados_cena_e_gizmo.md) |
| ✅ **3** (4) | **FECHADO na W122** — Parallelogram · Delay · Display · Off-page Connector. Mecanismo: [doc 06 §123](06_resultados_cena_e_gizmo.md) |

⛔⛔ **A linha do lote 3 dizia CINCO e contradizia-se três linhas abaixo:** ela punha a `Junction`
entre as formas a construir e o aviso seguinte dizia que ela tinha sido tirada. A resposta certa é a
do aviso, e agora com o mecanismo em vez do gosto — **a junção do ANSI é «disco ∪ cruz» e a cruz vive
DENTRO do disco**, então `min(disco, cruz) = disco` em todo o ponto: *a união de um conjunto com um
subconjunto dele é o próprio conjunto*. O sólido dela é o cilindro que a paleta já tem. Pelo mesmo
argumento ficam de fora o `PredefinedProcess` e o `NoteBracket`.

⇒ **a fila das duas listas deste documento está FECHADA.**

⛔⛔⛔ **E as duas que ficavam «desenhadas» também caíram, no mesmo dia** (W123, a pedido do dono:
*«usando fórmulas não ficam mais leves? Implemente»*). A recusa dizia que *«a distância a uma
espiral de Arquimedes / a uma senóide não é fechada»* — o que é **verdade e não é o que o módulo
pede**: uma marcha de esferas precisa de um **minorante**, nunca do valor exacto. As duas são hoje
primitivas de fórmula, e o preço de as ter deixado desenhadas estava medido desde 28/08: o **mesmo**
cilindro custa `1,79 ns/ponto` por fórmula e `181,44 ns` desenhado com 192 lados (**`101×`**).
⭐ Medido na espiral: `passo × ‖∇f‖` fica em `0,9899` de **1 a 32 voltas** — *o campo de uma espiral
não sabe quantas voltas ela tem*. Mecanismo: [doc 06 §124](06_resultados_cena_e_gizmo.md).

⇒ **não sobra nenhuma forma da lista por construir.** O contorno **autorado** continua a ser
desenhado, e é para isso que o vínculo desenho→peça existe.

⚠️ **Três do lote 3 são FRACAS em 3D e isso está medido pelo desenho, não pelo motor**
(PredefinedProcess · Junction · NoteBracket): elas nasceram como **desenhos de linha** de um
fluxograma, e um sólido de 3 mm de espessura com dois traços dentro não é a forma que o nome promete.
⇒ **decisão de produto**, e a lista acima já as tirou dos 5.

⛔ **E o critério de entrada continua a ser o §3.1 deste doc** — *quantas vezes o artista as quer, e
**não** o preço*: uma primitiva de fórmula custa `1,00×`–`1,03×` a esfera, e o preço nunca é o que
decide.

---

## §7 — ⭐⭐⭐ O LEVANTAMENTO de 05/09: o que a FÓRMULA ainda dá, e nós não temos

> **Enio, 05/09**, depois do smoke da espiral: *«diante do sucesso da espiral, faça pesquisa de
> shapes geradas por fórmulas que ainda não temos»*.

⚠️ **A pergunta mudou desde o §2.** Aquele levantamento contava contra o **catálogo vetorial da
casa** (47 formas) e contra as 15 sólidas — duas listas que hoje estão **fechadas**. Este conta
contra a **literatura de campos de distância**, que é uma lista maior e de outra natureza: ali as
formas não vêm de um menu de desenho, vêm de uma **fórmula publicada**.

⚠️⚠️ **E a régua de entrada é a mesma do §3.1** — *quantas vezes o artista a quer*, e **não** o
preço.

⛔⛔ **Mas o número do preço estava ERRADO nesta frase, e a W124 mediu-o** (doc 06 §125.1): *«uma
primitiva de fórmula custa `1,00×`–`1,03×` a esfera»* vale para a família **algébrica** (caixa,
cilindro, toro: `1,00`–`1,25×`) e **não** para a **transcendente** — a espiral custa `9,7×`, a mola
`11,9×` e o gyroid **`28,4×`**, porque o que custa é o `atan2`, o `sin` e o `cos`. ⭐ A conclusão
não muda: o gyroid é a primitiva mais cara desta casa e ainda assim é `4,7×` mais barato do que um
contorno de 192 segmentos (`134×`) — que, no caso dele, nem sequer se pode desenhar.

### §7.1 — A fonte, e o que ela diz que é EXACTO

As duas páginas que o `ops.rs` já cita — [3D](https://iquilezles.org/articles/distfunctions/) e
[2D](https://iquilezles.org/articles/distfunctions2d/) — trazem **29** funções 3D (26 exactas) e
**43** 2D. ⚠️ **A distinção «exacta / limitada» é a que importa aqui, e não a que parece**: desde a
W123 este módulo sabe que ele **nunca precisou da distância exacta** — precisa de um **minorante**
(doc 06 §124). ⇒ uma função *«bounded»* da fonte é tão utilizável quanto uma exacta, desde que o
limite seja honesto.

### §7.2 — Do catálogo 3D faltam TRÊS (a quarta shipou na W125; as outras 25 já se faziam)

| Forma | Classe | O que ela é, e por que não é composição |
|---|---|---|
| **Plane / semiespaço** | **A** | ⭐⭐ O corte. O [§4.2](#42--o-que-continua-a-faltar-e-por-que-cada-um-não-é-composição) **já a nomeia como a peça em falta**: hoje cortar em ângulo é *«uma caixa gigante rodada — um objecto a mais e um número sem sentido»* |
| ~~**Rounded Cylinder**~~ | — | ✅ **SHIPOU na W125** — o cilindro com bojo (rolha, pneu, botão), com um knob só. ⛔ Não é o nosso `round`, que só arredonda o aro |
| **Death Star** | **A** | ⭐⭐ Esfera **menos** esfera, com a distância **exacta** na cratera. A nossa subtracção dá a forma e **não** dá a distância exacta ali |
| **Vesica Segment** | **A** | A lente 3D entre dois pontos — a cápsula «com barriga para dentro» |

⛔ **`udTriangle` e `udQuad` ficam de fora por natureza:** são superfícies **sem volume** (distância
*não assinada*), e um modelador de sólidos não tem o que fazer com elas.

### §7.3 — Do catálogo 2D faltam ONZE que valem uma chapa

⛔⛔⛔ **CORRIGIDO em 06/09: QUATRO destas onze já se alcançam, e a lista comparava NOMES.**
A pergunta certa é *que **forma** o produto alcança* — e uma **rotação** é um gesto que o artista já
tem, não uma montagem:

| eu listei | e ela é |
|---|---|
| ~~Horseshoe~~ | o **`Tube` com ângulo** (um sector de anel com pontas quadradas) |
| ~~Rounded X~~ | a **`Cross` rodada `45°`** — ela já tem `round` |
| ~~Tunnel~~ | o **`Delay` rodado um quarto de volta** |
| ~~Uneven Capsule~~ | o **`RoundCone`**, que é a versão 3D dela |

⇒ ficam **sete** — e a W125 mediu **duas** dessas sete até à recusa (o ovo e a escada), pelo que sobram **cinco**. Mecanismo: [doc 06 §126](06_resultados_cena_e_gizmo.md).

| Forma | Classe | Por que ela |
|---|---|---|
| **Polygon (N vértices arbitrários)** | **A** | ⭐⭐ O polígono **irregular** por fórmula. Hoje isto obriga a **desenhar**, e desenhar custa por segmento |
| **Triangle (3 vértices arbitrários)** | **A** | ⭐ O prisma só faz **regulares**; um triângulo escaleno hoje é um desenho |
| ~~**Egg**~~ | — | ⛔ **CONSTRUÍDO e RECUSADO na W125** — o knob de barriga degenera num círculo no valor natural (recusas medidas, abaixo) |
| ~~**Stairs**~~ | — | ⛔ **CONSTRUÍDO 4× e RECUSADO na W125** — o filete dela é neutro em volume, e o gate que o exige está certo (recusas medidas, abaixo) |
| **Bezier (quadrático)** | **A** | ⭐ Um traço curvo com espessura, sem desenhar |
| **Parabola** | **A** | A curva, com espessura |
| **Circle Wave** | **A** | ⭐ A onda em anel — irmã directa da onda que o `Document` estreou |

✅ **E estas do catálogo 2D já se fazem**, para não as reconstruir: `sdCircle`/`sdBox`/`sdRoundedBox`
(caixa e cilindro), `sdChamferBox` (o **chanfro** é um controlo desta casa), `sdOrientedBox` (a pose),
`sdSegment` (cápsula), `sdRhombus`, `sdTrapezoid`, `sdParallelogram` (W122), `sdPentagon`/`sdHexagon`/
`sdOctogon` (o prisma de N lados), `sdStar` — e com ela o **pentagrama** e o **hexagrama**, que são a
mesma fórmula com `points` e a razão interna certa —, `sdPie`, `sdCutDisk`, `sdArc`, `sdRing`,
`sdVesica`, `sdMoon`, `sdRoundedCross`, `sdHeart`, `sdCross`, `sdEllipse`.
⛔ `sdfCoolS` é uma piada do autor; `sdHyperbola` e `sdQuadraticCircle` são curvas de estudo.

### §7.4 — ⭐⭐⭐ E as SEIS famílias que não estão em catálogo nenhum — é aqui que está o salto

| Família | Classe | Por que ela vale mais do que uma forma |
|---|---|---|
| **Hélice / mola** | **B** ⭐⭐⭐ | **O irmão 3D da espiral, e o mecanismo já está pago**: a volta mais próxima sai de um `round()`, exactamente como na W123. Molas, parafusos, cabos, corrimãos, DNA, escadas em caracol |
| **Gyroid e a família TPMS** | **C** ⭐⭐⭐ | `sin x·cos y + sin y·cos z + sin z·cos x = 0` — **uma linha** dá um enchimento infinito. É o que a impressão 3D usa dentro das peças, e o que um artista usa para «isto é uma estrutura». ⚠️ Não é distância exacta; o limite sai por **dividir pelo gradiente máximo**, que é a lei que a onda do `Document` acabou de estabelecer nesta linha |
| **Superfórmula (Gielis)** | **C** ⭐⭐ | **Uma fórmula, centenas de formas orgânicas** — folhas, conchas, flores, estrelas do mar. Seis números. É a maior razão forma/linha-de-código de toda esta lista |
| ~~**Superquadrática / superelipsóide**~~ | — | ✅ **SHIPOU na W127** — com **dois** expoentes (o de cima e o de lado), divisor em forma fechada e a esfera **exacta** no meio. ⛔ A astróide fica **fora com demonstração**: abaixo de `n = 1` o gradiente na superfície não tem limite (cúspides) |
| **Nó de toro (p, q)** | **C** ⭐⭐ | Dois inteiros dão uma família inteira de nós. Decoração, joalharia, matemática |
| **Rosca / knurling** | **B** ⭐ | A hélice varrida num cilindro — o parafuso a sério, e o punho serrilhado |

⚠️ **E há duas que são MODIFICADORES, não formas:** a **grade hexagonal** (favo de mel) é a
repetição que falta ao lado do `Array` e do `Radial`, e as **metabolas** são o `smooth-min` que já
existe com uma contagem por cima.

⛔ **Fractais (Menger, Sierpinski, Mandelbulb) ficam nomeados e fora desta fila.** O *distance
estimator* deles **é** um minorante por construção, o que os torna legítimos aqui — mas o custo é
por **iteração** e não por nó, e isso muda o preço do quadro. *É uma wave com medição própria, não
um item de lote.*

### §7.5 — A ordem que eu proporia

| lote | formas | por quê |
|---|---|---|
| ✅ **4** (2) | **FECHADO na W124** — *Coil* e *Gyroid Lattice*. Mecanismo: [doc 06 §125](06_resultados_cena_e_gizmo.md) |
| ✅ **5** (1) | **FECHADO na W125** — *Rounded Cylinder*, e ele é a primeira forma deste módulo com **distância exacta** (`‖∇f‖ = 1,0000`). ⛔ E das outras cinco que este lote nomeava: a **ferradura** e o **túnel** já se alcançam (acima); o **ovo** e a **escada** foram construídos e **recusados por medição** (o knob de barriga mente num quarto do curso; a escada tem o filete **neutro em volume**, `20 139 = 20 139`, e o gate que o mede está certo); e o **plano** precisa que a bola de recorte admita uma peça **infinita** — é maquinaria, não forma. Mecanismo: [doc 06 §126](06_resultados_cena_e_gizmo.md) |
| **6** (2) | **Polygon(N)** · **Triangle** | as que hoje **obrigam a desenhar**. ⚠️ E elas têm um preço próprio: *os vértices arbitrários são o que o desenho já é* — o ganho é só o custo |
| **7** (2) | **Superquadrática** · **Superfórmula** | um knob que morfa uma família inteira; é onde a fórmula humilha o desenho |
| ⏳ | Bezier · Parabola · Circle Wave · nó de toro · rosca | valem, e não são de primeira mão |

⚠️ **Nada disto é uma promessa de calendário** — é a fila que a medição e o alcance sugerem, e quem
a ordena é o dono do produto.

### §7.6 — ⭐ O PLACAR, contado das três tabelas acima (06/09)

⚠️ **Conte-o daqui, nunca de memória** — esta lista já esteve inflada em quatro (§7.3) e este placar
é a soma das três secções, com os riscados fora:

| de onde | ainda faltam | quais |
|---|---:|---|
| §7.2 — catálogo **3D** | **3** | Plane · Death Star · Vesica Segment |
| §7.3 — catálogo **2D** | **5** | Polygon(N) · Triangle · Bezier · Parabola · Circle Wave |
| §7.4 — **famílias** fora de catálogo | **3** | Superfórmula · Nó de toro · Rosca |
| **total** | **11** | |

⚠️ **Actualizado em 06/09**: a **Superquadrática** saiu da conta (shipou na W127 — [doc 06 §128](06_resultados_cena_e_gizmo.md)).

⛔ **Fora desta conta, de propósito:** as **duas** que são modificadores e não formas (grade
hexagonal, metabolas) e os **fractais** (§7.4), que são wave com medição própria. ⚠️ E o **Plane**
não é uma forma a construir — é a bola de recorte a admitir uma peça **infinita**, que é maquinaria.

⚠️ **O que já shipou não se conta aqui, conta-se no CÓDIGO:** o catálogo tem hoje **61** entradas
(`grep -c 'key: "panel.model3d.add' shells/desktop/src/field3d_shapes.rs`) sobre **52** primitivas
(`PrimitiveKind::ALL`). *Um número escrito num doc é o que envelhece primeiro.*
