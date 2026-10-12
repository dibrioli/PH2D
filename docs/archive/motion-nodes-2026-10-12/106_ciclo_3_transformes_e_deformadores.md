# ARQUIVO — 106_ciclo_3_transformes_e_deformadores.md (história, 667 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`106_ciclo_3_transformes_e_deformadores.md`](../../Motion%20Nodes/106_ciclo_3_transformes_e_deformadores.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-218,415-464,520-563,739-810,873-945,986-1082` do original.
>
> ⚠️ **A única alteração ao corpo:** 5 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

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
briefing B'): [`pivot`](../../../crates/ph2d-nodegraph/src/pivot.rs) — o `PARAM`, os `LABELS` (cuja
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

### ✅ W5a — A FRONTEIRA CURVA CHEGA AO DISPOSITIVO: **11 de 13 passam a 12** (2026-09-08)

O `motion.bezier_warp` não precisava de nada novo. Ele é a **mesma forma** do irmão
`motion.four_point_warp` — quatro reduções de caixa envolvente (`Min`/`Max`, que são exactas
sobre floats) e um mapa por elemento que as lê — com outra álgebra no meio, e essa álgebra é
**polinomial**: um patch de Coons são quatro cúbicas de Bernstein e uma mistura bilinear, sem uma
transcendental e sem sequer um `sqrt` (HR-5).

⚠️ **O que faltava era o TETO DO UNIFORME, e ele foi MEDIDO.** Os 24 offsets ocupam `24 × 4 = 96 B`;
com o cabeçalho (`count`, `playhead`) e o bit de broadcast o total é **`108 B` dos `128`** de
`ph2d_gpu_cook::UNIFORM_BYTES`. Sobram `20 B` ⇒ **cinco params cabem, um sexto não**, e a cura
nesse dia é subir o slot (o doc dele já diz porquê: *«isto é um slot, não uma alocação por
elemento — a folga é grátis»*), nunca cortar um controlo que a referência tem.

⭐ **E o portão passou a IMPRIMIR esse número.** O `motion_gpu_kernel_budgets` do shell media os
dois orçamentos e só imprimia o das **storages**; agora imprime também o do uniforme, e o
`motion.bezier_warp` é hoje o mais largo do registry. *A folga de um slot que ninguém vê é
exactamente a que se descobre a estourar.*

#### As duas identidades, escritas como ESTRUTURA

O `eval` tem dois caminhos que devolvem o stream **verbatim**: os 24 offsets a zero (o nó
recém-largado) e a caixa degenerada. Os dois são um `write_P(i, bw_p)` **próprio** no corpo, e não
o caso `s == p` da mistura geral.

⚠️ **`p + (p − p) · f` NÃO é `p`.** Com `p` a `-0.0` a soma devolve `+0.0`, e o caso neutro é
precisamente o *default* do nó — o defeito nasceria no dia em que alguém largasse o nó e não fizesse
nada com ele. É a mesma lei que o cisalhamento da W3 pagou: *uma identidade que se quer ao bit
escreve-se num ramo separado, senão ela é um facto de aritmética e a aritmética muda.*

**Medido no adapter** (`the_bezier_warp_deformer_matches_the_cpu_within_epsilon`, 16 384
instâncias):

| caso | pior `|Δpos|` |
|---|---:|
| neutro (o nó recém-largado) | **`0e0`** — byte-idêntico |
| uma borda arqueada | `1,14e-5` |
| as quatro bordas | `1,53e-5` |
| meio warp | `1,34e-5` |

#### ⚠️ As tangentes dos terços são CALCULADAS, nunca escritas como literal

`1.0 − (1.0/3.0)` em `f32` dá **`0,66666663`** — o caso é um empate exacto e arredonda para par —
enquanto o literal `0.6666667` é **`0,66666669`**. **Bits diferentes.** Escrever a tabela do
quadrado unitário à mão daria um patch neutro que já não é a identidade, e o defeito apareceria
como um layout a tremer um ulp ao largar o nó. *A porta é a EXPRESSÃO, não a tabela que ela
produz.*

Três gates: `the_bezier_warp_reaches_the_device` (device-free — a metade que nenhum gate numérico
apanha, porque sem kernel o nó continua a cozinhar **certo** na CPU e só o `is_fully_gpu()` muda),
a paridade acima, e `the_curved_boundary_actually_moves_the_layout` (o controlo contra duas
identidades a concordarem sobre um nó que nunca correu).

