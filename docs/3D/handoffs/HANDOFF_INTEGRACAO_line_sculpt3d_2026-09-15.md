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

> ⛔⛔ **LEIA O §42 ANTES DE IMPLEMENTAR DAQUI.** Esta secção continua correcta
> sobre a LEI — e a feature que ela cura foi **RETIRADA no mesmo dia, por ordem
> do dono**. O que fica aqui é o diagnóstico (que vale, e é por isso que não foi
> apagado); o que já não existe é o `Ctrl` deste pincel.


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

⭐⭐ **E ela reprovou OUTRA VEZ no portao do §42, com as TRÊS assinaturas da
família completas nesta jornada:**

1. **zero linhas de diff** — `git diff --stat` das seis waves de hoje sobre
   `crates/ph2d-mesh/` devolve **vazio**;
2. **o mesmo binário passou numa varredura e reprovou noutra** (a do §41 fechou
   `15 240/15 240` com ela VERDE; a do §42, sobre uma árvore que difere só em
   ficheiros de outra crate, deu-a vermelha);
3. verde sozinha com a carga baixa.

⚠️⚠️ **E o dado NOVO desta jornada é de onde vinha a carga: de OUTRA LINHA.**
A confirmação a `load 45` deu `2` reprovações em `3` — e o `ps` mostrou um
`nextest -E rdeps(ph2d-app-painter) + rdeps(ph2d-app-vec) + …` a correr noutra
worktree, mais um `cargo check -p ph2d-topdown`. ⇒ *o fan-out que quebra estes
gates não é só o da PRÓPRIA varredura: numa workstation com linhas paralelas
ele atravessa as árvores*, e a espera por *«máquina calma»* pode nunca chegar
(§5.0: *mede-se o MÍNIMO de N corridas com a mediana ao lado*).

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

---

## §42 — ⛔⛔⛔ O `Ctrl` do **Scene Project** SAI por ordem do dono, e isto é a RECUSA REGISTADA

Veredito dele depois do smoke da `=45`:

> *«Não vi utilidade na feature Scene Project + CTRL. Melhor retirá-la e
> documentá-la como indesejada.»*

(No mesmo report: **`Density Smoke OK`** — aquela metade está aprovada.)

### §42.1 — ⚠️ Retirar o GESTO retira a CAPACIDADE, e isso mediu-se ANTES de cortar

A pergunta que decide a forma da cura é *«o Ctrl é a única porta?»*:

| porta | medição |
|---|---|
| gesto | `scene.brush.invert = ctrl` no pen-down — **o único escritor de `Brush::invert` no produto inteiro** |
| painel | **nenhum** controlo *Add/Subtract* para este verbo |

⇒ **sim**. Logo não há a leitura branda *«tira-se o atalho e a capacidade fica»*
— tirar o gesto apaga a feature. E com o verbo fora do `Verb::honours_invert`, o
`sign` do `stroke_target` fica preso em `+1` para sempre: **um parâmetro que só
pode valer uma coisa é um órfão**, e *a cura de um órfão é apagar* (§5.0), não
deixá-lo vivo e inalcançável. ⇒ o `sign` desapareceu da assinatura do
`projectar::alvo_do_vertice`.

⚠️ **Divergência DECLARADA:** a referência **tem** a capacidade. Nós não a
queremos — e isso fica escrito, com o número ao lado, em vez de a ausência se
ler como incapacidade.

### §42.2 — ⛔ O que a decisão CUSTOU, contado

| grandeza | antes | depois |
|---|---|---|
| corpus vivo do oráculo | `16` reconstrutíveis | **`14`** |
| catraca `VERDE_N` | `15` de `16` | **`13` de `14`** |
| gate de unidade | `a_inversao_nega_a_translacao_e_nao_vira_o_raio` | **apagado** (o sujeito deixou de existir) |

⭐⭐ **As duas fixturas saíram VERDES** (`2,384e-7` cada, contra a barra de
`2e-6`). *Uma fixtura que sai por DECISÃO e uma que sai por DERROTA leem-se
igual numa lista* — o que as separa é a frase e o número ao lado, e por isso elas
vão para uma lista **nomeada** (`FORA_POR_DECISAO_DO_DONO`) e não para o esquecimento.

⛔⛔ **E a catraca a DESCER é o ponto perigoso deste commit.** `15 → 13` lê-se
como regressão e **não é**: é a população a encolher. O que os separa é o
denominador (`13` de **`14`**) e o gate novo. *Escrever `13` sem escrever porquê
seria a catraca a virar LICENÇA no sentido contrário: a próxima pessoa
afrouxaria o número e chamar-lhe-ia história.*

### §42.3 — ⭐⭐ O gate mede o BARRO, não o predicado

`o_ctrl_saiu_do_projectar_e_o_corpus_diz_quais_fixturas_isso_custou` tem duas
metades e **prova de mutação 2 de 2, uma por metade**:

1. `Verb::SceneProject.honours_invert()` é `false` — e a mensagem manda **repor**
   as duas fixturas e subir o `VERDE_N` no dia em que a decisão mudar (a metade
   de **obsolescência**, sem a qual a lista de exclusão vira licença).
2. ⭐ **correr a fixtura com `invert = true` dá o mesmo bloco de vértices, ao
   bit.** *Um predicado é um resumo do produto, e um resumo pode estar à frente
   dele* — a mutação que lê `brush.invert` dentro da lei passa a primeira metade
   e sangra nesta (`1,000e0`).

⚠️ **E a primeira tentativa de mutação NÃO ENTROU** (o `fmt` tinha reflowado o
`match` e o filtro casou `0×`): *uma mutação que não entra lê-se exactamente como
uma que sobreviveu* — o `assert` de contagem é o que separa as duas.

### §42.4 — ⛔⛔ E a lei REFUTADA de manhã tinha uma SEGUNDA cópia, que sobreviveu à cura

Ao remover a entrada, apareceu que o `brush_verb_predicados.rs` ainda afirmava
*«PROJECTAR honra o Ctrl, e ele vira o RAIO»* — **a lei que o §35 refutou nesta
mesma jornada**. A cura da manhã corrigiu o `projectar.rs` e não a cópia.

⚠️ *Uma lei escrita em dois sítios ainda não é uma lei — e a cópia que ninguém
relê é a que envelhece.* Ela só apareceu porque a remoção obrigou a olhar para
aquela linha; nenhum gate a media.

### §42.5 — O que fica escrito para quem reabrir

