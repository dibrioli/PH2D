# Handoff de integração MESTRE — `line/physics` (2026-08-12)

> **A linha NÃO integra nem faz ship** (CLAUDE.md §0.7). Este documento é o que o
> integrador precisa para não colidir nem regredir. DIRETRIZ §1.5.9.
>
> ⚠️ **Ele SUPERSEDE o
> [`HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md`](HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md)
> apenas como *o que integrar agora*** — o **detalhe de mecanismo** das sete
> waves de sensores (`W-Swim` · `W-SwimLine` · `W-ZoneForce` · `W-ShapeCast` ·
> `W-Probes` · `W-Probes2` · `W-FootFan`) continua LÁ e **não foi copiado**. Leia
> os dois: este para a superfície de colisão e para as TRÊS waves novas, aquele
> para o porquê de cada número das anteriores.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/physics` |
| HEAD | **o tip de `line/physics`** ⚠️ ver abaixo |
| merge-base com `main` | `76788440adbabb0e5b12f8fdafecc6f1e1183e1a` |
| commits | **65** |
| diff | 121 arquivos, +22.469 / −1.473 |

⚠️ **Todos são pós-integração de 2026-08-10** (a jornada `W-KinMove` / modo
cinemático, que já está no `main`). Nada aqui foi entregue antes.

⚠️ **O HEAD não é escrito aqui de propósito, e a razão é aritmética:** o commit
que o escreve MUDA o HEAD, então um sha nesta tabela é falso no instante em que é
commitado. O que identifica esta entrega é o **merge-base** acima mais *"o tip da
branch"* — que é o que um integrador usa de qualquer forma.

**O assunto é o PLAYER, em oito metades.** As duas primeiras estão no handoff
de 08-11 (o catálogo do plano 08, e os SENSORES). As seis novas aqui são o **PULO
DO AR** (§2), a **BEIRADA** (§2b), o **PLANEIO** (§2c), o **SENSOR DA BEIRADA**
(§2d) e os **dois reports** que fecharam a jornada — o **ACORDE** (§2e) e a
**ÂNCORA** dos gizmos (§2f).

⚠️ **As três últimas são de REPORT, não de plano**, e as três fecharam com o
smoke aprovado pelo Enio.

---

## 2. A primeira wave nova — `W-MultiJump`, o pulo do ar

**O `air actions counter` do tnua**, e o item mais pedido do catálogo do plano
08. `JumpConfig` ganhou **`air_jumps`** (a contagem; `0` desliga) +
**`air_jump_height`** (metros). A carga recarrega no CHÃO, no MESMO braço
`if grounded` do coyote — o **terceiro** consumidor da porta única
`JumpState::on_ground` (os outros: o coyote e o ARRANQUE), sem uma 2ª cópia do
predicado, que é exatamente o que o plano exigia.

### O que o plano NÃO previu, e é a parte que precisa de leitura

⚠️ **O proxy do ARRANQUE apodreceu com o terceiro pulo.** O `lib.rs` perguntava
*"a TRANSIÇÃO para o ar"* (`!antes.airborne && depois.airborne`) — exato enquanto
todo pulo começava com o pé em ALGO (chão, parede), e **falso para um pulo do
AR**, que acontece com `airborne` **já verdadeiro**. Ele dizia *não* justamente
no gesto que mais se encadeia com um arranque, contra o que o próprio comentário
de lá promete (*"um pulo de QUALQUER tipo cancela o arranque"*).

Nasceu **`JumpStep::jumped`**, e com ele o **terceiro** gate de cancelamento —
que o comentário do gate irmão da parede **já previa sem o saber** (*"é o gate
seguinte que a apanha, e por isso os dois existem"*).

### As decisões, cada uma com o motivo

⚠️ **A altura do ar é em METROS, não uma fração do primeiro pulo.** Este módulo
tem **três** pulos, e o da parede já é altura absoluta
(`WallConfig::jump_height`) — uma escala aqui faria dois falarem metros e um
falar multiplicador, na mesma seção do painel.

⚠️ **A precedência é a força do APOIO: chão > parede > ar.** Um pulo de parede
**não gasta carga** (a parede é apoio próprio), e o bloco do ar **não tem guard
de *"não estou no chão"***: os dois ramos acima já RETORNARAM em todo caso com
apoio, então chegar ali **é** estar no ar — um `!grounded` seria a 2ª cópia de
uma condição já decidida, e a cópia que envelhece quando um quarto apoio
aparecer.

⚠️ **`next.buffer = 0.0` no ramo do ar é load-bearing:** sem ele o mesmo aperto
re-dispara em tiques consecutivos e queima **as três cargas em ~6 tiques** —
três boosts empilhados, um foguete.

⚠️ **`takeoff: false`, pela mesma física do pulo de parede:** a 3ª lei devolve ao
chão o que o pé nele empurrou, e este pé não empurrou nada. Marcá-lo afundaria
uma jangada com um pulo dado no ar acima dela.

### Medido (`measure_multi_jump`, pela porta do produto)

| gesto | pico acima do repouso |
|---|---|
| um toque | **0,6176 m** |
| dois toques (o 2º no ar) | **1,2326 m** |
| um toque com 0 / 1 / 3 cargas | **0,6176 nos três** |
| duas rodadas com um pouso no meio | **1,2326 nas duas** |
| aperto SEGURADO, um pulo | **1,903 m** |
| aperto SEGURADO, dois pulos | **4,028 m** |

As duas últimas são o que escolhe as prateleiras da cena `=110`.

---

## 2b. A segunda wave nova — `W-Ledge`, a beirada

**O exemplo que o Enio deu quando o plano 08 foi escrito** (§4.5), e o item que
faltava para o player alcançar o que um plataforma 2D moderno oferece: o
personagem que erra o pulo **agarra o parapeito** em vez de escorregar pela face,
e sobe dali.

### ⚠️ A bifurcação que o plano tratava como decisão DISSOLVEU

O §4.5 nomeava uma escolha de arquitetura — *o pendurar escreve POSE (e a
simulação para de mandar) ou escreve VELOCIDADE?* — e depois de a lei estar
escrita **a pergunta não existe**: nem o pendurar nem a subida escrevem pose. Os
dois são um **`boost`** (o que estava lá é substituído) mais o cancelamento de
gravidade pelo canal `PlayerStep::gravity_hold` que o **arranque** já usava.

Isso é o que a torna barata e o que a torna correta ao mesmo tempo:

- **a lei é a MESMA nos dois modos** — o `kinematic_advance` integra o motor, a
  ponte dinâmica aplica o impulso, e nenhum dos dois aprende o que é uma beirada;
- **o solver continua a resolver contatos** — um personagem pendurado que
  encontre geometria no caminho da subida é parado por ela, e não a atravessa.

### O SENSOR é UM raio, e o que ele recusa é de graça

`ledge_ray` é **um raio para BAIXO, à frente da cabeça**: origem em
`[cx + lado·(meia_largura + grab), topo + grab]`, alcance `2·grab`.

⚠️ **`distance == 0` é a recusa de *"a parede continua acima da minha cabeça"*, e
ela não custa uma linha de lógica** — é o contrato de penetração que o
`cast_ray` já publica: origem DENTRO de geometria devolve zero. Sem isso o
personagem agarraria o meio de uma parede lisa.

⚠️ **E o `x` do raio É o alvo da subida.** `across = 2·meia_largura + grab` põe a
borda de DENTRO do corpo exatamente sobre o ponto que o raio provou ser
prateleira — então **a beirada não depende do sensor de parede/flanco**. Um
sistema a menos para manter de acordo.

### DOIS limiares de UM número (o idioma do `swim_enter`)

- **agarrar** exige `lip_rise > 0` (o lábio acima da cabeça);
- **segurar** aceita a banda inteira `[−grab, grab]`.

⚠️ **Sem essa assimetria o servo desliga-se a si próprio:** ele existe para levar
o topo do corpo ATÉ o lábio, e no instante em que consegue, `lip_rise` cruza zero
— com um limiar só, o sucesso dele seria a condição de largar.

### A subida é DISPARADA POR BORDA, e o smoke provou porquê

`LedgeState::was_jump` (gravado **também no braço ocioso**) faz do mantle um
gesto de **aperto novo**. Com disparo por NÍVEL o pendurar era **invisível**:
chega-se a uma beirada *a pular contra ela*, com o dedo já em baixo, então o
personagem subia no mesmo tique em que agarrava e o artista via um pulo alto.

### A subida é um L, não uma diagonal

Sobe primeiro, atravessa depois — uma diagonal corta a quina do bloco e o solver
a bloqueia.

### Medido, pela porta do produto (`measure_the_ledge_armed`)

| | |
|---|---|
| pendurado | o topo do corpo assenta a **2,5 mm** do lábio |
| depois da subida | **de pé** em `lábio + float_height` (4,4005 contra 4,4) |
| varredura | os **seis** pares `(grab, speed)` dão o mesmo número |

⚠️ **E dois números de PRODUTO que a cena obrigou a medir, porque a aritmética de
ar livre mente:** um pulo **colado à parede** alcança **0,745 m** contra os
**1,903 m** do ar livre — o atrito contra a face come **61%** da subida —, e o
topo do corpo pica em **2,145 m**. A primeira versão da cena 111 pôs o patamar
alto em 2,60 usando o número do ar livre, e **o corpo nunca chegava à janela**.

⚠️ **E empurrar contra uma parede com `wall_slide_speed = 0` NÃO faz cair:** o
atrito segura (medido: 0,14 m em 1,5 s ≈ 7,5 cm/s). O controlo de um gate de
produto teve de deixar de ser *"ele cai"* para ser *"ele não chega ao lábio"* — o
que a capacidade muda não é *cair ou não*, é **onde ele pára**.

---

## 2c. A terceira wave nova — `W-Glide`, o planeio

**O último item mensurável da fila do plano 08**, e a wave em que a **medição
refutou o que o plano supunha**.

### ⚠️ O plano previa um campo; a medição escolheu outra forma

O §4.6 escreveu a suspeita — *"planar é um multiplicador de gravidade sob botão;
se for isso, não é uma wave, é um campo"* — e mandou conferi-la. As três formas
candidatas, cada uma sendo uma que **este módulo já usa algures**:

| forma | onde vive | no ápice | a subir 8 m/s | a cair 12 m/s |
|---|---|---|---|---|
| **escala** | as fases do pulo | 0 | 0 | 0 — mas **nunca assenta** |
| **alvo** | o `wall_slide` | −2,0 | **−10,0** ⚠️ | +10,0 |
| **TETO** | ⚠️ em lugar nenhum | 0 | 0 | +10,0 |

**A escala nunca ASSENTA:** até `0,05` — 5% da gravidade do mundo — a descida
continua a crescer (**−1,71 / −2,21 / −2,80** a 1 / 3 / 6 m). *Quão depressa se
desce* passa a ser função de *quanto já se caiu*, e um planeio existe para se
saber onde se vai aterrar.

**O alvo INVERTE quem sobe** (`Δv = −10` apertado a subir): não é planar, é um
botão de *descer agora*.

⚠️ **E o motivo de o TETO não estar escrito está no doc do `wall_slide`:** a
versão-teto **dele** foi morta por medição — *"com o atrito default o personagem
não cai, e um teto nunca dispararia"*. Isso é verdade **da PAREDE**. **No ar não
há atrito**, então a objeção não viaja, e a forma que ela matou lá é a certa
aqui.

### A lei

`glide.rs`, sem estado (o molde do `wall_slide`). ⚠️ **O guard é UMA linha a
responder DUAS perguntas:** `rel_up >= −fall_speed` é o teto **e** é o que
garante `delta > 0` — ou seja, que este módulo **nunca consiga acelerar uma
queda**.

⚠️ **Lê o NÍVEL do botão de pulo, não a borda**, e é seguro precisamente por ser
o nível (a advertência do `wall_launch` é sobre a BORDA). Isso o faz **COMPOR**
com o pulo do ar: um toque dá o pulo, um segurar dá o pulo **e depois** o
planeio — Kirby e Yoshi.

### ⚠️ E uma mutação sobreviveu; o gate que a mataria achou OUTRO defeito

Tirar o `standing.is_none()` deixava lei, produto e cena **VERDES**. O gate
nasceu vermelho por um motivo diferente do esperado: **no tique da decolagem a
`standing` já é `None` de propósito** (a perna cala-se para não disputar o eixo
com o `boost` do pulo), então o planeio passava exatamente ali — medido, o motor
levava **`+18,26` do pulo e mais `+10,00`** do planeio. Sexta guarda:
`!jump.takeoff`.

⚠️ **E duas mutações anteriores foram NO-OPS SILENCIOSOS que eu li como
achados** — o `cargo fmt` colapsara a guarda numa linha e o `str.replace` não
casou ([[feedback_python_replace_silent_noop_after_fmt]], que está na memória do
repo). Toda mutação desta wave passou a **asserir a âncora antes de escrever**.

### ⚠️ Três fixtures nasceram a medir outra coisa

- **o vão da cena, 7,00 m**, veio da sonda que larga o personagem **parado**
  (4,18 m). Quem corre de um patamar **e pula** leva a velocidade toda: os DOIS
  atravessavam, e a cena não mostrava falha nenhuma. O número final (**12,00**)
  sai de `where_each_one_crosses_the_landing_level`, que mede **esta geometria**:
  sem planeio **7,18 m**, planando **18,47 m**;
- **o gate da cena media *"onde ele está depois de 360 tiques"*** — o planador
  atravessava, aterrava, e **continuava a andar** até sair pela outra ponta e
  cair; o gate reportava *"ele caiu"* sobre uma travessia bem-sucedida;
- **e *"aterrou"* era um teste de POSIÇÃO**, que um corpo a cair para o poço
  **atravessa** a caminho do fundo.

---

## 2d. A quarta wave — `W-LedgeSensor`, o sensor da beirada ganha POSIÇÃO e EXTENSÃO

> *"Em Ledge Grab precisamos de mais ajuste: pelo menos 3 controles — posições x
> e y e scale do sensor. Mas antes de construir faça uma pesquisa melhor sobre o
> assunto."* — e depois, com foto: *"O grab span deveria dividir o sensor na
> horizontal como na foto? ou os pontos de apoio na vertical? Outra coisa: não
> temos como mover os sensores na vertical."*

**A pesquisa REFUTOU a frase que o `grab` carregava** (*"até onde ele alcança é
uma grandeza só"*): GDevelop expõe `Grab tolerance` **e** `Grab offset` (*"to
match character animation"*), o Corgi configura *"origem e comprimento"*, o
mantle do Unreal paga **três** traços + **cinco** line traces + uma cápsula de
folga, e o hotspot estilo Sonic é um **SEGMENTO**.

**São QUATRO controles, e a matriz é a razão de cada um:**

| | posição | tamanho |
|---|---|---|
| **X** | `Ledge Grab` | `Grab Span` |
| **Y** | **`Grab Offset Y`** | `Grab Window` |

⚠️ **O span é HORIZONTAL, e a razão é o SENTIDO do raio** — a resposta à foto do
Enio: o raio aponta para BAIXO, e um raio para baixo **já integra a janela
vertical inteira num cast** (ele devolve *a que altura o lábio está*). Espalhar
amostras na vertical faria N raios responderem a mesma pergunta. A varredura
vertical é o desenho do Unreal, com traços para a **frente** — descartado de
propósito: um traço para a frente diz *que há parede* e ainda é preciso descobrir
a altura.

⚠️ **E o quarto controle nasceu de um erro MEU, nomeado:** eu chamei o `reach_y`
de *"o Y"* citando o `Grab offset` do GDevelop — **mas aquele é uma POSIÇÃO**, e
o que eu construí com o argumento dele foi o **TAMANHO**. A janela era sempre
centrada no topo do corpo, então alcançar um lábio mais alto custava **alargar a
histerese junto**: uma decisão sobre *quando ele solta* pagando por uma sobre
*onde ele olha*. O plano 08 §4.51 dizia *"é assim que os controles continuam
TRÊS em vez de quatro"* — a frase está **corrigida lá**, não apagada.

⚠️ **O rótulo também mentia:** *"Grab Height"* lê como posição; virou
**`Grab Window`**, com a dica a dizer *quão ALTA é a janela, acima e abaixo*.

⚠️ **O `offset_y` é o único da família SEM `max(0.0)`** — um offset é uma
**direção**, e deslizar para baixo é legítimo (uma arte cuja mão fica abaixo do
topo do collider).

**Duas reduções LITERAIS mantêm o degrau de schema barato:** `span = 0` é **uma**
amostra na posição exacta de antes, e `offset_y = 0` é a janela centrada no topo
da cabeça — o mundo aprovado fica byte-idêntico, e o **`c9` intocado** é quem o
prova.

⚠️ **A rejeição de *"a parede continua acima da cabeça"* deixou de ser grátis:**
ela era de graça enquanto o sensor era um PONTO (a origem cai dentro da geometria
e o cast devolve `distance == 0`). Num leque é feita à mão — **uma amostra dentro
recusa o leque INTEIRO**, que é o traço de folga que o mantle do Unreal paga em
separado.

**A lei do vencedor:** ganha o acerto mais **PERTO** do corpo — aproximando-se de
um patamar, as amostras de dentro caem no vazio e as de fora batem no topo, logo
a mais próxima **é a beirada** —, e é ele que define o `across`, o alvo do mantle.

**6 gates, 5 mutações, 5 sangram.** ⚠️ **E três nasceram sem poder falhar pelo
motivo que alegavam:** o do leque tinha a aritmética da fixture errada (o raio nu
ainda pousava dentro do bloco, e o **CONTROLE** reprovou); o da recusa
SOBREVIVEU **duas vezes** — na v1 todas as amostras nasciam dentro da parede
(onde `return None` e `continue` dão o mesmo resultado) e na v2 o corpo estava no
CHÃO, onde `ledge_probe_wanted` nem casta o sensor. A fixture que morde é: parede
FINA que sobe acima da cabeça + prateleira mais baixa **atrás** dela, com o corpo
no AR. E o gate do offset tem **TRÊS** fixtures porque a terceira é a que separa
*deslizar* de *crescer*: com offset 0,40 o teto sobe **exatamente** 0,40, então um
lábio 1,20 acima continua fora — sem ela o gate ficaria verde sobre um `reach_y`
disfarçado.

---

## 2e. A quinta — **um ACORDE nunca é entrada de jogo**

> *"Os players ficam dando pulinhos discretos sozinhos (sem input)"* — e depois
> *"o players pular e se movem sozinhos"*.

⚠️ **A física foi EXONERADA por medição, não por argumento:** pose bit-constante
ao longo de 1190 tiques pelo binário do produto, e o relógio nunca andou para
trás. Cinco sondas headless, todas negativas — e o resultado honesto de uma sonda
é às vezes *não existe bug aqui*.

**A causa está no caminho da ENTRADA:** `player_keys.key()` observava a tecla
**física** sem guarda de modificador, então **`Ctrl+Z` pulava**, `Ctrl+A` andava
para a esquerda, `Ctrl+D` para a direita e `Ctrl+S` agachava. ⚠️ **E o doc do
`player_input.rs` afirmava o oposto como impossível** — a frase é que estava
errada, não o report.

⚠️ **A SOLTURA passa sempre, e a assimetria é a correção:** uma guarda simétrica
trocaria um pulo espúrio por um personagem que **anda para sempre** (tecla
premida nua, solta com o Ctrl em baixo).

⚠️ **O gate mudou de FORMA:** ele pinava três linhas verbatim, o que reprova no
primeiro reflow; passou a afirmar a **propriedade** (o bloco de observação não
retorna cedo) e ganhou o irmão `a_modifier_chord_never_reaches_the_player`, que
exige as três perguntas de modificador e a linha do guarda.

---

## 2f. A sexta — **a marca de sensor pousa onde o corpo ESTÁ**

> *"Temos um drift dos gizmos dos sensores do platformer ao se mover o Player"*
> (com foto).

**Medido pela porta do produto ANTES de qualquer hipótese** (sonda
`measure_probe_lag`): o leque fica **0,1000 m atrás em regime** — a distância
EXATA que o personagem percorre num tique a 6 m/s —, constante e para sempre.
**Parado é zero**, e é por isso que nenhuma cena de repouso o mostrava; num
arranque a 18 m/s são 0,3 m.

**A causa não é geometria, é QUANDO:** a leitura é gravada **antes** do `step`
(tem de ser — é ela que a lei consome para decidir o tique) e o `readback`
publica a pose **depois**. O overlay desenhava a cápsula no fim do tique e o
leque no começo dele.

⚠️ **A cura é a ÂNCORA, não um re-cast.** Re-perguntar ao mundo depois do passo
seria a **segunda resposta** a *"o que este sensor viu neste tique?"*, e ela
discordaria da que a lei usou — a única que produziu movimento. O leque é rígido
em relação ao corpo, então o `readback` o desloca pelo que o corpo andou
(`reanchor_player_probes`), mantendo `hit`, `reach` e `skin` **exactamente** como
foram medidos. **No `readback` e não em cada ramo:** é o ponto por onde as DUAS
rotas que dão passo passam (o laço de tiques e o replay do rewind).

⚠️ **O `Sweep` não é tocado, e a assimetria é a prova de que o diagnóstico está
certo:** ele já viaja como `(corpo, deslocamento)` e o overlay o resolve contra a
pose viva — **por isso ele nunca driftou**. Deslocá-lo aqui somaria o movimento
do corpo a um número que já é relativo a ele.

**Resultado: 0,1000 → 0,0000 m.**

**Gate `the_published_fan_rides_the_body_it_belongs_to`:** o oráculo é a
**RELAÇÃO** (a distância do leque ao corpo não muda por o corpo estar em
movimento), não uma coordenada — um gate que cravasse a origem em mundo teria de
saber onde a perna nasce. **Parado é o CONTROLE**, e cada amostra em movimento
afirma primeiro que o corpo de facto **ANDOU** naquele tique: uma fixture parada
passaria sem conter o fenômeno.

**5 mutações:** sem a re-âncora (sangra com o número) · deslocar o `Sweep`
(sangra a assimetria) · um 2º leitor de `got.grounded` (sangra o arch-gate da
família) · afastar a aplicação da dedup (sangra a adjacência). ⚠️ **A quinta
SOBREVIVE e está documentada como higiene, não vendida como defesa:** o `clear`
das faixas no `preview` não pode ser observado, porque o `reanchor` as DRENA e o
`drive_players` as limpa antes de as reencher.

---

## 3. Superfície de colisão

| item | valor | nota |
|---|---|---|
| `PROJECT_SCHEMA` | **70 → 78** | ⚠️ **oito degraus**, ver §4 |
| tripla do pin | `(78, 13, 14)` | `project_schema_tests.rs` |
| `physics_ecs_c9` | **`1699123f9ed2844f…`, 117 corpos** | debug ≡ release, medido no tip. ⚠️ **NÃO se move com NENHUMA das seis waves** |
| registro `ph2d-physics-ecs` | **29, INTOCADO** | nenhum componente novo |
| registro `ph2d-ecs` + os 2 espelhos | **INTOCADOS** | |
| gizmo ids | **nenhum novo** (o último segue **973**, próximo livre **974**) | |
| ids novos | **20, todos `hash_node_id`** | ⇒ fora de todo gate de contagem |
| ADR | **nenhum** | ⇒ a linha fica **fora de toda disputa de número** |
| `Cargo.toml` / `Cargo.lock` | **ZERO** | nenhuma crate nova, nenhuma dep nova |
| `.typos.toml` | **+1 entrada** (`^PILAR$`) | ⚠️ lista COMPARTILHADA — ver abaixo |
| contrato congelado | **4/4** | rodado, não auto-relatado |
| `PLAYER_ROW_COUNT` | **42 → 50** | +2 do pulo do ar, +2 da beirada, +1 do planeio, **+3 do sensor da beirada** |
| `PLAYER_CARDS` | **9 → 11** | ⚠️ o card `GLIDE` é o primeiro de UMA row |
| cenas de smoke | maior **112** (próxima livre **113**) | ⚠️ o `=84` não existe, de propósito |

⚠️ **O `c9` intocado é a PROVA de que os degraus v74..v78 não movem física.** As
três capacidades nascem DESLIGADAS (`air_jumps = 0` · `ledge_grab = 0` ·
`glide_fall_speed = 0`), e os dois degraus do sensor da beirada **reduzem
LITERALMENTE** ao raio único (`span = 0` · `offset_y = 0`) — então nenhuma lane do
hash os exercita, e o hash é o único oráculo que não depende de eu afirmar coisa
alguma. ⚠️ **A ÂNCORA (§2f) também não o move, e ali a razão é outra:** ela é
DESENHO — a lista de marcas nunca é lida pelo solver.

⚠️ **E um GATE COMPARTILHADO mudou de afirmação:** o
`every_card_holds_its_own_rows_and_they_do_not_overlap` (§14) afirmava `hi > lo`
sobre os `y` das rows de um card — **impossível de satisfazer por um card de UMA
row**, e o `GLIDE` é o primeiro deste painel. A troca por **DISTINÇÃO** não é
concessão: o `hi > lo` também passava com três rows das quais **duas**
partilhassem o `y` (só apanhava o colapso total).

⚠️ **O único arquivo COMPARTILHADO que a linha toca é o `.typos.toml`**, e o
toque é **puramente ADITIVO**: uma entrada (`"^PILAR$"`) logo abaixo da
`"^pilar$"` que já existia. As entradas são regexes **sensíveis a maiúsculas**, e
este codebase usa caixa alta para ênfase em doc-comment, então a minúscula não a
cobria. ⚠️ **Entrada nova em vez de um `(?i)` na linha de cima**, de propósito:
reescrever uma linha de lista compartilhada é como um merge perde, em silêncio, o
que outra linha lhe acrescentou.

⚠️ **Os dois vermelhos de `typos` que isto drena eram MEUS** (o `main` está
verde, e os dois arquivos que disparavam são novos desta linha, da wave
`W-ShapeCast`) — achados só no gate de fechamento, porque o `typos` é
project-wide e um `cargo test -p` não o alcança.

⚠️ **E a linha cria ZERO ADR** (`git diff --name-only main...HEAD -- docs/architecture/decisions/`
é **vazio**) ⇒ ela fica fora de toda disputa de número desta janela.

---

## 4. Os degraus de schema

Os três primeiros estão no handoff de 08-11 (§4 de lá): **v71** os três campos
do nado · **v72** os quatro números dos sensores · **v73** os dois da perna em
leque.

**v74 (`W-MultiJump`):** o `PlatformPlayer` ganhou `air_jumps` +
`air_jump_height`, **no MEIO do struct** (logo depois do `jump_height`, que é
onde eles se leem), e o postcard é posicional ⇒ **quebra dura**.

⚠️ **Este degrau NÃO move física:** a contagem nasce em `0`, que é a capacidade
DESLIGADA — o precedente do wall slide e do wall jump —, então um projeto salvo
em v73 reabre com o pulo exatamente como estava. É o oposto do v73, que move (e
o handoff de 08-11 diz por quê).

**v75 (`W-Ledge`):** o `PlatformPlayer` ganhou `ledge_grab` + `ledge_speed`,
também no MEIO do struct ⇒ **quebra dura**.

⚠️ **Este degrau também NÃO move física:** o alcance nasce em `0`, que é a
capacidade desligada, então um projeto salvo em v74 reabre exatamente como
estava — e o `c9` intocado é quem o prova.

**v76 (`W-Glide`):** o `PlatformPlayer` ganhou `glide_fall_speed`, apendado ao
FIM ⇒ **quebra dura**. ⚠️ **Também NÃO move física** — o teto nasce em `0`.

**v77 e v78 (`W-LedgeSensor`):** o `PlatformPlayer` ganhou `ledge_reach_y` +
`ledge_span` (v77) e `ledge_offset_y` (v78), **no MEIO do struct** (junto dos
outros dois números da beirada, que é onde se leem) ⇒ **quebra dura**.

⚠️ **Os dois NÃO movem física, e a razão é diferente da dos irmãos:** aqui a
capacidade não nasce desligada — ela nasce **reduzida ao raio único**
(`span = 0` é uma amostra na posição exacta de antes; `offset_y = 0` é a janela
centrada no topo da cabeça). É por isso que a beirada aprovada no smoke da cena
`=111` continua byte-idêntica, e o **`c9` intocado** é quem o prova.

⚠️ **PROVISÓRIO — o valor se CONTA contra o `main` do dia.** Três linhas já
colidiram neste número por o terem *escolhido*, e da última vez o certo não
estava em nenhum dos dois lados. ⚠️ **E a colisão passa MUDA quando as duas
linhas escrevem o MESMO literal:** o `project.rs` não conflita e o git não sabe o
que o número significa — confira nos **DOIS** arquivos (`project.rs` **e**
`project_schema_tests.rs`).

---

## 5. LOC — os três cortes

### O corte de `jump.rs` (`W-MultiJump`)

`jump.rs` estava em **676** e a wave o levou a **796 > 700**. Corte por
**RESPONSABILIDADE**, não por tamanho: **`jump_config.rs`** leva *o que o artista
AUTORA* (o `JumpConfig` e o `STARTING_POINT`, quase inteiramente doc-comments com
as tabelas que escolheram cada número) e o pai fica com *o que acontece num
TIQUE* (`JumpState` / `JumpStep` / `jump_step`).

Re-exportado pelo pai (`pub use jump_config::JumpConfig`) ⇒ **nenhum caminho de
chamador muda**. Os dois arquivos ficam em **584** e **231**.

⚠️ **Um filho de `src/jump.rs` precisa de `#[path]`** — sem ele o compilador
procura `src/jump/jump_config.rs`; é a convenção que `player_leg.rs` e
`height_modes.rs` já seguem.

