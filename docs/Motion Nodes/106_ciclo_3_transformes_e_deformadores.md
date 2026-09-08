# Ciclo 3 — TRANSFORMES & DEFORMADORES · «Dobrar o mundo»

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
[`motion_deformadores_probe.rs`](../../shells/desktop/src/motion_deformadores_probe.rs), três
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

O censo do ciclo 1 (`no_param_the_panel_offers_falls_off_the_card`) e o do ciclo 2
(`every_param_the_card_hides_has_a_declared_reason`) correm sobre o grupo, e o cartão dos treze
é lido com o olho do doc 101 (secções, LOD, o que o `motion.bezier_warp` faz com 24 rows).

### W5 — OS DOIS QUE FALTAM AO DISPOSITIVO, ou a razão de cada um

`motion.bezier_warp` (24 params, patch de Coons, bbox no `eval`) e `motion.spline_wrap` (cúbica
autorada + reparametrização por comprimento de arco). ⚠️ **A saída aceitável é kernel OU uma
razão nomeada com o preço medido** — a lei nº 1 não admite a etiqueta `CPU-only` sozinha.

### W6 — A MEDIÇÃO (passo 5) e o TUTORIAL (passo 6)

---

## §4 — Registo das waves

### ✅ W1a — O CENTROIDE CHEGA AO DISPOSITIVO (2026-09-07)

**A recusa mais bem escrita do módulo estava errada por um dia que já tinha passado.** O
`motion.transform` declarava `applicable: Some(|p| Pivot::of(p("pivot_mode")) != Pivot::Centroid)`
com o comentário a dizer que o centroide *«é uma REDUÇÃO sobre o stream e não um mapa por
elemento»* e a **nomear a própria cura**: *«o canal `reduce → broadcast → map` que os deformadores
usam é o que a levantaria, e isso é uma wave, não uma linha»*. Esse canal já tinha shipado
(GPU/M5) e o `motion.spherize` já media o centroide com ele. ⇒ duas `ReduceSpec` (`cx`/`cy`) e a
recusa sai; o custo que ela declarava — *«um layout pivotando no próprio centro perde a residência
de GPU neste nó»* — desaparece.

#### ⛔⛔ E o red-first apanhou um segundo defeito, este a DESENHAR ERRADO

O kernel **ignorava o `pivot_mode`**: perguntava `if (params.pivot_x != 0.0 || …)` em vez de olhar
o modo. Um ponto digitado e deixado para trás — a row está escondida pelo `ParamGate`, o **valor
não é apagado** — vazava para o dispositivo: CPU `−14,629749` contra device `−13,260749`, **1,369
unidades de mundo**, sem erro nenhum em lado nenhum. ⚠️ *O gate que existia corria `pivot_mode = 1`
COM o ponto, onde os dois campos concordam — um gate que só corre o modo em que os dois concordam
não mede o campo, mede a coincidência.*

#### ⭐⭐⭐ E a medição do ε deu o achado da wave, com o sinal AO CONTRÁRIO

`measure_the_centroid_pivot_epsilon` / `measure_the_spherize_centroid_epsilon`:

| nó | fixtura | antes | depois |
|---|---|---:|---:|
| `motion.spherize` | 409 600 elementos, layout a `4` da origem | **54 365 % da barra** (`2e-4`) | **7,6 %** |
| `motion.spherize` | 16 384 — *o tamanho do próprio gate* | **535 %** | 1,9 % |
| `motion.transform` | 409 600, layout a `4,3` | **917 % da barra** (`2e-3`) | 0,4 % |

**O desvio não era do kernel, era da CPU.** A soma dela é **sequencial** e a do dispositivo é em
**árvore**, e a segunda é a mais certa (`log n · ulp` contra um passeio aleatório de `√n · ulp`
sobre parciais que chegam à magnitude do layout inteiro). ⛔ **O `motion.spherize` shipa assim há
meses com o gate de paridade verde** — porque ele mede UM tamanho (16 384) e uma lente de raio `6`,
pequena de mais para o centroide morder. *Uma folga medida num tamanho é uma afirmação sobre esse
tamanho.*

⇒ a média passa a ter **uma porta**, [`ph2d_nodegraph::reduce_meta::centroid_of`], ao lado da
`ReduceSpec` que é a metade de dispositivo dela, com acumulador `f64`. O gate dela corre **na CPU e
portanto no CI** — os de paridade precisam de adapter, são `#[ignore]` e **nunca correram**.