O mecanismo **não** se apaga com a feature (§5.0: *o que foi medido e rejeitado
não se reconstrói*). Ficam, no `alvo_do_vertice` e na nota que substituiu o gate
apagado: a tabela de duas colunas que separa as duas leis candidatas (com o alvo
abaixo elas são **indistinguíveis**, e só *«alvo acima, sem os dois sentidos»* as
separa), o número do oráculo (`1,415e-1 → 2,384e-7`), e o mecanismo da excursão
travada em `0,500000` que refutou a hipótese da fuga sem fim.

### §42.6 — E a cena deixou de ensinar o gesto

O roteiro da `=45` passou de **9 para 8 passos** — o passo do `Ctrl` saiu, os
seguintes renumeraram e o `DEU ERRADO SE` perdeu a cláusula correspondente.
*Uma cena que continuasse a mandar carregar em `Ctrl` ensinaria um gesto que não
existe, que é a espécie que o §5.0 chama de pior que uma cena ausente.*

---

## §43 — ⏳ O **TRIM** começou: o que a preparação MEDIU antes de existir uma linha de código

Ordem do dono (15/09): *«Creio que ainda não temos vários pincéis do blender: Vamos começar por
TRIM. Vá estudar o blender para implementar aqui.»*

⚠️ **Esta secção é toda do NOSSO lado.** A lei do alvo vive na
[`SPEC_trim_gesture.md`](../cleanroom/SPEC_trim_gesture.md), escrita por um subagente-E sob a
parede; o que está aqui foi medido nesta janela, que nunca viu o fonte do alvo.

### §43.1 — O substrato: o que já existe e o que é obra nova

| peça | estado |
|---|---|
| desfazer uma operação que muda a topologia | ⭐ **existe** — `StrokeUndo::Remeshed`, a troca simétrica, com **três** chamadores. Um corte seria o quarto |
| volta `malha → campo → malha` | ⭐ existe e ship (é o botão de remesh) |
| booleana de malha 3D | ⛔ **não existe** (a `ph2d-vec-boolean` é 2D; os outros `trim` do repo são `plane_trim` do raspador e `str::trim` em parsers) |
| gesto de caixa/laço no canvas da escultura | ⛔ **não existe** — obra nova |
| ⚠️ gesto de rectângulo sobre uma vista 3D | **existe no modelador** (`ph2d-app-field3d`), e ⛔ **não é o que o nome promete:** ele resolve *que objectos estão debaixo do rectângulo*, e o `subtracts` dele subtrai do **conjunto de selecção**, não de geometria. Serve de precedente de fiação, nunca de motor |

### §43.2 — ⭐⭐⭐ A decisão de arquitectura, e ela é MEDIDA dos dois lados

A sonda do §5 desta linha (`diag_o_preco_da_volta_por_campo`, commit anterior) e a §1 da espec
medem a mesma coisa por caminhos independentes, **e concordam**:

| | booleana | volta por campo (nossa) |
|---|---|---|
| relógio na escultura de `98 306` V | ⭐ **`67,88 ms`** | `171,7 ms` (res 128) |
| vértices de entrada preservados **ao bit** | `57 489` (`58 %`) | ⛔ **`0`**, em 6 de 6 células |
| vértices longe do corte | ⭐ `26 533` de `26 533` **intactos** | ⛔ `0` |
| contagem de saída | segue a ENTRADA | ⛔ segue a **resolução da grelha** |

⇒ **um corte por campo não é um corte: é um corte MAIS um remalhamento da peça inteira.** Uma
escultura tem densidade **autorada** — fino onde o artista trabalhou, grosso onde não — e a volta
por campo devolve-a uniforme. *A rota barata é `2,5×` mais lenta E destrói o que a outra preserva*,
o que é raro: normalmente há uma troca, e aqui não há.

### §43.3 — ⭐⭐ O motor NÃO se escreve: a porta está aberta, e eu verifiquei-a com os meus olhos

A triagem parte em duas e só **metade** paga clean-room (espec §1.4): a **lei do gesto** é T2; o
**solucionador de omissão** é uma biblioteca externa **permissiva**, logo T0.

Medido nesta janela, fora do repo (`cargo add` + `cargo build` num crate de rascunho):

| facto | medido |
|---|---|
| resolve de crates.io | ✅ `manifold-rust 0.13.1`, **12 pacotes** |
| constrói | ✅ **`16,66 s`** a frio, **sem `cc`, sem `cmake`, sem rede** |
| licença | ✅ **`Apache-2.0`** — lida no **ficheiro `LICENSE` do artefacto descarregado**, não numa página (é a distinção que o R-pré cobrou) |

⛔⛔ **E há um portão NOSSO que isto reprovaria, e ninguém tinha perguntado:** das 12 dependências
transitivas, **onze** são `MIT`/`Apache-2.0` e uma — `clipper2-rust 1.1.0` — é **`BSL-1.0`**. O
nosso [`deny.toml`](../../../deny.toml) **não tem `BSL-1.0` na lista geral**: ela é concedida
**por-crate** a `error-code` e `clipboard-win` (os dois puxados pelo `arboard`, do clipboard).
⇒ sem uma excepção nova, o `./scripts/ship.sh` reprova — e ele é o **último** portão antes do push,
que é o sítio mais caro para descobrir isto. A cura é precedida e barata (uma entrada
`[[licenses.exceptions]]` que **nomeia quem a puxa**, como as duas que já lá estão).
⏳ Por medir: se o `clipper2-rust` é evitável por feature (ele é clipping 2D, e o corte é 3D).

### §43.4 — ⛔⛔ O modo de falha da dependência é SILENCIOSO, e é a lei que o produto tem de escrever

Da espec §1.5.6, medido nos três motores: com malha **aberta** a operação devolve malha **VAZIA**
e o estado do **resultado** diz *«sem erro»*. O sinal verdadeiro está no estado da **ENTRADA**.

⇒ **quem verificar só o resultado entrega uma escultura APAGADA.** A lei: verificar a entrada
**antes** de operar e recusar **em voz alta** — e isto entra na família de recusas que a §33 desta
linha construiu (`recusa::Entradas`), que já é derivada dos predicados `precisa_d*` do motor.
⚠️ E o motor «robusto» **não** resgata malha aberta: ele aceita *soup fechada* (suja), que é outra
coisa. *É a distinção entre «suja» e «aberta», e ela separa a promessa do que se mede.*

### §43.5 — O estado do protocolo, e porque ainda não há código