### E o corte de `player_probes.rs` (`W-Ledge`)

A beirada levou-o a **709 > 700**. Corte pela mesma linha: o que **PERGUNTA** ao
mundo fica no pai (os raios, as varreduras, o `probe_ledge`), e o que **RELATA o
que foi visto** sai para **`player_marks.rs`** (`record_marks` +
`preview_player_probes`, 272 linhas).

⚠️ **O `record_marks` recebe a beirada em DOIS níveis** (`Option<Option<&LedgeProbe>>`)
— o padrão que a perna já usava: `None` é *"a lei não perguntou"* e `Some(None)`
é *"perguntou e não há lábio"*. Colapsá-los faz o marcador do sensor desaparecer
da tela exatamente quando ele é mais útil (o artista a mirar uma beirada e a
falhar).

### E o corte de `player.rs` (a ÂNCORA)

A faixa de âncoras levou-o a **708 > 700**. Corte pela mesma linha das outras
duas: o laço **COLHE** (o cast toma `&self`) e a ponte **APLICA** (a pose toma
`&mut self`) — a lista de `KinMove` já existia por essa razão, e o que muda é que
a metade que a consome ganhou casa própria em **`player_kinmove.rs`** (94
linhas), em vez de ser a cauda de um laço de quinhentas.

⚠️ **E o corte fez DOIS arch-gates reprovarem sobre produto CORRETO** — os dois
ancorados no ENDEREÇO `src/bridge/player.rs`, de onde o `kinematic_settle` e o
`push_from_hits` se mudaram: `the_law_asks_footing_not_the_controller` (o próprio
CONTROLE dele disparou, que é para o que ele existe) e `the_push_is_our_law`. Os
dois passaram a ler a **FAMÍLIA** (`player.rs` + `player_*.rs`) por uma porta
única nova — **`tests/player_bridge_source.rs`**, incluída por `#[path]` nos dois,
porque duas cópias de *"que texto é a ponte do player?"* divergiriam no primeiro
corte que uma delas não visse.

