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

⚠️ **E a mutação D achou uma RESTRIÇÃO DE PROJETO para o passo 5, não só um verde:** um cap ingênuo
nas pontas **quebra a identidade da §9.1** (cinco traços têm dez pontas, um traço tem duas) — o gate
`the_new_engine_makes_a_self_crossing_stroke_equal_separate_strokes` fica vermelho. Qualquer
primitivo de cap tem de ser **invariante à partição do caminho**, ou paga a joia que esta wave
acabou de comprar.

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

## 12. O QUE FALTA PARA ISTO SER PRODUTO (não é só o passo 5)

1. ~~**ANTI-ALIASING**~~ — **FECHADO no §11.**
2. **As features do §8 do handoff**, cada uma com a pergunta *"isto sobrevive à troca de lei?"* —
   airbrush · self-overlap (que provavelmente **desaparece**: ele existe para forçar acúmulo que
   agora é automático) · tip pontilhado · pressão · multiplano.
3. **O cap da ponta** (§9.3), com a restrição que a mutação D descobriu: tem de ser **invariante à
   partição do caminho**.
4. **O port para compute**, que é o único lugar onde o custo do §10 vira um número de produto.

---

## 13. Fontes

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
