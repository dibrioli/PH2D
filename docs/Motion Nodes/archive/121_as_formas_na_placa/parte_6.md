# ARQUIVO — parte_5.md (história, 686 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`parte_5.md`](parte_5.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-342` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §9.19 — A 4.ª ONDA: o passeio em paralelo, a parede do Vello, a ponta e a junta no cartão, a prova do SIGSEGV e o contacto em Play (2026-10-05, escrito ANTES de medir)

Ordem do dono (05/10): *«integrar só quando não houver nada em aberto»* e *«resolver tudo numa ÚNICA onda»*. Base
`a46c4c200` (o main andou um commit, só o guarda R4; rebase sem conflito). Cinco itens; cada um CONSTRUÍDO e medido ou
RECUSADO pela medição, com o critério daqui.

**(1) O PASSEIO EM PARALELO — 4.ª topologia ⇒ two-strikes: a PROVA do modelo vem antes do produto.** O código de
`afa0cc014` volta (a tabela, `cs_soma_pecas`, `cs_pecas`, `cs_fecha`, os modos `POR_PECA`) e ganha UM passe novo,
`cs_trocos`: um GRUPO de `64` fios por CÓPIA, um fio por entrada do eixo dela, em pedaços de `64` com o transporte
do pedaço anterior. Cada fio lê o seu troço e calcula o `arco`; um prefixo SEGMENTADO (a cabeça é o troço com
`SUB_INICIO`) dá o `fim` de cada troço, e o `s0` é o `fim` da entrada ANTERIOR lido da memória do grupo — os MESMOS
bits dos dois lados de um vértice (somar o `0` de um cabeçalho de bloco não muda bits); `k` e a linha saem de dois
prefixos inteiros; `n0`/`n1` do `s0`/`fim`; um 2.º prefixo inteiro dá as peças antes de cada troço; o último troço
de um FECHADO escreve o total na linha do 1.º. O `cs_escreve` deixa de andar o tracejado (escreve a cabeça das linhas
e o estado da cópia). ⚠️ Não é o GRUPO recusado do §9.14: aquele EMITIA por cópia (duas vezes, com a fase em série do
fio `0` e `13` KB de memória de grupo); este só anda os troços, e a emissão continua a ser um fio por PEÇA. A override
nova `PASSEIO` (`0` em série no `cs_escreve`, o de `afa0cc014` · `1` o `cs_trocos`) cruza com o `POR_PECA`.
- **Variantes da rodada (uma só, a sonda intercalada):** `F` · `F1` (A/A) · `P1` (a prova com o passeio em série, o
  controlo do §9.18) · **`Q1` a PROVA** (`PASSEIO=1`, `POR_PECA=1`) · `Q4` (a ablação: o fio da peça sai logo — o
  passeio paralelo + as linhas + o prefixo) · `Q2`/`Q3` o produto (reserva atómica por aresta · contada por peça).
  O que as ablações GUARDAM: o `Q1` segura o `pedaco` por uma escrita que nunca acontece (como o `P1`); o `Q4` escreve
  as linhas (o passeio paralelo não tem nada que o compilador possa cortar: tudo o que ele calcula vai para a tabela).
- **Prova:** `escreve` do `Q1` nas esticadas tracejadas da iGPU `≤ 0,13` ms; acima ⇒ (1) fecha por recusa medida, com a
  decomposição (`Q4` e `Q1`).
- **Produto (`Q2` ou `Q3`, o melhor):** iGPU esticadas tracejadas `escreve ≤ 0,15` e soma `≤ 0,86`; NENHUMA célula da
  sonda pior que `+5 %` na iGPU nem `+10 %` na RTX (soma, contra o `F`). **A troca aceitável, escrita:** a melhoria
  só é exigida nas esticadas tracejadas; uma célula que piora dentro da faixa (`≤ +5 %` iGPU, `≤ +10 %` RTX) é o preço
  aceite; fora dela, o produto não entra. **A imagem:** um prefixo paralelo soma noutra ordem, logo «imagem = F» deixa
  de ser o critério — a sonda ganha a coluna «alfa máx. contra o F», e o critério é alfa `≤ 1` em TODAS as cenas; mais os
  `14` gates de `ph2d-shape-gpu` e os `motion_shape_placa` (o arnês do tracejado contra o Vello) nas duas placas. A
  reserva: igual (as arestas reservadas da coluna `arestas` não mudam). Mutações dos pedaços aceites.

