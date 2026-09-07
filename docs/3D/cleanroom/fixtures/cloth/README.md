# Fixtures — traços do pincel de tecido do ORÁCULO, sobre malhas NOSSAS

⭐ **Estes arquivos são os vectores de teste da [`SPEC_cloth_brush.md`](../../SPEC_cloth_brush.md) §10**
— um traço scriptado por modo de deformação (e por variante de solver), com as posições de
repouso e as posições depois do traço.

## Proveniência (SKILL_Cleanroom §5 — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malhas de entrada** | ⭐ **nossas**, geradas pelo próprio harness: uma grelha plana `64×64` de lado `3,0` (4 225 vértices) e uma esfera UV `96×64` de raio `1` (6 082 vértices). ⛔ Nenhum asset do alvo |
| **Quem calculou** | o binário Blender 5.2.1 LTS, corrido pelo E **fora da árvore** (`~/Referencias/blender-cloth/oracle/`, ⛔ negado ao I) com um traço scriptado; o pincel usado é um preset do binário **só para existir um pincel de tecido activo** (a API não deixa criar+activar um de raiz), com TODOS os parâmetros reescritos para os valores do cabeçalho de cada fixture |
| **Estatuto legal** | ⭐ **dados** — «the output from the Program is covered only if its contents constitute a work based on the Program» (GPLv2 §0): posições de vértices de uma malha nossa não são |
| **Regenerar** | ⛔ acto de **E**, nunca do I (o harness vive na zona negada). O I pede pelo Enio, como emenda |
| **Data** | 2026-09-05 |

## O traço

Vista ortográfica; o cursor anda em linha recta ao longo de `+X`, comprimento `0,6`, em `passos`
passos iguais (o 1.º passo nunca simula — espec §1); no plano, sobre a face de cima (`z = 0`);
na esfera, sobre o equador visível (`y < 0`). Raio do pincel em espaço de objecto `0,35`
(≈ 7,5 arestas da grelha); força `1,0`, pressão `1`, curva *Smooth*, dureza `0`,
área *Local* (salvo indicação), limite `2,5`, banda `0,75`, massa `1`, amortecimento `0,01`,
plasticidade `0`, pino desligado, sem colisões, sem gravidade — i.e., **as omissões do código**
(espec §8.1), não as dos presets (§8.2).

⚠️⚠️ **As excepções a este parágrafo são VINTE E QUATRO de `78`, e a régua está escrita aqui porque
ela é metade da conta** (2026-09-06 — ⛔ leia sempre o cabeçalho da fixture, nunca esta lista):
compara-se o cabeçalho de cada `.deformado.txt.gz` com **as nove grandezas que o parágrafo fixa sem
ressalva** (`raio · limite · banda · massa · amortecimento · plasticidade · pino · força · curva`)
mais o **percurso**, medido como `max(x) − min(x)` sobre as linhas `c`, mais o **número de traços**
(a chave `tracos`, ausente = `1`); a **área** fica de fora
porque o parágrafo já lhe põe o «salvo indicação», e por ela `17` das `78` não são *Local*.
⭐ **As duas fixtures de 2026-09-07 (os traços LONGOS) NÃO são excepção em grandeza nenhuma** — só
em `passos` (`36`, que o parágrafo não fixa) e, numa delas, na área; o percurso continua a ser `0,6`
nas duas.

| grandeza | quantas | quais |
|---|---|---|
| **força** ≠ `1,0` | `6` | `plano_apertar_ponto_radial_local_origem_fraco` (`0,2`) · `plano_arrastar_radial_local_forca05` · `…_forca05_1passo` · `plano_expandir_radial_local_origem_1passo_forca05` (`0,5`) · `plano_empurrar_radial_local_origem_forca05` (`0,5`) · `…_forca025` (`0,25`) |
| **amortecimento** ≠ `0,01` | `6` | `plano_arrastar_radial_local_amort05` · `…_amort1` · `plano_agarrar_radial_local_amort06` · `…_preset` · `plano_gancho_radial_local_amort06` · `plano_empurrar_radial_local_origem_amort1` |
| **massa** ≠ `1` | `4` | `plano_arrastar_radial_local_massa2` · `…_massa2_1passo` · `plano_empurrar_radial_local_origem_massa2` · `plano_inflar_radial_local_origem_massa2` |
| **percurso** ≠ `0,6` | `3` | `plano_gancho_radial_local_origem_1passo_curto` (`0,05`) · `plano_empurrar_radial_local_origem_parado` (`0,0545`) · `plano_inflar_radial_local_origem_parado` (`0,0545`) |
| **plasticidade** ≠ `0` | `2` | `plano_arrastar_radial_local_plast05` · `plano_arrastar_radial_dinamica_preset` |
| **limite** ≠ `2,5` | `1` | `plano_agarrar_radial_local_preset` (**`5,0`**) — ⭐ load-bearing: é o **segundo ponto de `L`** que refuta a leitura antiga da banda (espec §2.2) |
| **pino** ligado | `1` | `plano_arrastar_radial_local_pino` |
| **curva** ≠ *Smooth* | `1` | `plano_gancho_radial_local_origem_1passo_constante` |
| **traços** ≠ `1` | `1` | `plano_inflar_radial_local_1passo_2tracos` (**`2`**) — ⭐ load-bearing: é a única fixture do corpus em que o pincel encontra a malha **já deformada por um traço anterior**, e é ela que fixa a que superfície pertencem as NORMAIS que o gesto lê (espec §4.2-ter) |

⛔⛔ **Esta conta esteve em `SETE` e a régua é que estava errada, não o número:** ela varria só
`força · curva · percurso · limite` e deixava de fora `amortecimento`, `massa`, `plasticidade` e
`pino` — **nove** fixtures que o mesmo parágrafo também descreve mal. *Uma lista de excepções sem a
régua ao lado não é auditável, e quem acrescenta um traço herda a régua que não vê.*
⚠️ **E ela cresceu outra vez em 2026-09-07, pelo mesmo motivo:** a régua varria nove grandezas do
cabeçalho e o **número de traços** não era uma delas — a fixture de dois traços escondia-se debaixo
da frase «um traço scriptado por modo de deformação» sem acusar nada. *Uma grandeza nova no
cabeçalho é uma coluna nova na régua, no mesmo commit.*
⚠️ E o percurso mede-se pelo **vão em `x`**: na esfera os pontos vivem no equador, logo a
poli-linha entre eles mede `0,6093` — quem contar comprimento de caminho acusa os oito traços de
esfera e não é isso que o parágrafo diz.

