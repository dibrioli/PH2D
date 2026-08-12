# O ENTALHE NO CRUZAMENTO — a aquarela tem o defeito do FLIP, e mais um em cima

> Enio, 2026-08-12, com foto de uma cruz de aquarela: *"em vários lugares deste app e principalmente
> na implementação do traço de FLIP tivemos problemas com o Alpha que criava reentrâncias nos
> cruzamentos de traços. Parece que o mesmo ocorre com watercolor. Descubra se isso é verdade.
> Estude a cura em FLIP e relate aqui."*

**Veredito: é verdade, e são DOIS mecanismos empilhados.** Um é literalmente o do FLIP; o outro é da
óptica da aquarela e é o que produz a cunha BRANCA da foto. Nada foi construído — este documento é a
medição e a avaliação da cura. Sonda: `crossing_probe::measure_the_crossing_notch`
(`cargo test -p ph2d-tool-painter --release crossing_probe -- --ignored --nocapture`).

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

## 5. Recomendação

**(b) primeiro, e sozinho.** É ele que produz a cunha da foto — **87/255** contra os 46/255 de (a) —,
é o único dos dois que é visível numa lavagem de opacidade normal, e não toca a lei do `max` que
sustenta *"sem build-up dentro de um traço"*. Fazer (a) antes moveria o desenho de toda arte de
aquarela já feita por um número menor que o defeito reportado.

⚠️ **E (a) tem um preço de produto que precisa de decisão sua, não de engenharia:** compor entre
passagens faz o cruzamento **escurecer**, e hoje ele é 0,255 contra 0,278 do braço. Escurecer é o que
tinta faz; mas é mudança de aparência em toda cruz, laço e hachura já pintados.

**Nada disto foi construído.** A sonda fica no repo porque o número tem de ser reproduzível.
