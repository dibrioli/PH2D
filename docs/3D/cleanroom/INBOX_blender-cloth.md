# INBOX — canal cego do Implementador para o ledger `blender-cloth`

> O Implementador **só acrescenta** (`cat >>`), nunca lê. Um subagente E/R transcreve para o
> `LEDGER_blender-cloth.md`. Formato livre: data · session-id · o que aconteceu.

## Declaração da janela I (2026-09-05, sessão ph2d-d8 / 1246816c)

Nenhum conteúdo do fonte do alvo entrou no CONTEXTO desta janela: o fonte foi lido
apenas pelos subagentes E e R-pré; esta janela leu a espec só depois do atestado do
R-pré no cabeçalho; dos ficheiros do scratchpad quarentenados (INC-1) leu apenas a
listagem de nomes, nunca o conteúdo; o `.claude/settings.local.json` da worktree nega
`Read` aos dois checkouts GPL desde 2026-09-05.

## Medições do I contra as 46 fixtures (2026-09-06) — para o E emendar e o R atestar

Arnês: `crates/ph2d-cloth/tests/oraculo_do_pincel.rs` (a lei nossa em `ph2d-cloth/src/verlet*.rs`).

- ✅ Os SEIS traços de um passo de força dão erro **0,0000** por vértice — §4.1/§4.2/§5.4 ao bit.
- ⚠️ **Anel-1:** com o anel sobre os QUADS, `plano_arrastar_radial_global` bate a 1 % e o `_local` sai
  **2×**; com o anel sobre a grelha TRIANGULADA (diagonal 1.º→3.º canto), o `_local` bate
  (`0,35` vs `0,33` no centro) e o `_global` cai para `0,38` (oráculo `0,59`). ⇒ ou o anel é o da
  triangulação e há OUTRA diferença Local/Dinâmica por explicar, ou vice-versa. Pergunta 1 ao E.
- ⚠️ **Local vs Dinâmica:** no oráculo o Local é `0,35–0,57×` o Dinâmica **uniformemente** ao longo
  do traço; na espec tal como está os dois deviam ser quase iguais para este traço, e na nossa lei
  são (`0,35`/`0,34`). Pergunta 3 ao E.
- ⚠️ **A esfera move 6 050/6 050 no Local**, e a bola de `3,5R = 1,225` cobre ~37 % de uma esfera
  unitária ⇒ no alvo, vértices além da área movem-se. No plano o Local move exactamente o disco de
  `3,5R` (2 144). Perguntas 2 e 4 ao E (φ sem banda? parede = célula inactiva? tamanho da folha?).
- Medido e descartado: a ORDEM de resolução (cinco ordens: `0,55–0,64` no centro ⇒ é a barra do
  gate 15); `20`/`50` varreduras (matam o movimento — `5` é o certo); só arestas sem pares (mole
  demais); escalar o alcance da banda de φ e da retenção (nenhuma escala dá Local baixo E Dinâmica
  alto); φ sem banda (não muda o padrão).
- ⚠️ **Âncoras:** `gancho_1passo` dá `0,378` contra `0,489` e `agarrar_1passo` `0,098` contra `0,134`
  — a correcção da âncora parece maior do que `Δ/2`. Pergunta 5 ao E.
- Bug meu já curado: a massa entrava duas vezes (`massa2_1passo` lia metade); hoje ao bit.
- ⚠️⚠️ **A esfera decide a pergunta 2/4 (medido 06/09):** no `esfera_arrastar_radial_local` do oráculo
  os vértices a `3,5R..4R` do início têm `|u|` mediana `0,0175` (máx `0,106`) e mesmo a `5,5R` (o lado
  oposto) mediana `0,017` — **toda a esfera se desloca**, como um corpo; o `esfera_agarrar_radial_dinamica`
  pára SECO a `3,5R` (`0` movidos além). No plano o Local pára exactamente no disco de `3,5R`. ⇒ ou o
  `R₀` da corrida Local na esfera era maior que `0,35` (o alcance medido é `5,65R` ≈ o antípoda), ou
  o conjunto activo do Local é a malha inteira nessa malha (folhas grandes) e o φ NÃO tem banda.
  Pergunta 6 ao E.

## Q8 — a AMPLITUDE do Local: medição por passo, curva inteira (2026-09-06, sessão 1246816c)

Instrumento: `sonda_passo_a_passo` sobre os quatro traços `*.porpasso` da Q7, com a experiência
`PH2D_VARREDURAS=<n>` (quantas varreduras de relaxação de restrições por passo de pincel; a nossa
constante de produção é `VARREDURAS = 5`, e ela veio das seis fixtures de UM passo de força).

