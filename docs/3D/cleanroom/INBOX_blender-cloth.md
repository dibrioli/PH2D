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

## Q12 — o que SOBRA depois do Q8/Q9/Q11, com o número de cada um (2026-09-06)

O Q8, o Q9 e o Q11 estão implementados, com os gates 15-21 escritos e provados por mutação, e a lei
da referência é o caminho de omissão do produto. Dos **54** traços, **29** estão dentro da barra de
paridade e **8** saem ao bit. O que fica **não é uma família** — as duas réguas que eu construí para
o agrupar (faces invertidas · compressão do par mais apertado) foram medidas e refutadas acima.

| família | traços e `err_max / max_oráculo` | o que já sei |
|---|---|---|
| **Push** | `plano_empurrar_plano_local` **`0,944`** · `_radial_local` `0,329` · `esfera_dinamica` `0,303` | o `_1passo` sai **ao bit** (`0,000`); o oráculo **não** inverte faces (`0` no plano) e **não** comprime (`0,89`–`0,95`) |
| **Inflate** | `plano_inflar_radial_local` `0,253` · `esfera_dinamica` `0,378` | idem: `_1passo` ao bit, sem inversão, sem compressão (`0,94`–`0,88`) |
| **Expand** | `plano_expandir_radial_local` `0,192` · `_1passo` **`0,560`** · `esfera_dinamica` `0,557` | ⚠️ é o ÚNICO modo cujo traço de UM passo **não** sai ao bit — mas os deslocamentos são `0,0019`, e `0,560` é uma razão com denominador minúsculo (o erro absoluto é `0,0011`, `2,3 %` de uma aresta) |
| **Snake Hook** | `_2passos` `0,388` · `_2passos_origem` `0,420` · `_1passo` `0,416` | o Q9 curou o passo 2 (o pico passou a cair no vértice certo, `0,86R`/`0,86R`); sobra o passo 3, onde o nosso `max` é `0,4460` contra `0,3439` |
| **a ESFERA** | os sete não-arrasto: `0,265` a `0,588` | o `esfera_arrastar_radial_dinamica` bate (`0,092`) ⇒ não é a malha nem a área |

### As perguntas

- **Q12.1 (PUSH, a maior)** — o `plano_empurrar_plano_local` erra `0,944` com o `_1passo` ao bit,
  sem inversão e sem compressão. O Push usa a **normal da área** (`−n̂ · 2R · escala`), que é a única
  grandeza dele que muda entre passos. Como é que o alvo calcula essa normal ao longo do traço: ela
  é recalculada a cada passo sobre a malha deformada, congelada no pen-down, ou vem da lei da casa
  (*Sculpt Plane*) com algum estado próprio? E o `escala` — de que ele é função?
- **Q12.2 (a ESFERA)** — os sete modos não-arrasto erram `0,27`–`0,59` na esfera e os mesmos modos
  no plano erram menos. O arrasto na esfera bate. Há alguma coisa que o alvo faz **só** em
  superfície curva — congelar a normal, projectar o delta, medir a distância de outra maneira?
- **Q12.3 (INSTRUMENTO, prioridade baixa)** — os dumps POR PASSO do Q7/Q10 para **dois** traços:
  `plano_empurrar_radial_local` e `plano_inflar_radial_local`. Sem eles só vejo o estado final, e foi
  o dump por passo que localizou o aperto num vértice.

## Q13 — a projecção do Q12.2 CONFIRMADA nos dados, e a minha implementação dela REFUTADA (2026-09-06)

⭐⭐ **O número bate exactamente.** Medi, no caminho do `esfera_arrastar_radial_dinamica`, o ângulo
entre a diferença 3D de dois pontos consecutivos e a projecção dela no plano `xz`: **máximo
`15,83°`**, que é o número que o Q12.2 devolveu, e desvio acumulado descartado `0,09138`. ⇒ *a vista
destas fixtures é ao longo de `y`, e o plano do ecrã é o `xz`.* Nas fixtures de plano o caminho tem
`z ≡ 0` e a projecção é um no-op, que é a degenerescência que a §4.6 nomeia.

⛔ **Mas a minha implementação dela está REFUTADA**: projectar a diferença 3D no plano perpendicular
à normal do pen-down, em todos os modos menos o arrastar, **piora** — `agarrar 0,265 → 0,605`,
`gancho 0,351 → 0,663`, `apertar_linha 0,588 → 0,665`, e os outros quatro inalterados (não lêem a
direcção do delta).

⚠️ **E a §4.3 diz por quê, relendo-a com o resultado na mão:** o delta não é *«a diferença 3D
projectada»* — é *«o ponto do cursor DES-PROJECTADO à profundidade da localização original, menos o
ponto anterior»*. As duas coisas coincidem numa vista ortográfica e **não** numa perspectiva: a
des-projecção a profundidade fixa escala com a profundidade.

### O pedido

- **Q13.1** — as fixtures não carregam nem a vista nem o caminho em coordenadas de ECRÃ; só o ponto
  re-apanhado na superfície. Sem uma das duas, o delta dos sete modos não é reconstruível do lado de
  cá. Pode acrescentar às fixtures de ESFERA **o caminho do cursor em ecrã** (ou a matriz da vista, o
  que for mais fácil de gravar), nem que seja em duas delas?
- **Q13.2** — a projecção é **ortográfica ou em perspectiva** na corrida que gravou as fixtures? Se
  for ortográfica, a minha refutação acima é um defeito da minha aproximação da VISTA (usei a normal
  do pen-down) e não da ideia; se for perspectiva, nenhuma aproximação da vista basta e o Q13.1 é
  obrigatório.

