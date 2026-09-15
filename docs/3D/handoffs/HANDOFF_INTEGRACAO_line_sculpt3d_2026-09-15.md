# HANDOFF de INTEGRAÇÃO — `line/sculpt3d`, 2026-09-15

> **Continuação de
> [`HANDOFF_INTEGRACAO_line_sculpt3d_CONTORNO_2026-09-14.md`](HANDOFF_INTEGRACAO_line_sculpt3d_CONTORNO_2026-09-14.md)**,
> que chegou a `196 KB` — acima do joelho medido no `CLAUDE.md` §5.0. A
> numeração das secções **continua a daquele** (§35 em diante), de propósito: os
> itens que estas waves fecham estão nomeados no §34.7 de lá, e renumerar
> partiria o endereço.
>
> **A jornada:** *«tudo que estiver em aberto corrija»* (ordem do dono). As seis
> waves abaixo fecham os itens que a linha tinha em aberto que eram **dela** —
> os que ficam são de outra janela, de outra linha ou do dono, e o §40 diz de
> quem é cada um, com o número ao lado.
>
> ⛔ **Nada integrado, nada enviado** (`CLAUDE.md` §0.7). Zero contadores
> partilhados mexidos: nem `PROJECT_SCHEMA`, nem `VEC_SCENE_SCHEMA`, nem
> `FLIP_SCHEMA`, nem `FIELD_DOC_VERSION`, nem os registos de componentes; zero
> contrato congelado; zero ADR; zero pacote externo.

---

## §35 — ⭐⭐⭐ A **INVERSÃO** do projectar nega a TRANSLAÇÃO e **não vira o raio**: o placar salta de `12` para `14`

### §35.1 — ⛔⛔ O desvio estava escrito no número que a espec publica

O par `…_invertido` / `…_subtrair` desviava **`1,415e-1`**, e a §34.3 já notara
que esse é **exactamente** o número que a espec §6.6 dá para `max |d + d′|`
entre a base e a invertida **no alvo**. *Ler o próprio desvio no número que a
espec publica para a assimetria do alvo é o diagnóstico inteiro:* ele diz que a
NOSSA saída é o espelho **exacto** da base e a dele não.

Confirmado no corpus, sem correr uma linha de produto:

| medida | valor |
|---|---|
| `max abs(dz_base + dz_invertido)` | `1,414508e-1` |
| `max abs(dz_base + dz_acima_bidir)` | **`0,000000e0`** |
| `max abs(dz_invertido − dz_acima_bidir)` | `1,414508e-1` |
| `max abs(dz_invertido − dz_subtrair)` | `0,000000e0` |

⇒ a `…_acima_bidir` **é** o espelho exacto da base, e a invertida **não é**. E a
nossa inversão produzia exactamente a `…_acima_bidir`, porque virar o raio e
ligar os dois sentidos encontram o mesmo alvo à mesma distância.

### §35.2 — ⛔⛔⛔ Eram DUAS suposições erradas que encaixavam uma na outra

1. a cena das duas fixturas fora reconstruída com o alvo **ACIMA** (`+0,5`),
   porque a altura foi lida do **deslocamento máximo**, que é para cima;
2. a lei fazia a inversão **virar o raio**, e o comentário dela registava o
   motivo: escrita como sinal no fim, a fixtura media **`0` vértices movidos
   contra `301`**.

*Cada uma «provava» a outra.* Com o alvo em cima e os dois sentidos desligados,
a negação no fim não tem o que negar — logo a lei «certa» parecia impossível, e
a cena «certa» parecia confirmada pelo deslocamento.

### §35.3 — ⭐⭐ O que as separou foi o CABEÇALHO, não a aritmética

`projectar_base` e `projectar_invertido` têm cabeçalhos **idênticos campo a
campo** (`sentido: ADD` nos dois — a inversão veio pelo GESTO), e a linha de
descrição da base diz *«contra um plano ABAIXO»* enquanto a da invertida diz *«o
MESMO, invertido»*. ⇒ **a cena é a mesma, com o plano abaixo.**

⚠️ **E a varredura da ALTURA fechou a porta antes da cura:** com a lei do raio
virado, **nenhuma** altura de `−1,0` a `+1,0` põe a fixtura dentro da barra (o
melhor é `1,250e-1` a `h = +0,625`, contra uma barra de `2e-6`). *Se nenhum
valor do parâmetro livre encaixa, o que está errado é a LEI e não a cena* — e a
varredura custou uma corrida de teste.