---

### ✅ W4b — O CARTÃO, e a acusação que a MEDIÇÃO refutou (2026-09-08)

⛔⛔ **O §3 deste plano acusava o cartão do `motion.bezier_warp` de pintar `In X · In Y · Out X ·
Out Y` QUATRO vezes sem dizer de que aresta — e é FALSO.** O cartão dobra as 24 rows em cinco
`CardSection`, medidas:

```
motion.bezier_warp > secções: Corners@0 · Top Edge@8 · Right Edge@12 · Bottom Edge@16 · Left Edge@20
```

Um rótulo repetido debaixo de um cabeçalho que o nomeia **não é ambíguo**. ⚠️ **A acusação saiu da
sonda `what_the_card_shows`, que lia só os RÓTULOS** — metade da superfície que o cartão pinta.
*Uma sonda que lê metade da superfície fabrica dívida.* A sonda passa a ler as duas metades.

#### O que a sonda corrigida ACHOU, e esse é real

Os dois warps deformam a **mesma** caixa envolvente com os **mesmos oito nomes de param**
(`tl_dx` … `bl_dy`) e chamavam-lhes coisas diferentes:

| nó | como ele chama o canto de cima à esquerda |
|---|---|
| `motion.four_point_warp` | `TL X` · `TL Y` |
| `motion.bezier_warp` | `Top-Left X` · `Top-Left Y` |

É o achado **§2.3** deste ciclo — *seis vocabulários para «onde é o centro»* — repetido um nível
abaixo. Unificado na forma **LONGA**, e ⚠️ **a escolha foi MEDIDA, não preferida**: com o painel
lateral fora, o cartão é a única superfície onde estes nomes aparecem, e ele tem `190 px`.

⭐ Gate novo `no_warp_label_is_cut_on_the_card`: ele **pinta a row e conta os GLIFOS** —
`Bottom-Right X` cabe inteiro ao lado do valor mais largo da faixa (`-10,00`). Se não coubesse, a
unificação teria ido para o lado curto: *um rótulo elidido lê-se como o vizinho dele*, e
`Bottom-Rig…` é indistinguível de `Bottom-Lef…`. Censo
`every_node_that_offsets_a_corner_calls_it_the_same_thing`, com a população derivada do
**MANIFESTO** e não de uma lista de dois nós.

⚠️ **E a primeira redacção daquele gate media a contagem ABSOLUTA de glifos, e leu `23` contra
`20`:** um cartão pinta também o **título** e os **rótulos dos pinos**, e um oráculo que os ignora
acusa como «cortado» um rótulo que está inteiro — *a régua errava para o lado que fabrica dívida*.
Hoje mede-se por **diferença** contra o mesmo cartão sem rótulo, e tudo o resto cancela-se.

---

### 📊 A TABELA DEPOIS DA W5 — e a sonda que respondia DUAS perguntas com UM grafo

⛔⛔ **A tabela de preços media cada nó nos DEFAULTS dele, e o `motion.bezier_warp` denunciou-a
assim que chegou ao dispositivo:** ele leu **`0,34 ×` o custo da grelha sozinha** — *mais rápido do
que não estar lá*. Com os 24 offsets a zero o `eval` toma o atalho da identidade e devolve o stream
clonado: **a tabela dizia que o patch de Coons é barato e o que ela cronometrava era um `clone`.**

⚠️ **Não é um caso especial dele.** Metade deste grupo nasce na identidade — o `motion.move` em
`(0,0)`, o `rotate` a `0°`, os dois warps com os cantos parados. *Um corpus no ponto neutro de um
knob não testa esse knob*, e uma tabela de PREÇO medida no neutro mede o preço de não fazer nada.

⇒ o nó passa a ser **ACORDADO** antes de ser cronometrado (cada slider a ¼ da faixa), e os knobs
movidos são **DERIVADOS dos hints** — uma lista escrita à mão envelheceria a cada param novo.

#### ⚠️ E acordar quebrou a OUTRA coluna — o defeito foi meu, e a tabela desmentiu-me na corrida seguinte

