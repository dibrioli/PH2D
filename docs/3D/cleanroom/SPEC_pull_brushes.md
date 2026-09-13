# SPEC — os gestos de PUXAR que faltam ao módulo 3D/Sculpt

```
Alvo: um editor 3D livre de referência, versão 5.2.0 (fonte) / 5.2.1 LTS (binário-oráculo)
Licença: GPL-2.0-or-later (lida no ficheiro do checkout)   ·   Degrau: T2
Ledger: aberto em docs/3D/cleanroom/LEDGER_blender-pull.md, 2026-09-13
Patente (§8.1): buscado em 2026-09-13 — ⛔ UMA VIVA, e ela EXCLUI uma das opções do âmbito (§9.1)
Filtragem §4.3: executada em 2026-09-13   ·   Sweep: verde em 2026-09-13
Auditoria §4.2 (R-pré): auditada contra §4.2 por R-pré em 2026-09-13 — ✅ VERDE
  (subagente `agent-ae2f81f455d29b69f`; sweep verde sobre esta espec e as 68 fixtures,
   vassoura de 298 entradas. O §9.1 foi corrigido pelo R-pré com facto de patente — ver lá.)
Mapa de leitura da literatura: nenhum paper é necessário para o que fica dentro do âmbito;
  a única lei desta família que tem literatura própria é a que a patente EXCLUI (§9.1)
Denylist de URLs: o repositório do alvo e os seus espelhos; qualquer busca de código
  («code search») pelo nome interno de qualquer símbolo desta família
"Este documento descreve comportamento; não contém expressão do alvo."
```

---

## §0 — O que isto é, em duas linhas

Faltam ao nosso catálogo **dois gestos de puxar** — que aqui chamo **POLEGAR** e **EMPURRÃO**
(os nomes públicos no alvo são `THUMB` e `NUDGE`) — e **quatro opções públicas** sobre gestos que
já temos. Esta espec dá a lei completa de cada um, com a proveniência de cada número.

⭐⭐ **O achado que organiza o documento inteiro: os dois gestos que faltam calculam EXACTAMENTE o
mesmo deslocamento.** A diferença entre eles não está na fórmula — está em **de que pose a pegada é
medida**. Um congela tudo no pen-down; o outro anda com o cursor. Tudo o resto é partilhado.

⚠️ **Duas coisas do âmbito saem, e as duas com número:** a variante *elástica* do gancho sai por
**patente viva** (§9.1) e o *alvo de deformação = simulação* sai por **medição** (§9.2).

---

## §1 — O referencial, e como um gesto vira um vector

Tudo em **espaço de objecto**; as fixtures também. A câmara das medições é **ortográfica**.

1. No **pen-down** grava-se uma **âncora**: o ponto da superfície sob o cursor. (Com a opção do
   §7.2 ligada, a âncora passa a ser a posição de um **vértice**.)
2. A cada evento, a posição do cursor no ecrã é **desprojectada no plano paralelo ao ecrã que passa
   pela âncora** — não na superfície. ⚠️ **É isto que faz o gesto continuar a funcionar quando o
   cursor sai da malha**, e é a razão de a lei ser independente da forma sob o rato.
   Chame-se `g_k` a esse ponto no evento `k`.
3. O **deslocamento do gesto** `Δ` sai de `g_k` de **uma de duas maneiras**, e é essa escolha que
   parte a família em duas (§4).

⚠️ **Num alcance em TUBO** (o alcance «2D», que ignora a profundidade) `Δ` é ainda projectado no
plano do ecrã antes de ser usado. Nas fixtures o alcance é sempre em **esfera**.

---

## §2 — O esqueleto partilhado: como nasce o PESO de cada vértice

Todo gesto desta família calcula, por vértice, um **peso** `w ∈ [0,1]`, e depois desloca o vértice
por `w × offset`. O peso nasce desta cadeia, **nesta ordem** — e a ordem é load-bearing, porque
duas das etapas não comutam (a dureza reescreve a distância *antes* de a curva a ler):

| # | etapa | o que faz |
|---|---|---|
| 1 | visibilidade e máscara | `w = 1 − mascara[v]`; vértice escondido ⇒ `w = 0` |
| 2 | recorte da vista | fora dos planos de recorte do 3D ⇒ `w = 0` (⚠️ testado na posição **espelhada** da passagem de simetria corrente) |
| 3 | face frontal *(opcional)* | `w *= max(dot(normal_da_vista, n_v), 0)` — ⚠️ é um **factor contínuo**, não um corte binário |
| 4 | distância ao cursor | `d = |p_v − centro|` (alcance em esfera) ou a distância à **recta da vista** que passa pelo centro (alcance em tubo) |
| 5 | corte pelo raio | `d >= R ⇒ w = 0` — ⚠️ **`>=`**, então um vértice exactamente no raio morre |
| 6 | dureza | reescreve `d` (ver §2.1) |
| 7 | curva do alcance | `w *= curva(d / R)` |
| 8 | auto-máscara | `w *= o que a auto-máscara disser` |
| 9 | textura | `w *= valor da textura` (só se houver textura de máscara) |

⛔ **A FORÇA NÃO ENTRA NESTA CADEIA** para os dois gestos que faltam — ela entra no **offset**
(§3). Multiplicar a força aqui *e* lá daria o quadrado errado.

### §2.1 — A dureza, exactamente

Com dureza `h` e raio `R`, seja `t = h·R`:

- `h = 0` ⇒ a distância não muda;
- `h = 1` ⇒ `d < t ? 0 : R` (um degrau: ou cheio, ou nada);
- caso geral ⇒ `d < t ? 0 : ((d/R − h) / (1 − h)) · R`.

Em todas as fixtures `h = 0`.

