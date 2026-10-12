# Ciclo 3 — TRANSFORMES & DEFORMADORES · «Dobrar o mundo»

> História verbatim: [arquivo](../archive/motion-nodes-2026-10-12/106_ciclo_3_transformes_e_deformadores.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| `motion.spline_wrap` no dispositivo (W5b) | Fica na CPU com a razão e o preço escritos; a recusa não é exprimível pelo `applicable` do irmão (lê params `f32`) | vivo W5b |
| `applicable: Some(..)` como prova de que a recusa existe | É um PROXY: prova a recusa, não que ela morda; mutação apagando a recusa do `motion.look_at` acusa-o | vivo W5b |
| Marchar o tique no dispositivo e na CPU | Simularia o tique duas vezes e um nó trataria o mesmo estado duas vezes | vivo «A caça de três rondas» |
| As duas curas do gizmo que sumiu | Foram palpites; a 2.ª provou a 1.ª errada, e a sonda do autor mentiu; a medição achou que a recusa nomeava a condição que não morde | vivo «A caça de três rondas» |
| Culpar a camada visual no report «não aparece no canvas» | Causa lida do código: `Camera2d` abre em `height_world = 10` e o zoom-out corta a simulação | vivo «O report do dono — a simulação» |
| `Flip Orientation` no passo do tutorial do Mirror | O passo do autor estava errado: a culpa era do passo, não do nó | vivo «O segundo report — o Mirror» |
| Seis vocabulários para “onde é o centro” | Achado do grupo (§2.3): o pivô passa a ser UMA porta só (W1) | vivo §2.3 / W1 |
| Despertar sem acordar dois dos treze | A tabela escondia-o; duas provas de mutação cobrem o desvio | vivo «O despertar não acordava dois dos treze» |

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) (os sete passos, as quatro leis) ·
> **Portas do código:** [doc 102](102_o_outro_patamar_plano_dos_nos_2026-09-04.md) ·
> **O cartão:** [doc 101](101_pesquisa_cartoes_ricos_2026-09-04.md) ·
> **Ciclos anteriores:** [104 (arranjo)](104_ciclo_1_arranjo.md) · [105 (animadores)](105_ciclo_2_animadores.md)

⚠️ **O ciclo 2 continua ABERTO até o Enio correr o PDF dele** (`tutoriais/02_animadores.pdf`) —
o passo 7 é a aceitação e é dele. Este doc é o ciclo **3**, aberto por ordem dele
(*«volte ao plano e siga implementando»*, 2026-09-07).

---

## §1 — O grupo (13 nós), e a premissa do tutorial

`motion.move` · `motion.rotate` · `motion.scale` · `motion.transform` · `motion.mirror` ·
`motion.look_at` · `motion.bend` · `motion.twist` · `motion.spherize` ·
`motion.four_point_warp` · `motion.bezier_warp` · `motion.kaleidoscope` · `motion.spline_wrap`

**A premissa do tutorial:** o artista tem uma coisa na tela e quer **dobrá-la, torcê-la, espelhá-la
e enquadrá-la** — e a pergunta que atravessa os treze é sempre a mesma: **em torno de QUÊ?**
O tutorial é a resposta a essa pergunta, treze vezes.

---

## §2 — Passo 2: a AUDITORIA

⚠️ **A auditoria começa por MEDIR o que existe.** Sonda:
[`motion_deformadores_probe.rs`](../../shells/desktop/src/motion/motion_deformadores_probe.rs), três
testes `#[ignore]`.

### §2.1 — O retrato (`audit_the_deformer_group`)

| nó | params | no cartão | device | portas | efeito |
|---|---:|---:|:---:|---|---|
| `motion.move` | 3 | 3 | 🟢 | 1→1 | Pure |
| `motion.rotate` | **1** | 1 | 🟢 | 1→1 | Pure |
| `motion.scale` | 4 | 2 | 🟢 | 1→1 | Pure |
| `motion.transform` | 8 | 5 | 🟢 | 1→1 | Pure |
| `motion.mirror` | 5 | 5 | 🔴 **NÃO** | 1→1 | Pure |
| `motion.look_at` | 5 | 5 | 🟢 | 3→1 | Pure |
| `motion.bend` | 7 | 7 | 🟢 | 2→1 | Pure |
| `motion.twist` | 5 | 5 | 🟢 | 2→1 | Pure |
| `motion.spherize` | 4 | 4 | 🟢 | 2→1 | Pure |
| `motion.four_point_warp` | 8 | 8 | 🟢 | 2→1 | Pure |
| `motion.bezier_warp` | **24** | 24 | 🔴 **NÃO** | 2→1 | Pure |
| `motion.kaleidoscope` | 5 | 5 | 🟢 | 2→1 | Pure |
| `motion.spline_wrap` | 18 | 19 | 🔴 **NÃO** | 2→1 | Pure |

⚠️ **O `19 > 18` do `spline_wrap` NÃO é um órfão** — o 19.º é o **text param** da forma
desenhada (`ParamWidget::ShapePicker`), que não é um `ParamSpec` (estes são `f32`). A régua da
coluna «no cartão» conta rows; a dos params conta o manifesto, e as duas contam coisas
diferentes de propósito.