Acordar põe `target_x`/`target_y` fora de zero, que é **exactamente** o que a cláusula `applicable`
do `motion.look_at` lê — e ele apareceu **🔴** na primeira corrida: *a MINHA perturbação, lida como
uma regressão do produto*. E o comentário que eu tinha escrito ao lado do despertar dizia que só um
`Enum` ou um `Toggle` alimenta um `applicable`. **Falso.**

⇒ as duas perguntas passam a ter **grafos SEPARADOS**: o relógio com o nó aceso, a coluna do
dispositivo nos **defaults** (que é o que o artista recebe ao largar o nó). E o desacordo entre elas
virou uma leitura própria — **🟡 = a residência DEPENDE de um knob**. *Não há subconjunto de knobs
seguro: a única forma de não perturbar uma medição é não a fazer no mesmo grafo.*

#### A tabela (`motion_deformadores_probe::measure_the_deformer_group`, grelha 320×320)

> ⛔ **ESTA TABELA FOI SUPERADA — ver «A TABELA NA MÁQUINA CALMA» no fim do documento.**
> Duas das linhas dela mediam o nó adormecido (o `rotate` a `0°` e o `spline_wrap` a
> devolver um `clone`), e a coluna do relógio correu a `load 21,52`.

⚠️⚠️ **A coluna do RELÓGIO não é citável nesta corrida: `load 21,52`**, quatro vezes acima da barra
do §5.0 (outra linha corria a suíte do shell na mesma máquina). Ela fica aqui porque a **ORDEM** de
grandeza e a razão entre nós já dizem o que a wave precisa; o número fino é da **W6**, e sai numa
máquina calma. A coluna `no device?` é `plan(..).is_fully_gpu()` — **determinística**, e não uma
leitura de relógio.

| nó | objectos | vs. só a grade | no device? |
|---|---:|---:|:---|
| `motion.move` | 102 400 | 1,33× | 🟢 |
| `motion.rotate` | 102 400 | 1,70× | 🟢 |
| `motion.scale` | 102 400 | 1,96× | 🟢 |
| `motion.transform` | 102 400 | 1,61× | 🟢 |
| `motion.mirror` | **204 800** | 2,41× | 🟢 |
| `motion.look_at` | 102 400 | 2,09× | 🟡 **só nos defaults** |
| `motion.bend` | 102 400 | 2,34× | 🟢 |
| `motion.twist` | 102 400 | 2,06× | 🟢 |
| `motion.spherize` | 102 400 | 1,97× | 🟢 |
| `motion.four_point_warp` | 102 400 | 1,97× | 🟢 |
| **`motion.bezier_warp`** | 102 400 | **3,84×** | 🟢 **(W5a)** |
| `motion.kaleidoscope` | **1 740 800** | 18,99× | 🟢 |
| `motion.spline_wrap` | 102 400 | 2,86× | 🔴 **NÃO** (W5b) |

⭐ **O grupo fecha em `12 de 13`** — os onze 🟢 mais o 🟡, que **chega** ao dispositivo com o nó como
ele nasce. O `motion.bezier_warp` foi de `0,34×` (o `clone`) para **`3,84×`**, que é o patch a
correr de facto.

⚠️ **E o 🟡 do `motion.look_at` não é um defeito novo:** ele é a troca que o comentário do próprio
kernel já declarava — *«um grafo que conduz o alvo por fio mantém a residência; digitar um ponto é
o que a custa»*. O que mudou foi a tabela passar a **dizê-lo**.

---

### ✅ W6 — O TUTORIAL, e a figura que teve de aprender a ver `rot` e `size` (2026-09-08)

📘 **[`tutoriais/03_transformes.pdf`](../../Motion%20Nodes/tutoriais/03_transformes.pdf)** — 9 páginas, com as 13
figuras e a tabela de controlos **derivadas do app** pela mesma função de onde o painel e o cartão
tiram os números. Fonte diffável em [`tutoriais/src/03_transformes.html`](../../Motion%20Nodes/tutoriais/src/03_transformes.html);
gerador `dump_deformadores_figures`.

#### ⛔⛔ A régua apanhou TRÊS figuras a `0,00` — e elas não estavam erradas: eu estava

