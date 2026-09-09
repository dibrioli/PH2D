# 10 — As outras juntas: o que existe além de Fillet, Chamfer e Organic

> **Pedido do Enio, 2026-09-09:** *«Além das junções de tipo Organic, Fillet e Chamfer, faça pesquisa
> de outros tipos funcionais, de boa aparência e de boa performance e me relate quais existem»*

Instrumento: [`probe_the_other_junctions.rs`](../../crates/ph2d-field-eval/tests/probe_the_other_junctions.rs).

```
cargo test --release -p ph2d-field-eval --test probe_the_other_junctions -- --ignored --nocapture
```

⚠️ **Nada deste documento está no produto.** O que ele entrega é a tabela com que a escolha se faz.

---

## §1 — As quatro réguas, e por que são quatro

| régua | responde a | sem ela |
|---|---|---|
| **recuo** / **mordida** | *que tamanho o artista vê?* | a fileira de caracteres deixa de medir a mesma coisa |
| **`‖∇f‖`** | *a marcha pode andar o valor do campo?* | a peça **fura** |
| **curvatura** (`κ máx`, `salto`) | *o brilho corre liso pela junta?* | «boa aparência» fica adjectivo |
| **nós** / **ns por ponto** | *quanto custa avaliar?* | «boa performance» idem |

⭐⭐ **A régua da curvatura é nova nesta casa.** O filete exacto é `G1`: a curvatura salta de `0` na
parede para `1/r` no arco, e é esse degrau que uma luz de estúdio desenha como uma banda na peça.
Ela está validada: o `Fillet` a `r = 0,25` mede `κ máx = 4,002` contra o `1/r = 4` analítico.

⚠️ **`κ = 200,000` é o TECTO da amostragem** (passo `0,01`), e lê-se como *«vinco vivo»*, não como um
número.

---

## §2 — ⭐⭐⭐ A lei que a medição deu: o preço de uma junta NÃO é a aritmética dela

| grandeza | pior / melhor | quem paga |
|---|---|---|
| **ns por ponto** | `1,98 / 1,17` = **`1,7×`** | só os pontos que tocam aquela aresta |
| **`‖∇f‖`** | `50,5 / 1,00` = **`50×`** | **a cena inteira, em cada pixel** |

O passo da marcha é `1/gradient_bound(doc)` e o `gradient_bound` percorre o **documento** ([`step.rs`](../../crates/ph2d-field-eval/src/step.rs)):
o pior operador da árvore fixa o passo de tudo o que se desenha. ⇒ **uma junta que infle o gradiente
para `8×` faz a cena inteira marchar `8×` mais devagar (ou furar).**

⇒ *A pergunta de performance de uma junta é «que `‖∇f‖` ela deixa?», e só depois «quantos nós?».*

---

## §3 — A tabela MEDIDA (canto côncavo de 90°, `r = 0,25`, `load 3,47`)

| junta | recuo | mordida | `‖∇f‖` | `κ máx` | salto | nós | ns/pto |
|---|---:|---:|---:|---:|---:|---:|---:|
| **Fillet** (a casa) | 0,2429 | 0,1036 | **1,0000** | 4,002 | 2,880 | 18 | 1,30 |
| **Chamfer** (a casa) | 0,2499 | 0,1768 | **1,0000** | 39,3 | 39,3 | 10 | 1,18 |
| **Organic G1** (a casa) | 0,2822 | 0,1036 | **1,0000** | 4,828 | 1,290 | 20 | 1,29 |
| **Organic G2** (cúbico) | 0,3543 | 0,0943 | **1,0000** | 6,988 | **0,473** | **16** | 1,26 |
| chanfro de 2 recuos `1:3` | 0,7497 | 0,2652 | **1,0000** | 73,5 | 73,5 | 14 | 1,20 |
| filete elíptico `1:3` | 0,7288 | 0,1645 | 0,9981 | 11,1 | 8,47 | 24 | 1,39 |
| **cordão de solda** | 0,1750 | 0,1750 | **1,0000** | 99,3 | 93,6 | **13** | **1,17** |
| **sulco** (linha de painel) | — | −0,1768 | **1,0000** | ⊤ | ⊤ | 14 | 1,19 |
| **friso** (nervura) | 0,1550 | 0,1768 | **1,0000** | ⊤ | ⊤ | 14 | 1,21 |
| gravação em V | — | −0,2121 | 1,3066 | ⊤ | ⊤ | 13 | 1,20 |
| **escada** `n = 3` | 0,1667 | 0,1179 | **1,0000** | ⊤ | ⊤ | 18 | 1,30 |
| escada `n = 6` | 0,2083 | 0,1768 | **1,0000** | ⊤ | ⊤ | 18 | 1,28 |
| colunata `n = 3` | 0,2085 | 0,1497 | **1,0000** | 90,8 | 75,7 | 32 | 1,68 |