| passo | estado |
|---|---|
| triagem de licença | ✅ T2 (lei do gesto) · **T0** (motor de omissão) |
| patente | ✅ 4 examinadas, todas expiradas/lapsadas; nenhuma viva alcança o método |
| espec + ledger + vassoura + fixturas | ✅ entregues |
| **R-pré (atestado)** | ⏳ **em curso — 1.ª passagem reprovou com `9` achados, curados; a 2.ª corre agora** |
| código de produto | ⛔ **zero linhas, de propósito** |

⚠️ **A 1.ª auditoria vale por si e fica registada**, porque os dois bloqueantes dela são leis desta
casa a repetirem-se: (1) **o instrumento que audita tinha um buraco e a espec estava a usá-lo** — a
cura anterior apanhava frases partidas em linhas mas não frases com **ênfase markdown no meio**, e
era exactamente aí que estava a única citação que restava (*é a terceira cegueira do mesmo tipo
nesta obra, e a espécie é sempre **alcance**, nunca padrões*); (2) uma afirmação sobre **quatro**
comportamentos de um cruzamento `2×2` cujos estados alcançáveis são **três** ⇒ implementá-la
shiparia **um braço que gesto nenhum atinge**, que é o «dreno de um braço só» do `CLAUDE.md` §5.0.
⭐ E o que a auditoria **ilibou** é o que dá confiança ao resto: os `292` identificadores internos do
ficheiro do alvo foram extraídos mecanicamente e varridos contra a espec — **zero fugas** —, e a
decomposição em fases dela **não** é a do alvo.

---

## §44 — ⭐⭐⭐ O **BOX TRIM**: o report da face cortada tinha uma causa que não era a triangulação

> **Report do dono, com foto** (2026-09-15): *«eu estava me referindo ao trim pincel de
> escultura e não a esse trim. Isso que vc tentou criar é o Box Trim, mas seu resultado não
> ficou legal. Tente melhorar. Veja que o remesh da face que vc cortou fica ruim demais.
> Coloque como Box Trim. Se não conseguir melhorar vamos passar para o pincel trim»* — mais
> uma segunda foto: **a saída do Box Trim da referência**, *«resultado superior»*.

### §44.1 — A medição veio ANTES de tocar em código, e desmentiu a leitura da foto

A foto mostrava uma mancha escura com estrias a irradiar de um ponto, e a minha 1.ª leitura foi
*«um leque de triângulos finos»*. A sonda `diag_a_tampa_do_corte` (em `ph2d-mesh-bool`) diz outra
coisa:

| peça | `T` | aresta p50 | **face cortada** | aresta da face |
|---|---|---|---|---|
| `uv_sphere(24,32)` | `768` | `0,1357` | **`2` T** | `1,2000` |
| `uv_sphere(48,64)` | `3 072` | `0,0661` | **`2` T** | `1,2000` |
| `sphere_with_triangles(20k)` | `10 000` | `0,0347` | **`2` T** | `1,2000` |

⇒ **`1 385 ×` menos triângulos do que a densidade da peça** na última linha. *Não era a
triangulação que estava torta: era a face a não ter malha nenhuma.*

⭐⭐ **E a metade que a foto de facto mostra é o SOMBREAMENTO.** A normal de um vértice é a média
das faces que o tocam; com **dois** triângulos, **todos** os vértices da face são de borda ⇒ cada
um mistura o plano com a esfera, e **não existe um único ponto da face que seja pintado plano**.
Uma face plana sombreada como curva é o aspecto derretido do report — e é por isso que a foto da
referência (onde a mesma face é coarse **e** nítida) parecia outra classe de resultado: o que
separa as duas não é a contagem, é a **normal**.

### §44.2 — A cura é a LÂMINA, nunca um pós-passe sobre o resultado

O que o motor devolve na superfície de corte é a tesselação da **PAREDE DO PRISMA** recortada pela
peça. ⇒ *um prisma tesselado dá um corte tesselado*, e ⭐ a propriedade que decide a arquitectura
desta linha — **longe do corte, nem um bit** — fica intacta **por construção**, porque nada toca a
malha da peça. (Um pós-passe de refino sobre a saída teria de a percorrer, e é exactamente o que o
gate `longe_do_corte_nenhum_vertice_se_move_um_bit` existe para proibir.)

`ph2d_trim::Resolucao { Minima, Ate(f32) }`, 7.º argumento da `prisma`. O `Minima` é a saída de
sempre **ao bit** (os 11 gates da forma ficaram verdes sem uma linha de alteração) e fica como rota
de bissecção.

**Resultado, de ponta a ponta** (`a_face_que_o_corte_deixa_tem_a_densidade_da_peca`, que corta de
verdade — a `ph2d-trim` ganhou a `ph2d-mesh-bool` como **dev-dependency**, e só como isso):

| | face cortada | aresta dela | pintada PLANA |
|---|---|---|---|
| lâmina mínima | `436` T | `1,5419` | **`0,0 %`** de 144 vértices |
| **lâmina à densidade da peça** | `11 380` T | **`0,0346`** | **`83,7 %`** de 1 336 |
| *a peça* | — | `0,0347` | — |

⚠️ **A contagem grossa não é `2` aqui e isso é o motor a trabalhar:** a parede é recortada pela
esfera, logo a fronteira dela é uma curva com muitos vértices e o motor tapa-a com um leque. ⇒
*contar triângulos não distingue uma face tesselada de uma face com um leque grande* — é a
**ARESTA** que decide, e a **fracção pintada plana** é a régua que corresponde ao que o dono vê.

### §44.3 — ⚠️ O PREÇO está medido, e ele diz o contrário do que se temia

`--release`, `Op::Subtrair`, o mesmo anel:

| peça | lâmina mínima | lâmina à aresta da peça | tampa |
|---|---|---|---|
| `10 000` T | `15,9 ms` | **`23,5 ms`** | `2 → 2 450` |
| `50 176` T | `85,3 ms` | **`128,7 ms`** | `2 → 12 482` |
| `199 809` T | `413,8 ms` | **`545,0 ms`** | `2 → 49 928` |

⇒ **o custo é da PEÇA e não da lâmina** (`+32 %` a `+51 %`), e a contagem da lâmina é
`perímetro/alvo × profundidade/alvo`, isto é, ela escala com a peça sozinha. ⛔ **Não há tecto de
qualidade a inventar.** O único tecto (`TECTO_DE_TRIANGULOS = 200 000`) é a rede contra um `alvo`
degenerado de quem chama, ele **nomeia o recurso** (o relógio do motor: `14 700` T de lâmina custam
`20,1 ms` e `927 408` custam `523,6`) e **renormaliza o alvo para cima — nunca recusa o gesto**.