⚠️ **Os `2 de 4` do `motion.scale` e os `5 de 8` do `transform` estão explicados** — o gate do
catálogo inteiro (`every_param_the_card_hides_has_a_declared_reason`, 810 controlos · 127
escondidos · **0 sem explicação**) já responde por eles, e ele corre em **todo** fecho.

### §2.2 — O PREÇO, e o 🔴 é o achado (`measure_the_deformer_group`)

`motion.grid(320×320) → <nó> → motion.output`, mediana de 3 cozimentos **frios**, `--release`.
⚠️ **O relógio é o do cozimento na CPU** (uma sonda headless não tem adapter); o que a coluna
`no device?` mede é o **planeador** — `ph2d_gpu_cook::plan(..).is_fully_gpu()` —, que é uma
propriedade determinística e não uma leitura de relógio.

| nó | objectos | cadeia no device? |
|---|---:|:---:|
| `motion.grid` sozinho | 102 400 | 🟢 |
| `motion.move` · `rotate` · `scale` · `transform` · `look_at` | 102 400 | 🟢 |
| `motion.bend` · `twist` · `spherize` · `four_point_warp` | 102 400 | 🟢 |
| **`motion.kaleidoscope`** | **614 400** (6×) | 🟢 |
| **`motion.mirror`** | **204 800** (2×) | 🔴 **NÃO** |
| **`motion.bezier_warp`** | 102 400 | 🔴 **NÃO** |
| **`motion.spline_wrap`** | 102 400 | 🔴 **NÃO** |

⭐ **O grupo chega ao dispositivo — 10 de 13.** ⛔⛔ **E os três que não chegam não dizem
porquê.** A lei nº 1 do protocolo é explícita: *um nó do grupo que caia para a CPU sai do ciclo
com a razão nomeada e o preço medido*. Hoje os três têm, no fonte, apenas a etiqueta `CPU-only`.

⭐⭐ **E o `motion.kaleidoscope` é a prova de que MULTIPLICAR A CONTAGEM não é a razão:** ele
devolve **seis vezes** os elementos que recebe e a cadeia continua reivindicada, porque ele
declara `count_law`. O `motion.mirror` faz `2n` — a **mesma forma**, com o irmão já a percorrê-la.
⇒ *não é lacuna de param, é lacuna de COBERTURA* — e a folha 05 da conferência já o tinha escrito
em 2026-08-09, no §0, sem nunca ter sido fechado.

### §2.3 — ⛔⛔ O ACHADO DO GRUPO: **SEIS vocabulários para «onde é o centro»**

Lido do `MANIFEST` e do `register_reduces` de cada crate, não de memória:

| como o nó responde *«em torno de quê?»* | nós |
|---|---|
| `pivot_x` / `pivot_y`, coordenada **absoluta de mundo** | `bend` · `twist` · `kaleidoscope` |
| **centroide + offset** (o offset é relativo, e há cerca escrita a dizê-lo) | `spherize` |
| a **bbox**, por quatro reduções, **sem controlo nenhum** | `four_point_warp` |
| a **bbox**, calculada no `eval`, **sem controlo e sem reduções** | `bezier_warp` |
| um **enum** `pivot_mode` (World Origin · Point · Centroid) | `transform` |
| a **linha do centroide** do que entrou, mais um `offset` escalar | `mirror` |
| **nada** — a origem do mundo, sempre | `move` · `rotate` · `scale` · `look_at` · `spline_wrap` |

⚠️ **Uma lei escrita em seis sítios não é uma lei** — e esta é a mesma pergunta seis vezes, com
seis respostas que não se convertem umas nas outras. O artista que aprende o `pivot_x` do
`motion.twist` **não** sabe usar o `offset_x` do `motion.spherize`, e o `motion.bend` a que ele
chega a seguir mede a extensão dele *a partir do pivô que ele digitou*.

⭐ **E a referência inteira faz o mesmo, cada uma à sua maneira** (uma fonte por afirmação):

| referência | como responde |
|---|---|
| **Blender** *Simple Deform* (Bend/Twist/Taper/Stretch) | um **objecto** `Origin` — o pivô é outra coisa da cena, e sem ele é a origem do próprio objecto |
| **Blender GN** *Rotate Instances* / *Scale Instances* | sockets `Pivot Point` / `Center` (Vector) + `Local Space` (bool) |
| **Blender GN** *Transform Geometry* | **não tem pivô** — age na origem da geometria |
| **After Effects** *Transform* (efeito) | **Anchor Point**, um par de números que o artista arrasta |
| **After Effects** *Twirl* / *Bulge* | `Twirl Center` / `Bulge Center` — de novo dois números |
| **Illustrator / Figma** | o **widget de 3×3** (nove âncoras da bbox) no painel de transformação |
| **Cinema 4D** *Fields* | *Fit to Parent* — um **botão que CONGELA** a bbox do pai no instante do clique |
| **Houdini** *Bend* | uma **capture region** com origem, direcção e comprimento, autorada à parte |

