# SPEC — o pincel de POSE (`Pose`) do modo de escultura

> **Este documento descreve COMPORTAMENTO; não contém expressão do alvo.**

| campo | valor |
|---|---|
| **Alvo** | Blender 5.2 LTS — pincel **Pose** do modo de escultura (nome **público** do valor do enum de tipo de pincel). Fonte lido: tag `v5.2.0`, commit `fbe6228777e7d9afefcd61a413844e790ae75db7`. Oráculo corrido: `/usr/bin/blender`, **5.2.1 LTS** (build 2026-09-01) |
| **Licença** | **GPL-2.0-or-later** (`COPYING` da raiz, lido 2026-09-13) · **Degrau: T2** |
| **Ledger** | [`LEDGER_blender-pose.md`](LEDGER_blender-pose.md), aberto 2026-09-13 (⛔ o Implementador não o abre) |
| **Patente (§8.1)** | buscado 2026-09-13 — termos e tabela no ledger. ⭐ **Nenhuma patente viva alcança o método.** A mais próxima (máscara topológica + linha de acção do *Transpose*, **US 9 460 556 B2**, Pixologic) está **EXPIRADA** por falta de anuidade ⇒ literatura livre. Duas cercas nomeadas: ⛔ nunca implementar «derreter de volta a uma pose de repouso por comparação com uma pose atractora» (US 8 704 828 B1, Pixar, viva até 2031) · ⛔ nunca implementar um modo elástico por Kelvinlet dentro deste pincel (US 10 586 401 B2, Pixar, viva até 2038) |
| **Filtragem §4.3** | executada 2026-09-13 (v2) · **Sweep:** verde em 2026-09-13, **com controlo positivo** — a vassoura cobre agora as **duas línguas** e acusa `5` achados na v1 desta espec, `0` nesta |
| **Auditoria §4.2 (R-pré)** | ⛔ a **v1 NÃO foi atestada** (6 achados, 4 substanciais); esta é a **v2**, reescrita pela regra do arquivo fechado. ⏳ Aguarda **R-pré novo** — condição para abrir a janela que implementa |
| **Mapa de leitura da literatura** | não há paper. A literatura livre é: (a) as **issues públicas** do rastreador do alvo, citadas por número no §15 — são a fonte da sabedoria dos autores (§4.1.12); (b) a documentação de utilizador pública do alvo (factos, nunca o *wording*); (c) a patente expirada acima. ⛔ **A PULAR:** qualquer *code search*, espelho de fonte, ou o repositório do alvo |
| **Denylist de URLs** | `projects.blender.org/blender/blender` (e `/src/`, `/raw/`, `/commit/`) · `github.com/blender/blender` e espelhos · `developer.blender.org/D*` (revisões diferenciais = diffs) · qualquer *grep.app* / *searchcode* / *sourcegraph* sobre o alvo. ⭐ **PERMITIDO:** `projects.blender.org/blender/blender/issues/<n>` (texto de utilizadores e triagem — é o que o §15 cita) e `docs.blender.org` (manual, para FACTOS) |

---

## §0 — O gesto, numa página

O artista aponta o cursor a um ponto da malha e **arrasta**. O pincel encontra sozinho, **pela
forma e pela ligação da malha** (não por um esqueleto), um **pivô** — como se houvesse uma
articulação enterrada ali — e uma **cadeia de segmentos** encostada a ele, e faz a região à volta
**rodar** em torno desse pivô acompanhando o arrasto. Com mais de um segmento, a cadeia dobra como
um braço: é uma cadeia de cinemática inversa resolvida a cada evento do ponteiro.

Três deformações, e cada uma tem um par: o modificador de inversão (Ctrl, ou a ponta invertida da
caneta) **troca de deformação** em vez de trocar o sinal da força.

| modo | gesto normal | com o modificador de inversão |
|---|---|---|
| **girar/torcer** (omissão) | **roda** a cadeia para seguir o arrasto | **torce** cada segmento em torno do próprio eixo, pelo movimento horizontal do ponteiro |
| **escalar/transladar** | **escala** ao longo do segmento | **translada** rigidamente |
| **espremer/esticar** | escala no eixo do segmento, com volume compensado nos outros dois | ⛔ **nada** — não há ramo para a inversão neste modo |

⚠️ **Os nomes acima são os NOSSOS** (e os que as fixtures usam no campo `modo`). O alvo expõe os
mesmos três na ordem desta tabela; a correspondência é posicional e está no cabeçalho de cada
fixture.

⚠️ **O que o pincel faz NÃO é função da posição do cursor a cada instante** — é função do
**deslocamento acumulado** desde o ponto onde o botão foi premido. O ponto de aplicação fica
**preso** (âncora) no sítio do primeiro evento; a cadeia é construída **uma vez**, no primeiro
evento, e **reutilizada** até o traço acabar.

---

## §1 — O que o pincel lê

### §1.1 — Os sete controlos próprios

⚠️ **Nomeados em vocabulário do DOMÍNIO** (§4.2): estes são os nomes com que a espec e as fixtures
falam deles, e são os nomes que a nossa implementação deve usar. As faixas e as omissões abaixo são
**factos observados** da interface pública do alvo, não escolhas nossas.

| controlo (nosso nome) | tipo | faixa | omissão | o que faz |
|---|---|---|---|---|
| **modo** | enum | `girar_torcer` · `escalar_transladar` · `espremer_esticar` | `girar_torcer` | qual das três deformações (§0) |
| **origem** | enum | `topologia` · dois modos por conjuntos de faces | `topologia` | como o pivô é achado. ⛔ **só `topologia` é especificado aqui** (exclusão C da missão) |
| **segmentos** | int | `1..20` | `1` | quantos segmentos a cadeia tem |
| **desvio da origem** | float | `0..2` | `0` | afasta o pivô do cursor, em múltiplos do raio |
| **suavizações do peso** | int | `0..100` | `4` | quantas suavizações os pesos levam (§4) |
| **ancorado** | bool | — | ver §1.4 | prende a extremidade distante da cadeia (§5.1) |
| **trava de rotação** | bool | — | ver §1.4 | no modo de escala, não roda antes de escalar (§5.4) |

### §1.2 — Controlos partilhados que este pincel de facto lê

| controlo | efeito aqui |
|---|---|
| **raio** | define a região do 1.º segmento **e** o comprimento de cada segmento seguinte (§3) |
| **força** | multiplica o deslocamento do arrasto (§5). ⚠️ entra **linearmente**, não ao quadrado |
| **curva de atenuação** | ⚠️ **só o modo de torção a usa**, e o argumento dela é o **índice do segmento**, não uma distância (§5.2) |
| **máscara** de escultura | multiplica o deslocamento final por `1 − máscara` (§9) |
| **auto-máscaras** | multiplicam o mesmo factor (§9) |
| **vértices escondidos** | factor `0`, e são saltados em toda travessia (§9) |
| **simetria de espelho X/Y/Z** | §8 |
| **«só conectado»** / **distância máxima entre peças** | §2.4 |
| **travas de eixo / recorte do modificador de espelho** | aplicados ao deslocamento final (§7.4) |
| **alvo da deformação** | **a geometria** (o pincel move os vértices) ou a simulação de tecido. Só o primeiro é especificado aqui |

