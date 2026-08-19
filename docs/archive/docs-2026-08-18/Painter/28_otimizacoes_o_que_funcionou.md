# ARQUIVO — 28_otimizacoes_o_que_funcionou.md (história, 6670 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`28_otimizacoes_o_que_funcionou.md`](../../../Painter/28_otimizacoes_o_que_funcionou.md) em 2026-08-18, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-54,152-706,1548-1627,2735-2907` do original.
> ⚠️ **A única alteração ao corpo:** 4 alvo(s) de link relativo foram **reancorados**
> para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e todo `../x`
> passaria a resolver noutro sítio. Texto, números e estrutura são byte-idênticos; a
> partição foi provada por sha256 **antes** desta reancoragem.

---

## 2. ✅ A frente que shipou — a LUT pré-convoluída (frente A)

### 2.1 O achado que a destravou

A decomposição por-modo mediu que **o AA do filme é 54% de um traço de impasto** (68,7 de 127,1 ms a
raio 100). Rodando o `film_at` com a closure REAL e com uma closure que devolve constante — as nove
chamadas de `film_of` e o laço intactos nas duas — a razão foi **10,3× a 12,3×**: ⚠️ **~91% do custo do
AA é a CADEIA de silhueta, não a curva.** E lida a cadeia, o que ela tem por amostra é `sqrt`: um na
norma do ponto deformado e, em `Sphere`/`Root`, outro no peso ⇒ **até 18 `sqrt` DEPENDENTES por texel de
banda**.

Não é a curva que é cara. **É a raiz.**

### 2.2 A derivação, e por que a cápsula era o caso mais FÁCIL

Os dois consumidores computam `t = |A·(r/radius)|`, com `A` o afim do footprint (rotação + flatten) e
`r` o resíduo. Escrevendo `w = A·(r/radius)` temos `t = |w|` e **no espaço deformado tudo é euclidiano**:

```
t(o) ≈ t + (ŵ·e) + [|e|² − (ŵ·e)²] / (2t),      e = B·o
```

`B` é **linear em cada região**, e é o chamador que sabe qual:

* **pigmento (disco)** e **CALOTAS da cápsula**: `B = A/radius` — uma calota *é* um disco no extremo;
* **BANDA da cápsula**: `B = A·P/radius`, com `P = I − uuᵀ`.

⚠️ **Na banda o termo de 2ª ordem é EXATAMENTE ZERO, não aproximadamente.** `P` projeta no complemento
de `u`, que em 2D é 1-D, então `P·o` e `P·d` são múltiplos do mesmo vetor; `A` preserva paralelismo ⇒
`e ∥ w` ⇒ `|e|² = (ŵ·e)²`. É *"distância a uma reta é afim"* saindo da álgebra. **A cápsula, que era a
incógnita declarada da wave, é o caso mais fácil dela.**

⚠️ **E a derivação consertou um bug SILENCIOSO:** a versão anterior assumia `t = |d|/r` euclidiano e
**errava sob Flatten & Rotate**, sem nenhum gate acender.

### 2.3 A admissibilidade, e a coincidência que não é coincidência

`FilmLut::admissible` é a porta única, com três cláusulas — e a terceira é do CHAMADOR porque é
por-texel:

1. **família de falloff SUAVE.** `Constant` sai por DOIS motivos ao mesmo tempo: é errático (um degrau
   interage com a grade de texels) **e é mais LENTO** (0,46× — a curva dele é a constante 1, não há raiz
   a economizar). `Custom` sai porque a tabela seria indexada por uma curva do documento;
2. **`raio × minor ≥ 40`.** O erro é o resto de **3ª ordem**, logo escala com a **CURVATURA**, que é
   governada pelo **menor raio local** — num bico achatado é `raio × minor`, não `raio`. Medido: uma
   elipse de `minor = 0,45` erra **6×** a redonda no mesmo raio, e `1/0,45² = 4,9`;
3. **o texel não pode STRADDLEAR a fronteira calota↔banda** — ali o `B` correto muda no meio da grade
   3×3 e nenhuma base única serve (0,77 nível contra 0,06 nas outras regiões).

⚠️ **A coincidência que não é coincidência:** a LUT é admissível a partir de `raio × minor ≥ 40`, e é a
partir daí que o AA custa caro (68,7 ms a raio 100 contra ~9 a raio 20). Os dois escalam com a pegada ⇒
**ela rende exactamente onde o custo está e é recusada exactamente onde erraria.**

### 2.4 O épsilon medido, por região

| região | r=20 | r=40 | r=100 |
|---|---|---|---|
| **CapsuleBand** | **0,02** | **0,00** | **0,00** |
| Disc | 0,44 | 0,06 | 0,00 |
| CapsuleCap | 0,66 | 0,06 | 0,00 |
| CapsuleStraddle *(excluído)* | 0,74 | 0,21 | 0,04 |
| Elipse (minor 0,45) | 2,68 | 0,30 | 0,01 |

Pior erro sobre **55 combinações**: **0,060 nível de u8**.

### 2.5 A fiação, em duas peças

1. **A LUT por TRAÇO, nunca por dab** (`height_film_lut::film_lut_for`) — memo de **UMA entrada**
   chaveado em `(falloff, bits da hardness)`, que é **tudo** o que `falloff_weight` lê (o
   `custom_falloff` fica fora porque `Custom` não é admissível ⇒ **a chave é COMPLETA**). ⚠️ Por dab
   seriam 16.384 avaliações contra ~1.800 amostras de banda a raio 20 — **9× mais caro que o que ela
   substitui**. O memo não é uma otimização do desenho: ele **é** o desenho. Nenhum dos **15** chamadores
   de `stamp_dab` mudou de assinatura.
2. **`FilmLutPlan`, a porta única por dab** — carrega a tabela e as DUAS bases e escolhe entre elas **por
   texel**, devolvendo `None` no straddle. A escolha mora ali, e não nos dois kernels, porque duas cópias
   da mesma pergunta divergiriam só na fronteira — onde ninguém olha.

⚠️ **A premissa da tabela é ESTRUTURAL, não uma condição conferida na porta:** ela tabula
`film_of(falloff_weight(t))`, ou seja assume que a silhueta É o falloff — e um `FilmAa` só existe quando
`film_aa_wanted` é verdadeiro, que exige `!shape_active`, que é **exatamente** quando o `silhouette_at`
dos dois kernels colapsa em `falloff_weight(t)`. **Ligar um Shape ao AA teria de passar por aqui.**

### 2.6 O resultado, e a projeção que eu errei

| traço (r=100, 2048², 600 px) | sem a LUT | com a LUT | ganho |
|---|---|---|---|
| 1º (tela virgem) | 134,84 ms | **110,22 ms** | **−18,3%** |
| 2º (sobre tinta) | 167,79 ms | **142,96 ms** | **−14,8%** |
| 3º | 172,57 ms | **144,92 ms** | **−16,0%** |

⚠️ **Eu havia projetado 31% (~39 ms) e o número honesto é 18% (~25 ms).** A projeção vinha do `10,3×`
medido entre a cadeia de silhueta e uma closure **CONSTANTE**, e tratava a substituição como **grátis**:
as nove amostras seguem custando nove leituras de tabela mais ~20 flops de expansão. **A razão contra
uma constante não é a razão contra uma tabela** — e essa é a lição transferível.

---

## 5. O que disto serve a **Watercolor** e a **Wet Paint**

### 5.1 ✅ Transfere INTEIRO: a cura do pen-down (§4)

É a resposta curta e a importante. O custo do pen-down é do **documento**, não do modelo de pintura:
Watercolor e Wet Paint pagam pelo menos o número do digital (**3 ms @2048² / 10,5 @4096²**) **mais** os
planos da sessão própria de cada um. Uma porta única de escrita de canvas + captura por região cura os
quatro modos com um desenho só.

### 5.2 ✅ Transfere: o harness de medição

`the_first_stroke_latency` e `the_impasto_dab_decomposition` dirigem o `PainterTool` e são
**mode-agnósticos** — armar o meio pelo dropdown e a mesma sonda mede Watercolor e Wet Paint. **Antes de
otimizar qualquer um dos dois, rode a decomposição:** esta jornada inteira existe porque a decomposição
disse onde o tempo estava, e três das minhas teorias sobre isso estavam erradas.

### 5.3 ✅ Transfere: o NEGATIVO da coalescência

`stamp_dabs` percorre a pegada de cada dab em **todos** os modos. Agrupar eventos de ponteiro não rende
nada em nenhum deles (§3.1). Item fechado para os três.

### 5.4 ⛔ A LUT **NÃO** transfere para Watercolor — e a razão é estrutural

O AA da aquarela usa **a MESMA grade 3×3** (`watercolor_field::AA_SS`, literalmente a mesma constante),
mas o que ele amostra é outra coisa: `smoothstep(e0, e1, bilinear(src, warp(pos)))` — **um campo
canvas-sized**, lido por interpolação bilinear, através do warp do Ragged Edge.

⚠️ **Não há `t` analítico, não há cadeia de falloff, não há `sqrt` a remover.** A LUT do filme é uma
tabela de uma função **fixa de um escalar**; a silhueta da aquarela é um **campo amostrado**. O custo
dela é **9 fetches bilineares de um buffer do tamanho da tela** — o gargalo é MEMÓRIA, não aritmética, e
a cura (se houver) é de localidade, não de tabela.

### 5.5 ⚠️ Transfere FRACO para Wet Paint — e o teto está medido

O Wet Paint chama `silhouette_at` **uma vez por texel por dab** (`wetpaint.rs:534`) — essa É a cadeia
analítica com `sqrt`, então uma tabela de `falloff_weight(t)` (não de `film_of ∘ falloff_weight`) a
substituiria.

Mas: **é 1 cadeia por texel, não 9.** O teto é ~9× menor que no impasto. E o custo do Wet Paint **não
está no stamp**: está no solver (0,83–0,89 ms/tick na sessão representativa; 18,9 ms no flood, que é o
pior caso declarado do ADR-0134). ⚠️ **Otimizar o stamp do wet seria trabalhar onde o tempo não está** —
a decomposição (§5.2) decide, não o palpite.

### 5.6 ✅ O que transfere de verdade é o PADRÃO, e o Wet Paint já o usou

O doc 24 tabulou a transferência sRGB (`libm::pow` 24 ns → 2,2 ns) e cortou o flood de 122,8 para
18,9 ms. Esta LUT é a **segunda instância da mesma ideia**: *tabular uma curva 1-D FIXA que é lida
milhões de vezes*. E as duas trazem a mesma cautela, aprendida no doc 24 e reconfirmada aqui:

* ⚠️ **tabular o que está DENTRO de um laço de realimentação caminha** (doc 24: a razão K/S fundida
  fazia uma lavagem PARADA derivar meio nível de byte em 5000 re-misturas). A LUT do filme é lida uma
  vez por texel por dab, **fora** de qualquer realimentação — por isso é segura;
* ⚠️ **o determinismo fica MAIS forte, não mais fraco:** os nós saem das mesmas funções do produto, e
  entre nós só correm `+ − * /`, que o IEEE-754 especifica exatamente.

### 5.7 ✅ E o segundo padrão que transfere: **um walk puro por-texel caminha por LINHAS**

O fold do relevo (§4.8.2) caiu 13,8× sem uma linha de aritmética nova — só a constatação de que cada
texel é **função pura de `(x, y)`** e de um estado congelado, logo as linhas são disjuntas (ADR-0109). O
teste para aplicar isto noutro lugar é uma pergunta só: *este laço lê algo que ele mesmo escreveu?*

* **Watercolor** — o `watercolor_field` **já** roda assim (`par_chunks_mut` em seis planos), e é dali que
  o desenho veio;
* **Wet Paint** — ⛔ **NÃO transfere, e não é falta de vontade:** o solver é **serial por semântica**
  (ADR-0134 diz explicitamente que o ADR-0109 é inaplicável e para não re-derivar). O que *poderia*
  paralelizar é o composite de folha cheia, que o doc 24 já derrubou de 133,3 para 18,2 ms por outra via;
* ⚠️ **o que NÃO transfere é o formato do GATE.** Paralelizar deixa a grandeza pequena, e uma razão entre
  dois números pequenos passa a medir o escalonador — quem paralelizar um caminho tem de reconferir os
  gates de perf que o cercam, que foi exatamente o que quebrou aqui.

### 5.9 🎯 A MEDIÇÃO DE 4 LINHAS FOI FEITA — e onde cada modo gasta um move

A §5.8 pedia esta tabela e ela é de minutos. `measure_the_four_media`, `on_canvas_pointer` (= o `INPUT
(fora do frame)` do app), pincel r=100:

| meio | move 2048² | move 4096² | composite | pen-up 2048²/4096² |
|---|---|---|---|---|
| Digital | 1,17 | 1,21 | 0 | 3,1 / 6,7 |
| **Watercolor** | **3,07** | **3,12** | 0 | 11,8 / 14,3 |
| Impasto | 2,00 | 1,93 | 2,9 | 19,4 / **39,6** |
| **Wet Paint** | **2,32** | **14,26** | 0 | 3,1 / 17,3 |

**Três dos quatro têm MOVE plano na tela** — limitado pela PEGADA, que é a forma correta.

⚠️ **O Wet Paint NÃO: 6× para 4× a tela.** Um move não pode escalar com o canvas, e isto é a assinatura
de uma varredura de plano inteiro — **a mesma família do fold que a §4.8.2 acabou de curar**. Não era o
alvo daquela wave e ficou **NOMEADO**, com o número, para não voltar como surpresa.

✅ **E FECHOU na wave seguinte (§5.12):** não era varredura nenhuma — era `Arc::make_mut` **copiando o
documento inteiro** porque o token de identidade do guard segurava um segundo `Arc` forte. Números novos
desta mesma tabela depois da cura: **Wet Paint 1,800 / 1,805** (move) e **3,0 / 5,05** (pen-up). Os
quatro meios passam a ter o move limitado pela pegada.

⚠️ E o **pen-up do Impasto a 4096² (39,6 ms)** é o maior da tabela — o candidato mais próximo do outlier
de 134,8 ms da §4.8.3, embora **não o alcance** com a fixture medida (traço de 960 px). Continua sem
causa atribuída.

### 5.10 ✅ DE QUE É FEITO UM MOVE DE AQUARELA — e a tabela que se auto-calibrou

`measure_what_a_watercolor_move_is_made_of`, a 4096², r=100. **Ablação por ENTRADA, nunca
instrumentação** (a lição da §4.8.2: uma sonda que re-implementa o laço fica cega à porta) — cada linha
dirige `on_canvas_pointer` e o que muda é um **knob do painel**.

| configuração | move ms | vs baseline |
|---|---|---|
| **AQUARELA (baseline)** | **3,082** | — |
| sem Warp | 2,012 | **−1,071** |
| sem Spread | 2,866 | −0,216 |
| sem Pigment mixing | 2,956 | −0,126 |
| sem Smudge | 2,982 | −0,100 ⚠️ *já era 0* |
| sem Granulation | 2,997 | −0,085 |
| sem Edge (gain 0) | 3,016 | −0,066 |
| sem Rewet | 3,021 | −0,061 ⚠️ *já era 0* |
| TUDO desligado | 2,011 | −1,071 |
| DIGITAL (o carimbo) | 1,167 | −1,915 |

⚠️ **A tabela trouxe o próprio CONTROLE, e isso não foi planejado:** `wet_smudge` e `wet_rewet` **já
valem 0 por default**, então aquelas duas linhas são **no-ops** — e mesmo assim medem −0,100 e −0,061.
**Esse é o piso de ruído da sonda.** Logo Granulation (−0,085), Edge (−0,066) e Pigment mixing (−0,126)
são **indistinguíveis de zero**, e teria sido fácil escrever três "otimizações" em cima delas.

O que sobrevive ao controle: **Warp (1,071)** e Spread (0,216). E `TUDO desligado` (2,011) é **igual** a
`sem Warp` (2,012) ⇒ **o warp é praticamente todo o custo ablacionável: 56% do que a aquarela cobra
sobre o Digital, 35% do move inteiro.**

⚠️ **E o custo é o NÚMERO de avaliações, não a avaliação:** com o supersample cortado de 9 taps para 1
(mutação de medição), o baseline cai para 2,401 e o warp para 0,511 — ~0,085 ms por avaliação, linear.
`warp_offset` roda **10× por texel** (centro + 9 taps). ⛔ **Cortar taps está FORA de discussão e não é
opinião:** rotear os 9 taps pelo warp foi o que **curou a borda serrilhada** numa wave anterior (warp 48
postava 226 cliffs; roteados, zero), e o doc do `aa_coverage` registra isso.

### 5.11 ✅ A fatoração dos dois eixos do warp — exata, e HONESTA sobre o que rendeu

`warp_offset` pede a **mesma oitava, na mesma posição**, para o eixo X e para o Y: só o `seed` difere.
Toda a aritmética de GRADE (2 divisões, 2 `floor`, 2 `smooth01`, 2 `wrap_cell`) **não depende do seed** e
era computada **duas** vezes. `value_noise_pair` a computa uma, e roda os 8 `hash2` (que dependem do
seed — é neles que a decorrelação dos eixos mora).

**Byte-exato por CONSTRUÇÃO:** mesmas operações, mesma ordem, mesmos valores; o prefixo comum é avaliado
uma vez em vez de duas. Não é aproximação, é fatoração — e o gate compara contra `warp_axis`, que é
**verbatim o que shipava**, com `assert_eq!` em `f32` (uma tolerância aceitaria uma aproximação, e a
afirmação que se quer é mais forte).

⚠️ **O NÚMERO, e a parte que não vende:** a função ficou **1,20×** mais rápida (153,41 → 127,86 ms em 4 M
avaliações, mínimo de 5 corridas). No **produto**, o ganho é **0,12–0,17 ms** de um move de 3,08 — contra
o piso de ruído de **±0,13** que a própria tabela calibrou. **Está dentro do ruído, logo não é um
resultado de produto**, e o commit não o chama de um. Fica porque é estritamente menos trabalho, exato,
gateado, e barateia o warp para qualquer consumidor futuro.

⚠️ **Duas consequências de higiene que valem mais que o número:**

1. **`warp_axis` ficou sem chamador de produção** ⇒ virou `#[cfg(test)]`, declarado **REFERÊNCIA
   CONGELADA**. Um `pub(super)` sem chamador não é código morto silencioso: é uma **segunda resposta**
   esperando alguém chamá-la — dois caminhos para *"qual é o deslocamento do warp aqui?"*, livres para
   divergir. E o doc do gate, que dizia *"é o código que shipava"*, teria virado **falso** sem o `cfg`.
2. ⚠️ **Uma mutação MINHA não sangrou, e o defeito era dela.** Eu escrevera que trocar a ordem dos termos
   (`bx·0,35 + ax·0,65`) sangraria *"porque em `f32` não é igual"*. **É igual:** a adição IEEE-754 é
   **COMUTATIVA** (`a + b == b + a` exatamente); o que falha é a **ASSOCIATIVIDADE**. Aquela mutação era
   um no-op e não podia sangrar. A mutação certa — **cruzar os seeds dos dois eixos**, que é o erro
   realista desta refatoração — sangra.

### 5.12 🎯🎯 FECHADA — o move do Wet Paint: **um token de identidade cobrava uma cópia do documento**

O censo da §5.9 nomeou o desvio: o Wet Paint era o único cujo MOVE **subia com a tela** (2,32 → 14,26 ms
de 2048² para 4096²) enquanto os outros três ficavam planos. Um move é limitado pela **PEGADA** — o
pincel cobre o mesmo número de texels seja qual for o documento —, então subir com a área é assinatura
de **varredura de plano**, a mesma família do fold que a §4.8.2 curou.

#### A FORMA respondeu antes do relógio

A primeira pergunta não foi *"quanto custa?"* e sim *"quanto ele MARCA?"* — a área que o move declara
suja, perguntada ao produto (`t.marks`, `measure_what_a_wet_move_marks`):

| tela | move ms | texels sujos | vs pegada | vs tela |
|---|---|---|---|---|
| 1024² | 1,867 | 15.625 | 0,33× | 1,49% |
| 2048² | 2,262 | 15.625 | 0,33× | 0,37% |
| 4096² | **13,711** | **15.625** | 0,33× | 0,09% |

**A região suja é CONSTANTE** ⇒ o composite não era o plano. Um cronômetro sozinho diria *quanto*; a
área disse *o quê* — e eliminou o suspeito óbvio numa linha.

#### A causa: `Arc::make_mut` só é grátis para o dono ÚNICO

`wetpaint_composite` termina em `Arc::make_mut(&mut self.canvas_rgba)`, que devolve o slice se o tool for
dono único e **CLONA O DOCUMENTO INTEIRO** se não for. Medido, logo depois de um move:

| tela | donos (Watercolor) | donos (Wet Paint) | cópia de tela |
|---|---|---|---|
| 1024² | 1 | **2** | 0,061 ms |
| 2048² | 1 | **2** | 0,389 ms |
| 4096² | 1 | **2** | **10,254 ms** |

O segundo dono era **`WetSession.canvas`** — um clone **forte** guardado *só* para responder ao guard de
identidade (*"alguém trocou o canvas debaixo de mim?"*). ⚠️ **Uma pergunta de IDENTIDADE estava sendo
paga com POSSE**, e o preço era uma cópia do documento por movimento do mouse.

⚠️ **A PRIMEIRA cópia é legítima e continua acontecendo:** `sess.base` é a tela **congelada** sobre a
qual todo composite renderiza, e escrever no lugar a destruiria. É por isso que o gate mede depois do
segundo composite, não do primeiro.

#### A cura foi MEDIDA antes de escolhida

Duas formas de tirar a posse sem perder a identidade — soltar o handle durante a escrita, ou guardá-lo
como **`Weak`**. A dúvida era o que `Arc::make_mut` faz com um `Weak` vivo; isso é afirmação sobre a
`std`, então foi medido em vez de citado:

| tela | dono único | com `Weak` | com `Arc` (o produto de então) |
|---|---|---|---|
| 1024² | 0,0000 | 0,0000 | 0,3514 |
| 2048² | 0,0000 | 0,0000 | 2,0226 |
| 4096² | 0,0000 | 0,0000 | **9,8580** |

**O `Weak` custa zero** — `make_mut` move os 24 bytes de cabeçalho do `Vec` para uma alocação nova e
deixa os pixels onde estão. E ele é o handle **certo por um segundo motivo**: um `Weak` **prende a
alocação**, então nenhum `Arc` futuro pode nascer naquele endereço ⇒ a comparação de ponteiro é sã. É
exatamente o ABA que torna insegura a identificação por endereço cru — a lição que o **ADR-0124** pagou
no editor de áudio (*"pergunte ao valor, nunca ao `as_ptr()` de algo que você não prendeu"*).

#### Resultado

Move **13,71 → 1,82 ms a 4096² (7,5×)**, e **plano na tela** (1,842 / 1,815 / 1,817). No censo dos quatro
meios o pen-up caiu junto (**17,3 → 5,05 ms**), porque ele compõe pela mesma porta. **Os quatro meios
agora têm o move limitado pela pegada** — o desvio de forma que a tabela da §5.9 mostrava sumiu.

#### Dois gates, e DOIS defeitos meus nos gates

* **A PROPRIEDADE** (`the_wet_session_does_not_own_the_live_canvas`): depois de um move o tool é dono
  único. Sem relógio, logo sem ruído. Mutação (token forte de volta) ⇒ **2 donos**, RED.
* **A CONSEQUÊNCIA** (razão entre duas telas, a disciplina do `warp_perf_kill_criterion`). Mutação ⇒
  **11,21×**, RED.

Eles **não são redundantes**: um passe canvas-sized *novo* no composite passaria pelo primeiro e cairia
no segundo — *a sessão não possui o canvas* × *o move não percorre o plano*.

⚠️ **O gate de razão nasceu CEGO, duas vezes, e as duas por FIXTURE:**

1. **O par era 1024²/2048².** A cópia custa 0,06 e 0,39 ms ali, contra ~1,8 ms de trabalho de pegada —
   então nas telas pequenas o defeito é **ruído**: sob a mutação a razão media **1,14×** e o gate ficava
   **VERDE**. Só a 4096² (10,25 ms) ele domina. *Uma fixture só prova o que ela contém*, e montar a tela
   grande é o preço de o gate poder falhar pelo motivo que alega.
2. **O redutor era o MÍNIMO.** O gate do fold (§4.8.2) lê o mínimo *com razão* — máquina carregada só
   sabe deixar mais lento —, mas lá **toda** amostra faz o mesmo trabalho. Aqui não: o **PRIMEIRO** move
   de um traço não compõe (o espaçamento ainda não emitiu dab, então a rota volta antes do composite) e
   mede **0,22 ms nas DUAS telas**, contra 1,0–3,6 e 12,0–13,6 nos oito seguintes. **O mínimo era
   exactamente a amostra sem o fenômeno**, e a razão entre duas delas dava **1,00×**. Mediana.

**A lição geral, que é nova neste doc:** *o mínimo é o redutor certo quando toda amostra faz o mesmo
trabalho, e o errado quando uma delas é estruturalmente diferente.* O redutor é parte da fixture.

⚠️ **E o guard continua vivo** — neutralizá-lo sangra 2 gates existentes (o swap estrangeiro no tick e o
eraser de re-stamp), então a troca `Arc`→`Weak` não comprou velocidade com correção.

**LOC:** `wetpaint.rs` bateu **720 > 700** com o doc-comment da medição ⇒ split por responsabilidade em
`wetpaint/session.rs` (*o que uma sessão É*) contra o que sobrou (*o que o tool FAZ com ela*: a rota do
dab, o ciclo do traço, o guard). 585 + 146.

### 5.13 🎯 O PIOR EVENTO DE UM TRAÇO É O **PEN-UP**, e ele é limitado pela TELA (frentes Q e R)

A §7 pedia *"comece medindo por FASE — um número só não nomeia culpado"*. A forma mais barata de fazer
isso não é instrumentar: é **rodar o ciclo inteiro de um traço e imprimir todo evento**, deixando o pico
se nomear sozinho (`the_worst_single_event_of_a_stroke_names_itself`, 4096², impasto).

| raio | pen-down | move (mediana) | **pen-up** |
|---|---|---|---|
| 20 | 4,34 | 0,76 | **31,50** |
| 100 | 10,66 | 1,97 | **40,94** |

⚠️ **O pior evento não é o pen-down nem um move: é o PEN-UP**, ~20× um move e ~4× o pen-down. E ele
**quase não responde ao raio do pincel** (31,5 contra 40,9 para 5× o raio) — a assinatura de trabalho
limitado pela **TELA**, não pela pegada.

#### Por tela e por meio

`what_the_pen_up_is_made_of` (traço FIXO em px, 8 traços, o 1º descartado, mediana dos sete):

| tela | Digital | **Impasto** |
|---|---|---|
| 1024² | 0,72 | 6,5 |
| 2048² | 1,25 | 11,9 |
| 4096² | **6,05** | **38,9** |

De 2048² para 4096²: **4,86× e 3,26× para 4× de área** ⇒ **plane-bound nos dois**. O impasto acrescenta
**32,8 ms** a 4096².

#### A aritmética, TESTADA em vez de afirmada

`commit_stroke_height` é **window-bound de propósito** — o doc-comment dele diz e gateia isso. Exceto
por uma coisa: ele chama `plane_fork::fork_par` em cada plano de relevo, e **um fork é canvas-sized**,
porque o `ModelSnapshot` que o pen-down tira para o undo segura um `Arc` de cada plano.

Se o fork for a causa, clonar os três planos tem de casar com os 32,8 ms — e isso é uma **previsão que
pode dar errado**, que é o que a torna um teste:

| plano | MB @4096² | clone |
|---|---|---|
| `covers` (u8) | 16 | 0,40 ms |
| `heights` (f32) | 64 | 9,94 ms |
| `mats` (7 B) | **112** | **18,13 ms** |
| **soma do que o impasto acrescenta** | **192** | **28,47 ms** |

**28,47 contra 32,8 medidos: fecha.** ⚠️ **E a 2048² ela NÃO fecha** (1,37 de clone contra 10,6 de
acréscimo) — ali os clones pequenos medem **48 GB/s** contra **6,5 GB/s** nos grandes, ou seja o
microbenchmark pega um caminho favorecido pelo alocador. **A afirmação fica escopada a 4096²**, que é o
tamanho onde o artista sente.

> ## ⛔ ERRATA — ESTA ARITMÉTICA FECHAVA POR COINCIDÊNCIA, E A CAUSA É OUTRA (ver **§5.14**)
>
> `Vec::clone` é um memcpy **SERIAL**, e o produto **não usa memcpy**: usa `plane_fork::fork_par`, que é
> **paralelo**. Medido pela porta do produto a 4096², o fork dos três planos custa **9,25 ms**, não
> **28,47** — a soma acima casava com os 32,8 **por acaso**. ⚠️ *Uma atribuição que bate com o total por
> acidente é pior que nenhuma: ela encerra a investigação no lugar errado* — e encerrou.
>
> A causa verdadeira é o **COMMIT DE UNDO** (§5.14). O parágrafo abaixo descreve um mecanismo **real mas
> menor**, e fica como registro do que a wave seguinte vai colher.

**O mecanismo, numa frase:** o snapshot de undo tirado no **pen-down** segura um `Arc` de cada plano de
relevo, então o commit no **pen-up** bifurca os três — ~192 MB de cópia a 4096². É a **mesma família** do
fork do canvas no pen-down (§4.3 / o gate `the_pen_down_is_still_a_canvas_copy`), e a cura é a mesma:
**capturar o "antes" por REGIÃO, sob demanda** — o *tile-based undo* do GIMP/Krita, que a §13.12.5 do
doc 25 já prescreve e que precisa de uma **porta única de escrita de plano** (hoje ~25 sítios chamam
`Arc::make_mut` direto). Isso é **wave própria**, e o preço dela — corrigido pela §5.14 — é
**pen-down 11,7 ms (canvas) + pen-up 9,2 ms (relevo)**, não os 28,5 que esta seção anunciou.

#### ⚠️ E DUAS coisas minhas que a medição derrubou

1. **A primeira versão da sonda media UM pen-up por configuração e NÃO REPRODUZIA:** a mesma célula deu
   **117,76 ms** numa corrida e **28,46** na seguinte. Buffers de 67–117 MB pagam *first-touch* e o
   alocador tem memória entre chamadas, então um gesto único mede o estado do heap tanto quanto mede o
   produto. *Um número que não reproduz não é um achado; é ruído com casas decimais.*
2. **A hipótese que eu montei em cima daquele 117,8 — *"é o first-touch do 1º traço"* — está REFUTADA
   pela própria sonda:** o 1º traço é **mais barato** (26–29 ms) que a mediana (38–39). Os 117,8 **não
   têm mecanismo**, e inventar um teria enterrado o número certo debaixo de uma narrativa.

⚠️ **Consequência para a frente R:** o outlier de **134,8 ms** do app **continua sem causa atribuída**.
O maior evento reprodutível que existe é o pen-up a **38,9 ms**, e ele não chega lá. O que esta wave
entrega é o candidato NOMEADO e MEDIDO (não o veredito), mais a demonstração de que uma amostra única
pode passar de 100 ms sem que nada esteja errado — o que torna o 134,8 do log **compatível com um
pen-up normal medido num instante ruim**, hipótese que só um segundo log com histograma decide.

---

### 5.14 ✅ E o pen-up era o **COMMIT DE UNDO** — 91% dele, e a §5.13 atribuiu ao lugar errado

Indo abrir a wave da porta única, a primeira coisa a fazer era o que §5.13 não fez: **perguntar quantos
donos cada plano tem e de quem são** — porque remover um de dois não remove cópia nenhuma. Duas sondas
(`measure_stroke_owners`) derrubaram as duas metades da §5.13.

#### 1. O dono extra é PERMANENTE, e é o HISTÓRICO

| momento | `canvas` | `heights` | `covers` | `mats` |
|---|---|---|---|---|
| repouso (2 traços commitados) | **2** | **2** | **2** | **2** |
| …depois de `undo.clear()` | 1 | 1 | 1 | 1 |
| dentro do gesto (pós pen-down) | 1 | 4 | 4 | 4 |

O `cursor` que a **U1** instalou como base de todo delta é um **segundo dono permanente**. A hipótese
fácil — *"o dono extra é o snapshot de pen-down"* — descrevia metade, e a consequência de projeto é
dura: **um journal que substituísse só o `paint.stroke_undo` deixaria a contagem em 2 e não mudaria um
milissegundo.**

#### 2. O fork custa um TERÇO do que a §5.13 somou

| plano | MB @4096² | `fork_par` (produto) | `Vec::clone` (o que a §5.13 somou) |
|---|---|---|---|
| `covers` | 16 | 0,29 | 0,73 |
| `heights` | 64 | 3,16 | 11,30 |
| `mats` | 112 | 5,79 | 19,15 |
| **soma** | 192 | **9,25 ms** | 31,2 |

⚠️ A §5.13 mediu **memcpy serial** e o produto usa **fork paralelo**. Os 28,47 que "fechavam" com os
32,8 fechavam **por coincidência**.

#### 3. O pen-up é o commit de undo; o `commit_stroke_height` é FOOTPRINT-BOUND

Ablação pela ENTRADA (`paint.stroke_undo = None` faz o `close_stroke` pular `commit_structural_edit`):

| pen-up, mediana de 7 | 1024² | 2048² | 4096² |
|---|---|---|---|
| impasto **completo** | 5,98 | 10,70 | **40,20** |
| impasto **sem o commit** | 3,35 | 3,43 | **3,49** |
| digital completo | 0,86 | 1,07 | 5,64 |
| digital sem o commit | 0,57 | 0,57 | 0,57 |

**91% do pen-up era o histórico** — e o que sobra é **plano na tela**, que é a forma correta para
trabalho limitado pela pegada. ⚠️ A ablação tira **duas** coisas de uma vez (o commit *e* o segundo dono
que aquele snapshot representa), e é por isso que os 36,7 ms de diferença são mais que o
`record_structural` isolado: `commit` + `forks` (9,25) + o `free()` dos buffers que o fork deixou órfãos.

#### A cura: o scan do commit é leitura pura sobre linhas disjuntas

`PlaneDeltas::split` roda **`diff_window` sobre todo plano que os `Arc`s não deram como idêntico** — com
impasto são quatro, ~**256 MB de comparação por traço** — num row-scan **serial**. É a forma exata que o
**ADR-0109** sanciona e que esta crate já usa quatro vezes (`sculpt_offset`, `sculpt_close`, o campo da
aquarela, o fold da luz): **byte-idêntica por construção**, porque `min`/`max` sobre índices é
associativo e comutativo — muda qual thread avalia qual linha, nunca o que a linha responde.

⚠️ **A varredura de COLUNAS também é paralela**, e não por simetria: num traço **VERTICAL** a faixa é a
tela inteira em linhas, e ela é element-a-element (não há memcmp que devolva *onde*).

| | antes | depois |
|---|---|---|
| `record_structural` @4096² digital | 5,04 | **3,87** |
| `record_structural` @4096² impasto | 25,03 | **10,96** (2,2×) |
| scans @4096² (canvas·covers·heights·mats) | ~15 | **0,72 · 0,32 · 1,09 · 2,12** |
| pen-up @4096² impasto | 40,20 | **32,34** |
| `snapshot_model` | — | **0,00** (clones de `Arc`) |

⚠️ **E o doc do `diff_window` foi CORRIGIDO junto:** ele dizia *"uma varredura por linha, uma vez por
commit (user-paced)"*, como se *user-paced* quisesse dizer barato. Um commit acontece no **pen-up de
todo traço**. A decisão de **calcular** a janela em vez de recebê-la continua certa (uma janela informada
errado não falha: some com texels em silêncio) — o que estava errado era o preço estimado dela.

#### ⚠️ E DUAS lições de medição que custaram a atribuição inteira

1. **A fixture ENVENENADA (a §5.40, outra vez).** O primeiro corte da sonda de
   produto tirava o snapshot **sem rodar `rebuild_active_region`** — que é o 1º
   estágio de todo passo do worker. Sem ele a máscara `active` do estado
   congelado está VAZIA, todo passe gateado nela faz early-out em TODA célula, e
   o `drying_pass` (que não é gateado do mesmo jeito) aparece sozinho. O
   `build_flow_field` media **4,64 ms** onde o produto paga **66,14** — **14×** —
   e os passes somavam 25 ms contra um passo medido de 63. *Números por passe
   que não RECONCILIAM com o passo são a assinatura desta doença.*
2. **A amostragem bilinear do `advect` custava 15,9 ms** — medido por ablação
   para *nearest* (que é rápido e dá a frente em BLOCOS, o artefato que a wave
   remove): o passo caía de 59,6 para 41,1 ms. Ela sozinha comia dois terços do
   que o `build_flow_field` economizava, e na fixture pequena era **invisível**
   (274k células ativas contra ~10 M).

### O `FlowRowSampler` — as 8 cargas por BLOCO, não por célula

A cura da (2), e o desenho é o que a mantém honesta: os quatro cantos e a
metade-`y` dos pesos **não mudam enquanto `x` anda dentro do mesmo bloco**, então
as 8 cargas acontecem uma vez por bloco. ⚠️ **Não é uma segunda resposta, e o
gate é o que garante isso:** ele computa `u`, os índices, os pesos e a soma **na
mesma ordem** que o `flow_at_point`, e há gate afirmando igualdade **BIT A BIT**
numa varredura de razões, larguras e posições — com o campo de teste
deliberadamente **estruturado**, porque um campo chato faria qualquer
amostragem concordar e o gate seria verde por vácuo.

**`advect` 41,93 → 34,51 ms · o passo 59,6 → 50,7 · a wave 1,10× → 1,25×.**

### Gates, e as duas mutações que ensinaram algo

O oráculo é a **varredura serial congelada sob `cfg(test)`** — o código que shipava, verbatim. ⚠️ Ela
mora sob `cfg` de propósito: um `fn` privado sem chamador de produção não é código morto silencioso, é
uma **segunda resposta** esperando alguém chamá-la. Fixtures **atravessam o `PAR_MIN`** (abaixo dele o
produto roda a rota serial, e um gate que só a exercitasse compararia o caminho antigo com ele mesmo) e
cobrem os quatro tipos que o histórico guarda — `f32` e `[u8; 7]` **não** comparam por memcmp.

**5 mutações: 4 sangram nos gates de identidade; a 5ª (voltar a ser serial) sangra só no de RAZÃO, por
desenho** — as duas rotas produzem a mesma janela, então só o relógio pode vê-las.

- ⚠️ **Uma sobreviveu por FIXTURE:** trocar a identidade da redução de colunas por `(0, 0)` passou porque
  **toda banda larga das minhas fixtures acertava a coluna 0** por acidente — e `min(0, c) = 0` concorda
  com a resposta certa. A fixture que faltava é uma banda **alta** cujas colunas ficam **longe do zero**.
- ⚠️ **Uma era mutação INVÁLIDA, não buraco de gate:** varrer as colunas a partir da linha 0 em vez da
  primeira linha diferente é **no-op semântico** (linhas iguais devolvem a identidade e não movem
  `min`/`max`) — desperdício, nunca erro. A que de fato erra exclui a ÚLTIMA linha, e sangra.

**Aberto, e agora com o preço certo:** a **porta única de escrita de plano** vale
**pen-down 11,7 + pen-up 9,2 ms** de forks — e ela precisa alcançar o **histórico** também, porque o
`cursor` é um dono permanente.

### 5.15 ✅ A PORTA ÚNICA de fork do canvas — e o `Weak` que ela contava como dono

Com a atribuição corrigida (§5.14), a frente da porta única foi aberta pela metade que é **mecânica e
segura**: fazer TODO sítio forkar o canvas pela rota paralela.

**O estado que ela encontrou:** só o depósito de pigmento (`stamp_cache`, 10 sítios) vinha pela porta;
**23 sítios** — fill, smear, blur, clone, seleção, warp, máscara, inpaint, aquarela, e o **composite do
Wet Paint, que roda a cada TICK** — chamavam `Arc::make_mut` cru, isto é, a cópia **serial**. E a
primeira escrita de **todo** gesto forka, porque o `cursor` do histórico é dono permanente (§5.14).

⚠️ **O doc do gate antigo justificava o escopo estreito com uma afirmação FALSA:** *"o pen-down é o
único sítio onde o `Arc` do canvas tem um segundo dono garantido"*. Ele tem dois donos **sempre**.

| blur pen-down | 2048² | 4096² |
|---|---|---|
| `Arc::make_mut` serial | 1,11 | **11,64** |
| `fork_par` paralelo | 0,85 | **3,66** (3,2×) |

#### ⚠️ E a migração expôs um defeito MEU — a frente V mordendo pelo outro lado

`fork_par` perguntava `Arc::get_mut(arc).is_none()` para decidir se havia dono. **`get_mut` devolve
`None` na presença de QUALQUER `Weak`**; `Arc::make_mut` só **copia** com outro **strong** (com só
`Weak` vivo ele *move* o valor — foi isso que a §5.12 mediu em 0,0000 ms). O guard de identidade do Wet
Paint é **precisamente um `Weak`**, então o composite passou a **copiar o canvas inteiro por movimento
do mouse**, com o `make_mut` da linha seguinte movendo-o de graça.

**A pergunta certa é a que o COPIADOR faz:** `strong_count > 1`. Ela é um palpite sobre *como* copiar,
nunca sobre *se* — quem decide continua sendo o `make_mut` final.

⚠️ O sintoma foi o **gate de razão da §5.12 voltando a 4,77×** — nunca uma falha de comportamento, porque
as duas rotas dão os mesmos bytes. Por isso o gate novo afirma a **propriedade direto** (*um `Weak` vivo
não dispara cópia*): um defeito que só um relógio enxerga é um defeito que uma máquina carregada esconde.

#### O arch-gate mudou de ESCOPO, e é a lição

O gate antigo lia **um arquivo**. O novo varre **`tool/paint/**` inteiro**, com controle positivo nas
duas pontas (arquivos varridos · escritas pela porta) e prosa isenta — *um gate por-arquivo protege o
arquivo que alguém lembrou de listar; o sítio 24 nasce coberto, que é exatamente como os 23 nasceram
descobertos.*

#### ⚠️ DUAS fixtures de medição nasceram cegas, por motivos diferentes

1. **Um FILL** custa ~130 ms por conta própria a 4096², e a variação entre corridas é **maior** que os
   ~6 ms do fork: a diferença saía **negativa**. *Um sinal só é mensurável contra um fundo menor que ele.*
2. **Ablar o HISTÓRICO** (`undo.clear()`) **não** remove o segundo dono de um traço — o `stroke_undo`
   nasce DENTRO do `paint_begin`, então os dois braços forkam e a diferença é **zero**. A ablação certa é
   trocar a **ROTA** no mesmo gesto.

**O que esta wave NÃO é:** ela **acelera** o fork, não o remove. Removê-lo é capturar o "antes" por
REGIÃO (o *tile-based undo*), e a §5.14 mostrou que essa wave tem de alcançar o **histórico** também.

### 5.16 ✅ O CTRL+Z era 46,6 ms — e a otimização que eu ia fazer foi MORTA pela medição

Abrindo a wave da captura por região, a primeira coisa foi a que a §5.13 **não** fez: perguntar de que o
pen-up é feito **pelas portas do produto**, em vez de atribuir por subtração. A ablação da §5.14 tira duas
coisas de uma vez (o commit **e** o segundo dono que o snapshot representa), então a diferença dela não é
o commit. Cronometrando as duas metades separadamente, com o snapshot VIVO, chamando
`commit_stroke_height` e depois o `Up` na ordem do `close_stroke`
(`measure_stroke_owners::what_the_two_halves_of_the_pen_up_cost`):

| pen-up impasto (ms)             | 1024² | 2048² | 4096² |
|---|---|---|---|
| `commit_stroke_height` (o fold) | 2,85 | 7,62 | **13,28** |
| o resto (o commit de undo)      | 6,14 | 10,40 | **23,72** |
| TOTAL                           | 8,99 | 18,01 | **37,00** |

E dentro do commit (`what_the_record_structural_is_made_of`): `PlaneDeltas::split` **é** o
`record_structural` (9,57 contra 10,64 a 4096²) — a contabilidade do controller custa ~1 ms.

#### ⛔ MEDIDO E REJEITADO — não refaça: paralelizar a EXTRAÇÃO da janela

A hipótese natural era que os ~5,5 ms que sobram entre os scans (4,02 medidos) e o `split` (9,57) fossem
o `Window::extract`, que copia `2 × janela` por plano num laço serial. **Instrumentando o `split` plano a
plano, é falso:** o custo é por-plano e proporcional ao TAMANHO DO PLANO, não ao da janela —

```text
[split] canvas_rgba  3,0–5,5 ms      [split] heights  2,4–5,2 ms
[split] covers       0,7 ms          [split] mats     4,0–9,0 ms
```

— enquanto a janela de um traço de 7 movimentos é ~160 linhas de ~280 colunas, isto é **kilobytes**. O
`split` está **limitado por LARGURA DE BANDA lendo os dois endpoints dos quatro planos** (~470 MB por
traço a 4096²), e a sonda sintética que media 4,02 ms media os mesmos buffers **quentes**. Uma extração
paralela renderia **zero**, e eu ia construí-la.

⚠️ **Corolário que decide a wave seguinte:** o único jeito de tirar esses 9,5 ms é **não ler os planos** —
receber a janela de quem escreveu, em vez de derivá-la do conteúdo. Ver §7.

#### ✅ O que a medição ACHOU no lugar: a materialização de um Ctrl+Z

O outro lado do delta nunca tinha sido medido pela porta. `StoredPlane::side` monta o estado a instalar
**clonando o plano do cursor** (é ele que serve tudo fora da janela) e depois blita a janela por cima —
uma cópia de documento **por plano que a entrada tocou**, num `Vec::clone` serial:

| Ctrl+Z (mediana de 7, impasto) | 1024² | 2048² | 4096² |
|---|---|---|---|
| antes (serial)                 | 0,42 | 3,12 | **46,56** |
| agora                          | 0,52 | 3,35 | **23,41** (2,1×) |

⚠️ **O número serial de 4096² não é largura de banda** — 5,8 GB/s é lento demais para isso. É o
*first-touch* de 67–117 MB recém-alocados, uma falha de página por vez, exatamente o mecanismo que o
`plane_fork` já tinha documentado; e é por isso que espalhar por threads o conserta.

#### ⚠️ O primitivo virou UM, e o limiar mudou de unidade

A porta de fork e o motor de delta copiam o mesmo tipo de coisa pelo mesmo motivo, então a cópia paralela
passou a morar em **`crate::plane_copy`** (`par_clone` + `worth_parallel`), com os dois perguntando à
MESMA função. ⚠️ **E o limiar deixou de ser em ELEMENTOS para ser em BYTES**, porque a medição obrigou: um
plano de `[u8; 7]` move sete vezes a memória de um de `u8` com a mesma contagem, e com o limiar em
elementos a 1024² mandava quatro cópias pequenas para o rayon e **dobrava** o Ctrl+Z (0,42 → 0,86 ms). A
virada medida está entre 29 e 67 MB ⇒ **32 MB**: a 2048² tudo segue serial (onde o serial ganha) e a
4096² os três planos grandes vão para o paralelo. O pen-down do Blur **não perdeu nada** com isso (0,82 a
2048² · 3,37 a 4096², contra 0,85/3,66 antes).

**Gates:** a rota serial de materialização ficou **congelada sob `cfg(test)`** como oráculo
(`serial_side`) — um `pub` sem chamador seria uma segunda resposta esperando alguém chamá-la, a lição que
o `warp_axis` da §5.11 pagou —, mais o gate de RAZÃO (3,2× num plano de `f32` de 4096²; barra em 1,5×
porque `ci-test` compila em `opt-level=1`) e um gate que pina a **unidade** do limiar. **4 mutações, 4
sangram:** voltar `side` ao clone serial ⇒ 1,0× · `worth_parallel` sempre falso ⇒ 1,0× · `par_clone`
corrompendo ⇒ 3 gates de paridade · o limiar de volta em elementos ⇒ o gate da unidade.

⚠️ **O que esta wave NÃO toca:** o pen-up. O `side` não roda num commit — ele roda num **undo**. O
pen-up segue em 37 ms a 4096² e a §7 diz o que o desmonta.

### 5.17 ⛔ S1 TENTADA E REVERTIDA — `mark_dirty` declara onde a IMAGEM mudou, não onde os BYTES foram escritos

A primeira tentativa da wave da janela (§7 item 1) escolheu o caminho de **zero churn**: `mark_dirty`
recebe o retângulo de toda escrita de canvas e já é atravessado por todas elas, então bastaria uni-los
numa janela e entregá-la ao commit — sem mexer nos 41 sítios de `fork_par`. A plumbing foi construída
inteira (`undo::window::WriteWindow` com o contador de proveniência que recusa um `before` anterior ao
último commit, `PlaneWindow::from_region`, o hint atravessando `record_structural` → `PlaneDeltas` →
`StoredPlane`), mais a rede que a tornaria segura: **em build de DEBUG o `split` deriva a janela
verdadeira e afirma que ela cabe na declarada**.

**A rede disparou na PRIMEIRA rodada da suíte** — e o que ela achou mata o desenho:

```text
a janela declarada nao contem a verdadeira:
  declarada  row 33  rows 94   col 33  cols 94
  real       row 32  rows 96   col 32  cols 96
```

⚠️ **A premissa era falsa, e o próprio código diz por quê.** O `mark_dirty` do Inflate está comentado
como *"Dirty exactly where the picture changed"* e recebe `moved`, o conjunto dos texels cujo relevo
**passou do `RELIEF_EPS`**. Mas o kernel escreve `target[gi] = next` em **toda** a janela `kr`,
incondicionalmente — então há bytes que mudam sem a imagem mudar. *Onde a imagem mudou* e *onde bytes
foram escritos* são perguntas diferentes, e o undo precisa da segunda.

**Consequência para o plano:** a declaração **tem** de sair de quem escreve, não de quem anuncia o
repaint — isto é, o S1 original (`fork_par` recebe a região), com o custo de churn que eu tinha tentado
evitar. ⚠️ E a variante barata não é resgatável com um `grow_region`: um pad arbitrário é um palpite
sobre um número que outro kernel pode mudar, exatamente o que a §0 do CLAUDE.md proíbe.

**O que fica desta tentativa, e vale mais que o código revertido:**

1. **A rede de verificação é o desenho certo e deve voltar junto com o S1.** Ela é barata (só em debug),
   roda em ~4 s sobre 866 testes que exercitam fill, seleção, warp, sculpt, máscara, inpaint, clone,
   aquarela, Wet Paint e os shape editors, e **pegou o defeito antes de qualquer gate especializado**.
2. **O contador de proveniência** (`hint_for`: aceita a janela só se o `before` for posterior ao último
   commit ⇒ superconjunto por construção) resolve as transações aninhadas sem enumerar chamadores, e
   serve igual no S1 original.
3. ⚠️ **E há um achado LATENTE fora desta wave:** se o `mark_dirty` do Inflate sub-declara o que foi
   escrito, então a pista de **upload parcial** recebe a mesma reivindicação curta. Aqui isso é
   invisível (os bytes a mais têm delta abaixo do `RELIEF_EPS`, que é o critério do `moved`), mas é uma
   sub-declaração real e ninguém a tinha visto. **Não corrigido nesta sessão** — o `moved` é o que dá o
   custo do repaint, e alargá-lo é decisão do dono do sculpt.

O patch da tentativa está preservado fora da árvore; refazê-lo sobre o `fork_par` é mecânico.

### 5.18 🔧 S1 pela porta de escrita — o DESENHO está certo, e é um GUARD (parcial, parado no borrow checker)

A §5.17 mostrou que a declaração tem de sair de quem escreve. A tentativa seguinte construiu isso, e o
desenho que ela achou é o certo — fica escrito para a próxima sessão retomar sem redescobri-lo.

**Não é um argumento do `fork_par`, é um GUARD.** A região quase sempre só é conhecida no FIM: os laços
de dab acumulam o `touched` *enquanto* escrevem, então `fork_par(arc, rect)` não teria o que receber.

```rust
pub(super) struct PlaneWrite<'a, T> { buf: &'a mut Vec<T>, win: &'a WindowCell, declared: bool }
impl PlaneWrite {
    fn wrote(self, r: Option<Region>);   // a região do canvas que este acesso escreveu
    fn wrote_everywhere(self);           // desiste explicitamente: o commit varre
}
impl Drop { if !declared { win.set(WriteWindow::UNKNOWN) } }   // ⚠️ esquecer é SEGURO
```

⚠️ **Esquecer marca a janela como desconhecida e o commit varre** — que é exatamente o que ele faz hoje.
O modo de falha de um sítio novo passa a ser *lento*, nunca *errado*: é a resposta à objeção do
`diff_window` que a §5.17 tentou dar pelo canal errado. E `Deref`/`DerefMut` para `Vec<T>` mantêm os 41
sítios escrevendo como escrevem hoje.

⚠️ **A janela precisa de `Cell`** (`WindowCell = Cell<WriteWindow>`): um gesto tem vários planos abertos
ao mesmo tempo — o Inflate escreve altura, cobertura, material e RGBA numa tacada — e com `&mut` os
guards se excluiriam. Isso torna o tool `!Sync`, o que é **livre**: `Tool: std::any::Any` e nada no editor
exige `Sync` (conferido).

**Onde parou, e qual é o próximo passo exato.** A migração mecânica dos 41 sítios deixou 104 erros, de
três famílias, e as três têm cura conhecida:

1. **`E0596` (12+)** — `let buf = …` vira `let mut buf`, porque agora o `DerefMut` precisa do guard
   mutável. Mecânico.
2. **`E0499` (14) — a mais interessante, e provavelmente a causa das outras:** `fork_par<'a>(arc: &'a mut
   …, win: &'a WindowCell)` **amarra os dois empréstimos ao MESMO tempo de vida**, então o compilador
   deixa de tratá-los como campos disjuntos. A cura é dar-lhes tempos de vida INDEPENDENTES
   (`fork_par<'a, 'w>` → `PlaneWrite<'a, 'w, T>`).
3. **`E0308` (15)** — sítios que passam o `&mut Vec<T>` adiante; viram `&mut *guard`.

O patch da tentativa está preservado fora da árvore. **A árvore está verde e inalterada** — nada disto
foi commitado, porque uma migração de 41 sítios pela metade é pior que nenhuma.

### 5.19 ✅ S1+S2 LANDARAM — a janela vem de quem escreve; pen-up **37,0 → 24,0 ms** a 4096²

O desenho que sobreviveu **não é o guard da §5.18**: o `Drop` estende o empréstimo até o fim do escopo e
produziu **14 `E0499`** — o borrow checker recusando a forma. O que ficou é mais simples e faz a mesma
promessa:

> **`fork_par` ABRE um acesso não-declarado na janela; `PainterTool::declare_wrote(rect)` o FECHA.**
> Enquanto houver acesso aberto, o commit **varre** — que é exatamente o que ele faz hoje.

Sem guard, sem `Drop`, sem lifetimes acoplados, **sem tocar um único `let buf = …`**. O mecanismo inteiro
é a contagem: o modo de falha de um sítio novo — ou de um que esquece — é **lento, nunca errado**, e o
contador zera em cada commit, então um sítio esquecido degrada aquele passo e nada além dele.

**A rede que torna isso conferível:** em build de **DEBUG** o `split` deriva a janela verdadeira mesmo
quando recebe a declarada e **afirma que ela cabe**. A suíte roda em debug em ~4 s e exercita fill,
seleção, warp, sculpt, máscara, inpaint, clone, aquarela, Wet Paint e os shape editors ⇒ o invariante é
**conferido a cada rodada** em vez de assumido. Foi ela que reprovou o atalho da §5.17 na primeira rodada,
e é ela que aceita as oito declarações de hoje — o que é a prova de que nenhuma sub-declara.

**Os oito sítios quentes:** os cinco de depósito de pigmento (`stamp_cache`, `stamp_color_cache`,
`stamp_color_dynamic`, `blur_route`, `clone`) declaram o `touched` que já acumulavam, e os três do fold do
relevo (`impasto_live`) declaram o `rect` do commit.

⚠️ **No fold a declaração é o `rect`, NÃO o `moved`** que o `mark_dirty` usa: o laço escreve `target[i]`
em toda a janela do kernel e não só onde o relevo passou do `RELIEF_EPS`. *Onde a imagem mudou* e *onde
bytes foram escritos* são perguntas diferentes — e é exatamente essa diferença que a §5.17 mediu.

| impasto @4096² | antes | agora |
|---|---|---|
| commit de undo | 23,72 | **12,16 ms** |
| **pen-up TOTAL** | 37,00 | **24,03 ms** |

⚠️ **O commit ainda custa 12,16 e não ~1, e o número fica ABERTO, não estimado.** Sobram a **extração**
dos dois lados da janela (que num traço longo não é pequena) e os planos que nenhum sítio declarou. A
próxima medição decide se vale atacá-la ou se o **S3** a subsume — ele remove os 11,9 ms do fold *e* a
extração de uma vez.

⚠️ **E duas vezes nesta sessão a cwd do Bash escorregou para a árvore PRIMÁRIA**, uma delas fazendo cinco
das oito declarações irem parar na `main` — o commit saiu com 1 arquivo em vez de 6 e a medição disse
"sem ganho", que eu quase reportei como achado. Nada foi commitado lá e a árvore foi restaurada. *No Modo
L, todo comando começa com o `cd` da worktree; a regra existe porque a cwd volta sozinha.*

### 5.20 🎯 S3 — a premissa foi MEDIDA, e ela re-precifica a wave em três lugares

O plano dizia: *"o journal guarda os PIXELS e o Ctrl+Z aplica o patch ao plano VIVO ⇒ o cursor larga os
planos, a contagem de donos cai para **um**, e o fold e o fork do pen-down somem juntos"*. Antes de
construir em cima disso, as três frases foram perguntadas ao produto. **As três voltaram diferentes.**

#### (1) *"O estado vivo não serve de base"* — verdade em **2 de 81**, e as duas exceções têm NOME

O `undo_delta` proíbe usar o vivo como base do delta (*"`restore_shape_overlay` RE-CARIMBA a figura"*).
Uma afirmação sobre o produto não se cita: mede-se. A rede é
`PlaneDeltas::divergences` — o **terceiro consumidor** da lista dos dezenove planos, ao lado do `split` e
do `side` — chamada no instante de **todo** undo/redo, com a suíte inteira em debug (~4 s; ela exercita
fill, seleção, warp, sculpt, máscara, inpaint, clone, aquarela, Wet Paint e os shape editors).

```text
  81 chamadas de undo/redo   ·   79 concordam   ·   2 divergem, as duas no canvas_rgba
```

E nos 79 o vivo não é meramente *igual*: é o **MESMO buffer** (`Arc::ptr_eq`), porque o commit toma o
cursor como `after.clone()`. Isso torna a comparação barata **e imune à armadilha do ADR-0124**
(endereço igual com conteúdo diferente): escrever no lugar exige dono único, e a referência forte do
cursor o impede.

As duas exceções, com o teste que as atribuiu:

| # | onde | o quê |
|---|---|---|
| 1 | `apply_and_keep_…` (no **redo**) | o **re-stamp do shape** — a frase do doc, real: o `restore_shape_overlay` re-carimba o editor sobre a base pristina |
| 2 | `undoing_a_wet_stroke_…` (no **undo**) | o **escorrido do Wet Paint** — a sim composita depois do pen-up sem gravar entrada |

⚠️ **Nenhuma das duas é bug hoje**, e o porquê é o que o S3 revogaria: a materialização constrói um
snapshot **completo** e o `restore_model` o instala **por atacado** (`self.canvas_rgba = m.canvas_rgba`),
então o resíduo é substituído em vez de sobreviver. Sob o S3 — que escreveria a janela *dentro* do plano
vivo — o que está **fora** da janela fica, e as duas viram corrupção silenciosa.

Pinado em `paint::undo_live_base_tests` (3 gates, 3 mutações, 3 sangram: o cursor com cópia própria mata
o de identidade · a rede cega mata o do escorrido · **o restore ficando com o plano vivo — o S3 no
extremo — mata o da instalação por atacado**).

⚠️ **Lição de método:** a 1ª rodada do censo imprimiu **nada**, e *nada parecia concordância* — o
`cargo test` **captura o stderr de teste que passa**. Foi o controle positivo (imprimir também quando
NÃO diverge) que separou *"zero divergências"* de *"a sonda nunca rodou"*
([[feedback_a_negative_search_needs_a_positive_control]]). Re-censar:
`PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter -- --nocapture 2>&1 | grep S3-AUDIT`.

#### (2) *"A contagem de donos cai para UM"* — cai, **mas só se o journal alcançar as TRÊS referências**

O §7 já registrava que um journal substituindo só o `stroke_undo` deixaria a contagem em 2. Medido pela
sonda que já existia (`who_holds_the_planes_when_a_stroke_begins`), agora com a **atribuição** por
ablação — a pergunta que a 1ª rodada não fez, e sem a qual o número sozinho engana:

```text
  REGIME (2 traços commitados, nenhum gesto aberto)   canvas 2 · heights 2 · covers 2 · mats 2
  depois de LIMPAR o histórico                        canvas 1 · heights 1 · covers 1 · mats 1

  DENTRO do gesto — após 1 traço                      canvas 1 · heights 4 · covers 4 · mats 4
  DENTRO do gesto — após 2 traços                     canvas 1 · heights 3 · covers 3 · mats 3
  DENTRO do gesto — após 4 traços                     canvas 1 · heights 3 · covers 3 · mats 3

  … sem o snapshot de pen-down                        canvas 1 · heights 3 · covers 3 · mats 3
  … e sem o histórico INTEIRO                         canvas 1 · heights 1 · covers 1 · mats 1
```

**São três donos em regime, e cada um tem nome:** o **tool** (irredutível) · o **`cursor`** da U1 · o
**`paint.stroke_undo`** (o `ModelSnapshot` do pen-down, cujo `snapshot_model` clona os `BTreeMap` ⇒ um
`Arc` por plano). O **quarto** do primeiro traço é a ENTRADA dele: o traço que **cria** os planos de
relevo não tem lado `before` a diferenciar, então o `split` grava `Whole { before, after }` — e `after`
é um `Arc` do plano vivo, para sempre. Do segundo em diante a entrada é `Patch` e não segura plano
nenhum.

⚠️ **Duas consequências, e a segunda corrige a leitura anterior desta seção:**

1. **O S3 é tudo-ou-nada** — `make_mut` copia com qualquer coisa acima de um, então remover UMA das três
   não compra milissegundo nenhum. Não existe versão parcial que ganhe o fork.
2. **Mas ele CHEGA a um.** Ablacionar o histórico inteiro (`undo.clear()`, que leva cursor **e** as
   entradas `Whole`) mais o `stroke_undo` deixa exatamente o tool. A leitura de que *"a contagem para em
   três"* saiu de ablacionar só o cursor — o alvo do S3 são as três referências, e o journal de pixels
   substitui as três: `stroke_undo` some com a captura-na-escrita, `cursor` e `Whole` somem com o journal.

(O `canvas_rgba` marca 1 dentro do gesto porque o fork **já aconteceu** — é o custo que se quer remover,
não a ausência dele.)

#### (3) *"Capturar por região"* precisa da região **antes** da escrita — e 47 sítios não a têm

O S1 deu à porta de escrita um **contador** de acessos não-declarados, não a região por tipo, e a §5.18
diz por quê (o guard com `Drop` morreu no borrow checker; passar o retângulo tocaria todo sítio).
Contado hoje: **47 chamadas de `fork_par` em 25 arquivos**, contra **12** que declaram região — e a
declaração é **depois** da escrita, por construção.

Para capturar o "antes" por região é preciso tê-la **antes**. Os dois caminhos quentes podem dá-la:

- **o fold** (`impasto_live.rs`) já computa o `rect` **acima** do fork — a região está na mão. ✅
- **o depósito** (`stamp_cache.rs`) acumula `touched` **durante** o laço de blit, mas a lista de dabs
  está toda na mão antes: o bbox dela é um **superconjunto** de `touched`, e superconjunto é
  exatamente o que o S1 já declarou seguro. ✅

Os outros ~39 passariam "não sei" ⇒ varredura/fork completo ⇒ **correto, só lento** — a mesma política
de falha do contador. Isso torna a migração incremental, **não** trivial.

#### O veredito

O S3 continua sendo a maior frente aberta (fold 11,9 + fork do pen-down ~11,7 + os 12,2 do commit + o
Ctrl+Z), **mas ele não é a wave que o plano descrevia**: é *região-na-porta de escrita* + *journal por
tile* + *fechar as duas exceções da tabela acima*, com a instalação por atacado deixando de ser a rede de
segurança que hoje é. ⚠️ **A meia-versão barata — "patchar o plano vivo mantendo tudo o mais" — foi
avaliada e NÃO construída**: ela compra só o Ctrl+Z (23,4 ms, ação user-paced e deliberada) ao preço de
mover os planos para fora do tool nos caminhos de falha, onde um `undo()` que devolve `None` deixaria o
documento **sem pixels**. Trocar risco de perder o documento por 1,4 quadro numa ação que o artista faz
de propósito é o trade errado, e fica registrado como recusa medida, não como esquecimento.

### 5.8 ✅ E a fronteira NOVA (§4.8.3) é dos QUATRO modos, não do impasto

O `INPUT (fora do frame)` mede o tempo dentro de `on_canvas_pointer` — que é a porta por onde **todo**
modo carimba dabs. Os 5,3–8,8 ms/frame medidos são de uma sessão de **Impasto**, mas o instrumento é
agnóstico ao meio: rodar o mesmo `PH2D_PAINT_PERF` em Digital, Watercolor e Wet Paint dá a mesma coluna
para os quatro, e é a comparação entre elas que diz quanto de cada número é *o carimbo* e quanto é *a
simulação daquele meio*.

⚠️ **Isto é uma medição de 4 linhas que ainda não foi feita**, e ela vale mais que qualquer palpite sobre
onde otimizar em seguida — inclusive porque o Wet Paint já tem o próprio custo medido pelo outro lado
(0,83–0,89 ms/tick na sessão representativa, doc 24), então para ele as duas colunas podem ser
reconciliadas em vez de estimadas.

---

### 5.21 🔬 O commit hinted custa 11,8 ms — e quase metade é o `free` da geração anterior

O S2 fechou os 12,16 ms restantes como *"extração dos dois lados da janela **+ planos que ninguém
declarou**"* — número declarado ABERTO. A segunda metade dessa frase é **falsa**, e a rede de auditoria
a derruba numa linha. Com `PH2D_UNDO_AUDIT=1`, todo commit de traço imprime:

```text
  [S3-AUDIT] commit: JANELA 338x156 em (0,123) · nao-declarados=0 · snapshot_writes=0
```

**A janela É oferecida** (`nao-declarados=0`) e o `split` retorna antes de varrer. Logo os 12 ms não são
varredura — e 338×156 texels sobre 1024² também não são 12 ms de extração. O custo é outra coisa, e ela
é **proporcional à tela**.

#### A ablação, pela porta

`WriteWindow::open_write` deixa um acesso aberto, e um acesso aberto faz `hint_for` devolver `None` —
**é exatamente o que um sítio esquecido faz**, então o braço "sem janela" é o produto pré-S2, não um
harness. O terceiro braço PINA a geração anterior dos planos de relevo (um clone tomado logo após o
pen-down, antes de o fold forkar), o que impede o commit de ser o último dono deles:

```text
  commit de undo, impasto (ms)             1024       2048       4096
  com a JANELA declarada                   4,29       7,02      11,51
  VARRENDO (um sítio esqueceu)             4,90       9,12      23,02
  com a janela, geração velha PINADA       3,93       5,53       6,49
  ─────────────────────────────────────────────────────────────────────
  o que a janela poupa                     0,61       2,09      11,51
  o que o `free` custa                     0,36       1,49       5,03
    (2ª testemunha: `drop` dos 3 planos)   0,00       0,00       2,44
```

⚠️ **As duas testemunhas concordam que o `free` existe e é o maior termo proporcional à tela, e
discordam em 2× na magnitude** (2,44 direto contra 5,03 por ablação a 4096²) — a faixa fica escrita
como faixa. A pinagem também impede o alocador de REUSAR aquelas páginas, então ela mede o `free` *e*
o que a não-reutilização custa aos vizinhos; o `drop` direto mede só o `free`, mas fora do padrão de
alocação do produto. Nenhuma das duas é a resposta sozinha, e escolher uma seria inventar precisão.

#### Por que isto RE-PRECIFICA o S3 para cima

O `free` não é um custo independente: **é a outra ponta do fork**. Todo traço aloca uma geração nova
(o `fork_par` do fold) e solta a anterior no commit — são a mesma decisão de projeto, cobrada duas
vezes. ⚠️ Então a wave que remove o fork **remove o `free` junto**, e o payoff do S3 é maior do que a
§7 dizia:

| o que o S3 mata | a 4096², impasto |
|---|---|
| o fold do relevo | 11,9 ms |
| o fork do pen-down | 11,7 ms |
| o `free` da geração anterior | 2,4–5,0 ms |
| o resto do commit (extração + contabilidade) | 6,5 ms |

⚠️ **E o que sobra depois de pinar — 6,49 ms a 4096² contra 3,93 a 1024² — segue número ABERTO.** Ele
cresce com a tela devagar demais para ser um passe de plano inteiro e depressa demais para ser só a
janela; atribuí-lo é a próxima medição, não a próxima hipótese.

### 5.22 🔨 S3, degrau 1 — o journal por tile existe, a porta do canvas existe, e o censo diz 12 de 894

O primeiro degrau da wave, construído pelo método que o S1 estabeleceu: **a rede antes da mudança**. O
journal é escrito e verificado contra a verdade de hoje, **sem ser autoritativo** — em release ele é um
no-op, porque capturar *e* forkar seria pagar as duas coisas.

**O que landou:** `undo_journal.rs` (`TileJournal`, grade de 128 elementos, *a primeira captura de cada
tile é a que vale*) · a porta **`fork_canvas`** com os **28 sítios** de canvas migrados · e o censo
`PH2D_UNDO_AUDIT=1`, que compara o journal com o estado do último commit.

#### As três coisas que a rede ensinou, todas contra premissas minhas

**(A) O journal não se alinha ao `before` do passo — alinha-se ao ÚLTIMO COMMIT.** Mirada no `before`,
a rede disparou em **16 testes** na 1ª rodada, com o journal na tela virgem (255) contra um `before` já
pintado (200). Não era defeito: escritas entre dois commits (um preview re-stampado, um tick de água)
armam o journal antes de o passo abrir. É a mesma reconciliação que o `absorb_foreign_writes` faz hoje
por diff — e que o journal passa a fazer **por construção**, porque capturou aquelas escritas também.

**(B) O passo não tinha fronteira única: 21 sítios, em 11 arquivos, movem o cursor.** Enquanto o estado
de escrita foi um campo do TOOL, zerar o journal era responsabilidade de cada um deles — e esquecer não
falha, só faz o journal descrever um passado velho demais, em silêncio. ⚠️ **A cura é estrutural e é
uma correção de posse:** *"o que mudou desde o último commit"* é conceito da HISTÓRIA, então
`WriteState` mudou-se para dentro do `UndoController`, e **`set_cursor` virou a porta única** — o cursor
andar e o journal zerar são o MESMO fato, e agora estão na mesma linha.

**(C) O censo: 12 divergências em 894 commits auditados — 98,7% já limpo.** Sobram escritores de canvas
que não passam pela porta. ⚠️ **Enquanto essa lista não for zero o journal não pode virar a FONTE do
undo**, e por isso ele entrou como **censo opt-in, não como gate**: um gate que falha não pode entrar
verde nem vermelho. O número fica à vista e a lista é finita.

#### E uma lição de gate que se pagou na hora

O arch-gate da porta casava a chamada INTEIRA (`fork_canvas(&mut self.canvas_rgba, …)`) e **morreu no
`cargo fmt`**, que quebra três argumentos em quatro linhas: ele passou a achar **zero** sítios. Quem o
salvou de virar verde-sobre-nada foi o **controle positivo**. *Gate ancorado em LAYOUT é proxy que
expira* — o literal virou o nome da porta, e a mutação (um sítio de volta ao `make_mut` cru) sangra os
dois gates com o arquivo e a linha do ofensor.

**Gates:** 8 no `TileJournal` (**3 mutações, 3 sangram** — ⚠️ a do reshape **sobreviveu à 1ª rodada por
FIXTURE**: os dois planos eram preenchidos por `i % 251`, então o byte no índice sondado era o MESMO nos
dois e *"devolveu o velho"* era indistinguível de *"devolveu o novo"*) + os 2 arch-gates da porta.
**Nenhum schema, nenhum contrato congelado, nenhum id/token** (`PROJECT_SCHEMA` 29); release é
byte-idêntico (a captura é `cfg(any(test, debug_assertions))`).

### 5.23 🔩 S3, degrau 2 — o censo foi a ZERO, e os 12 eram TRÊS mecanismos

⚠️ **Nenhum dos 12 era um escritor que esqueceu a porta**, que era a hipótese que o degrau 1 registrou.
Todos os três casos passam (ou passariam) pela porta; o que estava errado era **de que plano** os bytes
capturados eram, e **quando** o journal deixa de descrever a tela.

**(A) A TROCA de plano — 11 dos 12.** `stamp_dabs_mask` troca o scratch da máscara para dentro do campo
`canvas_rgba`, e `stamp_dabs_gated` faz o mesmo com o plano `free` da proteção: os dois pintam por todo
o pipeline de stamp **sem tocar a tela**. Enquanto a troca está de pé, um `fork_canvas` captura bytes do
plano ERRADO — e como *a primeira captura de cada tile é a que vale*, a poluição é **permanente**: a
projeção que escreve a tela logo depois encontra o tile já tomado, não recaptura, e o journal jura que a
tela começou o passo com os bytes do scratch (medido: **196.608 bytes**, o branco `255` do scratch onde
a tela é `(200,30,30)`).

⚠️ **A cura é um CONTADOR de profundidade, não um guard com `Drop`** — os dois sítios seguram `&mut self`
por dentro do trecho trocado, e um `Drop` estenderia o empréstimo até o fim do escopo (os 14 `E0499` que
o S1 já mediu). Porta única `swap_canvas_plane(canvas, other, write_state)`: **três empréstimos disjuntos
de campo numa chamada só**, então uma chamada é uma troca e a paridade das chamadas **é** a paridade das
trocas.

**(B) A SUBSTITUIÇÃO de plano — 13 sítios.** `Fill`, crop, resize, o Reset do warp, todo bind: o `Arc` é
trocado por outro, e **um fork não tem o que capturar** porque não há escrita incremental — o plano
simplesmente deixa de existir. Porta única `PainterTool::replace_canvas(new)`, que captura o plano velho
**inteiro** antes de o soltar. ⚠️ Custa exatamente o que o fork custa hoje ⇒ **nunca é regressão**, e uma
substituição que muda a FORMA é recusada pelo journal (o stride não mede o plano velho) — o que é
correto, porque forma diferente já força `Whole` no motor de delta.

**(C) A REINSTALAÇÃO de modelo — os 3 últimos.** `restore_model` troca TODO plano de uma vez, e depois de
(B) ela passa pela porta ⇒ ela **CAPTURA o estado de antes do undo**, que é exatamente o que o undo está
desfazendo. Uma linha no fim dela: o journal esquece o passo. *O que ele guardava descreve planos que já
não existem.*

#### A rede não pode viver dentro do relógio da coisa que observa

Com o censo em zero eu tirei o `PH2D_UNDO_AUDIT` para ele virar gate permanente — e **dois gates de
razão caíram na hora** (`the_fold_costs_what_the_window_costs_not_what_the_canvas_costs` e o irmão do
gate de proteção). A rede varre o canvas INTEIRO por commit, então ela é **canvas-proporcional** e entra
no numerador exato que aqueles gates medem; a suíte também foi de 4,2 s para 11,4 s. ⚠️ É a versão
espelhada da lição do §4.8.2 (*"um gate cujo oráculo se dissolve quando a coisa que ele vigia melhora"*):
aqui é o **instrumento** que corrompe o oráculo alheio. A varredura fica **opt-in**; a PROPRIEDADE fica
em quatro gates próprios, sem relógio nenhum.

⚠️ **E a rede ganhou uma pré-condição que faltava:** ela só se aplica a um passo que COMEÇA no cursor. Se
o `before` não é o cursor, alguém escreveu no meio — e o histórico responde a isso **re-partindo a
entrada do topo e movendo o cursor** (`absorb_foreign_writes`). Perguntar antes dessa reconciliação seria
afirmar mais do que o produto promete. O discriminante é o MESMO fato que a absorção usa, por identidade
de `Arc`: no caso comum nada escreveu desde o commit, o `before` clona o ponteiro do cursor, e a
igualdade é exata e barata.

#### Duas lições de gate, as duas minhas

⚠️ **O gate da máscara nasceu VAZIO e a mutação passou por cima dele.** Ele media o journal **depois** do
traço — e o pen-up commita, e um commit **zera o journal** (`set_cursor`) ⇒ *"nada divergente"* era
verdadeiro por construção, com o journal sempre vazio. A sonda que o pegou imprimiu `capturado=0`. O
traço agora fica **ABERTO**, e o mesmo defeito reinstalado sangra em 196.608 bytes. *Zero não falha a
menos que você o faça falhar* — a mesma forma que o `live_stroke_envelope` já custou a esta linha.

⚠️ **A fixture precisou de um controle explícito:** o scratch (branco) e a tela (vermelha) TÊM de diferir,
senão capturar do plano errado é indistinguível de capturar do certo. Está afirmado dentro do gate.

**Gates:** 4 em `journal_tests.rs` (a máscara não ensina o scratch · a substituição guarda o plano velho ·
a reinstalação esquece · a troca é PAREADA), **4 mutações, 4 sangram** — e a da troca desbalanceada sangra
**duas**, porque deixar a troca aberta faz toda escrita seguinte capturar do plano errado. **Censo: 12 →
0** em 894 commits. **Nenhum schema, nenhum contrato congelado, nenhum id/token** (`PROJECT_SCHEMA` 29);
release segue byte-idêntico.

⚠️ **O que este degrau NÃO é:** ele não compra um milissegundo. O journal é escrito e conferido, **não é a
fonte do undo** — os planos seguem com os mesmos três donos, e o fork, o fold e o `free` seguem sendo
pagos. O que ele compra é a **licença** para o degrau 3.

### 5.24 🔑 S3, degrau 3a — a CHAVE: a cadeia é estabelecida no undo também, e as 2 divergências vão a zero

O §5.20 mediu a premissa do S3 e achou o obstáculo: o plano vivo **é** o do cursor em **79 de 81**
undos, e em **dois** não é — o re-stamp do shape e o **escorrido do Wet Paint**. Enquanto isso for
verdade o cursor não pode largar os planos, porque escrever a janela DENTRO do plano vivo **preservaria
o que está fora dela**, e fora dela está a divergência.

⚠️ **A cura não é código novo — é a porta que já existia, chamada no segundo consumidor.** O invariante
da cadeia (*`entry[topo].after` == o estado que este passo encontra*) tem **dois** consumidores: o
commit, que o estabelece desde 2026-07-26 (`absorb_foreign_writes`), e o **undo**, que o **assumia**.
Absorver também na entrada do `undo_last`/`redo_last` leva o censo a **92 chamadas, 0 divergências**
(era 2), e os três gates pinados de `undo_live_base_tests` seguem verdes.

⚠️ **Custo zero no caso comum, por construção:** a pergunta usa o MESMO `PlaneDeltas::split`, que começa
por `Arc::ptr_eq` em cada plano ⇒ sem escrita estrangeira ele não lê um byte. O snapshot que ela recebe
é feito de clones de `Arc`.

**E há uma consequência de PRODUTO, nomeada em vez de contrabandeada:** a gota que a sim desenha depois
do pen-up passa a **pertencer ao traço que a causou**, então desfazer a remove *e refazer a devolve*.
Antes o redo trazia o traço **sem** o escorrido — o artista perdia tinta que o produto tinha desenhado.

#### A lição de gate: *a cura existe* e *a cura está ligada* são dois gates

O gate da propriedade chama a porta **direto**, então tirar a chamada do `undo_last` o deixa **VERDE** —
ele prova que a função funciona, não que o produto a usa. Quem sangra é o irmão de COMPORTAMENTO, que
dirige `undo_last`/`redo_last` de verdade. O doc do primeiro afirmava a mutação errada e foi corrigido.

**Gates:** 2 novos em `undo_live_base_tests` (a cura · a fiação), **1 mutação, sangra** no de fiação.
**Nenhum schema, nenhum contrato congelado, nenhum id/token** (`PROJECT_SCHEMA` 29).

⚠️ **FLAKE PRÉ-EXISTENTE, medida e NÃO causada por esta wave — ✅ FECHADA na §5.27:**
`the_fold_costs_what_the_window_costs_not_what_the_canvas_costs` falhava ~1 em 3 rodadas da suíte
**completa** desta crate, no commit anterior a esta mudança (medido nos dois lados). A causa era de
FIXTURE (as duas telas cronometradas em fases separadas, cada mínimo de um regime de carga diferente),
não da barra; ver §5.27 para o conserto e as 12 rodadas limpas.

**O que falta no degrau 3:** o cursor e o `stroke_undo` largarem os planos de fato (a contagem de donos
3 → 1), com a materialização aplicando o patch ao plano VIVO. É essa metade que mata o fold (11,9 ms), o
fork do pen-down (11,7) e o `free` (2,4-5,0). ⚠️ Continua **tudo-ou-nada**: remover um dono de três não
compra milissegundo nenhum (§5.14).

---

### 5.25 📐 A troca do S3, MEDIDA — a região é 15-73× mais barata que o fork, e o *fallback* não era

Antes de mexer no ciclo de vida dos snapshots, a pergunta que precede a construção: **o S3 troca uma
cópia do plano inteiro por uma cópia dos tiles escritos — troca por quanto?** Sonda
`measure_journal_cost::what_a_region_journal_costs_against_the_fork_it_replaces`, com a geometria REAL
de um traço (a pegada de cada dab, evento a evento) em vez de um palpite:

```text
                          captura        retém      fork do plano
  4096²  traço curto       0,04 ms       1,6 MB     3,16 ms · 64 MB   ⇒ 73× mais barato
  4096²  traço na tela     0,20 ms       7,3 MB     3,16 ms           ⇒ 15× mais barato
  2048²  traço curto       0,03 ms       1,6 MB     0,32 ms · 16 MB   ⇒  9× mais barato
  2048²  traço na tela     0,09 ms       3,9 MB     0,32 ms           ⇒  4× mais barato
```

A premissa do S3 é **sólida**, e por margem larga. ⚠️ **Mas a terceira linha da tabela derrubou uma
afirmação que era minha**, escrita no cabeçalho do próprio journal: *"quem não sabe onde escreve passa
`None`, e isso custa exatamente o que o fork custa ⇒ nunca é regressão"*. Medido:

```text
  4096²  None (não sei)   12,08 ms      64 MB      3,24 ms   ⇒ 3,73× PIOR que o fork
  2048²  None (não sei)    0,86 ms      16 MB      0,39 ms   ⇒ 2,20× PIOR
```

O mecanismo é banal e por isso mesmo invisível a olho: um plano inteiro em tiles são **1024 alocações de
16 KB montadas em série**, contra **uma** cópia contígua e paralela. E isso não é canto de tabela — a
§5.20 conta **39 dos 47 sítios** que passariam `None`, então o fallback *era* o caso comum da migração.
Uma política de falha que diz *"lento nunca, errado jamais"* não sobrevive a um fallback 3,7× pior que a
coisa que ele substitui.

**Duas correções, cada uma atacando um regime diferente:**

1. **Captura paralela por tile** (acima do limiar de `plane_copy`, o MESMO que a porta de fork consulta):
   os tiles são **disjuntos** e a leitura do buffer é **pura** — a forma que o ADR-0109 sanciona e que
   esta crate já usa em quatro lugares. Fecha o 4096²: **12,08 → 3,14 ms**.
2. **O caminho CONTÍGUO para a grade virgem**: `None` numa grade em que nenhum tile foi tomado copia o
   plano de uma vez (`plane_copy::par_clone`) em vez de o picar. Fecha o 2048², onde a cópia fica
   **abaixo** do limiar do rayon e o paralelo não tem o que salvar: **0,86 → 0,32 ms**.

⚠️ **A segunda tem uma metade load-bearing:** o atalho só vale numa grade **virgem**, porque ele copia o
buffer *de agora* — numa grade que já tomou tiles, o "agora" daqueles tiles já traz bytes que o passo
escreveu, e usá-lo apagaria a primeira captura. É o invariante do módulo (*a primeira captura é a que
vale*) chegando por outra porta, e ele tem gate próprio.

**Resultado: `None` custa 1,00× o fork nas DUAS telas** — a política volta a ser verdadeira onde estava
escrita. **3 gates novos, 3 mutações, 3 sangram** (tirar o guard de grade virgem ⇒ o byte modificado no
lugar do original · trocar `ty`/`tx` no filtro paralelo ⇒ só o gate de cima do limiar morde, e os outros
nove ficam verdes porque **nenhum plano pequeno percorre aquela rota** · `get` ignorar o buffer contíguo
⇒ dois gates).

⚠️ **E uma dívida LATENTE do degrau 1 fechou junto:** o `mod journal` era declarado incondicionalmente
enquanto o único campo que o constrói é `cfg(any(test, debug_assertions))` ⇒ **4 warnings de dead-code
que só aparecem em `--release`** (`cargo clippy -p` roda em debug, e foi por isso que ninguém os viu por
três commits). O módulo passou a carregar o MESMO `cfg` do campo, com a nota de que os dois saem na
mesma edição quando o journal virar a fonte do `before`.

⚠️ **Isto NÃO é ganho de produto e não se vende como tal:** o journal segue sendo rede de verificação e
o commit segue derivando o `before` de dois snapshots. O que a sonda entrega é o **número que autoriza a
wave** — e o número diz que ela vale.

---

### 5.26 ⚓ O journal é ancorado no PASSO, não no último commit — e o censo diz 742 de 878, zero divergências

Autorizada a wave (§5.25), a peça seguinte é de **ciclo de vida**, não de custo: para o journal ser o
lado `before` de um passo, ele tem de descrever *o que aquele passo encontrou*. Ancorado no último
**commit** — como estava — ele descreve outra coisa, e a diferença tem nome desde 2026-07-26.

**O mecanismo, em três estados.** Sejam `S0` o canvas no commit `N-1`, `S1` no pen-down de `N`, `S2` no
commit `N`. Entre `S0` e `S1` pode haver escrita **sem entrada de undo** — a sim do Wet Paint
compositando depois do pen-up, que é literalmente o que um *escorrido* é. O journal ancorado no commit
guarda `S0` em **todo** tile que alguém tocou desde então, e *a primeira captura é a que vale*: nos
tiles que a gota **e** o traço tocam, ele guarda `S0`. Mas o lado `before` do passo `N` é `S1`. Usá-lo
assim daria ao undo uma tela do passo **anterior**, nos texels exatos em que os dois se sobrepõem.

**A porta é `PainterTool::begin_undo_step()`**, chamada onde o `before` é capturado, e a ordem das duas
metades carrega o peso:

1. **a cadeia primeiro** (`absorb_foreign_writes_now`): a gota pertence ao passo anterior — foi ele que
   a causou — e tem de entrar nele **antes** de o journal esquecer que a viu;
2. **e só então o journal passa a descrever este passo** (`WriteState::begin_step`).

⚠️ **A migração é incremental por PROVENIÊNCIA, não por lista de chamadores.** `WriteState` guarda o
contador de escritas em que o journal foi zerado, e
`journal_describes_step_at(before.writes)` só diz `true` quando os dois batem **exatamente**. Um sítio
que ainda não abre passo deixa o journal ancorado num ponto mais velho, a pergunta responde `false`, e o
commit cai no caminho de sempre — ***lento, nunca errado***. É a mesma política do contador de acessos
não-declarados do S1, e é ela que impede que "esqueci um sítio" vire *"o undo devolveu pixels que nunca
existiram"*. Dois sítios abrem passo hoje (o pen-down de traço e o Fill); o resto cai no fallback.

**O censo, com a rede mirando o alvo FORTE** (o `before` do passo, em vez do cursor):

```text
  auditados        878 commits
    PASSO          742   dos quais VAZIOS 106   com bytes 636   (181,3 M bytes conferidos)
    COMMIT         136   dos quais VAZIOS  58   com bytes  78
  DIVERGENCIAS       0
```

**85% dos commits da suíte já têm o journal ancorado no passo, e em 181,3 milhões de bytes ele reproduz
o `before` do canvas byte a byte.** É a prova que faltava para ele virar a FONTE em vez da rede.

⚠️ **A comparação é UMA função para os dois alvos** (`audit_journal_against`) — duas cópias divergiriam
sobre o que *"o journal está certo"* significa, e o alvo forte nasceria com a asserção fraca. E ela
conta **quantos elementos o journal de fato responde**: um journal vazio concorda com qualquer coisa, e
*zero divergências sobre zero bytes* é exatamente o gate vazio que a §5.23 já pagou uma vez — daí os 106
"VAZIOS" aparecerem separados no readout em vez de somarem ao sucesso.

**Gate + mutação:** `a_foreign_write_between_two_steps_does_not_leak_into_the_second_ones_before` — uma
gota escrita pela porta entre dois traços, e o traço seguinte deixado **ABERTO** (o pen-up commitaria e
um commit zera o journal, tornando a asserção vacuosa — a armadilha que o gate da máscara já pagou).
Tirar o `begin_undo_step()` do `paint_begin` o deixa **VERMELHO** com o byte pré-gota no lugar do pós.

⚠️ **Uma ordenação ainda NÃO gateada, e fica dita em vez de contrabandeada:** absorver *antes* de zerar
só é load-bearing quando a absorção passar a LER o journal (hoje ela compara snapshots e não o toca), então
inverter as duas linhas não sangra nada agora. Está no doc-comment da porta, onde a próxima edição a lê.

---

### 5.27 🧱 O RELEVO ganhou a mesma porta — e o fold, que é 9,25 dos ms, já é descrito pelo journal

Os números decidiram a ordem. O fork do **canvas** no pen-down custa **3,16 ms** a 4096²; o fork dos
**três planos de relevo** no fold custa **9,25** (§5.14) — três vezes mais. E fazer a reestruturação
final **uma vez para os quatro planos** é mais barato e mais seguro que fazê-la duas.

**Três portas nomeadas** (`fork_heights` / `fork_covers` / `fork_mats`) ao lado do `fork_canvas`, e não
uma genérica: o journal é **tipado** (`f32` / `u8` / `[u8; 7]`) e o relevo é um **mapa por camada**, então
quem escreve tem de dizer **qual plano e de que camada** — e é essa exigência que separa as duas famílias.
⚠️ Dois planos de camadas diferentes têm a **mesma forma**, então o descarte-por-dimensão do
`TileJournal` não os separa: um passo que tocasse duas camadas misturaria os bytes das duas no mesmo
índice, com confiança e em silêncio. A camada é lembrada, e um segundo dono declara o journal
**MISTURADO** em vez de tentar reconciliar.

**A política de falha é a do S1, e agora vale para três causas:** a porta **genérica** (que não sabe o
plano), um plano que **ainda não existe** (a 1ª pincelada da camada — não há "antes" que o journal possa
descrever) e a **mistura**. Qualquer uma marca o passo como não-descrito, `relief_describes_step_at`
responde `false`, e o commit deriva como sempre — *lento, nunca errado*.

**O censo, com a razão separada** — e a separação é o que o torna útil, porque *"não havia o que
descrever"* e *"havia e não sei"* são coisas diferentes e só a segunda é dívida:

```text
  442  SEM-RELEVO    o passo não tocou relevo (correto, nada a fazer)
  259  INCOMPLETO    tocou pela porta genérica — a dívida: os 11 sítios de sculpt/warp
   42  DESCREVE      o FOLD, com 4,6 M elementos conferidos e ZERO divergências
```

**Gates:** unidade (a porta nomeada guarda o valor **velho**, recusa responder por outra camada, e a
genérica se declara incompleta) + **arch-gate** sobre o `impasto_live.rs` (o fold escreve pelas três
portas nomeadas) — arquitetural porque o defeito é **invisível ao comportamento**: as duas rotas dão os
mesmos bytes e a diferença é só se o journal aprende. **2 mutações, 2 sangram.**

⚠️ **E uma armadilha de build que só aparece em `--release`:** as capturas são `cfg(test/debug)` e as
portas as chamam incondicionalmente, então elas precisam de **irmãs no-op**, como a do canvas já tinha.
Sem isso o `cargo clippy -p` (debug) fica verde e o release não compila — a mesma família do miss do
`file_loc_caps`. **O fechamento roda os dois perfis.**

#### ✅ E a flake do gate de razão do fold FECHOU — a causa era de FIXTURE

A §5.24 registrou `the_fold_costs_what_the_window_costs_not_what_the_canvas_costs` falhando **~1 em 3**
rodadas da suíte completa e sempre verde isolada. A causa não era a barra: as duas telas eram
cronometradas em **fases separadas** (todo o 1024², depois todo o 2048²), então cada mínimo vinha de um
**regime de carga diferente** — sob o runner paralelo, outros testes começam e terminam entre as fases, e
a razão passava a medir o **escalonador**.

**Intercalar as duas** (uma amostra de cada, alternadas) levou a flake de ~1/3 para ~1/10 — e não a zero,
porque uma razão de wall-clock entre duas cargas paralelas **sub-milissegundo** é ruidosa por natureza.
⚠️ **A cura foi REPETIR a medição, não alargar a barra**, e a diferença importa: um fold que anda o
canvas dá ~4× em **toda** tentativa, então nenhuma rodada o salva — o que a repetição compra é uma
janela de máquina calma para o fold *correto* se mostrar. Alargar a barra compraria o oposto (a falha
registrada era **2,95×**, acima de qualquer bar que ainda pegasse a mutação).

**12 rodadas completas seguidas, limpas**, e a mutação documentada (o fold ignorando a janela) sangra a
**3,57×** — o laço de repetição não a torna infalível, que é a única coisa que poderia ter dado errado
neste conserto.

---

### 5.28 🔓 O ÚLTIMO BLOQUEIO CAIU — o cursor é RECONSTRUÍVEL, logo não precisa ser segurado

O `cursor` é um dono **permanente** do canvas (§5.20: dois donos em repouso) e `make_mut` copia com
qualquer coisa acima de um — é ele, junto com o `stroke_undo`, que faz a **primeira escrita de todo
gesto** pagar uma cópia do documento. Ele existe por **dois** motivos, e os dois agora se dissolveram:

1. **ser a BASE do delta** — dissolvido pelo 3a (§5.24): o vivo **é** o cursor em **92 de 92** undos;
2. **ser o ALVO da absorção** — que era o que sobrava, e é o que esta seção fecha.

**A observação que fecha:** o cursor é o estado do **último commit**, e o journal é zerado *naquele
mesmo commit* (`set_cursor`) e guarda os bytes velhos de **toda** escrita desde então. As duas frases
juntas dão uma identidade:

```text
    cursor[i]  ==  journal.get(i).unwrap_or(vivo[i])
```

Ou seja: **o cursor não é um estado que precisa ser guardado, é uma função de dois que já existem** — e
no caso comum (journal vazio) ela é o próprio `Arc` vivo, de graça.

⚠️ **Isso não se afirma, mede-se.** Rede em `begin_undo_step`, com a suíte inteira:

```text
  aberturas de passo com cursor   233
    RECONSTRUIDO                  233   dos quais com escrita estrangeira REAL   2
    bytes vindos do JOURNAL       192.000
    DIVERGENCIAS                    0
```

⚠️ **E os dois casos com escrita estrangeira são o que separa isto de um censo vácuo:** nos outros 231 o
journal está vazio e a identidade degenera em *"o vivo é o cursor"* (a propriedade do 3a, já provada);
são os **2** que exercitam a metade nova — 192.000 bytes reconstruídos do journal, zero divergentes.

**Gate + mutação:** `the_cursor_is_reconstructible_from_the_live_plane_and_the_journal`, com a gota
escrita pela porta (sem entrada de undo) e um **controle** que exige que o vivo e o cursor de fato
difiram — senão o gate afirmaria `x == x`. Tirar o `capture_canvas` do `fork_canvas` o deixa **VERMELHO**
(e derruba o irmão do §5.26 junto).

#### ⛔ E a MESMA pergunta feita no COMMIT não é respondível ali — a tentativa, e o porquê

A identidade tem dois regimes de âncora (o journal ancorado no *commit* e ancorado no *passo*), e o
segundo pedia medição própria. Feita: **4083 bytes divergentes**, numa única família de gesto (o
re-stamp do Deform), 588 de 589 chamadas concordando.

⚠️ **E a divergência não refuta a identidade — ela é a razão de existir da absorção, dita de novo.** No
`commit_structural_edit` o `absorb_foreign_writes` **ainda não rodou** (ele mora dentro do `record_*`,
uma linha abaixo), então o cursor ali é o de **antes** da reconciliação: perguntar se ele é reconstruível
é perguntar sobre um estado que a linha seguinte vai mover. O rastro do caso divergente mostra isso
literalmente — ele é o único que **não** imprime `journal/COMMIT`, ou seja o `before` não é o cursor, ou
seja a absorção ia disparar.

⚠️ **É o MESMO erro que a §5.24 já registrou** (*"o censo estava medindo do lado errado do absorb"*), e
ele reincidiu no mesmo dia, no mesmo módulo. A chamada foi **removida** e o motivo ficou escrito no
sítio, porque uma rede que faz a pergunta certa no lugar errado é pior que rede nenhuma: ela produz um
número que parece refutação. O lugar onde a pergunta cabe é `begin_undo_step` — **antes** do passo, que
é o instante em que a absorção de fato consome o cursor —, e é lá que ela está.

**O que isto autoriza, e o que ainda falta.** Autoriza a última troca: o `cursor` e o `stroke_undo`
largam os planos, o `split` toma o lado `before` do journal e a materialização parte do plano VIVO —
e com isso morrem o fork do pen-down (**3,16 ms**), o fold (**9,25**) e o `free` da geração anterior
(**2,4–5,0**). ⚠️ Falta **construir** essa troca, e ela é uma edição de ciclo de vida do `ModelSnapshot`,
não uma otimização local: os quatro planos têm de sair **juntos** (tudo-ou-nada por plano), a absorção
passa a reconstruir o cursor em vez de o ler, e o journal sai do `cfg(debug)` para o release. Os
**pré-requisitos estão todos medidos e gateados**; o que sobra é a troca.

---

### 5.29 🚪 TODO plano de relevo passa por uma porta que sabe NOMEÁ-LO — e o alargamento achou um sítio

A troca do S3 é **tudo-ou-nada por plano**: se o cursor larga os planos com o journal incompleto, o undo
devolve pixels errados **em silêncio**. Então o pré-requisito não era paralelo à troca, era anterior a
ela — e o que faltava eram os **dez sítios de sculpt/warp** que ainda escreviam relevo pela porta
genérica (`fork_par`), a que não sabe dizer *de que plano* são os bytes.

A migração é mecânica: cada sítio tem `layer`, `source_size` e a região de escrita em escopo, e em todos
os dez ela **limita** a escrita. ⚠️ **Declarar um superconjunto é seguro; um subconjunto é o bug
silencioso** — capturar tiles a mais guarda bytes que nunca mudaram (`journal.get(i) == live[i]` ali),
capturar de menos perde o *antes* de texels que o undo depois não restaura.

**⚠️ E o alargamento do gate achou um sítio que nenhum grep de porta acharia, porque ele não passava por
porta nenhuma:** o `impasto_material.rs` escrevia o plano `mats` com um `Arc::make_mut` **cru**. As três
metades pesam, e a terceira é a grave:

| metade | consequência |
|---|---|
| journal cego | o passo fica `INCOMPLETO` — o byte velho do material não é guardado |
| fork **serial** | a cópia de plano do §5.15, no caminho de um knob que o artista arrasta |
| **acesso não aberto** | o commit acredita que a **janela declarada cobre tudo** |

A terceira contradiz a promessa que o S1 fez: *"o modo de falha de um sítio novo — ou de um que esquece —
é **lento, nunca errado**"*. Ela vale para quem passa por **uma porta**, porque é a porta que incrementa
o contador de acessos não-declarados. Quem não passa é **invisível ao contador**, e aí a degradação
honesta vira perda silenciosa. O gate estreito (só `impasto_live.rs`) não podia vê-lo: *um gate
por-arquivo protege o arquivo que alguém lembrou de listar.*

**⚠️ `fork_par` ficou sem chamador de produção, e virou `cfg(test)`.** É a terceira vez que esta linha
encontra a mesma forma (`warp_axis` na §5.11, `serial_side` na §5.16): um `pub(super)` órfão não é código
morto silencioso, é uma **segunda resposta** esperando alguém chamá-la. Sob `cfg(test)` ela vira a coisa
certa — o oráculo dos gates de byte-identidade e do `Weak`, e o corpo que a sonda de custo mede.

⚠️ **E a metade do arch-gate que CONTAVA a porta genérica morreu junto, de propósito:** com o `cfg`, um
sítio de produto que a chame **não compila**. O compilador é o guarda mais forte, e uma asserção que não
pode falhar é pior que asserção nenhuma. O que sobrou no gate é o que ainda **pode** falhar: nenhum
`make_mut` cru sobre `self.heights`/`self.covers`/`self.mats`, varrido em `tool/paint/**` inteiro, com
controle positivo nas duas pontas. Mutação (o `make_mut` cru de volta): **RED**, nomeando o arquivo e a
linha.

**O censo, com a rede armada sobre a suíte inteira** (`PH2D_UNDO_AUDIT=1`, 891 testes):

| medida | antes | depois |
|---|---|---|
| passos com relevo `INCOMPLETO` | 260 | **202** |
| passos que **DESCREVEM** o relevo | 42 | **100** |
| divergências (relevo · cursor · canvas) | 0 | **0 · 0 · 0** (100 · 231 · 880 conferências) |

⚠️ **Os 202 que sobram têm agora UMA causa, e ela não é dívida — é uma distinção que falta.** O único
produtor restante de `INCOMPLETO` é o `else` das três portas nomeadas: *o plano não tinha forma de canvas
na hora da escrita* (a primeira pincelada de uma camada, um plano de outro documento). E um plano que
**não existia** no começo do passo **não tem *antes* a descrever** — o motor de delta já chama isso de
`OnlyAfter`. Enquanto as duas coisas dividem um estado, este número parece dívida e não é.

⚠️ **Separá-las é barato de escrever e NÃO é barato de provar, e essa distinção decide a ordem.** O
mecanismo é uma linha (`TileJournal::is_empty` já existe: journal vazio ⇒ o plano não existia; journal
com tiles ⇒ ele existia e a escrita seguinte perdeu a forma, aí sim é incompletude). O problema é o
**oráculo**: a rede de verificação compara o journal contra o **cursor**, e um plano ausente no `before`
**não tem chave no cursor** — então ela não olha para ele. Marcar esses 202 como `DESCREVE` seria uma
afirmação que a rede **não pode contradizer**, que é a forma exata do gate vazio (§5.23: *conte os
conhecidos ao lado dos divergentes*). O degrau, então, é **oráculo primeiro**: a rede tem de afirmar
`journal vazio ⟺ o cursor não tem a chave` antes de o estado ser promovido. Sem isso a promoção é uma
melhora de número, não de conhecimento.

**✅ E foi assim que ela foi feita, na mesma sessão — oráculo primeiro, promoção depois.** O `else` das
três portas deixou de dizer *"não sei"* e passa a dizer **qual** das duas coisas aconteceu
(`ReliefJournals::note_absent`, decidido por `TileJournal::is_empty`): journal vazio ⇒ o plano não
existia, registra-se `absent` e o passo segue descrevendo; journal **com tiles** ⇒ o plano existia e
perdeu a forma no meio, e aí `incomplete` como antes. A rede então **confere a afirmação nova**, antes do
tally e por isso mesmo:

> *o journal declarou o plano P AUSENTE no começo do passo, mas o `before` o tem em forma de canvas*

| medida | antes | depois |
|---|---|---|
| passos que **DESCREVEM** o relevo | 100 | **302** |
| passos `INCOMPLETO` | 202 | **0** |
| passos `MISTURADO` | 0 | 0 |
| divergências (relevo · cursor · canvas) | 0 · 0 · 0 | **0 · 0 · 0** |

**O journal agora descreve o relevo de TODO passo que tem relevo**, e é isso que a troca do S3 precisava.

⚠️ **A mutação não sangrou no run comum, e isso é o desenho, não um buraco:** a rede é opt-in
(`PH2D_UNDO_AUDIT=1`) porque *uma rede de verificação não pode viver no relógio do que ela observa*
(§5.23). Armada, ela nomeia o plano e o tamanho. Mas por isso a **propriedade** ganhou gate próprio, sem
relógio e sem env (`a_plane_that_never_existed_is_absent_not_incomplete`) — dois casos num teste, e a
mutação *"marque ausente sempre"* mata o segundo.

⚠️ **Duas coisas seguiram a porta genérica para o `cfg(test)`, e as duas por doc que virou falso:**
`note_untracked_write` (era a metade de declaração dela — *"não é gateado porque a porta genérica o chama
em qualquer perfil"* morreu com a porta) e, no caminho inverso, `TileJournal::is_empty` **saiu** do
`cfg(test)` porque ganhou um consumidor de produto. ⚠️ E o `ReliefPlane` **não pode** ser gateado: ele
atravessa a assinatura de uma porta que roda em qualquer perfil — a tentativa quebrou **só em
`--release`**, e `cargo clippy -p` roda em **debug**. *O gate de fechamento roda os dois perfis.*

⚠️ O comentário que nomeava a causa deste estado (*"alguém escreveu pela porta genérica"*) **virou falso
neste commit** e foi reescrito no mesmo commit: *um comentário que contradiz o código shipado é pior que
comentário nenhum.*

---

### 5.30 ✅✅ O WET PAINT A 4 FPS — três mecanismos, e o maior era a sim varrendo a CAIXA

**Report do Enio (smoke):** *"IMG 4096, 1 pincelada grande e molhada, FPS cai para 4."* E o log dele,
com `PH2D_FLUID_PROFILE=1`, nomeou a fase antes de qualquer teoria:

```
[frame] total=69.99ms (~14 fps) | painter-dispatch(cpu)=2.51 | tool-tick=57.49 | hero-paint=0.54
```

**`tool-tick` é 82% do frame.** O Painter — a metade que as quinze waves anteriores deste doc curaram —
custa 2,5 ms. ⚠️ **A instrumentação que responde isso já existia, atrás de outra flag**, e as minhas
medições do dia mediam o tool com fixture própria (pior caso 11,7 ms) enquanto o produto pagava 57,5.

#### (a) O tick REALIMENTAVA um frame lento — 4 FPS era um laço fechado

`on_tick(frame_ms_now)` recebe o **relógio do frame ANTERIOR**. O acumulador dava `dt / WET_STEP_S`
passos, capado em `WET_MAX_STEPS = 5`. Então: frame lento ⇒ `dt` grande ⇒ mais passos ⇒ frame mais
lento. **Realimentação POSITIVA**, e invisível a qualquer sonda de `dt` fixo — que é o que todas as
minhas eram. Medido a 4096²:

| dt do frame | tick |
|---|---|
| 16,6 ms (60 fps) | 2,08 ms |
| 250 ms | **50,93 ms** |

Ablação do cap: `1 → 2,69 · 2 → 5,79 · 3 → 9,74 · 5 → 50,93`. **Cap = 2**, e o precedente é da física
(`max_substeps` + `warn: dropped Xs of sim time`): **sacrifica-se tempo SIMULADO, nunca o quadro.**

#### (b) O OVER do composite era o que sobrava serial

Row-parallel pelo ADR-0109 (linhas disjuntas, leitura pura, byte-idêntico por construção) —
**16,17 → 11,71 ms**. ⚠️ E o gate que faltava não era de aritmética: o fan-out por linha não quebra a
conta, quebra o **mapeamento linha → offset global** (`gb = (cy0-1+k)*stride`). A mutação `gb = k*stride`
sobreviveu às **895** existentes porque toda fixture irmã pinta com região suja começando na linha 1.
Gate novo pinta longe do topo; a mutação sangra 0 de 81 texels.

#### (c) A SIM PAGAVA PELA CAIXA, NÃO PELA POÇA — e este era o grande

A pergunta que separa as duas explicações: **mesma água, mesmo comprimento de traço, só a FORMA muda.**

| forma | dabs | tick p50 | bbox/tela |
|---|---|---|---|
| horizontal | 60 | 8,15 ms | 2,1% |
| diagonal | 60 | **23,53 ms** | 18,6% |

⚠️ **Isto reinterpreta a tabela anterior** (400→3600 px, 2,37→16,17 ms): ali eu variei o COMPRIMENTO de
um traço *horizontal*, onde a caixa e a água crescem juntas — as duas explicações casavam com os mesmos
números, e só a forma as separa. Eu não tinha feito esse controle.

**A bbox é o CASCO da água**, e um casco mente sobre um traço diagonal: medido a 4096², a caixa é
**27,9% da tela** e as células ATIVAS são **2,4% dela**. 97,6% de cada varredura era desperdício. E uma
pincelada de artista nunca é horizontal.

**A FAIXA VIVA** (`Grid::row_lo`/`row_hi`): um intervalo **por LINHA** no lugar do casco. Todo passe já
fazia early-out por-célula (`active[i] == 0` → `continue`), então pular uma célula fora da faixa é pular
uma que não responderia nada — **byte-idêntico POR CONSTRUÇÃO** nos seis passes com essa forma. O
invariante que sustenta tudo é `active ⊆ faixa`, e ele **também** vale por construção: o rebuild escreve
`active` só dentro da própria janela e publica a faixa como a extensão viva dilatada por `SPAN_PAD = 5`
(folgadíssimo — `maxVelocity` default é 0,2 célula/frame e o rebuild roda a cada 2 frames).

⚠️ **O `advect` é a EXCEÇÃO e está escrito como tal:** o ramo inativo dele **escreve** (zera `vel`), então
o estreitamento se apoia num invariante — *fora da faixa, `vel` já é zero* — e não em construção.

**A rede de debug se pagou na PRIMEIRA execução da suíte:** os invariantes 1 e 2 passaram e o 3 falhou,
`(26, 22): (0.0069, -0.0044)`. O rastro de um drip que se afasta mais de 5 células ficava com velocidade
**fóssil** — e o fingerprint da sessão inclui `vel_x`/`vel_y`, ou seja **divergência real** do motor
original. Cura: `vel != 0` entra na definição de VIVO (a célula sai sozinha na passada seguinte, depois
de o advect zerá-la).

**Duas fugas do caso BASE da indução, as duas achadas pela mesma rede:**

- `empty_bbox` zerava a faixa — mas a água pode **acabar** com velocidade fóssil espalhada pelo rastro,
  e é a faixa que lembra onde. Quem zera a faixa passou a ser **quem zera a velocidade**
  (`clear_canvas`).
- o rebuild varria só as linhas da bbox, e **a bbox de um traço NOVO não tem por que cobrir o rastro de
  um antigo**. Agora varre toda linha de janela não-vazia (O(altura), sai na hora nas vazias).

⚠️ **E o SNAPSHOT carrega a faixa.** A alternativa — abrir a faixa inteira no restore e deixar o rebuild
reapertá-la — foi **MEDIDA e reprovada**: a varredura viva passa a cobrir a folha, **0,3 → 17,4 ms**, um
quadro perdido a cada Ctrl+Z do motor.

**MEDIDO** (4096², decomposição por passe pelas portas públicas de cada uma):

| passe | antes | depois |
|---|---|---|
| `rebuild_active_region` | 31,047 | **1,509** |
| `project` | 12,663 | **1,394** |
| `build_flow_field` | 11,935 | **3,072** |
| `advect` | 10,510 | **2,381** |
| **SOMA (diagonal)** | **48,607** | **11,354 ms (4,3×)** |

A razão diagonal/horizontal por passe caiu de **20-25×** para **~2×**. ⚠️ O caso horizontal ficou
**5,388 → 5,859 (+8,7%)**: a varredura viva é um passe a mais, e casco fino é justamente onde ela não
tem o que economizar. Nomeado, não vendido.

**E no PRODUTO, por ABLAÇÃO** (`Grid::spans_enabled` — o MESMO laço com o intervalo mais largo; não há
segunda implementação a divergir):

| forma | CAIXA | FAIXA | ganho |
|---|---|---|---|
| diagonal 4096² | 31,42 ms | **13,04 ms** | **2,41×** |
| horizontal 4096² | 9,63 ms | 8,65 ms | 1,11× |

**O gate é DIFERENCIAL, não um valor pinado:** seis sessões (horizontal · diagonal · drip sob gravidade ·
dois traços com a sim indo a **idle** no meio · Wet+Blend sobre tinta seca · traço em L) rodam nos DOIS
modos e **todo campo persistente** tem de sair idêntico ao byte — mais o Fast Dry e a rota de undo. E há
um gate de **PROPRIEDADE** (a faixa é fração pequena da bbox num diagonal), porque uma mudança futura que
devolvesse a bbox inteira continuaria **correta** e teria jogado fora o ganho inteiro em silêncio.

⚠️ **E o meu gate de ablação no tool NASCEU MENTINDO "1,02×":** a sessão de água nasce no **pen-DOWN**,
então armar o flag antes dele é um `if let` que não casa. *Busca negativa sem controle positivo*, outra
vez — agora há `expect`.

**Onde ficou o tick, por metade** (`measure_the_two_halves_of_a_wet_tick`, diagonal 4096²): sim
**13,84** · composite **2,79** (18,8% da tela suja). A sim segue sendo a metade grande, e agora o custo
dela é proporcional à **água**, que é a forma correta. Sem realimentação: `dt 16,6 → 1,71 ms`,
`dt 250 → 5,58`.

**Aberto e NOMEADO:** o retângulo sujo que o engine declara ao composite é um **casco pela mesma razão
que a bbox era** — mas ele custa 2,79 ms medidos, então **não é a fronteira**; e o resto do custo da sim
é trabalho honesto sobre 111k células ativas × 7 passes, cuja próxima alavanca é paralelismo (⚠️ o
ADR-0134 declara o solver **serial POR SEMÂNTICA** — não re-derive) ou porte para GPU.

---

### 5.31 ✅ E O CAP DE **PASSOS** VIROU UM ORÇAMENTO DE **TEMPO** — 12-25 → 5 ms/frame

> Enio, depois da §5.30: ***"ainda sem melhoras significativas"***.

A §5.30 cortou o custo **por passo** (48,6 → 11,4 ms nos passes) e não a propriedade que faltava. O
frame continuava pagando **um número de passos**, e `WET_MAX_STEPS = 2` é um cap de **CONTAGEM** —
que só limita o custo se o custo **por unidade** for limitado.

O custo de um passo de água **não é** limitado: ele é linear na área molhada, e a área molhada é o
que a mão do artista escolhe. **É a mesma forma de teto que este repo já descobriu ser um
MULTIPLICADOR duas vezes** — o `MAX_HISTORY = 64` do editor de áudio (ADR-0117) e o
`DEFAULT_MAX_DEPTH = 300` do undo do Painter (plano 26). O teto tem de estar no **RECURSO**.

#### 5.31.1 A medição que fechou a frente

| forma | passo p50 | células ativas |
|---|---|---|
| horizontal | 8,6 ms | 94 523 |
| diagonal | 12,7 ms | 111 283 |

E o número que decidiu tudo (`ph2d-wet-paint/tests/measure_density.rs`) — a **MESMA água** em quatro
telas:

| tela | ns por célula (horizontal) | ns por célula (diagonal) | grid |
|---|---|---|---|
| 512² | 20,1 | 21,4 | 15 MB |
| 1024² | 23,5 | 22,1 | 59 MB |
| 2048² | 21,9 | 23,6 | 236 MB |
| 4096² | 22,7 | 27,9 | **944 MB** |

⚠️ **`ns/célula` é PLANO de 512² a 4096², sobre um grid que cresce 63×.** O custo **não** é layout,
**não** é cache, **não** é TLB — é *trabalho por célula*. E o trabalho por célula não tem para onde
ir: **todo passe é Gauss-Seidel** (o `advect` deplecionA os quatro cantos **in-place**, então uma
célula posterior puxa de um canto que a anterior já esvaziou; o `drying_pass` conta `susp > 10` na
vizinhança 3×3 e **escreve o próprio `susp`**, então o vizinho seguinte lê o valor NOVO) ⇒ **não há
paralelismo byte-idêntico a colher.** O ADR-0134 está certo, e foi conferido passe a passe em vez de
citado.

**Se não dá para otimizar nem paralelizar, o que resta é ORÇAR.**

#### 5.31.2 O orçamento

`WET_STEP_BUDGET_MS = 4` (24% de um quadro de 60 fps) num **token bucket**: cada tick credita, cada
passo **debita o que de fato custou** (um custo *estimado* erraria, e o erro se acumularia no
bucket), crédito negativo é dívida que os frames seguintes pagam.

- **Teto do crédito = UM frame de orçamento.** Um bucket que entesoura devolve exatamente a rajada
  que ele existe para impedir.
- **Fundo da dívida = −100 ms.** É *fundo*, não conforto: enquanto a dívida não bate nele o custo
  amortizado por frame é **exatamente** o orçamento, e é o clamp que quebraria essa igualdade.

| passo | frames por passo | taxa da sim | veredito |
|---|---|---|---|
| 1 ms | 0,25 | 40 Hz (cheia) | **INERTE** (poça pequena) |
| 4 ms | 1 | 40 Hz (cheia) | inerte |
| 12,7 ms | 3 | ~20 Hz | a água escorre em meia velocidade |
| 100 ms | 25 | ~2,4 Hz | a água rasteja, **o app segue a 60 fps** |

⚠️ **O trade é o do `max_substeps` da física, e é deliberado:** sob carga a água simula MENOS tempo
em vez de derrubar o frame — e agora **o custo por frame é independente do tamanho da poça**, que é a
propriedade que o cap de contagem nunca teve.

**Resultado**, janela de 120 frames a 60 Hz sobre uma poça formada a 4096²: custo **médio** do tick
**12-25 → 5,06 ms (horizontal) / 5,26 (diagonal)**.

#### 5.31.3 O residual, com número

⚠️ **Um passo é ATÔMICO** — o orçamento decide se ele *rola*, nunca o interrompe no meio. Então um
tick que **trabalha** ainda custa 16-40 ms (p50 dos ticks com trabalho: **16,7 / 23,5**; máx 74/65).
A **média** é do orçamento; o **hitch** é do passo. Nenhuma política de agendamento esconde uma
unidade atômica de 20 ms dentro de um quadro de 16 — **só concorrência a tira do frame** (§7).

#### 5.31.4 Três hipóteses minhas, as três refutadas por medição

1. **"A sessão longa acumula água e por isso encarece"** — falso: o filme total vai de ~470 a **0 em
   ~40 passos (1 s)**; o passo custa 0,00 ms depois disso. O pico está nos ~30 primeiros passos.
2. **"'Pincelada GRANDE' = 5× as células"** — falso: de raio 50 a 400 px o tick fica em 5,5-8,2 ms e
   a região suja em **2,1-2,2% da tela, CONSTANTE**. O `TRAIL_HALF = 61` do engine clipa a janela do
   traço (item aberto já nomeado no handoff do Wet Paint). Quem move a área molhada é o
   **COMPRIMENTO**.
3. **"O composite virou a fronteira"** — falso: 1,03 / 2,88 ms (ele já é row-parallel desde a §5.30).

⚠️ **E uma armadilha de fixture que a sonda longa expôs:** a sim **não roda com o pincel encostado**
(`sim_should_run() = !stroke_down`), e `drive_stroke` termina com a cauda de release — então
`sim_after: 0` deixa `sim.frame = 0`, o primeiro passo cai num frame **ÍMPAR** e o
`rebuild_active_region` (que só roda em pares) nunca rodou: `active` lê **zero** sobre uma poça
cheia. *O redutor e a fase da amostragem são parte da fixture.*

#### 5.31.5 Os gates, e o defeito que a mutação achou no meu

Dois, mutação-provados:

- **`the_wet_tick_costs_the_frame_a_budget_not_a_puddle`** — o custo por frame é do ORÇAMENTO, nos
  dois regimes de `dt` (60 Hz e travado a 250 ms). ⚠️ O redutor é a **MÉDIA, não a mediana**: sob
  orçamento a maioria dos ticks custa ZERO e alguns custam um passo inteiro — a mediana reportaria
  0,00 e o gate ficaria verde por não medir nada.
- **`the_sim_time_budget_is_inert_on_a_small_puddle`** — 40 Hz cheios numa poça pequena. É ele que
  torna **seguro apertar a constante**: sem ele, baixar o orçamento deixaria toda a suíte verde
  enquanto a água inteira do produto entra em câmera lenta.

⚠️ **A primeira mutação SOBREVIVEU, e o defeito era do gate:** o teto era `WET_STEP_BUDGET_MS × 2,5`
— **derivado da própria constante que ele existe para vigiar** —, então mandá-la ao infinito levava o
TETO junto. É o oráculo-espelho que este repo já pagou três vezes. Teto **literal** (9 ms): a mutação
agora sangra em **15,64 ms/frame**.

---

### 5.32 ✅ E O ORÇAMENTO VIROU ADAPTATIVO — a água sai de 6 Hz para 38, no MESMO frame de 60 fps

> Enio, depois da §5.31: ***"o FPS não caiu abaixo de 60 mas a animação estava tão lenta e travada
> como se o FPS fosse 6"***.

E o log que ele mandou junto **diz a causa inteira**:

```text
[frame] total=16.03ms (~62 fps) | cpu-encode(raw)=4.32ms
        | present/acquire-stall=11.71ms | painter-dispatch(cpu)=0.01ms
        | tool-tick=0.00ms | stamps=0.00ms | hero-paint=0.33ms
```

⚠️ **A CPU passa 11,7 dos 16,0 ms PARADA esperando o vsync.** O orçamento **fixo** de 4 ms da §5.31
não protegia nada num frame com 12 ms de folga ociosa — ele **deixava o hardware parado** e punha a
água em `4 × 60 ÷ 40 ≈ 6` passos por segundo. **O "FPS 6" era a ÁGUA, não o app.**

A §0 já dizia por que o número estava errado: *o teto é o do HARDWARE, nunca o do caminho lento* — e
um orçamento fixo é um **palpite sobre um recurso que se mede a cada frame**.

#### 5.32.1 O controlador

`wetpaint/budget.rs` — AIMD sobre o `dt` que o próprio `on_tick` recebe. ⚠️ O `Tool` é **contrato
congelado** (§6), então não há parâmetro novo a pedir ao shell: o período do frame é o sinal
disponível, e ele basta.

| peça | o quê | por quê |
|---|---|---|
| **período** | EWMA do `dt` nos ticks em que a sim **não trabalhou** | é a régua, e ela é MEDIDA — 60 Hz, 144 Hz ou CPU-bound, o número é o do artista |
| **cresce** | +0,5 ms/frame enquanto o frame cabe em `período + 2` | num app com vsync a folga é o *present stall*, e gastá-la é **de graça** |
| **encolhe** | ×0,5 no instante em que estoura | o frame tem prioridade |
| **teto** | 60% do período | é ele que segura o frame quando a sim não alcança o relógio |

**Medido** (fixture do produto a 4096², `dt` pinado em 16,6 — o regime do vsync):

| | taxa da sim | tick médio |
|---|---|---|
| horizontal | **13,0 → 38,0 Hz** | 5,06 → 9,41 ms |
| diagonal | **11,0 → 36,5 Hz** | 5,26 → 11,38 ms |

Os ms **sobem de propósito**: a água passou a gastar a folga que estava ociosa. O frame não muda,
porque quem encolhe é o *stall*, não o trabalho.

#### 5.32.2 Três camadas, três gates — e as duas mutações que sobreviveram primeiro

| camada | gate | mutação |
|---|---|---|
| crescimento | o orçamento SOBE acima da semente sob `dt` pinado | `grow = 0` ⇒ fica em 4,00 ms |
| recuo | num app **CPU-bound** (overhead 20 ms) o frame assenta no próprio overhead | sem recuo ⇒ **1,75×** o overhead |
| teto | numa poça que a sim **não alcança**, ele estrangula | sem teto ⇒ **30 Hz e 37,74 ms/frame** |

⚠️ **A primeira rodada teve DUAS sobreviventes, e o motivo era estrutural:** as três metades se cobrem
mutuamente, então um gate só por cima não isola nenhuma
([[feedback_layered_defenses_need_per_layer_gates]]). Cada fixture agora **neutraliza as outras
camadas por REGIME**:

- com vsync o **piso de 16,6 ms ABSORVE** tudo que a água gasta ⇒ `dt` nunca passa do alvo ⇒ o recuo
  nunca dispara ⇒ o gate do recuo tem de rodar num app CPU-bound;
- numa poça leve o **`acc` já limita a sim a 40 Hz** ⇒ um teto infinito não muda um milissegundo ⇒ o
  gate do teto precisa de uma poça que a sim não alcance.

#### 5.32.3 E o oráculo do mecanismo deixou de ser um relógio

O gate do crescimento afirma **ESTADO** (`o orçamento subiu acima da semente`), que é determinístico.
Duas versões anteriores usavam wall-clock e **reprovaram sob a suíte carregada**: Hz absolutos
(25,5 contra piso 28) e depois uma razão entre janelas da mesma corrida (1,21× contra piso 1,3).
*Um gate cujo oráculo se dissolve quando a máquina está carregada será silenciado em vez de
acreditado.* O que restou de relógio é **controle** (piso folgado) ou não tem alternativa — e esses
**pulam em debug**, onde um passo de sim é ~16× mais lento e o número mediria o perfil de compilação.

---

### 5.33 ✅ E O CONTROLADOR PUNIA A ÁGUA POR UMA CONTA DE OUTRO — a atribuição

> Enio, terceiro smoke, **mesmo sintoma**: ***"FPS não cai abaixo de 60 mas simulação lenta e travada"***.

O log nomeia a causa inteira:

```text
[frame] total=19.15ms | stamps=13.96ms  | tool-tick=0.00ms
[frame] total=32.90ms | stamps=116.03ms | tool-tick=0.00ms
```

⚠️ **`tool-tick = 0.00` em TODA amostra** — a sim não roda. E o `stamps` ao lado é o carimbo de dabs
dentro do `on_canvas_pointer`: **outro inquilino do frame**, que a água não causa e não controla. O
controlador da §5.32 lia o `dt` **inteiro**, concluía *"não há espaço"* e estrangulava a sim até o
piso de 1 ms (~2 Hz). **Ele punia a água por uma conta que era de outro.**

#### 5.33.1 As duas correções

**(1) Atribuição.** `non_sim = dt − o que a sim gastou`. O recuo só dispara quando **o frame teria
cabido sem nós**; se `non_sim` já estourou sozinho, encolher a água não salva o frame e só congela a
tinta — ali o orçamento **segura**.

**(2) A régua virou o PISO.** O período era EWMA do `dt`, então um frame lento por culpa alheia
**levantava a régua e o teto junto** (`0,6 × 100 ms` = licença para comer 60 ms de quadro). Agora é o
`min` do `dt` observado com creep lento para cima (0,05%/frame): com vsync ele é o intervalo do
monitor, e nenhum inquilino estrangeiro o move.

#### 5.33.2 O eixo que nenhuma sonda de água tinha medido: o `stamps`

`measure_what_a_wet_stamp_costs` — cada chamada de `on_canvas_pointer` de um traço:

| tela | raio | down | move p50 | move p90 | move MAX |
|---|---|---|---|---|---|
| 2048² | 100 | 23,32 ms | 1,83 | 2,01 | 2,36 |
| 2048² | 300 | 33,54 | 5,11 | 6,39 | 6,49 |
| 4096² | 100 | **50,72** | 1,81 | 2,14 | 2,49 |
| 4096² | 300 | **52,85** | 5,25 | 6,21 | 7,12 |

⚠️ **O pen-down custa ~51 ms a 4096²** — a sessão nasce alocando ~1 GB de planos (o *first-touch* que
a §5.31 mediu no `measure_density`: `944 MB` de grid) — e cada move 1,8-5,2 ms, vezes os eventos de
ponteiro que cabem num frame. **É isso o `stamps` do log, e é a fronteira agora.** Fica NOMEADA, não
consertada nesta wave.

#### 5.33.3 O gate, e a fixture que acusava o código errado

`a_frame_slowed_by_another_tenant_does_not_starve_the_water`, com as **duas metades** (o orçamento não
desaba **e** não cresce) — porque as duas correções se cobrem numa fixture só.

⚠️ **A primeira versão do gate passava `dt = 60` FIXO enquanto o próprio tick custava ~40** — um frame
que **não fecha a própria conta**. Nele o `non_sim` caía abaixo do alvo, o recuo disparava *com razão*,
e o gate reprovava sobre produto **correto**. O `dt` agora é realimentado (`estrangeiro + o que o tick
custou`). *Uma fixture que não fecha a própria conta acusa o código errado.*

---

### 5.34 ✅ E O PASSO ATÔMICO CATRACAVA O ORÇAMENTO — a água a 40 Hz, a taxa nominal cheia

> Enio, quarto smoke, **mesmo veredito**: ***"simulação muitíssimo mais devagar"*** — com
> `tool-tick=17.31ms` numa amostra e `0.00` em todas as outras.

**O mecanismo é aritmético.** Um passo de sim custa 12-17 ms e um quadro de 60 Hz tem 16,6 ⇒ **o
frame que contém um passo estoura por construção**. Decidindo pelo `dt` **instantâneo**, o recuo
disparava em TODO passo — e sendo ×0,5 contra +1 de subida, o orçamento **catracava até o piso de
1 ms**. Eu construí um controlador que punia a água por fazer a única coisa indivisível que ela tem
a fazer.

#### 5.34.1 As quatro correções

| # | o quê | por quê |
|---|---|---|
| 1 | `dt` e `non_sim` viram **EWMA** (0,05) | a decisão é sobre carga **sustentada**; um passo isolado quase não move a média |
| 2 | a conta da água é o **TICK INTEIRO** | o composite é custo dela — atribuí-lo ao "outro inquilino" a fazia parecer inocente **em toda medição** (`non_sim` 25 contra alvo 20,4 ⇒ o recuo nunca disparava) |
| 3 | o ramo do inquilino estrangeiro **CRESCE** | *segurar* num piso é ficar preso nele: a sonda mostrou o orçamento parado em **1,04 ms por 80 frames** |
| 4 | teto = **um frame inteiro** (era 0,6) | o `acc` já limita a sim a 0,67 passo/frame ⇒ orçamento maior **não** compra mais simulação; com 0,6 o teto era 10 ms contra um passo de 12-17 e **todo passo era adiado** |

#### 5.34.2 E uma decisão de PRODUTO, declarada em vez de embutida

`WET_FRAME_SLACK_MS` **2 → 8 ms**. O Enio reportou **três vezes o mesmo veredito** — *"o FPS não caiu
abaixo de 60 mas a animação estava lenta"* — ou seja **a água tem prioridade sobre os últimos quadros
por segundo**. Com 2 ms de folga o controlador cedia metade da taxa da água para segurar o vsync
(medido em laço fechado: orçamento **7,0 ms ⇒ sim em ~25 Hz** com o frame a 60 fps). Com 8 ms o alvo
vira ~24,6 ms (40 fps): a água roda **40 Hz cheios** e o frame vai a ~20 ms enquanto ela está viva.

**A escada inteira:** `13,0 / 11,0` → `38,0 / 36,5` → **`40,0 / 40,0 Hz`**.

#### 5.34.3 Os dois gates do laço fechado viraram UNIDADE

⚠️ **Um gate de wall-clock não consegue separar o produto correto da mutação aqui:** sob máquina
carregada um passo custa mais, o frame estoura **de verdade**, e o recuo dispara **com razão**. Medido:
o produto dá 16,6 ms em repouso e **4,16 sob a suíte inteira**, contra 5,2-6,5 da mutação — *as faixas
se sobrepõem, e nenhum limiar as separa*. A versão anterior era **flake por construção**.

Os dois gates agora simulam o laço fechado sobre o `SimBudget` com o custo do passo como um **NÚMERO**
(zero relógio): *um passo de 17 ms não catraca* e *um passo de 120 ms ainda faz recuar* — o par, porque
sem o segundo o primeiro fica verde com o controlador desarmado.

---

### 5.35 ✅ E A RÉGUA DO ORÇAMENTO PREGAVA NO PISO — a água estava a 4 Hz

O split do tick (§5.34) deu o número no primeiro smoke:

```text
tool-tick: media 5.44ms pico 49.53ms em 45/120 frames | stamps: 0/120
agua: sim media 28.70ms pico 47.65ms x8 | composite media 1.91ms pico 2.22ms x8
```

⚠️ **`x8` — oito passos em 120 frames, 4 Hz.** E `28,70 × 8 + 1,91 × 8` fecha *exatamente* o total do
tick, então os outros 37 ticks não fizeram nada: o orçamento estava em **~1,9 ms** num app a 60 fps
com **14 ms de folga ociosa por frame**.

#### 5.35.1 Os dois estimadores que falharam, por motivos opostos

| tentativa | falha |
|---|---|
| **EWMA do `dt`** | um frame lento por culpa alheia **levanta** a régua e o teto junto; e com a água rodando a maioria dos frames contém um passo ⇒ a média inclui **o nosso próprio custo** e o laço é auto-realizável |
| **`min` do `dt`** | **catraca de mão única** — e a premissa era FALSA: `dt` abaixo do vsync **não é espúrio neste app**, é comum (dois frames em sequência depois de um evento dão `dt ≈ 1 ms`). Um único deles pregava a régua no piso ⇒ teto ≈ 2 ms ⇒ **água a 4 Hz**; voltar de 2 a 16,6 com o creep de 0,05%/frame levaria **~70 segundos** |

**O sinal disponível não separa as duas perguntas.** `dt` sozinho (o `Tool` é contrato congelado) não
distingue *"o display mudou de taxa"* de *"este frame foi rápido/lento por outro motivo"*. Então a
régua deixou de ser inferência: é o **quadro de 60 fps que o app tem como alvo**, declarado.

⚠️ **A limitação, nomeada:** num display de 144 Hz a água pode tomar 16,6 ms de um quadro de 6,9 ⇒
**enquanto há água viva o app roda a ~60 fps**, não a 144. É o mesmo trade que o Enio declarou três
vezes, agora no eixo do monitor. Levantá-lo exige o shell **contar** o período do display ao tool —
parâmetro novo no `Tool`, ou seja **§6 + ADR**.

#### 5.35.2 E o TETO FÍSICO da sessão dele, nomeado

Um passo custa **28,70 ms** na poça dele. `40 passos/s × 28,70 = 1148 ms/s > 1000` ⇒ **a taxa cheia
não cabe num núcleo naquela poça.** Com o orçamento em 16,6 ms/frame a água chega a **~35 Hz** (87% do
nominal), e o resto **não é agendável**: é concorrência (a sim fora da thread do frame) ou GPU.

⚠️ E o `composite` está inocentado pela medição: **1,91 ms médios, pico 2,22** — o casco do retângulo
sujo que a §5.30 deixou aberto **não é a fronteira**, exatamente como aquela nota dizia.

#### 5.35.3 O gate, e uma mutação inválida minha

`one_fast_frame_does_not_pin_the_ruler_to_the_floor` — unidade sobre o `SimBudget`, zero relógio: um
frame de 1 ms seguido de 60 de vsync limpo. Nasceu **VERMELHO em 2,06 ms**, e com a catraca
reinstalada dá **2,00** — o mesmo número que o log do produto implicava.

⚠️ **Uma mutação minha não sangrou por ser INVÁLIDA, não por buraco de gate:** eu mutei `dt_avg_ms`,
que o EWMA reescreve duas linhas abaixo. A válida usa campo próprio
([[feedback_a_mutation_that_does_not_bleed_may_indict_the_oracle_not_the_finding]]).

---

### 5.36 ⛔ E AQUI O AGENDAMENTO ACABA — o teto passa a ser FÍSICO, medido e nomeado

O smoke confirmou a §5.35: **`agua: sim ... x54`** contra `x8` ⇒ a água saiu de **4 para 27 Hz
(6,75×)**. O app foi a **55 fps** com `present/acquire-stall=0.00` — não sobrou folga nenhuma.

#### 5.36.1 De que um passo de 33 ms é feito

⚠️ A sonda mede na **escala do produto**, porque a cena diagonal de 2400 px dá 12,7 ms/passo e **não
contém o fenômeno** — decompor ali otimizaria o passe errado.

| passe | ms | % |
|---|---|---|
| `build_flow_field` | 10,258 | 26,5 |
| `rebuild_active_region` | 7,631 | 19,7 |
| `advect` | 7,513 | 19,4 |
| `project` | 7,379 | 19,0 |
| `drying_pass` | 3,069 | 7,9 |
| `smooth_velocity` | 2,671 | 6,9 |
| `apply_boundaries` | 0,226 | 0,6 |
| **total** | **38,747** | |

⚠️ **274.238 células ATIVAS para 46.067 com água — a região ativa é 6× a água.** E **não é gordura**:
é a dilatação 3×3 que a física exige (o campo de fluxo lê os vizinhos, o advect precisa de destino),
escrita no `rebuild_active_region`. Todo passe paga esse fator **por construção**.

#### 5.36.2 E paralelizar está fora, por decisão com mecanismo nomeado

O **ADR-0134** não diz *"é serial"* e para: ele nomeia as dependências — *o brake do flow lê `wet`
**vivo** escrito por células anteriores do mesmo passe, e o drying lê o vizinho esquerdo pós-update* ⇒
a lei de bandas do ADR-0109 é **inaplicável**. (Lido, não re-derivado.)

#### 5.36.3 O teto físico, e as três saídas com o valor MEDIDO de cada uma

`40 passos/s × 33,4 ms = 1336 ms/s > 1000` ⇒ **a taxa cheia não cabe em UM núcleo naquela poça, nem
num dedicado.**

| saída | o que compra | custo |
|---|---|---|
| **nada** | 27 Hz + 55 fps num pincel extremo a 4096² | ⚠️ **em pincel comum a água JÁ roda cheia** (fixture: 12,7 ms/passo ⇒ 40 Hz) |
| **sim fora da thread do frame** | o **app** volta a 60 fps; a água vai a ~30 Hz (+11%) | concorrência, não paralelismo (o passo segue serial) — refatorar quem POSSUI o grid: composite, undo, depósito, o guard de identidade |
| **GPU** | o único caminho aos 40 Hz nessa escala | wave própria, com o precedente do `ph2d-gpu-cook` |

⚠️ **E o `cpu-encode(raw)=21,71ms` do log NÃO é custo novo:** com o stall em `0.00` a espera do
swapchain **migra para dentro do encode** (`gpu-busy=0,87 ms` prova que a GPU está ociosa). Perseguir
aquele número seria otimizar uma espera.

---

### 5.37 ✅ O PASSO VIROU INTERROMPÍVEL — o bloco atômico de 38 ms deixou de existir

> Enio: ***"vamos ao estado da arte, ao padrão ouro, vamos pintar 4k com pincel grande. Resolva."***

O teto da §5.36 é aritmético e **nenhum agendamento o move**: um passo custa 38,7 ms contra 16,6 de um
quadro, então o frame que o continha estourava **por construção**. Duas saídas foram medidas e
descartadas antes desta — **paralelizar** (proibido com mecanismo nomeado no ADR-0134) e **a região
ativa** (o fator 6 é a dilatação 3×3 que a física exige).

**A cura: o maior ESTÁGIO sozinho custa 10,26 ms, e ele CABE na folga do quadro.** `sim_step_stage`
roda **um** estágio; o `StepCursor` carrega os params **capturados no início do passo**, o `grav`, o
`n` e o `vmax`. **`sim_step` virou O LAÇO sobre os estágios** — uma implementação, zero divergência —
e a correção é **byte-idêntica por construção**.

| | atômico | por estágios | |
|---|---|---|---|
| pior tick | 35,3 / 42,0 ms | **26,5 / 28,7** | −25% / −32% |
| tick médio | 9,9 / 12,1 | **7,9 / 9,7** | −20% |
| taxa da sim | 40,0 / 40,0 Hz | **43,5 / 44,5** | +10% |

Poça **pesada** (~38,7 ms/passo, a escala do log do Enio): sim **9,5 → 13,0 Hz**.

⚠️ **Os três melhoram juntos** porque o orçamento passou a ser gasto INTEIRO: antes, um passo que não
cabia no crédito era **adiado por um frame todo**.

⚠️ **Três leis, cada uma um bug se invertida:** o `acc` gateia COMEÇAR um passo, nunca CONTINUAR um ·
o composite roda **só quando um passo COMPLETA** (o artista vê estados inteiros) · toda ação de CANVAS
**drena** o passo em voo (`Engine::drain_step`).

#### 5.37.1 O oráculo tem de ser a rota atômica CONGELADA

⚠️ **O gate não pode ser `sim_step` contra `sim_step_stage`:** `sim_step` **É** o laço sobre os
estágios, então os dois lados passam pelo MESMO código e uma mutação dentro do estágio move os dois —
*razão entre dois doentes, verde por construção*. Medido: com o gate escrito daquele jeito, **três
mutações sobreviveram** (params re-colhidos por estágio · relógio andando por estágio · o
`apply_boundaries` pulado). Contra a referência congelada sob `cfg(test)`, duas sangram.

⚠️ **E uma mutação sobrevive, documentada em vez de escondida:** re-colher os params em cada estágio
não é pego. Duas fixtures tentadas (`Gravity` — inválida, ela vira o `grav` capturado à parte;
`ExtDiffusion` — o controle prova que o knob **não** é inerte e a mutação passou de todo modo). A lei
continua escrita no `StepCursor` e é real, mas está **sem oráculo** — e um gate que não pode falhar
pelo motivo que alega é pior que gate nenhum, então ele **não foi shipado**; o caminho para fechá-la
está escrito no lugar onde ele estaria.

**Fingerprint do engine INTACTO**, suíte de aceitação verde nos dois perfis.

---

## §5.38 — A SIM SAIU DA THREAD DO FRAME: água 15 → 33 Hz, pior tick 73 → 9 ms

O agendamento estava esgotado (§5.31-§5.37): o custo por passo é o piso da física
(16 ns/célula-passe, zero transcendental, faixa justa), e *a taxa visual da água É
a taxa de passos*, então enquanto a sim dividia a thread do frame a taxa dela era
`orçamento ÷ custo do passo` ≈ **15 Hz** numa poça de 4K. O que sobrava não era
*quanto custa*, era **quem paga**.

Agora a sim roda numa thread própria e o tick **mostra** em vez de simular:
**33,4 Hz de água** (84% do nominal de 40), **tick p50 0,049 ms**, **pior tick
9,0 ms** contra os 55-73 de antes. O `acc`, o cap de contagem e o orçamento de
milissegundos **morreram** — com eles, 4 gates e 2 sondas, que é o resultado
honesto.

⚠️ **A peça que fez a wave caber foi o `Deref` no slot do motor**, não o canal: os
**87 sítios** de `sess.engine.…` seguem intactos e o *field splitting* do composite
sobrevive. O preço é que o `Deref` panica com o motor fora de casa, então dez
portas chamam `bring_home()` — e o pânico nomeia o conserto.

⚠️ **`TICK_WAIT = 4 ms` é medido e tem mecanismo:** é o `IDLE_SLEEP` do worker, a
granularidade com que ele responde quando está dormindo. Varredura 0/1/2/4/8 ms →
sim 23,4/26,9/28,9/**33,4**/33,9 Hz. Bloquear de vez media **60,6 ms** de pior tick.

⚠️ **E a decomposição do passo derrubou uma afirmação do próprio repo:** o header
do `measure_wetpaint_tick.rs` dizia *"não há paralelismo byte-idêntico a colher (o
solver é Gauss-Seidel em toda parte — ADR-0134)"*. O ADR nomeia **dois** mecanismos
sequenciais e eles são **34%** do passo; o `project` é **JACOBI** (quatro laços,
cada um lê um buffer e escreve OUTRO) e o `smooth_velocity` é gather puro. Os dois
são row-disjoint. **Não foram feitos** porque o ADR-0109 exige ADR novo para todo
rayon (decisão do Enio) e o ganho é ~1,3× contra os ~2× desta wave — precificado
no doc 29 §6.

⚠️ **Três lições de gate, todas sobre mim** (detalhe no doc 29 §5): eu atribuí um
colapso de 60× ao mecanismo errado **antes de reler a minha própria medição** ·
encolher uma fixture para matar uma flake tirou os dentes do gate · e
`reconcile_facts` não era porta (ele sai antes do `bring_home` no caso comum), o
que o gate da ação de canvas pegou ao vivo.

## §5.39 — O SOLVER NÃO É GAUSS-SEIDEL EM TODA PARTE: um passo 16,08 → 10,34 ms (ADR-0145)

**Ordem do Enio, literal: *"rayon"*.** O que a §5.38 deixou aberto era o **custo por
passo**, e o repo tinha uma afirmação sobre ele que estava **errada**: o header do
`measure_wetpaint_tick.rs` dizia *"não há paralelismo byte-idêntico a colher — o
solver é Gauss-Seidel em toda parte (ADR-0134)"*. O ADR-0134 nomeia **DOIS**
mecanismos sequenciais e eles somam **34%** do passo, não o passo inteiro.

Lidos um a um (o **mecanismo**, não a nota), três passes são row-disjuntos:

| passe | por que ENTRA |
|---|---|
| `project` | é **JACOBI** — quatro laços, cada um lê um buffer e escreve **OUTRO** |
| `smooth_velocity` | **gather puro** — escreve `flow` no próprio índice, lê `vel`/`film`/`active` |
| `rebuild_active_region` | **3 de 4** sub-passadas: a limpeza · o scan da extensão viva (um par de escalares POR LINHA) · o passe 1, cujo trio `film[i±1]` é **HORIZONTAL** |

⚠️ **A terceira porta não estava no meu escopo inicial** — eu havia precificado a
wave em ~1,3× com dois passes; o `rebuild_active_region` a levou a **1,56×**.

⛔ **E quatro ficam SERIAIS por semântica, cada um com o mecanismo escrito:**
`advect` (SUBTRAI nos 4 cantos-fonte, linhas vizinhas) · `build_flow_field` (o
freio lê o `wet` VIVO **e** o backrun espalha em `susp[nb]`/`sett[nb]`) ·
`drying_pass` (lê a vizinhança 3×3 de `susp`, que ele escreve) · a **SAIA** do
rebuild (*"earlier 2s shape later sums"* — a ordem é load-bearing).

**Medido pela porta do produto, mesmo binário, mesma fixture** (pisos em
`usize::MAX` = toda rota serial, contra os pisos medidos; poça canônica de 3 faixas
diagonais a 4096², janela de 5,1 M células):

| um passo inteiro | serial | paralelo | |
|---|---|---|---|
| mediana | 16,083 ms (62,2 Hz) | **10,335 ms (96,8 Hz)** | **1,56×** |
| pior | 26,434 ms (37,8 Hz) | **19,070 ms (52,4 Hz)** | **1,39×** |

⚠️ **Um número que NÃO é o ganho:** comparar duas corridas do `measure_pass_cost`
(uma antes, uma depois do commit) mostrava o `advect` — que esta wave **não toca**
— oscilando **12,1 → 7,8 ms**, 36% de deriva de máquina. Uma soma cross-run
atribuiria isso ao ganho. O A/B tem de ser no MESMO processo, com uma linha de
diferença.

### O desenho: um corpo, dois walkers

O corpo de cada linha é **UMA** função e as duas rotas apenas a caminham
(`par::walk_rows`/`walk_rows2`/`walk_rows_reduce`/`walk_row_scalars2`). Não existe
"versão paralela" do kernel para divergir da serial — `Rows` escolhe o *walker*,
nunca a aritmética.

⚠️ **E isso LIMITA o que o gate de identidade pode provar, o que é a lição desta
wave:** um defeito no CORPO aparece nas duas rotas e é **invisível** para
"paralelo == serial". Provado: a mutação que faz o laço 2 do `project` ler a linha
errada **sobrevive aos quatro gates de identidade** e sangra o **fingerprint**. Os
dois conjuntos são complementares e nenhum substitui o outro —
[[feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body]].

### O piso é POR-PASSE, e a medição derrubou DUAS versões minhas

Escrevi um número único; a varredura mostrou o `rebuild_active_region` **perdendo
0,55×** (quase 2× mais lento) até ~200k células — ele varre TODA linha da tela, não
a bbox, então o número de tarefas é `altura` mesmo numa poça minúscula, e a saia
serial limita o teto dele a ~2,1× por Amdahl.

⚠️ **E a METODOLOGIA era parte do número:** sem restaurar o estado antes de cada
amostra a mesma varredura dava `smooth` a **3,01×** onde a honesta dá **0,95×** em
195k células — repetir um passe sobre o mesmo grid o deixa quente e, no caso do
`rebuild`, **APERTA a bbox que ele próprio varre**. Eu quase fixei os pisos 4×
baixos demais. Pisos finais: 256 Ki (project) · 256 Ki (smooth) · 512 Ki (rebuild),
os dois primeiros iguais **hoje** e mantidos como consts separadas de propósito.

### Gates: 6 + 8 mutações

Identidade byte a byte de todo plano escrito, por rota · a rota paralela repetida
seis vezes dá sempre o mesmo (um *race* benigno passaria no primeiro e falharia
aqui) · a fixture **cruza os três pisos** (senão seria a rota serial contra ela
mesma, a armadilha do `plane_copy`) · e um gate de RELÓGIO, porque identidade não
vê velocidade.

**Duas mutações não contam, e ficam registradas:** trocar os dois planos no
`walk_rows2` é **rejeitada pelo compilador** (os tipos genéricos a tornam
inexprimível), e zerar a identidade da redução da bbox é **semanticamente neutra**
— os extremos só ALARGAM janelas de varredura, então a mutação é mais lenta, nunca
errada.

⚠️ **E uma mutação achou um buraco de fixture na hora:** a poça dos gates é
construída pela porta do PRODUTO (`drive_stroke` → `step_simulation`), então a
mutação do `reduce` fazia o rebuild chamar `empty_bbox`, as DUAS poças saíam **sem
água**, e comparar dois grids vazios era verde. `assert!(has_fluid)` é o que torna
a comparação não-vazia — e com ela a mesma mutação sangra **quatro** gates.

**Aberto:** o passo segue **work-limited**, num número menor. Os 60% que sobram são
os quatro sequenciais, e eles **não têm caminho de CPU** — a próxima alavanca é a
**GPU**, que quebra o port 1:1 e o fingerprint pinado, e exige ADR próprio.

## §5.40 — A CADÊNCIA: por que o 1,56× do §5.39 chegou ao produto como 1,10×, e por que a CPU acabou

O smoke do Enio veio com a taxa da água **inalterada** — 29-38 composites por janela
de 2 s, contra os 37-38 de antes do rayon. Não era o build dele, e a wave não estava
errada: **o número que eu anunciei era da minha fixture.**

### O instrumento estava MUDO, e foi ele que atrasou a resposta

O log do produto imprimia `agua: sim media 0.00ms x0`. Ao mover a sim para fora da
thread do frame (§5.31-§5.38), ninguém mais chamou o `note_step` — quem dá o passo é
o worker. Aquela linha lê-se como *"a simulação não custa nada"* e significava
*"ninguém mede a simulação"*, **sobre exatamente o número que decide se a água lenta
é trabalho ou agendamento** (duas curas opostas). *Um instrumento silencioso é pior
que um ausente: ele TRANQUILIZA.*

Agora o worker reporta o COMPUTE por passo mais três baldes que **particionam** a
janela dele — **busy** (dentro de `step_stage`) · **away** (o motor está com o
frame) · **sleep** (o ritmo de 40 Hz da SPEC). Uma leitura, três mundos:

```text
  MAQUINA OCIOSA (a leitura que vale)
    um traco    busy 49,5%  away  7,0%  sleep 43,4%  ->  38,4 Hz, 12,91 ms/passo
    tres tracos busy 76,8%  away 19,2%  sleep  1,6%  ->  14,0 Hz, 56,90 ms/passo
```

**E a partição separa dois regimes com vereditos OPOSTOS:**

- **um traço já alcança o NOMINAL** — 38,4 dos 40 Hz da SPEC, com **43% de sleep**: o
  worker está adiantado e dorme de propósito. Não há nada a colher aqui, e nenhuma
  otimização mudaria o número (o teto é a SPEC, não a máquina).
- **três traços sobrepostos é work-limited** — busy 76,8%, sleep 1,6%. É a cena do
  smoke, e é a única em que a taxa cai.

⚠️ **A primeira corrida desta partição dizia `um traco: 26,3 Hz, sleep 18,4%`** e eu
quase escrevi que uma pincelada só não alcançava o nominal. Era a **máquina
carregada** — eu tinha dois `cargo` meus rodando ao lado. *Uma partição de duty cycle
mede o AGENDADOR do SO junto, então ela só é lida com a máquina ociosa* — e o erro é
para o lado pessimista, que é o que faz inventar trabalho.

### A CADÊNCIA, que não estava no meu modelo

O `sim_step_stage` **não roda todo passe em todo passo**. Amortizando a decomposição
por-passe da poça do produto pela cadência real:

| passe | custo cheio | cadência | por passo | % |
|---|---|---|---|---|
| **advect** | 26,24 ms | todo passo | **26,24** | 42,3 |
| **drying_pass** | 48,25 ms | ÷3 (`dry_every`) | **16,08** | 25,9 |
| **build_flow_field** | 61,76 ms | ÷4 | **15,44** | 24,9 |
| rebuild_active_region | 5,04 ms | ÷2 | 2,52 | 4,1 |
| smooth_velocity | 1,23 ms | ¾ (o lugar do flow) | 0,92 | 1,5 |
| project | 1,85 ms | ÷3 | 0,62 | 1,0 |
| apply_boundaries | 0,21 ms | todo passo | 0,21 | 0,3 |
| **MODELO** | | | **62,03** | |
| **MEDIDO pelo worker** | | | **62,05** | |

**0,03 ms de erro** — e o modelo diz que os três passes do §5.39 somam **4,06 ms de
62 = 6,5% do passo**, não os ~46% que a soma-sem-cadência sugeria. Seriais custariam
10,3 ⇒ a wave corta **6,2 ms**, que é o **1,10×** que o produto mostra (medido:
12,5 → 14,0 Hz).

⚠️ **A lei:** *um ganho por-passe só vira ganho de produto depois de passar pela
CADÊNCIA, e uma razão medida numa fixture não se transporta para outra cujo mix
por-passo é diferente.* A `measure_pass_cost::scene_big` dirige o `Engine` direto e
custa 10,34 ms/passo; a `heavy_puddle`, que dirige o `on_canvas_pointer` — o caminho
do artista —, custa **62,05**. Seis vezes. **Quando o número vira decisão de
produto, ele tem de sair da porta do produto.**

### A MESMA lição, segunda vez na mesma sessão

Eu inferi um *"imposto de células secas de 35-42%"* da razão diagonal÷horizontal do
`measure_pass_cost` (1,80× / 1,80× / 2,04× com só +18% de células ativas) e ia abrir
uma wave em cima disso. Modelando `custo = a·janela + b·ativas` sobre as **duas**
medições: `a = 0,16 ns/célula-de-janela`, `b = 15,5 ns/célula-ATIVA`. Na poça do
produto (8,42 M de janela, 1,61 M ativas) o imposto das secas é **1,33 de 26,24 ms =
5%**. As cenas da razão têm ~110k ativas; a do produto tem **1,61 M**, e ali quem
manda é o trabalho vivo. *A razão estava certa e a extrapolação, não.*

### E é aqui que a CPU acaba, com o número do piso ao lado

`b = 15,5 ns por célula ativa` contra os **16 ns/visita-de-célula-passe** que o
ADR-0134 declara como *"o teto escalar serial desta física"*. **Não há folga.** Os
93% do passo são `advect` (42%), `drying_pass` (26%) e `build_flow_field` (25%) — os
três recusados pelo ADR-0145 §2, cada um com o mecanismo escrito, e o `advect`
SUBTRAI nos quatro cantos-fonte de linhas vizinhas: nenhuma reordenação disso é
byte-idêntica.

**O que resta de CPU está medido e NÃO construído:** a metade do
`wetpaint_composite` que **não toca o motor** (o *straight-alpha over* de `pigment`
sobre `base`, mais os gates) roda com o engine na mão e por isso entra no `away`;
liberá-lo antes dela vale ~**1,06×** na taxa — abaixo do que o artista distingue.

⇒ **A próxima alavanca é a GPU**, e ela quebra o port 1:1 e o fingerprint pinado que
o ADR-0134 escolheu: ADR próprio + ordem do Enio, a mesma classe da palavra
*"rayon"*.

### Gates

**`the_worker_reports_what_a_step_costs`** — o instrumento não pode emudecer de
novo: passo reportado (`n > 0`, soma > 0) + `busy > 0` + `away > 0`. **3 mutações, 3
sangram**, uma por balde. ⚠️ O gate nasceu VERMELHO no `away` por um motivo que é o
próprio contrato: **ele é medido do `send` até o `recv` do worker VOLTAR**, então a
grandeza só existe quando a viagem FECHA — a pausa de 30 ms no fim do gate é isso, e
não folga de conveniência. ⚠️ E ele é o único teste não-`#[ignore]` que consome a
janela global; um segundo leitor zeraria a dele e o verde viraria sorte.

Sondas: `measure_what_the_off_thread_sim_buys` (a partição + Hz + ms/passo) e
**`measure_what_a_step_of_the_products_puddle_is_made_of`** (a decomposição na poça
que o PRODUTO constrói). ⚠️ A 1ª versão da segunda tirou o snapshot logo após o
pen-up, onde a máscara `active` está **VAZIA**, e todo passe gated em `active[i]==0`
fazia early-out em toda célula: `project` mediu 0,88 ms contra 3,48, e a soma casava
com os 62 ms do worker **por coincidência** — a mesma armadilha do §5.13. O worker
roda o rebuild como 1º estágio; a fixture tem de fazer o mesmo.

### E o smoke do Enio CONFIRMOU o veredito, com um número que eu não tinha

```
agua: sim media 46.73ms pico 106.32ms x38
worker: busy 88% away 12% sleep 0% | TAXA DA AGUA 19.0 Hz
```

**`busy 88%`, `sleep 0%`** — o worker não tem folga; é o regime work-limited na
cena dele, não numa fixture minha. E o **`pico` é 2,2× a média**, que é o que dá
o nome certo ao sintoma: *"não fluida"* ≠ *"lenta"*.

**A causa da irregularidade é a CADÊNCIA, e ela é aritmética.** Com os divisores
2/3/3/4, o custo de um passo depende de `n mod 12`:

```text
  n%12:    0     1     2     3     4     5     6     7     8     9    10    11
  ms:   143,2  27,6  32,6  77,6  93,2  27,6  82,6  27,6  93,2  77,6  32,6  27,6
```

Média **62,0**, faixa **27,6 a 143,2 — um swing de 5,2×** (a razão pico/média do
modelo é 2,3×; o log do Enio mede 2,2×). Todo passo avança a MESMA fatia de tempo
simulado, então a água cobre a mesma distância em 28 ms e em 143 ms: **ela parece
acelerar e frear 5×.** É isso que o olho chama de não-fluido.

⛔ **E evenar isso na CPU é impossível — as três saídas foram checadas e as três
morrem:** (1) mudar a cadência é mudar a física (o fingerframe é o contrato do
ADR-0134); (2) ritmar o worker pelo PIOR caso derruba a média de 62 para 143 ms/passo
= **7 Hz**, muito pior; (3) ritmar o DISPLAY não pode funcionar porque **o engine
guarda UM estado** — um passo pronto não pode ser segurado enquanto o worker continua,
o estado dele já foi sobrescrito. E compositar em fronteira de ESTÁGIO (que é seguro,
o módulo já o declara) **não ajuda**: o movimento todo mora no `advect`, um estágio de
sete, então as outras seis atualizações não mostram nada.

### As DUAS saídas, precificadas — e a escolha é do Enio

O custo é **linear nas células vivas** (`ns/célula` PLANO de 512² a 4096²), e a grade
do fluido é hoje **1:1 com os pixels do canvas** (`Engine::new(side, side)`; o
composite mapeia célula `(cx,cy)` → pixel `(cx-1,cy-1)`) — a 4096² a física paga
**16,7 M células**.

| | passo | taxa | o que custa |
|---|---|---|---|
| hoje (grade 1:1) | 32,5 ms | 30,7 Hz | — |
| **(A) grade 1/2** | **10,8 ms** | **92,3 Hz** | o detalhe SUB-CÉLULA do fluido; resample no composite + dab em coordenada de grade |
| **(B) GPU** | — | — | o port 1:1 e o fingerprint pinado do ADR-0134; wave grande; ADR próprio |

Medido pela porta do produto, MESMO desenho físico (tudo escalado: lado, raio,
comprimento): **3,00× mais rápido com 3,30× menos células** — a razão de tempo
acompanha a razão de células, como a linearidade exige.

⚠️ **(A) faz a água ALCANÇAR o nominal**, não só ficar mais rápida: média 62/3 =
**21 ms contra os 25 ms de um passo a 40 Hz**, e o swing (o pior passo cairia a ~48 ms)
é absorvido pelo `acc`, que existe exatamente para isso (`MAX_BEHIND_S` = 2 passos).
Sonda: `measure_what_a_coarser_grid_would_buy`.

⚠️ **(A) SHIPOU no mesmo dia — §5.41 — e duas coisas desta tabela estavam
ERRADAS:** ela dizia que a razão *"re-pina o fingerprint"* e que precisaria de
**ADR**. Nenhum dos dois: o motor sempre foi agnóstico de dimensão, o
`tests/fingerprint.rs` ficou intacto, e não há decisão de arquitetura a tomar
(a razão é um número que o host escolhe, não um contrato). E o ganho medido pela
porta do produto é **2,7× a 2:1 e 9,1× a 4:1**, não os 3,00× que a medição
substituta estimou. *A estimativa era minha; o número é do produto.*

---

## §5.41 — A GRADE DO FLUIDO DESACOPLOU DO PIXEL (2026-07-29)

> Ordem do Enio: *"Os dois (A) e (B). Inclusive quero ter total controle sobre a
> grade do fluido, quero um slider em que eu possa usar a grade desde 1:1 até
> 1:30 px. Quero esse slider como primeiro widget da seção wet paint, acima das
> tools."* — e, na mesma janela: *"EM Tuning: Pigment Mixing (K-M) temos séria
> queda de FPS. Resolva isso."*

### O que a §5.40 deixou aberto

A água era **work-limited** (`busy 88%`, `sleep 0%`, 47-51 ms/passo, 17-19 Hz) e
os 93 % do passo eram os três passes seriais por semântica — sem caminho de CPU.
A alavanca que sobrava não era *quanto custa uma célula*, era **quantas células
existem**: o custo é linear nas células vivas e a grade era **1:1 com os pixels**.

### O que shipou

**`wetpaint/grid_map.rs`** — a porta única da conversão pixel↔célula, com a
razão autorada em `WetPaintState.grid_ratio` (1..=30) e congelada por sessão em
`WetSession::ratio`. O slider **"Grid Size (px)"** é o primeiro widget da seção.

Medido **pela porta do produto** (4096², pincel r=100, 3 faixas diagonais;
`measure_what_the_grid_ratio_buys`):

| razão | grade | células vivas | ms/passo | Hz | ganho |
|---|---|---|---|---|---|
| 1:1 | 4096×4096 | 1.607.169 | 32,2 | 31 | — |
| **2:1** | 2048×2048 | 486.789 | **12,0** | **83** | **2,7×** |
| 3:1 | 1366×1366 | 227.319 | 6,0 | 166 | 5,3× |
| 4:1 | 1024×1024 | 128.000 | 3,6 | 282 | 9,1× |
| 8:1 | 512×512 | 32.391 | 0,77 | 1293 | 44,7× |

**A razão 2 já passa o nominal de 40 Hz da SPEC** — a água deixa de ser
work-limited e o worker volta a dormir adiantado.

### O K–M foi curado pelo MESMO slider, e o Glaze já estava curado

O custo do `km_mixing` é **por célula** (9 misturas de cor por célula
advectada), então a razão o corta na mesma proporção
(`measure_what_km_costs_at_each_grid_ratio`):

| razão | passo K–M off | passo K–M **ON** | custo | composite off → ON |
|---|---|---|---|---|
| 1:1 | 32,7 ms | **104,4 ms** (9,6 Hz) | **3,2×** | 14,9 → 14,9 (**1,00×**) |
| 2:1 | 11,3 | 31,0 | 2,8× | 21,1 → 20,0 (0,95×) |
| **4:1** | 3,0 | **8,3** | 2,7× | 17,8 → 17,9 (1,01×) |
| 8:1 | 0,76 | 2,1 | 2,8× | 17,2 → 17,2 (1,00×) |

Duas leituras que importam:

* o report é **real e grande** — a 1:1 o Pigment Mixing leva a água a **9,6 Hz**,
  praticamente parada;
* **o Glaze Layering JÁ estava curado** pelo doc 24 (a tabela sRGB): custo
  medido **1,00×**, zero. O que derruba o FPS é só a metade da SIM.

A 4:1 o K–M ligado custa **8,3 ms**, abaixo do kill de 12 do ADR-0134.

### Por que o fingerprint do ADR-0134 fica INTACTO

⚠️ **A nota que a §5.40 deixou dizia que a razão *"re-pina o fingerprint"*. É
FALSO, e a construção o derrubou.** O motor sempre foi agnóstico de dimensão —
a suíte de aceitação dele roda em 900×450, 300×200 e 60×60 justamente porque a
dimensão nunca foi parte da física —, então `Engine::new(gw, gh)` com números
menores é o **mesmo código**. O que a razão muda é *de quantos pixels o HOST
fala com ele*, e essa conversão vive toda no `grid_map`. `tests/fingerprint.rs`:
**2/2 verdes**, sem re-pin. *A estimativa era minha; o fato é do produto.*

### A convenção, e as quatro portas que TÊM de ser inversas

A célula `c` cobre os pixels `[(c−1)·r, c·r)`, logo o centro dela é
`(c−1)·r + r/2`. O motor recebe posições em que o centro da célula `c` vale
`c + 0,5` — é o que o `+ 1.0` que a rota do dab sempre somou significa. Com a
razão isso vira **`u = px / r + 1,0`**, e em `r = 1` a expressão é
*literalmente* `px + 1.0`.

| pergunta | porta | quem chama |
|---|---|---|
| de que tamanho é a grade? | `grid_dims` (⚠️ `div_ceil`) | o nascimento da sessão |
| onde, em células, está este ponto? | `px_to_cell` / `px_len_to_cell` | a rota do dab |
| que pixel é o centro desta célula? | `cell_center_px` / `cell_center_texel` | a silhueta, o Grain, o Paper |
| de que células sai este pixel? | `SampleU::at` | o composite |

⚠️ **Se o dab pousa numa célula e a silhueta é avaliada noutra, o carimbo sai
deslocado de meia célula** — a doença `seed == sample`. Escrever a aritmética
duas vezes é como isso acontece, então ela é escrita uma vez e há gate de
inversão em 7 razões.

⚠️ **O upsample é bilinear PREMULTIPLICADO.** Straight-alpha puxaria a cor de um
vizinho transparente para dentro da tinta (o halo); há gate com fixture de
vermelho-opaco ao lado de verde-transparente. E o **véu do show-wet é nearest de
propósito**: o menisco é um gradiente MEDIDO na grade (`film[i±1]`), e
interpolá-lo desenharia uma crista que o solver não tem.

⚠️ **Trocar a razão ENCERRA a sessão de água viva** (encerrar É o bake — a tinta
que se vê já está no `canvas_rgba`); reamostrar catorze planos de `f32` para a
resolução nova inventaria água que o solver não produziu. **Re-emitir o mesmo
valor não encerra nada** — o guard de igualdade é o que torna o chip numérico
seguro sob arrasto.

### O PREÇO, medido e nomeado em vez de escondido

A tile de cerdas do modelo (128×128) é indexada em unidades de **CÉLULA**, com
pontas de ~1-2 células. Então o que decide se o banco resolve o pincel é o
**raio em células**, `raio_px / ratio` — medido pelo caminho do produto (com a
silhueta do host):

| raio (células) | massa | cobertura |
|---|---|---|
| < 1,5 | **0,0** | 0 % ← **nada é depositado** |
| 1,5 | 16,6 | 14 % (uma célula) |
| 3,0 | 204,2 | 7 % |
| 6,0 | 1375,9 | 6,2 % |
| 12,5 | 4990,6 | 5,5 % ← o regime normal |
| 25,0 | 16258,3 | 5,2 % |

A cobertura converge em ~5 % (a densidade do banco — o depósito do modelo é
**inerentemente esparso**, é uma pincelada de *cerdas*), então **acima de ~6
células o depósito é auto-similar**: é o de sempre, resolvido mais grosso.
⇒ **a razão útil é ≈ `raio_px / 6`** (100 px suportam ~16:1; 12 px, ~2:1).

⚠️ **Sem piso e sem cap, de propósito.** Um piso no raio em células faria o
pincel pintar MAIOR do que o artista pediu (mentira silenciosa); um cap na razão
faria o pincel decidir a resolução do fluido. O comportamento honesto é o que o
gate pina: com a célula maior que o pincel, nada sai — e o slider diz
*"Grid Size (px)"* ao lado do tamanho do pincel, então a leitura é direta.

⚠️ **A cura possível está mapeada e NÃO foi construída:** ler a tile em escala de
CANVAS faria a granulação convergir para a da razão 1. É mudança de APARÊNCIA do
depósito ⇒ wave própria com smoke próprio, não contrabando dentro de uma wave de
perf.

### Custo que a wave ADICIONA, nomeado

O composite não cai com a razão (é O(pixels de canvas)) e a bilinear o encarece:
tela cheia a 4096² mediu **14,9 ms a 1:1 contra 17-21 a 2:1..8:1**. No produto o
composite é da região suja (~3,2 ms no log do Enio), então o acréscimo é
~0,5-1,3 ms — contra ~20 ms economizados no passo.

### As lições de fixture desta wave (SEIS, e todas minhas)

1. **"px 7 está a meio caminho"** — com `r` par nenhum pixel cai no meio de duas
   células (os centros ficam em 4,0 e 12,0; o meio, 8,0, é a *fronteira* entre os
   pixels 7 e 8). O gate falhou sobre código correto; a propriedade verdadeira é
   a **simetria** (os dois pixels que abraçam o meio somam a cobertura cheia).
2. **O gate da borda nasceu VERMELHO na razão 1** — isto é, sobre o mundo que já
   shipava: o stamp do motor recorta em `min(grid_w − 1)`, então a última coluna
   viva nunca recebe, em nenhuma razão. *Um gate que falha no controle está
   medindo a coisa errada.*
3. **A massa de tinta não é o oráculo da granulação** — cada célula pinta `r²`
   pixels, então a massa visível se preserva (medido: 74 %) enquanto a forma
   granula. Mesma armadilha que o doc 25 §13.10 registrou no eixo do traço.
4. **O probe do limiar passou `sil = None`** e mediu um cliff em 3 células; o
   produto SEMPRE passa a silhueta do host, e por ela o cliff é **1,5**. A
   fixture tem de conter o caminho do produto.
5. **O deslocamento de centroide que eu achei ser erro de conversão** era
   assimetria de ponta de traço + estatística de cerdas — o teste decisivo é um
   **dab único**, simétrico por construção.
6. **A mutação que não sangrou acusou a minha AFIRMAÇÃO** (a 2ª vez nesta linha,
   depois da comutatividade IEEE-754 no warp): forçar a razão 1 pela rota
   bilinear dá **0 bytes divergentes** — as frações são zero exatas, os cantos
   caem no `continue` de peso zero, e o `round() as u8` absorve a diferença de
   ordem. A rota de identidade existe pelo **CUSTO**, não pela identidade.

### Gates

10 novos: byte-identidade contra o **over congelado** (`over_as_it_shipped`, sob
`cfg(test)` — oráculo, não segunda resposta) · as portas inversas em 7 razões ·
`div_ceil` · upsample plano · o vazamento premultiplicado · o clamp de borda · a
grade encolhe **e** a tinta pousa no mesmo lugar · a troca de razão encerra a
sessão (e a re-emissão não) · o barramento do painel · a row é o **primeiro**
widget (comparação de `y` contra os 7 chips) e está **viva sob o mouse**
(`click_at` real) · o cliff do banco de cerdas.

**6 mutações, 5 sangram**; a sobrevivente está documentada acima (lição 6).

### Aberto

* a cura da granulação (a tile em escala de canvas) — **decisão de aparência**;
* o default fica em **1** de propósito: mudar a resolução do fluido por default
  mudaria o desenho de toda arte já feita, e o ponto de operação é do artista.
  Com o número da tabela na mesa, é escolha do Enio mover o default;
* a bilinear do composite não é otimizada (os pesos se repetem dentro de uma
  célula) — 0,5-1,3 ms contra 20 economizados, não vale a complexidade hoje.

### Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
env PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release
```

Canvas **4096**, pincel grande, Wet Paint no dropdown. O que olhar:

1. **A seção abre com "Grid Size (px)" no TOPO**, acima de Paint/Erase/Smear/…
2. Com **1** (o default) a água corre como antes: `TAXA DA AGUA ~19 Hz`,
   `busy 88%`.
3. Ponha **2** e pinte de novo: a taxa tem de **passar de 40 Hz** e o `sleep`
   tem de subir. Ponha **4** e ela voa.
4. **O K–M:** abra o Tuning → EXPERIMENTAL → Pigment Mixing. Com grade 1 a água
   quase para (9,6 Hz medidos); com grade 4 ela fica utilizável.
5. **O preço:** com grade 30 e um pincel PEQUENO nada é pintado — a célula é
   maior que o pincel. Com pincel grande a faixa inteira funciona.
6. Trocar a grade **encerra a água viva** (a tinta fica; o escorrido em voo, não).

---

## §5.42 — A MULTI-RESOLUÇÃO: o fluxo é grosso, o pigmento é da tela (2026-07-30)

> Ordem do Enio, com foto: *"Ainda não temos o AA funcionando! … Fique muito
> esperançoso com a possibilidade de grade grossa só para velocidade/pressão,
> pigmento e wetness na resolução da tela. Mas que cada ajuste desses seja
> colocado na UI junto ao nosso slider."*
>
> Plano: [`30_plano_multiresolucao.md`](../../../Painter/30_plano_multiresolucao.md).

### O que ficou

`vel_x` · `vel_y` · `flow_x` · `flow_y` moram numa grade **`Flow Grid` vezes
menor** que a do pigmento; `film`, `susp`, `sett`, cores, `wet`, `paper`,
`active` e `bloom` ficam onde estavam. Dois sliders no topo da seção Wet Paint —
`Grid Size (px)` e `Flow Grid (x)` — com um readout derivado embaixo
(`fluido 1024x512 - fluxo 256x128`).

### Os números — e ⚠️ **os primeiros que eu publiquei eram da porta ERRADA**

A tabela original desta seção saiu da fixture que dirige o **`Engine` direto**
(10,4 ms/passo). A §5.40 já tinha medido que as duas fixtures "grandes" dão
números incomparáveis — *"quando o número vira decisão de produto, ele TEM de
sair da porta do produto"* — e **eu caí nela na mesma wave em que a citei**.
Pela porta do artista (`on_canvas_pointer`, `heavy_puddle` 4096², ciclo de
cadência de 12 passos, mediana de 7):

| `Flow Grid` | ms/passo | Hz | razão | células de fluxo |
|---|---|---|---|---|
| 1 | 63,3 | 15,8 | 1,00× | 16,8 M |
| 2 | 55,9 | 17,9 | 1,13× | 4,2 M |
| **4** | **50,7** | **19,7** | **1,25×** | 1,05 M |
| 8 | 50,1 | 20,0 | 1,26× | 0,26 M |

Por passe, `Flow 1 → 4`, na MESMA poça:

| passe | Flow 1 | Flow 4 | razão | cadência |
|---|---|---|---|---|
| `build_flow_field` | 66,14 ms | **3,23** | **20,49×** | ÷4 |
| `project` | 2,07 | 0,40 | 5,14× | ÷3 |
| `smooth_velocity` | 1,20 | 0,28 | 4,30× | ×¾ |
| `advect` | 30,66 | 34,51 | 0,89× | ×1 |
| `drying_pass` | 46,78 | 46,46 | 1,01× (intocado) | ÷3 |
| `rebuild_active_region` | 5,28 | 4,93 | 1,07× | ÷2 |
| **amortizado** | **67,02** | **53,61** | **1,25×** | |

⚠️ **E o `drying_pass` virou o maior item isolado** (46,8 ms ÷3 = 15,6 = **29%
do passo**), sem ganho nesta wave e sem caminho de CPU nomeado.

⚠️ **A wave não é sobre velocidade.** O `Grid Size` da §5.41 compra **9,1×** na
razão 4 — 25× mais que isto — e o preço dele é o pigmento GROSSO, que é
exatamente a foto do Enio. A entrega aqui é **a borda fina com o fluxo barato**;
o 1,3× é troco.

### A F1 reescreveu o desenho antes de uma linha ser construída

A fase 1 existia para medir o risco #1 do plano (*a redução pode comer o
ganho*). Ela mediu e derrubou **três** afirmações do plano, as três minhas:

1. **A REDUÇÃO é a rota errada; a certa é AMOSTRAR.** Mediar os planos finos é
   `O(finas)` por construção — **não encolhe com `rf`** — e a **3,69 ms**
   custava mais que os dois passes que alimentaria (1,49 somados). Amostrar UMA
   célula fina por bloco é `O(grossas)`, **0,29 ms**, e encolhe por `rf²`.
   ⚠️ Não é a mesma resposta (uma feição de 1 px pode cair ENTRE dois pontos),
   mas o campo de fluxo é **suave por física** — a premissa inteira do inkwash —
   e o que sobra é pergunta de **aparência**, para o render-and-look.
2. **O `build_flow_field` não precisa ser FATORADO.** O plano chamava a
   fatoração de *"o item de maior risco da wave"*, na teoria de que o backrun
   (que espalha pigmento) o prendia no fino. **Medido: backrun +0,06 ms,
   fingering +0,04 — 99,4% do passe é o NÚCLEO**, que é justamente a parte que
   quer ser grossa.
3. **O ganho é 1,3×, não 1,7×**, e eu errei pelo motivo que a **§5.40 já tinha
   documentado**: a §1.4 do plano somava os passes **sem a CADÊNCIA**, e o
   `build_flow_field` roda **÷4**.

### A regra de unidade, em uma linha

A velocidade é sempre medida em **células FINAS por frame** (é o que o `advect`
back-traça e o que o `maxVelocity` significa), então **toda DIFERENÇA FINITA
tomada na grade de fluxo leva um `/rf`** — e nada mais leva. Médias (a
viscosidade, o `smooth_velocity`, a relaxação de Jacobi) são adimensionais;
velocidades injetadas direto (o push do backrun, o do fingering, a gravidade, o
sopro) já estão na unidade certa.

⚠️ **`rf = 1` reduz LITERALMENTE, não a um épsilon:** `(x−1)/1+1` **é** `x`, e
`x * 1.0` é **exatamente** `x` em IEEE-754. É isso que faz do fingerprint do
ADR-0134 a rede de segurança de cada fase em vez de uma promessa — e ele ficou
**intacto** da primeira à última.

### O momento NÃO virou passe próprio, e a razão é byte-identidade

O `advect` **escreve `vel` por célula fina** (`flow` amostrado na fonte +
`gravidade × film LOCAL`), então com `vel` residente no grosso essa escrita não
tem para onde ir. O desenho óbvio — extrair a atualização de momento para um
passe coarse próprio, como o inkwash — foi **descartado**: o `advect` é uma
varredura **SEQUENCIAL** cujas escritas de `film` alcançam as células ainda por
visitar, então o `f` que a gravidade multiplica depende de onde o laço está. Um
passe separado leria o film de ANTES de qualquer advecção, e a rede de segurança
de `rf = 1` cairia junto.

⇒ **a célula PROBE do bloco é quem escreve a velocidade** — a mesma lei da
amostra que os passes de fluxo usam. Em `rf = 1` toda célula é o próprio probe.

### ⚠️ DUAS regressões de perf minhas, achadas medindo e não supondo

A primeira medição do produto deu **0,78×** — a wave saindo **mais lenta**. A
decomposição por passe nomeou o culpado em uma linha: `build_flow_field` estava
10,25× mais rápido e o **`advect` estava 0,81×**, e ele roda **todo frame**
enquanto o build roda ÷4.

1. **`is_probe_cell` por célula são DUAS divisões inteiras** no laço mais quente
   do motor. A posse do bloco virou **CAMINHADA** (uma divisão por LINHA, soma e
   comparação no resto): **11,55 → 8,89 ms**, já em `rf = 1`.
2. **`flow_at_point` dividia por `rf` duas vezes por célula fina.** Recíproco
   pré-computado no `FlowGeom`: **10,17 → 8,96** em `rf = 4` (0,87× → **0,97×**).

### ⚠️ E QUATRO defeitos de fixture, todos falhando no CONTROLE

*Um gate que falha no controle está medindo a coisa errada* — a lição da §5.41,
cobrada quatro vezes seguidas:

1. **O alcance ABSOLUTO do escorrido não é oráculo:** a folha SECA enquanto a
   água corre, então a região molhada encolhe mesmo com o campo perfeito
   (medido, sem gravidade: 106 → 81 em 60 passos). O oráculo é a **DIFERENÇA**
   com e sem gravidade.
2. **40 px/frame amontoa 44% da tinta no fim do traço** — a janela de 123
   células do trail (o item aberto `TRAIL_HALF` do doc 21). A fixture media o
   trail, não a grade.
3. **"mais de 110 de 121 colunas pintadas" falhava em `rf = 1`**, que pinta
   **100** com um vão de 17: o vão é a estrutura de **CERDAS** do pincel. E o
   achado que a correção revelou: `rf = 4` reproduz o controle **idêntico**
   (100/121, os mesmos 2 blocos, os mesmos vãos).
4. **A MEDIANA de passos avulsos esconde o passe que a wave otimiza** — o
   `build_flow_field` roda 1 frame em 4, então a mediana é sempre um frame SEM
   ele: ela reportava **1,04×** onde o ciclo de 12 passos reporta **1,27×**.

### Gates

10 de porta (`ph2d-wet-paint/src/flow_tests.rs`) · 5 de comportamento
(`tests/flow_grid.rs`) · 5 de tool (`wetpaint/flow_ratio_tests.rs`) · 3 de seam
(`seam_wetpaint.rs`). **10 mutações, 10 sangram.**

⚠️ **Duas sobreviveram na primeira rodada e as duas eram buraco meu:**

* tirar o id do array `PAINTER_WETPAINT_FIELDS` deixava a row **pintada, viva
  sob o mouse e MUDA** — e o gate que parecia cobri-lo
  (`the_knob_rows_are_offered_only_while_armed`) **ITERA o próprio array sob
  teste**, então encolher o array encolhe a lista que ele percorre e ele segue
  verde afirmando menos. **Oráculo auto-referente.** Gate novo nomeia o id por
  LITERAL;
* e a minha primeira mutação de POSIÇÃO era **inválida** (o recorte movia a row
  para entre o readout e os tools, isto é, ainda ANTES deles).

### Aberto

* **O `advect` (8,7 ms) e o `drying_pass` (3,1) somam ~88% do que resta**, os
  dois FINOS e os dois já nomeados como *"não ganham nada"*. A próxima alavanca
  de CPU não está aqui.
* **A APARÊNCIA da amostra** — o backrun fica **esparso** em `rf > 1` (um sítio
  de nucleação por bloco em vez de um por célula) e o freio de absorção sonda a
  célula amostrada. São mudanças de desenho **para o smoke decidir**, não
  defeitos; nenhuma delas tem gate numérico, e é de propósito.
* **`MAX_FLOW_RATIO = 16`** não é teto de recurso (o custo já é desprezível a 8):
  é onde a grade de fluxo deixa de resolver o PINCEL. O número final é do smoke,
  e o readout torna o limite **visível**.

### Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
env PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release
```

Canvas **4096**, pincel grande, Wet Paint no dropdown. O que olhar:

1. A seção abre com **dois** sliders — `Grid Size (px)` e `Flow Grid (x)` — e um
   readout embaixo dizendo as duas grades resolvidas.
2. **Deixe `Grid Size` em 1** (pigmento na resolução da tela) e ponha
   **`Flow Grid` em 4**. A água tem de correr **igual** e a taxa subir.
3. **A pergunta da wave, e é de OLHO:** com `Grid Size 1 + Flow Grid 4` a borda
   do escorrido tem de ficar **fina** — compare com `Grid Size 4 + Flow Grid 1`,
   que é a granulação da §5.41. Se as duas parecerem iguais, a amostra do fluxo
   está grosseira demais e o `MAX_FLOW_RATIO` desce.
4. **O backrun esparso:** com `Flow Grid` alto, o padrão de *backrun* fica mais
   espaçado (um sítio por bloco). É esperado — reprove se ler como artefato.
5. Trocar qualquer uma das razões **encerra a água viva** (a tinta fica; o
   escorrido em voo, não).

---

## §5.43 — A SECAGEM: o `fmod` do JS e a janela deslizante (2026-07-30)

A multi-resolução (§5.42) deixou uma frente nomeada com número: com o
`build_flow_field` **20,49× mais barato**, o `drying_pass` virou **o maior item
isolado do passo** — 46,8 ms ÷3 de cadência = **29%** — *"sem ganho nesta wave e
sem caminho de CPU"*. Esta seção é a resposta a essa frase, e ela existia.

### §5.43.1 A forma, e uma fixture minha que mentiu

Primeiro a pergunta que não custa relógio (`measure_what_the_drying_pass_visits`,
poça do produto a 4096²):

| | células | |
|---|---:|---|
| grade | 16 777 216 | |
| faixa viva (o laço) | 1 955 483 | 11,7% da grade |
| **trabalham** | **1 293 992** | 66,2% da faixa |

⚠️ **E a mesma sonda disse *"re-wet: 0 células"*, o que é FALSO.** Ela lê o
`sett` do estado congelado, e o `sett` de uma poça fresca é zero — mas o bloco
de re-wet lê o `sett_c` **depois** do settle da MESMA célula, no mesmo passe.
*A fixture não continha o fenômeno no instante em que eu o medi*; a ablação do
produto o mostrou em 22 ms.

### §5.43.2 A atribuição veio do PRODUTO, porque o meu laço foi apagado

A primeira decomposição dissecava o passe com laços **próprios** e reportou
`só a varredura 2,97 · varredura + gather 5,93 · varredura + tocar as duas cores
2,93` contra um passe de **47,0 ms** — números que **não reconciliam com o
total**. Causa: ler `susp_rgb[i]` e escrever o mesmo valor de volta é **código
morto**, e o LLVM o remove. ⚠️ *Uma ablação que o otimizador pode provar inútil
mede zero e parece um achado.* A atribuição honesta é cortar o **PRODUTO** peça
por peça e medir pela porta:

| corte no produto | passe | atribuído |
|---|---:|---:|
| baseline | 46,2 ms | |
| sem o gather 3×3 | 37,6 | **8,7 ms** |
| sem o bloco de settle | 10,7 | 35,5 |
| `alpha_of_mass` → truncamento | 33,0 | **13,2 ms** |

### §5.43.3 O achado: a consulta de opacidade chamava a libm

`alpha_of_mass` é a tabela de opacidade da SPEC §3, e ela indexa por
`jsmath::to_int32_wrapping` — a semântica **ToInt32 do JS**, portada como
`v.trunc().rem_euclid(2^32)`. E `%` em `f64` **não é uma instrução: é uma
chamada a `fmod`**. Medido isolado, **2,51 ns por consulta contra 0,54** — e a
secagem faz **cinco** consultas por célula trabalhada.

**A cura é o mesmo raciocínio do doc 24** (tabelar o que a libm respondia), um
degrau abaixo: no domínio que uma massa de pigmento de fato ocupa
(`0 <= m < 2^31`) o `trunc()` já cai dentro de `[0, 2^32)`, então **o resto é a
identidade** e o `as i32` do Rust trunca para o mesmo inteiro. O caminho rápido
não é aproximação — é **a mesma resposta sem o `fmod`**; negativo, NaN,
infinito e `m >= 2^31` caem no caminho de sempre, **verbatim**.

⚠️ **O teto do guard é load-bearing:** em `2^31` o `as i32` do Rust **satura** e
o ToInt32 do JS **envolve**. A mutação que troca `m >= 0.0 && m < 2^31` por
`m >= 0.0` sangra **só** no gate novo.

⚠️ **A porta antiga ficou CONGELADA sob `cfg(test)`** (`alpha_of_mass_reference`)
para o gate ter um oráculo que não é o código sob teste — a lição do
`warp_axis` / `serial_side` / `sim_step_atomic_reference`. E `table_at` é a
porta ÚNICA do clamp, para os dois caminhos não ganharem tetos diferentes.

**O ganho cai em TODO consumidor**, não só na secagem: `trail/transfer`,
`tools`, `grid` (o composite) e o `drying` compartilham a porta.

### §5.43.4 E o gather 3×3 virou uma janela deslizante

O fator de borda pergunta *quantos dos 9 vizinhos carregam pigmento*. Escrito
direto são **nove cargas por célula**; mas o laço anda em `x`, e as três colunas
de uma célula são duas colunas da seguinte. Guardando uma **soma por coluna**, a
conta vira `c[x-1] + c[x] + c[x+1]` com **uma** coluna carregada por passo.

⚠️ **A metade que torna isto byte-idêntico é a linha do MEIO.** O `susp` da
linha `y` é escrito por este mesmo laço (Gauss-Seidel — o mecanismo que o
ADR-0134 nomeia), então a célula `x+1` lê em `x` o valor **pós-escrita**. Por
isso a soma de uma coluna é guardada em **duas partes** — `ud` (as linhas de
cima e de baixo, estáveis) e `m` (a do meio) — e quem escreve avisa a janela
(`note_write`). Fundir as duas num inteiro tornaria a correção inexprimível.

⚠️ **A janela avança em TODA célula**, inclusive nas que o early-out pula: uma
célula pulada não escreve, mas **é** vizinha das próximas.

⚠️ **O avanço carrega a coluna `i + 1`, nunca `i + 2`** — o pad da grade é de
UMA coluna (`s = w + 2`), então `bx1 + 2` cairia na linha seguinte. A primeira
versão fazia `i + 2` no rodapé da iteração; mover o avanço para o **topo**
(pulando a primeira célula, que a semeadura já cobre) tira o caso especial em
vez de o remendar.

**E o ganho é honesto: 1,65 ms de 33,8 (4,9%)**, não os 8,7 que a ablação
sugeriu — porque cortar o gather também deixava o compilador dobrar `(1 - e)`
para zero e matar trabalho a jusante. Fica pelo mesmo motivo que o
`value_noise_pair` da §5.11: **estritamente menos trabalho, exato e gateado**.

### §5.43.5 O número, medido A/B no mesmo estado de máquina

Dois builds costas-com-costas, a MESMA poça, pela porta do artista:

| | ANTES | DEPOIS | |
|---|---:|---:|---|
| **`drying_pass`** | 46,08 ms | **32,13 ms** | **1,43×** |
| passo, Flow 1 | 64,42 (15,5 Hz) | **60,79 (16,4 Hz)** | 1,06× |
| passo, Flow 4 | 50,81 (19,7 Hz) | **47,95 (20,9 Hz)** | 1,06× |
| soma amortizada, Flow 4 | 53,54 | 49,17 | 1,09× |

Reprodutível: três corridas do passo dão 47,42 / 47,48 / 47,60 ms.

**Fingerprint do ADR-0134 INTOCADO** — é ele que torna esta wave uma reescrita
de hot loop e não uma mudança de modelo. **5 mutações, 5 sangram** (3 na
opacidade, 2 na janela, estas últimas sangrando o gate de unidade **E** o
fingerprint). Nenhum schema, nenhum contrato congelado, nenhum id/token
(`PROJECT_SCHEMA` 37), **nenhuma dep nova**.

### §5.43.6 Aberto, com o preço certo

Com a secagem em 21,9% do passo, **o `advect` é 70,4%** — e o ADR-0146 já o
nomeia: ele **SUBTRAI** nos quatro cantos-fonte de linhas vizinhas (scatter),
então nenhuma reordenação dele é byte-idêntica, e o que sobra por célula é
gather/scatter de `susp` + `susp_rgb` (12 B) + `film` sobre duas linhas. **Não
há caminho de CPU nomeado para ele.** Os outros itens somam 8%.

E a varredura por `fmod` foi feita: o **único** outro `libm` nos passes quentes
é o `sin`/`cos` do `ext_fingering`, que é uma extensão gateada por knob e mede
+0,04 ms.

---

## §5.44 — O `advect`: o que a ablação atribuiu, e o que o RELÓGIO negou (2026-07-30)

Com a secagem em 21,9% (§5.43), o `advect` virou **70,4% do passo**. A §5.43
tinha acabado de ensinar que *"sem caminho de CPU nomeado"* é uma afirmação
sobre o que se procurou — então a frente foi aberta com a mesma receita:
ablacionar o **produto** peça por peça, pela porta do artista.

### §5.44.1 A decomposição

`measure_what_an_advect_is_made_of`, poça de 4096², mediana de 7:

| corte no produto | Flow 1 | Flow 4 |
|---|---:|---:|
| baseline | 38,15 ms | 35,81 ms |
| sem o **momento** (`flow_at_point`) | 23,51 (**−14,6**) | 34,11 (−1,7) |
| sem o gather de **pigmento** | 22,27 (−15,9) | 19,24 (−16,6) |

⚠️ **O momento custa 14,6 ms em `Flow Grid 1` e 1,7 em `Flow Grid 4`** — 8,6×
de diferença, e o mecanismo é o desenho da §5.42: `owns_flow` é verdadeiro para
**toda** célula em `rf = 1` (toda célula é a probe do próprio bloco) e para 1 em
16 em `rf = 4`. **O slider de Flow Grid já removia 90% deste item**, e ninguém
tinha notado porque a atribuição por-passe nunca desceu abaixo do passe.

### §5.44.2 ⛔ MEDIDO E REJEITADO — não refaça: reusar a moldura bilinear

A hipótese natural, e ela parecia forte: em `rf = 1` a `flow_at_point(sx, sy)`
**recomputa** `x0`/`y0`/`a`/`b`, os quatro índices e os quatro pesos que o laço
do `advect` **acabou de computar para o MESMO ponto**. Foi construída — porta
`flow_at_frame` (a soma bilinear com a moldura pronta), a `flow_at_point`
congelada sob `cfg(test)` como oráculo, gate bit-a-bit com frações
não-diádicas, 3 mutações.

**Medido A/B no mesmo estado de máquina: `advect` 29,98 → 30,24 ms.** Nada. O
passo não se moveu.

⚠️ **O porquê, e é a lição:** a `flow_at_point` é `#[inline]`, então as duas
"recomputações" viviam no MESMO bloco básico, e o LLVM já as tinha fundido por
**eliminação de subexpressão comum**. O que a ablação mediu em 14,6 ms são as
**8 cargas dispersas** de `flow_x`/`flow_y` na posição back-traçada mais as duas
escritas de `vel` — trabalho irredutível, porque a física precisa do fluxo
*onde a partícula veio de*. *Uma ablação atribui a um BLOCO; atribuir a uma
LINHA dentro dele é inferência de segunda ordem, e o compilador tem opinião.*

**Revertido inteiro** — porta, oráculo congelado e gate. Um doc-comment que
justifica uma porta com 14,6 ms que ela não entrega é pior que porta nenhuma.
Fica a sonda (`measure_what_an_advect_is_made_of`) e este parágrafo.

### §5.44.3 O que sobra, e para quem

Depois de duas waves de CPU (§5.43 e esta), o passo na poça pesada é:

| passe | Flow 4 | do passo |
|---|---:|---:|
| `advect` | 34,4 ms | **70,4%** |
| `drying_pass` (÷3) | 10,7 | 21,9% |
| `rebuild_active_region` (÷2) | 2,6 | 5,2% |
| `build_flow_field` (÷4) | 0,8 | 1,7% |
| `project` + `smooth_velocity` | 0,3 | 0,7% |

O `advect` é gather **e** scatter: por célula ele lê 4 `susp` + 4 `susp_rgb`
(12 B cada) + 4 `film` de duas linhas, e **subtrai** nos quatro cantos-fonte —
o mecanismo que o ADR-0146 nomeia e que torna qualquer reordenação
não-byte-idêntica. As duas metades que a ablação separa (momento 14,6 ms em
`rf = 1` · pigmento ~16 ms) são as duas coisas que ele **é**.

⇒ **A alavanca de CPU deste módulo está esgotada.** O que resta é o que o
ADR-0146 já descreve como *um segundo modelo, não o mesmo mais rápido*.

## §5.45 — O SOLVER FICOU INDEPENDENTE DE ORDEM: 52,1 → 11,0 ms/passo, e a água a 90,8 Hz

**Ordem do Enio: *"GPU do Wet Paint"***. O [ADR-0146](../../../architecture/decisions/0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md)
tinha medido que `advect` (70,4%) + `drying_pass` (21,9%) somam ~92% do passo, que os dois são
Gauss-Seidel, e que portá-los seria *"um segundo modelo, não o mesmo mais rápido"*.

⚠️ **A pergunta que faltava, e que reescreveu a wave inteira:** aquele ADR trata a reformulação
como **o preço da GPU**. Ela é o preço do **PARALELISMO** — e a máquina tem **32 núcleos**, com o
`advect` (70% do passo) rodando em **um**.

### §5.45.1 — O argumento NÃO é velocidade: é a simetria da cena

Antes de qualquer relógio, a pergunta certa é *qual dos dois está certo?*. O Gauss-Seidel varre em
ordem de raster e lê o vizinho que a célula anterior já reescreveu — isso não é física, é a direção
do laço, e tem assinatura. Numa folha **espelhada** (massa simétrica, fluxo antissimétrico, cena
cuja física é simétrica por construção — `tests/solver_symmetry.rs`):

| passe | Gauss-Seidel | independente de ordem |
|---|---:|---:|
| `advect` | **1189,29** unidades de massa de viés | **0,000000** |
| `drying_pass` | **554,82** | **0,000000** |

O viés do advect é **mais que uma célula cheia de pigmento** (o platô da fixture é ~900), deslocada
só porque o laço anda da esquerda para a direita. ⚠️ **E o CONTROLE pegou meu próprio erro na
primeira corrida:** as "dentes" da fixture usavam `x % 7`, que **não é simétrico** sob `x → W+1−x`
— erro de espelho 484,7 numa fixture que eu tinha declarado simétrica. Elas medem a **distância ao
eixo** agora.

### §5.45.2 — Como um scatter vira um gather sem atômicos

O destino `d` puxa a fração `w_k` de cada canto; escrito assim é escrita em célula alheia. Mas a
relação é **simétrica**: se `d` puxa `w` de `c`, então `c` **dá** `w` a `d`. Logo

```text
  novo[c] = velho[c] · (1 − saída[c])  +  Σ_k w_k · velho[canto_k]
```

e `saída[c] = Σ_d w(d→c)` é ela própria um gather, porque `|u| ≤ maxVelocity` torna a vizinhança
local. ⚠️ **A saída pode passar de 1** onde o fluxo converge, e a cura é uma escala em que **quem
recebe pergunta a escala da FONTE**: um clamp só no lado de quem sai CRIARIA massa, um clamp só no
lado de quem entra a DESTRUIRIA. Medido: a massa total da poça do produto fica **806983136,5 nos
dois modelos**, idêntica à primeira decimal em ~8×10⁸.

⚠️ **E um off-by-one aqui CRIA massa.** O alcance de um destino não é `ceil(maxVelocity)` e sim
**`ceil(M) + 1`**: com o **default** `M = 1` e `u = −1`, o destino em `x` puxa dos cantos `x+1` e
**`x+2`**, logo `c` é alcançada por um destino em `c−2`. Eu escrevi o raio errado primeiro.

### §5.45.3 — Duas reescritas medidas, e a primeira foi um brinquedo

O gather serial nasceu **5× mais caro que o Gauss-Seidel** (180,8 contra 36,0 ms a `Flow 4`),
porque a vizinhança re-amostrava a grade de fluxo **nove vezes por célula**. Duas correções, cada
uma medida:

1. **materializar o fluxo fino uma vez** (`SolverScratch::uv`) ⇒ o vizinho vira **carga**: 180,8 →
   84,1 ms;
2. **virar o laço do avesso** — em vez de cada célula varrer `(2·alcance+1)² = 25` retro-traços, cada
   **DESTINO** deposita os seus dois pesos num acumulador que é a própria fatia `&mut [f32]` da
   linha (scatter **privado da linha**, seguro sob rayon): 84,1 → **68,1 ms**, e paralelo **6,85**.

### §5.45.4 — O que a troca de modelo custa

**(a) O fingerprint se move — e o protocolo do doc 23 é honrado.** O pino ANTIGO virou um gate
executável na rota `Sim::order_invariant = false`
(`the_gauss_seidel_route_still_reproduces_its_own_pin`). ⚠️ **É esse gate que torna a troca
auditável em vez de um número que mudou:** ele prova que nem a secagem, nem o fluxo, nem a projeção,
nem o depósito, nem o `lift_settled` mudaram — e prova também que extrair o `dry_cell` (a
aritmética de uma célula, agora a **porta única das duas rotas**) foi *pure code motion*.

**(b) O escorrido corre ~18% menos.** Medido pelo deslocamento do **centroide de massa** do filme,
varrendo `Flow Grid` 1..8: **0,64–0,96×, média ~0,82×**, uniformemente, sem colapso. ⚠️ E **sem
viés de direção** — a hipótese óbvia (*"a varredura de cima para baixo cascateia com a gravidade"*)
foi **REFUTADA por medição**: os dois modelos correm igual para cima e para baixo (1,14× e 1,09×).
O knob **Gravity** cobre a diferença; o desenho é do smoke.

**(c) +25 B por célula do fluido**, alocados preguiçosamente no 1º passo. A 4096² grade 1:1 são
+420 MB, e o slider **Grid Size** é a resposta — como já era para os 43 B/célula de antes.

### §5.45.5 — ⚠️ Um gate reprovou a wave por um motivo que não era o dela

O `the_water_still_runs_when_the_flow_is_coarse` ficou **VERMELHO** (rf 2 carregou 9, o controle
21). A tentação é afrouxar a barra; a medição diz outra coisa. Ele media a **célula mais extrema
acima de um limiar** — uma estatística de UM valor, e caótica na razão de fluxo: varrendo `rf` 1..8
o **mesmo** motor devolvia **27, 23, 36, 18, 10, 14, 21** (3,6× de amplitude *dentro do mesmo
modelo*, com o rf=3 **acima** do controle).

Pelo **centroide de massa** a mesma varredura é lisa (**20,1 · 12,0 · 22,3 · 13,5 · 13,7 · 14,3**)
e a queda em `rf = 2` aparece **igual nos dois modelos** — **0,60** no Gauss-Seidel contra **0,64**
no independente de ordem. Ou seja: **ela é da GRADE DE FLUXO, não do solver.** A frente amplificava
um 0,6 compartilhado em 0,85 contra 0,43.

⇒ **o ORÁCULO foi corrigido, não a barra** (e a prova de que é honesto: o gate passa nos **dois**
modelos). Descartar o limiar como causa também foi medido — a 0,05 / 0,02 / 0,005 / 0,001 o poço em
rf=2 é o mesmo (9, 9, 10, 10).

### §5.45.6 — O ganho, pela porta do PRODUTO

A/B no **mesmo processo e na mesma poça**, `on_canvas_pointer` a 4096², ciclo de 12 passos:

| `Flow Grid` | Gauss-Seidel | independente de ordem | |
|---|---:|---:|---|
| 1 | 60,19 ms (16,6 Hz) | **29,29 ms (34,1 Hz)** | **2,06×** |
| 4 | 52,05 ms (19,2 Hz) | **11,02 ms (90,8 Hz)** | **4,72×** |

⇒ **a água sai do regime work-limited**: a `Flow Grid 4` ela corre a **2,3× o nominal de 40 Hz da
SPEC**, e o teto passa a ser a SPEC, não a máquina.

### §5.45.7 — E o que sobra para a GPU

O ADR-0146 chamava o port de **all-or-nothing sobre `advect` + `drying_pass`** porque os dois
exigiam um modelo diferente. **Esse modelo agora existe, roda em produção e está provado contra a
referência** — então a Fase 3 daquele ADR está feita para dois dos três passes, **na CPU, onde é
debugável**. O que resta é o que nunca foi sobre o solver: o **stamp** (a silhueta do Painter por
closure) e a **residência** dos 14 planos. E o gatilho mudou de número: o ganho que a GPU ainda
compra tem de ser medido contra **11 ms**, não contra 52.

---

## §5.46 — O CAMPO DE FLUXO SAIU DO GAUSS-SEIDEL: 64,1 → 4,2 ms, o passo 1,87×, e o fingerprint INTACTO

O ADR-0147 tirou dois dos três passes sequenciais do caminho quente. **Este é o
terceiro** — e o retrato que o abriu só apareceu porque o INSTRUMENTO foi
consertado antes de qualquer número ser acreditado.

### §5.46.1 — A sonda media a rota errada

O `sim_step_stage` escolhe `advect_jacobi`/`drying_pass_jacobi` sob
`order_invariant` desde a §5.45, mas a
`measure_what_a_step_of_the_products_puddle_is_made_of` continuava chamando os
irmãos **CONGELADOS**. É a lição da §5.11 (*sonda que re-implementa o laço fica
cega à porta*) na sua forma mais barata de cometer: **chamar a função de nome
parecido**. Roteada pelo flag, a decomposição mudou de dono:

```text
    passe                     cru(ms)  cadencia  amort(ms)   % do passo
    build_flow_field           64.101    0.250    16.025    54.4%   <- o dono
    advect                      8.309    1.000     8.309    28.2%
    rebuild_active_region       5.392    0.500     2.696     9.1%
```

E a sonda ganhou **a coluna da CADÊNCIA** (o `build` roda ÷4) e **o passo
MEDIDO ao lado da soma amortizada**. Essa última linha pagou-se três vezes na
mesma sessão: ela acusou o `ensure` de 141 MB dentro do relógio da sonda do
advect (99,8 ms contra 31,7 de soma), acusou uma corrida de máquina ruim (8×
todos os passes), e acusou o A/B com a rota forçada à mão (−52,5%).
*Uma tabela por-passe que ninguém reconcilia com um passo medido é aritmética
sem testemunha.*

### §5.46.2 — Os dois mecanismos, e a decomposição que os dissolve

A §2 do ADR-0145 recusou este passe com dois motivos, e os dois estavam certos:

1. **o freio LÊ `wet[probe]`** alguns pixels adiante, e o carimbo de umidade
   **deste mesmo passe** pode ter escrito aquela célula;
2. **o backrun ESPALHA** em `susp[nb]`/`sett[nb]`/`susp_rgb[nb]` — escritas em
   outras linhas.

A cura não é agendamento, é **decomposição**. Os dois efeitos viram passes
próprios e o que sobra — o núcleo, que a F1 do plano 30 mediu em **99,4% do
custo** — fica um **gather puro que escreve `flow_x`/`flow_y` e mais nada**:

| passe | lê | escreve |
|---|---|---|
| `flow_rows` | film, paper, vel, active, wet, susp, sett, bloom | `flow_x`, `flow_y` |
| `wet_stamp_rows` | film, paper, active | `wet` |
| `backrun_rows` | film, active, `bflags` | `bloom`, `susp`, `sett`, `susp_rgb` |

**A ORDEM é a lei**, não conveniência: o carimbo corre DEPOIS do núcleo (todo
freio lê o `wet` de antes do passo) e o backrun por último (o portão capilar e
o teste `sett[nb] > 0` também leem o estado de entrada).

### §5.46.3 — O backrun é *pure code motion*, e a razão é aritmética

O levante — `lift = sett·0,1`, `susp += lift`, `sett −= lift`, cor por
`mix(…, lift/(susp+lift))` — é função **só do estado da célula levantada**. O
vizinho que o dispara não entra em nenhum termo. Logo aplicá-lo `n` vezes é
`F^n`, e **`F^n` independe da ordem em que os `n` gatilhos foram descobertos**:
o gather só precisa CONTAR. Daí o gate
`the_backrun_lift_lands_on_the_same_pigment_as_the_serial`, que compara
`susp`/`sett`/`susp_rgb`/`bloom` entre os dois modelos e sai **byte-idêntico**.

⚠️ **O `bflags` existe por isso e só por isso:** contar exige perguntar
`sett[vizinho] > 0` e `bloom[vizinho] < 6`, dois planos que o passe escreve. Os
dois predicados são **invariantes sob o passe** (o levante multiplica `sett`
por 0,9, que preserva o sinal; `bloom` só é escrito pela própria célula), então
materializá-los é o mesmo bit computado uma vez — **e só é alocado com o knob
LIGADO** (`extBackrun` é `Hidden` e nasce em `0.0`).

### §5.46.4 — ⚠️ O ACHADO: no ponto de operação que shipa, os dois modelos são o MESMO

Eu esperava uma troca de modelo, como nos dois irmãos. **Não é**, e a razão é
aritmética: o carimbo só escreve onde `film > 3`, e o freio de quem sonda
aquela célula vale `clamp(film + 3·wet/255 − brake, 0.05, 1)`. Com o `brake`
**default de 1,5**, já `film − 1,5 > 1,5 > 1` ⇒ **satura em 1,0 e o termo de
umidade não entra**. Dito de outro modo: *a única célula cujo `wet` este passe
pode mudar é uma célula funda demais para que o `wet` dela importe.*

O gate `at_the_shipping_knobs_the_two_models_are_the_same_to_the_byte` pina
isso — e é ele que **mantém o fingerprint do ADR-0134 INTACTO** (3/3 pinos
verdes). A wave é reescrita de laço quente no ponto de operação do produto, e
vira mudança de modelo só onde o irmão mede: com `brake = 4` a saturação some
para o filme na faixa `(3, 5)`, e ali o Gauss-Seidel **quebra a simetria da
cena** (viés de espelho `> 1e-3` contra **0,000000** do independente de ordem).

### §5.46.5 — ⚠️ O piso do pool que eu escolhi por RACIOCÍNIO estava errado

Escrevi `MIN_CELLS_FLOW = 96 << 10` argumentando *"a célula é cara, logo o pool
se paga antes"*. A premissa estava certa — ele rende mais que os três do
ADR-0145 em toda janela grande — e a **conclusão não**:

```text
    celulas     flow
     60_000     0,89x
    122_952     0,92x   <- ainda PREJUIZO, e o piso baixo mandava para ca
    194_788     3,65x
    411_166     5,75x
  2_546_830     7,88x
  9_800_850    10,19x
```

O joelho está entre 123k e 195k ⇒ **`160 << 10`**. *Um piso é medição; a única
coisa que o raciocínio produz é a hipótese.*

### §5.46.6 — O ganho, pela porta do PRODUTO

A/B no **mesmo binário**, mesma poça (`heavy_puddle`, 4096²), ciclo de cadência
de 12 passos, com a rota do `sim_step_stage` trocada à mão e devolvida:

| | ms/passo | Hz |
|---|---|---|
| Gauss-Seidel | **30,11** | 33,2 |
| independente de ordem | **16,1** | **62,0** |

**1,87× no passo**, e o passe sozinho **64,10 → 4,18 ms (15,3×)**. A soma
amortizada reconcilia com o passo medido dentro de **−1,6 %**.

⚠️ **E o A/B mudou de forma no fim da sessão, porque a MÁQUINA é compartilhada.**
Esta worktree divide 32 núcleos com outras linhas e com o app do Enio rodando um
smoke; medido ao longo da sessão, **o MESMO passo do produto foi de 14,5 a 30,2
ms sem uma linha de código mudar**, e um A/B cross-run teria atribuído isso ao
ganho — a deriva de 36% que a §5.39 já documentou, agora com 2× de amplitude.
A cura é a mesma que aquela seção aplicou: **as duas rotas medidas
costas-com-costas DENTRO da corrida**, sobre o MESMO estado restaurado, o que
torna a carga um fator comum que levanta os dois juntos. A sonda passou a
imprimir isso, e o número reproduz:

```text
  corrida   gauss-seidel   ordem-invariante   razao   passo medido   reconcilia
     1         66,652 ms         4,133 ms     16,13x     14,503 ms      -1,1%
     2         64,518 ms         4,035 ms     15,99x     16,794 ms      +8,4%
     3         66,240 ms         4,436 ms     14,93x     15,894 ms      -0,9%
     4         66,315 ms         4,465 ms     14,85x     15,926 ms      +0,0%
```

A aritmética da cadência fecha por uma **terceira** via independente: `0,25 ×
(66,65 − 4,13) = 15,63 ms`, logo o passo previsto na rota antiga é **29,98 ms**
— contra os **30,11 e 30,21** que as duas corridas com a rota forçada à mão
mediram. *Três testemunhas concordando é o que separa um ganho de um número.*

### §5.46.7 — Quem é o dono agora, e o que a medição diz sobre ele

O `advect` passou a ser **57,9% do passo**, e a decomposição por sub-passe
(sonda nova `measure_what_an_advect_is_made_of_by_sub_pass`, filho
`#[cfg(test)]` porque os cinco sub-passes são privados) diz onde ele está:

```text
    momentum_rows    16,3%   prepare_rows 15,5%   outflow_rows  7,5%
    transport_rows   36,8%   commit_rows  23,9%          (soma reconcilia: 5,33 vs 5,47)
```

⚠️ **O gather 5×5 — a peça que eu ia atacar — é 7,5% do advect**, ou 4,3% do
passo. Quem custa são `transport` e `commit`, que são **largura de banda sobre
a faixa viva** (o segundo buffer que um passo de Jacobi exige, 20 B por
célula). E a sonda expõe o número que decide a próxima wave: a **faixa viva é
36,4% da bbox e só 5,2% dela está ATIVA** — a granularidade do span é o que
sobra, e é wave própria.

⛔ **MEDIDO E NÃO FEITO — o alcance do gather NÃO pode ser apertado por
raciocínio.** Com `maxVelocity = 1` o intervalo exato dos destinos é
`d − c ∈ [−2, +1]` (4 por eixo, 16 em 2D) contra os `±2` (25) que o código
percorre — 36% de vizinhança a menos. **Mas o limite se apoia em `|u| ≤ max_v`
valer EXATAMENTE**, e em `rf > 1` a amostragem bilinear soma quatro produtos
cujos pesos podem não somar 1,0 ao ulp: um `u = 1 + ε` põe um destino em
`d − c = +2`, fora do intervalo apertado, e a massa **some sem ninguém ver**.
Em `rf = 1` o sampler devolve o valor VERBATIM e o limite seria seguro — mas um
bound correto por acidente de configuração é a forma exata de bug que este
módulo passou a jornada inteira caçando.

### §5.46.8 — ⚠️ A CENA PESADA SAIU DO REGIME WORK-LIMITED — e é isso que fecha a frente

A pergunta *"ainda há o que otimizar?"* não se responde com um milissegundo,
se responde com a **partição do worker** (`busy` / `away` / `sleep`), que é o
único instrumento que distingue *"a água é lenta porque falta CPU"* de *"a água
está no ritmo dela"*. Contra os números da §5.40, na MESMA sonda:

| cena | antes | agora |
|---|---|---|
| um traço | `busy 49,5% · away 7,0% · sleep 43,4%` → 38,4 Hz | `busy 16,8% · away 3,8% · sleep 79,2%` → **38,9 Hz** |
| três traços sobrepostos | `busy 76,8% · away 19,2% · sleep 1,6%` → **14,0 Hz** | `busy 57,7% · away 19,7% · sleep 22,7%` → **36,7 Hz** |

A cena de três traços — **a do smoke do Enio**, e a que a §5.40 nomeou como
*work-limited* — passou a ter **22,7% de folga** e corre a 36,7 dos 40 Hz
nominais da SPEC. **2,6× no nível do produto.**

⇒ **Otimizar mais a CPU deste módulo não compra nada VISÍVEL no ponto de
operação default:** o worker apenas dormiria mais. O que resta de frente é o
que sai do default — o **K–M** do Tuning (a §5.41 mediu 3,2× o passo, então
com o passo em ~16 ms ele volta ao regime limitado por trabalho) — e o que não
é sobre o solver.

### §5.46.9 — ⚠️ NOTA RECONFERIDA: o `Flow Grid` não compra mais velocidade

A §5.42 entregou o slider medindo **1,27× a 4:1**, e a nota dizia que a
entrega dele era *"a BORDA FINA com o fluxo barato"*. Quem move o número que
justificava uma nota tem de reconferi-la (CLAUDE.md §0), e este passe moveu:
o `build_flow_field` saiu de 54,4% para **6,9%** do passo. Medido hoje, mesma
sonda:

```text
  rf   ms/passo (media do ciclo)   razao
  1                   6.123 ms    1.00x
  2                   6.110 ms    1.00x
  4                   6.196 ms    0.99x
  8                   5.744 ms    1.07x
```

**Zero.** O racional de VELOCIDADE do slider acabou — e ele não some por isso:
o que ele ainda faz é mudar o **LOOK** (movimento mais liso/em blocos, backrun
esparso), que era metade do desenho desde o começo. Mas o slider deixou de ser
um controle de performance, e quem o oferecer como tal estará vendendo um
número que a medição não sustenta.

### §5.46.10 — ⛔ MEDIDO E REJEITADO — não refaça: materializar o K/S do K–M

Com o passo em ~16 ms, o **K–M** do Tuning é o único regime que continua
limitado por trabalho: medido a 4096², razão 1, ele custa **4,75× o passo
(14,3 → 67,9 ms, 14,7 Hz)**.

A causa é exatamente a que o doc 24 nomeou e que ele reduziu, mas não removeu:
`km_weighted_mean_color` converte **as QUATRO cores-fonte por DESTINO** — 12
consultas forward + 3 inversas = **15 por célula advectada** — e uma célula é
canto de até quatro destinos, logo **cada cor é convertida ~4× por passe**.

O remédio é o mesmo padrão do `prepare_rows` (materialize uma vez, leia muitas):
um plano `ks_rgb` num pré-passe deixaria a média com 3 consultas em vez de 15,
**byte-idêntico por construção** (a transferência é função pura da cor, e a
ordem da soma não muda). Vale ~2,5× no custo do K–M.

**Não foi feito, e o que decide é a MEMÓRIA:** o valor tem de ser guardado em
`f64` — em `f32` a ida-e-volta arredonda e a identidade cai —, o que dá **24 B
por célula = 403 MB a 4096²**, sobre um grid que já paga 68 B/célula. Gastar
isso num knob **EXPERIMENTAL e `Hidden`** que já tem resposta sancionada — o
slider **Grid Size**, que a §5.41 mediu levando o K–M a 8,3 ms, abaixo do kill
de 12 — é o trade errado. ⚠️ E a alternativa compacta (um anel de 4 linhas por
thread) é **pior**, não melhor: com `par_chunks_mut` cada tarefa é UMA linha,
então o anel recomputaria 4 linhas por linha — 4× o trabalho que ele existe
para evitar.

Se um dia o K–M sair do EXPERIMENTAL, é aqui que a wave começa, e o número que
ela tem de bater é 403 MB.

### §5.46.11 — ⚠️ O LOG DO SMOKE ACHOU UM VAZAMENTO DE THREAD QUE NENHUM GATE VIA

O smoke de 2026-07-31 voltou com *"performance subjetivamente bem melhorada"* e
com esta linha:

```text
  worker: busy 69% away 31% sleep 909% | TAXA DA AGUA 392.8 Hz (779 passos em 2.0s)
```

Três baldes que dizem **partição** somando **1009%**, e uma sim **dez vezes**
acima dos 40 Hz nominais. Nenhuma das duas coisas é possível para **um** worker
— e as duas são exatamente o que **dez** produzem, cada um no seu ritmo
correto. *Um número impossível é mais informativo que um número ruim: ele
nomeia a classe da causa.*

**Duas causas, ambas reais, e a segunda foi achada por causa da primeira.**

**(a) A janela do diagnóstico era ASSUMIDA.** `span = frame_medio × 120` supõe
que 120 frames se passaram e que cada um durou a média — enquanto os contadores
do `wet_diag` acumulam em tempo REAL. Agora o span é **medido** por um
`Instant` que arma no primeiro frame e é trocado a cada dreno.

**(b) Uma sessão encerrada deixava uma THREAD simulando um motor órfão, para
sempre.** O doc do `SimWorker` afirmava *"a thread morre quando `to_worker` é
dropado (a sessão terminou)"*, e a afirmação é **falsa exatamente no caso que
importa**: com o motor COM ela, o `while let Ok(engine) = rx.recv()` só observa
o canal fechado no **TOPO** do laço externo, e o laço **INTERNO** só sai quando
`want` é setado — que é o que ninguém faz depois de a sessão morrer.

⚠️ **O preço não é só CPU:** cada worker vazado segura um `Box<Engine>` com os
quatorze planos dentro — a 4096² razão 1, da ordem de **um gigabyte por
sessão encerrada** —, e os nove rogues estavam disputando os mesmos núcleos com
o frame que o log mede. Parte do `painter-dispatch = 10,02 ms` e do
`tool-tick = 5,07 ms` daquele log é contenção com sims que não deviam existir.

O conserto é o `Drop`, e ele usa a **MESMA porta** que o tick usa para pedir o
motor de volta (`want`) — um sinal próprio de shutdown seria uma segunda porta
para *"largue o motor"*, e as duas divergiriam no dia em que uma ganhasse um
passo a mais.

⚠️ **A lição sobre a suíte:** nenhum dos 932 gates via isto, e não por
descuido — **todos usam uma sessão só**. O vazamento é sobre a SEGUNDA, e o
oráculo que o pega não é um relógio nem um pixel: é a **contagem forte do `Arc`
do pedido**, porque ela cair para 1 *é* a thread ter saído. Um `join` seria
melhor e não existe: a thread é solta de propósito.

⚠️ **E é a QUARTA vez nesta sessão que o instrumento erra antes do produto** (a
sonda chamando a rota congelada · o `ensure` dentro do relógio · o A/B
cross-run sob máquina compartilhada · a janela assumida). O padrão é sempre o
mesmo: *o número que não reconcilia é o que vale a pena perseguir*.

---

## §5.47 — TRÊS hipóteses minhas sobre a água lenta, medidas e REJEITADAS — e o instrumento que sobrou (2026-07-31)

O smoke seguinte veio com a partição do worker já sã (`busy + away + sleep ≈
100%`) e a taxa abaixo dos 40 Hz nominais — os dois consertos da §5.46.11
landaram. E trouxe o número novo:

```text
pintando:      sim media 45.52ms x35 | busy 80% away 23% sleep  2% | 17.7 Hz | painter-dispatch 6.90ms
so assistindo: sim media 20.11ms x70 | busy 71% away 20% sleep 10% | 35.0 Hz | painter-dispatch 0.03ms
so assistindo: sim media 21.01ms x67 | busy 70% away 22% sleep  8% | 33.5 Hz | painter-dispatch 0.05ms
```

⚠️ **O `busy` é praticamente o MESMO nas três janelas** (1593 · 1408 · 1407 ms
de 2000): o worker trabalha o mesmo tempo e entrega **metade** dos passos. Duas
leituras cabem nisso e elas pedem curas **opostas** — *a poça ficou maior*
(pintar acrescenta células molhadas; não há nada a consertar, e o slider `Grid
Size` já é a resposta) contra *a máquina ficou disputada* (o solver ficou
massivamente paralelo depois dos ADR-0145/0147, e a thread do frame trabalha
justamente enquanto o artista pinta). **O log não as distingue**, porque as duas
metades se movem juntas nele.

### §5.47.1 — BANDA contra NÚCLEO, e o CONTROLE que a 1ª versão não tinha

A medição que separa: **congelar a poça** (mesma cena, `snapshot_grid` /
`restore_grid` antes de cada amostra, ciclo de cadência de 12 passos) e
acrescentar carga por fora, um recurso de cada vez.

| carga | núcleos | banda | ms/passo | razão |
|---|---|---|---|---|
| controle | — | — | 15,923 | — |
| memcpy serial | 1 | 8,9 GB/s | 16,872 | **1,06×** |
| **ALU ×4 (controle)** | 4 | 0 | 17,048 | **1,07×** |
| **memcpy ×4** | 4 | 15,4 GB/s | 22,718 | **1,43×** |
| ALU ×32 | 32 | 0 | 170,530 | **10,71×** |
| memcpy ×32 | 32 | 25,9 GB/s | 233,075 | 14,64× |

⚠️ **A linha que decide é o CONTROLE, e a 1ª versão desta tabela não o tinha.**
Com só *memcpy serial* (1,06×) e *ALU ×32* (10,71×) a leitura óbvia — e errada —
é *"banda não custa nada"*. Quatro threads de cópia tomam **banda E quatro
núcleos**; quatro threads de ALU tomam **os mesmos quatro núcleos e zero
banda**. A diferença entre 1,43× e 1,07× é banda pura ⇒ **15,4 GB/s custam 34%
do passo**, e a frase do §5.46 (*o `advect` é limitado por LARGURA DE BANDA
sobre a faixa viva*) **sobrevive, agora com número ao lado**.

⚠️ **E `10,71×` é patológico, não é partilha:** 32 threads a mais em 32 núcleos
deveriam dar ~2× a cada lado. O fator que falta é a **BARREIRA** — um passe
row-parallel termina quando o ÚLTIMO chunk termina, então basta um worker
preemptado para segurar o passe inteiro, e um passo roda SETE passes.

### §5.47.2 — ⛔ MEDIDO E REJEITADO — não refaça: encolher o pool do rayon

Se a causa é amplificação de barreira, a cura óbvia não é *"mais rápido"*, é
*"menos sensível"*: um pool de `k < núcleos` quase nunca tem todos os workers
preemptados ao mesmo tempo, e deixa o resto da máquina para o frame. Construído
(`ThreadPool::install` em volta do ciclo) e medido:

| pool | sozinho | sob carga total |
|---|---|---|
| global (32) | 14,389 ms | 169,660 |
| 16 | 15,924 | 159,950 |
| 8 | 24,647 | 158,170 |
| 4 | 40,218 | 149,478 |

**O custo sob carga é ~150-170 ms em TODO tamanho de pool.** Nenhuma partição
protege contra saturação real: 32 spinners tomam todos os núcleos e qualquer
pool é faminto proporcionalmente. O preço, esse sim, é real e monotônico — a
coluna *sozinho* piora 2,8× de 32 para 4.

⚠️ **E a carga sintética é dura DEMAIS para decidir o produto:** 32 spinners
permanentes não é o que a thread do frame faz (6,90 ms num quadro de 16,6 =
**41% de duty**, e paralelo em rajadas). Na contenção que o frame de fato cria —
quatro núcleos, duty parcial — a tabela acima diz **1,07-1,43×**, não 2,2×.

### §5.47.3 — ⛔ MEDIDO E REJEITADO: a água publica um retângulo maior que os outros meios

O `painter-dispatch` é proporcional ao retângulo que o tool declara sujo (a
pista de GPU re-envia esse sub-rect), e ele mede **6,90 ms pintando contra 0,03
só assistindo**. O censo, pela porta do produto:

| meio | eventos | mediana px | pior px | full-canvas | telas/traço |
|---|---|---|---|---|---|
| Digital | 270 | 46.656 | 6.831.789 | 0 | 1,6 |
| Impasto | 270 | 92.416 | 7.887.516 | 0 | 2,3 |
| **WetPaint** | 180 | 15.625 | 16.777.216 | **1** (o publish `[0]`) | **1,9** |

**A água não é outlier** — ela fica ENTRE o Digital e o Impasto. O único
full-canvas é o **nascimento da sessão**, que é legítimo: não existe cache
anterior a remendar.

⚠️ **E a fixture precisou de TRÊS traços para conter a diferença.** Com um só, a
tabela dizia *pior 100% da tela, 1,1 telas/traço* — e *"uma vez por SESSÃO"* e
*"uma vez por TRAÇO"* são o mesmo número ali. Foi o índice do publish (`[0]` de
180) que respondeu, não a estatística.

### §5.47.4 — O que sobrou: o INSTRUMENTO, porque o impasse era de atribuição

Três hipóteses, três rejeições — e nenhuma delas era necessária: **o log não
tinha o divisor.** Agora tem.

O `[frame]` publica a **poça em milhões de células** e o **ns/célula** derivado.
A leitura é uma linha: *custo por célula CONSTANTE entre duas janelas = TRABALHO
(a poça cresceu); custo por célula SUBINDO = CONTENÇÃO*. A pergunta que consumiu
esta sessão passa a ser respondida por um smoke.

⚠️ **`Grid::live_span_cells` é `O(LINHAS)`, e é isso que a torna publicável a
cada passo:** a faixa viva de cada linha já está materializada em
`row_lo`/`row_hi` (o rebuild a escreve), então somá-las é uma varredura de 4096
inteiros — microssegundos contra os 14-45 ms de um passo. Contar `active` seria
`O(células)` e **pagaria o preço da pergunta**.

⚠️ **A faixa é o que os passes ANDAM, e ela não é a bbox.** Numa diagonal — a
forma de um traço de verdade — a bbox é um múltiplo dela, e um `ns/célula`
computado sobre a caixa mentiria para **baixo** exatamente quando a poça é
grande, que é quando alguém o lê. É o que o gate afirma, e a mutação que devolve
a bbox sangra.

⚠️ **O balde novo é drenado DENTRO do gate que já existe** (`the_worker_reports_-
what_a_step_costs`): ele é o único teste não-`#[ignore]` que consome a janela
global, e um gate irmão zeraria a dele — o verde viraria sorte.

### §5.47.5 — A leitura de produto que fecha a sessão

Com a poça congelada e a máquina livre, o passo custa **14,2-15,9 ms = 63-70 Hz**
sobre 1,61 M células vivas — **8,9 ns/célula**. O nominal da SPEC é 40 Hz (25
ms), o que dá um orçamento de **~2,8 M células a `Grid Size 1`**. Uma poça de
três traços sobrepostos num canvas de 4096² passa disso, e o slider é a resposta
que já existe (a razão corta células por `r²`).

⇒ **Otimizar mais o solver não compra nada visível**: sozinho ele já roda 1,6×
acima do nominal e o worker dorme. O que resta é quanto da máquina o **frame**
toma enquanto o artista pinta — e ali o item nomeado é o `painter-dispatch` de
6,90 ms, que **não é o retângulo** (§5.47.3) e vive na shell, fora do alcance de
uma sonda headless.

---

## §5.48 — O TRAÇO lento: a espiral REFUTADA, e a linha do log que não decidia nada (2026-07-31)

Report do Enio no smoke seguinte: *"o traço (deposição do pigmento antes do
mouse up) ficou muito mais lento. Já era lento e já precisava melhorar. Agora
piorou."*

```text
total=54.53ms (~18 fps) | present/acquire-stall=52.26ms | painter-dispatch=0.49ms
  tool-tick: media 0.11ms | stamps: media 105.82ms pico 531.42ms em 26/120
  agua: sim x0 | worker: busy 0% away 0% sleep 0% | poca: 0.00 M celulas
```

⚠️ **A água está inocente e o log prova**: `sim x0`, `worker 0%`, `poça 0,00 M`
— ela nem roda naquelas janelas. Quem custa é o **carimbo**: `stamps` mede 39 a
106 ms de média por quadro, com pico em **531 ms**.

### §5.48.1 — Duas medições antes de qualquer hipótese

**Não há regressão no custo por evento.** O censo dos quatro meios mede o move
do Wet Paint a **2,028 ms** a 4096², contra os **1,82 ms** que a §5.12 pinou —
mesma faixa, diferença de máquina.

⛔ **E a espiral está REFUTADA.** O comentário do `painter_canvas_move` descreve
o mecanismo (*mais eventos → quadro mais lento → mais eventos*) e isenta os
métodos incrementais do coalescing **de propósito**, porque cada evento deposita
dabs. A isenção só seria desonesta se o custo fosse **fixo por evento** — aí o
trabalho cresceria com a taxa de polling do mouse, e não com o traço, que é a
lei que esta linha já pagou cinco vezes.

O mesmo caminho de 640 px, entregue em passos diferentes:

| meio | passo | eventos | TOTAL | vs 40 px |
|---|---|---|---|---|
| Digital | 40 px | 16 | 17,62 ms | 1,00× |
| Digital | 1 px | 640 | 18,03 | **1,02×** |
| Impasto | 40 px | 16 | 29,06 | 1,00× |
| Impasto | 1 px | 640 | 29,95 | **1,03×** |
| WetPaint | 40 px | 16 | 27,85 | 1,00× |
| WetPaint | 1 px | 640 | 29,87 | **1,07×** |

**O total é constante.** O custo é por **DAB** — quarenta vezes mais eventos
sobre o mesmo caminho custam 2-7% a mais, não 40×. ⚠️ E a mediana **por evento**
cai a **0,000-0,004 ms** nos passos pequenos: a maioria dos eventos não emite
dab nenhum (abaixo do spacing) e não custa nada. *A taxa de polling não compra
trabalho.*

### §5.48.2 — O que sobrou: a linha do log não tem divisor

`stamps: media 105,82 ms` admite duas leituras, e elas pedem curas **opostas**:

- **UM** re-stamp de forma inteira a 105 ms ⇒ o alvo é o re-stamp;
- **CINQUENTA** entregas incrementais a 2 ms ⇒ o alvo é a taxa de entrega.

Com o custo por-evento medido em ~2 ms e o custo por-caminho constante, a
diferença entre as duas é **uma ordem de grandeza no número de eventos** — e a
média a escondia.

⚠️ **É a MESMA doença que o §5.47.4 acabara de curar do outro lado da mesma
linha do log**, um sistema adiante: *um custo sem divisor não é atribuível*. O
shell já contava as entregas (`paint_stamps_this_frame`) e **não as imprimia**.

Agora imprime: `({stamp_ev} entregas, {stamp_per:.2}ms cada)`. ⚠️ O divisor é
acumulado no **MESMO `if st > 0`** da soma — contado noutro lugar, os dois
baldes descreveriam janelas diferentes e a média por entrega seria de uma
amostra que ninguém escolheu.

### §5.48.3 — ⚠️ O gate nasceu VERDE-sobre-errado, por casar com a própria prosa

A 1ª versão do arch-gate ancorava na **frase** `"stamps: media"` — e casou com o
**doc-comment que esta mesma wave escreveu**, citando a linha do log para
explicar por que o divisor existe. O scanner leu a prosa e reprovou o código
correto.

**A âncora tem de ser algo que só o CÓDIGO tem** (aqui o placeholder
`{stamp_avg:`). *Um oráculo que casa com a documentação de si mesmo não está
olhando para o produto.*

### §5.48.4 — ✅ E o instrumento da §5.47 se pagou no MESMO log

A pergunta que a §5.47 não conseguiu responder — *trabalho ou contenção?* — foi
respondida pelo balde novo, no primeiro smoke em que ele apareceu:

| janela | poça | ns/célula |
|---|---|---|
| com carimbos (`stamps 10,00 ms x19`) | 0,70 M | **13,5** |
| sem carimbos | 0,76 M | **7,5** |
| sem carimbos | 0,87 M | 7,5 |
| sem carimbos | 1,06 M | 8,3 |

**Mesma poça, custo por célula 1,8×.** É **CONTENÇÃO**, medida — e ela aparece
exatamente enquanto o artista carimba, que é o regime que a §5.47.1 mediu em
1,07-1,43× com carga sintética.

⚠️ **E a água, quando o carimbo para, está EXATAMENTE onde a wave a deixou:**
`40,0 Hz` com `sleep 56-71%` e `busy 23-35%` sobre 0,76-1,06 M células. O
regime work-limited acabou; o que sobra no traço é o carimbo, e o divisor novo é
quem diz de que ele é feito.

---

## §5.49 — O log CALMO, e o que ele fecha (2026-07-31)

Com `load average 1,86` o mesmo build dá:

```text
poca: 2,16 -> 2,29 M celulas | 6,8-6,9 ns/celula  (CONSTANTE em quatro janelas)
worker: busy 59-61% away 12-16% sleep 23-29% | TAXA DA AGUA 38,6-40,1 Hz
total 16,42-17,26 ms (~58-61 fps) | GPU 1,2 ms
```

**A água está no nominal com folga**, e o `ns/célula` **constante** enquanto a
poça cresce 6% diz **TRABALHO** — o instrumento da §5.47 lendo exatamente o que
foi construído para ler.

⚠️ **E é o contraste que fecha os relatos de "tudo lento":** o log da rodada
anterior, `load average 74` em 32 núcleos (seis `rustc` de outras linhas a
300-600% cada), media **130-200 ns/célula** sobre uma poça do MESMO tamanho —
**17-27×**. A prova de que era a máquina e não o código é a linha *controle* da
tabela do §5.47.1: **mesmo binário, mesma fixture, 14,240 → 46,633 ms/passo sem
uma linha mudar**, e a tabela saindo incoerente (`memcpy ×4` a **0,61×**, mais
*rápido* que o controle) porque a carga oscilava entre amostras.

⚠️ **Corolário operacional:** *nenhum smoke desta máquina significa nada com o
load acima de ~5*, e a linha `poca:` é o detector — **um dígito de `ns/célula` =
máquina sã; três dígitos = o log não fala sobre o código.**

### §5.49.1 — E o instrumento foi exonerado por uma RAZÃO, não por uma opinião

Sob máquina saturada a suspeita cai sobre a última coisa que mudou, e **nenhum
número absoluto a defende**. O que sobrevive é uma razão medida na MESMA
corrida — as duas metades sobem juntas:

| | |
|---|---|
| régua (`live_span_cells`) | 0,0009 ms |
| passo | 9,4436 ms |
| **a régua vale** | **0,0096% do passo** |

Um décimo de milésimo, e continua verdade amanhã com a máquina noutro estado.

### §5.49.2 — O carimbo, medido com a SIM RODANDO (o defeito de fixture do censo)

⚠️ **O censo dos quatro meios mede o carimbo com o motor PARADO** — ele nunca
chama o tick, então o worker nunca recebe o engine e o `bring_home()`
**bloqueante** de cada evento nunca bloqueia. No produto ele bloqueia até a
**fronteira de estágio**, e a `ESPERA` medida é de 2,3-2,6 ms.

⚠️ **E a 1ª versão da sonda nova também não continha o fenômeno:** sem VÃO entre
a entrega e o carimbo seguinte, o worker não chega a ENTRAR num estágio e a
razão mede **0,99×** — *o handshake é grátis*, conclusão errada. Com o vão de um
quadro:

| raio | parado | simulando | razão | por evento |
|---|---|---|---|---|
| 60 px | 24,97 ms | 33,42 | **1,34×** | 0,557 ms |
| 100 px | 38,84 | 46,88 | **1,21×** | 0,781 |
| 200 px | 63,05 | 69,05 | **1,10×** | 1,151 |

Pior no pincel **PEQUENO**: o handshake é custo fixo, e um pincel pequeno tem
menos trabalho para amortizá-lo.

### §5.49.3 — A invariância de caminho SOBREVIVE — e o porquê é a resposta

O handshake é custo por **EVENTO**, e a tabela do §5.48.1 foi medida com o motor
parado — ela **não podia ver** a diferença que existe para procurar. Re-medida
com a sim rodando (um tick por quadro, como o produto):

| meio | 16 eventos | 640 eventos | razão |
|---|---|---|---|
| Digital | 17,70 ms | 17,99 | 1,02× |
| Impasto | 29,45 | 31,78 | 1,08× |
| WetPaint | 28,16 | 29,24 | **1,04×** |

**Sobrevive.** ⚠️ **E o mecanismo é o que decide:** o tick dispara **uma vez por
QUADRO**, então o `bring_home` bloqueia no **primeiro** evento depois de cada
tick e é **no-op** nos demais — *o handshake é por quadro, nunca por evento*.
Quarenta vezes mais eventos pagam o mesmo handshake.

⇒ **Com a máquina calma não há patologia no traço.** O custo é 0,56-1,15 ms por
evento conforme o raio, invariante ao número de entregas, com ~1,1-1,3× de
handshake amortizado por quadro. Os relatos de *"o traço ficou muito mais
lento"* saem de logs com `load average` 74.

---

## §5.50 — O primeiro log de TRAÇO com a máquina sã, e o que ele nomeia (2026-07-31)

```text
poca: 1,58 -> 1,90 M celulas | 7,0-7,1 ns/celula   ← EM TODAS as janelas
stamps: media  58,92ms em 42/120 (315 entregas,  7,86ms cada)
stamps: media 130,89ms em  4/120 ( 49 entregas, 10,69ms cada)
worker: busy 39% away 161% sleep 29%              ← soma 229%
```

⚠️ **A contenção está DESCARTADA por medição**: o `ns/célula` fica em 7,0-7,1
*inclusive nas janelas que carimbam*. Então **7,86-10,69 ms por entrega é
real** — contra os **0,78 ms** que a sonda irmã mede a raio 100 com a sim
rodando. **10-14×, e não é a máquina.**

### §5.50.1 — ⛔ MEDIDO E REJEITADO: a poça já existente não é o ingrediente

A hipótese natural — *um dab que cai em água tem outro trabalho pela frente* —
foi construída e medida: o MESMO traço numa folha limpa e por cima da
`heavy_puddle` (1,6 M células vivas), na mesma corrida.

| raio | folha limpa | sobre a poça | razão | por entrega |
|---|---|---|---|---|
| 100 px | 77,07 ms | 74,43 | **0,97×** | 1,861 ms |
| 200 px | 136,69 | 133,72 | **0,98×** | 3,343 |
| 300 px | 176,29 | 174,93 | **0,99×** | 4,373 |

**A água que já está lá não custa nada.** O que a tabela mostra é o **PINCEL**:
1,86 → 4,37 ms por entrega de raio 100 a 300, e o produto a 7,86-10,69 é
consistente com um pincel maior e/ou uma mão mais rápida. ⚠️ E a escala é
**sub-linear no raio** (1 : 1,8 : 2,3 contra 1 : 4 : 9 de uma pegada), o que
diz que o custo **não é a área** — provável assinatura do `TRAIL_HALF = 61`, que
clipa pincel grande. **Decompor isso é uma wave própria**, e ela tem alvo: o
depósito de um dab, não a água.

### §5.50.2 — E o `away 161%` era o instrumento, pela TERCEIRA vez

Três baldes que dizem PARTIÇÃO somando **229%**. ⚠️ **Um intervalo aberto
pertence à janela em que ele está ABERTO, não àquela em que fecha** — o `away`
era creditado inteiro no `recv`, então uma retenção de vários segundos (a
rajada de carimbos segurando o motor) caía toda na janela onde o intervalo
terminou.

**É a mesma classe do `sleep 909%` da §5.46.11, por outra via**: lá a janela era
*assumida*, aqui o intervalo *atravessa* a janela. O worker passa a **publicar**
a abertura e o `take_worker` credita a parte aberta antes de drenar,
re-baseando; o CAS impede a contagem dupla.

⚠️ **Duas lições nos meus próprios gates:**

- **UM teste, não dois** — os baldes são globais e a 1ª versão eram dois gates
  que **se drenaram mutuamente**, com um doc-comment meu afirmando que eles
  eram indiferentes a um vizinho. É a lição que o
  `the_worker_reports_what_a_step_costs` já pregava, violada um arquivo adiante.
- **A mutação do CAS NÃO sangra**, e fica **documentada em vez de gateada**: a
  contagem dupla só existe sob corrida real com o worker, que um teste
  single-threaded não produz (precedente do ADR-0145 §gates). Escrevê-lo é o
  que impede a próxima pessoa de "simplificar" o CAS achando que a suíte verde
  a autoriza.

⇒ **O placar do instrumento nesta sessão: três defeitos, todos achados por um
log do produto e nenhum por uma suspeita minha** (a janela assumida · o divisor
ausente do carimbo · o intervalo que atravessa a janela). *Um instrumento errado
não é neutro: ele responde com confiança à pergunta errada.*

---

## §5.51 — O teto de raio do MODELO de referência era o teto do PRODUTO (2026-08-01)

Report do Enio: *"todos esses testes tenho feito com raio 300, mas na prática o
app limita o tamanho para aproximadamente 200"*.

⚠️ **Medido pela porta do artista ANTES de tocar em código — e é pior que o
reportado:**

| meio | pedido | largura real | raio efetivo | razão |
|---|---|---|---|---|
| Digital | 100 → 400 px | 190 → 760 | 95 → 380 | **0,95×** em toda a faixa |
| **WetPaint** | 100 px | 119 | 59,5 | 0,59× |
| **WetPaint** | 200 px | **119** | 59,5 | **0,30×** |
| **WetPaint** | 300 px | **119** | 59,5 | **0,20×** |
| **WetPaint** | 400 px | **119** | 59,5 | **0,15×** |

O slider promete `BRUSH_SIZE_MAX_PX = 512` e o traço **saturava em 119 px de
largura** a partir do raio 100 — e **em SILÊNCIO**: nada na ferramenta dizia
que o pincel tinha parado de crescer.

**A causa:** `TRAIL_HALF = 61 // ceil(35 + 4*6) + 2`. O **35 é o teto de raio do
modelo JS de referência**, e ele era o teto deste produto. ⚠️ É a forma exata
que o **CLAUDE.md §0** nomeia — *nunca deixe o fallback definir o produto* — e o
§0 também diz o que fazer: **medir, e escrever o número que a medição deu**.

### §5.51.1 — A janela é função do PINCEL, e duas propriedades decidem o desenho

`Trail::fit_to(radius)` aplica a MESMA lei do `TRAIL_HALF` com o raio do pincel
no lugar do 35, e o `TRAIL_HALF` vira o **PISO**.

- ⚠️ **Cresce só com `dab_count == 0`**, onde a janela está vazia por construção
  (o `start_stroke` e todo transfer a zeram). Crescer no meio de uma janela
  invalidaria as coordenadas locais do que já está acumulado.
- ⚠️ **Só CRESCE, nunca encolhe.** Encolher exigiria decidir o que fazer com a
  tinta que já está lá, para devolver memória no meio de um traço — a troca
  errada. Um traço novo com pincel pequeno reusa a janela grande e paga só a
  varredura, que o `lx0..lx1` já limita ao que foi tocado.

⚠️ **E o PISO é o que mantém o fingerprint do ADR-0134 byte-idêntico POR
CONSTRUÇÃO, não por promessa:** um pincel dentro do teto do modelo produz a
janela EXATA de antes. **3/3 no fingerprint, 9/9 na aceitação.**

### §5.51.2 — O resultado, e o preço

| pedido | antes | depois | custo/entrega antes → depois |
|---|---|---|---|
| 100 px | 119 px | **153** | 1,861 → 2,343 ms |
| 200 px | 119 | **338** | 3,343 → 4,243 |
| 300 px | 119 | **514** | 4,373 → 5,747 |
| 400 px | 119 | **664** | — |

**+26-31% de custo por um pincel 2,8× mais largo no raio 200** (8× a área). A
razão do WetPaint fica em 0,77-0,86× contra os 0,95 do Digital — a diferença é
a **silhueta de cerdas** ser mais macia no aro, não um cap: a curva é monotônica.

⚠️ **Memória, nomeada:** o trail são 6 planos de `f32` em `size²`, alocados
**lazy** por lane. Raio 400 com o spacing de produto dá `half ≈ 482` ⇒ ~22 MB
por lane; com Symmetry a 8 lanes isso multiplica, e o `Grid Size` continua sendo
a resposta que já existe.

### §5.51.3 — O gate não pode espelhar a regra que ele julga

O oráculo é o **RETÂNGULO TOCADO** (`touched_extent_for_measure`), que não
conhece constante nenhuma — um gate que comparasse `half` com a fórmula estaria
verificando aritmética, não comportamento. A metade oposta (o piso) usa
`window_half_for_measure`, e é ela que prova a byte-identidade.

**A mutação (o cap de volta) sangra o primeiro e deixa o segundo VERDE** — que é
exatamente o par certo: um gate que morresse nos dois estaria medindo a mesma
coisa duas vezes.

---

## §5.52 — A "wave com alvo" do §5.50 DISSOLVEU na medição, e o cap escondia um segundo defeito (2026-08-01)

A §5.50 fechou nomeando o próximo alvo: *"o carimbo custa 1,86 / 3,34 / 4,37 ms
por entrega nos raios 100 / 200 / 300, e a escala é **sub-linear no raio**
(1 : 1,8 : 2,3 contra 1 : 4 : 9 de uma pegada), provável assinatura do
`TRAIL_HALF = 61` que clipa pincel grande. **Decompor o depósito de um dab é uma
wave própria, e agora ela tem alvo.**"

A wave seguinte (§5.51) **removeu esse cap**. O CLAUDE.md §0 é explícito sobre o
que isso obriga — *quem move o número que tornava algo inalcançável tem de
reconferir a nota* —, e a reconferência derrubou a nota inteira.

### 1. O depósito é limitado pela PEGADA, e é plano

`ph2d-wet-paint/tests/measure_dab_halves.rs`, 4096², 24 dabs, espaçamento do
produto (0,025 do diâmetro):

| raio | janela | accum ms | transf ms | total ms | **ns/r²** | transf % |
|---|---|---|---|---|---|---|
| 60 | 149 | 1,916 | 0,645 | 2,561 | **29,64** | 25,2% |
| 100 | 245 | 5,209 | 1,856 | 7,065 | **29,44** | 26,3% |
| 200 | 485 | 20,550 | 7,153 | 27,703 | **28,86** | 25,8% |
| 300 | 725 | 46,806 | 18,218 | 65,024 | **30,10** | 28,0% |
| 400 | 965 | 85,540 | 31,991 | 117,531 | **30,61** | 27,2% |

**`ns/r²` é PLANO: 1,03× sobre 6,7× de raio.** O depósito custa exatamente o que
uma pegada custa, e a divisão entre as duas metades é estável (**accumulate ~73%
· transfer ~27%**).

⚠️ **A sub-linearidade que a §5.50 mediu era o cap ESCONDENDO trabalho**, não uma
propriedade do depósito. Com a janela seguindo o pincel a escala virou `r²`
limpa — ou seja, **não há anomalia a consertar**. É a mesma forma do `soft_body`
da `line/gpu-nodes`: *o último item da fila dissolveu na medição*.

### 2. A fronteira com o HOST é grátis

O produto não usa o falloff do motor: ele passa a silhueta do Painter por um
`&mut dyn FnMut(i32,i32) -> f64` chamado **uma vez por pixel da caixa** do dab, e
a caixa de um disco tem `4r²` contra `πr²` do disco ⇒ **21% das chamadas caem
fora do pincel e devolvem zero**, pagando a chamada virtual do mesmo jeito.

Ablação entre as duas PORTAS reais (`accumulate_paint` × `accumulate_paint_shaped`),
com a `sil` fazendo a **mesma aritmética** do ramo interno — a diferença é a
indireção e nada mais:

| raio | motor ms | host ms | **razão** |
|---|---|---|---|
| 100 | 5,035 | 5,244 | **1,04×** |
| 200 | 21,177 | 20,528 | **0,97×** |
| 400 | 80,933 | 82,899 | **1,02×** |

**A indireção não é o alvo.** O custo do depósito são os PIXELS — ~7,5 ns por
pixel de caixa, com sete planos acessados de forma dispersa (`susp`, `sett`,
`paper`, `film`, `wet` no canvas + `pig`/`water` na janela). É o mesmo regime
limitado por banda em que o solver vive (8,9 ns/célula, §5.47), e a alavanca de
CPU ali já foi gasta (§5.44, §5.46).

### 3. O `Grid Size` paga — 4,5× e não 16×

Um custo honesto continua sendo um custo: `O(r²)` **sem teto** significa que o
artista pode pedir um pincel 8× mais caro que o de raio 141. A pergunta de
produto vira *a resposta já shipa?*, e a previsão era que sim: o dab é medido em
CÉLULAS (`cell_r = raio / razão`), então a pegada cairia com **razão²**.

Medido pela porta do artista (`on_canvas_pointer`, 4096², 24 entregas):

| raio | grid | total ms | por entrega | vs razão 1 |
|---|---|---|---|---|
| 200 | 1 | 76,67 | 3,195 ms | 1,00× |
| 200 | 2 | 25,96 | 1,082 ms | **0,34×** |
| 200 | 4 | 16,52 | 0,688 ms | **0,22×** |
| 400 | 1 | 99,34 | 4,139 ms | 1,00× |
| 400 | 2 | 35,15 | 1,465 ms | **0,35×** |
| 400 | 4 | 21,62 | 0,901 ms | **0,22×** |

**O slider paga 2,9× na razão 2 e 4,5× na razão 4** — real e grande, mas **não os
4× e 16×** que a contagem de células sozinha preveria. Resolvendo o sistema,
**~13-18% de uma entrega não cai com a grade do fluido**.

⚠️ **E a coluna é IDÊNTICA nos dois raios, o que NÃO discrimina de que o piso é
feito** — eu li isso primeiro como refutação do candidato "é o composite, que
escreve pixels", e a leitura estava errada: **todo termo escala com `r²`, então o
`r` cancela na razão por construção**. Os candidatos (o composite · e o AA do
`cell_subsamples`, cujo `n = min(razão, MAX_AA)` mantém as avaliações de
silhueta ~constantes em área de canvas, de propósito) ficam **NOMEADOS e não
atribuídos**: separá-los exige um relógio por fase dentro da entrega, e nenhum
veredito desta seção depende disso.

### 4. ⚠️ E a reconferência achou um DEFEITO que a wave do cap deixou

Indo ler o `transfer_paint` para decompor o custo — **não por suspeita** — o
passo 1 (auto-limpeza do bico, SPEC §10) ia `0..N`, e `N` é a área da janela do
**PISO** (`TRAIL_SIZE²` = 15129). Com o cap removido os buffers passaram a medir
`size²`, então num pincel maior que o piso a limpeza cobria os 15129 primeiros
índices **LINEARES** — que não são uma região, são as ~18 primeiras **LINHAS** de
uma janela de 845 de largura. **O corpo do pincel nunca mais limpava.**

É a classe que este repo já nomeou: **uma constante que era igual ao valor vivo**,
deixada para trás no dia em que o vivo virou variável. Alcançável: `Knob::TipClean`
é knob do grupo PAINT do painel Tuning (boot 0,0, faixa até 0,05).

Medido pelo gate: no centro da janela de um pincel grande o knob movia o azul do
bico de **164,16 para 164,16 — ZERO** — enquanto no controle (pincel dentro do
teto do modelo) movia 164,16 → 165,44. O fix é `0..self.tip_r.len()`, e num
pincel dentro do teto os dois números **coincidem** ⇒ fingerprint do ADR-0134
byte-idêntico **por construção** (3/3), aceitação 9/9.

⚠️ **Duas lições de gate, as duas minhas.** A 1ª versão lia o azul ANTES e DEPOIS
de um transfer, e o número andava **para BAIXO nos dois mundos** (189,66 →
165,44): o PICKUP e a LIMPEZA puxam em direções opostas dentro do mesmo transfer
e o pickup puxa ~16× mais forte — *medir o LÍQUIDO de dois efeitos que competem
não distingue "limpou" de "não limpou"*. O oráculo certo é o **A/B do próprio
knob**. E o `422` do centro da janela estava hardcoded, **espelhando a fórmula do
`fit_to` que o gate julga**; agora sai do `window_half_for_measure`.

Mais: o doc-comment do `lane_trails` afirmava *"ONE 123² window"* e
`lx >= TRAIL_SIZE`, os dois **falsos** desde a wave do cap.

### O que isto fecha e o que deixa aberto

✅ **FECHA a frente que a §5.50 abriu**: o depósito não tem anomalia, a fronteira
com o host é grátis, e o custo do pincel grande já tem resposta embarcada.

⚠️ **Aberto, com número:** os ~13-18% ratio-independentes de uma entrega (não
atribuídos, candidatos nomeados) · e a pergunta de PRODUTO que só o smoke
responde: **o artista encontra o `Grid Size` quando o pincel grande fica pesado?**

### 5. O smoke de 2026-08-01 fecha as duas perguntas, e o log traz um TERCEIRO fato

Enio: *"Parece correto o Tip Clean"* · *"O gridsize é sempre visível"*. As duas
metades que a §5.52 mandou para o smoke voltaram aprovadas — e a segunda **fecha
a pergunta de produto sem trabalho**: a resposta ao pincel caro já está na tela.

O log, porém, traz uma linha que eu já tinha lido **errado** antes:

```text
worker: busy 0% away 100% sleep 0% | TAXA DA AGUA 0.0 Hz (0 passos em 5.3s)
poca: 0.00 M celulas | 0.0 ns/celula
stamps: media 61.85ms pico 608.62ms em 69/120 (465 entregas, 9.18ms cada)
```

⚠️ Na §5.48 eu escrevi *"a água está INOCENTE e o log prova (`sim x0`, `worker
0%`)"* — li aquilo como *não é ela que custa* e **nunca perguntei por que ela
estava PARADA**. Um `away 100%` com zero passos em 5,3 s não é inocência, é a
água não rodando; e o instrumento estava dizendo isso desde julho.

**A causa é uma LEI, não um defeito**, e ela está em duas camadas que concordam:

- `Engine::sim_should_run() = !stroke_down || tool == Blow` — o motor **pausa a
  sim enquanto o traço está encostado**, que é o modelo de referência portado.
- `hand_off_sim` devolve cedo com `stroke_open`, porque entregar o motor ao
  worker durante o gesto compraria **zero passos** (o worker consulta o mesmo
  predicado, `offthread.rs:255`) e custaria uma viagem por dab.

⇒ **A água congelar sob a mão é o desenho**, e o `away 100%` é o balde
reportando-o com fidelidade. Fica escrito aqui para ninguém re-abrir isto como
fome de agendamento.

**E o carimbo do log RECONCILIA com a fixture.** Re-medindo o mesmo probe
depois do cap (`measure_the_stamp_landing_on_a_puddle_that_is_already_there`):

| raio | sob o cap (§5.50) | pós-cap | |
|---|---|---|---|
| 100 | 1,86 ms | **2,356 ms** | 1,27× |
| 200 | 3,34 ms | **4,200 ms** | 1,26× |
| 300 | 4,37 ms | **5,892 ms** | 1,35× |

O cap escondia 26-35% do trabalho nesses raios. A fixture não roda a sim, e a
§5.49 já mediu esse imposto em **1,10-1,34×** ⇒ `5,89 × 1,34 = 7,9 ms`, contra
os **9,18 ms/entrega** do produto — o resto cabe em tamanho de pincel e passo do
mouse. **Nenhum número órfão sobrou.**

Os 33 fps da janela de traço são, então, o preço honesto de um pincel no topo do
slider com `Grid Size 1`: 61,85 ms de carimbo por quadro. O `Grid Size 2` corta
isso 2,9× e o `4`, 4,5×.

⚠️ **O que o log deixa NOMEADO para depois:** na janela SEM carimbo o maior item
é `painter-dispatch(cpu) = 11,80 ms` (era 6,90 na §5.47) — ele **não é o
retângulo** e vive na shell, não na água.

---

## §5.53 — O `painter-dispatch` sem carimbo, e o divisor que faltava (2026-08-01)

Fechada a frente do depósito (§5.52), o maior item que sobrou no log é a janela
em que **ninguém está pintando**:

```text
[frame] total=16.21ms (~62 fps) | painter-dispatch(cpu)=11.80ms
  tool-tick: media 3.89ms em 115/120 | stamps: 0 entregas
  worker: busy 66% away 18% sleep 16% | TAXA DA AGUA 38.6 Hz
  poca: 2.37 M celulas | 7.2 ns/celula
```

**11,80 ms de dispatch com zero carimbos**, e `ns/célula` constante (7,2 contra
7,5 na janela que carimba) ⇒ **não é contenção**. Era **6,90 ms** quando a §5.47
o nomeou: quase dobrou.

⚠️ **E a linha não decidia nada** — `FRAME_PROF_DISPATCH_US` é **um balde só**. O
split existe (`PH2D_PAINT_PERF` divide em `preview`/`panel`/`overlay`/`upload` e
ainda por sub-chamada), mas o smoke não o liga; e as duas leituras possíveis
pedem curas **opostas**: *um retângulo grande de vez em quando* (o alvo é a
frequência) × *um retângulo grande sempre* (o alvo é o TAMANHO).

### O que a medição headless diz — antes de gastar outro smoke

`measure_what_the_preview_drain_costs_with_the_water_running`, reproduzindo a
janela 3 (poça de ~2 M células a 4096², um tick por quadro, **zero eventos**):

| condição | dreno ms | px publicados/quadro | células vivas | **razão** |
|---|---|---|---|---|
| poça viva | **0,001** | **8.892.819** | 2.068.506 | **4,30×** |
| tela seca (controle) | 0,000 | 0 | 0 | — |

**Duas coisas, e as duas importam:**

1. **O dreno do TOOL é grátis** — `take_preview_arc` devolve um `Arc` por
   refcount, não uma cópia. O custo não está lá.
2. **Mas ele publica metade da tela por quadro.** 8,89 M px numa tela de 16,8 M,
   **toda vez que a água anda**, para 2,07 M células de água viva ⇒ **o retângulo
   pede 4,30×** o que a água tocou. A bbox de uma faixa diagonal é um múltiplo
   da faixa — exatamente o que o censo do `live_span_cells` (§5.47) já nomeou
   para o outro lado: *"a faixa NÃO é a bbox"*.

⇒ O `painter-dispatch` é o *gather + premultiply + upload* dessa área, e o alvo
provável é **a região publicada**, não a velocidade de quem a move.

### A cura desta seção é o INSTRUMENTO, e é a mesma da §5.48

A linha `[frame]` passa a imprimir o divisor:
`painter-dispatch(cpu)=11.80ms (8.26 M px publicados em 57 quadros)`. A contagem
mora **onde a bbox é resolvida** (`painter_bridge`, logo depois do
`take_preview_upload_bbox`), senão o divisor descreveria quadros que o numerador
não pagou. Dois arch-gates, **2 mutações, 2 sangram**, uma em cada.

⚠️ **TRÊS defeitos de fixture nesta sonda, todos meus e todos já escritos neste
doc:**

- **sem VÃO entre quadros** os 90 "quadros" passam em microssegundos, o worker
  nunca acorda (`IDLE_SLEEP` = 4 ms), `fresh` é sempre falso, o composite nunca
  roda e a tabela saiu **0,000 ms / 0 px** — medindo uma água parada. É a lição
  que a sonda irmã do carimbo **carrega escrita**, violada um arquivo adiante;
- **dividi pelos 90 quadros do laço** e não pelos 57 que drenaram (a água corre
  a ~38 Hz contra 60 de display), diluindo o retângulo em 40% — **o divisor
  errado dentro do instrumento que existe para consertar divisores**;
- e o controle de tela seca é o que prova que a tabela não é vácuo: **0 px, 0
  quadros drenados**.

**Aberto:** confirmar pela porta do produto (o próximo log já traz o divisor) e,
se a área for mesmo o alvo, publicar por FAIXA em vez de bbox — o `row_lo`/
`row_hi` que o `live_span_cells` percorre já é a faixa, e ela não custa nada a
mais para quem já a mantém.

### ⚠️ CORREÇÃO (mesmo dia): os primeiros números desta seção estavam ERRADOS

O smoke seguinte veio com o divisor novo dizendo:

```text
painter-dispatch(cpu)=0.07ms (0.00 M px publicados em 80 quadros)
painter-dispatch(cpu)=4.47ms (0.00 M px publicados em 80 quadros)
painter-dispatch(cpu)=0.02ms (0.00 M px publicados em 80 quadros)
```

**`prev_n = 80`** (o sítio de contagem disparando, e batendo exato com os 80
passos da água) **com `0.00 M px`** — o instrumento acusando a si mesmo.

**A causa é minha:** `take_preview_upload_bbox` devolve **`(x, y, w, h)`** — o
doc-comment dele diz isso quatro linhas acima da assinatura — e eu computei
`(x1 - x0) * (y1 - y0)`, ou seja `(w - x) * (h - y)`.

⚠️ **O que torna esse erro perigoso é que ele às vezes dá um número plausível.**
Na fixture headless os retângulos caem perto da origem, `w > x`, e a sonda
reportou **8,26 M** contra os **8,89 M** verdadeiros — perto o bastante para eu
escrever um doc em cima. No PRODUTO os retângulos vivem longe da origem, `w < x`,
o `saturating_sub` devolve zero, e o log gritou. **Quem pegou foi o instrumento
do produto; a minha sonda não.** É a quarta vez nesta sessão que um log do
produto vence o meu raciocínio.

Corrigidos os dois sítios (o produto e a sonda), a tabela acima já traz os
números certos: **8,89 M px, 4,30×**. O veredito não muda — **muda a confiança
que ele merecia antes de ser conferido**.

⚠️ **E o buraco de gate era real:** os dois arch-gates da wave afirmam que o
placeholder é IMPRESSO e que a contagem mora no sítio certo — **nenhum olha para
o VALOR**. *Um gate de instrumento que nunca afirma que o número é plausível
deixa passar exatamente a classe de erro que o instrumento existe para caçar.*
Nasceu o terceiro, comportamental
(`the_upload_bbox_is_a_rect_and_a_corner_reading_gives_nonsense`), cuja fixture
põe o traço **longe da origem** — onde a leitura-por-cantos dá resposta absurda —
e afirma `x > w`, para que ela não volte a ser plausível por acidente.

### E o smoke NÃO reproduziu os 11,80 ms

Nas três janelas o dispatch custa **0,02 · 4,47 · 0,07 ms**, com a poça em
**1,04-1,17 M células** — metade da poça de 2,37 M da janela que media 11,80.
Com a água a **40,0 Hz** e `sleep 58-61%`, ela está no nominal da SPEC e o worker
dorme.

⇒ **O `painter-dispatch` acompanha o tamanho da poça, e nesta cena ele é
essencialmente grátis.** Os 11,80 ms eram uma poça 2× maior. O alvo continua
nomeado e agora tem instrumento: se um log futuro trouxer o par
`dispatch alto + M px alto`, a cura é publicar por **FAIXA** em vez de bbox
(o `row_lo`/`row_hi` que o `live_span_cells` já percorre É a faixa); se vier
`dispatch alto + M px baixo`, o custo está noutra fase e o `PH2D_PAINT_PERF` a
separa.

### ⚠️ E O PRODUTO FECHOU A FRENTE — sem wave, e derrubando a minha fixture

O log seguinte, com o instrumento consertado, é consistente em quatro janelas:

| janela | dispatch | px publicados | poça | **razão** |
|---|---|---|---|---|
| 1 | 3,89 ms | 2,10 M | 1,52 M | **1,38×** |
| 2 | 3,93 ms | 1,98 M | 1,45 M | **1,37×** |
| 3 | 4,09 ms | 1,97 M | 1,44 M | **1,37×** |
| 4 | 3,87 ms | 1,98 M | 1,45 M | **1,37×** |

**O retângulo pede 1,37× a água viva — não os 4,30× que a sonda headless
previu.** A `heavy_puddle()` são três traços **DIAGONAIS**, e a bbox de uma
faixa diagonal é um múltiplo enorme dela; a poça que o artista faz é compacta.
⚠️ É a lição deste doc pela terceira vez: *quando o número vira decisão de
produto, ele tem de sair da porta do produto* — e desta vez ela custou uma
hipótese de wave inteira.

**O veredito, pela regra que a §5.53 escreveu antes de medir:**

- publicar por FAIXA em vez de bbox compraria **1,37×** sobre 3,9 ms = **~1 ms**;
- e o frame **não é limitado por CPU**: `cpu-encode(raw) 8,25` + `present/acquire-stall 8,35` somam os 16,6 ms de um quadro de 60 fps, ou seja **a CPU passa metade do quadro esperando o GPU**;
- a água está em **39,5-40,5 Hz** (o nominal da SPEC) com `sleep 44-46%`.

⇒ **Não há wave aqui.** O `painter-dispatch` é 3,9 ms num quadro com 8,3 ms de
ociosidade, sobre uma região que já é quase justa. Os **11,80 ms** da §5.53 eram
uma poça 2× maior numa cena de traços cruzados — o custo acompanha a poça, e o
`Grid Size` continua sendo a alavanca de quem quiser cortá-lo.

**O que fica desta wave é o INSTRUMENTO**, e ele se pagou duas vezes: expôs o
meu erro de fórmula (o `0.00 M`) e depois **fechou a frente sem trabalho de
produto** — que é o melhor resultado que uma medição pode ter.

---

## §5.54 — O `stall` não era espera: era uma SUBTRAÇÃO com nome de medição (2026-08-01)

Fechada a frente do dispatch, o maior item do quadro passou a ser o
`cpu-encode(raw)` (8,25-10,02 ms). Antes de medi-lo, a primeira pergunta foi de
onde vêm os OUTROS números da mesma linha — e a resposta derrubou uma frase que
eu tinha acabado de escrever para o Enio.

**`present/acquire-stall` era `total − encode`.** Uma subtração, não uma
medição. E o `encode` (`frame_cpu_ms_ewma`) começa em **`cpu_start`**, que fica
**depois** do `tool-tick`, do flush de carimbo e do pump de eventos — a linha 697
do `render_loop` diz isso literalmente: *"Done before `cpu_start` so the re-stamp
stays OUT of the encode window"*.

⇒ **O resíduo continha trabalho de CPU sob um rótulo que diz *espera de GPU*.**
Com `stall 7,91` e `tool-tick 3,31` no mesmo log, a espera real é **~4,6 ms** e a
CPU trabalha **~12 ms** de um quadro de 16,6 — não os 8,25 que a linha sugeria.

⚠️ **E eu li o 8,35 como ociosidade e reportei ao Enio que *"a CPU passa metade
do quadro esperando o GPU"*.** Era o rótulo, não a máquina. *Um número derivado
por subtração absorve tudo que ninguém mediu, e herda o nome de quem o
publicou.*

**A cura:** o `acquire_frame` passa a ser cronometrado **no sítio**
(`note_acquire_wait`, em `present.rs`, dentro do braço `Ok`), e a linha publica
uma partição de três que soma por construção:

```text
total | cpu-encode(raw) | acquire(medido) | fora-do-encode
```

onde `fora-do-encode = total − encode − acquire` é a CPU que o quadro paga
**antes** da janela de encode — o `tool-tick`, o flush, o pump. Ela existia o
tempo todo; só não tinha nome.

⚠️ **O `cpu_total` do próprio present já excluía o acquire** (`work_before_acquire
+ work_after_acquire`), o que torna o encode uma medida honesta de CPU. O defeito
nunca esteve nele — esteve em **chamar o resto de "stall"**.

**Dois arch-gates, 2 mutações, 2 sangram** (uma em cada): tirar o cronômetro
mata o primeiro; fazer o resíduo voltar a `total − encode` mata o segundo — e é
essa segunda metade que impede a mentira de voltar em silêncio, porque os três
números continuariam sendo impressos com um deles contendo o outro.

**Aberto, e é o item 1 propriamente dito:** o `cpu-encode` continua sem split.
Sabemos que `painter-dispatch` (3,9) e `hero-paint` (0,9) estão dentro dele, o
que deixa **~3,4 ms sem dono**. O próximo log já dirá quanto do quadro é
realmente espera — e se `fora-do-encode` for grande, o alvo não está no encode.

### O item 1 FECHOU no primeiro log com a partição — e ele confirmou o que eu tinha retirado

Cinco janelas de observação, com a partição nova:

| | total | cpu-encode | **acquire (medido)** | fora-do-encode | tool-tick |
|---|---|---|---|---|---|
| 2 | 16,74 | 8,14 | **8,81** | 0,00 | 3,01 |
| 3 | 16,71 | 7,37 | **8,70** | 0,65 | 3,34 |
| 4 | 16,78 | 8,22 | **8,76** | 0,00 | 3,06 |
| 5 | 16,65 | 8,36 | **8,34** | 0,00 | 3,10 |
| 6 | 17,23 | 8,86 | **7,32** | 1,05 | 3,93 |

**`acquire` medido em 7,32-8,81 ms** ⇒ o quadro de observação é **limitado pelo
present**, e a CPU tem folga real. Pela regra que a §5.54 escreveu antes de
medir, **o item 1 fecha sem wave.**

⚠️ **E ele reabilita a frase que eu tinha retirado.** Eu havia dito ao Enio que
*"a CPU passa metade do quadro esperando o GPU"* lendo o resíduo, e depois
**retirei** a afirmação porque o instrumento não podia sustentá-la. Medido, ela
está **certa** — mas a retirada continua certa também: *ela era infundada quando
foi feita, e passou a ser fundada quando o acquire virou medição.* As duas
coisas não se contradizem.

⚠️ **E a partição derrubou um comentário do produto no primeiro log.** O bloco do
profiler afirmava *"`stamp` + `tick` happen BEFORE `cpu_start`"*. Só o **stamp**:
o flush coalescido roda na linha ~698 (antes do `cpu_start`, 712) e o `on_tick`
na ~1198 — **dentro** do encode. O log é a prova: `fora-do-encode` mede
**0,00-1,05 ms** nas janelas sem carimbo enquanto o `tool-tick` mede
**3,01-3,93**, o que seria impossível se o tick estivesse fora. Comentário
corrigido.

### E o que a partição NOMEIA como o custo real do produto

A janela em que o artista PINTA:

```text
total=59.58ms (~17 fps) | cpu-encode=1.51ms | acquire=8.27ms | fora-do-encode=49.80ms
  stamps: media 47.05ms pico 320.96ms em 77/120 (508 entregas, 7.13ms cada)
```

**`fora-do-encode` 49,80 ≈ `stamps` 47,05** — a partição nova aponta o dedo
exatamente para o flush coalescido, que é o único inquilino de fora da janela.
**17 fps ao pintar**, com 508 entregas a 7,13 ms.

⇒ **O item 1 não tem wave; o custo do produto é o CARIMBO**, que a §5.52 já
mediu como **honestamente limitado pela pegada** (~30 ns/r², plano de raio 60 a
400) e cuja alavanca embarcada é o **`Grid Size`** (0,34× na razão 2, 0,22× na 4
— medido pela porta do artista). Um pincel no topo do slider a `Grid Size 1`
custa 17 fps *por construção*, e o slider está sempre visível.

---

## §5.55 — O item 2 (K–M) DISSOLVEU, e a tabela mostrou o que eu não procurava (2026-08-01)

O CLAUDE.md nomeava o K–M como *"o único regime que segue work-limited — **4,75×
o passo**, 67,9 ms, 14,7 Hz a 4096²/razão 1"*. Medido hoje pela porta do produto
(`measure_what_km_costs_at_each_grid_ratio`, duas corridas limpas):

| razão | passo off | passo ON | **custo do K–M** |
|---|---|---|---|
| 1:1 | 15,8 / 16,6 | 19,1 / 18,8 | **1,21× / 1,13×** |
| 2:1 | 4,5 / 4,5 | 5,7 / 5,0 | **1,28× / 1,10×** |
| 4:1 | 1,4 / 1,3 | 1,7 / 1,9 | **1,17× / 1,42×** |
| 8:1 | 0,8 / 0,8 | 0,9 / 0,8 | **1,08× / 1,09×** |

**1,1-1,4×, não 4,75×.** A tabela de transferência sRGB (doc 24) já o levara de
20-34× para 2-3%, e o solver independente de ordem (ADR-0147) + a decomposição
do fluxo (§5.46) fecharam o resto. **A nota sobreviveu ao fato**, e foi
corrigida no CLAUDE.md.

⚠️ **E a PRIMEIRA corrida disse 3,39×** — com uma célula em que a razão 2 ligada
custava *mais* que a razão 1 ligada, o que é impossível com 4× menos células.
Não reproduziu. *Um número que não reproduz não é achado, é ruído com casas
decimais* (§5.13), e repetir custou 20 segundos.

### O que a tabela mostrou sem eu procurar

A coluna do COMPOSITE, estável nas três corridas limpas:

| razão | passo | **composite (tela CHEIA)** |
|---|---|---|
| 1:1 | 16,6 ms | **9,4 ms** |
| 2:1 | 4,5 | **18,5** |
| 4:1 | 1,3 | **17,6** |
| 8:1 | 0,8 | **17,0** |

**O composite DOBRA quando a grade fica mais grossa.** O mecanismo está no
desenho: ele tem duas fases — um laço por CÉLULA (que encolhe com `ratio²`) e um
laço por **PIXEL** (que não encolhe, e que na razão 1 é uma cópia direta e acima
dela vira a reconstrução smoothstep de quatro cantos).

⇒ **A alavanca do `Grid Size` é limitada pelo composite, não pelo solver.**

> ⚠️ **CORREÇÃO (mesma sessão, §5.56): esta coluna é o PIOR CASO, e eu a
> publiquei como se fosse a do quadro.** A sonda chama `mark_dirty_full()` antes
> de cada amostra — ela compõe a **tela inteira** de 4096², que o produto paga no
> nascimento da sessão e em ações autoradas, **nunca por quadro**. A frase *"a
> partir da razão 2 ele é ~93% do custo da água"* saiu daqui.
>
> *Um custo sem o seu escopo é inatribuível* — literalmente a regra que o
> `painter-dispatch` me custou um smoke três dias antes (§5.53), aplicada por
> mim a todo mundo e não à minha própria fixture. O número honesto, com o
> retângulo ao lado, está na §5.56 — e ⚠️ **a conclusão sobrevive**: com o
> retângulo real o composite é 26% do custo da água na razão 1 e **67 / 87 / 92%**
> nas razões 2 / 4 / 8.

### A fatoração que shipou — 1,24×, bit-idêntica

`SampleU::at` recomputava **por pixel** seis grandezas que só dependem de `py`
(a coordenada, o `floor`, o smoothstep, os dois índices de célula e os dois
offsets de linha) — e o composite percorre o retângulo **por linhas**, onde `py`
é constante. É a mesma fatoração que o `FlowRowSampler` fez no `advect` (§5.42),
e é byte-exata por construção: as mesmas operações, na mesma ordem, sobre os
mesmos `f64`.

`SampleU::row(py) -> SampleURow` + `SampleURow::at(pig, px)`. A rota antiga ficou
**CONGELADA sob `cfg(test)`** como oráculo (um `pub` sem chamador seria uma
segunda resposta esperando alguém chamá-la).

**A/B costas-com-costas na mesma corrida, sobre o mesmo plano:**

| razão | por-pixel | por-linha | **razão** |
|---|---|---|---|
| 2:1 | 0,869 ms | 0,702 | **1,24×** |
| 4:1 | 3,476 | 2,806 | **1,24×** |
| 8:1 | 13,980 | 11,432 | **1,22×** |

Gate de igualdade **BIT A BIT** contra a rota congelada, em seis razões, sobre um
campo **estruturado** de propósito (um plano chato faria qualquer amostragem
concordar — a lição do §5.42); mutação (trocar a ordem dos dois cantos, que é a
ordem das somas em `f64`) **sangra**. Campo morto `SampleURow::stride` removido —
os offsets de linha já o embutem.

⚠️ **O NÚMERO DE PRODUTO ESTÁ PENDENTE, e o motivo fica escrito:** as duas
corridas de verificação caíram com `load average 24,88` (três `rustc` de outras
linhas a 704%/360%/335%) e a MESMA célula deu **134,4 ms e 42,7 ms**. *Nenhum
número absoluto desta máquina significa algo com o load acima de ~5* (§5.49). O
A/B do amostrador é imune porque as duas rotas correm na mesma corrida; o
composite do produto precisa de uma máquina calma.

### O número que faltava, com a máquina calma (`load 4,76 / 1,52 / 0,84`)

| razão | passo off | passo ON (K–M) | composite ANTES | **composite AGORA** | |
|---|---|---|---|---|---|
| 1:1 | 15,59 | 18,54 | 9,4 | **9,25** | ← **o controle** |
| 2:1 | 4,52 | 5,63 | 18,5 | **15,84** | 1,17× |
| 4:1 | 1,31 | 1,62 | 17,6 | **15,13** | 1,16× |
| 8:1 | 0,77 | 0,86 | — | **15,00** | |

⚠️ **A razão 1 é o CONTROLE INTERNO da wave**: ela toma o caminho de identidade
(`is_identity()`), onde a fatoração não existe, então ela **tem** de ficar
parada — e ficou (9,4 → 9,25, dentro do ruído). Um ganho que aparecesse ali
significaria que o `one_cell_per_pixel` deixou de ser o caminho antigo.

O produto vê **1,16-1,17×** contra os **1,24×** do A/B do amostrador, e a
diferença é honesta: o composite faz mais coisa além de amostrar (o `over`, os
gates, o arredondamento), e a fatoração só toca a amostragem.

---

## §5.56 — O item 3 (a GPU do solver): os dois gatilhos MENSURÁVEIS do ADR-0146 fecharam (2026-08-01)

O terceiro item da fila era *"a GPU do solver"* — o [ADR-0146](../../../architecture/decisions/0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md),
em proposta desde 2026-07-29 com quatro emendas, cada uma re-precificando-o para
baixo. A regra do CLAUDE.md §0 corta nos dois sentidos: *quem move o número que
tornava algo inalcançável tem de reconferir a nota* — e quem move o número que
tornava algo **necessário** tem de reconferir a necessidade.

### O erro de fixture que eu tinha de consertar primeiro

A §5.55 publicou *"o composite é ~93% do custo da água a partir da razão 2"*
sobre uma sonda que chama **`mark_dirty_full()`** antes de cada amostra — isto é,
sobre uma composição de **tela inteira**, que o produto paga no nascimento da
sessão e em ações autoradas, nunca por quadro.

*Um custo sem o seu escopo é inatribuível.* É literalmente a regra que o
`painter-dispatch` me custou um smoke três dias antes (§5.53) — e eu a apliquei
a todo mundo menos à minha própria fixture.

A sonda nova (`measure_the_composite_with_the_rect_it_actually_writes`) mede as
DUAS colunas lado a lado, **cada uma com a área ao lado**:

| razão | cheio | M px | ns/px | **passo** | M px | **ns/px** | composite ÷ água |
|---|---|---|---|---|---|---|---|
| 1:1 | 8,56 ms | 16,78 | 0,51 | **5,59 ms** | 9,01 | **0,62** | **26 %** |
| 2:1 | 15,91 | 16,78 | 0,95 | **9,38** | 9,05 | **1,04** | **67 %** |
| 4:1 | 15,43 | 16,78 | 0,92 | **9,01** | 9,14 | **0,99** | **87 %** |
| 8:1 | 14,80 | 16,78 | 0,88 | **9,12** | 9,53 | **0,96** | **92 %** |

⚠️ **O `ns/px` concorda entre as duas colunas** (0,51 × 0,62 · 0,95 × 1,04 · …)
⇒ **o composite é limitado pela ÁREA**, que é a forma correta, e a diferença
entre as colunas é o **retângulo**, não uma patologia. *Não há wave; há um
retângulo.*

⚠️ **E o divisor teve DUAS versões, porque a primeira saturou:** eu li a área do
`take_preview_upload_bbox`, que só é preenchido pelo **dreno do frame** — as
duas colunas diziam `16,78 M px` para composições cujos tempos diferiam 1,7×. O
divisor honesto é o `dirty` do MOTOR convertido pela **porta do composite**
(`cell_rect_to_px`), nunca por uma segunda aritmética minha.

⚠️ **A conclusão da §5.55 sobrevive com o número certo:** o composite domina a
partir da razão 2 (67 / 87 / 92 %), e na razão 1 ele é 26 %.

### O que isso faz com o slider `Grid Size` — ele SATURA

Somando as duas metades (o passo + o composite do retângulo real):

| razão | passo | composite | **água/quadro** | ganho vs 1:1 |
|---|---|---|---|---|
| 1:1 | 15,59 | 5,59 | **21,18 ms** | — |
| 2:1 | 4,52 | 9,38 | **13,90** | 1,52× |
| 4:1 | 1,31 | 9,01 | **10,32** | **2,05×** |
| 8:1 | 0,77 | 9,12 | **9,89** | 2,14× |

**O solver sozinho cai 20×; a água cai 2,1×.** O `Grid Size` continua sendo a
alavanca sancionada, mas o teto dela é o composite — e isso está agora medido
em vez de suposto.

### Os dois gatilhos mensuráveis do ADR-0146

O ADR nomeia três condições que o RE-ABREM. Duas são mensuráveis e as duas
fecharam:

1. **"o smoke reprovar a razão 2-4 no DESENHO"** — moot: a tabela acima diz que
   **ninguém precisa engrossar a grade por velocidade**. A razão 1 custa 21,18 ms
   contra o nominal de **25 ms (40 Hz)** da SPEC. O slider vira decisão de LOOK.
2. **"1:1 a 4096² sem concessão como requisito de produto"** — **morto por
   medição**. O ADR foi escrito quando 1:1 custava **32,2 ms (31 Hz)**, abaixo do
   nominal. Hoje o passo custa **15,59 ms (64 Hz)**, e **18,54 (54 Hz) com o K–M
   ligado**. *A concessão que o gatilho existia para remover não existe mais.*
3. **"uma feature nova pedir campos que a CPU não alcança"** — hipotética,
   inalterada, e é a única que sobrevive.

⚠️ **E o produto confirma pela própria porta:** o último smoke aprovado do Enio
traz `busy 62 % away 16 % **sleep 22 %** | TAXA DA AGUA 38,5 Hz | poça 2,20 M |
7,3 ns/célula` — e `7,3 × 2,20 M = 16,06 ms/passo`, que reconcilia com os 15,59
da sonda. **A água não espera a CPU; ela espera o próprio relógio.** Uma GPU não
entrega mais que o nominal, e o nominal já chegou.

### A razão NOVA contra a Fase 2, que o ADR não tem

O ADR lista dois obstáculos que nunca foram sobre o solver: o **stamp** (a
silhueta do Painter por closure) e a **residência** dos 14 planos. A tabela
acrescenta um terceiro, e ele é estrutural:

⚠️ **O maior item por-quadro da água acima da razão 1 é o composite, e o
composite escreve o `canvas_rgba` — o DOCUMENTO do artista.** Ele é da CPU por
contrato: o undo por delta, o histórico por janela, o bake, o save e os ~25
sítios de escrita de plano leem dali. Levar o composite ao device obrigaria a
escolher entre *deixar o documento desatualizado* (o undo devolveria lixo) e
*ler de volta por quadro* (a mesma banda, com uma sincronização a mais).

⇒ A Fase 2 do ADR (*"os planos device-resident e o composite lendo do device"*)
resolve a LEITURA do pigmento e **não** resolve a escrita, que é o lado caro.

### Veredito

**Item 3 não tem wave.** Não porque seja difícil — porque **todo número que ele
atacaria já está abaixo do que domina**: o solver corre 1,6× acima do nominal na
razão 1, o worker dorme 22 % na cena do smoke, e o que sobra do custo da água é
um composite limitado pela área que escreve o documento do artista.

O ADR-0146 recebe a **Emenda 5** com esta tabela. A recomendação não muda de
sinal — ela deixa de ser *"não construir agora"* e passa a ser *"os dois
gatilhos mensuráveis fecharam; sobra o hipotético"*.

⚠️ **O que fica NOMEADO, não escondido:** o `ns/px` do composite **dobra** ao
sair da razão 1 (0,62 → 1,04) — é a reconstrução smoothstep de quatro cantos
contra a cópia direta. A fatoração da COLUNA (os dois índices de célula só mudam
quando o pixel cruza uma fronteira; `ratio` pixels consecutivos compartilham os
mesmos cantos) é a irmã exata da fatoração da LINHA que a §5.55 shipou, e vale
talvez ~1,2× sobre 9 ms. **Não construída de propósito**: na razão 1 — o default
— ela não muda um byte (caminho de identidade), e acima dela a água já corre com
folga contra o nominal. *É estritamente menos trabalho e não compra nada que o
artista veja*, que é o critério que a §5.11 fixou para não vender ruído como
ganho.

---

## §5.57 — O `away 24 %` do worker NÃO é núcleo ocioso: construído, medido, REVERTIDO (2026-08-01)

O log de smoke do Enio (janela **assistindo** — `stamps: 0 entregas`) trouxe:

```text
[frame]   agua: sim media 19.82ms pico 31.54ms x68 | composite media 6.06ms x62
[frame]   worker: busy 67% away 24% sleep 9% | TAXA DA AGUA 34.0 Hz
[frame]   poca: 1.88 M celulas | 10.6 ns/celula
```

Três leituras que reconciliam antes de qualquer hipótese: `10,6 ns/célula` é **um
dígito** ⇒ máquina sã (§5.49); `19,82 ms` sobre 1,88 M células escala dos
**15,59 ms** que a §5.56 mediu sobre ~1,6 M ⇒ o passo é o esperado a `Grid Size 1`;
e o **`composite media 6,06 ms`** casa com os **5,59** que a sonda do retângulo
real previu. *Três testemunhas concordando é o que separa um número de um achado.*

O item que sobra é o `away 24 %`: o tick pede o motor, composita e só o devolve no
`hand_off_sim` do FIM — então o worker fica **480 ms de uma janela de 2 s** sem o
motor. A §5.40 já tinha nomeado isto e o precificado em **~1,06×**, um número que
quatro waves moveram sem ninguém reconferir (CLAUDE.md §0).

### O split, medido no código que SHIPA

`cfg(test)` em torno dos dois blocos que a **fronteira do motor** já separava — não
uma sonda com laço próprio (§5.11) e não atribuição a uma LINHA dentro de um bloco
(§5.44); os dois blocos já são distintos, e o relógio só diz de que lado o tempo cai.

| razão | motor ms | pixels ms | total | **livre** |
|---|---|---|---|---|
| **1:1** (default) | 2,846 | **3,594** | 6,440 | **55,8 %** |
| 2:1 | 0,818 | 9,522 | 10,340 | 92,1 % |
| 4:1 | 0,260 | 9,548 | 9,808 | 97,4 % |
| 8:1 | 0,118 | 9,731 | 9,849 | 98,8 % |

⚠️ **E a aritmética que eu tinha feito ANTES estava errada**: extrapolando a tabela
de razões da §5.56 eu previ **81 % livre** na razão 1; medido, são **55,8 %**. A
medição direta ganha da inferência de segunda ordem sobre uma tabela — a lição da
§5.44, aqui do lado favorável (eu ia prometer mais do que havia).

### ⛔ MEDIDO E REJEITADO — não refaça: devolver o motor na fronteira

Construído: uma porta `wetpaint_composite_releasing` só para o tick (as ações
autoradas mexem no motor logo depois; o `dab_route` tem traço aberto e o
`hand_off_sim` recusa ali sozinho), com o **véu vetando** a entrega porque ele lê o
grid vivo depois do laço de pixels.

**A/B costas-com-costas, máquina calma (`load < 2`), três amostras cada,
`heavy_puddle` a 4096²:**

| | água | tick p50 | **tick max** |
|---|---|---|---|
| sem (a rota que shipa) | 35,6 / 36,2 / 36,1 Hz | 4,1-5,5 ms | **12,9 / 15,9 / 14,3 ms** |
| com a entrega antecipada | 36,6 / 36,6 / 36,9 Hz | 4,1-5,8 ms | **20,5 / 16,7 / 27,5 ms** |

**+0,6 Hz de água (2 %) ao preço de +50 % no PIOR TICK.** Um *hitch* que o artista
vê, trocado por uma taxa que ele não distingue — o inverso exato do trade que a wave
off-thread (§5.31-5.38) fez para chegar aqui. Revertida inteira.

⚠️ **O MECANISMO vale mais que o número, e é a lição desta seção:** o `away`
**não é núcleo ocioso**. As duas metades já são row-parallel — o composite pelo
ADR-0109, o solver pelo ADR-0145/0147 — e **saturam os 32 núcleos**, então
sobrepô-las não cria capacidade: só move a contenção para DENTRO do frame. *Um
balde de ESPERA só é oportunidade quando o recurso que ele espera está PARADO*, e
aqui o recurso é a máquina inteira, que já estava ocupada pelo outro lado da espera.

O `the_tick_never_waits_for_a_whole_stage` **reprovou por isso** (42,90 contra a
barra de 30, e depois 20-27 ms em máquina calma) — ele nomeou o preço antes de eu
o ter medido, que é exatamente o que um kill de wall-clock existe para fazer.

**Fica:** o split instrumentado (`cfg(test)`, custo zero no produto) e a sonda
`measure_how_much_of_the_composite_needs_the_engine` — quem quiser reabrir a
frente encontra o número, e o motivo pelo qual ele não basta.

---

## §5.58 — S3, degrau 3b: a premissa RE-MEDIDA, e a decomposição que decide por onde começar (2026-08-01)

O §5.28 fechou dizendo *"os pré-requisitos estão todos medidos e gateados; o que sobra é a troca"*.
Antes de construí-la, a premissa foi re-medida — e ela **moveu em dois lugares**.

### O preço, hoje, na máquina calma (4096², impasto)

| item que o S3 mata | §5.28 | **hoje** |
|---|---|---|
| fold (`commit_stroke_height`) | 9,25 ms | **11,92 ms** |
| fork do canvas no pen-down | 3,16 | **~3,2** |
| `free` da geração anterior | 2,4–5,0 | 2,4–5,0 |
| **total por traço** | — | **~17–20 ms** |

⚠️ **E a nota que eu tinha citado ao Enio (*"~21 ms, pen-down 11,7 + pen-up 9,2"*) estava velha:** os
11,7 são de ANTES da §5.15, que paralelizou a porta de fork; hoje o fork do canvas custa **3,2**. O
total honesto é o mesmo em ordem de grandeza, mas **a maior fatia é o FOLD, não o pen-down** — e é isso
que muda por onde a wave começa.

### O censo de donos separa dois regimes que a nota tratava como um

`who_holds_the_planes_when_a_stroke_begins`, hoje:

```text
REGIME (2 traços commitados, nenhum gesto)   canvas 2 · heights 2 · covers 2 · mats 2
DENTRO do gesto (logo após o pen-down)       canvas 1 · heights 4 · covers 4 · mats 4
  - sem o snapshot de pen-down               canvas 1 · heights 3 · covers 3 · mats 3
  - …e sem o histórico                       canvas 1 · heights 1 · covers 1 · mats 1
```

⇒ **O canvas já está em UM dono dentro do gesto** (o pen-down forkou uma vez e o tool ficou sozinho);
quem está em **quatro** é o RELEVO. E o fold — 11,92 ms, o maior item — é exatamente
`fork_covers`/`fork_mats`/`fork_heights` sobre esses quatro donos: **9,61 ms medidos hoje** pela porta
do produto (covers 0,476 · heights 3,291 · mats 5,847).

⇒ **A wave começa pelo RELEVO, não pelo canvas.** É ~9,6 dos ~17-20 ms, o escopo é menor, e o journal
**já descreve o relevo de 100 % dos passos** (§5.29: 302 de 302 DESCREVEM, 0 incompletos, 0 divergências).

### O desenho, e a peça que reduz o tamanho da troca

A leitura ingênua da identidade do §5.28 (`cursor[i] == journal.get(i).unwrap_or(vivo[i])`) sugere
**materializar** o `before` a partir do journal — e isso seria **trocar seis por meia dúzia**: a
materialização produz um `Vec` novo, que é a cópia que se queria evitar.

⚠️ **O `split` não precisa do `before` MATERIALIZADO — ele precisa do DELTA**, e o lado `before` do
delta **é literalmente o conteúdo do journal**: os bytes velhos dos tiles que o passo tocou. Então:

* `stroke_undo` **não guarda os planos de relevo** — nem elididos, nem clonados;
* no commit, o delta sai de `(journal, plano vivo)` **direto**, sem passar por um snapshot completo;
* no undo, a materialização parte do **plano VIVO** (a identidade do 3a: o vivo **é** o cursor em 92 de
  92, e a absorção a estabelece nos DOIS consumidores desde a §5.24);
* o cursor larga os planos de relevo pela mesma razão.

⚠️ **A promoção do journal para release NÃO pode landar sozinha:** ela paga **captura + fork** até o
fork morrer, o que é regressão pura. A troca é **atômica por plano** (§5.14: remover um dono de quatro
não compra milissegundo nenhum) e por isso **os três planos de relevo saem juntos**.

### O que fica pronto para o próximo degrau

Tudo o que a troca consome já existe e está gateado: o `TileJournal` com captura paralela e caminho
contíguo (§5.25), a proveniência `journal_since` que responde *"este journal descreve ESTE passo?"*
(§5.26), o oráculo `absent` vs `incomplete` (§5.29), a absorção nos dois consumidores (§5.24) e a
identidade do cursor medida em 233/233 (§5.28). **O que falta é escrever a troca** — uma edição do
ciclo de vida do `ModelSnapshot`, não uma otimização local.

### 5.58.1 🗺️ O MAPA da troca — os donos, onde cada um é posto, e a ordem

A leitura do código (não do plano) fecha três dos quatro donos e **deixa um por identificar**, que é
por onde o próximo passo começa.

| dono | onde é posto | como sai |
|---|---|---|
| **o tool** | os campos `heights`/`covers`/`mats` | irredutível — é o produto |
| **`paint.stroke_undo`** | `capture_shape_model()`/`snapshot_model()` no pen-down (`tool/paint.rs:215`) | elidir o relevo na captura do TRAÇO |
| **`cursor`** | `set_cursor(after.clone())` em `undo_record.rs:32/63/70`, **ANTES** do `split` (o comentário diz por quê: o cursor tem de ser o `after` COMPLETO) | elidir o relevo, e reconstruí-lo de `(vivo, journal)` — identidade medida em 233/233 (§5.28) |
| **a entrada do 1º traço** | `StoredPlane::Whole` — o 1º traço de uma camada CRIA o relevo, então não há `before` a diferenciar | não precisa sair: é **uma vez por camada** |

✅ **O quarto foi identificado, e ele estava no comentário da minha própria sonda** — a `who_holds…`
já dizia *"o PRIMEIRO traço numa camada CRIA os planos, então a entrada guarda `Whole`; do segundo em
diante é `Patch` e não segura plano nenhum"*, e a saída dela mede exatamente isso:

```text
  apos 1 traco(s), dentro do gesto: heights 4 · covers 4 · mats 4
  apos 2 traco(s), dentro do gesto: heights 3 · covers 3 · mats 3
  apos 4 traco(s), dentro do gesto: heights 3 · covers 3 · mats 3
```

⇒ **O regime é TRÊS donos** (tool · `stroke_undo` · `cursor`), e o quarto é um transiente do primeiro
traço de cada camada. *A nota que eu acabara de escrever (“não identificado”) sobreviveu ao fato por
uma sessão inteira* — o dado estava medido e comentado, e eu o re-derivei em vez de o ler.

⚠️ **E ele deixa um achado de MEMÓRIA que não é da troca e fica nomeado:** essa entrada retém o plano
INTEIRO (`Whole.after`) — a 4096² são 16 + 64 + 112 = **192 MB numa única entrada**, contra os 2,36 MB
que a §5.28 mede para um passo típico. Ela não é dona do plano VIVO depois do primeiro fork (o tool
troca de `Arc`), então **não custa milissegundo nenhum**; custa o cap de bytes do histórico, que a
evicta como qualquer outra. É o `OnlyAfter` do motor de delta chegando por outra porta.

**A ordem, e por que ela não é negociável:**

1. **identificar o quarto dono** (`who_holds_the_planes_when_a_stroke_begins` com uma ablação a mais);
2. **promover o journal para release** — ⛔ **não pode landar sozinha**, e agora com um segundo motivo
   além do custo: o `begin_step` é **no-op em release** e metade da API do journal só tem o AUDIT como
   consumidor, então a promoção sem o consumidor produz **dead-code que só aparece em `--release`** (a
   armadilha exata que a §5.25 já pagou, quatro warnings por três commits);
3. **o consumidor**: o `split` do relevo passa a sair de `(journal, plano vivo)` em vez de dois
   snapshots — o lado `before` do delta **é** o conteúdo do journal, e é isso que evita materializar
   (materializar produziria o `Vec` que a wave existe para não pagar);
4. **a materialização** parte do plano VIVO, o que exige `undo()`/`redo()` receberem o vivo — e o
   chamador **já o constrói uma linha acima** (`absorb_foreign_writes_now` faz `snapshot_model()`), então
   a mudança de assinatura não acrescenta trabalho;
5. **os três planos saem JUNTOS** (tudo-ou-nada por plano).

⚠️ **O AUDIT fica gateado.** Ele é a rede que confere a troca, e *uma rede de verificação não pode viver
no relógio do que ela observa* (§5.23) — a promoção é do **journal**, nunca do audit.

### 5.58.2 🧩 O DESENHO da troca, e os dois atalhos que a leitura do código matou

Com o mapa fechado (§5.58.1), o desenho tem de responder a UMA pergunta: *o `cursor` e o `stroke_undo`
largam o relevo — quem responde pelas perguntas que eles respondiam?* São três, e cada uma tem um
consumidor com um sítio exato:

| pergunta | consumidor | hoje | depois |
|---|---|---|---|
| *qual é o lado `before` do delta?* | `UndoEntry::split` (`undo_record.rs:33/66/71`) | `stroke_undo.relief` | **o conteúdo do journal** |
| *alguém escreveu no meio?* | `absorb_foreign_writes` (`undo_absorb.rs:44`) | `split(cursor, before)` | o **estado** do journal |
| *de que estado o undo parte?* | `UndoEntry::materialize` (`undo.rs:432`) | `cursor.relief` | o **plano VIVO** |

#### ⛔ Atalho 1, morto: *"reconstruir o cursor e seguir como está"*

A identidade do §5.28 (`cursor[i] == journal.get(i).unwrap_or(vivo[i])`) sugere materializar o cursor e
não mudar mais nada. **Ela é circular no lugar onde seria usada:** a materialização produz um `Vec`
novo — que é exatamente a cópia que a wave existe para não pagar —, e ela seria feita **a cada
`absorb_foreign_writes`**, isto é em todo `record_*` **e** em todo undo/redo. Trocar um fork por uma
materialização é trocar seis por meia dúzia (a mesma frase da §5.58, agora com o sítio).

#### ⛔ Atalho 2, morto: *"materializar só quando o journal não está vazio"*

O refinamento óbvio — journal vazio ⇒ clone de `Arc` (grátis); journal cheio ⇒ materializa (raro) — **é
falso na única cena que importa**: durante um traço o journal está SEMPRE cheio (o fold escreveu), então
o caminho caro é o caso comum, não o raro. *Um caminho rápido cuja condição é falsa no gesto que ele
existe para acelerar é código morto com todos os gates verdes* (a lição do ADR-0120, aqui de novo).

#### ✅ O desenho que sobra

**O relevo ganha um caminho de delta PRÓPRIO**, e é isso que o torna tratável: o canvas continua pelo
`split(before, after)` de dois snapshots (ele já está em UM dono dentro do gesto — §5.58 —, então não há
o que ganhar ali), e o relevo passa por `split_from_journal(journal, live, layer)`:

* o lado **`before`** são os tiles do journal — bytes que já existem, sem cópia nova;
* o lado **`after`** é a janela correspondente do plano VIVO;
* a **janela** é a união dos tiles tomados (o journal já a conhece: `taken`);
* *"alguém escreveu no meio?"* vira *"há tiles tomados fora da janela declarada do passo?"* — uma
  pergunta ao journal, não uma comparação de planos;
* a **materialização** aplica o patch ao plano VIVO, que o chamador já constrói uma linha acima
  (`absorb_foreign_writes_now` faz `snapshot_model()`).

⚠️ **E o guard de proveniência é o que mantém a política *lento nunca, errado jamais*:** se
`journal_describes_step_at(before.writes)` for falso, o journal não descreve ESTE passo e o relevo cai
no caminho de hoje — que só existe enquanto os snapshots ainda carregarem os planos. ⇒ **a elisão é do
caminho do TRAÇO** (onde o passo é aberto por construção), não de `snapshot_model()` em geral: uma
edição de camada continua carregando os planos, é user-paced, e é o fallback vivo em vez de um ramo
morto.

#### A ordem de landing, revisada

1. ~~identificar o quarto dono~~ ✅ (§5.58.1 — são três, e o quarto é transiente do 1º traço);
2. `split_from_journal` + a pergunta de escrita estrangeira pelo journal, **atrás do guard de
   proveniência**, com os snapshots ainda carregando tudo ⇒ **byte-idêntico, zero ganho, gateável**;
3. a materialização parte do vivo (`undo`/`redo` recebem o vivo, que o chamador já tem);
4. o journal sai do `cfg(debug)` **junto com** o `stroke_undo` e o `cursor` largarem o relevo — os três
   planos de uma vez. É aqui que os 9,6 ms caem, e é o único commit que muda um número.

⚠️ **O degrau 2 é a chave de tudo**: ele é *byte-idêntico por construção* (o delta que ele produz tem de
ser igual ao que o `split` de dois snapshots produz, e isso é um gate de igualdade, não uma promessa), e
com ele verde o degrau 4 vira mecânico.

---

## §5.59 — S3, degraus 2 e 3 CONSTRUÍDOS; e as três coisas que o degrau 4 não era (2026-08-01)

O §5.58.2 fechou com *"com o degrau 2 verde o degrau 4 vira mecânico"*. Os degraus 2 e 3 estão
construídos e gateados; **a frase sobre o 4 estava errada em três lugares**, e os três só apareceram
porque a construção os encontrou. Duas correções vieram de gates que nasceram vermelhos; a terceira, de
um censo.

### 5.59.1 ✅ Degrau 2 — o lado `before` do relevo vem do journal, e as duas rotas dão o mesmo delta

`StoredPlane::from_journal` / `StoredMap::from_journal` (filho novo `undo_delta_journal.rs`), atrás do
guard de proveniência, com os snapshots ainda carregando tudo. **Byte-idêntico, zero ganho, gateado.**
A lei é a identidade da §5.28, e é ela que torna a caixa de tiles utilizável:

```text
  before[i] == journal.get(i).unwrap_or(vivo[i])
```

⚠️ **A ORDEM no `record_structural_hinted` passou a carregar peso:** o split vem **antes** do
`set_cursor`, que ZERA o journal (os dois fatos são o mesmo fato). Ao contrário, o guard cairia no
caminho de sempre — em silêncio, com todos os gates verdes.

⚠️ **E a caixa de tiles sozinha REPROVOU no `measure_undo_capacity`.** Ela é 128-alinhada, então
arredonda o traço para fora em até 127 de cada lado: o passo típico saltou de **2,51 para 8,23 MB a
1024²**, e o delta passou a comprar 3,9× mais passos que um documento por endpoint em vez dos ~13×
medidos. **Com os endpoints materializados idênticos o tempo todo** — *conteúdo e memória são perguntas
separadas*, e cada uma ganhou gate próprio. A cura é cruzar a caixa com a janela DECLARADA: dois
superconjuntos, e a interseção ainda contém o escrito.

⚠️ **Duas armadilhas de FIXTURE, as duas minhas.** A 256² um tile mede meia tela ⇒ a caixa colapsa no
plano inteiro, as duas rotas caem no mesmo `Whole`, e **três mutações reais sobreviveram**. E **o relevo
não é escrito por dab** — o impasto é *por-traço*, e quem escreve os três planos é o **fold**, no
pen-up: a fixture que parava nos Moves media `SEM-RELEVO` e o gate acusava a rota como não-executada.

**5 gates · 7 mutações · 7 sangram**, cada uma no gate certo. O contador `RELIEF_FROM_JOURNAL` existe
para que nenhum deles possa ser verde por **vácuo** (as duas rotas caindo no mesmo fallback é a
armadilha do ADR-0120, que o oráculo de undo do ADR-0124 já pagou uma segunda vez).

### 5.59.2 ✅ Degrau 3 — e o QUARTO consumidor que a tabela do §5.58.2 não tinha

O `side` passa a receber **duas bases**: o cursor para os dezesseis planos de sempre, o **vivo** para os
três de relevo. Hoje as duas respostas são a mesma, então a troca é byte-idêntica; o que ela compra é o
cursor poder largar o relevo.

⚠️ **A tabela dizia que `UndoEntry::materialize` passa a partir do vivo. Ele tem DOIS chamadores, com
adjacências diferentes:**

| chamador | a entrada é adjacente a… | base do relevo |
|---|---|---|
| `undo` / `redo` | o estado de AGORA | **o vivo** |
| `absorb_foreign_writes` · extensão de run coalescido | o cursor ANTIGO | **o cursor** |

E no `absorb` o vivo difere do cursor **por construção** — é exatamente a divergência que ele existe
para reconciliar. A primeira versão passava o vivo nos dois: o `debug_assert` novo nasceu **vermelho em
três gates** (`warp::relief_tests` + os dois filtros do sculpt) e nomeou o sítio. *A base é o estado
ADJACENTE à entrada, e nem sempre ele é o de agora.*

### 5.59.3 ⛔ O degrau 4 não é mecânico — a elisão joga fora a ENTRADA do fallback

A política que sustenta todo o S3 é *lento nunca, errado jamais*: o guard recusa e o commit **cai no
caminho de sempre**. ⚠️ **Essa política não sobrevive à elisão**, e o motivo é de ordem no tempo:

* o guard é perguntado no **COMMIT**;
* a elisão acontece no **PEN-DOWN** (é ela que tira o dono, e é por isso que ela é a wave);
* o caminho de sempre precisa de `before.relief` — **que a elisão acabou de descartar**.

⇒ Um passo em que o guard recuse teria um `before` sem relevo e um `after` com: o `split` clássico
devolve `OnlyAfter`, e desfazer aquele passo **remove o relevo** em vez de o restaurar. Silenciosamente.
É a mesma classe do buraco que o `mats` fora do `ModelSnapshot` custou em 2026-07-13.

**Logo o guard tem de ser decidível no pen-down, ou o commit tem de RECUSAR** (descartar o histórico,
como o `side` já faz com um cursor incoerente). Não há terceira saída: um `Whole` com `before = after`
seria a mesma perda, vestida de delta.

### 5.59.4 📊 O censo que redesenha o guard — 774 passos, ZERO indescritíveis

`PH2D_UNDO_AUDIT=1` sobre a suíte inteira (`--lib`, ~5 s em debug, todo gesto que algum teste encena):

| estado do journal de relevo | passos |
|---|---|
| **DESCREVE** (`relevo/PASSO`) | **307** |
| **SEM-RELEVO** | **467** |
| MISTURADO | **0** |
| INCOMPLETO | **0** |

Três leituras, e a segunda é a que muda o desenho:

1. **Nenhum passo da suíte é indescritível.** O caminho de recusa existe e não é exercido — o que
   torna a política de *refusar no commit* barata de verdade, em vez de um regresso a esperar.
2. ⚠️ **A MAIORIA dos passos não escreve relevo, e hoje o guard os RECUSA.** O `speaks_for` exige
   `layer == Some(l)`, então um passo que não tocou relevo nenhum cai no fallback — que, com o `before`
   elidido, é justamente o caminho que perde a edição. *"Este passo não escreveu relevo"* é uma
   descrição perfeitamente boa e o journal já a tem: o guard tem de aceitar `layer.is_none()` e
   responder **`Unchanged` em toda chave**. Sem isso a elisão quebra em 467 dos 774 passos.
3. O `absorb` ganha a sua metade de graça: os journals estão **intactos** quando ele roda (o
   `begin_undo_step` só os zera *depois* dele), então *"alguém escreveu relevo no intervalo?"* é
   `relief journals vazios?` — a pergunta que a tabela do §5.58.2 já mandava fazer, agora com o sítio.

### 5.59.5 A ordem do degrau 4, revisada

1. o guard aceita **"nada escrito"** (`layer.is_none()` ⇒ `Unchanged` em toda chave) — com gate próprio,
   porque são 467 dos 774 passos e hoje eles caem no fallback;
2. `ModelSnapshot::without_relief` + os dois sítios de elisão (`stroke_undo` no pen-down · o `cursor`);
3. o `absorb` pergunta o relevo ao **journal** e, no re-split, **adota o delta de relevo da entrada
   velha** (o escorrido é do CANVAS; o relevo do topo não mudou — e isso é *conferido*, não assumido);
4. a mesma cirurgia na extensão de run coalescido;
5. o commit **RECUSA** quando o guard falha sobre um `before` elidido;
6. o journal sai do `cfg(debug)` — junto, nunca antes (§5.58.1);
7. gates: a sonda de donos vira gate (vai a **1**), o comportamental *pinte · desfaça · refaça — a tinta
   **e o relevo** voltam iguais*, e a razão do fold.

⚠️ **O prêmio segue lá:** o `what_the_two_halves` mede o fold em **12,33 ms a 4096²** contra os 11,92 do
§5.58 — mesma ordem, e a máquina estava com `load average 22`, então **não é um A/B limpo** (§5.49). O
censo de donos, que não é relógio, confirma o resto: `heights/covers/mats` a **3** dentro do gesto, e só
as duas elisões juntas levam a **1**.

> ⚠️ **CORREÇÃO (§5.60): a última frase está ERRADA.** As duas elisões juntas levam a 1 **apenas se a
> rota do journal produzir `Patch`** — com o `before` elidido e a rota recusando, o `split` cai em
> `OnlyAfter`, que guarda um `Arc` **forte do plano vivo**: o dono não sai, ele troca de lugar. Medido
> na sonda `what_each_owner_of_the_relief_costs_at_pen_up`, e o mecanismo está no
> `undo_delta_journal.rs:234`.

### §5.60 — E O DEGRAU 4 FOI RE-DESENHADO PELA TERCEIRA VEZ: elidir o `before` NÃO remove um dono, TROCA o dono de lugar — e "elidido" é indistinguível de "não existia"

> ⚠️ **CORREÇÃO (§5.61 — a wave FOI construída e mediu):** duas afirmações desta seção não
> sobreviveram à construção, e a §5.61 traz o número de cada uma.
>
> 1. *"Aceitar `layer.is_none()` como 'nada mudou' é INVÁLIDO"* — **é VÁLIDO com a testemunha.** O que
>    tornava o silêncio ambíguo era não haver como saber se alguém trocara o plano por fora; um `Weak`
>    por plano responde isso. Sem a correção, **64 gates** caíam no descarte.
> 2. *A testemunha `ptr_eq` como recusa do caminho quente* — **estrita demais, medido:** o próprio fold
>    substitui o `Arc` em todo traço de impasto, e exigi-la reprovava **58 gates**. Ela ficou onde é
>    necessária *e* suficiente: quando o journal está CALADO sobre aquele plano — e ali pegou o eraser
>    ao vivo.
>
> A ordem em 6 passos ao fim desta seção foi **seguida**, e o passo 3 escondia o custo real: não era o
> alocador (hipótese refutada por medição), era o **detector da absorção** disparando em todo pen-down.


O §5.59 fechou com a frase *"só as duas elisões juntas levam a 1"*, e ela está **ERRADA**. Indo
construir o passo 1 da ordem revisada, três medições — nenhuma delas um relógio — derrubaram o desenho
outra vez. As três ficam escritas porque **cada uma sozinha faria o degrau 4 perder relevo em silêncio**,
que é o único modo de falha que este módulo não aceita.

**(1) O guard não pode inferir "nada mudou" de "nada foi capturado".** O passo 1 dizia: *o guard aceita
`layer.is_none()` e responde `Unchanged` em toda chave* (467 dos 774 passos do censo). A premissa é que
**toda** escrita de relevo passa por uma porta que captura — e ela é uma **ENUMERAÇÃO**, que apodreceu:
o `grep` acha DUAS escritas de produção que substituem o plano inteiro sem passar por porta nenhuma —
o **eraser** (`impasto.rs:475`, `heights.insert(active, Arc::new(field))`) e o **reset do warp**
(`warp/relief.rs:200`, `heights.insert(layer, pre_h)`). Um passo que só faça um dos dois deixa os
journals **silenciosos** com o relevo trocado por inteiro. *A cura não é listar os dois: é uma
TESTEMUNHA* — e ela existe de graça, porque os dois **substituem o `Arc`**, então `Arc::ptr_eq` os
detecta em `O(camadas)`. O que a torna inaplicável hoje é o (2).

**(2) `OnlyAfter` segura um `Arc` FORTE do plano VIVO.** Medido pela sonda nova
(`what_each_owner_of_the_relief_costs_at_pen_up`), a coluna que decide **não é o relógio, é a de
DONOS**: a configuração *"sem o relevo do BEFORE"* segue com **3 donos** — ela não removeu dono nenhum.
O mecanismo está no `undo_delta_journal.rs:234`: com o mapa `before` vazio, **toda** chave cai em
`(None, Some(a)) => StoredEntry::OnlyAfter(Arc::clone(a))`. Antes o fork copiava porque o `before`
segurava o plano ANTIGO; depois copiaria porque a ENTRADA segura o de agora. ⇒ **A rota do journal
deixa de ser otimização e passa a ser PRÉ-REQUISITO** da elisão: só o `Patch` dela extrai uma janela em
`Vec` e não segura `Arc` nenhum (o ramo `Whole` da mesma rota **também** clona o vivo, então o prêmio é
condicional à janela ser pequena — o caso normal de um traço).

**(3) E o grave: `OnlyAfter` SIGNIFICA "este plano não existia antes".** Desfazer uma entrada assim
**REMOVE a chave**. Com o `before` elidido isso deixa de ser verdade e vira o oposto do que aconteceu:
*o undo apagaria o relevo anterior*. É a doença do §5.59.3 uma camada abaixo — **a elisão faz o motor
confundir *"não existia"* com *"eu não te contei"***. ⇒ O degrau 4 exige um **TERCEIRO estado** no
`ModelSnapshot` (presente · ausente · **elidido, pergunte ao journal**), e não um ajuste no guard. O
candidato natural é o **`Weak`**: ele é distinguível de ausente (a chave existe), **não** força cópia
(§5.12/§5.15 mediram `make_mut` com só um `Weak` vivo em 0,0000 ms, e o `fork_par` pergunta
`strong_count > 1` desde a §5.15), e **falha ao dar upgrade exatamente quando o plano foi substituído
por inteiro** — que é a testemunha que o (1) pediu, de graça e sem lista.

⚠️ **A tabela de relógio desta sessão NÃO decide nada e não é citada como ganho:** ela saiu incoerente
(ablações que só podem tirar trabalho medindo *mais* — 23,35 → 27,23 → 33,25 ms a 4096²) com a máquina
em `load average` 4-8. O que decide aqui é forma e contagem, e é só isso que está escrito acima.

**A ordem revisada do degrau 4, agora com o desenho corrigido:**

1. o `ModelSnapshot` ganha o terceiro estado do relevo (`Weak`), com gate provando que ele é
   **distinguível de ausente** — a mutação que o colapsa em ausente tem de fazer o undo apagar relevo;
2. `from_journal` aprende o estado: elidido + journal ⇒ `Patch`; elidido + upgrade FALHOU (substituição
   wholesale) ⇒ **recusa**, e quem recusa no commit tem de ter o que instalar (o §5.59.3);
3. só então as duas elisões (o `cursor` — que o degrau 3 já deixou sem leitor — e o `before`);
4. o journal sai de `cfg(debug)` **junto** com elas, nunca antes (§5.58.1), e o `expect(dead_code)` do
   `ReliefSource` vira erro e sai;
5. os gates: a sonda de donos vira gate (tem de ir a **1**), o comportamental *pinte · desfaça · refaça
   — a tinta **e o relevo** voltam iguais*, e a razão do fold;
6. re-medir com a máquina calma (`load average` < 5), porque nenhum número desta sessão serve.

---

### §5.61 — O DEGRAU 4 FECHOU: o relevo é DESCRITO em vez de segurado, os donos caem de 3 para 1 e o pen-up cai 4× — mas a causa do custo que quase matou a wave não era a que eu escrevi

**Estado: construído, medido, verde nos dois perfis. Pendente de smoke.**

O prêmio que a §5.16 nomeou e a §5.60 re-desenhou três vezes: o `cursor` era o **segundo dono
permanente** dos três planos de relevo (§5.14: dois donos em repouso, `undo.clear()` levava a um), e o
`before` do traço era o terceiro. Com três donos, a 1ª escrita de todo traço **copiava o documento**.

Medido pela porta do produto, **costas-com-costas na MESMA corrida** (a máquina é compartilhada e um
A/B cross-run atribuiria a deriva dela ao ganho — §5.46), com a ablação pela **ENTRADA**
(`UndoController::elide_relief` / `elide_cursor`, `cfg(test)`):

| tela | elide | pen-down | pen-up | donos |
|---|---|---|---|---|
| 2048² | nenhum | 3,43 | 10,54 | 3 |
| 2048² | **os DOIS** | 3,43 | **3,94** | **1** |
| 4096² | nenhum | 5,77 | 22,22 | 3 |
| 4096² | só o BEFORE | 5,86 | 22,29 | 2 |
| 4096² | só o CURSOR | 5,68 | 21,31 | 2 |
| 4096² | **os DOIS** | 5,65 | **5,57** | **1** |

**pen-up 22,22 → 5,57 ms a 4096² (3,99×)** e **10,54 → 3,94 a 2048² (2,68×)**, com o pen-down
**intocado**. ⚠️ **E só as duas elisões JUNTAS entregam:** nenhuma sozinha move o pen-up (21,31 e 22,29
contra 22,22 do controle) — é a §5.59 (*"só as duas elisões juntas levam a 1"*) confirmada pelo relógio
depois de o ter sido pela contagem.

#### O TERCEIRO estado, e por que ele não é cerimônia

`ModelSnapshot` ganhou `relief_elided` (`crate::undo::elide::ElidedRelief`): três
`BTreeMap<LayerId, Weak<Vec<T>>>`. **Três estados e não dois** — *presente* (a chave está no mapa),
*ausente* (não está) e **ELIDIDO** (não está, e este campo diz que ele EXISTIA). A §5.60 mediu o que
acontece sem ele: toda chave cai no braço `(None, Some(a))` do `from_journal`, que **significa**
`OnlyAfter` = *"não existia antes"* ⇒ desfazer REMOVE a chave e **o undo apaga o relevo**.

⚠️ **O `Weak` não conta como dono** — `Arc::make_mut` só copia com outro *strong* (§5.12 mediu 0,0000 ms
com só um `Weak` vivo) e a porta de fork pergunta `strong_count > 1` (§5.15). É essa propriedade que
torna a elisão um ganho em vez de um rearranjo.

#### TRÊS afirmações minhas que a construção derrubou

**(1) O `speaks_for` aceitando um journal SILENCIOSO é VÁLIDO — a §5.60 o marcou como inválido, e o que
mudou foi a testemunha.** Um passo que não tocou relevo tem `layer == None`, e recusá-lo fazia **64
gates** caírem no descarte. O silêncio era ambíguo (*nada mudou* × *alguém trocou o plano por fora*);
com um `Weak` por plano ele vira afirmação verificada.

**(2) A testemunha `ptr_eq` como recusa do caminho quente é ESTRITA DEMAIS, e isso está medido:** o
próprio fold (`commit_stroke_height`) monta um plano novo e o INSERE, então o `Arc` muda de identidade
em **todo traço de impasto** — exigi-la reprovava **58 gates** do caminho normal, com os bytes
perfeitamente cobertos pela captura. Ela ficou onde é necessária *e* suficiente: **quando o journal
está calado sobre aquele plano**. Nessa forma ela pegou o **ERASER** ao vivo, com a suíte verde em tudo
o mais — o sítio que a §5.60 nomeara por leitura de código.

**(3) A base do `materialize` do TOPO não é o vivo.** Passar `after` ali fez o gate
`a_coalesced_run_recomposes_the_delta` sangrar com `32.0` onde o estado adjacente tinha `26.0` — *a
base é o estado ADJACENTE à entrada, e nem sempre ele é o de agora*, que é literalmente o que o
`undo_planes.rs` já avisava. A porta única `base_for_top` devolve o **cursor REIDRATADO** pelas
testemunhas, e só quando o journal diz que ninguém escreveu relevo desde o commit (`relief_untouched`);
senão recusa, e a absorção e o run coalescido caem nos seus early-outs de sempre.

#### ⛔ A hipótese que quase virou a explicação — REFUTADA por medição

A primeira medição da fase B deu **pen-down 5,70 → 36,2 ms a 4096²**: a wave seria uma **PERDA líquida
de ~15 ms por traço**. A explicação natural era o alocador — a elisão passou a LIBERAR ~200 MB por
commit, e o `impasto.rs` **já documenta** que `vec![0.0; n]` só é barato enquanto o `alloc_zeroed`
recebe páginas frescas do SO (a troca por `clear() + resize` foi medida e reprovada em 2026-07-25:
17,6 → 47,5 ms). A hipótese fechava com o mecanismo, com a escala (≈ 4× a área) e com a aritmética.

**E está errada.** Segurar os planos liberados num `Vec` (a configuração `os DOIS + PIN` da sonda)
**não devolveu um milissegundo** — 36,23 → 37,53. *Repita antes de explicar.*

A causa era o **detector da absorção**. `absorb_foreign_writes` compara o cursor com o `before` pelo
`PlaneDeltas::split`; com o cursor ELIDINDO o relevo e o `before` segurando-o, **toda camada saía como
`OnlyAfter`** = *"apareceu agora"*, `heap_bytes()` nunca era zero, e a absorção fazia um re-split +
materialize completos **em todo pen-down**. O relevo saiu do detector, com o porquê escrito: aquela
porta existe para a escrita de **canvas** que não registrou entrada (o escorrido do Wet Paint), e
escrita de relevo fora da história é outra pergunta — quem a responde é o journal, pelo `base_for_top`.

#### E quando a aposta é perdida, a história é DESCARTADA

Elidir é apostar que o journal descreverá o passo. Perder a aposta significa que o estado de antes
**não existe em lugar nenhum** — nem na entrada, nem no tool, que o sobrescreveu no lugar. Guardar a
entrada assim mesmo seria o pior dos desfechos (ela sairia dizendo `Unchanged`/`OnlyAfter` e desfazê-la
**apagaria** o relevo, em silêncio). `discard_if_relief_is_lost` descarta, com o readout do journal ao
lado (*misturado · incompleto · camada errada · plano trocado* pedem correções diferentes) e um
`debug_assert` que a torna LOUD na suíte inteira. Os dois sítios que trocam o plano por inteiro agora
CAPTURAM — é a cura de projeto; a testemunha é a rede para o sítio que ninguém listou.

#### A promoção levou os dois JUNTOS

O journal do **RELEVO** e a proveniência saíram de `cfg(debug)` no mesmo commit que a elisão (§5.58.1 —
promover o journal sozinho paga captura *e* fork até o fork morrer). O journal do **CANVAS fica em
debug**: o `before` do canvas não é elidido, então capturar dele em release seria custo sem
contrapartida. O `expect(dead_code)` do `ReliefSource` virou erro e saiu **exatamente como ele próprio
prometia** — um `allow` teria sobrevivido calado.

#### Os gates, e o que o terceiro deles ensinou

- `an_elided_before_is_not_read_as_a_layer_that_had_no_relief` — o central. A mutação (colapsar o
  terceiro estado no segundo) faz o undo apagar o relevo **e a tinta volta certa**, então nenhum gate de
  pigmento pisca: este é o único que vê.
- `nobody_but_the_tool_holds_the_relief_planes` — sem relógio, logo sem ruído: a pergunta é
  `Arc::strong_count`, a MESMA que a porta de fork faz. Mutação: `(2,2,2)` contra `(1,1,1)`.
- `a_clean_pen_down_does_not_wake_the_absorption` — ⚠️ **o oráculo dele nasceu VAZIO.** A 1ª versão
  afirmava que `undo_depth()` não mudava, e a absorção faz **pop + push**: a profundidade não muda nem
  quando ela dispara, então o gate **não podia falhar pelo motivo que alegava**. A mutação SOBREVIVEU e
  o denunciou. A pergunta é *"ela disparou?"* e a forma honesta de a fazer é **contar** (`ABSORB_FIRED`,
  o idioma do `RELIEF_FROM_JOURNAL`); assim a mesma mutação sangra 1 contra 0.

**3 mutações, 3 sangram.** Suíte **942/0 debug · 944/0 release**, clippy limpo, LOC 2/2, contrato
congelado 4/4, fingerprint do ADR-0134 **3/3**, `PROJECT_SCHEMA` **intocado**.

#### Aberto, com o número ao lado

- O **pen-down segue sendo uma cópia de canvas** (§5.16 o pinou; ele não era o alvo desta wave e não se
  moveu: 5,65 ms a 4096²). A captura do "antes" por REGIÃO — o *tile-based undo* — é o que o fecha, e
  ela quer a porta única de escrita de canvas.
- O `relief_indescribable` **nunca disparou** na suíte depois das duas capturas. Se ele aparecer num
  caminho real, é o desenho da elisão que está errado, não o guard — e o readout diz qual das quatro
  causas foi.

### §5.62 — O SMOKE APROVOU O S3 E TROUXE UM OUTLIER DE 71 ms: a elisão foi EXONERADA por medição, e o que sobrou é o QUADRO DEPOIS DO CTRL+Z, plano na tela

O smoke do S3 (2026-08-01) voltou **aprovado no comportamento** — *"smoke OK em Undo/Redo do impasto"*,
a tinta e o relevo voltam iguais — com um `[paint-perf]` trazendo:

```
dispatch max=71.2 [preview 71.2 panel 0.0 overlay 0.0 upload 0.0]
WORST: GPU 4096x4096 branch=idle impasto=true trivial=true
90f GPU 54/CPU 36 | frame p50=16.6 | dispatch p50=0.0
```

**100% em `preview`, num quadro `branch=idle`** — a assinatura de um re-fold do canvas inteiro (a mesma
da §4.8.2), e o gesto que o smoke acabara de exercitar é justamente o que o força: um **Ctrl+Z**
reinstala os três planos de relevo, e o passe de luz tem de reler a pintura toda.

#### A pergunta não era *"quanto custa"*, era *"a elisão moveu isso?"*

Ablação pela **ENTRADA** (`elide_relief` / `elide_cursor`), **costas-com-costas na MESMA corrida** —
sonda nova `measure_undo_cost::what_a_ctrl_z_costs_with_and_without_the_elision`. Duas metades
cronometradas **separadas**, porque o número do log vive no QUADRO e não na chamada de undo.

| tela | elide | undo | +frame | redo | +frame | donos |
|---|---|---|---|---|---|---|
| 2048² | nenhum | 4,98 | 97,67 | 7,80 | 96,96 | 2 |
| 2048² | os DOIS | 3,88 | **95,91** | 3,93 | 97,53 | **1** |
| 4096² | nenhum | 30,23 | 381,27 | 29,36 | 381,01 | 2 |
| 4096² | os DOIS | 31,34 | **393,68** | 31,45 | 391,44 | **1** |

⚠️ **A leitura honesta é que não há sinal, e ela vem de DUAS corridas com SINAIS OPOSTOS:** na primeira
o braço "os DOIS" saiu **mais rápido** (385,35 contra 387,12), na segunda **mais lento** (393,68 contra
381,27). A dispersão entre corridas (~±3%) é maior que qualquer diferença entre os braços ⇒ **a elisão
não toca o caminho do Ctrl+Z**. E o controle disparou nas duas (donos **2 → 1**), então a ablação era
real: não é um A/B que mediu o mesmo caminho duas vezes.

#### O que a sonda achou no lugar, e o controle que o torna um achado

O irmão `measure_the_idle_tick` mede o MESMO `paint_tick` + dreno com nada sujo: **Impasto 4096² =
0,000 ms**. Contra isso:

| | tick ocioso | quadro depois de um Ctrl+Z |
|---|---|---|
| 2048² | 0,000 | **97,7** |
| 4096² | 0,000 | **381,3** |

**3,90× para 4× de área** ⇒ **plane-bound**: a forma exata de uma varredura de canvas inteiro, e não do
trabalho que o gesto de fato mudou. E ele é **13× a chamada de undo** (29 ms), que é onde eu teria
olhado se tivesse somado as duas metades — daí medi-las separadas.

⚠️ **Os 381 ms (CPU) e os 71,2 ms (GPU do log) são o MESMO evento em produtores diferentes, não o mesmo
número.** A sonda roda no compositor de CPU (um teste de unidade não tem device); o `WORST` do smoke diz
`GPU`. Afirmar que um é o outro seria vender o que não foi medido — o que os dois compartilham é a
FORMA (`preview` inteiro, `branch=idle`, plano na tela).

#### É PRÉ-EXISTENTE, e a próxima alavanca ficou disponível por acidente feliz

O braço "nenhum" — o mundo **antes** do degrau 4 — paga os mesmos 381 ms. Nada desta wave o criou.

⚠️ **E o mecanismo já tem cura nomeada:** um undo marca tudo sujo porque, em tese, ele *pode* mudar
qualquer coisa — mas **o delta SABE a própria janela**, e o S3 foi precisamente a wave que a tornou
explícita (`PlaneDeltas` a guarda; o degrau 2 a fez a fonte do `before`). Publicar **a janela que o
passo reescreveu** em vez do canvas inteiro é a continuação natural, e não é contrabando dentro de uma
wave fechada: é wave própria, com smoke próprio, porque muda o que a tela repinta.

### §5.63 — O UNDO PUBLICA A JANELA QUE ELE REESCREVEU: o quadro depois de um Ctrl+Z cai 381 → 3,2 ms e fica PLANO na tela

A §5.62 mediu o quadro pós-Ctrl+Z em **97,7 ms a 2048² e 381,3 a 4096²** contra **0,000** de um tick
ocioso, e nomeou a cura: *o delta SABE a própria janela, e o S3 foi a wave que a tornou explícita.*

**Medido pela mesma sonda, com o `meio-traço` servindo de controle interno:**

| tela | pós-undo antes | pós-undo depois | meio-traço (controle) |
|---|---|---|---|
| 2048² | 97,19 | **3,11–3,23** | 0,58–0,62 |
| 4096² | 386,74 | **3,11–3,23** | 0,58–0,62 |

⚠️ **O número que prova a wave não é o 124×, é a IGUALDADE das duas linhas.** Um custo plano na tela é a
forma de trabalho limitado pela PEGADA; enquanto ele quadruplicava com a área, nenhuma constante o
salvaria.

#### A prova de confinamento tem duas metades, e as duas são destructure EXAUSTIVO

* **os PLANOS** (`PlaneDeltas::confined_region`) — os dezenove são desestruturados sem `..`, então um
  plano novo **não compila** até ser classificado.
* **os METADADOS** (`ModelSnapshot::confined_to`) — os vinte e cinco campos, idem. Um passo pode não
  tocar pixel nenhum e mudar a figura em toda parte (opacidade, blend, ordem, visibilidade): a cerca de
  Chesterton do `invalidate_composite` **é estreitada, não derrubada**.

A assimetria é o argumento inteiro: reivindicar **de menos** custa um repaint; reivindicar **de mais**
deixa a figura anterior na tela **e nenhum gate de conteúdo pega**, porque dentro do retângulo está tudo
certo. Por isso `PlaneReach::Whole` é o default de todo caso que o módulo não sabe descrever, e por isso
o gate central é um **ORÁCULO** — duas telas comparadas byte a byte, uma com o confinamento ablacionado.

#### TRÊS defeitos, e nenhum foi achado lendo código

1. **`to_region` exigia alinhamento a pixel e devolvia `None` para todo traço.** O `diff_window` acha o
   primeiro e o último **ELEMENTO** que diferem; num plano RGBA são quatro por pixel, e um passo que
   muda um canal produz `col` que não é múltiplo de 4. A cura é arredondar **para FORA** — um dirty rect
   tem de ser SUPERCONJUNTO —, e o gate afirma **contenção**, nunca igualdade.
2. **Dois planos VAZIOS liam como `Whole`.** O `split` manda para `Whole` tudo o que não sabe medir, e
   `fits()` recusa comprimento zero; as **seis** superfícies da sessão de Sculpt são vazias num traço de
   pigmento comum. Isso era `Whole` conflando *"mudou em toda parte"* com *"não sei medir"* — e um plano
   vazio não é nenhum dos dois. **Azedava TODO passo do produto** (`spre=WHOLE samt=WHOLE …` em cada
   entrada).
3. **`restore_selection` invalidava o composite incondicionalmente.** Com (1) e (2) curados o
   confinamento disparava, o retângulo era publicado — e a drenagem seguia em `FullComposite`, com o
   quadro em 381 ms. Achado por **backtrace**, não por leitura.

#### ⚠️ Quatro lições, todas sobre mim

* **A primeira leitura disse "PIOROU" (97 → 182 · 386 → 414) e era a MÁQUINA** (`load average 23`). O
  que torna uma leitura confiável não é o número absoluto: é o **controle interno** — o `meio-traço`, que
  ficou em 0,58 o tempo todo. Sem ele eu teria revertido uma wave correta.
* **O meu scan de callees achou a função ERRADA:** procurei `fn restore_selection` e o grep casou com
  `restore_selection_shapes`, noutro arquivo, reportando *"não invalida"*.
  [[feedback_a_negative_search_needs_a_positive_control]].
* **Duas de cinco mutações sobreviveram, e as duas acusaram os meus GATES.** A do arredondamento passou
  porque a janela que o produto produz **calhava de ser alinhada** — a fixture não continha o fenômeno,
  e a cura foi um gate de unidade com janelas construídas para o conter. A dos metadados passou porque o
  gate estrutural mudava **só** a opacidade: sem escrita de canvas os planos já recusam sozinhos, então
  ele era **verde por vácuo sobre a metade que dizia julgar**.
* **O instrumento mora DENTRO do gate que precisa dele.** `confine_diagnosis`/`confine_report` ficaram
  sem chamador quando a sonda de diagnóstico foi removida — cinco warnings de código morto. A cura não
  foi apagá-los: eles entram nas **mensagens de falha**, e a próxima pessoa recebe a resposta em vez de
  reconstruir o instrumento.

**5 gates, 5 mutações, 5 sangram.** Suíte **948/0 debug · 950/0 release**, shell verde, clippy limpo,
LOC 2/2 nos dois gates, contrato congelado **4/4**, fingerprint do ADR-0134 **3/3**, `PROJECT_SCHEMA`
**intocado** (zero arquivos de shell tocados).

#### Aberto, nomeado

- **O PRIMEIRO traço de relevo numa camada não é confinado** e isso é correto: os planos entram como
  `OnlyAfter` (o plano inteiro nasce), e restaurar isso muda o relevo onde quer que ele tivesse
  cobertura — não há retângulo que descreva. Todo traço seguinte confina.
- O `bump_all_layer_pixels` **continua bumpando todas as camadas** num passo confinado. Na pista de CPU
  isso não custa nada (o número acima), e num documento de uma camada — o caso comum e o do smoke — ele
  É o bump da ativa. Estreitá-lo é ganho de pista **GPU**, e quem decide é o próximo `PH2D_PAINT_PERF`.

### §5.64 — "DEPOIS DE VÁRIOS TRAÇOS O IMPASTO FICA LENTO": três hipóteses minhas, três refutações, e o divisor que o log passou a publicar

Report do Enio (2026-08-01) com log. ⚠️ **O log já isenta o dispatch** (`max=12,4`, e `0,0` nas duas
janelas seguintes) e nomeia o balde: **`INPUT (fora do frame) p50=0,0 max=1016,5 ms`** — mediana grátis
e **UM evento de um segundo**, com `período real 73,6 ms/frame` contra `frame p50=16,1`.

⛔ **TRÊS hipóteses, TRÊS rejeições por medição — nenhuma confirmada:**

1. **O(histórico)** (*"depois de vários"* é a assinatura). **Refutada:** 80 traços a 4096², histórico de
   201 → **375 MB**, e pen-down/moves/pen-up ficam em **~7 ms cada, PLANOS**. Nenhum pico, inclusive
   depois de o cap de bytes morder.
2. **A CADÊNCIA do quadro** (a 1ª sonda nunca drenava o preview, e §5.49 mostrou que o handshake por
   quadro muda o custo por evento). **Refutada** por ablação 2×2: com dreno e sem dreno, **4,31 × 4,33 ·
   5,40 × 5,69 · 12,33 × 12,84 ms** nos raios 40/100/200. O dreno custa **zero**.
3. **O PINCEL grande.** Real, mas pequeno demais: o pior evento sobe para **40 / 71 / 66 ms** nos raios
   100 / 200 / 300 — e são **pen-down e pen-up**, não moves. Fica a **14× de distância** dos 1016 ms.

⚠️ **E a 2ª sonda quase virou uma atribuição errada:** ela mudou raio **e** cadência ao mesmo tempo e
reportou *"40-71 ms contra os 7 de antes"*, que se lê como *"a cadência custa 10×"*. Uma diferença
medida com dois fatores é uma diferença **sem dono** — o 2×2 é que separou, e a resposta era o outro
fator. *Ablação com confundidor produz um culpado plausível e errado.*

#### O que ficou: o log ganhou o DIVISOR que faltava

`INPUT p50=0,0 max=1016,5` admite **"um pen-up custa um segundo"** e **"um move custa um segundo"** —
e as duas pedem curas **opostas** (o commit do histórico × o carimbo de dabs). É literalmente a doença
da §5.48 (`stamps` sem as entregas) um sistema adiante, e a cura é a mesma: **publicar a fase**.

```
INPUT (fora do frame) down 0.0/0.0 move 0.0/0.0 up 0.0/0.0 ms (p50/max)
```

`input_ms` virou `[f32; 3]`, o mapeamento mora **ao lado do enum que ele produz** (o sítio de chamada
está no teto de 600 LOC), e o gate é sobre a **SEPARAÇÃO**, não sobre o total: três baldes que somam no
mesmo lugar são um balde só com três nomes. **Mutação (todo evento no balde `Move`): sangra.**

⚠️ **O que NÃO foi feito, e por quê:** nenhuma cura foi escrita. O maior evento que a sonda reproduz é
~71 ms e o produto marcou 1016 — **construir sobre esse vão seria escolher um culpado por eliminação**,
que é exatamente o que as três refutações acima mostram dar errado. O próximo log nomeia a fase, e a
fase escolhe a wave.

## §5.65 — E a fase era o PEN-UP: a fixture não continha o fenômeno (2026-08-02)

O divisor da §5.64 respondeu no primeiro log: **`INPUT (fora do frame) down 0,0/17,1 · move 0,0/106,1 ·
up 0,0/512,9 ms`**. É o pen-up, contra os **5,57 ms** que a sonda do S3 mede a 4096².

⚠️ **Duas ordens de grandeza não são deriva de máquina.** Toda sonda desta família usa o
`measure_stroke_owners::stroke`, que vai de `x=60` a `x=260` — **200 px numa tela de 4096, 0,1% da
área** — e o artista atravessa a tela. É a fixture certa para *quem segura os planos* e a errada para a
pergunta do artista ([[reference_topic_fixture_discipline]]).

### O que a fixture certa mediu

`what_a_pen_up_costs_as_the_stroke_crosses_the_canvas` imprime **a janela do delta AO LADO do relógio**:
um relógio sozinho diria *"ficou lento"*, o par diz *por quê*. E a 1ª linha da tabela é o traço de 200 px
como **CONTROLE INTERNO** — sob máquina carregada nenhum número absoluto se defende sozinho; o que se
defende é ele ficar onde sempre esteve enquanto as linhas de baixo explodem (a lição que salvou a wave da
§5.63).

| 4096², impasto, r=40 | pen-up | janela |
|---|---|---|
| 200 px (o controle) | 5,98 | 0,4% |
| 1200 px reto | 18,67 | 1,6% |
| 3896 px reto | 44,90 | 4,9% |
| 1200 px **diagonal** | 68,45 | 11,3% |
| **3896 px diagonal** | **656,20** | ~98% |

**O custo é linear na ÁREA DA JANELA** (11,3% → 68 · 98% → 656).

### As duas metades, e depois os dois knobs

Ablação pela ENTRADA (`paint.stroke_undo = None` faz o `close_stroke` pular o commit estrutural): **fold
348 · commit 269**. Dentro do fold, ablação pelos dois controles que o artista de fato tem (Smoothing e
Push — knobs do painel, **nunca instrumentação**, para a sonda não ficar cega à porta):

| 4096², diagonal | ms |
|---|---|
| fold como shipa | 343,25 |
| sem Smoothing | 149,14 ⇒ **o `settle` custa 194,11** |
| sem Push | 343,55 ⇒ o Push **default é 0** |
| os dois em zero | **147,59** (o piso) |

⚠️ E a coluna do Push é o **controle que a tabela trouxe sem eu planejar**: `impasto_push` nasce em 0,0
no Deposit, então aquela linha é um no-op que mede o piso de ruído da sonda (−1,27 a −15,74) — o mesmo
serviço que `wet_smudge`/`wet_rewet` prestaram na §5.10.

### A cura: três caminhadas por LINHA

O `settle` é um box blur `O(n·r)` que **re-soma a janela por texel de propósito** (a soma corrida deriva
ao longo da linha e quebraria a byte-identidade do crop, §11) — e rodava num núcleo com 32 disponíveis.
Ele, a derivação da altura e a escrita com a caixa passam a andar por linha (ADR-0109, o desenho que esta
linha já usa em cinco lugares; `rayon` já é dep desta crate ⇒ **nenhum ADR novo**).

⚠️ **O padrão é *um corpo, dois walkers*:** o kernel de uma linha (`blur_one_row`) é `#[inline]` e **os
dois caminhos o chamam**, então não existe versão paralela para divergir da serial. A caixa de dirty sai
de `min`/`max` sobre índices — **associativos e comutativos** —, então a árvore de redução do rayon
devolve os mesmos quatro números em qualquer agendamento.

Depois, as três construções de **PATCH** que a edição viva do card Body lê (`live_mat_base` 117 MB ·
`live_film` 16 MB · `live_relief_base` 67 MB a 4096²) — `Vec::with_capacity` + `push`, ~200 MB numa
thread — foram para `patch_in`, cujo mapeamento `c → índice global` é **uma expressão só** que os dois
walkers usam. O `collect` indexado do rayon aloca uma vez e preenche em paralelo, sem a escrita dupla de
um `vec![neutro; n]` sobrescrito (e a crate é `forbid(unsafe_code)`, então `set_len` não era opção —
nem precisou ser).

### Medido pela porta do produto

| | antes | depois |
|---|---|---|
| `settle` isolado | 194,11 | **46,61** (4,2×) |
| fold (diagonal 4096²) | 348 | **106,14** (3,3×) |
| **pen-up diagonal 4096²** | **656,20** | **382,33** (1,7×) |
| pen-up reto 4096² | 44,90 | **27,54** |
| controle de 200 px | 5,98 | 4,73 |

⚠️ **Uma leitura foi DESCARTADA por o controle ter se movido:** logo após a suíte, com `load 5,68`, o
controle mediu 9,95 (2,3× o conhecido) — a corrida inteira foi jogada fora e re-medida até ele voltar,
que é exatamente o que a §5.49 prescreve.

⚠️ **E os três primeiros números que eu ia publicar vieram das TRÊS sondas rodando em PARALELO** — elas
disputam o mesmo pool de rayon, então cada uma media o agendador das outras. `--test-threads=1` não é
higiene: é parte da fixture quando o que se mede é paralelismo.

**2 gates, 3 mutações, 3 sangram** — a identidade contra a rota serial **CONGELADA** sob `cfg(test)` (o
código que shipava, verbatim: um `pub` sem chamador seria uma segunda resposta esperando alguém chamá-la)
e a caminhada por linhas contra o `for_each_in`, que **segue vivo como a rota curta** ⇒ não é oráculo
morto. ⚠️ A mutação da identidade do `max` derruba **119 testes existentes**: aquela metade já era
coberta, e descobri-lo custou uma corrida em vez de um gate novo.

### ⚠️ O que a medição REFUTOU, e por isso a próxima wave não foi aberta

O commit é agora **276,19 ms = 72% do pen-up**. A hipótese óbvia era *"ele extrai uma janela
canvas-sized dos quatro planos"* — e o diagnóstico por-plano do próprio produto diz o contrário:

```
metadados=true planos=None | canvas=WHOLE images=- h=WHOLE c=WHOLE m=WHOLE
```

**Todos os quatro caem em `Whole`**, que guarda `Arc` e não copia byte nenhum. Então o custo dele é
outra coisa, e escrever uma cura em cima da hipótese refutada seria escolher um culpado por eliminação —
o erro que esta jornada documentou três vezes. **A próxima wave começa por uma decomposição do commit,
não por código.**

### E o item ESTRUTURAL que a tabela expõe, com o número

A janela é o **BBOX do traço**, não a pegada dele. Num traço diagonal de r=40 a tinta cobre uma banda de
~80 px ao longo de 5737 px de diagonal = **~2,8% do retângulo** que as duas metades percorrem: o bbox
reivindica **35× demais**. Curar isso vale mais que paralelizar tudo o que sobra — e **muda o crop
byte-idêntico**, que é a joia deste módulo (o `settle` de uma janela é bit-a-bit o de um canvas porque a
borda dela é zero; uma janela em TILES precisa que cada tile carregue a própria halo de reach). É wave
própria, com gates próprios, e não entra de carona num fix de perf.

### ✅ SMOKE APROVADO (2026-08-02) — e o número do PRODUTO é maior que o da sonda

| `INPUT (fora do frame)` max, 4096² impasto | antes | depois |
|---|---|---|
| **up** | **512,9** | **82,3 · 64,4** |
| move | 106,1 | 18,2 · 28,7 |
| down | 17,1 | 11,3 · 13,5 |

⚠️ **6-8× no produto contra 1,7× na sonda, e a diferença NÃO é sorte:** a sonda mede a diagonal de canto
a canto — o pior caso construído —, e o gesto do artista tem bbox menor, onde a fração paralelizada pesa
mais. *Uma sonda de pior caso subestima o ganho de produto pela mesma razão que uma fixture de melhor
caso o superestima.*

⚠️ **E o log moveu a fronteira outra vez.** Com o INPUT em baixo, o maior número passou a ser
**`dispatch max = 54,3 ms`, 100% em `preview`, com `branch=idle` e `trivial=true`** — *a drenagem não
fez nada e o preview custou 54 ms*.

**Isto NÃO é desta wave, e o mecanismo diz por quê:** o `commit_stroke_height` roda dentro do
`on_canvas_pointer` no pen-up, **nunca** dentro do dispatch, e a caixa de dirty que ele publica é a mesma
(a redução devolve os mesmos quatro números). A assinatura já aparecia antes: **26,8-39,3 ms** no log
anterior e o outlier de **71,2 ms** do smoke do S3 (§5.62).

**O `preview` é o próximo balde a ganhar DIVISOR** — ele admite *o composite drenou a tela inteira* e *o
fold do relevo para o passe de luz re-materializou os três planos*, que são curas opostas (§4.8.2 nomeia
o segundo: os planos são materializados por frame sujo, de propósito, porque uma versão teria de
rastrear toda entrada do fold). É a receita que se pagou três vezes nesta sessão (§5.48 · §5.64 · esta):
*um balde cujo p50 e cujo max admitem curas opostas tem de publicar suas sub-partes*.

## §5.66 — A janela NÃO é o problema: o bbox da MUDANÇA já é 89% (2026-08-02)

A §5.65 fechou nomeando dois itens abertos e eu escolhi o maior (o commit, 280 ms dos 392 do pen-up
diagonal). A cadeia que o commit `52dfff2b6` registrou era: *bbox ~98% → `from_window` manda para
`Whole` → `Whole` segura o plano VIVO → o `fork_par` do traço seguinte copia 267 MB*.

**Duas medições depois, ela está metade certa e metade REFUTADA.**

### 1. A posse, com o CONTROLE na mesma corrida

`who_holds_the_planes_after_a_canvas_wide_stroke` conta os donos em repouso, traço curto contra
diagonal, no mesmo canvas e na mesma corrida:

| fixture | após 1 | após 2 | após 3 | sem o histórico |
|---|---|---|---|---|
| curto (200 px) — o CONTROLE | 2/2/2/2 | **2/1/1/1** | 2/1/1/1 | 1/1/1/1 |
| diagonal de canto a canto | 3/2/2/2 | **3/2/2/2** | 3/2/2/2 | 1/1/1/1 |

(canvas/heights/covers/mats; idêntico a 2048² e 4096².) O controle **cai** para um dono nos três
planos de relevo — a elisão do degrau 4 funciona — e o diagonal **não cai nunca**. É por isso que o
pen-up seguinte paga o fork: `make_mut` copia com qualquer coisa acima de um.

⚠️ **E isto corrige o cabeçalho do `measure_stroke_owners` (§1b)**, que afirma *"do segundo traço em
diante a entrada é `Patch` e não segura nada"*. A frase foi medida com o traço de 200 px — **0,1% da
área a 4096²** — e é falsa para o traço que o artista dá.

### 2. ⛔ REFUTADO: apertar a janela não cura

O item (3) da §5.65 dizia que a janela declarada é o **bbox** e a pegada é 2,8% dele, então declarar
menos resolveria. **O delta não precisa de onde se escreveu — precisa de onde o conteúdo DIFERE**, e
`what_a_stroke_declares_against_what_it_changes` mede exatamente isso comparando os planos antes e
depois:

| plano | 2048² | 4096² |
|---|---|---|
| heights | 79,13% | **89,26%** |
| covers | 78,48% | 88,92% |
| mats | 78,48% | 88,92% |
| canvas | 78,47% | 88,91% |

O corte do `from_window` é **50%**. Ou seja: mesmo a janela **derivada** — a mais apertada que existe
sem trocar de representação — cai em `Whole`. *Um traço diagonal tem bbox de ~90% do plano por
GEOMETRIA, não por declaração larga.* Nenhum aperto de declaração compra um byte, e a cura teria de
trocar o RETÂNGULO por **tiles** (que é o journal do S3, hoje `cfg(debug_assertions)`).

### 3. E a minha atribuição estava incompleta — o `Patch` cura o CANVAS, não o relevo

Ablação de uma linha (`from_window` nunca escolhe `Whole` por tamanho), medida e revertida:

| | canvas | heights | covers | mats |
|---|---|---|---|---|
| como shipa | 3 | 2 | 2 | 2 |
| `Patch` sempre | **2** | 2 | 2 | 2 |

⚠️ **O `Whole` era o TERCEIRO dono do canvas e nada mais.** Os três planos de relevo têm um segundo
dono que **sobrevive à ablação**, não é o `Whole` por tamanho, e não é o cursor (que elide relevo
incondicionalmente em release — `set_cursor`, degrau 4). `undo.clear()` o remove, então ele mora no
controller. **Quem é, está aberto** — os candidatos são o braço de forma/stride do `split` (que
devolve `Whole` e é exigido por correção), `OnlyBefore`/`OnlyAfter`, e a entrada de um run coalescido.

### 4. A bisseção fecha a POSSE: são as ENTRADAS, nos quatro planos

O `undo.clear()` remove duas coisas de uma vez, então a ablação foi partida
(`UndoController::probe_drop_entries`, `cfg(test)`: derruba as pilhas e deixa o cursor de pé):

| diagonal, 2048² e 4096² | canvas | heights | covers | mats |
|---|---|---|---|---|
| como shipa | 3 | 2 | 2 | 2 |
| só as ENTRADAS fora | **2** | **1** | **1** | **1** |
| …e o cursor também | 1 | 1 | 1 | 1 |

**As entradas seguram os quatro**; o cursor segura só o canvas (ele elide relevo
incondicionalmente), e é o mesmo 2 que o traço CURTO tem — pré-existente e fora deste assunto.

⚠️ **Isto corrige o §3 acima:** o `Patch`-sempre deixou o relevo em 2 **e o dono continua sendo a
entrada** ⇒ os três planos de relevo **não entram no `Whole` pelo limiar de tamanho**. Sobram o braço
de forma/stride do `split` (exigido por correção) e `OnlyBefore`/`OnlyAfter`. O canvas entra pelo
limiar (a ablação o curou); o relevo entra por outra porta, **e ela é o próximo passo** — uma medição,
não uma hipótese: imprimir a VARIANTE de cada plano.

⚠️ **A tabela foi medida com `load average 23,9`** (outra linha compilando ao lado), e vale mesmo
assim: isto é **CONTAGEM de `Arc`, não relógio** — determinística e imune à carga. A regra da §5.49
governa wall-clock; aplicá-la a um invariante seria descartar uma medição sã.

**Consequência para o plano:** a wave não é *"mude o limiar"*. São **duas portas** para o `Whole`, uma
identificada e uma não, e fechar só uma não compra milissegundo nenhum — a lei tudo-ou-nada que o
próprio §1b do `measure_stroke_owners` já enuncia.

## §5.67 — O relevo entra no `Whole` pelo journal — e dois doc-comments MENTIAM (2026-08-02)

> ⚠️ **Esta seção foi publicada com o veredito ERRADO e reescrita horas depois.** O que ela dizia —
> *"a sonda não vê o caminho do produto, porque o journal é `cfg(test)`"* — é **falso**, e o texto
> abaixo diz por quê. O caminho até o erro fica registrado porque ele é a lição.

### 1. A variante literal, por plano

`PlaneDeltas::variant_report` (`cfg(test)`, irmão do `confine_report` — que colapsa
`Whole`/`OnlyBefore`/`OnlyAfter` num tag só, de propósito, porque para ELE as três significam
*repinte tudo*):

```
[curto] entrada 2: canvas patch · heights [patch]  · covers [patch]  · mats [patch]
[diag ] entrada 2: canvas WHOLE · heights [WHOLE]  · covers [WHOLE]  · mats [WHOLE]
```

(A entrada 1 é `ONLY-AFTER` nos dois: o primeiro traço **cria** os planos e não tem lado `before`.)

### 2. A ablação, agora COM o relatório

Re-rodando o `Patch`-sempre da §5.66 §3 com a variante à vista: **canvas vira `patch`, relevo continua
`WHOLE`** ⇒ o relevo não passa pelo limiar do `from_window`. E instrumentando o outro braço
(`before.len() != after.len() || !fits(…)`), ele dispara **só para planos VAZIOS** (`before 0 after 0`
— os auxiliares de máscara/seleção/deform/sculpt, que `fits` recusa por `len != 0`). Nenhum relevo.

### 3. ⛔ A conclusão que eu tirei daí, e por que ela era falsa

Sobrou o `StoredMap::from_journal`, cujo header dizia *"tudo aqui é `cfg(any(test, debug_assertions))`"*
e ao lado do qual o `undo_planes.rs` afirmava *"em release ela é hoje SEMPRE o caminho de sempre,
porque o journal ainda é `cfg(debug)`"*. Como **`cargo test --release` LIGA `cfg(test)`** (isso é
verdade), eu concluí que a sonda rodava um caminho que o produto não toma, e publiquei.

**Os dois doc-comments estavam OBSOLETOS.** `undo_delta_journal.rs` não tem **um** `#[cfg]`, e
`mod journal_route` também não; o único `#[cfg]` dentro do `relief_maps` é o contador da sonda. O que
é `cfg(debug)` é o journal do **CANVAS** (`WriteState::capture_canvas` tem no-op de release ao lado) —
o do **RELEVO** foi promovido pelo degrau 4, e tinha de ser: elidir o `before` sem journal derruba a
história a cada traço (o próprio `snapshot_model_eliding_relief` diz isso). *As duas frases eram
verdadeiras no degrau 2 e ninguém as reconferiu quando o degrau 4 as tornou falsas.*

### 4. E a resposta verdadeira, que reconcilia tudo

`from_journal` tem **o mesmo limiar de 50%** do `from_window`, e o seu braço `Whole` faz:

```rust
return Some(Self::Whole { before: Arc::new(b), after: Arc::clone(live) });
```

⇒ **`after: Arc::clone(live)` É o segundo dono.** A ablação da §5.66 §3 mirou o limiar do
`from_window` e não o gêmeo dele, e é exatamente por isso que o canvas curou e o relevo não. Tudo
fecha: os quatro planos entram no `Whole` pelo MESMO limiar, por duas portas irmãs.

**O alvo da cura é agora singular e nomeado:** os **quatro** sítios que constroem `Whole` guardando
`Arc::clone(live)` no lado `after`. O `before` deles já é material próprio (`par_clone` + patch do
journal) e não segura nada — *só o `after` é o dono extra*.

### 5. As duas lições, e a segunda é a cara

**(a)** `cargo test --release` liga `cfg(test)`: uma sonda de unidade **não pode** observar um caminho
gateado nele, e quando isso importar ela tem de **imprimir em que rota está** (a sonda passou a
imprimir `(relevo pelo JOURNAL: Nx)`, e fica). Verdade geral, e continua valendo.

**(b)** ⚠️ **Mas aqui o defeito não era o instrumento — eram DOIS doc-comments do produto**, e eles me
fizeram publicar um veredito falso sobre a minha própria medição. *Um doc-comment que nomeia um `cfg`
é uma afirmação que EXPIRA: grepe o atributo, não leia a prosa.* Os dois foram corrigidos no mesmo
commit que esta seção, cada um dizendo o que afirmava antes e o que a mentira custou.

---

## §5.68 — Um gate de MAX não é salvo por uma razão: quando a propriedade é estrutural, o oráculo é o fonte (2026-08-02)

O `the_tick_never_waits_for_a_whole_stage` (a wave off-thread, §5.31-§5.38) falhou na suíte. A doc dele
**já admitia a flake de carga com o número** — *30,64 ms sob a suíte em paralelo contra a barra de 30*,
medido em 2026-07-29 — mandava *"re-rode sozinho antes de suspeitar de uma regressão"* e concluía que
*"a barra fica onde está: subi-la para acomodar a máquina carregada tiraria o dente que separa 4 ms de
espera de um estágio inteiro"*.

Ficou. E com `load average 38` (outras linhas compilando nos mesmos 32 núcleos) ele passou a falhar
**ISOLADO**: `FAILED / ok / FAILED` em três corridas seguidas, sozinho. ⚠️ *Um gate que alterna sob
carga não é acreditado — é **silenciado**, que é a única coisa pior que não o ter.*

### 1. A razão foi construída, medida e REPROVADA

A cura óbvia — e a que este repo ensina em vários lugares (o kill do Deform, §W4: *"por ser razão é
imune à deriva da máquina"*) — é trocar o wall-clock por uma **razão**: o pior tick dividido pelo custo
de um passo, medido na mesma corrida pela porta `wet_step_sync` (que roda os MESMOS estágios do worker,
então não é uma segunda resposta a *"quanto custa um passo?"*). A teoria: o modo de falha força
`worst ≥ passo` **por construção**, e sob carga os dois termos inflam juntos.

**Medido, quatro corridas:** `1,82 · 1,41 · ok · 0,77` — com o pior tick em **47,63 ms contra 26,12 de
passo**. A razão flaka igual.

⚠️ **E o número diz por quê:** `worst` é um **MAX sobre 90 amostras**. Ele não mede o desenho — mede *a
pior preempção do SO em 1,5 s*. Isso é ruído **aditivo e só no numerador**, e razão nenhuma o cancela.

> **A regra fica mais afiada, não invertida:** uma razão cancela a deriva da máquina quando os dois
> termos são **reduções comparáveis do mesmo trabalho** (média contra média, mínimo contra mínimo). Um
> **MAX** não é isso: ele é um ímã de outlier, e o denominador não tem outlier nenhum para cancelá-lo.

### 2. O que ficou: a propriedade é ESTRUTURAL

O que separa os dois desenhos não é um milissegundo — é a porta do tick (`WetSession::try_bring_home`)
pedir o motor com **espera limitada**. Com um `recv()` nu no lugar do `recv_timeout(TICK_WAIT)`, o tick
contém o estágio **por construção**; e isso um scanner de fonte vê em qualquer máquina.

- **`the_tick_asks_for_the_engine_with_a_bounded_wait`** (roda em **0,00 s**), com **controle positivo
  nas duas pontas** — o padrão do irmão `the_frame_does_not_run_a_sim_stage`: a porta **bloqueante**
  `bring_home` tem de continuar com o `recv()` nu, senão o gate passaria por *o scanner estar olhando o
  lugar errado*.
- **`measure_the_worst_tick_against_a_step`** (`#[ignore]`) guarda o número, com o aviso de que sob
  carga ele **não fala sobre o código**.

**Mutações: 2, as 2 sangram.** `recv_timeout(TICK_WAIT)` → `recv()` na porta do tick ⇒ RED. E a porta
bloqueante deixando de bloquear ⇒ RED no controle — ⚠️ essa segunda **não é só controle, é um defeito
real**: um clique do artista voltaria de mãos vazias, que é o que o doc-comment dela promete que nunca
acontece.

⚠️ **E o precedente já estava escrito NESTE arquivo de testes**, doze linhas acima do gate que eu
mantive por um ano: o 1,2× do relógio contínuo do worker *"fica sem gate, de propósito, com o número
escrito no `worker_loop`"*. A prática existia; ela só não tinha sido aplicada ao vizinho.

---

## §5.69 — O `Whole` por limiar da rota do journal: uma regra transplantada para onde a premissa dela é falsa (2026-08-02)

A caçada das §5.66/§5.67 terminou nomeando um alvo singular: os sítios que constroem `Whole` guardando
`Arc::clone(live)` no lado `after`. Com a máquina calma (`load 0,37`), o payoff foi medido — e ele é
**maior e mais simples** do que o desenho que eu tinha esboçado (elidir o `after` atrás de uma
testemunha `Weak`), porque a medição derrubou a premissa que sustentava o ramo inteiro.

### 1. O alvo, pela porta do produto

Pen-up de uma diagonal de canto a canto, impasto (`what_the_commit_half_is_made_of`):

| tela | pen-up | com o histórico LIMPO | ⇒ o commit custa |
|---|---|---|---|
| 2048² | 95,60 ms | 24,85 | **70,75 (74%)** |
| 4096² | 380,57 ms | 108,05 | **272,52 (72%)** |

### 2. A ablação pelo LIMIAR, e ela atribuiu a UMA das duas portas

Forçando a rota `Patch` (o limiar de 50% desarmado), o commit a 4096² cai para **151,60 ms** — **−120,9
ms**. ⚠️ E desarmando **também** o limiar do `from_window` o pen-up **PIORA** (257,02 → 281,46): aquele
está certo como está. *A doença é só a porta do JOURNAL.*

### 3. A premissa era falsa, e o doc-comment a declarava

O ramo dizia, por escrito, que *"ali o `Whole` guardaria os dois planos inteiros de qualquer forma — o
`split` clássico faz exatamente a mesma escolha, no mesmo limiar"*. A escolha é a mesma; **a premissa
não é**:

- no `from_window` o `Whole` **MOVE** os `Arc` que já existem — custo zero, nenhuma cópia;
- no `from_journal` ele **COPIA**: `par_clone` do plano inteiro **mais** uma varredura `j.get(i)` de
  plano inteiro — e faz isso **descartando o `before`/`after` que as duas linhas acima já extraíram**.

⇒ Uma regra transplantada para o sítio onde o que a justificava não vale. O ramo saiu.

### 4. ⚠️ E ele perdia no eixo em que eu supunha que ganhava: BYTES

Eu ia escrever que o trade era *tempo contra memória* (`Whole` guarda 1 plano, `Patch` 1,8). **Medido,
é o contrário** — `Whole` guarda **os dois lados dos QUATRO planos inteiros**:

| rota | bytes/passo (2048²) | bytes/passo (4096²) | passos retidos |
|---|---|---|---|
| `Whole` (como shipava) | 134,22 MB = **8,00×** um plano RGBA | 536,87 MB = **8,00×** | 3 · 1 |
| `Patch` (a cura) | 123,70 MB = **7,37×** | 513,95 MB = **7,66×** | 3 · 1 |

O **8,00× exato nas duas telas** é a assinatura: `4 + 4 + 1 + 7 = 16 B/px` × dois lados = 32 B/px = 8
planos RGBA. Não há trade — o ramo perdia em tempo, em bytes **e** na posse.

### 5. O resultado, e o que ele NÃO comprou

**Commit 272,5 → 151,6 ms a 4096²** (pen-up 380,6 → **256,3**) e 70,8 → 42,3 a 2048² (95,6 → **64,7**).
Posse: os **três planos de relevo** deixam de ter um segundo dono permanente (2 → 1).

⚠️ **O FOLD não melhorou** (108,5 → 104,2 ms, dentro do ruído) — e a explicação já estava escrita no
cabeçalho da sonda de posse: *dentro* do gesto os donos são **três** (`tool` · `cursor` ·
`paint.stroke_undo`), então remover um não compra milissegundo nenhum. **O S3 continua tudo-ou-nada**;
o que esta wave entrega é o commit.

⚠️ **O canvas continua `WHOLE` e com 3 donos** — ele vai pela outra porta (a barata, que move `Arc`), e
os donos dele são o `cursor` e a entrada. Essa é a fronteira que resta.

### 6. O gate, e por que ele não existia

`a_wide_window_stays_a_patch_and_never_pins_the_live_plane`. ⚠️ **Todas as fixtures do arquivo de gates
do journal usavam um traço CURTO**, cuja janela nunca alcança os 50% — *o ramo removido nunca era
executado por nenhum gate deste repo*. O cabeçalho do `tool_mid_step` já avisava a versão irmã disto (a
256² as duas rotas colapsam no mesmo `Whole` e **três mutações reais sobreviveram**).

O **controle positivo usa um mecanismo INDEPENDENTE**: a rota clássica ainda carrega o limiar de 50%,
então se ela cai em `Whole` sobre o mesmo passo, a janela da fixture de fato cruza o limiar. **Mutação:
devolver o ramo ⇒ RED**, nomeando `heights [WHOLE]`.

---

## §5.70 — E o que sobrava do commit era DIVISÃO INTEIRA por elemento: 151,6 → 67,0 ms (2026-08-02)

A §5.69 deixou o commit em **151,6 ms** a 4096² e não disse de que ele era feito. Ele era do lado
`before`: `win.extract_by(|i| j.get(i).unwrap_or(live[i]))` — e o `TileJournal::get` faz, **por
elemento**, `i/stride` · `i%stride` · `x/TILE` · `y/TILE` · `y%TILE` · `x%TILE`, mais dois lookups com
bounds check. Numa janela de ~96% a 4096² isso são ~16,7 M chamadas **por plano**.

⚠️ **É a família do `is_probe_cell` da §5.42** (*"duas divisões inteiras no laço mais quente"*) e do
`alpha_of_mass` da §5.43 (*"`%` em `f64` é uma chamada a `fmod`, não uma instrução"*): o custo não era o
trabalho, era **a pergunta sendo refeita**.

### A fatoração, e ela é exata

Dentro de uma linha `y` é fixo ⇒ `ty` e `y % TILE` são fixos; dentro de uma corrida de `TILE` colunas
`tx` também é. `TileJournal::read_row_into` resolve o tile **uma vez por corrida** e copia
contiguamente (`extend_from_slice`); tile não capturado cai no plano vivo pela mesma lei do `before`
(§5.28), agora honrada por corrida em vez de por elemento.

| 4096², diagonal de canto a canto | §5.68 (antes) | §5.69 | §5.70 |
|---|---|---|---|
| commit de undo | 272,5 ms | 151,6 | **67,0** |
| pen-up | 380,6 ms | 256,3 | **179,1** |

E a 2048²: commit **70,8 → 40,1 → 18,6**; pen-up **95,6 → 64,7 → 43,3**. Acumulado, o commit ficou
**4,07× mais barato** e o pen-up **2,12×**.

### ⚠️ E o `dead_code` que só o build de RELEASE mostra

Trocar o chamador de produção deixou o `TileJournal::get` **sem chamador num build de release** — os
quatro que restam (`canvas_before`, `heights_before`, `covers_before`, `mats_before`, a rede de
conferência) são `#[cfg(any(test, debug_assertions))]`. Ele leva o mesmo `cfg`.

**A lição é sobre onde eu procurei.** Rodei `clippy --profile ci-test --all-targets` e ele saiu limpo,
porque em build de teste os chamadores existem; e `cargo check` (perfil dev) também, pelo mesmo motivo.
⚠️ *Um aviso de código morto pode ser visível só na forma de build em que o chamador some* — e esta é a
imagem espelhada do miss da mesma sessão, em que o `items after a test module` só aparecia **com**
`--all-targets`. **Nenhuma das duas formas basta sozinha.**

⚠️ **E o meu grep de "todo chamador" terminava em `| head`** — ele cortou os quatro de `undo_window.rs`,
eu li a saída como completa e congelei o método sob `cfg(test)`, o que **quebrou o build**. *Uma busca
por "todos os X" com `head` responde outra pergunta.*

### O gate, e o controle que teve de mudar de nível

`the_run_walk_reads_what_the_element_walk_reads`, contra a rota por-elemento **CONGELADA sob
`cfg(test)`** — *o código que shipava*, não uma re-derivação (a lição do `warp_axis`/`serial_side`). A
ORDEM de percurso é parte da afirmação: o `Patch` guarda os dois lados e eles se correspondem índice a
índice.

⚠️ **A fixture cruza fronteira de tile nos dois eixos**, e inclui tile capturado, tile **não**
capturado, um tile exato e a captura de plano inteiro — dentro de um tile só as duas rotas leem o mesmo
bloco e a corrida nunca é exercitada.

⚠️ **E o controle nasceu ERRADO por um nível:** eu o escrevi por PAR (*"esta janela contém bytes do
journal"*) e ele **disparou** — uma janela que não cruza a área capturada lê tudo do plano vivo, o que é
legítimo e é justamente um dos casos a testar. A cobertura é propriedade do **CONJUNTO**: o que não pode
é *nenhum* par tocar o journal. **3 mutações, 3 sangram** (o offset dentro da corrida · o comprimento da
corrida · o offset de coluna do fallback).

---

## §5.71 — A lavagem reconstruía uma vez por EVENTO de ponteiro, e o doc dela dizia QUADRO (2026-08-02)

A tarefa era *avaliar o modo Watercolor e tentar otimizá-lo*, e o handoff 31 entregava a base já
medida: **3,1 ms/move**, plano no tamanho da tela, com o **warp valendo 56%** do que a aquarela cobra
sobre o Digital — mais o veredito honesto de que *"3,1 ms contra um orçamento de 16,6 não é um
problema de performance hoje"*.

⚠️ **Aquele veredito era verdadeiro exatamente onde foi medido: raio 100.** A tabela inteira mede um
único raio, o slider vai a `BRUSH_SIZE_MAX_PX = 512`, e o relato mais recente do Enio (§5.51) é de
pintar a **raio 300**. Reconferir a nota quando alguém move o número que a tornava verdadeira é o §0.

### 5.71.1 A varredura de raio, e o confound que ela tinha de evitar

O espaçamento de um dab é `spacing × 2 × raio`, então um passo de mouse FIXO emite ~10 dabs a r=20 e
**menos de um** a r=300 — uma mediana por-move passa a pegar justamente o move que não carimbou nada,
e foi assim que uma varredura anterior deste repo viu **o Digital ficar mais barato com pincel maior**.
Medindo o **traço inteiro sobre caminho de comprimento fixo** (o que o artista faz: arrastar o mouse
uma distância), a 4096²:

| raio | moves | por move | pen-down |
|---|---|---|---|
| 20 | 20,6 ms | 0,71 | 81,5 |
| 100 | 85,5 | 2,93 | 82,1 |
| 300 | 267,5 | **9,20** | 97,7 |
| 400 | 342,7 | **11,87** | 112,3 |

**Linear no raio** (0,0295 × raio, constante em toda a faixa) ⇒ a r=400 um único movimento custa **71%
de um quadro**. E o **pen-down é ~75-112 ms quase INDEPENDENTE do raio** — trabalho de TELA, que a
tabela de ablação não vê porque o `move_ms` dela descarta o pen-down de propósito.

### 5.71.2 A pergunta que decidiu tudo: por DAB ou por EVENTO?

O mesmo caminho de 1200 px, entregue em passos diferentes (r=100):

| eventos | aquarela | Digital |
|---|---|---|
| 30 | 87,5 ms (1,00×) | 34,7 (1,00×) |
| 240 | **172,5 (1,97×)** | 36,0 (**1,04×**) |

O Digital é **plano** — o custo dele é por dab, a taxa de polling não compra trabalho, e ele é o
**controle que prova que o teste sabe dizer "não há patologia"**. A aquarela dobra: ela pagava trabalho
do tamanho da PEGADA **por evento de ponteiro**.

### 5.71.3 O mecanismo, e a cura que já tinha sido tentada do lado errado

`paint_extend` chamava `apply_watercolor` + `pour_canvas_wet` em TODO Move. A janela da lavagem é
*"os dabs desde o último composite" **padeada pelo raio de influência***, que é do tamanho da pegada —
então o pad domina e encolher o passo do mouse **não encolhe a passada**, só multiplica quantas vezes
ela acontece. E o doc do `apply_watercolor` dizia **três vezes** que a cadência é o QUADRO (*"each
frame recomposites"*, *"renderFrame"*, *"the frame dirty rect"*).

⚠️ **A duplicação já tinha sido VISTA.** O comentário do soak em `paint_tick` registra um profile de
2026-07-07 onde `stamps` e `tool-tick` carregavam cada um um composite cheio — e a correção de então
suprimiu **o do TICK**, o único-por-quadro, sob a premissa de que *"o flush de Move já recompôs a
janela deste quadro"*. A premissa vale para os métodos **COALESCIDOS** (uma entrega por quadro) e é
falsa para o freehand incremental, que é o pincel de aquarela padrão: ali o flush é um por evento raw.

⚠️ **E a atribuição inicial estava INVERTIDA.** O composite é gateado em dab (`wet_frame_dirty` só
existe com região de dab, então um evento sem dab não compõe); quem **não** era gateado em nada era o
`pour_canvas_wet`, que ainda por cima caminha o rect **CUMULATIVO** desde o pen-down. Era ELE o termo
que escalava sem limite com a contagem de eventos. Os dois moravam no mesmo braço, então a cura pega
os dois — mas a frase *"o composite roda mesmo com zero dabs"* era falsa e está corrigida aqui.

### 5.71.4 O resultado, e a propriedade que autorizou a deferição

Mesmo traço, 30 quadros (0,5 s), variando só quantos eventos caem em cada quadro:

| dispositivo | ev/quadro | por evento (antes) | **por quadro (agora)** | ganho |
|---|---|---|---|---|
| 120 Hz | 2 | 130,9 ms | **92,2 (1,00×)** | 1,42× |
| 240 Hz | 4 | 146,6 | **89,9 (0,97×)** | 1,63× |
| 480 Hz | 8 | 179,0 | **90,6 (0,98×)** | 1,98× |
| 960 Hz | 16 | 234,1 (1,79×) | **91,3 (0,99×)** | **2,56×** |

**Plano em 8× a taxa do dispositivo** — o custo passa a depender do DESENHO e não do mouse do artista.

⚠️ **Byte-idêntico, MEDIDO e não argumentado:** o mesmo caminho em 15 e em 120 eventos pinta telas que
diferem em **0 bytes**. Isso responde de quebra a hipótese pior — *a aparência da aquarela não dependia
da taxa do mouse*. E o `pour_canvas_wet` é seguro pelo mesmo motivo por outra via: ele escreve
`max(existente, cobertura)` sobre uma cobertura que só CRESCE, então `max(max(a,b),c) = max(a,b,c)`.

⚠️ **Latência ZERO, e é a ordem do frame que garante:** o tick roda em `render_loop` ~1198, depois do
flush de ponteiro (~698) e **antes** do upload do preview (~3397). O quadro que recebeu os Moves é o
quadro que mostra a tinta.

⚠️ **NEGATIVO honesto: a 1 evento por quadro a cura não compra nada** (21,3 contra 20,6 ms — ruído).
As duas rotas fazem o mesmo número de composites ali. Ela paga a partir de 2 ev/quadro, que é todo
dispositivo moderno a 60 fps.

### 5.71.5 O pen-down: 268 MB para reproduzir uma cor chapada

`composite_below` **alocava e PREENCHIA** o acumulador `[f32;4]` — 268,4 MB a 4096² — antes de a linha
seguinte perguntar se há algo abaixo da âncora. Num documento de **uma camada** não há: o `descend`
não produz fatia nenhuma, o laço de composite não roda, e 335 MB de tráfego produzem a cor de papel.
*O guard existia; a alocação estava ACIMA dele, então o early-out era inalcançável por construção.*

**pen-down 81,5 → 26,4 ms** (r=20) · 82,1 → 36,7 (r=100) · 112,3 → 62,2 (r=400).

⚠️ Byte-idêntico por construção (compor o conjunto vazio sobre o chão deixa o chão) — e a premissa que
faltava virou **gate**: o preenchimento chapado só bate com `encode(decode_byte(chão))` porque o
round-trip de byte do sRGB é a **identidade nos 256 valores**. Este repo já se queimou presumindo
precisão de tabela de transferência (doc 24), então isso é medido, não suposto.

### 5.71.6 O que fica ABERTO, com número

- **O pour ainda caminha o rect CUMULATIVO** uma vez por quadro ⇒ o custo por quadro cresce ao longo
  do traço: **1,23× / 1,32× / 1,51×** do 1º para o 4º quarto, em traços de 24 / 48 / 96 quadros
  (`measure_whether_the_frame_cost_grows_along_the_stroke`). A cura tem a mesma forma (dar-lhe o rect
  do QUADRO), **mas a premissa não foi verificada**: o filtro de dono (`wet_styles.owner`, por
  recência) pode mudar a elegibilidade de um texel no meio do traço. Wave própria, com gate de
  byte-identidade de `canvas_wet` — não construída porque a correção não está provada.
- **O WARP segue sendo 56%** do que a aquarela cobra sobre o Digital, e **não tem caminho de CPU**: os
  9 taps de AA foram a CURA da borda serrilhada (warp 48: 226 degraus → zero) e cortá-los está fora de
  discussão. O que resta é aproximar o warp dentro do texel — classe que este repo já mediu e
  **rejeitou duas vezes** no AA do impasto —, e exige oráculo de APARÊNCIA + ordem do Enio.
- **Os shape editors (`DragDot`/`Anchored`/`Line`) compõem por evento mesmo com a cura**, porque
  `clear_wet_coverage` dobra o rect cumulativo no do quadro. No app eles são **coalescidos** pela shell
  (uma entrega por quadro), então não é um defeito vivo — mas é o mesmo mecanismo noutra rota.

---

## §5.72 — O Watercolor a 250 px: a CONTAGEM decidiu com a máquina cheia, e a premissa aberta desde §5.71 está VERIFICADA (2026-08-02)

> Report do Enio: *"um pincel de 250px e as configurações na imagem provocam grande queda de FPS.
> Desempenho pior em imagens grandes (4096)."* — com `Rewet 0.400`, `Smudge 0.197`, `Dilution 0.168`,
> `Charge 0.755`, `Pull 0.477`, `Drying Time 10`. **Máquina a `load average 27,7`** ⇒ pela regra da
> §5.49, **nenhum número de tempo desta sessão vale**. Então esta wave não mediu tempo nenhum: ela
> **CONTOU TEXELS**, que é reprodutível sob carga, e leu os ESCRITORES em vez de supor.

### 1. Duas coisas no report contradiziam a tabela herdada

O doc 31 (o handoff que abriu a frente) media a aquarela em **3,07 / 3,12 ms** a 2048²/4096² — **plana
na tela** — e decompunha o custo com **`Smudge` e `Rewet` em ZERO**, usando as duas linhas como o
**piso de ruído** da sonda. O Enio pinta com os dois LIGADOS, e num raio que a sonda nunca visitou (ela
mede **um raio só**, 100).

### 2. O que a contagem achou — e o que ela REFUTOU

**Sonda `measure_the_area_the_wash_walks_per_frame`** (áreas de retângulo, zero relógio), r=250,
caminho de 1500 px, 48 quadros:

| canvas | tela | pour 1º quarto | pour últ. quarto | razão | vs pegada |
|---|---|---|---|---|---|
| 2048² | 4,19 M | 0,35 M | **0,94 M** | **2,68×** | 3,7× |
| 4096² | 16,78 M | 0,35 M | **0,94 M** | **2,68×** | 3,7× |

⚠️ **A minha hipótese do CLAMP foi refutada na primeira corrida:** o pour é **idêntico nas duas
telas**. Ele é limitado pelo **TRAÇO**, não pela tela — mas **cresce 2,68× dentro de um único traço** e
termina em **3,7× a pegada de um dab**.

**Sonda `measure_the_area_a_watercolor_frame_walks`** (ablação **pela ENTRADA** — knobs do painel —
sobre o contador novo `WashCadence::window_px`, que o produto soma):

| raio | ablação | janela/quadro | vs pegada |
|---|---|---|---|
| 250 | como o Enio ajustou | 0,36 M | 1,4× |
| 250 | sem Dilution | 0,36 M | 1,4× |
| 250 | sem Rewet | 0,35 M | 1,4× |
| 250 | sem os dois | 0,32 M | 1,3× |
| 60 | como o Enio ajustou | 0,05 M | 3,4× |

⚠️ **E ela refutou a MINHA segunda hipótese, que eu já tinha escrito:** eu li em `window.rs:110` que
`Dilution > 0` liga o `watered` e **dobra** o `reach` (`spread_any * 2`), e concluí que os knobs dele
inflavam a janela. **Dobram o `reach` e não movem a janela** (0,36 contra 0,32) — porque o `spread`
padrão é pequeno e o pad desaparece contra um pincel de 250 px. *Um multiplicador sobre um número
pequeno continua pequeno; ler o código diz o FATOR, só a contagem diz o PESO.*

**O retrato que sobra, por quadro a r=250:** a janela do composite é **0,36 M e PLANA**; o pour vai de
0,35 a **0,94 M** e **não tem teto**. ⇒ *no fim de um traço longo o pour caminha 2,6× mais que a
reconstrução óptica, e é o único dos dois que cresce.*

### 3. A dependência de TELA é por TRAÇO, e está no pen-down

`freeze_watercolor_ground` roda uma vez por traço e faz **três varreduras de plano inteiro**:

| a 4096² | o quê |
|---|---|
| ~67 MB | `build_wet_backdrop` — aloca `n×4` e o preenche |
| ~67 MB | `wet_substrate` — anda `n` floats pondo `NaN` (invalidação do memo de papel) |
| ~16 MB | `wet_soak` — zera `n` bytes (só em sessão nova) |

A 2048² é **um quarto** disso. É a única metade do módulo que responde ao tamanho do documento, e ela é
paga **a cada traço** — com um pincel de 250 px, muitas vezes.

⚠️ **E o `wet_substrate` é preenchido PREGUIÇOSAMENTE** (`fill_substrate_cache`, só sobre a região de
SAÍDA do composite) ⇒ **o `NaN` de tela inteira invalida pixels que nunca foram preenchidos**.

### 4. A premissa que estava aberta agora está VERIFICADA (e ela libera a wave)

O handoff de 2026-08-02 deixou o pour em aberto com a ressalva *"a premissa não foi verificada — o
filtro de dono por recência pode mudar a elegibilidade de um texel no meio do traço"*. **Verificada
lendo os escritores, não supondo:**

- `stroke_coverage` só é mutado **por-dab** (`watercolor_accum.rs:361-366`);
- `wet_styles.owner` idem (`:364`) — a recência é escrita **onde o dab cai**;
- os dois `zip` de plano inteiro daquele arquivo (`:322`, `:345`) são **backfills ÚNICOS**, guardados
  por `len() != fw*fh` — não são por-evento, e é por isso que o censo mede o move plano;
- e o pour é **idempotente** (max-blend, o doc dele na linha 270).

⇒ **Um texel fora do rect do QUADRO não pode ter mudado nem de cobertura nem de dono**, e re-despejá-lo
produz o mesmo byte. **O rect do quadro basta.** É a lei do S1/S2 (*a janela vem de quem ESCREVE*) no
terceiro sítio deste módulo.

### 5. O que ficou, e o que NÃO foi construído

**Ficou** (tudo verificável sob carga): as duas sondas de contagem e o contador `window_px`, somado
pelo produto ao lado do `composites`.

⛔ **A wave NÃO foi construída, e o motivo é a regra da casa:** uma wave de performance sem um número
de PRODUTO medido não é uma wave, e este box não pode produzir esse número hoje. A receita está pronta
e é pequena:

1. o acumulador de dabs **declara** onde escreveu (`wet_pour_dirty`, unido por-dab, no molde do
   `declare_wrote` da §5.17-19), e `pour_canvas_wet` o **consome** em vez de ler o cumulativo;
2. `freeze_watercolor_ground` invalida o `wet_substrate` **só onde ele foi preenchido** — a mesma
   ideia, um plano adiante;
3. o `build_wet_backdrop` é cacheável **dentro de uma sessão** com chave na pilha de camadas — ⚠️ este
   é o de MAIOR risco (o modo de falha de esquecer uma entrada é um chão velho que ninguém vê que é
   velho, a cerca que a nota do passe de luz já pregou), então ele vem por último e com gate próprio.

⚠️ **Gate de cada um é BYTE-IDENTIDADE, não relógio** — o `canvas_wet` e a tela composta têm de sair
iguais pelas duas rotas —, e o **benefício também se conta** (texels caminhados), então a wave inteira
é auditável com a máquina cheia. Só o veredito final (*o Enio sente?*) pede o box calmo.

---

## §5.73 — E era o SMUDGE forkando o canvas em todo evento: o quadro 83,4 → 27,4 ms, e PLANO na tela (2026-08-02)

> Continuação direta da §5.72, com a máquina calma (`load average 1,40`) e o Enio autorizando medir.
> A §5.72 tinha contado texels e apontado para o `pour`. **A medição matou esse alvo e achou outro.**

### 1. O alvo da §5.60 morreu na primeira medição

O `pour` foi construído com rota de ablação e medido **costas-com-costas na mesma corrida** (§5.46):

| | ganho | o que muda na tinta |
|---|---|---|
| rect do QUADRO contra o cumulativo | **1,02–1,03×** | **100% dos texels molhados**, pior delta 20/255 |

⛔ **Todo o custo, nenhum benefício** ⇒ a porta foi **REVERTIDA inteira** (a lei da §5.44: *um doc que
justifica uma porta com um número que ela não entrega é pior que porta nenhuma*).

⚠️ **E a premissa de byte-identidade da §5.72 estava ERRADA, com um buraco que eu mesmo abri:** eu
enumerei as FONTES do pour (`stroke_coverage`, `wet_styles.owner` — as duas só mutadas por-dab) e
esqueci que o **ALVO decai**. O `dry_canvas_wet` roda no MESMO `paint_tick`, então `canvas_wet` seca a
cada quadro e o pour cumulativo o levanta de volta sobre a pegada inteira; caminhar só o quadro deixa a
cauda do traço secar enquanto o artista pinta. *Enumerar as fontes de uma escrita não basta quando o
destino tem vida própria.*

⚠️ **E a lição maior, sobre o método da §5.72: ÁREA NÃO É CUSTO.** O pour caminha **2,6× mais texels**
que o composite e custa **~2%** dele — o composite faz ordens de grandeza mais trabalho por texel.
Contar é imune à carga, mas contagem é **proxy**, e um proxy precisa ser calibrado contra o real pelo
menos uma vez antes de virar alvo.

### 2. O que a decomposição achou

Traço de 1500 px, r=250, os knobs do report do Enio, quadro = 4 eventos + tick:

| | 2048² | 4096² | cresce |
|---|---|---|---|
| **carimbo** (`on_canvas_pointer`) | 6,9 ms | **50,1 ms** | **7,28×** |
| tick (composite + pour + secagem) | 22,2 | 33,3 | 1,50× |
| quadro | 29,1 | **83,4** | 2,87× |

⇒ é o **CARIMBO**, e ele cresce **pior que linear na área**. Três hipóteses minhas caíram antes disso,
todas por medição: o **clamp** (o pour mede igual nas duas telas), o **pad da janela** (`Dilution`
dobra o `reach` e move a janela de 0,32 para 0,36 M — *ler o código dá o FATOR, só a contagem dá o
PESO*), e o **segundo dono do `Arc`** do canvas (`strong_count = 1`, medido; e o preview custa 0,3%).
A janela do composite é **idêntica** a 2048² e 4096² (0,36 M texels) e não há trabalho `n`-sized dentro
dele: o composite é honestamente window-bound.

**A ablação por ENTRADA sobre o carimbo nomeou o dono em uma linha:**

| ablação | 2048 | 4096 | cresce |
|---|---|---|---|
| como o Enio ajustou | 6,71 | 49,60 | 7,40× |
| sem Charge / Dilution / Pull / Rewet | ~6,3 | ~49,5 | ~7,8× |
| **sem Smudge** | **1,62** | **1,65** | **1,01×** |

### 3. A causa, e o código a documentava

`smear_wet_base` muta a base pelo `Arc::make_mut`, e o comentário dele diz: *"na primeira pincelada da
sessão os dois campos compartilham um `Arc`, então o `make_mut` os FORKA"* — mas a **re-partilha no fim
da função restabelece o par**, então o fork acontecia em **TODO evento de smudge**, não uma vez. A
67 MB por cópia a 4096², com 4 eventos por quadro, isso é ~268 MB de memcpy por quadro.

**A cura é a da §5.12, um módulo adiante: soltar a segunda referência ANTES do `make_mut`** — com um
dono só ele **MOVE** em vez de copiar, e a re-partilha no fim devolve o invariante (o pickup do mixer
lê a tinta esfregada). *Uma pergunta de identidade não se paga com POSSE.*

| | antes | depois | |
|---|---|---|---|
| carimbo @4096² | 49,60 ms | **5,06** | **9,8×** |
| carimbo, cresce com a tela | 7,40× | **1,01×** | limitado pela PEGADA |
| **quadro @4096²** | **83,4 ms** | **27,4** | **3,05×** |
| quadro, cresce com a tela | 2,87× | **0,89×** | plano |

Suíte **961 verde em release, 959 em debug** — a byte-identidade está preservada por construção
(`make_mut` com um dono devolve o mesmo buffer).

### 4. Os gates, e os três defeitos que eles tiveram antes de morder

**`the_smudge_does_not_fork_the_canvas_on_every_event`** (contagem, sem relógio: o produto conta os
forks em `WashCadence::base_forks`, irmão do `composites`) · **`the_smudging_stamp_is_footprint_bound_not_canvas_bound`**
(razão 2048÷4096). Mutação (reinstalar o fork): **18 forks em 8 eventos** e razão **6,61×** — os dois
sangram, e não são redundantes (um passe canvas-sized novo passaria pelo primeiro e cairia no segundo).

⚠️ **Três coisas minhas erraram primeiro, e as três valem mais que o resultado:**

1. **O oráculo do ENDEREÇO do buffer foi derrotado pelo ALOCADOR.** A 1ª versão do gate de propriedade
   comparava `as_ptr()` da base a cada evento. Sob a mutação ele lia *"não moveu"* — porque o fork
   libera a alocação anterior e o alocador a devolve no evento seguinte, então o ponteiro **sai e
   VOLTA**. *Um oráculo que o alocador pode satisfazer por acidente não é oráculo.* Trocado por um
   contador no produto.
2. **A 1ª fixture da razão era pequena demais** (r=100, 1024÷2048): o fork é proporcional à ÁREA, e
   contra um fundo grande a razão ficava sob a barra. O par tem de ser **onde o fenômeno foi medido**.
3. ⚠️ **E eu passei quatro rodadas medindo um gate que nunca tinha sido escrito:** um `cd` relativo
   falhou, o `&&` curto-circuitou, e o `python3` que reescrevia o arquivo **nunca rodou** — com o teste
   seguinte passando sobre a versão antiga. É a armadilha de cwd que o `project-memory` já registra;
   a regra que a fecha é **caminho absoluto em toda edição de arquivo**, não só no `cargo`.

### 5. Aberto, com número

- **O pen-down: 33,4 ms @2048² contra 46,0 @4096², `1,38×`** — ⚠️ **re-medido DEPOIS do fix, e o fix
  moveu o número** (a leitura de antes era 34/63, com o fork do smudge dentro dela; §0: *quem move o
  número reconfere a nota*). Só **~12,6 ms** dele responde ao tamanho da tela; os outros ~33 são
  limitados pela pegada, em qualquer documento. A parte canvas-sized é o `freeze_watercolor_ground`
  (backdrop ~67 MB + substrato ~67 MB + soak ~16 MB a 4096²), e ⚠️ o `wet_substrate` é preenchido
  **preguiçosamente** (só sobre a região de saída do composite), então o `NaN` de tela inteira invalida
  pixels que **nunca foram preenchidos** — a cura é a mesma lei (*a janela vem de quem escreve*).
  **Não construída**: o teto do que ela compra é uma fração de 12,6 ms, contra os 56 ms/quadro que esta
  wave já devolveu, e ela pede um campo novo num arquivo que está em 700 LOC exatos. Sonda pronta:
  `measure_what_starting_a_watercolor_stroke_costs`.
- **O tick segue em 22-33 ms** e é a maior metade agora; ele é window-bound e a janela é 1,4× a pegada.

---

## §5.74 — E o log do PRODUTO achou o que nenhuma sonda minha podia ver: o VÉU de umidade, 42,6 ms/quadro no shell (2026-08-02)

> Enio, depois do relatório da §5.73: *"provável piora significativa mesmo com valores padrão (sem
> smudge). Melhor colocar logs em todo lugar para ver se descobre."* **As duas metades da frase estavam
> certas, e a segunda decidiu.**

### 1. Os defaults: ele estava certo em desconfiar, e a medição refina

| pincel, r=250 | 2048² | 4096² | de 16,6 ms |
|---|---|---|---|
| de fábrica | 9,6 | 9,5 | **0,6×** |
| os knobs dele | 28,0 | 28,4 | **1,7×** |

Tudo **plano na tela** depois da §5.73. Com o default o quadro CABE; com os knobs dele, não — e o dono
mudou de metade: agora é o **TICK** (6,7 → 22,9 ms), e a ablação por entrada diz **`sem Rewet` → 12,9**,
ou seja o `Rewet 0.400` paga **~10 dos 23 ms**.

### 2. Mas o log do produto mostrou outro número, uma ordem de grandeza acima

```
CHROME p50/max: wet 2.13/4.68 → 9.67/30.47 → 42.64/48.67
dispatch p50=44.7 [overlay 42.7 (chrome 42.7)]   frame p50=72.3   (~14 fps)
EVENTO->FRAME p50=88.9 max=322.3
```

O custo **não estava no tool** — estava no **SHELL**, no `draw_wetness_overlay`: o véu de umidade do
slider **`Preview`** (card Wetness, `0.300` na sessão dele). ⚠️ **Nenhuma sonda de bancada podia vê-lo**,
porque todas medem o `PainterTool`; foi o log que o achou, e não uma suspeita minha. *Quando o número
vira decisão de produto, ele tem de sair da porta do produto* — pela quarta vez nesta linha.

**O mecanismo:** por quadro, sobre o rect **CUMULATIVO** de umidade (que só cresce), ele aloca um
`f32` plane, borra-o com um box blur que aloca **mais dois**, monta um RGBA de `4·N` bytes e **faz o
upload da imagem inteira**. Medido isolado (`measure_the_wetness_veil`):

| região | M texels | build | ns/texel |
|---|---|---|---|
| 512² | 0,26 | 1,79 ms | 6,8 |
| 1024² | 1,05 | 9,22 | 8,8 |
| 2048² | 4,19 | **35,44** | 8,4 |
| 4096² | 16,78 | **201,68** | 12,0 |

Os 35 ms a 2048² casam com os 42,6 do log (o rect dele era maior, mais o upload).

### 3. A cura é livre de mudança de aparência POR CONSTRUÇÃO

**O véu só precisa cobrir o que a JANELA mostra.** O que está fora da viewport ninguém vê, então
recortar troca um custo que cresce com a **PINTURA** por um limitado pela **TELA** — e não muda um
pixel do que aparece. `clip_to_viewport` inverte a afim imagem→tela, leva os quatro cantos da janela
de volta ao espaço da imagem e intersecta com o rect de umidade.

⚠️ **A margem do blur é load-bearing:** o véu é borrado por `BLUR_R`, então um texel logo fora da
janela contribui para um que está dentro. Sem a folga, a borda do véu mudaria conforme o artista
**panha** — uma mudança de aparência que depende da posição da câmera, o tipo de defeito que ninguém
reproduz. Afim degenerada devolve o rect inteiro (uma inversa que não existe não é motivo para o véu
sumir).

### 4. Três defeitos meus, os três pegos pelos próprios gates

1. ⚠️ **A minha linha de log foi para o LOGGER ERRADO.** Eu a pus no bloco `[frame]` (env
   `PH2D_FLUID_PROFILE`) e o Enio roda `PH2D_PAINT_PERF` — **ela não apareceu no log do smoke**. *Um
   instrumento no logger errado é indistinguível de um instrumento que não existe.* Movida para o
   relatório `[paint-perf]`, ao lado da linha `CHROME wet` que é a outra metade do mesmo quadro.
2. ⚠️ **Os três gates de unidade do recorte são CEGOS à fiação:** apagada a CHAMADA, os três passam
   verdes sobre um produto que reconstrói a pintura inteira. É a lição que a `line/anim` já pagou, e a
   cura é o arch-gate que afirma *a atribuição acontece ANTES do build* (mutação: sangra).
3. ⚠️ **E o arch-gate nasceu reprovando o código CORRETO:** ele ancorava na CHAMADA e depois exigia a
   atribuição na fatia `call..build` — mas o `let` vem **antes** da chamada, na mesma linha, então a
   fatia nunca podia contê-lo. *Uma âncora só é oráculo se descrever a forma que o produto tem.*

### 5. Aberto, com número

- **O `Rewet` paga ~10 dos 23 ms do tick** (r=250, os knobs dele). É por-pixel no composite, dentro da
  janela — não é varredura de plano, é trabalho honesto de um efeito ligado. Se o smoke ainda achar
  pesado, o alvo é o kernel do rewet, não a janela.
- **O véu recortado ainda é `ns/texel` × área da VIEWPORT** por quadro (~8-12 ns/texel). Numa janela de
  1920×1080 isso é ~2 M texels ⇒ **~20 ms** se o artista estiver com a tela inteira molhada e o zoom a
  1:1. As duas alavancas seguintes são **construir em resolução reduzida** (o véu já é borrado e
  desenhado em `ImageQuality::Low`) e **não reconstruir todo quadro** (a secagem leva ~10 s, então o
  alfa muda ~1 nível por quadro) — as duas mexem na APARÊNCIA e por isso ficam para depois do smoke,
  com o número ao lado em vez de contrabandeadas.

---

## §5.75 — O véu é construído na densidade em que é VISTO: 220,8 → 16,6 ms a 4096² (2026-08-02)

> O 2º log do Enio, com o recorte da §5.74 já dentro: `CHROME wet` seguia em **12,3 → 26,0 → 37,1 ms**.
> ⚠️ **O recorte estava certo e não bastou** — ele resolve o zoom PARA DENTRO, e o artista estava
> vendo a pintura INTEIRA: aí a viewport *é* o rect molhado, e recortar não recorta nada.

### 1. O desperdício que o log expôs

Com uma pintura de 4096² cabendo numa janela de ~1000 px, o véu era montado em resolução de **IMAGEM**
para ser exibido em resolução de **TELA** — 16× de detalhe que a GPU descarta ao reduzir. *Construir
acima da densidade de exibição não é qualidade, é desperdício por definição.*

`veil_downscale` lê a escala da própria afim imagem→tela (`sqrt(|det|)`) e amostra o véu nesse passo.

| região | passo | build | ns/texel |
|---|---|---|---|
| 4096² | 1 | **220,8 ms** | 13,2 |
| 4096² | 4 | **16,6** | 1,0 |
| 4096² | 8 | **8,8** | 0,5 |

⚠️ **A média do bloco, nunca uma amostra dele:** um `nearest` num campo de umidade cintila na borda
conforme a câmera anda — e a média já é metade do desfoque que o véu quer, então o raio do blur cai
para `BLUR_R / step` (piso 1). ⚠️ **`div_ceil`, nunca `div`:** a última coluna parcial tem de existir,
senão a borda do desenho fica sem véu (a mutação sangra). ⚠️ **O passo nunca é menor que 1** (aproximar
não é motivo para superamostrar) e é **capeado em 8** (um zoom muito longe levaria o véu a um punhado
de texels e a borda passaria a piscar entre passos vizinhos — trocar custo por cintilação é o negócio
errado).

### 2. E o log fechou a conta do quadro, o que nenhuma sonda tinha feito

A linha `AQUARELA` nova (o lado do TOOL) ao lado do `CHROME wet` (o lado do SHELL):

```
composite 10,62 x47 | carimbo 2,31 x670 | pour 9,70 x47 | secagem 12,81 x90
janela 0,66 M texels/composite | 16,1 ns/texel        CHROME wet 37,05
```

**10,6 + 9,7 + 12,8 + 37 ≈ 70 contra `frame p50 = 67,3`** — os baldes reconciliam com o quadro, que é
o que separa uma atribuição de um palpite.

⚠️ **E o `ns/texel` CAIU** (23,8 → 18,4 → 16,1) enquanto a janela crescia (0,13 → 0,27 → 0,66 M): pela
leitura que a própria linha carrega, isso é **TRABALHO** e não contenção — o composite está honesto,
a janela é que cresce com o pincel.

### 3. ⚠️ E ele derrubou uma conclusão minha da §5.72

Lá eu medi a rota do pour em **1,02×** e escrevi que *"o pour não é o custo"*. No log do produto ele é
**9,70 ms/quadro**, e a `secagem` — que caminha o MESMO rect cumulativo — é **12,81**. A diferença é a
FIXTURE: eu media um traço de 1500 px numa tela limpa, e a sessão do artista acumula. *Uma medição só
vale sobre a cena que ela contém* — e as duas fases que caminham o rect cumulativo somam **22,5 ms**,
que é a próxima frente e agora tem número.

### 4. Aberto, em ordem de tamanho

1. **`secagem` + `pour` = 22,5 ms/quadro**, os dois sobre o rect CUMULATIVO. A cura é a lei que este
   doc repete: a janela vem de quem escreve. ⚠️ E a §5.73 provou que trocar o pour pelo rect do quadro
   **muda a tinta** (o `dry_canvas_wet` do mesmo tick decai o alvo), então a cura tem de ser outra —
   provavelmente o par *secar e despejar sobre o mesmo rect, uma vez, com o decaimento embutido*.
2. **O `Rewet` paga ~10 dos 23 ms do tick** (r=250) — trabalho honesto de um efeito ligado, por-pixel
   dentro da janela.
3. **`composite max 163,39 ms`** num único quadro, contra p50 de 10,62 — outlier sem causa atribuída.

---

## §5.76 — E os dois passes por-quadro da umidade caminham em PARALELO: 40,9 → 3,6 ms/quadro (2026-08-02)

O 2º log do Enio fechou a conta do quadro dele — `composite 10,62 · pour 9,70 ·
secagem 12,81 · CHROME wet 37,05 ≈ 70` contra um `frame p50 = 67,3` — e depois
que a §5.75 derrubou o véu, **os dois maiores itens que sobraram eram a SECAGEM
e o DESPEJO**, os dois caminhando a união cumulativa em todo quadro.

### ⛔ Três curas construídas, três medidas em ~1,00× — não as refaça

A hipótese natural era a FORMA do passe de secagem: ele tirava um **snapshot do
rect inteiro** (`vec![0; rw*rh]` alocado por quadro + a cópia completa) só para
que o gather de 4 vizinhos lesse valores pré-passo. Três curas byte-idênticas
saíram daí, e as três foram **medidas pela porta do produto**:

| cura | ganho |
|---|---|
| **janela deslizante** (`up` de uma linha de scratch · `left` de um ESCALAR · `down`/`right` do mapa ainda não escrito) | **1,02×** |
| **piso da erosão** (`erode = gap·step·2/255` só alcança 1 acima de `ceil(255/(step·2))`, e `gap ≤ o` ⇒ abaixo do piso os 4 vizinhos não mudam um byte) | ~1,00× |
| **o rect ENCOLHE** para a bbox do não-zero (a secagem é edges-to-centre por desenho, então a poça recua) | ~1,00× |

⚠️ **A alocação e a cópia NÃO eram o custo.** O passe custa **2,2 ns/texel**, e o
que ele faz é caminhar: 12,85 M texels a 4096² = **28,5-30,4 ms/quadro**. A
janela deslizante foi **REVERTIDA** — não por ser errada (ela é byte-idêntica e
estritamente menos trabalho), mas porque a **dependência entre linhas** que ela
cria é exatamente o que impede o paralelo que funciona. *Uma otimização que não
mede nada e bloqueia a que mede é pior que nenhuma.*

O piso da erosão e o rect que encolhe **ficaram**: valem ~1,0× no relógio, são
byte-idênticos, e o rect é lido também pelo **véu de umidade do shell** — o
consumidor que a §5.75 acabou de baratear.

### ✅ O que move: os dois passes são row-parallel

Emenda de 2026-08-02 no [ADR-0109]. Os três invariantes dele valem verbatim, e o
que a cerca de contenção nomeia — *"redução/acumulação cuja ordem importe"* — é
justamente o caso que ela isenta: a redução da secagem é `max` sobre `u8` (o
`wettest`, que decide o teardown da sessão) e `min`/`max` sobre índices (a bbox),
**associativas e comutativas sobre inteiros**; o despejo **não tem redução
nenhuma**.

Medido pela porta do produto, mesma corrida, estado restaurado antes de cada
amostra:

| | antes | agora | |
|---|---|---|---|
| secagem, um passe @4096² | 30,44 ms | **3,28** | **9,3×** |
| secagem, 120 quadros secando | 28,50 ms/quadro | **2,93** | **9,7×** |
| despejo @4096² | 12,46 ms | **0,63** | **19,8×** |
| secagem @2048² | 7,96 ms | 1,09 | 7,3× |

⚠️ O 19,8× do despejo inclui a **tabela de dureza** (`smoothstep(SS0,SS1,c/255)·255`
é função pura de um `u8` ⇒ 256 respostas), porque o oráculo congelado é o produto
pré-wave inteiro. E ⚠️ **essa corrida saiu com `load average 6,7`**: as razões
sobrevivem (as duas rotas são cronometradas costas-com-costas, a carga é fator
comum) e os **absolutos não** — a corrida calma, `load 1,0`, dava 28,87 → 3,45.

### As lições de gate, todas minhas

1. **A 1ª versão do gate de identidade usava só 128²**, abaixo do piso do pool
   ⇒ ela **nunca entrou na rota paralela que a wave instala**. Verde sobre o
   caminho que não mudou. Hoje varre os dois lados do piso.
2. **O `step` é fixture tanto quanto o tamanho.** A mutação *"leia um vizinho já
   escrito"* **SOBREVIVEU** ao gate original: a erosão é inteira, o erro vale
   ~`step`, e ele só atravessa a quantização quando `step² · 2 ≥ 255` — ou seja
   `step ≥ 12`, e eu tinha escolhido 5. O produto anda a `step = 1`, onde o erro
   é invisível; mas *"invisível no ponto de operação de hoje"* é como um vizinho
   errado vive até alguém mexer no Drying Time. O gate varre 1/5/17/51.
3. **O gate da tabela era uma TAUTOLOGIA** — comparava a LUT com a função que a
   constrói. Mudar a expressão movia os dois lados. O oráculo é a lei ESCRITA no
   teste. (A `line/physics` documentou essa forma em três gates.)
4. **Comparar as duas rotas do PRODUTO provaria só o walker**, porque as duas
   compartilham o mesmo corpo — a lição do [ADR-0145]. Os dois oráculos são
   rotinas congeladas sob `cfg(test)`, e é por isso que a mutação *"o `base`
   esquece o offset da banda"* sangra.
5. **Serial e paralelo dão os MESMOS bytes** — que é a propriedade que torna a
   cura segura, e exatamente por isso nenhum gate de identidade pega a regressão
   de **uma letra** (`par_chunks_mut` → `chunks_mut`). Daí o arch-gate, irmão do
   que o fold do impasto já carrega neste crate.

**7 mutações vivas, 7 sangram.** Suíte 966 verde, clippy limpo, `PROJECT_SCHEMA`
intocado, nenhuma dep nova (o `rayon` já era desta crate desde o ADR-0109).

**Aberto, com número:** o `composite` (10,6 ms p50, 163 ms de pico num quadro
isolado, ainda sem causa atribuída) e o `Rewet 0.400`, que paga ~10 dos 23 ms do
tick e é trabalho honesto por-pixel dentro da janela.

[ADR-0109]: ../architecture/decisions/0109-rayon-exception-watercolor-composite.md
[ADR-0145]: ../architecture/decisions/0145-wet-paint-solver-row-parallel-passes-rayon-exception.md

---

## §5.77 — E o composite virou a fronteira: a atribuição, e o piso de banda que a fecha (2026-08-02)

O smoke do Enio validou a §5.76 — *"pela primeira vez consegui pintar uma imagem
de 4096 com fluidez **nos parâmetros padrão** da aquarela"*. A palavra que abre
esta seção é **padrão**: na foto dele o **Rewet estava em 0,400**, e é ele que
sobra.

### O que o Rewet cobra, e por quê

Sonda nova `measure_the_window_the_composite`, que pede ao próprio `wash_diag` o
divisor que o log do produto publica:

| | composite | janela | ns/texel |
|---|---|---|---|
| Rewet 0,400 (Enio) | **19,40 ms** | 0,38 Mtex | 50,8 |
| Rewet 0 (padrão) | **8,18 ms** | 0,23 Mtex | 35,1 |

⚠️ **Ele cobra nas DUAS pontas**: a janela cresce 1,65× (o `reach` do pad sai de
`core_r` para `spread`, e **dobra** sob `soaked || watered`) *e* o custo por
texel cresce 1,45×. Não é um dos dois — são os dois.

### A decomposição, e a lição de como quase a li errado

⚠️ **Eu instrumentei o composite por estágio, li a tabela do PRIMEIRO e a chamei
de "o composite".** Ela dizia `substrato 6,67 ms de 20,7` — porque no primeiro
composite o cache de substrato está **frio** e tudo é falha. O `wash_diag`
reporta a **MÉDIA**. Com a média de 14 composites quentes:

| estágio | delta |
|---|---|
| A `cov_src` + B `hard` (seriais) | 0,36 |
| C blur do feather | 0,73 |
| **D `build_rewet_fields`** | **8,74** |
| E campos de estilo | 1,79 |
| F substrato | **0,08** |
| **H laço paralelo** | **5,45** |

*Um custo de UMA VEZ é invisível numa média, e uma média é invisível numa amostra
de uma vez.* As duas metades da mesma armadilha, no mesmo dia.

### ✅ O que entra

**`fill_substrate_cache` era serial POR DESENHO** — o doc do chamador dizia que o
pré-passe enche as falhas *"serialmente para o laço paralelo ler imutável"*. A
segunda metade continua verdadeira; **a primeira nunca foi necessária**. Row-
parallel sob a mesma emenda do ADR-0109, e o caso mais simples da família:
`paper_h_px` é função pura de `(x, y)`, escritas disjuntas, **zero redução**.
Medido **19,40 → 18,49 ms**: os 0,9 ms são o custo dos quadros **FRIOS**, que é
exatamente onde ele estava.

### ⛔ E o que foi medido e NÃO é resultado

**O `box_blur` alocava o buffer de prefixo por LINHA** — ~12 mil alocações por
quadro nas dez chamadas. Trocado por `for_each_init` (um buffer por *task*,
mesma aritmética na mesma ordem ⇒ byte-idêntico): **18,49 → 18,28, dentro do
ruído**. Fica por ser estritamente menos trabalho, **não por ser um ganho** — o
mesmo veredito do `value_noise_pair` (§5.11) e das três curas da §5.76.

### O que fica NOMEADO, com número

**`build_rewet_fields` = 8,74 ms, 51% do composite**, e são **DEZ box blurs em
resolução CHEIA** (4 *near* em `r1`, 4 *far* em `r2`, mais os halos de soak e
água) — o downsample `ds` fica em **1**, porque o Spread do artista não alcança o
limiar `REWET_DS_SPREAD`. Dentro dele: preencher 0,88 · os 4 *near* 3,27 · os
halos + *far* 4,51.

⚠️ **E o blur não é o problema que parece:** ele já é **O(n) por prefix sums e já
é paralelo**. A 2,1 ns/texel sobre ~12 MB movidos por chamada, ele está no **piso
de largura de banda** — as duas curas de constante que tentei (alocação por linha,
e antes disso o cache de substrato) mediram 0,2 e 0,9 ms.

**As duas alavancas que restam são decisões de PRODUTO, não de engenharia:**

1. **Baixar `REWET_DS_SPREAD`** para o downsample disparar em Spread pequeno.
   Corta o custo por `ds²` — e **muda o LOOK** (o campo de rewet passa a ser
   aproximado onde hoje é exato).
2. **Cachear os campos derivados da base congelada.** `pres`/`wr`/`wg`/`wb`
   dependem só de `base` (a base da sessão) e `ground` (o backdrop) — **os dois
   congelados pela sessão** —, então os blurs deles são constantes canvas-ancoradas
   e são recomputados todo quadro. O preço é **4 planos canvas-sized de `f32`** —
   268 MB a 4096², que é a classe de número que o ADR-0117 existe para não deixar
   passar sem medição.

**Nenhuma das duas é minha para escolher.** O que a sessão entrega é o número ao
lado de cada uma.

---

## §5.78 — Sob a mão, o GIZMO é o preview: o move da cena do report 308 → 0,02 ms (2026-08-07)

Pedido do Enio: *"temos boa performance de modo geral, exceto usando as shapes
vivas. Que acha de ligar o pigmento apenas no mouse up quando o usuário estiver
em repouso?"* — e, depois do smoke da primeira versão: *"mesmo o preview plano
(digital comum) é extremamente custoso e numa imagem de 4096, 4 círculos com
boolean +, fps cai para 2 ou menos. Minha ideia é deixar só as linhas do gizmo."*

### A v1, e o que a derrubou

A primeira versão desarmava o **MEIO CARO** (o corpo do Impasto, a lavagem da
Aquarela) durante o gesto e media, pela porta do artista, **99,10 → 5,66 ms**
num move de figura única. O número era real e a wave estava gateada.

⚠️ **E ela não resolvia o problema do Enio, porque a fixture não era a cena
dele.** A minha usava **UMA** elipse de 400 px com pincel r=96; a dele tem
**quatro** círculos de 900 px com Operation **Add** num 4096. Medida
(`measure_boolean_cost::measure_the_scene_the_report_describes`):

```text
formas  raio    EVENTO   geom (boolean)   carimbo
     1   120     90,79        79,16        11,64
     4   120    308,16       284,10 (92%)  24,06 (8%)
     4    40    289,23       280,85         8,38
```

**308 ms/quadro é o `2 fps ou menos` do report** — a cena reproduz. E a
atribuição é o achado: **a tinta era 8%**. Os 92% são o composite booleano —
rasterizar as quatro figuras num buffer supersampleado (142 ms) e traçar os
contornos (124) — e ele **quase não responde ao pincel** (r=120 dá 308 ms,
r=40 dá 289). Desarmar só o meio caro levaria 308 para ~300.

*Um ganho de 16× medido numa fixture que não contém o fenômeno não é um ganho.*

### ⛔ Medido e rejeitado — não refaça

**Rascunhar o composite em resolução menor.** `SS` de 3 para 1 leva a cena a
**60,10 ms** (5×, ainda 16 fps) **e muda o desenho sob a mão**: o contorno cai de
30.884 para 10.291 pontos, visivelmente mais grosseiro. Não há ponto de operação
nessa direção — ela paga aparência e não chega a lugar nenhum.

### A lei que shipou

Enquanto um gesto de figura está em voo, o `restamp_shapes_preview` **descasca o
preview e volta**: nenhum composite, nenhum carimbo. O guia amarelo (perímetro +
alças) e os badges de Operation já são desenhados pela shell no vector scene,
**fora do carimbo**, e portanto de graça.

```text
formas  raio    MOVE (antes → depois)      SOLTAR
     1   120     90,79 → 0,018             111,3
     4   120    308,16 → 0,019             370,2
```

⚠️ **O custo não sumiu — ele mudou de lugar, e isso está nomeado:** soltar custa
**370 ms** nessa cena, uma vez por gesto em vez de por quadro. É a diferença
entre *inutilizável* e *uma pausa ao largar*, e a cura da pausa é a **rota
analítica** do [doc 35 §4](../../../Painter/35_boolean_o_que_o_vector_ensina.md) — o booleano
sobre CURVAS em vez de sobre pixels, `O(segmentos)` em vez de `O(área)`. A wave
de 06/08 já havia levado a rota de raster ao piso do método; esta lei é o que
torna a cena usável enquanto aquela decisão não é tomada.

### A metade que não era sobre velocidade

⚠️ **Nenhuma captura de undo pode ver a tela rascunhada.** Um `ModelSnapshot`
guarda o `drag_preview` como `preview_patch`, e a primeira montagem escrevia a
figura de volta **depois** de o editor commitar — uma escrita estrangeira
pós-commit, que o `undo_absorb` (doc 28 §5.14 da jornada do journal) **absorve no
passo anterior**. Consequência medida: o **redo** de um arrasto de curva
devolvia uma cena **sem a curva** (`curve_overlay()` = `None`).

Os **dois gates de undo que já existiam** nasceram vermelhos com isso
(`a_curve_point_move_is_undoable_and_redoable`,
`line_fillet_persists_through_undo_redo_snapshot`) — e é por isso que a mutação
*"o commit deixa de assentar"* é a mais forte da wave: ela sangra pelos gates de
outro sistema, escritos anos antes, que sabiam o que este desenho não sabia.

A cura é uma porta: **`settle_shape_draft`**, chamada por `commit_shape_txn`
ANTES de capturar. O sinal é o FATO (`shape_stale`), nunca uma lista de quais
ramos de Up de quais editores re-carimbam — o ramo `editing` do `ellipse_up`
fecha a transação de undo e sai por `return true` **sem re-carimbar**, e sem o
fato guardado a figura ficaria invisível depois de todo arrasto de ajuste, que é
exatamente o gesto reportado.

### O que a wave custou em fixtures alheias

**Seis** gates de `stamp_banded`/`stamp_device` dirigiam `Down` + `Move` e
mediam o carimbo. Com a lei, um Move não produz lote nenhum — eles passaram a
**SOLTAR**. Não é afrouxamento: o assunto deles (*a estrada em banda roda numa
figura viva*) continua verdadeiro, só que **em repouso**.

### Verificação

5 gates, **5 mutações, 4 sangram**. A 5ª — o `settle` do Up — fica
**DOCUMENTADA**: hoje todo Up dos quatro editores passa por `commit_shape_txn`,
que já assenta (varrido pelo `no_editor_leaves_the_canvas_owing_at_rest`), então
a linha é no-op. Ela fica porque as duas portas existem por razões **diferentes**
— aquela é da CORREÇÃO do undo, esta é da FEATURE (o artista tem de ver a figura
ao soltar) —, e delegar a segunda à primeira faz a lei depender de todo Up abrir
transação, que é a enumeração que este desenho evita.

### Aberto

O arrasto de um **slider do painel** também re-carimba por quadro e **não** passa
por esta porta (ele não tem Down/Up de canvas). Se afinar um knob sobre uma cena
booleana engasgar, a cura é fiar o `held_button` da shell no mesmo `shape_draft`
— mesma lei, segundo fio.

## §5.79 — O contorno é o que se VÊ e o que se CLICA (2026-08-07)

O smoke da §5.78 aprovou a lei e trouxe **dois reports que são o mesmo defeito
visto pelos dois lados** (Enio):

> *"O gizmo está invisível ao ser criado. E fica invisível ao criar outro
> círculo. Mas agora precisamos de todos os gizmos sempre visíveis."*
> *"Se clicar dentro de uma forma já desenhada, não aceita desenhar outra."*

⚠️ **Os dois são consequência de uma premissa que a §5.78 invalidou sem que
ninguém reconferisse.** Enquanto a TINTA aparecia sob a mão, uma figura podia ser
representada por uma **CAIXA**: o desenho pintado dizia onde ela estava, e a
caixa só precisava dizer *"ainda dá para clicar aqui"*. Com o gesto rascunhado a
caixa virou a **única coisa na tela**, e as duas metades do acordo quebraram de
uma vez:

- **o que se VÊ** — `stroke_op_badges` dava uma moldura AABB apagada por figura
  parqueada, e o `ellipse_overlay`/`polygon_overlay` devolvia `None` até o Up
  (*"`None` until the radius drag is released"*, escrito no próprio tipo). Quatro
  círculos viravam quatro retângulos, e o círculo em criação, nada;
- **o que se CLICA** — `hit_parked_shape_bbox` aceitava o **INTERIOR** da caixa.
  A caixa de um círculo cobre `4/π` da área dele, então o passo 4 do
  `maybe_switch_or_new_shape` (*reativar uma parqueada*) engolia todo Down e o
  passo 5 (*parquear a ativa e começar outra*) ficava **inalcançável**. O sintoma
  se lê como intermitente porque depende de o clique cair fora de TODAS as
  caixas.

### A cura é uma porta só

`stroke_outline.rs` produz **o contorno**, e os dois consumidores que precisam
concordar leem dele: o gizmo que a shell desenha e o hit-test que decide se um
clique alcança a figura. ***O que é DESENHADO é o que é CLICÁVEL.***

A lei de cada família **não é escolha** — ela espelha o gizmo ATIVO daquela
família, senão reativar uma figura a faria SALTAR: Ellipse/Polygon com o **Offset
aplicado** (como os `*_overlay` deles), Curve/Line **PRISTINO** (o offset é
*drawing-only*, Enio 2026-07-05).

O hit-test tem **duas** regiões e as duas são coisas que o artista vê: o contorno
e o **quadrado central do badge** (com o glifo de Operation dentro). Alvo
desenhado que não responde é a metade oposta do mesmo defeito.

E as **ALÇAS** seguem a fase (`EllipseOverlay::editing`, o espelho exato do
`LineOverlay::editing` que já existia): o contorno aparece desde o primeiro pixel
do arrasto, as alças só depois do Up — no meio do arrasto de criação nenhum Down
as alcança (`ellipse_down` sai por *"mid radius-drag — ignore extra Downs"*), e
alça desenhada que não responde é chrome morto.

### O bônus medido: o badge parou de construir DABS

`shape_state_bbox` chama `parked_shape_dabs`, ou seja **monta a lista de dabs
inteira de cada figura parqueada, a cada quadro**, para guardar quatro floats —
exatamente o que o header de `measure_idle_frame_of_a_live_shape` já nomeava como
*"o suspeito"*. O contorno é a linha, não o pigmento.

A/B costas-com-costas, mesma corrida, máquina calma (`load 3,5`),
`measure_idle_frame_of_a_live_shape`, coluna `badges` em µs:

| parqueadas | caixa (via DABS) | contorno | razão |
|---|---|---|---|
| 1 | 7,0 | 1,3 | 5,4× |
| 2 | 13,4 | 2,7 | 5,0× |
| 4 | 26,9 | 5,1 | 5,3× |
| 8 | 57,7 | 10,8 | 5,3× |
| 16 | 105,0 | 19,8 | 5,3× |

⚠️ **A cena da sonda é PEQUENA** (figura de 400 px, raio 48) — o valor absoluto é
de microssegundos. O que a tabela entrega é a **razão**, constante em 5,3×, e o
fato de o custo ter deixado de crescer com o TAMANHO da figura (a lista de dabs
cresce com a área varrida; o contorno, não).

### O que as mutações ensinaram

**7 mutações, 7 sangram** — e a que importa é a que **sobreviveu na 1ª rodada**:
*"tire o quadrado central do hit-test"* passava em tudo. O buraco era da
**FIXTURE**: os três círculos estavam tão próximos que o centro do primeiro caía
a **1,66 px do contorno do segundo**, então o gate do quadrado central passava
pelo ramo do CONTORNO — verde pelo motivo errado. Refeita com **≥ 45 px** de
folga entre todo centro/sonda e QUALQUER contorno, cada ramo é medido sozinho e a
mesma mutação sangra.

⚠️ **E quatro gates PRÉ-EXISTENTES pinavam o MECANISMO em vez da intenção:** *"no
handles while drawing"* estava escrito como `overlay().is_none()`. A intenção
sobrevive intacta — o que mudou foi onde ela se afirma (`editing` /
`points.is_empty()`). *Um gate que afirma o mecanismo em vez da propriedade
reprova a correção junto com a regressão.*

### LOC

`curve.rs` cruzou o teto e o corte foi por **assunto**: o snapshot read-only saiu
para `curve_overlay.rs` — módulo **FILHO**, não irmão. Ele lê quatro campos
privados do `CurveEditor` (`grab`/`freehand`/`anchor`/`draft_to`), e em Rust um
filho enxerga o privado do ancestral; um irmão obrigaria a alargar os quatro para
`pub(super)` só por causa do teto, que é **mover o problema de lado** em vez de
cortá-lo (a lição do corte que levou `paint.rs` de 596 a 613).

### O segundo fio: o PAINEL (2026-08-07)

Pedido do Enio depois do smoke: *"o mesmo mecanismo de apagar o preview deve ser
aplicado quando se está mudando os parâmetros do painel para shapes vivas (Size,
Offset, etc.)"* — o item que esta seção e o handoff já deixavam **nomeado em
aberto**.

Um arrasto de **knob** re-carimba a figura INTEIRA a cada quadro: o mesmo
trabalho, pela mesma porta (`restamp_shapes_preview`), sem passar pelo
`route_shape_draft` (não há Down/Up de canvas). O fio que faltava é
`PainterTool::set_shape_draft_hold(bool)`, alimentado pelo **`held_button`** da
shell.

⚠️ **O sinal não foi escolhido por conveniência:** é a MESMA porta que o
`post_frame_undo` consulta para *"um arrasto é UM passo de undo"*. A pergunta
*"estamos no meio de um gesto?"* já tem dono nesta shell, e inventar um segundo
sinal é a segunda resposta que diverge no dia em que um deles ganhar um caso
especial.

**Campo próprio** (`shape_draft_hold`), e não uma 2ª escrita no `shape_draft`:
são dois gestos com donos diferentes (o roteador de ponteiro × a shell). Cada um
escreve o seu; quem LÊ faz o OU — *a pergunta é uma só, as respostas é que vêm de
dois lugares*.

**A fiação tem DOIS sítios, cada um por um motivo:**

- **armar** no drain de `ToolPanelEvent`, porque **é o próprio edit que
  re-carimba**. Publicado um quadro depois, o 1º quadro do arrasto pagaria o
  composite inteiro (~300 ms na cena do report) e o engasgo apareceria ao *pegar*
  o slider;
- **soltar** no `painter_bridge::dispatch`, que roda todo quadro, porque quando o
  artista solta **não chega evento de painel nenhum**. Sem ele a figura ficaria
  fora da tela até o próximo edit — o pior modo de falha desta lei.

E o `settle` derruba as **duas** bandeiras: assentar promete uma tela honesta
AGORA, e uma bandeira de pé faria o próximo re-carimbo descascar o que acabou de
voltar.

⚠️ **A lição de gate:** a 1ª versão chamava `set_brush_size_px` — o **setter
cru**, que só escreve o número; quem decide re-carimbar a figura aberta é o
`handle_panel_event` (`refill_if_appearance_changed`). O gate media o
**SILÊNCIO**: a tela ficava com o carimbo anterior e ele passava verde com a lei
desligada. *Dirija a porta do artista, não o campo que ela escreve.*

7 mutações, 7 sangram — incluindo o controle que separa *"a mão está no knob"* de
*"um valor mudou"* (`a_panel_edit_with_no_hand_on_it_leaves_the_shape_on_screen`,
que sangra com o `draft_stamp` cravado em `true`).

### O Delete apaga a figura em mãos (2026-08-07)

Pedido do Enio: *"permita usar del para deletar a forma selecionada por último"*.

O verbo (`delete_active_shape`) é o **gêmeo exato do `park_active_shape` menos o
empurrão**: parquear guarda a geometria na lista, apagar a joga fora; as duas
depois limpam os slots vivos e re-carimbam o conjunto que sobrou sobre a linha de
base pristina. Nada mais precisa saber que uma figura sumiu, porque **os pixels
são cache DERIVADO do conjunto** — a invariante do cabeçalho do `stroke_multi`.

⚠️ **Não é o Esc.** O `cancel_open_shape` descarta o conjunto INTEIRO (ativa +
parqueadas) e não deixa passo de undo, porque nada foi commitado. Este apaga UMA
figura, deixa as outras de pé, e é **um passo de undo** — o artista tem de poder
trazê-la de volta. E nada fica selecionado depois, de propósito: promover uma
parqueada escolheria por ele qual, e a próxima que ele quer é a que ele vai
clicar.

**A PRECEDÊNCIA é a feature.** Há uma tecla só e três donos: âncora de curva →
**figura** → falloff. Com um nó selecionado numa curva o Delete tira o nó (o
`curve_delete_selected` já se gateia nisso e recusa quando sobrariam menos de 2
pontos); sem nó, tira a figura. É a divisão que o Illustrator faz com **duas
ferramentas** (seta branca × seta preta) — aqui é uma tecla, então a ordem é o que
a expressa. ⚠️ E **um gate de unidade não vê isso**: cada verbo, chamado sozinho,
está certo; o que erra é a ordem, e ela só existe no roteador.

⚠️ **O degrau novo fecha um buraco que já existia:** sem ele, um Delete com figura
viva caía no caminho genérico do hero e apagava a **ENTIDADE** — o sprite inteiro,
com a arte dentro.

**5 mutações, 5 sangram** — e a lição é de oráculo: *"não re-carimbe o conjunto
que sobrou"* matava só o gate de **undo**, enquanto o gate cujo NOME promete *"as
outras continuam de pé"* media a **LISTA** de badges e não a **TELA**. Com a lista
intacta e a tinta da figura apagada ainda no canvas, ele passava. *O oráculo do
que o artista vê são os texels.*

**LOC:** três arquivos cruzaram o teto e cada corte foi por assunto —
`stroke_router.rs` (*o que um Down SIGNIFICA quando há mais de uma figura*) e
`keyboard_painter.rs`, que leva a cadeia **e os verbos dela**: eles não são
entrada de CANVAS, são o que uma TECLA faz, e vê-los ao lado da ordem que os
chama é o que torna a precedência auditável de um relance.