⚠️ **Duas armadilhas de fixtura, as duas apanhadas por mutação:** `65 536 × 4 = 2^18` soma-se
**exactamente** em `f32` (uma fixtura de valores iguais deixa o gate verde sobre o fold que ele
existe para reprovar), e a `40` em vez de `400` a mutação morre por `2,1×` em vez de `159×` — *o
erro é proporcional à magnitude das parciais, logo a distância à origem é metade da fixtura*.

### ✅ W1b — ⛔⛔ O `motion.drive` NÃO COMPILAVA NO DISPOSITIVO em três canais, e o gate que devia vê-lo era cego às VARIANTES (2026-09-07)

Achado a correr as suítes de paridade de GPU do módulo: `the_colour_loop_closes_the_same_way_on_the_device`
morria com `unknown identifier: drive_resolve`. A variante `DRIVE_HSV` (canais **Hue**,
**Saturation**, **Value**) tem uma `wgsl_lib` **própria**, que é uma cópia à mão de metade da do
irmão: ela traz `drive_round` e `drive_combine` e **não traz `drive_base` nem `drive_resolve`**, que
o corpo chama. ⇒ conduzir matiz/saturação/valor no dispositivo nunca funcionou.

⭐⭐ **A causa de segunda ordem é o instrumento:** o
`every_registered_kernel_validates_across_the_whole_presence_space` — o gate que valida WGSL **sem
adapter, em toda lane de CI** — varria `reg.gpu_kernel(id)`, o kernel **BASE**. Um nó com
`variant_by_param` nunca dispacha o base: o sequenciador chama `GpuKernel::resolve(param)`. **Dez
nós declaram variantes** (`drive` · `move` · `noise` · `oscillator` · `wiggle` · `spring` ·
`stagger` · `orbit` · `falloff` · `distribute_radial`) e nenhuma delas alguma vez encontrou um
compilador nessa lane. *Um gate que varre os kernels REGISTADOS é cego às variantes, e a variante é
onde a lei se copia.*

**Cura em duas metades:** o gate passa a varrer as variantes (um param de cada vez a partir dos
defaults, com os índices exactos de cada `Enum` — ⚠️ uma variante escolhida por uma **combinação**
escapa, e isso está nomeado em vez de prometido), e a `DRIVE_LIB_HSV` **concatena** a biblioteca do
irmão em vez de copiar metade dela (o `concat!` só aceita literais ⇒ ela virou um `macro_rules!`).

### ✅ W1c–W1f — A PORTA, e os quatro nós que a adoptaram (2026-09-07)

Módulo irmão novo em `ph2d-nodegraph` (**append-only**, sem tocar contrato nenhum — ADR-0107 /
briefing B'): [`pivot`](../../crates/ph2d-nodegraph/src/pivot.rs) — o `PARAM`, os `LABELS` (cuja
**ordem é contrato**, porque o valor vai no ficheiro), o `PivotMode` com `of`/`resolve`, o
prólogo em WGSL (`pivot_wgsl!`) e o par de reduções `CENTROID_CX`/`CENTROID_CY`.

E o módulo escreve o que **não** faz: não decide o DEFAULT de nenhum nó (esse é a lei da
identidade byte-a-byte, e é diferente em cada um), e nomeia quem fica de fora por **natureza**
(`rotate`/`scale` escrevem `rot`/`size`, não `P`) e por **cerca medida** (o offset do `spherize`
é relativo ao centroide).

| nó | default | reduções | o que custou |
|---|---|---|---|
| `motion.transform` | `World Origin` | 2 | a recusa saiu; o kernel passou a olhar o MODO |
| `motion.kaleidoscope` | **`Point`** | 2 | o **divisor** não é `params.count` (ver abaixo) |
| `motion.bend` | **`Point`** | 4 | a extensão deixou de depender do pivô |
| `motion.twist` | **`Point`** | 3 | o substrato aprendeu a **encadear** reduções |

⚠️ **Os defaults DIVERGEM de propósito.** O `motion.transform` sempre escalou em torno da
origem; os outros três sempre honraram o ponto digitado. Com o pivô em `(0,0)` os dois valores
dão o mesmo número — mas não a mesma UI: com `World Origin` o `ParamGate` esconderia dois
sliders que o artista já usa. *O default é a lei da identidade de cada nó, não uma propriedade
do enum.*

#### As três coisas que só a construção revelou

