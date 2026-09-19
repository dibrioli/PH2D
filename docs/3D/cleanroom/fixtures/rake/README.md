# Corpus do PENTE DE TOPOLOGIA — proveniência e a RÉGUA

> ⚠️ **O CABEÇALHO DE CADA FICHEIRO É A FONTE; esta prosa nunca é.** Uma grandeza nova que não
> entre no cabeçalho esconde-se debaixo da frase que descreve o corpus, e **nenhuma varredura a
> acusa**.

## §1 — Proveniência

| | |
|---|---|
| **Entradas** | **NOSSAS**, geradas pelo harness: um plano `28×28` células de lado `2,4`, triangulado com diagonais **alternadas** e com abanão determinístico de `0,55` da célula nos vértices interiores (a entrada canónica, `entrada_9fb3d9dea0d0.txt.gz`); mais um plano de **quads**, um plano com **furo** (bordo aberto interior) e uma esfera UV. ⛔ Nenhum asset do alvo entra como entrada. |
| **Saídas** | Lista de posições e de faces devolvida pelo alvo, corrido **sem interface** por script sobre as entradas acima. Saída de programa é **dado** (GPLv2 §0). |
| **Alvo corrido** | binário **5.2.2 LTS**, pacote `17:5.2.2-1`, build **2026-09-15**. ⚠️ **As obras anteriores desta linha correram `5.2.1-2`** — este corpus e os delas **não vêm do mesmo binário**, e é por isso que a versão está no cabeçalho de cada célula. |
| **Data da colheita** | 2026-09-17 |
| **Quem colheu** | o subagente **E**; o harness vive **fora da árvore** e não viaja com ela. |

Cada célula é `<família>/<nome>.txt.gz`, com o cabeçalho `# chave=valor`, depois o percurso do cursor
(uma linha `# ponto x y z` por ponto), depois `V n` + `n` posições e `F m` + `m` faces.
A **entrada** de cada célula é nomeada no cabeçalho (`malha_de_entrada=`) e vive em `entrada/`.

⚠️ Os valores como `DRAW`, `SMOOTH`, `CONSTANT`, `SUBDIVIDE_COLLAPSE` são **identificadores públicos
de enumeração** da interface do alvo, presentes só porque são a **chave de regeneração** da célula —
é por eles que uma corrida futura reproduz o mesmo enquadramento.

## §2 — A RÉGUA, e porque ela é esta

A grandeza desta obra é a **direcção do fluxo de arestas**, não a posição. Para cada aresta cujo
ponto médio caia dentro da pegada, mede-se `α ∈ [0°, 90°]`, o ângulo entre a aresta e a direcção do
traço, ambos projectados no plano do ecrã (uma aresta não tem sentido, logo o ângulo dobra-se). A
régua é o **parâmetro de ordem de quatro dobras**:

```
Q = média( cos 4α )           Q = +1  toda aresta a 0° OU a 90° do traço  (uma GRADE alinhada)
                              Q =  0  isotrópico
                              Q = −1  toda aresta a 45° do traço
```

⛔⛔ **A régua ÓBVIA — a de duas dobras, `S = média(cos 2α)` — é CEGA a este fenómeno, por
construção**, e foi a primeira que se escreveu. Ela vale `+1` a 0° e `−1` a 90°, logo uma grade
alinhada, que tem as duas famílias, dá-lhe `≈ 0`. Medido nas quatro rotações do traço:

| rotação do traço | `S` desligado | `S` ligado | `Q` desligado | `Q` ligado |
|---|---|---|---|---|
| 0° | +0,0412 | +0,0595 | −0,0456 | **+0,1243** |
| 22,5° | +0,0327 | **+0,0215** | −0,0154 | **+0,0901** |
| 45° | +0,0157 | +0,0368 | +0,0273 | **+0,1491** |
| 67,5° | +0,0535 | +0,0547 | −0,0123 | **+0,0940** |

*A régua de duas dobras **troca de sinal** entre rotações do mesmo traço (a `22,5°` ela DESCE) e o
seu `Δ` é `5×` a `90×` mais pequeno; a de quatro separa nas quatro rotações, sempre no mesmo sentido
e nunca por menos de `+0,10`.* ⇒ **a régua escolheu-se MEDINDO**, e o que ela mede é uma **grade**, não uma
direcção só.