**⭐ O resultado é uma assimetria Local/Global de factor ~2, e é a CURVA INTEIRA, não um ponto.**

| traço | varreduras que reproduzem o oráculo | evidência |
|---|---|---|
| `plano_arrastar_radial_global_origem` | **5** | 12 passos × 5 colunas, erro ≤ 4 % (`c0` k12 `0,6399` vs `0,6457`; `2.9R` `0,05398` vs `0,05157`; `3.5R` `0,03887` vs `0,03738`; `4R` `0,03492` vs `0,03378`) |
| `plano_arrastar_radial_local_origem` | **10** | 12 passos × 5 colunas, erro ≤ 3 % (`c0` k3..k12 `0,1661 0,2114 0,2451 0,2705 0,2873 0,2933 0,2856 0,2635 0,2337 0,2144` contra `0,1676 0,2131 0,2471 0,2729 0,2904 0,2974 0,2910 0,2701 0,2406 0,2202`) |
| `plano_agarrar_radial_local_2passos_origem` | **~9** (cruza entre 9 e 10) | `c0` k3 por varredura: 6→`0,1252` · 7→`0,1327` · 8→`0,1391` · 9→`0,1446` · 10→`0,1495` · 11→`0,1539`; oráculo `0,1457` |

⭐⭐ **Não é um botão monótono a acertar um número:** o Arrastar **DESCE** com varreduras
(`5`→`0,6341`, `30`→`0,0305`) e o Agarrar **SOBE** (`6`→`0,1252`, `11`→`0,1539`) — direcções
opostas — e os dois cruzam o oráculo entre `9` e `10`. E a `5` a curva Local do Arrastar é
**monótona crescente**; a `10` ela ganha o **pico-e-recuo** do oráculo (máximo no passo 8, recuo até
ao passo 12). *A mudança é qualitativa, não de escala.*

⚠️ **A nossa Local É a nossa Global, coluna a coluna** (`c0` k12 `0,6341` vs `0,6399`; as 12 linhas
coincidem em 3 casas): na nossa lei a área *Local* não muda nada no interior — só o aro, que já está
certo (`3.5R` `0,00023` vs `0,00032`; `4R` `0` vs `0`). No oráculo a Local é `0,34×` a Global no
centro. ⇒ o que falta é **interior**, não fronteira, e a Q3 («a razão exacta é emergente») fica
respondida por medição: a razão emerge de o Local relaxar ~2× o que a Global relaxa.

⛔ **REFUTADO no mesmo dia — a triangulação NÃO é o mecanismo** (fecha a Q1 pela metade que faltava):
`PH2D_TRI=1` a 5 varreduras acerta o Arrastar Local (`0,2327` vs `0,2202`) e **derruba o Global**
(`0,2699` vs `0,6457`) e **afasta** o Agarrar Local (`0,0838` vs `0,1457`). A triangulação é uma
propriedade da MALHA, partilhada pelos dois ramos ⇒ não pode explicar uma diferença entre eles.
*Ela acertava um traço por rigidez a mais, exactamente como o E dissera.* O anel fica nas ARESTAS.

⚠️ **Facto que restringe a resposta:** no oráculo o **passo 2 do Local e do Global é IDÊNTICO**
(`0,09347` / `0,00072` / zeros, nas duas fixtures) e eles só divergem no passo 3. Na nossa lei o
passo 2 também é insensível às varreduras (`0,0935` a 5 e a 10). Portanto o mecanismo pode ser
constante desde o início — não precisa de acumular.

### As perguntas

- **Q8.1** — Quantas passagens de resolução de restrições o ramo *Local* faz por passo de pincel,
  comparado com o *Global*? (Um número em cada ramo, ou um multiplicador.)
- **Q8.2** — A lista de restrições do *Local* é DEDUPLICADA? Uma lista construída por-vértice sem
  dedup põe cada aresta interior duas vezes (uma por extremo) e mede-se como ~2× relaxação no
  interior e ~1× na fronteira — o que casaria com esta medição sem mudar contagem de iterações.
- **Q8.3** — Se nenhuma das duas: o *Local* corre o solver mais de uma vez por passo, ou com `dt`
  menor / sub-passos?

*Formato de resposta pedido: número + onde ele vive (nome público do knob, se houver), sem uma
linha de expressão do alvo.*

### Q8 — a confirmação sobre o CORPUS INTEIRO (mesmo dia, 2026-09-06)