⚠️ **Duas coisas do harness que mudam a leitura de uma fixture:**
- o centro da área *Local* é o ponto da superfície sob o cursor **no hover antes do pen-down** — o
  harness move o cursor do sistema para o pixel do pen-down e deixa a janela redesenhar antes do
  traço, e é por isso que o centro coincide com o 1.º ponto do caminho (sem isso, ficava num ponto velho
  e a simulação nascia noutro sítio — foi medido e a matriz refeita);
- nas variantes `_1passo` o caminho tem **dois** pontos, logo nos modos de âncora (Grab, Snake Hook)
  o passo simulado carrega **o percurso inteiro de `0,6`** de uma vez; nos modos de força o percurso
  só dá a DIRECÇÃO, e a magnitude é a da espec §4.1.

### ⭐ As NOVE corridas de 2026-09-06 que isolam a REDE de restrições (`*_origem_1passo*`)

Gravadas a pedido do I para a emenda Q14 (espec §5.2-quater e §10.8), na **mesma** sessão e com o
**mesmo** harness. Pen-down na **origem** e **um** passo simulado, em três pares A/B com **uma**
variável cada:

| par | o que muda entre os dois lados |
|---|---|
| `…_local_origem_1passo` × `…_global_origem_1passo` (gancho · agarrar · expandir) | a **área**, e com ela o número de projecções por restrição e por passo: `10` na *Local* (a lista vem em duplicado) contra `5` na *Global* |
| `plano_expandir_radial_local_origem_1passo` × `…_forca05` | a **força** (`1` → `0,5`), que num modo sem força nem âncora muda **só** o desvio de repouso |
| `plano_gancho_radial_local_origem_1passo` × `…_curto` | o **percurso** (`0,6` → `0,05`), i.e. o tamanho do gesto num passo |

⚠️ **Duas destas nove QUEBRAM o parágrafo acima, e os cabeçalhos dizem-no:**
`plano_gancho_radial_local_origem_1passo_constante` é a **única fixture do corpus inteiro com curva
`constant`** (todas as outras são `smooth`), e `…_curto` tem um percurso que não é `0,6` — leia-o no
campo `caminho` dela, não nesta prosa. ⚠️ **A cláusula «a ÚNICA com percurso ≠ `0,6`» caducou em
2026-09-06**: as duas fixtures `_parado` da emenda Q16 avançam `0,0545` e depois param.
⚠️ **Validação da sessão, antes de qualquer uma ser gravada:** uma corrida de controlo do gancho com
o caminho `0 → 0,3 → 0,6` devolveu `máx = 0,343869`, idêntico a seis casas ao da fixture
`plano_gancho_radial_local_2passos_origem` da sessão anterior. *Sem esta corrida, um número novo e um
número velho não são comparáveis.*
⚠️ **Proveniência igual à das outras**: malha nossa, gerada pelo harness; a saída é dado (§5 da
skill). Regenerar continua a ser acto de **E**.


## ⭐ O instrumento POR PASSO (`*.porpasso.txt.gz`, pedido do I em 2026-09-06)

**O que é.** Para os **dezassete** traços da tabela abaixo — os `_origem` —, as posições
**depois de CADA passo** — um ficheiro por traço com um
bloco `passo k` por passo. **Como foi obtido:** o traço do binário é uma chamada só e a simulação vive
dentro dela, logo não se pode «pausar»; mas a simulação **nunca olha para a frente**, então uma corrida
NOVA com só os primeiros `k` elementos do MESMO caminho, sobre uma malha fresca, termina exactamente no
estado do passo `k` da corrida inteira. Cada ficheiro traz a **prova**: `prova_do_fatiamento` = a
diferença máxima por vértice entre o bloco `k = N` e uma corrida inteira da MESMA sessão — tem de ser
`0,000000` (a 6 decimais).

⚠️⚠️ **`ls *.porpasso.txt.gz | wc -l` devolve `26`, e só VINTE E DOIS deles são o instrumento**
(R-pré, 2026-09-06; a contagem era `13`/`9` antes das oito da emenda Q16 e `21`/`17` antes das cinco
da Q18 — ⛔ **conte-os**):
os outros **quatro** são a 1.ª geração, com o pen-down em `x = −0,3`, e ⛔ **TRÊS deles
NÃO passam a prova do fatiamento** — `plano_arrastar_radial_local` (`0.330421`),
`plano_agarrar_radial_local_2passos` (`0.115064`) e `plano_gancho_radial_local_2passos` (`0.004244`).
⛔ **Não os use como oráculo**: ficam como o REGISTO da medição que obrigou ao pen-down na origem.
⭐ E o quarto, `plano_arrastar_radial_global`, dá `0.000000` — *porque a área dele é **Global** e não
tem centro para ficar refém do sobrevoo*, que é exactamente o mecanismo explicado a seguir.
⚠️ **O pen-down dos DEZASSETE está NA ORIGEM do objecto** (`caminho` de `(0,0,0)` a `(0,6,0,0)` —
nas duas `_parado` o caminho pára em `(0,0545,0,0)`), e o sufixo `_origem` diz-o (o `_fraco` é um
deles). Motivo, medido: o centro da área *Local* é o ponto de HOVER do cursor antes do
pen-down, e num traço scriptado esse hover é **refém do ponteiro físico** — numa sessão inteira saiu
certo, na seguinte saiu na origem em todas as corridas e a zero em duas. Com o pen-down na origem, o
centro é o mesmo quer o hover dispare quer não ⇒ determinístico por construção. Os outros fixtures
*Local* (pen-down em `x = −0,3`) foram **verificados** um a um: o disco de vértices movidos das corridas
completas está centrado em `x = −0,305` (o pen-down), não na origem — ver a coluna no ledger.

