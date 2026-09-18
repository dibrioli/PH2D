# SPEC — o PENTE DE TOPOLOGIA: o fluxo das arestas alinha-se com o traço

```
Alvo: Blender 5.2 (fonte disponível na tag v5.2.0; ORÁCULO CORRIDO = binário 5.2.2 LTS,
  pacote 17:5.2.2-1, build 2026-09-15) · Licença: GPL-2.0-or-later · Degrau: T2
  ⚠️ O binário SUBIU de 5.2.1-2 para 5.2.2-1 em 2026-09-15, entre a obra do pincel afiado e
  esta. Este corpus e os das obras anteriores NÃO vêm do mesmo binário.
Rótulo público do controlo no painel do alvo: «Topology Rake». Nesta casa: o «pente de topologia».
Ledger: aberto em docs/3D/cleanroom/LEDGER_blender-rake.md, 2026-09-17, ANTES da primeira leitura
  de conteúdo do fonte. Autor: subagente-E, 2026-09-17.
MÉTODO: ⭐ esta espec foi escrita SEM LER UMA LINHA do fonte do alvo. Tudo o que ela afirma saiu de
  CORRER o alvo sem interface sobre malhas NOSSAS (221 ficheiros em fixtures/rake/) e de medir a
  saída. A superfície pública de propriedades saiu de um despejo da API, corrido.
Patente (§8.1): buscado em 2026-09-17 — troca de diagonal alinhada à direcção do traço · relaxação
  de vértices por fluxo de arestas durante escultura · alinhamento de fluxo por pincel. Resultado:
  NENHUMA patente viva alcança o método. A vizinha mais próxima é US 9 349 216 B2 (Disney, viva até
  2034), cuja reivindicação 1 é sobre GERAR retalhos de quadriláteros a partir de uma rede de curvas
  desenhadas — não sobre trocar diagonais nem mover vértices de uma malha existente durante um
  traço. Arte anterior pública esmagadora: a troca de diagonal como melhoria local é de Lawson,
  1977, e a própria opção do alvo é pública desde Janeiro de 2019. Veredito: prosseguir.
Filtragem §4.3: executada em 2026-09-17 sobre este texto (zero código, zero nome interno de função
  ou de ficheiro, zero comentário, zero wording de manual, zero tabela transcrita; as fórmulas são
  matemática do método em notação nossa; todo `code span` é nosso, de fixtura nossa, ou
  identificador PÚBLICO de propriedade/enumeração usado como chave de regeneração — §4.1.13).
Sweep: ver o ledger. Re-corrido sobre a versão emendada e sobre as 221 fixturas.
Auditoria §4.2 (R-pré): 1.ª passagem, subagente-R independente, 2026-09-17 — ⛔ **REPROVADA**
  (13 achados + um limite que esta espec não declarava). ⭐ A tese central, a régua e a barra
  resistiram inteiras; o que reprovou foi o DOCUMENTO.
EMENDA DO E, 2026-09-17 (1.ª): os oito pontos do LEDGER §4.2.6 estão curados, e o limite do
  §4.2.3 deixou de ser um item por declarar — ele foi **MEDIDO** (§14), e a medição **REFUTOU** a
  inferência que esta espec fazia no §11.1. O corpus foi **re-emitido** com o cabeçalho a
  regenerar todos os eixos (A-3) e ganhou a família `lei_unica/` (26 células novas).
  ⛔ A janela I NÃO implementa até uma 2.ª passagem de R-pré atestar esta versão.
Mapa de leitura da literatura: a troca de diagonal local (Lawson 1977) e a cadeia de remalhagem
  incremental partir/colapsar/**trocar**/alisar (Botsch–Kobbelt 2004) — as duas já portadas nesta
  casa. ⛔ Não há apêndice do alvo a ler; não é preciso nenhum.
Denylist de URLs (⛔ o Implementador não abre): projects.blender.org · developer.blender.org ·
  devtalk.blender.org · qualquer espelho ou pesquisa de código do alvo.
"Este documento descreve comportamento; não contém expressão do alvo."
```

---

## §1 — O que ele é, numa frase

O artista está a esculpir com a **topologia dinâmica armada**. Com o pente ligado, à medida que o
traço corre, a malha debaixo dele reorganiza-se de modo que as suas arestas deixam de apontar em
todas as direcções e passam a formar uma **grade**: uma família ao longo do traço, outra através
dele.

⭐⭐ **E o nome engana: medido, ele NÃO troca uma única diagonal.** Ver §3.

---

## §2 — Quando é que ele age: as pré-condições

### §2.1 — A topologia dinâmica tem de estar ARMADA — é a única pré-condição de estado

Com ela desarmada, os dois lados do controlo dão a **MESMA malha, byte a byte**.
⚠️ **A receita do resumo, porque a 1.ª redacção não a dava e um terceiro teve de a descobrir por
varredura de seis definições:** `sha256` do **corpo** de cada ficheiro — as linhas que **não**
começam por `#`, juntas por `\n`, que é a malha sem o cabeçalho. Os dois lados dão
`5db23f3ef9375358`.
**Fixturas:** `porta/c_nodyn_p000` · `porta/c_nodyn_p100`.

### §2.2 — Mas ele NÃO depende do passe de refino

