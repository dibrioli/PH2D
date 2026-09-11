# 06 — O PLANO do que falta no pincel de tecido

> ⚠️ **Leia isto ANTES de pegar qualquer item.** Cada linha traz o número que a define, o que já foi
> tentado e medido, e a pergunta exacta que a destrava — porque **onze** explicações já foram
> construídas, medidas e refutadas nesta linha, e reconstruir uma delas é trabalho pago duas vezes.
>
> A vitória e as réguas: [`05_a_vitoria_medida.md`](05_a_vitoria_medida.md).
> A espec clean-room (atestada): [`SPEC_cloth_brush.md`](../cleanroom/SPEC_cloth_brush.md).
> ⛔ **Quem implementa NUNCA abre o fonte do alvo** — as perguntas vão por subagente-E, pelo INBOX.

## §1 — O estado, em números

⭐⭐⭐ **`79` dos `86` traços do oráculo estão dentro da barra de paridade (`0,13`) pelo MÁXIMO — e
pela régua que a espec prescreve onde o máximo não julga (o `p95`), fica UM de fora em 86.** ⇒ o que
resta é o `plano_apertar_ponto_plano_local`, que é o regime §5.2-ter em que o próprio alvo inverte a
malha e a ORDEM decide: **decisão do dono, não lei em falta.** Os DEZ traços
por passo de empurrar/inflar reproduzem-se sobre a malha inteira e nos doze passos à RESOLUÇÃO DO
FICHEIRO** (`err_max ≤ 5,2·10⁻⁶`, contra os `0,2369`/`0,2446` de 06/09). *A barra de `0,13` deixou de
ser a régua desses traços: a lei ali é exacta.* — 07/09.

⭐⭐ **E o PRODUTO corre a mesma lei, com número:** o gate `o_produto_corre_a_lei_do_oraculo`
(`ph2d-sculpt3d/tests/`) constrói a malha do oráculo, corre o caminho dele **pela porta do artista**
(`SculptStroke::dab`) e reproduz **cinco** traços a `3`–`21 · 10⁻⁶`. ⛔ Até 07/09 nada ligava as duas
metades: a bancada provava a lei e o adaptador — a tradução `Brush → Pincel`, a ordem de visita, o
anel-1, o `δ` projectado — podia entregar outra coisa com a suíte inteira verde. **Três mutações do
adaptador passam pelos 363 testes da crate e morrem lá.**

⛔⛔⛔ **DOS OITO QUE SOBRAM, SÓ QUATRO SÃO DECIDÍVEIS PELA BARRA.** A emenda Q18 gravou a **banda de
realização** da esfera — quatro corridas da MESMA configuração do oráculo dão saídas diferentes —, e
posta na unidade da barra (`0,13 × o maior deslocamento`) ela diz:

| traço | barra em posição | banda | nosso erro | veredito |
|---|---|---|---|---|
| `esfera_agarrar_radial_dinamica` | `0,0307` | `0,0200` | **`0,0078`** | ⭐⭐⭐ **DENTRO da lotaria** (`0,39×`) — ver abaixo |
| `esfera_gancho_radial_dinamica` | `0,0220` | `0,0362` | `0,0431` | ⛔ **INDECIDÍVEL** — a barra está `1,6×` ABAIXO da lotaria |
| `esfera_expandir_radial_dinamica` | `0,0061` | `0,0218` | `0,0271` | ⛔ **INDECIDÍVEL** — `3,6×` abaixo |

⭐⭐⭐ **E o agarrar FECHOU no mesmo dia, com a única lei que faltava de facto: a área simulada do Grab
não segue o cursor, nem na área *Dynamic*.** A pista não foi a amplitude — foi a **contagem**:
movíamos `2123` vértices e o alvo move `1863`. Com a área fixa no pen-down movemos `1864`, e o erro
cai de `0,0427` para `0,0078`. ⚠️ **É o PAR que cura** (a pertença e a banda): só a banda dá `0,129`,
só a pertença não move nada, e as duas dão `0,033`. ⇒ INBOX **Q20**.

⚠️ **E o quociente que a espec publica mistura unidades** (`5×`–`26×` sai de dividir um erro
RELATIVO por uma banda ABSOLUTA); na mesma unidade ele é `1,19×`–`2,15×`. A leitura *«a lotaria não
os explica»* sobrevive, com uma ordem de grandeza menos margem. ⇒ INBOX **Q19**.

> A leitura de 06/09 dizia `31 de 56`; a de 05/09 dizia `29 de 56`. O corpus cresceu (`86` traços) e
> a contagem muda com ele — **conte-a com a [`sonda_da_paridade_com_o_oraculo`]**, nunca daqui.

⭐⭐⭐ **E a fila deixou de ser ordenada pelo TAMANHO do erro** (06/09, 3.ª sessão). O tamanho não diz
se falta lei: Gauss–Seidel não comuta, e parte do resíduo de alguns traços vive num regime em que a
**ordem** de resolução decide tanto quanto a lei (a família do §5.2-ter, provada no aperto). A sonda
[`sonda_do_chao_de_ruido`](../../../crates/ph2d-cloth/tests/it/oraculo_do_pincel.rs) corre cada traço
nas **duas** ordens e devolve o resíduo **em unidades do que uma ordem errada custa** — e essa razão
**inverte a fila que este plano tinha**:

| traço | erro | ordem errada | razão | leitura |
|---|---|---|---|---|
| `esfera_empurrar_radial_dinamica` | `0,3231` | `0,0952` | **`3,39`** | **Push** — lei em falta, e a ordem quase não o move |
| `plano_inflar_radial_local`(`_origem`) | `0,2533` | `0,1025` | **`2,47`** | **Inflate** — idem |
| `esfera_expandir_radial_dinamica` | `0,5574` | `0,2728` | `2,04` | |
| `plano_empurrar_radial_local`(`_origem`) | `0,2141` | `0,1153` | `1,86` | **Push**, o resíduo do plano |
| `esfera_inflar_radial_dinamica` | `0,3780` | `0,2414` | `1,57` | |
| `plano_gancho_radial_local_1passo` | `0,4161` | `0,2894` | `1,44` | Snake Hook |
| `plano_apertar_ponto_radial_local` | `1,3800` | `1,0454` | `1,32` | aperto — o §4 |
| `plano_gancho_radial_local_2passos`(`_origem`) | `0,3881` | `0,3854` | `1,01` | Snake Hook |
| `plano_apertar_ponto_plano_local` | `0,5457` | `0,7137` | `0,76` | aperto |
| `plano_expandir_radial_local` | `0,1918` | `0,6322` | **`0,30`** | **Expand — o MENOS estrutural do corpus** |

> ⛔⛔ **ESTA TABELA ENVELHECEU, e três linhas dela estão desmentidas pela medição de 07/09 — conte com a sonda, nunca daqui:** o `plano_expandir_radial_local` lê **`0,001`** (era `0,1918`; `190×`), e o Push e o Inflate — que a tabela põe no topo da fila — **fecharam**. ⇒ *a fila que este §2 ordena já não existe.* O que sobra dos `86` são **`7`**, e eles partem-se em DUAS coisas e nenhuma é «lei em falta»: **`5` são o regime de inversão** (os apertos — §4, e todos com o 1 passo e a força fraca a saírem **exactos**) e **`2` estão DENTRO da lotaria do próprio alvo** (o gancho e o expandir da esfera, `q ≈ 1` passo a passo na `sonda_de_onde_a_esfera_diverge`). *Não há, neste corpus, lei em falta que a barra saiba decidir.*

⇒ **Push e Inflate são os resíduos determinísticos** e vão à frente. O gancho de poucos passos, o
aperto e o Expand vivem onde a ordem manda, e apertá-los seria ajustar a NOSSA ordenação.

## §2 — A FILA, pela razão da §1

### ✅ FECHADO em 06/09 — o censo da ESFERA (§4.6) inteiro

As **seis** grandezas que degeneram num plano visto de frente estão todas respondidas deste lado:

| # | grandeza | estado |
|---|---|---|
| 1 | o deslocamento `δ` projectado no plano do ecrã | ✅ implementada (Q12, 2.ª sessão) |
| 2 | a normal da área (disco de meio raio, peso `3p²−2p³`) | ✅ implementada — Push `0,944 → 0,215` |
| 3 | **o centro da área** (a média das posições **puxadas para o cursor**) | ✅ **implementada agora** — os quatro traços de falloff de plano melhoram, dois entram na paridade |
| 4 | a normal do vértice sobre a malha deformada | ✅ a FORMA já estava certa (soma face a face, malha actual, normalizada no fim); o **peso** é a metade que a própria §4.6 declara aberta, e o corpus continua sem o decidir (`PH2D_PESO_NORMAL=uniforme`: `0,253 → 0,253`) |
| 5 | os dois baldes pelo sinal contra a vista | ✅ implementada com a #2 |
| 6 | a distância como **corda 3D**, nunca geodésica | ✅ já era — toda distância desta crate passa pelo mesmo `dist` euclidiano |

⛔ **Fechar o censo não «cura a esfera», e é isso que ele serve para dizer:** o que sobra nos traços
de esfera é o resíduo do **modo**, não da curvatura — o arrasto, que é o CONTROLO da §4.6, bate
(`0,092`), e os outros sete carregam exactamente o resíduo que o mesmo modo tem no plano.

### ✅ FEITO em 06/09 — o CENTRO DA ÁREA (§4.4)

O plano de queda deixou de passar pelo cursor. Ele passa pelo **centro da área**, que **não é o
centroide do disco**: cada vértice entra na média já puxado para o cursor por `1 − a`, com `a` a mesma
*smoothstep* da normal — ⇒ zero no cursor, cresce para a borda.

    plano_empurrar_plano_local        0,215 → 0,106   (entra na paridade)
    plano_agarrar_plano_local         0,180 → 0,124   (entra na paridade)
    plano_arrastar_plano_local        0,233 → 0,134
    plano_apertar_ponto_plano_local   0,613 → 0,546