⚠️ **A PEGADA É UM DISCO DE ECRÃ, e medi-la em 3D dá `None`.** A primeira redacção media a distância
do ponto médio ao percurso em três dimensões. Com duas passagens o relevo sobe a `0,24` e a região
de raio `0,175` fica **vazia** — a régua devolvia *«não há arestas»* sobre uma célula perfeita. As
fixturas planas são vistas de topo em projecção ortográfica, logo **a projecção XY é o ecrã**.

⚠️ **A região usada nos números abaixo é `raio/2`** e não o raio inteiro. Medido: a `raio/2` a
separação é `2`–`3×` maior (0° ligado: `+0,0353` no raio inteiro contra `+0,1243` a `raio/2`; desligado, `−0,0632` contra `−0,0456`) —
**o efeito concentra-se na faixa central do traço**, e o raio inteiro dilui-o com a orla.

⛔ **A régua é PLANAR.** Numa esfera ela lê o mesmo valor com o pente ligado e desligado
(`+0,0453` nos dois) enquanto a malha mudou mesmo — `2 032` faces e `3 048` arestas dos dois lados,
com **`1 260`** de diferença simétrica no conjunto de arestas: ali as arestas junto da
silhueta projectam curtas e o ângulo projectado não é o ângulo na superfície. ⇒ **dívida NOMEADA:
uma régua para malha curva projecta a aresta e a direcção do traço no plano tangente local.** Os
números deste corpus valem para as fixturas **planas**.

## §3 — O VALE, e de onde sai a barra

⛔ **A população tem de ser UMA COISA.** A primeira tentativa varreu «tudo o que tem detalhe
constante» e devolveu um vale **invertido**, porque lá dentro estavam células de outra malha de
entrada, células sem passe de refino e a esfera. A população é:

> `malha_de_entrada = entrada_9fb3d9dea0d0.txt.gz` · `topologia_dinamica = armada` ·
> `modo_de_detalhe = CONSTANT` · `passe_de_refino = SUBDIVIDE_COLLAPSE` ·
> `resolucao_do_detalhe = 18` · `PASSAGENS = 1` · **sem** `SEQUENCIA_DE_TRACOS` ·
> `pontos_usados_truncagem = 10` (percurso inteiro) ·
> e o **controlo**: o verbo subdividiu (`v_saida > 1,5 × v_entrada`).

⚠️ **Todos os nove critérios são CHAVES DO CABEÇALHO**, de propósito: a 1.ª redacção descrevia-os em
prosa e um terceiro a aplicá-los obteve outro `n` que o declarado.

| | n | o pior da população | célula |
|---|---|---|---|
| **pente desligado** | **32** | `Q = +0,02982270` (o MAIOR) | `escada/k_a0450_p0000` |
| **pente no máximo** | **32** | `Q = +0,06323128` (o MENOR) | `verbos/v_blob_p100` |

⛔⛔ **Este `n` é a TERCEIRA redacção do mesmo número, e as duas anteriores (`37`, `34`) estavam
erradas** — a segunda **depois** da cura desenhada para o tornar reprodutível. O `32` sai de aplicar
as nove chaves **à letra**, como uma comparação de strings do cabeçalho; largar qualquer uma de
`malha_de_entrada`, `passe_de_refino` ou `resolucao_do_detalhe` devolve `34`.
⭐ **E a barra é ROBUSTA a isso, medido:** com `32`, `34` e `35` as duas células extremas são as
MESMAS e as reprovações são **zero dos dois lados**. ⇒ o `n` é escrituração, não segurança — mas é
escrituração que já falhou três vezes, e é por isso que ele vai agora com a receita ao lado.

**VALE = `[+0,0298 , +0,0632]`**, largura `0,0334`.
⭐ **Os dois lados são saída do PRÓPRIO alvo** — o lado «aprovado» é o alvo com o pente ligado —,
que é a condição que duas barras do corpus do tecido desta casa foram retiradas por não cumprir.