**(2) A PAREDE DO VELLO NA ROTA PELA LEI.** Candidatos, medidos na MESMA rodada (`custo_do_encode_tracejado`):
`U` uma forma por TRAÇO — os polígonos de todas as peças, juntas e pontas de um traço da cópia num ÚNICO caminho
`nonzero` (um `fill` em vez de um por peça: o Vello paga por caminho a montagem, a caixa e as etiquetas); `U+A` o
mesmo e as arestas que a FAIXA partilha não se escrevem (como na placa: as duas metades cancelam-se). Critério: parede
`≤ +10 %` da rota de ANTES nas quatro células da sonda (densa e `16 384`, RTX e iGPU), o `encode` sem perder mais de
`10 %` do ganho (`−83 %` ⇒ `≤ −73 %`), a família do traço rente alfa `≤ 1` nas duas placas, as `4` famílias de hoje não
pioram (alfa `46`, `165` px). Senão, recusa medida com a tabela.

**(3) A PONTA E A JUNTA NO CARTÃO DA FORMA.** Dois parâmetros de escolha no `source.shape` (`ParamWidget::Enum`, o
padrão do `KIND`): `Cap` (Butt · Round · Square) e `Join` (Miter · Round · Bevel), omissão Butt/Miter = o produto de
hoje. ADR-0039 lido: o contrato é o `NodeManifest` de `8` CAMPOS; um `ParamSpec` a mais não é campo ⇒ o contrato não
muda. Critério: cada combinação (`3 × 3`) pela placa contra a rota Vello (alfa `≤ 1` na família do traço rente com os
valores novos) e pela rota do dispositivo; o catálogo/manifesto do nó; um projecto antigo (sem as linhas) abre com
Butt/Miter; ida e volta da gravação com os valores novos.

**(4) A PROVA DIRECTA DA CURA DO SIGSEGV DA RTX.** Binário de ANTES: `453354fb5^` (o `pass_profiler` antigo) com SÓ o
controlo A/A da sonda aplicado (`PH2D_SONDA_AA`, o `variantes()` da cura sem o `shutdown`), num worktree temporário
com `CARGO_TARGET_DIR` próprio; DEPOIS: o HEAD da linha antes desta onda (`d253432ac`). A MESMA sonda com `13`
variantes (`PH2D_SONDA_AA=10`), na RTX, `__GL_SHADER_DISK_CACHE=0` (o ambiente do `mede_intercalado.sh`), `N = 20`
corridas de cada, ALTERNADAS (antes, depois, antes, …), com a carga natural da máquina (as outras linhas e os builds
desta) ao lado de cada corrida. Critério: ANTES `≥ 2` falhas e DEPOIS `0` ⇒ provada; ANTES `0` ⇒ a condição não se
reproduz nesta máquina hoje, e o item fecha com a evidência que existe (o experimento `com × sem` do §9.18 F).

**(5) O CONTACTO EM PLAY — a medição que faltava.** A `=114` com `PH2D_PILHA_LADO` `32` · `64` · `128` (`1 024` · `4 096`
· `16 384` peças por taça; as duas taças: a da esquerda sem `Collide`), as duas placas, pelo `mede_formas_na_placa.sh`
(`release`) e o `[frame] MOTION`. Antes: confirmar que a cena ANDA no roteiro (a ponte do Motion faz `playhead.play()`
ao abrir a ferramenta, `motion_bridge_surfaces.rs`); se não andar, a porta `PH2D_PLAY=1`. Critério (o do §9.18 D): a
`4 096` peças o quadro da iGPU `> 16,7` ms com o contacto a ser o custo ⇒ a recusa expirou e o contacto da CAIXA no
dispositivo entra NESTA onda; senão a recusa renova-se com o número do app e o ponto de expiração escreve-se.
⚠️ O resultado de (5) muda o que se constrói (uma wave inteira, ou nada): por isso mede-se ANTES de construir, em série.

**(1) — o resultado: RECUSA MEDIDA, a prova falha por pouco e o produto por muito.** Código medido em `2774e1d17`
(`cs_trocos` + as variantes), retirado em `c524f9b3d`. A rodada intercalada (`7` variantes com o A/A `F1` × `6`
cenas × `7` rodadas, as duas placas, carga `11`–`24`; `target/prova/onda4/r1_passeio_paralelo.txt`). `escreve`
mínimo, ms, e a soma contra o `F`:

| esticadas tracejadas | iGPU `escreve` | iGPU soma | RTX `escreve` | RTX soma |
|---|---:|---:|---:|---:|
| `F` (o produto) · `F1` (A/A) | `0,239` · `0,237` | `0,948` · `−0,6 %` | `0,159` · `0,159` | `0,301` · `+0,1 %` |
| `P1` a prova do §9.18 (o passeio em SÉRIE) | `0,182` | — | `0,109` | — |
| **`Q1` a PROVA** (o passeio em PARALELO + a tabela + o `pedaco`) | **`0,134`** (critério `≤ 0,13`) | — | `0,070` | — |
| `Q4` a ablação (o fio da peça sai logo) | `0,116` | — | `0,067` | — |
| `Q2` o produto, reserva atómica | `0,245` | `+0,2 %` | `0,076` | `−27,6 %` |
| `Q3` o produto, contado por peça | **`0,226`** (critério `≤ 0,15`) | **`0,930`** (critério `≤ 0,86`) | `0,075` | `−27,9 %` |

