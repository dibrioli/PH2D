# 108 — CICLO 5: SIMULAÇÃO — deixar a física decidir

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — um grupo por ciclo, params **no cartão**,
> e o entregável (e o smoke) é um **tutorial em PDF**.
> **Premissa do tutorial:** *«Deixar a física decidir»*.

Nos quatro ciclos anteriores o artista disse **onde** cada coisa está, **como** ela anda e **quem**
é afectado. Este grupo entrega a última decisão a uma **lei** — e o que ele pede em troca é um
**relógio**.

---

## §1 — Passo 2: A AUDITORIA

### §1.0 — O grupo é meio LISTA e meio FAMÍLIA, e a família é DERIVADA

`sim.zone` · `sim.spawn` · `sim.step` · `sim.lifetime` · `sim.collide` · `motion.integrate` — mais
**toda** a família `force.*`, pedida ao registry (`motion_ciclo_probe::familia("force.")`).

⚠️ **Uma lista de família escrita à mão envelhece em silêncio** no dia em que um nó dela nasce, e o
ciclo fecharia com ele por auditar. ⭐ Gate `the_force_family_is_derived_and_not_empty`, com piso:
sem ele, um prefixo mal escrito deixaria as cinco sondas a auditar **seis** nós em vez de doze —
todas verdes, todas caladas.

### §1.1 — ⛔ O catálogo já está FECHADO (as folhas 02, 03 e 13)

Como no ciclo 4: a conferência auditou estes nós contra Houdini/C4D/MOPs e as três folhas fecharam
a **zero P1**. ⇒ a auditoria do ciclo pergunta o que mudou desde então — e o que mudou é a
**superfície**.

### §1.2 — O RETRATO (`audit_the_sim_group`)

| nó | params | no cartão | device | portas | efeito |
|---|---:|---:|:---:|:---:|---|
| `sim.zone` | 5 | 3 | sim | 2→1 | Temporal |
| `sim.spawn` | 6 | 6 | sim | 2→1 | Temporal |
| `sim.step` | 4 | 4 | sim | 1→1 | Temporal |
| `sim.lifetime` | 3 | 3 | sim | 1→3 | Pure |
| `sim.collide` | **15** | **7** | sim | 1→1 | Pure |
| `motion.integrate` | 1 | 1 | sim | 2→1 | Temporal |
| `force.attractor` | 11 | 10 | sim | 2→1 | Pure |
| `force.buoyancy` | 8 | 8 | sim | 1→1 | Temporal |
| `force.curl` | 11 | 11 | sim | 1→1 | Temporal |
| `force.drag` | 3 | 3 | sim | 1→1 | Pure |
| `force.vortex` | 8 | 7 | sim | 1→1 | Pure |
| `force.wind` | 12 | 9 | sim | 1→1 | Temporal |

⭐ **Os doze estão no dispositivo.**

### §1.3 — O VOCABULÁRIO: dezasseis params partilhados, UMA divergência

`air_resist` · `angle` · `center_x/y` · `curve` · `lacunarity` · `loop_period` · `octaves` ·
`radius` · `roughness` · `seed` · `strength` · `substeps` · `type` — **todos** com o mesmo rótulo
em todos os nós que os declaram.

⚠️ A excepção era o **`mode`**: `sim.zone` chama-lhe **`Life Cycle`** (`Forever | Once | Loop`)
e o `force.vortex`/`force.wind` chamavam-lhe **`Mode`** (`Force | Target Velocity`).

⛔⛔ **O `Mode` vago era a SEGUNDA aparição em dois ciclos** (no 4 foi o `field.combine`), e aqui
era pior, porque os dois forces fazem **a mesma pergunta** — *«isto empurra, ou impõe uma
velocidade?»* — e ela tem um nome, que os doc-comments deles já citam da referência (o
*Treat as Wind* do POP Axis Force do Houdini).

