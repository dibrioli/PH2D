# HANDOFF DE INTEGRAÇÃO · `line/motion-value` · **bloco Z** — 2026-08-23

> **A linha NÃO integrou e NÃO pushou** (`CLAUDE.md` §0.7). **Vinte e um** commits locais, à espera
> de ordem explícita do Enio. **Quatro blocos** no mesmo dia: os TETOS (§1), a folha 11 (§0-bis, §9),
> o defeito que o smoke da folha 11 devolveu (**§0-ter — leia-o primeiro se você integra**: ele é o
> único item deste handoff que muda o pixel de **toda sprite do app**) e a folha 06 (§0-quater).

**Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` · **branch:**
`line/motion-value` · **base:** `main` em `35f937cb2`.

---

## §0-bis — SEGUNDO BLOCO no mesmo dia: **a folha 11 (fx raster)**

Depois do bloco Z, a mesma linha fechou **seis das sete** células da folha 11 —
7 P2 → **1**, e a conferência de 82 para **76**. Registo: as próprias células, que ficaram
densas de propósito (é a forma da conferência), e o §9 abaixo.

| célula | cura | onde |
|---|---|---|
| modo da sombra | `fx.drop_shadow::shadow_blend` | STREAM |
| eixo da lente | `fx.rgb_split::center_x`/`center_y` | STREAM |
| raio limpo | `fx.rgb_split::start` | STREAM |
| operação do halo | `fx.glow::operation` (`Add`/`Screen`) | TELA |
| fonte do bright-pass | `fx.glow::source` (`Luminance`/`Alpha`) | TELA |
| cor do halo por rampa | `fx.glow` + LUT de 512 texels | TELA |
| ⏳ *dirt texture* | **fica**, com o preço corrigido por medição | — |

**Cena de smoke: `=84`.**

---

## §0-ter — O smoke devolveu um DEFEITO, e ele é do renderer, não do nó

**Enio, 2026-08-23, sobre a `=84`:** *"shadow multiply parece não obedecer o alpha da cor"*.

⚠️ **Este é o item de MAIOR alcance do handoff inteiro** — ele muda como **toda sprite do
app** com `BlendMode::Multiply` compõe em alfa parcial (`ph2d-render`, caminho partilhado).
Registo completo, com hipóteses e lições:
[`BUGS_motion_nodes.md` Bug #4](../BUGS_motion_nodes.md).

**O nó estava inocente.** O `fx.drop_shadow` escreve a alfa do fantasma correctamente. O
defeito era o par de fatores do `Multiply` em
[`ph2d_render::pipeline::blend_state_for`](../../../crates/ph2d-render/src/pipeline.rs).

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

**Gates novos** ([`blend_mode_regression.rs`](../../../crates/ph2d-render/tests/blend_mode_regression.rs)):

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
([`layer_composite.wgsl`](../../../crates/ph2d-render/src/shaders/layer_composite.wgsl)).
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

## §0-quinquies — O gate de fecho da jornada (batched, 1×)

| passo | resultado |
|---|---|
| `cargo fmt --all -- --check` | limpo |
| `clippy --all-targets` sobre as **34** crates que o diff nomeia (alvo DERIVADO do diff, nunca escrito à mão) | **0** |
| `typos` project-wide | **0** |
| `nextest-impacted.sh` | 11.423 ✓ |
| **workspace inteira**, `--no-fail-fast`, perfil `ci-test`, **duas corridas independentes** | **17.923 testes · 17.922 ✓ · 1 ✗** nas duas |
| gates de GPU novos (`--ignored`, device real) | paridade da curva 8/8 · paridade XY 2 nós × 2 eixos · sonda da LUT |

**A workspace corrida de propósito, e não só o seletor de impacto:** o diff toca
`ph2d-render` e `ph2d-gpu-cook`, que são foundational — o `nextest-impacted.sh` é o passo
do protocolo, mas um seletor cego sobre uma mudança de renderer era risco que esta jornada
não podia correr. Foi essa corrida que expôs a flake.

⚠️ **O único ✗ é uma FLAKE NOVA e pré-existente, registada no `CLAUDE.md` §5.0 como a
sétima:** `the_region_refresh_is_bound_by_the_footprint_not_by_the_mesh`
([`ph2d-mesh`](../../../crates/ph2d-mesh/tests/measure_normals.rs)) — **verde 3 de 3
sozinha**, a `0,65 s` contra `1,53 s` no fan-out, numa crate que este diff **não toca**.
Ela divide dois relógios de parede (`costs[1] / costs[0]`, barra `3,0`).

⚠️ **O que a torna digna de nota é o doc-comment dela**, que se declara imune: *"o gate é
a FORMA, não o relógio"*. E a forma é medida **dividindo dois relógios** — que é
exactamente o que uma corrida de 17,9 mil testes em paralelo quebra. *Um gate que se diz
independente do relógio ainda o é, se o numerador e o denominador forem tempos.*

### ⚠️ E um alarme FALSO que eu mesmo disparei, porque a busca não tinha controle

Depois da primeira corrida, a cwd do Bash escorregou para a árvore primária (a armadilha
nº 1 do Modo L) e um commit de docs caiu no `main` — revertido cirurgicamente por
`reset --soft` + `restore`, **nunca `--hard`**, porque a árvore primária tinha trabalho
alheio não-commitado (`project-memory/`).

Ao investigar, procurei os testes NOVOS na saída da corrida e achei **zero** — e concluí
que o gate inteiro tinha medido o `main`. ⚠️ **Estava errado:** o comando terminava em
`| tail -20`, então o arquivo tinha vinte linhas e a busca era vazia **por construção**.
É o [[feedback_a_negative_search_needs_a_positive_control]] outra vez, agora contra um LOG
truncado em vez de uma árvore errada. O desempate custou um comando (`nextest list`
mostra os 5 testes novos no conjunto da workspace) e as duas corridas darem o **mesmo**
`17.923 / 1 ✗` já o dizia — se uma delas fosse o `main`, faltariam os ~30 testes da linha.

⇒ **Um gate longo deve IMPRIMIR a árvore que mediu.** A segunda corrida foi lançada com
`pwd && git branch --show-current` na frente, e é por isso que ela se auto-verifica.

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
| `shells/desktop/src/motion_state_demo_conferencia_body.rs` | **NOVO** — o anúncio |
| `shells/desktop/src/motion_state_demo_router.rs` | `Some("87")` + `MAX_DEMO_LEVEL` 84 → **87** |

**Dois splits por HR-18:** `columns.rs` (o `lib.rs` bateu em 697/700) e `port_tests.rs`
(o `tests.rs` bateu em 731/700).

**Gate deste bloco:** fmt limpo · clippy **0** sobre as 2 crates derivadas do diff ·
typos **0** project-wide · **3 917** testes verdes (`ph2d-node-motion-soft-body` +
`ph2d-host-desktop`).

---

## §0-septies — OITAVO BLOCO: **o eco deixa de lembrar** (folha 07, o P1 — e é FOUNDATIONAL)

⚠️⚠️ **LEIA ESTA SECÇÃO ANTES DE INTEGRAR: é a única wave desta linha que toca o
`ph2d-nodegraph`.** [ADR-0163](../../architecture/decisions/0163-a-node-may-cook-its-own-input-at-n-instants-a-time-fan.md).

A folha 07 fecha o P1 dela. Placar: **71 P2 + 2 P1 → 71 P2 + 1 P1**; ✅ 235 →
**236**. O único P1 que sobra é da folha **01** (`motion.emitter` *inherit
velocity*) e é **decisão de produto do Enio**, com as duas saídas já medidas.

### A superfície foundational, e por que ela não colide

| o que entrou | onde | colide? |
|---|---|---|
| `pub type TimeFans = BTreeMap<NodeId, Vec<TimeMap>>` | `cook.rs` | **símbolo NOVO** |
| `Cook::cook_scoped_fanned` · `Cook::advance_tick_fanned` | `cook.rs` | **métodos NOVOS**; os irmãos delegam com leque vazio |
| `EvalCtx::fan(k)` · `EvalCtx::fan_len()` + o campo `fan` | `cook_eval_ctx.rs` | **campo e métodos NOVOS** |
| `cook_node` ganha o 8.º argumento | `cook.rs` (privado) | não sai da crate |
| `MotionCookPump::set_time_fans` / `time_fans` | `ph2d-eval-motion/fans.rs` | **ficheiro NOVO** |

⚠️ **NENHUMA assinatura existente mexeu**, e isso foi desenho e não sorte: a
alternativa (pendurar um argumento no `advance_or_scrub_scoped`) tocaria **29
sítios de chamada** espalhados por duas crates e o shell — um ímã de conflito para
toda linha viva. CLAUDE.md §0.2 pede exactamente isto ao criar foundational novo:
*ponto de extensão append-only*.

⚠️ **O contrato congelado do §6 NÃO se moveu:** `NodeOp` continua com 2 métodos,
`OpResolver` com 1, `NodeManifest` com 8 campos. O `EvalCtx` nunca esteve
congelado, e é onde a capacidade nova mora.

### O que o rastro ganhou

`motion.trail` + `Source` (`Remembered` = o ring, **o default, byte-idêntico** ·
`Resampled` = a entrada re-cozida) + `Forward Steps`.

- ⭐ **eco para a FRENTE** — o que a AE faz por re-renderizar.
- ⭐ **scrub EXACTO** sem `CheckpointRing` (gate: saltar para o quadro 90 é o mesmo
  que andar até lá).
- ⭐ **`length` sem tecto de memória** e **espaçamento não-uniforme** (a lei aceita
  deslocamentos arbitrários; falta-lhe UI, e isso é autoria, não mecanismo).
- ⭐⭐ **A cauda re-cozida é a mais CERTA das duas**, medido: o ring promove a
  cabeça a fantasma **periodicamente**, então as idades dos ecos dele passeiam por
  `1..=spacing` conforme a fase do quadro — até `spacing−1` tiques de erro de
  fase, o tempo todo.

### As três leis que a wave escreve

1. **A lei das gerações vive numa função só** (`echo_offsets`), com **dois
   leitores** — o construtor dos mapas e o `eval`, que dela tira a IDADE para
   desbotar. A escada escrita duas vezes poria o desenho num instante e a cor
   noutro.
2. **`at_age(span, 1)` é `per_tick(span)`**, expressão por expressão — a
   generalização contém o caso antigo, e é por isso que há **uma** lei.
3. **`forward = 0` devolve exactamente a cauda do ring** — a redução que faz o
   modo nascer no neutro.

### ⚠️ Uma afirmação minha ENCOLHEU por mutação

O doc do `TimeFans` dizia que fatias no mesmo instante *«partilham a faixa e o
custo»* graças ao `push_scope`. Trocá-lo por `in_key` deixou **seis gates
verdes** — dentro do laço cada leitura segue a própria cozedura, então os valores
saem certos de qualquer maneira. E o **primeiro** gate que escrevi a perseguir a
mutação (a fotografia do `pre` a ser pisada) **também não a matou**.

A cura foi **encolher a afirmação até ao que a máquina faz**: o que a faixa
própria compra é o instante repetido **fora de ordem**, que é o caso do
espaçamento não-uniforme. Memória:
`feedback_a_claim_no_mutation_can_kill_is_a_claim_about_nothing`.

### Prova de mutação

| mutação | gates que morrem |
|---|---|
| o modo `Resampled` cai no ring | 2 (`the_forward_echo_leads_…`, `the_resampled_tail_is_the_same_whether_you_walk_or_jump_…`) |
| o leque sai da impressão digital | 2 (`a_fan_cooks_the_same_input_at_every_instant_it_names`, `changing_the_fan_recomputes_the_node`) |
| todas as fatias na mesma faixa | 1 (`repeating_an_instant_out_of_order_still_hits_the_memo`) |

### Cena `=88` e o que ela custou de régua

Três bolinhas no MESMO caminho: **LEMBRADO** (o controle) · **RE-COZIDO** (tem de
ficar igual) · **PARA A FRENTE**.

- ⚠️ **A figura é uma Lissajous que se CRUZA**, de propósito: num círculo as duas
  caudas pousariam no mesmo arco e as linhas de baixo seriam indistinguíveis.
- ⚠️ **A régua da direção mede o eco VIZINHO, não o mais velho** — este está a
  meio quinto de volta daqui e projecta **−0,31** numa cauda que vai à frente.
  *Uma régua de direção mede um passo, não uma viagem.*
- ⚠️ **O gate da redução compara a CABEÇA ao bit e os ECOS dentro de um ciclo de
  promoção.** Uma barra apertada mediria o erro de fase do modo ANTIGO e
  reprovaria conforme o quadro calhasse no ciclo.

### Ficheiros e splits

Três ficheiros passaram o tecto de LOC por causa desta wave, e os três foram
cortados (HR-18): `ph2d-node-motion-trail/resample.rs` (a lei e o construtor de
mapas) · a sonda `test.fan` mudou-se do `cook_tests.rs` para junto dos seus gates
· `ph2d-eval-motion/fans.rs`.

`MAX_DEMO_LEVEL` vai a **88**.

**Gate deste bloco:** fmt limpo · clippy **0** sobre as 4 crates derivadas do diff
· typos **0** project-wide · **4 104** testes verdes.

---

## §0-octies — NONO BLOCO: **duas folhas fecham** (09 e 10), e o gate que estava vermelho há um bloco

Placar: **68 P2 + 1 P1** (era 71 + 1); ✅ 236 → **239**. As folhas **09 (cor)** e
**10 (field)** vão a **zero**. O único P1 que resta é da folha 01
(`motion.emitter` *inherit velocity*) e é **decisão de produto do Enio**.

As três células eram a mesma forma: **um número onde a pergunta tem dois lados**.

| célula | cura | ⚠️ a escolha que interessa |
|---|---|---|
| `field.radial_sweep` — softness angular vs radial | `soft_angular`, um **multiplicador** (rótulo *Angular Bias*) | a **CERCA declarada** escolheu a forma |
| `field.remap` — `Clamp Min`/`Clamp Max` | **enum de 4 estados** no param que já existia | um param novo mudaria o sentido de `Clamp = 0` em toda cena salva |
| `motion.color_ramp` — interp por stop | geração **`g4`** + `RampStop::interp` + o botão a ciclar a parada | o `_slot` que o botão já recebia e ignorava era onde a resposta estava |

### ⭐ A cerca do `radial_sweep` escolheu a própria cura

O doc-comment declarava: *"uma knob adimensional, porque as duas bordas vivem em
unidades diferentes — graus vs mundo"*. Um segundo `soft` **absoluto** reabriria
exactamente a pergunta que essa cerca fechou. A cura é um **viés relativo**:
`1` = as duas iguais (byte-idêntico, `x·1.0` é `x` exacto), `0` = borda angular
DURA com a radial macia — o caso que a célula nomeia. Memória:
`feedback_a_declared_fence_chooses_the_shape_of_its_own_cure`.

⚠️ **A sonda errou antes do código:** medir a borda angular a meio raio media a
RADIAL (com `soft = 0.9` o platô acaba em `0,8` de um raio 8), e a faixa dava 159
mesmo com a angular dura. *Uma sonda de uma borda tem de estar longe da outra.*

### ⛔ O `field.remap` NÃO ganhou um `clamp_max`

A primeira tentativa apendou o param — e o gate `multiplier_scales_then_clamp_bounds`,
que já existia, reprovou: com `clamp` a significar *o piso*, toda cena salva com
`Clamp` **desligado** passaria a ter o teto ligado, cortando em silêncio o excesso
que ela desenhava. A escada `0 = Off · 1 = Both · 2 = Min Only · 3 = Max Only`
preserva os dois estados que um documento pode guardar, e os novos vivem onde
nenhum deles chega.

### A rampa: `g4`, e o botão que já recebia a resposta

`<pos>:<r>,<g>,<b>,<a>:<stop_interp_u8>` — o segundo `:` espelha o `x:y:interp` do
`ph2d-curve` que o cabeçalho do módulo já citava como modelo. `STOP_INTERP_GLOBAL
= 255` **e não `RampInterp::COUNT`**: a contagem cresce quando alguém acrescenta
um modo, e um sentinela que anda por cima do primeiro modo novo reinterpreta em
silêncio toda rampa salva. A versão continua escolhida pelo conteúdo — dar e
**tirar** a escolha devolve a string de antes byte a byte, com gate.

⛔ O dispositivo herda de graça: o LUT é assado na CPU pela mesma
`parse_gradient`, então não há WGSL a escrever nem segunda expressão da lei.

---

### ⚠️⚠️ E o portão de fecho apanhou QUATRO vermelhos — leia esta parte

**Um deles estava vermelho desde o bloco do `motion.bezier_warp`** e ninguém o
viu, porque o portão de cada bloco corria só as crates do diff. *O portão do fim
da linha é sobre a WORKSPACE, e é para isto.*

**1. `the_scroll_is_inert_at_todays_row_cap_…`** — o teto de linhas subiu de 20
para 24 e a rolagem deixou de ser inerte. O gate previa-o em texto: *"o dia em
que o teto subir é o dia em que a blindagem passa a ser necessária"*, e nomeava
duas curas candidatas.

⭐ **A cura é *uma banda, dois consumidores*, e a ferramenta JÁ EXISTIA:** o
painel chamava `scene.push_clip(body_rect)` e nunca o gémeo
`hit_index.push_clip(body_rect)` — a mesma pilha que o `section_header::body`
usa desde que nasceu. Duas linhas.

**2, 3, 4.** As outras três caíram *por causa da cura*, e todas pela mesma razão:
mediam **o fundo do último hit-rect**, que com a blindagem **satura na altura da
janela**. O `motion.bezier_warp` passou a ler `802` de um dock de `880` e o gate
acusou-o de ter perdido params. Não perdeu — a sonda deixou de medir o nó.

A régua passou a ser a altura de **conteúdo publicada** (`panel_content_h`), e a
barra com ela: `content_h` contra `visible_h`, o mesmo par que o `dispatch_wheel`
usa. ⚠️ O retrato nomeado do `bezier_warp` **mudou de `1083` para `969`** sem uma
linha de produto se mexer — 114 px é a faixa do título, que a régua velha incluía
e a nova não. Memória:
`feedback_shielding_the_hit_index_changes_what_every_probe_measures`.

⚠️ **E o quarto era WGSL:** o `soft_angular` entrou no kernel sem entrar na lista
de params que gera a `struct KernelParams` — invisível a todo `cargo test` de
crate, apanhado pelo `generated_wgsl_validates`. *Um param novo num kernel são
DOIS sítios.*

### Provas de mutação

| mutação | gate que morre |
|---|---|
| tirar o `hit_index.push_clip` do painel | `nothing_scrolled_above_the_body_can_still_be_clicked_under_the_title` |

### Splits por HR-18 (três)

`ph2d-node-field-remap`: `clamp_tests.rs` + `curve_offset_tests.rs` (o `tests.rs`
bateu em 765/700) · `ph2d-panel-motion-params`: `gradient_row_tests.rs` (o cap dos
painéis é **600**, não 700 — `architecture_panel_loc_cap`).

**Gate deste bloco:** fmt limpo · clippy **0** sobre as 4 crates do diff · typos
**0** · **workspace inteira** verde.

---

## §0-nonies — DÉCIMO BLOCO: **a conferência vai a ZERO P1**

⭐⭐⭐ **Não há mais nenhum P1 aberto na conferência dos nós.** Placar final desta
jornada: **68 P2 · 0 P1 · ✅ 240**.

E o que fechou o último não foi uma capacidade nova — foi **reconferir uma recusa
que o próprio bloco anterior tinha dissolvido**.

### ⚠️ A recusa dissolveu porque o SUBSTRATO mudou, e quem o mudou fui eu

A célula (folha 01, *inherit velocity*) dizia **NÃO**, e a razão era estrutural e
correcta *quando foi escrita*: as duas saídas eram **estado** (que paga a
propriedade que define o nó) ou **velocidade autorada** (que é outra feature). A
**terceira** — re-cozinhar a origem nos instantes de nascimento — não existia.

O bloco §0-septies criou-a (ADR-0163). CLAUDE.md §0.0 diz o resto: *«fora de
escopo porque é inalcançável» é uma afirmação sobre um número que outra pessoa
pode mudar — **quem move o número tem de reconferir a nota***. Numa linha longa,
essa outra pessoa é você mesmo, três blocos depois.

### ⭐⭐ E a medição achou a metade que vinha ANTES da célula

O `P` do emissor é a posição de **nascimento**, e ela era a origem de **AGORA**
para toda partícula viva ⇒ **arrastar o emissor arrastava o penacho inteiro,
rigidamente**. Sonda: `origem +5,0 ⇒ todas as partículas +5,0`.

A célula pedia a velocidade; o defeito de baixo era a **posição**, e nenhuma
célula o tinha nomeado. Os dois curam no mesmo sítio.

### O que o nó ganhou

`Emitter Motion` (enum apendado, `0` = o de sempre **ao bit**):

| modo | o quê |
|---|---|
| **`Carry`** | o penacho anda junto. ⚠️ **Não é um bug com nome bonito** — um efeito ANEXADO (a chama que anda com a tocha) quer exactamente isto |
| **`Leave`** | a partícula fica onde nasceu (a base de toda referência) |
| **`Inherit`** | e ainda leva a velocidade da fonte |

mais **`Inherit Strength`** (o *Strength* da Cavalry), **gateado** ao modo que o
lê — um knob vivo que não muda nada é a doença que o `PARAM_GATES` deste nó já
curava para outros três.

### Os dois números, e de onde saíram

⚠️ **A resolução da história é uma TAXA (240 Hz), não uma contagem.** Uma contagem
fixa repartida pela vida faria a resolução **piorar** quando o artista alonga a
vida — o oposto do que ele pediu. E 240 Hz é escolhido contra a referência que se
quer bater: um motor com estado amostra a posição do emissor **uma vez por
quadro**, então quatro vezes isso já é melhor do que aquilo que se imita.

⚠️ **O tecto é 1024 amostras, e ele nomeia o recurso: TEMPO.** Medido em release
(`custo_de_uma_fatia`), uma fatia de leque custa **~300-490 ns**:

| fatias | ms/quadro | % de um quadro |
|---|---|---|
| 512 | 0,168 | 1,0 % |
| **1024** | **0,435** | **2,6 %** ← shipa aqui |
| 2048 | 0,913 | 5,5 % |
| 4096 | 2,005 | 12,0 % |

2,6% de um quadro por um knob **opcional** de um nó é o que *fácil de usar*
tolera; 5,5% não.

⛔ **Fronteira NOMEADA:** os modos novos são **CPU-only**. O device precisaria de
uma tabela ALIMENTADA pelo leque, e o `LutSpec::fill` vê params e texto, nunca o
grafo — o `applicable` recusa-os, como já recusava `probability < 1`.

### ⚠️⚠️ E o substrato tinha um defeito que só este nó revelou

O leque contava as fatias da **PORTA 0**. Um nó **sem portas** — que é toda FONTE
— lia **zero** fatias com o leque montado e cheio: o emissor ignorava **529
amostras** da própria história em silêncio, com a cena a desenhar e os três modos
idênticos.

⚠️ **E o gate do leque não o apanhou porque contava o trabalho FEITO** (as
cozeduras aconteceram) e a afirmação era sobre o trabalho **RECEBIDO**. A sonda
passou para dentro do consumidor. A lei que fica: **uma entrada por fatia,
sempre** — *o que falta é o conteúdo da porta, não a fatia*. ADR-0163 emendado, e
o leque passou também a re-resolver os params **dirigidos**
(`EvalCtx::fan_param`), sem o que ele não serviria fonte nenhuma.

### Cena `=89`, e a régua que estava invertida

Três fontes iguais varridas pelo mesmo relógio: **CARREGA** (o controle) ·
**DEIXA** · **HERDA**.

⚠️ **A origem é DIRIGIDA POR FIO de propósito** — sem movimento não há história e
os três modos coincidem *por aritmética*; um smoke com a fonte parada ficaria
verde sobre nada.

⚠️ **A minha régua estava invertida:** afirmei que a herança ESPALHA o jacto e
medi **0,1522 contra 0,7943**. Ela **aperta** — uma partícula que leva a
velocidade da fonte **viaja com ela**. A física corrigiu a afirmação, não o
código.

### Provas de mutação

| mutação | gates que morrem |
|---|---|
| a história é ignorada | 4 |
| a herança sai da soma | 2 |

Split por HR-18: `ph2d-node-motion-emitter/history.rs`.

**Gate deste bloco:** fmt · clippy **0** · typos **0** · **workspace inteira:
18 029 testes, 18 029 ✓** — sem uma única flake.

---

## §1 — O que entrou, em uma frase

**Todo teto deste catálogo passa a dizer de que RECURSO ele é** (`CLAUDE.md` §0.0) — 27 params
curados em 11 crates, os dois grampos de integração medidos, e a legenda das cenas de smoke
mudou-se para o canvas. Conferência 89: **89 → 82 P2** (7 células, 4 folhas).

Registro completo: [`docs/Motion Nodes/91_os_tetos_que_ninguem_mediu.md`](../91_os_tetos_que_ninguem_mediu.md).

---

## §2 — Os commits (ordem de aplicação)

| sha | o que |
|---|---|
| `311176ba8` | folha 04: o `falloff` do `motion.kaleidoscope` REFUTADO por medição |
| `1d1634cdb` | a porta que promete um CAMPO e lê só o elemento 0 (`spline_wrap`, `lattice`) |
| `e6e324509` | `motion.wave`: a altura escolhe o canal (`height_channel`) |
| `029a939be` | cena `=83` — o campo que era um número |
| `079305096` | **bloco Z**: 25 tetos de precisão + o `rate` + o ângulo + os dois `MAX_DT` |
| `ef97264af` | a legenda das cenas de smoke vai para o CANVAS |
| `e947b6c98` | o teto de paradas do gradiente ganha derivação (e a medição confirmou o 8) |
| `e04f092b5` | o doc 91 + as 7 células fechadas + as contagens reconciliadas |
| `7538b0d3d` | o handoff do bloco Z + duas flakes novas no §5.0 |
| `32d813b84` | **folha 11**: o modo da sombra e a lente do `rgb_split` (STREAM) |
| `ea8818872` | **folha 11**: a operação e a fonte do halo + o gate de WGSL que faltava |
| `57e3174ec` | **folha 11**: a cor do halo por RAMPA (LUT de 512 texels, medida) |
| `+2` | a cena `=84`, a folha 11 fechada e o split do `motion_fx.rs` por HR-18; e duas memórias |

---

## §3 — Superfície de colisão (o que o integrador tem de saber)

### §3.1 — ⚠️ **Um TOKEN novo** (o único item foundational-ish)

`docs/design/tokens.json` ganha **`chrome.panel-min-w: 220`**, re-exportado como
`ph2d_tokens::PANEL_MIN_W_PX`. O valor **já existia** — era um `const MIN_W` privado *dentro* de
`clamp_panel_rect`, e o comportamento não muda um pixel.

⚠️ **Se outra linha tocou `tokens.json`, este é o ponto de merge.** É um acréscimo de uma linha
num bloco ordenado; a lista de re-export em `ph2d-tokens/src/lib.rs` é alfabética e o nome novo
entra entre `PANEL_HEAD_PAD_PX` e `PANEL_RADIUS_PX`.

### §3.2 — Crates tocadas (11 crates-nó, 3 de infra, 1 shell)

`ph2d-node-{field-box, field-radial-sweep, field-remap, force-attractor, force-buoyancy,
force-vortex, force-wind, motion-emitter, motion-four-point-warp, motion-integrate, motion-move,
motion-spherize, motion-voronoi, sim-spawn, sim-step, value-lfo, registry-init}` ·
`ph2d-{tokens, editor-core, panel-motion-params, gpu-cook}` · `shells/desktop`.

⚠️ **Quase toda a edição é APENDAR uma `static PARAM_HARD_MAX`/`PARAM_HARD_MIN` e uma linha de
`register_*`** — colisão de mesmo-símbolo é improvável, e um merge textual que perca uma das duas
metades é apanhado pelo gate (que exige a entrada E o número).

### §3.3 — Dois splits por HR-18

- `ph2d-node-field-remap/src/params.rs` (novo) — hints/gates/grupos/teto saem do `lib.rs` (686 → 503)
- `ph2d-node-motion-four-point-warp/src/bounds.rs` (novo) — os 16 limites dos 8 cantos (681 → 693)

### §3.4 — Uma dev-dependency nova

`ph2d-node-registry-init` ganha `ph2d-core` em `[dev-dependencies]` (só o gate dos tetos, que mede
à cadência de `DEFAULT_HZ`). **`Cargo.lock` mexe.**

---

## §4 — MUDANÇAS DE COMPORTAMENTO (leia antes de integrar)

### §4.1 — ⚠️ O `MAX_DT` dos dois integradores: `0,1` e `0,05` → **`0,03`**

**Em regime é byte-idêntico** (o tique fixo é `1/60 = 0,0167`, muito abaixo do grampo, e o
`FixedStep` da casa entrega um tique por cozedura mesmo num ecrã lento). O que muda é **quanto de
um SCRUB o sim absorve** — e absorver é a resposta certa ali.

A medição (doc 91 §5): a `0,1`, o laço fechado real com uma força que se alcança **arrastando**
atira uma grelha nascida em raio `1,0` a **127,19**; o irmão `sim.step`, a `0,05`, segurava a mesma
cena em `2,49`.

⛔ **`motion.spring`, `motion.boids` e `motion.wave` NÃO foram tocados.** O `spring` deriva do
`MAX_DT` dele **três** tetos medidos e é wave própria; os outros dois ficam registados como dívida.

### §4.2 — ⚠️ Três testes deixaram de estar pinados a literais

`motion.integrate` tinha três gates com `dt` escrito à mão (`0.1`, `0.05`, `0.1`) que reprovavam
**sobre produto correcto** quando o grampo desceu. Passam a derivar do `MAX_DT`.

### §4.3 — ⚠️ A fixtura do mar da paridade CPU/GPU: `density 40 → 90`

Com o grampo mais apertado, dois tiques deixaram de mover o campo acima do piso de **vacuidade**
(`0,0923` contra `0,1`) — e esse piso é o que impede o gate de comparar dois campos congelados.
⛔ **Alargar o PERCURSO está REFUTADO por medição** (4 tiques dão `0,00526 > ε`, 3 dão `0,00266`):
aquela fixtura é caótica **em número de passos**, e o comentário dela já o dizia. A cura é a FORÇA
(paridade final: `2,3e-4`, 8× de folga sob o ε).

### §4.4 — Nada de UI muda de aparência

Os `ParamHardMax`/`ParamHardMin` só alargam o campo **digitável**; nenhum `ParamUiHint` foi tocado,
então **todo arrasto é idêntico**. A legenda das cenas de smoke é chrome novo e é **no-op** quando
nenhuma cena publicou (todo arranque normal do editor).

---

## §5 — Gate de fecho (batched, 1×)

| | |
|---|---|
| `cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast` | **17.893 testes · 17.892 ✓ · 1 ✗** |
| clippy `--all-targets --all-features`, alvo DERIVADO do diff (**29 crates**) | **0** |
| `cargo fmt --all` | limpo |
| `typos crates/ shells/ docs/Motion Nodes/ CLAUDE.md` | **0** |
| `file_loc_caps` · `architecture_widget_loc_cap` · `architecture_panel_loc_cap` · `architecture_workspace_file_loc_cap` | ✓ |
| `placar_conferencia.py` | verde · **76 P2 · 4 P1 · ✅ 228** |
| paridade CPU/GPU de sim (`--ignored`, RTX) | **29/29 ✓** |
| pipelines do halo num **device real** (as 2 operações, as 2 fontes, com e sem LUT) | ✓ |
| `doc-index.sh` · `architecture_docs_paths_and_smokes_resolve` | ✓ |

⚠️ **O `1 ✗` da corrida final é a flake nº 2 do §5.0** (`a_round_live_offset_costs_like_the_other_joins`,
`ph2d-vec-boolean`) — verde **3 de 3** sozinha, em crate que esta linha não toca. As duas do bloco Z
(a máscara do Painter e o zero-alloc da timeline) não reapareceram nesta corrida, o que é o
comportamento de uma flake e não de uma regressão.

### ⚠️ Os 2 ✗ são FLAKES pré-existentes, em crates que esta linha não toca

| teste | crate | sozinho |
|---|---|---|
| `the_mask_stroke_cost_does_not_follow_the_canvas` | `ph2d-tool-painter` | **4 de 5 ✓** (é flaky mesmo sozinho) |
| `apply_from_doc_is_zero_alloc_steady_state` | `ph2d-timeline` | **3 de 3 ✓** |
| `a_wet_move_costs_what_the_footprint_costs...` | `ph2d-tool-painter` | **3 de 3 ✓** (já listada no §5.0) |

As duas primeiras foram **acrescentadas ao `CLAUDE.md` §5.0** como a 5.ª e a 6.ª — a de zero-alloc
é espécie nova naquela lista.

⚠️⚠️ **E há um achado de PROCESSO ali:** a primeira corrida (com fail-fast) parou em **11.240 com
1.007 testes por correr**. *Um vermelho de flake esconde o resto da suíte.* O `--no-fail-fast` é o
que torna o gate batched uma medição em vez de uma amostra — está escrito no §5.0 agora.

---

## §6 — Prova de mutação (gates novos)

| gate | mutação | RED |
|---|---|---|
| `every_scene_labels_both_halves_on_opposite_sides` | as duas fichas do mesmo lado | ✓ |
| `every_caption_is_chip_sized` | a frase inteira do terminal numa ficha (70 chars) | ✓ |
| `every_precision_bound_param_types_to_the_measured_ceiling` | apanhou, ao vivo, um erro MEU de sinal (`-A - B` em vez de `-(A - B)`) na reescrita dos literais | ✓ |

⚠️ **O terceiro não é uma mutação encenada: é o gate a fazer o trabalho dele durante este bloco.**
Ao trocar os literais truncados pela aritmética exacta (`2_097_152.0 - 0.125`), o `replace` deixou
os pisos negativos como `-2_097_152.0 - 0.125`, que é outro número. *Um piso simétrico escrito como
literal é um sinal à espera de se perder;* a forma segura é a do `bounds.rs`, com um `const REACH` e
`-REACH`.

---

## §7 — O que fica ABERTO (dívida nomeada)

1. ⏳ **`motion.boids::MAX_DT` e `motion.wave::MAX_DT`** continuam a `0,1` **por medir** — copiaram
   o número sem derivação, como os dois que este bloco curou. A sonda `excursion` já está escrita
   (`integrator_ceilings.rs`) e serve-lhes com outro grafo.
2. ⏸️ **`motion.spring`** — mexer no `MAX_DT` dele move **três** tabelas medidas (`friction`,
   saturação do sub-passo, `tension`). Wave própria, com o oráculo de três braços dele.
3. **`MAX_CURVE_POINTS = 8`** (o irmão do teto de paradas, no mesmo arquivo) continua sem
   derivação: *"matches the field.remap text param's practical ceiling"*. O editor de curva tem o
   MESMO `GRAB_R = 9,0`, então a conta é a mesma — só não foi feita.
4. **A legenda no canvas cobre 2 cenas** (`=82`, `=83`). Cena nova = uma `captions()` pura + uma
   linha no `publish`. A lista está em `motion_demo_legend_tests.rs::scenes()`.
5. ⏳ **A *dirt texture* do `fx.glow`** — a única célula da folha 11 que fica, e a estimativa dela
   foi **corrigida por medição**: ela não é «um asset». Uma máscara de sujidade é um overlay de
   TELA no passe do halo e precisa de uma textura que o composite consiga LIGAR — e a textura de
   uma sprite é uma de **três** coisas (`Atlas{key}` · `Individual{texture_id}` · `CookedTexture`,
   ver `sprite_appearance`), das quais só a primeira é um rectângulo no atlas partilhado. Cobrir só
   essa daria uma feature que funciona com umas imagens e falha em silêncio com outras.
6. ⚠️ **Sinal pré-existente:** `conferencia_vs_manifesto.py` sai vermelho na metade *"já existe no
   manifesto"* com 4 células. Lidas uma a uma, são **falsos positivos**: a ferramenta casa o nome
   do param mencionado na CURA proposta (*"um 9º `ease_curve = Custom`"*), não um param que já
   fecharia a célula. A metade das CONTAGENS está verde (127 nós).

---

## §8 — O smoke, para o Enio

Ele já smokou a `=83`. O que este bloco lhe dá para conferir é **o que estava trancado**:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=13 cargo run -p ph2d-host-desktop --release
```