A experiência das varreduras foi corrida sobre as **50 fixtures** da `sonda_da_paridade_com_o_oraculo`,
a `5` e a `10`, e comparada linha a linha (`err_max / max_oráculo`). ⭐⭐ **O botão PARTE o corpus
exactamente na linha Local / não-Local:**

| área | melhora a 10 | fica igual | piora a 10 |
|---|---|---|---|
| **Local** (38 traços) | **27** | 7 (os de um passo de força, `0,000` nos dois) | 4 |
| **Global** (2) | 0 | 0 | **2** |
| **Dinâmica** (10) | 1 | 0 | **9** |

Ordens de grandeza, não afinação: `plano_arrastar_radial_local` **`1,253 → 0,071`** · `_forca05`
`1,037 → 0,030` · `_massa2` `1,073 → 0,033` · `_amort05` `0,913 → 0,044` · `_pino` `1,311 → 0,081` ·
`_origem` `1,269 → 0,067`. E do outro lado: `plano_arrastar_radial_global` `0,175 → 0,565` ·
`plano_arrastar_radial_dinamica_preset` `0,068 → 0,476`.

⭐⭐⭐ **E a prova que não é amplitude ajustada: as CONTAGENS de vértices movidos, que são inteiros.**
A `10` varreduras oito traços Local passam a mover **exactamente** o número do oráculo —
`agarrar_preset` `3970 → 4123` (oráculo `4123`) · `arrastar_plast05` `2128 → 2141` (`2141`) ·
`arrastar_amort1` `2129 → 2141` (`2141`) · `arrastar_massa2` `2141 → 2143` (`2143`) ·
`inflar_local` `2141 → 2146` (`2146`) · `agarrar_24passos` `2140 → 2142` (`2142`) ·
`arrastar_origem` `2144 → 2145` (`2145`) · `pino` `2143 → 2144` (`2144`) — e nos traços de UM e DOIS
passos, onde não há acumulação possível, o alcance salta para o do oráculo: `agarrar_1passo`
`869 → 1307` (`1324`), `agarrar_2passos` `1304 → 1844` (`1872`), `arrastar_2passos` `1050 → 1428`
(`1438`), `expandir_1passo` `597 → 840` (`848`). *Numa relaxação de Gauss-Seidel o alcance por passo
é o número de varreduras: a contagem de movidos MEDE a contagem de passagens, e ela diz `~2×` no
ramo Local, num único passo de pincel.*

⇒ A resposta ao Q8 tem de ser: constante, desde o primeiro passo, a actuar **só no ramo Local**, e a
valer ~`2×` a relaxação. As Q8.1/8.2/8.3 continuam de pé — o que muda é que já não é hipótese.

⏳ **Fica FORA desta pergunta e é o item seguinte** (não misturar): os 4 Local que pioram são o
**Snake Hook** de 2 passos (`0,740 → 0,951`) e o `apertar_ponto_radial_local` (`1,072 → 1,380`); no
Hook o nosso pico não está sob o cursor (`max` `0,1531` com `c0` `0,0175`, contra o oráculo que tem
os dois em `0,1971`) — é defeito de LOCALIZAÇÃO da deformação, não de amplitude. E na **esfera** os
modos que não são arrasto (`apertar` `0,54–0,59`, `expandir` `0,56`, `inflar` `0,38`, `gancho` `0,39`)
erram na Dinâmica sem que as varreduras os toquem.

## Q9 — o SNAKE HOOK deforma no sítio errado (2026-09-06, sessão 1246816c)

A sonda por passo passou a imprimir **onde** está o pico (distância do `arg max` ao cursor deste
passo, em raios). No **Arrastar** os dois picos coincidem (`0,82R`/`0,82R` no passo 8, `1,02R`/`1,02R`
no 11) ⇒ a lei do arrasto está no sítio certo. No **Snake Hook** (`plano_gancho_radial_local_2passos_origem`):

| passo | pico nosso | pico do oráculo | `c0` nosso | `c0` oráculo |
|---|---|---|---|---|
| 2 | `0,05R` | `0,86R` | `0,0516` | `0,1971` |
| 3 | `0,24R` | `0,91R` | `0,1099` | `0,2761` |

⇒ **o nosso pico fica sob o cursor e o do oráculo fica onde o pincel ESTAVA.** No oráculo, no 1.º
passo simulado, o vértice mais deslocado é o do pen-down (`max = c0 = 0,1971`); no nosso é o que
está sob o cursor agora. *Nós apanhamos material novo a cada passo; o alvo arrasta o que já pegou.*