A primeira redacção do gerador desenhava **nuvens de pontos**, como os dois ciclos anteriores. O
`motion.rotate`, o `motion.scale` e o `motion.look_at` leram **deslocamento zero**: eles não mexem
em `P` — escrevem `rot` e `size` —, e *uma nuvem de pontos é estruturalmente cega a isso*.

⚠️ **A figura não conseguia distinguir «o nó não fez nada» de «o nó fez uma coisa a que eu sou
cego»**, e num tutorial as duas leituras são o mesmo desastre: o artista veria a grelha intacta
debaixo do título «Rotate».

⇒ o elemento passa a ser uma **MARCA** — um traço curto com a direcção de `rot` e o comprimento de
`size` — e a régua mede a **PONTA** dela, que é o único ponto derivado que se move quando qualquer
uma das três colunas muda.

#### E o DESENHO passou a ser derivado, três vezes, cada uma por um defeito VISTO

| tentativa | o que saiu | a leitura |
|---|---|---|
| traço entrada → saída | um **NOVELO** no `motion.twist` | a `150°` os vizinhos vão para lados diferentes e as linhas cruzam-se todas |
| tirar o traço | **pior** — marcas horizontais dispersas | *a torção vive na relação entre VIZINHOS*, e nenhuma marca sozinha a contém |
| **a MALHA** | a grelha deformada, legível de relance | é a figura que toda referência usa para um warp |

⚠️ **E não é a distância que decide se um traço ajuda** — o `motion.move` desloca `4` vãos e
desenha-se perfeitamente, porque os traços dele são **paralelos**. O que torna um traço ilegível é
ele **cruzar o do vizinho**.

⚠️ **A marca só é desenhada onde ela FALA.** Nos dez nós que não escrevem `rot` nem `size` ela é
sempre o mesmo traço horizontal, e numa malha torcida dez marcas iguais são **ruído sobre a única
coisa que a figura tem para dizer**. A pergunta é feita à colheita, nunca autorada por figura:

```
def_rotate     │ malha + marcas      def_twist    │ malha
def_scale      │ malha + marcas      def_mirror   │ marcas soltas
def_look_at    │ malha + marcas      def_kaleidoscope │ marcas soltas
```

⛔ **Não há malha onde a contagem muda** — o `motion.mirror` devolve `2n` e o
`motion.kaleidoscope` `n × segmentos`; ali o `i + 1` já não é o vizinho da direita de nada.

⚠️ **A moldura é QUADRADA neste ciclo e não no anterior.** Lá as figuras são filas — largas e
baixas — e um quadrado desperdiçaria 70 % de cada uma; aqui são folhas, e o que se ganha é o
**ALINHAMENTO**: treze molduras de proporções diferentes deixam as legendas em degraus.

#### O corte de LOC, e ele é por RESPONSABILIDADE

A paleta e a moldura saíram para `motion_bridge_tutorial_draw.rs`, com dois consumidores.
⚠️ **O que se partilha entre ciclos é a PALETA e o ENQUADRAMENTO, não o desenho** — duas paletas
seriam dois tutoriais com duas caras, e *isso* é que seria a mesma pergunta respondida duas vezes.

#### O tutorial só nomeia rows e gestos MEDIDOS

Cada linha de painel que um passo manda clicar foi lida em `what_the_card_shows` (o `Pivot` do
Twist, o `Skew X` do Transform, o `Axis Offset` do Mirror, o `Rows` do Grid).

⛔⛔ **E um passo descrevia um gesto que NÃO EXISTE.** O «anime o pivô» mandava arrastar um fio
para cima da row `Pivot X`; o gesto real é largar o fio no **CORPO** do cartão e escolher o param
na lista que abre (`GraphIntent::DriveParam` — *«o socket aparece porque o fio existe, não o
contrário»*). *A lei de que um passo que nomeia uma row tem de provar que a row está na lista vale
igual para um passo que nomeia um GESTO.*

---

### ✅ O TERCEIRO REPORT — o `motion.spline_wrap` nasce INERTE (08/09)

> *«Em vez de nascer com uma curva default com pontos no painel, melhor nascer inerte com um botão
> para selecionar um path no canvas ou na hierarchy. Até que o path esteja selecionado, um sinal de
> alerta fica visível no nó.»*