1. Clique no nó **Spherize** (a lente).
2. No painel, ache **Radius**. Ele diz **320**.
3. Arraste o slider: ele salta para **20 ou menos**, e daí não volta.
4. Escreva `320` na caixa e dê Enter. **Antes deste bloco a caixa recusava** (parava em 20); agora
   ela aceita, e a lente volta ao que a cena tinha.

**Deu errado se** a caixa recusar o 320, ou se aceitar e a lente não voltar ao que era.

E a legenda nova:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=83 cargo run -p ph2d-host-desktop --release
```

Cada figura tem agora **uma ficha em cima dela** dizendo o que é. **Deu errado se** as fichas não
aparecerem, ou se aparecerem sobre a figura errada.


---

## §9 — O que a folha 11 custou, em erros meus

1. ⚠️⚠️ **Escrevi por cima de um arquivo que já existia.** A cena nova foi para
   `motion_state_conferencia_demos_fx.rs`, que **é** a cena `=70` (a família `fx.*`, 140 linhas de
   gates). O `Write` respondeu *«updated»* e não *«created»*, e eu li a resposta como sucesso sem
   reparar no verbo — só o compilador acusou, três passos depois. Restaurado do git antes de
   continuar; a cena nova chama-se `_fx_modes`. ⚠️ **O gatilho é estrutural:** um nome BOM para uma
   cena de FX é exactamente o nome que a cena de FX antiga já escolheu, pela mesma boa razão.
   Memória: `feedback_write_on_an_existing_path_says_updated_not_created`.
2. **A régua da rampa corrigiu-se DUAS vezes** (§0-bis): a representação (uma grelha uniforme não
   representa a esquina de uma parada) e depois o critério (num degrau o que encolhe com a
   densidade é a LARGURA da banda, não a altura do erro). Memória:
   `feedback_a_uniform_grid_cannot_represent_a_corner`.
3. **Dois nós ganharam um argumento a mais** e nenhum dos 21 gates antigos o herdou por default: o
   caso neutro tem NOME (`sink_blend`, `Lens::CENTRED`), a lei do `unlimited` do `sim.step`.
