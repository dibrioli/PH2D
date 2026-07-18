# 10 — A REGIÃO POR CURVAS: a malha do fill nasce dos vértices das linhas

> **Wave proposta.** Plano escrito antes de código (regra do handoff). Ainda **não
> aprovada** — o §7 diz o que precisa da ordem do Enio.
>
> **Origem:** smoke de 2026-07-18, com duas fotos lado a lado. *"a malha que o fill cria
> não usa os vertex das linhas para gerar a própria malha. Não se apoia no centro da linha
> onde os vertex estão. Diferente do Draw:Filled que faz exatamente como eu estou
> dizendo."*

---

## §1 — O problema, em uma frase

O balde produz **dois conjuntos de vértices** para uma fronteira só.

A linha tem os dela (o que a mão desenhou, simplificado). O preenchimento tem os dele
(marching squares → RDP → alisamento sobre um raster). Eles descrevem a mesma curva e não
se conhecem: nas quinas o traçado chanfra, nas retas ele desliza.

Tudo o que se fez desde o BUGS #14 foi **administrar a distância entre esses dois
conjuntos** — âncora no eixo (#14), dilatação (#15), auto-preenchimento quando a região é
uma forma só (#16/#17), compensação com sinal e margem zero (#21). Cinco rodadas, todas
corretas, todas tratando o sintoma de haver dois conjuntos.

O **Draw:Filled** não tem o problema, e não porque seja mais bem ajustado: **ele não tem o
segundo conjunto.** O preenchimento é a triangulação dos pontos do próprio traço. Esculpir
a linha move a cor junto, de graça, em qualquer zoom, para sempre.

**Esta wave é sobre estender essa propriedade para a região delimitada por VÁRIAS linhas**
— o caso que hoje cai obrigatoriamente na rota vetorizada.

---

## §2 — Dois becos já percorridos (leia antes de propor qualquer coisa)

### 2.1 — Costurar o contorno traçado às curvas ⛔ TENTADO E DESCARTADO

Projetar cada vértice do contorno no eixo da linha e reinserir os vértices dela. Está no
**BUGS #16**, com o veredito:

> *Funciona em geometria mansa — e **destrói o anel numa quina aguda**: os dois lados do
> bico estão à mesma distância, a projeção alterna entre eles, o contorno vai-e-volta e a
> região vira um nó de área zero. Impor a direção do percurso salvou dois dos três casos, e
> aí ficou claro que a abordagem estava errada: **proximidade não é ordem**.*

⚠️ É a armadilha natural desta wave — parece a solução óbvia e é a primeira coisa que
qualquer um (inclusive eu, em 2026-07-18) propõe. **A ordem do percurso não pode sair de
"que ponto está mais perto".**

### 2.2 — Aumentar a precisão do raster ⛔ NÃO ATACA A CAUSA

Mais resolução aproxima melhor a mesma curva com **outros** vértices. O defeito não é a
magnitude do erro, é a existência de um segundo conjunto de pontos. O BUGS #16 já mediu:
o erro é de **forma**, não de escala, e nenhuma constante o fecha.

---

## §3 — O desenho: a topologia dá a ORDEM, o raster dá a ESCOLHA

A saída dos dois becos é separar duas perguntas que hoje uma única ferramenta responde mal:

| pergunta | quem responde bem | por quê |
|---|---|---|
| **QUAL** região o usuário clicou | o **raster** (o que já existe) | é robusto a arte imperfeita: vãos, traços que não se tocam, tinta rala. É o que o balde do GP faz e funciona. |
| **ONDE** a fronteira dessa região está | um **arranjo planar** das linhas | a ordem do percurso vem da topologia (aresta → próxima aresta em torno do vértice), nunca de distância. |

O raster deixa de ser a fonte da geometria e passa a ser **um localizador**: ele diz em que
face do arranjo o clique caiu. A geometria sai das curvas originais.

### 3.1 — O motor

1. **Interseções** entre os segmentos das polilinhas de line-art (as mesmas
   `boundaries()` que o solver já recebe — inclusive os **fechamentos de gap**, que já são
   traços de verdade no documento, e é isso que faz uma arte com vãos ainda funcionar).
2. **Grafo planar**: cada interseção vira um nó; cada trecho de traço entre duas
   interseções consecutivas vira uma aresta que **carrega os vértices originais** daquele
   trecho. É aqui que a promessa se cumpre — a aresta *é* um pedaço da linha, não uma
   reamostragem dela.
3. **Face**: a partir de um ponto semente dentro da região (dado pelo raster), caminha a
   fronteira pela regra *"na chegada a um nó, siga a aresta seguinte no sentido
   horário/anti-horário"* — o `next` do half-edge clássico. **A ordem é topológica.**
4. O anel resultante é a concatenação dos trechos, com os vértices das linhas intactos e
   pontos NOVOS só nas interseções — e esses caem **exatamente sobre as duas linhas**, que
   é onde eles têm de estar.

### 3.2 — O que isto dissolve

- A **dilatação** e toda a família dela (margem, compensação com sinal, alisamento do
  desvio): não há erro de vetorização quando não há vetorização. O `w + 2s` vira `w`.
- A **dessincronização do zoom** (#16): um conjunto só de vértices não pode descolar.
- O **`filled_shape_target`** vira um caso particular (região de uma face cuja fronteira é
  um traço só), não um ramo especial com critério de abraço.

⚠️ Isso é uma quantidade grande de código **apagado**, e é o principal argumento a favor
da wave. Mas apagar só é seguro depois que o caminho novo prove que cobre os casos — daí o
fatiamento do §5 manter os dois vivos até a última fatia.

---

## §4 — Os riscos, com o que fazer em cada um

| risco | por que é real | o que decide |
|---|---|---|
| **Arte com vãos** | o arranjo não fecha uma face que não está fechada | os fechamentos de gap já são traços e entram como arestas. **Se o clique cair numa face não-limitada, cai-se no caminho de hoje** (o raster), não se inventa geometria. |
| **Trapped-ball (C1)** | ele fecha passagens estreitas sem materializar traço | fatia C1 é recente e resolve um caso que o arranjo não vê. **Medir**: quantos cliques reais dependem só do trapped-ball. Se forem muitos, o arranjo precisa consumir o resultado dele. |
| **Robustez numérica** | interseção de segmentos em `f32` é o clássico gerador de degenerescência (quase-paralelos, pontos coincidentes) | o repo **já tem** `ph2d-vec-boolean` (linesweeper) resolvendo isto para paths fechados. Ver §6: reusar ou não é a 1ª decisão técnica. |
| **Perf** | O(n²) ingênuo em interseções; um quadro pode ter centenas de traços | o balde é operação de CLIQUE, não de frame. Orçamento: **≤ 100 ms**. Acima disso, sweep line. |
| **A arte muda** | a cor passa a ter outra forma nos casos hoje vetorizados | é o objetivo — mas exige **smoke**, e a fatia R1 entrega isso lado a lado com o caminho velho para comparar. |

---

## §5 — Fatias

| fatia | entrega | como se vê |
|---|---|---|
| **R0** ✅ | o **motor** de arranjo sobre polilinhas: interseções + grafo + walk de face. Sem UI, sem integração. | `cargo test -p ph2d-flip-fill arrange` |
| **R1** | o balde **escolhe** a rota: se o clique cai numa face limitada do arranjo, usa a geometria dela; senão, o caminho de hoje. Sem apagar nada. | preencher a grade do smoke e ver os vértices do fill **em cima** dos da linha |
| **R2** | a **dilatação recua** onde a rota nova serve (não há erro para compensar) | a franja some estruturalmente, não por constante |
| **R3** | aposentar o que ficou órfão (o `filled_shape_target` vira caso particular; a família da margem morre) | LOC apagada; gates que perderam o objeto |

**R0 é a fatia que decide a wave.** Se o motor não sair robusto em `f32` sobre arte de mão,
o resto não existe — e o custo até ali é contido.

---

## §6 — A 1ª decisão técnica: reusar o `ph2d-vec-boolean`?

Ele **já resolve** arranjo planar com linesweeper, e o Shape Builder o usa para exatamente
"que faces existem e qual foi tocada".

**A favor:** robustez numérica já paga, e uma 2ª implementação de interseção divergiria da
primeira (a lição de sempre neste repo).

**Contra:** ele opera sobre `VecPath` **fechados**; line-art de Flip é polilinha **aberta**.
Um arranjo de segmentos abertos é um problema mais simples, e adaptar o outro pode custar
mais que escrever este.

**Como decidir, e não no papel:** a R0 começa escrevendo os **gates** (as fixtures de arte
real) e tenta os dois caminhos contra eles. Quem passar primeiro, com menos código, ganha.

---

## §7 — O que exige ordem do Enio

1. **A wave em si.** Ela reescreve como o balde produz geometria, e a arte muda de forma
   nos casos hoje vetorizados. Não começa sem "vai".
2. **A ordem contra a C2 (LazyBrush).** As duas são grandes; a COLORIZE está a meio
   (C1 fechou e passou no smoke). Recomendação: **esta primeiro** — ela ataca a causa que
   cinco rodadas vinham administrando, e a C2 herda o balde melhor.
3. **Se a R0 reprovar** (motor não robusto em arte de mão), a wave morre ali e o balde
   fica como está — que hoje é aceitável, só não é o Draw:Filled.

---

## §8 — Critério de morte (escrito ANTES, como o §7.2 da 09)

- **R0**: se o walk de face não fechar um anel correto em **≥ 95 %** das fixtures de arte
  de mão (as mesmas que o `gpu_fill_fit` já usa, mais a grade do smoke), a abordagem
  topológica não sobrevive ao `f32` e a wave morre.
- **Perf**: > 100 ms num quadro de 200 traços após a 2ª tentativa ⇒ morre.
- **R1**: se a rota nova disparar em **< 30 %** dos cliques de um desenho real, ela não
  paga a complexidade de manter dois caminhos ⇒ reverte-se para o de hoje.

---

## §9 — R0: FECHADA (2026-07-18)

`ph2d-flip-fill::arrange` — `region_at(strokes, seed) -> Option<Region>`. **6 gates**, e o
oráculo de todos é a promessa da wave: *cada vértice do anel ou é um vértice de uma linha,
ou é um ponto que está em cima de duas* (uma interseção). O contorno vetorizado de hoje não
passa nisso — é a diferença que o Enio viu, virada asserção.

Fixtures: grade reta · **grade trêmula** (o critério de morte do §8) · vão ⇒ `None` ·
forma fechada sozinha · **quina aguda** (a que matou a abordagem do BUGS #16) · aninhadas.

### Dois defeitos que só as mutações acharam

1. **O `via` estava sempre vazio** — eu empurrava `0.0` e `1.0` para toda lista de cortes
   *"por conveniência"*, e com isso **todo vértice virava nó**. O anel saía certo (os
   vértices entravam como nós em vez de carona), então os 5 gates passavam e a mutação que
   **apaga o `via`** — o mecanismo que é o coração da wave — sobrevivia. Funcionava por
   acidente. Hoje: nós só em interseções, e essa mutação mata **4** gates.
2. **A regra do MÍNIMO nunca era exercida** — em todas as fixtures uma única face continha
   a semente, então trocar *menor área* por *maior* não derrubava nada. Faltava o
   fenômeno: formas **aninhadas**, onde o clique no miolo está dentro de dois anéis.

> **Lição (2ª instância nesta linha):** *fixture que não contém o fenômeno não prova a
> regra que fala dele*, e a mutação é o único jeito de descobrir isso — os gates estavam
> verdes, corretos e cegos.

Um bug real que um gate pegou: um traço fechado **sem interseção nenhuma** volta ao seu
único nó, e o guard `prev != ni` engolia a aresta de fechamento ⇒ o círculo sozinho dava
`None`. Um laço é um trecho legítimo; o que não é aresta é chegar ao mesmo nó **sem ter
andado** (`via` vazio).

### Perf — passa, com pouca folga

| traços | segmentos | ms |
|---|---|---|
| 20 | 580 | 0,5 |
| 50 | 1450 | 2,8 |
| 100 | 2900 | 13,3 |
| **200** | **5800** | **80,8** |

O critério do §8 é 100 ms ⇒ **vive**, mas o passo de interseções é O(segmentos²) e isso se
vê na tabela. ⚠️ **Não foi otimizado de propósito** — o número está sob a barra, e o
caminho, se um quadro real for mais denso, já é conhecido: broadphase por grade (o repo já
tem o padrão, BUGS #5). Otimizar antes de haver o problema é o que o `project_m5_perf_validated`
proíbe; o que não se pode é *não medir*, e está medido.

### O que a R0 NÃO decide

Ela é motor puro: não toca no balde, não muda um pixel do produto. A R1 é quem escolhe a
rota — e é lá que a arte muda e o smoke decide.
