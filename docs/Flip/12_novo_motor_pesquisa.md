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

**A ordem de execução que eu proponho, se aprovado** — ⚠️ os passos 0 e 1 **já rodaram** (§5, §6) e
mudaram o resto da tabela:

| # | passo | estado |
|---|---|---|
| 0 | **Medir o pedágio** do alvo por-traço | ✅ **FEITO** (§6). A bbox morreu (67 telas/frame); a granularidade é **TILE**, e com ela o alvo por-traço **deixa de existir** |
| 1 | Quantificar **integral × soma finita** contra `painter_deposit_sized` | ✅ **FEITO** (§5). Corpo a **±2/255**, cruzamento incluso; densidade exatamente constante; `sub = 4` |
| 2 | O **binning por tile** + o walk por-tile (o esqueleto, sem lei) | o esqueleto que o §6.3 desenhou; é onde o `neighbors.rs` morre |
| 3 | O kernel `τ` + resolve dentro do walk, com `hardness = 1` byte-idêntico | o CONTROLE de todos os smokes (§8 do handoff) |
| 4 | Reconstruir a bateria do §6 do handoff contra o motor novo | os oráculos são de COMPORTAMENTO e sobrevivem |
| 5 | **Caps e joins como primitivo** (§5.5) — deixou de ser risco e virou escopo | + tips; o `self_overlap` some sozinho |

**O que eu NÃO recomendo:** portar o Ciallo esperando a cura (§2.1 — ele declara a limitação),
continuar na linhagem GP (§2.2 — segue quebrada no 5.0), ou adotar o `Soft` do Drawpile sem medir o
beading (§2.4 — o ponto fixo dele é o `max` que já foi reprovado na tela).

---

## 5. ⭐ A LEI, MEDIDA (passo 1 executado — `tests/integral_law.rs`, CPU, sem adapter)

O passo 1 do §4 era o que podia **matar o C4 antes de uma linha de GPU**: a integral contínua
reproduz o depósito FINITO do Painter, que é o oráculo? Rodei. O resultado decide, e não é o que o
número agregado sugere.

`cargo test -p ph2d-flip-render --release --test integral_law -- --ignored --nocapture`

### 5.1 O agregado assusta — e mente sobre onde o erro mora

Contra `painter_deposit`, o pior desvio global vai de **−47 a −101** (reta, quina, cruz), e cresce com
a dureza. Se eu tivesse parado aqui, teria reprovado a lei.

### 5.2 A separação que responde: TAMPA · JUNTA · **CORPO**

| figura | h | TAMPA | JUNTA | **CORPO** |
|---|---|---|---|---|
| reta | 0,0 / 0,4 / 0,7 | −47 / −66 / −101 | +0 / +0 / +0 | **+0 / −1 / −1** |
| quina | 0,0 / 0,4 / 0,7 | −47 / −65 / −100 | +2 / +4 / +11 | **−1 / +1 / +2** |
| **cruz** | 0,0 / 0,4 / 0,7 | −47 / −63 / −100 | +5 / +9 / +31 | **−1 / +1 / −1** |
| estrela | 0,0 / 0,4 / 0,7 | −11 / −18 / −45 | +5 / +9 / +31 | **+1 / −1 / −1** |

⭐ **O CORPO do traço bate o depósito do Painter em ±2/255 — na reta, na curva, na quina e no
AUTO-CRUZAMENTO.** O cruzamento é o defeito que custou a saga inteira, e a integral o acerta **por
construção**: não há partição de passagem, não há dicotomia união/composição, não há lista de
vizinhos. A soma simplesmente soma.

### 5.3 A lei é fato do CAMINHO — exatamente, não aproximadamente

A MESMA estrela, de `0,80·r` a `0,04·r` de passo (60 → 1155 segmentos):

```
h=0.4:  -18  -18  -18  -18  -18  -18      ← constante, 6 densidades
h=0.7:  -45  -45  -45  -45  -45  -45
```

**Zero variação.** É a propriedade que o `sampling_invariance.rs` pina e que o penhasco do motor
atual quebrou. Aqui ela não é um conserto: é o que a integral por arco **é**. (Os −18/−45 são o
resíduo de tampa+junta da §5.2, que não se movem com a densidade.)

### 5.4 Quadratura: `sub = 4` já satura

`quina h=0,4`: `1→−73 · 2→−67 · 4→−65 · 8→−65 · 16→−65 · 32→−65 · 64→−65`. **Quatro sub-amostras por
pitch** bastam — é o custo do kernel, medido e não escolhido.

### 5.5 ⚠️ A TAMPA é GEOMETRIA, não um termo — e a medição matou minha própria hipótese

Euler–Maclaurin prevê `Σ g(kh) = (1/h)∫g + ½(g(0)+g(L))`, ou seja **meio dab em cada extremo**.
Implementei. **Overshoot:** −101 → **+87**; −47 → +39.

Varri o coeficiente `k` medindo **na região da tampa** (o pior global muda de lugar quando o termo
entra — medir o global aqui mentiria):

```
reta   h \ k    0.00    0.15    0.25    0.35    0.50
       0.4       -66     -46     -34     +40     +54
       0.7      -101     -70     -54     +68     +87
```

⚠️ **O erro salta de −54 para +40 sem passar por zero.** Não existe `k` que feche: o pior pixel
está **trocando de lugar**, não encolhendo. A conclusão é estrutural, não numérica — **a fileira
FINITA do Painter põe um DISCO no primeiro dab, e a integral contínua tem uma ponta MACIA. São
formas diferentes, não amplitudes diferentes.**

**E isto não é dívida nova do C4.** É a MESMA pergunta que (a) o módulo já tem na fila como wave
dedicada de **joins & caps** (`03 §8`) e (b) o motor ATUAL também erra hoje — a baseline da §1 mede
o resíduo dele exatamente ali: **+140 no tip convexo, 166 px, com 0 px faltando no corpo**.

### 5.6 O que isto muda no plano

| | antes | depois da medição |
|---|---|---|
| a LEI | hipótese | ✅ **validada no corpo a ±2/255, cruzamento incluso** |
| densidade | hipótese | ✅ **exatamente constante** em 6 densidades |
| quadratura | desconhecida | ✅ `sub = 4` |
| caps | "risco 2, medir" | ⚠️ **escopo explícito**: cap é primitivo geométrico, entra na wave |
| juntas | não previsto | ⚠️ **+31 em h=0,7** — aberto, mesma família do cap |

⚠️ **Um número que ainda não tem explicação e não vou vender como se tivesse:** a junta cresce com a
dureza (+2 → +11 → +31). Cai na mesma investigação do cap.

⚠️ **E um defeito do ORÁCULO que a varredura expôs de graça:** em `h = 0,9` os desvios explodem
(+170, +250) em pixels na PONTA do caminho. O `painter_deposit` caminha por `pitch` e o último dab
cai **até um pitch antes do fim do caminho** — então o oráculo não pinta a ponta que o caminho tem.
Não é a lei: é a fronteira do oráculo, e qualquer gate de aceitação em `h ≥ 0,9` tem de saber disso.

---

## 6. ⭐ O PEDÁGIO, MEDIDO (passo 0 executado — `tests/architecture_toll.rs`, CPU)

O kill-criterion declarado no §4: os dois candidatos precisam de um **alvo intermediário por
traço**, e o Flip re-rasteriza tudo a cada frame. Cabe no orçamento?

⚠️ **A medição matou a mitigação nº 1 da §3.3 — o `set_scissor_rect` pela bbox do traço, que eu
tinha proposto como a saída preferida.**

### 6.1 O desperdício da bbox num gesto só (tela 1920×1080, r = 6 px)

| caso | fita (px) | bbox (px) | **bbox/fita** | tile64/fita |
|---|---|---|---|---|
| horizontal curto | 2 512 | 2 544 | **1,0×** | 13,0× |
| **DIAGONAL de canto a canto** | 25 872 | 1 990 384 | **76,9×** | 8,7× |
| arco amplo (o gesto de animação) | 23 066 | 707 056 | **30,7×** | 8,2× |

**A bbox de um traço diagonal é a tela inteira.** Num traço curto ela é perfeita (1,0×) — e é por
isso que a ideia parecia boa: ela só falha na figura que o módulo existe para desenhar.

### 6.2 O pedágio agregado, em TELAS CHEIAS por frame

| cena | n | fita | **bbox** | tile64 | **tile16** |
|---|---|---|---|---|---|
| curtos (hachura) | 200 | 0,14 | 0,62 | 1,86 | **0,47** |
| **LONGOS (gesto)** | 10 | 0,10 | **3,39** | 0,86 | **0,28** |
| **LONGOS (gesto)** | 50 | 0,46 | **16,12** | 4,08 | **1,34** |
| **LONGOS (gesto)** | 200 | 1,86 | **67,43** | 16,52 | **5,44** |

⛔ **67 telas cheias por frame** com 200 gestos. O alvo por-traço scissorado pela bbox **não é
viável** — e não é afinável, porque o número é geometria pura.

✅ **Em granularidade de TILE de 16 px o pedágio é ~3× a fita** (5,44 contra 1,86; e 0,47 contra
0,14 nos curtos), consistente nas duas cenas. Tile de 64 é grosso demais (16,52) — o ladrilho grande
desperdiça em traço fino.

### 6.3 ⭐ A conclusão que reescreve a arquitetura: **C2 e C4 não são alternativas — o C2 é como o C4 roda**

Se a granularidade tem de ser o TILE, então o alvo intermediário **por-traço deixa de existir**, e
com ele o kill-criterion inteiro:

> Um renderizador **binado por tile** percorre, para cada ladrilho, a lista de traços que o tocam,
> **em ordem de z**. Para cada traço ele acumula `τ` sobre os segmentos daquele traço naquele
> ladrilho, resolve `α = 1 − exp(−τ)` e **compõe `over`** no acumulador do ladrilho — tudo em
> **registradores**, dentro de **UM dispatch**.

Isso apaga, de uma vez:

- o **alvo por traço** (não há textura de scratch, não há clear, não há N render passes);
- o **teto (B)** — a lista por tile é limitada por memória, não por uma constante;
- a **eleição (C)** — dentro do traço a soma é comutativa; entre traços a ordem é o z, que é
  explícito e não um depth test;
- e o `neighbors.rs` inteiro (587 linhas + 397 de teste), que existia só para contar ao fragment
  qual caminho está perto.

**É o modelo do Vello aplicado à nossa lei de tinta** — e é por isso que o §2.3 concluiu que o
stroke expansion é *"uma peça, não um motor"*: ele é a peça de **execução**, e a lei da tinta
(§5) é a peça que faltava a ele.

### 6.4 O risco que sobra, nomeado

O custo por pixel passa a ser *"quantos segmentos deste traço estão ao alcance"*, e numa mão lenta
isso é grande (o `flip_lenta` tem **1065 segmentos**). ⚠️ **Isso afeta a VELOCIDADE, nunca o
resultado** — a §5.3 mediu o desvio exatamente constante em 6 densidades —, o que já é uma inversão
do motor atual, onde a densidade corrompia a *tinta*. A mitigação é fundir segmentos quase-colineares
no `pack` (o `7ca83d6fb` já funde cápsulas hoje), e ela **não é pré-requisito**: é afinação.

---

## 7. ⭐ O ESQUELETO BINADO (passo 2 executado — `src/binning.rs`, CPU, sem adapter)

O §6.3 desenhou; isto constrói. `binning.rs` (410 linhas) + os 11 gates do irmão.

### 7.1 O que existe agora

| peça | o que faz |
|---|---|
| `ScreenSpace` | mundo → px, espelhando os 3 números da `CameraRaw`; **porta única do RAIO** (com o piso do shader) |
| `bin_segments` | percorre traços em ordem de `sid` e segmentos em ordem de caminho, depositando cada um nos ladrilhos que ele **alcança** |
| `TileBins` | `[offset, count]` por ladrilho + a lista concatenada; ordem **(traço, segmento)** estável |
| `walk_pixel` | lê a lista do ladrilho, **agrupa por traço com um scan de run** e compõe `over` em ordem de z |

**Duas decisões que apagam casos especiais:**

1. **O alcance é medido da CAIXA do ladrilho, nunca do centro.** A caixa contém todo pixel do
   ladrilho ⇒ `dist(seg, caixa) ≤ r` já inclui todo segmento capaz de influenciar qualquer pixel
   dali ⇒ **o percurso não precisa de halo**: nenhuma lista de vizinho, nenhum caso de borda.
   (Este é literalmente o buraco que o `neighbors.rs` inteiro existe para tapar.)