### §44.4 — A régua do alvo é a porta que a casa já tinha

`ph2d_mesh::edge_for_tri_count(surface_area, tris)` — a **mesma** do alvo de topologia, ancorada na
**ÁREA**. ⚠️ E ancorada nela de propósito: *a densidade não é propriedade da vista nem da caixa*, que
é a lei que este módulo já pagou duas vezes (o `Quad Size` absoluto do botão, e o alvo de topologia
que variava `4,9 ×` com o zoom). Medida contra a mediana das arestas nas fixturas da casa, concorda
a **`1,03 ×`–`1,10 ×`** e custa uma passagem sem alocação, contra ordenar `3 × T` números.

⚠️ **Triângulos, não faces** (`verts().len() − 2`): um quad conta dois, e ler `face_count` daria um
alvo `√2 ×` grosso numa peça ainda não triangulada. Há mutação a prová-lo.

### §44.5 — ⛔⛔ A DÍVIDA das tampas, declarada no doc E afirmada por gate

O anel adensado é **obrigatório** na tampa: um ponto que o adensamento põe numa aresta do anel
pertence às paredes, e se a tampa continuasse a ir de canto a canto nascia uma **junta em T** — que
o motor lê como superfície **ABERTA** (medido na 1.ª sonda deste trabalho: seis grelhas sem vértices
partilhados devolveram `LaminaAberta`, e uma lâmina aberta não corta nada). O **interior** da tampa,
esse, fica com a triangulação grossa mais um leque: medido, a diagonal de uma tampa de `1 × 1` com
alvo `0,4` mede `1,414`.

⭐ **Não toca o produto:** com `Profundidade::DaPeca` as tampas ficam **FORA da peça** por construção
(o enchimento da `faixa` afasta-as), logo nunca aparecem na superfície cortada. ⏳ **Dívida nomeada
para o dia em que a `DoCursor` chegar à interface** — ali a tampa **é** a face do corte. ⚠️ E o gate
`nenhuma_aresta_das_paredes_passa_do_alvo` tem a metade que **AFIRMA a dívida**: quem triangular a
tampa com pontos interiores vê essa metade reprovar, e a dívida sai do doc **no mesmo diff**.

### §44.6 — ⚠️ Uma mutação SOBREVIVENTE escreveu um gate que o doc já prometia

Trocar o centro do leque da tampa por um **canto** passava a suíte inteira. Os pontos de uma aresta
subdividida são **colineares** com os cantos dela ⇒ um leque a partir de um canto emite triângulos de
**área ZERO** — que não abrem a malha e não mudam o volume, logo o gate de fecho e o de volume ficam
os **dois** verdes sobre eles. *Uma face sem área é uma face sem DIRECÇÃO*
(`ph2d_mesh::face_normal` devolve o vector nulo), e entregá-la a um solucionador exacto é pedir a
resposta que ninguém mediu. ⇒ `nenhuma_face_do_prisma_tem_area_zero`.

**Prova de mutação: 8 de 8 sangram** (adensamento inerte · filas em `1` · a junta em T · o leque do
canto · o tecto sem renormalizar · o alvo a contar faces · o corte a voltar ao `Minima` · a lei a
sair do sítio).

### §44.7 — O NOME, e a cena que a peça certa torna honesta

`Forma::label()` → **`Box Trim`** / **`Lasso Trim`**, num sítio só, com gate nas duas metades (o
rótulo certo · o teclado a **ler** o rótulo em vez de escrever o nome à mão). ⚠️ São os nomes da
referência de propósito: o dono chamou-a *Box Trim* antes de eu lhe ter dado nome nenhum, e um
artista que venha de lá procura por estes.

**Cena `=46`** (`scenes::CENAS` 45 → 46), e a peça com que ela abre é um **número**:

| peça | triângulos | o corte custa |
|---|---|---|
| cubo subdividido `3×` | `768` | `1,3 ms` |
| esfera `20 k` | `19 800` | `23,8 ms` |
| **esfera `50 k`** (a `=46`) | `49 612` | **`58,4 ms`** |
| `sculpt_sphere` (o default do módulo) | `196 608` | **`380,6 ms`** |

⇒ no default — **que é onde o dono testou** — ele larga o rato e espera mais de um terço de segundo,
que é à letra o report que esta família já pagou uma vez (*«meio travado»*, 14/09, com a mesma causa:
a cena a fabricar a peça pesada). ⚠️ **E o extremo barato também não serve:** com `768` triângulos a
face cortada sai com uma dúzia deles e o passo (3) do roteiro — *que manda olhar para a malha dela* —
não teria o que afirmar. O gate afirma as **duas** cercas.

### §44.8 — ⛔ Um achado de SWEEP que não é desta jornada: `NoError`

A `VASSOURA_blender-trim` acusa `NoError` em `crates/ph2d-mesh-bool/src/lib.rs`. **É um falso
positivo sobre a API pública de uma dependência permissiva:** `NoError` é variante do
`manifold_rust::types::Error` (Apache-2.0, `types.rs:145`), o motor que esta porta **liga**, e a
linha acusada é `m.status() == manifold_rust::types::Error::NoError`. ⚠️ **Pré-existente** — `git
diff HEAD` naquele ficheiro é **vazio** nesta jornada; ele entrou com o commit `d4a4a4a67`.

⇒ **Isenção NOMEADA, nunca silêncio** (a lei que o cabeçalho do próprio sweep escreve: *uma entrada
que dispara sobre uso lícito treina quem corre o sweep a ignorar achados*). ⛔ **A triagem é do R,
não da janela I** — ou a entrada da vassoura é mais larga que a regra que implementa, ou ela precisa
de revisão. As outras **seis** vassouras fecham limpas sobre os caminhos tocados.

### §44.9 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **`Resolucao::Minima` não é uma rota morta** — ela é a saída de sempre **ao bit**, é o que os 11
   gates da forma medem, e é a rota de bissecção.
2. **O adensamento não muda a FORMA do volume**, e isso tem gate (`adensar_nao_move_a_superficie`,
   volume e caixa nas duas resoluções, nos dois modos de parede): as paredes são **regradas** e as
   tampas **planas**, logo todo ponto novo é interpolação *na própria superfície*.
