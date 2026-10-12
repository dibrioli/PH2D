# ARQUIVO — 88_plano_parametros_nos_unidades_e_slider.md (história, 194 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`88_plano_parametros_nos_unidades_e_slider.md`](../../Motion%20Nodes/88_plano_parametros_nos_unidades_e_slider.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-269,288-301,478-715` do original.
>
> ⚠️ **A única alteração ao corpo:** 1 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---


> A §6 dizia *"nó a nó, ou família a família"*. A §0 do CLAUDE.md manda **medir antes de
> decidir** — então a varredura começa por um censo executável, não por ordem alfabética.

**A sonda:** `cargo test -p ph2d-node-registry-init --test param_census -- --ignored --nocapture`
(roda na crate que registra os 118 nós — o build mais barato que os enxerga; uma sonda na
shell mediria o mesmo e custaria o app inteiro).

**O retrato de 2026-08-08:** **118 nós · 411 params · 395 com hint · 105 com unidade.**
⚠️ **Re-medido em 2026-08-09: 118 nós · 420 params · 404 com hint · 116 com unidade · 48
magros** (era 50). *Um censo é um retrato COM DATA* — dois destes params são o `offset_x`/
`offset_y` da §9.1 e os outros sete já estavam no `main` quando esta linha reabriu. Não
acredite no número desta linha: rode a sonda.
Só **um** nó tem param sem hint nenhum (`value.attribute`, cujo controle real é o picker
de canais — o `f32` ali é o `mode` que o picker escreve). **Cinquenta** nós têm ≤ 2
controles, e é aqui que a curadoria mora: *magro por natureza* e *magro por omissão* são
coisas diferentes, e o censo não sabe distingui-las — quem distingue é a referência.

### O mapa, com o veredito de cada família

| Família | Estado MEDIDO | Veredito |
|---|---|---|
| **TRANSFORM** (`move` · `rotate` · `scale` · `transform` · `mirror` · `orbit`) | `scale` escrevia uma coluna **Vec2** a partir de **um** número; `mirror` pregava a linha no **centroide** | ✅ **FECHADA** (2026-08-08) — o 2º eixo e o `Axis Offset`, smoke `PH2D_TRANSFORM_SMOKE=1` |
| **ECHO** (`motion.trail`) | 3 params contra os 8 da referência | ✅ **FECHADA no PADRÃO-OURO** (2026-08-08) — **SETE** knobs: `length · spacing · fade · shrink · hue_shift · saturation · spin`. Smoke `PH2D_ECHO_SMOKE=1`. ⚠️ O `spacing` **não custou estado novo** (a coluna `trail_age` já sabia há quantos ticks o último eco foi deixado ⇒ a promoção é pergunta ao ESTADO, não a um contador), e **nenhuma referência tem `spin`** |
| **DEFORMERS** (`bend` · `twist` · `spherize` · `four_point_warp` · `kaleidoscope` · `slit_scan`) | `four_point_warp` 8 · `kaleidoscope` 4 · `bend`/`twist` 3 · **`spherize` 1** · **`slit_scan` 1** | ✅ **FECHADA** (2026-08-09) — **UM** dos dois magros era omissão: o `spherize` ganhou `offset_x`/`offset_y` (§9.1). O `slit_scan` é magro **por natureza**, com o mecanismo escrito (§9.2) |
| **VALUE** (24 nós) | quase todos 1-2 params | ⛔ **RECUSADA COM MOTIVO** — um `value.unary` é *um verbo sobre um número*; um param a mais ali é cerimônia, e a §12 (domínio de valor) desenhou essa família justamente para ser pequena e componível |
| **ESTRUTURAIS** (`util.reroute*` · `motion.combine` · `integrate` · `output` · `luminance` · `make_point` · `morph` · `sim.zone` · `value.switch` · `pulse.sample_hold`) | 0 params | ⛔ **RECUSADA** — zero params é o contrato deles, não uma lacuna |
| **RIG** (`fk` · `ik_2bone` · `fabrik` · `rubber_hose` · `skin_deformer` · `skeleton`) | magros | ⏸ **ADIADA** — o CLAUDE.md §5 defere rig+skinning "pro FIM de tudo" |