1. ⛔ **O divisor da média NÃO é sempre `params.count`.** A média é `Σp / n` com `n` a contagem
   do stream sobre o qual a **redução** correu. Num kernel por elemento isso é `params.count`;
   no `motion.kaleidoscope`, que é um `StreamOp::SourceRows` e emite `segments · n`, aquilo é a
   contagem da **saída** e o centroide saía `segments` vezes menor — medido, **`6,7e-1` de
   divergência, `3300×` a barra**, na primeira corrida do gate. O prólogo passa a receber o
   divisor como argumento.
2. ⭐⭐ **A extensão do `motion.bend` passou a ser DERIVADA:** `max|x − p|` é
   `max(xmax − p, p − xmin)`, e em `f32` a igualdade é **exacta** (a subtracção é correctamente
   arredondada e o arredondamento é monótono). Sem isso o `Centroid` seria uma redução a
   depender de outra. ⭐ E isto deixa o `direction` — hoje recusado no dispositivo *«porque a
   redução `x_extent` não roda com o quadro»* — a um passo, porque a redução deixou de ler o
   pivô. **Nomeado, não feito:** o eixo rodado precisa da senoide parabólica dentro do `value`
   de uma redução, e uma `ReduceSpec` não tem biblioteca.
3. ⭐⭐⭐ **O `motion.twist` obrigou o SUBSTRATO a crescer, e a adição é append-only.** O `r_max`
   dele mede um **raio**, que não é separável como a extensão em X. ⇒ *uma redução pode agora
   ler as reduções declaradas ANTES dela* (`reduce_<nome>()` no módulo de mapa dela). Uma spec
   que não chame nada gera o mesmo módulo de sempre, byte a byte.
   ⚠️ **E a primeira redacção rebentou no `create_bind_group`:** uma binding declarada e **não
   lida** desaparece do layout reflectido, e o bind group ficava com uma entrada a mais
   (`(4) does not match … (3)`) — a mesma lei que a `src` do módulo já obedecia. A declaração e
   a entrada passam a ser decididas pelo **mesmo predicado** (`reads_earlier`).

#### E as premissas MINHAS que caíram

- **«identidade de ponteiro prova que o nó usa a porta»** — falso: mesmo num `static`, o `&[…]`
  é uma constante **promovida** e o leitor noutra crate re-materializa-a (`0x…21b0` contra
  `0x…9178`). O gate reprovava sobre código correcto. *Uma régua de identidade que a linguagem
  não garante mede o compilador, não o código* — o que fica compara os CAMPOS.
- **«a ε do centroide é mais larga»** — verdade, e por muito mais do que eu escrevi: ver W1a.

#### Quatro tectos de LOC, todos curados por CORTE (nunca por isenção)

`kaleidoscope` 823 → **587** · `spherize` 740 → **630** · `bend` 705 → **598** · e o `twist` que
não chegou a estourar recebeu a mesma costura. Os quatro ficam com a mesma forma que o
`motion.drive` e o `motion.noise` já tinham: **`lib.rs` = o que o nó É · `kernel.rs` = o que o
dispositivo corre · `ui.rs` = como ele se apresenta · `*_tests.rs` = uma pergunta por ficheiro.**

### ✅ W2 — O ESPELHO CHEGA AO DISPOSITIVO: **10 de 13 passam a 11** (2026-09-07)

Não era lacuna de param, era lacuna de **cobertura**, e a folha 05 §0 escreveu-a em
**2026-08-09** sem nunca ter sido fechada. Kernel `StreamOp::SourceRows` na forma do irmão
`motion.kaleidoscope`, com a linha de espelho a sair das duas reduções da porta do pivô
(divididas por `window_src_n`, **não** por `params.count` — a armadilha que a W1d mediu).

⚠️ **Um tropeço novo, com nome: `read_P` e não `read_in_P`.** O prefixo da porta só aparece num
nó com **mais de uma** entrada. O portão de validação do naga apanhou-o **sem GPU nenhuma**, e
só na máscara em que a coluna está **ausente** — com ela presente o símbolo existia na mesma.

⛔ **Dois knobs recuam, com o mecanismo escrito e um gate a fixá-lo:** o `reindex` (escreve
`Index`/`Count` novos — o mesmo `applicable` do caleidoscópio) e o `flip_rot` (reflectir `rot` e
`vel` do gémeo pede uma binding de escrita **condicional à presença da coluna no template**; um
`rot` ausente seria **cunhado** pelo device, que é divergência de SHAPE e não um ε).

### ✅ W3 — O CISALHAMENTO: o terço do afim que faltava (2026-09-07)