Nas outras células: densas tracejadas `Q3` **`+21,9 %`** iGPU e **`+15,7 %`** RTX (`escreve` `0,390` contra `0,147`);
conformes tracejadas `+3`–`+8 %` (os despachos que correm vazios); as contínuas iguais. A imagem: `Q2`/`Q3` alfa
**`1`** contra o `F` em todas as células (o critério de imagem passava). ⇒ **o passeio deixou de ser o chão**: em
paralelo ele custa `Q4 − (despachos)` ≈ `0,10` na iGPU (o `P1 → Q1` tira `0,048`), e a RTX ganha `28 %`. **O chão
passou a ser a EMISSÃO por peça:** `Q3 − Q1 = 0,092` ms na iGPU (`2` passes do `emite_pedaco`: contar e escrever, ALU
de uma placa de `2` CUs), e nas densas (`1 225` cópias, `~26` peças cada) o fio por cópia já enchia a placa — por peça
só se paga a tabela, as buscas e a emissão dupla. A conta que fecha a porta: com a prova a `0,134`, o produto só
caberia em `0,15` se a emissão custasse `≤ 0,016`; ela mede `≥ 0,045` por passe. **Quem voltar:** a única porta é
uma emissão de UM passe sem atómico por aresta (um sítio fixo por peça, o enchimento pago no `cs_deposita`) E um
seletor por cópia (as densas pelo fio da cópia) — duas topologias novas, e a conta acima diz que nem assim a iGPU
chega a `0,15`.

**(4) — o resultado: PROVADA.** `40` corridas alternadas na RTX (`13` variantes, `__GL_SHADER_DISK_CACHE=0`, carga
natural `3,3`–`38`; `target/prova/onda4/sigsegv_antes_depois.tsv`): o binário de ANTES (`453354fb5^` + só o A/A)
morreu com `SIGSEGV` (`rc 139`) DEPOIS do `test result: ok` em **`14/20`**; o de DEPOIS (`d253432ac`) **`0/20`**. A falha
de antes deu-se também com a máquina calma (`load 3,3`): a carga não era a condição — era o perfilador a segurar o
dispositivo até ao `exit`, como o §9.18 F isolou. Worktree temporário removido.

**(5) — a medição: a recusa do doc 115 EXPIROU, e não pelo contacto.** A `=114` no app (`release`, iGPU, `PH2D_PLAY=1`,
a taça a crescer com a pilha; `target/prova/onda4/pilha/`):

| peças por taça | `Collide` | quadro | `[frame] MOTION` (cozer + separar) |
|---|---|---:|---:|
| `1 024` | OFF (o controlo) | **`16,7` ms (`60` fps)** | `0,97` ms |
| `4 096` | OFF | **`16,6` ms (`60` fps)** | `2,07` ms |
| `1 024` | ON | **`247` ms (`4` fps)** | **`151,6`** ms |
| `4 096` | ON | `> 2 000` ms (nenhuma janela de `120` quadros em `45` s; a barra lê `2,2`–`2,8` s) | — |

⇒ o quadro passa os `16,7` ms já a `1 024` peças (**a recusa expirou** — o resultado continua abaixo, depois do (2)).

**(2) — o resultado: RECUSA MEDIDA, e o candidato «um caminho por traço» já era o estado.** O `TracoDaPlaca`
já entrega UM `fill` por cópia (todas as peças, juntas e pontas de um traço num `BezPath`): o `U` não tinha nada
a construir. O `U+A` (a FITA: os pedaços que se ligam pela faixa num só polígono, sem a aresta partilhada) foi
construído e medido (`e115108de`, retirado em `274ed670e`). A sonda `custo_do_encode_tracejado` (`3` variantes
intercaladas, `15`/`5` rodadas, carga `6`–`16`; `target/prova/onda4/parede_vello_{igpu,rtx}.txt`), mais a contagem
do que o Vello recebe:

| a `=127` tracejada | `encode` antes → lei → lei+fita | segmentos para o Vello | parede RTX (lei · fita) | parede iGPU (lei · fita) |
|---|---|---|---|---|
| densa `35 × 35` | `10,9` → `2,7` → `2,7` ms (`−75 %`) | `49 000` → `120 050` → `120 050` | `+4,8 %` · `+4,3 %` | **`+24,4 %` · `+24,8 %`** |
| `16 384` cópias | `143,6` → `38,6` → `36,4` ms (`−73`/`−75 %`) | `655 360` → `1 605 632` → `1 605 632` | **`+19,7 %` · `+18,0 %`** | **`+42,1 %` · `+41,2 %`** |