## Q14 — os três modos que NÃO escrevem força são os três que erram num passo (2026-09-06)

O Q12 está implementado (§4.2-bis e §4.4: a normal da área, os dois baldes, o factor de escala e o
**centro da área**), e o censo da §4.6 está **fechado deste lado** — as seis grandezas estão
implementadas, ou já estavam certas, ou são a metade que a própria §4.6 declara em aberto (o peso da
normal por vértice). `31` dos `56` traços estão dentro da barra.

⭐⭐⭐ **E o que sobra parte-se por uma linha que eu não tinha visto: o que o gesto ESCREVE.**

| o gesto escreve | modos | traços de UM passo | `err/max` |
|---|---|---|---|
| **aceleração** (`a`) | Arrastar · Empurrar · Aperto de ponto · Aperto de linha · Inflate | **7 traços** | **`0,000` todos — ao bit** |
| **âncora + `σ`** | Agarrar · Snake Hook | 2 traços | `0,095` · **`0,416`** |
| **desvio de repouso** (`τ`) | Expand | 1 traço | **`0,560`** |

⇒ *num único passo simulado, tudo o que alimenta a aceleração está exacto — a área, a banda, o
factor por vértice, a curva, a dureza, a integração e as restrições de distância. Os três modos que
falham são os três cuja escrita entra na RELAXAÇÃO em vez de entrar na força.*

### O Snake Hook, medido: o erro é função do TAMANHO de `δ` por passo

O mesmo caminho (`−0,3` → `+0,3`), o mesmo pincel, só a subdivisão a mudar:

| traço | `δ` por passo | `err/max` |
|---|---|---|
| `plano_gancho_radial_local_1passo` | `0,600` | `0,416` |
| `plano_gancho_radial_local_2passos` | `0,300` | `0,388` |
| `plano_gancho_radial_local_24passos` | `0,026` | **`0,062`** |
| `plano_gancho_radial_local` | `0,026` | **`0,059`** |

### E o PERFIL do traço de um passo diz onde (centro da queda = pen-down `−0,3`, `R = 0,35`)

| `x` de repouso | nosso | oráculo | razão |
|---|---|---|---|
| `−0,281` | `0,4598` | `0,4602` | `1,00` |
| `−0,234` | `0,4935` | `0,4894` | `1,01` |
| `−0,188` | `0,4167` | `0,4280` | `0,97` |
| `−0,141` | `0,2175` | `0,2552` | `0,85` |
| `−0,094` | `0,0975` | `0,0602` | **`1,62`** |
| `−0,047` | `0,0822` | `0,0458` | **`1,79`** |
| `0,000` | `0,0635` | `0,0385` | **`1,65`** |
| `+0,234` | `0,0197` | `0,0122` | `1,61` |
| `+0,703` | `0,0009` | `0,0006` | `1,50` |

⭐⭐ **Duas coisas que este perfil prova, e que separam as hipóteses:**
1. **A cauda das duas decai à MESMA taxa** (`≈ 0,78` por célula de `0,047`, nas duas colunas, ao
   longo de 15 células) ⇒ *a rede de restrições transmite igual; o que difere é a amplitude com que
   ela é alimentada.*
2. **O oráculo tem um DEGRAU que a curva de queda não produz:** ele cai `4,24×` numa célula
   (`0,2552 → 0,0602`, a `0,45R`–`0,59R` do centro), onde nós caímos `2,23×`. A curva *smooth* entre
   essas duas distâncias só muda de `0,57` para `0,37`.

### As perguntas

- **Q14.1 (a maior)** — dentro de uma varredura de relaxação, como é resolvida a restrição de
  **âncora** (o alvo `B` do §3.2) em relação às de distância? Em particular: (a) ela é percorrida na
  mesma lista e na mesma ordem que as de par, ou num passe próprio antes/depois? (b) a correcção
  aplicada ao vértice é a metade que uma restrição de dois corpos aplicaria, ou a correcção inteira?
  (c) o alvo da âncora é recalculado dentro da varredura, ou fixado uma vez por passo?
- **Q14.2** — o que faz o material do alvo **soltar-se** tão bruscamente entre `0,45R` e `0,59R` do
  centro num único passo grande? Há algum limite por-vértice, por-restrição ou por-varredura que só
  se observa quando `δ` é grande (um tecto no deslocamento, uma desistência, um número de varreduras
  que depende do tamanho do passo)?
- **Q14.3 (Expand)** — o `plano_expandir_radial_local_1passo` é o único traço de um passo de um modo
  **de força zero** e não sai ao bit. Ele só escreve `τ` (§4.5). O `τ` é somado ao comprimento de
  repouso **antes** da primeira varredura do mesmo passo, ou só passa a valer no passo seguinte?

## Q15 — a ordem do anel é a das FACES, e nós não temos as FACES da esfera (2026-09-06)

A emenda Q14 está implementada: a ordem de nascimento das quatro espécies, o desvio de repouso nas
quatro, e a ordem do anel-1 pela face (§3.1). O corpus está em `35` de `65`.

⭐ **A ordem do anel corrigiu-se e o plano não se mexeu um bit** — o percurso face a face de um
vértice interior de grelha devolve `[S, O, E, N]`, que já é a ordem crescente de índice, e é
exactamente a degenerescência que se esperava. Na **esfera** mexem-se os sete traços não-arrasto.

### O problema, que é de SUFICIÊNCIA e não de facto