⚠️ **É a SEGUNDA METADE de um pedido de 2026-08-12** que está citado **verbatim** no fonte deste nó
— *«pontos e alças em sliders num painel. Absurdo! … um botão no painel do nó para o usuário
desenhar sua curva no canvas»*. A primeira metade (a row `Shape`) shipou nesse dia; a cúbica ficou
a ser o estado de **nascimento**.

O cartão passa de `Shape · Follow Curve · … · P0 X · P0 Y · … · P3 Y` para:

```
Shape · Use Selected Path · Follow Curve · … · Size Profile
  > secções: Curve@12 (FECHADA)
```

#### O que foi feito, em três peças

| peça | mecanismo |
|---|---|
| **nasce sem curva** | os oito pontos nascem na origem, e o `eval` ganha um ramo que devolve a folha **verbatim** |
| **o ⚠** | `Deficit::MissingChoice` no diagnoser, por um canal novo: `register_required_text_params` — o **irmão de TEXTO** do `register_required_inputs` |
| **o botão** | `ParamWidget::PickSelection` → `ClickDoes` → `GraphIntent` → a shell, que é quem tem o mundo |

⛔⛔ **O ramo inerte é obrigatório, não uma optimização.** Com uma cúbica de comprimento zero o
`frame_at` devolve a origem e a tangente `(0,0)` para todo elemento: a folha **COLAPSA num ponto**.
*Um default inerte que destrói o layout é pior que o default que ele substitui.*

⚠️ **Os oito pontos FICAM, e a razão é medida:** as **quatro** cenas da conferência que usam este
nó escrevem os oito valores à mão, então mudar o default não lhes custa nada e apagá-los custaria
as quatro. Eles saem do **nascimento**, não da existência — a secção `Curve` passa a nascer
**fechada** (`ParamGroup::folded`, que já existia), que é a explicação que o censo
`every_param_the_card_hides_has_a_declared_reason` aceita e que os deixa a **um clique**.

#### ⛔⛔ Três defeitos MEUS que os portões apanharam antes do dono

1. **O censo do `Deficit::ALL`**: sem braço próprio no `explain`, o variante novo caía no
   catch-all e o artista recebia *«este nó produz dados que ninguém consome»* — uma frase sobre
   outro defeito. *Era exactamente para isso que aquela lista existe.*
2. **O portão de fecho**: a primeira regra só olhava o texto, e acusava de inertes as quatro cenas
   da conferência — que embrulham com a curva escrita à mão. *Um ⚠ sobre um nó que funciona ensina
   o artista a ignorar o ⚠.* ⇒ o `RequiredTextParam` ganhou um `only_when` (a forma do
   `GpuKernel::applicable`) que faz a **mesma pergunta que o `eval`**: há curva?
3. **A régua das figuras do tutorial**: a figura do `def_spline_wrap` saiu a **`0,00`** de
   deslocamento na primeira corrida depois da mudança, e o gerador recusou-se a escrevê-la. *Uma
   fixtura que herda um default mede o default, e um default que muda leva a figura com ele.*

#### ⛔⛔ E havia um `_ => ClickDoes::Nothing` no mapa widget → gesto

Com ele, o `PickSelection` atravessou o `cargo check --workspace --all-targets` **inteiro sem um
aviso** e o botão teria nascido **morto sob o dedo** — a espécie do §5.0 do `CLAUDE.md` (*«um
handler cujo braço não cobre a variante»*), que sobrevive a todo gate de registo. O catch-all
morreu; esquecer passou a ser erro de compilação.

⭐⭐ **E o nome da forma seleccionada sai do MESMO passe que publica as formas** — não de uma
segunda varredura. O filtro que decide o que o grafo consegue ver (tem nome · não é reservado · tem
arco) é o mesmo que decide esta, então o botão nunca pode ligar o nó a algo que o grafo não vê.

---

### ✅ A COSTURA GANHA PORTÃO — o que faltava à família (2026-09-08)

O defeito de cima viveu **entre** dois gates verdes: a porta funcionava
(`cooking_the_taps_fills_them_without_marching_the_clock`, em `ph2d-eval-motion`) e a tinta
aparecia (`the_gizmo_paints_geometry_and_paints_none_when_inactive`, ao lado do overlay).
*Nenhum dos dois perguntava se a rota que o app de facto corre chega a chamar a porta.*