✅ **Curado na [W2](#-w2--o-mode-que-não-diz-nada-o-achado-da-13--2026-09-09): o rótulo é
`Acts As`**, e a divergência que fica no grupo é a CERTA — duas perguntas diferentes, dois
nomes específicos (`Life Cycle` · `Acts As`), sem a palavra vaga em lado nenhum.
⚠️ **E o censo alargado devolveu o número que ninguém tinha:** `Mode` era pintado por **26 nós
sobre 24 perguntas distintas** no catálogo inteiro.

### §1.4 — ⭐ E o `target_*` do attractor NÃO é o sexto vocabulário do pivô

O `force.attractor` chama ao seu ponto de mundo **`target_x`/`target_y`**, enquanto o
`force.vortex` e o `sim.collide` chamam **`center_x`/`center_y`**. Isso **parece** o achado §2.3 do
ciclo 3 e **não é**:

| palavra | o que ela quer dizer | quem a usa |
|---|---|---|
| `center_*` | **onde esta coisa está** | `field.box` · `field.radial_sweep` · `motion.falloff` · `force.vortex` · `sim.collide` |
| `target_*` | **aquilo para onde eu aponto** — e pode vir de um STREAM | `force.attractor` (`target_mode`) · `motion.look_at` (`Aim At`) |

⇒ são duas perguntas, e a casa já as separa de forma consistente. ⛔ Unificar destruiria sentido —
é uma cerca de Chesterton com a razão visível no `target_mode`.

### §1.5 — ⛔⛔ O ACHADO: TRÊS nós põem coisas no mundo e nenhum tem alça

`which_sim_nodes_have_a_place`, derivado (quem declara o vocabulário espacial contra o que o
`field_gizmo::spec_for` conhece):

| nó | centro | raio | ângulo | alça de canvas |
|---|:---:|:---:|:---:|:---|
| `sim.collide` | sim | sim | sim | ⛔ **NÃO** |
| `force.vortex` | sim | sim | — | ⛔ **NÃO** |
| `force.attractor` | `target_*` | sim | — | ⛔ **NÃO** |

⚠️ **É o achado do ciclo 4 um grupo adiante** — e aqui morde mais: o **colisor** é literalmente
uma coisa que se põe num sítio, e pôr um chão com dois sliders é o idioma que o gizmo existe para
substituir.

---

## §2 — O PLANO, wave a wave

### ✅ W1a — A ALÇA DO COLISOR, e a `spec_for` que passou a ver os PARAMS (2026-09-09)

O `sim.collide` é **literalmente uma coisa que se põe num sítio** — um chão, um prato, uma caixa —
e punha-se com dois sliders. Hoje tem caixa no canvas: mover · girar (`angle`) · redimensionar.

⚠️⚠️ **Ele é o primeiro nó cuja spec depende dos PARAMS e não só do TIPO:** as quatro formas
(`Plane · Disc · Bowl · Box`) medem-se com params **diferentes**, e o `motion.falloff` do ciclo 4
escapou a isso porque uma `Disk` servia as três formas dele. ⇒ `spec_for(type_id, &params)`.

⚠️ **O `Plane` recebe a caixa do `Box` de propósito:** um plano é infinito e não tem extensão para
agarrar, mas o `angle` e o `height` **são** o que a alça move. A alça de tamanho escreve num param
que aquela forma não lê — **inerte, não mentiroso**. ⛔ Não lhe dar spec nenhuma tirava também o
mover e o girar, que é exactamente o que ele precisa.

⛔ **Duas mutações, ambas nomeadas:** tirar o braço do colisor, ou congelar a leitura da forma, dão
*«a forma `Box` mede-se pelas duas extensões»*.

### ⛔ W1b — AS OUTRAS DUAS ALÇAS estão BLOQUEADAS, com o preço medido

O `force.vortex` e o `force.attractor` põem-se no mundo e **não têm param de ângulo** — girar um
vórtice em torno do próprio centro não move um texel. Uma spec deles teria `rotation: None`, e o
artista arrastaria a argola de rodar **sem nada acontecer**.

⇒ a cura certa é a `GizmoView` saber **suprimir** a argola, e o preço está **medido: 28 sítios de
construção dela**, em crates de **outras linhas** (`ph2d-editor-core`, `ph2d-tool-painter`). Um
campo obrigatório num struct partilhado é o oposto de *«projecte o foundational para isolamento»*
(CLAUDE.md §0.2), e colide na integração.

⛔ **Nomeado, não contrabandeado — e nada morto shipa.** O `rotation` da spec já é `Option`, então
o dia em que a `GizmoView` souber suprimir, as duas entram com um braço cada.

### ✅ W2 — O `Mode` que não diz nada (o achado da §1.3) — 2026-09-09

O `force.vortex` e o `force.wind` fazem **a mesma pergunta** — *«isto empurra, ou impõe uma
velocidade?»* — e chamavam-lhe **`Mode`**. Hoje chamam-lhe **`Acts As`**, e o cartão lê-se
`Acts As: Force` / `Acts As: Target Velocity`.

⚠️ **Só o RÓTULO mudou.** A chave do param continua a ser `mode` ⇒ **nenhum documento já
gravado se mexe**, e não há degrau de schema nenhum.

#### ⭐⭐⭐ A régua é DERIVADA, e o censo dela é o achado

⛔ Nada aqui é uma lista de palavras proibidas nem um limiar escolhido. A lei
(`the_mode_label_names_the_question_it_asks`) é: **todo nó que pinta este rótulo oferece as
mesmas opções** — se outro nó pinta a mesma palavra sobre um enum de valores diferentes,
então a palavra não é o nome de uma pergunta, é um espaço reservado.

Corrida sobre o registo inteiro (`the_house_census_of_enum_labels`, 2026-09-09):

| rótulo | nós | perguntas distintas |
|---|---:|---:|
| `Curve` | 7 | **1** |
| `Time Mode` · `Range` | 3 | **1** |
| `Noise Type` · `Edge` · `Distance` | 2 | **1** |
| `Pivot` | 5 | 2 |
| `Shape` | 8 | 5 |
| `Direction` | 4 | 4 |
| ⛔ **`Mode`** | **26** | **24** |

⭐ **A casa cumpre a lei quase toda** — um rótulo partilhado quase sempre significa a mesma
coisa. O `Mode` é o oposto exacto: **a palavra mais usada do catálogo é a que menos diz**, e
o artista tem de a reaprender em cada cartão.

⚠️ **A referência já dá nome a esta pergunta** (o *Treat as Wind* do POP Axis Force do
Houdini) e ali ela é uma **caixa**; aqui fica um enum de dois valores, porque são os
VALORES que ensinam o que cada lado faz — `Acts As: Target Velocity` diz-se sozinho,
`Treat as Wind: ☑` não.

#### ⛔ Os outros 24 ficam NOMEADOS com a medição, não contrabandeados

É o mesmo corte do ciclo 4 (os rótulos de mistura: `Blend` · `Shadow Blend` ·
`Flash Operator` · `Echo Operator` · `Mode`): alinhar o resto é wave dos ciclos que
**possuem** aqueles grupos, e mexer neles aqui tocaria em nós que este ciclo não auditou.
⇒ a lei corre **sobre o par**, e o censo da casa inteira corre à mão.

#### ⚠️ Três mutações, e cada uma morre no gate CERTO

| mutação | `both_forces_speak_…` | `the_mode_label_names_…` |
|---|:---:|:---:|
| só o `force.vortex` volta a `Mode` | ✗ | ✓ |
| **os dois** voltam a `Mode` | ✓ | ✗ |
| os dois passam a `Shape` (8 nós, 5 perguntas) | ✓ | ✗ |

⭐⭐ **A segunda linha é a que justifica o gate novo:** com os dois a divergir, quem acusa é a
régua de irmãos que já existia; com os dois a **concordar na palavra vaga**, ela fica verde e
só a lei derivada vê o defeito. ⭐ E a terceira prova que a lei **não está colada à palavra
`Mode`** — ela mede a propriedade, não o literal.

### ✅ W3 — O CARTÃO do grupo: três paredes de sliders, e o piso que a casa já tinha desenhado — 2026-09-09

#### ⛔ A premissa da fila estava ERRADA, e a medição desfê-la

Esta wave estava escrita como *«os 8 de 15 do `sim.collide`»*. Medido: os 8 params que aquele
cartão esconde estão escondidos por **`ParamGate`**, e cada um deles é a cura certa — as
extensões da caixa não aparecem num disco, o ângulo não aparece numa forma com simetria de
rotação, a semente não aparece com a aleatoriedade a zero. ⭐ **Um param escondido pelo modo
que não o lê é o oposto de um defeito**, e o `sim.collide` já era o nó **mais** arrumado do
grupo: **15 de 15** params em secção desde a folha 13.

⇒ *o que a auditoria de um ciclo escreve na abertura é uma hipótese, e a wave começa por a
medir.*

#### ⭐⭐⭐ O defeito real: **1 de 12** nós do grupo tinha secções

| nó | params | secções (antes) |
|---|---:|---|
| `sim.collide` | 15 | **Shape · Particle Size · Response** |
| `force.wind` | 12 | ⛔ nenhuma |
| `force.attractor` | 11 | ⛔ nenhuma |
| `force.curl` | 11 | ⛔ nenhuma |
| `force.buoyancy` · `force.vortex` | 8 | — |
| os outros seis | 1–6 | — |

É o achado do ciclo 4 (o `field.box` a pintar nove rows em fila ao lado de um irmão idêntico
que já as agrupava) um grupo adiante, e aqui ele acusava **três** cartões.

#### ⭐⭐ O PISO é medido, não escolhido — e o vale existe

⛔ Nenhum número foi inventado para decidir *«quão grande é grande»*. Corrido sobre o registo
inteiro: **o menor cartão que esta casa alguma vez julgou valer uma secção tem `9` params** —
e é o `field.box`, a wave do ciclo 4. Do outro lado, **14** nós do catálogo estão em `9`+ sem
uma única secção (três deles neste grupo).

⇒ `SECTION_FLOOR = 9`, e ⚠️ **abaixo dele a decisão é ficar em fila**: `band_len = params +
secções`, uma secção aberta custa **+1 fileira**, e a lei que o `field.box` já escreve é que
*numa carta de 3, 4 ou 6 rows dois cabeçalhos organizam menos do que ocupam*. É por isso que
o `force.vortex` e o `force.buoyancy` (8 cada) **ficam como estão** — decisão medida, não
esquecimento.

#### As três tabelas, e de onde vem cada palavra

| nó | soltos (a razão de existir) | secções |
|---|---|---|
| `force.attractor` | Strength · Repel | **Placement** (Target, Target X/Y, Predict) · **Falloff** (Radius, Curve, Min/Peak/Reversal Distance) |
| `force.curl` | Strength | **Field** (Noise Type, Scale, Offset X/Y, Octaves, Lacunarity, Roughness, Seed) · **Timing** (Speed, Loop Period) |
| `force.wind` | Angle · Strength · Acts As · Air Resistance | **Gust** (Gust, Noise Type, Octaves, Lacunarity, Roughness, Seed) · **Timing** (Gust Frequency, Loop Period) |

⭐ **Nenhuma palavra é nova.** `Placement`/`Falloff` são as que o `field.box` e o
`field.radial_sweep` shiparam no ciclo 4; `Field`/`Timing` são as do `motion.noise`.
*Quando o objectivo é alinhar irmãos, a autoridade é o irmão.*

⚠️⚠️ **A excepção é `Gust`, e ela é MEDIDA no nó:** o ruído do `force.curl` é **espacial**
(tem `scale` e `offset_*`) e o do `force.wind` é **temporal** — cada elemento amostra a sua
linha em `t · gust_freq`, e o doc do `eval` dele escreve-o à letra (*«ele não é espacial»*).
⇒ herdar o título `Field` alinharia a palavra e faria a secção **mentir sobre o que contém**.
*Alinhar irmãos é alinhar perguntas, não carimbar títulos.*

⚠️ **`Placement` no atractor é o ALVO, e isso não colapsa a distinção da §1.4:** a secção diz
*onde esta força opera*, e para ele esse sítio é o ponto para onde aponta — que pode vir de um
STREAM, e é por isso que o `target_mode` abre a secção.

#### ⚠️ A catraca tem CENSO DE OBSOLESCÊNCIA, e ele é metade do gate

*Uma catraca sem censo não desce: ela vira licença* (`CLAUDE.md` §5.0). O piso deste gate é
re-medido **a cada corrida**: `SECTION_FLOOR` tem de continuar a ser o menor cartão com
secções de **todo o registo**. No dia em que outra linha agrupar um cartão mais pequeno, a
casa terá baixado a própria régua — e o gate diz isso em voz alta, com o número, em vez de
derivar em silêncio e acender o grupo inteiro sem explicação.

#### As mutações — três, e cada uma morre numa asserção DIFERENTE

| mutação | onde morre |
|---|---|
| o `force.wind` deixa de registar as secções | a lei (`no_big_card…`, a parede) |
| uma linha de secção nomeia `loop_periodo` | o gate irmão (`every_section_…`, a secção muda) |
| `SECTION_FLOOR` passa a `8` | o **censo** (`no_big_card…`, a metade da obsolescência) |

⭐ A terceira é a que importa: sem ela o piso seria um número a envelhecer sozinho.

### ✅ W4 — A CENA, o TUTORIAL e as FIGURAS (passo 6) — 2026-09-09

#### A cena `=113` — *Deixar a física decidir*

Uma chuva de peças cai sobre um **bloco**, empilha-se em cima dele e escorrega pelos lados; o
`sim.zone` está em `Loop`, então a queda **recomeça sozinha** — sem isso o monte assentava uma
vez e todo passo do smoke daí em diante mexeria num knob sem nada para mexer.

⚠️ **O colisor é um `Box` e NÃO o chão, e a razão é o tutorial:** a alça da W1a faz **três**
coisas — mover, girar e redimensionar — e num `Plane` só duas são lidas (a alça de tamanho
escreve num param que a forma ignora: *inerte, não mentiroso*). ⛔ **Um tutorial não aponta para
um controlo inerte.**

⚠️ **O índice de enum é PERGUNTADO ao registo, nunca digitado** (`indice_de`): as outras cenas de
simulação escrevem o número à mão com o nome num comentário, e aqui o `sim.collide` tem **quatro**
formas e o ciclo acabou de lhes dar uma alça que **muda com a escolhida** — um literal que
envelhecesse poria a cena a montar a forma errada **sem erro nenhum**.

#### ⛔⛔ O gate do tutorial apanhou-me antes do dono, como no ciclo 4

Eu tinha escrito **`Collide`** e **`Zone`** no anúncio e no gate. Os cartões pintam-se
**`Collider`** e **`Simulation Zone`** — o passo mandava o dono procurar cartões que não existem,
que é exactamente a [memória](../../project-memory/feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list.md)
que esta casa já pagou.

⚠️ **E este gate corre sobre a CENA, não sobre os defaults** — neste ciclo a diferença morde: o
colisor da cena é um `Box`, logo o cartão mostra `Box Width`/`Box Height` onde o default mostraria
`Radius`. *O dono vê a cena.*

#### ⚠️ E a §5 do tutorial estava errada contra o código

A 1.ª redacção escrevia *«Integrate — quem aplica o que as forças somaram; uma cena tem um»* — e a
cena **não tem nenhum**. São **duas maneiras** de correr uma simulação e escolhe-se uma: dentro de
uma zona o que circula **é** o estado e quem dá o passo é o `sim.step`; o `motion.integrate` é para
quando a sim é um **desvio de uma animação**. ⛔ O doc do próprio `sim.step` diz que pôr os dois dá
à sim **duas memórias**, que discordam na primeira peça que desaparecer.

#### ⭐⭐⭐ As figuras, e a régua que se corrigiu DUAS vezes

| redacção | régua | porquê caiu |
|---|---|---|
| 1.ª | a queda **total** (`2,458` contra `1,824`) | uma diferença real e **12 % da moldura** — as duas imagens saíram quase iguais, com uma legenda a dizer *«acelera»* e outra *«satura»* por cima de dois desenhos que o olho lê como o mesmo |
| 2.ª | os **vãos iguais** sob `Target Velocity` | reprovou **com razão**: com a `Air Resistance` da cena (`1`) a constante de tempo é **um segundo**, e o segundo que a queda dura não chega (`6,83×`) |
| ✅ 3.ª | **a segunda diferença** | `1,00×` sob `Force` contra `0,55×` sob `Target Velocity` |

⭐ *Uma figura que precisa da legenda para se distinguir da irmã não é uma figura, é uma
afirmação.* ⛔ **E a cura da 2.ª não foi subir a resistência até a figura ficar bonita** — isso
mediria um knob que a cena não tem. A grandeza que separa os dois modos em **qualquer** janela é
como o vão **cresce**: com aceleração constante ele cresce sempre pela mesma quantidade; com
saturação cada acréscimo é menor que o anterior.

⚠️ **A moldura também se corrigiu:** a fila inteira são 12 peças sobre `~4,6` unidades de largura
contra `~2,5` de queda — o quadro saía deitado e o vão vertical, que **é** a mensagem, ficava com
meia dúzia de pixels. Três peças adjacentes do meio põem o quadro em pé.

#### Os gates da cena

| gate | o que mede |
|---|---|
| `the_block_catches_part_of_the_rain_and_the_rest_falls_past_it` | 60 peças: **20** pousam, **40** passam ao lado — as duas metades, porque um colisor que apanha tudo ensina que é um chão |
| `the_target_velocity_mode_caps_the_fall_and_force_does_not` | pico `10,92` sob `Force` contra `4,45` sob `Target Velocity` (`Strength = 5`), a MESMA cena com um param de diferença |
| `every_row_the_sim_tutorial_names_is_on_the_card` | cada título, linha e **secção** que o tutorial manda procurar |

**O tutorial:** [`tutoriais/05_simulacao.pdf`](tutoriais/05_simulacao.pdf), 7 páginas, com a tabela
«o que cada controlo faz» derivada da porta única (11 nós).

### ⏳ W5 — A MEDIÇÃO (passo 5): a RESIDÊNCIA está feita, o RELÓGIO está bloqueado

#### ✅ A residência — e ela não é uma leitura de relógio

⭐ **Dez dos doze nós são reivindicados pelo dispositivo** na cadeia `grid → nó → output`, com
`320 × 320 = 102 400` objectos. Isto é uma propriedade do **planeador**, não do relógio, então
vale independentemente da carga da máquina:

| nós | device |
|---|---|
| `sim.step` · `sim.lifetime` · `sim.collide` · `motion.integrate` · as **seis** `force.*` | 🟢 a cadeia inteira |
| `sim.zone` · `sim.spawn` | ⚪ não mensuráveis nesta cadeia (ver abaixo) |

#### ⛔⛔ O RELÓGIO NÃO FOI MEDIDO, e escrever os números seria pior que não os ter

A corrida saiu a **`load 13,06`**, e a lei da casa é que *nenhuma leitura de relógio desta
workstation vale nada acima de `load ~5`* (`CLAUDE.md` §5.0 — o mesmo binário já deu `11,36` e
`5,50 ms` para o mesmo passe). O esperador ficou **25 minutos** à espera de calma e desistiu.

⚠️ **A causa não é desta linha:** a máquina tinha três binários de teste de **outras worktrees**
a correr — dois órfãos da `line/Vector` (`ph2d_poly2d`, reparentados ao init, **33 e 37 minutos**
a ~270 % de CPU cada) e um da `line/3DModeling`. ⇒ *a medição fica pendente de uma máquina
calma, e a tabela que a corrida imprimiu **não** entra neste doc.*

#### ⭐⭐⭐ E a corrida devolveu um defeito do INSTRUMENTO PARTILHADO, que vale mais que a tabela

A régua que decide *«esta linha leva número?»* perguntava se o nó **DECLARA** precisar de outra
porta (`required_inputs`). O `sim.zone` e o `sim.spawn` **não declaram nenhuma** — e a tabela
imprimiu-lhes `9,29 ms · 0,38×` e um veredito de dispositivo sobre **zero objectos**.

⭐ *Um número sobre um stream vazio não é um número pequeno: é a ausência de medição com cara de
medição.* A declaração é um **proxy**; a **contagem é o facto** ⇒ `porque_nao_medir(reg, nó, n)`,
com a razão original mantida (ela é a mais informativa das duas) e `n == 0` acrescentado.

⚠️ **A lei do gate é uma IMPLICAÇÃO, não uma lista** — *emitiu zero ⇒ recusada* —, com os **dois**
controlos: no grupo tem de haver pelo menos um nó medido e pelo menos um recusado, senão ela é
vácua nos dois sentidos. Mutação (a régua volta a ser só o proxy): ✗, e a mensagem nomeia o
`sim.zone`.

⚠️ **E o mesmo instrumento tinha uma armadilha ao lado:** a mediana era `ms[1]`, um literal que
presumia `repeticoes == 3`. Um chamador com outro número saía por *index out of bounds* — que é
uma armadilha para o ciclo seguinte, não uma mensagem. Hoje é `ms[ms.len() / 2]`.

### ⏳ W6 — o smoke do dono (passo 7)