⚠️ Esta é a metade que uma leitura rápida entende ao contrário. Com a topologia dinâmica armada mas
o passe de refino **desarmado** — seja porque o modo de detalhe é o manual, seja porque o refino só
tem autorização para colapsar e não há aresta curta — **o pente continua a agir, e com um efeito
MAIOR do que no regime com refino** (`Q` de `+0,1511` para `+0,3572`, `ΔQ = +0,2061`).
⚠️ **Não é o maior do corpus**: a mesma configuração com `4` passagens dá `ΔQ = +0,2188` (§5.2) e
`verbos/v_layer` dá `+0,2026`. O superlativo estava errado na 1.ª redacção; a **lei** (§11.3) não
dependia dele.
**Fixturas:** `composicao/m_manual_p000|p100` · `composicao/m_collapse_p000|p100`.

⇒ **A cerca é «a topologia dinâmica está armada», nunca «o passe vai correr».**

### §2.3 — É uma OPÇÃO de pincel, não um pincel

Ele é uma propriedade do pincel (um número por pincel), não uma entrada do catálogo de ferramentas.
Quem o honra e quem o ignora está no §6.

---

## §3 — A LEI, por passo — e o achado que muda o desenho

### §3.1 ⭐⭐⭐ Onde o operador de topologia do alvo não trabalha, o pente não muda UMA aresta — ele MOVE VÉRTICES.

⭐⭐ **A prova PRINCIPAL é esta, e ela mata a objecção óbvia.** A objecção é: *«no modo manual o
passe de topologia não corre, logo o teste não distingue ‹o pente não troca diagonais› de ‹trocar
está atrás do passe que foi desligado›»*. Há células em que o passe está **ARMADO e autorizado a
subdividir** — modo de detalhe constante, refino a partir **e** a colapsar — e cuja lista de faces
sai **igual à da entrada dos DOIS lados**:

| célula | topologia dinâmica | refino | detalhe | faces `==` faces da entrada | `ΔQ` do pente |
|---|---|---|---|---|---|
| `verbos/v_layer_p000|p100` | armada | `SUBDIVIDE_COLLAPSE` | `CONSTANT` | **sim, nos dois lados** | **`+0,2026`** |
| `verbos/v_smooth_p000|p100` | armada | `SUBDIVIDE_COLLAPSE` | `CONSTANT` | **sim, nos dois lados** | `+0,0271` |

⇒ **com o operador de topologia do alvo armado e com autoridade, o pente reorganizou a malha ao
ponto de mover `Q` em `+0,2026` e não mexeu numa única aresta.**

E a mesma resposta repete-se no regime **determinístico** (§9), com o passe desarmado, em `1, 2, 4,
8 e 16` passagens do mesmo traço — onde o conjunto de arestas é idêntico entre os dois lados **e
idêntico ao da entrada** (`841` V · `1 568` F · `2 408` E):

| passagens | arestas (pente `0` × pente `1`) | vértices que o pente moveu | maior deslocamento |
|---|---|---|---|
| 1 | **IGUAIS** | 170 | `0,03199` |
| 2 | **IGUAIS** | 185 | `0,03083` |
| 4 | **IGUAIS** | 186 | `0,03731` |
| 8 | **IGUAIS** | 187 | `0,04324` |
| 16 | **IGUAIS** | 187 | `0,05284` |

**Fixturas:** `mecanismo/x_man_x01..x16_p000|p100` (o cabeçalho de cada uma declara `PASSAGENS=`).

⚠️ **As duas colunas numéricas têm réguas declaradas, porque a 1.ª redacção deu uma coluna que não
re-derivava:** *«moveu»* conta os vértices com deslocamento **acima de `eps = 1e-7`**, e *«maior
deslocamento»* é a maior **norma** do vector `Δ` (a 1.ª redacção dava a maior **componente**, que é
outra grandeza e mede `0,02509`–`0,03787`).

⛔⛔ **E o que estas células provam tem uma fronteira, que a 1.ª redacção desta espec atravessou sem
a ver.** Elas provam *«quando o operador de topologia do alvo não emite nem colapsa nada, o pente
não muda uma aresta»*. Elas **NÃO** provam o que aquela redacção inferia — *«logo a topologia
diferente é o passe a decidir sobre geometria já relaxada»* —, porque isso é uma afirmação sobre o
**interior** do alvo. **Essa inferência foi MEDIDA e está REFUTADA: ver o §14.**

### §3.2 — O deslocamento é TANGENCIAL e é uma RELAXAÇÃO, não um arrasto

Medido sobre os 170 vértices que se movem (célula `composicao/m_manual`, o traço ao longo de `+x`):

| grandeza | valor |
|---|---|
| média de \|Δx\| (ao longo do traço) | `0,008142` |
| média de \|Δy\| (através do traço) | `0,006796` |
| média de \|Δz\| (fora da superfície) | `0,002426` |
| razão tangencial / normal | **`4,37`** |
| norma da soma dos Δ sobre a soma das normas | **`3,84 %`** |
| maior norma de Δ | `0,033643`, ou `0,34` do comprimento médio de aresta (`0,0995`) |

⇒ **o material não viaja** (a soma vectorial cancela a `96,2 %`): cada vértice desliza **sobre** a
superfície até que as arestas à volta dele fiquem alinhadas com o quadro do traço. O deslocamento
fora da superfície existe e é `4,4×` menor que o tangencial.

