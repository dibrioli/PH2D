# 28 — Otimizações do Painter: o que funcionou, o que NÃO funcionou, e o que serve aos outros modos

> **Este doc é o registro de uma jornada de perf inteira** (2026-07-26, `line/Painter`), escrito para que
> ninguém reconstrua o que já foi medido e reprovado. O plano operacional vive no
> [26_plano_performance_procreate.md](26_plano_performance_procreate.md); aqui está o **saldo**: cada
> frente com o número que a matou ou a aprovou, e o mecanismo por trás do número.
>
> ⚠️ **A regra que governou tudo (CLAUDE.md §0):** nenhum limite, nenhuma barra e nenhum veredito sem
> MEDIÇÃO. Três teorias minhas, plausíveis e erradas, morreram nesta jornada — e cada uma está aqui com
> o número que a derrubou, porque uma teoria refutada que não é escrita volta como trabalho planejado.
>
> **O saldo, numa linha:** o frame pior de um traço a 4096² saiu de **232,7 ms** para **1,1–1,3 ms**
> (§4.8.3, medido no app pelo Enio), o `dispatch p50` ficou em **0,7 ms — 4% de um quadro de 60 fps**, e
> o censo dos quatro meios não tem mais nenhum desvio de FORMA: **todo move é limitado pela pegada**
> (§5.12 fechou o último, o Wet Paint, 13,71 → 1,82 ms a 4096²). E o **pen-up**, que a §5.13 atribuiu ao
> fork do relevo, era **91% commit de undo** — o scan do histórico ficou paralelo e o commit caiu
> **25,03 → 10,96 ms** a 4096² (§5.14), com a errata da atribuição escrita no lugar onde ela foi feita.
>
> ⚠️ **E é por isso que a §7 foi reescrita três vezes.** Cada medição não só respondeu a pergunta: ela
> **mudou qual era a pergunta**. A última tirou o vencedor da mesa — com o dispatch em 0,7 ms, o custo
> que estava escondido atrás dele (`INPUT (fora do frame)`, 5,3–8,8 ms) virou a fronteira. *Um doc de
> perf cuja fila nunca se reordena é um doc que parou de medir.*

---

## 1. O placar

| # | frente | veredito | número |
|---|---|---|---|
| A | **LUT pré-convoluída do filme** | ✅ **SHIPOU** | traço **134,84 → 110,22 ms** (−18,3%) virgem · **167,79 → 142,96** (−14,8%) sobre tinta |
| B | Undo por DELTA da janela (U1) | ✅ shipou | passo **67,8 → 2,36 MB** · retido **1.627 → 242 MB** |
| C | **Coalescência de eventos de ponteiro** | ⛔ **construída e REVERTIDA** | **1,00×** — o ganho aparente era **+86% de dabs**, não orla de lote |
| D | Fundir as duas varreduras de silhueta | ⛔ **impossível** | premissa falsa: pigmento supersampleia o DISCO, altura a CÁPSULA |
| E | AA com 5 amostras (estimativa separável) | ⛔ construído e **REJEITADO** | `Constant` erra **2/9 exato = 56,67 níveis** em TODO raio |
| F | Gatear o AA por raio | ⛔ rejeitado | **105.660 bytes** diferem, pior delta **62** — muda a arte |
| G | Encolher a tabela pelo cache | ⛔ **refutado por medição** | N=512/1024/2048/16384 → **110,29 / 109,99 / 111,79 / 110,22 ms** |
| H | Reusar a alocação para matar o page-fault | ⛔ **refutado por medição** | com o buffer já mapeado a cópia é **11,68 dos 12,35 ms** ⇒ a alocação vale **5%** |
| I | **Latência do pen-down — o fork SERIAL** | ✅ **fechada** (§4.3) | 4096² digital **10,3 → 3,9 ms** · impasto **18,6 → 12,0** |
| J | **Latência do pen-down — o resto** | 🟡 aberto, e **menor do que parecia** | contra um MOVE: fork **3,4** + planos **1,8**; o resto (5,5) é **um dab comum** (§4.5) |
| M | O 1º traço compilava pipelines | ✅ fechada (§4.8), mas **não era a causa** | ~10-28 ms (varia com a ordem), e **independente da tela** — o smoke refutou (§4.8.1) |
| N | 🎯 **O 1º traço ALOCAVA as texturas** | ✅ **fechada** (§4.8.1) | **0,76 / 2,72 / 13,21 ms** a 1024²/2048²/4096² — a escada que o Enio descreveu |
| **O** | 🎯🎯 **O 1º traço dobrava o CANVAS INTEIRO** | ✅ **fechada e CONFIRMADA no smoke** (§4.8.2–§4.8.3) | `dispatch max` **232,7 → 1,1–1,3 ms** no app · `p50` **0,7** (4% de um quadro). O fold **201,53 → 14,55** headless |
| P | Semear os planos da luz no BIND | 🟡 aberto, **decisão de PRODUTO, e o preço agora é MEDIDO** | vale **12,7 ms** no produto (era estimativa minha de ~31) e cobra **~218 MB de VRAM em TODO bind** — inclusive de quem nunca liga o impasto |
| **Q** | 🎯 **`INPUT (fora do frame)` — o carimbo de dabs** | ✅ **DECOMPOSTA e a maior parte FECHADA** (§5.13 → **§5.14**) | o pior evento é o **PEN-UP**, e **91% dele era o COMMIT DE UNDO** (40,20 completo × **3,49** sem ele, este plano na tela). ⛔ a atribuição da §5.13 (o fork dos 3 planos, 28,47) fechava **por coincidência** — memcpy serial contra `fork_par` paralelo, que custa **9,25**. Scan paralelizado: `record_structural` **25,03 → 10,96** · pen-up **40,20 → 32,34** |
| R | O outlier de **134,8 ms** num evento | 🔴 **segue aberto** (§5.13/§5.14) | o maior evento REPRODUZÍVEL é o pen-up, **38,9 ms** — não chega lá. ⚠️ mas uma amostra ÚNICA de pen-up já foi medida em **117,76 ms** sem nada estar errado ⇒ o 134,8 é compatível com um pen-up num instante ruim; decide um 2º log |
| S | `EVENTO→FRAME` 16,8 contra alvo **9** | ⚪ **não é compute** | `p50 ≈ periodo real (16,5)` ⇒ é **cadência**, e o dispatch é 4% dela |
| **T** | 🎯 **O WARP é 56% do custo da aquarela** | ✅ **medido** (§5.10) | 1,071 ms de um move de 3,082 · 10 avaliações/texel · a tabela trouxe o próprio **controle** (2 knobs já em 0 ⇒ piso de ruído ±0,13) |
| U | Fatoração exata dos 2 eixos do warp | ✅ shipou, **e não é ganho de PRODUTO** (§5.11) | função **1,20×** (153,4 → 127,9 ms/4 M) · produto 0,12–0,17 ms = **dentro do ruído** |
| **V** | 🎯🎯 **O move do Wet Paint escala com a TELA** | ✅ **FECHADA** (§5.12) | o token de identidade do guard era um `Arc` FORTE ⇒ todo composite copiava o documento. **13,71 → 1,82 ms a 4096² (7,5×) e PLANO** (1,842 / 1,815 / 1,817); pen-up **17,3 → 5,05** |
| K | **A tabela lida FORA da banda** | ✅ **fechada** (§4.6.1) | AA **2,60 → 1,43 ms/dab** · traço **110,2 → 96,9** virgem, **143,0 → 130,2** sobre tinta |
| L | Colapsar a grade em 3 leituras | ⛔ **construído (2 formas) e REJEITADO** | **4,949** e **5,344** níveis contra **0,060** da grade. Casar mais um momento PIOROU ⇒ o erro não é dos momentos, é das QUINAS de `F` (§4.6.2) |

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