**BARRA = `+0,046527`, arredondada a `+0,0465` — o MEIO do vale, e ela é UMA SÓ:** `Q ≥ +0,0465`
para *«o pente está no máximo»* e `Q ≤ +0,0465` para *«o pente está desligado»*, com margem
**`±0,0167`** para as duas células extremas.
⛔⛔ **A 1.ª redacção dava `Q ≤ +0,0298` para o lado desligado — o próprio extremo, arredondado a
quatro casas PARA BAIXO** — e a célula que o gerou mede `+0,02982270`: margem `−0,0000227`, ou seja
**o gate nascia vermelho sobre a saída do próprio alvo**. *Uma barra tirada de um extremo tem margem
zero por construção e o arredondamento decide o sinal dela.*

⚠️ **A barra separa DESLIGADO de MÁXIMO, não desligado de qualquer posição.** A meio curso os
valores atravessam o vale (ver a escada no `SPEC`), e é isso que a torna honesta.

## §4 — O ORÁCULO NÃO REPETE, e a banda está medida

Seis corridas da **mesma** célula (`banda/b_con*`):

| | v de saída | `Q` | amplitude |
|---|---|---|---|
| **pente desligado** | `2525` nas seis | `−0,04555619` .. `−0,04555623` | `4e-8` |
| **pente no máximo** | `2444` .. `2448` | `+0,11428` .. `+0,12542` | **`0,0111`** |

⇒ **vale / banda = `3,0`.**

⛔⛔ **E a primeira leitura desta não-repetição estava ERRADA, de uma forma que vale para toda
bancada futura desta casa.** Comparando as duas corridas **por índice**, `522` de `~4 200` arestas
«diferiam» com o pente **desligado** — e a contagem de vértices, a contagem de arestas da região e o
`Q` eram idênticos a sete casas. A causa: **o passe de refino emite a mesma geometria com os
vértices por OUTRA ORDEM**. Ordenadas as posições, `10` de `2 525` diferem (`0,4 %`).
⇒ *uma comparação por índice mede a ORDEM, não a malha* — e é por isso que **toda régua deste corpus
é invariante à ordem**. Com o pente ligado a contagem de vértices ela própria muda (`2444`–`2448`),
e aí a variação é real.

⭐ **No regime SEM passe de refino o alvo é determinístico nos dois lados**: o conjunto de arestas é
idêntico entre corridas e `Q` repete à 4.ª casa. É esse o regime em que as leis de mecanismo da
`composicao/` e da `mecanismo/` foram fixadas, e é por isso que elas se podem cobrar **exactamente**.

## §5 — As famílias

⚠️ **Os números desta tabela são CONTADOS da pasta** (`find <pasta> -name '*.gz' | wc -l`) — a 1.ª
redacção escreveu-os de memória e **três** células estavam erradas.

| pasta | ficheiros | o que ela responde |
|---|---|---|
| `entrada/` | **14** | as malhas de entrada, todas nossas |
| `porta/` | **6** | a **pré-condição**: com a topologia dinâmica desarmada a saída é **byte-idêntica** com o pente a `0` e a `1` |
| `escada/` | **29** | a **escada do botão** (7 posições × 3 rotações, `k_*`) **mais** as 8 de passagens repetidas (`n_x1..n_x8`) |
| `rotacao/` | **10** | o **discriminador da direcção**: 4 rotações do traço × 2 lados, mais 2 pares de passagens |
| `verbos/` | **62** | **38** do censo (`v_*`: 19 verbos × 2 lados) **mais 20** de composição (`t_*` o modo de detalhe · `q_res*` a resolução · `s_sim` a simetria) |
| `composicao/` | **26** | os dois verbos **refeitos** (`m_rot`, `m_mask`), o refino (`m_subdiv`/`m_collapse`/`m_manual`), os degenerados de direcção e os de malha |
| `mecanismo/` | **30** | o regime **determinístico** (sem passe de refino): é aqui que se prova o que o pente faz |
| `banda/` | **18** | a repetibilidade, nos dois regimes |
| `lei_unica/` | **26** | ⭐ **a medição que decide se é UMA lei ou duas** (`SPEC` §14): o alvo a fazer as duas metades juntas contra fazê-las em série, com o entrelaçamento a `1×`, `3×`, `4×` e `9×` |

## §6 — Duas coisas que o cabeçalho declara e que a 1.ª redacção não declarava

⭐ **O `sha256` do §2.1 da `SPEC`** é sobre o **corpo** do ficheiro: as linhas que **não** começam
por `#`, juntas por `\n`. (Sem esta receita um revisor teve de a descobrir por varredura de seis
definições.)