⚠️ **Quem afirma ORDEM DENTRO de uma função continua a ler só o arquivo do
laço** (`player_loop()`): ordem entre dois pontos de arquivos diferentes seria a
ordem alfabética dos irmãos, não uma propriedade do produto. *Afirme a
PROPRIEDADE, nunca o endereço* — e as duas perguntas são diferentes.

⚠️ **Provado que a varredura da família não ficou vazia:** um segundo leitor de
`got.grounded` no irmão faz o contador sangrar (`achei 2`), e afastar a aplicação
da dedup faz a adjacência sangrar (`3513 bytes depois`).

---

## 6. Gates e mutações

**10 gates novos de lei/produto** + **4 de cena** + as duas rows na varredura de
seam.

- `ph2d-platformer/src/jump_air_tests.rs` (7) — a carga · a altura própria · a
  recarga junto com o coyote · o pulo que não empurra · a parede que não gasta ·
  um aperto uma carga · o controle com `air_jumps = 0`.
- `ph2d-platformer/src/lib_dash_tests.rs` (1) — o **terceiro** cancelamento.
- `ph2d-physics-ecs/tests/player_multi_jump.rs` (3) — pela porta do produto.
- `shells/desktop/src/physics_smoke_multi_jump_tests.rs` (4) — a aritmética em
  tempo de COMPILAÇÃO · o contraste · o controle · **o pouso**.