## 3. ⛔ O que NÃO funcionou (e o mecanismo de cada um)

### 3.1 Frente C — coalescência de eventos de ponteiro: **construída e revertida no mesmo dia**

**A hipótese:** o mouse entrega ~60 eventos/s e cada um vira um batch; agrupá-los daria uma orla de lote
maior e menos re-trabalho de borda.

**Medido: 1,00×.** E o *"+84%"* que a sonda inicial mostrou era **+86% MAIS DABS** (21 contra 39)
pintando **exactamente os mesmos 177.760 pixels**.

⚠️ **O mecanismo, e é o que torna a frente morta em qualquer modo:** `stamp_dabs` percorre **a pegada de
CADA dab**, não uma janela por batch. Agrupar eventos não reduz trabalho nenhum — só muda quantos dabs o
espaçamento produz sobre o mesmo caminho. **Não tente de novo, em modo nenhum.**

A wave inteira (módulo, porta única de flush, 4 gates, 3 mutações, contrato congelado intacto) foi
revertida. O que sobreviveu é a sonda que a matou, em `measure_input_cost.rs`.

### 3.2 Frente D — fundir as duas varreduras de silhueta: **premissa falsa**

O doc afirmava que o pigmento e a altura tomam *"a MESMA fração"* da silhueta, então uma varredura
serviria as duas. **O código diz o contrário:** o pigmento supersampleia o **DISCO** (`dab/bands.rs`), a
altura supersampleia a **CÁPSULA VARRIDA** (`height.rs`). São geometrias diferentes sobre a mesma lista
de dabs. Não há o que fundir.

### 3.3 Frente E — AA com 5 amostras: **construído, medido, rejeitado**

**A ideia:** amostrar de verdade só a cruz (centro + 4 axiais) e estimar as 4 QUINAS pela extensão
separável `s(ox,oy) ≈ s00 + [s(ox,0) − s00] + [s(0,oy) − s00]` — exata para qualquer silhueta separável.

Morreu em duas frentes:

1. **`Constant` erra `2/9` EXATO — 56,67 níveis de u8 — em TODO raio e em TODOS os texels da banda**
   (788 de 788 a r=100). Com borda dura o `film_of` é um DEGRAU e a extensão separável erra dois dos
   nove termos **por construção**. É o caso pelo qual o AA existe;
2. **o erro dos falloffs suaves NÃO é monotônico no raio:** `Sharp` erra 0,13 nível a r=40, **2,02 a
   r=50**, 0,03 a r=60, **2,01 a r=70**. Picos isolados em raios arbitrários ⇒ **nenhum limiar de raio
   limita o erro**, e um limiar tirado da tabela seria o *"limite que só diz por segurança"* que o §0
   proíbe.

⚠️ **E não há troca de quadratura que salve:** com menos amostras a cobertura de um DEGRAU fica
quantizada mais grosso (4 pontos ⇒ quartos, contra nonos), então uma grade rotacionada de 4 é *pior*
justamente no caso 1. **O custo do AA é a contagem de amostras, e a qualidade dele também é.**

⛔ E o `clamp(0,5 − f/|∇f|)` dos livros (o AA de SDF) foi rejeitado ANTES, no papel: ele troca a *curva*
junto com as amostras, assumindo que a cobertura é a área de um lado de uma reta — e `film_of` não é
isso (é uma S saturando em `W_SOLID`).

O `film_at_exact` nasceu dessa tentativa e **FICA**: é o oráculo contra o qual a LUT é medida.

### 3.4 Frente F — gatear o AA por raio: **não é de graça**

Desligar o AA abaixo de um raio faria **105.660 bytes** diferirem, com pior delta **62**. Isso não é uma
otimização, é uma mudança de arte.

### 3.5 Frente G — encolher a tabela pelo cache: **teoria plausível, REFUTADA**

`N = 16384` são 64 KB, que estouram o L1, e a teoria óbvia é que os 9 lookups por texel pagam por isso.

**Medido, mesma sonda:** N = 512 → **110,29** · 1024 → **109,99** · 2048 → **111,79** · 16384 →
**110,22 ms**. ⚠️ **O tamanho é IRRELEVANTE** — texels vizinhos leem `t` vizinho, então a tabela fica
quente em qualquer tamanho. `N = 16384` fica pela precisão, **de graça**.

### 3.6 Frente H — reusar a alocação para matar o page-fault: **refutada por decomposição**

A receita do §13.12.5 dizia *"reuso da alocação mata o page-fault"*. Medido: com o buffer **já mapeado**
a cópia custa **11,68 dos 12,35 ms** ⇒ **a alocação vale 5%**. A metade útil da receita é a OUTRA:
captura do "antes" por **REGIÃO sob demanda**.

---

## 4. 🔴 O que ficou ABERTO — e é exactamente o que o Enio sente

> *"bem melhor e bastante aceitável com exceção do delay do primeiro traço que precisamos resolver"*
> (Enio, 2026-07-26)

### 4.1 A medição, e a hipótese que ela derrubou

Sonda `the_first_stroke_latency` (`measure_impasto_cost.rs`), **ms por pen-down**:

| tela | modo | 1º pen-down | pen-downs seguintes | move |
|---|---|---|---|---|
| 2048² | digital (controle) | 3,41 | 2,79 · 2,83 · 3,32 · 3,02 | ~0 |
| 2048² | **impasto** | 12,06 | **13,22 · 13,12 · 10,78 · 10,47** | ~0 |
| 4096² | digital (controle) | 11,54 | 10,55 · 10,27 · 10,29 · 10,18 | ~0 |
| 4096² | **impasto** | 18,06 | **18,37 · 18,96 · 18,71 · 18,80** | ~0 |

⚠️ **A hipótese "é a alocação lazy dos planos na estreia da camada" está REFUTADA:** o custo **não cai**
depois do primeiro — é **POR GESTO**. O nome certo do defeito não é *"o delay do primeiro traço"*, é
**"o delay de TODO pen-down"**; o artista o nota no primeiro porque é quando ele está esperando o traço
começar.

E o número é o veredito: **a 4096² com impasto, todo toque no canvas custa mais de um frame de 60 fps**
(18,5 ms contra 16,7). A 2048² são 72% de um frame.

### 4.2 O mecanismo, pela aritmética

O custo escala com **bytes do documento**, não com a pegada do pincel:

* digital = **4 B/px** (o RGBA): 2048² = 16 MB → 3 ms · 4096² = 64 MB → 10,5 ms;
* impasto = **4 + 12 B/px** (`heights` f32 + `covers` u8 + `mats` [u8;7]): 2048² = 67 MB → 12 ms ·
  4096² = 268 MB → 18,5 ms.

É **cópia canvas-sized limitada por largura de banda de memória**: o pen-down clona o documento inteiro
para ter um estado "antes" do gesto.

### 4.3 ✅ PRIMEIRA METADE FECHADA — o fork do canvas era SERIAL

⚠️ **A cura mais barata já estava no repo e o depósito não a usava.** O `plane_fork::fork_par` — um
`Arc::make_mut` com a cópia **paralelizada**, semanticamente idêntico por construção e gateado como
byte-idêntico — existia desde a wave do sculpt, e o doc dele nomeia os três donos: *"o sculpt, o Reshape
e o Smear"*. **O depósito de pigmento não estava na lista** e forkava a tela em SÉRIE, uma vez por traço
(o `stroke_undo` que o `paint_begin` acabou de tirar é o segundo dono garantido, então o primeiro
`make_mut` do traço **sempre** copia o canvas inteiro).

Roteados os 5 sítios do `stamp_cache`:

| tela | modo | antes | **depois** | |
|---|---|---|---|---|
| 2048² | digital | 2,79–3,32 | **1,09–1,55** | −55% |
| 2048² | impasto | 10,47–13,22 | 9,49–12,79 | ~igual |
| 4096² | **digital** | 10,18–10,55 | **3,85–5,00** | **−62%** |
| 4096² | **impasto** | 18,37–18,96 | **11,96–12,11** | **−36%** |

⚠️ **E a medição corrigiu o escopo:** eu havia roteado **oito** sítios (5 no `stamp_cache`, 3 no
`impasto_live`) e medido os oito juntos. Isolando: os do `impasto_live` **não mudam nada no pen-down** —
eles são o caminho do **pen-UP** (`commit_stroke_height`). O ganho inteiro é do `stamp_cache`. Os três
ficam roteados porque são estritamente melhores no commit, mas **o número acima é de cinco sítios, não
de oito**.

**Gate:** `the_pigment_deposit_forks_the_canvas_in_parallel` — arquitetural, e tem de ser, porque **as
duas rotas dão os MESMOS BYTES**: trocar uma pela outra deixa a suíte inteira verde e custa 3× o tempo
do gesto que o artista mais sente. Controle positivo (a porta tem de existir) + 2 mutações, as duas
sangram.

### 4.4 O que SOBRA, decomposto

Pen-down a 4096², depois do fix:

| | ms |
|---|---|
| só PIGMENTO (o fork do canvas, 64 MB, paralelo) | **3,6** |
| só RELEVO (o `alloc_zeroed` dos 5 planos do traço, 235 MB) | **5,1** |
| os dois (o default) | **12,0** |

Superaditivo em ~3,4 ms: os dois disputam a mesma banda de memória.

⚠️ **A metade do RELEVO já tem cerca de Chesterton COM NÚMERO e não se mexe:** os 5 planos usam
`vec![0.0; n]` e **não** `clear() + resize`. A troca é tentadora (o `reset_stroke_height` preserva a
capacidade) e foi **MEDIDA E REPROVADA** em 2026-07-25: o pen-down a 4096² subiu de **17,6 para
47,5 ms**. `vec![0.0; n]` é `alloc_zeroed`, que pede páginas já zeradas ao SO e **não escreve um byte**;
reusar a capacidade obriga um `memset` explícito dos mesmos 235 MB. **Reusar memória é mais caro que
pedir memória nova quando a nova vem zerada de fábrica.**

### 4.5 ⚠️ E então a medição mudou o ALVO: metade do "delay" é UM DAB COMUM

A decomposição da §4.4 comparava o pen-down consigo mesmo. Faltava a comparação que decide tudo: **o
pen-down contra um MOVE** — um dab comum, o custo que todo movimento do pincel paga.

| 4096², r=100 | pen-down | move (1 dab) | **excesso do pen-down** |
|---|---|---|---|
| só pigmento | 3,8 | 0,39 | **3,4** (o fork do canvas) |
| só relevo | 5,1 | 3,30 | **1,8** (setup dos planos) |
| **default** | **12,1** | **5,50** | ~6 |

⚠️ **Metade do que o artista sente como "delay do pen-down" é simplesmente o custo de UM DAB** — o
mesmo que ele paga em todo move. O que é *específico* do pen-down são ~5 ms: o fork do canvas (3,4) e o
setup dos planos (1,8).

⚠️ **E a sonda do move nasceu MENTINDO:** ela andava menos que o `dab_spacing_px`, então **nenhum dab
nascia** e a coluna reportava `0,00 ms`. Um zero desses lê como *"o move é grátis"* quando o que ele diz
é *"o move não aconteceu"* — e era essa leitura que mantinha o pen-down parecendo um caso especial.

### 4.6 De onde vem o dab de RELEVO — e a sonda que eu tive de reescrever DUAS vezes

O número que salta na tabela da §4.5 é outro: **o dab de relevo custa 8× o de pigmento**. Decompor isso
custou três versões da sonda, e as duas primeiras mentiram de maneiras diferentes:

* **v1 — um `PainterTool` por configuração**, mediana de 8 moves. ⛔ **Era ruído.** Uma mutação de
  medição que **não podia** tocar a linha de controle (*"só o depósito"* tem o AA desligado, logo o
  caminho mutado nem é chamado ali) mesmo assim a viu saltar de **4,26 para 5,99 ms**. Canvas novo de
  64 MB por linha ⇒ páginas novas, alocador em outro estado: **±40% de deriva**, com um efeito medido de
  38% em cima.
* **v2 — pareada** (um tool, um traço, configurações alternadas move a move). Melhor, e ainda errada: a
  **mediana por-move deu `0,00`** porque nem todo move produz um dab (o `dab_spacing_px` decide), e a
  mediana de uma amostra majoritariamente vazia mede *quantos moves ficaram vazios*, não o custo do dab.
* **v3 — pareada e SOMADA por grupo**, com **controle de contagem de dabs** (`assert` de que os grupos
  carregaram o mesmo número; se não carregaram, eles não percorreram o mesmo trabalho e a comparação não
  vale). Reproduz: três corridas consecutivas deram AA = **2,57 · 2,59 · 2,60 ms**.

⚠️ **E a PRIMEIRA corrida da v3 também mentiu, por ser FRIA:** ela deu controle 5,38 e AA 4,18; as
seguintes, controle 3,5 e AA 2,6. Binário recém-compilado, cache frio. **Uma corrida só não é uma
medição** — e os números que este doc trazia antes vinham dela.

**Os números confiáveis** (r=100, 4096², pareado, somado, controle de dabs):

| configuração | ms/dab |
|---|---|
| tudo ligado (o default) | **6,35** |
| **sem o AA do filme** | **3,75** |
| sem o PUSH | 6,27 |
| sem nenhum dos dois | 3,53 |

⇒ **o AA do filme é 2,60 dos 6,35 ms — 41% — mesmo depois da LUT**, e é o maior item que sobrou.
O **PUSH custa 0,07–0,26 ms** (ruído); o SETTLE saiu da tabela porque roda no **commit**, não por dab,
então uma sonda de MOVE não pode vê-lo — a v1 media ruído nas duas colunas dele.

### 4.6.1 ✅ E o custo do AA não era a grade — era a TABELA lida onde ela não é necessária

**2,60 ms sobre 7 758 texels de banda = 335 ns/texel.** Nove leituras numa tabela quente não custam
isso, então o modelo *"o AA é as nove amostras"* **não fechava**. Duas explicações minhas morreram
medidas antes da certa:

1. ⛔ **"a closure da silhueta é construída por texel mesmo sem ser chamada"** — trocada por uma trivial
   (imagem errada de propósito): AA **2,53** contra 2,57 da base. **Sem diferença.**
2. ⛔ **"o corpo da closure infla o laço quente"** — `film_at_exact` com `#[inline(never)] + #[cold]`
   (imagem INTACTA): AA **2,83**, ou seja **PIOR**. Outlinar cobra do caminho inadmissível e não devolve.

**O que respondeu foi a ablação**: desligar a expansão inteira (um lookup por texel, imagem errada) levou
o AA de **2,60 para 1,02 ms** ⇒ a grade custa 1,58 e **sobra 1,02 ms só por ter o AA ligado com UM
lookup**. E o erro do meu modelo estava aí: eu atribuía o custo da **bbox inteira** aos texels da
**banda**. A banda é ~25% da área do disco (tabela acima), então **três quartos dos texels de todo dab**
caíam no early-out da LUT — que serve `lut.at(t)`, uma leitura de 64 KB, por texel.

