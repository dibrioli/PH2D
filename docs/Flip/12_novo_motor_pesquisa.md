# 12 — O MOTOR DE TRAÇO NOVO: baseline medido, pesquisa, e a comparação dos candidatos

**Data:** 2026-07-28 · **Linha:** `line/FLIP` · **HEAD:** `9b2e72ee4` · **Status:** PESQUISA —
**nenhuma linha de motor escrita**, por ordem do handoff
([`HANDOFF_line_FLIP_NOVO_MOTOR_DE_TRACO_2026-07-28.md`](../HANDOFF_line_FLIP_NOVO_MOTOR_DE_TRACO_2026-07-28.md) §11.4).

> A ordem do Enio: *"encontrar um modo completamente novo de renderização do stroke e descartar
> completamente o atual … Ele deve pesquisar o estado da arte, o padrão ouro."*

Este documento entrega os passos 2 e 3 do §11 do handoff: **de que baseline eu parto** (medido, hoje)
e **o mapa dos candidatos** com o que cada um quebra das três propriedades do §3. A decisão de qual
construir é do Enio.

---

## 1. A BASELINE — medida hoje, no HEAD desta linha

O handoff (§1) avisa que o último conserto (`7ca83d6fb`) estava **pendente de smoke**: o veredito do
Enio foi dado sem que ele estivesse na tela. Rodei os oráculos do §6 para que o número de partida
fosse o número real. **Isto não reabre a decisão** — está aqui porque um plano construído sobre a
baseline errada mede a coisa errada.

`cargo test -p ph2d-flip-render --release --test painter_look --test sampling_invariance -- --ignored`

| oráculo | resultado hoje |
|---|---|
| **A LEI** (`the_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled`) | **−3 em TODA densidade**, de `0,80·r` a `0,04·r`, em h=0,4 e h=0,7 |
| o penhasco (`measure_where_the_neighbour_budget_breaks`) | `falta −4`, **0 px** falhando, em todas as 5 densidades |
| **vs o DEPÓSITO do Painter** (o oráculo de verdade) | **0 px FALTAM tinta** · 166 px SOBRAM, pior **+140**, *todos no tip convexo* · controle (traço reto, miolo) **+1/255** |
| **um traço vs cinco** (o oráculo do Enio) | **−64** em h=0,4 (154 px) · h=0,7 (88 px) · **h=1,0 (58 px)** |
| render-and-look (`flip_diff`, `flip_lenta`) | corpo do traço **indistinguível**; o resíduo são as **5 pontas externas** |

**O que isto diz, e o que NÃO diz.**

- O **penhasco morreu**: o item 10 do cadáver (−184 / −255, a tinta SUMINDO em mão lenta) não
  reproduz mais. O item 11 (`7ca83d6fb`) segurou, e o desvio constante −3 dele confere.
- O que sobrou **não é falta de tinta, é excesso**, e vive **na ponta convexa** — não no corpo nem no
  cruzamento. O gate `measure_the_flip_against_the_painters_deposit` reparte explicitamente:
  *0 px faltam (a cunha escura da foto) · 166 px sobram (o tip convexo)*.
- ⚠️ **O `−64` do oráculo do Enio aparece IGUAL em `hardness = 1,0`** (58 px). Uma diferença que
  sobrevive à dureza máxima **não pode ser da lei da tinta macia** — ela é **geometria de junta e
  tampa**: cinco traços separados põem *duas tampas redondas sobrepostas* em cada ponta, um traço
  põe *uma quina*. Isso é a wave de **joins & caps** que o `03 §8` já tem na fila, não o defeito
  de cruzamento.

⚠️ **Consequência honesta, e é decisão sua:** a baseline de hoje **não exibe** o defeito catastrófico
que motivou a ordem. Ela exibe (a) excesso na ponta convexa e (b) uma diferença de tampa/junta. Isso
**não** invalida o diagnóstico estrutural do §3 — que é sobre o *desenho*, não sobre o número de hoje
— e **não** invalida a ordem. Mas muda o que um motor novo tem de *provar*: ele parte de um
concorrente que hoje acerta o corpo do traço a ±3/255.