2. **A ordem de saída sai de graça.** Os `sid` já crescem com o z e os segmentos já vêm em ordem
   de caminho; o depósito é um **counting-sort**, que é estável ⇒ **zero ordenação**, e o
   agrupamento por traço é um scan de run.

### 7.2 As duas propriedades do §3 que MORRERAM aqui

- **(B) o teto:** gate `the_bin_has_no_fixed_ceiling` — 24 traços atravessando o MESMO ladrilho, e
  os 24 estão na lista (o `MAX_EXTRAS_PER_SEGMENT = 16` de hoje truncaria em 16).
- **(C) a eleição:** gate `the_walk_composes_the_later_stroke_on_top` — a ordem é o percurso, não
  um depth test. ⚠️ A mutação que reinstala *first-wins* (o `DEPTH_GREATER` estrito) sangra nele.

### 7.3 O oráculo, e o que ele NÃO afirma

O gate central é **`the_binned_walk_is_the_brute_force_walk`**: a lista acelerada tem de dar a
**mesma imagem** que a lista completa, pixel a pixel. Um binning é estrutura de aceleração — a
única coisa que ele pode fazer de errado é mudar a resposta.

⚠️ **Não há lei de tinta aqui, de propósito.** O `stroke_deposit` resolve a **união dura**
(`dist ≤ r`) = a semântica de `hardness = 1` **sem anti-aliasing**. O passo 3 substitui **só essa
função** pela integral `τ` da §5; binning, agrupamento e composição não mudam.

### 7.4 ⚠️ Duas mutações sobreviveram, e as duas expuseram o LADRILHO como anestésico

Das 9 mutações, 7 sangraram de primeira. As 2 que passaram são a lição desta wave:

| mutação | por que sobreviveu |
|---|---|
| alcance `min` em vez de `max` | o traço que afina corria **ao longo** de uma linha de ladrilhos ⇒ distância 0 à caixa ⇒ os dois raios binam **exatamente as mesmas tiles** |
| binner contorna a porta do raio (perde o piso 0,65 px) | as fronteiras de ladrilho mais próximas estavam a 6 e 10 px; a janela onde 0,65 e 0,1 px **diferem** tem **0,14 px** |

**Um ladrilho de 16 px engole toda diferença de alcance menor que ele** — então um gate de
comportamento sobre a lista é *cego* a erros sub-ladrilho, por construção.

As curas são **diferentes**, e é isso que importa:

- a do `min`/`max` **é fixture**: o traço que afina foi para **4 px de uma fronteira**, onde o raio
  grosso (6) alcança a coluna vizinha e o fino (0,65) não. Agora sangra.
- a do piso **não é fixture** — seria um gate cujo oráculo vive numa janela de 0,14 px, exatamente
  a classe que este repo já aprendeu a desconfiar. Virou **arch-gate**
  (`the_binner_asks_the_screen_for_the_radius`: o binner tem de PERGUNTAR à porta única e não pode
  conter `px_per_world`) + um gate direto do piso. As duas mutações sangram agora.

**9 mutações, 9 sangram.**

### 7.5 O que falta (passo 3)

Trocar `stroke_deposit` pela integral: `τ += f(dn) · Δs/pitch` sobre os segmentos do run,
`α = 1 − exp(−τ)`, com `sub = 4` (§5.4). O agrupamento já entrega exatamente a lista certa: **os
segmentos daquele traço que alcançam aquele pixel**, sem teto e sem ordem imposta — que é
precisamente o que a lei aditiva precisa e o que o motor de hoje não consegue entregar.

---

## 8. ⭐ A LEI ENTROU NO PERCURSO (passo 3 — `src/tau.rs`)

### 8.1 ⚠️ A descoberta que reescreve o enunciado: **a lei não é nova**

Indo portar a integral, fui ler o `hardness_mask` do `flip.wgsl` — e ele **já compõe uma fileira
de dabs por `over`**, devolvendo `1 − Π(1 − w_k)`. Tome o log:

```text
  1 − α = Π (1 − w_k)  ⇒  −ln(1 − α) = Σ −ln(1 − w_k) = Σ f(d_k)  ⇒  α = 1 − exp(−τ)
```

**O motor de hoje JÁ calcula `α = 1 − exp(−τ)`.** O que ele faz de errado é somar `τ` sobre uma
**RETA FICTÍCIA** que passa pelo ponto mais próximo (`d = √(dn² + along²)`, uma fileira infinita e
reta). Isso explica a baseline da §1 inteira, item por item:

| sintoma medido na §1 | o que a ficção faz |
|---|---|
| traço reto: **+1/255** | ali a ficção **é** a verdade |
| cruzamento errado | a reta fictícia **não tem cruzamento** para ver |
| ponta convexa: **+140/255**, e **zero** faltando | a ficção tem caminho **infinito** onde o real acaba |

⇒ o C4 não é "uma lei nova candidata". É **a mesma lei sobre a geometria que existe**. E a forma
de soma é o que a torna comutativa e sem teto — exatamente o que a lista por-ladrilho entrega.

### 8.2 O gate que é a entrega da wave

**`the_new_engine_reproduces_the_shipping_profile_on_a_straight_stroke`** (em
`tests/hardness_law.rs`, ao lado da lei do motor velho): num traço RETO as duas TÊM de coincidir,
porque é ali que a ficção é verdade.

| hardness | desvio contra o perfil que shipa |
|---|---|
| 0,10 | **0,27**/255 |
| 0,20 | 0,32 · 0,30 → 0,38 · 0,40 → 0,43 |
| 0,60 | 0,75 |
| 0,80 | **1,33** |

Sub-nível-de-byte na faixa toda. O resíduo **cresce com a dureza e vive perto do aro**
(`dn ≈ 0,83–0,93`) — a assinatura de **soma finita contra integral contínua** (a fileira do shader
para em `DEPOSIT_HALF = 4`, e dab mais duro é perfil mais estreito para ela truncar), não de duas
leis diferentes. ⚠️ **É este gate que pina o NÍVEL ABSOLUTO de tinta**: sem ele, esquecer o `pitch`
na integral escalaria a cobertura inteira e nenhum gate de forma piscaria (mutação ML3).

### 8.3 ⚠️ Três coisas que a medição corrigiu em mim

**(a) `p*p*(3−2p)` NÃO é `3p²−2p³` em `f32`.** Escrevi a curva do Painter na álgebra certa e na
**ordem errada**, e o gate de paridade nasceu vermelho em `dn = 0,01`. É literalmente a disciplina
*"as MESMAS operações na MESMA ordem"* que o doc-comment do WGSL prega — e eu a quebrei na 1ª
tentativa. Uma cópia por MOTOR é aceitável (a do velho morre com ele); uma cópia **sem gate**, não.

**(b) O gate de densidade estava medindo GEOMETRIA.** A 1ª fixture reamostrava uma senoide em 4 e
40 pontos — com 4 cordas aquilo é **outro desenho**, e o desvio (24/255) era a corda, não a lei.
A fixture certa **subdivide as MESMAS pernas retas**: geometria idêntica, amostragem diferente.

**(c) E aí ela falhou de novo, por 254,8/255 — no regime errado.** Em `hardness = 1` a cobertura é
um **DEGRAU**: a borda é resolvida até um passo de quadratura (~0,06 px), e um pixel cujo centro
cai nessa casca **flipa 255 de uma vez**. Medido, varrendo `SUB`:

| `SUB` | pior desvio, `h = 1` | pior desvio, `h = 0,4` |
|---|---|---|
| 1 | — | 3,60/255 |
| 2 | **254,82**/255 | < 1,0 |
| 4 | 1,06 | < 1,0 |
| 8 | < 1,0 | < 1,0 |

⇒ o gate de densidade roda **macio** (onde a lei é o assunto) e a metade dura tem gate PRÓPRIO
(`at_hardness_one_the_integral_is_the_hard_union`, que mede **onde** elas discordam: uma casca de
**< 0,75 px** em torno da silhueta). E `SUB = 4` deixa de ser "a §5.4 disse que satura": é o 1º
valor confortavelmente dentro da região plana **nos dois regimes**.

### 8.4 As duas mutações que sobreviveram, e o que elas nomearam

| mutação | por que passou |
|---|---|
| raio do MEIO em vez do interpolado | **nenhuma fixture da lei tinha largura variando** — e pressão é o caso normal |
| `opacity` dentro do `f` | **nenhuma fixture tinha opacidade ≠ 1** |

As duas curas são gates novos, e o 2º vale por si: **`opacity` multiplica DEPOIS da cobertura e
nunca entra no `f`** — é a regra do GP que o `flip.wgsl` documenta (*um traço a opacity 0,5 não
escurece sobre si mesmo*), e com ela dentro do `f` o cruzamento acumularia opacidade. Medido no
gate: braço **0,5000**, cruzamento **0,5000**.

**16 mutações no total (9 do binning + 7 da lei), 16 sangram.**

### 8.5 O que o motor novo já responde, e o que falta

✅ o cruzamento acumula por construção (gate: τ do cruzamento **> 1,2×** o de um braço) · ✅ a tinta
é fato do caminho, não da densidade · ✅ dureza 1 é a união dura · ✅ o perfil reto é o que shipa ·
✅ sem teto, sem depth, sem `neighbors.rs`.

**Falta (passo 5):** **caps e joins como primitivo** (§5.5 do handoff) · e o **port para GPU**,
onde este percurso vira um dispatch de compute por ladrilho. ⚠️ **Nada disto está ligado ao
produto ainda** — o motor velho segue intocado, e o novo não tem chamador de produção.

---

## 9. ⭐ A BATERIA DE ORÁCULOS CONTRA O MOTOR NOVO (passo 4)

O handoff §6 é explícito: *"não escreva oráculo novo antes de usar estes"*. Então nada aqui é
oráculo novo — a figura é o `star_path`, a referência é o `painter_deposit` (o depósito REAL do
Painter), a exclusão é o `in_the_silhouette_fringe`, e o gesto é o do Enio. O que muda é **quem
responde**: `walk_pixel` em vez do `FlipRenderer`.

⚠️ **Os quatro gates rodam HEADLESS.** Os irmãos que eles espelham são `#[ignore]` + adapter, e
por isso não correm na varredura normal — que é exatamente onde uma regressão destas leis tem de
aparecer.

### 9.1 O oráculo do Enio mede ZERO

| | pior desvio | px fora de ±8 |
|---|---|---|
| motor de HOJE (`measure_the_star_one_stroke_against_separate_strokes`, h=0,4) | **−64/255** | **154** |
| motor NOVO (`the_new_engine_makes_a_self_crossing_stroke_equal_separate_strokes`) | **0** | **0** |

A estrela desenhada **sem levantar a caneta** contra as mesmas cinco pernas como traços separados,
em h = 0,4 · 0,7 · 1,0. E o zero não é "pequeno o bastante", é uma **identidade**:

```text
  um traço, duas passagens:  α = 1 − exp(−(τ₁+τ₂))
  dois traços, `over`:       α = 1 − (1−a₁)(1−a₂) = 1 − exp(−τ₁)·exp(−τ₂)
```

⚠️ **E ela é mais forte do que parece:** a integral **não sabe onde um traço termina**. Partir o
caminho em cinco pedaços é partir o DOMÍNIO de uma integral, e isso não muda a integral ⇒ **não há
primitivo de JUNÇÃO a construir** para este caso. (O cap da PONTA é outra coisa — §9.3.)

### 9.2 A ponta convexa: +140/255 → +14

O gate do motor de hoje precisa admitir **+140/255** de tinta a MAIS no vértice de 36°. Integrando
sobre o caminho que existe, o excedente colapsa — e `n_sobra` é **0** em todas as durezas:

| hardness | 0,1 | 0,2 | 0,3 | 0,4 | 0,5 | 0,6 | 0,7 | 0,8 | 0,9 |
|---|---|---|---|---|---|---|---|---|---|
| sobra | +5 | +6 | +7 | +8 | +9 | +11 | **+14** | +13 | +0 |

### 9.3 ⚠️ O NEGATIVO: o déficit que apareceu, e de que ele é feito

Trocar a ficção pelo caminho real matou o excedente e **deixou um déficit** que o motor de hoje não
tinha. Ele é de DUAS coisas, e a medição as separa:

