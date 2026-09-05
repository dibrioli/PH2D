---
titulo: "Cloth (W10) — a pesquisa: o regime de um PINCEL, o estado da arte medido contra ele, e a triagem de licença"
tags: [modulo/3d, tipo/pesquisa, status/ativo, wave/W10]
status: ativo
modulo: 3D
atualizado: 2026-09-05
resumo: "Por que a escolha do plano (XPBD, 2016) está uma geração atrás EXATAMENTE onde um pincel vive; o que o VBD (2024) e o AVBD (2025) mudam; e a triagem T1 que achou porta permissiva e encerrou o clean-room antes de ele começar."
---

# Cloth (W10) — a pesquisa

> **O que este doc decide:** qual método, com que fonte, sob que licença. O *plano de
> implementação* é o [`02_plano.md`](02_plano.md).
>
> ⚠️ **Ele contradiz o [plano das ferramentas §5.1](../21_plano_modos_e_ferramentas.md)
> num ponto, com número.** A tabela de papers de lá manda `Müller et al. 2007 / Macklin,
> Müller & Chentanez 2016 — PBD / XPBD` para o Cloth. A pesquisa desta janela diz que
> essa escolha está **uma geração atrás, e atrás exatamente na propriedade que um pincel
> precisa**. O §2 traz o mecanismo.

---

## §0 — A pergunta certa não é *"qual é o melhor simulador de tecido"*

Um pincel de tecido **não é um simulador de roupa**, e quase todo eixo em que a
literatura compete é irrelevante aqui. O regime, item a item — e cada um deles é uma
restrição que ELIMINA candidatos:

| o que é | consequência |
|---|---|
| **O tecido é a própria superfície esculpida** — não há malha nova, não há molde, não há costura | o solver recebe uma malha de triângulos **arbitrária**, não uma grade nem um retalho de quads |
| **O repouso é congelado no pen-down** | comprimentos e ângulos de repouso são medidos UMA vez por traço; tudo que dependa do repouso pode ser **pré-computado** |
| **Só a pegada simula, e o anel de falloff é PREGADO** | o sistema tem **razão de massa infinita por construção**: massa infinita na borda contra massa finita no miolo |
| **O orçamento é UM evento de ponteiro** | o número de iterações é um **teto imposto de fora**; o solver tem de ser correto quando alguém o interrompe |
| **O artista arrasta na velocidade que quiser** | não existe passo "razoável": tem de ser **incondicionalmente estável** |
| **Uma lei da casa: determinismo** (`BTreeMap`, hash de replay) | ordem de percurso e coloração têm de ser **derivadas da malha**, nunca de iteração de hash |
| **Um traço = UM passo de undo** | o estado do solver vive **dentro do traço** e morre no pen-up |
| **Risco nomeado pelo próprio plano:** *"Cloth vira um projeto dentro do projeto"* | o método não pode pedir **afinação de constante por cena** |

⇒ **Duas propriedades decidem, e as duas são as mesmas duas:**

1. **estabilidade com o orçamento de iterações TRUNCADO** — porque quem trunca é o
   relógio do quadro, não o solver;
2. **comportamento sob razão de massa/rigidez alta** — porque o pincel a **fabrica**,
   ele não a encontra.

Precisão de drapeado, garantia de não-interpenetração, escala de GPU para milhões de
vértices: nada disso está na conta.

---

## §1 — A árvore da família, medida contra ESSE regime

