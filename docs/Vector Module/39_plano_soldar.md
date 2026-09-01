# Plano 39 — **SOLDAR** (a rede que o balde precisa)

> Ideia do Enio (2026-08-31): *"e se pudéssemos soldar linhas cruzadas? Ou seja: linhas cruzadas
> compartilham o mesmo nó de modo que criem várias áreas fechadas interligadas?"*
> Decisão dele, no mesmo dia: **soldar CONSOME os traços originais.**

## §1 — A pesquisa

| onde | o modelo | o preço |
|---|---|---|
| **Figma** (*vector networks*, 2016) | o objecto é um **multigrafo não-dirigido com identidade de aresta**: um nó tem 3+ segmentos e sobrevive à edição | refazer o tipo *caminho*. Eles contam *"becos sem saída"* e que quase desistiram |
| **CAD** (Fusion, SolidWorks) | um esboço **já é** uma rede; as regiões fechadas (*profiles*) formam-se sozinhas | é por isso que o Trim de lá parece natural |
| **Illustrator** (*Pathfinder > Divide*) | corta tudo nos cruzamentos, uma vez | ⛔ **perde as partes de caminhos abertos que ficam de fora** — a queixa documentada |

⭐ **A nossa é a metade barata da Figma com o defeito do Illustrator curado.**

## §2 — A lei

> **Cada contorno parte-se em ARCOS nos pontos onde encontra os outros**, e as pontas dos arcos
> vizinhos caem exactamente no mesmo sítio — porque saem do mesmo cruzamento.

⭐ **O grafo não é uma estrutura de dados: é implícito nas coordenadas coincidentes.** Nenhum tipo
novo, nenhum contrato mexido.

| o contorno | os cruzamentos | o que sai |
|---|---|---|
| qualquer | nenhum | ele próprio, **intacto** (os mesmos vértices, não uma reconstrução) |
| aberto | `n` | `n + 1` arcos |
| fechado | `n` | `n` arcos abertos (o último dá a volta pela emenda) |
| fechado | `1` | **um** arco aberto — um anel cortado num ponto, não um degenerado |

⛔ **E não é automático.** Se cruzar duas linhas as colasse sozinho, seria impossível apenas
**sobrepor** dois traços. É um verbo sobre a selecção (o botão **Weld**, colado no *Join*: aquele
solda duas pontas, este solda os cruzamentos).

⭐⭐⭐ **E o nó SOBREVIVE ao dedo** — report do Enio, com foto: *"weld dividiu e não soldou (eu que
afastei os pontos)"*. Ele estava certo, e a falta eram **duas** coisas:

1. **Cortar não é soldar.** As duas metades de um cruzamento nascem de contornos DIFERENTES: cada um
   converte a mesma travessia para a SUA fracção e avalia a SUA cúbica ali — os pontos ficam perto e
   **não iguais**. *Dois pontos perto não são um nó, são dois nós.* ⇒ `weld::fuse_endpoints` funde
   cada aglomerado numa coordenada só (o **centroide**, para a solda não depender da ordem da
   selecção), e a alça acompanha a âncora.
   ⚠️ **A folga é DUAS vezes a flecha da amostragem**, não uma: cada lado erra a sua, em direcções
   opostas. Medido em dois círculos de raio 100 — as pontas a `0,1376` com flecha de `0,12`, e com
   uma flecha só a solda **não pegava**.
2. **Arrastar tem de levar todos.** `PenTool::welded_with` responde *"quem mais partilha esta
   ponta"* (comparação **exacta**, `WELD_TOL`), e o arrasto de âncora move a união *selecção ∪
   juntas*. É a lei do esboço de CAD: **duas pontas no mesmo sítio são UM nó**.

⛔ **O que a solda ainda NÃO promete:** *acrescentar* uma linha nova ao nó depois não a solda — a
junta é a coincidência, e quem a cria é o Weld, o Join ou o encaixe. E o modelo completo da Figma (a
aresta com identidade, que sobrevive a qualquer edição) continua fora.

## §3 — O que reusou

Tudo o que o **Trim** (plano 38) pagou: `trim_tool::crossings_against` (cruzamentos + **toques**),
`trim_tool::piece_geometry` (o arco entre duas fracções), `arc_cut::Geom`. O módulo novo
([`weld.rs`](../../crates/ph2d-vec-scene/src/weld.rs)) tem **um** algoritmo: as fronteiras ordenadas
e um arco entre cada par consecutivo.

⚠️ **A pose entra na conta** (`vec_weld.rs`): dois traços só se cruzam depois de o `Transform` os
pôr no lugar, e cada operando é assado no MUNDO — a mesma convenção do `apply_vec_boolean`.

## §4 — ⛔⛔ O DEFEITO DO TRIM que este plano encontrou

O gate de soldar reprovou com *"dois cortes num anel dão DOIS arcos: left 1, right 2"* — e a causa
estava **no Trim**, três horas mais velha:

- o corte vai por `strands_of`, que **normaliza** uma faixa que atravessa a costura de um anel;
- o realce ia por `piece_geometry`, que **não** normalizava ⇒ um pedaço que passa pela emenda
  **não acendia** e ainda assim era comido.

⚠️ **É exactamente a divergência «acende uma coisa e apaga outra» que o desenho do Trim proíbe**, e
o gate de «a mesma porta» **não a apanhou**: as duas portas só discordam sobre a emenda, e nenhuma
fixtura tinha um pedaço que passasse por lá. *Uma porta única prova que a resposta é a mesma; ela
não escolhe as perguntas que se fazem.* Dois gates novos no `trim_tool`.

## §5 — ⏳ Aberto

- ⏳ **O balde** — é o consumidor desta rede, e a razão de ela existir.
- ⏳ **Uma linha DESENHADA depois não entra no nó** — a junta é a coincidência, e desenhar não a
  cria (o encaixe cria, o Join cria, o Weld cria). Soldar outra vez resolve.
- ⏳ **Um composto soldado perde o buraco**: depois da solda os contornos são arcos, e um arco não
  tem dentro. É o preço declarado de *"consome os originais"*.
- ⏳ **Custo não medido** — `O(contornos²)` em arestas de amostragem, sobre a selecção. É um verbo
  de clique, não um laço de quadro; sem número, a acusação seria palpite.