**`the_fully_gpu_route_cooks_the_taps_it_never_marched`**
([`motion_bridge_gpu_taps_tests.rs`](../../shells/desktop/src/render_loop/motion_bridge_gpu_taps_tests.rs))
monta a **cena do report** (a `=111` com o `motion.bezier_warp` enfiado dos dois lados do espelho),
entrega o quadro a um adapter **de verdade** pela porta do produto (`cook_gpu`), e afirma quatro
coisas em cadeia: a fixtura arma tomada · a ponte devolve `Handled` · a rota que ela **diz** ter
tomado é a totalmente-na-GPU · a tomada armada está cozida e o `resolve` devolve `Some`.

⚠️ **A rota lê-se do produto (`MotionState::route_said`), nunca replaneada no teste** — um segundo
cálculo seria uma segunda opinião sobre que rota este documento toma, e o gate deixaria de medir a
do app. É a mesma cerca que fez a sonda mentir, virada do avesso.

⭐ **A fixtura teve de ser PARTIDA para o gate existir.** A montagem da cena e a **marcha na CPU**
viviam na mesma função da sonda — e é a marcha que cozinha as tomadas de passagem, logo qualquer
fixtura que marche **esconde exactamente este defeito**. Hoje
[`warp_gizmo_fixtures`](../../shells/desktop/src/render_loop/warp_gizmo_fixtures.rs) monta e arma
**sem marchar**, e quem chama escolhe a rota: a sonda marcha na CPU, o portão entrega ao device.
⚠️ E a selecção do grafo é **global**, então as fixturas partilham uma trava — duas cenas em
paralelo trocavam de nó seleccionado uma à outra.

⛔ **Prova de mutação:** apagar a chamada a `cook_taps_only` na rota `FullyGpu` devolve
`a tomada NodeId(2) foi armada e NAO foi cozida — []` — o report do dono à letra.

⚠️ **`#[ignore]`, e a razão é o adapter** (§5.0: *skip gracioso não é verde*): sem GPU não há rota
`FullyGpu` para medir, então o gate **estoura** em vez de saltar em silêncio — um verde sem device
afirmaria sobre um programa que não é este. Corrido nesta máquina: **2 de 2 verde**.

⚠️ **O CONTROLO é a outra metade do preço:** `the_fully_gpu_route_cooks_nothing_when_no_tap_is_armed`
prova que sem tomada armada nada é cozido. Sem ele, uma implementação que cozinhasse tomadas
incondicionalmente passaria o gate de cima e a frase escrita no doc da porta — *«só se paga enquanto
há tomada armada»* — deixaria de ser verdade sem ninguém ver.

---

### ✅ W7 — A DIREÇÃO DA DOBRA CHEGA AO DISPOSITIVO, e o que a segurava era o SUBSTRATO (2026-09-08)

> A secção acima reconferiu a recusa e nomeou o que a dissolveria. **Foi construído.**

O `motion.bend` era o último nó do grupo com uma recusa **condicional a um knob**: com
`direction ≠ 0` o `applicable` devolvia `false` e a cadeia inteira caía para a CPU — uma queda que
o [doc 98](../../Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md) mede em **`50,9×`**.

#### O canal PARTILHADO (`wgsl_shared`)

O módulo de uma redução era montado com os params, as ligações, os `reduce_<earlier>()`, o
`reduce_value` e o `main` — **e nada mais**. Uma [`ReduceSpec::value`] que precise de uma função
auxiliar não tinha onde a pôr.

⇒ `KernelResolver::wgsl_shared(ty) -> &'static str`, **append-only** (o default é `""`, e quem não
o declara gera exactamente os módulos de sempre byte a byte), colado pelo gerador **nos dois**
módulos — o do kernel e o de cada redução. ⛔ Colar o `wgsl_lib` inteiro na redução **não** serve:
ele chama `read_in_*`/`write_*`, que ali não existem — e é por isso que o canal é separado.

⚠️⚠️ **E o WGSL partilhado entra na CHAVE DE CACHE da redução, senão há colisão REAL:** a
`pivot::CENTROID_CX` é *literalmente a mesma* `ReduceSpec` em vários nós, e o primeiro a declarar
um canal emprestaria o módulo dele a todos os outros — um pipeline com funções que a expressão do
vizinho não pede, ou pior, sem as que ela pede.

#### O nó