3. **O `TECTO_DE_TRIANGULOS` não é um tecto de qualidade** — o caminho do produto nunca lá chega.
4. **A `ph2d-mesh-bool` é `dev-dependency` da `ph2d-trim`, não dependência** — a `ph2d-trim` continua
   a não cortar, e é essa separação que permite gatear a forma do volume sem o motor.
5. **`83,7 %` não é uma barra frouxa:** a banda que sobra é o anel de **um triângulo** junto à aresta,
   que é exactamente o que faz o corte parecer nítido em vez de chanfrado.
6. **O corte continua a acontecer no LARGAR** e essa decisão não mudou — o que mudou foi quanto ele
   custa e o que ele deixa.

### §44.10 — ⏳ ABERTO

* O **interior das tampas** (§44.5) — acto de quem ligar a `Profundidade::DoCursor` à interface.
* A **linha** e a **polilinha** (espec §4.2): das quatro variantes da referência temos duas. A linha
  é a mesma máquina com um quadrilátero fabricado e o modo **forçado** a subtrair.
* Os outros **modos** (juntar · intersectar): o `Op` existe na porta e nenhum gesto o alcança.
* A **simetria** (espec §9): `N` booleanas, uma por passagem.
* O **chip no painel** — o corte está no teclado, e *uma tecla é alcançável mas não DESCOBRÍVEL*.
* ⛔ **O veredito do dono sobre a face cortada** é o que decide se esta wave fecha ou se a fila passa
  ao **pincel** de trim que ele nomeou.

---

## §45 — ⭐⭐⭐ O Box Trim vira **FERRAMENTA**, ganha o **círculo** e a **suavização do traço**

> **Ordem do dono** (2026-09-15, depois de aprovar a wave do §44 com *«Muito
> Bom»*): *«O laço merece um grau de suavização do traço. Crie o botão nos tools
> para box trim. Nos parâmetros botões box e circle (novo) e laço. Em laço um
> parâmetro para suavizar o traço.»*

### §45.1 — ⭐⭐ A decisão que tudo o resto segue: o corte é um **VERBO**

`Verb::BoxTrim` (o **33.º**; conte-o em `Verb::ALL`). ⭐ **A exclusividade com os
pincéis passa a ser por construção** — escolher um pincel desarma o corte e
escolher o corte larga o pincel, sem uma única regra escrita à mão. Um botão
separado ao lado da fileira precisaria dessas duas regras, e elas são exactamente
o que apodrece.

⇒ o campo `Trim::armado` **foi apagado**: *«está armado?»* passou a ser a mesma
pergunta que *«qual é o pincel na mão?»*, e mantê-lo seria a segunda resposta que
diverge no dia em que uma delas mudar.

⚠️ **O `L` continua a existir e mudou de significado:** ele **pega na
ferramenta** e, das vezes seguintes, roda entre as três formas — e **depois da
última devolve o verbo que interrompeu** (`Trim::verbo_anterior`, lido só pelo
atalho). *Um atalho que arma e não desarma deixa o artista preso à ferramenta que
ele espreitou.*

### §45.2 — ⛔⛔ O que um verbo novo obriga a decidir, e o que quase passou calado

O compilador acusou **3** `match` exaustivos. Os outros predicados têm ramo
`_ =>`, e **é ali que mora o risco**:

| predicado | Box Trim | porquê |
|---|---|---|
| `sem_lei_por_vertice` | **`true`** | herda de graça `writes_through_applicator = false`, `accumulates = false` e `a_forca_chega_ao_barro = false` |
| `mexe_na_topologia` | **`false`** (braço explícito) | ⚠️ o default `!anchors()` responderia **`true`** |
| `corre_sem_o_interruptor` | **`false`** | ver §45.3 |
| `o_auto_smooth_chega` · `a_lei_le_a_distancia_ao_cursor` | `false` | não há laço por-vértice nem cursor a que medir distância |
| `o_raio_chega_ao_barro` (**novo**) | `false` | ver §45.4 |

### §45.3 — ⛔⛔⛔ Uma resposta a servir DUAS perguntas, e ela divergiu no SEGUNDO membro

`corre_sem_o_interruptor` derivava de `sem_lei_por_vertice`. Enquanto o
`Density` era o único membro, as duas tinham a mesma resposta **por acaso** — e
são perguntas diferentes: *«este verbo tem lei por-vértice?»* e *«o passe de
topologia corre para ele mesmo com o interruptor desligado?»*. O Box Trim
responde `true` à primeira e **`false`** à segunda.

⚠️⚠️ **É a armadilha que este módulo já pagou três vezes ao contrário** (duas
respostas à mesma pergunta, que divergem no terceiro membro); aqui foi **uma
resposta a duas perguntas, a divergir no segundo**.

⛔⛔ **E o ARNÊS do censo guardava uma CÓPIA da derivação** (`let topologia =
b.verb.sem_lei_por_vertice()`). No dia em que o produto a separou, o arnês
continuou a correr o passe de topologia sobre um verbo sem dab — fazendo-o
*«acordar»* com **`704` unidades de desvio que eram trabalho do próprio arnês**.
*Uma cópia de uma derivação é uma bomba com o relógio do dia em que a original
mudar.*

### §45.4 — ⭐⭐ O CENSO DOS KNOBS MORTOS apanhou o verbo novo na primeira corrida

`[("Box Trim", "panel.sculpt3d.radius"), ("Box Trim", "panel.sculpt3d.falloff")]`.

* **RAIO** ⇒ porta nova `Verb::o_raio_chega_ao_barro`, e o painel **esconde** a
  pista. ⚠️ **O `Density` responde `true`** — ele não move um vértice e o raio
  dele decide **ONDE** a malha afina, que é a distinção que uma derivação de
  `sem_lei_por_vertice` apagaria.
* **CURVA** ⇒ não pode ser escondida (cerca de produto medida e gateada), logo
  leva a razão à vista — e uma razão **própria**, `CurvaInerte::OGestoNaoCarimba`.
  ⛔ Reusar a do `Density` diria ao artista *«o efeito é sobre a topologia»* com
  uma ferramenta de **corte** na mão: um rótulo que mente.

⚠️ E o Box Trim entra na catraca dos **ADORMECIDOS** (que tinha ido a zero no dia
anterior) **por LEI**: medir num dab um verbo que não tem dab é medir o programa
errado. *Sem saída prescrita, ao contrário da entrada do `Density`* — a lei dele
tem bancada própria na `ph2d-trim`.