⇒ **três formas em toda a indústria: um número que se digita · um objecto que se aponta · uma
bbox que se congela.** Nenhuma delas é *«o centro do que está a passar por aqui, agora»* — e é
exactamente isso que o nosso substrato entrega **de graça e no dispositivo** (`reduce → broadcast
→ map`, declarado como metadado no registry), e que a folha 05 §3 já nomeara como o `SUPERAR` S1
sem nunca ter sido construído.

### §2.4 — Os outros dois achados

**A. Falta o CISALHAMENTO (skew).** Nenhum dos treze o exprime. O *Transform* do After Effects
tem **Skew + Skew Axis**; o Illustrator tem a ferramenta **Shear** (ângulo + eixo); o Blender tem
`Shear` no menu de transformação e a matriz completa no *Transform Geometry* por composição. A
folha 04 da conferência já o tinha nomeado — *«o `Skew` continua faltando (e é afim: caberia no
`motion.transform`, não aqui)»* — e ficou por fechar. ⚠️ **É afim**: entra no `motion.transform`,
que já é o nó do afim de layout, e **não** um nó novo.

**B. O `motion.rotate` gira em torno da ORIGEM DO MUNDO, e a volta que existe custa `Temporal`.**
A recusa do pivô está escrita no doc-comment dele e é uma **fatoração legítima** (ele só acumula
um escalar em `rot`, o que o mantém transcendental-free, HR-5) — quem roda `P` em torno de um
centro é o `motion.orbit`. ⛔ **Mas o `motion.orbit` é `Effect::Temporal` porque tem `speed`**, e
a tabela do custo já está paga (`ph2d-gpu-cook::measure_static_orbit`, 120 quadros, params
parados): **0,0003 ms** para um nó `Pure` contra **0,6477 ms** a 102 400 elementos — **2294×** —
*mesmo com `speed = 0`*. ⇒ hoje **girar um layout em torno de um ponto sem animar carimba a
sub-árvore inteira como temporal**. A folha 04 §3 chama a cura de *«um conserto, sete nós»*.

---

## §3 — O PLANO, wave a wave

⚠️ **A ordem é por VALOR PARA O ARTISTA** (a auditoria já disse que o grupo está no dispositivo,
10 de 13), com a excepção da W2, que é a lei nº 1 do protocolo e é barata porque o irmão já a
percorreu.

### W1 — ⭐⭐⭐ O PIVÔ É UMA PORTA SÓ (o `SUPERAR` S1, por fim)

Um `pivot_mode` **único**, com o mesmo vocabulário e a mesma ordem de valores em todo nó que o
tem, e com os modos que **só nós** podemos oferecer porque a redução corre no dispositivo:

| valor | o que é | como se calcula |
|---:|---|---|
| `0` | **World Origin** | `(0, 0)` — nenhuma redução |
| `1` | **Point** | `(pivot_x, pivot_y)` — o de hoje |
| `2` | **Centroid** | `Σp / n` — duas reduções `Sum`, as do `spherize` |
| `3` | **BBox Centre** | `(min+max)/2` — quatro reduções, as do `four_point_warp` |

Nós: `motion.bend` · `motion.twist` · `motion.kaleidoscope` (que hoje só têm `Point`) e
`motion.transform` (que ganha o `BBox Centre` que lhe falta).
⛔ **`motion.spherize` fica de fora e a cerca é dele:** ali o `offset` é **relativo ao
centroide** por três razões escritas (folha 04, `CERCAS:` 3) e um centro absoluto *«arrancaria a
lente do assunto em todo documento já autorado»*.
⛔ **`motion.rotate` e `motion.scale` ficam de fora por NATUREZA** — eles escrevem `rot` e
`size`, não `P`; a recusa está escrita nos dois doc-comments e foi verificada.

**Defaults:** `pivot_mode = 1 (Point)` nos três que hoje têm `pivot_x/y` ⇒ **o caminho de omissão
é byte-idêntico**. No `transform` o default fica no `0` que ele já tem.

⚠️ **O preço tem de ser MEDIDO antes de escrito:** as reduções são estáticas por tipo de nó (o
`ReduceSpec` não tem gate de param), então um `bend` em modo `Point` passa a pagar as reduções do
centroide na mesma. A wave mede `bend` hoje contra `bend` com as reduções, nos dois caminhos, e
escreve o número. ⛔ Se o preço morder, a saída é uma **variante de kernel por param**
(`variant_by_param`, o mecanismo que o `motion.move` já usa para o `space`), nunca uma isenção.

### W2 — ⭐⭐ O ESPELHO VAI AO DISPOSITIVO (a lei nº 1)

`motion.mirror` ganha `count_law` e kernel, copiando a forma do `motion.kaleidoscope`
(`SourceRows` + `count → k·n`), que a auditoria mediu a devolver **614 400** objectos e a
manter-se 🟢. ⚠️ **O `keep = Reflection Only` muda a lei da contagem** (`n`, não `2n`) e o
`reindex` **recua o device** no irmão por `applicable` — as duas coisas têm de vir na declaração,
não no kernel.