### §3.1 — A família da PLENITUDE (norma-`p`), e o precipício de custo

`u = (r−a)⁺`, `v = (r−b)⁺`, superfície em `u^p + v^p = r^p`:

| `p` | recuo | mordida | `‖∇f‖` | `κ máx` | salto | nós | ns/pto **geral** | ns/pto **barato** |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0,25 | 0,2500 | 0,3315 | ⛔ **50,48** | 116,9 | 116,9 | 24 | — | 1,56 |
| 0,50 | 0,2500 | 0,2652 | ⛔ **8,337** | 115,1 | 115,1 | 19 | 34,04 | **1,39** |
| 0,75 | 0,2500 | 0,2132 | 2,922 | 85,2 | 85,2 | 28 | 33,77 | — |
| **1,00** | 0,2499 | 0,1768 | 1,4142 | 39,3 | 39,3 | 13 | **1,25** | — |
| 1,50 | 0,2482 | 0,1308 | 1,1225 | 12,4 | 6,50 | 28 | 37,68 | — |
| **2,00** | **0,2429** | **0,1036** | **1,0000** | **4,002** | **2,880** | 18 | **1,34** | **1,34** |
| 3,00 | 0,2234 | 0,0729 | 1,0000 | 7,13 | **0,494** | 28 | 36,64 | — |
| 4,00 | 0,2000 | 0,0563 | 1,0000 | 10,1 | 1,033 | 27 | 36,54 | **1,46** |
| 8,00 | 0,1281 | 0,0293 | 1,0000 | 21,5 | 4,69 | 28 | 36,56 | **1,59** |
| 16,0 | 0,0677 | 0,0150 | 1,0000 | 43,1 | 16,6 | 30 | — | **1,90** |
| →∞ | 0,0000 | 0,0000 | 1,0000 | ⊤ | ⊤ | 13 | 1,28 | — |

⭐⭐⭐ **`p = 2` é o `Fillet` desta casa, número a número** (`0,2429 / 0,1036 / 1,0000 / 4,002 /
2,880`) **e `p = 1` é o `Chamfer`** (`0,2499 / 0,1768`, mesmo nível — o operador da casa divide por
`√2` a mais, e é por isso que ali ele lê `1,0000` onde a família lê `1,4142`).
⇒ *os dois caracteres que o artista já conhece são DOIS PONTOS de um eixo só.*

⭐⭐ **E o eixo é BARATO nas potências de dois.** `p = 2^m` escreve-se com `m` quadrados e `m` raízes;
o expoente arbitrário passa por `exp`/`ln` e custa **`36,54` contra `1,46` ns — `25×`** (as duas
colunas dão a **mesma forma**, dígito a dígito). ⇒ *uma fileira de chips é grátis; um slider contínuo
custa 25× por avaliação.*

⛔ **E o lado «mais cheio que o chanfro» (`p < 1`) está FORA:** `‖∇f‖` vai a `8,3` e a `50,5`, e o
gradiente de `(u^p+v^p)^{1/p}` é **ilimitado** nas tangências quando `p < 1`. Ver §5.

### §3.2 — A junta CÓNICA (o `Rho` do CAD)

`a·b = λ·(a+b−r)²` com `λ = (1−ρ)²/(4ρ²)` — **um produto e um quadrado**, contínua em `ρ`, sem
transcendental nenhum.

| `ρ` | a cónica | recuo | mordida | `‖∇f‖` | `κ máx` | salto | nós | ns/pto |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0,150 | quase a corda | 0,2500 | 0,1503 | 1,9074 | ⊤ | ⊤ | 19 | 1,30 |
| 0,300 | elipse | 0,2457 | 0,1237 | 1,9557 | 8,57 | 6,32 | 19 | 1,30 |
| **0,4142** | **circunferência** | **0,2429** | **0,1036** | 1,7024 | **4,002** | 2,880 | 19 | 1,30 |
| 0,500 | parábola | 0,2401 | 0,0884 | **1,0000** | 5,66 | 1,549 | 18 | 1,29 |
| 0,650 | hipérbole | 0,2320 | 0,0619 | 1,2390 | 10,5 | 1,225 | 19 | 1,29 |
| 0,850 | quase a quina | 0,1993 | 0,0265 | 1,3985 | 31,3 | 10,8 | 19 | 1,30 |