### §45.5 — ⭐⭐⭐ A SUAVIZAÇÃO: uma mutação sobrevivente refutou a minha explicação

A lei é `ph2d_trim::suaviza` — **pares de passagens de Taubin** sobre o anel
FECHADO, buffer **duplo** (um Gauss-Seidel faria a saída depender de **onde o
anel começa**, que é propriedade da mão e não da forma).

⛔⛔⛔ **O `MU` era `−0,52` com um doc a afirmar que `μ = −λ` fazia as duas
passagens cancelarem-se. Uma mutação SOBREVIVENTE mandou medir, e a afirmação era
FALSA.** A composição de um par tem resposta `H(k) = (1 − λk)(1 − μk)` com
`k = 1 − cos θ ∈ [0, 2]`; com `μ` **mais** negativo que `−λ` aparece um termo
linear positivo ⇒ **`H > 1` nas frequências baixas**:

| `μ` | harmónica de ordem 3 | tremor que sobra | a área moveu | canto perdido |
|---|---|---|---|---|
| **`−0,50`** | **`−0,2 %`** | `2,3 %` | **`0,006 %`** | `2,45 px` |
| `−0,52` | **`+0,6 %`** | `4,5 %` | `0,160 %` | `2,01 px` |
| `−0,55` | **`+1,8 %`** | `9,4 %` | `0,391 %` | `1,29 px` |

⇒ `MU = −LAMBDA`, e o gate novo é
`a_lei_nunca_amplifica_uma_forma_que_o_artista_desenhou`. *Uma lei de suavização
pode errar por preservar de menos; nunca por CRIAR.*

### §45.6 — ⚠️⚠️ A FIXTURA corrigiu-se DUAS vezes antes de a lei ser julgada

1. **Não era periódica.** `sin(i · 2,3999)` não fecha em `i = n` ⇒ a fixtura
   trazia uma **descontinuidade real** no ponto onde o laço fecha, e o gate do
   anel fechado lia o resíduo dela (`0,1148 px`) como prova de que a lei tratava
   o anel como aberto. *Uma régua que acusa a lei sobre um defeito da própria
   fixtura não afirma nada.*
2. **Não era banda larga.** Com duas harmónicas altas o tremor é aniquilado em
   `8` pares e a curva do tecto fica plana — *uma entrada fácil demais faz a lei
   parecer melhor do que ela é*.

⛔ **E a RÉGUA corrigiu-se uma terceira vez:** ela media o desvio radial ao
círculo verdadeiro, e sobre uma entrada de banda larga lia **`97,8 %` de «tremor
que sobra» sobre uma lei a funcionar** — porque a maior parte daquele desvio é
uma harmónica BAIXA, que é **forma**. *Uma régua que chama tremor a uma feição
acusa o alisador de não destruir o desenho.* O que fica é a **rugosidade local**
(quanto cada ponto se afasta da média dos vizinhos), que é o que o olho lê.

### §45.7 — O TECTO das passagens, derivado da resolução do GESTO

| pares | tremor que sobra | a área moveu | canto perdido | CONTROLO: laplaciano puro |
|---|---|---|---|---|
| `8` | `38,5 %` | `0,04 %` | `1,61 px` | `1,93 %` |
| `32` | `27,9 %` | `0,02 %` | `2,45 px` | `7,43 %` |
| **`64`** | **`24,1 %`** | **`0,00 %`** | **`2,98 px`** | `14,29 %` |
| `128` | `20,6 %` | `0,02 %` | `3,59 px` | `26,54 %` |
| `192` | `19,1 %` | `0,03 %` | **`4,00 px`** | `37,04 %` |

⇒ a `192` o canto desloca-se **exactamente** o `PASSO_MINIMO_PX` — a distância
com que o laço guarda pontos —, logo dali para cima a lei apaga feição que o
traço ainda conseguia representar. `PARES_MAX = 64` deixa o canto em `2,98 px`
(três quartos dessa resolução), e **a folga é declarada**: a medição corre no
espaçamento MÍNIMO, e um arrasto rápido guarda pontos mais afastados.

⚠️ **A derivação atravessa a fronteira de duas crates** (a lei não sabe o passo
do gesto; o gesto não sabe as passagens da lei) ⇒ o gate vive onde as duas
constantes se encontram, com a metade que impede um tecto escolhido **por
baixo**. ⚠️ **Ela errou duas vezes antes de assentar:** a `2 ×` o canto ainda
cabe (`3,59 px`) e a `3 ×` pousa **exactamente em cima** da régua — *uma
comparação de `f32` no fio da navalha, que é o que um gate não pode ser*.

⛔ **A coluna do CONTROLO justifica a lei inteira:** o laplaciano puro encolhe a
área `14,29 %` contra `0,00 %`. Num contorno de corte isso é cortar **por
dentro** da linha que o artista desenhou.

### §45.8 — O CÍRCULO, e porque o gesto dele é centro-para-fora

`TrimForma::Circulo` (a forma vive no `Brush`, ao lado do `SmearMode` e do
`ProjectMode` — era a condição para ter chip). ⛔ **Ele não vem da referência**
(caixa, laço, linha, polilinha) — é pedido do dono, e é a mesma máquina.

⚠️ **Centro-para-fora e não canto-a-canto**, ao contrário da caixa: um arrasto
canto-a-canto dá uma **elipse** sempre que não for quadrado, e um controlo
chamado *Circle* que entrega elipses é a espécie de rótulo que mente. ⏳ Se o
dono quiser a elipse, ela é o outro gesto e merece o nome dela.

⭐ **A contagem de lados sai do ECRÃ, nunca escolhida:** a flecha de uma corda de
`n` lados é `r·(1 − cos(π/n)) ≈ r·π²/(2n²)`, e exigi-la abaixo de meio pixel dá
`n ≥ π·√(r/(2·flecha))` ⇒ o polígono é **indistinguível de um círculo no ecrã em
que foi desenhado**, e a contagem cresce com `√r` em vez de linearmente.

### §45.9 — ⚠️ TRÊS censos tiveram a premissa morta, e nenhum foi afrouxado

1. `the_second_pass_is_refused_for_three_different_reasons` — **terceira** morte
   da mesma premissa, no dia em que a segunda ainda estava fresca.
