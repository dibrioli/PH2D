---
titulo: "Cloth (W10) — o plano: quatro metades, e a primeira coisa NOVA que este módulo vai ter"
tags: [modulo/3d, tipo/plano, status/ativo, wave/W10]
status: ativo
modulo: 3D
atualizado: 2026-09-05
resumo: "O Cloth é o primeiro verbo cujo ESTADO sobrevive entre eventos dentro de um traço. O plano parte nas quatro metades que podem shipar em ordem, com o gate de cada uma."
---

# Cloth (W10) — o plano

> **A pesquisa que escolheu o método está no [`01`](01_pesquisa_o_estado_da_arte.md).**
> Este doc é o que se constrói, em que ordem, e o que cada passo tem de provar.

---

## §0 — ⭐⭐ O que o Cloth traz que este módulo nunca teve: ESTADO que sobrevive ao evento

Todos os 23 verbos de hoje respondem à mesma pergunta: **`alvo = f(pre_congelado, dab)`**,
e o aplicador interpola. É uma função pura do gesto — e é por isso que o undo é trivial e
o re-carimbo é previsível.

Uma simulação **não é uma função do gesto**: ela tem **velocidade**, e o resultado do
evento *N* é a entrada do evento *N+1*.

⚠️ **Este módulo já encontrou esta forma uma vez, e ela era um DEFEITO ali.** Na W9a as
três leis de anel liam a malha viva, e num filtro isso fazia duas chamadas na mesma força
**comporem** — *o desenho passava a depender de quantos eventos o rato mandou*; a cura foi
repor a pose congelada antes de cada chamada. ⭐ **Aqui a composição é exatamente o que se
quer** — ela É a simulação —, e a diferença entre as duas situações é **o relógio**: o
filtro não tem nenhum e o tecido tem o dele, em sub-passos determinísticos. *A mesma forma
é defeito quando não há relógio e é a feature quando há.*

⇒ o Cloth pede **um `Grip` novo**. E o doc do [`Grip`](../../../crates/ph2d-sculpt3d/src/grip.rs)
já escreveu a lei de como isso entra: *"quem acrescentar um quarto grip não compila até
dizer, em cada consumidor, o que ele significa"* — o `match` exaustivo é a rede.

---

## §1 — As quatro metades, na ordem em que podem shipar

### ✅ W10a — o SOLVER, puro, sem uma linha de UI — **FECHADA (2026-09-05)**

`ph2d-cloth` (crate-folha nova, ADR-0075: feature nova = drop-crate) — ou módulo dentro
da `ph2d-sculpt3d` se a medição de acoplamento disser que a fronteira é falsa.

**Conteúdo:** o passo do VBD (§3 da pesquisa) — inércia + membrana StVK + dobra quadrática
+ Rayleigh, tudo acumulando no mesmo bloco `3×3` por vértice; coloração de vértices
derivada da malha; pregar por salto; inicialização adaptativa; sub-passos.

**Gates — e cada um mata uma maneira de estar errado:**

| gate | o que ele impede |
|---|---|
| **o repouso é ponto fixo** — a malha parada, sem força externa, não se move **ao bit** | o solver que "relaxa" o que ninguém tocou; é o controle positivo de toda a suíte |
| **a energia global DESCE a cada iteração**, em toda fixtura | a promessa central do método; se ela não vale, o resto é decoração |
| **uma iteração só é estável** — 500 quadros, 1 iteração, malha não diverge | é o regime real do pincel (§0 da pesquisa) |
| **estresse: o arrasto na velocidade máxima não explode** | ⚠️ o risco nomeado — o VBD **não projeta** a Hessiana indefinida, e quem validou isso foi a bancada dos autores, não a nossa |
| **determinismo** — a mesma entrada dá o mesmo `f32`, e a coloração não depende de ordem de hash | a lei do `BTreeMap` desta casa, com hash de replay a cobrá-la |
| **pregado é pregado** — vértice preso não se move, nem um ULP, sob força nenhuma | o anel de falloff é a feature; um pregado que escorrega é a transição que estoura |
| **a razão de massa infinita é servida** — o gate que o XPBD não passaria | é a justificativa medida da troca de método (§2 da pesquisa) |
| **custo por TAMANHO DE PEGADA**, não por tamanho de malha | ⚠️ nenhum teto entra sem a tabela ao lado (CLAUDE.md §0.0) |

### ✅ W10b — o PINCEL — **FECHADA (2026-09-05)**