⚠️ **E o `motion.lattice` SAIU da linha dos deformers** — ele é `NodeUiCategory::Source`, um
gerador de grade com jitter (`rows`/`cols`/`spacing`/`seed`), não um deformer. Estava ali por
engano de classificação, e uma tabela que classifica errado é como alguém "completa" a família
errada; os 4 params dele são a forma dos DISTRIBUIDORES, e é lá que ele deve ser julgado.

### §9.1 — `motion.spherize`: magro por OMISSÃO (a lente não podia ser POSTA em lugar nenhum)

Params medidos: **`radius`, e só.** O centro era o **centroide** do layout, derivado de duas
reduções `Sum`, e **não era autorável**.

**Por que isto passa na lei desta varredura** (*um param entra quando nomeia capacidade
inexprimível no grafo inteiro*) — as três saídas foram tentadas e nenhuma existe:

- `motion.move` antes do spherize **leva o centroide junto** ⇒ a lente nunca sai do meio.
- O campo `falloff` mascara a **MISTURA**, não o **CENTRO** ⇒ ele só esmaece um bulge que
  continua radial em torno do ponto errado.
- Um elemento-isca longe para arrastar o centroide muda a CONTAGEM e a arte.

E a evidência mais forte é interna: **a própria família já nomeia a capacidade** —
`motion.bend`, `motion.twist` e `motion.kaleidoscope` carregam `pivot_x`/`pivot_y` desde que
foram escritos. A lente era a única que não a exprimia. ⚠️ E **não havia cerca de Chesterton**:
o [doc 24](../../Motion%20Nodes/24_four_point_warp_spherize_nota_adr.md) diz *"`radius` (world) centrado no
centroide"* como FATO, nunca como decisão com motivo.

⚠️ **É um OFFSET do centroide, não um centro absoluto — e a razão não é gosto:**

1. **O default TEM de reduzir LITERALMENTE** (a lei desta §9): `c + (0,0)` **é** `c`. Um
   `center_x`/`center_y` absoluto com default `0` seria a **origem do mundo**, e arrancaria a
   lente do assunto em **todo documento já autorado**.
2. **Uma lente sobre um conjunto VIVO segue o assunto** — estes streams derivam (um emitter os
   alimenta, um `motion.move` os carrega), e um centro absoluto escorregaria do layout enquanto
   o layout vai embora.
3. **O nome é próprio** porque `pivot_x`/`pivot_y` significam *coordenada ABSOLUTA de mundo*
   nos três irmãos; reusar a palavra para uma grandeza relativa poria dois sentidos num nome só
   dentro da mesma família.

**Três gates, três mutações, três sangram** — e cada uma no seu:

| Mutação | `…can_be_placed_off_the_centroid` | `a_zero_offset_leaves_…on_the_centroid` | `…travels_with_its_subject` | paridade GPU |
|---|---|---|---|---|
| a CPU **ignora** o offset | 🔴 | verde | verde | verde |
| o offset vira **absoluto** | verde | 🔴 | 🔴 | verde |
| o **kernel do device** larga o offset | verde | verde | verde | 🔴 |

⚠️ **O oráculo é o LUGAR, nunca uma magnitude:** a lente tem um **ponto fixo** (quem está
exatamente no centro não tem direção para ser empurrado e volta verbatim), então *onde a lente
está* se lê direto da saída, sem constante de tolerância. Mover a lente para `x=1` faz o
elemento em `x=0` ser empurrado **para o outro lado** — uma troca de SINAL que nenhuma força
falsifica.

⚠️ **E o gate de paridade CPU×GPU tinha o buraco clássico — a fixture não continha o
fenômeno.** O `spherize_chain` já tinha um `offset`, mas era a **TRANSLAÇÃO DA GRADE** (para o
centroide não ser a origem por simetria); o offset da LENTE nunca era exercitado, então o
device podia ignorar `params.offset_x`/`offset_y` com a paridade **verde nos dois lados em
zero**. Agora a cadeia recebe **dois deslocamentos DISTINTOS** (`translate` e `lens`) — um
kernel que os trocasse também sangraria — e a 3ª linha da tabela é o **controle** (lente de
volta no centroide).

### §9.2 — `motion.slit_scan`: magro por NATUREZA (o mecanismo, para ninguém o "completar")

