# O ENTALHE NO CRUZAMENTO — a aquarela tem o defeito do FLIP, e mais um em cima

> Enio, 2026-08-12, com foto de uma cruz de aquarela: *"em vários lugares deste app e principalmente
> na implementação do traço de FLIP tivemos problemas com o Alpha que criava reentrâncias nos
> cruzamentos de traços. Parece que o mesmo ocorre com watercolor. Descubra se isso é verdade.
> Estude a cura em FLIP e relate aqui."*

⚠️ **LEIA O §9 ANTES DO RESTO.** Os §§1-8 são a 1ª rodada: eles diagnosticam DOIS mecanismos (a união
da cobertura + o aro que não vira a quina) e constroem a cura do segundo. O smoke seguinte reprovou, e
a medição do §9 **REFUTOU o primeiro** — a união não deixa cunha nenhuma. A causa real, e a cura que
shipa, estão nos §§9-10. O que os §§1-8 dizem sobre o ARO continua válido; o que eles dizem sobre a
UNIÃO está superado.

Sondas: `crossing_probe` (a cena pequena: união × composição) e `crossing_scale_probe` (a varredura de
escala: o défice contra `edge_spread / raio`) —
`cargo test -p ph2d-tool-painter --release crossing -- --ignored --nocapture`.

---

## 1. O que a sonda mede, e por que ela não precisa de imagem de referência

Duas faixas ortogonais de meia-largura `R` se cruzam. Um ponto na bissetriz, a `s` px de **cada**
eixo, recebe de cada faixa a MESMA cobertura `f(s)` que receberia dela sozinho. As duas leis
possíveis dão números diferentes e o oráculo é o próprio ombro da faixa, medido longe do cruzamento:

| lei | cobertura na axila | leitura |
|---|---|---|
| **UNIÃO** (`max`, envelope) | `f(s)` | axila **igual** ao ombro solitário |
| **COMPOSIÇÃO** (cobertura independente) | `1 − (1−f)² = 2f − f²` | axila **acima**, máximo em `f = 0,5` (0,50 → 0,75) |

O **controle DIGITAL** é a lei que ninguém reportou como quebrada, e ele responde limpo.

## 2. As medições

Pincel do produto (`radius 24`, `hardness 0`, `Falloff::Smooth`), cruz de dois traços, tela 256².
`s` é a distância perpendicular ao eixo, de 0 (o eixo) a `R` (a borda).

### DIGITAL — o controle: **COMPÕE**

| s | ombro | axila | composto | axila−ombro |
|---|---|---|---|---|
| 12 | 0,788 | 0,843 | 0,955 | +0,055 |
| 16 | 0,525 | 0,725 | 0,775 | +0,200 |
| 18 | 0,322 | 0,522 | 0,540 | +0,200 |
| **20** | 0,122 | **0,231** | **0,228** | +0,110 |
| **22** | 0,016 | **0,031** | **0,031** | +0,016 |

Nas duas últimas linhas a axila **é** o composto, ao milésimo. O ombro preenche a axila; não há
entalhe, e o mapa de forma sai com um degradê monótono em volta da quina.

### AQUARELA — **NÃO compõe**, e além disso perde o ARO

| s | ombro | axila | composto | axila−ombro |
|---|---|---|---|---|
| 0 | 0,278 | 0,255 | 0,479 | **−0,024** |
| 4 | 0,267 | 0,275 | 0,462 | +0,008 |
| 8 | 0,263 | 0,275 | 0,456 | +0,012 |
| 12 | 0,278 | 0,275 | 0,479 | −0,004 |
| **16** | **0,624** | **0,282** | 0,858 | **−0,341** |
| **18** | **0,612** | **0,404** | 0,849 | **−0,208** |
| 20 | 0,231 | 0,478 | 0,409 | +0,247 |

Duas coisas nesta tabela:

- **No CORPO** (`s ≤ 12`) a axila casa com o ombro e **não** com o composto: o défice contra a
  composição é **0,18 de alfa ≈ 46/255**. Isso é a UNIÃO — o mesmo desvio que o FLIP mediu no
  próprio defeito (**48/255** em hardness 0,4).
- **NA BORDA** (`s = 16..18`) o ombro solitário vale **0,62** — é o **ARO** (edge darkening), 2,2× o
  miolo — e na axila ele **não existe** (0,28). Défice **0,34 ≈ 87/255**, e é este o número da foto.

⚠️ **E o centro do cruzamento é mais CLARO que o miolo de um braço** (0,255 contra 0,278). Tinta que
passa duas vezes ficando mais clara é, sozinha, a assinatura de que a lei ali não é de tinta.

### A FORMA (render-and-look headless, quadrante da quina, dígito = `alpha × 9`)

```
AQUARELA                                  DIGITAL (controle)
2222222222222233445432222334445555555556  8888888888777766543221111111111111111111
222222232222223345542.....12233444444444  8888888888777766543211..................
222222222222223445543........12233322222  8888888888777766543211..................
222222222222223455653...................  8888888888777766543211..................
```

Na aquarela, entre o aro vertical (`3345542`) e o aro horizontal (`334445…`) há uma **faixa de
dígitos baixos e depois zeros** correndo pela bissetriz: a cunha clara. No digital o degrau é
monótono e não há faixa nenhuma.

### E o contraste com o FLIP que decide o diagnóstico