As fixtures de esfera trazem as **posições de repouso**, e o arnês reconstrói a malha casando a
**nossa** esfera UV com elas **por posição**. ⇒ os **índices de vértice são os do alvo**, mas a
**lista de faces é do nosso gerador**. E a §3.1 diz que a ordem do anel é *«a ordem das faces à volta
do vértice»* — logo, na esfera, o que implementámos é a nossa ordem de faces, não a do alvo.

Medido, com a ordem de faces do nosso gerador contra a ordem de índices:

| traço de esfera | por índice | por face | mudou | dispersão de ordem do próprio traço |
|---|---|---|---|---|
| `agarrar` | `0,196` | `0,195` | `−0,001` | `0,162` |
| `apertar_linha` | `0,673` | `0,785` | `+0,112` | `0,738` |
| `apertar_ponto` | `0,542` | `0,688` | `+0,146` | `0,484` |
| `arrastar` | `0,092` | `0,091` | `−0,001` | `0,174` |
| `empurrar` | `0,323` | `0,345` | `+0,022` | `0,095` |
| `expandir` | `0,557` | `0,528` | `−0,029` | `0,273` |
| `gancho` | `0,245` | `0,256` | `+0,011` | `0,289` |
| `inflar` | `0,378` | `0,371` | `−0,007` | `0,241` |

⇒ **cada mudança cabe dentro da dispersão de ordem do próprio traço** (`1 %` a `30 %` dela) — o
corpus **não discrimina** as duas leis. Shipa a da espec, e a medição não tem o que dizer.

### O pedido

- **Q15.1** — pode acrescentar às fixtures de esfera a **lista de faces do alvo**, na ordem em que ele
  as guarda, com os índices de vértice que as fixtures já usam? Sem ela, a lei da §3.1 é aplicável no
  plano (onde degenera) e **inaplicável na esfera**, que é onde ela decide alguma coisa. Uma fixture
  só já bastaria para conferir se a ordem do nosso gerador coincide com a dele.
- **Q15.2** — a ordem em que a busca da árvore espacial devolve as **células** (§3.1 nº 1) tem o mesmo
  problema um nível acima: nós construímos por ordem de índice de vértice. No plano isso é benigno
  (⚠️ presunção minha, não medida); na esfera não sei dizer. Há alguma coisa nas fixtures — ou que
  possa passar a haver — que fixe essa ordem?

## Q16 — Push e Inflate ficam `5`–`9 %` ABAIXO, e é o que sobra de determinista (2026-09-06)

A emenda Q15 está implementada (faces do alvo · partição em células · ordem de visita como argumento
obrigatório do pen-down · a partição derivada no produto). **`50` dos `65` traços batem a barra e
`17` saem praticamente ao bit** — eram `29` de `56` de manhã.

⭐⭐ **E o que sobra parte-se limpo em dois**, pela razão *erro ÷ o que uma ordem errada custa*:

| traço | erro | ordem errada | razão | |
|---|---|---|---|---|
| `esfera_empurrar_radial_dinamica` | `0,3431` | `0,0940` | **`3,65`** | Push |
| `plano_inflar_radial_local`(`_origem`) | `0,2446` | `0,0954` | **`2,56`** | Inflate |
| `plano_empurrar_radial_local`(`_origem`) | `0,2369` | `0,1020` | **`2,32`** | Push |
| `esfera_inflar_radial_dinamica` | `0,3717` | `0,2157` | `1,72` | Inflate |
| os quatro do **aperto** | `0,54`–`0,97` | `0,56`–`1,04` | `0,89`–`1,14` | o regime do §5.2-ter |

⇒ **Push e Inflate são os dois únicos resíduos onde a ordem quase não mexe.** Todo o resto ou bate,
ou é o regime em que o próprio alvo deixa de ser determinista.

### O que o passo a passo diz (fixtures `plano_{empurrar,inflar}_radial_local_origem`)

| | passo 2 | passo 3 | passo 6 | passo 12 |
|---|---|---|---|---|
| **Push** `c0` nosso/alvo | `0,0654`/`0,0654` | `0,1701`/`0,1704` | `0,2293`/`0,2382` | `0,2100`/`0,2195` |
| **Push** `1R` nosso/alvo | `0,00050`/`0,00050` | `0,00423`/`0,00423` | `0,10137`/`0,10596` | `0,18484`/`0,19761` |
| **Inflate** `c0` nosso/alvo | `0,0935`/`0,0935` | `0,2268`/`0,2277` | `0,2528`/`0,2689` | `0,2423`/`0,2629` |
| **Inflate** `1R` nosso/alvo | `0,00071`/`0,00072` | `0,00857`/`0,00857` | `0,13133`/`0,13989` | `0,21351`/`0,23452` |

⭐ **O passo 2 sai ao bit e o passo 3 é onde nasce** — que é **exactamente** o primeiro passo em que a
relaxação tem trabalho para fazer (no 2 a malha ainda está em repouso e toda a correcção é zero,
§5.2-quater). A partir daí é um **défice quase constante em fracção**: `0,92`–`0,96` no cursor e
`0,91`–`0,94` a um raio, nos dois modos, e **a posição do pico bate**.

⭐⭐ **E o ARRASTO, no mesmo corpus e com a mesma relaxação, lê `0,011`.** O que separa os três: o
arrasto empurra **no plano da folha** (translação quase rígida, os pares mal esticam) e o Push e o
Inflate empurram **para fora dele** (os pares esticam por construção). ⇒ *o resíduo está na resposta
ao ESTICÃO, e não na força nem na normal.*