### §35.4 — A cura, e porque ela não explode

A espec §1.2 escreve o mecanismo: *«o sinal entra no factor, logo a translação
nega»*. O raio é o do pincel; o `sign` entra no deslocamento. Medido
**`1,415e-1 → 2,384e-7`** nas duas; **`VERDE_N` `12 → 14`** de `16`.

⭐ A minha objecção — *«a negação empurra para longe, o dab seguinte mede mais
longe, e isso explode»* — estava errada por uma peça: **a pegada é uma esfera em
torno de um centro PARADO**, e o vértice sai dela. O vértice do centro do 1.º
dab anda o vão inteiro (`w = 1`) e no dab seguinte já está a `0,5` de um raio de
`0,35` ⇒ **fora da pegada**, com a excursão travada em exactamente `0,5`. Foi
esse `0,500000` cravado no corpus que refutou a hipótese.

### §35.5 — O gate, e a TERCEIRA metade é o discriminador

`a_inversao_nega_a_translacao_e_nao_vira_o_raio`. ⛔⛔ **As duas primeiras
metades são satisfeitas pelas DUAS leis** — com o alvo abaixo, negar a
translação e virar o raio dão o mesmo sinal no primeiro dab —, e foi por isso
que a errada shipou. O que as separa:

| lei | alvo ABAIXO, `Ctrl` | alvo ACIMA, `Ctrl`, sem os dois sentidos |
|---|---|---|
| virar o raio | sobe | **sobe** (o raio virado encontra-o) |
| **negar a translação** (a nossa) | sobe | **nada se move** |

Mutação: restaurar a lei antiga reprova-o.

### §35.6 — E o roteiro da `=45` ensinava o contrário

Os passos (2) e (3) diziam *«o Ctrl vira o raio para o lado de lá»*. Reescritos:
o `Ctrl` faz o barro **SUBIR** (afasta-o da mesa), e quem alcança o outro lado é
o `Search Both Ways`, demonstrado a **olhar a peça por baixo**. *Uma cena que
ensina o contrário do que acontece é pior que uma cena ausente* (§5.0).

---

## §36 — ⭐⭐⭐ A **DIRECÇÃO DO RAIO** não deriva com a trincheira que o traço abre: `14 → 15`

### §36.1 — O desvio era inteiramente LATERAL

`…_normal_plano_area` desviava `1,281e-1`, e a sonda do perfil mostrou de onde:

| | nosso | oráculo |
|---|---|---|
| deslocamento **lateral** (`dx`/`dy`) | `1,281e-1` | **`0,000e0`** |

E no corpus aquela fixtura é **byte-idêntica** à `…_base` (`max abs(Δ) = 0,0`
nos `1 681` vértices) ⇒ com a peça a ser o plano `z = 0` visto de frente, a
direcção do plano e a da vista coincidem **e continuam a coincidir** ao longo
dos seis dabs, com o barro já a afundar.

⭐ **No modo `Plane` a normal da área É a direcção do raio**, e lida da
superfície VIVA ela inclina sobre o vale que os dabs anteriores cavaram: o raio
parte de lado e o barro é **transportado** em vez de empurrado. ⚠️ *Nenhum outro
verbo sente isto*, porque em todos os outros a normal decide um deslocamento que
é uma **fracção do raio**; aqui o `d` é uma distância da **CENA**.

### §36.2 — A cura tem DUAS metades, e a segunda quase passou despercebida

**(a) O plano passa a ser ajustado sobre a superfície CONGELADA.** O mecanismo
já existia: o `fit_plane_over` tem uma lista por verbo (`ClayStrips`,
`ClayThumb`, `MultiplaneScrape`) e o projectar entra nela **sem a cláusula do
`Accumulate`**, porque aquele interruptor deixou de lhe ser oferecido (§34.4).
Medido: `1,281e-1 → 1,036e-2`.