⚠️ **E ela nunca foi necessária lá: o chamador JÁ TEM a resposta exata.** O kernel de altura computa a
silhueta `w` para o próprio envelope, e `film_of(w)` é o valor EXATO onde `lut.at(t)` é o interpolado.
A porta (`film_at_planned`) passou a testar a banda PRIMEIRO e a devolver o single-sample do chamador
fora dela — **byte-idêntica ao produto pré-LUT** naqueles três quartos, e mais barata:

| | AA (ms/dab) | dab (ms) |
|---|---|---|
| antes | 2,60 (41%) | 6,35 |
| **depois** | **1,43 (28%)** | **5,15** |

E o traço a 2048²: **110,22 → 96,92 ms** (virgem) · **142,96 → 130,17** (sobre tinta). Somado à LUT, a
jornada inteira leva o traço de **134,84 → 96,92 (−28,1%)** e de **167,79 → 130,17 (−22,4%)**.

**Gate `outside_the_band_the_caller_answers_not_the_table`** — comportamental, e tem de ser: a tabela
interpola a MESMA curva, então servi-la fora da banda erra ~1e-5, que **nenhum gate de bytes vê**. O
oráculo é uma tabela **deliberadamente errada** (construída de outro falloff): fora da banda o resultado
tem de ignorá-la por completo, e **dentro** dela tem de vazar — senão o fixture não contém o fenômeno.
2 mutações, as duas sangram.

⚠️ **E agora o modelo de custo FECHA**, o que reabre a avenida seguinte com base sólida: os 1,43 ms
restantes sobre 7 758 texels são **184 ns/texel**, consistente com 9 leituras que caem em L2 (~20 ns
cada). A convolução tabulada de verdade — **um** lookup em vez de nove — vale portanto ~1,27 ms, e agora
é uma estimativa derivada de um modelo que bate com a medição, não de um palpite.

### 4.6.2 ⛔ E o passo seguinte — colapsar a grade em três leituras — foi MEDIDO e REJEITADO

Com o modelo fechado, a cura óbvia dos 1,43 ms era reduzir a **contagem de leituras**. Uma convolução é
caracterizada pelos seus **momentos**, e os da grade 3×3 saem em forma fechada (os ímpares são zero por
simetria):

```text
  m2 = (2/3)(u² + v²)          m4 = (2/3)(u⁴ + 4u²v² + v⁴)
```

Uma média de **três** pontos `{−σ, 0, +σ}` casa `m2` com `σ = h·|a|`; com **pesos** ela tem dois graus de
liberdade e casa `m2` **e** `m4`. Construído nas duas formas e varrido nas MESMAS 55 combinações que
aprovaram a grade:

| kernel | leituras/texel | pior erro sobre o admissível |
|---|---|---|
| a grade de 9 (o que shipou) | 9 | **0,060 nível de u8** |
| colapso de 3, casando o 2º momento | 3 | **4,949** (`Sphere` r40, elipse — 192 texels) |
| colapso de 3 PESADO, casando 2º **e** 4º | 3 | **5,344** — *pior ainda* |

⚠️ **É a terceira linha que fecha a questão, não a segunda.** Se o erro fosse dos momentos, casar mais um
momento melhoraria tudo monotonicamente — e ele **piorou o pior caso**, embora melhorasse as linhas
fáceis (`Sphere` r100 disco 0,374 → 0,230). Uma quadratura por momentos pressupõe que a função é
**analítica sobre o suporte**, e `F = film_of ∘ falloff_weight` **tem QUINAS**: os `clamp` do
`body_profile` e do `film_opacity`. Sobre uma quina, mais momentos não ajudam.

⚠️ **E o pior caso é o caso COMUM:** `Sphere` é o falloff **default do impasto** e a elipse é o Flatten &
Rotate — não são cantos exóticos do espaço de parâmetros.

⚠️ **Restringir a admissibilidade salvaria os números** (só redondo, só raio ≥ 100) — e seria uma segunda
regra com **dois limiares tirados de uma tabela**, exactamente o *"limite que só diz por segurança"* que
o §0 proíbe e que já matou a estimativa de cinco amostras (§3.3). Recusado pelo mesmo motivo.

**É a lição da §3.3 outra vez, numa forma nova: o custo do AA é a contagem de amostras, e a qualidade
dele também é.** O kernel rejeitado foi removido; o número vive no doc-comment do `film_at_lut`, que é
onde a próxima pessoa com esta ideia vai ler.

### 4.8 🎯 E o PRIMEIRO traço era outra coisa: 28 ms de COMPILAÇÃO DE SHADER

> *"ainda não resolvemos o primeiro traço"* (Enio, 2026-07-26, depois de todas as medições acima)

⚠️ **A §4.5 mediu que o custo é POR GESTO e concluiu que "o delay do primeiro traço" era o delay de todo
pen-down. Isso estava certo sobre o que a sonda via, e ERRADO sobre o que o artista sentia** — porque
toda sonda desta jornada mede o `PainterTool`, e **o `PainterTool` não tem GPU**. O que só o primeiro
traço paga vive na SHELL.

O `PainterGpuPreview` era construído **lazily** (`get_or_insert_with`, dentro do `drive`), ou seja no
primeiro frame que precisasse do preview GPU — o primeiro traço. E as três peças dele **compilam
shaders**. Medido na RTX com o driver já quente
(`ph2d-render/tests/measure_first_stroke_pipelines.rs`):

| peça | ms |
|---|---|
| `LayerCompositor::new` | 6,01 |
| **`ImpastoLightPass::new`** | **16,30** |
| `PreviewPremul::new` | 5,70 |
| **total, pago no 1º traço** | **28,01** |

Quase **dois quadros de 60 fps**, uma vez, exactamente no gesto em que o artista está esperando a tinta
aparecer — e somados aos ~12 ms do pen-down dão ~40 ms antes do primeiro dab.

⚠️ **A sonda que mede o SEGUNDO pen-down não pode ver isto**, e foi ela que me fez chamar o problema de
*"o delay de todo pen-down"* enquanto o Enio o chamava, com precisão, de *"o delay do PRIMEIRO traço"*.
**Quando o relato do artista e a medição discordam sobre QUAL gesto dói, a medição está olhando para o
lugar errado.**

⚠️ **E o número que a sonda descartou é o maior de todos:** a primeira chamada a qualquer pipeline neste
processo custou **1 177,84 ms** (o compilador de shader do driver acordando). No app isso é pago no
boot, pelo renderer principal — mas é o aviso de que compilação de pipeline **nunca** é barata, e de que
a única pergunta que importa é *em que gesto ela cai*.

**A cura é QUANDO, não o quê:** `prewarm` no **bind do documento** — o artista escolhe o sprite, depois
leva o mouse até a tela e clica, e há tempo HUMANO nesse vão. No boot cobraria os 28 ms de quem nunca
pinta; por frame seria pior que o lazy.

⚠️ **O `get_or_insert_with` do `drive` FICA**, e não é redundância: o pré-aquecimento é uma otimização de
*quando*, e o produto não pode depender dela para funcionar — uma rota que chegue ao `drive` sem passar
pelo bind tem de continuar produzindo preview, só que pagando os 28 ms ali. Há gate para as duas metades.

**Gates** (`the_first_stroke_does_not_compile_shaders.rs`) — arquiteturais, e **têm de ser**: a sessão
existe nos dois casos e produz os mesmos pixels; o que muda é *quando* ela é construída, e nenhum gate de
comportamento vê isso. A afirmação é **posicional** (o pré-aquecimento vem depois do bind **e dentro da
mesma cadeia de guards**, contando chaves — senão ele rodaria noutra condição, por exemplo em todo
frame, que é pior que o lazy). Controle positivo nos dois alvos; **3 mutações, 3 sangram**.

### 4.8.1 ⚠️ E o SMOKE refutou a §4.8 pela metade: pipeline não escala com a tela

