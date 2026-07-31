# BUGS da Timeline — os cuja CAUSA enganava

> Irmão do [`BUGS_physics.md`](../Physics/BUGS_physics.md) e do
> [`BUGS_painter.md`](../Painter/BUGS_painter.md). Aqui entra o bug cuja causa não era a
> que a aparência sugeria — para ninguém re-derivar o diagnóstico, e para o repro virar
> gate. Bug de rotina fecha no commit e não precisa de linha aqui.

---

## #1 — O `lead_out` do ÚLTIMO strip de uma lane era INERTE (2026-07-30)

**Report (Enio, com screenshot):** *"em Arrange na lane 2 temos um fade do lado direito
para fora e ele não funcionou corretamente."*

**A aparência sugeria** que o fade estivesse desligado, ou que o peso não fosse calculado.
**Medido pelo apply real, era o contrário:** o peso rampava perfeitamente e a pose não
andava.

```
t      weight_at   hold_at                  x pintado
3.000    1.000     None                     +10.000
3.500    0.500     strip[1..3] w=0.500      +10.000   ← o peso caiu pela metade
4.000    0.000     strip[1..3] w=1.000      +10.000   ← e a pose não se moveu
4.125                                        -5.000   ← degrau
```

**Causa:** o `hold_at` deixava o strip que **acabou de terminar** responder *"algo ainda
vem"* sobre **si mesmo**, por duas metades independentes —

1. `lead_end() > t`: um `lead_out` ESTENDE o strip para além do `t_end`;
2. a isenção da borda inclusiva perguntava só `blend_out <= 0`, que é o blend de
   **SOBREPOSIÇÃO** — e um fade para FORA não tem sobreposição nenhuma, então ela valia
   para a janela inteira do `lead_out`.

O hold devolvia então a pose congelada DELE com peso `1 − w`, a cobertura da lane voltava a
exatamente **1**, e a fórmula do fade **cruzava contra ela mesma**. O fade só MOVIA o corte
por `lead_out` segundos.

⚠️ **Com uma PRÓXIMA strip nada disto aparecia** — o braço `fade_out_target` dispara antes
e a travessia sempre funcionou (+10 → +30, suave), com gate próprio verde
(`a_lead_out_plays_the_clip_fully_then_fades_in_the_gap`). O defeito vivia **só no último
strip de uma lane**, onde o que está do outro lado do fade são **as lanes de baixo**.

**Gate:** `a_lead_out_on_the_last_strip_fades_to_the_lane_below` (`gap_fade_out.rs`) —
oráculo de **MONOTONIA** mais o ponto médio EXATO da smoothstep, nunca um endpoint: um
endpoint sozinho fica verde sobre um degrau. 2 mutações, 2 sangram.

**Não era regressão** da remoção da autoria de expressões: a mesma sonda num worktree em
`HEAD~2` dá a tabela idêntica, e os dois `fade_fingerprint` seguem verdes com o MESMO hash.

---

## #2 — A trajetória de um clip aparecia (e era agarrável) em outro (2026-07-30)

**Report (Enio):** *"O Path criado em um Clip contamina e aparece alças em outro Clip
criado depois."*

**São DOIS defeitos com a mesma raiz**, os dois fechados — o segundo por uma mudança de
MODELO, depois de a primeira tentativa ser reprovada na tela.

### 2a. As alças invisíveis — FECHADO

O caminho mora no **BINDING**, que é do DOCUMENTO; as keys moram no **CLIP**. O `marks` (o
desenho) perguntava pela track do clip ATIVO e devolvia zero marcas — mas o `anchor_screen`,
o `tangent_screen` e o `motion_path_curve_hit` liam só `b.path`. Medido num clip novo e
vazio:

```
clip A (ativo, keyado)      marks=6  ancoras=2  alcas=2  agarravel=SIM
clip B (novo, VAZIO)        marks=0  ancoras=2  alcas=2  agarravel=SIM   ← nada desenhado,
clip B (com UMA key)        marks=4  ancoras=2  alcas=2  agarravel=SIM      tudo agarrável
```