**7 mutações, 7 sangram:** a feature inteira · o buffer não consumido · a
recarga no ar · a altura errada · empurrar o chão · o ar preceder a parede · o
proxy antigo de volta.

### Da `W-Ledge`

- `ph2d-platformer/src/ledge_tests.rs` (13) — o controle com a beirada desligada
  (o mundo de antes desta wave) · apanhar PARA a queda · o servo leva o TOPO ao
  lábio · os dois limiares · largar é a ausência do gesto · a subida sobe antes
  de atravessar · não deixa nada para trás · nada a interrompe · SUBSTITUI a
  velocidade em vez de somar · o sensor só é pedido onde a lei pode agir · o
  pulo que o trouxe não o faz subir.
- `ph2d-physics-ecs/tests/player_ledge.rs` (7) — pela porta do produto.
- `shells/desktop/src/physics_smoke_ledge_tests.rs` — a aritmética das alturas em
  tempo de COMPILAÇÃO contra os números MEDIDOS do pulo colado à parede.

**11 mutações, 11 sangram:** o `distance == 0` que recusa a parede · os dois
limiares colapsados · disparo por nível · atravessar antes de subir · a subida
deixar de ser comprometida · o motor SOMAR em vez de substituir · a travessia
esquecer a meia-largura de dentro · a subida esquecer a `float_height` · a LEI
recusar no chão · a PORTA do sensor recusar no chão · o cancelamento de
gravidade.