⇒ a fita **não funde nenhum pedaço** nestas cenas (a contagem de segmentos é a mesma ao segmento): a faixa dos
dois lados de um vértice é calculada a partir de argumentos em ORDEM diferente (`(a, b, c)` de quem chega,
`(d, a, b)` de quem parte), os bits não coincidem, e a fita só continua sobre os MESMOS bits (uma fusão por
tolerância deixaria uma lasca por cobrir ou a dobrar). E a conta fecha a porta mesmo a uma fusão perfeita: a rota
pela lei manda `2,45×` os segmentos (`98` por cópia contra `40`), e a parede sobe `24`–`42 %` com isso (uma
elasticidade de `0,17`–`0,29` por cada `+100 %` de segmentos); a fusão máxima possível (dois quadriláteros que
viram um hexágono: `−25 %` das arestas de um pedaço que cruza um vértice) não leva a iGPU a `+10 %`. O critério
do `encode` passava (`−73 %`, perda `≤ 10 %` do ganho) e o arnês do traço rente alfa `1` nas duas placas; a
PAREDE falha em três das quatro células. **A troca que fica (a do §9.18 C):** o quadro da rota Vello (`encode` +
parede) continua `58`–`79 %` mais curto que o de antes. **Quem voltar:** o custo é o volume de segmentos que a CPU
expande — o caminho é a rota Vello mandar ao Vello o traço por EXPANDIR (o caminho fonte e o tracejado, como o
traçador dele faz) e corrigir só as peças que mordem; sem isso, nada no lado da CPU fecha os `+10 %`.

**(5) — a causa, e a medição de novo (SÉRIE, escrita antes de medir).** Lido o caminho da CPU antes de levar
o contacto para a placa: o `sim.step` chama `ph2d_contact::separate` (pela grelha) e depois
`ph2d_contact::impulsos` — e o `monta` dos impulsos percorria **todos os pares** (`for lo, for hi > lo`):
`O(n²)` por sub-passo, `×8` sub-passos da zona. A `1 024` peças com colisor são `524 k` testes de par por
sub-passo; a `4 096`, `8,4` milhões. ⇒ a separação que o doc 115 mediu (`1 024` peças `0,5` ms) era a metade
barata; a cara estava ao lado dela. **A cura, exacta:** o `monta` tira os pares da MESMA grelha da separação —
superconjunto dos contactos, em ordem crescente, logo a mesma lista de restrições pela mesma ordem e o
Gauss–Seidel dá os mesmos bits (gate `os_impulsos_pela_grelha_dao_os_bits_de_todos_os_pares`: caixas rodadas,
discos, uma peça GRANDE e um obstáculo, material e deslize — velocidade, giro e spin iguais ao bit). A sonda do
cozimento (`custo_do_cozimento_da_pilha`, as duas taças, com o `advance_tick` — ⚠️ a 1.ª redacção dela não
avançava a simulação e lia `0,02` ms, a leitura do estado): `2 048` peças `1,04` ms por tique, `8 192` peças
`2,57` ms (mediana; `Collide` OFF `0,15` · `0,41`).
⇒ **SÉRIE, e porquê:** o critério do §9.18 D é sobre o QUADRO DO PRODUTO, e o produto mudou — a cura tirou o
termo que fazia o quadro passar os `16,7` ms. A régua do app corre outra vez sobre o produto curado (as mesmas
células: `1 024` · `4 096` · `16 384` por taça, `Collide` ON e OFF, as duas placas). **O critério, o mesmo:** a
`4 096` por taça, quadro da iGPU `> 16,7` ms ⇒ o contacto da caixa vai para a placa nesta onda; `≤ 16,7` ⇒ a
recusa do doc 115 renova-se com o número do app, e a expiração escreve-se onde o quadro passar dos `16,7` ms. ⚠️ E a premissa do doc 115 não era a do
produto: ela mediu a SEPARAÇÃO na CPU (`1 024` peças `0,5` ms, §9.18 D), mas com uma forma com `Collide` a cerca do
`motion_bridge_gpu_colisor.rs` derruba o cozimento INTEIRO para a CPU — a queda, o vento, a taça e as `2 048` peças das
duas taças pagam o caminho da CPU (`151` ms contra `1`–`2` no dispositivo). ⇒ a cura é o contacto da CAIXA no
dispositivo, para a cerca deixar de cair: entra nesta onda.
⚠️ Dois defeitos da régua achados pela foto antes de medir: (a) o roteiro abria a cena PARADA (o arranque pausa o
relógio, e o auto-play do Motion só corre ao abrir a ferramenta) — a porta `PH2D_PLAY=1`
(`motion_bridge_surfaces.rs`, toca enquanto o relógio estiver parado em `t = 0`); (b) o `PH2D_PILHA_LADO` crescia a
pilha e não a taça: `4 096` quadrados (`198` u²) numa taça de `10` u² nasciam sobrepostos — a régua media a
vizinhança degenerada. A taça cresce agora com a pilha (`medida_de`; a cena do smoke fica byte a byte).