| zona | pior | n | o que é |
|---|---|---|---|
| tudo | **−36** | 16 px | inclui a PONTA do traço |
| cego a um disco `r` nas PONTAS | **−27** | ≤3 px | só as quinas convexas |

1. **A PONTA (o cap).** O depósito do Painter carimba um dab **no primeiro ponto**
   (`painter_deposit_sized` abre com `vec![pts[0]]`); a integral não tem caminho além do fim. É o
   passo 5.
2. **A QUINA convexa.** O Painter compõe **dabs discretos** a `0,1·diâmetro`; a integral é o limite
   denso da mesma composição, e numa quina de 36° a discretização dele deposita um pouco mais.
   ⚠️ **NÃO é a quadratura, e isto foi MEDIDO:** subindo `SUB` de 4 para 16 o número anda ≤1/255
   (−20→−19 · −24→−24 · −27→−26). Casar isto exigiria reproduzir a discreteza do Painter, que é
   **outro motor** (o candidato C1, o buffer de dabs).

O gate `the_new_engines_deficit_is_the_endpoint_and_the_corner_and_these_are_its_numbers` **pina o
defeito** para o número não virar folclore, e a terceira asserção dele é uma **tripwire**: no dia
em que o passo 5 fechar o cap, ela fica vermelha pedindo os números novos.

### 9.4 A invariância de densidade é ESTRUTURAL

`the_new_engine_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled` — a MESMA estrela de
`0,80·r` a `0,04·r` (45 → 885 segmentos, **20× de subdivisão**):

```text
  -31  -31  -31  -31  -31  -31      ← o mesmo número, ao byte
```

⚠️ **A barra deste gate NÃO é a do irmão de GPU (−24), e a diferença é nomeada:** o −31 constante é
o déficit da §9.3 servindo de FUNDO; quem julga aqui é a segunda asserção (*a resposta não anda*).
No motor de hoje a invariância teve de ser conquistada contra uma constante (`MAX_EXTRAS_PER_SEGMENT`);
aqui não existe constante para correr contra — a lista por ladrilho é limitada por memória, e
subdividir o domínio de uma integral não a muda.

### 9.5 As mutações — 4, e as quatro sangram

| # | mutação | sangra |
|---|---|---|
| A | cobertura **linear** (`tau.min(1)`) em vez de `1−exp(−τ)` | identidade · ponta |
| B | `d_tau = fv` (sem o passo de arco ⇒ tinta = função da CONTAGEM) | ponta · déficit |
| C | **teto de 64** por ladrilho (a propriedade (B) reinstalada) | invariância (**as duas metades**) · déficit |
| D | **cap** de meio dab nas pontas (o passo 5 antecipado) | os três do `painter_look` |

⚠️ **B sobrevive ao gate de invariância — por SATURAÇÃO** (τ fica ~4× maior, o alfa satura em 1 nas
duas densidades e o desvio *deficitário* não se move). Não é buraco: B sangra em dois outros gates,
e o defeito da CLASSE deste gate é o C. ⚠️ **A 1ª versão do C usava teto 16 e matava tudo** (−255 em
TODA densidade) — ela fazia a BARRA disparar, não a invariância; com **64** o retrato é o do defeito
histórico (`0,8 → −31 · 0,4 → −31 · 0,2 → −255 · …`) e **as duas** asserções sangram.

⚠️ **E a mutação D achou que um cap ingênuo quebra a identidade da §9.1** (cinco traços têm dez
pontas, um traço tem duas). ⛔ **A conclusão que eu tirei disso — *"qualquer cap tem de ser
invariante à partição do caminho"* — está ERRADA, e quem a refutou foi a própria referência**
(§13.3): o depósito do Painter **não é invariante** (medido: −59/255 em 178 px · −102 em 123 ·
−255 em 17, sempre NOS CANTOS). A identidade que a §9.1 mediu em zero era artefato de o motor
**ainda não ter cap nenhum**. Ela vale onde o caminho é o MESMO — o CRUZAMENTO, que é o que o
oráculo do Enio de fato pergunta — e o gate foi re-escopado para dizer isso.

---

## 10. ⭐ O CUSTO POR FRAME (o 2º eixo do §11.3 do handoff — `tests/walk_perf.rs`)

O §6 mediu o pedágio de **ÁREA**; isto mede **o que custa um pixel**. ⚠️ **O número serial de CPU
não é o número do produto** (o alvo é um dispatch de compute por ladrilho, onde cada pixel é uma
thread) — o que ele responde, e o produto herda, é a **FORMA**.

### 10.1 A forma está certa: o custo é LOCAL

Um pixel de um canto VAZIO, com a cena crescendo do outro lado da tela:

| traços | segs | canto vazio | faixa densa |
|---|---|---|---|
| 1 | 39 | **3,3 ns** | 423 ns |
| 10 | 390 | **3,3 ns** | 834 ns |
| 50 | 1950 | **3,4 ns** | 1678 ns |
| 200 | 7800 | **3,4 ns** | 4147 ns |

**Plano de 1 a 200 traços.** É o requisito que decide o desenho, e ele vale em qualquer
dispositivo: um pixel só paga pelos segmentos que o ALCANÇAM. A faixa densa sobe porque ali os
traços de fato se empilham (o fixture os sobrepõe) — trabalho legítimo, ~400 ns por traço que
cobre o pixel.

### 10.2 Um frame de 1080p, serial

| traços | segs/tile (méd \| máx) | bin | walk | ns/px | ns/px COM TINTA | tinta |
|---|---|---|---|---|---|---|
| 1 | 0,04 \| 2 | 0,03 ms | 18,4 ms | 9 | 844 | 1,1% |
| 10 | 0,38 \| 6 | 0,09 ms | 93,4 ms | 45 | 476 | 9,5% |
| 50 | 1,87 \| 11 | 0,34 ms | 412,8 ms | 199 | 555 | 35,9% |
| 200 | 7,60 \| 34 | 1,30 ms | 1593,7 ms | 769 | 1004 | 76,6% |

O **binning** (1× por frame) é barato: **1,3 ms** para 7800 segmentos. O percurso é o custo.

### 10.3 De que o pixel é feito, e as alavancas MEDIDAS

- **O logaritmo é ~20%, não o dominante.** Ablação (`f_of` devolvendo `w` cru): 4147 → **3170 ns**.
  Uma LUT de `f_of` — o padrão que o Painter já pagou no doc 24 — compra ~20%, não 5×.
- **O dominante é o NÚMERO DE AMOSTRAS**, governado por `SUB`. ⚠️ **E não há 2× de graça:** a
  bateria inteira do §9 foi rodada em `SUB` 2 · 3 · 4, e **2 e 3 REPROVAM** — o déficit da quina
  convexa sai de −27/3 px para **−30/11 px**. Para cima, 4 → 16 move ≤1/255 (§9.3). **`SUB = 4` é
  o joelho, agora defendido pelos oráculos e não por uma sonda só.**

### 10.4 ⚠️ Duas lições de SONDA, as duas minhas

1. **`black_box` vai na ENTRADA, dentro do laço.** Com ele só no fim, `walk_pixel` é pura sobre
   argumentos invariantes e o LLVM a computa uma vez: a 1ª versão mediu **3,3 ns no canto E na
   faixa densa** — o retrato de nada rodando.
2. **A coordenada de tela nunca se escreve à mão sob uma câmera Y-FLIPADA.** Eu cravei
   `y = h·0,6` como px de tela; a câmera espelha, a "faixa densa" caiu em espaço VAZIO, e as duas
   colunas mediram o mesmo nada — *com a correção do (1) já aplicada*, que é o que torna o erro
   difícil de ver. Os dois pontos agora são projetados por `ScreenSpace::point_px`, e **o alfa vai
   impresso ao lado** para uma sonda que não encosta na tinta se denunciar.

---

## 11. ⭐ O ANTI-ALIASING — o ponto cego que os oráculos criaram (fechado)

Todos os gates contra o depósito do Painter **excluem a franja da silhueta**, e com razão (o
depósito não tem AA nenhum). O preço: o motor novo **nunca tinha sido comparado na borda**, e não
tinha AA. O oráculo novo é a **ÁREA** que a silhueta cobre no pixel (super-amostragem 16×16).

⚠️ **Super-amostrar aqui é legítimo, e a proibição do §6 do handoff é sobre outra coisa:** lá ela
protege o oráculo do *depósito do Painter*, que também amostra no centro do texel — super-amostrar
mediria uma verdade que nenhum dos dois computa. Aqui a pergunta **é** *que fração do pixel a
silhueta cobre*, e área é a definição dela.

### 11.1 O `edge` é o do shader, sem a derivada de tela

O `flip.wgsl` usa `clamp(0.5 + (1 − dn)/aa, 0, 1)` com `aa = fwidth(dn)`. Como `dn = d/r` e um
pixel vale `1/r` em `dn`, o termo `(1 − dn)/aa` **é** `r − d`: a distância com sinal em PIXELS.
A mesma expressão aqui é `0.5 − sd` — e o `sd` é **exato**, porque o percurso tem os segmentos na
mão. (O shader precisa do `fwidth` de um `min`, que **salta na costura**, e por isso o AA de lá é
por-PASSAGEM; o comentário dele registra o preço.)

### 11.2 ⚠️ E aí o `edge` sozinho não funcionou — o achado da wave

Em `hardness = 1` a integral **morre no instante em que o centro do pixel sai da silhueta** (a
janela de cada segmento fica vazia ⇒ `τ = 0`), então a meia-borda de FORA ficava em zero:
**−127/255 contra a área**. O `flip.wgsl` escapa disso com um **ramo** (`profile = 1.0`
incondicional quando a borda é dura) e paga o preço de o perfil e o AA serem duas leis.

A resposta melhor **apaga o caso especial**: amostre o perfil **meio pixel DENTRO da silhueta**.
Um mecanismo, os dois regimes — em dureza 1 o perfil ali é 1 (⇒ a máscara vira o `edge`, como no
shader) e num pincel macio ele já é ~0 (⇒ a máscara continua ~0, como no shader).

⚠️ **E o número 0,5 não foi afinado até passar.** Empurrar *de leve* (`1e-3`) **não funciona**: a
corda amostrada fica quase tangente (meio comprimento `√(2rε)` ≈ 0,12 px contra um passo de
quadratura de 0,35) e a integral não pega amostra nenhuma — medido, **−98/255**. O 0,5 é a
meia-largura do próprio filtro-caixa; a varredura `0,25 · 0,5 · 0,75 · 1,0` só confirma que a
vizinhança concorda (**−9/255** em todas), porque em dureza 1 o perfil é chapado.

**Resultado: pior desvio −9/255 contra a área, 0 px fora de 24**, num traço a **30°** (borda
alinhada aos eixos é exata em qualquer filtro-caixa e esconderia o erro) com as pontas **fora da
tela** — a 1ª versão do fixture as deixou dentro e mediu **−156**, que é o **cap** do passo 5
reportado com o nome da borda.

### 11.3 ⚠️ O preço: a identidade do §9.1 deixou de valer NA FRANJA

`over` de dois alfas com AA **não** é o AA da união — é o artefato de **conflação**, e quem está
CERTO é o traço único. O gate da identidade teve de ser re-escopado **duas vezes**, e cada tentativa
frouxa deixou um resíduo que nomeia a próxima:

| escopo excluído | sobra no "miolo" |
|---|---|
| a franja da UNIÃO | −3/255 em 8 px |
| \+ discos nas PONTAS de perna | −3/255 em 4 px |
| **a menos de 1,5 px da silhueta de QUALQUER perna** | **0** |

O ofensor final — (20, 18) — está a **7,07 px** do eixo da perna 4→0 (ou seja, EM CIMA da silhueta
dela) e a 14 px do canto mais próximo: um flanco enterrado dentro da perna vizinha. A lei continua
exata no miolo; a borda é convenção de composição, e tem oráculo próprio.

### 11.4 Custo e mutações

O AA acrescenta uma distância por segmento e **poupa a integral inteira** onde `sd ≥ 0,5` (o pixel
não é tocado). Líquido: frame de 200 gestos **1594 → 1811 ms (+14%)**, pixel denso 4147 → 4344
(+5%), canto vazio inalterado. **2 mutações, 2 sangram** (tirar o `edge` · tirar o empurrão).

---

## 12. ⭐ O CONTROLE DO §8 E A AUDITORIA DA SUPERFÍCIE

### 12.1 `hardness = 1.0` — byte-idêntico no corpo, e MELHOR no cruzamento

