# Flip W4 — Fill (o balde): o doc definitivo

> **Estado: FECHADA em 2026-07-12** (pendente o smoke do Enio). Clean-room do *pixel solver* do
> Grease Pencil 5.2 ([`02 §6`](02_referencia_algoritmos_blender_5.2.md)), com os quatro upgrades
> que a [`04 §3`](04_alem_do_blender.md) mandou adotar.
>
> Código: `crates/ph2d-flip-fill/` (o solver, CPU puro) · `shells/desktop/src/flip_fill.rs` (a
> fronteira modelo↔solver) · `crates/ph2d-flip-render/src/fill_holes.rs` (a triangulação).

---

## §1 — A promessa: o resultado é GEOMETRIA

Um preenchimento **é um traço**. Não é uma camada de pixels, não é um objeto de outra classe:
é um `FlipStroke` fechado, com `hide_stroke` (o contorno dele não é rasterizado) e `fill`
(a cor). Consequência direta, e é o motivo de o GP fazer assim:

- **selecionar / mover / apagar** um fill = as mesmas ops de qualquer traço;
- **animar** um fill = ele entra no `tween` como qualquer traço (pareamento por índice);
- **undo** = a fila global, sem nada especial.

Nenhum sistema paralelo. Zero código novo para as cinco coisas acima.

**Os buracos vivem NO traço** (`FlipStroke.holes: Vec<Vec<Vec2>>`), não em traços irmãos
agrupados por um `fill_id` (que é como o GP faz). A razão: um fill é **uma** unidade de seleção,
de undo, de delete e de animação — e um grupo em que apagar um anel destrói a forma não é isso.
A letra "O" é o caso trivial que exige a decisão.

---

## §2 — O pipeline (cada etapa num módulo, cada uma testável sozinha)

```
linhas + clique
  → gap::closures          fecha os vãos (pontas + quinas), cortando na colisão
  → raster::Grid           rasteriza as fronteiras a MEIA espessura (radius_scale 0.5)
  → raster::flood          span fill 4-conexo + filtro de vazamento CRUZADO
  → raster::grow           Grow/Shrink (mata o halo do AA; a cor entra por baixo da linha)
  → trace::trace_contours  marching squares — os buracos saem de graça
  → trace::simplify_ring   binomial leve + RDP  (NESSA ORDEM — §5)
  = FillResult { outer, holes, closures }
```

### §2.1 — `radius_scale = 0.5`: a linha mais importante do subsistema

As fronteiras são rasterizadas com **metade** da espessura visual da linha.

| Raio usado | O que acontece |
|---|---|
| espessura CHEIA | o contorno traçado fica na borda EXTERNA da linha → o preenchimento nasce com um **halo** claro entre ele e o traço |
| raio ZERO | o preenchimento **vaza** pelas frestas do anti-aliasing |
| **METADE** | o contorno cai DENTRO do corpo da linha → **a cor entra por baixo dela** |

É o mesmo insight do *"fill up to vector paths"* do Clip Studio. Somado ao **Grow +2px** default,
o preenchimento se enfia sob o line-art e não sobra fio claro em canto nenhum.

### §2.2 — O filtro de vazamento CRUZADO

Ao expandir na **vertical**, o que bloqueia é fronteira na **horizontal** (e vice-versa), até 3px.
Isso fecha as frestas diagonais de um pixel por onde o flood escaparia. **Inverter a semântica faz
o filtro AJUDAR o vazamento** — é o erro clássico ao portar isto, e há um teste que o pega
(`the_crosswise_leak_filter_closes_a_diagonal_seam`).

### §2.3 — Vazou? Então DIGA

Se o preenchimento toca a borda da grade, o solver **recusa** (`FillError::Leaked`) em vez de
pintar o documento inteiro. É o "No fill created" do GP — e a UI vira isso num toast que aponta
para a solução: *"Fill leaked — raise Gap Closure to seal the outline"*. Um balde que não faz nada
em silêncio parece quebrado; um que explica parece inteligente.

---

## §3 — Gap Closure, e o twist do Harmony

Line-art à mão quase nunca fecha. Duas fontes de extensão (as duas do GP):

1. **Pontas** — cada extremidade de um traço aberto se prolonga na tangente.
2. **Quinas apertadas** — onde a virada passa de 120°, a **bissetriz externa** (a direção do
   "bico") vira um raio. É por isso que o GP fecha cantos em "V" que outros baldes não fecham: num
   "V" as pernas se cruzam *visualmente*, mas o vértice fica fora da região.

Cada extensão é **cortada na 1ª colisão**, e uma que não colide é **descartada** — uma linha solta
atravessando o desenho faria mais mal que o vazamento.

### O twist (adotado da `04 §3`): o fechamento é PERSISTENTE

Um fechamento que funcionou vira um **traço invisível** no desenho (`hide_stroke`, sem `fill`), não
um estado efêmero da ferramenta. Consequência prática enorme:

- re-preencher com outra cor **não precisa do Gap Closure ligado** — o vão já está fechado;
- preencher o quadro vizinho, ou reabrir o arquivo amanhã, também não;
- o fechamento é um traço: dá para apagá-lo com a borracha se estiver errado.

Gate: `gap_closure_leaves_a_persistent_invisible_stroke` — depois de fechar com o Gap Closure, um
2º fill com `gap = 0` funciona.