**(b) E o `base_nrm` NÃO é o pen-down: é o PRIMEIRO TOQUE.** A captura é
preguiçosa — um vértice entra no `base_*` quando o primeiro dab o alcança —,
logo um vértice que só entra no 3.º dab é fotografado com a normal que tem
**nessa altura**, já inclinada pelos vizinhos que os dois dabs anteriores
afundaram. ⇒ `SculptStroke::nrm0_do_pen_down` fotografa as normais da malha no
**primeiro dab** (onde ela ainda é a do pen-down por construção, porque nada foi
escrito) e **só para este verbo neste modo**. Medido: `1,036e-2 → 1,639e-7`, que
é o número da `…_base`.

⚠️ *A diferença entre «congelado no pen-down» e «congelado no primeiro toque» é
de `5 000×` a barra desta bancada.* ⚠️ E ela nasce no primeiro dab e não no
`begin` por uma razão de **assinatura**: o `begin` não recebe o pincel, logo
pagaria um `O(V)` a todos os verbos para servir um.

### §36.3 — O gate, e a régua é o deslocamento LATERAL

`a_direccao_do_raio_nao_deriva_com_a_trincheira`, com controlo positivo (a
trincheira tem de ser cavada, senão o gate mede o nada). *Um desvio vertical
seria força a mais; este é o barro a andar para o lado, e é isso que nomeia o
mecanismo.* Mutação **2 de 2** — repor o plano vivo reprova, e apagar a
fotografia das normais também: **as duas metades sangram separadas**.

### §36.4 — ⏳ A `1` que fica, com o diagnóstico escrito

`…_dureza05`, `2,367e-2`. Medida vértice a vértice: **`3` de `301`** desviam
(`2,4e-2`, `1,2e-2`, `1,6e-3`) e os outros **`298` batem a `≤ 5,0e-5`**.

⚠️ **Não é um erro de lei** — um erro de lei move a população inteira. Os três
estão na **BORDA da pegada**, que com o `from_live` se **move** enquanto o barro
afunda: o dab em que um vértice sai da esfera é decidido ao último bit, e a
dureza `0,5` desloca o ponto onde isso acontece. *É a mesma família da banda de
empate do raio que a bancada já declara, um dab mais tarde.*

---

## §37 — ⭐⭐⭐ Um alvo **ESCONDIDO** não conta (espec §6.1)

### §37.1 — A cláusula que não tinha como chegar

A espec §6.1 define os alvos como *«todo objecto da camada de vista que **não**
é o activo, **é** malha, e **não** está escondido»*. A última cláusula não
chegava: o `SceneObject` não carrega visibilidade e a `Sculpt3dScene` não vê o
mundo ECS.

### §37.2 — ⛔⛔ E o laço estava escrito DUAS vezes

Letra a letra, nos dois pen-downs (o do traço e o do filtro de tecido). *Duas
cópias de um filtro são duas respostas à mesma pergunta, e a cláusula nova teria
entrado só numa delas.* ⇒ **`Sculpt3dScene::alvos_visiveis`**, com **três**
consumidores.

⭐ **E a terceira é a que torna a cura honesta:** a contagem da recusa em voz
alta. Sem ela, um traço cujo único alvo está escondido moveria zero vértices **e
ficaria calado** — exactamente o defeito que o §33 construiu o módulo para não
ter. *Filtrar a lista sem filtrar a contagem troca um pincel inerte por um
pincel inerte e mudo.*

### §37.3 — A recusa diz a CURA, não só o facto

| estado | o que ela diz |
|---|---|
| não há outra peça | *«precisa de OUTRA peca na cena»* |
| a(s) que há está(ão) escondida(s) | *«precisa de OUTRA peca À VISTA — abra o olho dela na Hierarquia, ou saia do isolamento»* |

*As duas levam o artista a gestos opostos — uma manda criar geometria, a outra
manda abrir um olho —, e num contador só leem-se exactamente igual.*

### §37.4 — Onde o espelho vive, e porque não é uma porta do host

`Sculpt3dScene::escondidas`, um **CONJUNTO de ids** reconstruído do mundo a cada
quadro pelo `entities_sync` — o **único** sítio da família que segura o mundo e
a cena ao mesmo tempo, e que já corre antes de a Hierarquia ser desenhada.
⛔ Uma **sétima** porta no `AppHost` reprovaria o censo por licença que a Fase B
pagou.

⚠️⚠️ **A nota do `isolated` ao lado recusa um `hidden: bool` por peça, e isto
NÃO a contradiz.** Ela recusa um estado que **nenhum gesto de isolamento pode
produzir** (*«duas escondidas e uma à vista sem ninguém ter isolado»*); o olho da
Hierarquia é **por-linha por construção**, logo esse estado é exactamente o que
o artista autora. *A diferença é qual estado é AUTORÁVEL.*