2. `the_basic_level_never_hides_the_two_knobs_every_brush_has` — a frase *«o raio
   é de TODOS»* era uma asserção **incondicional**, e *uma asserção incondicional
   é uma premissa à espera do primeiro membro que não couber nela*. As duas
   pistas passam a ser afirmadas pela porta do motor, com a população dos dois
   lados.
3. `a_lista_do_censo_cobre_os_knobs_incondicionais` — a lista dos
   sempre-visíveis ficou **VAZIA**. ⭐ **A cura não foi baixar o piso outra vez:
   foi corrigir a POPULAÇÃO** (os verbos que CARIMBAM). *A pergunta sempre foi
   «que knob todo PINCEL pinta?», e a resposta era `Verb::ALL` só enquanto todo
   verbo era um pincel.*

E o gate da shell `o_pen_down_toma_o_gesto_e_fotografa_o_acerto` trocou a agulha
`scene.trim.armado` por `Verb::BoxTrim` — o campo foi **apagado**, e é isso que
torna a exclusividade uma propriedade em vez de uma regra.

### §45.10 — Números, provas e cortes

* **Mutação: 12 de 12 sangram** (o par de Taubin · o anel aberto · o
  Gauss-Seidel · o tecto das passagens · os lados do círculo · o laço a ignorar a
  pista · o raio de volta a TODOS · a pista a ignorar a forma · os chips fora do
  `populate` · o arnês a derivar do predicado errado · e as duas do §44 que
  continuam vivas).
* **Dois tectos de LOC** curados por **CORTE** — `brush_verb.rs` (`704`, o nome
  da UI saiu para `brush_verb_label.rs`) e `stroke_target.rs` (`705`, as três
  contas de vector saíram para `stroke_target_vetores.rs`). ⛔ Nenhum no
  `FILE_OVERAGE_OK`.
* **Sweep clean-room limpo nas SETE vassouras** sobre os caminhos tocados.
  ⚠️ A isenção nomeada do §44.8 (`NoError`, API pública do motor Apache-2.0)
  continua de pé e continua a ser triagem do **R**.

### §45.11 — ⏳ ABERTO

* A **linha** e a **polilinha** (espec §4.2) — duas das quatro variantes da
  referência ainda faltam; a linha é a mesma máquina com um quadrilátero
  fabricado e o modo **forçado** a subtrair.
* Os outros **modos** (juntar · intersectar): o `Op` existe na porta e nenhum
  gesto o alcança.
* A **simetria** (espec §9): `N` booleanas, uma por passagem.
* O interior das **tampas** (§44.5), para o dia da `Profundidade::DoCursor`.
* ⛔ **A ELIPSE** — se o dono a quiser, ela é um gesto próprio e não um modo do
  `Circle`.

---

## §46 — ⭐⭐ As BORDAS do corte, e o círculo que media a coisa errada

> **Report do dono** (2026-09-15, com foto): *«circle ficou com baixa resolução
> nas próprias linhas do círculo. O algoritmo remesh produz bordas mais corretas
> que o algoritmo da Box Trim. Tente melhorar a topologia das bordas do corte.»*

Dois defeitos independentes, e a medição separou-os antes de qualquer código.

### §46.1 — ⛔⛔⛔ A costura: vértices DUPLICADOS que o motor emite

Medido na saída **crua** do motor (esfera de `49 612` T, aresta `0,0242`):

| | aspecto p90 | p99 | **MAX** | aresta mínima |
|---|---|---|---|---|
| cru | `3,45` | `25,7` | **`2 573 809`** | **`8,74e-9`** |

Uma aresta **`2,8` milhões de vezes** menor que a malha. *Um triângulo de aspecto
dois milhões não tem normal utilizável, e é isso que a borda mostra.*

⭐ A cura é soldar os coincidentes e colapsar as arestas curtas, **com duas
cercas que não são zelo**:

1. ⛔ **Uma aresta entre DOIS vértices antigos é da PEÇA.** Sem esta cerca a
   limpeza varre a malha inteira: medido, **`1 251` de `14 136`** vértices longe
   do corte deixavam de ser bit-idênticos — ela quebrava sozinha a propriedade
   que decide a arquitectura desta linha.
2. ⭐ **Quando um extremo é antigo, o sobrevivente é ELE.**

Com as duas: `MAX` **`2 573 809 → 33`** no caminho do produto, `14 136` de
`14 136` ao bit, bordo `0`, não-manifold `0`, volume `−2,2e-6` relativo.

**O limiar é o joelho de uma curva medida** (a coluna que decide é o `MAX`,
porque é o triângulo impossível que estraga a borda): `0,20` da aresta da peça —
o `MAX` cai `7,5 ×` entre `0,10` e `0,20` e **mais nada** entre `0,20` e `0,35`,
enquanto o volume paga `8 ×`.

⛔ **RECUSA MEDIDA:** o flip de arestas (`ph2d_mesh::relax_valence`) por cima
compra `640 → 632` piores-que-`20` e leva o `MAX` de `33` para `26` — pagando
por mudar a **ligação** da malha **longe do corte**. *Uma cura que muda a peça
inteira para ganhar oito triângulos não é uma cura.*

⏳ **Nomeado:** os `~632` que sobram **não são arestas curtas** — são cunhas
finas onde a curva de interseção passa rente a um vértice da peça, e nenhum
colapso as alcança. Curá-las mexeria na malha da peça.

### §46.2 — ⚠️⚠️ DUAS mutações sobreviveram, e as duas eram a FIXTURA

As duas cercas passaram a suíte inteira ao serem apagadas:

* O gate irmão `longe_do_corte_nenhum_vertice_se_move_um_bit` corre numa
  `uv_sphere(24,32)`, **cuja aresta mais curta está ACIMA do limiar** ⇒ sem nada
  para colapsar, apagar a cerca não é observável. ⇒ gate novo na peça que **tem**
  arestas curtas, com o piso de população a afirmá-lo.
* ⛔ **E a 1.ª redacção DELE ainda não apanhava a segunda cerca:** ela filtrava
  *«fora da lâmina»* por `x < 0,4`, e os vértices antigos que a limpeza de facto
  toca vivem em `x ≈ 0,64` — **fora do cubo pelas paredes de `y`/`z`**. São `14`
  colapsos de par misto, contados com a mutação na mão. *Um filtro que descreve a
  fixtura pela metade mede a metade errada da peça.*

⭐ E o `corta_cru` (só para gates) é a única maneira honesta de a régua ser uma
**afirmação**: *uma barra sobre a saída curada não diz que a cura fez alguma
coisa*. Controlo medido: `14 331` contra `256` na fixtura dura.