**`*.porpasso.rastreio.txt`** (texto): por passo, `|u|` de sete vértices de repouso nomeados —
sob o pen-down, a `1R`, `2R`, no início da banda (`2,875R`), a meio dela (`3,2R`), no limite (`3,5R`)
e fora (`4R`), todos deslocados **perpendicularmente** ao traço a partir do pen-down — mais o vértice
sob o cursor do passo `k`. ⛔ O factor `f` da força e o `φ` das restrições **não são observáveis** sem
recompilar o binário (o checkout é esparso e não compila); calculam-se da espec (§4.1, §2.2, §5.2)
sobre estas posições, e o rastreio dá o lado MEDIDO da comparação.

| traço (`_origem`) | passos | prova do fatiamento | movidos | máx `|u|` |
|---|---|---|---|---|
| `plano_arrastar_radial_local_origem` | 12 | `0.000000` | 2145 | `0.329649` |
| `plano_arrastar_radial_global_origem` | 12 | `0.000000` | 4225 | `0.645708` |
| `plano_gancho_radial_local_2passos_origem` | 3 | `0.000000` | 1950 | `0.343869` |
| `plano_agarrar_radial_local_2passos_origem` | 3 | `0.000000` | 1869 | `0.14572` |
| `plano_apertar_ponto_radial_local_origem` | 12 | `0.000000` | 2145 | `0.303401` |
| `plano_apertar_linha_radial_local_origem` | 12 | `0.000000` | 2137 | `0.100744` |
| `plano_apertar_ponto_radial_local_origem_fraco` | 12 | `0.000000` | 2029 | `0.004082` |
| `plano_empurrar_radial_local_origem` | 12 | `0.000000` | 2145 | `0.259368` |
| `plano_inflar_radial_local_origem` | 12 | `0.000000` | 2145 | `0.317081` |
| `plano_empurrar_radial_local_origem_parado` | 12 | `0.000000` | 2141 | `0.122854` |
| `plano_inflar_radial_local_origem_parado` | 12 | `0.000000` | 2145 | `0.144297` |
| `plano_empurrar_radial_local_origem_forca05` | 12 | `0.000000` | 2141 | `0.219673` |
| `plano_empurrar_radial_local_origem_forca025` | 12 | `0.000000` | 2122 | `0.110869` |
| `plano_empurrar_radial_local_origem_massa2` | 12 | `0.000000` | 2145 | `0.249664` |
| `plano_inflar_radial_local_origem_massa2` | 12 | `0.000000` | 2145 | `0.296328` |
| `plano_empurrar_radial_local_origem_amort1` | 12 | `0.000000` | 2133 | `0.19698` |
| `plano_empurrar_radial_global_origem` | 12 | `0.000000` | 4225 | `0.324889` |
| `plano_arrastar_radial_local_origem_36passos` | 36 | `0.000000` | 2145 | `0.267205` |
| `plano_arrastar_radial_global_origem_36passos` | 36 | `0.000000` | 4225 | `1.893192` |
| `esfera_agarrar_radial_dinamica` | 12 | `0.017736` ⚠️ | 1863 | `0.236509` |
| `esfera_gancho_radial_dinamica` | 12 | `0.036474` ⚠️ | 2234 | `0.169025` |
| `esfera_expandir_radial_dinamica` | 12 | `0.021864` ⚠️ | 2096 | `0.046715` |

⚠️⚠️ **As três últimas linhas têm a prova do fatiamento DIFERENTE DE ZERO, e isso não é um defeito
do ficheiro — é um facto da esfera** (2026-09-07, espec §10.13): **duas corridas da mesma
configuração de esfera não dão a mesma saída**. Quatro realizações de cada uma diferem entre si até
`0,020` (agarrar) · `0,036` (gancho) · `0,022` (expandir), e a prova do fatiamento de cada uma é
**igual a essa banda** — *se o prefixo não fosse o passo `k`, a prova seria MAIOR que a banda; ela
é igual.* ⇒ os três ficheiros trazem no cabeçalho `dispersao_entre_realizacoes_da_corrida_inteira`
e `desvio_ao_deformado_deste_repo`, e o `.rastreio` deles tem uma coluna a mais,
**`banda_de_realizacao`**, com a banda medida **por passo** (`0` no passo `1`, `~10⁻⁴` no `2`,
`0,015`–`0,040` no `12`). ⛔ **Uma comparação por passo na esfera que não leia essa coluna não quer
dizer nada**; e uma barra de gate escrita abaixo dela reprova o próprio oráculo.
⭐ No **plano** a dispersão é `0,000000` (medida em três realizações dos dois traços longos e quatro
do de 12 passos), e é por isso que ali a prova do fatiamento é literal.

⭐⭐ **O `_fraco` (2026-09-06) é um CONTROLO, não mais um traço** — a espec §10.6 e §5.2-ter. É o
traço de aperto de ponto da linha acima com **uma** coisa mudada, a força (`1,0 → 0,2`), e existe
para separar duas leituras que se confundiam: à força cheia o aperto vira a malha do avesso debaixo
do cursor logo no 1.º passo simulado (`10` quadriláteros de orientação invertida) e a saída do
oráculo passa a **quebrar a simetria de espelho do próprio traço** (`0,675` do maior deslocamento no
passo 3); à força `0,2` não há uma única face invertida em doze passos e a quebra cai para `0,103`,
que é o piso do arrasto. ⛔ **Não a use como fixture de amplitude** (o deslocamento é `0,004`, perto
da resolução do ficheiro): ela serve às perguntas «inverteu?» e «quanto é que a ordem decide?».

### ⭐⭐⭐ As OITO corridas de 2026-09-06 que separam a fase do GESTO da fase do SOLVER (espec §10.10)

Gravadas a pedido do I para a emenda Q16, na **mesma** sessão e com o **mesmo** harness, todas com
pen-down na **origem** e 12 passos, para serem comparáveis passo a passo com os `_origem` acima:

| variante | o que muda | o que a alavanca retira |
|---|---|---|
| `…_parado` (empurrar · inflar) | ⭐ o caminho **avança uma vez e depois repete o mesmo ponto** `10` vezes — a 1.ª e única coisa do corpus com pontos de caminho repetidos | a fase do gesto inteira do passo 3 ao 12 (um passo sem deslocamento de cursor não aplica força — espec §4.2): ficam **dez passos de solver puro** |
| `…_forca05` · `…_forca025` (empurrar) | a força (`1,0 → 0,5 → 0,25`), i.e. `¼` e `1/16` do impulso | a não-linearidade da resposta ao esticão |
| `…_massa2` (empurrar · inflar) | a massa (`1 → 2`), i.e. metade do impulso com a fase do gesto **idêntica** | idem, pelo outro lado |
| `…_amort1` (empurrar) | `amortecimento = 1` | a memória de velocidade de Verlet |
| `plano_empurrar_radial_global_origem` | a área (*Local* → *Global*) | metade das projecções por restrição (`10 → 5`) e a banda (`w ≡ 1`) |

⚠️⚠️ **DUAS coisas que se lêem ao contrário se não estiverem escritas:**
- **O CONTROLO do instrumento não é fixture, é um número:** uma corrida cujo caminho tem **todos** os
  pontos iguais devolve `0` vértices movidos e `máx |u| = 0,00000` em 12 passos, no empurrar e no
  inflar. É a prova de que a fase do gesto está mesmo calada num passo parado — sem ela, o `_parado`
  seria uma conjectura. (Reproduz-se com o caminho degenerado; não foi gravada como fixture porque é
  uma malha de zeros.)
- ⛔ **`massa2` chama-se assim porque `2` é o TECTO** — a corrida foi pedida com `4` e a porta de
  propriedades do binário **coagiu para `2,0`, em silêncio**, e é o `2,0` que está no cabeçalho.
  *Um valor pedido não é um valor aplicado: leia o cabeçalho, que traz o que ficou.*

⚠️ **O cabeçalho destas oito traz uma chave NOVA — `passos_com_cursor_parado`** (quantos pontos do
caminho repetem o anterior): `10` nas duas `_parado`, `0` nas outras seis. As fixtures anteriores
**não** foram reescritas e não a têm; o `gera_indice.py` passou a conhecê-la como inteiro.

⭐ **Os dois de APERTO foram acrescentados em 2026-09-06 a pedido do I** (a divergência dos modos de
aperto nasce entre o 1.º passo e o fim do traço, e só o dump por passo diz **em que** passo). ⚠️ O
rastreio deles mostra o que o traço inteiro esconde: sob o pen-down o aperto de PONTO **não é
monótono** (`0,093 · 0,184 · 0,118 · 0,106 · 0,197 · 0,208 · 0,201 · 0,187 · 0,160 · 0,149 · 0,154`
nos passos 2..12) — a força aponta para o cursor, que se afasta, logo o vértice é puxado e largado a
cada passo; e o de LINHA quase não move o pen-down (`≤ 0,006`) e move o vizinho a `1R`.

⭐ **Os dois de FORÇA NORMAL — `empurrar` e `inflar` — foram acrescentados em 2026-09-06, também a
pedido do I** (espec §10.7). Eles são o par que separa as **duas** normais do alvo: no 1.º passo
simulado a folha está plana e em repouso, logo a normal da área e a normal do vértice são a mesma
coisa e a razão dos dois é exactamente `2R` (`0,06543 / 0,09347 = 0,7000`); a partir do passo 3 elas
divergem, e a do Push **roda com a vala que o traço abre** (espec §4.2-bis). ⚠️ O rastreio mostra o
que o traço inteiro esconde: sob o pen-down o Push **satura e recua** (`0,2397` no passo 7 → `0,2195`
no 12) enquanto o Inflate fica (`0,2701` → `0,2629`), e nos dois o aro está preso (`3,5R` em
`0,0004`/`0,0005`, `4R` em zero exacto).
`plano_arrastar_radial_local_origem` — `|u|` depois do passo k (excerto do rastreio):

| passo | sob o pen-down | a 1R | no limite 3,5R | fora, 4R | sob o cursor do passo |
|---|---|---|---|---|---|
| 1 | `0.00000` | `0.00000` | `0.00000` | `0.00000` | `0.00000` |
| 2 | `0.09347` | `0.00072` | `0.00000` | `0.00000` | `0.09986` |
| 3 | `0.16758` | `0.01176` | `0.00000` | `0.00000` | `0.14865` |
| 4 | `0.21314` | `0.03171` | `0.00000` | `0.00000` | `0.17944` |
| 5 | `0.24711` | `0.05551` | `0.00000` | `0.00000` | `0.18102` |
| 6 | `0.27291` | `0.07947` | `0.00002` | `0.00000` | `0.19755` |
| 7 | `0.29041` | `0.10136` | `0.00004` | `0.00000` | `0.21078` |
| 8 | `0.29737` | `0.11971` | `0.00008` | `0.00000` | `0.22034` |
| 9 | `0.29100` | `0.13338` | `0.00013` | `0.00000` | `0.22642` |
| 10 | `0.27014` | `0.14161` | `0.00019` | `0.00000` | `0.22915` |
| 11 | `0.24062` | `0.14418` | `0.00025` | `0.00000` | `0.20411` |
| 12 | `0.22022` | `0.14132` | `0.00032` | `0.00000` | `0.20429` |

`plano_arrastar_radial_global_origem` — `|u|` depois do passo k (excerto do rastreio):

| passo | sob o pen-down | a 1R | no limite 3,5R | fora, 4R | sob o cursor do passo |
|---|---|---|---|---|---|
| 1 | `0.00000` | `0.00000` | `0.00000` | `0.00000` | `0.00000` |
| 2 | `0.09347` | `0.00072` | `0.00000` | `0.00000` | `0.09986` |
| 3 | `0.20530` | `0.01013` | `0.00000` | `0.00000` | `0.18029` |
| 4 | `0.27913` | `0.03197` | `0.00005` | `0.00001` | `0.22558` |
| 5 | `0.33335` | `0.06380` | `0.00028` | `0.00011` | `0.24307` |
| 6 | `0.38610` | `0.10068` | `0.00103` | `0.00049` | `0.24223` |
| 7 | `0.44464` | `0.14042` | `0.00273` | `0.00157` | `0.23429` |
| 8 | `0.49123` | `0.18174` | `0.00584` | `0.00388` | `0.30696` |
| 9 | `0.54262` | `0.22325` | `0.01074` | `0.00797` | `0.22634` |
| 10 | `0.58434` | `0.26075` | `0.01766` | `0.01427` | `0.25184` |
| 11 | `0.61514` | `0.29525` | `0.02659` | `0.02293` | `0.25337` |
| 12 | `0.64571` | `0.32416` | `0.03738` | `0.03378` | `0.21922` |


