# ARQUIVO — HANDOFF_INTEGRACAO_line_motion_value_BLOCO_Z_2026-08-23.md (história, 309 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`HANDOFF_INTEGRACAO_line_motion_value_BLOCO_Z_2026-08-23.md`](../../../Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_BLOCO_Z_2026-08-23.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-47,223-269,404-1040` do original.
>
> ⚠️ **A única alteração ao corpo:** 4 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

## §0-ter — O smoke devolveu um DEFEITO, e ele é do renderer, não do nó

**Enio, 2026-08-23, sobre a `=84`:** *"shadow multiply parece não obedecer o alpha da cor"*.

⚠️ **Este é o item de MAIOR alcance do handoff inteiro** — ele muda como **toda sprite do
app** com `BlendMode::Multiply` compõe em alfa parcial (`ph2d-render`, caminho partilhado).
Registo completo, com hipóteses e lições:
[`BUGS_motion_nodes.md` Bug #4](../../../Motion%20Nodes/BUGS_motion_nodes.md).

**O nó estava inocente.** O `fx.drop_shadow` escreve a alfa do fantasma correctamente. O
defeito era o par de fatores do `Multiply` em
[`ph2d_render::pipeline::blend_state_for`](../../../../crates/ph2d-render/src/pipeline.rs).

**Medido antes de tocar** (fundo 55, frente 128, byte do centro):

| modo | α=0,00 | α=0,25 | α=0,50 | α=0,75 | α=1,00 |
|---|---|---|---|---|---|
| `Add` · `Subtract` · `Screen` · `Mix` | **55** | … | … | … | … |
| **`Multiply` (antes)** | **0** | 3 | 6 | 9 | 12 |
| **`Multiply` (depois)** | **55** | 44 | 34 | 23 | 12 |

Não era *"não obedece"*: era **invertido**. `α = 0` pintava **preto**, subir a alfa
**clareava**, e não havia valor em que a sombra sumisse.

**Mecanismo.** O `sprite.wgsl` emite `vec4(rgb·α, α)` — fonte **pré-multiplicada**, que
codifica *"não contribui"* como **zero**. Isso dá a resposta à alfa **de graça** a todo modo
cujo elemento neutro é `0` (`Add`, `Subtract`, `Screen`, o `over`). O neutro do `Multiply` é
**`1`**: com `dst_factor: Zero` a pré-multiplicação levava o produto para preto em vez de
para nada. Cura: `src: Dst`, `dst: OneMinusSrcAlpha` ⇒ `dst·(α·src + 1 − α)`.

⚠️ **As duas colunas coincidem em `α = 1`** — é isso que garante que nada opaco mudou, e é
exactamente o ponto que o gate antigo media.

**Para quem integra:**

1. ⚠️ **Um golden/regressão de imagem de OUTRA linha que contenha uma sprite `Multiply` com
   alfa parcial vai mudar de valor, e a mudança é a CURA.** Nenhum caso opaco se move.
2. O gate `blend_modes_composite_as_advertised` **fica como está e continua verde** — ele
   não estava errado, estava incompleto.
3. Nada de contrato congelado foi tocado: `BLEND_PIPELINE_COUNT` continua `6`, a assinatura
   de `blend_state_for` é a mesma, e `ph2d_ecs::BlendMode` não se moveu. Só o par de
   fatores de **uma** tag mudou.

**Gates novos** ([`blend_mode_regression.rs`](../../../../crates/ph2d-render/tests/it/blend_mode_regression.rs)):

| gate | o que prende |
|---|---|
| `zero_alpha_is_absence_in_every_mode` | `α = 0` devolve o fundo **medido no mesmo passe** (`fg = None`), nos seis modos, com controle positivo |
| `the_multiply_alpha_slider_runs_from_the_backdrop_to_the_full_product` | monotonia do curso + excursão real |
| `measure_alpha_response_of_every_mode` | a sonda que imprime a tabela (não afirma nada) |
| `the_alpha_row_varies_the_alpha_and_nothing_else` (shell) | a linha 3 da `=84` varia **só** a alfa |

**Prova de mutação.** Os dois primeiros foram vistos **VERMELHOS sobre o defeito real**,
antes da cura — a espécie mais forte, porque a mutação não foi sintética. O gate da cena foi
mutado duas vezes (alfas iguais ⇒ `0.85 vs 0.85`; modos diferentes ⇒ `so' a alfa muda`),
vermelho nas duas, restaurado por `git checkout` sobre commit limpo.

**E a fronteira fica registada** no `keep_dst_alpha`: um par de fatores fixos **não exprime**
o `Cs' = (1−αb)·Cs + αb·B` da W3C (precisa da alfa do DESTINO como termo). A fórmula inteira
já existe e está **correcta** onde o fundo é translúcido de propósito
([`layer_composite.wgsl`](../../../../crates/ph2d-render/src/shaders/layer_composite.wgsl)).
⛔ *Quem tentar "consertar" a divergência de faixa parcial com outro par de fatores não vai
conseguir — o caminho é o passe programável.*

**A cena `=84` ganhou a terceira linha (ALFA)**, porque o defeito só é julgável a olho num
PAR: as duas metades MULTIPLICAM e só a alfa muda (15% × 85%). Uma metade só diria *"está
escuro"*.

---

## §0-quater — QUARTO BLOCO: **a folha 06 (animadores)**

Uma pergunta só, e cinco células: *o animador não sabia a FORMA que o artista desenha,
nem escrever os DOIS eixos.* A folha 06 vai de **11 P2 para 7** (e o placar de 76 para
**72**); ✅ 228 → **232**.

| célula | cura | onde |
|---|---|---|
| onda `Custom` | `motion.oscillator::wave = 5` + text param `curve` | CPU + **device (LUT)** |
| ease `Custom` | `motion.stagger::ease_curve = 8` + text param `curve` | CPU + **device (LUT)** |
| `Separate Channels` | `motion.wiggle::channel = 4` — **FECHA a célula** | CPU + device |
| `Separate Channels` | `motion.noise::channel = 4` — metade (falta *Use Layer as Seed*) | CPU + device |
| alinhar pela NORMAL | `motion.path::align = 2` | CPU |

**Cena de smoke: `=85`** · **`PH2D_MOTION_NODE_PATH_SMOKE=3`** (a normal precisa da curva
desenhada, que só o smoke próprio do `motion.path` encena).

### O que o integrador tem de saber

1. ⚠️ **Nada é breaking.** Os três enums são **apendados** (`wave` 0..4 · `ease_curve`
   0..7 · `channel` 0..3 continuam a valer o que valiam) e o `align` era um `Toggle` cujos
   `0`/`1` continuam a ser *nada*/*tangente* — o `align >= 0.5` de ontem e o `!= 0` de hoje
   concordam em todo documento que existe.
2. ⚠️ **`ph2d-curve` é dependência NOVA de `ph2d-node-motion-oscillator` e
   `-stagger`**, e `ph2d-gpu-cook` ganhou-a como dev-dep. O `Cargo.lock` mexeu.
3. ⚠️ **O `apply_channel_delta` continua BYTE-IDÊNTICO nas cinco crates-folha.** O canal
   novo não passa por ele: quem despacha é o `eval`, para uma `apply_channel_delta_xy`
   irmã. *Uma cópia deliberada que só duas das cinco recebem deixa de ser uma cópia* — e
   as três cópias do WGSL de um destes nós **já divergiram uma vez**, com um grafo a
   correr uma taxa no device e outra na CPU.
4. ⚠️ **Dois números vivem escritos DUAS vezes** (const de Rust + literal na string WGSL):
   `AXIS_SEED_OFFSET = 7919` e `AXIS_ROW_OFFSET = 7919,5`. Quem os prende são os gates
   `the_wgsl_carries_the_same_axis_offset_as_the_rust` (que derivam a agulha da const) **e**
   a paridade no device — nunca a memória de quem edita.

### As decisões que custaram medição

**A curva vai ao DEVICE, não derruba o nó para a CPU.** A saída fácil era o kernel
declarar-se `applicable: false` na forma `Custom`; está **rejeitada** — o `field.remap` já
mostra que um `LutSpec` resolve sem tirar o nó do caminho rápido, e um animador é
precisamente o nó que não se quer perder dali. ⚠️ **E o preço foi medido, porque a LUT é
assada em TODO cozimento** (o `build_luts` não pergunta o modo): **0,0029 ms por cozimento
por nó = 0,018% de um quadro**. Fica sem predicado. Sonda `measure_lut_build_cost_per_cook`.

**A resolução da LUT é 512, o dobro do `field.remap`.** Uma tabela uniforme converge com
`1/n` numa esquina e não converge de todo num degrau — o que a densidade encolhe é a
LARGURA da banda errada. Aqui a tabela é lida como MOVIMENTO ao longo de um ciclo (não como
a cor de um pixel), então essa banda vira um solavanco visível.

**A `natural_range` teve de responder pela forma nova.** A `Custom` é unipolar `[0,1]` (o
quadrado do editor) e as quatro clássicas são bipolares: sem declarar a polaridade, a conta
que toda a gente faz de cabeça entregaria `Min/Max = [−2,3]` como `[0,5, 3,0]` — metade da
excursão com o piso ao CENTRO, em silêncio. É a armadilha do `Spike` que esta folha já
pagou, reaberta por uma forma nova.

**A ease `Custom` ignora o `ease_dir`**, como o `Linear`, e por razão de PRODUTO: a curva
desenhada é a declaração de intenção do artista. ⭐ **E isso não custou uma linha de UI** —
o `PARAM_GATES` enumera as famílias que *usam* a direção (`1..7`), então a nona nasce com o
`ease_dir` escondido: *o gate estava escrito na forma certa antes de existir o nono caso.*

**A célula do `motion.path` dizia «NÃO» sobre um facto que o código do lado já tinha.** A
justificativa era *"a normal, que nada publica"* — e o `motion.spline_wrap` computa
`un = [-ut.y, ut.x]` da MESMA curva desenhada, com o doc deste nó a chamar-lhe *"o segundo
consumidor"*. **A nona célula desta folha a envelhecer assim.**

### ⚠️ Duas RÉGUAS corrigiram-se antes do algoritmo, e as duas por RESOLUÇÃO

1. **A barra da decorrelação estava colada em ZERO.** Primeira versão: `0,25`, e o device
   mediu `−0,222` — 12% de folga. A sonda `probe_r_vs_scale` disse porquê: o resíduo cai
   de `0,120` para `0,009` só ao AFINAR o campo. Um acoplamento real não se dissolve assim;
   **ruído de amostragem sim** — um campo COERENTE tem tantas amostras independentes
   quantas FEIÇÕES, não quantos elementos. ⇒ a barra passou a medir a distância a **`1`**
   (o valor exacto do defeito), não a proximidade de `0`; uma barra em `0,15` mediria o
   tamanho da grelha e reprovaria numa fixture legítima mais grosseira.
2. **A régua da cena exigia mais precisão do que a fixture tem.** Ela media a POSIÇÃO DO
   PICO com margem `0,1`; com 15 peças o índice anda de `1/14 = 0,071`, e os dois picos
   ficam a **um passo** um do outro. A régua nova é a que o olho usa (a curva autorada é
   unipolar e fica toda ACIMA da linha de repouso; a senoide atravessa-a), com a assimetria
   como segunda metade e margem de **meio passo**.

### Prova de mutação (as cinco células)

| mutação | o que morre |
|---|---|
| a `Custom` deixa de declarar a polaridade | o piso vira **`0,5`** — o número exacto que o gate nomeia como a conta errada |
| a `Custom` do stagger cai no caminho da direção | `the_custom_ease_is_the_authored_shape` |
| a onda `Custom` ignora a curva autorada | `a onda desenhada É a curva autorada` |
| o stagger nunca lê o text param | `an_unset_custom_ease…` + a forma |
| os dois eixos com o MESMO seed | `r = 0,9999999` — o defeito, exacto |
| o deslocamento de linha deixa de ser fracionário | **dois** gates (a propriedade **e** o gémeo do WGSL) |
| o modo `Normal` cai no ramo da tangente | `diferenca 0` onde tem de ser 90° |

### ⚠️ E um erro meu que REPETIU

Escrevi a cena nova em `motion_state_conferencia_demos_shape.rs` — e aquele nome já era a
cena **`=55`** (Pulse Width / Offset), **da mesma folha 06**. Dois arquivos por cima, o
`Write` a responder *"updated"* nos dois. É exactamente o que a memória escrita **ontem**
manda conferir, e o gatilho é o mesmo: *quanto melhor o nome, maior a chance de ele estar
ocupado*. O compilador acusou (`defined multiple times`), restaurei por `git checkout --`
com o meu copiado ao lado, e o módulo passou a chamar-se `drawn`. ⇒ **a memória ganhou o
passo que faltava**: um `ls` do diretório é GATILHO de todo `Write` num caminho não lido,
não o item 1 de uma lista — *uma lista não corre*.

---

## §0-sexies — SÉTIMO BLOCO: **o corpo deixa de ser um retângulo** (folha 03, o P1)

A folha 03 (simulação) **fecha o P1 dela e vai a zero**. Placar: **72 P2 + 3 P1 → 71 P2 +
2 P1**; ✅ 233 → **235**. Os dois P1 que sobram estão nas folhas **01** (`motion.emitter`
*inherit velocity* — **decisão de produto do Enio**, com as duas saídas já medidas) e
**07** (`motion.trail` eco para a FRENTE — estrutural).

### A célula, e por que a cura não é "uma porta"

> *forma inicial ≠ retângulo `rows×cols` (Cavalry põe soft body em QUALQUER shape;
> Vellum em qualquer geo)*

A porta `shape` é o pedaço fácil. O trabalho é que `rows`/`cols` respondiam a **três
perguntas que nunca foram sobre a grelha**:

| a pergunta | era | é |
|---|---|---|
| quem é **pino** | `i < cols` | a **aresta de cima** do repouso |
| qual é o **contorno** que a pressão defende | o passeio do anel | o **casco** do repouso |
| como o corpo se **divide em regiões** | bandas de **índice** | bandas de **coordenada** |

A camada nova é `crates/ph2d-node-motion-soft-body/src/layout.rs` (`BodyLayout`). O
`shape.rs` passou a receber o **anel** por argumento (`boundary_area(pos, rows, cols)` →
`ring_area(pos, ring)`) e o `cluster.rs` a receber as **regiões** já como listas de
índice. **A LEI fica partilhada; o FORNECEDOR é que difere.**

### ⚠️ A malha autorada continua a dar os MESMOS BITS

E não por promessa: ela é o seu **próprio fornecedor** das três respostas, e cada uma
devolve a **sequência de índices** que o código percorria à mão. O anel e as regiões
alimentam somas em `f32` — os mesmos elementos noutra ordem dariam outro número, e
moveriam arte já autorada por ruído de arredondamento. Três gates comparam contra o laço
antigo **copiado como oráculo** (`walk_as_it_shipped`, `nested_as_it_shipped`), e os 45
gates que já existiam passam **sem uma edição**.

⭐ **E o casco com os COLINEARES MANTIDOS reproduz o anel da grelha índice a índice** — é
isso que faz *entregar a malha pela porta* devolver o corpo autorado e não um primo dele.
Um casco estrito daria quatro cantos, cuja área é a mesma em aritmética exacta e **outra**
em `f32`.

⚠️ **Não ao bit, e a razão é a que a wave do `falloff` já medira:** quem entra pela porta
**tem** de ser re-centrado (a pressão escala os goals sobre o centro do quadro, e só pode
tratar isso como a mesma operação com o repouso na origem), e o centroide **somado** de
uma malha já centrada não é zero — `−1,19e-7` numa 8×8. Medido: **2 ULP**. Um
`assert_eq!` de bits mediria a representação do centroide em vez da costura.

### ⭐⭐ O smoke achou um defeito de PRODUTO, e a lei mudou

Com o pino a valer *o `y` máximo a menos de um epsilon*, uma malha é pregada pela linha
de cima e um **DISCO é pregado pelo seu ponto mais alto, UM SÓ**. Ele balança como um
pêndulo e a envergadura cresce **1,74×** em dois segundos.

A lei que fica é **meia FILEIRA**, e o número é **derivado**: numa malha a fileira
seguinte está a um `spacing` inteiro ⇒ a fatia é exactamente `0..cols`; numa nuvem a
altura de uma fileira sai da grelha equivalente (`Bands::row_height`).

⚠️ **A fixtura que a lei antiga usava não podia falsificar a nova** — a grelha é o caso
em que as duas concordam por construção. Memória:
`feedback_generalising_an_index_law_needs_a_derived_thickness_not_an_epsilon`.

### ⚠️ DOIS defeitos meus, e a RÉGUA errou antes do algoritmo nos dois

1. **A envergadura medida do ponto de MUNDO onde a cena pousa o corpo.** Um corpo
   pendurado balança e cai: a malha de controle lia-se **2,94×** perfeitamente inteira.
   *Forma é propriedade interna* — ancore no centroide.
2. **A extensão conta INTERVALOS e a contagem conta PONTOS.** Uma malha `16×8` mede
   `15s × 7s`, razão **2,143** e não 2; derivar a grelha equivalente da razão crua dava
   `17×7`, que o `counts` cortava a **metade** das regiões, em silêncio. A conta certa é
   a raiz positiva de `r·lb² + (1−r)·lb − n = 0`.

Memória: `feedback_a_ruler_anchored_in_the_world_measures_the_gesture_not_the_shape`.

### ⛔ Fronteira NOMEADA (não é buraco, é decisão medida)

O casco é o contorno de um conjunto de pontos **sem ligações** — a resposta canónica, não
a perfeita. Uma forma de repouso **côncava** (uma lua, o anel da cena `=87`) tem o
**envelope** defendido em vez da área. A pressão fica **mais fraca, nunca invertida**, e o
sinal continua a ser o do repouso, então um corpo virado do avesso continua a ser
detectado. Está no doc-comment do `hull_ring`.

### Prova de mutação

| mutação | gates que morrem |
|---|---|
| **M1** a porta é ignorada (`if false && …`) | 5 — `a_shape_on_the_port_becomes_the_body` · `the_port_pins_the_top_of_the_shape` · `the_port_decides_the_body_not_rows_and_cols` · `the_ring_has_a_hole_and_the_mesh_does_not` · `the_cross_is_a_cross_and_the_mesh_is_not` |
| **M2** o pino volta a ser um ponto (`row * 1e-6`) | 1 — `every_body_swings_and_keeps_its_span` (o gate que ACHOU o defeito) |

⚠️ Uma terceira mutação — tirar a guarda dos `MIN_SPAN` membros por região —
**sobreviveria de propósito**: sobre a malha autorada ela nunca dispara. Em vez de a
gatear por mutação, a afirmação virou gate próprio:
`no_region_of_an_authored_mesh_is_too_small_to_fit`.

### + o P2 da mesma folha: as três secções do painel

**Mesh** / **Physics** / **Pin** — as três perguntas que a célula nomeava. ⚠️ Nada fica
solto, e é uma escolha: um param solto aparece **ANTES** de toda secção. ⚠️ A secção
`Mesh` só responde com a porta VAZIA, e o painel **não a esconde de propósito** —
desligar o fio devolve o corpo que ela descreve.

### Cena `=87` — o que o integrador vê

Três gelatinas do mesmo mastro: **RETÂNGULO** (o controle, porta vazia) · **ANEL**
(`motion.distribute_radial`) · **CRUZ** (dois `motion.grid` num `motion.combine`).
⚠️ O oráculo dos gates é **TOPOLOGIA** (o buraco do anel, os cantos vazios da cruz) e
nunca o tamanho: uma régua de contagem ou de caixa envolvente daria verde sobre três
retângulos.

⚠️ **E o `MAX_DEMO_LEVEL` estava em 84 com as cenas 85 e 86 já no roteador** — o próprio
doc-comment dele previa este modo de falha (*"se alguém esquecer, o controle de contagem
do gate não acusa"*), e aconteceu comigo em dois blocos seguidos. Vai a **87**.

### Ficheiros

| ficheiro | o quê |
|---|---|
| `crates/ph2d-node-motion-soft-body/src/layout.rs` | **NOVO** — `BodyLayout`, `grid_ring`, `hull_ring`, `top_edge`, `Bands` |
| `…/layout_tests.rs` · `…/port_tests.rs` | **NOVOS** — os gates do arranjo e os da porta |
| `…/columns.rs` | **NOVO** — split por HR-18 (os leitores do stream de estado) |
| `…/lib.rs` | a porta `shape` (índice **3**), `is_pinned(layout, i)`, `simulate(…, shape_in, …)` |
| `…/shape.rs` · `…/cluster.rs` | `ring_area(pos, ring)` · `cluster_goals_weighted(…, buckets, …)` |
| `…/params_ui.rs` | `PARAM_GROUPS` (Mesh / Physics / Pin) |
| `shells/desktop/src/motion_state_conferencia_demos_body{,_tests}.rs` | **NOVOS** — a cena `=87` |
| `shells/desktop/src/motion/motion_state_demo_conferencia_body.rs` | **NOVO** — o anúncio |
| `shells/desktop/src/motion/motion_state_demo_router.rs` | `Some("87")` + `MAX_DEMO_LEVEL` 84 → **87** |

**Dois splits por HR-18:** `columns.rs` (o `lib.rs` bateu em 697/700) e `port_tests.rs`
(o `tests.rs` bateu em 731/700).

**Gate deste bloco:** fmt limpo · clippy **0** sobre as 2 crates derivadas do diff ·
typos **0** project-wide · **3 917** testes verdes (`ph2d-node-motion-soft-body` +
`ph2d-host-desktop`).

---