### §2.2 — A curva do alcance

Com `x = d/R` e `f = 1 − x` (e `w = 0` para `x >= 1`):

| curva | factor |
|---|---|
| **suave** *(a das fixtures)* | `3f² − 2f³` |
| afiada | `f²` |
| suavíssima | `f³·(f·(6f − 15) + 10)` |

⚠️ As restantes formas existem no alvo e **não foram exercitadas** — nenhuma fixture as usa, logo
esta espec não as afirma.

---

## §3 — A FORÇA, e o achado que muda todos os números

⭐⭐⭐ **O slider de força entra ao QUADRADO nestes dois gestos.** A força efectiva é

```
forca_efectiva = s² × pressao × plumagem_de_simetria
```

onde `s` é o valor do slider. ⚠️ **Não é uma escolha minha — é medido:** com `s = 0,5` sobre a
mesma malha e o mesmo caminho, o pico do deslocamento cai para **exactamente `1/4`**
(`0,600000 → 0,150000`, fixtures `polegar_plano_origem` e `polegar_plano_forca05`, razão medida
`0,25` ao bit). O alvo declara por escrito, no sítio onde o quadrado é feito, que a intenção é dar
mais sensibilidade à parte baixa do slider.

⚠️⚠️ **E ele NÃO é quadrático em toda a família — esta é a armadilha da tabela:**

| gesto | força efectiva |
|---|---|
| **POLEGAR** | `s² × pressao × plumagem` |
| **EMPURRÃO** | `s² × pressao × sobreposicao′ × plumagem` |
| agarrar (o nosso `Move`) | **`s`** `× plumagem` — **linear** |
| gancho (o nosso `SnakeHook`) | **`s`** `× plumagem` — **linear** |
| deslize (o nosso `SlideRelax`) | `2 × s² × pressao × sobreposicao × plumagem` |

⇒ *o mesmo slider a `0,5` dá metade num gesto e um quarto no outro*, e um implementador que
generalize de um para o outro erra por `2×`.

- **pressão**: `1` quando a força não segue a pressão da caneta (o caso de todas as fixtures);
  senão, a pressão passada por uma curva própria.
- **plumagem de simetria**: `1` salvo se a preferência de *plumagem* estiver ligada — aí é
  `1 / (soma das sobreposições das passagens de simetria)`. **`1` em todas as fixtures.**
- **sobreposição**: ver §8.4 — ⚠️ vale **`1` em todas as fixtures**, e isso **não** é o que o
  produto interactivo faz.
- ⛔ **O sinal de inversão (o `Ctrl`) NÃO entra** em nenhum destes dois gestos: quem dá o sentido é
  a direcção do gesto. (Entra noutros gestos da casa — não generalize.)

---

## §4 — As duas famílias do deslocamento: o eixo que separa tudo

| | **ANCORADA** (o POLEGAR) | **QUE VIAJA** (o EMPURRÃO) |
|---|---|---|
| `Δ` do evento `k` | **acumulado**: `Δ += g_k − g_{k−1}` ⇒ o **total** desde o pen-down | **incremento**: `Δ = g_k − g_{k−1}` |
| centro da pegada | **congelado na âncora** | o ponto sob o cursor, a cada evento |
| pose que mede o peso (§2) | a do **pen-down** | a **viva** |
| normal da área (§5.2) | **congelada** no pen-down | **re-amostrada** a cada evento |
| a malha antes de cada evento | ⭐ **reposta na pose do pen-down** | não é reposta |
| depende de quantos eventos? | ⛔ **NÃO** | ⭐ **SIM** |

⭐⭐ **A última linha é a consequência da penúltima, e é a propriedade mais importante desta espec.**
Porque a malha é reposta antes de cada evento e `Δ` é o total, o gesto ancorado é **idempotente**:
só o último evento conta. Medido: `polegar_plano_origem` (12 eventos) e `polegar_plano_passos24`
(24 eventos) dão `max_deslocamento` **idêntico**, `0,600000`. O gesto que viaja soma contribuições e
os mesmos dois números dão `0,599054` contra `0,595226` — **diferentes**.

⚠️ **Na nossa casa isto já tem vocabulário:** a família ancorada é o `Grip::Hold` e a que viaja é o
`Grip::Hook` (ver `crates/ph2d-sculpt3d/src/grip.rs`, que já escreve exactamente esta distinção —
«puxar de volta devolve o barro» contra «a matéria foi transportada»). ⇒ **não se inventa modelo
novo**: o POLEGAR é um `Hold` e o EMPURRÃO é um `Hook`.

⚠️ A reposição-antes-de-cada-evento vale para a família ancorada **e** para qualquer gesto cujo
método de traço seja *ancorado* ou *ponto-arrastado*. Fora disso não há reposição.

---

## §5 — O POLEGAR (o gesto ancorado que falta)

### §5.1 — A lei, inteira

Por evento (e, pela §4, só o último importa):

```
n      = normal da área, congelada no pen-down            (§5.2)
Δ      = g_k − ancora            (o total do gesto)
offset = tangencial(Δ, n) × forca_efectiva
para cada vértice v da pegada:
    w        = peso da cadeia da §2, medido na pose do PEN-DOWN
    p_v     += offset × w
```

onde **`tangencial(Δ, n) = Δ − n·(n·Δ)`** — a componente de `Δ` no plano perpendicular a `n`
(com `|n| = 1`; a forma equivalente que o alvo usa é um duplo produto vectorial).

⭐ **É isso que o separa do agarrar:** o agarrar leva o gesto **inteiro** (podendo inclinar-se para
a normal por um peso próprio); o polegar leva **só a parte tangencial**. Por isso ele *espalma* em
vez de *levantar* — um polegar a alisar barro, que é o que o nome público diz.