**(5) — a medição sobre o produto curado** (o mesmo roteiro, `release`, `PH2D_PLAY=1`, as três últimas janelas de
`120` quadros; `target/prova/onda4/pilha_curada/`):

| peças por taça | `Collide` OFF (iGPU · RTX) | `Collide` ON iGPU: quadro · `MOTION` | `Collide` ON RTX: quadro · `MOTION` |
|---|---|---|---|
| `1 024` | `16,6` · `16,7` ms | `32,1` / `16,7` / `16,8` ms · `2,7`–`11,2` | `16,7` (×3) · `4,5`–`8,2` |
| `4 096` | `16,7` · `16,7` | **`17,0` / `20,0` / `28,0`** · `31`–`32` (picos `~480`) | `16,7` / `17,6` / `71,0` · `23`–`25` (picos `~380`) |
| `16 384` | `16,7` · `16,7` | `283` / `206` / `126` · `97`–`132` (picos `~840`) | `26,6` / `44,8` / `45,8` · `97`–`100` |

⇒ a cura levou `1 024` peças de `247` ms a `60` fps, mas a `4 096` a iGPU passa dos `16,7` ms: **pelo critério, o
contacto da caixa vai para a placa**. ⚠️ Os picos (`~480` ms a `4 096`) vêm a cada recomeço da queda (a zona em
`Loop`, `3,6` s): o quadro do recomeço paga a separação das peças a nascer.

**(5) — a PROVA DO MODELO antes da placa (two-strikes; escrita antes de construir).** O passo da CPU é
`separar` (Jacobi com média — já paralelo, o kernel dos discos é ele) e `impulsos` (**Gauss–Seidel**: `8`
iterações sobre as restrições, cada uma a ler a velocidade que a anterior acabou de escrever). A placa não corre
um Gauss–Seidel (uma restrição por fio em série seria `~80 k` passos em série por sub-passo a `4 096` peças); ela
corre um **Jacobi com média** — cada restrição lê a fotografia da iteração e cada peça aplica a média do que os
contactos dela pedem. É OUTRA lei, e a pilha desta cena custou quatro reports do dono ao zumbido (doc 111). ⇒ a
prova corre a variante Jacobi NA CPU (as mesmas restrições, a mesma lei por restrição — o `resolve_um` — só a
composição muda; `PH2D_CONTACT_JACOBI=<iterações>`), na `=114` (`25` peças, a do smoke, e `1 024`), e compara com o
Gauss–Seidel no instante `2,6` s (a pilha assentada, antes do recomeço). **Critério (cada um, `8` OU `32`
iterações):** a distância mediana ao vizinho a `±5 %` da do Gauss–Seidel; a velocidade média da pilha
`≤ 1,5 ×` a dele `+ 0,01` u/s; a sobreposição mais funda `≤` a dele `+ 2 %` do lado; e os gates da pilha
(`the_rolling_on_the_card_calms_the_pile`, `the_pieces_tumble…`, `only_the_half_whose_shape_collides…`) verdes com ela.
Passa ⇒ o modelo do dispositivo está provado e constrói-se o kernel (separação + impulsos Jacobi + a cerca a
cair para a caixa). Falha ⇒ o contacto da caixa NÃO vai para a placa nesta forma (recusa medida: a lei da placa
não segura a pilha que o dono aprovou), e o item fecha com a cura da CPU e o ponto onde o quadro passa.

**(5) — a prova, 1.ª leitura, e a correcção da régua (escrita antes da 2.ª).** No instante `2,6` s: pilha de `25`
(o lado `5`, com a taça da porta) — Gauss–Seidel vizinho `0,2222` · velocidade `0,129` u/s · sobreposição `24,9 %`;
Jacobi `8` `0,2140` · `0,168` · `23,1 %`; Jacobi **`32`** `0,2214` (`−0,4 %`) · `0,113` · `14,3 %`; Jacobi `64`
`0,2237` · `0,096` · `11,9 %` ⇒ a `32` e a `64` passam. Pilha de `1 024`: Gauss–Seidel `0,0866` · **`0,740` u/s** ·
`105 %`; Jacobi `8` `0,0417` · `32` `0,0743` · `64` `0,1005` · `128` `0,1313` — sem ordem nenhuma em torno do
Gauss–Seidel. ⚠️ **A régua estava no instante errado para `1 024`:** a `0,74` u/s a própria lei do produto ainda cai
(a taça cresceu com a pilha, e a queda é mais longa); o critério pedia «a pilha assentada» e o `2,6` s só o é na
pequena. ⇒ a pilha de `1 024` mede-se no FIM DA PAUSA (`3,5` s, antes do recomeço), com o MESMO critério; e um
controlo: a velocidade do Gauss–Seidel lá tem de ser `< 0,1` u/s, senão o instante ainda não é repouso.
Os gates da pilha sob Jacobi `32` e `64`: `11/11` verdes.