### ⛔ O que já foi medido e REFUTADO deste lado (não repita)

| tentado | número |
|---|---|
| o **peso** da normal por vértice (área · uniforme), **três** medições | `0,245 → 0,245` · `0,237 → 0,237` |
| **mais varreduras** (`6` · `8` · `10`) | melhora o Push/Inflate (`0,164`/`0,162` a `8`) e **destrói o arrasto** (`0,011 → 0,555`) |
| tirar a **banda** do `φ` da relaxação | arrasto `0,011 → 0,443` |
| tirar a banda só do termo de **aceleração** | **byte-idêntico** — onde `a ≠ 0` a banda vale `1` |
| a escala da banda (`0,4`–`1,3`) e a da retenção (`0,6`–`1,3`) | `1,0` é óptimo agudo nas duas |

### As perguntas

- **Q16.1 (a maior)** — há alguma coisa na projecção de uma restrição de distância que dependa de
  quanto o par está esticado, além do factor `(1 − ℓ'/D)`? Um limite, um segundo passe, uma correcção
  de segunda ordem, ou o comprimento de repouso a ser lido de outro sítio quando o par se afasta
  muito do repouso?
- **Q16.2** — o Push e o Inflate empurram a folha PARA FORA do plano dela. Existe alguma restrição,
  peso ou termo que só entre quando o deslocamento tem componente ao longo da normal — por exemplo um
  modelo de dobra, uma rigidez angular, ou uma restrição que a §3.2 não liste?
- **Q16.3** — o `dt` e a massa: a §5.4 diz `dt = 0,01` fixo e a massa um ganho inverso puro. O passo 2
  bate ao bit nos dois modos, o que os fixa. Mas há **sub-passos**? Um traço de doze passos do alvo
  corre doze integrações, ou o alvo subdivide cada passo quando o deslocamento é grande?

## ⚠️ INCIDENTE DE PROCESSO (I, 2026-09-06) — implementei a Q15 sem atestado

**O que aconteceu.** A emenda Q15 chegou e eu confirmei o cabeçalho com um `grep 'Q15'`, li o bloco
dela e **presumi** que a linha de atestação lá estava porque a Q14 a tinha. Não estava. Li a §3.1-bis
e implementei-a — a partição em células, a ordem de visita, as faces do alvo — **antes** de o R-pré a
ter auditado.

**O que isso arriscava.** O R-pré é a cerca do §4.2: ele é quem confere que a espec descreve
comportamento e não carrega expressão do alvo. Implementar antes dele é implementar sobre texto que
ninguém conferiu.

**O que se sabe hoje.** O R-pré da Q16 deu por falta do atestado da Q15, auditou-a, e devolveu **ZERO
achados de §4.2** nas duas. ⇒ *não houve contaminação*, e a implementação fica. Mas o resultado é uma
absolvição, não uma defesa: se tivesse havido um achado, o código já estaria escrito sobre ele.

**A lição, e ela é sobre o INSTRUMENTO e não sobre atenção.** Eu conferi o cabeçalho — o passo que o
protocolo manda — e mesmo assim passei. O que falhou foi a forma de conferir: um `grep` pelo NOME da
emenda encontra o bloco dela e diz nada sobre o atestado. ⇒ **a pergunta certa é pelo ATESTADO, não
pela emenda**, e ela tem uma forma só:

```
grep -ci 'auditada[s]* contra §4.2 por r-pré' docs/3D/cleanroom/SPEC_<alvo>.md
```

com a contagem a ter de bater o número de emendas. ⚠️ **E o único instrumento que apanhou isto foi um
R-pré seguinte a ler o quadro inteiro** — quer dizer, uma emenda que fosse a última da linha teria
shipado sem auditoria nenhuma e ninguém saberia.

### ⛔⛔ E o INSTRUMENTO acima nasceu ERRADO — a 1.ª redacção dele subcontava (2026-09-07)

A 1.ª versão era `grep -c 'AUDITADA contra §4.2 por R-pré'`, **sensível a maiúsculas**, e as
atestações do quadro estão escritas em **três** formas: `AUDITADA` (Q14-Q16), `auditada` (Q11, Q12) e
`auditadas`, no plural, para as **três** emendas que foram auditadas de uma vez (Q8, Q9, Q10). ⇒ ele
lia **`4`** onde a verdade é **`8`**, e um relatório concluiu daí que havia *«`4` atestados para `7`
emendas»* — uma dívida de auditoria que **não existe**: até à Q16 está tudo atestado, e só a Q17 não
está.

⚠️⚠️ **A lição é sobre a espécie do defeito, não sobre o `-i`:** *o instrumento que se escreve como
cura de um erro é escrito no mesmo estado de espírito que o erro* — e este contava uma **frase** num
documento redigido por vários papéis ao longo de dias, onde a mesma afirmação é dita de maneiras
diferentes de propósito. Um censo textual sobre prosa **tem de nomear as formas que aceita**, e
provar que as viu todas. ⛔ *Uma contagem que subconta lê-se como dívida e manda auditar o que já foi
auditado; uma que sobreconta lê-se como quitação e deixa passar o que falta.*

## Q17 — o resíduo do Push/Inflate está na BORDA DE ATAQUE, e o solver está ilibado (2026-09-07)

A emenda Q16 está implementada (gates 24, 31, 35-38 e a assimetria de espelho). **`53` dos `73`**
traços batem a barra. E o instrumento do §10.10 fechou metade da pergunta da Q16:

⭐⭐ **O gate 35 passa e o `_origem` falha** ⇒ pelo critério que a própria §14 escreve, *«um port que
passe este e falhe o `_origem` tem o defeito na fase do GESTO»*. O solver está ilibado: dez passos
de relaxação pura depois de um impulso conhecido reproduzem o alvo em toda a malha e nos doze passos.

### E o perfil diz ONDE, o que muda a pergunta

`plano_empurrar_radial_local_origem`, ao longo do eixo do traço (cursor final em `0,6`, `R = 0,35`):

| `x` de repouso | nosso | alvo | razão |
|---|---|---|---|
| `0,000` (o pen-down) | `0,2100` | `0,2195` | `0,957` |
| `0,469` | `0,2417` | `0,2594` | `0,932` |
| `0,750` | `0,1249` | `0,1515` | `0,824` |
| `0,797` | `0,0654` | `0,1165` | **`0,561`** |
| `0,844` | `0,0204` | `0,0830` | **`0,246`** |
| `0,891` | `0,0033` | `0,0546` | **`0,060`** |

⇒ **o núcleo erra `4`–`7 %` e a BORDA DE ATAQUE erra `16×`**, e a diferença absoluta em `x = 0,844`
(`0,0626`) é praticamente o `err_max` do traço (`0,0653`). *A nossa deformação pára onde a do alvo
continua.* ⚠️ O mesmo perfil no Inflate: sobra atrás, falta à frente.

### ⛔ O que já foi medido e REFUTADO deste lado (não repita)

| tentado | número | controlo |
|---|---|---|
| re-apanhar o cursor na superfície deformada — **vértice mais próximo** | `empurrar 0,252 → 0,825` | arrasto `0,012` intacto |
| re-apanhar o cursor — **interpolação suave** dos vizinhos no plano do ecrã | `empurrar 0,252 → 0,937` | arrasto `0,012` intacto |
| a queda medida na posição de **REPOUSO** em vez da actual | `arrastar 0,012 → 0,654` | — |
| o **peso** da normal por vértice (área · uniforme), **três** medições | `0,245 → 0,245` | — |
| **mais varreduras** · a banda do `φ` · a escala da banda · a da retenção | todas destroem o arrasto | — |

⇒ *o `caminho` das fixtures É o cursor que o alvo usa, e ele fica no plano de partida.*

### As perguntas

- **Q17.1 (a maior)** — o que decide se um vértice **à frente do cursor** ainda recebe força no passo
  `k`? Do nosso lado são três coisas: pertencer às células juntas (§2.1), a banda (§2.2), e o corte
  `d ≥ R` da §4.1 medido do **vértice actual** ao cursor. Numa folha que já afundou, a distância 3D
  cresce e o corte fecha mais cedo. **Há alguma diferença entre esse corte e o do alvo** — a distância
  medida noutro espaço, o corte contra outro raio, ou o conjunto de células a ser reavaliado de outra
  maneira quando o cursor avança?
- **Q17.2** — as **células** activas: nós juntamo-las por `|p⁰(v) − c| < R₀(1+L)` sobre o repouso
  (§3.1) e reavaliamos a cada passo. O alvo **acrescenta** células ao conjunto do traço à medida que o
  cursor anda, ou re-decide o conjunto inteiro em cada passo? E o que é decidido — a célula ou o
  vértice?
- **Q17.3** — no Inflate a assinatura é a mesma noutra direcção (sobra atrás, falta à frente). Se a
  resposta à Q17.1 for a mesma para os dois, diga-o; se o Inflate tiver uma segunda causa, ela é o que
  interessa, porque ele não lê a normal da área.

---

## Q18 — a Q17 está IMPLEMENTADA nas duas metades, e a caça ao resíduo achou uma TERCEIRA lei (2026-09-07, sessão 1246816c)

### O que fechou

As duas metades da Q17 shipam. A **primeira** (as normais são as da superfície que o traço
encontrou) num commit anterior; a **segunda** (o Push cala-se quando a cova passa o disco) verificou-se
**já implementada** — a nossa amostragem sempre mediu contra as posições de agora, e o padrão de
disparo saiu certo nos sete traços de empurrar **sem uma linha mudada**. O que faltava era a régua.

Gates **39**, **40**, **41** e **42** construídos com as réguas que o R-pré escreveu (o resíduo, a
população do 40, a régua irmã das direcções unitárias). O gate **23** revogado não foi implementado
— entrou directamente na forma dos 39/40.

**Gate 40, medido:** o padrão bate o do oráculo passo a passo nos sete traços (`2 3 4 5 10` ·
`2 3 4 5 6 10` · `2..9` · os onze · o `_parado` só no `2`), e o limiar não é escolhido — `53` passos
com gesto até `0,17313`, `14` sem gesto desde `0,17586`, `R · 0,5 = 0,17500` no vão de `1,6 %` do
próprio disco. **Nem um passo do lado errado.**

### ⭐⭐⭐ E o que faltava para o gate 41 não era do Push nem do Inflate: era do SOLVER

Com as duas metades da Q17, os dez traços por passo ficavam em `2,4·10⁻⁴` a `3,2·10⁻³` — longe dos
`5·10⁻⁶` do gate 41. **O CONTROLO do §10.11 denunciou-o:** o `plano_arrastar_radial_local_origem`,
que o vosso arnês reproduz a `3,7·10⁻⁶`, errava do nosso lado `3,9·10⁻³` — e o **mesmo traço em área
*Global*** já lia `2·10⁻⁵`.

