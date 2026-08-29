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

## §5 — ⭐⭐⭐ A FILA FECHOU (W103, 29/08)

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

| Filete que só é arco a **90°** | `(1 − 1/√2)·r/sin α` contra `r·(1/sin α − 1)`; as duas curas (canto exato · raio compensado) **pioram** a sonda de arestas (doc 06 §102.5) |
| Canto exato dobrado sobre expressão composta | a corda e o disco são globais: `0,0 %` → `1,3 %` de aresta viva no prisma (doc 06 §102.5) |
| Sonda de arestas semeada só por FORA | cega ao vinco **côncavo**: a mutação do vale da estrela sobrevive (`1,4 %` → `1,3 %`, ruído) — doc 06 §102.1 |