O §8 do handoff exige que o traço duro fique byte-idêntico **ou venha a medição que justifique a
diferença**. Medido contra o motor que SHIPA, sem excluir a franja (o §11 tornou isso possível):

| figura | corpo | cap |
|---|---|---|
| reto | **+0 (0 px)** | −53 (40 px) |
| estrela, 5 cruzamentos | +127 (178 px) | −71 (14 px) |

⚠️ **O corpo do traço reto é BYTE-IDÊNTICO** — é ali que *"não mexer no acervo"* se verifica. Numa
figura que CRUZA eles divergem, e a diferença foi levada ao **ÁRBITRO** (a área que a união dura de
fato cobre, super-amostrada):

> **NOVO mais perto em 164 px · SHIPA em 14 · erro médio 8,1/255 contra 30,8.**

O motor novo não reproduz o que shipa: ele é **~4× mais fiel** ao que a tinta cobre. O gate afirma
essa RELAÇÃO (novo ganha ≥ 8:1 e com metade do erro), não uma barra — uma barra admitiria a
diferença ser regressão vestida de melhoria. **O veredito visual continua sendo do Enio.**

⚠️ **O CAP é o que sobra, e agora tem número no regime do controle:** −53/255 em 40 px num traço
reto. Ele é o passo 5.

### 12.2 ⚠️ E o controle achou um BUG do motor novo — a quadratura pula a lasca

`(11, 57)` da estrela: `sd = −0,132` (**dentro** da silhueta), a área diz **169/255** e o motor
devolvia **0**. O binning estava inocente (a lista do ladrilho tinha o segmento certo, verificado).
A causa é a quadratura: junto da silhueta o arco que o disco cobre fica **mais curto que meio passo
de amostragem**, a única amostra cai fora do disco e `τ = 0` num pixel genuinamente coberto — o
mesmo modo de falha da corda quase tangente do §11.2, do outro lado do zero.

A cura é a MESMA regra com o domínio certo (`sd > −½` em vez de `sd > 0`), e depois dela o motor
novo dá **161** onde a área diz 169 e o que shipa dá 152.

### 12.3 ⚠️ E aí a profundidade do empurrão teve de ser DERIVADA, não varrida

Empurrar meio pixel inteiro **super-estima um perfil suave**: o gate do passo 3 acusou
**24,19/255 em `dn = 0,98`** com dureza 0,8, e o desvio crescendo com a dureza — a assinatura de um
perfil íngreme amostrado fundo demais. A conta que resolve:

```text
  C = ∫_{−½}^{½} P(sd + v) dv ;  a parte COBERTA é v ∈ [sd − ½, 0]
  comprimento = ½ − sd = edge   ·   ponto médio u* = (sd − ½)/2
  ⇒ C ≈ edge · P(u*)   ⇒   empurrão = sd − u* = (sd + ½)/2      ← METADE do que eu tinha
```

Acerta os dois regimes **por construção**: perfil chapado ⇒ `P(u*) = 1` e a máscara vira o `edge`;
perfil suave ⇒ vira a média certa. 24,19 → **8,62**, e o resíduo que sobra é estrutural (dentro da
faixa de AA a nossa fórmula é a média de caixa e a do shader é `P(centro)·edge` — **elas discordam
de propósito**, e a média é a certa).

⚠️ **Isso mudou o escopo de um gate, e a razão estava escrita nele desde o passo 3:** o
`the_new_engine_reproduces_the_shipping_profile_on_a_straight_stroke` amostrava até `dn = 0,98` com
o comentário *"a borda exata é o AA, outra pergunta"* — a intenção certa, e um número escolhido
quando o motor **não tinha AA nenhum**. Agora a fronteira é **derivada do raio** (`dn ≤ 1 − ½/r`).

### 12.4 A superfície do §8, item a item

| item | veredito | evidência |
|---|---|---|
| posição · largura · opacidade · cor | ✅ | a integral carrega os quatro; largura em MUNDO |
| `closed` | ✅ | `a_closed_stroke_gets_its_seam_binned` |
| `hardness` | ✅ | §9.2 · §12.1 · o gate de perfil do passo 3 |
| `self_overlap` **OFF** (o default) | ✅ | `opacity_scales_the_ink_and_never_darkens_the_crossing` — a opacidade entra na COR, nunca no `f`, e é isso que preserva a regra do GP |
| `self_overlap` **ON** | 🔧 uma linha | é a opacidade entrar no `f`; a lei já é uma soma, então acumular é o caso natural |
| `airbrush` | 🔧 mecânico | outro perfil de dab dentro do MESMO `f_of`; a lei (soma) não muda |
| `tip` Dots/Squares + `dot_spacing` | 🔧 mecânico | dabs discretos são `τ = Σ f(d_k)` **sem** o peso de arco — já é a forma da lei |
| `cap` Flat/Round | ⛔ **passo 5** | medido: −53/255 em 40 px (§12.1). ⚠️ com a restrição do §9.5: **invariante à partição** |
| `material` · `fill`+`holes` · `hide_stroke` · `selected` | ✅ fora do rasterizador | `fill.rs`/`composite.rs` não são tocados |
| multiplano · ghost tint · overlay do colorize · dobra do preview | ✅ fora do rasterizador | vivem no `flip_pass.rs`/pipeline |
| fade sub-pixel | 🔧 mecânico | multiplica a cobertura, como no shader |

**Sobra UM item de projeto** (o cap) e quatro mecânicos. Nada na superfície do §8 pede outra
arquitetura.

---

## 13. ⭐ O CAP — e ele é um TERMO DE FRONTEIRA, não uma geometria nova

### 13.1 A conta

O Painter **SOMA** dabs; nós **INTEGRAMOS**. Euler-Maclaurin diz exatamente em que elas diferem:

```text
  Σ_{k=0}^{N} g(k) = ∫_0^N g(u) du + [g(0) + g(N)]/2 + …
```

⚠️ **No MEIO do caminho os termos de fronteira estão no infinito, onde `g = 0`** — e é por isso que
o corpo já concordava (+1/255 contra o depósito, **+0** contra o motor que shipa em dureza 1).
**Na PONTA o termo sobrevive, e ele é meio dab.** Não houve forma nova a desenhar: a silhueta
redonda já vinha do `t` clampado do `closest_on_seg`.

### 13.2 ⚠️ Só no COMEÇO — e a assimetria é da REFERÊNCIA

O depósito do Painter carimba um dab **exatamente** no primeiro ponto e depois anda por `pitch`,
então o percurso dele **acaba ANTES do último**, num lugar que depende do comprimento total: a
fronteira do começo é exata (o termo sobrevive), a do fim é fracionária (o termo médio é **zero**).
Medido nas duas pontas de um traço reto, erro médio contra o depósito:

| dureza | INÍCIO | FIM com meio dab | FIM sem |
|---|---|---|---|
| 0,4 | 2,3 | 13,6 | **1,2** |
| 0,7 | 3,3 | 14,9 | **1,8** |
| 0,2 | 1,9 | 12,7 | **1,3** |

⚠️ **A FORMA do cap continua simétrica** (a silhueta redonda está nas duas pontas); o que é
assimétrico é a **correção de quadratura**, e ela é invisível na geometria. `FLAG_CLOSED` não tem
ponta; `FLAG_START_FLAT` corta o termo — um cap Flat é exatamente a ausência do arredondamento.

**Resultado na região da ponta** (erro médio contra o árbitro): dureza 1 **1,7 → 0,9** (contra a
ÁREA) · 0,4 **16,7 → 2,3** · 0,7 **16,8 → 3,3** · 0,2 **15,7 → 1,9**.

### 13.3 ⛔ A CORREÇÃO: a §9.5 concluiu errado, e a referência a refutou

A §9.5 dizia *"qualquer primitivo de cap tem de ser invariante à partição do caminho"*. Fui medir a
referência antes de contorcer o desenho para satisfazê-la — **o depósito do Painter não é
invariante**: um caminho contra cinco pernas compostas por `over` difere em **−59/255 em 178 px**
(dureza 0,4), **−102 em 123** (0,7) e **−255 em 17** (1,0), **sempre nos CANTOS** (cada perna abre
com um dab em `pts[0]`).

A identidade que a §9.1 mediu em ZERO era artefato de o motor **não ter cap**. Ela vale onde o
caminho é o MESMO — os CRUZAMENTOS, que é o que o oráculo do Enio pergunta — e é ali que o gate
agora a afirma (as pontas de perna saem, como saem no depósito de referência).

### 13.4 A tripwire disparou e o gate se INVERTEU

O `..._deficit_is_the_endpoint_and_the_corner_...` afirmava que a PONTA dominava (−36 total contra
−27 cego a ela) com uma terceira asserção escrita para ficar vermelha quando o cap fechasse. Ela
ficou. O gate virou `..._only_deficit_is_the_convex_corner_...`, e agora exige o OPOSTO:

> esconder as pontas **não pode mudar nada** — `pior_tudo == pior_cego` e `n_tudo == n_cego`.

O que sobra é só a **quina convexa** (−27, ≤3 px): a discreteza dos dabs do Painter, não a nossa
quadratura (SUB 4→16 move ≤1/255; **abaixo** de 4 piora — SUB=2 leva a −30 em 11 px).

**2 mutações, 2 sangram:** tirar o termo (o cap reabre, o gate invertido falha) · torná-lo simétrico
(a ponta transborda, o gate de overshoot falha).

⚠️ **Ponto para o SMOKE:** o oráculo não carimba dab de **cauda** no pen-up e o Painter do produto
carimba. Se o Enio vir a ponta final fina demais, o termo do fim volta — e o número dele já está
medido na tabela acima.

---

## 14. ⭐⭐ O PORT PARA COMPUTE — **2,16 ms** num frame de 1080p com 200 gestos

O §10 mediu o percurso serial de CPU e disse o que ele NÃO era: *"o número serial não é o número do
produto; o que ele responde é a FORMA"*. A forma estava certa (o custo é local). Este é o número.

### 14.1 O kernel

`src/shaders/walk.wgsl` + `src/walk_gpu.rs` — **o primeiro compute desta crate**. Um workgroup de
**16×16 É um ladrilho** (`DEFAULT_TILE`, o número que a §6.2 mediu): as 256 threads leem a MESMA
lista, que é a razão inteira de o binning existir.

⚠️ **O binning fica na CPU, e é MEDIDO, não conveniência:** ele é 1× por frame; o percurso é
por-PIXEL. Portar o binner é a wave seguinte, e a tabela abaixo já diz quanto ela vale.

⚠️ **A superfície é `prepare` + `record`, não um `run` monolítico** — é a forma que o produto quer
(gravar no encoder do frame, sem readback). O `run` existe para o gate, e **medir com ele mediria o
PCIe**: o readback é de **33 MB** a 1080p.

### 14.2 A paridade

| | |
|---|---|
| pior \|Δ\| contra o `walk_pixel` | **4,05e-6** |
| canais acima de 1/255 | **0** |
| erro médio no alfa com tinta | **6,8e-8** |

Cena com **cinco perguntas num desenho só**: a estrela que cruza a si mesma · duro · macio ·
opacidade < 1 (a regra do GP) · **afilado** (largura variando ⇒ o raio interpolado da quadratura).

⚠️ **A saída é `vec4<f32>`, não uma textura de 8 bits** — senão o gate mediria a quantização junto
com a divergência e não saberia dizer qual é qual. ⚠️ **A barra é `1e-4`, não `1/255`:** meio nível
de byte (3,9e-3) seria folga de 1000×, larga demais para pegar divergência real de kernel; 1e-4
deixa **25×** sobre o medido e ainda é 39× mais apertada que meio byte.

### 14.3 O CUSTO

| traços | segs | bin (CPU) | **walk (GPU)** | ns/px | vs CPU serial |
|---|---|---|---|---|---|
| 1 | 39 | 0,03 ms | **0,12 ms** | 0,1 | 159× |
| 10 | 390 | 0,09 | **0,16** | 0,1 | 568× |
| 50 | 1950 | 0,45 | **0,57** | 0,3 | 723× |
| 200 | 7800 | 1,76 | **2,16** | 1,0 | **739×** |

**200 gestos a 1080p custam 2,16 ms de device — 13% de um quadro de 60 fps.** O desenho fecha.