### ⭐⭐⭐ As TRÊS corridas de 2026-09-07 que fixam a que SUPERFÍCIE pertencem as normais do gesto (espec §4.2-ter e §10.11)

Dois dos oito modos tiram a direcção de uma **normal da malha** — o *empurrar* (uma normal de área,
uma por passo) e o *inflar* (a normal de cada vértice). O corpus do plano não podia decidir de que
malha ela sai, porque no plano a normal de repouso, a normal da vista e a normal de qualquer
sub-conjunto valem todas `(0, 0, 1)`. Estas três resolvem-no:

| fixture | o que ela decide |
|---|---|
| `esfera_empurrar_radial_local_1passo` | ⭐ **a direcção do empurrar é a normal da ÁREA, não a da vista.** Num passo simulado sobre a esfera em repouso a relaxação é um no-op (as restrições nascem satisfeitas), logo o deslocamento é `u · f(v) · dt` e `u` lê-se por mínimos quadrados: dá `\|u\| = 0,700000 = 2R` com resíduo `6,1·10⁻⁷`, a **`0,023°`** da normal de repouso no cursor e a **`17,43°`** da normal da vista |
| `esfera_inflar_radial_local_1passo` | o **controlo** do anterior: mesma cena, mesmo passo, mas a direcção é **por vértice**; amplitude `1,000000`, resíduo `5,0·10⁻⁵` contra a normal da esfera |
| `plano_inflar_radial_local_1passo_2tracos` | ⭐⭐ **as normais são as da superfície que O TRAÇO encontrou, e refrescam-se de traço para traço.** O 1.º traço deixa uma cova de `0,0992` (é o `max_deslocamento` de `plano_inflar_radial_local_1passo`, `0.09917`; normais inclinadas até **`22,4°`**); no 2.º, a direcção por vértice bate as normais da malha **no pen-down dele** com resíduo `3,5·10⁻⁷` e amplitude `1,00000`, contra resíduo `0,32` e amplitude `0,946` para as normais planas do 1.º. ⭐ **E ela discrimina uma coisa que a espec dava por indecidível** — o PESO da soma por face (espec §4.6 linha 4): medida a distância entre direcções unitárias, a mediana sobre os `171` movidos é `1,7·10⁻⁵` (uniforme) · `3,0·10⁻⁴` (ângulo) · `6,6·10⁻⁴` (área) |

⚠️ **A terceira é a única fixture do corpus com DOIS traços** (chave `tracos 2` no cabeçalho, ausente
em todas as outras); o `caminho` dela é o dos dois, que são o mesmo.

⛔⛔ **ERRATA de 2026-09-07: a última célula da tabela acima responde a uma pergunta que aquela malha
NÃO consegue decidir.** Medidas as três candidatas de peso contra o vector de normais que o **próprio
programa** guarda para aquela mesma malha, elas concordam entre si a **`0,000°` de mediana e `0,31°`
de máximo** — a cova é rasa e a grelha é regular, logo os três pesos dão praticamente a mesma
normal. ⇒ o veredito *uniforme* está **certo** (§4.2-quater fecha-o com a régua que o decide), mas
não é esta fixture que o prova. *Uma régua indirecta que acerta continua a ser indirecta.*

### ⭐⭐⭐ As CINCO fixtures de 2026-09-07 da emenda Q18 (espec §10.12 e §10.13)

**Duas** respondem à pergunta *«qual é a saída do alvo num traço LONGO?»* e **três** à pergunta
*«em que passo nasce a divergência na esfera?»*.

| fixture | passos | o que ela decide |
|---|---|---|
| `plano_arrastar_radial_local_origem_36passos` | 36 | ⛔ **um traço mais longo NÃO é mais fundo**: `0,76 R` contra `0,94 R` do mesmo caminho em 12 passos. Em área *Local* a profundidade satura, e o comprimento do caminho não a move (`0,66`–`0,94 R` em toda a família varrida) |
| `plano_arrastar_radial_global_origem_36passos` | 36 | ⭐⭐ **o regime FUNDO existe e é um facto da ÁREA**: `5,41 R`, sem assentar (o vértice do pen-down cresce nos 36 passos, de `0,0993` a `1,0550`). É o lado aprovado para qualquer régua de relevo local que viva acima de `1 R` |
| `esfera_agarrar_radial_dinamica` (por passo) | 12 | os três primeiros dumps por passo de ESFERA do corpus. ⚠️ **Cada bloco é UMA realização**, e o `.rastreio` traz a coluna `banda_de_realizacao` com a diferença medida entre quatro corridas do mesmo prefixo |
| `esfera_gancho_radial_dinamica` (por passo) | 12 | idem — é a de maior banda (`0,036` ao 12.º passo, `21,8 %` do sinal) |
| `esfera_expandir_radial_dinamica` (por passo) | 12 | idem — é a de menor sinal, e por isso a de maior razão banda/sinal (`60 %`) |

⛔⛔⛔ **O achado que estas cinco trouxeram, e que muda como se lê o corpus inteiro: na ESFERA duas
corridas da mesma configuração não dão a mesma saída.** Medido com quatro realizações de cada
configuração, na mesma sessão: o **plano** dá `0,000000` (em traços de 12 **e** de 36 passos, nas
duas áreas); a esfera dá `0,027`–`0,037` ao 12.º passo, e a divergência já existe ao **1.º passo
simulado** (`4,3·10⁻⁴`), amplificando `~65×`. ⛔ **Quatro explicações foram medidas e refutadas** —
concorrência (uma só linha de execução dá `0,036`), semeadura do sobrevoo (`0,041`), pré-traço
(`0,038`) e desenho forçado da vista (`0,033`) — e a diferença é **difusa** (`481` de `631` vértices
movidos já diferem ao 1.º passo simulado), não pontual.
⇒ **os `.deformado` de esfera são UMA realização cada**, e uma barra de gate escrita abaixo da banda
reprova o próprio oráculo. ⚠️ **E não é desculpa para os erros abertos**: eles são `5×` a `26×` a
banda. Detalhe e tabelas: espec §10.13.