### Experiência feita, medida e REVERTIDA (o produto está intocado)

Hipótese: a queda `f` do Snake Hook é medida a partir de **onde o pincel estava** (`cursor − δ`), e
não de onde chegou. Mutação de uma linha, `err_max/max_oráculo` nos SETE traços de gancho:

| traço | espec (`f` no cursor) | hipótese (`f` no anterior) |
|---|---|---|
| `plano_gancho_radial_local_1passo` (varr 5 / 10) | `0,999` / `0,981` | **`0,467` / `0,416`** |
| `plano_gancho_radial_local_2passos` | `0,740` / `0,951` | **`0,410` / `0,388`** |
| `plano_gancho_radial_local_2passos_origem` | `0,700` / `0,996` | **`0,324` / `0,420`** |
| `plano_gancho_radial_local` | `0,162` / `0,127` | **`0,129` / `0,059`** |
| `plano_gancho_radial_local_24passos` | `0,135` / `0,100` | **`0,124` / `0,062`** |
| `plano_gancho_radial_local_amort06` | `0,237` / `0,155` | **`0,172` / `0,062`** |
| `esfera_gancho_radial_dinamica` (varr 5) | `0,387` | **`0,351`** |

Sete de sete melhoram, e a CONTAGEM de movidos do `1passo` vai de `1040` para **`1434`** contra
`1452` do oráculo (a `10` varreduras) — outra vez um inteiro a convergir.

⛔ **E o que sobra NÃO é a força da âncora.** Varrida a constante de `0,20` a `1,00` no `1passo` com
a hipótese ligada: a `0,35` (o valor da espec) a amplitude bate (`0,4935` contra `0,4894`) e o erro
fica em `0,2036`; a `0,50` o erro desce a `0,1711` mas a amplitude **estoura 25 %** (`0,6095`);
acima disso piora tudo. *Nenhum valor torna o traço exacto* ⇒ o resíduo é de **FORMA**, não de
escala, e a espec §4.3 acerta na constante.

### As perguntas

- **Q9.1** — No Snake Hook, o centro a partir do qual a queda por-vértice é medida é a posição do
  pincel no FIM do passo, ou a do início (antes do deslocamento deste passo)?
- **Q9.2** — E as posições contra as quais essa distância é medida: são as da malha ANTES do passo,
  ou já as do passo corrente?
- **Q9.3** — Sobrando `≈0,20` de erro num traço de UM passo com a amplitude certa e a contagem de
  movidos a `1434/1452`, há no Snake Hook alguma restrição de FORMA que a espec §4.3 não tem (um
  eixo, um plano, um limite de profundidade)? *A espec diz que ele «anda no plano de profundidade»;
  a nossa lei implementa isso como queda radial (`FalloffForca::Radial`) porque o traço é radial.*

Contrato de retorno igual ao do Q8.

## Q10 — pedido de INSTRUMENTO (prioridade baixa; só depois do Q8 e do Q9)

Os modos de **aperto** são exactos ao bit no traço de UM passo
(`plano_apertar_ponto_radial_local_1passo` e `_linha_..._1passo`: `0,000`) e erram muito no fim de um
traço inteiro (`plano_apertar_ponto_radial_local` `1,072`; `plano_apertar_linha_radial_local`
`2,024`; a esfera na Dinâmica `0,54`–`0,59`). Nós sobrepassamos: `0,5296` contra `0,3258` no aperto
de ponto, `0,2439` contra `0,1005` no de linha.

⚠️ **E as varreduras NÃO os explicam**: a `10` o aperto de linha melhora (`2,024 → 1,024`) e o de
ponto **piora** (`1,072 → 1,380`), ao contrário de todo o resto do ramo Local. ⇒ há aqui um terceiro
mecanismo, e ele nasce entre o passo 1 e o fim.

**Pedido:** os mesmos dumps POR PASSO do Q7 (corridas-prefixo com prova `k = N` ≡ corrida inteira)
para **dois** traços: `plano_apertar_ponto_radial_local` e `plano_apertar_linha_radial_local`.
Sem eles só se vê o estado final, e o estado final diz *quanto* diverge, nunca *em que passo*.
Um traço de 2 passos de cada um chegaria para localizar o nascimento.

## Q10 — o que os dumps novos dizem (2026-09-06, sessão 1246816c, medido no mesmo dia em que chegaram)