### §46.3 — ⛔⛔ O círculo: uma lei certa sobre o que promete, a prometer a coisa errada

A contagem de lados saía da **FLECHA** — a distância da corda ao arco — exigida
abaixo de meio pixel: `n ≥ π·√(r/(2·flecha))`. A `r = 250 px` são **`50` lados,
cada um com `31 px` de RETA no ecrã**. *A flecha é sub-pixel e o olho vê a quebra
de TANGENTE em cada vértice*, que é o que uma silhueta sombreada mostra.

⭐⭐⭐ **A segunda régua é o `passo_px` — a aresta da peça medida em pixéis — e ela
é GRÁTIS:** o prisma já subdivide cada segmento do anel até à aresta da peça
(`Resolucao::Ate`), logo os pontos vão ser criados de qualquer maneira — **só que
sobre as CORDAS**. Gerá-los sobre o CÍRCULO custa exactamente o mesmo e entrega a
forma certa.

⚠️ **As duas contam e fica a MAIOR** (numa peça grosseira o `passo_px` é enorme,
e aí é a flecha que impede o polígono de se ver), e o tecto nomeia o recurso (a
tampa é triangulada por corte de orelha, `O(n²)`).

⭐ A conversão mundo→ecrã vive na cena (`passo_do_corte_px`), medida à distância
do **centro da peça**: o acerto pode não existir (o gesto começa fora da peça,
espec §3), e a escala de um pixel varia tão pouco ao longo de uma peça que
medi-la no centro é o valor honesto.

### §46.4 — ⏳ ABERTO

* As `~632` cunhas finas da §46.1 — a cura mexeria na malha da peça.
* A **linha** e a **polilinha** (espec §4.2), os outros **modos**, a **simetria**
  e o interior das **tampas** continuam como no §45.11.

---

## §47 — ⛔⛔⛔ O espigão da borda era a CURA da §46 a criar lixo novo

> **Report do dono** (2026-09-15, com foto): *«Circle ficou bom! Borda melhorou
> mas não está perfeita»* — um **espigão** a sair da silhueta da peça.

### §47.1 — A causa, e a explicação que a medição derrubou

Com o corte a **SAIR pela beira** da peça a saída traz **`2` pares espelhados**:
o mesmo triângulo duas vezes, um virado ao contrário. Juntos encerram volume
**ZERO** — uma aba infinitamente fina, e é isso que o sombreamento desenha como
uma farpa.

⛔⛔ **A minha 1.ª explicação dizia que o MOTOR os emitia, e o gate escrito para a
provar REPROVOU NO PRÓPRIO CONTROLO:** a saída **crua** traz `0` almofadas em
todas as posições varridas. *Fundir dois vértices faz dois triângulos distintos
passarem a ter o mesmo trio* ⇒ **quem os cria é o colapso da §46.1**. A cura de
ontem abriu este defeito hoje.

⇒ *uma cura que cria uma segunda espécie de lixo tem de a varrer também*, e é
por isso que a limpeza ganhou um terceiro passo em vez de o colapso ser
afrouxado (afrouxá-lo desfaria o ganho medido: `MAX 2 573 809 → 33`).

### §47.2 — ⚠️⚠️ A FIXTURA não continha o fenómeno — a SÉTIMA vez neste módulo

* Com o círculo **CENTRADO**, a borda do corte vive a `|z| = 0,8` e **nunca
  encontra a silhueta** (que é o equador) ⇒ `0` almofadas. *A minha fixtura era
  a centrada.*
* ⛔ E a lâmina **GROSSA** (o cubo de seis faces da `ph2d-mesh-bool`) **não as
  produz em posição nenhuma** — varridas cinco: `0` em todas. Só o **cilindro
  tesselado a sair pela beira** o faz.

⇒ a régua vive na `ph2d-trim` (onde a lâmina real é construída) e varre **quatro
posições do centro**, com o controlo a exigir que **pelo menos uma** produza o
fenómeno. *Sem esse controlo o gate passa por vácuo, que é exactamente o estado
em que a régua estava antes do report.*

### §47.3 — ⭐ Descartam-se os DOIS lados, e a distinção é load-bearing

Eles não são «um triângulo a mais»: são um par que não descreve superfície
nenhuma, e guardar um deixaria uma aba de face única pendurada. É a mesma decisão
que a linha do quad remesh tomou com a almofada dela (`mirrored_cells`).

⛔⛔ **E uma MUTAÇÃO SOBREVIVENTE escreveu o gate que faltava:** trocar *«pares
ESPELHADOS»* por *«qualquer trio repetido»* passava a suíte inteira — nenhuma
fixtura do corte produz um duplicado de **mesmo enrolamento**. ⚠️ A distinção
decide o que é correcto: dois espelhados encerram volume zero e saem os **dois**;
dois com o mesmo enrolamento são a **mesma** face escrita duas vezes, e descartar
ambos **abre um buraco**. ⇒ fixtura sintética (tetraedro com uma face repetida)
com as duas metades, e *curar o segundo caso é outra lei, que não vive aqui*.

### §47.4 — A feature que atravessa a fronteira

`corta_cru` e `limpa_a_costura_relatando` atravessam a fronteira da crate por uma
feature **do tamanho exacto do que atravessa** (`test-support`, **dois** itens):
a régua do caminho real vive na `ph2d-trim`, onde a lâmina real é construída, e
**sem o lado *antes* ela não afirma que a cura fez alguma coisa**. ⚠️ Nada do
produto a liga.

### §47.5 — Números e cortes

* Varrido em **4** posições do centro: bordo `0` e não-manifold `0` em todas.
* Mutação **3 de 3**.
* **Dois tectos de LOC** curados por **CORTE** — `lib_tests.rs` da `mesh-bool`
  (`709`) e `lib_resolucao_tests.rs` da `trim` (`1 028`), os dois partidos entre
  **DENSIDADE** e **COSTURA**. ⛔ Nenhum no `FILE_OVERAGE_OK`.
* Portão `15 297/15 297`; sweep limpo nas **sete** vassouras.

### §47.6 — ⏳ ABERTO

* As `~640` **cunhas finas** (altura `2,4 %` da aresta) continuam: elas não são
  arestas curtas nem almofadas, e nenhuma das duas curas as alcança. ⛔ Curá-las
  mexeria na malha da peça.
* O resto da fila do §45.11 (linha · polilinha · modos · simetria · tampas).