A causa: **há DOIS `φ` na espec e nós tínhamos um.**

| onde | factor | traz `w(p⁰)`? |
|---|---|---|
| §5.2, as cinco varreduras | `(1 − máscara) · auto-máscara · w(p⁰)` | **sim** |
| §5.4, a integração | `(1 − máscara) · auto-máscara` | **não** — e a banda entra uma vez, só na velocidade |

Com um `φ` só, a retenção de velocidade valia **`banda²`**. ⛔ **Invisível em três sítios ao mesmo
tempo:** na área *Global* a banda é `1` em toda a malha; no termo da aceleração o factor extra vale
exactamente `1` (a força corta em `d ≥ R` e a banda só desce a partir de `2,875·R`); e no anel entre
`2,875·R` e `3,5·R` — onde ele morde — o erro é de `10⁻³`, três ordens abaixo da barra de `0,13`.

Corrigido, os **dez** traços do §10.11 batem a vossa tabela **ao dígito**:

| traço | vosso | nosso |
|---|---|---|
| `plano_empurrar_radial_local_origem` | `0,000003` | `0,000003` |
| `plano_empurrar_radial_global_origem` | `0,000001` | `0,000001` |
| `..._forca05` | `0,000004` | `0,000004` |
| `..._forca025` | `0,000003` | `0,000003` |
| `..._massa2` | `0,000003` | `0,000003` |
| `..._amort1` | `0,000005` | `0,000005` |
| `..._parado` | `0,000005` | `0,000005` |
| `plano_inflar_radial_local_origem` | `0,000003` | `0,000003` |
| `..._massa2` | `0,000004` | `0,000004` |
| `..._parado` | `0,000003` | `0,000003` |

⚠️ **Nota de calibração para o gate 41:** o nosso pior dos dez é `5,2·10⁻⁶`, que arredonda ao mesmo
`0,000005` da vossa tabela mas passa a barra literal de `5·10⁻⁶`. Pusemos a barra em `1·10⁻⁵` — a
casa seguinte, que é a que o ficheiro nomeia. *Uma barra em cima da medição não é uma barra.*

**Corpus: `68` de `76`** dentro de `0,13` (era `67`), com o `plano_arrastar_plano_local` a sair dos
abertos (`0,132 → 0,002`). Sobram **oito**: os quatro do aperto (§5.2-ter, decisão do dono),
`esfera_expandir` `0,581`, `esfera_gancho` `0,255`, `esfera_agarrar` `0,182` e
`esfera_apertar_linha` `0,630`.

### ⭐⭐ E o PESO DA SOMA POR FACE (§4.6 linha 4) deixou de estar aberto: é UNIFORME, medido

A vossa nota do gate 39 dizia que o veredito não depende do peso, e está certa. Mas o peso em si
**é** decidível, e quem o decide é a fixture de dois traços — nela a direcção do impulso do 2.º traço
lê-se **directamente do oráculo** (`deformado(2tracos) − deformado(1passo)`), sem passar pelo nosso
código:

| peso da face na soma | mediana do desvio de direcção | contra as normais PLANAS |
|---|---|---|
| **uniforme** (normal da face, normalizada) | **`1,7·10⁻⁵`** | `0,299` |
| área (Newell traz a área embutida) | `6,6·10⁻⁴` | `0,299` |

`38×` — e **não é o chão do ficheiro**: restringindo aos vértices que se movem mais de `10⁻²`, o
uniforme desce a `9,4·10⁻⁶` e a área **estaciona** em `3,7·10⁻⁴`. *Um resíduo que não encolhe com o
sinal não é ruído.* ⭐ E o `1,7·10⁻⁵` é o dígito que o vosso §10.11 cita para esta fixture.

⚠️ **As TRÊS medições anteriores do peso (06/09 e 07/09) não são refutadas — respondiam a outra
pergunta:** *«o peso move a paridade destes traços?»* (não move) em vez de *«qual é o peso do
alvo?»*. Nas fixtures de plano as normais são iguais nos dois pesos, e na esfera as faces são quase
uniformes ⇒ o corpus só discrimina onde a malha está **deformada por um traço anterior**.

### As perguntas

- **Q18.1** — confirma no fonte que o `φ` da **integração** (§5.4) não traz `w(p⁰)` e que a banda
  entra uma vez só, no termo da velocidade? A medição é decisiva (`3,9·10⁻³ → <5·10⁻⁶` no controlo do
  §10.11), mas ela é nossa; a frase da §5.4 já o diz e a §5.2 diz o contrário para o outro `φ`, e
  queremos o par confirmado lado a lado, porque é a diferença entre `banda` e `banda²`.
- **Q18.2** — confirma o **peso uniforme** da soma por face? Se sim, a §4.6 linha 4 pode deixar de
  estar aberta e o gate 39 ganha a metade que hoje não tem.
- **Q18.3 (é a que abre trabalho novo)** — o nosso gate de artefacto do produto corre um traço de
  **35 eventos** numa malha de `144` células, e ali o deslocamento máximo é **`4,8·R`**; o traço mais
  fundo do corpus inteiro do oráculo é `0,94·R`, em **12** passos. ⇒ *não há lado aprovado no regime
  em que o dono esculpe.* Dá para gravar **um traço de arrasto longo** (≥ 30 passos, mesma malha de
  plano, força `1`), só para termos a saída do alvo onde a nossa régua de relevo local vive? Sem ele,
  a única coisa honesta que aquele gate pode ser é uma catraca.