⚠️ **A recusa de 06/09 não foi refutada — ela respondia a outra pergunta.** «O plano pelo centro da
área afasta-o» (`empurrar 0,944 → 1,250`) foi medido com um **centroide**, e o alvo não usa um
centroide; o plano pelo cursor é a aproximação de **primeira ordem** desta lei.
⚠️ E o disco é centrado na **localização do cursor do modo**, que no Agarrar fica no pen-down durante
todo o traço — com o disco a seguir o cursor que anda, aquele traço sobe a `1,094`.

### ✅ FECHADO em 07/09 — o PUSH e o INFLATE, e com eles TODO traço de área *Local*

⭐⭐⭐ **Eram DUAS leis, e a segunda não era do Push nem do Inflate: era do SOLVER.**

**(1) As normais que o gesto lê são as da superfície que o TRAÇO encontrou** (§4.2-ter). Dentro de um
traço o pincel deforma a malha e continua a ler as normais com que o traço começou. Só duas coisas as
lêem — a normal da área do Push e a normal por vértice do Inflate — e as duas obedecem.
`plano_empurrar_radial_local` `0,237 → 0,002` · `plano_inflar_radial_local` `0,245 → 0,008` ·
`esfera_empurrar` `0,343 → 0,066` · `esfera_inflar` `0,372 → 0,077`.

**(2) O Push CALA-SE quando a cova passa o disco de amostragem** (§4.2-bis (8)). O disco que decide se
a normal da área existe tem raio `R · 0,5` e mede-se contra as posições **de agora**; numa folha que o
próprio Push afundou chega um passo em que nenhum vértice está a menos disso do cursor ⇒ o gesto não
escreve aceleração nenhuma. ⭐ **E volta a disparar** quando o cursor avança para terreno pouco
afundado — o padrão de `plano_empurrar_radial_local_origem` é `2 3 4 5` e `10`. ⚠️ **Nós já o
fazíamos**: a nossa amostragem sempre mediu contra as posições actuais, e o padrão saiu certo nos
sete traços sem uma linha mudada. *O que faltava era a régua que o prova* — gate 40.

**(3) ⭐⭐⭐ E o que sobrava era o `φ` DA INTEGRAÇÃO, que não traz a banda.** A espec tem **dois** `φ` e
nós tínhamos um: o das cinco varreduras (§5.2) traz `w(p⁰)`, o da integração (§5.4) **não** — ali a
banda entra uma vez só, e só no termo da velocidade. Com um `φ` só, a retenção valia `banda²`.
⛔ **Invisível em três sítios ao mesmo tempo**, e é por isso que sobreviveu a tudo: na área *Global* a
banda é `1` em toda a malha; no termo da aceleração o factor extra vale exactamente `1` (a força corta
em `d ≥ R` e a banda só desce a partir de `2,875·R`); e no anel entre `2,875·R` e `3,5·R` — onde ele
morde — o erro é de `10⁻³`, três ordens abaixo da barra de `0,13`.
⭐ O CONTROLO que o denunciou: o `plano_arrastar_radial_local_origem`, que a espec §10.11 diz
reproduzir-se a `3,7·10⁻⁶`, errava `3,9·10⁻³` — e o mesmo traço em área *Global* já lia `2·10⁻⁵`.
*Uma paridade de `0,012` numa barra de `0,13` é um traço «que passa»; contra a resolução do ficheiro
ele é um traço errado por mil vezes.*

**(4) ⭐⭐ E o PESO DA SOMA POR FACE deixou de ser uma escolha: é medido.** A §4.6 linha 4 declarava-o
aberto (área · ângulo · uniforme). A fixture de dois traços lê a direcção do impulso do 2.º traço
directamente do oráculo, e a mediana do desvio de direcção dá **`1,7·10⁻⁵` com peso uniforme** contra
`6,6·10⁻⁴` com área — `38×`, e não é o chão do ficheiro (restringindo aos vértices que se movem mais
de `10⁻²`, o uniforme desce a `9,4·10⁻⁶` e a área **estaciona** em `3,7·10⁻⁴`). ⚠️ **O produto já
estava certo** (`ph2d_mesh::normals` normaliza cada face antes de somar) — era a BANCADA que somava
por área, logo ela media o produto por baixo.

Gates **39** (as normais são as do traço, três metades) · **40** (o silêncio do Push e o limiar
medido) · **41** (os dez traços à resolução do ficheiro) · **42** (a direcção é a normal da área, não
a da vista). A espec **revogou o gate 23**, que dizia o contrário sobre as normais.

### ✅ FECHADO em 07/09 — os SETE controlos que o motor tinha e o artista não

A tradução `Brush → Pincel` escrevia `Radial` **literal** na forma de queda e caía no
`Pincel::default()` para os outros seis. O corpus do oráculo tem fixture para **cada um**:

| controlo | o que o mede |
|---|---|
| *Force Falloff* (Radial · Plane) | os quatro `plano_*_plano_local` |
| *Simulation Limit* `L` | `plano_agarrar_radial_local_preset` (limite `5`) |
| *Simulation Falloff* `F` | a errata da banda (§2.2) |
| *Pin Simulation Boundary* | `plano_arrastar_radial_local_pino` |
| *Cloth Mass* | `*_massa2` (quatro traços) |
| *Cloth Damping* | `*_amort05`, `*_amort1`, `*_amort06` |
| *Soft Body Plasticity* | `plano_arrastar_radial_local_plast05` |