**Fill vs. fechamento — os dois são `hide_stroke`, e o solver PRECISA distingui-los:**

| | `hide_stroke` | `fill` | É fronteira do próximo balde? |
|---|---|---|---|
| preenchimento | sim | **Some** | **NÃO** (senão a 2ª cor nunca entraria por baixo da 1ª) |
| fechamento de gap | sim | **None** | **SIM** (é para isso que existe) |

---

## §4 — Semântica de balde de ANIMAÇÃO (Toon Boom)

| Modo | O que faz | Para quê |
|---|---|---|
| **Paint** | preenche, entrando atrás do line-art e **na frente** dos fills que já existem | o balde de sempre |
| **Paint Behind** | entra atrás de **tudo**, inclusive dos fills antigos | colorir o que ainda não foi colorido, sem tocar no que já está |
| **Unpaint** | remove o fill sob o clique (o de cima primeiro) | corrigir |

Mais **Grow/Shrink** (o "Area Scaling" do CSP: +2px default enfia a cor sob a linha e mata o halo)
e **Precision** (resolução do buffer).

---

## §5 — Os dois bugs que os testes pegaram (e a lição de cada um)

**A bissetriz apontava para dentro.** O raio da quina saía por `d1 - d0` — que é a bissetriz
*interna*, apontando para DENTRO da cunha, onde ele colide com a própria linha e não fecha vão
nenhum. O bico é `d0 - d1`. *Quando um vetor "quase funciona", desenhe o caso concreto: um "V" de
duas pernas resolve em dez segundos o que uma hora de álgebra não resolve.*

**O alisamento rodava DEPOIS do RDP.** Alisar um anel já reduzido às quinas opera sobre pontos que
estão a 20 unidades um do outro — e a média binomial de um canto de quadrado com esses vizinhos
puxa o canto para o meio. **Um quadrado de área 400 virava um losango de área 26.** Com o anel
DENSO (um ponto por pixel), a mesma média move cada ponto por uma fração de pixel — que é
exatamente o serrilhado que se quer tirar. *Filtre o denso, ajuste o filtrado* — a mesma lição do
pré-filtro do record (`§17.1` da timeline). Gate:
`smoothing_a_dense_ring_preserves_the_shape`.

---

## §6 — Divergências declaradas do GP (e por quê)

| GP | PH2D | Razão |
|---|---|---|
| solver abusa do canal **R** da textura | buffer de **flags** dedicado (`Vec<u8>`) | é o TODO do próprio Blender; cada bit diz uma coisa só |
| pós-processo = smooth 20× + decimação `2^n` | **RDP** (ε≈1.25px) + binomial leve | o smooth 20× deixa polilinhas densas e moles |
| a `04 §3` mandava **fit Schneider** | **não fizemos** | o `FlipStroke` é uma **polilinha** (sem handles): uma Bézier seria re-achatada no instante seguinte, e o único efeito seria perder precisão na ida e volta. Se o traço ganhar handles, o fit entra no `trace.rs`, num lugar só |
| ear-clipping + `fill_id` para furos | **decomposição trapezoidal even-odd** | ear-clipping com pontes trava: uma ponte cria vértices coincidentes que fazem o teste "nenhum vértice dentro da orelha" rejeitar orelhas legítimas, e o fill sai com um pedaço faltando em 1 desenho a cada 20 |
| solver Delaunay (5.2, alternativo) | **não portado** | é o v2 (`02 §6`); o pixel solver é o robusto, e é o que tem overlay/UX |

**GPU: não.** O fill é uma operação de **clique**, não de frame. O span fill é ~10× o BFS por pixel
e roda em poucos ms num buffer de milhões de pixels. A GPU seria o primitivo errado — o JFA **salta
paredes** (não é geodésico), e o readback para vetorizar o contorno seria inevitável de qualquer
jeito (`04 §3`).

---

## §7 — O que depende do ZOOM (e por que isso é honesto)

A espessura do brush do Flip é em **px de TELA** (absoluta — Enio 2026-07-11), e o solver trabalha
no espaço do documento. Então a meia-espessura em unidades do documento **depende do zoom**: com o
zoom afastado, as linhas ficam relativamente mais grossas e os vãos fecham sozinhos.

Isso não é um bug — é a mesma dependência que o GP tem (lá o solver também é em espaço de tela), e
é o que um artista espera: "eu vejo o vão fechado, então ele preenche". O **Precision** multiplica a
resolução do buffer por cima disso.

---

## §8 — Carry-overs declarados (não são omissões)

- **T4.5 — Fill multiframe** (rodar o balde em N quadros selecionados de uma vez): precisa da
  multi-seleção de chaves na tira, que é carry-over da W3. O solver já é por-desenho: é wiring.
- **Ajuste modal ao vivo do Gap Closure** (scroll com os helpers visuais nos vãos pendentes — a
  killer feature de UX do GP): hoje o Gap é um slider. O `closures()` já devolve os segmentos, então
  o overlay é desenhar o que ele devolve.
- **Modo Radius do Gap Closure** (círculos-guia nas pontas, linha centro-a-centro): o Extend cobre a
  maioria dos casos; o Radius fecha os que o Extend não vê.
- **Colorize** (LazyBrush / trapped-ball — o "colorir tudo" e o *onion fill*): é wave própria
  (`04 §3`), a feature que só o TVPaint entrega.
