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
(≈ 7,5 arestas da grelha); força `1,0` (salvo `_forca05`), pressão `1`, curva *Smooth*, dureza `0`,
área *Local* (salvo indicação), limite `2,5`, banda `0,75`, massa `1`, amortecimento `0,01`,
plasticidade `0`, pino desligado, sem colisões, sem gravidade — i.e., **as omissões do código**
(espec §8.1), não as dos presets (§8.2).

⚠️ **Duas coisas do harness que mudam a leitura de uma fixture:**
- o centro da área *Local* é o ponto da superfície sob o cursor **no hover antes do pen-down** — o
  harness move o cursor do sistema para o pixel do pen-down e deixa a janela redesenhar antes do
  traço, e é por isso que o centro coincide com o 1.º ponto do caminho (sem isso, ficava num ponto velho
  e a simulação nascia noutro sítio — foi medido e a matriz refeita);
- nas variantes `_1passo` o caminho tem **dois** pontos, logo nos modos de âncora (Grab, Snake Hook)
  o passo simulado carrega **o percurso inteiro de `0,6`** de uma vez; nos modos de força o percurso
  só dá a DIRECÇÃO, e a magnitude é a da espec §4.1.


## ⭐ O instrumento POR PASSO (`*.porpasso.txt.gz`, pedido do I em 2026-09-06)

**O que é.** Para os **nove** traços da tabela abaixo — os `_origem` —, as posições
**depois de CADA passo** — um ficheiro por traço com um
bloco `passo k` por passo. **Como foi obtido:** o traço do binário é uma chamada só e a simulação vive
dentro dela, logo não se pode «pausar»; mas a simulação **nunca olha para a frente**, então uma corrida
NOVA com só os primeiros `k` elementos do MESMO caminho, sobre uma malha fresca, termina exactamente no
estado do passo `k` da corrida inteira. Cada ficheiro traz a **prova**: `prova_do_fatiamento` = a
diferença máxima por vértice entre o bloco `k = N` e uma corrida inteira da MESMA sessão — tem de ser
`0,000000` (a 6 decimais).

⚠️⚠️ **`ls *.porpasso.txt.gz | wc -l` devolve `13`, e só NOVE deles são o instrumento** (R-pré,
2026-09-06): os outros **quatro** são a 1.ª geração, com o pen-down em `x = −0,3`, e ⛔ **TRÊS deles
NÃO passam a prova do fatiamento** — `plano_arrastar_radial_local` (`0.330421`),
`plano_agarrar_radial_local_2passos` (`0.115064`) e `plano_gancho_radial_local_2passos` (`0.004244`).
⛔ **Não os use como oráculo**: ficam como o REGISTO da medição que obrigou ao pen-down na origem.
⭐ E o quarto, `plano_arrastar_radial_global`, dá `0.000000` — *porque a área dele é **Global** e não
tem centro para ficar refém do sobrevoo*, que é exactamente o mecanismo explicado a seguir.
⚠️ **O pen-down dos NOVE está NA ORIGEM do objecto** (`caminho` de `(0,0,0)` a `(0,6,0,0)`), e o
sufixo `_origem` diz-o (o `_fraco` é um deles). Motivo, medido: o centro da área *Local* é o ponto de HOVER do cursor antes do
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

⭐⭐ **O `_fraco` (2026-09-06) é um CONTROLO, não mais um traço** — a espec §10.6 e §5.2-ter. É o
traço de aperto de ponto da linha acima com **uma** coisa mudada, a força (`1,0 → 0,2`), e existe
para separar duas leituras que se confundiam: à força cheia o aperto vira a malha do avesso debaixo
do cursor logo no 1.º passo simulado (`10` quadriláteros de orientação invertida) e a saída do
oráculo passa a **quebrar a simetria de espelho do próprio traço** (`0,675` do maior deslocamento no
passo 3); à força `0,2` não há uma única face invertida em doze passos e a quebra cai para `0,103`,
que é o piso do arrasto. ⛔ **Não a use como fixture de amplitude** (o deslocamento é `0,004`, perto
da resolução do ficheiro): ela serve às perguntas «inverteu?» e «quanto é que a ordem decide?».

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
  disco tem **47** objectos (para `54` fixtures) e as chaves **do harness**, não as de cima; a linha
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
| `esfera_expandir_radial_dinamica.` | expandir | 12 | 2096 | `0.046715` |
| `esfera_gancho_radial_dinamica.` | gancho | 12 | 2234 | `0.169025` |
| `esfera_inflar_radial_dinamica.` | inflar | 12 | 2181 | `0.267017` |
| `plano_agarrar_plano_local.` | agarrar | 12 | 2146 | `0.307644` |
| `plano_agarrar_radial_local.` | agarrar | 12 | 2139 | `0.16991` |
| `plano_agarrar_radial_local_1passo.` | agarrar | 2 | 1324 | `0.134099` |
| `plano_agarrar_radial_local_24passos.` | agarrar | 24 | 2142 | `0.158543` |
| `plano_agarrar_radial_local_2passos.` | agarrar | 3 | 1872 | `0.146115` |
| `plano_agarrar_radial_local_2passos_origem.` | agarrar | 3 | 1869 | `0.14572` |
| `plano_agarrar_radial_local_amort06.` | agarrar | 12 | 2131 | `0.131488` |
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
| `plano_arrastar_radial_local_pino.` | arrastar | 12 | 2144 | `0.323528` |
| `plano_arrastar_radial_local_plast05.` | arrastar | 12 | 2141 | `0.234305` |
| `plano_empurrar_plano_local.` | empurrar | 12 | 2146 | `0.520138` |
| `plano_empurrar_radial_local.` | empurrar | 12 | 2145 | `0.258986` |
| `plano_empurrar_radial_local_1passo.` | empurrar | 2 | 171 | `0.069419` |
| `plano_empurrar_radial_local_origem.` | empurrar | 12 | 2145 | `0.259368` |
| `plano_expandir_radial_local.` | expandir | 12 | 2134 | `0.011523` |
| `plano_expandir_radial_local_1passo.` | expandir | 2 | 848 | `0.001902` |
| `plano_gancho_radial_local.` | gancho | 12 | 2140 | `0.09155` |
| `plano_gancho_radial_local_1passo.` | gancho | 2 | 1452 | `0.489383` |
| `plano_gancho_radial_local_24passos.` | gancho | 24 | 2142 | `0.02932` |
| `plano_gancho_radial_local_2passos.` | gancho | 3 | 1950 | `0.364813` |
| `plano_gancho_radial_local_2passos_origem.` | gancho | 3 | 1950 | `0.343869` |
| `plano_gancho_radial_local_amort06.` | gancho | 12 | 2135 | `0.063396` |
| `plano_inflar_radial_local.` | inflar | 12 | 2146 | `0.317159` |
| `plano_inflar_radial_local_1passo.` | inflar | 2 | 171 | `0.09917` |
| `plano_inflar_radial_local_origem.` | inflar | 12 | 2145 | `0.317081` |

**56 traços** (47 da matriz + 9 do instrumento por passo) — ⚠️ **conte-os**
(`ls *.deformado.txt.gz | wc -l`), esta linha já esteve parada em `53`. ⚠️ As fixtures de ESFERA são todas de área **Dinâmica** (centro no cursor). A área *Local* na esfera NÃO foi gravada: um traço scriptado não dispara o hover que fixa o centro da área Local, que fica na ORIGEM do objecto — e numa esfera unitária a origem põe toda a malha dentro da banda (ver ERRATA no ledger). A área Local está medida no PLANO (onde a origem cai na superfície).