> *"primeiro traço ainda com atraso. Quanto menor o IMG menor o atraso. 1024 nem se percebe. Restante da
> pintura com boa performance"* (Enio, 2026-07-26)

**Uma escada com o tamanho do canvas não pode ser compilação de shader** — um pipeline é compilado uma
vez e é **independente da tela**; os 28 ms seriam os mesmos a 1024 e a 4096. A §4.8 achou um custo real
e o moveu, mas ele não era **o** custo.

⚠️ **E a medição da §4.8 também não era estável:** re-rodada com os testes em outra ordem, o total dos
pipelines saiu **10,43 ms** em vez de 28,01, e o "aquecimento do driver" que eu descartava saiu **8,51 ms**
em vez de 1 177,84. **Quem roda primeiro paga a inicialização preguiçosa do driver** — o número por-peça
não é atribuível, só o total é, e mesmo ele varia com a ordem.

**O que escala com a tela são os RECURSOS.** As texturas dos três passes nascem no tamanho do canvas, e a
**primeira execução** as aloca e as semeia (upload dos planos por PCIe). Medido — o que só a 1ª execução
paga:

| tela | 1ª execução | 2ª | **diferença** |
|---|---|---|---|
| 1024² | 1,87 | 1,11 | **0,76 ms** |
| 2048² | 7,99 | 5,26 | **2,72** |
| 4096² | 30,42 | 17,20 | **13,21** |

É exactamente a escada relatada, incluindo o *"1024 nem se percebe"*.

⚠️ **E o GATILHO fecha o relato:** `gpu_eligible` recusa uma pilha **trivial sem relevo**, que é o que um
documento recém-bindado é ⇒ **nada é alocado**. O **primeiro traço com relevo** a torna não-trivial, o
caminho GPU engata, e tudo nasce naquele instante. Por isso é o *primeiro* traço, por isso escala com a
tela, e por isso *"o restante da pintura"* vai bem.

**A cura:** o `prewarm` passou a **cozinhar um frame** no bind, não só construir pipelines — e ele
**passa por cima da `gpu_eligible` de propósito**, porque consultá-la ali é exatamente o que adiava o
custo. O slot do renderer é liberado no frame seguinte (a pilha ainda é trivial) e isso está certo: o
que precisava sobreviver são as texturas **internas** dos passes, que moram no `session_slot` e que o
`release_slot` não toca.

⚠️ **E um buraco no meu próprio gate, achado por mutação:** a asserção *"o corpo contém `drive(`"* é
satisfeita por **código morto** — um `return` antes dele deixa o texto no lugar e a chamada
inalcançável, e essa mutação **SOBREVIVEU**. *"Contém a chamada"* não é *"a chamada roda"*. O gate agora
também conta as SAÍDAS: o pré-aquecimento pode desistir por **dois** guards documentados (a `flatten`
recusou · canvas 0×0) e por mais nenhum. **3 mutações, 3 sangram.**

### 4.8.2 🎯 E o SEGUNDO smoke nomeou o custo de verdade: **o FOLD do relevo, 201,5 ms**

> *"muito delay ainda"* (Enio, 2026-07-26)

Parei de deduzir. Duas hipóteses minhas já tinham sido refutadas pelos smokes dele (a §4.8 e depois a
§4.8.1), e as duas erraram pela mesma razão: **eu media peça isolada em vez do produto**. O
`PH2D_PAINT_PERF` já existia e já reportava `max=`, mas o *split* de fases era todo **p50** — e um custo
de UMA VEZ é invisível numa mediana **por construção**: ele é exatamente o outlier que a mediana existe
para descartar. Acrescentei a linha do frame **PIOR** e pedi o número.

```
90f GPU 35/CPU 55 | dispatch p50=0.0 max=232.7 | WORST: GPU 4096x4096 impasto=true
WORST split: dispatch=232.7 [preview 232.7 panel 0.0 overlay 0.0 upload 0.0]
EVENTO->FRAME p50=16.8 p95=24.5 max=247.1 ms
```

**232,7 ms, e 100% dentro de `preview`.** E o `prewarm` da §4.8.1 **rodou** — a 1ª janela do log traz o
frame CPU `trivial-fast` do bind. O custo não era nada do que eu tinha atacado.

⚠️ **A causa estava ESCRITA no próprio produto**, num doc-comment que eu li e não conectei
(`compose_light_premul`): *"Measured, at 4096²: **202 ms a frame whole** against 2,8 ms for a 512²
window, and the walk is the entire cost."* O passe de luz **recusa upload parcial** até ter segurado a
pintura inteira uma vez (`ImpastoLightPass::planes_seeded`), então o **primeiro frame com relevo dobra o
CANVAS**, por mais curto que tenha sido o traço. Medido de novo hoje pela porta do produto: **201,53 ms**.

**A cura é a que esta linha já usa em quatro lugares:** o walk é **puro por-texel** e as linhas são
**disjuntas** (ADR-0109 — o mesmo desenho de `sculpt_offset`, `sculpt_close` e `watercolor_field`), então
ele passa a `par_chunks_mut` por linha. **Byte-idêntico por construção:** muda qual thread avalia qual
linha, nunca o que a linha diz.

| canvas | serial | **paralelo** | |
|---|---|---|---|
| 2048² | 45,29 ms | **4,53** | 10,0× |
| 4096² | **201,53 ms** | **14,55** | **13,8×** |
| janela 512² | 2,85 | **0,39** | o estado permanente de todo traço |

**Os gates que já existiam SÃO o oráculo:** um compara a porta contra um laço **serial** dos mesmos
samplers, texel a texel; o outro usa janela de origem `(10, 16)`, que é a aritmética `y = ry + row` que
a reescrita introduziu. A mutação do mapeamento `row → y` sangra.