### W3 — ⭐ O CISALHAMENTO (`motion.transform`)

`skew` (graus) + `skew_axis` (graus), a forma do After Effects, no nó do afim. Default `0`/`0` ⇒
byte-idêntico. ⚠️ **Dobra no mesmo afim que o pivô já dobra** (`(p−c)·M + c + o`), então a lei por
elemento continua a ser **uma** matriz — não um segundo afim para a máscara de `falloff` e o
kernel discordarem, que é a cerca que o `pivot_mode` do próprio nó já escreveu.

### W4 — O CARTÃO E O CENSO DO GRUPO

> ⛔ **A acusação escrita abaixo sobre o `motion.bezier_warp` foi REFUTADA por medição** — o
> cartão já dobra as 24 rows em cinco secções que nomeiam a aresta, e a acusação saiu de uma
> sonda que lia só os rótulos. O que a medição achou no lugar dela está em §4 W4b.

O censo do ciclo 1 (`no_param_the_panel_offers_falls_off_the_card`) e o do ciclo 2
(`every_param_the_card_hides_has_a_declared_reason`) correm sobre o grupo, e o cartão dos treze
é lido com o olho do doc 101 (secções, LOD, o que o `motion.bezier_warp` faz com 24 rows).

### W5 — OS DOIS QUE FALTAM AO DISPOSITIVO, ou a razão de cada um

> ✅ **FECHADA** — o `motion.bezier_warp` foi ao dispositivo (§4 W5a) e o `motion.spline_wrap`
> ficou com a razão nomeada e o preço (§4 W5b).

`motion.bezier_warp` (24 params, patch de Coons, bbox no `eval`) e `motion.spline_wrap` (cúbica
autorada + reparametrização por comprimento de arco). ⚠️ **A saída aceitável é kernel OU uma
razão nomeada com o preço medido** — a lei nº 1 não admite a etiqueta `CPU-only` sozinha.

### W6 — A MEDIÇÃO (passo 5) e o TUTORIAL (passo 6)

---

## §4 — Registo das waves

### ⛔⛔ O REPORT DO DONO — *«a simulação funciona nos nós mas não aparece no canvas»* (08/09)

**A cena `=111` que eu entreguei era invisível, e o defeito era meu.** Os cartões cozinhavam
`180 000` objectos, cada pré-visualização de nó tinha pontos, o canvas estava vazio.

⚠️⚠️ **E o HUD dizia `0 inst`, o que quase mandou a investigação para o sítio errado.** Aquele
contador conta **entidades `RenderInstance` do ECS** (`snapshots.rs`), e uma cena de motion
residente no dispositivo **não cria nenhuma**: ele lê `0` em toda cena de motion que funciona.
*Um contador que lê zero no caminho saudável não é evidência de nada.*

⛔ **A causa, lida do código:** `ph2d_render::Camera2d` abre em `height_world = 10` e o zoom-out
**pára** em `ZOOM_MAX_HEIGHT_WORLD = 100`. Eu dimensionei a cena no idioma dos demos de
**performance** (`=12` é `700 × 700` a `1` unidade de passo) em vez do idioma das cenas que se
**leem**, e pus o pano entre **`300` e `600`** unidades. *Ele não estava fora do ecrã: estava fora
do ALCANCE, e nenhum gesto o encontrava.*

#### ⭐⭐⭐ E a régua que faltava estava na própria árvore — não foi escolhida, foi MEDIDA

`where_each_demo_scene_lives` varre o roteador inteiro e a partição aparece sozinha:

| classe | quantas | onde vivem |
|---|---:|---|
| cenas **com legenda** (`=82`..`=110`) | **29** | todas dentro de **`±11`** unidades |
| cenas **sem legenda** (`=1`, `=2`, `=6`, `=7`, `=12`..`=16`) | **9** | **`180`..`800`** — os demos de perf, que ninguém lê |
| **`=111`, a minha** | 1 | **`300,5`..`599,5`** — a única com legenda do lado errado |

⇒ **a LEGENDA é o discriminador**: uma cena que pousa uma ficha no canvas está, por construção, a
dizer *«alguém vai ler isto»*. O gate
`a_scene_with_a_legend_fits_inside_what_the_camera_can_reach` corre sobre o catálogo (⚠️ **constrói
todas e só COZINHA as que têm legenda** — a `=1` tem 2 M elementos), com a barra lida do código
(`ZOOM_MAX_HEIGHT_WORLD / 2`) e não escolhida. Prova de mutação: repor o `450` reprova com
`cena 111: x 448,8..451,2 (alcance 451,2 > 50)`.

### 🔬 A CENA DE SMOKE — `PH2D_GPU_COOK_DEMO=111`

`grid(80×80) → scale → move → mirror → twist → transform → output`: um pano de **2,4 unidades**,
`12 800` objectos, a **3,0 unidades** da origem (`1,25 ×` a largura dele — derivado, nunca digitado
duas vezes). ⚠️ **O deslocamento é a cena inteira:** com o pano centrado os três modos dão a mesma
imagem, e a cena ensinaria que a escolha não importa.