⚠️ **As sete omissões são as do CÓDIGO do alvo** (§8.1) ⇒ a tradução entrega exactamente o
`Pincel::default()` que a bancada corre, e o mundo pré-wave é **byte-idêntico**. O gate
`os_sete_knobs_do_tecido_chegam_ao_motor` tem as duas metades: cada knob fora da omissão move o pano,
e o neutro é o de antes.

⛔⛔ **E a wave achou que os chips de 06/09 estavam MORTOS SOB O PONTEIRO.** Os oito de *Deformation*
e os três de *Simulation Area* eram pintados, hit-indexados e roteados — e **não estavam no
`populate`**, logo o clique era descartado em silêncio. Nenhum gate o via: o
`every_painted_control_is_clickable_where_it_is_drawn` arma o **Crease**, e com outro pincel na mão a
fileira do tecido nem é desenhada. *A fixtura tem de conter o fenómeno* — a sexta vez que aquele
módulo o escreve, e a primeira em que a frase custou uma wave inteira de controlos.

### ✅ FECHADO em 07/09 — o pincel tem AGORA todos os controlos que o alvo oferece

Os **três** que faltavam do §8.1 fecharam no mesmo dia, e cada um por uma razão diferente:

| controlo | veredito |
|---|---|
| ***Normal Weight*** | ⛔ **NÃO EXISTE neste pincel** — as três fixtures (`0`, `0,5`, `1`) dão o mesmo bloco de vértices, linha a linha. Uma lei a menos para escrever, e duas linhas saem da espec |
| ***Persistent* + *Set Persistent Base*** | ⭐ implementado, e ele **satura** em vez de atenuar |
| ***Use Collisions*** | ⭐ implementado, e o sujeito estava lá: a cena de escultura guarda **várias peças** |

⛔⛔ **DIVERGÊNCIA DECLARADA na colisão:** a espec dá ao *cast* uma **espessura de raio** de `0,3` em
unidades de mundo e o `Mesh::raycast` desta casa lança um raio FINO — num colisor fino visto de
raspão o nosso passa e o do alvo apanharia. ⚠️ **Não há fixture de colisor no corpus**, logo o gate 53
declara-se de ESPEC (as cinco cláusulas verificadas por construção) e nada disto tem lado aprovado.

### ✅ FECHADO em 07/09 — a BASE PERSISTENTE, e o AGARRAR que o artista tinha estava a `1/11` da lei

⭐⭐ **A base persistente (§6.4) é a última lei do §8.1 que faltava.** Ela substitui o repouso em
EXACTAMENTE quatro leituras, todas na construção — o comprimento estrutural, o filtro de raio, o
teste e a força da âncora radial do Agarrar, e a condição do pino — e **não** toca nos alvos nem na
banda. O efeito medido é **saturação**: `0,169 → 0,306 → 0,415` sem base, `0,169 → 0,171 → 0,176`
com ela. ⚠️ *É preciso o TERCEIRO traço para o dizer* — com dois, `0,306` contra `0,171` ainda se lê
como amplitude errada.

⛔⛔⛔ **E ligá-la ao produto descobriu um defeito de `11,5×`:** a §4.3 diz que o Grab leva o `δ`
**acumulado desde o pen-down** e os outros sete o incremental; o `Dab::path` é o incremento **por
definição**, e o adaptador entregava-o aos oito. O Grab do artista movia `0,0147` onde a lei move
`0,1690`. ⚠️ **A bancada nunca o veria** — ela constrói o delta total no laço dela —, e o gate de
costura escrito nessa mesma manhã não tinha um traço de Agarrar na lista. *Cinco traços a `10⁻⁶` não
dizem nada sobre o sexto modo;* quem o apanhou foi a metade **anti-vácuo** dele.

⛔ **E o *Normal Weight* NÃO EXISTE neste pincel:** as três fixtures (`0`, `0,5`, `1`) dão o mesmo
bloco de vértices, linha a linha. Uma lei a menos para escrever.

### ⭐ 1.º — o SNAKE HOOK de poucos passos

**O que se sabe.** Os dois têm o traço de um passo **ao bit** e a ordem de resolução quase não os
move (razões `3,39` · `2,47` · `1,86` · `1,57`) ⇒ *falta lei, e ela não se esconde atrás da
não-comutatividade.* O perfil radial do `plano_inflar_radial_local_origem` diz que **não é
magnitude uniforme**: o pico está no mesmo sítio (`x = 0,516`) e a banda é idêntica, mas a nossa
deformação **sobra atrás** do traço (`0,0307` contra `0,0215`) e **falta à frente** (`0,0076` contra
`0,0481`, `84 %` abaixo). *A frente do oráculo alcança muito mais longe que a nossa.*
⏳ A pergunta está no INBOX como **Q14** — e ela pode ser a mesma resposta do gancho, porque as duas
são sobre a **resposta da rede**, não sobre o gesto.