**(5) — a prova, 2.ª leitura: o Jacobi REFUTADO, o Gauss–Seidel POR CORES aceite.** No `2,95` s (o último instante
com peças antes do recomeço; no intervalo da pausa a cena não emite peças) a pilha de `1 024` AINDA cai: o
controlo falha (Gauss–Seidel `0,266` u/s `> 0,1`) — não há repouso nesta cena a esta escala, e o critério compara
as leis no MESMO instante. Pilha de `1 024`, `2,95` s:

| lei | vizinho mediano | velocidade média | sobreposição mais funda |
|---|---:|---:|---:|
| Gauss–Seidel (o produto) | `0,0796` | `0,266` u/s | `109,2 %` do lado |
| Jacobi `32` | `0,0613` (`−23 %`) | `0,493` (`1,85×`) | `104,9 %` |
| Jacobi `64` | `0,0868` (`+9 %`) | `0,488` (`1,84×`) | `105,2 %` |
| Jacobi `128` | `0,1191` (`+50 %`) | `0,531` (`2,0×`) | `105,9 %` |
| **Gauss–Seidel por CORES** | `0,0758` (`−4,8 %`) | `0,258` (`0,97×`) | `108,5 %` |

⇒ **o Jacobi falha** (a pilha de milhares quase o dobro mais agitada, a compactação fora de `±5 %` em qualquer
número de iterações) — passa na de `25` e não onde o dispositivo é preciso. **O Gauss–Seidel por cores passa**
na de `1 024` (no `2,6` s também: `−3,1 %` · `1,05×`). ⚠️ E ele é a MESMA lei sequencial com outra ordem, logo a
diferença dele é a banda que a ordem sozinha produz numa cena caótica — o controlo A/A que o critério não tinha;
na pilha de `25` a sobreposição mais funda dele saiu `+4,4 %` do lado (o critério pedia `≤ +2 %`): um MÁXIMO de
`25` peças, dentro da banda da ordem. (Memória: critério de semelhança sem A/A.)
**E o custo é o contacto, não o resto:** a sonda `custo_do_tique_do_app_na_pilha` (o `advance_or_scrub_scoped` da
ponte, `480` tiques, com e sem o passe do sink): `1 024` por taça mediana `2,8` ms, mas `14`–`20` ms por tique
com a pilha formada (tique `117`+); `4 096` mediana `9,5`, **`48`–`93` ms por tique** com a pilha formada; o passe
do sink não pesa (com e sem: iguais). ⇒ a `4 096` peças a CPU paga `48`–`93` ms de cozimento por tique com a pilha
formada — o quadro do app não cabe em `16,7` ms em Play (⚠️ a tabela do app acima NÃO é Play: ver a correcção da
régua no fim do §9.19). **Decisão:** o contacto da caixa vai para a placa
com o Gauss–Seidel por cores.

**(5) — a prova de CUSTO do modelo no dispositivo.** O contacto por cores pede, por tique, `8` sub-passos × (a
separação: `8` varreduras com a grelha · a montagem das restrições · as cores · `8` iterações × as cores) —
`~1 000` despachos dependentes. A sonda `custo_de_um_despacho` (`k` despachos de `4 096` fios, cada um a ler o que
o anterior escreveu, o mínimo de `9`): **um passe com `1 000` despachos `1,6` ms (`1,57`–`1,58` µs por despacho,
iGPU e RTX)**; um passe por despacho `8,7`–`8,9` µs. ⇒ dentro de um passe o cabeçalho cabe (`~1,6` ms por tique);
o desenho junta os despachos de um sub-passo num passe. E a CPU não chega lá sozinha (medido com relógio
temporário, uma taça de `4 096` com a pilha formada, por sub-passo: separar `~3,0` ms + impulsos `~4,0` ms ⇒
`~56` ms por tique com `8` sub-passos; paralelizar só os impulsos deixava `~28`).