⭐ `ρ = √2 − 1` reproduz o `Fillet` **na forma** (`0,2429 / 0,1036 / 4,002 / 2,880`).
⛔ **O que a impede de shipar hoje é o RECORTE, e as duas saídas foram medidas:**

| recorte | forma | `‖∇f‖` |
|---|---|---|
| só o tecto (`max(a−r, b−r)`) | **certa** | `1,70`–`1,96` |
| a caixa inteira (com `−a` e `−b`) | ⛔ **descola a junta da parede** | `1,0000` |

⭐⭐ *O material de um filete TOCA as duas faces; um recorte que exclui `a < 0` exclui também
`a = 0`, e a tangência deixa de existir.* A cónica crua é uma **quadrática**, não uma distância —
e a distância a uma cónica não tem forma fechada (é por isso que o CAD a resolve por Newton sobre
NURBS).

---

## §4 — As famílias, e o que cada uma É

### §4.1 — ⭐⭐⭐ A PLENITUDE: um controlo que contém os dois que já existem

Um número só percorre **quina viva → arco apertado → o Fillet de hoje → o Chamfer de hoje**. É o
*Profile* do Bevel do Blender e o *Conic Rho* do SolidWorks/NX/Fusion, e resolve por composição uma
pergunta que hoje precisa de dois chips e dois sliders.

⚠️ **Ele não substitui a fileira de caracteres — ele explica-a.** `Organic` continua fora do eixo
(é outra lei: mistura os dois campos em vez de recortar a quina).

### §4.2 — ⭐⭐⭐ As DECORAÇÕES DA COSTURA: o que nenhuma composição exprime

Todas partem do mesmo par: `d = min(a,b)` é a superfície da união e `s = (a−b)/√2` é a distância com
sinal à costura. **Com esse par, a costura acha-se sozinha.**

| junta | o que é | onde se usa |
|---|---|---|
| **cordão** | tubo de raio `r` sobre a costura | solda, cola, vedante, junta de fundição |
| **sulco** | canal escavado sobre a costura | **linha de painel** (o detalhe nº 1 de hard-surface) |
| **friso** | nervura levantada sobre a costura | reforço, moldura, filete de plástico |
| **gravação em V** | incisão em V | ranhura, entalhe, marca de separação |
| **escada** | a transição em `n` degraus | maquinado, sci-fi, *greeble* |
| **colunata** | `n` colunas na diagonal | caneluras, arquitectura |

⭐⭐⭐ **E elas são as MAIS BARATAS da tabela inteira** (`13`–`18` nós, `1,17`–`1,30` ns, `‖∇f‖`
`1,0000`) — mais baratas que o `Fillet` que já shipa.

### §4.3 — As ASSIMÉTRICAS: quando as duas faces não são iguais

- **chanfro de dois recuos** (`ca ≠ cb`): `14` nós, `‖∇f‖ = 1,0000`. É o *two-distance chamfer* do
  CAD, e o custo é **zero** — o plano já existia, só recebe dois divisores.
- **filete elíptico** (`ra ≠ rb`): `24` nós, `‖∇f‖ = 0,9981` (minorante ⇒ seguro).

⚠️ **Não são cosmética:** um rebordo que encontra uma chapa quer raio grande do lado da chapa e
pequeno do lado do rebordo, e hoje isso é inexprimível.

### §4.4 — ⭐⭐ O `Organic` de segunda ordem: mais liso E mais barato

| | `κ máx` | **salto** | nós | ns/pto |
|---|---:|---:|---:|---:|
| `Organic` de hoje (polinomial `G1`) | 4,828 | 1,290 | 20 | 1,29 |
| **cúbico (`G2`)** | 6,988 | **0,473** | **16** | 1,26 |
| (`Fillet`, para comparar) | 4,002 | 2,880 | 18 | 1,30 |

⭐ **`2,7×` menos degrau de curvatura que o `Organic` de hoje e `6,1×` menos que o `Fillet`, com
`4` nós A MENOS.** É a junta com melhor comportamento de brilho de toda a tabela.

⚠️ **A calibração muda** (o `k` cru já não é o mesmo número): o `ORGANIC_REACH = 4 − 2√2` foi
derivado para o polinomial de grau 2, e a versão cúbica precisa do seu (a linha da tabela acima
usa `k = 1,6 r` e entrega mordida `0,0943` contra `0,1036` — ainda por igualar).