⚠️ **ESTA CONCLUSÃO FOI REFUTADA — ver o §21.** O `1,76` saiu de **uma amostra não-aquecida** de um
instrumento que, re-rodado três vezes no mesmo binário, media `1,33 / 2,30 / 4,00`; com mediana e o
1º descartado o binner mede **~1,0 ms**, e depois das cinco features do §19 o percurso subiu para
2,7-2,9. A fronteira é o PERCURSO, não o binner. O texto original segue abaixo por honestidade
histórica: a próxima alavanca não é mais o percurso — é portar o binner, ou incrementá-lo (a lista só muda onde o traço
em curso mexe).

---

## 15. ⭐ O QUADRO DO VEREDITO — render-and-look, os três motores lado a lado

`render_the_verdict_three_engines_side_by_side` escreve em `/home/enio/flip_veredito/`:

```
PAINTER (a referência) | FLIP que SHIPA | FLIP NOVO | diff (NOVO − PAINTER)
```

mais o recorte **`PONTA_ampliada_*`** (nearest ×3), em quatro durezas, na estrela de **um traço**
com **mão LENTA** (`0,106·r`, o lado da cerca onde o defeito vive).

**O PICO contra o Painter, fora da franja:**

| dureza | SHIPA | NOVO |
|---|---|---|
| 0,2 | **+129** | **−12** |
| 0,4 | **+131** | **−17** |
| 0,7 | **+175** | **−46** |

E a imagem diz o mesmo que o número: o Painter faz a ponta **rombuda**, o motor que shipa a faz
**mais alta e pontuda**, o motor novo **reproduz a forma do Painter**.

### ⚠️ Duas lições de FIXTURE, as duas nesta sonda, as duas minhas

1. **A PROPORÇÃO é parte da fixture.** A 1ª versão usou `r = 26` sobre raio 250 (razão **0,10**)
   quando o defeito foi medido em `r = 7` sobre 26 (razão **0,27**). As três colunas saíram
   indistinguíveis e o diff saiu preto: **a imagem dizia "está tudo bem" sobre um desenho que não
   continha o fenômeno.**
2. **A MÉDIA dilui um defeito LOCAL.** Com a média sobre 640² os números eram `0,42` contra `0,04`
   — verdadeiros e inúteis, porque a queixa do Enio é uma **cunha**. O pico conta a história:
   `+131` contra `−17`.

E a ampliação existe pela MESMA razão: na tela inteira as três estrelas parecem iguais.

---

## 16. O QUE FALTA PARA ISTO SER PRODUTO (não é só o passo 5)

1. ~~**ANTI-ALIASING**~~ — **FECHADO no §11.**
2. ~~**As features do §8**~~ — **AUDITADAS no §12.4**: um item de projeto (o cap) e quatro
   mecânicos; nada pede outra arquitetura.
3. ~~**O cap da ponta**~~ — **FECHADO no §13** (um termo de fronteira, não uma geometria).
4. ~~**O port para compute**~~ — **FECHADO no §14: 2,16 ms, paridade 4e-6.**

**Não sobra item de PROJETO.** O que resta é integração: os quatro mecânicos do §12.4 (self_overlap
ON · airbrush · tip Dots/Squares · fade sub-pixel), trocar a saída para textura, ligar o passe no
`flip_pass.rs` no lugar do `flip.wgsl`, e **o smoke do Enio** — que é quem decide.

---

## 18. ⭐⭐ A FIAÇÃO — o motor novo está NO APP (`PH2D_FLIP_NEW_ENGINE=1`)

O Enio aprovou a forma (*"painter e Novo iguais"*) e pediu o build de smoke. Não existia: o motor
vivia na crate, atrás de gates e sondas, e nada no shell o chamava.

### 18.1 O ponto de entrada é UM, e é o Pass A do `stage_layer`

`FlipCompose::stage_layer` é a única porta por onde uma camada de Flip vira textura:

```
Pass A  rasteriza a camada no `hdr` (premult 16F)   ← ISTO é o que troca
Pass B  resolve premult → straight (Rgba8Unorm)     ← intocado
        `inject_slice_from_texture` → compositor 22-modos → blit
```

Trocar só o Pass A é o que mantém a decisão honesta: **compositor, blend por-camada, multiplano,
tint de fantasma e `inject` não sabem que a tinta mudou de produtor.** A troca passa a ser uma
afirmação sobre *quais pixels o traço acende* — exatamente o escopo do §1 — e não um segundo
pipeline de Flip correndo em paralelo com o primeiro.

O interruptor é `PH2D_FLIP_NEW_ENGINE=1`, lido **no shell** (`flip_pass.rs`), UMA vez, num
`OnceLock`. A crate não lê o ambiente: a escolha em dois lugares é a falha de duas-portas que este
repo paga toda semana, e um `var()` por frame ainda faria o A/B depender de *quando* o artista olhou.

### 18.2 A saída do kernel virou UMA textura — e a paridade pagou por isso

O kernel escrevia um `array<vec4<f32>>` para o gate poder medir em `f32`. Agora escreve o `hdr` do
produto (`rgba16float`), e **não há segunda saída**: dois caminhos para o mesmo pixel significam que
a paridade mede um enquanto o outro shipa.

O preço é exato e a aritmética o nomeia:

| saída | pior \|Δ\| contra `walk_pixel` | o que o número É |
|---|---|---|
| buffer `f32` (antes) | **4,05e-6** | divergência do KERNEL |
| textura `rgba16float` (agora) | **4,883e-4** | **2⁻¹¹** — o arredondamento de meia precisão em magnitude 1 |

⚠️ **O kernel nunca foi o limite:** ele está 120× abaixo do quantum do alvo que o produto usa.
A barra do gate deixou de ser escolhida e passou a ser **derivada**: `1e-3` ≈ 2× o quantum do
formato, ainda **3,9× mais apertada** que meio nível de byte (a resolução em que alguém vê).

### 18.3 O FILL é o piso, não trabalho do kernel

`FlipGpuData` carrega os fills como **malha de triângulos** (`fills`), desenhada por um pipeline
próprio. O percurso os ignorava — e um smoke assim vinha com armadilha: Colorize e toda forma
fechada perderiam o preenchimento, e o sintoma leria como *"o motor novo comeu os fills"*.

A cura não é ensinar o kernel a preencher (seria uma segunda resposta a *onde está o interior desta
forma?*, e o fill de borda dura nunca foi o defeito): o fill sai do pipeline que sempre o desenhou,
para um alvo próprio, e o kernel **inicia o acumulador lendo esse piso** — fill abaixo, traço em
cima, a ordem do `draw`. A VRAM é paga só por quem arma o motor.

⚠️ Duas coisas que a medição corrigiu no caminho: o pipeline de fill **declara depth-stencil**, logo
a passagem tem de fornecê-lo — e isso é melhor que o *"sem depth"* que eu ia justificar, porque a
ordem entre fills volta a ser o MESMO teste GREATER por sid do `draw`; e o harness de paridade
recebe piso **VAZIO** de propósito, porque o `walk_pixel` da CPU (o oráculo) também não conhece
fill, e um piso pintado tornaria a comparação torta.

### 18.4 Os dois gates que provam a costura

Os dois vivem em `tests/composite_blend.rs`, o arquivo que já dirigia o seam REAL
(`stage_layer` → `inject` → compositor → blit) com wgpu de verdade.

- **`the_staged_slice_comes_from_the_new_engine_when_it_is_armed`** — a fatia que o compositor
  recebe casa com o `walk_pixel` no **ALFA**. O alfa é a escolha certa e não conveniência: o Pass B
  des-premultiplica o RGB (aritmética própria, outro assunto) e **deixa o alfa em paz**, então ele é
  a única grandeza que atravessa a costura inteira sem ser transformada — e é literalmente a
  pergunta do §1. Barra **1,5/255** (derivada: `f32` → meia precisão → 8 bits, e o último domina).
  ⚠️ O gate compara contra o irmão de **CPU**, nunca contra o rasterizador: os dois motores
  discordam por PROJETO, então exigir que casem seria um gate que só passa com o trabalho desfeito.
  Medido: **0,54/255**. Mutação (o `stage_layer` ignorar o motor armado) sangra **255/255 na ponta
  convexa** — exatamente onde o §11.3 documenta a divergência.
- **`the_new_engine_keeps_the_fill_under_the_stroke`** — oráculo no **INTERIOR**, longe de qualquer
  borda, onde nenhum dos dois motores tem opinião. Mutação (não desenhar o piso) sangra `alpha = 0`.

### 18.4b ⚠️ O SMOKE ACHOU O QUE 23 GATES VERDES NÃO PODIAM VER — o Y do `point_px`

1º smoke (Enio): *"o traço do novo parece bom, mas o canvas está todo bugado, invertido, o pincel não
pinta no lugar certo"*. **Uma causa, três sintomas.**

`ScreenSpace::point_px` mapeava clip → pixel com `y = (cy/2 + 0,5)·h`. Mas **clip `+1` é o TOPO** da
imagem e **a linha 0 de uma textura é o topo** — o correto é `y = (0,5 − cy/2)·h`. Com o sinal
errado o desenho inteiro sai **espelhado na horizontal-média**; e como o traço é simétrico, a forma
parecia certa e o LUGAR não (daí *"o pincel não pinta no lugar certo"*: a tinta aparece espelhada).

⚠️ **Nenhum gate podia pegar isso, e vale como lei:** o percurso da CPU (`walk_pixel`, o oráculo) e
o do device leem **a MESMA** `point_px`, então um erro de convenção ali **move os dois lados igual**
e a paridade segue verde. Os 23 gates do `painter_look` também passavam — todos comparam FORMA, ou
comparam o percurso contra um oráculo que atravessa a mesma porta. É a cegueira door-contra-door que
o fold da luz do Painter já tinha documentado (doc 28 §4.8.2), aqui num sinal que o olho vê na hora.

**O único oráculo possível é o RASTERIZADOR** — ele passa pelo pipeline gráfico, que é quem define o
que "linha 0" significa. Foi ele que nomeou o defeito em uma medição:

```
traço em MUNDO: x 8..28, y ~6 (perto do topo, à esquerda)
RASTER    linhas 3..8    colunas 5..30
PERCURSO  linhas 55..60  colunas 5..30      ← 55 = 64−1−8: espelho vertical exato
```

Gate novo **`both_engines_put_the_ink_in_the_same_place`** (caixa da tinta ±2 px + **centroide**
±1,5, que pega um espelho mesmo se a caixa der simétrica por acaso). Fixture **assimétrica de
propósito** — um traço centrado é invariante ao espelho e o gate passaria sobre o bug — e
`hardness = 1.0`, porque este gate fala de POSIÇÃO: na borda macia os dois motores divergem por
projeto, e isso é assunto dos gates de forma. Mutação (reinstalar o sinal) sangra imprimindo o
diagnóstico: *"espelho vertical? SIM — o Y do point_px"*.

⚠️ **E uma FIXTURE codificava o bug:** `the_bin_has_no_fixed_ceiling` cravava o ladrilho no pixel
`(34, 36)` porque *"linha == y de mundo"* era verdade — e era verdade só por causa do Y invertido.
Ela **sobreviveu ao bug e caiu junto com a correção**. Agora ela **PERGUNTA** ao `point_px` onde as
linhas caem: uma fixture que crava coordenada de tela codifica a convenção em vez de testá-la.

### 18.5 Rodar

```
cargo build -p ph2d-host-desktop --release
env PH2D_FLIP_DEMO=1 PH2D_FLIP_NEW_ENGINE=1 cargo run -p ph2d-host-desktop --release   # o motor NOVO
env PH2D_FLIP_DEMO=1                        cargo run -p ph2d-host-desktop --release   # o CONTROLE
```

⚠️ **O A/B é o ponto** — a mesma cena, a mesma mão, os dois builds. O que o §11.3 diz que muda:
o cruzamento com `hardness` (a queixa original), a ponta convexa, e nada mais.

## 19. ⭐ O PLANO ATÉ O PADRÃO-OURO — a auditoria por GREP, e o airbrush

Smoke do §18 aprovado. O §16 dizia *"não sobra item de projeto, resta integração"* — mas ele é de
**antes** da integração de 27/07, e a auditoria por grep (nunca por auto-relato) mostra o vão real:
**o rasterizador tem SETE leitores de `Stroke`/flags e o percurso tinha QUATRO.**

| item | raster | percurso | estado |
|---|---|---|---|
| `closed` · largura · cor · opacidade · `hardness` | ✅ | ✅ | — |
| `FLAG_AIRBRUSH` | ✅ | ✅ | **FECHADO no §19.1** |
| `tip` Dots/Squares + `dot_spacing` + `ref_width` | ✅ | ✅ | **FECHADO no §19.3** |
| `fade` sub-pixel | ✅ | ✅ | **FECHADO no §19.4** |
| `FLAG_END_FLAT` (a silhueta trunca) | ✅ | ✅ | **FECHADO no §19.5** |
| `FLAG_SELF_OVERLAP` **ON** | ✅ | ✅ | **FECHADO no §19.6** |