| ano | método | o que traz | serve a um pincel? |
|---|---|---|---|
| 2003 | **Discrete Shells** (Grinspun, Hirani, Desbrun, Schröder) | energia de dobra por *hinge* (dois triângulos numa aresta interior) | ✅ é a peça de DOBRA, e é o que dá escala às pregas |
| 2006 | **Quadratic Bending / Discrete Quadratic Curvature** (Bergou et al.) | dobra isométrica com **Hessiana constante e semi-definida positiva** | ⛔ **REFUTADO na implementação — §5**: a propriedade é real e a hipótese dela é **repouso PLANO**, que uma escultura nunca é |
| 2007 | **PBD** (Müller, Heidelberger, Hennix, Ratcliff) | projeção de restrições, sem forças | ⛔ a rigidez **depende do número de iterações** e do passo |
| 2014 | **Projective Dynamics** (Bouaziz et al.) | pré-fatoração global, iterações baratíssimas | ⛔ §6 — a amortização precisa de um sistema FIXO, e a pegada muda a cada evento |
| 2015 | **Chebyshev** (Huamin Wang) | aceleração de ponto fixo | ⚠️ é um acelerador, não um método; o VBD já o incorpora |
| 2016 | **XPBD** (Macklin, Müller, Chentanez) | rigidez **independente do passo** — a `compliance` | ⚠️ **a escolha do plano.** Ver §2 |
| 2019 | **Small Steps** (Macklin, Storey, Lu, Terdiman, Chentanez, Jeschke, Miller) | *n* sub-passos de 1 iteração batem 1 passo de *n* iterações | ⭐ o achado é REAL e sobrevive: ele vale para o VBD também |
| 2020 | **IPC** (Li et al.) | não-interpenetração **garantida** por barreira | ⛔ §6 — ordens de grandeza fora do orçamento |
| **2024** | ⭐⭐⭐ **VBD — Vertex Block Descent** (Chen, Liu, Yang, Yuksel · SIGGRAPH 2024) | descida por blocos de VÉRTICE sobre a forma variacional do Euler implícito | ⭐⭐⭐ **é a escolha.** §3 |
| **2025** | **AVBD — Augmented VBD** (Giles, Diaz, Yuksel · SIGGRAPH 2025) | Lagrangiano aumentado: restrições **duras** de rigidez infinita + convergência sob razões de rigidez altas | ⏳ §6 — adiado com o gatilho nomeado, não recusado |
| 2025 | **MGPBD** (multigrid sobre XPBD global) | acelera o XPBD por multigrid | ⛔ cura o sintoma do XPBD com maquinaria global |
| 2026 | **BS-Cloth** (Meng et al. · SIGGRAPH 2026) | FEM de B-spline quadrática, `C¹`, sem *membrane locking* | ⛔ §6 — **retalhos de quads e offline**; o mais novo não é o certo |

---

## §2 — Por que o XPBD do plano está uma geração atrás, e EXATAMENTE onde

O plano escolheu XPBD com um motivo correto, escrito na tabela de riscos dele:

> *"XPBD (rigidez independente do passo) é escolhido justamente para não afinar constante
> por cena"*

O motivo continua verdadeiro: o XPBD **entrega** isso, e é por isso que ele substituiu o
PBD. O problema é que ele o entrega **junto com duas fraquezas que são o retrato do nosso
regime**, e as duas estão documentadas pelos autores do método que as corrigiu:

1. **As aproximações da formulação do XPBD divergem da solução do Euler implícito, e a
   divergência cresce com PASSO GRANDE e CONTAGEM DE ITERAÇÕES LIMITADA** — que é,
   literalmente, o §0: um pincel corre com o passo que o artista dita e o número de
   iterações que sobra no quadro.
2. **O XPBD sofre particularmente com RAZÕES DE MASSA ALTAS** — e o pincel *fabrica* uma
   razão de massa infinita toda vez que prega o anel de falloff, que é a própria feature
   (o «lock vertices in the simulation falloff area» que existe para a transição não
   estourar).

⇒ **O plano pediu uma propriedade e escolheu o método que a tem pela rota fraca.** O VBD
entrega a mesma garantia por uma rota mais forte — ele **converge para o Euler implícito**,
então a rigidez é *material*, não afinação — e acrescenta a que o pincel precisa e o plano
não sabia nomear: **estabilidade quando alguém trunca o orçamento**.