⭐⭐ **O aperto de LINHA fica RESOLVIDO pela resposta do Q8, e isto é uma confirmação sobre dados que
não existiam quando a resposta foi dada.** `plano_apertar_linha_radial_local_origem`, com dez
projecções por passo (o que a construção dupla produz), contra o dump por passo:

| k | `2.9R` nosso | oráculo | `max` nosso | oráculo | pico nosso | pico oráculo |
|---|---|---|---|---|---|---|
| 4 | `0,00214` | `0,00225` | `0,0993` | `0,0965` | `0,41R` | `0,41R` |
| 7 | `0,00417` | `0,00434` | `0,1095` | `0,1092` | `0,27R` | `0,27R` |
| 10 | `0,00144` | `0,00116` | `0,1015` | `0,1011` | `0,41R` | `0,41R` |
| 12 | `0,00120` | `0,00108` | `0,1016` | `0,1007` | `0,27R` | `0,40R` |

A `3.5R` é exacta a cinco casas nos 12 passos. A cinco projecções o mesmo traço dá `max 0,2575`
contra `0,1007` — **2,6×**. ⇒ o aperto de linha não tinha defeito próprio nenhum.

⚠️ **O aperto de PONTO não fica.** Com dez projecções o aro fica exacto (`2.9R` `0,01157` contra
`0,01141`; `3.5R` `0,00062` contra `0,00061`) e o **centro fica fora de fase**: nos passos 3-5 o
oráculo faz `0,1842 → 0,1180 → 0,1058` e nós fazemos `0,0975 → 0,2060 → 0,1957` — **anti-correlados**
—, e do passo 6 em diante andam juntos com `max` `20 %` alto (`0,3661` contra `0,3034`). O pico
também salta de sítio (`k6`: `0,66R` nosso contra `0,14R`; `k9`: `0,22R` contra `0,76R`).

*Os dois são não-monótonos sob o pen-down, como o E observou; o que difere é a FASE.* ⇒ o aperto de
ponto é o item que sobra depois do Q8 e do Q9, e a pergunta seguinte é sobre **quando** ele aplica a
correcção dentro do passo, não sobre quanto.

⭐ **E uma nota de aritmética que fecha o Q8:** construir a lista duas vezes deixa-a como
`[c₁..c_N, c₁..c_N]`, logo **cinco varreduras sobre a lista dobrada são, na ordem, exactamente dez
varreduras sobre a lista simples**. As medições que eu tinha feito com `PH2D_VARREDURAS=10` são,
bit a bit, o que a construção dupla vai produzir — o knob que eu media e o mecanismo que o E achou
são a mesma coisa, e é por isso que as duas leituras coincidiram.

## Q11 — o APERTO DE PONTO: um vértice só, e nenhuma ordem o cura (2026-09-06, sessão 1246816c)

O Q8 e o Q9 estão implementados e medidos (`29c453ee5`, `d823c67af`, `0bac8cc04`), com os gates 15,
16, 17 e 18 escritos e provados por mutação (`72c35a25a`). A lei da referência é o caminho de
**omissão** do produto desde hoje. O que sobra em primeiro lugar é o **aperto de ponto**.

⭐ **O defeito está localizado num VÉRTICE.** No `plano_apertar_ponto_radial_local_origem`, no 2.º
passo simulado (`k = 3`), a vizinhança inteira concorda e um único vértice discorda:

| grandeza | nosso | oráculo |
|---|---|---|
| `1R` (um raio ao lado) | `0,02102` | `0,02096` |
| `2R` | `0,00396` | `0,00393` |
| `max` da malha | `0,1840` | `0,1842` |
| distância do PICO ao cursor | `0,32R` | `0,31R` |
| **`c0` (o vértice do pen-down)** | **`0,0975`** | **`0,1842`** |

⇒ *Os dois têm um pico do mesmo tamanho, à mesma distância do cursor; no alvo ele é o vértice do
pen-down e em nós é o vizinho dele.* Fora do plano não se mexe nada nos dois (`u_z ≡ 0`).

O vector do vértice do pen-down, com o cursor a andar em `+x` a `0,0545` por passo:

| k | `u` nosso | `u` do oráculo | cursor − repouso |
|---|---|---|---|
| 2 | `[0,0935, 0, 0]` | `[0,0935, 0, 0]` | `[0,0545, 0, 0]` |
| 3 | `[0,0872, −0,0437, 0]` | `[0,1734, −0,0622, 0]` | `[0,1091, 0, 0]` |