⚠️ **Armado, o motor novo apagava CINCO features em silêncio — três delas integradas dois dias
antes** (airbrush, tip pontilhado, pressão; a pressão é largura, que o percurso já lê). Enquanto
isso for verdade **não existe conversa sobre default**: ligá-lo seria regredir trabalho aprovado, e
nenhum gate de paridade pode ver a regressão (todos comparam o percurso contra um oráculo que
atravessa a mesma porta — a lição do §18.4b).

**Só depois desta lista a perf volta a ser a fronteira** (o binner na CPU é 1,76 ms = 45% do frame,
§14) — otimizar um motor que não cobre a superfície é otimizar a coisa errada.

### 19.1 O canal: `DabProfile`, e por que ele não tem `Default`

Cada flag que fica de fora é uma feature apagada em silêncio, com tudo verde. Um par
`(hardness, bool, bool, …)` solto convida a passar `false` onde o traço tinha a resposta ⇒ nasceu
**`DabProfile`** (`hardness` + as flags), **sem `Default`**, construído por `DabProfile::of(&stroke)`
— a lei do `ShapeFrame` do Painter: não há como esquecer, e o compilador acha os leitores quando a
próxima flag entra.

### 19.2 O AIRBRUSH — a fórmula bonita estava errada, e a medição a matou

A primeira tentativa foi elegante e falsa. O rasterizador escreve a transmitância de Beer-Lambert,
`w = 1 − exp(−k·√(1−dn²))`; na lei do percurso `f = −ln(1−w)`, então

```text
f = −ln(exp(−k·√(1−dn²))) = k·√(1−dn²)      ← o log e o exp CANCELAM
```

Medido contra o raster (reta, raio 10, `hardness` 0,5, alfa em `dn` 0/0,3/0,5/0,7/0,9):

```
RASTER    252 251 249 242 192
PERCURSO  255 255 255 255 247      ← super-entinta
```

⚠️ **O `√(1−dn²)` do rasterizador é a projeção de ABEL da esfera — a corda pelo TUBO varrido, já a
resposta do traço INTEIRO, não de um dab.** Integrá-la ao longo do caminho a multiplica pelo número
de dabs. A cura sai da **inversão de Abel**: o kernel aditivo cuja integral de caminho **é** a corda
é a **indicadora do disco** (conferido numericamente a 4 decimais, `∫[√(y²+u²)<1] du = 2√(1−y²)`), e
a normalização vem de `C·2r·√(1−y²)/pitch = k·√(1−y²)`:

```text
dτ = k · step / (2r)        dentro do disco — e o PITCH CANCELA
```

**Isso é a física**: um spray deposita densidade por unidade de **CAMINHO**, não por dab. Os dois
perfis integram contra **medidas diferentes**, e a porta única `d_tau_of` é onde essa escolha mora
(`step/pitch` para o padrão · `step/2r` para o airbrush). Corolário: **o airbrush não tem termo de
fronteira** — Euler-Maclaurin corrige uma soma discreta, e a integral sobre o caminho real já é
exata nas pontas.

Resultado: **252/251/249/241/189** contra 252/251/249/242/192 do raster — pior **3/255**, o AA da
borda macia mais a quadratura, onde os dois divergem por projeto.

⚠️ **E o percurso fica MAIS correto que o rasterizador, não igual:** a corda fechada só vale numa
reta infinita; o percurso integra a densidade ao longo do caminho de verdade, então **na curva e no
cruzamento ele responde o que a forma fechada não sabe responder**. Numa reta os dois coincidem — e
é exatamente por isso que a fixture do gate é reta: é o único lugar onde o raster é oráculo.

Gate **`the_airbrush_reaches_the_walk_and_matches_the_closed_form`**, em duas metades — o perfil
casa com a forma fechada (barra 5/255) **e** o airbrush não é o perfil padrão (borda 189 contra 12).
Sem a segunda, um `d_tau_of` que ignorasse a flag passaria na primeira. **2 mutações, 2 sangram**
(a corda por dab · ignorar a flag).

### 19.3 O TIP PONTILHADO — a outra lista de dabs, e o quadrado que a janela recortava

**Não há kernel novo, e isso é o desenho inteiro.** A lei já é uma SOMA de dabs; uma linha cheia é a
soma sobre dabs tão juntos que ela converge para a integral de arco (é a definição do `pitch`,
§5.1), e uma fileira de contas é a MESMA soma com os dabs longe um do outro. Só a **lista** muda:

- **contínuo:** `dτ = f(dn) · step/pitch` — a medida é o CAMINHO;
- **contas:** `τ = Σ_k f_bead(dn_k)` — **sem peso de arco**, porque uma conta é UM carimbo.

E `f_bead` reconcilia as duas medidas que o §19.2 separou: num carimbo o airbrush volta a ser a
**corda** `k·√(1−dn²)` (a projeção de Abel de um dab), que é exatamente `1 − exp(−…)` = o
`hardness_mask` do `flip.wgsl`. A fórmula que a wave anterior mediu e reprovou estava certa — só
estava na medida errada.

**O canal:** `TipShape` (`Continuous` | `Beads{pitch, square}`) ao lado do `DabProfile`, e os dois
viajam no **`StrokeStyle`**, com UMA `of(&stroke)` — dois tipos soltos deixam um chamador construir
um e esquecer o outro, que é o modo de falha que a auditoria por grep achou. `arc_len` (o buffer que
o rasterizador já lia) virou a **binding 7** do kernel: o arco é cumulativo, não se deriva de um
segmento solto.

**Três decisões que não são detalhe:**

1. **A silhueta passa a ser a das CONTAS.** Um disco é uma cápsula degenerada, então é a mesma
   fórmula com outra lista — sem isso o `edge` mediria a borda da FITA (1 em toda a extensão dela) e
   as contas sairiam sem anti-aliasing, com o `p_eval` empurrado para a linha-de-centro em vez de
   para dentro do carimbo.
2. **A posse de cada conta é meio-aberta** (`arc_a ≤ k·pitch < arc_b`): a conta de uma JUNÇÃO tem de
   ter UM dono, senão ela entra na soma duas vezes e a junção escurece. ⚠️ **Com uma exceção: a
   PONTA.** O último ponto de um traço aberto não tem segmento seguinte para adotá-la, e sem a
   exceção o carimbo da ponta **desaparece** sempre que o arco total é múltiplo da pitch — o caso de
   todo traço em números redondos, e o da própria fixture do gate.
3. **Conta mais junta que o dab do pincel É a linha cheia** (`dot_spacing ≤ PAINTER_SPACING`). O
   limiar sai da LEI, não de um palpite, e é ele que **limita o laço**: a janela de um pixel mede
   `2r` de arco ⇒ no máximo `1/PAINTER_SPACING = 10` contas nela. Sem cap escolhido a dedo, e sem
   contagem que dispara quando o slider vai a zero.

#### O defeito que a paridade expôs sem poder achar

A paridade CPU×device nasceu **VERMELHA nos quadrados** (`pior |Δ| 1,0`, 384 canais) e verde nos
discos. A causa não era o espelho: **a janela da quadratura é um DISCO de raio `rmax`, e a quina de
um carimbo QUADRADO fica a `r√2`** — então os dois motores perdiam a quina IGUAL, e a comparação
entre eles deveria ser verde. O que a denunciou foi o defeito cair **em cima** da fronteira
`disc <= 0`, onde a GPU contrai em FMA e o ulp discordava: um sintoma de *precisão* apontando para um
buraco de *geometria*. Medido, uma fileira de quadrados antes/depois: as linhas de fora saíam
**9 px** de largura contra 10 das de dentro; agora todas as linhas medem o mesmo.

Fix: **`tau::dab_reach`** — a porta única de *quão longe da linha-de-centro este pincel põe tinta* —
perguntada pelo **binner** E pela janela, porque um ladrilho que listasse o segmento por `r` deixaria
a quina sem o segmento que a pinta. `pior |Δ| 4,883e-4` (o quantum do formato), **0 canais acima de
1/255**.

⚠️ **E o lado da quina é parte da FIXTURE:** a janela é clampada ao SEGMENTO, e a conta pertence ao
segmento que COMEÇA nela — um pixel adiante do começo mantém a janela aberta mesmo com alcance
curto, enquanto o pixel ATRÁS colapsa `t1 ≤ t0` e o segmento é descartado inteiro. Medido: com a
quina da frente a mutação do `dab_reach` **passa**; com a de trás ela sangra.

#### Gates — e a lição de que a lei do tip tem UMA CÓPIA POR MOTOR

Produto: **`the_dotted_tip_reaches_the_walk_and_the_beads_land_where_the_raster_puts_them`** (na
fatia que o compositor recebe): as contas do percurso caem nas fronteiras do raster **exatamente**
(`(4,11) (20,27) (36,43) (52,59)`, incluindo a da ponta), e o MESMO traço em `Continuous` sai em UM
bloco — sem essa metade, um `TipShape::of` que devolvesse sempre `Continuous` passaria comparando
uma linha cheia com ela mesma.

Unidade (o que a paridade **não pode** ver, porque é erro que os dois motores cometem igual): a conta
da junção é carimbada uma vez · a conta da ponta existe · o vão entre contas é ZERO onde a linha
cheia tem tinta · **um carimbo quadrado é um quadrado** (a quina tem tinta; a mesma quina com conta
REDONDA é vazia).

**7 mutações, 7 sangram.** ⚠️ E uma delas ensinou a estrutura: o `TipShape::of` do **Rust** mutado
para `Continuous` sangra 3 gates de unidade e a paridade, e **NÃO** sangra o gate do produto — porque
a decisão que o produto executa mora no `bead_pitch_of` do **WGSL**. A lei do tip tem uma cópia por
motor (é o espelho declarado no topo do `walk.wgsl`), então **um gate por lado**, e a paridade é o
que os amarra.

### §19.4 — O FADE SUB-PIXEL: o par que faltava do piso de largura

**A linha fina saía GROSSA e OPACA.** Os dois motores clampam o raio em `MIN_WIDTH_PX/2 = 0,65 px`,
e o rasterizador paga o preço desse clamp de volta com uma multiplicação na cobertura
(`mask *= smoothstep(0, 1, thickness)`, o `gpencil_frag.glsl:534`); o percurso tinha **só a metade
do clamp**. Medido no produto, pico de alfa de um traço reto de dureza 1:

```text
  largura   raster   percurso ANTES   percurso DEPOIS
    0,15 px     10        166               10
    0,30 px     36        166               36
    0,50 px     83        166               83
    0,80 px    148        166              148
    1,00 px    166        166              166
    2,00 px    255        255              255
```

Um traço de 0,15 px — o que qualquer desenho vira depois de um zoom out — saía com **16× a tinta**
que ele pede. Hoje os dois motores dão o **MESMO byte** em toda largura.

**Não há kernel novo.** O clamp e o fade são **um par**, e cada metade sozinha erra para um lado:
sem o clamp a fita não cobre o centro de nenhum pixel e a linha **pisca** ao mover (o rasterizador
acerta ou erra); sem o fade ela fica com a tinta do piso. Junto, a **forma** fica no piso e a
**cobertura** desce — a energia é preservada.

**O fade multiplica a COBERTURA, nunca o `τ`,** e as duas rotas não são equivalentes:
`1 − exp(−fade·τ)` satura junto com o `τ`, então em dureza 1 (onde `f = F_MAX` e a exponencial já
está em 1) escalar o `τ` deixaria a linha fina **opaca** — exatamente o defeito que o fade existe
para remover. É por isso que a fixture do gate usa dureza 1: é o único regime onde a confusão é
indistinguível de não fazer nada.

**⚠️ O fade é do DAB, não do traço** — e essa é a única decisão de projeto da wave. Um pixel é
tocado por muitos dabs de larguras diferentes (um traço de pressão afina), então o fade viaja no
acumulador como **média ponderada por `dτ`**, o MESMO peso que a cor já usa. Um fade por-traço
desenharia a agulha de uma ponta com a tinta da barriga: medido, barriga α 1,0000 · agulha α 0,3205,
contra 1,0000 nas duas com o fade lido do traço.