## ⚠️ O EIXO DA VISTA (não está no cabeçalho, e é diferente nos dois corpora)

As corridas são em vista **ORTOGRÁFICA**. O cabeçalho traz o `caminho` (as linhas `c`, em espaço do
objecto) e **não** traz a vista — que é o que falta para reconstruir deste lado o deslocamento de
cursor que sete dos oito modos lêem (espec §4.3). Ela é, derivada das próprias fixtures:

| corpus | superfície | eixo da vista | consequência |
|---|---|---|---|
| plano | folha em `z = 0` | **`z`** | a projecção é um **no-op**: o deslocamento é a diferença dos pontos do caminho, ao bit |
| esfera | esfera unitária, caminho em `y = −√(1−x²)` | **`y`** | o deslocamento vive no plano `x–z`; é a componente `y` que se perde (até `15,83°` de direcção) |

⭐ **Que é ortográfica lê-se nos números:** o passo do caminho da esfera é `0,6/11 = 0,054545…` e o
deslocamento de cursor mede `0,05455` em **todos** os 12 passos, apesar de os pontos do caminho
estarem a profundidades diferentes — em perspectiva os dois não podiam coincidir.

## O formato (texto, `gzip`, vocabulário do domínio)

`<superficie>.repouso.txt.gz` — uma vez por superfície:
```
vertices <N>
v <x> <y> <z>        # N linhas, índice = ordem
```
`<superficie>_<modo>_<falloff>_<area>[_<variante>].deformado.txt.gz` — por corrida:
```
superficie plano|esfera · modo · falloff_da_forca radial|plano · area local|global|dinamica
raio · limite · banda · massa · amortecimento · plasticidade · pino · forca · curva · passos
movidos <n>  max_deslocamento <d>          # recontados pelo verificador
caminho <k>  +  k linhas  c <x> <y> <z>    # os pontos do cursor, em espaço de objecto
vertices <N> +  N linhas  d <x> <y> <z>    # as posições DEPOIS do traço, mesma ordem do repouso
```

## O verificador

`python3 verifica_traco.py` (neste diretório) relê tudo, reconta `movidos` e `max_deslocamento` e
compara com o cabeçalho — **exit 0 = coerente**. Ele não carrega algoritmo nenhum: é a prova de que
o ficheiro diz o que contém.

## Os dois índices derivados (JSON)

- `indice.json` — um par `[fixture, cabeçalho]` por `.deformado.txt.gz`, com as **mesmas chaves** do
  cabeçalho deles (derivado deles; serve para escolher fixtures sem descomprimir).
  ⚠️⚠️ **Ele é DERIVADO e envelheceu duas vezes em silêncio:** em 2026-09-06 tinha `48` entradas para
  `54` ficheiros — faltavam-lhe os **seis** traços `_origem` (os do instrumento por passo), que são
  precisamente os mais usados. ⇒ **regenere-o** varrendo os `.deformado.txt.gz` sempre que
  acrescentar um; a contagem certa é `ls *.deformado.txt.gz | wc -l`, ⛔ nunca um número escrito
  aqui. *Um índice derivado que ninguém regenera é uma lista escrita à mão com cara de derivada.*
  ⭐ **Desde 2026-09-06 ele TEM gerador: `python3 gera_indice.py`** (neste diretório) — varre os
  `.deformado.txt.gz`, escreve uma entrada por ficheiro com as chaves do cabeçalho deles, e imprime a
  contagem. *A regra da casa é «índice de diretório se GERA, não se escreve»; até aqui a regra estava
  escrita e a ferramenta não existia, e foi por isso que ele envelheceu duas vezes.*
- `analise.json` — 46 objectos com as grandezas que a espec §10 tabela, calculadas pelo harness do E
  a partir de repouso + deformado (⛔ **não são oráculo**: são leituras NOSSAS sobre o dado):
  `fixture` · `corrida_oraculo` (o nome interno da corrida no harness — só para o E regenerar) ·
  `movidos` · `max_deslocamento` · `alcance` (distância máxima de um vértice movido ao caminho) ·
  `alcance_sobre_raio` · `fraccao_normal` (`Σ|u·n⁰| / Σ|u|`) · `coerencia` (módulo do vector unitário
  médio dos deslocamentos grandes) · `u_normal_max` / `u_normal_min` · `desloc_no_passo1` /
  `desloc_no_fim` (deslocamento medido no 1.º / no último ponto do caminho, conforme o harness) ·
  `delta_area` (fracção; só significativa no plano) · `passos` · `raio`.
  ⚠️⚠️ **ESTA DESCRIÇÃO NÃO É A DO FICHEIRO — conferido em 2026-09-06.** O `analise.json` que está no
  disco tem **47** objectos (contra `78` fixtures em 2026-09-07 — eram `54` quando isto foi escrito
  e `73` quando isto foi conferido, e o desvio só cresce) e as chaves **do harness**, não as de cima; a linha
  que dizia «renomeado pelo R-pré em 2026-09-05 … 46/46» descrevia uma renomeação que **não está no
  ficheiro**. Ele continua a ser dado nosso e o sweep passa sobre ele; o que não vale é acreditar
  nesta secção. ⇒ **quem o regenerar escreve-o com as chaves de cima e com uma entrada por
  `.deformado.txt.gz`** — e até lá leia o próprio ficheiro. *Duas descrições da mesma tabela e a que
  se lê primeiro é a que envelheceu.*

## As corridas

