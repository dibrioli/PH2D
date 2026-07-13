# ADR-0119 — Live Corners: o documento guarda a quina AFIADA; o mundo consome a COZIDA

- **Status:** aceito (implementado, pendente smoke do Enio)
- **Data:** 2026-07-13
- **Linha:** `line/Vector`
- **Supersede:** nada. **Emenda:** ADR-0108 (o motor novo `ph2d-vec-*` ganha um conceito).

## Contexto

O Enio pediu a **alça de raio na quina** (o *Live Corners* do Illustrator / o *Corner
Tool* do Affinity): arrastar uma alça no canto e vê-lo arredondar, de forma
**não-destrutiva** — o raio continua editável depois, e voltar a zero devolve a quina.

A pesquisa anterior ([`20_pesquisa_ferramentas_de_artista.md`](../../Vector%20Module/20_pesquisa_ferramentas_de_artista.md))
classificou isto como **pequeno**: *"já temos o motor (`corners.rs`); falta a alça"*.

**Essa estimativa estava errada, e o erro é instrutivo.** O `corners.rs` recebe uma
**polilinha de pontos** (`&[[f64; 2]]`) e **produz** uma forma: é o gerador do polígono, da
estrela e do round-rect. Ele não é um operador sobre um caminho **desenhado** — não sabe o
que fazer com uma quina entre duas **curvas**, e não tem onde guardar um raio.

E aí aparece o problema de verdade, que não é geometria: **um caminho desenhado não tem
slot de receita.** Uma Live Shape guarda parâmetros no `VecShape::Param` e a geometria dela
é derivada (`recook_into` reescreve `verts`). Num caminho desenhado, os `verts` **são** a
fonte — é o que a caneta edita. Um raio não-destrutivo exige, portanto, **duas
representações**: o que o documento guarda e o que o mundo consome.

Isso é exatamente o `inkscape:original-d` + `d` — e é a espinha dos *Live Path Effects*, o
item #1 da mesma pesquisa. A "alça pequena" era a espinha, em miniatura.

## Decisão

### 1. O raio mora **dentro do vértice**

`VecVertex.corner_radius: f64` (apendado por último — postcard é posicional;
`VEC_SCENE_SCHEMA_VERSION` 7→8 e `PROJECT_SCHEMA` 7→8).

**E não num vetor paralelo ao lado dos `verts`.** Cerca de 60 sítios inserem, apagam,
invertem, soldam e transformam vértices (split, delete, reverse, weld, booleana, merge).
Cada um deles teria de lembrar de mexer no vetor paralelo *também*, e o compilador ficaria
**calado** sobre os que esquecessem. Dentro do vértice, o raio **viaja junto** — de graça,
e dessincronizar é impossível. É o mesmo movimento de "matar a classe" que o `NUMBER_FIELDS`
fez no painel.

### 2. A fonte é `verts`; o consumo é `VecPath::cooked()`

O documento guarda a **quina afiada + o raio**. Quem renderiza, aponta, enquadra e corta lê
`cooked()` — a geometria com as quinas já viradas arco.

**Sem raio nenhum, `cooked()` devolve a própria fonte emprestada** (`Cow::Borrowed`): mesmo
ponteiro, zero alocação, zero aritmética. Como todo path nasce sem raio, ligar o cozimento
em **todo** consumidor de geometria do módulo não mudou uma vírgula do comportamento de
hoje — e é o que tornou a mudança segura.

Quem **edita** continua vendo a fonte: a caneta, a seleção, e o overlay de âncoras. Senão o
usuário veria **dois** vértices onde autorou **um**.

O funil existia e ninguém tinha reparado: 6 dos 8 leitores de geometria já entravam por
`VecPath::contour(c)`. Trocar o miolo de ~8 funções cobriu ~42 call sites.

### 3. Uma **forma viva** não tem raio por-vértice

`recook_into` reescreve `verts` INTEIRO a cada mudança de parâmetro; um raio gravado ali
sobreviveria até o próximo arrasto de slider e sumiria **sem erro**. E não há conserto: a
**contagem** de vértices é função dos parâmetros (o slider de lados de um polígono muda
quantas quinas existem) — não há para onde levar o raio da quina que deixou de existir.