**⚠️ O acumulador virou UM tipo (`Ink`), e não por estética:** `end_dab` recebia três out-params e
o quarto fez o `clippy` reclamar — mas o defeito real é que três out-params são três coisas que se
pode esquecer de acrescentar. `Ink { tau, rgba, fade }` é o que já viajava junto (somas ponderadas
durante a soma, médias ao devolver), e agora o compilador o carrega inteiro.

**O atalho do caso comum é EXATO:** onde as duas pontas do segmento medem ≥ 1 px, toda amostra entre
elas é uma combinação convexa — logo também ≥ 1 —, e ali `sub_pixel_fade` devolve `1.0` exato (o
`clamp` satura e `1·1·(3−2) = 1`). Um traço de espessura normal não paga um ciclo por esta wave, e
há gate varrendo 24×24 pares de larguras × 65 posições para pinar isso.

**Gates:** o do PRODUTO compara contra o **rasterizador**, que aqui é oráculo **exato** e não
aproximado (o fade é uma multiplicação na cobertura, sem quadratura envolvida) — e a segunda metade
dele exige que o alfa **cresça** com a largura, senão um fade constante passaria. Mais quatro de
unidade na CPU: a forma fechada `α(w) == sub_pixel_fade(w) · α(1,3)` (abaixo do piso o raio clampado
é o MESMO, então a geometria é idêntica e só o fade difere — oráculo sem folga), a agulha do afilado,
a exatidão do atalho, e o polinômio pinado contra valores computados **fora** do codebase.

⚠️ **E a cena de paridade estava CEGA ao fade** — todas as larguras dela eram ≥ 1 px, então o atalho
disparava em cada segmento e o fade era a identidade em toda a imagem. A cena ganhou uma **oitava
pergunta** (um traço que afina de 1,6 a 0,1 px, atravessando a fronteira do atalho no meio, para os
dois ramos correrem no mesmo traço); com ela a mutação *"o fade da CPU é sempre 1"* sangra em
**6,289e-1 e 452 canais**, e sem ela passava em silêncio. **8 mutações, 8 sangram.**

**⚠️ Uma divergência FICA ABERTA, com número:** a conta sub-pixel. O `flip.wgsl` usa a espessura
**crua** como raio de conta, então a 0,40 px ele **apaga a fileira inteira** (zero pixel aceso); o
percurso usa o raio clampado e desenha um pontilhado fraco (76 px, pico 57). Não foi alinhado ao
raster de propósito — desaparecer ao dar zoom out é o modo de falha que o par clamp+fade existe para
remover, e adotar a regra do raster só nas contas seria uma segunda regra dentro de um motor. Acima
de 1,3 px os dois convergem, que é onde todo pincel pontilhado do produto vive. **O smoke decide**;
a sonda que mede está em `measure_the_sub_pixel_bead_in_both_engines`.

### §19.5 — A TAMPA CHATA: a feature que o percurso tem de expressar por OUTRO mecanismo

Um extremo `Cap::Flat` **corta** a fita no ponto em vez de arredondá-la, e o percurso arredondava
sempre. As duas primeiras waves do §19 foram ports (a mesma fórmula, outra medida); esta **não é**:

**No rasterizador uma tampa chata não é um campo de distância — é a AUSÊNCIA de geometria.** O
vertex estende o quad por `r` ao longo da reta numa tampa Round (`ext_a = r_a`) e por **zero** numa
Flat, então a meia-lua simplesmente não é rasterizada; o `capsule_dn` do fragment é sempre o
redondo. **O percurso não tem quad** — todo pixel do ladrilho pergunta à silhueta —, então a
truncagem tem de morar no SDF: interseção com um semi-plano, ou seja um `max` sobre o `sd`.

Medido no produto, a faixa de tinta de um traço reto de raio 6 (`x = 16` a `48`):

```text
                 raster        percurso ANTES   percurso DEPOIS
  tampa Round    (10, 53)      (10, 53)         (10, 53)
  tampa Flat     (16, 47)      (10, 53)         (16, 47)
```

**Três decisões, e nenhuma é detalhe.**

**(1) A truncagem é por-SEGMENTO, nunca um semi-plano global** — e a diferença é arte que
desaparece. No raster só o quad do PRIMEIRO (ou último) segmento não estende; os outros cobrem o que
cobrem. Um traço que se enrola de volta e passa **por cima do próprio começo cortado** pinta ali
(medido: α 0,9595 atrás do plano, sobre a perna de volta). Um plano global apagaria a volta inteira.

**(2) A tampa é dos EXTREMOS, e de mais nada.** Uma mutação sobreviveu — `cap_head.is_some()` em vez
de `cap_head == Some(seg.a)` — e o que ela produz **não** é o plano global: é um **entalhe em toda
quina**, porque no lado de fora da curva os dois semi-planos vizinhos se somam e abrem uma fatia. Os
dois probes que eu tinha não passavam por junção nenhuma; o gate que faltava mede a quina externa de
um "L".

**(3) `cut` nasce em `NEG_INFINITY`, e o `max` com ele é a identidade EXATA** ⇒ todo traço de tampa
redonda é byte-intocado. A mutação que o troca por `0.0` sangra **dez** gates.

**⚠️ Uma segunda mutação sobreviveu, e a resolução dela é fixture, não código:** tirar o gate
`!closed` do `flat_caps` passou num anel CONTÍNUO — e não por buraco de gate, por **álgebra**: o
meio-disco que o plano tira do primeiro segmento está inteiro dentro do disco que a **tampa redonda
do segmento de FECHO** cobre no mesmo ponto, então o `min` sobre os segmentos devolve o mesmo número.
Nas **CONTAS** não: a conta do arco 0 pertence ao primeiro segmento e o de fecho não a possui (o
`bead_range` dele é meio-aberto e `tail` é `None` num anel), então metade dela desapareceria. O gate
do anel agora roda nas duas versões, contínua e pontilhada, e a mutação sangra.

**⚠️ E o percurso fica MELHOR que o raster num ponto — e o número só apareceu quando a fixture
parou de mentir.** Lá a borda reta é a fronteira do quad (dentro-ou-fora, sem meio-tom); aqui ela sai
do mesmo `edge = 0,5 − sd` de sempre, com anti-aliasing — o `max` de dois SDFs não sabe nem se
importa de onde cada um veio. **Eu escrevi isso por raciocínio e a primeira medição não confirmou
nada**, porque o traço acabava em `x = 48,0`, uma **fronteira exata de pixel** (os centros ficam em
47,5 e 48,5), onde a rampa de AA é degenerada e os dois motores dão `…255, 0…`. Movendo o fim para
`48,4`, meio pixel adentro:

```text
  tampa Flat, alfa em x = 47,5 … 50,5
    raster     255,   0, 0, 0     (passo duro: o fim em 48,0 e em 48,4 renderizam IGUAL)
    percurso   255, 102, 0, 0     (102/255 = 0,40 = a cobertura exata de um corte a 0,1 px do centro)
```

⚠️ **Isto é uma DIVERGÊNCIA nomeada, não só uma vitória:** numa tampa chata que não cai em fronteira
de pixel os dois motores diferem em até ~102/255 na coluna da borda. O gate do produto mede **onde a
tinta acaba** a meia-cobertura, então ele passa; a divergência fica aqui com o número, ao lado da
conta sub-pixel.

**Gates:** o do PRODUTO compara contra o raster nas DUAS tampas e exige que elas **difiram** entre si
por ~um raio de cada lado (sem essa terceira metade, um percurso que ignorasse a flag passaria se o
raster também a ignorasse). Mais quatro de unidade: o corte de um traço reto com as pontas
**independentes**, a perna de volta, a quina sem entalhe, e o anel inerte (contínuo + pontilhado).
**7 mutações, 7 sangram** — duas só depois dos gates que as sobreviventes nomearam.

⚠️ **A cena de paridade ganhou a NONA pergunta** (um "J" de tampa chata que volta sobre o próprio
começo — a única forma que prova o por-segmento no device também).

⚠️ **LOC:** o `tau_tests.rs` bateu 736 > 700 e o corte foi por responsabilidade, não por tamanho:
`cover = (1 − exp(−τ)) · edge · fade`, e os dois termos que **não** são o `τ` (o fade e a tampa)
saíram para o irmão **`cover_tests.rs`**. Contagem de gates antes e depois do split: **57 e 57**.

### §19.6 — O SELF OVERLAP: a lista fecha, e a partição é uma SUB-LISTA

A última flag, e a única cujo mecanismo no rasterizador é o **DEPTH**: o bit troca a profundidade
por-traço por uma por-SEGMENTO, então faces sobrepostas de partes diferentes do mesmo traço passam o
GREATER estrito e blendam `over` em vez de serem descartadas. O percurso não tem faces nem depth — ele
resolve UM depósito por traço, e a cobertura **satura** no cruzamento (a regra `OFF` do GP,
*"the stroke cannot overlap itself"*).

**Medido, um "X" de um traço só a opacidade 0,5** (sem opacidade < 1 a flag é invisível: tinta opaca
já satura):

```text
                    braço   cruzamento   junção
  OFF  raster        127        127        127
  OFF  percurso      127        127        127     (já concordavam — o controle)
  ON   raster        127        191        127
  ON   percurso      127        127        127     ANTES
  ON   percurso      127        191        127     DEPOIS
```

`191/255 = 0,749 = 1 − (1−0,5)²` — duas passagens compostas. ⚠️ **E a junção é 127 no raster
também**: o truque de depth dele **não** duplica tinta nas quinas (era a minha suspeita, agora
medida).

**A partição é inevitável, e é ÁLGEBRA:** em opacidade 1 os dois casos coincidem
(`1 − Π exp(−τ_p) = 1 − exp(−Σ τ_p)`, que é o que o percurso já calcula), então a diferença é inteira
sobre **como o `opacity` entra** — e isso exige o `τ` de cada passagem separado. ⛔ **Deixar o
`opacity` entrar no `f` do dab NÃO serve, e a razão é a doença que esta linha curou quatro vezes:** uma
passagem são muitos dabs, então o braço passaria a depender da densidade de amostragem.

**E então a implementação some:** *uma passagem é uma SUB-LISTA, e uma sub-lista já é o que o
`stroke_deposit` consome.* Com o bit, cada passagem é tratada como se fosse um traço, pela MESMA
composição `over` que o `walk_list` já faz entre traços — zero lei nova, zero kernel novo, e a
silhueta passa a ser por-passagem, que é o certo (o AA da borda de cada uma).