### §5.2 — A normal da área

Amostrada **uma vez**, no pen-down (a menos que o método do traço seja *ancorado*, caso em que é
re-amostrada):

```
r = R × fracao_normal                       (fracao_normal = 0,5 nas fixtures; 0,3 numa delas)
para cada vértice com |p_v − centro| <= r, na pose do PEN-DOWN:
    peso = clamp(3f² − 2f³, 0, 1)  com f = 1 − |p_v − centro|/r
    balde = (dot(normal_da_vista, n_v) <= 0) ? 1 : 0
    soma[balde] += n_v × peso
n = normalize(soma[0]) se soma[0] ≠ 0, senão normalize(soma[1])
```

⚠️ **Os dois baldes não são um detalhe:** eles separam os vértices virados para a câmara dos
virados ao contrário, e o balde **virado para a câmara ganha sempre**. Sem isso, numa peça fina os
dois lados cancelam-se e a normal colapsa.

⚠️ A normal por vértice é a **ponderada pelo ângulo** do canto (a convenção que a nossa
`ph2d-mesh` já usa).

⚠️ Fora do modo de escultura a fracção não se aplica; aqui aplica-se sempre.

### §5.3 — Medições

⭐⭐⭐ **Os quatorze vectores do polegar reproduzem a lei acima a `≤ 2,4e-07`** — ruído de `f32`,
sobre deslocamentos de ordem `0,04` a `0,60`:

| fixture | o que prende | resíduo |
|---|---|---|
| `polegar_plano_origem` | a lei base, plano | `2,8e-07` |
| `polegar_plano_forca05` | ⭐ a força ao **quadrado** | `6,9e-08` |
| `polegar_plano_passos24` | ⭐ independência da amostragem | `2,8e-07` |
| `polegar_plano_diagonal` | direcção obliqua | `1,7e-07` |
| `polegar_plano_origem_k02…k11` | truncagens: só o **total** conta | `≤ 4,3e-07` |
| `polegar_plano_espelhox` | ⭐ simetria (§8.1) | `1,6e-07` |
| `polegar_esfera_topo` | a normal da área numa superfície curva | `2,0e-07` |
| `polegar_esfera_topo_raionormal03` | ⭐ a **fracção do raio da normal** | `1,8e-07` |
| `polegar_esfera_frente` | outra vista | `1,2e-07` |
| `polegar_esfera_topo_k02…k08` | truncagem sobre superfície curva | `≤ 2,4e-07` |

⚠️⚠️ **Três destes resíduos estavam em `0,03`–`0,15` numa primeira análise, e o defeito era da
RÉGUA, não do alvo** — o script anterior fixava o raio da amostragem da normal num literal em vez
de o derivar da fracção, e somava as passagens de simetria com o centro errado. *Uma discordância
com o oráculo é uma hipótese sobre o alvo **ou** sobre a nossa régua, e a régua é a mais barata de
conferir primeiro.*

### §5.4 — ⭐⭐⭐ A direcção NÃO muda com o número de eventos (emenda medida, 2026-09-13)

Perguntado se algo muda no alvo entre o 8.º e o 10.º evento — a normal re-amostrada, o centro a
sair da âncora, um segundo termo acima de um limiar, ou um `Δ` acumulado em planos diferentes —,
a resposta é **não, nada muda**, e está medida sobre **nove** comprimentos de traço na mesma
esfera (2, 3, 4, 5, 6, 7, 8, 9 e 10 eventos; vista de topo, raio `0,35`, força `1`, fracção da
normal `0,5`):

| grandeza | valor | espalhamento nas nove |
|---|---|---|
| direcção do deslocamento | `(0,957277 · 0 · −0,289172)` | **idêntica às 6 casas decimais** |
| `pico / |Δ|` | **`0,9549005`** | **`5,7e-06`** |
| `|offset| / |Δ|` | `0,957277082` | `= √(1 − (n̂₀·Δ̂)²)`, exacto |
| `pico / |offset|` | `0,9975179` | o peso da §2 no vértice mais perto da âncora |
| normal da área congelada `n₀` | `(0,289172 · 0 · 0,957277)` | uma só, do pen-down |

⇒ **as quatro hipóteses estão refutadas**, cada uma por medição:

1. *a normal é re-amostrada?* ⛔ Não — se fosse, a direcção rodaria; ela é a mesma nas nove.
2. *o centro sai da âncora?* ⛔ Não — o número de vértices movidos é **`492` em todas** as que
   passam do 3.º evento, e a pegada não anda.
3. *entra um segundo termo acima de um limiar?* ⛔ Não — `pico/|Δ|` é **constante** de `|Δ| = 0,044`
   a `|Δ| = 0,400`, uma faixa de `9×`; um termo extra partiria a proporcionalidade.
4. *o alvo acumula incrementos projectados em planos diferentes?* ⛔ Não. Ele **acumula
   incrementos** (isso é verdade, §4), mas todos no **mesmo** plano — o plano paralelo ao ecrã que
   passa pela **âncora**, que não se mexe durante o traço (§1.2). ⇒ a soma telescopa e é
   exactamente `g_N − âncora`. *Acumular incrementos e usar o total são a mesma coisa aqui, e é por
   isso que a §4 pode dizer «o total» sem mentir.*

⭐⭐ **E a direcção é ortogonal a `n₀` mas NÃO está no plano do ecrã:** `dot(direcção, n₀) = 3,9e-07`
e `dot(direcção, eixo da vista) = −0,289172`. *O gesto lê-se no plano do ecrã; o deslocamento vive
no plano tangente de `n₀`* — e as duas coisas são diferentes numa superfície curva. Quem projectar
o **offset** de volta ao plano do ecrã (em vez de projectar só o **gesto**) ganha um erro que cresce
com a curvatura sob o traço.