Um clique sobre uma alça **invisível** pegava e arrastava a trajetória do OUTRO clip, e o
duplo-clique inseria âncora numa curva que ninguém via.

⚠️ **A ironia que nomeia o defeito:** o doc-comment do `anchor_screen` já declarava ser *"a
porta ÚNICA … quem PINTA e quem faz HIT-TEST têm de concordar sobre ONDE a âncora está"* —
e as duas metades discordavam sobre algo anterior: **se ela existe neste clip**.

**Fix:** `active_path` — a pergunta *"o clip ativo tem key para este alvo?"* feita UMA vez,
com os quatro consumidores passando por ela. **Gate:**
`a_clip_that_does_not_animate_the_path_shows_no_handles`, com controle positivo (o clip A
tem marcas, âncoras E alças) e o ponto sobre a curva **pedido ao produto**, não chutado.
3 mutações, 3 sangram.

⚠️ Duas metades desse gate nasceram VERDES sobre nada: o `doc_with` monta o caminho com
`PathAnchor::corner`, cujas alças têm comprimento zero e o `tangent_screen` PULA — e o
ponto de duplo-clique que eu chutei caía fora do raio de pega, então devolvia `None` nos
DOIS clips. *A fixture tem de conter o fenômeno.*

### 2b. O TRILHO COMPARTILHADO foi CONSTRUÍDO e REPROVADO NA TELA — a trajetória é do CLIP

Assim que o clip B ganha **uma key qualquer**, o `marks` passa e o que se desenha é a
trajetória do clip A. Isso não era o overlay: era o **modelo**. O `binding.rs` defendia o
armazenamento no documento —

> *"a trajetória é propriedade do MOVIMENTO deste objeto, não da leitura de um clip: dois
> clips que a animam são duas CRONOMETRAGENS da mesma jornada"*

— e a linha seguinte do MESMO doc-comment dizia:

> *"âncora `i` pareia com a key `i` da track."*

**As duas só podem ser verdade enquanto existir UMA cronometragem**, porque a track é
por-CLIP. E o produto já sabia: apertar K num clip criado depois disparava o `debug_assert`
do `rewrite_path_key_values` — *"a track tem 1 keys para 3 âncoras"*. Em release o assert
não dispara e o `zip` deixa a key de CHEGADA com a distância de uma âncora do MEIO: *"o
percurso do objeto encolhe"*.

#### A tentativa B — construída, e reprovada de OLHO

A primeira escolha do Enio foi **B**: manter a trajetória no documento e fazer dela um
**TRILHO compartilhado** — *quem lança o trilho é a única cronometragem que existe; a partir
da segunda o K keya PROGRESSO ao longo do que já existe*. Foi construída inteira (porta
única `active_clip_authors_the_rail`, três camadas, três gates, três mutações sangrando) e
**reprovada no smoke, com foto**:

> *"não prestou. Veja clip 1 com o path. Ao criar Clip 2 tudo buga, alças em path fantasma
> aparecem, não consigo criar keys onde quero. Cada clip novo deve ser um branco e criar do
> zero seu próprio PATH"* (Enio, 2026-07-30)

⚠️ **Os dois sintomas eram a lei de B funcionando como especificada**, e é por isso que
nenhum gate os pegou: com uma key qualquer no clip 2, o `marks` passa a desenhar — a
trajetória do clip 1, que é a única que existe — e o K, obedecendo, projeta a pose no trilho
alheio e grava uma DISTÂNCIA em vez de pôr a âncora onde o dedo clicou. *"Não consigo criar
keys onde quero"* é a frase exata do braço de progresso.

⚠️ **A opção A (caminho por-clip) tinha sido MEDIDA e adiada com três custos**, que ficam
registrados porque **o primeiro e o segundo continuam abertos**:

1. **"Distância" só significa alguma coisa sobre UMA curva.** O `prop.rs` diz: *"blending
   DISTANCES is what keeps a crossfade ON the trajectory … blending the two POINTS would cut
   the corner off it."* Com uma curva por clip, um crossfade entre trajetórias DIFERENTES não
   tem arco comum para compor.
2. **Isso muda o que uma lane ADITIVA de Position significa** — o `algebra()` a define como
   *"go further along it"*, e sobre curvas diferentes isso não se pode dizer.
3. **O `fade_fingerprint_channels` é exatamente essa cena** (clip A em distância 2, clip B em
   14, com sobreposição, sobre um L de 10+6; meio caminho é 8, **a quina**) — ⚠️ **e este
   custo NÃO se materializou:** os dois clips do fixture percorrem a MESMA jornada, então a
   composição segue em distância sobre a mesma curva e o hash `0x69dca8811eb0f8f8` ficou
   **byte-idêntico**. O custo era real e a medição o dissolveu.

#### O que shipou — e como os dois primeiros custos foram pagos

A trajetória mudou-se para o **CLIP**: `NamedClip.paths: BTreeMap<AnimTarget, MotionPath>`,
e `TargetBinding.path` **morreu**. `DOC_VERSION` **16 → 17** — o segundo bump desta escada
que TIRA um campo em vez de só acrescentar (o outro foi o v8 do nesting), então um v16 não
fica curto: os bytes dele significam outra coisa a partir dali, e o load recusa.

**Duas portas, e a distinção é a espinha da wave:**

| porta | quem pergunta | tem recuo? |
|---|---|---|
| `clip_path(clip, target)` | o **DESENHO** e o hit-test do canvas | **não** — um clip sem trajetória não tem alça a mostrar nem curva a agarrar |
| `path_for(target)` | o **AVALIADOR** (distância → ponto) e as conversões | **sim** — clip ativo, e como recuo o primeiro clip que tenha uma |

⚠️ **O recuo é o que paga os custos 1 e 2 sem tocar o blend.** Sob o Arrange o clip ativo é o
que o dropdown selecionou e pode ser um que nunca autorou trajetória; sem ele, um documento
autorado no clip 1 e composto no Arrange pararia de mapear distância→ponto e o objeto iria
para a origem. E ⚠️ **o limite fica NOMEADO em vez de escondido:** o blend compõe DISTÂNCIAS,
que só têm significado sobre UMA curva, então um crossfade entre dois clips de trajetórias
DIFERENTES percorre a que o `path_for` escolher. **Compor PONTOS é wave própria** — e não é
regressão: antes desta wave existia uma trajetória só, para o documento inteiro.

**As camadas, cada uma com gate PRÓPRIO** (a ausência tem várias causas independentes e uma
só ficaria verde sobre as outras):

| camada | o que ela garante | gate |
|---|---|---|
| `NamedClip::paths` nasce vazio | um clip novo é BRANCO | `a_clip_created_later_starts_with_no_path_at_all` |
| `add_path_key` instala no clip ATIVO | o K constrói a trajetória DELE, do zero | `the_first_key_of_a_new_clip_builds_that_clips_own_path` |
| `active_path_mut` é do clip ATIVO | reformar uma não toca a outra | `reshaping_one_clips_path_leaves_the_others_untouched` |
| `duplicate_clip` clona `paths` | a única forma de partir de um trilho pronto | `duplicating_a_clip_copies_its_trajectory` |
| o recuo do `path_for` | a composição não perde a trajetória | `the_evaluator_still_finds_the_trajectory_from_a_clip_that_has_none` |
| `unbind` leva a geometria junto | nada de curva órfã no arquivo | `unbinding_a_position_track_forgets_that_clips_trajectory` |
| `active_path` do overlay lê a porta CRUA | sem alça fantasma | `a_clip_that_does_not_animate_the_path_shows_no_handles` |

**6 mutações, 6 sangram** — e a que nomeia a wave inteira é trocar o `clip_path` do overlay
pelo `path_for`: a alça fantasma da foto volta na hora.