### ⚠️ Três coisas que as mutações acharam nesta wave, e ficam escritas

**(a) As duas camadas de *"no chão não há beirada"* têm cada uma o seu gate,** e
foi conferido em separado: a da LEI sangra o `letting_go_is_the_absence_of_the_gesture`,
a da PORTA do sensor (a camada de CUSTO) sangra o
`the_probe_is_only_wanted_where_the_law_can_act`. Gates diferentes ⇒ a política
de [[feedback_layered_defenses_need_per_layer_gates]] está honrada.

**(b) A mutação do cancelamento de gravidade SOBREVIVEU, e a cura foi um gate
NOVO — não uma desculpa.** Tirar a beirada do `gravity_hold` deixava treze gates
da lei e cinco dos seis do produto **VERDES**, porque o PENDURAR não consegue
medir esse termo: o servo re-mira em todo tique a partir da velocidade VIVA,
então a gravidade de um tique é absorvida pelo seguinte e o assentamento move
**0,1 mm**. A SUBIDA é o outro regime — alvo CONSTANTE —, e lá o termo vale
**1,011× o autorado com ele contra 1,048× sem** (`ledge_speed = 2,0`, dez
tiques). Gate novo `the_climb_walks_at_the_speed_the_artist_wrote` + sonda
`measure_whether_the_climb_walks_at_the_authored_speed`.

