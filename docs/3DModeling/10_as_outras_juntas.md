# 10 — As outras juntas: o que existe além de Fillet, Chamfer e Organic

> **Pedido do Enio, 2026-09-09:** *«Além das junções de tipo Organic, Fillet e Chamfer, faça pesquisa
> de outros tipos funcionais, de boa aparência e de boa performance e me relate quais existem»*

Instrumento: [`probe_the_other_junctions.rs`](../../crates/ph2d-field-eval/tests/it/probe_the_other_junctions.rs).

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

## §8 — W145: o que a pesquisa virou PRODUTO (ordem do Enio, 09/09: *«vamos lá»*)

A fileira de caracteres passa de **3 para 7** chips, e ela é derivada do `Character::ALL` — logo
aparece na UI sem uma linha de painel. `FIELD_DOC_VERSION` **21 -> 22**; ⭐ **o `PROJECT_SCHEMA` NÃO
sobe**, e a razão está registada na escada dele: as variantes entram no **fim** do `Blend`, logo
nenhum índice do `postcard` se move e todo `FieldVerb` já gravado lê-se ao bit.

| chip | o que é | segundo número |
|---|---|---|
| **Soft** | a transição lisa `G2` — o `Organic` um grau acima | — |
| **Bead** | o cordão sobre a costura | — |
| **Groove** | o sulco (linha de painel) | *Seam Width* |
| **Ridge** | o friso | *Seam Width* |
| (**Chamfer**) | ganhou *Chamfer Bias* — o corte desigual | *Chamfer Bias* |

### §8.1 — As três decisões que a MEDIÇÃO tomou, contra o que eu tinha prometido

1. ⛔ **A «fileira do cheio» colapsou num chip só.** A lei desta casa manda todos os caracteres
   entregarem a **mesma mordida**; calibrada assim, a norma-`p` a `p = 4` entrega **a mesma peça**
   que o polinómio cúbico — recuo `0,3812` contra `0,3907`, `2,4 %`. *Dois chips para um look é
   ruído numa fileira*, e o cúbico é o mais barato dos dois (`16` nós contra `22`).
2. ⛔ **A gravação em V ficou de fora**: `‖∇f‖ = 1,6591` a 30°, **acima** do `√2` que o balde da
   marcha paga. Ela precisa da mesma cura de recuo/normalização que a W111 deu ao chanfro.
3. ⭐ **O chanfro desigual não ganhou chip.** A pergunta *«que forma tem esta junta?»* tem uma
   resposta — «um corte reto» —, e o desequilíbrio é um **número** dela. `bias = 1,0` volta a ser um
   `Chamfer` (byte a byte), e há gate a provar que as duas leis dão o mesmo campo com diferença
   **`0,000e0`**.

### §8.2 — O `SOFT_REACH` é ANALÍTICO, e os dois números são a mesma lei

O polinómio de grau `n` desce `k/(2n − 2)` onde as duas superfícies estão à distância `d`; igualar
isso à mordida do filete (`d/√2`) dá `k = 2(n − 1)(1 − 1/√2)·d`:

| grau | dip | alcance cru |
|---|---|---|
| 2 (`Organic`) | `k/4` | `4 − 2√2` |
| 3 (`Soft`) | `k/6` | **`6 − 3√2`** |

⚠️ **Previsto e depois medido:** a sonda correu o cúbico a `k = 1,6 r` e leu mordida `0,0943`; a
forma fechada dá `0,09428`. O gate `the_four_characters_measure_the_same_radius` prende a constante
pelo caminho do produto.

### §8.3 — O balde da marcha, medido pelo caminho do PRODUTO

`the_seam_characters_stay_inside_the_march_bucket`, sobre um `FieldDoc` real, de 30° a 150°:

| carácter | 30° | 60° | 90° | 120° | 150° |
|---|---:|---:|---:|---:|---:|
| `Soft` | 0,9937 | 1,0000 | 1,0000 | 1,0000 | 1,0000 |
| `Bead` | **1,3660** | 1,2247 | 1,0000 | 1,2247 | **1,3660** |
| `Groove` / `Ridge` | 1,0000 | 1,0000 | 1,0000 | 1,2247 | **1,3660** |
| `Bevel` | 1,2638 | 1,1597 | 1,0000 | 1,0000 | 1,0000 |

⇒ todas **dentro** do `√2` que o `Exact` já paga: nenhuma marcha nova, nenhum passo perdido. O
`Soft` entra no balde do `Organic` por não inflar em ângulo nenhum.

### §8.4 — ⛔⛔ O DUAL DE UM SULCO É UM FRISO, e por isso as decorações NÃO passam por De Morgan