⚠️ **Mas nenhum deles gateia os 13,8×, e isso era um buraco meu** (achado ao conferir a pergunta *"a
cura está bem documentada?"*, e fechado): o gate de **razão** não vê a regressão — um fold **serial**
também é limitado pela janela, então a razão dele continua ~1 e ele fica verde enquanto o primeiro
frame com relevo volta de 14,55 para 201,5 ms. Os gates de **forma** também não: serial e paralelo dão
os **mesmos bytes**, que é precisamente a propriedade que torna a cura segura. E `par_chunks_mut →
chunks_mut` num refactor é **uma letra**.

Daí `the_fold_walks_in_parallel_because_the_rows_are_disjoint`, **arquitetural de propósito**: *"este
laço roda em paralelo"* é afirmação sobre a FORMA do código, não sobre um resultado observável — um
gate de comportamento mediria wall-clock, e o `ci-test` compila em `opt-level=1`, então uma barra de
milissegundos mediria o **perfil do build** e não o produto. O número vive na sonda; o gate guarda que o
mecanismo que o produziu continua lá. **Mutação medida: só ELE fica vermelho, os outros cinco passam** —
que é exatamente o que o torna não-redundante.

⚠️ **E ela expôs o ponto cego do gate de janela:** *door-contra-door não vê erro que desloca os DOIS
lados igual*. Com o off-by-one instalado, `a_window_folds_exactly_what_the_whole_canvas_folded_there`
fica **VERDE** — a janela e o canvas se movem juntos e continuam concordando — e só o gate contra os
**samplers** morde. Os dois não são redundantes: um pina *a janela é igual ao canvas*, o outro *o canvas
é igual ao fold*, e um erro de indexação uniforme satisfaz o primeiro.

⚠️ **Um gate meu quebrou por ficar rápido demais, e isso é uma lição de forma.** O
`the_fold_costs_what_the_window_costs_not_what_the_canvas_costs` media uma janela de **128²**, que caiu
de ~0,18 para **0,044 ms** — e uma razão entre dois números desse tamanho mede o **escalonador do
rayon**, não a propriedade. Ele falhou sob a suíte inteira (0,0839 contra 0,2470, "2,95×") enquanto a
varredura mostra a janela **plana** no tamanho do canvas a menos de 10%:

| canvas | full | 1024² | 512² | 256² | 128² | 64² |
|---|---|---|---|---|---|---|
| 2048 | 4,53 | 1,19 | 0,41 | 0,159 | 0,042 | 0,024 |
| 4096 | 14,89 | 1,23 | 0,39 | 0,153 | 0,046 | 0,021 |

*Um gate cujo oráculo se dissolve quando a coisa que ele vigia fica mais rápida é um gate que será
silenciado em vez de acreditado.* Agora usa janela **512²** (~0,39 ms, dez vezes o piso de ruído) sobre
canvases 1024/2048, e lê o **MÍNIMO** das amostras — máquina carregada só sabe deixar mais lento.
Re-provado: a mutação *"dobra o CANVAS, devolve a JANELA"* — a falha que **só um relógio pode ver** —
sangra em **3,70×**, e os 4 gates de FORMA ficam verdes sob ela.

⚠️ **E a sonda varrida nasceu com a fixture MENTINDO:** a janela era centrada no **canvas** e o traço
vive em `x ∈ [200, 700]`, então a 2048² ela cobria tinta e a 4096² cobria **papel nu** — as duas linhas
precificavam fenômenos diferentes, e a tela GRANDE saía *mais barata* (18,6 contra 51,0). Fixture, não
fold. Centrada no TRAÇO, a tabela acima é estável entre execuções.

⚠️ **O que fica ABERTO, nomeado com o número e com o preço:** o `prewarm` **não semeia** os planos da
luz (uma pilha recém-bindada não tem relevo ⇒ `impasto_gpu_planes_in` recusa ⇒ a luz não roda ⇒ nem as
texturas nem o `planes_seeded` acontecem), então o primeiro traço ainda dobra o canvas inteiro — **14,55
ms em vez de 0,38** — mais a alocação das 5 planes (~218 MB a 4096², 13,21 ms medidos na §4.8.1).
Fechá-lo é **decisão de produto, não correção mecânica**: qualquer pré-aquecimento da luz cobra VRAM
canvas-sized de **todo bind**, inclusive de quem nunca liga o impasto — o mesmo argumento que já mantém
este pré-aquecimento fora do boot, num tamanho dez vezes maior. **O número que decide é o próximo
`PH2D_PAINT_PERF`.** O doc-comment do `prewarm` foi corrigido: ele afirmava alocar as texturas dos
**três** passes, e as da luz não estavam entre elas.

### 4.8.3 ✅ O SMOKE confirmou — e **moveu a fronteira de lugar**

> *"A impressão que tive é que ficou muito bom!"* (Enio, 2026-07-26)

Quatro janelas de 90 frames, canvas **4096²**, Impasto, `GPU 90/CPU 0` em todas — o produto ficou
inteiramente na pista GPU.

| janela | `frame p50` | `dispatch p50` | `dispatch max` | split do PIOR | eventos/frame |
|---|---|---|---|---|---|
| A | — | — | **1,2** | `preview 1,2` | 2,0 |
| B | 10,4 | **0,7** | **1,1** | `preview 1,1` | 1,9 |
| C | 6,6 | **0,8** | **1,3** | `preview 1,2` | 1,9 |
| D | 16,7 | 0,0 | **12,7** | `preview 12,7` | **0,3** |

**O frame pior saiu de 232,7 para 1,1–1,3 ms** nas três janelas em que se pintava de fato — e o
`dispatch p50` é **0,7–0,8 ms**, ou seja **4% de um quadro de 60 fps**. É o número que o *"muito bom"*
descreve.

⚠️ **A janela D não é uma regressão, é a frente P se mostrando** — 12,7 ms, **100% em `preview`**, numa
janela quase parada (0,3 evento por frame, 15 amostras de latência). Essa é a assinatura de um **re-fold
de canvas inteiro**, e o número bate com os 14,55 ms medidos headless. ⚠️ **O log não diz se aquele
frame foi o primeiro traço** — e não vou afirmar que foi; o que ele diz, e basta, é que **o fold de
canvas inteiro agora custa 12,7 ms no produto e portanto CABE dentro de um quadro** (16,7 ms), onde
antes custava catorze quadros. Isso re-precifica a frente P: o preço dela deixou de ser os **~31 ms de
resíduo aritmético** que eu havia estimado e passou a ser **12,7 ms medidos**, contra ~218 MB de VRAM em
todo bind. *A estimativa era minha; o número é do produto, e é o número que vale.*

#### O que a medição REVELA agora que o dispatch saiu da frente

Com o dispatch em 0,7 ms, duas grandezas que estavam escondidas atrás dele viraram as maiores da tabela.

**(1) A latência é EXATAMENTE um período de frame — logo ela não é mais compute.**

| janela | `EVENTO→FRAME` p50 | p95 | `periodo real` |
|---|---|---|---|
| A | 16,9 | 17,7 | 16,5 |
| B | 16,9 | 19,3 | 16,5 |
| C | 16,8 | 22,2 | 16,5 |

⚠️ **`p50 ≈ periodo real` é o piso desta arquitetura, não um defeito a caçar:** o evento chega, espera o
próximo frame, e o instrumento o fecha no fim dele. O dispatch é **4%** disso. O alvo público de **9 ms**
que o instrumento carrega ao lado (`alvo 9`) portanto **não sai de otimizar cálculo** — o próprio
doc-comment do L0 já dizia de onde ele saiu no Apple Pencil: *"não foi compute — foi pipeline"*. Reduzir
17 para 9 é mexer em **cadência** (frame rate, quando o evento é servido, quando o present acontece),
que é uma frente de outra natureza e de outro dono.

⚠️ **E o instrumento mede *evento → fim do frame*, nunca *→ pixel*** — o present acontece a uma fração de
ms depois. O doc dele é explícito sobre isso, e citar este número como "latência até o pixel" seria
vender o que ele não mede.

**(2) O custo do PAINTER migrou do frame para o INPUT.**

`INPUT (fora do frame)` — o tempo dentro de `on_canvas_pointer`, isto é, carimbar dabs — mede
**p50 5,3 · 5,4 · 8,8 ms** nas três janelas de pintura, contra **0,7–0,8 ms** de dispatch. Com ~2
eventos por frame, são ~2,7–4,4 ms **por evento**.

🎯 **Isto reordena o doc inteiro:** o trabalho das §4.5/§4.6 (a decomposição do dab, o AA a 1,43 ms/dab,
a LUT do filme) sempre foi sobre *este* número, e ele estava atrás do dispatch enquanto o dispatch valia
232 ms. Agora ele é **7–12× o dispatch** e é o maior custo do Painter no caminho quente. A fila da §7 é
reescrita em cima disso.

**(3) E um outlier que eu NÃO vou enterrar: `INPUT max = 134,8 ms`.**

Na janela D, um único evento custou **134,8 ms** dentro do `on_canvas_pointer` (e a latência daquele
frame foi 148,8 ms). ⚠️ **O log não o atribui, e eu não vou inventar a causa.** O que se sabe: é tempo do
**tool**, não do frame; caiu na janela quase parada, junto do re-fold de canvas inteiro; e os candidatos
que a §4.7 e o [doc 25 §13.12.5](25_avaliacao_gpu.md) já nomeiam com números da mesma ordem a 4096² são
o **pen-down** (o clone canvas-sized: 24,5 ms protegido · 11,7 livre) e o **commit de pen-up** do
impasto. Nenhum deles chega a 134,8 sozinho pelo que está medido, então **falta medição, não hipótese** —
e a forma de obtê-la é a mesma que funcionou aqui: instrumentar por fase o que hoje é um número só, em
vez de deduzir. **É o maior número que sobrou no log.**

### 4.7 A metade que FALTA — e por que ela é uma WAVE e não um fix

A U1 (undo por delta) resolveu o que sobra **DEPOIS** do traço; isto é o que a cópia custa **DURANTE**
ele, e **basta UMA segunda referência ao canvas para o 1º dab pagar um `make_mut`**.

A cura é a metade viva da receita do §13.12.5: **captura do "antes" por REGIÃO, sob demanda** — o
tile-based undo do GIMP/Krita. Só que ela exige uma **porta ÚNICA de escrita de canvas** contra os ~25
sítios que hoje chamam `Arc::make_mut` direto, e é isso que a torna uma wave com desenho próprio em vez
de um patch.

⚠️ **E é a frente com o melhor retorno que sobrou, por um motivo que não é o tamanho do número:** ela
mora **acima do modelo de pintura** — no documento e no undo. **Ela cura Digital, Impasto, Watercolor e
Wet Paint de uma vez.**

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

#### Gates, e as duas mutações que ensinaram algo

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

## 6. As lições de método (as que custaram tempo)

1. **Uma razão contra uma constante não é uma razão contra uma tabela.** O `10,3×` que projetou 31%
   media a cadeia contra uma closure que devolve constante. A substituição real custa alguma coisa, e
   entregou 18%.
2. **Um limite escolhido por palpite é um gate que não pode falhar.** A barra do gate de bytes nasceu em
   64 *("o aro tem ~1300 texels")* e uma mutação real sobreviveu a **duas** rodadas por causa dela. O
   produto correto diverge em **0..5** bytes; sem o `P` da banda, em **12..39**. A barra é **8**, e os
   dois números vivem no gate.
3. **Contador é a única defesa contra caminho rápido morto** (a lição do ADR-0120). O gate conta **58.322
   disparos** no laço real e exige **727 straddles** — sem isso, uma LUT que nunca é chamada deixa a
   suíte inteira verde e o produto exatamente como estava.
4. **Contadores globais + testes em paralelo = medição envenenada.** A 1ª rodada de mutações mediu
   poluição cruzada (uma mutação numa base "matou" o gate que conta hits, que ela não toca). A trava é
   segurada por **todo gate que RODA** o caminho rápido, não só pelos que o LEEM.
5. **Não existe alavanca que desligue uma otimização preservando o desenho quando a admissibilidade É
   uma afirmação sobre a imagem.** A única A/B honesta é dirigir a função do produto duas vezes,
   diferindo só no campo que se quer testar.
6. **A cwd do Bash volta para a árvore primária.** O mesmo path relativo existe nas duas árvores, e
   editar a errada compila e commita **sem erro**. Todo comando leva o `cd` da worktree.
7. 🎯 **Meça o PRODUTO, não a peça — e uma mediana não pode ver um custo de UMA VEZ.** Duas hipóteses
   minhas para *"o delay do primeiro traço"* (§4.8 e §4.8.1) foram refutadas pelos smokes do Enio, e as
   duas erraram do mesmo jeito: eu cronometrava um componente isolado num harness meu. O `PH2D_PAINT_PERF`
   já existia, já reportava `max=`, e o **split de fases era todo p50** — ou seja, cego por construção ao
   outlier que é exatamente o fenômeno. Uma linha de diagnóstico (o split do frame **pior**) nomeou a
   causa em **um** smoke, depois de duas rodadas perdidas deduzindo.
8. **Uma sonda que re-implementa o laço fica CEGA à porta.** A `measure_what_the_fold_is_made_of`
   dissecava o custo com um laço próprio — certo para *"de que isto é feito"*, e ela seguiria imprimindo
   o custo **serial** depois de o produto parar de pagá-lo. Toda cura precisa de uma medição que passe
   pela **porta do produto** (`measure_the_fold_the_product_runs`).
9. **Um gate pode quebrar por a coisa que ele vigia ficar RÁPIDA.** A razão do fold media uma janela que
   caiu para 0,044 ms e passou a medir o escalonador do rayon. *Um gate cujo oráculo se dissolve quando o
   produto melhora será silenciado em vez de acreditado* — a cura é escolher a grandeza dez vezes acima
   do piso de ruído e ler o **mínimo**, não afrouxar a barra.
10. **Pergunte a FORMA antes do relógio.** No Wet Paint (§5.12) a pergunta *"quantos texels este move
    marca?"* eliminou o suspeito óbvio numa linha — a região suja é **constante nas três telas**, logo o
    composite não era o plano. Um cronômetro diz *quanto*; uma área diz *o quê*, e um número estrutural
    não flaka.
11. **O REDUTOR é parte da fixture.** O mínimo é o redutor certo quando toda amostra faz o mesmo
    trabalho (§4.8.2) e o **errado** quando uma delas é estruturalmente diferente: o primeiro move de um
    traço não compõe e mede 0,22 ms nas DUAS telas, então o mínimo lia **exactamente a amostra sem o
    fenômeno** e o gate dava 1,00× sobre o defeito reinstalado (§5.12). Antes de confiar num min/max,
    pergunte se alguma amostra é de outra natureza.
13. **Um número que não reproduz não é um achado — é ruído com casas decimais.** A sonda do pen-up
    mediu **117,76 ms** numa corrida e **28,46** na seguinte, na mesma célula, com o produto correto
    (§5.13). Pior: eu já tinha começado a construir um mecanismo em cima do 117,8 (*"é o first-touch do
    1º traço"*) — e a mesma sonda, repetida, o **refutou** (o 1º traço é o mais BARATO). *Repita antes
    de explicar*: uma explicação boa para um número errado é mais cara que nenhuma explicação.
12. **Uma pergunta de IDENTIDADE não se paga com POSSE.** O guard do Wet Paint queria saber *"este ainda
    é o meu canvas?"* e segurava um `Arc` forte para responder — o que fazia `Arc::make_mut` copiar o
    documento a cada movimento do mouse (**9,86 ms a 4096²**). Um `Weak` responde a mesma pergunta por
    **zero**, e ainda **prende a alocação**, que é o que torna a comparação de endereço sã (o ABA do
    ADR-0124). *Se você guarda algo só para comparar, guarde a coisa mais fraca que ainda compare.*
14. **Uma atribuição que casa com o total por ACIDENTE é pior que nenhuma.** A §5.13 somou `Vec::clone`
    (memcpy **serial**) para explicar os 32,8 ms do pen-up e obteve 28,47 — *"fecha"*. O produto usa
    `fork_par`, que é **paralelo** e custa **9,25**: os 20 ms restantes estavam noutro lugar, e a soma
    coincidente **encerrou a investigação**. *Meça pela porta que o produto usa, mesmo quando a
    aritmética já bateu* — e desconfie especialmente quando ela bate na primeira tentativa.
15. **Um custo "user-paced" não é um custo desprezível.** O doc do `diff_window` justificava a varredura
    canvas-inteira como *"uma vez por commit (user-paced)"*. Um commit acontece no **pen-up de todo
    traço**: eram **91% do pen-up**. *Escreva com que FREQUÊNCIA o gesto acontece, não em que categoria
    ele cai.*
16. **Uma mutação que não sangra pode ser inválida em vez de reveladora.** Varrer as colunas a partir da
    linha 0 em vez da primeira linha diferente **passou em tudo** — e está certo: linhas iguais devolvem
    a identidade e não movem `min`/`max`. É desperdício, nunca erro. *Antes de escrever um gate para uma
    mutação sobrevivente, confira se ela muda alguma resposta* (a irmã que de fato erra — excluir a
    ÚLTIMA linha — sangra na hora). E a outra sobrevivente da mesma rodada era o oposto: um buraco real,
    escondido por **toda banda larga das fixtures acertar a coluna 0 por acidente**.

17. **Uma sonda SINTÉTICA mede buffers quentes; o produto mede buffers frios — e a diferença foi 3×.** O
    `what_the_commit_scan_is_made_of` dizia que os quatro planos custam 4,02 ms de varredura; instrumentado
    **dentro do `split` do produto**, o mesmo trabalho custa 10–20. O custo não era outro código: era a
    mesma leitura sobre memória que ninguém tocava há um traço. *Uma sonda que constrói os próprios dados
    logo antes de medi-los mede o cache, não o produto* — e foi essa diferença que quase me fez
    paralelizar a extração da janela, que rende **zero**.
18. **Um limiar de paralelização é em BYTES, não em elementos — e o erro só aparece no canvas PEQUENO.** O
    mesmo `PAR_MIN` que estava certo para o fork de um plano de `f32` a 4096² **dobrou** o custo de um
    Ctrl+Z a 1024², porque em elementos ele não distingue `u8` de `[u8; 7]`. Otimização medida numa ponta
    da faixa é regressão silenciosa na outra: **meça as três telas, sempre**.

---

## 7. Próxima etapa recomendada

⚠️ **A medição REORDENOU a fila DUAS vezes, e as recomendações anteriores deste doc estão superadas.**
Primeiro os planos canvas-sized do traço saíram do topo (§4.5: valem 1,8 ms, não 5,1). Depois o
`PH2D_PAINT_PERF` mostrou que **nada disso era o "delay do primeiro traço"**: era o fold do relevo
dobrando o canvas inteiro, 201,5 ms, e ele está curado (§4.8.2).

**E o smoke da §4.8.3 a reordenou uma TERCEIRA vez** — desta vez não por derrubar uma hipótese, mas por
**tirar o vencedor da mesa**: com o dispatch em 0,7 ms, o que estava atrás dele apareceu.

**A fila de hoje, por número MEDIDO no produto:**

1. 🎯 **A JANELA VEM DE QUEM ESCREVE — a wave que desmonta o pen-up, agora com o preço CERTO.**
   O pen-up a 4096² com impasto custa **37,00 ms** e ele tem duas metades **medidas pelas portas do
   produto** (§5.16): o **fold** 13,28 e o **commit** 23,72. E as duas morrem pela mesma mudança:

   - o **commit** é `PlaneDeltas::split`, limitado por **largura de banda lendo ~470 MB** para
     *derivar* uma janela que o escritor já conhecia. ⚠️ **Paralelizar a extração foi MEDIDO e
     REJEITADO** (§5.16): o custo é o SCAN, e ele já é paralelo.
   - o **fold** é uma cópia de 192 MB que só existe porque o `stroke_undo` é um **segundo dono** dos
     três planos de relevo.

   A cura é a mesma que o doc 25 §13.12.5 prescreve (*tile-based undo* do GIMP/Krita): **o "antes" é
   capturado por REGIÃO, no momento da escrita**, em vez de ser um `Arc` do plano inteiro. E há uma
   ordem que a mede em três degraus, cada um verde por si:

   - **S1 — a porta recebe a REGIÃO.** `plane_fork::fork_par` já é o único jeito de um plano
     canvas-shaped virar escrevível (arch-gate sobre `tool/paint/**`, 41 sítios). Dando-lhe o retângulo,
     *esquecer é impossível por TIPO* — que é a resposta exata à objeção que o `diff_window` documenta
     (*"uma janela informada errado não falha: some com texels em silêncio"*). Sítio que não sabe passa
     "plano inteiro" ⇒ varredura completa ⇒ correto. **Sem isto, S2 e S3 não são seguros.**
   - **S2 — o commit usa a janela** em vez de a derivar. Mata os 23,7 ms do commit.
   - **S3 — o journal guarda os PIXELS da região** e o Ctrl+Z passa a **aplicar o patch ao plano vivo**
     em vez de instalar um snapshot materializado. Aí o `cursor` não precisa mais segurar os planos, a
     contagem de donos cai para **um**, e **o fold e o fork do pen-down somem juntos** — mais o Ctrl+Z,
     que deixa de custar uma cópia de documento (§5.16) e passa a custar a janela.

   ⚠️ **A contagem de donos é a espinha e já está medida:** em repouso os quatro planos têm **dois**
   donos, e o segundo é o `cursor` da U1 — um journal que só substituísse o `paint.stroke_undo`
   deixaria a contagem em 2 e **não mudaria um milissegundo**. ⚠️ **Wave própria, com gates próprios**, e
   é a única frente que cura os QUATRO modos de uma vez.

2. 🔴 **O outlier de 134,8 ms num único evento (frente R) segue aberto** (§5.13/§5.14). O maior evento
   **reprodutível** é o pen-up a 38,9 ms e ele não chega lá. ⚠️ **Mas uma amostra única de pen-up já foi
   medida em 117,76 ms com o produto correto** — então o 134,8 é compatível com um pen-up normal pego
   num instante ruim. O que decide é um **segundo log com histograma**, não mais teoria.
3. 🟡 **Semear os planos da luz no bind (frente P): 12,7 ms contra ~218 MB de VRAM em todo bind.**
   Decisão de produto, e agora com o preço dos dois lados medido em vez de estimado.
4. ⚪ **`EVENTO→FRAME` 16,8 contra o alvo 9 (frente S) — NÃO é compute.** `p50 ≈ periodo real`, o
   dispatch é 4% dela: é frente de **cadência/pipeline**, de outra natureza e outro dono.
5. ✅ **FECHADA — o move do Wet Paint (frente V), 13,71 → 1,82 ms a 4096², e PLANO** (§5.12). Não era
   varredura: era `Arc::make_mut` copiando o documento porque o token do guard segurava um `Arc` forte.
   **Nenhum desvio de FORMA sobrou na tabela dos quatro meios.**
6. ⛔ **O warp da aquarela (frente T) NÃO tem caminho de CPU óbvio, e isso está medido.** O custo é o
   número de avaliações (10/texel), cortar taps foi **rejeitado por LOOK** numa wave anterior, e a
   fatoração exata rendeu 1,20× na função sem sair do ruído no produto. O que sobra é **aproximar** o
   warp dentro do texel — exatamente a classe que esta jornada já mediu e rejeitou **duas** vezes no AA
   do impasto (§3.3, §4.6.2), onde o erro vinha das QUINAS e casar mais um momento PIOROU. Só com
   oráculo de APARÊNCIA (a borda serrilhada) e ordem do Enio.

**A fila antiga continua válida abaixo dela, e é o que alimenta o item 1:**

1. ⛔ **A redução de amostras do AA está ESGOTADA como eixo.** Três tentativas, três medições, três
   rejeições: cinco amostras (§3.3), três casando o 2º momento e três casando o 4º (§4.6.2). O que
   sobra do AA (1,43 ms/dab) só cai por um caminho que **não** troque amostragem por aproximação — por
   exemplo cozinhá-lo no **dispositivo**, onde nove leituras de uma tabela pequena são grátis e o
   problema é embaraçosamente paralelo. Isso é a avaliação de GPU do [doc 25](25_avaliacao_gpu.md),
   não uma wave de CPU.
2. **O fork do canvas no pen-down, 3,4 ms uma vez por traço** — captura do "antes" por REGIÃO. Continua
   sendo a única frente que **cura os quatro modos de uma vez**, porque mora acima do modelo de pintura.
3. **O setup dos planos do traço, 1,8 ms uma vez por traço** — representá-los por JANELA em vez de por
   tela. ⚠️ Sem atravessar a cerca do `alloc_zeroed` (§4.4), que já custou 17,6 → 47,5 ms a quem tentou
   o atalho óbvio.