⚠️ **`PROJECT_SCHEMA` fica onde está** (37): o `TimelineDoc` viaja como blob DENTRO do
`ProjectFile` e carrega a própria versão — a forma do `ProjectFile` não mudou.

⚠️ **E os dois `fade_fingerprint` saíram VERDES com o MESMO hash**, antes e depois — a prova
executável de que o sistema de fade não foi tocado.

### 2c. E o Arrange passou a tocar UM caminho só — a trajetória é do STRIP que dirige

**Report (Enio, no smoke seguinte):** *"só o path do clip selecionado na aba keys toca em
arrange que possui outros clips e outros paths em outras strips. O arrange toca apenas um
clip."*

⚠️ **Consequência direta da §2b, e ela estava NOMEADA no doc do `path_for`** — o que faltava
era ligar o nome ao Arrange. Aquela porta responde pelo **clip ATIVO** (com recuo para o
primeiro que tenha trajetória), o que é exato no Keys e **errado sob a composição**: as
distâncias que o strip de B keya iam parar na curva de A, então o objeto percorria uma
trajetória só, qualquer que fosse o strip tocando.

**A cura é perguntar ao STRIP, não ao dropdown:** `TimelineDoc::driving_path(scratch,
target)` — o **maior peso** entre os strips vivos cujo clip TEM trajetória para o alvo,
desempate pela ordem do scratch (frame, lane, posição). O scan é **plano** de propósito: os
strips de um container interior estão no MESMO vetor com o `frame` deles, então o nesting
sai de graça, e a vista de container (`apply_views`) usa a mesma porta.

⚠️ **Numa SEQUÊNCIA — strips que não se sobrepõem, o Arrange normal — isto é EXATO em todo
instante.** O caso sem resposta escalar continua sendo o **crossfade entre duas curvas
diferentes**: ali o dominante vence e a trajetória TROCA no meio do cruzamento (medido:
a virada cai no meio da sobreposição, onde os pesos empatam). Compor **PONTOS** é a cura, e
é wave própria — ela precisa de três coisas que hoje são escalares: um `rest` que é ponto, o
canal de prop-link (`composed`/`links`, um `f64` por `(entity, prop)`) com dois números, e
outra resposta para o **auto-orient**, que pede a TANGENTE — ou seja, uma distância sobre
UMA curva.

**Gates (2 novos, e eles não são redundantes):**

| gate | o que só ele vê |
|---|---|
| `each_strip_in_arrange_walks_its_own_clips_trajectory` | a SEQUÊNCIA: dois strips, o Keys aberto em A o tempo todo — a variável que o report acusa |
| `during_a_crossfade_the_dominant_strip_picks_the_curve` | a regra do MAIOR peso |

⚠️ **E o segundo nasceu porque o primeiro não continha o fenômeno:** numa sequência há um
strip vivo de cada vez, então *"maior peso"* e *"menor peso"* dão a MESMA resposta e a
mutação que inverte a comparação **sobrevivia**. O oráculo do crossfade também não pode ser
o ponto exato (num cruzamento a distância é uma mistura) — é **em que CURVA** ele está: A
anda na horizontal (`y == 0`), B na vertical (`x == 0`).

**8 mutações no total, 8 sangram** — e a do `apply` voltando ao `path_for` sob composição
reproduz o report ao número: `[10.0, 0.0]` onde a curva de B mandava `[0.0, 10.0]`.

### 2d. E o FADE misturava réguas — agora ele compõe PONTOS, como o modo sem Path

**Report (Enio):** *"O Fade gera Path de transição entre um path de uma strip e outro path
de outra strip. Isso acaba deformando os paths de ambas as strips. O Fade precisa ser
similar ao modo sem Path."*

A §2c fez cada strip **escolher** a curva certa; ela não podia fazer os dois strips
coexistirem. A track de Position guarda uma **DISTÂNCIA**, e distância só significa algo
sobre UMA curva — então o blend cruzava dois números de **réguas diferentes** e avaliava o
resultado numa curva só. Medido no cruzamento de um caminho horizontal com um vertical:

```
t=1.25  [5.47, 0.00]     <- ainda na curva de A, mas numa distância já puxada por B
t=1.50  [5.00, 0.00]
t=1.75  [0.00, 4.53]     <- SALTO de 5 unidades para a curva de B
```

É essa a *"path de transição que deforma os dois"*: durante o fade o objeto percorre a
trajetória errada, e depois salta.

**A cura é a que o report nomeia:** o modo Separate (sem Path) blenda **coordenadas**, então
Position passa a fazer o mesmo. `Query::axis` (`Some(0)`/`Some(1)`) faz **cada strip
converter a própria distância na PRÓPRIA curva** (`stack_eval_path.rs::on_axis`) antes de o
blend somar qualquer coisa; o que cruza é uma coordenada.

⚠️ **O maquinário do fade não muda UMA LINHA, e a razão é aritmética:** a média por lane, o
`lerp` de Override e a soma de Additive são todos **AFINS** nos valores de origem, e aplicar
um blend afim componente a componente **É** blendar pontos. Depois:

```
t=1.25  [5.27, 0.20]
t=1.50  [3.75, 1.25]     <- a viagem entre as duas curvas
t=1.75  [1.37, 3.16]
```

**O que mudou, medido antes×depois nas MESMAS 161 amostras da cena do fingerprint:** o canal
`morph` difere em **0** delas; Position em **18** — as janelas de fade, e só elas —, com
delta máximo de **2,59** unidades. O irmão `fade_fingerprint.rs` (canais de transform) ficou
**byte-idêntico**. O `CHANNEL_FINGERPRINT` foi **re-pinado no mesmo commit com o motivo**,
que é o protocolo do pin.

⚠️ **Três consequências, cada uma com a sua decisão:**

1. **O auto-orient PROJETA em vez de pedir um segundo blend.** A tangente é uma grandeza
   sobre UMA curva, e depois de compor pontos não há mais distância; projetar o ponto de
   volta na trajetória que dirige devolve a distância que ele ocupa — exato fora do
   cruzamento (o ponto ESTÁ na curva) e a leitura honesta dentro dele. Rodar o blend uma
   terceira vez em distância faria pose e ângulo saírem de duas composições diferentes
   ([[feedback_derived_coordinate_seed_must_match_sample]]).
2. **O canal de prop-link continua publicando a DISTÂNCIA** — é o que uma fórmula
   `Nome.position` sempre leu, e mudá-lo seria outra quebra sem pedido.
3. **A rota de PROBE (`invert_stack`, o K sob uma pilha) resolve em DISTÂNCIA** (`axis:
   None`), porque distância é a unidade que a key guarda. Ela fica byte-idêntica ao que era.

**Gate:** `a_crossfade_between_two_curves_travels_instead_of_jumping` — ⚠️ e o oráculo é a
**CONTINUIDADE**, não um ponto: um endpoint sozinho fica verde sobre um salto. Ele mede o
MAIOR passo ao longo do cruzamento (a barra recusa o salto, não um passo de fade mais
rápido), exige que **fora** da sobreposição cada strip esteja EXATAMENTE na sua curva, e que
o **meio** do fade esteja fora das duas — que é o que viajar entre elas significa.

⚠️ **E uma medição minha nasceu ERRADA, do jeito mais fácil de não perceber:** usei a
MUTAÇÃO (ignorar o `axis`) como *"antes"*, e ela não é o modelo antigo — com o apply já
roteando Position pelo `sample_stack_point`, ignorar o eixo devolve a distância nos DOIS
eixos (`x=14, y=14` para distância 14, um ponto que não está em curva nenhuma). Dava *"140
de 161 amostras moveram"*, e eu quase reportei isso. **Uma mutação não é uma máquina do
tempo: para medir antes×depois desliga-se a ROTA, não o miolo dela.**