**O defeito que segue ABERTO e MEDIDO, esse sim, é do desenho:** `self_overlap` conta duas vezes
(**até 43/255**), pela razão do §3(C) — cada face compõe a união GLOBAL, então `N` faces sobrepostas
dão `1−(1−u)^N` com o MESMO `u`.

---

## 2. A PESQUISA — o que o estado da arte de fato entrega

### 2.1 Ciallo (SIGGRAPH 2024) — o candidato acadêmico mais próximo, e ele NÃO resolve isto

`Ciallo: GPU-Accelerated Rendering of Vector Brush Strokes` (Shen Ciao & Li-Yi Wei) é o trabalho mais
próximo do nosso problema, e o handoff o nomeia como candidato 4. **Fui ler a limitação declarada
pelos próprios autores**, e ela é a nossa:

> *"When rendering transparent strokes, Ciallo's strokes create self-overlapping areas. While this
> may be considered an undesired limitation for **vanilla strokes**, it is actually a desirable
> feature for stamp and airbrush strokes. The current techniques only support alpha blending, and
> have limitations when rendering **transparent vanilla strokes**."*

⚠️ **O "vanilla stroke" do Ciallo é exatamente o nosso traço de borda macia**, e a auto-sobreposição
transparente dele é **limitação assumida no paper**, não um detalhe de implementação. As técnicas
dele são declaradamente *alpha-blending only*.