**(c) Colapsar os dois limiares NÃO sangra o produto, e isso é observação
honesta, não buraco.** Os gates da LEI sangram; os de produto ficam verdes porque
a trepidação que o colapso introduz cabe dentro da tolerância de 2 cm com que o
oráculo de POSE mede o assentamento. O oráculo está certo (o que o jogador vê é
onde ele pára), e a lei tem os seus próprios gates.

### Da `W-Glide`

- `ph2d-platformer/src/glide_tests.rs` (7) — a lei.
- `ph2d-platformer/src/lib_glide_tests.rs` (5) — as guardas de **COMPOSIÇÃO**,
  e ⚠️ **é este o nível certo para as testar**: duas fixtures de água inteiras
  foram construídas e as duas mediram outra coisa (uma deixava o nadador SAIR da
  água, a outra media a velocidade de CHEGADA ao mergulho — as duas legítimas).
  Uma guarda de composição pergunta *quem escreve o eixo*, e quem responde é a
  porta que compõe.
- `ph2d-physics-ecs/tests/player_glide.rs` (5) — pela porta do produto.
- `ph2d-physics-ecs/tests/player_swims.rs` (+1) — o planeio na água de verdade.
- `shells/desktop/src/physics_smoke_glide_tests.rs` (4) + a sonda que dimensiona
  o vão.