### §1.3 — Controlos que este pincel **NÃO** lê (medido — a ausência é o facto)

- ⛔ **Pressão da caneta não altera a força.** A força efectiva é `força_do_pincel × pluma_de_simetria`,
  sem factor de pressão, sem o factor de direcção do pincel e sem o factor de sobreposição que os
  pincéis de deslocamento usam. A **pluma de simetria** vale `1` a menos que a opção *Feather* da
  simetria esteja ligada.
  *Prova:* `figura_girar_dedo_pressao03` (pressão `0,3` por evento) produz uma saída
  **idêntica ao BIT** à de `figura_girar_dedo` (`max|dif| = 0,0` sobre 4 930 vértices); o modelo de
  referência, que ignora pressão, concorda com as duas a `1,7e-7`.
- ⛔ **O modificador de inversão não nega a força** — escolhe a outra deformação (§0).
- ⛔ **A direcção «Add/Subtract» do pincel não tem efeito.**
- ⛔ **A dureza (*hardness*) não tem efeito** — não há atenuação radial: um vértice é governado pelos
  **pesos dos segmentos** (§2, §4), não pela distância ao cursor.
- ⛔ **A forma de atenuação (esfera/tubo)** só alcança este pincel pela projecção do deslocamento do
  arrasto no plano da vista, que é genérica do traço — não muda a região nem os pesos.

### §1.4 — ⚠️ As duas omissões que dependem de quem pergunta

**Ancorado** e **trava de rotação** são `false` na estrutura de dados **crua**, mas o
pincel que o artista de facto activa vem de uma biblioteca de pincéis distribuída com o alvo, e nela
**ancorado chega LIGADO** (observado no dump de configuração de todas as 69 fixtures: ancorado
`True`, trava de rotação `False`).
⇒ **Para nós a decisão é de produto:** o comportamento que o artista conhece é o **ancorado**.
Recomendação: nascer ancorado, e a fixture `figura_girar_dedo_solto` é o lado desligado.

---

## §2 — Fase A: achar o pivô e a região do primeiro segmento

Entrada: a posição do cursor na superfície (`C`), o raio (`R`), o factor de desvio (`d`).
Saída: uma **origem** `O₀` (o pivô) e um **peso** por vértice, em `[0,1]`.

### §2.1 — A semente

⚠️⚠️ **São DUAS grandezas distintas, e a 1.ª redacção desta espec descrevia-as como uma.** Elas
coincidem em todas as fixturas publicadas, mas **não por construção**:

| | o que é | o que faz |
|---|---|---|
| **o vértice ELEITO** | o vértice que a amostragem do ponteiro elege sobre a superfície sob o cursor | **semeia a varredura** (§2.2), junto com os espelhos dele |
| **o mais-próximo GLOBAL** | o vértice mais próximo de `C` por uma busca **sem limite de distância**, sobre as posições do **início do traço**, saltando escondidos | recebe **peso `1`** antes de a varredura começar |

A semente da varredura é, então: **o vértice eleito**, mais — para cada combinação de eixos de
simetria activa — o vértice mais próximo da imagem espelhada dele, **e só se essa distância for
menor que `R`**. A lista é **ordenada por índice crescente** antes de começar.
⚠️ A ordenação é exigência de **determinismo**: a ordem de visita decide qual vértice fica
registado como «o mais afastado» (§2.3) em caso de empate.

⭐ **O peso posto pelo mais-próximo global é OBSERVACIONALMENTE INERTE neste corpus, e isso é
medido:** em **`69` de `69`** fixturas a varredura alcança esse vértice — e a varredura põe peso `1`
em tudo o que visita —, logo apagar esse pré-peso do modelo de referência muda a saída em
**`0,000e+00`** nas quatro fixturas testadas. ⇒ **as duas grandezas só se separam quando o
mais-próximo global NÃO é alcançável a partir do eleito** — outra peça da malha, ou uma peça mais
perto no espaço do que a que está sob o cursor, com a ligação entre peças (§2.4) desligada.
⏳ **Nomeado e por fechar:** o corpus não tem essa fixtura. Ela é construtível (cursor sobre uma
peça, outra peça mais perto em 3D, «só conectado» ligado) e é o que falta para decidir se o
pré-peso é uma ilha de influência real ou lixo inerte. ⛔ **Não é** o que explica o resíduo aberto
da §12.3 — isso foi medido e está lá.

### §2.2 — A travessia

Uma **varredura em largura** (fila FIFO) sobre o grafo de adjacência de vértices. Um vértice é
vizinho de outro quando partilham uma aresta de alguma face visível; a lista de vizinhos de um
vértice constrói-se percorrendo as faces incidentes e tomando, em cada face, os **dois** vértices
adjacentes a ele no anel da face, **sem repetir** (isso suporta topologia não-manifold).

A travessia parte a malha em três conjuntos, e é deles que tudo o resto se deriva. Seja
`dentro(v)` o predicado *«`v` está **estritamente** a menos de `R` de `C` ou de alguma imagem
espelhada válida de `C`»* (§12.1 — a comparação é estrita e em `f32`):

| conjunto | definição | o que produz |
|---|---|---|
| **visitados** | os vértices alcançáveis a partir da semente por uma cadeia de vizinhos em que **todos os elos anteriores** satisfazem `dentro` | cada um recebe **peso `1`** |
| **interior** | os visitados que satisfazem `dentro` | são os únicos por onde a travessia continua |
| **franja** | os visitados que **não** satisfazem `dentro` — o primeiro anel para lá do raio | os que passam no **teste de lado** (§2.5) entram na **média que dá o pivô** (§2.3) |

⚠️ Cada vértice entra nos conjuntos **uma vez só**; a ordem de visita é FIFO e importa apenas para
o desempate do §2.3.

### §2.3 — A origem

- Se a média do passo 4 recebeu pelo menos um vértice: **`O₀` = essa média**.
- Caso contrário: `O₀` = **o vértice visitado mais afastado de `C`** (mantido durante a travessia
  pela regra «substitui o corrente sempre que o novo estiver mais longe de `C`»), ou o próprio `C`
  se nada foi visitado.

⚠️ **`O₀` é a média da FRANJA, não do interior.** É por isso que o pivô cai *do lado do corpo*
quando o cursor está numa extremidade: a franja de um dedo é quase toda do lado da mão.

### §2.4 — Peças desligadas («só conectado»)

Com a opção **desligada**, o grafo de adjacência recebe **ligações artificiais** entre peças
desligadas da malha, e a travessia atravessa-as:

- calculam-se as **ilhas** de topologia (componentes ligados);
- para cada vértice ainda **sem par**, procura-se o vértice mais próximo que esteja **noutra ilha**,
  **ainda sem par**, e a menos da *distância máxima entre peças*; achado, os dois ficam
  **emparelhados um com o outro** (a ligação é simétrica e cada vértice tem **no máximo uma**);
- o par entra na lista de vizinhos na travessia e no crescimento (§3), **mas não na suavização**
  dos pesos (§4).

⚠️ **O emparelhamento é sequencial e gananciosa por ordem de índice de vértice** — não é o
emparelhamento óptimo, e a escolha de quem fica com quem depende da ordem. Os autores registam o
sintoma: com muitas peças, a deformação sai com picos e inconsistente
([issue #133739](https://projects.blender.org/blender/blender/issues/133739)).
⚠️ **Custo: esta construção é quadrática no número de vértices.** A proveniência é **pública e
observável** — o relato de [#128329](https://projects.blender.org/blender/blender/issues/128329) é
que, com esta opção desligada, **cada** movimento de câmara volta a pagar a construção e o editor
engasga a cada zoom/pan; o de [#127259](https://projects.blender.org/blender/blender/issues/127259)
é o mesmo custo numa malha densa. §13.

### §2.5 — O teste de lado (usado em §2.2, §3 e em mais lado nenhum)

Dado um ponto `p` e um pivô `q`, para **cada eixo de simetria activo** `i`:

- se `q[i] == 0` e `p[i] > 0` ⇒ **reprova**;
- se `p[i] · q[i] < 0` ⇒ **reprova**.

Passa se nenhum eixo reprovar. Com simetria desligada **passa sempre**.
⚠️ O primeiro ramo é o que faz o pivô exactamente **sobre** o plano de simetria escolher
determinadamente o lado negativo.

### §2.6 — O desvio da origem

Se `d ≠ 0`:

1. `O₀ ← O₀ + normalizar(O₀ − C) · R · d`;
2. **crescer os pesos uma vez** com a regra de **aproximação** (§3.2), tomando `O₀` como alvo — ou
   seja, a região do 1.º segmento engorda até a frente dela deixar de se aproximar do pivô deslocado.

⚠️ Se `O₀ == C` a normalização é de um vector nulo; o alvo devolve zero nesse caso (§11.1).

---

## §3 — Fase B: os segmentos seguintes

Só corre se a cadeia tiver mais de um segmento.

⚠️ **Requisito: nos modos de escala e de espremer/esticar a cadeia tem exactamente UM segmento**,
seja qual for o valor do controlo. *Observado:* `figura_escalar_braco_ik3` e
`figura_esticar_braco_ik3` pedem `3` segmentos no cabeçalho e produzem a mesma cadeia de um
segmento que as fixturas de `1`.

O comprimento-alvo de cada segmento é `L = R · (1 + d)`.

### §3.1 — Uma varredura de crescimento

Sobre **todos** os vértices da malha, com os pesos da varredura anterior congelados (é um passo de
Jacobi):

- `m ← max(peso_anterior[u])` sobre os vizinhos `u` de `v` (incluindo as ligações artificiais, §2.4);
- se `m > peso_anterior[v]`: `peso[v] ← m`, e `v` conta como **recém-alcançado**;
- todo recém-alcançado que passe no **teste de lado** (§2.5) contra o alvo corrente entra numa
  **média de posições** `A`.

### §3.2 — Quando parar

Duas regras, conforme o uso:

| uso | continua enquanto | ao parar |
|---|---|---|
| **origem do próximo segmento** | `‖A − alvo‖ < L` | a origem do segmento novo é `A` **da última varredura**, e os pesos **voltam** ao estado da varredura anterior |
| **compensação do desvio** (§2.6) | `‖A − pivô‖` **continuar a diminuir** | os pesos **voltam** ao estado da varredura anterior |

Se uma varredura não alcançar ninguém (`A` vazia), pára; nesse caso a origem devolvida é o próprio
alvo corrente.

⚠️ **O retrocesso dos pesos é load-bearing:** a varredura que dispara a paragem é descartada, e é
a anterior que fica. Sem isso cada segmento ficaria um anel mais gordo do que deve.

### §3.3 — Os pesos de cada segmento

Depois de crescer para o segmento `i`, o peso desse segmento é a **diferença**
`peso_agora − peso_depois_do_segmento_anterior`. ⇒ os segmentos repartem a malha em **anéis
disjuntos**, e a soma dos pesos de todos os segmentos num vértice é `0` ou `1` **antes** da
suavização (§4).

### §3.4 — Cabeças, origens e comprimentos

Depois de todas as origens estarem achadas:

- a **cabeça** do segmento `0` é `C` (o cursor); a cabeça do segmento `i` é a **origem** do
  segmento `i−1`;
- `comprimento_i = ‖cabeça_i − origem_i‖`;
- guardam-se `origem_inicial_i` e `cabeça_inicial_i` (cópias, usadas como referência o traço todo);
- a escala inicial de cada segmento é `(1,1,1)`.

---

## §4 — Fase C: suavizar os pesos

Cada segmento leva, **independentemente**, `N` = **suavizações do peso** iterações, e cada iteração
substitui o peso de cada vértice pela **média simples dos pesos dos vizinhos** — ⚠️ **o próprio
vértice não entra na média**, e as **ligações artificiais de §2.4 não entram** na vizinhança aqui.

⚠️ **Um vértice sem vizinhos** (solto) recebe, nesta rotina, uma média de um conjunto vazio.
Não há guarda neste caminho (existe uma variante com guarda, que este uso não chama). ⇒ malha com
vértices soltos é caso a evitar; ver §11.4.

⚠️⚠️ **Determinismo — o achado que muda a nossa implementação:** no alvo esta suavização corre
**por partição espacial, em paralelo, escrevendo no mesmo arranjo** que está a ler. Dentro de uma
partição o passo é Jacobi limpo; **através da fronteira entre partições, um vizinho pode já ter sido
reescrito nesta mesma iteração** — o resultado depende da partição e do escalonamento das threads.
⇒ **A nossa implementação faz Jacobi limpo (determinístico)**, e a paridade com o oráculo é exacta
só quando a região cabe numa partição. Nas 69 fixtures publicadas ela cabe (malhas de 1 298 e 4 930
vértices), e é por isso que a paridade medida fecha a `~1e-7` (§12).

---

## §5 — Fase D: resolver a cadeia a cada evento

O **deslocamento do arrasto** `G` é a diferença entre o ponto do ponteiro projectado no plano de
profundidade constante que passa pelo ponto de aplicação ancorado, e esse mesmo ponto no primeiro
evento. ⚠️ No alvo ele é **acumulado** evento a evento (soma telescópica); matematicamente é a
diferença directa, numericamente difere por arredondamento de `f32` acumulado.
⚠️ **Não confundir esta acumulação com a memória do §5.1-bis:** esta é inofensiva (telescópica); a
outra muda a resposta.
Seja `s` a força efectiva (§1.3) e `T = C + G·s` o **alvo**.

### §5.1 — Rotação (cadeia de cinemática inversa)

Os segmentos resolvem-se **do mais próximo do cursor para o mais distante**, cada um contra um
**alvo** `T` que começa em `C + G·s` e que **cada segmento reescreve para o seguinte**.

Para o segmento `i`, sejam `O⁻` a origem em que o **evento anterior** o deixou (§5.1-bis) e
`comprimento_i` o comprimento fixado no início do traço (§3.4). O evento tem de deixar o segmento
nesta configuração:

| grandeza | valor exigido | o que ela é |
|---|---|---|
| **direcção** `d` | `normalizar(T − O⁻)` | para onde o segmento aponta agora: do sítio onde ficou, para o alvo |
| **rotação** `rot_i` | a rotação que leva `normalizar(cabeça_inicial_i − origem_inicial_i)` em `d` | ⚠️ medida contra o estado **INICIAL**, não contra `O⁻` — é isto que faz a deformação ser sempre a pose acumulada desde o princípio do traço, e não um incremento |
| **origem** `origem_i` | `T − d · comprimento_i` | ⭐ a origem é posta de modo que **um segmento do comprimento original, apontado ao longo de `d`, tenha a ponta distante exactamente em `T`** |
| **cabeça** `cabeça_i` | `O⁻ + d · comprimento_i` | o mesmo avanço, medido a partir da origem **ANTIGA** |
| **alvo do seguinte** | `origem_i` | a origem deste é o alvo do próximo |

⚠️⚠️ **A cabeça e a origem guardadas NÃO ficam à distância `comprimento_i` uma da outra** — elas
são medidas a partir de origens diferentes (`O⁻` e `T`), e a separação delas vale
`|2·comprimento_i − ‖T − O⁻‖|`. Isto é **invisível em todos os modos menos um**: a cabeça só é lida
pelo referencial do espremer/esticar (§6), e é lá — e só lá — que a inconsistência chega a pixel.
*Uma implementação que "corrigisse" a cabeça para `origem_i + d·comprimento_i` diverge nesse modo e
em mais nenhum.*

**Epílogo da âncora.** Com a âncora ligada, a cadeia **inteira** é a seguir deslocada pelo vector
que devolve a origem do **último** segmento ao sítio onde ela nasceu
(`origem_inicial_do_último − origem_do_último`), somado a `origem` **e** a `cabeça` de todos os
segmentos. ⇒ a extremidade distante fica presa e o gesto roda a cadeia em torno dela.

⭐ **Consequência com UM segmento e âncora ligada:** a origem volta sempre ao lugar, logo a
translação da matriz (§7.1) é nula e a deformação é **rotação pura em torno do pivô**.

### §5.1-bis — ⚠️⚠️ O solver é INCREMENTAL: a cadeia carrega estado entre eventos

Repare na tabela acima: a **direcção** `d` sai de `O⁻` — a origem que o evento **anterior** deixou —
e não de `origem_inicial_i`. Só a **rotação** é medida contra o estado inicial. ⇒ **este não é um
solver fechado avaliado no deslocamento final: é uma relaxação que dá UM passo por evento do
ponteiro**, partindo da configuração em que o evento anterior a deixou.

**Consequências, as duas medidas:**

1. ⛔ **A pose final DEPENDE de quantos eventos o ponteiro entregou**, para cadeias de mais de um
   segmento. O mesmo arrasto, amostrado em `4`, `12` e `36` eventos:

   | fixture | `12` vs `4` eventos | `12` vs `36` eventos |
   |---|---|---|
   | `figura_girar_braco_ik1` (1 segmento, ancorado) | **`0,0` — idêntico ao bit** | **`0,0` — idêntico ao bit** |
   | `figura_girar_braco_ik3` (3 segmentos, ancorado) | `5,321e-2` | `1,087e-2` |
   | `figura_girar_braco_ik3_solto` (3 segmentos, solto) | `3,780e-2` | `1,258e-2` |

2. ⭐ **A âncora é que apaga o estado — mas só do último segmento.** Com **um** segmento ancorado, o
   deslocamento da âncora repõe `origem` exactamente em `origem_inicial` a cada evento ⇒ o solver
   fica **sem memória** e a saída é independente da taxa de eventos (a primeira linha da tabela, ao
   bit). Com mais segmentos, só o último é reposto; os outros acumulam a deriva.

⚠️ **Os outros modos NÃO têm esta memória:** torção (§5.2), translação (§5.3) e espremer/esticar
(§5.5) escrevem sempre a partir do estado **inicial**, logo são funções fechadas do deslocamento do
arrasto. O modo de escala (§5.4) herda a memória **só quando a trava de rotação está desligada**,
porque aí ele chama esta rotina.

⚠️ **Para nós isto é uma DECISÃO, não um detalhe a copiar cegamente:** a dependência da taxa de
eventos significa que o mesmo gesto dá poses diferentes conforme a carga da máquina. Reproduzi-la é
o que dá paridade com o oráculo (o nosso modelo de referência reproduz o `5,321e-2` **exactamente**);
não a reproduzir dá um pincel mais previsível e **incompatível** com as fixtures de mais de um
segmento. ⇒ recomendação: implementar o incremental (é o comportamento que o artista conhece) e
**gatear a taxa de eventos** nas fixtures, para que a diferença nunca seja acidental.

### §5.2 — Torção (a inversão do modo de rotação)

`ângulo = (x_do_ponteiro_no_primeiro_evento − x_do_ponteiro_agora) · s · k` **radianos**, com

> ⭐ **`k = 0,020000 rad por pixel`, MEDIDO da saída do oráculo** (2026-09-13): num segmento de peso
> `1` a deformação é uma rotação pura em torno do eixo do segmento, e o ângulo dela lê-se da malha
> deformada. Sobre `46` vértices de peso `1`, `figura_torcer_braco_ik1` dá `+0,910416 rad` para um
> percurso de `45,52 px` a força `1,0` ⇒ `0,020000`; `figura_torcer_braco_ik3_forca05` dá
> `+0,455208 rad` no mesmo percurso a força `0,5` ⇒ `0,020000`. Dispersão entre vértices: `1,8e-6`.
> *O par de fixturas mede a constante **e** confirma que a força entra linearmente.*

Cada segmento `i` de `n` roda em torno do **próprio eixo inicial**
(`normalizar(cabeça_inicial_i − origem_inicial_i)`) por `ângulo · curva(i, n)`, onde `curva` é a
curva de atenuação do pincel avaliada com `p = 1 − i/n` (para `i ≥ n` daria `0`, o que não acontece):

⭐ **São DEZ, e os nomes são os da casa** — o vocabulário já está fixado em
[`crates/ph2d-sculpt3d/src/falloff.rs`](../../../crates/ph2d-sculpt3d/src/falloff.rs), onde as
mesmas leis já vivem (o `Falloff::ALL` da casa tem `12`: estas nove mais a do SculptGL e as duas
próprias):

| curva (nome da casa) | `curva(p)` |
|---|---|
| **Smooth** (omissão) | `3p² − 2p³` |
| **Sharp** | `p²` |
| **Sharper** | `p⁴` |
| **Smoother** | `p³(6p² − 15p + 10)` |
| **Linear** | `p` |
| **Constant** | `1` |
| **Root** | `√p` |
| **Sphere** | `√(2p − p²)` |
| **InvSquare** | `p(2 − p)` |
| **personalizada** | a curva autorada, avaliada em `1 − p` |

⚠️ **A `InvSquare` faltava à 1.ª redacção desta espec** — ela é escolhível pelo artista e este modo
lê-a como qualquer outra. ⚠️ E a `Sphere` escreve-se `√(2p − p²)`, que é a **mesma** curva que a
casa guarda como `√(1 − t²)` com `t = 1 − p`: *duas formas da mesma parábola* (a nota do
`falloff.rs` documenta a álgebra, e a casa já pagou o erro de as ler como curvas diferentes).

⚠️ **A rotação guardada é a INVERSA dessa rotação** (equivalentemente, uma rotação de `−ângulo`
em torno do mesmo eixo). As posições de cabeça e origem **não** se mexem neste modo.
⚠️ Este é o **único** sítio em que o pincel lê **pixels** — e portanto o único cuja saída depende da
resolução do ecrã e do zoom. Para reproduzir uma fixture é preciso o factor pixels-por-unidade, que
vai no cabeçalho de cada uma.

### §5.3 — Translação (a inversão do modo de escala)

`cabeça_i ← cabeça_inicial_i + G·s`, `origem_i ← origem_inicial_i + G·s`, `rot_i ← identidade`,
para todo `i`. É um deslocamento rígido de toda a cadeia; os pesos continuam a decidir quanto cada
vértice acompanha.

### §5.4 — Escala

1. Se a **trava de rotação** estiver **desligada**, resolver primeiro a cadeia como em §5.1 (com a
   âncora conforme o controlo) — a rotação entra como parte do gesto de escala;
2. seja `plano` o plano que passa pela **cabeça inicial do segmento 0** com normal
   `normalizar(cabeça_inicial_0 − origem_inicial_0)`, e `δ` a **distância com sinal** de `T` a esse
   plano;
3. `escala = comprimento_0 / (comprimento_0 − δ)`, aplicada **igual nos três eixos** a **todos** os
   segmentos.

⚠️ **`escala` tem um POLO em `δ = comprimento_0`** e muda de sinal ao atravessá-lo. Não há
saturação neste modo. §11.2.

### §5.5 — Espremer / esticar

`escala_z` é o mesmo quociente de §5.4. Depois:

- se `|escala_z| < 1e-5` ⇒ **escala = (0,0,0)** (a guarda que existe, e a razão de ela existir está
  em [#130465](https://projects.blender.org/blender/blender/issues/130465): sem ela a malha ia a
  `NaN` e o desfazer não a recuperava);
- caso contrário `escala_x = escala_y = sinal(escala_z) · √(1/|escala_z|)` — o que **conserva o
  volume** do factor de escala (`x·y·z = sinal·z/|z| · … = ±1` em módulo).

Neste modo a rotação guardada de cada segmento **não é usada** como rotação (ver §7.2).

---

## §6 — Fase E: da cadeia para as matrizes

Cada vértice é governado, por segmento, por **um mapa afim**. Há um mapa por segmento e por
**octante de espelho** — as 8 combinações de sinal dos três eixos —, e o octante de um vértice é
lido dos **sinais das coordenadas do próprio vértice** (coordenada `< 0` liga o bit do eixo).

**A lei, em álgebra.** Para o segmento `i` no octante `a`, com `Õ` e `Õ₀` a origem corrente e a
inicial já espelhadas para `a`, `R` a rotação do segmento (também espelhada), `S = diag(escala)` e
`F` o referencial local:

```
X(p)  =  T(Õ) · F · [ R · S · T(Õ − Õ₀) ] · F⁻¹ · T(−Õ) · p
```

— isto é: leva-se `p` ao referencial do pivô, aplica-se lá dentro *rotação, escala e a translação
que a origem sofreu*, e devolve-se. O deslocamento do vértice é `X(p₀) − p₀`, com `p₀` a posição do
**início do traço**.

**Os dois casos, e é só nisto que os modos diferem:**

| | `R` | `F` |
|---|---|---|
| girar · torcer · escalar · transladar | a rotação do segmento | **identidade** |
| espremer/esticar | **identidade** | base ortonormal com o eixo **z** em `normalizar(cabeça_espelhada − Õ)` |

⚠️ **É `F` que faz o espremer/esticar agir ao longo do SEGMENTO** e não ao longo do eixo `z` do
mundo — e é por isso que naquele modo a orientação vive em `F` e a rotação `R` não é usada.
⚠️ É também o único sítio onde a **cabeça** é lida — ver o aviso do §5.1 sobre ela.

**O espelhamento para um octante**, para cada eixo activo: um ponto inverte a coordenada desse eixo
**uma vez** se o bit do eixo está no octante, e **outra vez** se a coordenada correspondente do
**ponto âncora do traço** for negativa (duas inversões cancelam-se ⇒ não inverte). Uma rotação
espelha num eixo negando **a componente do eixo no vector de rotação e o ângulo**, o que é a
conjugação da rotação pela mesma reflexão.

---

## §7 — Fase F: aplicar aos vértices

### §7.1 — O deslocamento de um vértice

Para cada vértice `v`, com `p₀(v)` a posição dele **no início do traço**:

```
deslocamento(v) = Σ_i  peso_i(v) · [ P₄_i,a(v) · M_i,a(v) · P₄⁻¹_i,a(v) · p₀(v)  −  p₀(v) ]
```

onde `a(v)` é a área de simetria do vértice (§6). Depois: `deslocamento(v) ×= factor(v)` (§9).

### §7.2 — ⚠️ O que cada modo de facto escreve

| modo | `M` carrega | `F` carrega |
|---|---|---|
| rotação / torção | a rotação (+ escala unitária) + translação da origem | identidade |
| escala / translação | rotação (identidade na translação pura) + escala uniforme + translação | identidade |
| espremer/esticar | **só escala + translação** | a **direcção do segmento** |

### §7.3 — Rebase (porque não há passo de «restaurar»)

O deslocamento acima é medido **a partir das posições do início do traço**. Antes de ser aplicado,
subtrai-se-lhe `posição_actual(v) − p₀(v)`. ⇒ aplicar o resultado à posição actual aterra
exactamente em `p₀(v) + deslocamento(v)`, e **o traço não acumula** mesmo sem repor a malha entre
eventos.

⚠️ **Requisito equivalente, enunciado pelo que se observa:** a malha **não** é reposta ao estado do
passo de desfazer entre eventos — e não precisa de o ser, porque o rebase já entrega o mesmo
resultado. *Observável:* a pose depois de `N` eventos é a que o `G` acumulado pede, sem acumulação
de traço (as fixturas por evento medem-no directamente, §14).

### §7.4 — Depois do rebase

Aplicam-se, por esta ordem: as **travas de eixo** da escultura (zeram a componente) e o **recorte do
modificador de espelho** (um vértice a menos de uma tolerância do plano de espelho não o atravessa).

---

## §8 — Simetria

⚠️⚠️ **Requisito observável: a simetria inteira resolve-se numa passagem só do traço; as passagens
seguintes não produzem deslocamento nenhum.** É isso que os 8 octantes do §6 compram — as
transformações de todos os octantes existem já na primeira passagem, e cada vértice escolhe a sua.

Consequências, as três medidas:

1. a semente da travessia já inclui os parceiros espelhados (§2.1);
2. a franja que forma o pivô é filtrada pelo teste de lado (§2.5), logo **o pivô fica no lado do
   cursor**;
3. ⛔ **a simetria RADIAL é simplesmente ignorada** — os autores confirmam-no como defeito aberto
   ([#141625](https://projects.blender.org/blender/blender/issues/141625)). Para nós é **fronteira
   declarada**, não comportamento a copiar.

---

## §9 — Máscara, auto-máscara, ocultação

O factor por vértice que multiplica o deslocamento final é:

- `1 − máscara(v)` se houver atributo de máscara, senão `1`;
- **`0`** se o vértice estiver escondido;
- vezes o factor das **auto-máscaras** activas.

⚠️ **A máscara escala o deslocamento; não muda os pesos nem o pivô.** Prova:
`figura_girar_dedo_mascara05` e `figura_girar_dedo_mascara1` têm o mesmo pivô que
`figura_girar_dedo` e deslocamento máximo `0,192` e `0,108` contra `0,385`.

⚠️ A auto-máscara de topologia já foi um defeito conhecido deste pincel
([#74496](https://projects.blender.org/blender/blender/issues/74496), fechado).

---

## §10 — Ciclo de vida dentro de um traço

| quando | o que acontece |
|---|---|
| **1.º evento** | fixa-se o ponto de aplicação (âncora) e o raio em espaço de objecto; **constrói-se a cadeia inteira** (§2–§4) |
| **cada evento** | actualiza-se `G`; resolve-se a cadeia (§5); reconstroem-se as 8×`n` matrizes (§6); aplica-se (§7) |
| **fim do traço** | a cadeia é largada |
| **passar o rato sem premir** | ⚠️ **a cadeia é construída na mesma**, só para desenhar o indicador do pivô e dos segmentos sob o cursor |

⚠️⚠️ **O terceiro caso é a causa da má fama de desempenho deste pincel**: a construção completa
corre a cada movimento do rato, sem traço nenhum
([#127259](https://projects.blender.org/blender/blender/issues/127259),
[#73172](https://projects.blender.org/blender/blender/issues/73172),
[#106994](https://projects.blender.org/blender/blender/issues/106994)).
⇒ **Decisão recomendada para nós:** ou não desenhar o indicador ao passar o rato, ou calculá-lo com
um orçamento e memória própria. **Não** replicar a reconstrução por movimento.

---

## §11 — Degenerescências e bordas (todas medidas)

### §11.1 — ⭐ Pivô em cima do cursor ⇒ o pincel não faz NADA

Se a franja fizer o pivô coincidir com o ponto de aplicação, o 1.º segmento tem comprimento nulo. A
direcção inicial é a normalização de um vector nulo, a rotação resultante é a identidade, e — **com
a âncora ligada e um segmento** — a translação cancela-se (§5.1) ⇒ **deslocamento zero em toda a
malha**.

*Medido:* `figura_girar_dedo_origem_no_cursor` — comprimento do 1.º segmento `4,8e-8`, oráculo
`movidos = 0`, deslocamento máximo `0,000000`, apesar de um arrasto de `0,6`.
⚠️ **É um caso de produto, não um crash:** o artista arrasta e não acontece nada. Vale a pena
**avisar** em vez de imitar o silêncio.
⚠️ **Exigência de implementação:** a rotação entre dois vectores tem de devolver **identidade**
quando um deles é nulo. Um modelo que ali produza `NaN` ou uma rotação arbitrária diverge — foi
exactamente o que aconteceu no nosso modelo de referência (erro `2,5e-1` nessa fixture).

### §11.2 — ⭐ O polo da escala

`escala = comprimento₀ / (comprimento₀ − δ)` cresce sem limite quando o arrasto aproxima `δ` de
`comprimento₀`, e **muda de sinal** ao atravessar. No modo de escala **não há guarda**; no modo
espremer/esticar há a guarda de `1e-5` (§5.5).

*Medido:* `braco3d_esticar` atinge deslocamento máximo **`11,2`** numa peça de extensão `2,0`
(arrasto de `0,3`), e a fixture `figura_escalar_dedo_atravessa_origem` arrasta **através** do pivô.
⇒ **Perto do polo a paridade não é asserível**: a nossa concordância nessas duas fixtures é
`4,0e-4` e `7,8e-3` — em termos relativos `3,5e-5` e `1,5e-2` do deslocamento. §12.

### §11.3 — ⭐ Cursor exactamente sobre o plano de espelho

Com simetria em X e o cursor em `x = 0`, o parceiro espelhado da semente **é a própria semente**, e
o teste de lado (§2.5, primeiro ramo) reprova toda a franja com `x > 0`. O alvo produz então um
deslocamento **quase nulo** (`max = 0,001` em `figura_girar_cabeca_no_plano_simetria_x`) e o
resultado é **descontínuo** na posição do cursor: deslocar o cursor de `1e-6` muda a saída em
`2,4e-1`. ⇒ **caso a evitar, e a nomear na UI** se alguém o atingir.

### §11.4 — Vértices soltos e malha vazia

- Vértice sem nenhuma face: não é alcançado pela travessia (peso `0`) mas **entra na suavização
  §4 sem guarda** ⇒ caminho para valor indefinido. **Não reproduzir**: a nossa suavização deve
  deixar um vértice sem vizinhos com o peso que tinha.
- Se não houver vértice nenhum ao alcance, **a cadeia não é construída e o traço não faz nada**
  (retorno silencioso).

### §11.5 — Malha de várias peças com «só conectado» desligado

Ver §2.4: emparelhamento ganancioso ⇒ picos e deformação inconsistente, confirmado pelos autores
([#133739](https://projects.blender.org/blender/blender/issues/133739)).

---

## §12 — Determinismo, precisão, e de onde sai a barra de paridade

⭐ **A forma do resultado é determinística**, dadas: a ordem da semente (§2.1), a ordem FIFO da
travessia (§2.2), e uma suavização de Jacobi limpa (§4). ⛔ **O alvo não é determinístico na
suavização** através de fronteiras de partição (§4).

### §12.1 — ⭐⭐ O achado que decide a barra: a região é decidida por um `<` estrito em `f32`

O teste do §2.2 passo 3 é uma comparação **estrita** `distância < raio`, avaliada em **precisão
simples**. Um único vértice a atravessar essa fronteira muda a **franja**, logo muda o **pivô**,
logo muda a deformação **inteira** — e não por um infinitésimo.

**Medição (2026-09-13, 69 fixtures).** Corremos o mesmo modelo de referência duas vezes, mudando
**só** a precisão dessa comparação:

| fixture | raio | vértices que trocam de lado entre `f64` e `f32` | erro com `f64` | erro com `f32` |
|---|---|---|---|---|
| `braco3d_girar_ik1` | `0,300` | **1** | `5,652e-3` | **`1,081e-7`** |
| `braco3d_torcer_ik1` | `0,300` | **1** | `1,520e-2` | **`8,373e-8`** |
| `braco3d_girar_ik1_r029` | `0,290` | 0 | `5,787e-8` | `5,787e-8` |
| `braco3d_girar_ik3_r029` | `0,290` | 0 | `1,168e-7` | `1,168e-7` |
| `figura_girar_dedo` | `0,250` | 0 | `1,749e-7` | `1,749e-7` |

⇒ **Regra para a nossa implementação:** *fazer a comparação de região em `f32`, com `<` estrito, e
a distância calculada como raiz da soma dos quadrados em `f32`* — senão a paridade com o oráculo
degrada de `1e-7` para `1e-2` em malhas cujos vértices calham sobre o raio.

⚠️ **E a fragilidade fica, mesmo acertando:** em `figura_girar_braco_ik3` e `figura_girar_dedo`, um
raio `1e-6` maior muda a saída em `1,5e-1` e `9,3e-3` **sem que o erro contra o oráculo mude** —
as duas realizações caem do mesmo lado. *Estar perto da descontinuidade não obriga a discordar;
obriga a ser frágil.*

### §12.2 — A mesma espécie, segundo sítio: o critério de paragem do crescimento

`‖A − alvo‖ < L` (§3.2) é o mesmo tipo de comparação estrita, sobre uma **média**. Quando a
distância entre origens consecutivas calha **exactamente** em `L`, uma varredura inteira entra ou
sai, e o segmento muda de sítio.
*Medido:* `figura_girar_braco_ik5` e `braco3d_girar_ik3` têm margem `0,0000` e erro `2,2e-2`;
`figura_girar_braco_ik5_r021` e `braco3d_girar_ik3_r029` têm margem `0,0107`/`0,0100` e erro
`3,8e-7`/`1,2e-7`. Perturbar o raio em `1e-6` move a saída das duas primeiras em `1,0e-1` e
`2,1e-2` — assinatura de descontinuidade.

### §12.3 — ⭐ A barra que recomendamos

| classe de fixture | barra | de onde sai |
|---|---|---|
| **normal** (sem vértice sobre o raio, longe do polo) | **`1e-6` absoluto** em posição de vértice | é `~10×` o arredondamento de `f32` observado: o corpo das 54 fixtures concordantes fecha entre `5,5e-8` e `1,0e-6` |
| **sobre a descontinuidade** (§12.1, §12.2) | ⛔ **não asserir paridade** — asserir a **forma** (`χ`, sem auto-intersecção, monotonia do deslocamento ao longo do arrasto) | um `<` estrito não tem barra: a resposta salta |
| **perto do polo da escala** (§11.2) | **relativa**, `1e-4` do deslocamento máximo | o polo amplifica o arredondamento de entrada sem limite |
| **degenerada** (§11.1, §11.3) | asserir o **veredito** (deslocamento nulo), não números | o alvo produz `0` exacto |

⭐ **Estado medido do modelo de referência** (bancada fora da árvore, 2026-09-13, com a regra de
§12.1 aplicada): **54 das 69** fixtures concordam com o oráculo a `≤ 1e-5`, a esmagadora maioria em
`~1e-7`. As **15** restantes, uma a uma:

| quantas | classe | erro | mecanismo |
|---|---|---|---|
| `2` | descontinuidade do crescimento | `2,1e-2` · `2,2e-2` | §12.2 |
| `2` | polo da escala, atravessado ou perto | `7,8e-3` · `4,0e-4` | §11.2 |
| `7` | família da escala, longe do polo | `2,0e-5` a `2,6e-5` | arredondamento amplificado pelo quociente de §5.4 — relativo `≤ 1,6e-5` |
| `2` | degeneradas | `2,5e-1` · `1,7e-4` | §11.1 e §11.3 — ⭐ **as duas são bugs do nosso MODELO, não limites de paridade**: o alvo devolve deslocamento nulo e o modelo não |
| `1` | auto-suavização não modelada | `2,6e-4` | §15 — o modelo não implementa a opção |
| `1` | **por explicar** | `2,7e-3` | `figura_girar_cabeca_no_plano_sem_simetria`: `2 %` de um deslocamento de `0,137`. ⛔ **Não é descontinuidade** — perturbar o raio ou o cursor em `1e-6` move a saída em `≤ 2,4e-6`, logo é um desvio **sistemático**, não um salto. ⛔ **E não é a confusão das duas sementes do §2.1**, que foi o candidato nomeado e caiu na medição: apagar o pré-peso muda a saída desta fixtura em `0,000e+00`. Fica **aberto**, com esses dois mecanismos já eliminados |

⚠️ **Bit-parity NÃO é a meta** e não deve ser prometida — cerca da casa ([ADR-0162](../../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md)), mantida aqui.

---

## §13 — Custo

| fase | custo | notas |
|---|---|---|
| achar o vértice do cursor | `O(log V)` pela árvore espacial | sem limite de distância |
| travessia do 1.º segmento (§2.2) | `O(região)` | só alcança o que está ligado à semente |
| **ligações entre peças** (§2.4) | ⛔ **`O(V²)`** | só com «só conectado» **desligado**; recalculado sempre que a distância máxima muda |
| **cada varredura de crescimento** (§3.1) | `O(V)` sobre a malha **inteira** | e são várias varreduras por segmento |
| suavização (§4) | `O(V · N)` **por segmento** | `N` até `100`, segmentos até `20` |
| aplicação por evento (§7) | `O(V · n_segmentos)` | ⚠️ **nenhuma parte da malha é excluída pelo raio**: um vértice a qualquer distância do cursor pode ter peso, porque a cadeia cresce pela ligação e não pela vizinhança espacial. ⇒ o custo por evento é da malha **inteira**, não da região sob o pincel |
| memória | `O(V · n_segmentos)` | um peso por vértice **por segmento** |

⚠️ **O produto destes factores é a queixa dos autores e dos utilizadores** (§10): `20` segmentos ×
`100` suavizações × `V` é o pior caso, e ele é pago **por movimento do rato**.
⭐ **Recomendação:** limitar segmentos e suavizações por orçamento medido, guardar a cadeia entre
eventos (o alvo já o faz), e **não** reconstruir ao passar o rato.

---

## §14 — Vectores de teste

`fixtures/pose/` — **69** traços scriptados sobre **3 malhas nossas**, mais **11** séries
**por evento** (o estado depois de *cada* evento do mesmo traço, não só o final).
Formato, vocabulário e proveniência: [`fixtures/pose/README.md`](fixtures/pose/README.md).
Cada fixture traz no cabeçalho **todos** os controlos do §1 com que foi gerada.

Cobertura, por eixo:

| eixo | fixtures |
|---|---|
| três modos × normal/invertido | `girar` · `torcer` · `escalar` · `transladar` · `espremer` · `esticar` |
| segmentos | `1`, `2`, `3`, `5` |
| desvio da origem | `0`, `0,5`, `1,0`, `2,0` |
| suavizações | `0`, `4`, `10` |
| âncora | ligada / desligada (`_solto`) |
| trava de rotação | `_travado` |
| peças desligadas | `pulso_*_conectado` vs `_solto001` / `_solto01` (distância máxima `0,01` e `0,1`) |
| simetria X | `_simetria_x` |
| máscara | `_mascara05`, `_mascara1` |
| pressão | `_pressao03` (controlo negativo: tem de dar o mesmo que sem pressão) |
| força | `_forca05` |
| curva | `_constante` |
| degenerado | `_origem_no_cursor` (deslocamento nulo) · `_cabeca_no_plano_*` |
| resolução de eventos | `_4ev`, `_36ev` (o mesmo arrasto em 4 e em 36 eventos) |

⚠️⚠️ **`_4ev` / `_36ev` NÃO são um gate de «não depende da taxa de eventos» — são o gate que MEDE a
dependência** (§5.1-bis). Com **um** segmento a pose final é idêntica ao bit nas três amostragens;
com **três**, difere em `5,3e-2` e `1,1e-2`. O gate certo é *«a nossa saída bate a do oráculo **em
cada taxa**»* — medido `3,4e-7` (4 eventos), `4,3e-7` (12) e `3,4e-7` (36).
⛔ Um gate escrito como «as três taxas concordam entre si» **reprova sobre produto correcto**.

---

## §15 — O que os próprios autores registam que está mal

Re-dito em palavras nossas, com a fonte (§4.1.12). ⭐ **Cada linha é uma escolha que já foi paga:**

| # | o que é | o que fazemos |
|---|---|---|
| [#141625](https://projects.blender.org/blender/blender/issues/141625) | simetria **radial** ignorada | fronteira declarada (§8) |
| [#133792](https://projects.blender.org/blender/blender/issues/133792) | a **auto-suavização** do pincel só age dentro do raio inicial, enquanto a deformação alcança muito mais longe ⇒ o efeito dela «desaparece» longe do cursor | ⚠️ é a causa do resíduo de `2,6e-4` da nossa fixture `_suavizacao05`. **Não copiar**: se oferecermos auto-suavização aqui, ela segue **os pesos**, não o raio |
| [#133739](https://projects.blender.org/blender/blender/issues/133739) | muitas peças + «só conectado» desligado ⇒ picos | §2.4; emparelhamento ganancioso |
| [#127259](https://projects.blender.org/blender/blender/issues/127259) · [#128329](https://projects.blender.org/blender/blender/issues/128329) · [#73172](https://projects.blender.org/blender/blender/issues/73172) · [#106994](https://projects.blender.org/blender/blender/issues/106994) | **passar o rato** sobre malha densa trava o editor; com «só conectado» desligado, cada zoom/pan volta a pagar o `O(V²)` | §10, §13 — **não** reconstruir ao passar o rato |
| [#130465](https://projects.blender.org/blender/blender/issues/130465) | espremer/esticar levava a malha a `NaN`, e o desfazer não a recuperava | a guarda de `1e-5` (§5.5) é a cura deles; a nossa tem de existir **e** ser testada |
| [#70385](https://projects.blender.org/blender/blender/issues/70385) | o **desvio da origem** partia o pincel | §2.6 — a compensação por crescimento é a resposta |
| [#74496](https://projects.blender.org/blender/blender/issues/74496) · [#87596](https://projects.blender.org/blender/blender/issues/87596) | auto-máscaras ignoradas | §9 — ligar desde o início |
| [#82034](https://projects.blender.org/blender/blender/issues/82034) | o alvo de deformação mudou de comportamento entre versões | fora de escopo aqui |

⭐ **O modo por conjuntos de faces nasceu (2020) de uma necessidade nomeada pelo autor:** dar
**controlo explícito** sobre onde os pivôs caem, quando a forma sozinha não chega. ⇒ se algum dia
alguém quiser controlo de pivô aqui, o precedente diz que a saída é **marcar a malha**, não afinar
a heurística.

---

## §16 — Fora de escopo (nomeado, não esquecido)

- ⛔ **Os dois modos de origem por conjuntos de faces** (incluindo o modo de cinemática directa) —
  exclusão C da missão: não temos conjuntos de faces. ⚠️ **É só ali** que o deslocamento do arrasto
  leva um termo de correcção adicional; no modo de topologia esse termo é **exactamente zero**
  (verificado).
- ⛔ **Multiresolução e malha dinâmica** — os dois caminhos existem no alvo e seguem a **mesma** lei;
  as diferenças são de como os vértices são endereçados, não de comportamento.
- ⛔ **O alvo de deformação «simulação de tecido»**.
- ⛔ Qualquer pincel de **cor**.

---

## §17 — Lista de verificação para quem implementa

1. [ ] Rotação entre vectores devolve **identidade** para vector nulo (§11.1) — com gate.
2. [ ] Comparação de região em **`f32`**, `<` **estrito** (§12.1) — com gate sobre `braco3d_girar_ik1`.
3. [ ] Sementes **ordenadas** e travessia **FIFO** (§2.1, §2.2).
4. [ ] Retrocesso dos pesos na varredura que pára o crescimento (§3.2).
5. [ ] Pesos por segmento são **diferenças** (§3.3).
6. [ ] Suavização **exclui o próprio vértice** e **exclui** as ligações entre peças (§4).
7. [ ] Vértice sem vizinhos **mantém** o peso (§11.4) — divergência deliberada do alvo.
8. [ ] Escala/espremer forçam **um** segmento (§3).
9. [ ] Guarda de `1e-5` no espremer/esticar (§5.5) — com gate que prova o `NaN` sem ela.
10. [ ] Âncora repõe a origem do **último** segmento (§5.1).
10b. [ ] **Solver incremental** — a rotação mede-se contra o estado inicial, a posição avança a
    partir do estado do evento anterior (§5.1-bis). Gate: `figura_girar_braco_ik3` nas **três**
    taxas de evento, cada uma contra a fixture dela.
11. [ ] Rebase em vez de repor a malha (§7.3).
12. [ ] Pressão **não** entra na força (§1.3) — com gate sobre `figura_girar_dedo_pressao03`.
13. [ ] Inversão **troca de modo**, não de sinal (§0).
14. [ ] Indicador ao passar o rato **não** reconstrói a cadeia (§10, §13).
15. [ ] Auto-suavização, se existir, segue os **pesos** e não o raio (§15).