⭐ **A contagem de «vértices que o pente moveu»** usa `eps = 1e-7` sobre a maior componente de `Δ`;
e «maior deslocamento» é a maior **norma** de `Δ`. *Metade de uma tabela reproduzível e a outra
metade não, sem nada no texto a separá-las, é pior que uma tabela ausente.*

⛔ **As células `verbos/v_rotate_*` e `verbos/v_mask_*` estão SUPERADAS e trazem-no no cabeçalho**
(`SUPERADA_POR=` + `SUPERADA_PORQUE=`): a primeira mediu um verbo **inerte** (um percurso recto dá
ângulo zero a um verbo de torção ancorado) e a segunda não trazia o controlo de **canal**. As
refeitas são `composicao/m_rot_*` e `composicao/m_mask_*`. *Elas ficam no corpus porque a medição
que as invalidou é ela própria um facto — mas ficam MARCADAS.*

---

## §7 — O censo

⚠️ **Cada célula do censo de verbos tem o CONTROLO dentro**: o número de vértices que o próprio
verbo moveu contra o repouso. Sem ele, um verbo **inerte** lê-se exactamente como um verbo que
ignora o pente — e duas células deste corpus (a torção com um percurso recto, e a esfera sem passe
de refino) foram apanhadas assim, **medidas como inertes e refeitas** com a fixtura que contém o
fenómeno (uma varredura angular; o passe armado).

⚠️ **Família `acumula` (18 células), colhida 2026-09-18** — a metade da tabela-verdade
que faltava: o interruptor de acumulação nos **dois** estados, com o cursor parado e
`N ∈ {1,2,4,8,14,27,40}` carimbos, mais quatro espelhos das células do `mecanismo`.
⚠️ **Elas NÃO entram na população do §3** (são `modo_de_detalhe=MANUAL` e não mudam a
contagem de vértices), logo o vale e a barra ficam intactos. ⭐ A âncora que as liga ao
corpus antigo é uma IDENTIDADE: `acumula/escada_n14_off` é byte-idêntica, no corpo, a
`mecanismo/y_parado_p000`.

⛔ **Risco de leitura nomeado:** em `escada/` o prefixo `n_x*` quer dizer **passagens**;
em `acumula/` o `escada_n*` quer dizer **carimbos**.

---

## §8 — A família `acumula` cresceu para **125** células (2ª colheita, 2026-09-18)

⚠️ **O número é CONTADO da pasta** (`find acumula -name '*.gz' | wc -l`), como manda o §5.
Mesmo binário da 1ª colheita — **5.2.2 LTS, pacote `17:5.2.2-1`, build 2026-09-15** — e a prova
disso é a **regeneração byte-idêntica** de duas células da 1ª leva, uma de cada estado do
interruptor: `escada_n27_on` → `ceb750f09e5b` e `escada_n40_off` → `5e07463ab717`, iguais no
`sha256` do corpo (receita do §6).

### §8.1 — O regime longo (`escada_n60..n800`, +12 células)

O lado **LIGADO** fica em `0.21000004` de `N = 4` a **`N = 800`** — o tecto não se move.
O lado **DESLIGADO** não sobe para sempre: ele **converge para o RAIO**.

| | `N=27` | `40` | `60` | `80` | `120` | `200` | `400` | `800` |
|---|---|---|---|---|---|---|---|---|
| desligado | 0,33280 | 0,33833 | 0,34218 | 0,34410 | 0,34603 | 0,34758 | 0,34874 | **0,34933** |

⭐ Ajustado por Richardson em `d(N) = A − c/N`, os pares `(60, 80)` e `(120, 200)` dão
**`A = 0,3499` os dois** — e `A` é o **raio** (`0,35`). A previsão foi feita ANTES de colher
a cauda e as duas células novas caem em cima dela: `d(400)` previsto `0,34885` / medido
`0,34874`; `d(800)` previsto `0,34942` / medido `0,34933`.

### §8.2 — `CLAY_THUMB`: o interruptor é INERTE neste verbo, e a escada parada não o mede