⚠️⚠️ **A armadilha que explica uma comparação `|pico|` contra `|pico|`:** o vértice do pico **não
está na âncora**, logo o peso dele é `0,9975179`, não `1`. Comparar o nosso `|offset|` com o `|pico|`
do alvo mistura a lei com o alcance e devolve `+0,25 %` de erro que não existe. ⇒ compare
**campo contra campo**, ou divida o pico por `|Δ|` e compare o quociente — que é a constante acima.

⭐ **Quatro invariantes para bissectar uma implementação que discorde**, do mais barato ao mais caro:

1. `pico / |Δ|` tem de ser **constante** ao variar o comprimento do traço (aqui `0,9549005 ± 3e-06`).
   *Se cresce com o comprimento, o erro está na magnitude do offset, não na pegada.*
2. a **direcção** tem de ser a mesma em todos os comprimentos.
3. `dot(direcção, n₀) = 0`.
4. o campo é **unidireccional**: acima de `10 %` do pico o cosseno com a direcção do pico é
   `1,000000000` nos `307` vértices; acima de `0,1 %` é `≥ 0,99999992`. ⛔ Abaixo disso é ruído de
   `f32` (vértices a `6e-08`, onde a direcção não significa nada) — **não meça espalhamento sem
   um piso de magnitude**.

**Fixtures:** `polegar_esfera_topo_k02/k04/k06/k08`, `polegar_esfera_passo03/05/07/09` e
`polegar_esfera_topo` — nove comprimentos do **mesmo** traço.

---

## §6 — O EMPURRÃO (o gesto que viaja)

### §6.1 — A lei, inteira

```
por evento k = 1..N-1:
    n       = normal da área, re-amostrada AGORA no ponto sob o cursor   (§5.2)
    Δ       = g_k − g_{k−1}                     (o INCREMENTO)
    offset  = tangencial(Δ, n) × forca_efectiva
    para cada vértice v da pegada (centrada no ponto sob o cursor):
        w    = peso da cadeia da §2, medido na pose VIVA
        p_v += offset × w
```

⇒ **a mesma fórmula do §5.1**, com três trocas: incremento em vez de total, pegada que anda,
pose viva. Nada mais.

### §6.2 — O que isto significa para o artista

O polegar **espalma um sítio**; o empurrão **varre matéria ao longo do traço**. Voltar pelo mesmo
caminho **não** devolve o barro (`empurrao_plano_ida_volta`: o pico fica em `0,259160` depois da
ida e da volta, não em zero) — é uma integral de linha, exactamente como o nosso `Grip::Hook`.

### §6.3 — Medições, e o que fica ABERTO

| fixture | resíduo | veredito |
|---|---|---|
| `empurrao_plano_origem` | `4,2e-07` | ⭐ **confirmado** |
| `empurrao_plano_forca05` | `1,3e-07` | ⭐ a força ao quadrado |
| `empurrao_plano_passos24` | `3,0e-07` | ⭐ a dependência da amostragem |
| `empurrao_plano_parado` | `1,3e-07` | cursor parado |
| `empurrao_plano_ida_volta` | `2,8e-07` | transporte de matéria |
| `empurrao_plano_origem_k02…k11` | `≤ 5,8e-07` | truncagens |
| **`empurrao_esfera_*`** | **`3,3e-02` a `1,3e-01` relativo** | ⏳ **ABERTO** |

⛔⛔ **O caso curvo NÃO fecha, e a causa NÃO é a lei.** Quatro hipóteses sobre a normal foram
construídas e medidas — normal da pose do pen-down · da pose viva · com a fracção do raio · com o
raio inteiro — e **as quatro dão o mesmo `~10 %`**. ⇒ o resíduo não está na normal.

⭐ **A explicação com endereço:** este gesto **exige um ponto de superfície por evento**, e esse
ponto é o **acerto do raio na superfície VIVA** — que, numa superfície que se deforma debaixo do
traço, deixa de ser função do caminho de entrada. Num plano o traço é tangencial, a superfície
continua plana, o acerto coincide com o ponto analítico, e o resíduo cai para `4e-07`. Numa esfera
não coincide. ⚠️ **O gesto ancorado é imune a isto por construção** — ele nunca volta a perguntar à
superfície.

### §6.3-bis — A decomposição POR EVENTO existe (2026-09-13), e ela move a culpa

⭐ O instrumento foi construído e corrido: **truncar o mesmo traço em `2..10` eventos** e subtrair
truncagens vizinhas. Como `P_k = P_{k−1} + offset_k · w_k`, a diferença de duas truncagens **é** o
campo do `k`-ésimo carimbo — logo o `offset` e o centro de cada evento recuperam-se **sem
qualquer acesso ao interior do alvo**. Fixtures `empurrao_esfera_passo02…passo10`.

| evento | `|offset_k|` | direcção do `offset_k` |
|---|---|---|
| 3 | `0,041407` | `(0,93609 · 0 · −0,35176)` |
| 4 | `0,040832` | `(0,92901 · 0 · −0,37005)` |
| 5 | `0,040572` | `(0,92049 · 0 · −0,39076)` |
| 6 | `0,040182` | `(0,91024 · 0 · −0,41408)` |
| 7 | `0,039289` | `(0,89698 · 0 · −0,44208)` |
| 8 | `0,038024` | `(0,88030 · 0 · −0,47443)` |
| 9 | `0,036440` | `(0,85947 · 0 · −0,51119)` |
| 10 | `0,034732` | `(0,83085 · 0 · −0,55650)` |