### §3.3 — O que ele optimiza é uma GRADE, não uma direcção

Medida a régua de quatro dobras do `README` das fixturas, sempre relativa à direcção do traço, o
pente faz crescer **as duas** famílias — a paralela **e** a transversa — à custa das diagonais. Em
percentagem das arestas da pegada, com o traço ao longo de `+x` (`rotacao/d_a0000`):

| faixa de ângulo à direcção do traço | desligado | no máximo |
|---|---|---|
| `0°`–`10°` (ao longo) | `14,0 %` | **`16,7 %`** |
| `30°`–`60°` (diagonal) | `38,8 %` | **`29,8 %`** |
| `80°`–`90°` (através) | `9,8 %` | **`16,1 %`** |

⛔ É por isso que a régua tem de ser a de **quatro** dobras: a de duas, que é a primeira que ocorre
a quem escreve, **troca de sinal** entre rotações do mesmo traço e é `5×` a `90×` mais pequena que
a de quatro dobras — **cega por construção** (`README` §2, e o gate **G-13**).

### §3.4 — A ordem importa, e está medida

O mesmo percurso percorrido **ao contrário** dá uma saída diferente, no regime determinístico:
`186` de `841` vértices diferem entre ida e volta com o pente no máximo, contra `139` com ele
desligado. ⇒ **o resultado é função do CAMINHO percorrido, não só do conjunto de pontos.**
**Fixturas:** `mecanismo/y_ida_p000|p100` · `mecanismo/y_volta_p000|p100`.

---

## §4 — A direcção contra a qual ele alinha

### §4.1 — É a direcção do TRAÇO, e a prova é rodá-lo

Rodando o traço e medindo sempre **contra ele**, a régua separa no mesmo sentido nas quatro
rotações — o que só acontece se a direcção de referência for o traço e não os eixos do mundo nem a
estrutura da malha de entrada:

| rotação do traço | `Q` desligado | `Q` no máximo | `ΔQ` |
|---|---|---|---|
| `0°` | `−0,0456` | `+0,1243` | `+0,1698` |
| `22,5°` | `−0,0154` | `+0,0901` | `+0,1056` |
| `45°` | `+0,0273` | `+0,1491` | `+0,1218` |
| `67,5°` | `−0,0123` | `+0,0940` | `+0,1063` |

**Fixturas:** `rotacao/d_a0000|a0225|a0450|a0675 × p000|p100`.

⚠️ A linha dos `45°` tem o lado desligado mais alto (`+0,0273` aqui, `+0,0298` na célula gémea da
escada, que é a que fixa o vale) porque a malha de entrada é uma
grade e a `45°` as diagonais dela caem alinhadas com o traço. **É essa célula que fixa a fronteira
inferior do vale** (`README` §3) — *o pior caso do lado desligado é uma propriedade da FIXTURA, e
por isso ele tem de estar no corpus.*

### §4.2 — Numa curva ele segue a direcção LOCAL

Com um percurso em arco de `150°` o pente age e a régua separa (`−0,0322` → `+0,0958`) medindo cada
aresta contra o **troço de percurso mais próximo**. ⛔ Esta espec **não** afirma que medir contra a
corda daria outro número: isso não foi separado.
**Fixturas:** `composicao/m_arco_p000|p100` · `mecanismo/y_arco_p000|p100`.

### §4.3 ⭐ Quando a direcção é degenerada, ele é INERTE — e é byte-idêntico

No regime determinístico, com o verbo a agir (o controlo está em cada célula):

| gesto | o verbo moveu | o pente moveu | veredito |
|---|---|---|---|
| cursor **parado**, 14 carimbos no mesmo sítio | `49` vértices | **`0`** | **inerte, saída byte-idêntica** |
| **um** carimbo só | `56` vértices | **`0`** | **inerte, saída byte-idêntica** |
| **dois** carimbos | `70` vértices | `53` | **age** |

**Fixturas:** `mecanismo/y_parado` · `mecanismo/y_umdab` · `mecanismo/y_doisdab`, cada uma nos dois
lados do controlo.

⇒ **a direcção nasce do movimento entre carimbos consecutivos, e com menos de dois carimbos não
existe.** O alvo não inventa uma direcção nem usa a última: ele não faz nada.

---

## §5 — O controlo, com a faixa e o valor de fábrica MEDIDOS

Despejados da superfície pública de propriedades, **correndo** o alvo:

| | valor |
|---|---|
| tipo | número real |
| **valor de fábrica** | **`0,0`** — ⭐ ele nasce DESLIGADO |
| faixa oferecida na interface | `0,0` a `1,0` |
| faixa que a porta aceita | `0,0` a `1,0` (a mesma: não há folga digitável) |

⚠️⚠️ **ARMADILHA DE NOME:** o pincel do alvo tem **DOIS** controlos públicos cujo rótulo começa por
«rake», e são de assuntos diferentes. O outro governa o **ângulo da textura do carimbo** a seguir o
traço (o *rake* que o Painter desta casa já tem, `docs/Painter/rake_rewrite_design.md`), tem valor
de fábrica `0,0`, faixa de interface `0..1` e faixa de porta `0..10`. ⛔ **Não são o mesmo controlo
e não têm o mesmo tecto.** Quem procurar por nome apanha o errado.