O raio de uma forma viva é um **campo dela** (o `Radius` do painel — por-canto, no
round-rect). O por-vértice é para caminho **desenhado**. É a divisão do Illustrator: uma
live shape tem as propriedades dela; *Live Corners* é o que se ganha depois de **expandir**
— e aqui "expandir" é o **Convert to Curves**, que já existe (descartar o `VecShape`).

### 4. A booleana **assa** as quinas

Ela consome o cozido, e os vértices que saem nascem com raio 0. Uma booleana é destrutiva
por natureza: o vértice que o raio arredondava pode nem existir depois do corte. É o que a
Figma faz, e é o único contrato honesto.

## A construção geométrica (e por que a identidade se preserva)

Numa quina entre duas retas, o `corners.rs` recua `t = r / tan(θ/2)` nas duas arestas e liga
os pontos por um arco de handle `(4/3)·tan(α/4)·r`. O `corner_live.rs` faz **a mesma conta**,
com duas generalizações e só duas:

1. **A direção da aresta** vira a **tangente da curva** na quina (com a cascata de handle
   degenerado — um handle nulo cai no controle anterior, como no `cubic_bezier.cc` do
   Chromium). Numa reta, a tangente **é** a aresta.
2. **O recuo** vira uma distância **ao longo da curva** (bisseção no parâmetro). Numa reta, o
   parâmetro é proporcional à distância.

O handle da cúbica de ligação é `k · d`, com `k = (4/3)·tan(α/4)` e `d` = a distância até a
**interseção das duas tangentes**. Numa quina reta, a interseção **é o vértice original** e
`d = t` — então `k·d` reduz **exatamente** ao `h` de sempre. Um gate compara os dois motores
byte a byte.

Consequência: o caso reto continua sendo o **arco circular exato**; uma quina com um lado
curvo ganha um blend **tangente (G1)**. É o que Illustrator e Affinity de fato entregam — o
filete circular exato entre duas cúbicas é uma curva de **grau 10**, e ninguém a resolve em
forma fechada.

## Consequências

**Boas:**
- O raio é **vivo**: editável depois, reversível, e sobrevive a mover/escalar/rodar (ele é um
  comprimento local, e escala com a geometria — pelo mesmo helper que já escalava o raio do
  gradiente radial; os dois moram na mesma função agora, para que ninguém escale um e esqueça
  o outro).
- A distinção **fonte ≠ cozido** agora existe no modelo. É o pré-requisito dos *Live Path
  Effects* (item #1 da pesquisa) — o próximo efeito não-destrutivo herda a costura.

**O preço:**
- Dois `SCHEMA_VERSION` bumparam. Saves antigos são rejeitados (é a política do módulo desde
  o cutover: migração robusta = cutover).
- `cooked()` aloca um `VecPath` por chamada **num path que tem raio**. Hoje isso é
  micro-custo (uma bisseção de 60 iterações por quina, e as chamadas por frame são dezenas),
  e **não foi otimizado de propósito**: a lição do roteador de conectores (1–6 µs medidos,
  otimização desnecessária) é que se mede antes. Se um dia doer, o caminho é um cache
  invalidado na edição — não é um redesenho.

## Alternativas consideradas

- **Vetor paralelo `corner_radii` no `VecPath`** — rejeitado: gerador de bug de
  dessincronização em ~60 sítios de mutação, com o compilador calado.
- **Assar o arredondamento nos `verts` ao arrastar** (destrutivo) — rejeitado: contradiz o
  idioma do módulo (Live Shape, conector vivo, rótulo vivo) e torna "voltar a afiar"
  impossível. *"Difícil de ajustar" é um bug de DESIGN*, não de calibragem.
- **Guardar a fonte num componente ECS** (espelhando o `VecShape`) — rejeitado: a caneta
  edita `path.verts`, então a fonte **tem** de ser `path.verts`; guardar um path inteiro
  dentro de um componente duplicaria o documento.