---

## §5 — §5.0: a composição já exprime isto?

| família | a composição exprime? | mecanismo |
|---|---|---|
| plenitude | ⛔ **não** | não há forma de pedir um expoente hoje |
| assimétricas | ⛔ **não** | há **um** número por junta |
| `G2` | ⛔ **não** | é outra fórmula, não outro valor |
| **decorações da costura** | ⛔ **não, e é o caso mais forte** | ver abaixo |

⭐⭐⭐ **A prova de que a costura não é dado, é achado:** uma esfera unida a uma chapa tem por costura
um **círculo** que ninguém calculou. O sulco segue-o, em `30` nós e `1,95` ns:

```
  ........................#############........................
  ....................#####################....................
  ..................#########################..................
  ############.......#######################.......############
  #############.....#########################.....#############
  #############################################################
  #############.....#########################.....#############
  ############.......#######################.......############
  ..................#########################..................
  ....................#####################....................
```

Para exprimir isto por composição seria preciso **modelar o círculo** — e ele muda de raio a cada
arrasto do slider da esfera. *Uma decoração de costura é barata exactamente porque não conhece a
costura.*

---

## §6 — ⛔ Recusas MEDIDAS

| o que | porquê | número |
|---|---|---|
| **slider contínuo de plenitude** | o par `exp`/`ln` no caminho de avaliação | `36,54` contra `1,46` ns — **`25×`** |
| **plenitude `p < 1`** (o cordão pela norma) | `‖∇f‖` ilimitado na tangência | `8,34` a `p = 0,5`; `50,48` a `p = 0,25` — e o passo é do DOCUMENTO |
| **cónica com recorte de caixa inteira** | descola a junta da parede | forma partida a `‖∇f‖ = 1,0000` |
| **gravação em V com o `√½` publicado** | a normalização é a de 90° | `‖∇f‖ = 1,3066` (a mesma cura que o chanfro já tem) |

⭐ **E o cordão convexo que a norma-`p` não consegue, o TUBO entrega de graça:** mordida `0,1750` a
`‖∇f‖ = 1,0000` e `13` nós, contra `0,2652` a `‖∇f‖ = 8,34`. *Duas construções para o mesmo look, e
uma delas é a mais barata da tabela.*

---

## §7 — A escada de recomendação (ordem por valor/preço medido)

1. ⭐⭐⭐ **As decorações da costura** (`sulco`, `friso`, `cordão`) — o maior salto de vocabulário por
   `13`–`14` nós e `‖∇f‖ = 1,0000`. Nada no app faz isto hoje, e nenhuma composição o alcança.
2. ⭐⭐⭐ **O eixo da plenitude, por CHIPS** (`p ∈ {1, 2, 4, 8, 16}` + a quina) — contém os dois
   caracteres que já existem, custa `1,34`–`1,90` ns, `‖∇f‖ = 1,0000` a partir de `p = 2`.
3. ⭐⭐ **O chanfro de dois recuos** — `14` nós, gradiente intacto, resolve um pedido clássico de CAD.
4. ⭐⭐ **O `Organic` cúbico (`G2`)** — mais liso e mais barato que o de hoje; falta calibrar o `k`.
5. ⭐ **A escada** (`n` degraus) — `18` nós, `‖∇f‖ = 1,0000`, e é um *look* inteiro (maquinado).
6. ⏸️ **O filete elíptico** — funciona (`0,9981`), custa `24` nós; espera um pedido.
7. ⛔ **A cónica contínua** — o recorte não tem saída boa medida; a plenitude por chips dá o mesmo
   eixo com o campo válido.

---

## §8 — Aberto

- A calibração do `k` do cúbico (`G2`) contra a régua da mordida, como o `ORGANIC_REACH` foi feita.
- O `‖∇f‖ = 1,4142` da plenitude a `p = 1`: é o **mesmo balde** que o `Exact` já paga, mas o
  `Chamfer` de hoje paga `1,0000` — falta a normalização geral para `p ∈ (1, 2)`.
- ⛔ **A junta n-ária continua sem ângulo** (`SEM_ANGULO_FICA_N_ARIO`): tudo o que está medido aqui é
  do par, sobre faces **ortogonais**. Numa ponta de estrela (`α = 19,2°`) os números mudam, e a
  W144 já mostrou que a diferença é de `5,67×`.
- A gravação em V precisa da mesma cura de recuo/normalização que a W111 deu ao chanfro.