Esta crate combina por De Morgan de propósito: *«uma fórmula a mais seria a segunda resposta à mesma
pergunta»*. ⚠️ **Para as três decorações, honrar essa regra produz o defeito que ela existe para
evitar.** `¬groove(¬a, ¬b)` dá, termo a termo, a fórmula da nervura com `d = max(a,b)` — a conta está
certa e o **nome** fica errado: um chip `Groove` numa **subtração** levantaria uma nervura à volta do
furo. *Um controlo que faz o contrário do que o rótulo diz.*

⭐⭐ **E a cura não é uma segunda fórmula: é um ARGUMENTO.** As três passam a receber a superfície
(`min(a,b)` na união, `max(a,b)` na intersecção) em vez de a deduzir — uma função, um parâmetro.
*A lei de De Morgan existe para que «arredondar» signifique o mesmo nas três operações; aqui ela
contrariava esse propósito, porque a grandeza escolhida é uma **feição** e não um arredondamento.*

Medido (`a_groove_carves_and_a_ridge_lifts_in_all_three_operations`, células que mudam de lado):

| operação | sulco | friso | cordão |
|---|---|---|---|
| União | **−8 868 / +0** | −0 / **+26 940** | −0 / **+10 768** |
| Intersecção | **−2 592 / +0** | −0 / **+20 800** | −0 / **+17 248** |
| Subtração | **−3 240 / +0** | −0 / **+11 532** | −0 / **+16 412** |

⇒ cada coluna é **de um lado só**: o sulco nunca põe, o friso e o cordão nunca tiram.

### §8.5 — O que a construção apanhou e que a pesquisa não tinha visto

- ⛔⛔ **O `Chamfer` NUNCA esteve no corpus da paridade numérica** (`the_numeric_law_is_the_same_law_as_the_tree`)
  — ele existe desde a W99 e a lei em `f32` dele passou um bloco inteiro **sem juiz**. *Um corpus
  escrito à mão envelhece com cada variante nova, e não há gate que o diga.* Hoje a lista tem as
  nove misturas.
- ⛔⛔ **A fileira do CARÁCTER partilhava o `MAX_MODES` com a dos modificadores e não tinha censo.**
  O gate de 2026-08-30 cobria só o `UnaryKind`; a fileira de caracteres passou de `3` para `7` e a
  folga que a salvou foi a subida de teto que **outra** família pagou. ⇒ `the_panel_has_a_slot_for_every_character`.
- ⚠️ **A minha régua do recuo mediu os EIXOS numa bancada que põe as paredes a `±45°`** e leu
  `1,0000` contra `0,0000`. *A bancada de todo o resto deste módulo é ortogonal, e eu escrevi a
  régua como se esta também fosse.* Curada, e com **controlo**: a mesma régua sobre o chanfro
  simétrico tem de ler `1,000`.
- ⭐ **Um `match` que reconstrói um `Op` estava escrito em dois sítios e eu ia escrever o terceiro**
  — virou a porta `Op::with_blend`.