**O que se sabe.** ⭐⭐ **O erro é função do TAMANHO de `δ` por passo**, no mesmo caminho e com o mesmo
pincel: `δ = 0,600 → 0,416` · `0,300 → 0,388` · `0,026 → 0,062`. E o perfil do traço de um passo
prova duas coisas: as duas caudas **decaem à mesma taxa** (`0,78` por célula, ao longo de 15 células)
⇒ a rede transmite igual; e o oráculo tem um **degrau de `4,24×` numa célula** a `0,45R`–`0,59R` do
centro, que a curva *smooth* não produz (nós caímos `2,23×`).
⏳ INBOX **Q14.2**.

### 2.º — o APERTO e o EXPAND, que vivem onde a ORDEM manda

Razões `1,32` a `0,30`. ⛔ **Não procure uma lei em falta aqui antes de o 1.º e o 2.º fecharem** — o
`plano_expandir_radial_local` erra `0,192` e uma ordem errada custa `0,632`, ou seja *o resíduo dele é
menos de um terço do que a nossa própria escolha de ordenação vale*. O aperto de força alta é o §4,
que é decisão do dono.

## §3 — ⛔ RECUSAS MEDIDAS — não as reconstrua

| o que foi tentado | o número que o matou |
|---|---|
| ⛔⛔ **a LEI DO ALVO CONVERGIDA** — limitar o avanço dos dois apertos ao que falta até ao alvo (`Pincel::converge_aperto`, a linha que a espec §5.2-ter prescreve para a saída (b)) | ⭐ **mexe em `9` dos `86` e deixa `77` byte-idênticos**, e mesmo assim é uma perda: `79 → 75` dentro da barra, porque **quatro traços exactos** (`0,000`) caem para `0,465`–`0,811`. E o que compra não é o que a espec promete: os invertidos vão só de `544` para `436` nos nove apertos (`−20 %`), com o nó a FICAR (`303 → 269`, alvo `280`). ⇒ §4 — *a saída (b) foi refutada na metade que a justificava* |
| o anel-1 pela **triangulação** | acerta o Arrastar *Local* e derruba o *Global* de `0,6457` para `0,2699` |
| **`PH2D_VARREDURAS=10`** como lei do *Local* | bit-idêntico à construção dupla nos modos de força e **diverge** nos de âncora |
| o filtro de raio na criação de restrições da área *Dynamic* | `0,181 → 0,182`, e as outras nove inalteradas |
| a **direcção do aperto medida no repouso** | melhora o plano (`1,380 → 1,012`), piora a esfera (`0,542 → 0,939`) |
| a **trava** que impede o vértice de ultrapassar o alvo | **não é inerte**: parte os traços de um passo que hoje saem ao bit |
| o plano de queda pelo **CENTROIDE** do disco | `empurrar 0,944 → 1,250`, `arrastar 0,233 → 0,716`. ⚠️⚠️ **Esta recusa respondeu a UMA pergunta, e não é a da §4.4:** o alvo não usa um centroide — usa a média das posições **puxadas para o cursor**, que shipa desde 06/09 e melhora os quatro traços de plano. *O plano pelo cursor era a aproximação de 1.ª ordem dela, e é por isso que passava quase.* |
| a projecção do `δ` no plano **tangente do pen-down** | `agarrar 0,265 → 0,605`, `gancho 0,351 → 0,663` |
| **faces invertidas** como régua de classificação | não discrimina: o arrasto tem `41`–`57` e bate a `0,071` |
| a **compressão** do par mais apertado como régua | explica a família do aperto e nada mais |
| a **soma crua** das normais da área (sem o peso `3p²−2p³`) | ⚠️ **melhora** cinco traços (`0,214 → 0,194`) e mesmo assim **não** shipa: a lei da espec foi lida no fonte e atestada, e o que a medição diz é que o peso não é a causa do resíduo do Push |
| o **peso** da normal por vértice, RE-MEDIDO contra a lei de hoje | `uniforme` contra `área`: `inflar 0,253 → 0,253`, `esfera 0,378 → 0,379`, `empurrar 0,214 → 0,213` — a recusa de 06/09 continua de pé depois do Q12 e do centro da área |
| a ordem inversa como **CHÃO DE RUÍDO** (ruler minha, construída e apagada) | ⛔ inverter a ordem não é ruído, é uma lei ERRADA: erramos `0,0713` contra o oráculo no `plano_arrastar_radial_local` e a ordem inversa move-nos `0,2932` — *estamos 4× mais perto do alvo do que ela está de nós*, e um chão de ruído nunca é maior que a distância ao alvo |
| partir o corpus em «ruído de ordem» contra «lei em falta» com barra em `0,35` | ⛔ **sem VALE**: as razões dos 25 abertos são um contínuo de `0,30` a `3,39` e o maior vazio (`0,92`) está no topo, entre os dois últimos ⇒ qualquer barra a meio seria escolhida (CLAUDE.md §0.0). O gate foi apagado; a sonda ficou sem veredito por traço |
| a **banda `φ`** das restrições como causa do espalhamento (`PH2D_ESC_PHI` de `0,4` a `1,3`) | `1,0` é um óptimo AGUDO e o corpus di-lo pelo arrasto: `plano_arrastar_radial_local` mede `0,482 · 0,359 · 0,192 · 0,071 · 0,447` na varredura, e o Push/Inflate mal se mexem (`0,214`/`0,253` contra `0,207`/`0,274` a `0,8`) |
| **mais varreduras** de relaxação (`PH2D_VARREDURAS` `5 → 10 → 20`) | destrói o arrasto: `0,071 → 0,709 → 0,910`. ⚠️ O Inflate MELHORA (`0,253 → 0,191`) e não compra nada — *um knob que cura um traço e parte outro não é a lei que falta* |
| a **retenção** de velocidade (`PH2D_ESC_RET` de `0,6` a `1,3`) | `1,0` está no óptimo e satura acima dele: `arrastar 0,138 · 0,082 · 0,071 · 0,070` |
| tirar a **banda `φ`** da relaxação (leitura literal da §5.4, `PH2D_PHI_SEM_BANDA`) | destrói o arrasto: `plano_arrastar_radial_local` `0,011 → 0,443`, e o Push/Inflate pioram (`0,237 → 0,255`, `0,245 → 0,266`). ⇒ a banda **está** no `φ` da relaxação |
| tirar a banda só do termo de **ACELERAÇÃO** da integração (a outra leitura da §5.4) | **byte-idêntico nos 65 traços** — e a razão é estrutural: dentro do pincel a banda vale `1`, e fora dele a força vale `0`. *Onde `a ≠ 0` a banda é `1`* ⇒ a frase da espec e o nosso código dizem a mesma coisa, e o knob seria morto. ⚠️⚠️ **E é aqui que o censo ficou INCOMPLETO:** a banda entra em TRÊS sítios — o `φ` da relaxação (§5.2), o termo de aceleração e o termo de **velocidade** (§5.4) —, as duas linhas acima mediram os dois primeiros, e **o terceiro nunca foi corrido**. Era o único que mordia. *Um censo de dois de três sítios lê-se exactamente como um censo completo, e conclui o contrário* |
| **mais varreduras**, RE-MEDIDO com a ordem de visita certa | `5` é agora um óptimo AGUDO e quem o diz é o arrasto: `0,011 · 0,254 · 0,555 · 0,705` para `5 · 6 · 8 · 10`. ⚠️ E o Push/Inflate MELHORAM com `8` (`0,164`/`0,162`) — *dois erros a compensarem-se, não a lei que falta* |
| o **peso** da normal por vértice, medido pela TERCEIRA vez | com a ordem de visita certa: `inflar 0,245 → 0,245`, `empurrar 0,237 → 0,237`. A metade que a §4.6 declara aberta continua sem o corpus a decidir |
| **re-apanhar o cursor na superfície deformada** (o vértice mais próximo no plano do ecrã, posição actual) | destrói tudo: `arrastar 0,012 → 0,814`, `empurrar 0,252 → 0,825`, `inflar 0,248 → 0,630`. ⚠️ **A recusa é da APROXIMAÇÃO, não da ideia** — o vértice mais próximo faz o cursor saltar de vértice em vértice e herdar a cova inteira. O `caminho` das fixtures traz `z = 0` nos doze passos, e a §4.3 diz que o cursor é re-apanhado; a pergunta continua aberta |
| **re-apanhar o cursor na superfície deformada**, 2.ª tentativa: interpolação SUAVE dos vizinhos no plano do ecrã (⛔ não o vértice mais próximo, já refutado) | pior ainda: `empurrar 0,252 → 0,937`, `inflar 0,248 → 0,608`, e o **arrasto fica intacto** (`0,012`) nas duas tentativas, porque no plano ele não afunda. ⇒ *o `caminho` das fixtures É o cursor que o alvo usa; ele fica no plano de partida* |
| a queda medida na posição de **REPOUSO** em vez da actual (`PH2D_D_REPOUSO`) | destrói: `arrastar 0,012 → 0,654`, `inflar 0,248 → 0,862`. A §4.1 diz **actual** para todos menos o Agarrar, e a medição concorda |
| a regressão do §9 nº 20 da espec como causa do aperto | foi **fechada em 2024**, dois anos antes da versão que gravou as fixtures |
| a **área *Dynamic* medida no REPOUSO** em vez das posições de agora | **byte-idêntico** nos oito traços de esfera — numa esfera os vértices mal se deslocam contra o raio do disco, e o corpus **não discrimina** as duas leituras |
| a área do **Gancho** avançada por `δ` (a leitura literal de *«`c + δ` para o Gancho»*) | destrói os onze traços de gancho do PLANO, que hoje saem a `0,000`–`0,003`: `plano_gancho_radial_local` vai a `0,091`, o `_24passos` a `0,145`, e as contagens caem centenas de vértices. E **não ajuda a esfera** (`0,255 → 0,256`) |
| o desacordo de **contagem** do gancho da esfera (`2158` contra `2234`) como conjunto simulado diferente | ⭐⭐ **é FRANJA, medido:** os `100` que só o alvo move estão a `3,13`–`3,72 R` do pen-down (o limite da banda é `3,5 R`) com deslocamentos de `10⁻⁵` a `1,7·10⁻⁴`, e os `24` que só nós movemos estão a `4,02`–`4,60 R`, **para lá** do limite — onde só o cursor que anda alcança. As duas populações **não se sobrepõem** e o maior vale `0,3 %` da amplitude do traço. ⇒ *um desacordo de contagem tem duas leituras que se leem igual; esta é a do limiar do censo, não a do conjunto* |
| ⛔⛔ **as TRÊS medições do peso da normal por vértice** (06/09 e 07/09: `uniforme` contra `área`, `0,245 → 0,245`, `0,237 → 0,237`, `0,378 → 0,379`) | ⭐⭐ **SUPERADAS em 07/09 — elas respondiam a outra pergunta.** Perguntavam *«o peso move a paridade destes traços?»* (não move: nas fixtures de plano as normais são as mesmas nos dois pesos, e na esfera as faces são quase uniformes) e não *«qual é o peso do alvo?»*. A régua que decide é a fixture de **dois traços**, onde a direcção do impulso do 2.º se lê **directamente do oráculo**: `uniforme 1,7·10⁻⁵` contra `área 6,6·10⁻⁴`, e o resíduo da área **não encolhe com o sinal** (`3,7·10⁻⁴` nos vértices que mais se movem) — *um resíduo que estaciona não é ruído, é lei errada.* ⇒ **uniforme**, e a §4.6 linha 4 fecha |