⭐⭐ **A direcção roda a cada evento** (`21,6°` no total) — que é o esperado numa superfície curva, e
o contrário do gesto ancorado (§5.4). O que a decomposição **exclui** é onde estávamos a procurar:

- ⛔ **NÃO é o centro.** Trilaterando o centro de cada carimbo a partir do próprio campo (invertendo
  o alcance para uma distância por vértice), ele fica a `0,001`–`0,036` do ponto analítico do
  caminho, com resíduo de ajuste `0,013`–`0,022` — ou seja, **consistente com o ponto analítico**.
  E a hipótese que este documento preferia — *o centro é o acerto do raio na superfície VIVA* —
  está **REFUTADA com número**: esse acerto explica pior que o ponto analítico em **7 de 8** eventos
  (`0,048` contra `0,036` no pior).
- ⏳ **É a NORMAL, e ela ATRASA-SE.** Convertendo cada direcção medida na normal que a explicaria,
  essa normal fica **atrás** do centro do evento, e o atraso **cresce** ao longo do traço. Duas
  amostragens candidatas foram medidas e **as duas falham, as duas com erro crescente**: a calota
  na pose de **repouso** erra `1,8° → 10,4°` (média `6,8°`), a calota na pose **viva** erra
  `0,4° → 15,1°` (média `5,8°`). *Nenhuma das duas é a lei.*

⇒ **o que fica aberto é estreito e tem os dados na mão:** qual pose/instante alimenta a normal da
área do gesto que viaja. ⛔ **Não é o centro** (medido), ⛔ não é a calota no repouso nem a calota
na pose viva (medidas). Quem pegar nisto começa das oito linhas da tabela acima, não do zero.

⚠️ **E o gesto ancorado é imune a tudo isto por construção** — ele amostra a normal uma vez e nunca
mais pergunta à superfície (§5.4). É por isso que ele fecha a `2e-07` na mesma esfera em que este
fica a `~10 %`: *não são dois níveis de qualidade da mesma lei, são duas leis com números de
perguntas diferentes.*

⚠️ **Enquanto isso não fechar, a barra de paridade do empurrão vale sobre as fixtures PLANAS**
(onde ele está confirmado ao bit) e as curvas ficam como *regressão*, não como paridade.

---

## §7 — As quatro opções públicas sobre gestos que já temos

⚠️ **Conferido no nosso repo em 2026-09-13: as quatro FALTAM.** `Verb::ALL` tem 24 verbos e
nenhum deles oferece silhueta, âncora em vértice, modos de deslize ou modos de gancho; o nosso
`SlideRelax` implementa a lei de *relaxar* e não as três direcções abaixo; e não existe no
repo nenhum conceito de *alvo da deformação*.

### §7.1 — `use_grab_silhouette` — a silhueta (sobre o nosso `Verb::Move`)

Uma etapa **extra** no fim da cadeia da §2, só para o agarrar:

```
sinal    = sign( dot(normal_do_pen_down, Δ) )
direccao = normalize(offset) × sinal
w       *= max( dot(direccao, n_v), 0 )          com n_v da pose do PEN-DOWN
```

⇒ mantém o barro cuja normal **acompanha** o puxão e apaga o que lhe dá as costas: agarra-se um
lado da silhueta sem arrastar o outro.

⛔⛔ **ARMADILHA MEDIDA, e ela é uma divisão por zero disfarçada:** quando o puxão é **exactamente
tangencial** à superfície, `dot(normal, Δ) = 0`, logo `sinal = 0`, logo `direccao = 0` e **todos os
pesos vão a zero — o gesto inteiro morre em silêncio**. Isto não é hipótese: é o que as cinco
fixtures `*_silhueta_sim` mostram (`movidos = 0`). ⚠️ **Ver a §12.1 antes de concluir seja o que
for**: há uma segunda causa possível, do harness, e as duas produzem o mesmo zero.
⇒ **a nossa implementação trata `sinal = 0` explicitamente** (a escolha honesta é *não filtrar*,
i.e. `sinal = +1`), e o gate nomeia o caso.

### §7.2 — `use_grab_active_vertex` — a âncora no vértice (sobre o `Verb::Move`)

Muda **uma coisa só**: no pen-down, a âncora deixa de ser o ponto da superfície sob o cursor e
passa a ser a **posição do vértice activo** (o vértice sob o cursor). Tudo o resto é igual.

⚠️ A posição lida é a da malha **base**, não a deformada (com uma chave de forma activa, é a dela).

⭐ **Medido, e o efeito é visível:** numa grelha grossa (`8×8`, fixtures `agarrar_grelha8_vertativo_*`)
ligar a opção leva o pico de `0,446338` para `0,500001` — ou seja, **o pico passa a ser exactamente
o deslocamento do gesto** (`0,5`), porque a âncora cai em cima de um vértice e aquele vértice
recebe peso `1`. Um vértice a mais entra na pegada (`8 → 9`).

⇒ numa malha densa a diferença tende a zero; ela existe para malhas grossas e para o artista poder
apontar a um vértice concreto.

### §7.3 — `slide_deform_type` — as três direcções do deslize (sobre o `Verb::SlideRelax`)

⚠️ **Isto NÃO é um modo do nosso relaxar — é a outra metade daquele gesto.** A nossa
`SlideRelax` faz a média do anel com a componente normal removida. Estes três escolhem uma
**direcção** e depois deixam a vizinhança decidir quanto:

```
por evento (⛔ o PRIMEIRO evento de cada passagem de simetria não faz nada):
    peso w    = cadeia da §2, medida na pose do PEN-DOWN, × forca_efectiva
    direccao d_v, por vértice, na pose VIVA:
        DRAG    → normalize(centro_agora − centro_do_evento_anterior)   (igual para todos)
        PINCH   → normalize(centro_agora − p_v)                          (para o cursor)
        EXPAND  → normalize(p_v − centro_agora)                          (para longe do cursor)
    t_v = Σ  sobre os vizinhos q de v, na pose VIVA:
              seja u = p_q − p_v ;  se dot(d_v, normalize(u)) > 0:
                  t_v += normalize(u) × dot(d_v, u)
    p_v += t_v × w
```

⭐ **A soma sobre os vizinhos é o que faz o vértice DESLIZAR e não voar:** ele só se move na
direcção de vizinhos que já estão do lado para onde se quer ir, e o quanto é a projecção da aresta.
A superfície é preservada porque o passo é sempre uma combinação de arestas existentes.

⚠️ **Os pesos medem-se na pose do PEN-DOWN e as direcções na pose VIVA** — as duas poses no mesmo
passo. Não é descuido: a pegada tem de ficar quieta enquanto os vértices se reorganizam.

⚠️ **A força efectiva leva um `× 2`** (§3), e é quadrática no slider.

⭐ Medições: as três direcções sobre o mesmo caminho reproduzem a lei acima com resíduo
`1,4e-08` a `2,7e-03` (fixtures `deslizar_plano_{arrastar,apertar,expandir}_origem`). ⚠️ Os
resíduos maiores são do lado `apertar`/`arrastar` e **não foram atribuídos** — a hipótese com
endereço é a mesma do §6.3 (o centro por evento vem do acerto na superfície viva).

⛔ **Uma aproximação nomeada que a nossa casa já declara:** o alvo impede um vértice de atravessar
a fronteira de um *conjunto de faces*; nós não temos esse conceito (decisão do dono), logo o nosso
desliza através dela. A nota já está escrita no doc-comment do nosso `SlideRelax` — **mantenha-a**.

### §7.4 — `snake_hook_deform_type` — os dois modos do gancho (sobre o `Verb::SnakeHook`)

- **`FALLOFF`** (o de omissão) — é **a lei que já temos**: peso da cadeia da §2 na pose viva,
  vezes a força efectiva (⚠️ **linear** no slider, §3), aplicado ao incremento do gesto; mais o
  aperto lateral opcional e a rotação de *rake*. ⇒ **nada a construir; só o chip que nomeia o modo.**
- **`ELASTIC`** — ⛔ **FORA DO ÂMBITO POR PATENTE VIVA.** Ver §9.1.

⇒ **o selector nasce com um item só** enquanto a patente viver. ⚠️ Um selector de um item é um
controlo morto pela régua da casa — ⇒ **não construa o selector**; documente que o gancho é
sempre o modo de alcance, e o dia em que houver um segundo modo ele nasce com dois.

### §7.5 — `deform_target = CLOTH_SIM`

⛔ **FORA DO ÂMBITO POR MEDIÇÃO.** Ver §9.2.

---

## §8 — O partilhado: simetria, máscara, undo, espaçamento, pressão

### §8.1 — Simetria

Cada passagem de simetria corre o gesto inteiro com **tudo espelhado** — o centro, o deslocamento
do gesto, a normal da área, a normal da vista. As contribuições **somam-se** na mesma pose.

⭐ **Medido e fechado:** `polegar_plano_espelhox` reproduz-se por **pura sobreposição** das duas
passagens, resíduo `1,6e-07`. ⚠️ A reposição da malha (§4) acontece **uma vez por evento**, não uma
vez por passagem — senão a segunda passagem apagaria a primeira.

⚠️ **Espelhar um vector é trocar o sinal da componente do eixo** — para um gesto ao longo de `+Y`
com espelho em `X`, as duas passagens movem no **mesmo** sentido (medido: as duas metades da
fixture dão direcção média `[0, 1, 0]` e pico idêntico `0,497099`). *Quem esperar sentidos opostos
constrói o gate ao contrário.*

### §8.2 — Máscara

A máscara entra como `1 − mascara[v]` na **primeira** etapa (§2), logo ela multiplica tudo o resto.
É o que a nossa casa já faz.

### §8.3 — Undo e a pose original

- A família ancorada exige guardar a **pose do pen-down** dos vértices tocados, e **repor** a
  malha nela antes de cada evento (§4). Na nossa casa o `base`/`pre` congelado do `Grip::Hold` já
  é exactamente isto — ⇒ **o undo continua trivial** nas duas famílias.
- ⚠️ A pegada e os pesos da família ancorada medem-se **na pose reposta**, nunca na viva.

### §8.4 — Espaçamento entre carimbos, e uma armadilha do oráculo

- ⭐ **O POLEGAR não usa espaçamento**: ele é um gesto de *agarrar* e, como todos eles, recebe
  **um carimbo por evento de entrada** — nunca um passo de arco. O mesmo vale para o agarrar e o
  gancho.
- O **EMPURRÃO** usa espaçamento: os carimbos são pousados a passo constante ao longo do caminho, e
  o passo é uma percentagem do raio. ⇒ a densidade de carimbos é do **caminho**, não do polling —
  que é a lei que a nossa casa já escreve no `walk`.
- ⛔⛔ **A sobreposição (§3) vale `1` em TODAS as fixtures, e o produto interactivo NÃO faz isso.**
  Um traço conduzido por lista de eventos (que é como o oráculo é conduzido) **nunca** calcula o
  factor de sobreposição — ele fica no valor inicial `1`. No app, com atenuação de espaçamento
  ligada e passo abaixo de `100 %`, o empurrão e o deslize levam um factor extra `(1 + sob)/2`
  com `sob < 1`. ⇒ **as magnitudes medidas do empurrão e do deslize são as de sobreposição `1`**;
  as do polegar não são afectadas (ele não lê sobreposição). *Uma fixture de magnitude é uma
  afirmação sobre a configuração que a produziu.*