**(3) — o resultado: CONSTRUÍDO.** O `source.shape` ganhou `Cap` (`stroke_cap`: Butt · Round · Square) e `Join`
(`stroke_join`: Miter · Round · Bevel), `ParamWidget::Enum`, apendados ao `ALL` (entram na chave da geometria) e
ao `SPECS` com omissão `0`/`0` (= o traço de sempre; um projecto antigo, sem as linhas, abre Butt/Miter); visíveis
só com `Stroke Width > 0` (como o `Dash`); i18n em `node_params.rs`/`node_options.rs`
(`node.opts.node_motion_shape.{cap,join}_labels.N`). O `build_shape_path` põe-nos no `StrokeSpec`
(`mistura::traco::ponta_e_junta`, `motion_shape_gen.rs` fica em `672` linhas). A placa e a rota Vello já honravam
as três pontas e as três juntas pelo eixo (`junta_de`/`ponta_de`). ADR-0039 lido: o contrato é o `NodeManifest`
de `8` campos e um `ParamSpec` não é campo — nada congelado mudou. Gates: `the_cap_and_the_join_are_declared_keyed_
read_and_labelled` (nó) · `the_cap_and_join_of_the_card_reach_the_stroke_spec` · `the_cap_and_join_labels_resolve_
to_words` · `the_cap_and_join_survive_the_save_and_an_old_project_opens_butt_and_miter` (o caminho do produto:
gravar → ler → publicar) · `the_stroke_cap_and_join_survive_the_round_trip_and_an_old_doc_has_neither`
(`ph2d-motion-doc`) · **`a_ponta_e_a_junta_do_cartao_desenham_como_a_placa`** (as `3 × 3` combinações, estrela e
engrenagem, esticadas e conformes, rota Vello × placa alfa `≤ 1`, com o controlo de que a ponta e a junta mudam a
área pintada). Porta do smoke: `PH2D_TRACO_ESTICADO_PONTA=0|1|2`.

**As mutações** ([`mutacao_a_onda_4_2026-10-05.py`](../../ferramentas/mutacao_a_onda_4_2026-10-05.py), sozinho na
árvore, pré-voo `12/12`, corrida limpa verde `27` testes com o gate de imagem na RTX): **`12/12` sangram** —
`c1` a ponta redonda chega rente · `c2` o chanfro chega esquadria · `c6` o `build_shape_path` ignora as duas (as
três reprovam o `StrokeSpec`, a gravação e a IMAGEM) · `c3` o índice trocado · `c4` a ponta fora da chave · `c5` a
omissão redonda · `c7` os rótulos trocados (o gate do nó) · `i2` a grelha perde vizinhos · **`i1` o par consigo
próprio e `i3` a grelha a metade do alcance SOBREVIVERAM à 1.ª corrida**: a referência «todos-os-pares» passa pelo
MESMO filtro (o `i1` mudava as duas por igual) e a fixtura tinha sempre a peça grande perto de todas (o `i3` não
perdia par nenhum). O gate passou a comparar a LISTA das restrições, a exigir `lo < hi` e a correr também sem a
peça grande — os dois sangram · `p1` a taça não cresce · `p2` a cena do smoke muda (o gate da taça).

**(5) — a DECISÃO: o contacto da caixa NA PLACA fica como a única coisa em aberto desta onda, com o modelo e o
custo provados.** O que se construiu: a cura do `O(n²)` dos impulsos (`1 024` peças por taça `247` ms → `60` fps,
exacta ao bit), a porta da taça, as sondas (o tique do app, a prova dos impulsos, os despachos).
O que se provou para a placa: o Jacobi dos impulsos NÃO serve (refutado), o Gauss–Seidel POR CORES serve (a banda
da ordem), e `1 000` despachos num passe custam `1,6` ms. **Porque não se construiu nesta janela:** não é «um
kernel»; o inventário (agente, 05/10) mostra QUATRO peças, e duas são topologias novas para o cozimento da placa —
(a) um estágio de VÁRIAS passadas feito à mão logo a seguir ao `sim.step` (a grelha, a separação com caixas e o
giro por varredura, a montagem das restrições numa lista, as cores por rondas, `8` iterações × as cores), (b) os
SUB-PASSOS da zona no dispositivo (hoje a `sim.zone` é passagem no sequenciador da placa), (c) as colunas do
colisor a viajarem pela rota da forma no dispositivo e o `sim.collide` a pousar uma caixa pelo suporte, (d) a lei
de CPU passar a ser a mesma das cores (as cores por rondas com prioridade por par, determinísticas nas duas mídias,
para a paridade por passo ser `ε`). Meio-construído isto seria pior que a cerca de hoje (a cena cai inteira para a
CPU e fica CERTA). **O ponto onde a recusa expirou, escrito:** com a cura, a `4 096` peças por taça o cozimento da CPU paga `48`–`93`
ms por tique com a pilha formada (a sonda do tique do app); a `1 024`, `14`–`20` ms.
Prompt da próxima janela: [`HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_CONTACTO_NA_PLACA.md`](../../handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_CONTACTO_NA_PLACA.md).