### 2e. E o que DEFORMAVA os dois paths era o AUTOKEY plantando uma âncora por frame

**Report (Enio, terceira rodada, com a §2d já shipada):** *"não funcionou. Está criando o
Path de transição."*

⚠️ **As minhas duas correções anteriores estavam certas e nenhuma tocava a causa** — porque
eu tinha lido *"deformando os paths"* como aparência, e o Enio estava descrevendo
**geometria**: as âncoras mudavam mesmo.

⚠️ **A primeira medição cortou o espaço de busca ao meio e me tirou da hipótese errada:**
rodar `apply_from_doc` 49 vezes ao longo do fade **não altera uma âncora**
(`probe_does_the_fade_write_geometry`: `GEOMETRIA MUDOU? NÃO`). Logo não era o apply, não
era o blend, e não era o overlay (que amostra a track e o path do clip ATIVO — não pode
desenhar uma terceira curva).

**A causa é o AUTOKEY.** Ele keya quando o mundo difere do que o documento diz, e num canal
de trajetória isso planta uma **ÂNCORA** (`AutokeyPlan::path_key` → `key_the_path` →
`add_path_key`), ou seja **geometria nova**. E o lado que LÊ reconstruía a pose como
`position_path(entity).at(distância_composta)` — um ponto **SOBRE** uma curva —, enquanto o
apply, desde a §2d, escreve o ponto **COMPOSTO**, que durante o fade está **ENTRE** as duas.

Repro red-first, varrendo o cruzamento inteiro sem o artista tocar em nada:

```
o autokey plantou 32 âncora(s) sobre a pose que o apply acabou de escrever
primeiras: (1.031, [5.14, 0.0004]) (1.062, [5.25, 0.0035]) (1.094, [5.33, 0.0116]) …
```

Uma por frame, cada uma na posição de trânsito. **É literalmente uma curva de transição
sendo desenhada dentro do path**, e ela deforma o caminho que a recebe — depois o
`rewrite_path_key_values` reescreve as distâncias de todas as keys daquele clip, e a
cronometragem vai junto.

⚠️ **A lei quebrada está citada no doc do próprio `autokey_props`:** *"whatever writes a
derived coordinate and whatever reads it must be the SAME function"*
([[feedback_derived_coordinate_seed_must_match_sample]]). O doc-comment do `position_shown`
ainda **afirmava a garantia** — *"`position_shown` and the apply both read `path.at(distance)`
at the same instant, so an on-curve pose is byte-equal and never re-keys"* — e essa frase
virou **FALSA** no commit que fez a composição compor pontos. Corrigida onde estava.

**A cura é a que o irmão escalar já usava** (ADR-0152 C2/W6 — *leia o que o apply
ESCREVEU*): o apply publica a pose de trajetória que escreveu
(`StackScratch::composed_points`, irmão exato do `composed_links`) e o `position_shown` a
LÊ. Mapa vazio ⇒ o apply não escreveu esta entidade neste frame (não há pilha, ou ela estava
sob a mão do artista) ⇒ **re-derivar é o certo ali**, porque a diferença é movimento de
verdade.

**Gate:** `the_autokey_plants_no_anchor_on_the_pose_the_apply_itself_wrote` — varre os 33
instantes do cruzamento, e em cada um pergunta ao autokey sobre **exatamente a pose que o
apply acabou de escrever**. **2 mutações, 2 sangram**, e as duas com o mesmo número (32
âncoras): o leitor re-derivar · o escritor não publicar. Elas **não são redundantes** — são
as duas pontas da mesma igualdade, e cada uma sozinha a quebra.

⚠️ **DOIS erros de PROCESSO meus nesta rodada, os dois do mesmo tipo — trabalho perdido por
não OLHAR a saída:**

1. Um script `python` morreu num `assert` e eu **só grepei o stdout do cargo que vinha
   depois**, então o traceback passou invisível: três edições ao `apply.rs` nunca
   aconteceram e eu diagnostiquei o resultado como se tivessem
   ([[feedback_pipe_masks_script_exit_code]], terceira vez).