⇒ **no passo 3 o vértice do alvo recebe outro impulso inteiro em `+x` e ULTRAPASSA o cursor; o nosso
avança zero** (recua `0,006`). ⚠️ **E o mesmo vértice, no mesmo passo, com o mesmo `f` e a mesma
direcção, no modo ARRASTAR recebe o impulso** (`0,0935 → 0,1661`, oráculo `0,1676`) — a nossa
maquinaria de força e de integração está certa; o que muda é a resposta COLECTIVA do aperto.

⛔ **E não é a ordem de resolução** (a hipótese óbvia, porque o aperto puxa tudo para um ponto e o
Gauss-Seidel não comuta). Medido em quatro ordens nossas, `err_max / max_oráculo`:

| ordem | arrastar local | aperto de ponto | aperto de linha |
|---|---|---|---|
| **directa (a nossa)** | **`0,071`** | `1,380` | `1,024` |
| inversa | `0,273` | `0,797` | `0,629` |
| por célula `0,05` | `0,607` | `0,833` | `1,673` |
| por célula `0,20` | `0,374` | `1,259` | `0,748` |

*O arrasto é `4×` a `8×` melhor na nossa ordem que em qualquer outra — a nossa ordem é a do alvo. E
NENHUMA ordem põe os apertos abaixo de `0,6`.* ⇒ há lei por descobrir, não ruído de ordenação.

⭐ E o aperto de LINHA é o mesmo defeito mais fraco: o `_origem` dele lê `0,263` e a curva inteira
bate (Q10 acima), enquanto o `plano_apertar_linha_radial_local` — o mesmo gesto com o pen-down em
`x = −0,305` — lê `1,024`. *A única diferença entre os dois é onde o pen-down cai na grelha, isto é,
qual vértice fica mais perto do cursor.*

### As perguntas

- **Q11.1** — Nos dois modos de aperto, o vértice que está sobre o cursor (distância ≈ `0`) recebe
  força? A direcção «do vértice para o cursor» degenera ali. O alvo trata a direcção nula de alguma
  maneira própria — devolve zero, salta o vértice, usa um mínimo?
- **Q11.2** — A força do aperto é aplicada a partir da posição do vértice ANTES da relaxação deste
  passo, ou depois? (A espec §5.2 diz que a relaxação corre antes da integração; a pergunta é se o
  `f` e o `u` do aperto são avaliados no mesmo instante que os dos modos de arrasto.)
- **Q11.3** — Há no aperto algum limite que o arrasto não tem — um tecto de deslocamento por passo,
  um corte quando o vértice ultrapassa o cursor, um amortecimento próprio?

Contrato de retorno igual ao do Q8.

### Q11 — duas hipóteses minhas, CONSTRUÍDAS, MEDIDAS e REFUTADAS (2026-09-06)

Antes de perguntar, testei as duas coisas que a própria espec deixava em aberto para mim. Registo-as
para não voltarem:

⛔ **(1) O filtro de raio na criação de restrições da área *Dynamic*.** A espec §2.1 diz que no
*Local* a criação é filtrada por `|p⁰ − c| < R₀(1+L)` e no *Dynamic/Global* é **sem filtro** (todos
os vértices das células tocadas, e numa malha pequena a célula é metade dela). A nossa Dinâmica
filtra por um disco de vértices. Tirei o filtro: `plano_arrastar_radial_dinamica` `0,181 → 0,182` e
as outras nove **inalteradas**. ⇒ *quem segura os vértices longe é a banda `w`, exactamente como a
§2.1 diz — o portão grosso da célula não tem efeito observável nesta escala.*

⛔ **(2) O peso da normal por vértice.** Todos os modos de FORÇA são exactos ao bit num traço de UM
passo e derivam ao longo de um traço inteiro, e o que muda entre passos neles é a normal da malha
deformada (o Inflate lê-a por vértice, o Push e o aperto de linha lêem a da área). Troquei o peso do
Newell (por ÁREA) pelo peso uniforme por face: **19 traços medidos, 18 inalterados** e um `0,008`
pior. ⇒ o resíduo dos modos de força não é a normal.

⚠️ **E o que fica NOMEADO por esta medição:** o **arrasto** é o único modo de força que NÃO deriva
(`0,071` num traço de 12 passos), e ele é o único cuja direcção não depende do estado da malha —
`δ̂` é a mesma para todos os vértices. Os outros quatro derivam. *A causa comum tem de estar em algo
que a direcção por-vértice lê e a direcção global não.*

### Q11 — a TERCEIRA hipótese, e o facto que a mata (2026-09-06)