## §4 — ⛔⛔ A DECISÃO que é do DONO — e a saída (b) NÃO entrega o que promete

> ⛔⛔⛔ **REFUTADO EM 2026-09-07, com número: a promessa da saída (b) é falsa.**
> A frase que o dono ia ler para decidir diz que ao limitar *«o aperto nunca ultrapassa o ponto
> para onde puxa, **o nó não aparece em força nenhuma**»*. Implementei **exactamente a linha que a
> espec prescreve** (`Pincel::converge_aperto` — *«limitar o impulso do aperto à distância que falta
> até ao alvo, que é a única linha que a inversão pede»*) e medi o corpus inteiro:
>
> | | sem a trava | com a trava | o alvo |
> |---|---|---|---|
> | quadriláteros invertidos no traço mais forte | `303` | **`269`** | `280` |
> | idem, somados nos nove apertos | `544` | **`436`** (`−20 %`) | `525` |
> | traços dentro da barra (`0,13`) | **`79`/86** | **`75`**/86 | — |
> | traços que a trava move | — | `9` (os outros `77` ficam **byte-idênticos**) | — |
>
> ⭐⭐ **Por que ela falha, e é uma distinção que a espec FUNDE:** *«um vértice passar o cursor»* e
> *«um quadrilátero inverter»* não são a mesma coisa. A trava impede a primeira por construção; a
> segunda nasce de dois vizinhos avançarem quantidades **diferentes** — e a trava, que morde mais no
> vértice mais perto do alvo, **aumenta** essa diferença tanto quanto a reduz. A espec lê as duas
> como uma porque no primeiro passo elas aparecem juntas (`9` vértices passam · `10` quadriláteros
> invertem); *correlação no primeiro passo não é identidade nos doze.*
>
> ⚠️ **E o preço é maior do que a espec diz.** A terceira frase — *«deixa de casar com o alvo
> exactamente nos traços FORTES»* — subestima: **quatro traços que hoje saem a `0,000` caem para
> `0,465`–`0,811`**, e três deles são aperto de LINHA, o modo em que o alvo mal inverte (`2` e `6`
> faces). *A trava cobra onde não há nó a desfazer.*
>
> ⇒ **A decisão estava a ser posta entre (a) e um miragem.** Gates:
> `a_trava_do_aperto_nao_desfaz_o_no_e_a_espec_promete_que_sim` +
> `a_trava_do_aperto_custa_quatro_tracos_exactos`.
> ⚠️ Os dois pinam a **refutação**, não a FORMA do limitador: trocar o `min` por um corte duro
> sobrevive aos dois (mutação M2), e quem o trocar tem de re-medir o `79 → 75`.