### §37.5 — ⭐ Uma consequência de produto que vem de graça

O `visible_pieces` já era consumido pelo **DESENHO** e pelo **PICK**, logo
esconder uma peça na Hierarquia passa a escondê-la também no canvas da
escultura. **Antes o olho não fazia nada ali** — a peça ficava à vista e o
pincel recusava-se a tocá-la.

### §37.6 — ⏳ LIMITE DECLARADO

O `Visibility` desta casa **não propaga para descendentes** (está escrito no
próprio componente), logo uma peça dentro de um **grupo** escondido continua a
contar como alvo. Curá-lo é andar os ascendentes, que é o que a
`ph2d-entity-visibility` faz para outro meio — e essa crate **não é dependência
desta**.

### §37.7 — Os gates

A lei é uma função **LIVRE** (`space::aparece`) e não um método da cena, porque
a cena pede um `wgpu::Device` e o gate nasceria `#[ignore]`: *o CI nunca o
correria.* Mutação **2 de 2** — apagar a leitura do conjunto reprova a lei pura,
e apagar quem enche o espelho reprova o censo das três rotas (que também proíbe
o regresso do `if i != activo` escrito à mão).

---

## §38 — ⭐⭐⭐ O painel deixa de **prometer** knobs que o barro não sente

### §38.1 — A lei já estava escrita no pintor desta crate

*«Uma row condicional é PULADA, não desenhada apagada: um controlo apagado que
ainda despacha mente, e um que não despacha é a affordance morta que esta casa
varre.»* ⇒ **esconde-se o que se pode esconder, e diz-se a razão onde não se
pode.**

| knob | verbo | medição | cura |
|---|---|---|---|
| `Strength` | `Density` | `0,000e0` entre `0,1` e `1,0` | **escondido** (`Verb::a_forca_chega_ao_barro`) |
| `Auto-Smooth` | `Density` | `0,000e0` | **escondido** (3.ª razão do `o_auto_smooth_chega`) |
| a **CURVA** | `Mask` · `Density` · `Pose` fora da torção | `0,000e0` | **razão à vista** |

⚠️ **A terceira razão do auto-smooth não é a segunda com outro nome:** o `Cloth`
e o `Boundary` **têm** região e resolvem-na eles próprios; o `Density` **não tem
vértice nenhum para alisar**. *Duas leis que dão a mesma resposta hoje e por
razões diferentes separam-se no dia em que uma delas mudar.*

### §38.2 — Porque a curva não pode ser escondida, e o que ficou em vez disso

A fileira da curva é a única que o painel pinta **SEMPRE**, por cerca de produto
medida e gateada (`the_basic_level_never_hides_the_curve_that_shapes_the_dab`).
⭐ **O próprio comentário daquela cerca já prescrevia esta saída e dizia que ela
«não existe hoje».** Agora existe: `Brush::curva_inerte` devolve a **RAZÃO** — um
enum, **não** uma chave de i18n, porque o motor não sabe o vocabulário da
interface — e o painel mapeia-a para o texto.

⭐ **Os doze chips ficam VIVOS de propósito:** a curva é um valor **autorado** do
pincel, e escolhê-la com este verbo na mão continua a valer para o seguinte. *O
que faltava era o app dizer que o barro de AGORA não a sente.*

### §38.3 — ⭐⭐ E a lei da POSE foi MEDIDA, não deduzida

Varridas as cinco deformações × `1/2/4/8` segmentos pela porta do produto:

| deformação | seg 1 | seg 2 | seg 4 | seg 8 |
|---|---|---|---|---|
| Rotate · Scale · Translate · Squash | `0` | `0` | `0` | `0` |
| **Twist** | `0` | `2,755e-1` | `3,866e-1` | `3,216e-1` |

⇒ **DUAS** condições, e nenhuma se adivinha da outra: só a torção lê a curva
(espec §1.2), e com **UM** segmento não há nada para ela repartir — que é
exactamente o valor de **FÁBRICA**. ⚠️ *Escrever só a primeira faria o painel
prometer uma curva viva onde ela não faz nada.*