2. Rodei um **workflow de investigação na MESMA worktree** em que estava editando; os
   agentes escreveram sondas nos meus arquivos e **um deles reverteu o `apply.rs`** — o
   sintoma foi um `git diff` VAZIO num arquivo que eu tinha acabado de editar. Lição:
   *investigação paralela lê; quem escreve é um só.*

### 2f. E a trajetória era oferecida em TODA aba — só a Keys tem um clip ativo

**Report** (Enio, 2026-07-31, depois do smoke que aprovou o 2e): *"Funcionou! Mas os paths
estão visíveis e editáveis em strips Arrange e Containers. Isso não pode acontecer. Path
editável apenas em Keys: Clips."*

**Mecanismo.** As waves 2b-2e moveram a trajetória para o CLIP, e o overlay a lê do clip
**ATIVO** (`doc.clip_path(doc.active_index(), …)`). Quem escolhe o clip ativo é o dropdown
da aba **Keys** — mas o overlay era desenhado e agarrável em **qualquer** aba. Em
Arrange/Containers o que dirige o objeto é a **PILHA** (o apply compõe as strips), então a
curva sob o cursor podia nem ser a que move o que se vê, e arrastar uma âncora ali editava
um clip que a aba nem nomeia.

**A cura entra na porta ÚNICA.** `active_path` — a mesma que as waves anteriores usaram
para matar a alça fantasma — ganhou `keys_tab: bool` e recusa antes de olhar o documento.
As cinco portas (pintar · âncoras · alças · hit · hit-de-curva) atravessam ela, então
**pintar e agarrar não podem discordar**; a regra copiada nos cinco chamadores seria
literalmente o defeito do §2a de volta (*nada desenhado, tudo agarrável*).

⚠️ **`keys_tab` não é uma preferência de visibilidade: é o MESMO booleano que decide se o
documento SOLA o clip ativo** (`TimelineState::keys_mode` → `apply_active_clip` contra
`apply_scene`, e o `solo` do `autokey_pass`). Por isso a regra é coerente e não arbitrária
— e por isso ela cai **também com o painel FECHADO**: ali o `paint` publica `false` e o
apply já está compondo a cena, então oferecer a alça seria oferecê-la sobre um clip que
ninguém escolheu nesta tela. **Consequência de produto NOMEADA, não escondida.**

**Gates.** `the_trajectory_is_offered_only_on_the_keys_tab` (as cinco portas, presença E
ausência, no MESMO documento e nas MESMAS coordenadas — o oráculo de agarrar é lido do
DESENHO, senão metade ficaria verde sobre um ponto chutado) + o arch-gate de shell
`the_motion_path_is_offered_only_on_the_keys_tab`, que varre o `src/` INTEIRO e recusa um
literal no 1º argumento de qualquer uma das cinco portas.

⚠️ **Os dois gates NÃO são redundantes, e isso está medido:** com o `draw` revertido para
`true` literal, **os 20 testes de unidade do overlay ficam VERDES** e só o arch-gate sangra
— os cinco sítios vivem em `render_frame` e nos handlers de ponteiro, que exigem janela e
GPU. **3 mutações, 3 sangram** (a porta ignorando `keys_tab` · o `draw` literal · um sítio
de `input_dispatch` literal).

**LOC:** o `_tests.rs` cruzou 600 com o argumento novo, então o gate do CLIP (§2b) e o da
ABA foram para o irmão `motion_path_overlay_scope_tests.rs`. O corte é por assunto, não por
tamanho: os dois medem *ONDE a trajetória é oferecida* — as duas metades da mesma porta —
enquanto o `_tests.rs` mede *como ela é desenhada*.

**A metade de AUTORIA fechou na mesma sessão — ver §2g.**

### 2g. E a mesma lei na AUTORIA: o K e o AutoKey não ancoram fora da aba Keys