### §5.1 — A escada: o botão é MONÓTONO e SATURA

`Q` na faixa central do traço (região `raio/2`), sobre a malha canónica, nas três rotações:

| posição do botão | traço a `0°` | a `22,5°` | a `45°` |
|---|---|---|---|
| `0,000` | `−0,0456` | `−0,0154` | `+0,0298` |
| `0,125` | `+0,0243` | `+0,0583` | `+0,0826` |
| `0,250` | `+0,0494` | `+0,0857` | `+0,1075` |
| `0,375` | `+0,0821` | `+0,0913` | `+0,1210` |
| `0,500` | `+0,0946` | `+0,0976` | `+0,1332` |
| `0,750` | `+0,1162` | `+0,1003` | `+0,1443` |
| `1,000` | `+0,1156` | `+0,0971` | `+0,1507` |

**Fixturas:** `escada/k_a0000|a0225|a0450 × p0000..p1000` (21 células).

⭐ **Leitura de produto:** **metade do efeito já está no primeiro oitavo do curso** (`0 → 0,125`
compra `+0,070` dos `+0,161` totais a `0°`), e **acima de `0,75` o botão é praticamente inerte** —
a `0°` ele até desce (`+0,1162` → `+0,1156`), dentro da banda de repetição de `0,0111` (§9).
⇒ *a faixa útil deste botão é `0` a `0,5`; o resto do curso é enfeite.*

### §5.2 — As passagens acumulam, e o lado desligado chega a um ponto fixo

| passagens | `Q` desligado | `Q` no máximo | vértices de saída, desligado |
|---|---|---|---|
| 1 | `−0,0456` | `+0,1235` | `2525` |
| 2 | `−0,0316` | `+0,1023` | `2648` |
| 4 | `−0,0299` | `+0,1635` | `2627` |
| 8 | `−0,0299` | `+0,1693` | `2627` |

**Fixturas:** `escada/n_x1|x2|x4|x8 × p000|p100` (o cabeçalho declara `PASSAGENS=`).

⇒ sem o pente a malha **assenta** (4 e 8 passagens dão exactamente os mesmos `2627` vértices e o
mesmo `Q`); com ele **continua a alinhar-se**. ⚠️ A célula de 2 passagens lê abaixo da de 1 —
`0,0212` de diferença, que é **duas vezes** a banda de repetição: a subida não é monótona passagem a
passagem, e esta espec **não** a declara monótona.

⛔ **E num regime SEM refino, muitas passagens PIORAM:** `+0,3611` (1) · `+0,3725` (4) · `+0,3031`
(8) · `+0,2828` (16). **Fixturas:** `mecanismo/x_man_x*`.

---

## §6 — O CENSO: quem honra o pente e quem o ignora

Um traço igual, o mesmo enquadramento, o botão a `0` e a `1`, **e o controlo dentro de cada célula**
(quantos vértices o próprio verbo moveu contra o repouso).

### §6.1 — Os que honram, mudando a malha (12)

`DRAW` · `CLAY` · `CLAY_STRIPS` · `CREASE` · `BLOB` · `INFLATE` · `PLANE` · `PINCH` · `NUDGE` ·
`SNAKE_HOOK` · `SIMPLIFY` · `MULTIPLANE_SCRAPE`. O `Q` sobe entre `+0,052` (`BLOB`) e `+0,173`
(`SIMPLIFY`), e a contagem de vértices de saída difere entre os dois lados por **`37`** (`CREASE`) a **`109`**
(`NUDGE`), muito acima da banda de `±4` (§9).

### §6.2 — Os que honram, mexendo só nas posições (2)

`SMOOTH` (`+0,027`) e `LAYER` (`+0,203`). Nestas células o conjunto de arestas não muda de lado
nenhum — são o §3.1 outra vez, por outra porta.

### §6.3 ⛔ Os CINCO que o ignoram, com a saída **byte-idêntica**

| verbo | o próprio verbo agiu? | saída com `0` × com `1` |
|---|---|---|
| `DRAW_SHARP` | sim — moveu `187` vértices | **idêntica** |
| `THUMB` | sim — moveu `56` | **idêntica** |
| `GRAB` | sim — moveu `56` | **idêntica** |
| `ROTATE` | sim — moveu `49` (com varredura **angular**) | **idêntica** |
| `MASK` | sim — escreveu o canal (soma `0` → `34,777`, `188` valores distintos) | **idêntica** |

**Fixturas:** `verbos/v_draw_sharp|thumb|grab × p000|p100` · `composicao/m_rot_*` ·
`composicao/m_mask_*`.

⚠️⚠️ **DUAS destas células tiveram de ser REFEITAS, e é a lição mais transferível deste censo.** Na
primeira redacção a torção corria sobre um percurso **recto** — e um verbo de torção ancorado varre
ângulo **zero** numa recta, logo moveu `0` vértices: a célula lia-se exactamente como *«ignora o
pente»* sendo que era **inerte**. A máscara moveu `0` vértices por lei (ela escreve um canal), e só
o **controlo de canal** a separa de uma célula morta. ⇒ *um verbo inerte e um verbo que ignora o
controlo dão a mesma leitura; o que os separa é o controlo escrito ao lado.*