⚠️⚠️ **E a entrada da catraca dizia a coisa certa pela razão errada:** ela
explicava o morto com *«este censo mede o modo de OMISSÃO»*, e a 1.ª medição
desta wave leu `0,000e0` na torção também — porque o arnês do censo tem **um**
segmento e nenhum arrasto de ecrã. *Uma explicação que aponta para o sítio certo
por um mecanismo errado sobrevive a todo gate.*

### §38.4 — ⛔⛔ TRÊS gates cujas premissas MORRERAM, e as três foram reescritas

| gate | premissa morta |
|---|---|
| `the_second_pass_is_refused_for_two_different_reasons` | as razões eram **três** ⇒ renomeado |
| `the_basic_level_never_hides_the_two_knobs_every_brush_has` | **nem todo pincel tem força** |
| `a_lista_do_censo_cobre_os_knobs_incondicionais` | o piso era `2` e a **população** encolheu para `1` (só o raio) |

⭐ A excepção do segundo é **DERIVADA** da mesma porta que o painel consulta —
uma lista aqui e um predicado lá seriam duas respostas à mesma pergunta —, e as
**duas** metades da população estão afirmadas, senão um predicado constante
passaria. ⚠️ E o terceiro é a forma que o §5.0 nomeia: *um piso que segurasse o
`2` mediria uma lista que já não existe*; ele mudou de **grandeza** e não de
força (passou a ser *«os knobs que o painel pinta para quase todo verbo»*).

### §38.5 — E o censo ganhou a TERCEIRA metade

Todo morto que o painel **PINTA** tem de ser **EXPLICADO na tela**. *Nomear um
morto num comentário de teste não o cura para o artista* — ele continua a
arrastar o controlo e a não ver nada.

O gate de pixel usa **GLIFOS** e não a banda reservada (o achado §4.2 da
auditoria do L-System: *apagar a pintura inteira deixava o gate verde*), com o
**mesmo verbo** dos dois lados e só o `Segments` a mudar. Mutação **2 de 2**.

---

## §39 — ⭐⭐⭐ A catraca dos **ADORMECIDOS** vai a ZERO, e os **ids soltos** ganham censo

### §39.1 — A entrada do `Density` prescrevia a própria saída

*«O arnês teria de correr o `refine_for_dab` e comparar a CONTAGEM de vértices
em vez das posições.»* ⛔ E não podia chamar os motores soltos: o cabeçalho
daquele ficheiro declara que *«a régua é o PRODUTO, nunca as funções soltas»*, e
uma segunda cópia da ordem colapso→refino divergiria da primeira.

⇒ **`dyntopo::passe_nos_motores`** — o miolo do `refine_for_dab` **sem CENA e
sem DEVICE**, com **dois** chamadores. O que fica na cena é o que precisa dela:
as três recusas, o alvo de aresta, a costura com o traço em voo
(`shrink_with`/`grow_with`), a queixa e o `mesh_rebuilt`.

⚠️ **A ORDEM viaja dentro da porta.** O colapso primeiro: as duas metades falam
com o traço em voo por canais diferentes (renumeração · nascimentos), e refinar
antes faria a renumeração descrever índices que já não existem. *Uma ordem que
vive no corpo de quem chama é uma ordem que o segundo chamador pode escrever ao
contrário.*

### §39.2 — ⛔⛔ E a TRIANGULAÇÃO é do produto, não do arnês

Os dois motores **recusam quads por geometria**, e quem os tritura é o pen-down.
A peça do censo é uma esfera UV, que é toda quads ⇒ **sem ela o passe seria um
no-op silencioso e o verbo continuava adormecido com o censo a dizer que
acordou.** Há mutação a prová-lo (ela reprova **dois** gates).

### §39.3 — A régua ganhou a CONTAGEM

*Dois barros de tamanhos diferentes são diferentes*, e sem essa metade o `zip`
das posições compara o **PREFIXO** e lê `0,0` sobre uma malha que ganhou dez mil
vértices. ⚠️ **Duas grandezas numa porta, de propósito:** o censo só pergunta
*«é zero?»*, e escrever uma segunda régua ao lado faria o censo escolher qual
chamar por verbo — a lista paralela que aquele ficheiro existe para não ter.

**Medido:** `Density × radius` lê **`1,404e3`**; `strength`, `hardness` e
`auto_smooth` estão **escondidos** (§38) e por isso não são medidos; `falloff` é
**MORTO** e entra na catraca com a razão à vista. **Verbos que o arnês não
acorda: `0`.**