O §2f fechou o que se **vê** (o desenho e as alças). Havia duas outras portas que
**escreviam** a mesma geometria, invisíveis: o **K** (`TimelineIntent::AddPathKey`) e o
**AutoKey armado** (`AutokeyPlan::path_key`). As duas plantavam uma âncora no path do clip
ATIVO a partir de Arrange ou de dentro de um container — e uma âncora não é uma key a mais:
ela reescreve a distância que **todas** as keys daquele clip guardam
(`rewrite_path_key_values`). Um gesto local, uma edição grande, num clip que a aba nem
nomeia.

⚠️ **Gatear não bastava, e é por isso que a nota anterior dizia "não construído":** um gesto
que não faz nada e não diz nada é indistinguível de uma ferramenta quebrada — com uma
agravante aqui, porque o apply seguinte devolve o objeto à curva e o artista vê a pose
saltar sem motivo. Então a recusa é um **VALOR**, a lei que este documento já segue desde o
R9: `KeyRefusal::PathNeedsKeysTab`, com mensagem própria.

⚠️ **As outras recusas são sobre um valor que o clip não consegue EXPRIMIR; esta é sobre um
clip que o animador não está OLHANDO.** Por isso ela não entra na lista `refused` (cujo
motivo o chamador deriva do mapa da strip — o toast diria *"uma lane acima sobrepõe"* sobre
uma trajetória): `AutokeyPlan` ganhou o campo próprio **`path_refused`**, e o `is_empty()` o
inclui — o que também prende o anti-spam: um objeto **parado** sob uma pilha não keya nem
**reclama**, senão Arrange com AutoKey armado cospe um toast por frame.

**As duas portas, cada uma com a pergunta no lugar certo:**

- **AutoKey** — o `solo` do `autokey_props_in` **é** a pergunta (o mesmo `keys_mode` do
  overlay), então a lei mora no documento e o shell só a repassa ao toast que já existia.
- **K** — nasceu a porta **`timeline_bridge::path_key_time`**, que lê o `keys_mode` do
  próprio `TimelineState` e devolve `Result<RationalTime, KeyRefusal>`. ⚠️ **O chamador não
  tem booleano a passar (nem a passar errado)**, e o `key_insert_time` — que atendia este
  caso — responde a outra pergunta (*onde esta STRIP toca?*), que para geometria de clip
  não é a pergunta certa.

**Gates:** `the_anchor_is_refused_under_a_stack` (presença E ausência sobre o MESMO
movimento, só a PORTA muda) · `the_k_anchors_a_trajectory_only_on_the_keys_tab` (idem, só a
VISTA muda) · `an_anchor_refused_outside_the_keys_tab_says_so` (as três asserções são
independentes: nada é escrito · algo é dito · o motivo é o CERTO) · e o arch-gate
`the_k_authors_an_anchor_through_the_door_that_can_refuse`, que afirma a PROPRIEDADE
(*todo `AddPathKey` do shell nasce de um `path_key_time`*) com controle positivo de
**exatamente um** emissor. **4 mutações, 4 sangram.**

⚠️ **Três fixtures foram corrigidas, não afrouxadas:** elas testavam *"o modo Path ancora e
nunca keya os eixos"* pela porta da PILHA — uma vista onde isso não pode acontecer. Agora
declaram a premissa (`autokey_props_solo` / `st.keys_mode = true`) em vez de herdá-la de um
default, que é o que inverte de sentido no dia em que o default se move
([[feedback_a_fixture_that_reaches_its_state_by_toggle_inverts_when_the_default_moves]]).

**LOC:** `autokey.rs` estava em 698/700 ⇒ o `mod tests` foi para o irmão
`autokey_tests.rs` por `#[path]` (segue FILHO, então `use super::*` alcança os privados —
o corte é de TAMANHO, não de visibilidade).

**Escopo que NÃO muda:** o AutoKey em Arrange segue vivo para os **escalares** (o
`invert_stack` do ADR-0152 é feature desenhada); só a geometria de trajetória é recusada.