Um grupo chamado *TRANSFORMES* sem shear é um buraco que um profissional nota na primeira hora
(AE *Transform* ▸ **Skew + Skew Axis** · Illustrator ▸ **Shear** · Blender ▸ `Shear`). A folha 04
nomeara-o — *«é afim: caberia no `motion.transform`»* — e ficou por fechar.

⛔ **É uma INCLINAÇÃO e não um ÂNGULO, por duas razões medidas:** (1) HR-5 — a casa é
transcendental-free e um `tan` construído da senoide parabólica *não é uma ε mais larga, é outra
curva*; (2) `tan` **explode** a `±90°`, então um slider em graus gastaria metade do curso num
regime que devolve `NaN` e o teto teria de ser um número escolhido. ⇒ o knob é a inclinação —
exacta, ilimitada — com a régua escrita: `1` é 45°, `0,5` é ~26,6°, `2` é ~63,4°.

⚠️ **O ramo NEUTRO fica escrito à parte, nos dois caminhos**, e não sai da expressão geral com o
knob a zero: `a + 0.0` **não** é `a` quando `a` é `−0.0`. *A identidade é da ESTRUTURA* — a mesma
disciplina que o `folded_offset` já tinha para o pivô, e o gate compara BITS.

⚠️ **E com cisalhamento a dobra do pivô deixa de ser por EIXO** (`c − c·M` mistura os dois), o
que faz do pivô o único regime em que o kernel e a CPU têm o que discordar — é lá que a paridade
corre. A régua do gate é o **PONTO FIXO**: o elemento que está no pivô não se move, qualquer que
seja o cisalhamento.

⚠️ E uma assersão MINHA reprovou sobre código correcto: pedi `x == 0` a um cisalhamento em Y
sobre um ponto em `x = 3` — *um cisalhamento não move o eixo que ele lê*. A fixtura passou a ter
`y ≠ x`, porque em `(3, 0)` o resultado `[3, 3]` satisfaz as duas afirmações por acidente.

### ✅ W4a — O CENSO DA FAMÍLIA, e as duas coisas que ele apanhou no CARTÃO (2026-09-07)

Um censo sobre o **catálogo inteiro** (`the_pivot_question_has_one_vocabulary`, em
`ph2d-node-registry-init`): todo nó que declara o param do pivô pinta-o como `Enum`, **com os
rótulos da porta e na ordem da porta**, chama-lhe **a mesma palavra**, gateia as duas
coordenadas ao modo que as lê, e declara as duas somas sem as quais o `Centroid` existiria no
cartão e o kernel leria um símbolo que não há. ⚠️ Ele varre o catálogo, não uma lista — o quinto
nó entra sozinho. Prova de mutação: mudar o rótulo do `bend` para *«Pivot Mode»* reprova.

E a sonda que imprime **as rows que o cartão de facto pinta**
(`what_the_card_shows`) — o instrumento que prova que um passo de smoke que manda clicar numa
linha não está a mandar clicar numa linha que não existe — apanhou duas:

1. ⛔ **O `motion.transform` chamava ao pivô «Scale About»** enquanto os três irmãos diziam
   «Pivot». Renomeado — e o nome antigo já estava **errado** desde a W3, porque o pivô passou a
   ser também o ponto fixo do cisalhamento.
2. A ordem das rows dele passou a ser a do *Transform* do After Effects:
   **Pivot · Scale · Uniform · Skew X · Skew Y · Offset X · Offset Y**.

⏳ **E ela deixou à vista um terceiro, que é da W4:** o `motion.bezier_warp` pinta
`In X · In Y · Out X · Out Y` **quatro vezes**, sem dizer de que aresta é cada grupo — 24 rows
em que o artista não consegue escolher.

### 🔬 A CENA DE SMOKE — `PH2D_GPU_COOK_DEMO=111`

`grid(300×300) → move → mirror → twist → transform → output`, com o pano a **450 unidades** da
origem. ⚠️ **O deslocamento é a cena inteira:** com o pano centrado os três modos dão a mesma
imagem, e a cena ensinaria que a escolha não importa.

### ⏳ ABERTO — dois vermelhos de GPU que já estavam no `main`

1. **`value_slope_kernel_matches_the_cpu_on_the_device`** — mede `1,05023384e-4` contra a barra de
   `1e-4`, **5 % acima**. Já vem com o diagnóstico escrito no próprio teste (atribuído por ablação
   em 2026-08-11, e a nota pede o número **noutra máquina** antes de recalibrar). Reproduzido
   idêntico numa worktree limpa em `main` — **não é desta linha**.
2. *(fechado nesta wave)* `the_colour_loop_closes_the_same_way_on_the_device` — era a W1b.