### §8.5 — Pressão

Nenhuma fixture usa pressão (fica em `1`). Com a pressão ligada ela multiplica a força efectiva,
passada por uma curva própria. ⚠️ O nosso repo já mede que o backend não entrega pressão nesta
máquina (`pencil_input.rs`) — ⇒ **não é bloqueador**, é um multiplicador que nasce em `1`.

---

## §9 — O que fica FORA, e o número de cada exclusão

### §9.1 — ⛔⛔⛔ PATENTE VIVA — o modo elástico do gancho

**US 10 586 401 B2** — *Sculpting brushes based on solutions of elasticity*, Pixar.
Prioridade 2017-05-02, concedida 2020-03-10, **expira 2038-05-02** (lido em 2026-09-13).

⚠️ **São TRÊS reivindicações independentes, não uma** (conferidas pelo R-pré em 2026-09-13, no
texto concedido): **1** (método), **13** (meio legível por computador que armazena o programa) e
**20** (sistema — processador + memória com o programa). As três têm o **mesmo corpo**: *receber a
escolha de um pincel e de um tamanho de pincel · receber um movimento de um dispositivo de entrada ·
determinar a deformação de um objecto gráfico com base, pelo menos em parte, nesse pincel e tamanho,
nesse movimento, e em **uma ou mais soluções regularizadas de uma equação da elasticidade linear,
em que essas soluções incluem um valor especificado do coeficiente de Poisson** · e renderizar uma
ou mais imagens do objecto com a deformação determinada.* O modo elástico do gancho é exactamente
isso.

⚠️ **Que sejam três muda o ALCANCE, e é por isso que o número importa:** a 1 é sobre **executar** a
lei, a 20 sobre a **máquina que a corre**, e a 13 sobre **o programa distribuído** — uma delas
alcança o acto de enviar o binário, não só o de o usar.

⇒ **a opção sai da espec**; o selector do §7.4 não se constrói.

⚠️⚠️ **E há uma segunda metade que o dono tem de ver:** esta reivindicação **lê também sobre o que
a nossa casa JÁ SHIPA** — o nosso módulo tem um `kelvinlet` com aterrissagem na borda, usado pelo
modo elástico dos verbos de agarrar (`crates/ph2d-sculpt3d/src/kelvinlet.rs`, cena `=28`), que é um
**modo opcional** (o chip `L`) e **não** o caminho de omissão. Lido **elemento a elemento** pelo
R-pré, os quatro elementos do corpo estão **presentes**: o artista escolhe o verbo e o modo e tem
raio de pincel · o gesto vem do ponteiro · a deformação sai de uma solução regularizada da
elasticidade linear com um coeficiente de Poisson **declarado como constante** · e o resultado é
desenhado no ecrã.

⛔⛔ **DUAS saídas que parecem existir e NÃO existem** — as duas foram conferidas, e quem as repetir
gasta a jornada:

1. *«o nosso coeficiente é `1/2`, logo estamos fora»* — **falso**: a reivindicação **12**
   (dependente) restringe-se ao caso em que *o valor não é infinito nem um meio*, logo a
   independente de que ela depende **cobre** o um meio — uma dependente é mais estreita que a sua
   independente, nunca mais larga.
2. *«o coeficiente cancela na normalização, logo não é parâmetro»* — **verdadeiro só para o modo de
   ESCALA**; no campo de **agarrar** a anisotropia move-se com ele (`1,125×` a `1,333×` na tabela
   medida dentro do nosso próprio módulo), logo ali ele é parâmetro vivo.

⭐ **A alavanca real é o TERRITÓRIO, e esta espec não a dizia:** a patente **não tem família fora
dos EUA** — a única outra publicação é a pré-concessão americana do mesmo pedido
(`US 2018/0330554 A1`), e não foram achados membros europeus, brasileiros ou asiáticos. ⇒ a leitura
só morde onde há uso ou distribuição **nos Estados Unidos**; qual é o mercado é decisão do dono.

⚠️ Tudo isto é **parecer técnico de leitura de reivindicação, não aconselhamento jurídico**:
clean-room **não protege contra patente** (SKILL §8.1), e a decisão é do dono, com parecer humano.

⛔ Nenhuma outra patente viva lê sobre esta obra — foram examinadas **três** outras famílias
(escultura progressiva; referencial tangente para pintura em superfícies; alisamento adaptativo) e
nenhuma alcança o polegar, o empurrão, a silhueta, a âncora em vértice, o deslize ou o modo de
alcance.

### §9.2 — ⛔⛔ O alvo de deformação «simulação» — excluído por MEDIÇÃO

**Duas medições independentes, e as duas dizem para não construir:**

1. ⭐ **O oráculo move ZERO vértices**, nos **cinco** gestos testados (polegar, empurrão, agarrar,
   deslize, gancho — fixtures `*_alvo_pano`), enquanto os pares `*_alvo_geometria` movem
   normalmente. `movidos = 0`, `max_deslocamento = 0,000000`, em duas corridas independentes.
2. ⭐ **A opção não é alcançável pela própria interface do alvo** para nenhum destes gestos: ela só
   é oferecida em dois outros gestos, que não são desta família. Forçá-la pela API é que a põe cá.

⇒ **não se constrói.** ⚠️ A leitura honesta tem duas saídas — ou a opção é inerte para esta família
no alvo, ou o caminho scriptado não satisfaz alguma pré-condição — e **nenhuma delas justifica
construir**: no primeiro caso construiríamos um no-op, no segundo não temos comportamento nenhum
para copiar. *Antes de reabrir isto, alguém tem de a ver funcionar no app, à mão.*