⭐ **A família dos cinco tem forma:** são os que lêem as posições de **repouso** ou que trabalham
**ancorados**, mais o que não escreve posição nenhuma. É coerente com um defeito público conhecido
do alvo (§10.2).

---

## §7 — Como ele compõe

| com o quê | medido | fixturas |
|---|---|---|
| **refino só a partir** | age (`−0,0327` → `+0,1033`), e a malha difere | `composicao/m_subdiv_*` |
| **refino só a colapsar** | age, **sem trocar aresta nenhuma** (`+0,1511` → `+0,3572`) | `composicao/m_collapse_*` |
| **partir + colapsar** | age (`−0,0456` → `+0,1291`) | `verbos/t_constant_*`, `escada/k_a0000_*` |
| **modo de detalhe constante** | age (`−0,0456` → `+0,1291`) | `verbos/t_constant_*` |
| **relativo** | age (`+0,0955` → `+0,1972`) | `verbos/t_relative_*` |
| **pelo pincel** | age (`+0,0092` → `+0,0880`) | `verbos/t_brush_*` |
| **manual** | age, **sem trocar aresta** (`+0,1511` → `+0,3571`) | `verbos/t_manual_*` |
| **resolução do detalhe** `8 / 12 / 18 / 30` | age em todas; `ΔQ = +0,1253 / +0,1095 / +0,1678 / +0,1323` | `verbos/q_res08|12|18|30_*` |
| **simetria em x**, traço a `30°` | age (`+0,0110` → `+0,1183`), e o lado espelhado também | `verbos/s_sim_*` |
| **máscara** | ⛔ **não medido** — ver §12 | — |

⚠️ **A linha «partir + colapsar» da 1.ª redacção citava `composicao/m_manual_*`, que é a fixtura do
regime OPOSTO** (o cabeçalho dela diz `modo_de_detalhe=MANUAL` e ela mede `+0,1511` → `+0,3572`), e
essa é a fixtura declarada de **G-5** e de **G-6**. *Uma fixtura descrita no regime errado faz o
implementador ler a lei errada exactamente no gate que ela defende.*

---

## §8 — Os casos degenerados de MALHA

| malha | o que acontece | fixturas |
|---|---|---|
| **quadriláteros** (nenhum triângulo) | age. A topologia dinâmica tritura a malha e o pente aplica-se ao resultado: `625` → `2095` (desligado) contra `2027` (no máximo), `Q` `−0,0768` → `+0,1210` | `composicao/g_quads_*` |
| **bordo aberto** (plano com furo, traço a atravessá-lo) | age, e sem recusar: `Q` `−0,0625` → `+0,1159`. ⚠️ Aqui a contagem de vértices **sobe** com o pente (`1889` → `1920`), ao contrário das peças fechadas, onde desce | `composicao/g_furo_*` |
| **superfície curva** (esfera) | age — as contagens saem iguais (`2 032` faces e `3 048` arestas dos dois lados) e a **diferença simétrica do conjunto de arestas é `1 260`**, logo a malha mudou mesmo. ⛔ **E a régua desta espec é planar: ela lê `+0,0453` nos DOIS lados sobre essa malha mudada**; ver §12 | `composicao/g_esfera_*` |

---

## §9 ⛔⛔ O ORÁCULO NÃO REPETE — e a banda tem de estar no gate

Seis corridas da **mesma** célula, com o passe de refino armado:

| | vértices de saída | `Q` | amplitude de `Q` |
|---|---|---|---|
| pente desligado | `2525` nas seis | `−0,04555619` a `−0,04555623` | `4·10⁻⁸` |
| pente no máximo | `2444` a `2448` | `+0,11428` a `+0,12542` | **`0,0111`** |

**Fixturas:** `banda/b_con1..6 × p000|p100` e `banda/b_man1..3 × p000|p100`.

⇒ **com o passe de refino desarmado o alvo é determinístico nos dois lados** (conjunto de arestas
idêntico entre corridas, `Q` à 4.ª casa). **Com ele armado e o pente ligado, a saída varia entre
corridas.** Toda barra desta espec conhece essa banda: o vale medido (`0,0334`) é **`3,0×`** a banda.

⛔⛔ **E há uma armadilha de instrumento que vale para toda bancada desta casa:** comparando duas
corridas **por índice de vértice**, `522` das `7 460` arestas da malha (**`7,0 %`**) «diferem»
mesmo com o pente **desligado** — enquanto a contagem de vértices (`2 525` nas duas), a contagem de
arestas da pegada (`1 852` nas duas) e o `Q` são idênticos a sete casas.
⚠️ **As duas contagens têm de nomear a população, e a 1.ª redacção misturou-as:** ela dividia o
`522` (que é sobre a malha INTEIRA) pelas `~4 200` arestas **da pegada**, o que sobrestima o
fenómeno `1,8×`. A causa é que o passe de refino **emite a mesma geometria com os vértices por outra
ORDEM**: ordenadas as posições, a diferença é `266` exacta · `118` acima de `1e-8` · **`2` acima de
`1e-7`** · `0` acima de `1e-6` (a 1.ª redacção dizia `10`, que não re-deriva a tolerância nenhuma).
*Uma comparação por índice mede a ordem de emissão, não a malha.* ⇒ **toda régua desta obra é invariante à ordem**, e é essa a razão
técnica de a régua ser uma estatística da distribuição de ângulos.

---