Param medido: **`lag`, e só.** A referência (o *Time Displacement* do AE, o Slit Scan da
Cavalry) nomeia três coisas além do deslocamento máximo, e **as três já são exprimíveis**:

- **O EIXO da varredura** — a rampa segue a ordem do stream, e o próprio doc-comment do nó
  manda pôr um **`motion.sort`** a montante (ordenar por X faz o atraso crescer da esquerda
  para a direita). É a arquitetura de GRAFO fazendo o que ela existe para fazer.
- **A DIREÇÃO** — `motion.sort` tem `descending`, medido. Um param de "reverse" aqui seria a
  segunda porta para uma pergunta que já tem dono.
- **A FORMA da rampa** — o campo `falloff` multiplica o atraso **por elemento**, então um
  `field.*` + `field.remap` com uma **Curva** molda a rampa arbitrariamente. Um "ease" próprio
  duplicaria o canal de curva que a família `field.*` já entrega.

*O que sobra — `Time Resolution` — é amostragem de raster e não tem sentido num stream de
instâncias.* ⇒ **1 param é o número certo**, e fica escrito para que a próxima varredura não o
leia como lacuna.

> ⚠️⚠️ **EMENDA (2026-09-16, ciclo 7 W3 — doc 112 §4-sexies): o primeiro ponto respondia a OUTRA
> pergunta.** O `motion.sort` a montante dá o eixo **reordenando o stream para sempre** (a ordem
> de desenho, o pareamento por índice, o `id` — o efeito colateral que a folha 10 da conferência
> nomeou e curou no `field.index_range`), e o campo **multiplicava** a rampa por índice, então um
> atraso só pela POSIÇÃO não se exprimia sem tocar na ordem. ⇒ o nó ganhou **`Delay By`**
> (`Order` · `Field`); os outros três pontos ficam de pé. *Uma recusa medida responde UMA
> pergunta.*

### §9.3 — O MAPA cobre 54 dos 118 nós, e o resto foi varrido por PROPRIEDADE

⚠️ **A tabela acima diz *"o veredito de cada família"* e nomeia SEIS** — TRANSFORM (6 nós) ·
ECHO (1) · DEFORMERS (6) · VALUE (23) · ESTRUTURAIS (12) · RIG (6) = **54 de 118**. Os outros
**64** (`field.*` · `force.*` · `fx.*` · `pulse.*` · `sim.*` · `source.*` · distribuições ·
simulação · cor · streams) nunca tiveram linha, e uma tabela que se anuncia completa e não é
faz a próxima varredura parar cedo.

Varridos pelo censo, o retrato deles é gordo (`field.remap` 12 · `field.radial_sweep` 10 ·
`motion.emitter` 10 · `motion.spline_wrap` 10 · `boids`/`noise`/`oscillator`/`source.shape`/
`value.pattern` 9). Os magros restantes são magros **por natureza** pelo mesmo teste do
`slit_scan`: `force.drag` (1) é *um coeficiente de arrasto*, `sim.step` (1) é *um passo*,
`value.unary`/`median`/`slope`/`smooth`/`normalize`/`reduce` (1) são a família já recusada
com motivo.

**O que a varredura ACHOU foi outra coisa, e num eixo que o mapa não olha:** quatro nós com
mais params do que hints — `motion.tint` 9/3 · `fx.glow` 9/6 · `fx.drop_shadow` 6/3 ·
`motion.strobe` 6/3. A leitura ingênua é *"quinze params desenhados com o nome de fio"*, que é
exatamente a tela que a Wave A existiu para remover. **Ela está errada, e os quatro estão
certos:** o catálogo tem DOIS widgets que se declaram num hint só e desenham params vizinhos —
`ParamWidget::Color` (os quatro canais RGBA numa amostra) e `ParamWidget::Channels` (o
`mode_param` f32 dobrado dentro do seletor). ⚠️ O único que não fecha à primeira vista é o
`motion.strobe`, cujo 4º canal de cor é `flash_amount` e não um alfa — e **o arquivo dele já
carrega a explicação**: o kernel faz `a = flash_amount · glow` e compõe por `over`, então
aquele número **é** o alfa daquela cor.