**7 mutações, 7 sangram:** o teto vira alvo · o teto cravado · ignorar o botão ·
a guarda do nado · a da perna · a da decolagem · nunca compor.

### ⚠️ Duas coisas que as mutações acharam em MIM, e ficam escritas

**(a) O gate novo do arranque nasceu SEM DENTES.** A fixture punha o personagem
a CAIR, e quem nunca pulou entra com `airborne` **falso** — ali o proxy antigo
acerta por acidente. Medido: sem `state.jump.airborne = true` a mutação do proxy
**não sangra**; com ela, sangra sozinha.

**(b) Eu escrevi uma afirmação FALSA num doc de gate.** O gate de PRODUTO dizia
ser o guardião do consumo do buffer, e ele fica **VERDE** sob aquela mutação —
naquele caminho a decolagem do CHÃO já zerou o buffer antes de o personagem
chegar ao ar, então o aperto guardado nunca alcança o ramo do ar. Quem apanha é o
irmão de unidade, cuja fixture entra **já no ar** com uma borda fresca. Doc
corrigido em vez de contrabandeado.

### Da `W-LedgeSensor`

- `ph2d-physics-ecs/tests/player_ledge_sensor.rs` (6) — o leque acha o lábio que
  um raio erra · uma amostra DENTRO recusa o leque inteiro · o vencedor é o mais
  PERTO · o offset DESLIZA sem redimensionar · e os dois **CONTROLES** de redução
  literal (`span = 0` e `offset_y = 0` reproduzem o raio único).
- `ph2d-panel-inspector/tests/seam_player.rs` — as três rows novas na varredura.

**5 mutações, 5 sangram** — e a da recusa **sobreviveu duas vezes por FIXTURE**
(§2d).

### Da ÂNCORA (§2f)

- `ph2d-physics-ecs/tests/player_probe_view.rs` (+1) —
  `the_published_fan_rides_the_body_it_belongs_to`, com o repouso como CONTROLE e
  a asserção de que o corpo ANDOU no tique medido.
- `ph2d-physics-ecs/tests/player_bridge_source.rs` — a porta única do texto da
  ponte, partilhada pelos dois arch-gates (§5).
- Sonda `measure_probe_lag` (`--ignored --nocapture`) — a tabela que atribuiu o
  defeito e mede a cura.

**5 mutações, 4 sangram; a 5ª está documentada como higiene** (§2f).

---

## 7. Como rodar o gate

```
cargo test -p ph2d-platformer -p ph2d-physics-ecs -p ph2d-panel-inspector --no-fail-fast
cargo test -p ph2d-host-desktop --no-fail-fast
cargo run -q -p ph2d-physics-ecs --bin physics_ecs_c9 --release   # e sem --release
```

⚠️ **`--no-fail-fast` não é preferência:** sem ele o primeiro binário vermelho
esconde o resto, e na jornada de 08-11 a diferença foi entre *"um gate caiu"* e
*"dez caíram"*.

---

## 8. Smokes

| cena | o quê |
|---|---|
| `PH2D_PHYSICS_SMOKE=105` | **nadar** |
| `=106` | a **correnteza** nos três modos |
| `=107` | as quatro **pedras** que não cabem num raio |
| `=108` | **o que ele vê** — os cinco sensores |
| `=109` | **A FENDA** — o leque, com o controle DENTRO do quadro ✅ **aprovado 2026-08-12** |
| `=110` | **A PRATELEIRA ALTA** — o pulo do ar |
| `=111` | **O PARAPEITO** — a beirada |
| **`=112`** | **O VÃO** — o planeio |

⚠️ **A cena 110 em uma frase:** duas raias iguais, dois personagens iguais, e o
teclado dirige **os dois ao mesmo tempo** (`hand_input_to_players` entrega a
todo `PlatformPlayer`) — então um gesto só move os dois lado a lado, e o controle
está **dentro do quadro**. A prateleira **baixa (1,5 m)** cabe num pulo e prova
que os DOIS sabem pular; a **alta (3,0 m)** cai no vão entre 1,903 e 4,028, então
só o da direita a alcança.