Ao apertar com força alta, a inversão nasce no **primeiro** passo, antes de a relaxação correr, logo
nenhuma afinação do solver a evita. As duas saídas, nas frases que a espec §5.2-ter fixa:

- **(a) reproduzir** — o retalho debaixo do cursor vira do avesso, as faces atravessam-se e a
  superfície fica com um nó que nada desfaz depois; é o que o alvo faz hoje, e apertar com força
  baixa continua limpo. **É o que shipa agora.**
- **(b) limitar** — o aperto nunca ultrapassa o ponto para onde puxa, o nó não aparece em força
  nenhuma, e a nossa saída deixa de casar com a do alvo exactamente nos traços fortes.

⚠️ **O alvo sabe que (a) é defeito dele** — são duas entradas ABERTAS do tracker dele.

## §5 — Os instrumentos que já existem (⛔ não construa outro sem olhar)

```
cargo test --release -p ph2d-cloth --test oraculo_do_pincel <sonda> -- --ignored --nocapture
```

| sonda | o que devolve |
|---|---|
| `sonda_da_paridade_com_o_oraculo` | o corpus inteiro: movidos, máximos, erro e a razão, **nos dois lados** |
| `sonda_passo_a_passo` (`PH2D_TRACO=<nome>`) | passo a passo: o anel imediato, o **erro do passo**, onde está o **pico**, o vector do vértice do pen-down |
| `sonda_dos_artefatos_do_oraculo` | espinho · rasgo · estica · **faces invertidas** · **compressão**, nos dois lados |
| `sonda_do_perfil` · `sonda_da_cadeia_com_parede` | o perfil radial e a cadeia 1D |