⚠️⚠️ **Uma catraca VAZIA não é uma catraca morta — é a mais apertada que
existe:** já não há uma linha onde alguém possa escrever um verbo inerte calado,
e o piso **inverteu-se** (a afirmação passa a ser *«o arnês acorda os 32»*).

Os **12** gates de GPU da densidade passam contra a porta extraída.

### §39.4 — Os IDS SOLTOS, e o buraco é MAIOR do que a nota dizia

O censo das fileiras cobria os grupos de **array** e a tabela de **toggles**; um
id **solto** pintado à mão fora das duas era invisível. São **`5`**, contados
mecanicamente das declarações (`REF_MODE_ALL`, os três eixos do espelho, o
`CLOSE`).

⛔⛔ **E os dois gates de paridade do `ph2d-editor-core` extraem o conjunto
VAZIO para este painel**, por duas razões independentes:

1. o leitor de fontes filtra por **nome de ficheiro** (`paint` / `sections`) e
   salta os **seis** módulos de `src/paint/`, que é onde os widgets são
   desenhados;
2. o extractor só colhe `ids::LITERAL` como **primeiro argumento** de
   `.register(`, e este painel nunca o escreve — os ids viajam como argumentos
   para helpers.

⇒ **nenhum** id deste painel estava ao alcance deles, e a mesma forma atinge
qualquer painel que ponha os pintores num subdirectório ou passe ids a helpers.
⏳ **Dívida NOMEADA: a cura vive no `ph2d-editor-core`.**

⚠️ E dos cinco, o **`CLOSE` não tinha gate de costura nenhum** — ele é chrome
pintado pelo helper partilhado, logo a costura dele é da **fundação**. Fica como
**isenção nomeada** dentro da catraca, em vez de se ler como coberto.

Mutação **3 de 3** (apagar um registo · uma entrada fantasma na catraca · uma
entrada em falta), cada uma no seu assert.

### §39.5 — Dois tectos curados por CORTE

| ficheiro / função | era | cura |
|---|---|---|
| `paint/brush.rs::paint_brush_tail` | `223` de `200` | `paint_a_curva` (a cerca, os chips e a razão) |
| `censo_dos_knobs_tests.rs` | `826` de `700` | `censo_dos_knobs_arnes.rs` (a régua separada do que ela mede) |
| `populate_censo_tests.rs` | `656` de `600` | `populate_censo_soltos_tests.rs` |

⛔ **Nenhum por uma entrada no `FILE_OVERAGE_OK` / `FN_OVERAGE_OK`** (§5.0).
⚠️ O primeiro foi estourado **pela wave anterior desta mesma jornada** — *um
tecto de função também soma dentro de uma linha, não só entre linhas*.

---

## §40 — ⏳ O que fica ABERTO depois desta jornada

| # | item | estado | de quem |
|---|---|---|---|
| 1 | `…_dureza05`, a **última** fixtura reconstrutível fora da barra | ⏳ **diagnosticada**: `3` vértices de `301` na BORDA MÓVEL da pegada; os outros `298` batem a `≤ 5,0e-5`. Fechá-la pede aritmética bit-exacta com o alvo, que a espec §8.1 já declara impossível | — |
| 2 | as **6** fixturas que precisam da geometria do alvo | ⏳ **acto do E** — uma emenda ao emissor | **E** |
| 3 | a **folga simétrica** (§10.3) | ⏳ **decisão do dono**, e a lei alternativa continua escrita, medida e `#[cfg(test)]`. *Reproduzir o alvo é reproduzir um defeito; divergir sem o dizer é pior* — shipa a do alvo, que é a que o corpus mede | **dono** |
| 4 | o `Visibility` **não propaga** para descendentes | ⏳ limite declarado (§37.6): uma peça dentro de um grupo escondido continua a contar como alvo | — |
| 5 | os gates de paridade do `ph2d-editor-core` são **cegos a este painel inteiro** | ⏳ dívida nomeada (§39.4), e a cura vive naquela crate | outra linha |
| 6 | o `Scale`/`Translate`/`Squash` lêem **só** a componente axial do arrasto | ⛔ é a lei da espec §5.4/§5.5, escrita no roteiro. Mudá-la é **decisão de produto** | **dono** |
| 7 | o **auto-smooth** do `Cloth` e do `Boundary` | ⛔ **divergência declarada**: nenhuma das duas especs o prescreve, e *inventar uma lei para um pincel de clean-room sem referência é o que a parede existe para impedir*. O painel **já o esconde**, logo não há controlo morto — as duas saídas (medir o alvo numa janela **E**, ou ordem do dono para o construir seguindo os pesos) continuam nomeadas | **E** ou **dono** |
| 8 | o `tip_roundness` da vassoura | ⏳ **pré-existente e de outra linha**, com a medição agora feita — ver §40.1 | outra linha |