| fixture | modo | passos | movidos | máx |u| |
|---|---|---|---|---|
| `esfera_agarrar_radial_dinamica.` | agarrar | 12 | 1863 | `0.236509` |
| `esfera_apertar_linha_radial_dinamica.` | apertar_linha | 12 | 2162 | `0.249739` |
| `esfera_apertar_ponto_radial_dinamica.` | apertar_ponto | 12 | 2183 | `0.463862` |
| `esfera_arrastar_radial_dinamica.` | arrastar | 12 | 2183 | `0.582806` |
| `esfera_empurrar_radial_dinamica.` | empurrar | 12 | 2102 | `0.479385` |
| `esfera_empurrar_radial_local_1passo.` | empurrar | 2 | 120 | `0.069165` |
| `esfera_expandir_radial_dinamica.` | expandir | 12 | 2096 | `0.046715` |
| `esfera_gancho_radial_dinamica.` | gancho | 12 | 2234 | `0.169025` |
| `esfera_inflar_radial_dinamica.` | inflar | 12 | 2181 | `0.267017` |
| `esfera_inflar_radial_local_1passo.` | inflar | 2 | 120 | `0.098807` |
| `plano_agarrar_plano_local.` | agarrar | 12 | 2146 | `0.307644` |
| `plano_agarrar_radial_global_origem_1passo.` | agarrar | 2 | 881 | `0.094722` |
| `plano_agarrar_radial_local.` | agarrar | 12 | 2139 | `0.16991` |
| `plano_agarrar_radial_local_1passo.` | agarrar | 2 | 1324 | `0.134099` |
| `plano_agarrar_radial_local_24passos.` | agarrar | 24 | 2142 | `0.158543` |
| `plano_agarrar_radial_local_2passos.` | agarrar | 3 | 1872 | `0.146115` |
| `plano_agarrar_radial_local_2passos_origem.` | agarrar | 3 | 1869 | `0.14572` |
| `plano_agarrar_radial_local_amort06.` | agarrar | 12 | 2131 | `0.131488` |
| `plano_agarrar_radial_local_origem_1passo.` | agarrar | 2 | 1323 | `0.134311` |
| `plano_agarrar_radial_local_preset.` | agarrar | 12 | 4123 | `0.132623` |
| `plano_apertar_linha_radial_local.` | apertar_linha | 12 | 2135 | `0.100451` |
| `plano_apertar_linha_radial_local_1passo.` | apertar_linha | 2 | 156 | `0.087609` |
| `plano_apertar_linha_radial_local_origem.` | apertar_linha | 12 | 2137 | `0.100744` |
| `plano_apertar_ponto_plano_local.` | apertar_ponto | 12 | 2146 | `0.623884` |
| `plano_apertar_ponto_radial_local.` | apertar_ponto | 12 | 2146 | `0.325769` |
| `plano_apertar_ponto_radial_local_1passo.` | apertar_ponto | 2 | 171 | `0.09917` |
| `plano_apertar_ponto_radial_local_origem.` | apertar_ponto | 12 | 2145 | `0.303401` |
| `plano_apertar_ponto_radial_local_origem_fraco.` | apertar_ponto | 12 | 2029 | `0.004082` |
| `plano_arrastar_plano_local.` | arrastar | 12 | 2146 | `0.8996` |
| `plano_arrastar_radial_dinamica.` | arrastar | 12 | 2508 | `0.612821` |
| `plano_arrastar_radial_dinamica_preset.` | arrastar | 12 | 2455 | `0.329617` |
| `plano_arrastar_radial_global.` | arrastar | 12 | 4225 | `0.644607` |
| `plano_arrastar_radial_global_origem.` | arrastar | 12 | 4225 | `0.645708` |
| `plano_arrastar_radial_global_origem_36passos.` | arrastar | 36 | 4225 | `1.893192` |
| `plano_arrastar_radial_local.` | arrastar | 12 | 2144 | `0.331637` |
| `plano_arrastar_radial_local_1passo.` | arrastar | 2 | 171 | `0.09917` |
| `plano_arrastar_radial_local_2passos.` | arrastar | 3 | 1438 | `0.135888` |
| `plano_arrastar_radial_local_amort05.` | arrastar | 12 | 2142 | `0.254386` |
| `plano_arrastar_radial_local_amort1.` | arrastar | 12 | 2141 | `0.219903` |
| `plano_arrastar_radial_local_forca05.` | arrastar | 12 | 2139 | `0.073252` |
| `plano_arrastar_radial_local_forca05_1passo.` | arrastar | 2 | 168 | `0.024792` |
| `plano_arrastar_radial_local_massa2.` | arrastar | 12 | 2143 | `0.154596` |
| `plano_arrastar_radial_local_massa2_1passo.` | arrastar | 2 | 171 | `0.049585` |
| `plano_arrastar_radial_local_origem.` | arrastar | 12 | 2145 | `0.329649` |
| `plano_arrastar_radial_local_origem_36passos.` | arrastar | 36 | 2145 | `0.267205` |
| `plano_arrastar_radial_local_pino.` | arrastar | 12 | 2144 | `0.323528` |
| `plano_arrastar_radial_local_plast05.` | arrastar | 12 | 2141 | `0.234305` |
| `plano_empurrar_plano_local.` | empurrar | 12 | 2146 | `0.520138` |
| `plano_empurrar_radial_global_origem.` | empurrar | 12 | 4225 | `0.324889` |
| `plano_empurrar_radial_local.` | empurrar | 12 | 2145 | `0.258986` |
| `plano_empurrar_radial_local_1passo.` | empurrar | 2 | 171 | `0.069419` |
| `plano_empurrar_radial_local_origem.` | empurrar | 12 | 2145 | `0.259368` |
| `plano_empurrar_radial_local_origem_amort1.` | empurrar | 12 | 2133 | `0.19698` |
| `plano_empurrar_radial_local_origem_forca025.` | empurrar | 12 | 2122 | `0.110869` |
| `plano_empurrar_radial_local_origem_forca05.` | empurrar | 12 | 2141 | `0.219673` |
| `plano_empurrar_radial_local_origem_massa2.` | empurrar | 12 | 2145 | `0.249664` |
| `plano_empurrar_radial_local_origem_parado.` | empurrar | 12 | 2141 | `0.122854` |
| `plano_expandir_radial_global_origem_1passo.` | expandir | 2 | 724 | `0.001525` |
| `plano_expandir_radial_local.` | expandir | 12 | 2134 | `0.011523` |
| `plano_expandir_radial_local_1passo.` | expandir | 2 | 848 | `0.001902` |
| `plano_expandir_radial_local_origem_1passo.` | expandir | 2 | 846 | `0.001914` |
| `plano_expandir_radial_local_origem_1passo_forca05.` | expandir | 2 | 662 | `0.000478` |
| `plano_gancho_radial_global_origem_1passo.` | gancho | 2 | 1068 | `0.343172` |
| `plano_gancho_radial_local.` | gancho | 12 | 2140 | `0.09155` |
| `plano_gancho_radial_local_1passo.` | gancho | 2 | 1452 | `0.489383` |
| `plano_gancho_radial_local_24passos.` | gancho | 24 | 2142 | `0.02932` |
| `plano_gancho_radial_local_2passos.` | gancho | 3 | 1950 | `0.364813` |
| `plano_gancho_radial_local_2passos_origem.` | gancho | 3 | 1950 | `0.343869` |
| `plano_gancho_radial_local_amort06.` | gancho | 12 | 2135 | `0.063396` |
| `plano_gancho_radial_local_origem_1passo.` | gancho | 2 | 1451 | `0.456101` |
| `plano_gancho_radial_local_origem_1passo_constante.` | gancho | 2 | 1735 | `0.878999` |
| `plano_gancho_radial_local_origem_1passo_curto.` | gancho | 2 | 1129 | `0.032433` |
| `plano_inflar_radial_local.` | inflar | 12 | 2146 | `0.317159` |
| `plano_inflar_radial_local_1passo.` | inflar | 2 | 171 | `0.09917` |
| `plano_inflar_radial_local_1passo_2tracos.` | inflar | 2 (×2 traços) | 171 | `0.178894` |
| `plano_inflar_radial_local_origem.` | inflar | 12 | 2145 | `0.317081` |
| `plano_inflar_radial_local_origem_massa2.` | inflar | 12 | 2145 | `0.296328` |
| `plano_inflar_radial_local_origem_parado.` | inflar | 12 | 2145 | `0.144297` |
**78 traços** (47 da matriz + 9 do instrumento por passo + 9 das corridas que isolam a REDE de
restrições + 8 das corridas que separam a fase do GESTO da fase do SOLVER, de 2026-09-06, + 3 das
corridas que fixam a que superfície pertencem as NORMAIS do gesto e **2 dos traços LONGOS**, de
2026-09-07 — espec §10.8, §10.10, §10.11 e §10.12) — ⚠️ **conte-os**
(`ls *.deformado.txt.gz | wc -l`), esta linha já esteve parada em `53`, em `56`, em `65`, em `73` e
em `76`.
⚠️ **As fixtures de ESFERA de DOZE passos são todas de área Dinâmica** (centro no cursor), e a área
*Local* de doze passos na esfera continua por gravar. ⛔ **A razão escrita aqui até 2026-09-07 estava
ERRADA e mandava não tentar:** dizia que «um traço scriptado não dispara o hover que fixa o centro
da área Local». Dispara — basta o harness semear o hover, e as duas fixtures de esfera de **um**
passo de 2026-09-07 são de área *Local*. ⚠️ **O que nelas não é observável é a BANDA**, e é por outro
motivo: só se movem `120` vértices, todos a menos de `0,35` do cursor, onde `w = 1` quer o centro
esteja no pen-down quer na origem do objecto (numa esfera unitária **toda** a superfície dista `1,0`
da origem, e `1,0 < R(1+L·F) = 1,00625`) ⇒ elas fixam a **direcção** e a **magnitude** do gesto, não
a área. *Uma fixture prova o que contém.*