### §8.6 — Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=32 cargo run -p ph2d-host-desktop --release
```

Seis salientes sobre chapa, a mesma geometria nos seis: **Fillet · Soft · Bead · Groove · Ridge ·
Chamfer desigual**. ⚠️ A costura de um saliente sobre uma chapa é um **anel fechado** — duas caixas
a cruzar dariam uma costura recta, e uma decoração recta lê-se como um bisel; *a metade que
interessa é a costura VIRAR*.

---

## §9 — ⛔⛔⛔ A AUDITORIA das juntas novas (report do Enio, 09/09: *«groove não parece correto»*)

Três fotos: um saliente sobre uma chapa saía com a **base esvaziada** e o bloco **solto**.

### §9.1 — O que a bancada da W145 não podia ver

Tudo o que autorizou aquela wave foi medido num **canto de 90° feito de dois SEMIESPAÇOS**. Ali o
conjunto `{a = b}` — onde as duas peças estão à mesma distância — é um **plano** que toca a
superfície numa **recta**, e uma decoração sobre ele lê-se como uma decoração sobre a costura.

⚠️ **Numa peça a sério isso é falso.** Debaixo de um saliente as duas faces são quase coincidentes e
`a ≈ b` sobre uma **REGIÃO inteira**. Uma decoração que ACRESCENTA não se nota (pôr matéria dentro de
matéria é invisível); uma que **ESCAVA** come a região toda.

⭐⭐⭐ **E os VALORES ali são indistinguíveis dos do vinco verdadeiro:** no vinco (`a = 0, b = 0`) e a
meio da solda (`a = b = −t`) os dois campos leem o mesmo par de números — o que difere são os
**gradientes**. ⇒ *nenhuma fórmula escrita só em `a` e `b` separa os dois casos.* É uma demonstração,
não um palpite, e ela fecha uma família inteira de curas antes de alguém as construir.

### §9.2 — A régua que faltava, e o que ela mediu

⛔ **Nenhuma régua deste repositório via o defeito**: o campo continua válido, o `‖∇f‖` continua no
balde, o volume muda de forma plausível e o chip fica igual. O que muda é a **contagem de componentes
ligadas** — a mesma lição que a «almofada» do remalhador de quads já tinha cobrado.

| junta | peças (antes) | peças (depois) | % da chapa tocada |
|---|---:|---:|---:|
| Fillet · Soft · **Bead** · Ridge · Bevel | 1 | 1 | 0,0 % |
| **Groove** | ⛔ **2** | **1** | 11,8 % → 12,0 % |

⭐ **O `Bead` foi o único que já estava certo, e a razão é o mecanismo:** ele localiza-se por
`‖(a,b)‖` — a distância ao **vinco** — e não por `|a − b|`, que é o conjunto medial.

### §9.3 — A cura: o sulco é o DUAL EXACTO do cordão, mais uma guarda

```
cordão  =  min(d,  ‖(a,b)‖ − r)          acrescenta um tubo à volta do vinco
sulco   =  max(d,  min(r − ‖(a,b)‖,  max(a,b)))   remove o mesmo tubo
```

⭐⭐ **A guarda `max(a, b) > 0` pergunta se o `d` está a MENTIR.** Numa união, `min(a, b)` não é a
distância à fronteira dentro da sobreposição — ali ele devolve a distância a uma face **interior**.
O termo diz *«este ponto não está dentro das duas»*, que é o mesmo que *«aqui o `d` diz a verdade»*.
⇒ **um sulco só corta onde pelo menos uma das duas peças não está**, e a solda fica intacta por
construção.

⛔ **O segundo número do sulco SAIU.** Um tubo à volta de uma curva tem **um** raio; a
profundidade-e-largura da 1.ª versão descrevia uma caixa sobre o conjunto medial. *O número que saiu
nunca descreveu nada que existisse.*

### §9.4 — Duas fronteiras DECLARADAS, com o número

1. **A penetração é load-bearing.** Com as duas faces exactamente coincidentes (um saliente pousado
   sem penetrar) a sobreposição tem medida zero, a guarda não tem onde morder, e a união só está
   ligada por uma superfície. *A robustez cresce com a penetração* — a cena de smoke passou de `0,04`
   para `0,09` num prato de `0,18`, e foi essa mudança que levou o sulco de `3` peças para `1`.
2. **Um raio de canal comparável à espessura local deixa uma CAVIDADE FECHADA** onde duas faces sem
   relação (a base do saliente e o fundo da chapa) passam a menos de `r` uma da outra. Ela é interna
   (invisível ao traçado, e a peça continua ligada — medido), e é o mesmo mecanismo do §9.1: `‖(a,b)‖`
   é um minorante da distância ao vinco, e fica frouxo quando as duas superfícies são **paralelas**.
   ⇒ *não faça o sulco maior do que a espessura da peça.*

### §9.5 — E as outras quatro passaram

`Soft`, `Bead`, `Ridge` e `Bevel` deixam a peça em **1** componente e não tocam o interior da chapa
(`0,0 %`). ⭐ O `Ridge` e o `Bead` escrevem sobre o mesmo conjunto medial do sulco e **não** sofrem
com ele, pela razão do §9.1: *acrescentar matéria dentro de matéria é invisível.* A régua que apanha
um é cega ao outro — e é por isso que a lista de decorações não podia ser tratada como uma família só.

---

## §10 — Aberto

- ⏳ **As juntas novas valem entre FORMAS, não entre CÓPIAS de uma repetição.** A costura de uma
  `Array` ou de uma `Radial` passa pelo [`ph2d_field::Joint`] (`chamfer` + `fillet`), que é outro
  tipo e outro caminho ([`ops_joint`](../../crates/ph2d-field-eval/src/ops_joint.rs)). Levá-las lá é
  uma wave própria, e a pergunta que a abre é se um `Joint` deve virar um `Blend` ou ganhar um
  carácter ao lado dos dois números.
- ⏳ **A gravação em V** espera a cura de recuo/normalização da W111 (mede `1,6591`).
- ⏳ **A escada e a colunata** estão medidas e não construídas (`18` e `32` nós, `‖∇f‖ = 1,0000`) —
  ficaram fora da ordem de 09/09 por não estarem na recomendação que ela aprovou.

- A calibração do `k` do cúbico (`G2`) contra a régua da mordida, como o `ORGANIC_REACH` foi feita.
- O `‖∇f‖ = 1,4142` da plenitude a `p = 1`: é o **mesmo balde** que o `Exact` já paga, mas o
  `Chamfer` de hoje paga `1,0000` — falta a normalização geral para `p ∈ (1, 2)`.
- ⛔ **A junta n-ária continua sem ângulo** (`SEM_ANGULO_FICA_N_ARIO`): tudo o que está medido aqui é
  do par, sobre faces **ortogonais**. Numa ponta de estrela (`α = 19,2°`) os números mudam, e a
  W144 já mostrou que a diferença é de `5,67×`.
- A gravação em V precisa da mesma cura de recuo/normalização que a W111 deu ao chanfro.