### ⏳ ABERTO — dois vermelhos de GPU que já estavam no `main`

1. **`value_slope_kernel_matches_the_cpu_on_the_device`** — mede `1,05023384e-4` contra a barra de
   `1e-4`, **5 % acima**. Já vem com o diagnóstico escrito no próprio teste (atribuído por ablação
   em 2026-08-11, e a nota pede o número **noutra máquina** antes de recalibrar). Reproduzido
   idêntico numa worktree limpa em `main` — **não é desta linha**.
2. *(fechado nesta wave)* `the_colour_loop_closes_the_same_way_on_the_device` — era a W1b.

---

### ⛔ W5b — O `motion.spline_wrap` FICA NA CPU, e agora com a razão e o preço

A lei nº 1 do protocolo não aceita a etiqueta `CPU-only` sozinha. São **duas lacunas de
SUBSTRATO**, e nenhuma delas é deste nó:

**1. A forma DESENHADA chega pelo canal de externos, e o dispositivo não tem canal.**
O `eval` lê `ctx.external(&curve_of(nome))` — a polilinha que a shell publica da Hierarquia. O
sequenciador não conhece a palavra: **`ph2d-gpu-cook` tem ZERO ocorrências de `external`**, e um
`ColumnBinding` só endereça uma coluna de uma PORTA de entrada.
⚠️ Registar um kernel hoje **não deixaria o nó lento: deixá-lo-ia ERRADO** — o dispositivo cairia
na cúbica dos oito params e devolveria uma curva **plausível**, sem estouro e sem aviso, enquanto a
CPU segue a forma que o artista desenhou.
⛔ E a recusa **não é exprimível** pelo mecanismo que o irmão usa: o `applicable` lê params `f32`,
e *qual forma?* vive no canal de **TEXTO** (a mesma assimetria que obrigou o `ParamGateText` a
existir ao lado do `ParamGate`).

**2. Ele escreve TRÊS colunas, e duas por PRESENÇA.** Com o `follow_rotation` desligado o `rot` é
**COPIADO**, não reescrito com o mesmo valor; o `size` idem fora do afunilamento — é isso que faz o
default byte-idêntico por ESTRUTURA. Uma `ColumnBinding` de escrita é incondicional. ⚠️ **É a MESMA
lacuna que o `flip_rot` do `motion.mirror` já tem nomeada na W2.**

**O preço** não é o cozimento deste nó: é a cadeia INTEIRA deixar o dispositivo. A auditoria de
performance do módulo ([doc 98](98_auditoria_de_performance_2026-09-01.md)) mediu **`50,9×`** entre
os dois caminhos a 4,19 M objectos, e um nó CPU-only a meio de uma cadeia paga esse factor por
**todos** os nós dela.

⇒ este nó volta à fila no dia em que o substrato ganhar **um dos dois**: o canal de externos no
dispositivo, ou a escrita condicionada à presença da coluna.

#### O censo que guarda a fronteira

`an_external_reader_that_reaches_the_device_declares_a_refusal` (censo do catálogo): **um nó cujo
produto lê o canal de externos ou não tem kernel, ou declara `applicable`.** Medido: **11** nós lêem
externos e **1** deles tem kernel — o `motion.look_at`, que **já estava certo antes do censo
existir** (os modos `Object` e `Cursor` recusam o dispositivo, com o custo nomeado no comentário).
*Este ficheiro não inventa a lei — impede que o próximo nó a redescubra por report.*

⚠️ **`applicable: Some(..)` é um PROXY, e o censo diz isso de si mesmo:** ele prova que a recusa foi
**pensada**, nunca que ela é a certa. O que ele mede é a AUSÊNCIA de pensamento, que é o modo de
falha que passa em silêncio. Prova de mutação: apagar a recusa do `motion.look_at` acusa-o pelo
nome.

---

### ⛔ O REPORT DO DONO — o gizmo do warp (08/09)

> *«o gizmo de Bezier Warp tem ponto e handles/alças muito pequenos e está sendo desenhado por
> trás das shapes»*

**Duas coisas, e a segunda não era o que parecia.**

#### 1. O tamanho — e a casa já tinha a lei, com este gizmo sozinho fora dela

| gizmo | pinta | agarra |
|---|---:|---:|
| `connector` | `7` | `7` |
| `envelope` (a gaiola de deformação) | `6` | `6` |
| `vec_text_ride` | `15` | `15` |
| **`warp` (este)** | **`4,5` / `3,5`** | **`11`** |

O alvo era **`2,4×` a `3,1×`** maior do que a tinta. *O artista via um ponto e apontava para
outra coisa.* ⚠️ **Uma constante, dois consumidores** é a forma que todos os outros têm; aqui
eram dois números, e o que o artista vê é sempre o que envelhece.