## §10 — Defeitos PÚBLICOS conhecidos do alvo (a usar para SUPERAR)

⚠️ **Cada item traz a FONTE, porque sem ela um revisor não consegue decidir se a frase é um facto
re-dito ou uma tradução — e a tradução é a única classe que o instrumento da parede não apanha por
construção.** ⛔ O Implementador **não abre** estes endereços (denylist do cabeçalho); eles estão
aqui para o **revisor** poder auditar.

1. **Custo.** O manual público do alvo declara que esta opção tem impacto severo de desempenho e
   recomenda-a só em malhas de baixa contagem.
   **Fonte:** manual público do alvo, página das definições de pincel do modo de escultura
   (`docs.blender.org/manual/en/latest/sculpt_paint/brush/brush_settings.html`), consultada
   2026-09-17. ⇒ **medir o nosso custo e publicá-lo é onde se ganha** (esta espec **não** o mediu
   — §12).
2. ⭐ **Ruído nos verbos que lêem o repouso.** Discussão pública dos programadores do alvo regista
   que certas ferramentas precisam de ter o atributo de coordenadas de repouso tratado durante o
   alisamento e durante este pente **para evitar ruído** — e a nossa medição do §6.3 mostra que hoje
   o alvo simplesmente **não aplica o pente** a esses verbos.
   **Fonte:** discussão pública do refactor da topologia dinâmica no rastreador do alvo
   (`projects.blender.org/blender/blender/pulls/104613`), consultada 2026-09-17. ⇒ *os cinco
   excluídos não são um desenho acabado; são uma dívida do alvo.* **Se esta casa resolver o
   repouso, pode oferecê-lo aos cinco.**
3. **A automáscara por topologia e por conjuntos de faces é ignorada** quando a topologia dinâmica
   está armada. **Fonte:** relato público `#115515` no rastreador do alvo. Não é do pente, mas vive
   no mesmo interruptor.
4. **A topologia dinâmica desarma-se sozinha** ao sair do modo de escultura. **Fonte:** relato
   público `#128584` no rastreador do alvo.

---

## §11 — As leis que a medição deu e que o desenho não adivinharia

1. ⭐⭐⭐ **Onde o operador de topologia do alvo não emite nem colapsa nada, o pente não muda uma
   aresta — ele move vértices** (§3.1: `verbos/v_layer` com o passe **armado e autorizado**, `ΔQ =
   +0,2026` e faces iguais às da entrada; mais cinco células de `1` a `16` passagens).
   ⛔⛔ **A 1.ª redacção continuava a frase com *«…logo a topologia diferente é obra do passe a
   decidir sobre geometria já relaxada»*, e isso é uma INFERÊNCIA sobre o interior do alvo que
   agora está MEDIDA e REFUTADA — ver o §14.** *Compor a relaxação com o passe em série não
   reproduz o alvo; reproduz ZERO.*
2. ⭐⭐ **O que ele optimiza é uma GRADE de duas famílias**, não uma direcção só — e a régua natural
   (duas dobras) é **cega** a isso por cancelamento (§3.3).
3. ⭐⭐ **Ele não depende do passe de refino**, só de a topologia dinâmica estar armada — e é
   exactamente no regime sem refino que ele tem o efeito **maior** (§2.2).
4. ⭐ **Com menos de dois carimbos ele é inerte e byte-idêntico**: não inventa direcção nem reusa a
   última (§4.3).
5. ⭐ **Cinco verbos do catálogo ignoram-no, e a saída é byte-idêntica** — e são os que lêem o
   repouso ou trabalham ancorados (§6.3).
6. ⭐ **A faixa útil do botão é a primeira metade**: `0 → 0,125` compra `43 %` do efeito total e
   acima de `0,75` ele satura dentro da banda de ruído (§5.1).
7. ⛔ **Sem refino, muitas passagens PIORAM o alinhamento** (`+0,3725` a 4 passagens contra `+0,2828`
   a 16) (§5.2).
8. ⛔ **O alvo não é repetível com o refino armado**, e a não-repetição que se mede por índice é
   sobretudo **ordem de emissão de vértices** (§9).

---

## §12 — O que ficou POR MEDIR, e de quem é

| item | porquê | de quem |
|---|---|---|
| ⛔⛔ **O QUE SEPARA «o pente não troca» de «o pente ENVIESA as trocas do passe»** | **já não é uma suposição: foi MEDIDO (§14) e a resposta é que NÃO é uma lei só.** O que fica por separar são as DUAS causas possíveis do resultado — (a) o pente enviesa as decisões do passe, ou (b) a relaxação tem de correr **dentro** do laço por-carimbo, sobre a malha acabada de refinar. ⚠️ **Nenhum dos 14 gates do §13 vê a diferença**, e uma implementação só-de-relaxação entrega `≈ 0` no regime que o artista usa | **E**, e o instrumento está escrito no §14.4 |
| **A régua para malha CURVA** | a desta espec projecta no ecrã; numa esfera ela lê `+0,0453` nos dois lados sobre uma malha cuja diferença simétrica de arestas é `1 260`. A cura é projectar a aresta e a direcção do traço no **plano tangente local** | **E** (emenda), antes de qualquer gate sobre peça curva |
| **O CUSTO** (o §10.1) | nenhuma célula deste corpus mediu relógio. É a coluna em que o alvo declara publicamente que é fraco | **E**, com uma varredura de contagem de vértices |
| **A interacção com a MÁSCARA** | não foi corrida | **E** |
| **Se a direcção é a LOCAL ou a CORDA** num arco | o arco age, mas as duas leituras não foram separadas (§4.2) | **E** |
| **A não-monotonia entre 1 e 2 passagens** (§5.2) | `0,0212`, duas vezes a banda; pode ser real ou ser a banda a compor | **E** |
| **Se o nosso pente deve valer para os CINCO excluídos** | o alvo exclui-os por dívida dele (§10.2), não por lei | **o dono** |
| **Se a faixa do nosso botão deve parar em `0,5`** | medido que o resto do curso é inerte (§5.1); encurtar a faixa é decisão de produto | **o dono** |