### §40.1 — ⭐ O `tip_roundness`, MEDIDO

| facto | valor |
|---|---|
| ocorrências em `crates/` no **merge-base** | `14` em 8 ficheiros |
| ocorrências em `crates/` no **HEAD** | `14` em 9 ficheiros |
| adições desta linha | **zero** (o único commit do ramo que o toca é um **movimento** de `brush.rs` para `brush_default.rs`) |
| commit que o introduziu | `e1b8343da`, a wave da FAIXA — **em `main`**, de outra linha |

⚠️ **E ele sobreviveu ao commit que levou a `ph2d-sculpt3d` a zero citações**, o
que é a prova mecânica de que **a vassoura foi estendida depois**.

⭐⭐ **O que a medição acrescenta, e que muda a pergunta:** o
`docs/3D/ferramentas/blender_sculpt_oracle.py` **atribui-o como propriedade
pública** (`br.tip_roundness = …`) ao lado de `area_radius_factor`,
`plane_offset` e `plane_trim` — ou seja ele é da mesma classe que o cabeçalho da
`SPEC_reescrita_dos_comentarios_com_nomes_do_alvo.md` **autoriza por escrito**
(`hardness`, `normal_radius_factor`, `tip_scale_x`, …), e a nossa
`ph2d-sculpt3d/src/brush.rs` já cita `tip_scale_x` ao abrigo dessa mesma
cláusula.

⛔⛔ **A janela I NÃO decide isto.** A triagem da parede é do **R**, e o que esta
secção entrega é a medição para ele: *ou a vassoura é mais larga que a regra que
implementa, ou a regra mudou e a autorização do cabeçalho precisa de ser
revista.* Renomear seria uma mudança de **produto** (um `NodeId` **hasheado** e
uma chave de i18n), e fazê-la sem esse veredito é caro e reversível ao contrário.

---

## §41 — ⛔⛔ O portão de fecho apanhou **QUATRO** vermelhos, e o mais instrutivo é uma **classificação por NOME**

Corrida: `bash scripts/nextest-impacted.sh` — **`15 240` testes, `15 236` verdes,
`4` vermelhos**, a `load 123` no pico do fan-out.

| vermelho | espécie | veredito |
|---|---|---|
| `measure_normals_parallel_speedup` (`ph2d-mesh`) | **flake de recurso sob fan-out**, já listada no `CLAUDE.md` §5.0 | confirmada |
| `no_tofu_glyphs_in_ui_strings` (`ph2d-editor-core`) | meu, trivial | curado |
| `the_edge_target_comes_from_the_piece_never_from_the_brush` (shell) | meu, **estrutural** | curado na RAIZ |
| `the_refinement_is_off_by_default_and_the_guard_is_the_first_question` (shell) | meu, **estrutural** | curado, e o gate ficou **mais forte** |

### §41.1 — A flake, com a régua ao lado

Ela leu `ganho 1,36×` contra a barra, **a `load 123`** (o fan-out de 15 mil a
esvaziar). Sozinha, com a carga impressa ao lado de cada corrida (§5.0: *a régua
que desmente a flake não pode ser a própria flake*):

| corrida | `load` | serial | paralelo | ganho |
|---|---|---|---|---|
| no portão | **123,56** | `13,166 ms` | `9,707 ms` | **`1,36×`** ✗ |
| 1 | 12,98 | `7,399 ms` | `1,108 ms` | `6,68×` ✓ |
| 2 | 13,07 | `7,923 ms` | `0,943 ms` | `8,41×` ✓ |
| 3 | 13,07 | `7,509 ms` | `0,995 ms` | `7,55×` ✓ |