### §9.3 — Fora por ordem do dono

Conjuntos de faces (não existem na casa) e tudo o que é **cor**. Onde o alvo filtra por conjunto de
faces, a nossa lei simplesmente não filtra — e isso está declarado, não escondido (§7.3).

---

## §10 — Determinismo, precisão, e de onde sai a barra

- **Determinismo.** Nada nesta família é aleatório. A ordem de visita dos vértices não altera o
  resultado: cada vértice recebe `offset × w` com `w` função só da pose lida e o offset é comum ao
  carimbo. ⭐ O deslize é a excepção parcial — ele lê as posições **vivas** dos vizinhos, logo a
  ordem *poderia* importar; no alvo as translações são todas calculadas **antes** de qualquer
  escrita, dentro do mesmo carimbo. ⇒ **calcule tudo, depois escreva** (a nossa casa já o faz).
- **Precisão.** As posições são `f32` dos dois lados. Os resíduos confirmados desta espec vivem em
  `10⁻⁷`–`10⁻⁶` sobre deslocamentos de `10⁻²`–`10⁰` — ou seja, **no ruído de `f32`**.
- **A barra.** ⇒ a barra derivada é `≈ 1e-6` **relativo ao deslocamento máximo da fixture**, que é
  o que `f32` dá após ~12 acumulações. ⛔ **Não** um epsilon de conforto; ⛔ **não** paridade ao bit
  (o precedente da casa recusa prometê-la num pipeline T2 — ADR-0162).
- ⚠️ **A barra só vale onde a lei está confirmada:** fixtures planas para o empurrão e o deslize,
  todas para o polegar (§6.3, §7.3). Nas curvas do empurrão, **regressão**, não paridade.
- ⚠️ **O que depende da taxa de eventos** está na §4 e é metade da espec: um gate que alimente o
  empurrão com outro número de eventos mede outro programa.

---

## §11 — Os vectores de teste

`docs/3D/cleanroom/fixtures/pull/` — 62 traços + 5 malhas de repouso, formato e proveniência no
[README](fixtures/pull/README.md) de lá. Cada `.deformado.txt.gz` traz no cabeçalho **as 20
grandezas** que decidem o resultado, e o gate lê o cabeçalho — ⛔ nunca a prosa.

⚠️ **Dez das 62 medem ZERO de propósito** (cinco do alvo de deformação, cinco da silhueta): elas
são a prova de que aqueles dois comportamentos **não foram exibidos**, e existem para impedir que
alguém escreva que os mediu.

---

## §12 — O que fica ABERTO, com o instrumento de cada item

1. ⏳ **A silhueta nunca foi exercitada.** As cinco fixtures com ela ligada dão zero, e há **duas
   causas possíveis que produzem o mesmo zero**: (a) a armadilha do sinal nulo (§7.1), que é real e
   está lida; (b) **o valor que a lei usa é amostrado no HOVER**, e um traço scriptado nunca faz
   hover — o harness tentou mover o cursor do sistema e a ferramenta de que precisa não está
   instalada nesta máquina. ⇒ **fechar com uma corrida de E**: instalar essa ferramenta (ou
   conduzir um hover real) e repetir as cinco. ⚠️ Até lá, a lei do §7.1 está **lida, não medida**.
2. ⏳ **O empurrão em superfície curva** (§6.3) — **estreitado em 2026-09-13**, já não é «fechar
   gravando o centro»: o centro está medido e é o analítico; o que falta é **qual pose alimenta a
   normal da área por evento**, com duas candidatas já refutadas e a tabela dos oito eventos
   publicada (§6.3-bis). ⛔ O gesto ancorado **não** partilha este buraco.
3. ⏳ **Os resíduos `~2e-03` de duas direcções do deslize** (§7.3) — a suspeita era a mesma do
   item 2 e ⚠️ **a parte «o centro» dela caiu**; o instrumento é o mesmo (truncar e subtrair).
4. ⏳ **As curvas de alcance não exercitadas** (§2.2) — só três das formas foram medidas.
5. ⏳ **A versão.** O fonte lido é `5.2.0`; o binário-oráculo é `5.2.1 LTS`. Nenhuma diferença de
   comportamento foi observada nesta família, e nenhuma foi **procurada** — o checkout não tem a
   etiqueta `5.2.1`. ⚠️ Registado como risco, não como facto.

---

## §13 — O que isto custa à casa, em uma tabela

| item | estado na casa | o que falta |
|---|---|---|
| **POLEGAR** | ⛔ não existe | verbo novo; é um `Grip::Hold` cujo offset é a **parte tangencial** do gesto |
| **EMPURRÃO** | ⛔ não existe | verbo novo; é um `Grip::Hook` com o mesmo offset tangencial |
| a normal da área com **dois baldes** | parcial | conferir que a nossa amostragem separa frente/verso (§5.2) |
| a **fracção do raio da normal** | ⛔ | um knob (nasce em `0,5`) |
| **força ao quadrado** | ⛔ | ⚠️ só nestes dois verbos e no deslize — **não** no agarrar nem no gancho (§3) |
| **silhueta** | ⛔ | uma etapa no fim da cadeia do `Move` + o tratamento do sinal nulo |
| **âncora em vértice** | ⛔ | uma linha no pen-down do `Move` |
| **três direcções do deslize** | ⛔ | a outra metade do `SlideRelax` (§7.3) |
| **modo de alcance do gancho** | ✅ **já é o que temos** | nada — ⛔ não construa o selector (§7.4) |
| **modo elástico do gancho** | — | ⛔ **patente viva** (§9.1) |
| **alvo de deformação = simulação** | — | ⛔ **medido a zero** (§9.2) |