⛔⛔ **O cursor parado deixa este verbo INERTE** — `maxd = 0,00000000` e `movidos = 0` em
`N ∈ {5, 40, 120, 200}`, nos dois estados (`thumb_n*`, 8 células). ⚠️ **Sem controlo, esses
zeros lêem-se como «o interruptor não faz nada a este verbo» quando o que não aconteceu foi o
VERBO.** Os dois controlos que os separam ficam no corpus:

- `thumbpuro_n*_off` — o **`THUMB`** simples, que já vive em `verbos/`, é **igualmente inerte**
  parado ⇒ é uma propriedade da família **ancorada**, não deste verbo nem do arnês;
- `thumbdeg_n*` — carimbos **idênticos** (âncora na origem, todos os outros pontos no mesmo
  sítio deslocado): `N = 5` e `N = 200` dão a **mesma** saída (`0,00003946`) ⇒ só a **primeira
  transição** age, e as outras `N−2` não fazem nada. *O que este verbo conta são EVENTOS COM
  MOVIMENTO, não carimbos.*

⇒ duas escadas que **contêm** o fenómeno, com nome próprio porque o enquadramento é outro
(e o cabeçalho de cada uma declara-o em `comprimento_do_percurso` e `forma_do_percurso`):

| `N` | `thumbmov` (percurso fixo `0,0100`) | `thumbfix` (passo fixo `0,0001`) |
|---|---|---|
| 5 | 0,00111620 | 0,00111386 |
| 40 | 0,08741962 | 0,08720104 |
| 120 | 0,28273601 | 0,28348203 |
| 200 | 0,28390929 | 0,28851722 |
| 400 | — | 0,29953384 |

⭐ **Ele SATURA**: de `120` para `200` o `thumbmov` move `+0,4 %`. ⚠️ A subida que o `thumbfix`
ainda mostra a `N = 400` é **confundida**: ali o cursor já andou `0,0399` (11 % do raio) e a
pegada passou de `50` para `57` vértices — deixou de ser uma pilha e passou a ser um traço.

⛔ **E o interruptor não muda nada**: `off` e `on` são iguais ao bit em `N ∈ {5, 40}` e diferem
`1,7e-8` em `{120, 200}` — **um ulp de `f32`** naquela magnitude, com `maxd` idêntico a oito
casas. ⚠️ O chão de ruído deste regime está medido ao lado (`thumbmov_n200_off_rep1/rep2` e
`_on_rep1`): **três corridas do MESMO spec dão dois corpos diferentes**, até `2,1e-9`.
*Sem esse chão, «os corpos diferem» não separa o interruptor de uma corrida que não repete.*

### §8.3 — A lei do tecto (`tecto_*`, 60 células)

⚠️ **Cada ponto leva DOIS controlos, e é isso que o torna legível**: o gémeo `_off` (o travão
**mordeu** naquele enquadramento?) e o `_on_n200` (o valor lido é o **planalto** ou ainda subia?).

**Contra o RAIO — a lei é limpa e é PROPORCIONAL** (força `0,5`, `N = 27`, os cinco usáveis):

| raio | 0,15 | 0,20 | 0,25 | 0,35 | 0,50 |
|---|---|---|---|---|---|
| tecto | 0,08963172 | 0,11977170 | 0,14986703 | 0,21000004 | 0,30014169 |
| **tecto / raio** | 0,5975 | 0,5989 | 0,5995 | **0,6000** | 0,6003 |

**Contra a FORÇA — ela NÃO é limpa, e o corpus diz onde** (raio `0,35`, `N = 27`):

| força | 0,125 | 0,25 | 0,375 | 0,50 | 0,625 | 0,69 | 0,70 | 0,71 | **0,725** | 0,75 | 0,775 | 0,80 | 0,875 | 1,00 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| tecto/raio | *0,3718* | 0,5022 | 0,5555 | 0,6000 | 0,6484 | 0,7306 | 0,7418 | **0,7525** | **0,5248** | 0,5617 | 0,5997 | 0,6390 | 0,7645 | *0,9985* |

⛔⛔ **Há um PENHASCO entre `0,710` e `0,725`** — o tecto **cai** `0,7525 → 0,5248` e depois
volta a subir. ⚠️ **Não é amostragem de um vértice**: a soma de `|Δ|` sobre a pegada inteira cai
com ele (`4,742 → 2,870`). ⚠️ **Nem é convergência**: os dois lados do penhasco são planaltos
(`ON` a `N = 27` e a `N = 200` são iguais ao bit nos dois).