As duas reduções passam a dobrar **`bd_axis(v, direction)`** — a projecção no eixo da dobra —, e o
kernel compara-as contra `bd_axis(pivô)`. ⭐ **`max|proj(v) − proj(pivô)|` é atingido num extremo
porque `proj` é LINEAR**, que é a mesma identidade que já justificava o `max(xmax − p, p − xmin)`.

⚠️ **Em `direction == 0` tudo é o de sempre ao bit**, por **ramo literal** nos dois lados (o
`bd_axis` do device e o `local` da CPU) — `bd_dir_cos(0)` é `1,0` e `bd_dir_sin(0)` é `0,0` ao bit,
**conferido e não assumido**, mas `x·1 + y·0` não é `x` para `−0,0` nem para `±inf`.

#### Medido

| caso | pior `|Δpos|` |
|---|---:|
| `direction = 0` (o caminho de omissão) | `6,68e-6` |
| `direction = 37°` | `1,24e-5` |
| `direction = −90°` | `1,24e-5` |
| `direction = 180°` | `4,77e-6` |

16 384 instâncias por caso, dentro da `EPS_POS` da família (o `bend` sem direcção já media
`1,14e-5`). ⚠️ **O ε fora do zero tem razão nomeada:** a CPU projecta `(p − pivô)` e o dispositivo
faz `proj(p) − proj(pivô)` — a mesma conta noutra ordem, a classe de ε do raio do `motion.twist`.

#### Gates

- `the_bend_direction_reaches_the_device` — a cadeia é reivindicada inteira em `0`, `37`, `−90`,
  `180` (sem device: é um facto do **planeador**).
- `the_rotated_bend_matches_the_cpu_within_epsilon` — a paridade acima (device).
- `the_bend_direction_actually_turns_the_axis` — ⚠️ **o controlo**: rodar `37°` tem de mover o
  layout, senão o gate de paridade compararia duas vezes a mesma dobra.
- ⛔ **E o gate que DEFENDIA a recusa foi invertido, não apagado**
  (`the_knob_is_painted_and_the_device_no_longer_steps_back_for_it`): ele afirmava *«com direção o
  device sai»*, e afirma agora que **não há `applicable` nenhum** — *um que volte a nascer ali tem
  de trazer o próprio motivo escrito*.

---

### 📊 A TABELA NA MÁQUINA CALMA — o número fino que a W6 devia (2026-09-08)

**`load 3,98`** (a barra do §5.0 é `≤ 5`), grelha `320×320`, `--release`, **três corridas** do
mesmo binário. A coluna que se cita é a **absoluta**; a razão vai ao lado e traz o ruído do
denominador consigo (ver a nota abaixo dela).

| nó | objectos | cozimento (mediana de 3 corridas) | vs. só a grade | no device? |
|---|---:|---:|---:|:---|
| *(só a grade)* | 102 400 | `0,56 ms` | — | 🟢 |
| `motion.rotate` | 102 400 | `0,80 ms` | 1,43× | 🟢 |
| `motion.move` | 102 400 | `0,84 ms` | 1,50× | 🟢 |
| `motion.transform` | 102 400 | `0,87 ms` | 1,55× | 🟢 |
| `motion.spherize` | 102 400 | `0,88 ms` | 1,57× | 🟢 |
| `motion.look_at` | 102 400 | `0,93 ms` | 1,66× | 🟡 **só nos defaults** |
| `motion.four_point_warp` | 102 400 | `0,97 ms` | 1,73× | 🟢 |
| `motion.scale` | 102 400 | `0,98 ms` | 1,75× | 🟢 |
| `motion.mirror` | **204 800** | `1,00 ms` | 1,79× | 🟢 |
| **`motion.bend`** | 102 400 | `1,12 ms` | 2,00× | 🟢 **nas DUAS colunas (W7)** |
| `motion.twist` | 102 400 | `1,16 ms` | 2,07× | 🟢 |
| `motion.bezier_warp` | 102 400 | `2,00 ms` | 3,57× | 🟢 |
| **`motion.spline_wrap`** | 102 400 | `2,53 ms` | 4,52× | 🔴 **NÃO** (W5b) |
| `motion.kaleidoscope` | **1 740 800** | `9,38 ms` | 16,75× | 🟢 |