⚠️ **E o gate novo apanhou-me a confundir o RAIO com o ALCANCE:** a primeira redacção pôs o
losango a `GRAB_PX` cru, e a apótema dele (`r/√2`) ficou a `0,71×` da barra. Hoje as duas marcas
cobrem o mesmo raio **inscrito** e distinguem-se só pela forma e pela tinta — que é o que o
cabeçalho do módulo já prescrevia antes do report.
Gate `a_painted_handle_covers_the_radius_it_is_grabbed_at`, com os valores que shipavam como
controlo de falsificação (`0,41×` e `0,22×`).

#### 2. O «por trás» — o gizmo ESTAVA por cima

A legenda da cena aparece sobre os mesmos objectos, e ela é chrome como o gizmo: o chrome já
compõe acima dos sprites. O que acontece é que um traço de `1,5 px` e uma alça de `4,5` em ciano
claro **somem** sobre um pano de discos brancos.

⚠️ ***Um manipulador que desaparece sobre o conteúdo que ele manipula é indistinguível de um que
está por baixo, e o report que volta é o mesmo.***

A cura também já existia na casa, e o comentário dela **nomeia** o defeito — o
`draw_connector_handles`: *«o anel branco destaca a alça de qualquer fundo … sem o anel a bolinha
some sobre um traço claro»*. Aqui o anel é **escuro**, porque este gizmo tem de ser lido sobre
conteúdo claro tanto quanto sobre o fundo escuro do canvas.

#### 3. E havia uma ordem REAL errada, só que noutra cena

O desenho vivia ~2 000 linhas **antes** de dois produtores da mesma cena do Vello — o documento
vectorial e as formas vivas do Motion —, e ali a ordem de codificação **é** a ordem de z. Numa
cena com `source.shape` ou com um documento vectorial aberto, o manipulador passava mesmo por
baixo do que ele manipula. Mudou para o fim do bloco do Motion.

---

### ⛔ O SEGUNDO REPORT — o Mirror (08/09), e a culpa é de um passo meu

> *«Em Mirror Flip Orientation parece não funcionar. Reindex não parece mudar nada! como usar
> reindex?»*

**Os dois funcionam.** Nenhum tem sobre o que agir na cena `=111`:

| controlo | age sobre | por que ali não se vê |
|---|---|---|
| `Flip Orientation` | a **direcção** (`rot`) e a velocidade do gémeo | a cadeia não produz coluna `rot`, e os elementos são discos redondos |
| `Reindex` | as colunas de identidade `Index`/`Count` | é uma **renumeração**: ela nunca muda a imagem sozinha, só o que um consumidor a jusante lê |

⛔⛔ **A culpa é do passo que eu escrevi.** O tutorial mandava clicar em `Flip Orientation` e a
seguir dizia *«nesta cena não muda nada que se veja»*. ***Um passo que instrui um gesto e anuncia
que ele não faz nada não é um passo*** — e o report que ele produziu foi exactamente o previsível.