> **O que a construção mudou em relação ao que esta secção previa**, e cada item
> foi medido:
>
> - ⛔⛔ **A dobra prescrita pela pesquisa foi REFUTADA** — o modelo quadrático
>   assume repouso PLANO, e uma escultura é curva. Hoje é ângulo diedro com
>   ângulo de repouso, Hessiana de Gauss-Newton. Ver [`01` §5](01_pesquisa_o_estado_da_arte.md).
> - ⛔⛔ **A conversão «gesto → força» tinha uma CONSTANTE INVENTADA**, e o gate a
>   mediu: com a mão a percorrer `0,24`, o pano respondia `5,6e-4` — `0,2 %`. Ela
>   foi **apagada**: sob o dedo o pano **segue a mão** (posição e momento, pesados
>   pela curva do pincel) e a prega nasce do solver a arrastar a vizinhança. *Um
>   número que não nomeia recurso nenhum não é um teto, é um palpite.*
> - ⚠️ **O tecido SAIU de dois censos do aplicador, por LEI e não por nome**
>   (`Verb::writes_through_applicator`): ele não escreve `accum` nem `target`. A
>   dívida foi paga no mesmo commit — ele tem seis gates próprios.
> - ⚠️ **Ele não oferece chip de MODO**, e a ausência é a decisão: a lei dele é
>   uma só (VBD, paper com nome e ano). Um dropdown de uma opção é controlo morto.
> - ⚠️ **Encostar sem mover não deforma** — ele responde à VIAGEM da mão. É
>   produto correto e é o que o separa dos outros 23 verbos, que carimbam no
>   primeiro dab.

**O que era previsto, e ficou:**

O `Grip` novo; a região de simulação, a coloração e o repouso nascendo **UMA vez no
pen-down** (a `GripLaw::frozen` que já existe); o anel de falloff pregado; a força do
pincel entrando como força externa **em esfera ou em plano**; os modos de deformação; e
**um** passo de undo por traço, pela porta `close_stroke` que já existe.

**Gate de costura:** o gesto real (`seam_*`), não `Click` sintético — o precedente do
`seam_bool` do Vector, onde um chip morto sob o ponteiro passava por todo teste sintético.

### W10c — o PAINEL

As rows saem da tabela, como em todo painel deste módulo (`rows.rs` percorre UMA lista) —
massa, amortecimento, rigidez de membrana e de dobra, força, limite de simulação, falloff,
pregar a borda. `Basic` × `Pro` pela lei do §2 do plano das ferramentas.

**Gate:** nenhum chip morto — para cada `(verbo, modo)` oferecido existe uma cena em que o
resultado difere do vizinho acima do piso de paridade, varrendo a **matriz** e não uma
lista escrita à mão.

### W10d — os CINCO FILTROS DE TECIDO

`Gravity · Inflate · Expand · Pinch · Scale` sobre o mesmo solver, pelo driver de filtro
que a **W9a já construiu** (`stroke_filter.rs`, o arrasto horizontal, um passo de undo).
⭐ É a metade barata, e é barata **porque a W9 pagou o driver** — o precedente do *Filter
Layer* do Painter (*"não há kernel novo, e isso é o desenho inteiro"*) vale aqui pela
segunda vez.

---

## §2 — O que fica FORA da W10, com o gatilho de cada um

| fora | gatilho que o acorda |
|---|---|
| **auto-colisão** | decisão de produto do Enio — é onde as pregas param de se atravessar. O pincel do Blender também não a tem |
| **colisão com objeto** | o mesmo, e ⚠️ **é o gatilho do AVBD** (§6 da pesquisa): no dia em que entrar, a referência **MIT** do Lagrangiano aumentado é a porta |
| **rodar no device** | a sonda de custo decide. ⚠️ E a lei da casa é que **o teto é do hardware, nunca do caminho lento** — se a CPU bastar para a pegada, ela basta; se não bastar, quem manda no teto é o device |

---

## §3 — A ordem, e por que ela é esta

O **W10a sozinho não tem smoke** — é um solver sem gesto, e a lei da casa diz que *a etapa
acaba num smoke*. ⇒ **W10a e W10b fecham juntos**, e é o par que o Enio testa; o `c` e o
`d` são incrementos que já têm onde aparecer.

⚠️ **E o `d` depende do `a`, não do `b`:** os cinco filtros de tecido precisam do solver e
do driver de filtro, nunca do pincel. Se a W10b atrasar, a W10d ainda pode andar.