⛔ **A direcção do aperto medida no REPOUSO** (como o Grab faz), em vez de na posição actual:
`plano_apertar_ponto_radial_local` `1,380 → 1,012` e o `_origem` `1,079 → 1,051`, mas a
`esfera_apertar_ponto_radial_dinamica` **piora** de `0,542` para `0,939`. ⇒ refutada.

⭐⭐ **E a medição que estreita a pergunta: o erro por passo.** A sonda passou a imprimir o pior erro
por vértice em cada passo. O ARRASTO acumula e **estabiliza**: `0,000 · 0,006 · 0,011 · 0,013 ·
0,014 · 0,014 …` — um desvio pequeno e constante. O APERTO DE PONTO salta de `0,000` para
**`0,125` num único passo** (o k=3) e depois cresce. *As duas malhas são idênticas ao bit no fim do
passo 2 e o passo seguinte separa-as em `0,125`.* ⇒ não é acumulação nem ruído: é uma lei que muda
quando a malha deixa de estar em repouso.

⚠️ **E a aritmética do vértice diz onde procurar.** No fim do passo 2 o anel imediato de `c0` está em
`0,0894` e o próprio `c0` em `0,0935` — as restrições entre eles estão **quase satisfeitas**, logo a
relaxação do passo 3 mal deveria mexer em `c0`, e o impulso de força (`≈0,09`, que a espec §4.1
fixa em `10·dt/massa`) deveria levá-lo a `≈0,18`. É exactamente o que o oráculo faz (`0,1842`). Em
nós ele acaba em `0,0975`, com uma componente `−0,0437` **perpendicular ao traço** que a força do
aperto não pode produzir (ali `u` é paralelo ao traço). ⇒ *o que nos tira o impulso vem da
RELAXAÇÃO, não da força* — e no arrasto, no mesmo passo e no mesmo vértice, a mesma relaxação
deixa o impulso passar (`0,0935 → 0,1661`, oráculo `0,1676`) e o anel imediato bate **exactamente**
(`0,1585` contra `0,1585`).

⇒ Q11.4 (nova): o que a relaxação faz de diferente num passo de APERTO e num de ARRASTO, sendo o
estado de partida o mesmo? *A única coisa que distingue os dois passos é o campo de força ser
convergente (todos os vértices para um ponto) em vez de uniforme.*

## Q11 — a resposta do E VERIFICADA no nosso lado, e duas réguas minhas refutadas (2026-09-06)

⭐⭐⭐ **A fixtura de força reduzida ILIBA a nossa lei do aperto.** Corrido o
`plano_apertar_ponto_radial_local_origem_fraco` (o mesmo traço, força `1,0 → 0,2`):

| | erro por passo | `err_max / max_oráculo` |
|---|---|---|
| força `0,2` | **`0,000` nos DOZE passos** | **`0,063`** |
| força `1,0` (o irmão) | salta para `0,125` no passo 3 | `1,079` |

⇒ *a lei está certa; o que divergia é o regime em que o alvo deixa de ser determinista.* A fixtura
entrou na lista verde do gate de paridade (`acfee8a6e`), e o censo das duas listas acusou-a sozinho
antes de eu a nomear — que é o que ele existe para fazer.

⛔ **A trava que impediria o vértice de ULTRAPASSAR o alvo: construída, medida, REFUTADA.** Ela não
é inerte — parte os traços de UM passo que hoje saem ao bit (`0,000 → 0,465` no aperto de linha e
`0,000 → 0,811` no de ponto). *O alvo ultrapassa já no primeiro passo e isso é a LEI, não o defeito;
não há guarda local que separe a ultrapassagem fiel do caos que vem depois dela.*

⛔ **E DUAS réguas minhas para classificar os abertos, as duas refutadas** (instrumentadas na
`sonda_dos_artefatos_do_oraculo`, que fica porque a medição fica):

- **contar FACES INVERTIDAS na saída do oráculo** — não discrimina: o arrasto tem `41`–`57` e bate a
  `0,071`; o `plano_arrastar_plano_local` tem `273` e erra `0,233`.
- **a COMPRESSÃO do par mais apertado** (`min D/ℓ`, a grandeza que faz o factor de correcção inverter
  o sinal) — explica a família do APERTO e nada mais: a fixtura fraca lê `0,8316` e o irmão `0,1288`;
  mas o `plano_empurrar_plano_local` lê `0,8929` **sem compressão nenhuma** e erra `0,944`, e o
  `plano_arrastar_plano_local` lê `0,0959` — compressão extrema — e erra `0,233`.