⇒ **Zero defeitos, e a medição vira LEI:** `every_declared_param_is_drawn_by_some_widget`
(a 4ª do `param_widget_conventions`, e a única escrita do lado dos PARAMS). Três mutações,
três sangram, cada uma nomeando o param: um hint renomeado ⇒ `motion.bend.pivot_y` · um canal
fora do grupo de cor ⇒ `motion.tint.a` · a varredura esquecer o `Channels` ⇒
`value.attribute.mode`.

⚠️ **E a 3ª mutação é o achado do processo, não um rodapé.** O 1º corte do gate conhecia só o
`Color` e isentava `value.attribute.mode` numa allowlist *"com o motivo escrito"* — o motivo
estava **ERRADO**: aquele param não é exceção, ele é **coberto**. A allowlist não documentava
uma decisão, escondia um buraco na varredura; um segundo nó `Channels` teria sido acusado e a
cura óbvia — mais uma linha na tabela — teria entrincheirado o erro. Quem expôs isso foi uma
**mutação que NÃO sangrou** (e que era, ela própria, inválida: `register_param_ui` é um
`insert`, e a chamada injetada vinha ANTES da original). *Uma mutação que não sangra é um
buraco de gate ou uma mutação inválida — e a única forma de saber qual é ir ver.* A allowlist
e o gate de staleness dela foram **deletados**: tabela vazia mais gate que a itera é um gate
que não pode falhar.

### ⚠️ O `motion.duplicator` tem ZERO params, e isso está quase todo CERTO

A tabela §B da pesquisa do Cavalry é grande (Distribution · Shape Position/Rotation/Scale
· Visibility/Opacity · Auto Id / Shape Id · Shape Time Offset · Use Index Context · Skip
Invisible), e a leitura ingênua é *"faltam sete params"*. **Ela está errada**, e o motivo
é arquitetural: o Duplicator do Cavalry é um **mega-nó** e o nosso é um **grafo**.

- *Distribution* — lá é um dropdown de 21 tipos; aqui são **21 nós** que alimentam a porta
  `points`. Trazê-la para dentro seriam duas portas para a mesma pergunta.
- *Shape Position/Rotation/Scale/Opacity por-cópia* — lá são alvos que Falloff/Stagger
  modulam dentro do nó; aqui são `motion.move`/`rotate`/`scale` **a jusante**, que é o que
  torna o falloff composável.
- *Skip Invisible Duplicates* — é o `motion.cull`.

**O gap REAL é um só:** *que forma vai em que ponto*. Nós fazemos o **produto cartesiano**
(N formas × M pontos = N·M cópias) e a referência tem `Auto Id` (cicla) / `Shape Id`
(fixa). Com 3 formas e 100 pontos, *"cem pontos, cada um recebendo uma das três formas"*
**não é exprimível hoje**. Isso é semântica de contagem, não um knob: ⇒ **wave própria**.

### ⚠️ E a recusa do `hueShift` estava na CAMADA ERRADA — a correção vale mais que o knob

Esta seção dizia que o hue shift não entrava porque *"a cor neste app passa por OKLCH"*.
Isso é verdade da **AUTORIA** — o picker, o editor de gradiente, a paleta — e **falso do
COOK**: a coluna `tint`, o `motion.color_ramp` (*"the ramp is evaluated in linear RGB —
the same space the tint column and the compositor use"*) e a lowering inteira falam
**linear RGB**. Girar matiz em OKLCH ali dentro meteria um **segundo espaço de cor no meio
do cozimento**, e o preço não é teórico: a ida polar é uma `cbrt` + `atan2` **por linha e
por tick**, num laço por-elemento.

O operador certo nesta camada é a **rotação que PRESERVA a luma**, uma matriz 3×3 sobre
linear RGB — exatamente o `feColorMatrix type="hueRotate"` do SVG, que é especificado em
linearRGB pelo mesmo motivo. `sincos` **uma vez por tick**, nunca por linha; a matiz e a
saturação compostas numa matriz só.

*A lição não é sobre cor: uma recusa tem de nomear a CAMADA em que ela vale. "Este app usa
X" pode ser verdade de uma camada e falso da vizinha, e uma recusa que não diz qual delas
bloqueia trabalho legítimo.*