**⚠️ O PARTIDOR: duas versões construídas, e a primeira foi MEDIDA e reprovada.** O
`neighbors.rs` já proíbe por escrito a partição por **ARCO** (*"a v1 desta wave cortava por ARCO e
estava ERRADA — não re-derive"*), então eu fui para a por **ALCANCE**, que é a lei que ele usa. Ela
erra por um motivo que só a medição mostra: `stroke_deposit` amostra em **`p_eval`**, empurrado até
meio pixel para dentro, então um segmento que o alcance chama de "buraco" ainda **deposita** — no X
saíram **passagens fantasma de 1 segmento a 23-25% de cobertura** em cada lado das reais, com
cruzamento **205** onde o raster põe 191 e junção **143** onde ele põe 127.

**A versão que ficou não tem predicado nem épsilon: uma passagem é uma cadeia CONTÍGUA da polilinha
presente nesta lista.** A licença é do binner — ele lista **todo** segmento a `r` do LADRILHO, e o
pixel está no ladrilho, então *estar na lista* é implicado por *poder alcançar o pixel*; logo um buraco
na cadeia significa que os segmentos do meio nem alcançam o ladrilho, ou seja o traço foi embora e
voltou. Medido no X: quebras de cadeia em `[5, 18]` no cruzamento (⇒ 2 blocos com tinta, 191), `[]` no
meio da perna (⇒ 1 bloco, 127). **Exatos.**

**⚠️ A LIMITAÇÃO é nomeada e gateada:** um cruzamento que **nunca sai do ladrilho** fica contíguo e lê
como UMA passagem — a flag não compõe ali. A degradação é a conservadora (volta ao `OFF`, o
*first-wins* histórico do GP), a mesma postura dos tetos do `neighbors.rs`, e há gate pinando o limite
para ninguém o descobrir por acidente.

**⚠️ E um gate MEU nasceu vermelho sobre código certo.** Eu afirmei *"em opacidade 1 a flag não muda
nada"* pela álgebra acima; ele mediu `|Δ| = 1,21e-1`. O que a álgebra ignora é o **`edge`**, que passa a
ser por-passagem — dois ombros parciais compostos dão mais que a união deles. E a medição do PRODUTO
decidiu contra o gate, não contra o código: o **rasterizador muda MAIS** ali — pior Δalfa **+63 em
16 px** contra **+31 em 12 px** do percurso —, então o efeito é da semântica `over`, não desta
implementação. A afirmação certa, que o gate faz agora: **onde a flag mexe, o pixel é de BORDA**; o
miolo é intocado.

**Gates:** o do PRODUTO compara os dois motores com a flag OFF e ON e exige que ela mude **o
cruzamento e só ele** (as duas últimas asserções são as que matam os no-ops: uma flag inerte e um
partidor que corta onde não há cruzamento). Mais três de unidade na CPU: a composição só no
cruzamento, o ombro em opacidade 1, e a limitação do lacinho. **6 mutações, 6 sangram.**

⚠️ **A cena de paridade ganhou a DÉCIMA pergunta** (um X de um traço com a flag e opacidade 0,5 — sem
os dois a partição não roda e a cena fica cega, como já ficou no fade e na tampa).

⚠️ **LOC:** `tau.rs` bateu 741 ⇒ split pela distinção que o próprio arquivo já documenta (o
`DabProfile` é *a FORMA da queda*, a `TipShape` é *ONDE os dabs estão*): a **geometria da lista de
dabs** — alcance, contas, tampas, passagens, a janela da quadratura — saiu para **`dabs.rs`** (544 +
209). Nada em `dabs.rs` sabe quanto vale um dab; tudo nele sabe onde ele pode estar.

## 20. ⭐ A LISTA FECHOU — e o que sobra

Com o §19.6 o percurso lê **todas as sete** entradas de `Stroke`/flags que o rasterizador lê. O §19
começou porque *"armado, o motor novo apagava CINCO features em silêncio"*; nenhuma continua apagada.

**O que a comparação com o raster deixou NOMEADO em vez de alinhado** (as três divergências, cada uma
com número e sonda):

| divergência | raster | percurso | onde |
|---|---|---|---|
| conta sub-pixel (0,40 px) | apaga a fileira | pontilhado fraco (76 px, pico 57) | §19.3 |
| borda de tampa chata fora de fronteira de pixel | passo duro (perde a posição sub-pixel) | AA exato (102/255) | §19.5 |
| ombro de cruzamento em opacidade 1 com a flag | +63 em 16 px | +31 em 12 px | §19.6 |

Nas três o percurso é o mais correto pelas próprias regras que o raster afirma nos comentários dele, e
nas três **o smoke decide** se isso é o produto.

**A fronteira volta a ser a PERF — e o §21 mediu que ela NÃO era onde este doc dizia.**

## 21. ⭐⭐ A PERF, MEDIDA CONTRA O MOTOR QUE SHIPA — o número que decide o default

O §14 fechou o desenho do percurso com *"2,16 ms, 13% de um quadro"* e o §20 apontou a próxima
alavanca para o binner. **As duas afirmações estavam erradas, e por motivos diferentes.**

### 21.1 O instrumento media UMA amostra não-aquecida

O `1,76 ms` do §14 é `bin_ms` de **uma** chamada de `bin_segments`, sem aquecimento e incluindo as
alocações dela. Re-rodando a MESMA sonda três vezes no mesmo binário: **1,33 · 2,30 · 4,00 ms.**
*Um número que não reproduz não é achado, é ruído com casas decimais* — e foi sobre uma dessas
amostras que este doc concluiu *"o binner é 45% do frame"* e mandou a próxima wave para lá.

Corrigido (12 chamadas, a 1ª descartada, **mínimo** — aqui toda amostra faz trabalho idêntico, então
o mínimo é o que a máquina consegue e o resto é carga alheia):

| traços | binner (CPU) | percurso (GPU) | total |
|---|---|---|---|
| 1 | 0,01 | 0,08 | 0,09 |
| 10 | 0,04 | 0,21 | 0,25 |
| 50 | 0,21 | 0,75 | 0,96 |
| **200** | **0,97** | **2,73** | **3,70** |

O binner é **26%** do total, não 45%. A fronteira é o percurso.

### 21.2 ⭐ O número que NUNCA foi medido: o motor que SHIPA

*Rápido* e *lento* só existem contra alguma coisa, e a coisa é o rasterizador que está no produto.
`measure_what_each_engine_charges_for_the_same_scene` mede o **seam real** (`stage_layer`: Pass A
produtor + Pass B resolve, idêntico nos dois ⇒ a diferença é o produtor), 1080p:

| traços | raster | percurso | razão |
|---|---|---|---|
| 1 | 0,072 ms | 0,166 | **2,3×** |
| 10 | 0,082 | 0,365 | **4,5×** |
| 50 | 0,125 | 1,286 | **10,3×** |
| 200 | **0,293** | **4,332** | **14,8×** |

**O percurso custa 14,8× o motor que shipa, e a razão CRESCE com a contagem de traços.** Em termos de
quadro: 1,8% contra **26%** de um quadro de 60 fps a 200 traços.

⚠️ **E o CRESCIMENTO é o diagnóstico, não o valor absoluto:** o raster cresce 4× para 200× os traços
(0,072 → 0,293) e o percurso cresce **26×** (0,166 → 4,33). O raster paga **área de tinta**; o percurso
paga **pixels × segmentos perto deles** — ele multiplica onde o raster soma.

⚠️ **A 1ª versão desta sonda mediu no canvas 64×64 do harness** e deu 19,4×: ali o percurso é dominado
pelo binning e pelo dispatch, não por pixels. *A TELA é parte da fixture.*

### 21.3 Duas alavancas MEDIDAS — uma inexistente, uma reprovada

**O ladrilho JÁ ESTÁ no ótimo** (varredura 8/16/32/64 × 1/10/50/200 traços, total bin+walk):

| ladrilho | 10 traços | 50 | 200 |
|---|---|---|---|
| 8 | 0,35 | 1,43 | 5,75 |
| **16** (o que shipa) | **0,25** | **0,96** | 3,70 |
| 32 | 0,27 | 0,97 | **3,55** |
| 64 | 0,28 | 1,09 | 4,15 |

32 ganha **4%** a 200 traços e perde a 10; 8 é muito pior. **O `DEFAULT_TILE = 16` é medição, não
palpite** — e o achado que importa é outro: **o percurso é quase INSENSÍVEL ao ladrilho** (2,73 / 2,79 /
3,05 / 3,85 num fator 8 de área), o que **refuta** a hipótese natural de que ele é limitado pelo
comprimento da lista por ladrilho. Ladrilho menor encurta a lista e não compra nada.

**O piso é desprezível:** um traço só custa 0,08 ms dos 2,73 (2,07 M pixels de load+store = 0,04 ns/px)
⇒ os 2,7 ms são **trabalho de quadratura de verdade**, não largura de banda.

⛔ **`SUB = 2` — CONSTRUÍDO, MEDIDO e REPROVADO. Não refaça.** Ele compra **−30% do device**
(2,73 → 1,90 ms) e custa:

- **o DOBRO do erro na TAMPA** de um traço reto contra o depósito do Painter: **−53 → −134**;
- o árbitro do cruzamento caindo de **11,7× para 7,1×**, abaixo da barra de 8× do gate de controle
  (`the_new_engine_leaves_the_hard_default_where_the_shipping_engine_put_it`).

⚠️ **E isso corrige a §5.4:** a tabela dela mediu numa **QUINA** (`h = 0,4`) e concluiu *"4 satura"*
(`1→−73 · 2→−67 · 4→−65`). A saturação é real e a conclusão era **limitada pela fixture** — a
quadratura não dói na quina, dói na **TAMPA**, onde vive o termo de fronteira do §13. `SUB = 4` é o
**piso**, não o conforto.

⚠️ **Registro de leitura errada, minha:** eu li `estrela corpo +127 (178 px)` na saída do gate como
regressão do `SUB=2`. Ele é **pré-existente em `SUB=4`** — é a divergência de PROJETO no cruzamento
(o árbitro do próprio gate diz que ali o motor novo é que está mais perto da área verdadeira). *Uma
linha de saída não é um delta; delta pede as duas colunas.*

### 21.4 ⭐ A ABLAÇÃO: onde os 2,7 ms de fato moram

Antes de projetar wave, **ablação por dentro do kernel** — as três âncoras do `walk.wgsl`, uma por
corrida (baseline 3,06 ms, 200 traços, ladrilho 16, 1080p):

| ablação | percurso | Δ |
|---|---|---|
| baseline | 3,06 ms | — |
| `f_of` devolve `w·F_MAX` (mata o **`log`**) | 2,97 | **−3%** |
| `d_tau_of` devolve constante (mata `dab_weight` + `log`) | 2,20 | −28% |
| `n = 1` (mata o **LAÇO de quadratura**) | **0,72** | **−76%** |

⛔ **O `log` é 3% — tabelá-lo é INÚTIL, e isso mata a transferência de um precedente forte.** O doc 24
do Painter tabelou a transferência sRGB e ganhou 20-34×, mas lá o `pow` era **libm na CPU a ~24 ns**;
aqui `log` é **instrução de SFU no device**. *Um precedente é sobre um mecanismo, não sobre um nome
de função.*

**A conta é o NÚMERO DE AMOSTRAS**, e a aritmética o confirma: `pitch = 0,2r` e `ds = pitch/SUB =
0,05r`, com janela de ~`2r` ⇒ **~40 amostras por (pixel, segmento)**. `SUB` é piso (§21.3), então a
única alavanca é **não amostrar**.

### 21.5 A wave que sobra: a ANTIDERIVADA (não a LUT do `log`)

Para `r` constante, a contribuição de um trecho de segmento é

```text
  ∫ f(dn) ds / pitch,   dn = √(y² + u²)   (y = distância perpendicular ÷ r, u = arco ÷ r)
```

e a substituição `s = r·u` **tira o `r` de dentro**: o integrando passa a depender só de `(hardness,
y, u)`. Logo existe uma **antiderivada universal** `H(hardness, y, u)` e o laço inteiro colapsa em
**duas leituras e uma subtração** — `(r/pitch)·[H(h, y, u₁) − H(h, y, u₀)]`. Isso é exato, não uma
aproximação do perfil: o que a tabela discretiza é o *acumulado*, e o `SUB` sai de cena.

⚠️ **Não confundir com a "reta fictícia" que o §1 acusa no motor que shipa.** Ele integra sobre uma
reta **INFINITA** (por isso erra +140 na ponta convexa e não vê cruzamento); aqui os limites `u₀, u₁`
são os do **segmento que existe**, e a soma sobre os segmentos do caminho continua sendo a soma da §5.

**Dois riscos, os dois a MEDIR antes de construir:**

1. **`r` varia ao longo de um segmento** (traço de pressão), e a substituição assumiu `r` constante.
   O `resample_smooth` densifica a `0,4 × largura` ⇒ segmentos de ~`0,8r`, então a variação dentro de
   um segmento é pequena e o erro é de 2ª ordem — **"é pequeno" não é medição.**
2. **O airbrush tem OUTRA medida** (`dτ = k·step/2r`, densidade uniforme no disco — §19.1), então ele
   quer a própria antiderivada, que é trivial (`k·(u₁−u₀)/2`) mas é uma 2ª tabela ou um 2º ramo.

Tamanho: `[0,1]³` a 64³ = 262 k entradas × 4 B = **1 MB**, uma leitura trilinear. A precisão pede o
protocolo do doc 24 (medir a deriva, desconfiar de mal-condicionamento), e aqui `H` é monótona e lisa
em `u` — bem-condicionada por construção.

### 21.6 O que NÃO é alavanca (medido, para ninguém re-derivar)

- **o ladrilho** (§21.3: o percurso é insensível num fator 8 de área);
- **o piso de dispatch** (1 traço = 0,08 ms de 2,73 ⇒ pular ladrilho vazio compra ~nada);
- **o `log`** (3%);
- **`SUB = 2`** (−30% e reprova o gate de controle);
- **portar o binner** (ele é 26% do total, não 45% — o §14 estava sobre uma amostra ruidosa).

**A decisão de DEFAULT é do Enio, e agora ela tem os dois números:** 14,8× de custo de device a 200
traços, contra as três divergências do §20 em que o percurso é o mais correto e a família de defeitos
do cruzamento/ponta que ele existe para curar. **O smoke aprovou a imagem; o preço está medido.**

## 17. Fontes

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