⛔⛔ **E o penhasco mora no PAR `(raio, força)`, não na força** (`tecto_r020_f071*`,
`tecto_r020_f0725*`): a raio `0,20` **não há penhasco** entre `0,71` e `0,725` — ali sobe
`0,5018 → 0,5232`. *A raio `0,20` ele já aconteceu antes de `0,71`.*

⛔ **Dois dos pontos NÃO medem o tecto, e ficam marcados em itálico acima:**

- **força `0,125`** — a `N = 27` o travão **não morde** (`ON == OFF` ao bit): o lado sem travão
  ainda não chegou onde o travão o apanharia. O planalto dele existe mais fundo e está colhido:
  `tecto_f0125_on_n200` = `tecto_f0125_on_n800` = **`0,17565575`** ⇒ o ponto é recuperável, mas
  **só com `N ≥ 200`**;
- **força `1,00`** — `ON` e `OFF` diferem `1,1e-4` (`0,011 %`): ali o lado **sem** travão já
  chegou ao próprio limite (o raio), logo o travão fica **em cima** dele e não se pode dizer que
  mordeu. O tecto ali é, à vista, o próprio raio.

⛔ **Risco de leitura nomeado:** em `acumula/` o `escada_n*` e o `thumb*_n*` contam **carimbos**
(ou, no `thumb*`, *eventos com movimento*), enquanto o `n_x*` de `escada/` conta **passagens**.

### §8.4 — ⛔⛔ O QUE O ALVO RECEBE É UM PONTO 3D, E ELE NÃO O RE-PICA

⚠️ **Sem esta secção o corpus lê-se ao contrário.** O harness entrega, por ponto do percurso,
**DUAS** coisas: a posição 3D (`# ponto x y z`) e a projecção 2D dela no ecrã. *O harness não
decide qual delas o alvo usa* — quem decidiu foi a medição.

⭐ **Discriminador, com o plano em `z = 0` EXACTAMENTE e vista de topo ortográfica** — os três
pontos projectam-se no **mesmo pixel**, `(655,5 · 440,0)`, e isso está no log do próprio harness:

| célula | ponto alimentado | saída |
|---|---|---|
| `altura_acima` | `(0, 0, +2)` | `0,00000000` · **0** vértices |
| `altura_no_plano` | `(0, 0, 0)` | `0,31712657` · 49 vértices |
| `altura_abaixo` | `(0, 0, −2)` | `0,00000000` · **0** vértices |

⇒ **o alvo lê a posição 3D e NÃO re-lança raio do rato.** Com um raio do rato os três dariam a
mesma saída, porque o pixel é o mesmo.

⇒ **e o centro NÃO se move entre carimbos:** ele é o ponto alimentado, verbatim, nas `N` vezes.
A prova é a convergência da §8.1 — o barro sobe e **foge** de um centro que fica parado, logo o
vértice pára exactamente a `R` do centro, que é onde a queda vale zero. *Se o centro seguisse a
superfície, o deslocamento crescia sem limite.*

⭐⭐ **A lei por carimbo sai disto e reproduz o alvo a `0,00 %` nos TREZE `N` (1..800):**

```
h ← h + forca² · R · queda( |posição VIVA − centro FIXO| / R )
queda(t) = 1 − (3t² − 2t³)        // 1 no CENTRO, 0 no RAIO
```

⛔⛔ **A distância é medida da posição VIVA, e é isso que decide tudo.** Medida da posição de
REPOUSO a mesma lei diverge — linear, `69,89` a `N = 800` contra `0,3493` (`+19 908 %`).

⚠️⚠️ **LIMITE DO CORPUS, nomeado:** como o harness entrega a posição, **a política de re-pique do
próprio alvo é CONTORNADA e este corpus não a mede.** Ele mede a lei do pincel com o centro
PREGADO. Quem comparar um produto que re-pica ao vivo contra estas células está a variar **duas**
coisas ao mesmo tempo — a polaridade da curva e a posição de onde a distância é medida — e as duas
leituras dão conclusões opostas.

