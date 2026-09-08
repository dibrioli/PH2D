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

*(cada wave escreve a sua secção aqui ao fechar, com a tabela medida e as premissas que a
implementação derrubou)*