Um quadro de 60 fps tem `16,67 ms`. ⭐ **O grupo inteiro cabe num quadro com folga** — o mais caro
é o caleidoscópio, e ele custa `56 %` de um quadro para produzir **1,74 milhões** de objectos.

#### ⚠️ O que MUDOU contra a tabela antiga, e o que não mudou

- **`motion.spline_wrap`: `2,86×` → `4,52×`.** A leitura antiga cronometrava um **`clone`** — os
  quatro pontos de controlo colapsados num ponto, curva de comprimento zero, atalho inerte. Este é
  o preço do embrulho a sério, e é ele que sustenta a recusa da W5b.
- **`motion.bend`: 🟢 nas DUAS colunas.** Antes ele lia 🟢 porque o `direction` estava adormecido;
  hoje o despertar acende-o **e o nó continua no dispositivo** — que é a W7.
- ⭐ **`motion.rotate` mal se mexeu (`1,70×` → `1,43×`), e isso é uma resposta, não ruído:** ele
  escreve `rot`, um escalar por elemento, sem redução e sem tocar em `P` — *para este nó a
  identidade custa o mesmo que o trabalho*. Era o único dos dois adormecidos cuja linha antiga não
  estava a medir a coisa errada.

#### ⛔⛔ E a coluna da RAZÃO carrega o ruído do denominador — medido, e a «cura» foi revertida

A base (`só a grade`) leu `0,56` · `0,81` · `0,52 ms` nas três corridas — **±36 %** — enquanto os
absolutos dos nós repetiram dentro de `~15 %`. Ela é o menor número da tabela e é o denominador
das treze linhas, então o ruído dela propaga-se a todas: o `motion.kaleidoscope` leu `16,63×`,
`13,46×` e `17,92×` **sobre tempos absolutos de `9,25`, `10,97` e `9,38 ms`**.

⛔ **Dar-lhe mais amostras foi construído, MEDIDO e REVERTIDO no mesmo dia.** Com `9` repetições a
base leu **`0,26 ms`** — metade do que lê com `3`, e fora da dispersão das três corridas. ⭐ *O
«cozimento frio» não é frio: o processo AQUECE ao longo das repetições* (alocador, caches,
preditor), então a mediana de nove assenta abaixo da mediana de três — e a razão passou a comparar
uma base bem aquecida contra um nó frio, com o caleidoscópio a saltar para **`39,02×`**.

> ⭐ **A lei que fica: o número de repetições faz PARTE da medição, então os dois lados de uma
> razão têm de o partilhar.** E a coluna que se cita é a absoluta.

⚠️ **A corrida 2 foi sistematicamente mais lenta em TODAS as linhas** (`+15 %`), o que é a máquina
e não um nó — é por isso que a tabela publica a **mediana de três corridas** e não uma delas.

---

### ⚠️ O TUTORIAL JÁ AFIRMAVA O QUE A W7 SÓ HOJE TORNOU VERDADE (2026-09-08)

A tabela *«Onde isto roda»* do [tutorial](../../Motion%20Nodes/tutoriais/src/03_transformes.html) lista o **Bend** entre
os que correm na placa **«sim, sempre»** — e isso era **falso no dia em que foi escrito**: bastava
o artista tocar no `Direction` para a cadeia inteira cair para o processador, em silêncio. A W7
tornou a frase honesta, e a tabela na máquina calma confirma-o (`motion.bend` 🟢 nas duas colunas,
com o knob aceso).

⛔ **A leitura não é «tivemos sorte», é um buraco nomeado:** *nada liga aquela tabela ao
planeador*. Ela foi escrita à mão a partir da coluna `no device?` de uma corrida da sonda, e uma
recusa nova — ou uma que caia, como esta — **não a acorda**. É a família da *cena de smoke que
ensina o contrário do que acontece* (§5.0), a mais cara de todas, porque é o primeiro sítio onde o
dono olha.

⭐ **A cura tem forma e não foi construída:** aquela tabela é **derivável** —
`plan(..).is_fully_gpu()` sobre cada nó do grupo, nos defaults e aceso, que é exactamente o que a
sonda já imprime. O tutorial já gera a tabela de params por ficheiro
(`fig/params_deformadores.html`); esta é a **segunda** que devia nascer do código em vez de ser
copiada dele.
