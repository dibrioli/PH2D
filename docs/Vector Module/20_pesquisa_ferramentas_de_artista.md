# Pesquisa — o que falta para o Vector ser extraordinário para artistas

> 2026-07-12. Fontes primárias (código-fonte do Inkscape, fonte do flubber, docs oficiais de
> Corel/Adobe/Rive/Cavalry, papers). O que **não** foi confirmado por fonte está marcado.

## A tese, antes da lista

A pesquisa toda converge num ponto que eu não esperava, e que muda a ordem de tudo:

**O Inkscape tem ~50 "Live Path Effects" — efeitos NÃO-DESTRUTIVOS e EMPILHÁVEIS sobre um
caminho. É a coisa mais poderosa que um editor vetorial livre já construiu. E a arquitetura
deles é literalmente um sistema de nós.**

A mecânica (confirmada no `effect.cpp` do Inkscape): o caminho original fica guardado em
`inkscape:original-d`; o resultado do efeito vai para o `d` padrão (então qualquer renderer de
SVG vê a geometria certa); os parâmetros ficam em atributos. Os efeitos **empilham**,
**reordenam por arrastar**, e há um **Flatten** que assa só os N primeiros da pilha.

Isso é um grafo de cozimento. Entrada → operador → operador → geometria. **Nós já temos isso**
(`ph2d-nodegraph` + Motion Nodes, e cada nó é uma drop-crate). Os *path operators* do After
Effects (Repeater, Trim Paths, Zig Zag, Offset, Pucker & Bloat, Twist) são a mesma ideia com
outro nome; os *Behaviours* do Cavalry também ("cada behaviour recebe e modifica a saída do
anterior" — doc oficial).

Ou seja: **a ferramenta mais transformadora que podemos construir não é uma ferramenta — é a
espinha que faz cada ferramenta futura custar uma drop-crate.** Um "Live Effect" no PH2D seria
um nó que come um `VecPath` e cospe outro. Depois disso, cada efeito da lista abaixo é
incremental.

E há um precedente exato no nosso próprio código: a **Live Shape** (a forma paramétrica que
continua editável) e o **conector vivo** já são isso — geometria como função pura de
parâmetros, re-cozida por frame. O padrão se pagou três vezes. Esta é a quarta, e a maior.

---

## 1. Os três que o Enio pediu

### 1.1 Gizmo de raio na quina (Live Corners) — **PEQUENO, faça primeiro**

Já temos o **motor** (`corners.rs`: raio por-canto + squircle G2). Falta o **gizmo**: a alça na
quina que arredonda ao arrastar.

O que os outros fazem:
- **Affinity Corner Tool**: tipos de canto Round / Concave / Chamfer / Cutout.
- **Illustrator Live Corners**: uma alça por canto; arrasto proporcional.
- **Inkscape LPE "Corners" (Fillet/Chamfer)**: virou alças triangulares arrastáveis na 1.3.

**Custo:** pequeno. É UI sobre um motor que existe. **Retorno:** alto — é a queixa do Enio, e é
o tipo de detalhe que faz um app *parecer* bom.

**Gap real:** hoje só arredondamos quina entre RETAS. Arredondar entre duas CURVAS é outro
problema (o filete tem de ser tangente a duas cúbicas, não a duas retas).

### 1.2 Texto em caminho — **PEQUENO/MÉDIO, alto retorno**

**A armadilha, e ela é a coisa toda:** o parâmetro `t` de uma Bézier **não é proporcional ao
comprimento de arco**. Espaçar letras por `t` aglomera as letras nas curvas e as espalha nas
retas. Quem não sabe disso escreve a versão errada primeiro, e ela *parece* certa numa reta.

A cura é padrão e conhecida: tabela de comprimento de arco (Gauss-Legendre por segmento) +
inversão por Newton (Peterson, *Arc Length Parameterization of Spline Curves*). **`kurbo` já
tem `arclen` e `inv_arclen`** — o trabalho pesado está feito.

Segunda armadilha: nas curvas apertadas as letras **colidem no lado interno** (o raio de
curvatura é menor que a altura da letra). Ninguém resolve isso perfeitamente; os bons detectam
e avisam, ou empurram a linha de base.

### 1.3 Interpolação de formas (Blend) — **MÉDIO, e o mercado inteiro entrega menos do que promete**

O problema difícil **não é interpolar** — é a **correspondência** entre formas de topologia
diferente. E o achado da pesquisa é libertador:

| Quem | Como resolve a correspondência |
|---|---|
| **flubber** (a lib mais usada da web, **aberta**) | Reamostra por comprimento de arco; acolchoa a forma menor até igualar a contagem; **força bruta O(n²)** sobre todos os deslocamentos, escolhendo o que **minimiza a distância total percorrida**. O autor: *"uma heurística cuja justificativa é que geralmente funciona bem"* |
| **GSAP MorphSVG** (fechado, grátis desde abr/2025) | Três heurísticas nomeadas (`size`/`position`/`complexity`) + **`shapeIndex` MANUAL** + uma ferramenta de debug (`findShapeIndex()`) cuja existência **admite que o automático erra** |
| **CorelDRAW Blend** | Comando **"Map Nodes"**: o usuário **clica um nó em cada forma** para fixar a correspondência à mão |
| **Lottie / After Effects** | **Nenhuma.** Lerp por ÍNDICE, exige a mesma contagem de pontos. Bug aberto desde 2015, ainda aberto em 2024 |
| **Rive** | **Nenhuma.** Lista de vértices de tamanho fixo, keyframada uma a uma |

**Conclusão:** ninguém resolveu isso. O alvo honesto é **bom automático + escape manual óbvio** —
e o escape manual (o "Map Nodes" do Corel) é o que separa uma ferramenta usável de uma
frustrante. Implementar o flubber (arc-length + padding + força bruta) é **um dia de trabalho**,
e nos põe no nível do estado da arte prático.

**O que faz o blend parecer rígido em vez de derreter:** a literatura é clara e vale ler antes —
Sederberg & Greenwood 1992 (*A Physically Based Approach to 2-D Shape Blending*, o trabalho de
referência: modela uma forma como arame dobrado e acha a deformação de **trabalho mínimo**) e
Alexa et al. 2000 (*As-Rigid-As-Possible Shape Interpolation*). O lerp ingênuo de coordenadas
encolhe a forma e a auto-intersecta — é por isso que o GSAP tem um modo "rotational".

---

## 2. O que a pesquisa achou que eu NÃO esperava

### 2.1 Power Stroke / largura variável — **o buraco mais gritante do nosso traço**

Hoje o traço tem uma largura só. Um artista quer **largura variável ao longo do caminho** — é o
que separa um desenho de um diagrama.

- **Inkscape Power Stroke**: alças arrastáveis na linha; guarda `(posição, largura)` pares. O
  **Width Tool** (tecla W) é só o front-end disso.
- **Illustrator Width Tool**: o mesmo, com perfis salvos.

**O aviso técnico, e ele é sério:** o offset exato de uma cúbica de Bézier é uma curva
**analítica de grau 10** — não fechada em Bézier. Todo mundo aproxima e subdivide.

- **`kurbo` NÃO tem largura variável.** Só `Stroke { width: f64 }` — um escalar. O time da
  Linebender **declarou publicamente** (blog, mar/2025) que pretende explorar largura variável,
  *"uma feature frequentemente pedida"*. Não existe hoje.
- **Skia também não** — só um protótipo em `samplecode/`, fora do stroker de produção.

Ou seja: **não há de prateleira.** Teríamos de construir. Mas o Raph Levien (o autor do kurbo)
publicou o caminho: *GPU-friendly Stroke Expansion* (2024, com Arman Uguray) e a série sobre
espirais de Euler — a parallel curve de uma espiral de Euler tem forma fechada, e é isso que
elimina as cúspides que o offset ingênuo cria.

### 2.2 Cavalry "Falloff" — **a ideia mais exportável da pesquisa inteira**

Um nó que produz um **campo escalar espacial**: 1.0 no centro, 0.0 na borda, com uma **curva de
resposta editável**. Cinco formas (círculo, retângulo, linear, sweep, **forma arbitrária**).

E aqui está o gênio: ele **não faz nada sozinho**. Ele pluga na entrada "força" de *qualquer*
outro nó — deformadores, duplicadores, físicas. Ele desacopla **"onde há influência"** de **"o
que a influência modula"**.

Isso é exatamente o tipo de primitiva que um sistema de nós quer, e é barata. Nós já temos
falloffs no Painter (o pincel) — mas como parâmetro enterrado, não como nó de primeira classe.

### 2.3 CorelDRAW — quatro coisas que o Illustrator não tem

A própria Corel publica uma tabela "Comparando ferramentas" para migrantes do Illustrator. Ela
**não lista equivalente** para: **Contour**, **Extrude**, **Impact** e **Add Perspective**. É a
Corel dizendo, com a própria boca, onde ela é única.

- **Contour**: N offsets concêntricos com **progressão de cor** e **aceleração** (a taxa de
  mudança não é linear). Não é o nosso "offset" — é offset × N + cor.
- **Color Harmonies**: os hues ficam **ligados numa roda**; girar a roda desloca todos
  **preservando o espaçamento relativo**. Não achei equivalente em Illustrator, Figma nem
  Affinity. Nós temos OKLCH — isso encaixaria lindamente.
- **Block Shadow**: sombra sólida vetorial (não borrada). No Illustrator só se faz com Blend
  (que gera centenas de objetos e incha o arquivo).
- **PowerClip**: o clipping deles é um **objeto de verdade** (frame + conteúdo, ambos
  selecionáveis, aninháveis, com trava de movimento opcional), não a convenção "o objeto de cima
  vira máscara" do Illustrator/Figma.

### 2.4 Figma Vector Networks — **avaliei e recomendo NÃO fazer agora**

O modelo de grafo (um vértice pode ter 3+ arestas) é uma vantagem topológica **real** — junções
em T e arestas compartilhadas são impossíveis num caminho sequencial.

Mas o custo é alto e a pesquisa achou dois problemas que **nem a Figma resolveu**:

1. **A continuidade de tangente quebra num vértice de 3+ arestas.** O clássico
   "suave/quina/quebrado" deixa de fazer sentido — a que par de arestas a suavidade se aplica?
2. O preenchimento exige detectar auto-interseções (duas cúbicas se cruzam em até **9** pontos),
   expandir o grafo e achar a **base mínima de ciclos**. Bem mais caro que o preenchimento de um
   caminho.

E o veredito do melhor artigo técnico sobre o assunto (Alex Harri):
> *"Vector Networks não permitem criar algo que você não poderia. Elas habilitam workflows que
> antes não eram possíveis."*

Além disso, o comentário mais votado do HN sobre o tema afirma que, na prática, **as pessoas não
usam** — a Figma nunca pareou o modelo com uma caneta melhor.

**Recomendação: não.** É um refactor de fundação, com problema não-resolvido embutido, por um
ganho de workflow. Nosso modelo (âncora + 2 handles, estilo Rive) está certo para desenho.

### 2.5 Caneta sem handles (Spiro / Hobby) — **o diferencial silencioso**

Não confirmei com fonte primária nesta rodada (o agente morreu), mas o Inkscape tem **Spiro
spline** e **BSpline** como LPEs, e o `effect.cpp` os lista. A ideia: uma caneta em que você
põe **pontos**, e a curva "mais bonita" passa por eles — sem tocar num handle. É o algoritmo do
MetaPost (Hobby) e as curvas de curvatura contínua do Levien (espirais de Euler).

Para um artista que não é engenheiro, **isto pode ser mais transformador que qualquer efeito da
lista** — porque ataca a barreira de entrada da caneta Bézier, que é o que faz gente desistir de
vetor.

---

## 3. A lista completa, ordenada por RETORNO ÷ ESFORÇO

### Faixa A — faça (retorno alto, esforço baixo/médio)

| # | Ferramenta | Esforço | Por quê |
|---|---|---|---|
| 1 | **Live Effects como NÓS** (a espinha) | Médio | Faz todos os itens abaixo custarem uma drop-crate cada. É o multiplicador |
| 2 | **Gizmo de raio na quina** | Pequeno | O motor existe; falta a alça. É o pedido do Enio |
| 3 | **Texto em caminho** | Pequeno/Médio | `kurbo` já tem `inv_arclen`. Retorno enorme por linha de código |
| 4 | **Trim Path** (revelar o traço) | Pequeno | Arc-length + 2 escalares. É o "draw-on" de motion design, e nós **temos timeline** |
| 5 | **Repeater / Duplicator** (grade, radial, ao longo de caminho) | Pequeno/Médio | Um nó. Multiplica o que o artista desenha |
| 6 | **Interpolação de formas (Blend)** | Médio | Copiar o flubber (arc-length + força bruta) + o "Map Nodes" do Corel como escape |
| 7 | **Largura variável (Power Stroke)** | Médio/Grande | O buraco mais gritante. Não há de prateleira — mas há o caminho do Levien |

### Faixa B — vale, depois da espinha

| # | Ferramenta | Esforço |
|---|---|---|
| 8 | **Zig Zag · Roughen · Wiggle · Pucker/Bloat · Twist** (os path operators do AE) | Pequeno **cada**, se a espinha existir |
| 9 | **Contour** (offsets concêntricos + progressão de cor) | Pequeno (temos offset) |
| 10 | **Falloff como nó** (a ideia do Cavalry) | Pequeno, e composta com tudo |
| 11 | **Pattern Along Path** (arte repetida ao longo do caminho) | Médio |
| 12 | **Sketch / Hatches** (traço à mão, hachura) | Médio — e casa com o Flip |
| 13 | **Envelope / Warp** (deformar por 4 lados, ou por forma) | Médio/Grande — ver a armadilha abaixo |
| 14 | **Color Harmonies** (a roda de hues ligados) | Pequeno — e temos OKLCH |
| 15 | **Knot** (cruzamentos "por baixo", nós celtas) | Médio, e é lindo |

### Faixa C — não agora

- **Vector Networks** (§2.4): refactor de fundação, problema não-resolvido, ganho de workflow.
- **Gradient Mesh**: o Illustrator e o Corel têm; é grande, e o nosso gradiente multi-ponto (IDW)
  já cobre boa parte do caso de uso.
- **Rig + skinning**: já está deferido para o fim, e continua certo.

---

## 4. A armadilha do Warp, que vale saber ANTES

Deformar os **pontos de controle** de uma Bézier por uma função **não-afim** (um warp, um
envelope) **não deforma a curva corretamente** — a curva resultante **não é a imagem** da curva
original. Só transformações **afins** comutam com a avaliação de Bézier.

Os apps lidam com isso **subdividindo até a tolerância** e, às vezes, refitando. É por isso que
um envelope no Illustrator "solta" pontos.

O CorelDRAW é o mais sofisticado aqui, e a doc oficial dá os nomes: **4 modos de restrição**
(Straight Line / Single-Arc / Double-Arc / Unconstrained) × **4 modos de mapeamento**
(Original / Putty / Horizontal / Vertical). O Illustrator não tem nada disso.

Se formos fazer warp, a família de algoritmos vale um estudo próprio (FFD/Sederberg 1986;
MLS/Schaefer 2006; ARAP/Igarashi 2005; Bounded Biharmonic Weights/Jacobson 2011). **Não fazer
isso de improviso.**

---

## 5. Recomendação — o TOP 3

**1. A espinha de Live Effects como nós.** Não é uma ferramenta; é o que faz as próximas vinte
custarem pouco. O Inkscape provou o valor (50 efeitos), o Cavalry provou a arquitetura
(behaviours encadeados), e **nós já temos o sistema de nós**. Fazer as ferramentas uma a uma,
hardcoded, é construir a dívida que o Inkscape não tem.

**2. O trio do Enio, nesta ordem: gizmo de raio → texto em caminho → blend.** Os dois primeiros
são baratos e o motor já existe (cantos, `kurbo::inv_arclen`). O blend é médio e, agora que
sabemos que *ninguém* resolveu a correspondência, podemos entregar o estado da arte prático em
pouco tempo — desde que com escape manual.

**3. Largura variável no traço.** É o buraco que mais separa "editor de diagrama" de "ferramenta
de artista". Não há de prateleira (nem kurbo nem Skia), então é investimento de verdade — mas é
o que faz um traço parecer desenhado em vez de gerado.

**O de menor retorno aparente e maior custo real:** Vector Networks. **O de maior surpresa:** a
caneta sem handles (Spiro/Hobby) — pode valer mais para um artista que metade da lista.