⇒ **os abertos que sobram NÃO são uma família.** Ficam nomeados, com o número: o **Push**
(`0,944` no plano, `0,329` radial, `0,303` na esfera), o **Expand** (`0,557` · `0,192` · `0,560`),
o **Inflate** (`0,378` · `0,253`), o **Snake Hook de 2 passos** (`0,39`–`0,42`) e os modos
não-arrasto na **esfera**. Cada um precisa da sua pergunta.

### E uma QUINTA hipótese refutada, sobre a família do falloff de PLANO (2026-09-06)

Os quatro traços `*_plano_local` são sistematicamente piores que os irmãos radiais
(`empurrar 0,944` · `apertar_ponto 0,613` · `arrastar 0,233` · `agarrar 0,180`), e a §4.4 diz que o
plano de queda passa pelo **centro da área** com normal `δ̂`, enquanto nós o fazemos passar pelo
**cursor**. Trocado para o centro da área: `empurrar 0,944 → 1,250`, `apertar_ponto 0,613 → 0,798`,
`arrastar 0,233 → 0,716`, `agarrar` inalterado. ⇒ **refutada** — o plano pelo cursor é o que
reproduz o alvo, e a frase da §4.4 não se lê como nós a líamos.

⏳ **Pergunta para uma próxima ronda** (não urgente, e nomeada para não se perder): na área *Local*,
o «centro da área» de que a §4.4 fala é a localização inicial fixa (o que a §2.1 define) ou a do
cursor? A medição diz cursor; a espec, lida à letra, diz a inicial.

## Os gates 19, 20 e 21 estão IMPLEMENTADOS — e o 20 precisou de uma correcção medida (2026-09-06)

Os três da emenda Q11 vivem em `crates/ph2d-cloth/tests/oraculo_do_pincel.rs`, com as duas réguas da
§5.2-ter escritas como a espec as define (quadrilátero invertido pela normal de Newell contra o
repouso — ⛔ não a soma das metades triangulares; assimetria de espelho com o numerador em norma do
máximo e o denominador na euclidiana).

- **19 — verde.** O aperto inverte no 1.º passo simulado, o arrasto não, e a fixtura de força fraca
  não inverte em passo nenhum. Mutação que o mata: travar o impulso do aperto à distância que falta.
- **21 — verde.** Fora da inversão o aperto erra `0,063` contra `0,067` do arrasto no mesmo retalho.
  ⚠️ **Ele SOBREVIVEU à primeira mutação que tentei** (a direcção medida no repouso), porque a
  fixtura fraca desloca `0,004` e ali repouso ≈ actual — *um corpus no neutro de um detalhe não
  testa esse detalhe*. A mutação que o mata é a força do aperto pela metade (`0,495` contra `0,067`).

⚠️⚠️ **20 — a barra que a espec propõe NÃO é propriedade de nenhum dos dois lados.** Ela diz *«a
nossa assimetria não pode passar a do oráculo no mesmo passo»*; medido passo a passo no aperto a
força cheia, nós ficamos ACIMA em `k = 5, 7, 11` e ABAIXO nos outros nove:

| k | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|---|
| nós | `0,586` | `0,479` | **`1,979`** | `1,381` | **`0,901`** | `1,106` | `0,726` | `0,706` | **`1,294`** | `0,730` |
| oráculo | `0,675` | `1,218` | `0,931` | `1,463` | `0,856` | `1,316` | `1,448` | `1,169` | `0,608` | `1,060` |

*Uma barra «sempre abaixo» sobre um regime que a própria espec diz ser decidido pela ORDEM é uma
barra que reprova por sorteio.* ⇒ o gate mede **dois regimes**, com as barras derivadas da tabela:

- **passos SEM inversão** (onde a comparação por passo vale): a razão medida é `1,15`–`1,26` no
  arrasto e `1,49`–`1,67` no aperto fraco ⇒ barra **`2,0`**, no vazio entre `1,67` e o `2,12` do
  pior passo do regime caótico.
- **passos COM inversão**: compara-se o **ENVELOPE** do traço, não o passo — `1,979` contra `1,463`,
  razão `1,35` ⇒ barra **`2,0`**. ⚠️ Ele **não** afirma que reproduzimos o alvo ali; afirma que não
  somos pior por uma ordem de grandeza num regime que o alvo também não controla.

⏳ **Para o R-pós / a próxima emenda:** a §14 gate 20 deve passar a dizer isto, com a tabela.