**⛔ (5) — a régua do app, CORRIGIDA DUAS VEZES (a 2.ª em 06/10, depois do smoke do dono).** Em 05/10 escrevi aqui
que a `=114` não caía no roteiro de foto e que as tabelas do app mediam a cena parada — **errado**: a demo força a
ferramenta `motion` (que entra em Play) e a cena cai sozinha; a foto tinha apanhado o RECOMEÇO do ciclo (a queda
recomeça a cada `3,6` s). A porta `PH2D_PLAY` era desnecessária e saiu. O que estava mesmo errado era outra coisa:
**a janela de `120` quadros do `[frame]` cobre um ciclo inteiro e a média esconde o instante da pilha**, e eu dei ao
dono um smoke de desempenho em `--profile smoke` (sem LTO; o `CLAUDE.md` §2 manda `--release` para isso). O dono viu
`≤ 10` fps no instante denso. ⇒ a régua nova `[motion-quadro]` (`PH2D_MOTION_RELOGIO=1`, `motion_bridge_relogio.rs`):
a cada `30` quadros a parede, o Motion médio e máximo e os tiques por quadro. Pior janela por ciclo, `=114` com
`Collide` (`target/prova/onda4/relogio/`):

| peças por taça | perfil | antes da montagem em paralelo | com a montagem dos pares em PARALELO (os mesmos bits) |
|---|---|---|---|
| `1 024` | `release` | `56`–`60` fps; alguns recomeços `16`–`18` | **`60` fps em todas as `38` janelas** |
| `1 024` | `smoke` | `12`–`20` fps no instante denso (`3`–`8` tiques por quadro) | `23`–`32` fps |
| `4 096` | `release` | `10`–`15` fps | `13`–`25` fps |
| `4 096` | `smoke` | — | `8`–`9` fps |

⇒ a queda corre a `60` fps; o quadro cai no INSTANTE MAIS DENSO da pilha (o fim de cada ciclo): o tique passa do
orçamento e o relógio passa a recuperar até `8` tiques por quadro, o que o afunda. **A montagem dos pares em
paralelo** (`impulso.rs`: um balde por `lo`, juntos pela ordem de `lo`; gate de bits verde) tirou o `release` de
`1 024` por taça do mergulho. A `4 096` e no perfil `smoke` o mergulho continua — é o que o contacto na placa cura.
A decisão do item (5) não muda.
Mutação da montagem em paralelo: juntar os baldes ao contrário (`rev()`) SOBREVIVEU ao gate de bits (a referência
passa pela mesma junção); o gate passou a exigir a ordem `(lo, hi)` crescente e ela sangra.

**(5) — o ORÁCULO (06/10, depois de o dono dizer *«estamos muito aquém da performance de uma Unity»*).** A física 2D do
Unity é o Box2D; o PH2D já tem um motor da mesma família, o `rapier2d` `0.35.3` (o do módulo de Física, ADR-0131,
Apache-2.0). A MESMA pilha corrida nele, fora do repositório (`ferramentas/oraculo_rapier_pilha/`: `k × k` caixas
de meio-lado `0,11` numa taça de segmentos com o raio da `medida_de`, gravidade `4`, `1/60` s por tique, `1`
passo e `4` iterações — a omissão dele —, `6` s; `release`, carga `7`–`15`; duas corridas cada, iguais):

| caixas | rapier, `1` núcleo (tique med · p95) | rapier, `parallel` | o nosso contacto por tique (a pilha formada) |
|---|---|---|---|
| `1 024` | **`0,40`** · `0,52` ms | `0,48`–`0,56` · `0,61`–`0,70` | `~15`–`18` ms (`2,2` ms por sub-passo × `8`) |
| `4 096` | **`2,9`–`3,0`** · `4,1`–`4,2` | `1,7` · `2,3` | `48`–`93` ms |
| `16 384` | `15,8`–`18,5` · `33`–`37` | `5,8`–`7,0` · `13` | — |

As pilhas assentam (a peça mais funda a `0,11` do fundo: pousada pela face). ⇒ **o nosso contacto é `~20`–`30×` mais
lento que um motor da família do Box2D na mesma CPU, e a diferença é de ALGORITMO, não de placa:** por tique o
nosso refaz cada contacto `64` vezes (`8` sub-passos × `8` varreduras de separação, a grelha reconstruída a cada
uma, e mais `8` iterações de impulso sobre manifestos refeitos), sem lembrar nada do tique anterior; o rapier (e o
Box2D) guarda os contactos entre tiques (fase estreita incremental), aquece o solver com os impulsos da vez
anterior e resolve com `1` passo e `4` iterações. ⇒ **levar o NOSSO algoritmo para a placa portaria o desperdício**:
o item aberto muda de pergunta — antes da placa, o contacto das peças do Motion pelo motor da casa (o `rapier2d`
da Física) ou o nosso reescrito como ele (contactos persistentes, aquecimento, menos passos). ⚠️ Isto muda o
COMPORTAMENTO da pilha que o dono aprovou (doc 109–111: o zumbido curado com os `8` sub-passos): é decisão de
produto e pede o smoke dele.