**3 de 3 verde**, e o discriminador aparece nos dois lados: sob fan-out o
*serial* inflou `1,7×` e o *paralelo* **`9×`** — é o escalonador a não ter
núcleos para dar, não uma lei que mudou.

### §41.2 — ⛔⛔⛔ O ARNÊS DE TESTE tinha nome de PRODUTO, e a varredura da família leu-o como produto

**Os dois vermelhos do shell têm UMA causa.** O `sculpt_src()` — que alimenta
todos os arch-gates da família — junta os `.rs` da crate e **exclui os
`*_tests.rs`**, com a razão escrita lá desde sempre (*um gate que afirma AUSÊNCIA
passaria a ler o texto dos próprios testes*). O `censo_dos_knobs_arnes.rs`, que a
§39 cortou do censo por tecto de LOC, é compilado **só** sob `cfg(test)` (quem o
declara por `#[path]` é o `censo_dos_knobs_tests.rs`, que já está de fora) — e
**não acaba em `_tests.rs`**, logo entrou na varredura como produto.

As duas leituras que isso produziu são diferentes, e é isso que as torna úteis:

1. **`edge_target_for_mesh` lido `2` onde a lei diz `1`.** O arnês chama-o de
   propósito (ele reproduz a ordem do produto), e o gate conta chamadas no
   cluster para proibir *a segunda resposta à mesma pergunta*. `left: 2, right: 1`
   — **um gate certo sobre uma população errada**.
2. **O motor saiu do corpo do `refine_for_dab`.** O `expect("e só então chama o
   motor")` reprovou porque os dois motores vivem agora na porta
   `passe_nos_motores`. ⭐ *É o modo de falha BOM de mover código* (§5.0: **a que
   fica verde é a que se leva para o `main`**).

⭐⭐ **A cura é a CLASSIFICAÇÃO, e ela passa a ser DERIVADA:** *o que um ficheiro
de teste declara por `#[path]` é código de teste*, transitivamente
(`declarados_por_um_teste`). Não é o sufixo do nome — é a **declaração**, que é a
mesma coisa que o compilador usa. É a lei que o §5.0 já cobra dos censos que
varrem por **prefixo de nome**, um nível acima; a única sorte aqui foi a falha
ser **barulhenta** em vez de muda.

⚠️ **E ela tem guarda para a direcção PERIGOSA.** Excluir a mais é **mudo**: o
gate deixaria de ver um ficheiro que ship e ficaria verde por vácuo. Por isso um
nome que um ficheiro de **produto** também declare é recusado em voz alta.

⭐⭐ **Prova de mutação** (a fila da varredura nasce vazia ⇒ a exclusão não faz
trabalho): `the_edge_target_comes_from_the_piece_never_from_the_brush` **sangra**.

### §41.3 — ⭐⭐ E o gate da guarda ficou **mais forte** do que era

A premissa da 1.ª redacção morreu (o motor mudou de casa) e a reescrita está no
diff, com a **metade que a extracção CRIOU** e que antes não tinha onde existir:

- em `refine_for_dab`, o `self.dyntopo.armed` precede a chamada à porta;
- **e os motores são alcançáveis SÓ pela porta** (`refine_in_sphere(` e
  `collapse_in_sphere(` têm **uma** ocorrência cada no cluster, dentro dela).

Sem a segunda metade a guarda seria contornável: bastava um chamador novo do
motor para o passe correr desarmado, com a asserção de ordem **verde** por cima
de um produto errado. ⇒ *a frase que o gate sempre prometeu — «quem esquecer a
pergunta herda a resposta certa» — só agora é uma propriedade e não uma
intenção.*

### §41.4 — O `clippy --all-targets` e o argumento que sobrava

Cinco avisos, todos desta jornada: um `⇒` fora da fonte ASCII num literal, um
`let` devolvido a seguir, três blocos de `use` que o corte da §39 deixou mortos —
e **um que era desenho**:

⛔ **`passe_nos_motores` tinha `8` argumentos contra o tecto de `7`.** A cura é
agrupar, **nunca um `allow` por cima do aviso**: os três buffers (`remap`,
`births`, `region`) são **um** conceito — o rascunho do passe —, e os **dois**
chamadores já os seguravam juntos (a cena em campos `dyn_*` para o caminho quente
não alocar; o censo na mesma linha). Hoje são o `Rascunho<'_>`, e a porta fica em
`6`.