**Dois traços e UM traço cruzando a si mesmo dão a MESMA tabela, aos três decimais.** No FLIP os dois
casos eram *diferentes* — e essa diferença era o diagnóstico dele (*"com traços distintos o depth
difere e o mais novo pinta por cima, ou seja **já compõe**; um traço cruzando a si mesmo tem o mesmo
depth e caía na união"*). Na aquarela **os dois caem na união**: o wash nunca compõe no cruzamento.

## 3. Os dois mecanismos, nomeados no código

O cabeçalho do `watercolor_render.rs` já escreve a óptica inteira:

```text
cover = smoothstep(SS0, SS1, coverage(warp(x,y)))
inner = blur(coverage)                        // ~1 dentro, →0 no aro
edge  = clamp(cover·(1 − inner)·edge_gain, 0, 1)
D     = (cover·fill + edge)·gran
```

**(a) A UNIÃO.** `coverage` sai do `accumulate_wet_coverage`, que é um **max-blend** por dab
(`if v > cov[idx] { cov[idx] = v }`) — e a sessão molhada o mantém entre traços. Então no cruzamento
a cobertura é `max(a, b)`, nunca `a + b − ab`. É a lei que o FLIP chama de união, com o vinco na
bissetriz.

⚠️ **E o max-blend não é um descuido:** ele É a decisão *"sem build-up dentro de um traço"* — é por
causa dele que o Accumulate é escondido sob o wash como redundante (doc 13 #4). Trocá-lo por
composição **por-dab** faria a lavagem escurecer ao longo do traço em função do **Spacing**, que é a
doença I1 que esta linha curou quatro vezes no relevo.

**(b) O ARO QUE NÃO VIRA A QUINA — e é o que se vê.** `inner = blur(coverage)`. Numa quina
**côncava** o borrão enxerga MAIS interior que num flanco reto, então `inner` sobe, `(1 − inner)`
cai, e o `edge` **desaparece** exatamente ali. O aro de cada faixa termina na quina em vez de
contorná-la, e o que sobra entre os dois aros escuros é uma faixa clara na bissetriz. Um filtro
passa-baixa linear não representa uma fronteira reentrante na escala do próprio aro — é a mesma
família do problema de **offset de curva em quina côncava** que o `curve_offset` do Painter pagou
(BUGS #1).

## 4. A cura do FLIP, e o que dela transfere

**A cura** (`flip.wgsl`, §"UMA PASSAGEM, UMA COBERTURA", 2026-07-28, 2º report do Enio):

> Tomar `hardness_mask(min(...))` sobre TODAS as passagens é a UNIÃO, e `min` de duas funções lisas
> tem **VINCO** na bissetriz do cruzamento. Compor as coberturas — `1 − (1−a)(1−b)`, a hipótese de
> cobertura independente, exatamente o que o `over` de dois traços produz — é liso e faz as duas
> rotas desenharem a mesma coisa.

Três propriedades que a tornam segura, e que são o que vale copiar:

1. **União DENTRO de uma passagem, composição ENTRE passagens.** O habilitador é a lista de vizinhos
   **particionada por passagem** (`neighbors::SegExtras`: os primeiros `n_ribbon` são a própria fita,
   o resto são outras passagens).
2. **Compõe-se a COBERTURA, nunca o ALFA.** A opacidade multiplica depois — então um traço a
   opacity 0,5 **não escurece sobre si mesmo**, que é a regra que o artista espera.
3. **Sem cruzamento é BYTE-IDÊNTICO por construção** (`n_ribbon == n_all` ⇒ o ramo nem roda).

Medido lá: o desvio entre as duas rotas caiu de **48/255** (hardness 0,4) e 35/255 (0,7) para
**1/255**.

**O que transfere, e o que não:**

- **(a) transfere com uma pergunta a responder antes:** *o que é uma "passagem" na aquarela?* O
  splat de cobertura recebe uma lista de dabs sem identidade de passagem, e a sessão molhada
  deliberadamente funde traços. Compor entre passagens exige o análogo do `SegExtras` aqui — e a
  propriedade 1 é justamente o que impede a cura de virar build-by-spacing.
- **(b) NÃO transfere: o FLIP não tem aro.** O `inner = blur(coverage)` é da aquarela e precisa de
  resposta própria. Duas candidatas, **nenhuma medida ainda**: computar o `inner` por passagem e
  compor também o `edge` (o que faz o aro contornar a quina porque cada passagem contorna a sua), ou
  trocar o borrão por uma medida de distância com regra explícita de quina côncava.

## 5. Recomendação — e o que foi CONSTRUÍDO (ordem do Enio: *"faça como sugere"*)

**(b) primeiro, e sozinho.** É ele que produz a cunha da foto — **87/255** contra os 46/255 de (a) —,
é o único dos dois que é visível numa lavagem de opacidade normal, e não toca a lei do `max` que
sustenta *"sem build-up dentro de um traço"*. Fazer (a) antes moveria o desenho de toda arte de
aquarela já feita por um número menor que o defeito reportado.

⚠️ **(a) NÃO foi construído, e tem um preço de produto que é decisão sua:** compor entre passagens faz
o cruzamento **escurecer**, e hoje ele é 0,255 contra 0,278 do braço. Escurecer é o que tinta faz; mas
é mudança de aparência em toda cruz, laço e hachura já pintados.

---

## 6. A cura de (b), como ela ficou

Crate-módulo novo [`watercolor_rim.rs`](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_rim.rs):

```text
inner := min(blur(hard), P(sd, r))
P(sd, r) = clamp((sd + r + 0.5) / (2r + 1), 0, 1)      // a resposta do box blur a um DEGRAU
```

`sd` é a distância **assinada** à fronteira `hard = 0.5`, e o teto é *o `inner` que um flanco RETO
daria à mesma distância*. Quatro propriedades, cada uma medida:

1. **É TETO, não substituição.** A correção que a quina côncava precisa só tem um sinal — mais aro e
   menos franja, as duas saem de um `inner` menor. Um `min` nunca enfraquece o aro em lugar nenhum.
2. **Nos DOIS lados.** A versão só-por-dentro foi construída e é metade da cura: ela engrossa o aro
   ao aproximar-se da quina (`44543` → `66542` no mapa) e **deixa o vão onde estava**, porque o vão
   está FORA — e aro mais forte ao lado de um vão intocado *aumenta* o contraste que faz a cunha ser
   vista. Nos dois lados o vão fecha (`2222` → `333333`, buraco de 5 px → 2).
3. **UMA EDT, não duas.** A 1ª versão pedia a distância de dentro e a do complemento e custava **3,67
   borrões** no caminho quente. A pergunta certa é a distância ao **conjunto-fronteira** — uma
   transformada, com o sinal vindo da máscara de graça: **1,69 borrões**, medido costas-com-costas na
   mesma corrida (razão, nunca wall-clock: esta máquina oscila).
4. **A EDT é a que já existe** — `sculpt_close::distance_inside`, cujo doc diz que um segundo
   consumidor sempre foi a intenção. Este é o terceiro; zero kernel novo.

**Gates:** `the_rim_turns_the_concave_corner_instead_of_leaving_a_wedge` (cruz de dois traços) ·
`the_first_stroke_of_the_session_turns_the_corner_too` · os três de unidade do módulo.

⚠️ **O oráculo é DERIVADO:** *uma cunha é um lugar mais claro que a tinta em volta*. A 1ª versão do
gate levou uma barra tirada do mapa (`pit > 0.20`) e **a mutação sobreviveu** — sem a cura o vão mede
0,247, que passava. Medidos os dois estados (**0,247** sem cura, **0,322** com), o que os separa não é
um número a escolher: é o **miolo do braço** (0,278), que está entre eles e sai da mesma corrida.

## 7. ⚠️ O PREÇO, e é ele que o smoke julga

**Todo aro de aquarela muda um pouco.** Medido byte a byte na fixture do pino de fingerprint (traço
RETO, raio 40, warp 6), com e sem a cura:

| | |
|---|---|
| bytes que diferem | **6247 de 262144 (2,4%)** |
| pixels tocados | **2486 de 65536 (3,8%)** |
| pior delta | **18/255** |
| onde | **só na banda do ARO** — miolo e papel não se movem |

E ele move num traço **reto de propósito, não por acidente**: aquela fixture tem `warp = 6`, e um
contorno ondulado é localmente **côncavo em metade das ondas**. É a mesma correção da quina, na escala
da ondulação — *a lei antiga errava em toda concavidade, e não só no cruzamento que a foto mostrou*.

⚠️ **O pino `smooth_edges_off_is_the_pre_aa_render_byte_for_byte` MOVEU** (`0xc5ebf8cf645fb6f6` →
`0xe59f2fb788ce5874`), re-escrito **com a justificação e os números ao lado**, nunca em silêncio — o
protocolo do doc 23.

## 8. O que sobra, com o número

O vão **encolhe de 5 px para 2**, não para zero. Os 2 px que ficam são **(a)**, a união da cobertura —
que segue não construída de propósito. Se o smoke disser que ainda se vê, é (a) que decide, e ela é a
pergunta de produto do §5.

## 9. ⛔ O SMOKE DISSE QUE AINDA SE VÊ — e a medição REFUTOU (a)

Enio, 2026-08-12, com a mesma foto: *"melhor mas ainda com o alpha errado"*.

A recomendação do §5 mandava, se isto acontecesse, ir para **(a)** — a união da cobertura. A varredura
abaixo diz que **(a) não é a causa**, e que a causa é outra coisa, com uma tabela.

### 9.1 O que foi ablacionado, e o que cada célula disse

Tudo pela ENTRADA (knobs do pincel), um termo por vez, no **regime do produto** (`fill` 0,12 —
ver 9.2). Sonda `measure_the_crossing_notch`:

| cena | a quina côncava |
|---|---|
| só a SILHUETA (aro 0 · gran 0 · warp 0) | **quina RETA, limpa** — nenhum vão |
| silhueta + aro | limpa |
| silhueta + granulação | limpa |
| silhueta + **warp** | **o entalhe aparece** |
| silhueta + warp, **sem aro** | **o entalhe continua** |

E a medida que fecha **(a)**: o alcance ao longo da bissetriz. A união de duas faixas de meia-largura
`w` tem quina reta em `(w, w)`, logo a bissetriz alcança `w·√2` — **exatamente, sem constante
escolhida**. Medido nos QUATRO cantos, em três escalas de pincel: o alcance real é `w·√2` **ou mais**
(30,0–36,0 contra 28,3 esperados a raio 24), e o perfil ao longo da bissetriz é **monótono** — não há
buraco. ⇒ **a cobertura não deixa cunha nenhuma; compor a união não tinha o que consertar.**

### 9.2 ⚠️ Duas fixtures desta sonda nasceram CEGAS, e as duas eram minhas

* **A ESCALA do mapa.** Com o `fill` de fábrica (0,12) a lavagem inteira vive em alpha ≈ 0,28, então
  `alpha*9` colapsava a cena toda em `2` — um mapa que só sabe dizer 1, 2 e 3 **não pode responder
  onde a tinta acaba**, e foi ele que me fez ler ruído de granulação como cunha. Subir o `fill` para 1
  resolve o dígito e **troca o regime** (o interior satura e o aro deixa de dominar, o oposto do
  produto) ⇒ quem sobe é a **escala do dígito**, nunca o pincel.
* **O ponto do flanco reto** era `c − 4·raio`, que num pincel de 75 cai **fora da tela** (o `as u32`
  satura em 0) e a sonda passou a medir a **calota do começo do traço** em vez do flanco: `w = 48`
  para um braço de 65.

### 9.3 A CAUSA, com a tabela

O aro é `gain·(cw − blur(hard))` e o borrão tem raio `core_r = min(edge_spread, raio/2)` — um número
em **px ABSOLUTOS**, enquanto o ombro da silhueta **escala com o pincel**. Num pincel grande o ombro
fica muito mais largo que o borrão, `blur(hard) → hard`, e **o aro enfraquece**; na quina côncava ele
**se rompe**, porque ali o borrão enxerga a tinta do OUTRO braço (a quina é geometricamente mais
funda: a distância à fronteira vale `t·√2` onde cada frente está a `t`).

Sonda `measure_the_rim_deficit_against_spread_over_radius` — pico do aro no flanco reto contra o
**pior dos quatro cantos**:

| raio | spread | spread/raio | flanco | quina | défice |
|---:|---:|---:|---:|---:|---:|
| 24 | 7 | 0,29 | 0,72 | 0,71 | −1% |
| 24 | 16 | 0,67 | 0,74 | 0,74 | +1% |
| 48 | 7 | **0,15** | 0,60 | 0,54 | **−9%** |
| 48 | 16 | 0,33 | 0,70 | 0,70 | −1% |
| 48 | 32 | 0,67 | 0,72 | 0,73 | +2% |
| 75 | 7 | **0,09** | 0,53 | 0,36 | **−33%** |
| 75 | 16 | 0,21 | 0,69 | 0,61 | −11% |
| 75 | 32 | 0,43 | 0,74 | 0,70 | −5% |
| 110 | 7 | **0,06** | 0,44 | 0,36 | **−19%** |
| 110 | 32 | 0,29 | 0,72 | 0,70 | −3% |
| 110 | 48 | 0,44 | 0,75 | 0,73 | −2% |

**O défice colapsa num número só: `edge_spread / raio`.** Acima de ~0,3 ele é ≤ 3% (invisível);
em 0,09 o aro da quina vale **dois terços** do aro do flanco, e a 110 px o perfil da quina chega a
**zerar no meio** (buraco de 2,5 px cercado de tinta, medido) — que é a cunha branca da foto.

⚠️ **E a coluna do FLANCO diz a mesma coisa sobre o produto inteiro:** com `spread` fixo em 7 o aro
do flanco cai de 0,72 (raio 24) para **0,44** (raio 110). O aro não é invariante de escala; a quina é
só onde a perda vira **ruptura**.

### 9.4 O que fazer, e por que a decisão é do Enio

**Hoje, sem tocar em código:** o slider **Spread** vai até 48 e a tabela diz que a partir de
`spread ≈ 0,3 × raio` a cunha some. Num pincel de 75 px isso é **Spread ≈ 24**; o default é 7.

As duas curas possíveis, com o que cada uma custa:

1. **Piso relativo no `core_r`** (`max(edge_spread, 0,3·raio)`) — uma linha, e a tabela já mediu que
   funciona. ⚠️ **Mas o clamp de cima já é `raio/2`**, então a faixa útil do knob viraria
   `0,3·R .. 0,5·R`: o Spread deixa de obedecer o artista em pincel grande. *Um knob que para de
   obedecer é pior que o defeito que ele cura.*
2. **Régua corner-safe para o aro** — o `inner` é inflado na quina porque o borrão é isotrópico; a
   grandeza certa é a distância à **frente mais próxima**, e ela **está na COBERTURA** (a cobertura é
   um `max` de falloffs, logo `cov = f(min_i dᵢ)`), não na geometria do nível dela. A cura seria
   `sd_usado = min(sd_geométrico, t(cov))`, com `t` vindo de uma LUT da inversa do falloff por estilo:
   flanco **inalterado por construção**, interior inerte, quina igualada ao flanco. É **wave própria**
   (LUT por estilo no `WetStrokeStyle`, gate de identidade no flanco, orçamento no caminho quente).

⚠️ **A (1) muda o LOOK de toda lavagem de pincel grande** (aro mais largo e mais forte) e mexe na
semântica de um knob shipado; a (2) não muda nada fora da quina. **Recomendação: (2)** — e o
Spread continua sendo do artista.


## 10. A CURA (2) — a régua sai da COBERTURA (ordem do Enio: *"vamos tentar 2"*)

### 10.1 A lei, em duas linhas

O teto do §6 limita `inner` por `P(régua, core_r)`. O que estava errado era a **régua**: ela era a
distância geométrica à fronteira da UNIÃO, e numa quina côncava um ponto a `t` px de cada braço está
a `t·√2` dali (o vizinho mais próximo é o vértice). Agora ela é o **mínimo** de duas leituras:

```text
  régua = min( distância geométrica (EDT) , (cov − COV_HALF) / |∇cov| )
```

⚠️ **É um `min`, como o teto** — só pode DAR aro, nunca tirar. E a segunda leitura é a estimativa
analítica clássica da distância ao nível `cov = COV_HALF`, que é **exatamente** o nível que a EDT
semeia (via o endurecimento): as duas medem a mesma fronteira, por caminhos diferentes.

⚠️ **A propriedade que ela estabelece não é *"a quina mede a distância à frente mais próxima"*** — essa
frase era MINHA e está errada, e foi o gate que a derrubou. A cobertura é um `max`, então o valor num
texel é o da faixa que o cobre **melhor**; a régua devolve a profundidade que a COBERTURA implica, e a
propriedade é ***mesma cobertura, mesmo aro***. É a certa: a cobertura é o que decide o TOM da
silhueta ali, e o aro tem de acompanhar o tom, não uma distância que o artista não vê.

### 10.2 Três decisões, cada uma com o número que a obrigou

* **Lê a cobertura CRUA, não a endurecida.** Com `hard` a régua morre onde o `smoothstep` satura
  (±6 px num pincel grande) — exatamente onde o aro vive. Medido: com `hard` a cura valia **4 pontos
  percentuais**; com `cov`, **20**.
* **O termo de UM LADO é a CRISTA, não higiene.** Na bissetriz exata de um `max` de dois campos lisos
  a diferença central mede **metade** (para a frente o campo é constante, para trás ele sobe) e `|∇|`
  sai `√2` pequeno demais — na linha exata que a quina tem. Sem ele, a cura trocaria a cunha por uma
  **rachadura de 1 px** correndo pela bissetriz. E `|∇|` (e não o máximo por eixo) é quem serve o
  flanco a 45°: as duas metades do `max` cobrem casos opostos, e nenhuma sozinha é isotrópica.
* **A régua só é calculada na FAIXA em que o teto pode agir** (`−(r+½) < geom < (r+1½)·2`), e fora
  dela ela é **provadamente inerte** (`P` satura em 0 ou em 1 nos dois lados). É o corte que a torna
  barata: **≈1 borrão** varrida a janela inteira contra **0,16 borrão** com ele, porque a faixa é o
  PERÍMETRO e não a janela. ⚠️ O `2` é um **limite NOMEADO**: numa quina de ângulo `θ` a frente está a
  `geom·sen(θ/2)`, e ele cobre até **60°** — dois traços cruzando mais agudo que isso mantêm o aro de
  hoje na quina.

### 10.3 O resultado, medido nos QUATRO cantos

| raio · spread | antes | depois |
|---|---:|---:|
| 24 · 7 | −1% | −1% |
| 48 · 7 | −9% | **+3%** |
| 75 · 7 | **−33%** | **−13%** |
| 75 · 16 | −11% | **−4%** |
| 110 · 7 | −19% | −19% |
| 110 · 16 | −14% | **−4%** |
| 110 · 32 | −3% | −2% |

E o que a foto mostra: **os BURACOS sumiram**. Antes, a 110 px, os perfis dos cantos zeravam no meio
(vão de 2,5 px cercado de tinta); agora a sonda mede **0,0 px de buraco nos quatro cantos, nas quatro
escalas**, e os perfis são monótonos.

### 10.4 O preço, e o que NÃO fecha

**Custo:** +0,16 borrão no campo do aro (sonda `the_coverage_ruler_costs_this_many_blurs`,
intercalada e com o mínimo como redutor — esta máquina deriva 4× sob carga).

**Aparência:** o pino `smooth_edges_off_is_the_pre_aa_render_byte_for_byte` moveu de novo
(`0xe59f2fb788ce5874` → `0x9744233f9f852066`), e o preço num traço RETO é **494 bytes de 262144
(0,2%) · 192 px de 65536 (0,3%) · pior delta 14/255** — **um quinto** do 1º movimento, porque num
flanco as duas réguas concordam e o `min` é quase inerte.

⚠️ **`raio 110 · spread 7` (razão 0,06) NÃO melhora, e a causa está medida:** ali o ombro endurecido
mede ~46 px contra um borrão de 15, então `blur(hard) ≈ hard` e ele fica **sempre abaixo** do teto —
o `min` nunca morde, e a régua (que só entra pelo teto) não tem como agir. Naquele regime o aro do
próprio flanco já vale 0,44 contra os 0,74 de um pincel pequeno: **quem está degenerado é o modelo do
aro em pincel grande**, não a quina. A resposta que existe hoje é o slider **Spread** (a partir de
`≈ 0,3 × raio` o défice é ≤ 3% em toda a tabela); a resposta estrutural seria o aro deixar de ser
derivado de um borrão de raio ABSOLUTO — outra wave, e ela muda o look de toda lavagem.

### 10.5 ⚠️ E o commit anterior shipou um vermelho-latente de LOC

`watercolor_render.rs` foi de 688 para **725** no `28d2b2596` e cruzou o teto de 700 **sem ninguém
ver**: o gate mora em `ph2d-editor-core/tests/` e um fechamento por `cargo test -p ph2d-tool-painter`
**não o alcança** — a mesma causa estrutural que physics, motion-value e Vector já documentaram.
Curado por corte de RESPONSABILIDADE, não por tamanho: `hard`, os borrões do `inner` e a régua do teto
**nascem juntos e só o aro os lê**, então a receita virou `watercolor_rim::rim_fields` e o composite
ficou com o laço que compõe o pixel. O mesmo corte na sonda: o pai responde *união ou composição?* na
cena pequena, o irmão `crossing_scale_probe` responde *o défice contra `spread/raio`* — assunto, não
tamanho.