⭐ **E a cena que os mostra já existia:** a `=69` (a família Transform da conferência) tem o **par
verde** (`/// ///` contra `/// \\\`) e o **par rosa** (o degradé que recomeça em cada fatia contra
o que atravessa as seis). O tutorial passa a levar lá, com o comando.

---

### ⛔⛔⛔ A CAÇA DE TRÊS RONDAS — o gizmo que sumiu, e a lacuna de substrato por baixo (08/09)

Report do dono, **três** vezes: *«sumiu com o gizmo do Bezier Warp»* → *«ainda invisível no
canvas»* → a linha que resolveu tudo, colada do terminal dele:

```
[warp-diag] nao ha' retrato publicado (`view()` = None)
```

#### A causa, e ela não estava no gizmo

As **TOMADAS** — o espreitar ao stream de um nó a meio do grafo — são cozidas dentro do
`cook_target_into`, que corre **na MARCHA** da bomba. E a ponte da GPU, na rota `FullyGpu`,
**retorna antes de qualquer marcha**: o device produziu o quadro, a bomba não corre, e
`tap_streams` fica vazio.

⇒ o gizmo lê a caixa envolvente de uma tomada ⇒ `box_from_tap` → `None` ⇒ `resolve` → `None` ⇒
**não há gizmo**, sem um erro e sem um aviso.

⚠️ **A rota HÍBRIDA já estava coberta**, e o doc do `set_taps` conta essa cura: as tomadas
passaram a ser estado da **bomba** precisamente para cavalgarem *«a marcha que houver»*. O que
ninguém viu é que na `FullyGpu` **não há marcha nenhuma**. *Uma lei implementada num braço do
`match` e não no outro é meia lei* — e este repo já pagou essa exacta frase noutro sítio.

⚠️⚠️ **E foi a W5a que acordou o defeito.** Ele existe desde que a rota `FullyGpu` existe; faltava
um consumidor de tomada num grafo que o device reivindicasse **inteiro**. Dar um kernel ao
`motion.bezier_warp` foi exactamente isso: antes, ele derrubava a cadeia para a CPU e a tomada
aparecia **por acidente**.

⚠️ **Segunda vítima, que ninguém tinha reportado:** os **sinais** de um documento inteiramente no
device estavam mudos pela mesma razão.

**A cura:** `MotionCookPump::cook_taps_only` — **cozinha, não marcha**, no mesmo playhead em que o
device acabou. ⛔ Marchar ali simularia o tique duas vezes (uma no device, outra na CPU) e um nó
sequencial andaria a dobrar. O preço está nomeado: a tomada do gizmo cozinha na CPU a cadeia a
montante do nó **seleccionado**, e só se paga enquanto há tomada armada.

#### ⛔ As duas curas anteriores foram PALPITES, e a segunda provou a primeira errada

| ronda | o que eu fiz | como se soube que estava errado |
|---|---|---|
| 1 | movi o desenho ~2 000 linhas no quadro (ordem de z **inferida de números de linha**) | o gizmo **sumiu** |
| 2 | revertí a mudança de sítio | *«ainda invisível»* — logo a mudança nunca fora a causa |
| 3 | **instrumentei o quadro** | a linha do dono nomeou a condição em dez segundos |

⭐ **A de tamanho e casing, essa, era medida — e fica.** Era o que o report original pedia.

#### ⛔⛔ E a lição mais cara: A MINHA SONDA MENTIU

A sonda `why_the_warp_gizmo_is_not_there` deu `resolve = Some` na cena **exacta** do report,
enquanto o app dava `None`. Ela chama `advance_or_scrub_scoped` — a marcha da **CPU** — e o app
corre a do **device**.

> ***Uma sonda que escolhe a rota mede a rota que ela escolheu.***

É a segunda instância da família *«uma sonda que arma o sistema de outra maneira que o utilizador
mede outro programa»* ([memória](../../project-memory/feedback_a_probe_that_arms_a_module_by_env_var_measures_another_program_than_the_pill.md)).

⚠️ **E a resposta só veio porque a sonda do app imprime os DOIS lados.** O ramo do `None` só existe
porque eu já tinha errado duas vezes e precisei de distinguir *«não há retrato»* de *«a sonda não
corre»* — sem ele, um terminal silencioso não concluiria nada.

⏳ **ABERTO e nomeado:** não há portão que veja um overlay de canvas sumir do **quadro**. Os gates
desta jornada medem as duas metades — o retrato (`why_the_warp_gizmo_is_not_there`) e a tinta
(`the_gizmo_paints_geometry_and_paints_none_when_inactive`) — e a costura entre elas, que é onde o
defeito viveu, só se mede com uma janela real.

> ✅ **Meia frase desta ficou desactualizada no mesmo dia — ver §4 «A COSTURA GANHA PORTÃO».** A
> costura ROTA → TOMADA passa a ter gate com prova de mutação; o que fica sem instrumento é a
> outra metade, *o overlay sumir do quadro DESENHADO*, que continua a precisar de janela real.

---

### ✅ A DECISÃO EM ABERTO DO *«Use Selected Path»* — respondida por MEDIÇÃO, e o achado foi outro (2026-09-08)

A pergunta que ficou do report do `motion.spline_wrap` era: **o botão deve NOMEAR sozinho um
desenho sem nome, em vez de o recusar?**

⇒ **Não, e o caso não existe.** `vec_entities::initial_name` dá a todo path novo o nome
`Path {id}` no mesmo passe que lhe spawna a entidade — *um desenho deste app não pode nascer sem
nome*. Construir a nomeação automática seria construir a cura de uma população vazia.

#### ⛔ Mas a MEDIÇÃO devolveu o defeito verdadeiro: **a recusa nomeava a condição que não morde**

A frase única era *«Select a drawing … it needs a name and at least two points»* — e metade dela
descreve o que nenhum desenho falha, enquanto o motivo real (*nada seleccionado* · *o que está
seleccionado não é um desenho* · *o desenho tem menos de dois pontos*) **ficava por dizer**. É a
família da [nota ao lado de uma conta](../../project-memory/feedback_the_note_beside_a_count_describes_the_population_it_had_when_it_was_written.md):
a frase descrevia a população que ela tinha no dia em que foi escrita.

⇒ **`FormaEscolhida`**, quatro estados e quatro frases, na mesma lei do `fell` da ponte de GPU —
*toda saída passa por aqui e nomeia-se*. ⚠️ **E sai do MESMO laço que publica as formas**, nunca de
uma segunda varredura: quem está seleccionado e não aparece ali **não é um desenho**, porque o mapa
só tem `VecPath` — a recusa por omissão cai desse facto, e as outras três são escritas no elemento
que É o seleccionado.

⭐ **Gate:** `each_refusal_of_the_button_names_its_own_reason` mede que as quatro frases são
**distintas** e que nenhuma recusa toca o documento. ⚠️ **A régua é a distinção, não o texto** — um
gate que fixasse a redacção obrigaria toda melhoria de linguagem (e a i18n) a editar um teste.
⛔ Prova de mutação: colapsar dois braços na mesma frase reprova nomeando o par.

---

### ⛔⛔ O DESPERTAR NÃO ACORDAVA DOIS DOS TREZE — e a tabela escondia-o (2026-09-08)

Antes de correr a tabela na máquina calma, a régua foi auditada. **Dois defeitos, os dois na
família que o próprio despertar tinha sido escrito para curar** (*um corpus no ponto neutro de um
knob não testa esse knob*):

**1. O `motion.rotate` não tem um único `Slider`.** O controlo dele é um `ParamWidget::Angle`, e o
despertar só mexia em `Slider` ⇒ a linha dele cronometrava uma rotação de **0°**, que é
exactamente a identidade de que a tabela se queixava. ⚠️ **Um `Angle` é um NÚMERO com unidade em
graus, não um modo** — a razão escrita para excluir widgets (*«mexer num `Enum` ou num `Toggle`
mudaria o MODO do nó»*) nunca o cobriu. O `motion.bend` tem a mesma forma no **`direction`**, que é
justamente o param que a cláusula `applicable` dele lê: a coluna do dispositivo dizia **🟢** sobre
um nó com aquele knob adormecido.

**2. A fracção era a mesma para todos os hints.** Os oito `P0X..P3Y` do `motion.spline_wrap`
recebiam **o mesmo número** ⇒ os quatro pontos de controlo colapsavam num ponto, a cúbica media
comprimento zero, e o nó tomava o **atalho inerte**: a linha dele cronometrava um `clone` — *o
mesmo defeito que o despertar curou no `bezier_warp`, um nível abaixo dele próprio*. Hoje a
fracção varia pela **razão áurea** (determinística, nunca repete em `n` pequeno, e ⛔ não precisa
de uma lista escrita à mão de *«estes dois hints são um ponto»*, que envelheceria a cada param
novo). ⚠️ A faixa é `0,25..0,75` e o `0,25` em `i = 0` é **load-bearing**: a faixa do
`motion.scale` é `0..5`, e uma fracção de `0,2` pediria `Scale = 1`, que é a identidade.

⭐ **O censo que faltava:** `waking_a_node_takes_it_off_the_identity` — para cada um dos treze, a
saída do nó **aceso** difere da saída dele nos defaults. ⛔ **Duas provas de mutação, cada uma a
nomear o nó que previa:** voltar a lista de widgets a `Slider` acusa `motion.rotate`; congelar a
fracção acusa `motion.spline_wrap`.

⚠️⚠️ **E a 1.ª redacção do próprio gate comparava só a coluna `P`** — a **terceira** vez que esta
linha paga a mesma cegueira (a régua da figura do tutorial pagou-a antes): o `rotate` e o `scale`
escrevem `rot` e `size`, **nunca `P`**, e liam-se *«não mudou»* com o nó a girar. A comparação é do
**stream inteiro**, a única que não tem de saber o que cada nó escreve.

> ⭐ *Um `clone` e um deformador barato leem-se **iguais** numa coluna de razão* — é por isso que a
> régua do despertar tem de ser a **SAÍDA**, nunca o relógio.

---

### ⏳ A RECUSA DO `motion.bend ▸ direction` — RECONFERIDA, e continua de pé (2026-09-08)

⚠️ **§0.0 manda reconferir uma nota de inalcançabilidade sempre que alguém mexe no número que a
sustentava** — e esta linha mexeu no canal das reduções três vezes neste ciclo (a W1a, a
`reduce_<earlier>()` da W1, o `bezier_warp` na W5a). Reconferida:

A recusa (`applicable: Some(|p| p("direction") == 0.0)`) diz que o `x_extent` é um `Max` sobre
`abs(v.x − pivot_x)` e que num quadro rodado ele teria de dobrar sobre `abs(dx·cos + dy·sin)` —
com o `cos`/`sin` a serem o **polinómio HR-5** escrito uma segunda vez dentro da string da
redução.

⇒ **A premissa está CERTA hoje, e foi verificada no gerador, não deduzida:**
[`reduce_stage.rs`](../../crates/ph2d-gpu-cook/src/reduce_stage.rs) monta o módulo da redução com
os params, as ligações, os acessores `reduce_<earlier>()`, o `reduce_value` e o `main` — e **não
cola o `wgsl_lib` do nó**. Não há hoje onde pôr uma função auxiliar.

⭐ **O que a dissolveria, nomeado:** um campo `helpers: &'static str` no `ReduceSpec` (append-only,
como a `reduce_<earlier>()` já foi), colado antes do `reduce_value` **naquele módulo**. ⛔ Colar o
`wgsl_lib` inteiro **não** serve: ele chama `read_in_*`/`write_*`, que não existem ali. E a
objecção das «duas cópias» cai se o WGSL do polinómio for **um `const` da crate do `bend`** usado
pelos dois sítios — *uma string colada duas vezes pelo compilador não é uma lei escrita duas
vezes* (⚠️ e a casa já copia o `trig.rs` **31 vezes** entre crates, por convenção escrita: *«o que
se partilha é o algoritmo, não um símbolo»*).

⛔ **Não foi feito nesta janela**, e a razão é escopo: é substrato (`ph2d-nodegraph`) mais paridade
medida no dispositivo, que é uma wave com espec própria — não a cauda desta.

---

