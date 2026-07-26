# 28 — Otimizações do Painter: o que funcionou, o que NÃO funcionou, e o que serve aos outros modos

> **Este doc é o registro de uma jornada de perf inteira** (2026-07-26, `line/Painter`), escrito para que
> ninguém reconstrua o que já foi medido e reprovado. O plano operacional vive no
> [26_plano_performance_procreate.md](26_plano_performance_procreate.md); aqui está o **saldo**: cada
> frente com o número que a matou ou a aprovou, e o mecanismo por trás do número.
>
> ⚠️ **A regra que governou tudo (CLAUDE.md §0):** nenhum limite, nenhuma barra e nenhum veredito sem
> MEDIÇÃO. Três teorias minhas, plausíveis e erradas, morreram nesta jornada — e cada uma está aqui com
> o número que a derrubou, porque uma teoria refutada que não é escrita volta como trabalho planejado.

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
| K | **O AA do filme POR DAB** | 🎯 **o maior item que sobrou**, mas 🔴 **não diagnosticado** | **2,60 dos 6,35 ms** — 41%, depois da LUT. ⚠️ 335 ns/texel de banda: o modelo "são as nove amostras" NÃO fecha (§4.6.1) |

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

### 4.6.1 ⚠️ E o custo do AA NÃO é o que eu supunha — duas hipóteses medidas e REFUTADAS

A banda do AA é conhecida (`how_wide_is_the_aa_band`), e para o falloff default do impasto ela é
**24,7% da área do disco — 7 758 texels a r=100**:

| falloff | `t_lo .. t_hi` | fração da área | texels a r=100 |
|---|---|---|---|
| **Sphere** (default do impasto) | 0,805 .. 0,946 | **24,7%** | 7 758 |
| Smooth | 0,437 .. 0,611 | 18,2% | 5 721 |
| Smoother | 0,448 .. 0,591 | 14,9% | 4 671 |
| Sharp | 0,229 .. 0,418 | 12,2% | 3 837 |

**2,60 ms sobre 7 758 texels = 335 ns/texel.** Nove leituras numa tabela quente não custam isso — então
o modelo *"o AA é as nove amostras"* **não fecha**, e as duas explicações que eu tinha morreram medidas:

1. ⛔ **"a closure da silhueta é construída por texel mesmo sem ser chamada"** — trocada por uma trivial
   (`|_,_| 0.0`, imagem errada de propósito): AA **2,53** contra 2,57 da base. **Sem diferença.**
2. ⛔ **"o corpo da closure infla o laço quente"** — `film_at_exact` marcado `#[inline(never)] + #[cold]`
   (imagem INTACTA): AA **2,83**, ou seja **PIOR** que a base. Outlinar cobra do caminho inadmissível e
   não devolve nada.

⚠️ **Fica ABERTO onde os 335 ns/texel estão**, e a próxima tentativa tem de começar por descobrir isso —
não por desenhar uma aproximação nova. Projetar a convolução 2-D (um lookup em vez de nove) sobre um
modelo de custo que a medição já contradisse seria construir a cura de uma doença não diagnosticada.

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

---

## 7. Próxima etapa recomendada

⚠️ **A medição REORDENOU a fila, e a recomendação anterior deste doc está superada.** Eu havia
apontado os planos canvas-sized do traço como o item maior; a comparação contra um MOVE (§4.5) mostrou
que eles valem **1,8 ms**, não 5,1 — o resto era um dab comum. E a decomposição do dab (§4.6) mostrou
onde o tempo realmente está.

**A fila, por tamanho medido:**

1. 🎯 **O AA do filme, 2,60 ms POR DAB** (41% do dab de relevo, depois da LUT) — o maior item do
   módulo, pago em **todo move**. ⚠️ **Mas o próximo passo dele é DIAGNOSTICAR, não construir:** são
   335 ns por texel de banda, o que o modelo *"são as nove amostras"* não explica, e as duas
   explicações alternativas já morreram medidas (§4.6.1). A convolução 2-D (um lookup em vez de nove)
   é a cura ÓBVIA — e projetá-la agora seria curar uma doença não diagnosticada, com uma aproximação
   nova, num kernel onde a §3.3 registra que aproximações morrem por casos imprevistos.
2. **O fork do canvas no pen-down, 3,4 ms uma vez por traço** — captura do "antes" por REGIÃO. Continua
   sendo a única frente que **cura os quatro modos de uma vez**, porque mora acima do modelo de pintura.
3. **O setup dos planos do traço, 1,8 ms uma vez por traço** — representá-los por JANELA em vez de por
   tela. ⚠️ Sem atravessar a cerca do `alloc_zeroed` (§4.4), que já custou 17,6 → 47,5 ms a quem tentou
   o atalho óbvio.