---

## §13 — Os gates propostos, cada um com a barra e a fixtura

⚠️ Cada barra abaixo sai do vale medido no `README` das fixturas §3, e **os dois lados do vale são
saída do próprio alvo**.

⛔⛔ **E há UMA barra, não duas — a do MEIO do vale, usada com `≥` de um lado e `≤` do outro.** A 1.ª
redacção escreveu a do `G-3` como o **extremo** do lado desligado, arredondado a quatro casas *para
baixo* (`+0,0298` sobre uma célula que mede `+0,02982270`): margem **`−0,0000227`**, ou seja **o gate
nascia VERMELHO sobre a saída do próprio alvo**. *Uma barra tirada de um extremo tem margem zero por
construção, e o arredondamento decide o sinal dela; uma barra tirada do MEIO do vale tem margem dos
dois lados* — aqui, `±0,0167`.

| # | o que ele afirma | barra | fixtura |
|---|---|---|---|
| **G-1** | com a topologia dinâmica desarmada, o botão não muda um bit | igualdade **exacta** da malha | `porta/c_nodyn_p000|p100` |
| **G-2** | com ela armada e o botão no máximo, a grade alinha-se | `Q ≥ +0,0465` na região `raio/2`; margem `+0,0167` sobre a pior célula ligada (`+0,06323128`) | `escada/k_a0000_p1000` e as 33 irmãs do lado ligado |
| **G-3** | com o botão a zero, ela **não** se alinha | **`Q ≤ +0,0465`** — a MESMA barra, do meio do vale; margem `−0,0167` sobre a pior célula desligada (`+0,02982270`) | as 34 do lado desligado |
| **G-4** | ⭐ o pente não troca arestas **onde o operador de topologia do alvo não trabalha** — ⛔ e o título não pode prometer mais do que isso (§14) | conjunto de arestas **idêntico** entre os dois lados: nas duas células com o passe **armado** e faces iguais às da entrada, **e** com o refino desarmado em `1, 2, 4, 8, 16` passagens | `verbos/v_layer_*` · `verbos/v_smooth_*` · `mecanismo/x_man_x*` |
| **G-5** | o deslocamento é tangencial | razão tangencial/normal `≥ 3` (medido `4,37`) | `composicao/m_manual_*` |
| **G-6** | e é uma relaxação, não um arrasto | a norma da soma dos Δ `≤ 10 %` da soma das normas (medido **`3,84 %`**) | `composicao/m_manual_*` (⚠️ regime **sem** passe de refino) |
| **G-7** | a direcção é a do traço | `Q` sobe nas **quatro** rotações, cada uma medida contra o seu próprio traço | `rotacao/d_a*` |
| **G-8** | com `< 2` carimbos é inerte | igualdade **exacta** | `mecanismo/y_parado_*` · `mecanismo/y_umdab_*` |
| **G-9** | com `2` carimbos age | `≥ 1` vértice movido pelo botão (medido `53`) | `mecanismo/y_doisdab_*` |
| **G-10** | o botão é monótono no primeiro meio curso | `Q(0) < Q(0,125) < Q(0,25) < Q(0,375) < Q(0,5)` nas três rotações | `escada/*` |
| **G-11** | ⚠️ **e SATURA** — o gate EXIGE que a saturação exista | `Q(1,0) − Q(0,75) ≤` a banda de `0,0111` | `escada/k_*_p0750|p1000` |
| **G-12** | os cinco excluídos são byte-idênticos **e o verbo agiu** | as duas metades, senão o gate mede um verbo inerte | `verbos/v_draw_sharp|thumb|grab_*` · `composicao/m_rot_*` · `composicao/m_mask_*` |
| **G-13** | a régua de **duas** dobras não serve | `ΔS` **troca de sinal** entre as quatro rotações (`+0,0183 · −0,0111 · +0,0211 · +0,0012`) enquanto `ΔQ` é positivo nas quatro e nunca abaixo de `+0,10` | `rotacao/d_a*` |
| **G-14** | a malha de quadriláteros e a de bordo aberto não são recusadas | `Q` sobe acima da barra nas duas | `composicao/g_quads_*` · `composicao/g_furo_*` |

⛔ **G-13 existe porque a primeira régua desta obra foi a errada.** Sem ele, a próxima janela
reescreve a régua cega e não tem como saber.

---

## §14 ⛔⛔⛔ UMA LEI OU DUAS? — **MEDIDO: não é uma lei só**