Experiências por env, para bissecar: `PH2D_VARREDURAS` · `PH2D_ORDEM` (`inversa`, `celula:<n>`) ·
`PH2D_PARES=0` · `PH2D_TRI` · `PH2D_ESC_PHI` · `PH2D_ESC_RET`. No produto: `PH2D_CLOTH_LAW=vbd`.

## §7 — ⭐⭐⭐ O CUSTO, MEDIDO (2026-09-07) — e o tecto que ele nomeia

⛔⛔ **Não há um único tecto escrito no caminho do tecido** — nem a `ph2d-cloth` nem o adaptador têm um
`MAX_*`, um «por ora» ou uma cerca de densidade. Isso está **certo** pela §0.0 do CLAUDE.md (*meça
antes de limitar*) e deixava a outra metade por fazer: ninguém sabia onde o pincel deixa de caber num
quadro. Medido agora ([`sonda_do_custo_de_um_dab_de_tecido`](../../../crates/ph2d-sculpt3d/src/stroke_cloth_mode_tests.rs),
`load 5,6`, duas corridas a concordar a `1 %`):

| grelha | vértices | 1.º dab (constrói) | regime | % de um quadro de 60 fps |
|---|---|---|---|---|
| `64²` | `4 225` | `1,6 ms` | **`1,43 ms`** | `8,5 %` |
| `128²` | `16 641` | `6,9 ms` | **`6,0 ms`** | `36 %` |
| `160²` | `25 921` | `10,3 ms` | **`9,1 ms`** | `55 %` |
| `160²` (área *Global*) | `25 921` | `11,7 ms` | `10,1 ms` | `60 %` |

⭐ **É LINEAR nos vértices, e o coeficiente é `0,35 µs` por vértice por dab** (`0,34` · `0,36` ·
`0,35` nas três densidades). ⇒ **um quadro de `16,7 ms` compra `~48 000` vértices a um dab por
quadro**, e o dono esculpe a `~25 000`.

⚠️ **O recurso que este tecto nomeia é TEMPO DE CPU nas cinco varreduras de relaxação** — e ⛔ **elas
não são paralelizáveis, por construção e não por preguiça:** a ordem em que as restrições são
resolvidas é **metade da lei** desta linha (Gauss–Seidel não comuta; varrer por índice crescente
deixa `30` traços acima da barra e pela célula deixam `15`). *Uma varredura paralela responde outra
coisa, e é a coisa que este módulo passou uma semana a provar que importa.*

⭐⭐ **E a COLISÃO tem preço próprio, medido no mesmo dia** — ela acrescenta **um raio por vértice
activo, por colisor e por passo**:

| grelha | vértices | colisores | sem | com | razão |
|---|---|---|---|---|---|
| `64²` | `4 225` | `1` | `1,53 ms` | `4,24 ms` | **`2,8×`** |
| `64²` | `4 225` | `3` | `1,52 ms` | `9,24 ms` | **`6,1×`** |
| `128²` | `16 641` | `1` | `6,40 ms` | `16,63 ms` | **`2,6×`** |
| `128²` | `16 641` | `3` | `6,41 ms` | `35,97 ms` | **`5,6×`** |

⇒ **com um colisor a `16 641` vértices o dab já é um quadro inteiro**, e com três são dois. *É este
número que justifica a opção nascer desligada* — e ⛔ ele é o preço de UM dab, não de um quadro.

⚠️ **E o número é POR DAB, não por quadro:** o traço emite a `0,15 × raio` de espaçamento, logo uma
mão rápida entrega vários dabs no mesmo quadro e multiplica isto. *A medição diz o preço de um; quem
decide quantos cabem é o dono.*

⛔ **Nada disto é licença para optimizar** (§0.0): é a tabela que tem de existir **antes** de alguém
escrever um limite, para que ele diga de que recurso é e traga o número ao lado.

## §6 — O PROTOCOLO, em quatro passos

1. **Meça** com a sonda por passo até localizar o defeito num vértice, num passo, ou numa grandeza.
2. **Refute o que puder sozinho** — metade das perguntas morre aqui, e cada refutação vai para o
   INBOX com o número.
3. **Escreva a pergunta no INBOX** (`docs/3D/cleanroom/INBOX_blender-cloth.md`) com a tabela dentro,
   e despache um **subagente-E**. ⛔ Contrato de retorno: uma frase, cinco linhas funcionais, zero
   identificador do alvo.
4. **Despache o R-pré** antes de LER a emenda, implemente, meça outra vez, e escreva o gate com a
   prova de mutação.