⚠️ **A cena 111 em uma frase:** a mesma forma da 110 — duas raias iguais, dois
personagens iguais, o teclado a dirigir os dois — e a única diferença entre eles
é **o alcance do braço** (0 à esquerda, 0,60 m à direita). O patamar **baixo
(1,0 m)** cabe num pulo e prova que os DOIS sabem pular; o **alto (2,4 m)** fica
acima dos 2,145 m que o topo do corpo alcança **colado à parede**, então só o da
direita o alcança — e ele fica **pendurado**, não em cima.

⚠️ **O passo 8 é o que fecha a lei:** com a beirada armada, andar contra o
patamar BAIXO **no chão**, sem pular, **não** deve pendurar — um degrau que se
sobe a pé não é beirada.

⚠️ **E o passo 9 é o do SENSOR** (§2d): os quatro controles, um a um —
`Grab Span` alarga o leque na horizontal · `Grab Window` muda quão ALTA é a
janela · **`Grab Offset Y` DESLIZA o sensor na vertical sem o redimensionar**
(suba para 0,40 e ele alcança lábios mais altos **sem** que a histerese do
pendurar mude) · `Ledge Grab` é o alcance à frente.

⚠️ **A cena 112 em uma frase:** a mesma forma das duas anteriores — duas raias
iguais, dois personagens iguais, um teclado — e a única diferença é o **teto de
descida** (0 à esquerda, 2,00 m/s à direita). O vão de **12,00 m** fica entre os
**7,18** que o sem-planeio atravessa e os **18,47** do planador, então um cai no
poço e o outro aterra, **com o mesmo gesto**.

⚠️ **O passo 5 é o que fecha a lei:** no chão, segurar o pulo e pular tem de dar
a **MESMA altura** nas duas raias — o planeio nunca empurra para baixo, então não
pode encolher uma subida. *Se o da direita pular mais baixo, PARE.*

**Próxima cena livre: `113`** (o `=84` não existe, de propósito).

---

## 9. Aberto, com o preço ao lado

Os itens do handoff de 08-11 continuam abertos e **não foram tocados**
(`min_float_height` conservadora · a metade *"um degrau íngreme não é chão"* sem
fixture de unidade).

Da wave nova:

- **A carga não é VISÍVEL na tela**, e é decisão, não esquecimento: um contador
  de pulos restantes é um **HUD**, e este app não tem um. A metade visível desta
  wave é o próprio comportamento — o personagem pula uma segunda vez, e a cena
  `=110` é onde isso se julga. Um readout na §14 seria um número que o artista
  não pode usar enquanto joga.
- **Um pulo do ar não zera o `wall_lock`.** Depois de um pulo de parede o
  controle aéreo fica calado por `jump_lockout` (0,2 s), e um pulo do ar dentro
  dessa janela não o encurta. É pequeno (o relógio corre de qualquer forma) e
  **não foi medido**: se o smoke mostrar que atrapalha, o lugar é o mesmo braço
  que já zera o coyote.
Da `W-Ledge`:

- **Não há alça de canvas para o alcance**, e é a mesma decisão do contador de
  pulos: o que se julga é o comportamento, e a cena `=111` é onde ele se julga.
  As duas rows na §14 são a autoria.
- **A beirada não interage com o agarrar-se à parede** (`W23`, o
  `wall_grab_stamina`): quem tem os dois armados agarra a beirada assim que ela
  entra na janela, porque a lei da beirada corre antes. **Não foi medido** se
  isso incomoda — se o smoke mostrar que sim, o lugar é a precedência, não um
  knob novo.
- **Um lábio em movimento** (uma plataforma que sobe) não foi exercitado. O
  sensor é re-lançado em todo tique, então a lei deve seguir — mas *deve* não é
  *foi medido*.

Da `W-Glide`:

- **Não há knob de DURAÇÃO**, e é decisão: um regime dura enquanto o dedo dura, e
  um segundo número responderia o que o botão já responde.
- **O planeio não muda o controle aéreo**, então o alcance vem só do tempo a mais
  no ar. Se o smoke pedir mais travessia, o lugar é o controle aéreo — não um
  segundo número no planeio.
- **Um planador não é travado por uma parede** (`clinging.is_none()` cala-o ali,
  e quem manda é o `wall_slide`). Não foi medido se incomoda.

Da `W-LedgeSensor`:

- **Não há alça de canvas para nenhum dos quatro** — a mesma decisão das outras
  waves: o que se julga é o comportamento, e o **overlay do sensor** (`W-Probes`)
  já desenha o leque onde ele está. As quatro rows na §14 são a autoria.
- **O span não tem teto**, e é deliberado: um leque largo custa `n` casts e o
  custo já está medido (18 ns por raio); um teto seria um número sem recurso por
  trás.

Do ACORDE (§2e):

- **A guarda é do PLAYER, não do app inteiro.** Um acorde continua a chegar a
  toda a outra ferramenta, porque ali ele É o gesto (Ctrl+Z é o undo). O que
  mudou é que ele deixou de virar *entrada de jogo* — e um atalho de jogo que
  seja **Shift**+tecla continua a funcionar, porque o Shift não é acorde.

Da ÂNCORA (§2f):

- **O leque desenhado descreve o tique ANTERIOR**, e continua a descrever: o que
  a wave corrige é ONDE ele pousa, não QUANDO ele foi medido. As distâncias
  (`hit`, `reach`, `skin`) são as que a lei consumiu — re-castá-las seria a
  segunda resposta que o §2f recusa.

- **A fila do plano 08 acabou:** ~~`W-Ledge`~~ ✅ → ~~`W-Glide`~~ ✅ → *o ajuste*
  (§4.7, que é **decisão de aparência e o instrumento é o smoke**, não uma wave).