---

## ⭐ As QUATRO fixtures de TOPOLOGIA (2026-09-06, emenda Q15 — espec §3.1-bis e §10.9)

⚠️ **Estas quatro NÃO são traços** — não têm posições, não entram no `indice.json` (o gerador só vê
`*.deformado.txt.gz`) e não contam para os `76` acima. Elas descrevem a **malha**, e existem porque
a ordem de criação das restrições (espec §3.1) é *célula → vértice próprio → anel*, e nenhuma das
três se lê das posições de repouso: quem reconstrói a malha casando-a com as fixtures **por posição**
fica com os índices de VÉRTICE certos e sem as faces nem as células.

| ficheiro | o que traz |
|---|---|
| `plano.faces.txt.gz` · `esfera.faces.txt.gz` | a lista de faces na **ordem de armazenamento**, `f i j k …` por linha, na ordem de percurso da face, com os índices de vértice das fixtures de repouso |
| `plano.celulas.txt.gz` · `esfera.celulas.txt.gz` | as células da árvore espacial por **índice crescente**: `cv <i> <n> …` = os vértices **próprios** da célula, na ordem de visita · `cf <i> <m> …` = as faces dela (para se poder calcular a caixa e aplicar o teste do §2.1) |

| malha | vértices | faces | células | faces por célula | próprios por célula |
|---|---|---|---|---|---|
| plano `64×64` | `4 225` | `4 096` (todos quads) | `2` | `2 048` · `2 048` | `2 145` · `2 080` |
| esfera `96×64` | `6 050` | `6 144` (`5 952` quads + `192` triângulos) | `4` | `1 536` × 4 | `1 569` · `1 520` · `1 504` · `1 457` |

**Proveniência** — a mesma das outras: as malhas são **nossas**, geradas pelo mesmo harness e pela
mesma lei (conferidas contra `plano.repouso` e `esfera.repouso` vértice a vértice, diferença máxima
`0,0` às seis casas); o que a aplicação de referência calculou é a **partição**, e ela é **dado**
(uma permutação de inteiros sobre uma malha nossa). ⛔ Regenerar continua a ser acto de **E**.

⭐⭐ **A partição é MEDIDA, não derivada.** Ela não é observável pela API de scripting, mas a
aplicação tem um gesto que **reordena a malha** para o consumo desta mesma árvore, pela mesma lei —
e a permutação que ele devolve lê-se comparando as posições antes e depois (nas duas malhas todas as
posições são distintas, logo o emparelhamento é exacto, sem tolerância). Ela bate a partição
derivada **elemento a elemento nas duas malhas**. ⚠️ **A malha destas fixtures NÃO está reordenada**
— esse gesto é um comando explícito do artista, o harness não o corre, e num port que assuma a malha
reordenada a ordem de visita passaria a ser a crescente global, que é precisamente o caso errado.

⚠️⚠️ **DOIS `2 145` diferentes no plano:** a célula `1` tem `2 145` vértices próprios **e** a banda
de `3,5 R` contém `2 145` vértices — conjuntos distintos (intersecção `1 099`), e é o segundo que é
a coluna `movidos` da tabela acima.