> Esta secção nasceu de um achado do R-pré: o corpus da 1.ª versão provava *«onde o operador de
> topologia do alvo não trabalha, o pente não muda uma aresta»* e a espec **inferia** daí que a
> topologia diferente era o passe a decidir sobre geometria relaxada. Inferência sobre o interior do
> alvo não é medição. ⇒ **foi medida.**

### §14.1 — A pergunta, e o instrumento

*Com o passe de refino armado, a topologia que o alvo entrega é explicada por **relaxar e depois
deixar o passe decidir**?* Se sim, a nossa lei é **uma** (relaxação) e o passe é o nosso, que já
temos. Se não, falta-nos uma segunda lei.

O instrumento é o próprio alvo, obrigado a fazer as duas metades **em série**: um traço com o pente
no máximo e o passe **desarmado** (só relaxa), seguido de um traço com o pente a zero e o passe
**armado** (só refina). O **controlo** é o mesmo par com o pente a zero nos dois — mesmo número de
traços, mesmo deslocamento do verbo. Depois sobe-se a **granularidade do entrelaçamento**,
partindo o percurso em `3` e em `9` troços e alternando dentro de cada um.

### §14.2 — A ESCADA, medida

| como as duas metades são compostas | `ΔQ` do pente | % do que o alvo entrega |
|---|---|---|
| **JUNTO** — o alvo faz as duas coisas, que é o que o artista usa | **`+0,1670`** | `100 %` |
| em série, `1×` (relaxa o traço todo, depois refina o traço todo) | `−0,2081` | `−125 %` |
| em série, `3×` | `−0,0496` | `−30 %` |
| em série, `4×` | `−0,0128` | `−8 %` |
| em série, `9×` (o mais fino que a porta sem interface alcança) | **`+0,0084`** | **`5 %`** |

**Fixturas:** `lei_unica/w_junto_p0|p1_r1..r3` · `w_serie_p0|p1_r1..r3` · `w_inter_p0|p1_r1..r3` ·
`wf_n3|n9_p0|p1_r1..r2`. Cada célula declara a receita no cabeçalho (`SEQUENCIA_DE_TRACOS=`), e
cada configuração corre `2`–`3` vezes: a maior amplitude de repetição das seis é **`0,0115`**.

### §14.3 — O VEREDITO: **duas leis, não uma**

A composição em série **não converge para o alvo** — ela converge para **zero**. A `9×`, a mais
fina que o instrumento alcança, ela entrega `+0,0084` contra os `+0,1670` do alvo: **`5 %`**, e a
diferença (`+0,1586`) é **14× a banda de repetição**. Grosseira, ela é ainda **pior que não fazer
nada** (`−0,2081`): relaxar uma malha grosseira e só depois a subdividir **destrói** o alinhamento,
porque os vértices que o passe insere não sabem nada do traço.

⇒ ⛔ **uma implementação que faça só a relaxação medida no §3.2 e deixe o nosso passe de refino
decidir a seguir entrega `≈ 0` exactamente no regime em que o artista trabalha.**

⚠️ E o sinal está no outro canal também: a fracção de vértices irregulares na pegada **desce** com
o pente no caminho junto (`0,791` → `0,755`) e **sobe** na composição em série (`0,788` → `0,890`).

### §14.4 — O que este instrumento NÃO separa, e o que o separaria

As duas causas possíveis ficam **nomeadas e não separadas**:

- **(a)** o pente **enviesa as decisões do passe de refino** — uma segunda lei, dentro do operador;
- **(b)** a relaxação tem de correr **dentro do laço por-carimbo**, sobre a malha **acabada de
  refinar** — uma questão de ONDE ela corre, não de que lei é.

⛔ **A porta sem interface não as separa**, e isto é uma impossibilidade **medida**, não uma
suposição: o gesto público que o oráculo aceita é **um traço inteiro**, logo o mais fino que se pode
alternar é um troço de percurso (o `9×` acima, `2`–`3` carimbos por troço). Abaixo disso não há
gesto a emitir.

⭐ **O instrumento que as separaria** (acto do **E**, com o alvo ainda por cima da mesa): correr o
alvo com o passe armado e comparar a **distribuição de valências** da região contra a de uma malha
que o nosso próprio passe produza a partir da saída relaxada do alvo. Se a do alvo for mais próxima
de `6` do que a nossa pelo mesmo número de operações, a causa é **(a)**; se coincidirem, é **(b)**.

### §14.5 — O que isto manda ao Implementador

1. ⛔ **Não implemente a relaxação como um passe separado antes ou depois do refino.** Está medido
   que compor assim entrega `5 %` no melhor caso e o sinal trocado no pior.
2. ⭐ **Implemente-a DENTRO do laço por-carimbo**, sobre a malha que o refino acabou de produzir —
   é a hipótese **(b)**, é a mais barata, e é a que o `+0,0084` a `9×` sugere estar a convergir.
3. ⚠️ **E meça o resultado contra o `G-2`.** Se a nossa malha ficar abaixo da barra com a relaxação
   já no sítio certo, então a causa é a **(a)** e há uma segunda lei a construir — e nesse dia o
   §14.4 diz qual é o instrumento.
4. ⛔ **Nenhum dos 14 gates do §13 vê esta diferença**, porque com o refino armado a nossa topologia
   não bate com a do alvo de qualquer maneira. O `G-2` sobre a NOSSA saída é o único que a apanha.