⭐ E é isto que explica a §5 das células de ESFERA (`x_esf_*`, `y_esf*`): o percurso delas corre
**dentro** da esfera (raio `0,8`, centro na origem, pontos de `x = −0,45` a `+0,45` em `z = 0`), e
o vértice mais próximo fica a `0,35`–`0,80` — **nunca dentro** do raio do pincel. Por isso o alvo
não move um vértice: *o ponto cai no MIOLO da peça*. Um raio de cima teria acertado em `z = +0,8`.

---

## §9 — ⛔⛔⛔ O PENTE NÃO ALINHA PELO DESLOCAMENTO: ALINHA PELO PASSE DE REFINO

⚠️⚠️ **Esta secção CORRIGE a leitura do §2.** A tabela das quatro rotações lá em cima está certa
como medição e a conclusão tirada dela — *«o alvo segue o traço»* — atribui o efeito ao sítio
errado. As células do §2 são **todas `CONSTANT`**, logo o passe de refino corre em todas, e
nenhuma delas isola o deslocamento.

### §9.1 — O controlo que faltava (`rotacao/m_*` e `rotacao/c_*`, 16 células, 2026-09-18)

Tudo igual — força `0,2` · 1 passagem · a mesma malha de entrada · o mesmo traço — e **só** muda o
`modo_de_detalhe`:

| rotação do traço | `m_*` **MANUAL** (sem refino) | `c_*` **CONSTANT** (com refino) |
|---|---|---|
| 0° | **+0,22920** | +0,16727 |
| 22,5° | **−0,00276** | +0,16217 |
| 45° | **−0,00355** | +0,12741 |
| 67,5° | **+0,01295** | +0,15458 |

⇒ **com refino o alvo alinha nas QUATRO rotações, uniformemente; sem refino ele não alinha em
NENHUMA** — excepto a `0°`, e ali o traço **coincide com a grelha da malha de entrada**, que é
regular e alinhada com `x`. *A `0°` «alinhar ao traço» e «regularizar a malha» são a mesma coisa, e
foi por isso que a `0°` passou por alinhamento durante todo este tempo.*

⛔ **Consequência para quem implementa:** uma lei que só move vértices **não pode** reproduzir o
alinhamento. Ele nasce de o passe de partir/fundir correr dentro de uma pegada que **ANDA** ao
longo do traço.

### §9.2 — A lei do deslocamento, ajustada à saída

`96,3 %` do `Δ` do pente é **tangencial**. O termo dominante é uma **relaxação ISOTRÓPICA**:

```
por DAB, para cada vértice na pegada:
    m ← centróide(anel) − p                  # vizinhos da malha VIVA
    p ← p + queda(dist_ao_centro_do_dab / R) · tangencial(m)
```

⭐ **Um passo COMPLETO por dab** (`α = 1,0`; a escala óptima sai `0,957`). O perfil radial medido
não é a queda — é a **SATURAÇÃO** dela sobre os dabs que tocam cada vértice (`k ≈ 0,85` no planalto,
`0,019` na borda).

| | cos ponderado | resíduo |
|---|---|---|
| encaixe duro em 4 eixos (o que se shipa) | `0,583` | — |
| só regularizar o raio médio | `0,886` | — |
| **relaxação ao centróide, 1 passo por dab, com queda** | **`0,9830`** | **`0,2272`** |

⛔ **Qualquer peso direccional PIORA** (`cos2θ` contra o traço, `k` de `−0,6` a `+0,6`): o óptimo
do grid de 84 células é **`k = 0,00`**. *Não há direcção na lei por vértice para encontrar — foi
por isso que as cinco tentativas falharam.*

### §9.3 — ⏳ O que este ajuste NÃO explica

⚠️ Fora do eixo o alvo move **~30 % MAIS** (`Σ|Δ|` de `2,00` a `0°` para `2,52`–`2,64` nas outras
três) e em direcções que a relaxação isotrópica não prevê:

| rotação | cos ponderado | resíduo |
|---|---|---|
| 0° | `0,983` | `0,227` |
| 22,5° | `0,802` | `0,640` |
| 45° | `0,711` | `0,682` |
| 67,5° | `0,811` | `0,621` |

⛔ **E esse excesso NÃO é alinhamento** — o `ΔQ` dele é zero (§9.1). É um termo real, por medir.
*Uma lei isotrópica não tem porque ajustar-se pior quando o traço roda; que se ajuste, é a prova de
que falta um termo.*