⚠️ **E isto não é o §0.0 da casa mordendo *«quem move o número reconfere a nota»*: ninguém
moveu número nenhum.** O VBD é de 2024 e o plano é de 2026-08-15 — *a nota nasceu velha
porque foi escrita sem varrer o campo*. É a lição mais barata desta pesquisa: **a lista de
papers de um plano tem data de validade, e a validade dela é o dia em que alguém a
varreu.**

---

## §3 — O que o VBD é, em uma página

A atualização de **um vértice**, que é o método inteiro:

```
Δxᵢ = Hᵢ⁻¹ · fᵢ

Hᵢ = (mᵢ/h²)·I  +  Σ_{j ∈ Fᵢ}  ∂²Eⱼ/∂xᵢ²
fᵢ = −(mᵢ/h²)·(xᵢ − yᵢ)  −  Σ_{j ∈ Fᵢ}  ∂Eⱼ/∂xᵢ
```

`Fᵢ` são os elementos incidentes ao vértice `i`; `yᵢ` é a posição prevista pela inércia.
É **um Newton 3×3 por vértice**, resolvido analiticamente.

- **Gauss-Seidel por COR de vértice.** Colorem-se os vértices de modo que nenhum elemento
  ligue dois da mesma cor; dentro de uma cor tudo é paralelo, entre cores é Gauss-Seidel.
  ⭐ Colorir **vértices** dá muito menos cores que colorir elementos — medido no paper:
  **8 cores para 3 891 vértices** contra **76 cores para 14 802 tetraedros**.
- **A estabilidade não vem de amortecimento nem de *line search*:** cada energia local
  `Gᵢ` é garantidamente reduzida, e a soma das reduções locais **é** a redução da energia
  global. ⇒ vale com **uma iteração só**. ⚠️ Os autores **testaram *line search* e a
  recusaram**: `+40 %` de custo, nenhum benefício mensurável.
- **A Hessiana indefinida NÃO é projetada.** O argumento deles: o Euler implícito
  variacional procura `∇G(x) = 0`, não um mínimo local; o 3×3 analítico anda para o
  extremo da aproximação quadrática, que é um estado estável. A degenerescência é tratada
  por **salto**: `|det(Hᵢ)| ≤ ε` ⇒ o vértice não se move nesta iteração. (A Hessiana da
  inércia é sempre posto cheio, então o caso é raro.)
- **Inicialização adaptativa:** `x = xᵗ + h·vᵗ + h²·ã`, com `ã` a fração da aceleração
  externa recuperada do quadro anterior, **presa em `[0,1]`** — inclui a gravidade quando
  o movimento parece queda livre e **mantém a posição quando o corpo está parado**, que é
  o que evita esticar e penetrar numa solução parcialmente convergida.
- **Amortecimento de Rayleigh entra no MESMO 3×3**, nos dois lados (Hessiana e força).
  Nada global.
- **Aceleração de Chebyshev** é opcional, aplicada depois de cada passada de cores.
- **Medido pelos autores:** 230 K vértices, 120 iterações, `1/120 s` ⇒ **15–17 ms/quadro**;
  e os testes de estresse ficam estáveis com **1 a 10 iterações**.

⭐⭐ **Por que esta FORMA é a nossa:** todo kernel deste módulo já é *"para cada vértice na
pegada, acumule dos elementos incidentes"*. O VBD **é** essa forma. A adjacência CSR e a
octree que ele precisa já existem no [`ph2d-mesh`](../../../crates/ph2d-mesh/); o
congelamento do repouso no pen-down já é a `GripLaw::frozen`; e o *"um traço, um passo de
undo"* já é a porta `close_stroke`.

---

## §4 — A triagem de licença (T1), com o texto LIDO

⚠️ **O protocolo manda ler a licença REAL, não a reputação**
([SKILL_Cleanroom §2](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)).
As cinco abaixo foram lidas no arquivo `LICENSE`, não em página de projeto:

| implementação | o que traz | licença **lida** |
|---|---|---|
| [`savant117/avbd-demo2d`](https://github.com/savant117/avbd-demo2d) · [`avbd-demo3d`](https://github.com/savant117/avbd-demo3d) | **referência oficial do AVBD**, escrita pelo 1º autor | **MIT**, © 2025 Chris Giles |
| [`AnkaChan/Gaia`](https://github.com/AnkaChan/Gaia) | **referência oficial do VBD**, do 1º autor; VBD + XPBD, malha tri/tet, colisão | **Apache-2.0** |
| [`alexrodag/spg`](https://github.com/alexrodag/spg) | 7 solvers **lado a lado** — XPBD · VBD · Baraff-Witkin · Newton BDF1/BDF2 · quasi-estático | **MIT**, © 2024 Alejandro Rodriguez |
| [`InteractiveComputerGraphics/PositionBasedDynamics`](https://github.com/InteractiveComputerGraphics/PositionBasedDynamics) (Bender) | a referência acadêmica canônica de PBD/XPBD | **MIT** |
| [`VigorFox/PhysX_AVBD`](https://github.com/VigorFox/PhysX_AVBD) | AVBD dentro do PhysX — referência de **produção** | **BSD-3-Clause** (licença do PhysX SDK) |

⇒ **DEGRAU T0.** O clean-room **acabou antes de começar**: não há parede, não há espec, não
há ledger, não há subagente E/R. É **porte fiel com atribuição**, feito pela própria janela
— exatamente o precedente do SculptGL (MIT) e do Instant Meshes (BSD) desta casa.

⚠️⚠️ **E o alvo GPL não é necessário.** O `sculpt_cloth.cc` do Blender é GPL e **não
precisamos dele**: o que se quer daquele lado é **comportamento** — quais modos o artista
espera encontrar —, e comportamento vem do **manual público** (fatos, nunca o *wording* —
[SKILL_Cleanroom §1.2](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md), a
lição do litígio SAS×WPL) e do nosso próprio
[`20_divergencias_tools.md`](../20_divergencias_tools.md). *A porta permissiva estava
destrancada e o degrau T1 é o que a pressa pula.*

**Comportamento que o alvo declara** (manual do Blender, e o nosso doc 20): área de
simulação que **acompanha o pincel** limitada por raio fixo · força aplicada como **esfera
ou plano** · **limite de simulação** e uma zona de **falloff** desenhada tracejada em volta
do cursor, cujos vértices podem ser **pregados** · **massa** por partícula · quanto a força
**se propaga** pelo tecido · quanto o tecido **preserva a forma original** (corpo mole) ·
colisão opcional com outros objetos · modos `GRAB` / `SNAKE_HOOK` / `EXPAND`, com força
`10×` por omissão. **O ZBrush** (2021) tem o sistema irmão — *Dynamics* com pincéis
`ClothTwister`, `ClothWind`, `ClothPinchTrails` —, e é a referência de **produto**, T4:
observar o que ele faz, nunca o que ele é por dentro.

---

## §5 — As peças, cada uma com a fonte publicada

| peça | fonte | por que esta |
|---|---|---|
| **membrana (estiramento/cisalhamento)** | **StVK** — a escolha do próprio paper do VBD para tecido | gradiente e Hessiana analíticos por triângulo, baratos |
| **dobra** | ⛔⛔ **REFUTADO — ver abaixo.** Hoje é o **ângulo diedro com ângulo de repouso** (Grinspun, Hirani, Desbrun & Schröder, *Discrete Shells*, SCA 2003), com Hessiana de **Gauss-Newton** (PSD por construção) | vale em superfície de qualquer curvatura, e é zero no repouso por construção |
| **amortecimento** | Rayleigh, dentro do mesmo 3×3 (§3) | local; nada global entra na conta |
| **pregar o anel de falloff** | o próprio VBD: vértice preso é **vértice que não se atualiza** | exato, massa infinita de verdade, **sem termo de penalidade e sem constante para afinar** |

⛔⛔⛔ **E ESTA LINHA ESTAVA ERRADA — a implementação (2026-09-05) refutou-a.** Ela dizia:

> *«num pincel o repouso é congelado no pen-down, e a Hessiana do modelo quadrático de dobra
> só depende do repouso ⇒ ela é montada UMA vez por traço … a peça mais cara do tecido vira
> uma tabela constante»*

O raciocínio sobre o **congelamento** está certo e continua a valer (é o §0). O que está
errado é a premissa sobre o **modelo**: o modelo quadrático de dobra **assume o repouso
PLANO** — a isometria do material é exatamente a condição que o torna válido, e é por isso
que ele é a escolha certa para *pano de roupa*, que nasce plano.

**O repouso de um pincel de escultura é a superfície esculpida, que é curva em todo lugar
interessante.** Usá-lo ali daria força no repouso: a peça mexer-se-ia sozinha ao encostar o
pincel — exatamente o que o gate *«o repouso é ponto fixo»* existe para proibir.

⇒ a peça de dobra é o **ângulo diedro com ângulo de repouso** (Discrete Shells), com a
Hessiana de **Gauss-Newton** — o termo com `∂²θ` é descartado, o que a torna um produto
externo e portanto **PSD por construção**: com o gradiente exato e uma métrica PSD, o passo
local é garantidamente de descida. *Trocar a exatidão da métrica pela garantia de descida é
o negócio certo num pincel, onde quem trunca as iterações é o relógio do quadro.*

⚠️ **A lição do episódio, e é a mesma do §2:** eu escolhi o modelo pela PROPRIEDADE que ele
anunciava (Hessiana constante e PSD) sem conferir a HIPÓTESE sob a qual ele a tem. *Uma
propriedade citada sem a hipótese dela é uma promessa sem contrato.*

⭐ **E o que sobra do achado é real:** o congelamento do repouso continua a pagar — área,
forma de repouso, ângulo e peso de cada dobradiça, e a massa de cada vértice são medidos
**uma vez por traço**, nunca por evento.

---

## §6 — O que fica FORA, e o motivo de cada um

> ⛔ Esta seção existe para ninguém "completar a lista" daqui a seis meses sem saber que
> houve uma decisão. Nenhum item aqui é *"não tivemos tempo"*.

| fora | motivo |
|---|---|
| **BS-Cloth** (SIGGRAPH 2026) | pede **retalhos de quads** e é **offline** (grava resultados intermediários em `.yaml`). Uma escultura é malha de triângulos arbitrária e o orçamento é um evento de ponteiro. ⚠️ **O mais novo não é o certo** — ele resolve *membrane locking* em roupa de produção, que não é o nosso problema |
| **IPC / GIPC** | garantia de não-interpenetração por barreira: ordens de grandeza fora do orçamento de um pincel, e o Blender **também não a tem** no pincel dele |
| **Projective Dynamics** | a pré-fatoração global amortiza sobre um sistema **fixo**; a pegada do pincel muda a cada evento ⇒ a amortização nunca é paga |
| **MGPBD** | multigrid **sobre** o XPBD: cura por maquinaria global o sintoma cuja causa o VBD remove |
| **auto-colisão** | não está no pincel do Blender e **não entra na W10**. É onde as pregas param de se atravessar, então é feature de verdade — ⏳ decisão de produto, com preço a medir |
| **AVBD** (2025) | ⏳ **ADIADO com o gatilho nomeado, NÃO recusado.** O que ele compra é (a) restrição **dura** de rigidez infinita e (b) convergência sob razão de rigidez alta. A nossa única restrição dura é **pregar**, e o VBD já a faz *exatamente* por salto; e o porte 2D de referência reporta o solver dual **~2× mais lento**. ⇒ **gatilho:** no dia em que o pincel ganhar colisão com objeto, empilhamento ou junta, o AVBD é a porta — e a referência dele é **MIT** |

---

## §7 — Os riscos, nomeados

| risco | por que é real | o que o contém |
|---|---|---|
| **A Hessiana indefinida não é projetada** | é uma escolha declarada do paper, não um descuido — mas quem a validou foi a bancada deles, não a nossa | fixtura de estresse: o artista arrastando na velocidade máxima, com o gate a medir que a malha **não diverge** e que a energia não sobe |
| **Coloração e determinismo** | a coloração decide a ORDEM de Gauss-Seidel, e ordem diferente é resultado diferente | a cor sai de um percurso **derivado da malha** (índice crescente), nunca de iteração de `HashMap` — a lei do `BTreeMap` desta casa, e há hash de replay para cobrá-la |
| **A pegada muda a cada evento** | recolorir e re-medir o repouso por evento seria O(pegada) repetido e mudaria a lei no meio do traço | a região de simulação, a coloração e o repouso nascem **UMA vez no pen-down** — que é o que a `GripLaw::frozen` já faz para os outros verbos |
| **A malha é `f32`** | medido, e já refutou premissa de briefing antes (o pivô do quad remesh) | a aritmética do 3×3 em `f64` com armazenamento `f32`, que é **exatamente** o que o `ref_kernels.rs` desta crate já faz |
| **O custo** | um pincel que passa do quadro não é um pincel | ⚠️ nenhum teto entra sem tabela medida ao lado (CLAUDE.md §0.0); a sonda mede **por tamanho de pegada**, não por malha |

---

## §8 — O que esta pesquisa NÃO respondeu

1. **A tabela de paridade `b-mode` do Blender, campo a campo.** O manual é renderizado por
   JS e o `WebFetch` recebe `403`; o que temos são os fatos do §4, suficientes para
   decidir o método e **não** para prometer paridade. ⇒ quem implementar coleta a tabela
   antes de escrever o painel.
2. **Se o artista quer auto-colisão.** É a diferença entre pregas que encostam e pregas
   que se atravessam. Pergunta de produto, com preço a medir.
3. **CPU ou device.** O VBD nasceu para GPU, e o número dele (230 K vértices, 120
   iterações, 15–17 ms) é de GPU. A nossa pegada é ordens de grandeza menor — a sonda de
   custo é que decide, e a lei da casa é que **o teto é do hardware, nunca do caminho
   lento**.

---

## Fontes

- Chen, Liu, Yang & Yuksel — **Vertex Block Descent**, ACM TOG (SIGGRAPH 2024) ·
  [arXiv:2403.06321](https://arxiv.org/abs/2403.06321) ·
  [projeto](https://graphics.cs.utah.edu/research/projects/vbd/)
- Giles, Diaz & Yuksel — **Augmented Vertex Block Descent**, ACM TOG (SIGGRAPH 2025) ·
  [projeto + PDF](https://graphics.cs.utah.edu/research/projects/avbd/)
- Macklin, Müller & Chentanez — **XPBD**, MIG 2016 ·
  [PDF](https://matthias-research.github.io/pages/publications/XPBD.pdf)
- Macklin et al. — **Small Steps in Physics Simulation**, SCA 2019 ·
  [PDF](https://mmacklin.com/smallsteps.pdf)
- Grinspun, Hirani, Desbrun & Schröder — **Discrete Shells**, SCA 2003 ·
  [PDF](https://multires.caltech.edu/pubs/ds.pdf)
- Bergou et al. — **Discrete Quadratic Curvature Energies** ·
  [PDF](https://ddg.math.uni-goettingen.de/pub/bendingCAGD.pdf)
- Meng et al. — **Efficient B-Spline Finite Elements for Cloth Simulation**, SIGGRAPH 2026 ·
  [arXiv](https://arxiv.org/html/2506.18867v4) · [código](https://github.com/Simulation-Intelligence/BS-Cloth)
- Blender Manual — [Cloth brush](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/cloth.html)
  · [Cloth Sculpting improvements in 2.91](https://code.blender.org/2020/10/cloth-sculpting-improvements-in-blender-2-91/)
- Pixologic — [ZBrush 2021 features](https://pixologic.com/zbrush2021/) (referência de produto, T4)