**Conclusão:** portar o Ciallo **não** compra a cura. Ele é excelente no que nós já pegamos dele (o
airbrush de Beer-Lambert, `03 §8`, já shipado) e nos modelos **stamp**/**airbrush**, onde o acúmulo
por sobreposição é *desejado*. Para o vanilla transparente, o estado da arte publicado está no mesmo
lugar que nós.

### 2.2 Blender Grease Pencil — o ancestral continua quebrado, no 5.0

O handoff aponta a issue #140075 como defeito aberto lá. A varredura de hoje mostra que a família
segue viva depois da reescrita 4.x/5.x:

- **#154433** — *"Strokes of different opacity or hardness cause transparent aliasing in render and
  viewport"*, reportada em **5.0.1 e 4.2.18 LTS**.
- Relatos irmãos: *"when self overlap is off it produces artifacts on edges"* e *"drawing with low
  strength (opacity), overlapping a stroke onto itself multiplies the overlap"*.

**Conclusão:** a linhagem GP não tem a resposta a copiar. Nós divergimos dela de propósito (a união
global) e chegamos mais longe; continuar nela é continuar a consertar o mesmo desenho.

### 2.3 Vello / GPU-friendly Stroke Expansion — resolve (B), e só (B)

Li o paper (Levien et al., arXiv 2405.00127). Ele paraleliza a *expansão* do traço em outline e
alimenta binning/tiling **depois**. A frase que decide:

> *"It produces geometric outlines only — not soft/feathered brush effects."*

**Conclusão:** confirma o que o handoff §5 já suspeitava. Tile binning tira o **teto** (propriedade
B) porque a lista por tile é limitada por memória e não por uma constante — mas a **lei da tinta**
continua sendo problema nosso. É uma peça, não um motor.

### 2.4 ⭐ O achado que a pesquisa in-repo não tinha: a TERCEIRA lei (Krita "Soft" / Drawpile)

O [`docs/Painter/25 §13.9.1`](../Painter/25_avaliacao_gpu.md) comparou **duas** leis e concluiu, com
razão, que a arquitetura de GIMP e Krita é a MESMA (buffer por-traço aplicado sobre o estado
congelado) e que só a **lei** difere:

- **GIMP / Build-up** — o perfil do dab é uma **TAXA** rumo a um teto que é a *opacidade do TRAÇO*.
  Com opacidade 100% o teto é vácuo ⇒ **endurece a borda**. (Medido no Painter: band 3,53 px numa
  passada → 1,38 px em quinze.)
- **Krita Wash / Alpha Darken** — o **ALVO** guardado por `max`. Mata o endurecimento e deixa a
  estrutura por-dab à VISTA ⇒ **contas** (beading). (Construído no Painter e **reprovado na tela**
  pelo Enio, §13.10.)

**Existe uma terceira, shipada e nomeada, que nenhum dos dois docs cobre — o modo `Soft` do Krita,
vindo do Drawpile:**

> **Wash:** *"the opacity of each pixel being limited by the stroke's overall opacity"*
> **Soft:** *"each pixel is limited **individually**, preventing build-up within the stroke"*

E a lista de problemas que ele declara resolver é, literalmente, a nossa:

> *"**Soft edges remain soft** during painting, avoiding **hardening through stroke overlap**" ·
> "No re-painting artifacts when soft brushes overlap within the same stroke."*

⚠️ **E a cautela, que vale tanto quanto o achado.** A lei do `Soft` troca o teto GLOBAL (opacidade do
traço) por um teto **POR PIXEL** (a cobertura do próprio dab ali). O ponto fixo dessa recorrência é
`max_k(w_k)` — **o mesmo `max` da lei Wash que o Painter já reprovou por beading** —, apenas
alcançado por uma taxa em vez de instantaneamente. Se a taxa suaviza o suficiente para o beading
sumir, isto é a cura dos dois defeitos de uma vez; se não, é o beading outra vez com outro nome.
**Isso se MEDE, não se assume** — e é barato de medir, porque o Painter já tem a sonda de aparência
(`probe_mask_beading_along_the_axis`) e o número do endurecimento num teste executável
(`the_documented_hardening_is_still_there_and_this_is_its_number`).

⚠️ Nota do próprio desenvolvedor, que é honestidade e não ressalva: *"very soft brushes have a funky
effect when you draw over the same area in a single stroke … the mode working as intended, it's not
a bug."*

---

## 3. OS CANDIDATOS — o que cada um quebra das três propriedades do §3

Lembrete do §3: **(A)** a cobertura é função da DISTÂNCIA, não da tinta depositada · **(B)** o
fragment precisa que lhe CONTEM o caminho, por um canal lateral de tamanho FIXO
(`MAX_EXTRAS_PER_SEGMENT = 16`, `MAX_RIBBON_EXTRAS = 16` em `neighbors.rs`) · **(C)** o depth elege
UM fragmento por pixel. *Um motor novo tem de quebrar pelo menos duas das três.*

| | **C1** buffer de dabs | **C2** binning por tile | **C3** integral analítica *(como o handoff a descreve)* | **⭐ C4** integral **ADITIVA** *(o que proponho)* |
|---|---|---|---|---|
| quebra **(A)**? | ✅ o buffer **É** o depósito | ❌ | ✅ | ✅ |
| quebra **(B)**? | ✅ não há canal lateral | ✅ lista por tile, limitada por memória | ❌ o fragment ainda precisa do caminho | ✅ **cada segmento contribui sozinho** |
| quebra **(C)**? | ✅ não há eleição | ❌ | ❌ | ✅ **blending aditivo é order-independent** |
| é o que o Enio pediu? | ✅ literalmente | — | ⚠️ é evolução, não motor novo | ✅ mesma lei de tinta, sem carimbar dabs |
| resolução-independente | ⚠️ o spacing dos dabs entra | ✅ | ✅ | ✅ |

### 3.1 ⭐ C4 — a integral de arco acumulada por BLENDING ADITIVO

**A correção que quero submeter ao Enio:** o handoff §5 avalia a integral analítica e conclui que ela
*"quebra (A), **mantém (B)** (o fragment continua precisando que lhe contem o caminho) e mantém
(C)"*. Isso é verdade **se e somente se** a integral inteira for avaliada DENTRO de um fragmento.

Mas a integral é uma **SOMA sobre pedaços de caminho** — e é exatamente essa aditividade que remove
a necessidade do canal lateral:

```
τ(p)  =  Σ_segmentos  ∫_seg  f(d(s,p)) ds        com  f(d) = −ln(1 − dab(d))
α(p)  =  1 − exp(−τ(p))
```

⇒ **cada segmento desenha o próprio quad com `BlendOperation::Add` num alvo de canal único** (a
*espessura óptica* `τ`), sem saber que existem vizinhos, e um passe de resolve final aplica
`1 − exp(−τ)`.

- **(B) morre:** não existe `seg_extras`, não existe `neighbors.rs`, não existe teto. O comprimento
  de caminho que influencia um pixel é ilimitado por construção.
- **(C) morre:** soma é comutativa e associativa ⇒ **nenhum depth, nenhuma eleição, nenhuma ordem**.
  O `self_overlap` deixa de ser um modo com defeito e passa a ser o comportamento natural (cruzar o
  próprio traço soma duas vezes — que é o que a tinta faz).
- **(A) morre:** é o limite contínuo da fileira de dabs, então quina, cruzamento e fronteira de
  segmento **compõem sozinhos**. A partição de passagem e a dicotomia união/composição — as duas
  peças que custaram as rodadas 3-6 do cadáver — **deixam de existir**.
- **Custo:** *menos* fragmentos que hoje (um quad por segmento, sem lista lateral) e muito menos que
  C1 (que carimba um dab a cada `0,1·diâmetro` de arco). Sem `sqrt` de lista, sem `min` sobre 16
  cápsulas.

**Os riscos, nomeados antes de construir:**

1. **Precisão do alvo `τ`.** Precisa de `r16float`/`r32float`; `τ` satura rápido no miolo
   (`α → 1`). O `hardness = 1` tem de sair **byte-idêntico** (§8 do handoff, o CONTROLE de todos os
   smokes) — e ali a lei é binária, então o caminho duro provavelmente **não** passa pela integral:
   é um `if` no resolve, e isso é uma decisão a tomar, não um detalhe.
2. **O oráculo é uma soma FINITA, a integral é o limite contínuo.** O `painter_deposit_sized` compõe
   dabs a `0,1×diâmetro` de arco. A integral é o limite desse processo — logo há uma diferença
   sistemática pequena e *medível* entre os dois. Ela precisa ser quantificada antes de virar
   critério de aceitação, senão o gate mede o limite e o produto mede a soma.
3. **Um `τ` por TRAÇO, `over` entre traços.** Dentro do traço a lei é aditiva; entre traços é `over`
   (cores e opacidades diferentes). Isso traz o mesmo problema de lote do C1 (abaixo) — **é o item
   caro dos dois, e é a pergunta que decide a wave.**
4. **`exp` no resolve** — transcendental na GPU, o que é aceitável (o airbrush já o tem lá, `03 §8`,
   e o HR-5 governa o caminho determinístico da CPU, não o shader).

### 3.2 C1 — o buffer de dabs (o que o Enio pediu ao pé da letra)

É o que GIMP, Krita, Procreate, Photoshop e **o nosso próprio Painter** fazem, e a arquitetura já
existe dentro do repo. Quebra as três propriedades. **Contra:** carimbar dabs a `0,1·diâmetro` de
arco é muito mais fragmento que um quad por segmento; o **spacing vira parâmetro do motor** (a lei
volta a depender de quão fino o motor amostrou o caminho — exatamente o que a
`sampling_invariance` proíbe, e o que matou o motor atual); e a lei herda o dilema medido do §2.4
(taxa endurece · `max` conta contas), com a terceira via ainda por medir.

### 3.3 A pergunta que decide C1 **e** C4: o custo por frame

O Flip é vetorial e **re-rasteriza tudo a cada frame, em qualquer zoom**, com N traços, ghost frames,
multiplano e fill. Os dois candidatos precisam de um alvo intermediário **por traço** (é o escopo da
lei), e o modo ingênuo — um clear + um composite por traço — é N passes por frame.

**As saídas a investigar, em ordem de preferência:**

1. **Scissor pela bbox do traço.** O alvo intermediário não é o canvas: é o retângulo de tela do
   traço. O custo extra vira ~1 passada a mais sobre a área que o traço **já** cobre hoje ⇒ da ordem
   de **2× o fill rate atual**, mais N render passes pequenos.
2. **Lote por não-sobreposição.** Traços cujas bboxes não se cruzam podem dividir o mesmo alvo e o
   mesmo resolve ⇒ o N cai para o número de *grupos que se sobrepõem*.
3. **O `TessCache` que já existe** (por-desenho, no shell): arte commitada é cacheada e só o traço
   VIVO re-renderiza por frame. É o truque padrão, e já temos a estrutura.

⚠️ **Nada disto está medido.** É a primeira coisa que eu meço se o Enio aprovar — e é o
**kill-criterion** natural da wave (DIRETIVA §5: *"declare o kill-criterion ANTES do build"*).

---

## 4. RECOMENDAÇÃO

**Construir o C4** (integral de arco acumulada por blending aditivo), com o C1 como plano B
declarado.

Porquê, em três linhas:

1. É o **único candidato que quebra as três** propriedades do §3, e quebra (B) e (C) *pela
   representação*, não por um remendo — a aditividade **apaga o caso especial**, que é o padrão que
   este repo já premiou várias vezes.
2. Entrega a lei da tinta que o Enio pediu (o depósito do Painter) **sem** herdar o parâmetro de
   spacing nem o dilema de lei do §2.4 — e transforma o `self_overlap` de *defeito aberto de 43/255*
   em comportamento natural.
3. É **mais barato** que o C1 em fragmentos, e os dois pagam exatamente o mesmo pedágio de
   arquitetura (o alvo por traço), então o pedágio não é argumento a favor do C1.

**A ordem de execução que eu proponho, se aprovado:**

| # | passo | o que decide |
|---|---|---|
| 0 | **Medir o pedágio** (§3.3) num protótipo mínimo: N traços, bbox-scissor, 2 passes | **kill-criterion**: se o frame não couber no orçamento a 4K, a forma do motor muda antes de qualquer linha de lei |
| 1 | Quantificar a diferença **integral × soma finita** contra `painter_deposit_sized` | fixa o critério de aceitação honesto (risco 2 da §3.1) |
| 2 | O kernel `τ` + resolve, com `hardness = 1` byte-idêntico | o CONTROLE de todos os smokes (§8) |
| 3 | Reconstruir a bateria do §6 contra o motor novo | os oráculos são de COMPORTAMENTO e sobrevivem (§9 do handoff) |
| 4 | Só então: caps/joins, tips, e a wave de `self_overlap` que some sozinha | |

**O que eu NÃO recomendo:** portar o Ciallo esperando a cura (§2.1 — ele declara a limitação),
continuar na linhagem GP (§2.2 — segue quebrada no 5.0), ou adotar o `Soft` do Drawpile sem medir o
beading (§2.4 — o ponto fixo dele é o `max` que já foi reprovado na tela).

---

## 5. Fontes

- Ciao, S. & Wei, L.-Y. — *Ciallo: GPU-Accelerated Rendering of Vector Brush Strokes*, SIGGRAPH 2024.
  [ACM](https://dl.acm.org/doi/10.1145/3641519.3657418) ·
  [CIS Lab](https://cislab.hkust-gz.edu.cn/publications/ciallo-gpu-accelerated-rendering-of-vector-brush-strokes/) ·
  [tutorial do autor](https://shenciao.github.io/brush-rendering-tutorial/)
- Levien, R. et al. — *GPU-friendly Stroke Expansion*, [arXiv:2405.00127](https://arxiv.org/html/2405.00127v1)
- Blender — [#154433](https://projects.blender.org/blender/blender/issues/154433) (opacity/hardness,
  5.0.1 e 4.2.18 LTS) · [corner overlap artifacts](https://devtalk.blender.org/t/grease-pencil-corner-overlap-artifacts/3032)
- Krita — [Opacity and Flow](https://docs.krita.org/en/reference_manual/brushes/brush_settings/opacity_and_flow.html) ·
  [Soft painting mode](https://krita-artists.org/t/feedback-wanted-soft-painting-mode/167535) (Drawpile)
- In-repo, **não re-derivar**: [`docs/Painter/25 §13.9–§13.13`](../Painter/25_avaliacao_gpu.md) ·
  [`docs/Flip/03 §8.6–§8.7.2`](03_traco_rasterizacao.md)