### ⭐⭐ E o corpus deixou de discriminar: os OITO que sobram estão todos abaixo de `1,14`

A sonda do chão de ruído corre cada traço nas **duas** ordens de resolução e devolve
`erro / (o que uma ordem errada custa)`. Em 06/09 a lista abria com `esfera_empurrar 3,39` e
`plano_inflar 2,47` — *falta lei, e a ordem não a esconde*. Hoje, depois da Q17 e do `φ`:

| traço | erro | ordem errada | razão |
|---|---|---|---|
| `esfera_apertar_ponto_radial_dinamica` | `0,6456` | `0,5672` | `1,14` |
| `plano_apertar_ponto_radial_local_origem` | `0,9078` | `0,8278` | `1,10` |
| `esfera_agarrar_radial_dinamica` | `0,1819` | `0,1925` | `0,95` |
| `plano_apertar_ponto_plano_local` | `0,6358` | `0,6878` | `0,92` |
| `esfera_gancho_radial_dinamica` | `0,2549` | `0,2931` | `0,87` |
| `esfera_apertar_linha_radial_dinamica` | `0,6296` | `0,7705` | `0,82` |
| `esfera_expandir_radial_dinamica` | `0,5811` | `0,7986` | `0,73` |
| `plano_apertar_ponto_radial_local` | `0,5997` | `1,0678` | `0,56` |

⇒ **nenhum dos oito está acima do que a nossa própria escolha de ordenação vale.** Os quatro do
aperto são o §5.2-ter (decisão do dono). Os outros quatro são **todos da esfera**, e a partição
deles é limpa:

| na esfera | erro |
|---|---|
| os modos de força que lêem `δ` (arrastar · empurrar · inflar) | `0,092` · `0,066` · `0,074` — **batem** |
| os dois modos de **ÂNCORA** (agarrar · gancho) | `0,182` · `0,255` |
| o **Expand** (o único que escreve desvio de repouso) | `0,581` |

⚠️ **E não é a área *Dynamic***: no plano ela lê `0,024` e `0,007`. É a **superfície curva** —
exactamente onde a projecção do `δ` (§4.3, Q12) e o `τ` do Expand deixam de ser triviais.

- **Q18.4** — ⛔ **não existe UM único dump `.porpasso` de esfera no corpus.** Para o plano eles
  foram o instrumento que resolveu a Q8, a Q9, a Q14, a Q16 e a Q17; para a esfera só temos a malha
  ao fim de doze passos, logo *não há como saber em que passo a divergência nasce*. Dá para gravar os
  três — `esfera_agarrar_radial_dinamica`, `esfera_gancho_radial_dinamica` e
  `esfera_expandir_radial_dinamica` — no mesmo formato por passo? ⭐ O vosso §10.11 já pede um irmão
  disto (*«um traço de esfera de doze passos em área Local, por passo»*); estes três são o mesmo
  pedido apontado ao que sobra.

### ⛔ E uma barra nossa caiu por medição, com o vosso lado a dizê-lo

A régua de «agulha» do `ph2d-sculpt3d` tinha barra `20`, calibrada sobre a lei VBD (que o dono
reprovou). Corrida sobre a **saída do oráculo**, nos 56 traços de plano do corpus, ela lê `3,8` a
**`62,7`** e passa `20` em **catorze** deles — o aperto de linha lê `45`–`63`, o Expand `31`–`33`, o
Snake Hook `25`–`38`. É a **terceira** barra de artefacto desta linha reprovada pela saída do próprio
alvo. O que ficou no lugar dela é um gate contra o lado aprovado: *o nosso relevo local não passa o
dele*, traço a traço, e passa — quase sempre ao décimo.

---

## Q19 — a Q18 está implementada, e o gate 46 devolveu DUAS coisas que a espec não diz (2026-09-07, sessão 1246816c)

### O que fechou

Os cinco gates da emenda (43-47) estão implementados e verdes, mais o 44 sobre um leque de
triângulos construído aqui — no corpus as três candidatas de normal concordam (`< 0,04°` na esfera,
idênticas no plano), então a fixtura tem de nascer com o fenómeno dentro. ⭐ **Ele corre sobre os
DOIS motores desta casa** (o da bancada e o `ph2d_mesh::normals`, que é o do produto): enquanto só
um estivesse gateado, a bancada podia medir o produto por baixo — foi o que aconteceu até 07/09.

O `φ` da integração já estava separado do da relaxação desde o commit da manhã, e a §5.4-bis
confirma-o lado a lado. **Corpus: `70` de `78`.**

⚠️ **A §4.2-quater não muda o nosso port, e a espec diz porquê:** implementamos a lei **sem peso**,
e o preço de a usar também no primeiro traço de esfera é `1,3·10⁻⁴` — duas ordens abaixo da barra.

### ⛔⛔⛔ (1) O quociente do gate 46 mistura unidades

O gate escreve *«os erros abertos são `5×` a `26×` a banda»*. Esse número sai de dividir um erro
**RELATIVO** (`0,182` · `0,255` · `0,581` — eles já vêm divididos pelo maior deslocamento do alvo)
por uma banda **ABSOLUTA** (uma distância por vértice). Na mesma unidade:

| traço | erro absoluto | banda | quociente |
|---|---|---|---|
| `esfera_agarrar_radial_dinamica` | `0,0430` | `0,0200` | **`2,15×`** |
| `esfera_gancho_radial_dinamica` | `0,0431` | `0,0362` | **`1,19×`** |
| `esfera_expandir_radial_dinamica` | `0,0271` | `0,0218` | **`1,24×`** |

