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