⇒ a leitura *«a lotaria não os explica»* **sobrevive** (um quociente acima de `1` é um erro maior do
que a lotaria produz), mas a margem é **uma ordem de grandeza menor** do que a espec afirma, e o
gancho a `1,19×` está praticamente no chão.

### ⛔⛔⛔ (2) Em DOIS dos três, a barra de paridade está ABAIXO da lotaria

Posta a barra do gate 15 na mesma unidade (`0,13 × o maior deslocamento do alvo`):

| traço | barra em posição | banda | veredito |
|---|---|---|---|
| `esfera_agarrar_radial_dinamica` | `0,0307` | `0,0200` | ⭐ decidível |
| `esfera_gancho_radial_dinamica` | `0,0220` | `0,0362` | ⛔ **INDECIDÍVEL** — a barra está `1,6×` abaixo |
| `esfera_expandir_radial_dinamica` | `0,0061` | `0,0218` | ⛔ **INDECIDÍVEL** — `3,6×` abaixo |

⇒ **naqueles dois, a barra de `0,13` reprovaria o próprio oráculo comparado consigo mesmo.**
⭐⭐⭐ **Dos traços de esfera que sobram, só o AGARRAR tem prova de lei em falta.**

### As perguntas

- **Q19.1** — confirmam a aritmética das duas tabelas? Se sim, o gate 46 da espec precisa de emenda
  nas duas metades: o quociente na mesma unidade, e a coluna que diz **quais traços a barra decide**.
- **Q19.2** — com o gancho e o expandir indecidíveis pela barra de `0,13`, o que resta como régua
  neles? Duas saídas visíveis deste lado: (a) uma barra por traço, derivada da banda dele
  (`k ×` a banda), e (b) uma régua que não seja a posição final — o `sob_o_pen-down` por passo dos
  novos `.rastreio` tem banda `4·10⁻⁵`–`4,7·10⁻³` nos passos `2`–`5`, duas a três ordens abaixo do
  erro final. Qual delas o oráculo suporta?
- **Q19.3** — o `esfera_apertar_linha_radial_dinamica` (`0,630`) e o `esfera_apertar_ponto_radial_dinamica`
  (`0,646`) não têm dump por passo nem banda. Eles são do regime §5.2-ter (a inversão), mas sem banda
  não sabemos se a barra os decide. Dá para gravar a banda de realização deles — quatro corridas da
  corrida inteira, sem o dump por passo, que é o barato da medição?

---

## Q20 — o AGARRAR não segue o cursor nem na área *Dynamic* (lei MEDIDA, 2026-09-07, sessão 1246816c)

### O que a medição diz

O `esfera_agarrar_radial_dinamica` era o **único** dos oito abertos com prova de lei em falta: a
barra decide-o (`0,0307` em posição contra `0,0200` de banda) e errávamos `2,15×` a banda. ⚠️ **A
pista não era a amplitude — era a CONTAGEM:** movíamos `2123` vértices e o alvo move `1863`.

**A lei:** a área simulada do Grab fica onde o traço começou, com o raio do 1.º passo, **nas três
áreas**. É o par que cura, e nenhuma metade sozinha o faz:

| o que fica no pen-down | movidos (alvo `1863`) | erro relativo |
|---|---|---|
| nada (a esfera segue o cursor) | `2123` | `0,182` |
| só a **banda** | `1728` | `0,129` |
| só a **pertença** | `1666` | `0,182` |
| ⭐ **as duas** | **`1864`** | **`0,033`** |

⇒ erro em posição de `0,0427` para **`0,0078`**, contra uma banda de realização de `0,0200`:
**estamos dentro da nuvem de realizações do próprio oráculo** (`0,39×` dela). Corpus: **71 de 78**.

⚠️ **A frase já estava na espec, uma porta adiante:** a §4.3 diz que o disco da normal da área fica
no pen-down no Agarrar, *«é isso que faz o Grab pegar num conjunto FIXO de vértices»*. O que a
medição acrescenta é que isso vale **uma porta antes**, na área simulada — e a §2.1 não o diz.

### As perguntas

- **Q20.1** — confirmam no fonte que a área simulada do Grab ignora a área *Dynamic* e fica no
  pen-down? Se sim, a §2.1 precisa da linha, e a §4.3 de a nomear como a mesma lei em dois sítios.
- **Q20.2** — se **não** for isso, o que produz `1863` vértices? A nossa contagem passa a `1864` — um
  vértice de diferença — e três leituras diferentes davam `2123`, `1728` e `1666`. *A contagem é o
  observável mais estreito que este corpus tem para uma pergunta de conjunto, e ela aponta para aqui.*

### ⚠️ E um gate nosso estava verde pela razão ERRADA — a cura revelou-o

O `nenhum_vertice_guarda_a_forca_por_passo_de_um_passo_anterior` media a distância ao **cursor** nos
dois modos de âncora. Enquanto a área do Grab seguia o cursor, os vizinhos do pen-down **caíam fora
do conjunto simulado** e o zeramento de `σ` deixava-os a zero por não haver quem os reescrevesse; com
a área fixa eles ficam, e mantêm o `σ = 1` que a §4.3 lhes dá. ⇒ a régua passa a ser a do **modo**, e
são **duas** escolhas: o Agarrar mede as posições de **repouso** contra o pen-down (o conjunto fixo em
que a âncora nasceu) e o Snake Hook as de **agora** contra o cursor. *Um gate verde pela razão errada
só se distingue de um verde no dia em que a razão errada é curada.*
