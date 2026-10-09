# 126 — CICLO 17: O RITMO (quando cada um se mexe)

> **Ordem do dono, 2026-10-09**, depois da avaliação da stack de referência (Remotion · GSAP · Motion
> Canvas · D3 · Three.js · Manim · Rive) — o prompt da avaliação está no
> [handoff de continuação](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-09_AVALIACAO_DA_STACK.md):
> *«só o ponto 1 [export de vídeo] não me interessa. Vamos fazer os demais. Planeje e comece»*.
> Base: `line/motion-value` em `7de1550f3`, sobre o `main` `2a925859d` (ciclos 13–16 por integrar).

## §0 — A obra que a ordem abre (ciclos 17 → 23)

A avaliação (resumo dela ao dono, 09/10) deixou sete frentes fora o export. Cada uma vira um ciclo do
protocolo ([doc 103](103_dinamica_dos_ciclos.md) §1: grupo → auditoria → cartão → upgrade → medir →
tutorial → smoke do dono). A ordem é por **dependência e ganho ÷ esforço**:

| ciclo | grupo (o que o artista reconhece) | por que nesta posição | estado medido na avaliação |
|---|---|---|---|
| **17** | **O RITMO** — easings de uma só tabela + a cascata (stagger) com origem no espaço e na grelha, também como ATRASO no tempo (`value.time`) | os ciclos seguintes (crossfade do 19, pós do 20, morph do 21) usam as MESMAS curvas | 3+ cópias de easing (`ph2d-anim/src/easing.rs:36`, `motion-stagger/src/ease.rs`, `ph2d-curve`, `map-range` `Interp`); a ordem do stagger é só índice/centro/hash (`motion-stagger/src/order.rs:48`) |
| **18** | **A COR QUE O OLHO VÊ** — OKLab/OKLCH em `color_ramp` · `color_array` · `tint` · misturas de cor | o mais barato; a matemática já existe (`ph2d-color/src/oklab.rs`, `oklch.rs`, já usada pelo Vector) | os nós interpolam em RGB linear (`color-ramp/src/lib.rs:17`, `tint/src/lib.rs:14`) |
| **19** | **O JOGO MANDA** — entradas publicadas do grafo (`value.input`: número · sim/não · gatilho · cor) escritas pelos componentes/Luau/sinais; o tempo de uma sub-árvore DIRIGIDO por um valor; crossfade de estados pelo mixer | maior ganho dentro do jogo; a porta já existe (`ph2d-nodegraph/src/external.rs`, `set_external`, memo-aware) | nenhum input publicado; a `StateMachine` do jogo é só por sinal, sem duração (`ph2d-ecs/src/state_machine.rs:73-110`) |
| **20** | **O ACABAMENTO** — pós-processamento como nós sobre a camada do Motion (blur · glow · sombra de pixel · warp · grade/LUT · vinheta · grão · aberração) | reusa o `FxStackPass` (`ph2d-render/src/fx_stack.rs`) e o alvo HDR isolado do bloom (`ph2d-render/src/motion_fx.rs`) | só o bloom é pixel; drop-shadow/rgb-split são geometria |
| **21** | **UMA FORMA VIRA OUTRA** — morph de contorno (e de texto, glifo a glifo, com os iguais pareados) + write-on por glifo | reusa o `ph2d-vec-blend` (Vector); usa as curvas do 17 | `motion.morph` só desloca pontos pareados pela ordem |
| **22** | **NO COMPASSO** — marcas da timeline lidas por NOME no grafo e no Luau (`waitUntil`), forma de onda e batidas do áudio na timeline, snap às batidas | depende do 19 (o grafo a ler o mundo de fora) | markers nomeados emitem sinal (`ph2d-timeline/src/doc_markers.rs`) mas ninguém os lê por nome; sem onset/waveform |
| **23** | **KEYFRAMES NOS NÓS** — os params dos nós animados pela timeline | era a decisão de 2026-07-09: «para o FIM do módulo, com a timeline PRONTA» (memória `project_motion_keyframes_deferred_timeline_integration`) — a timeline está pronta; fica por último | nenhuma ligação keys → params (`ph2d-timeline/src/apply*.rs`) |

⛔ **Recusados na avaliação (não se reconstroem sem número novo):** cena = f(frame) como LEI (mata as
simulações; o anel de checkpoints já dá o scrub) · corrotinas como forma de AUTORAR motion (não
memoiza nem faz scrub) · a mini-linguagem de posição do GSAP em texto · nó de shader escrito pelo
utilizador (sem resposta igual na CPU, §0.0) · 3D (ADR-0179) · Lottie/Rive como formato de troca ·
layouts de grafo/árvore do D3 · FFmpeg linkado. O export de vídeo ficou FORA por ordem do dono.

⚠️ **Contratos congelados:** nenhum dos sete ciclos toca `NodeOp`/`OpResolver`/`NodeManifest` (§6 do
CLAUDE.md) — tudo entra como nó novo, param apendado no fim da escada, ou canal ao lado
(`set_external`, o canal de texto, o de params dirigidos). Quem descobrir o contrário PARA e reporta.

## §1 — O grupo do ciclo 17 e a premissa do tutorial

**Nós:** `motion.stagger` · `value.time` (o seu `stagger`) · a tabela de easing única (nova, leaf) que
passam a ler o `motion.stagger`, o `value.time`, o `ph2d-anim` (keys da timeline) e o `value.map_range`.

**Tutorial «O ritmo — quando cada um se mexe»:** uma grelha de formas que acorda **do centro para
fora** como uma onda numa poça; a mesma onda a partir de um CANTO, depois de um PONTO arrastado; uma
fila de letras que entra uma a uma com `Back` (o salto do GSAP); e as curvas novas (`Steps`, `Sine`,
`Elastic`, `Spring`, a curva desenhada) vistas lado a lado — com uma simulação (corpos com campo) a
receber a onda, como manda o §1 do 103.

## §2 — Auditoria contra o estado da arte (passo 2)

**O oráculo corrido** (W0): `docs/Motion Nodes/ferramentas/animejs_stagger_oracle/corre.sh` baixa o
anime.js 3.2.2 (licença `MIT`, conferida no `package.json` do ARTEFACTO) para uma pasta temporária,
corre `oracle.cjs` sobre 20 grelhas nossas (filas de 9/10, grelhas 7×5, 10×10 e 8×3 PARCIAL, cada
origem e eixo) e 12 curvas, e grava `crates/ph2d-ease/tests/fixtures/animejs.txt` com cabeçalho. O
que a SAÍDA ensinou (lida, nunca o fonte):

| lei observada | o que fizemos |
|---|---|
| na grelha, a distância é EUCLIDIANA até à célula de partida; `center` é o centro da grelha `((cols−1)/2, (rows−1)/2)`, `last` é a célula da última linha (numa grelha parcial, não o canto) | igual (`order::grid_raw`) — o gate passa nos 20 casos |
| normaliza pela maior distância às linhas EXISTENTES | igual, em forma fechada: o máximo está num vértice do invólucro das células ocupadas (5 candidatos), sem redução — fica no dispositivo |
| com `axis` sem easing a distância tem SINAL (`−1,5…1,5`); com easing o sinal some | não adoptado: a nossa posição é sempre `0..1` (distância); o sentido escolhe-se pela origem (`First`/`Last`/`Cell`) |
| o easing aplica-se como `máx · ease(d/máx)` | igual (`ease(raw)` com `raw = d/máx`) |
| arredonda a distância a centésimos (`√13 → 3,61`) | a régua do gate é `0,006` sobre o normalizado |
| `easeInBack` é `t²(3t−2)` (não o de Penner) | mantido o de Penner (`c1 = 1,70158`, ao bit com o ciclo 2); o `overshoot` passa a ser param |
| `Elastic(1, .3)` é o de Penner (`c4 = 2π/3`) | igual, com o `period` como param; gate `< 2e-6` |
| `cubicBezier` = CSS Easing L1 | igual por 24 bissecções; gate `< 1e-5` nas três curvas do oráculo |
| `spring(m, k, c, v)` chamado fora de uma animação devolve `1666,67` (depende da duração interna) | não serve de oráculo ⇒ a mola fechada é conferida contra a mola INTEGRADA em `f64` (RK4, 200 000 passos): `< 2e-5` em seis `ζ` |
| `from: 'random'` com elementos sintéticos devolve `NaN` | a nossa `Random` é a do ciclo 2 (hash por elemento), ao bit |

⛔ **O GSAP não foi oráculo** (licença não-OSI desde a compra pela Webflow); o anime.js responde a
mesma pergunta (`stagger({grid, from, axis, ease})`).

**O que cada referência oferece e nós agora também:** a cascata com origem na GRELHA e no ESPAÇO
(GSAP/anime.js `grid`/`from`/`axis`) · o atraso TOTAL distribuído (`amount`, o nosso `delay`) ao
lado do por-elemento (`each`, o `stagger` de sempre do `value.time`) · `steps(n)` (CSS) · a mola em
forma fechada (CSS Easing L2 / Motion One) · `cubic-bezier` (CSS) · as curvas da timeline iguais às
dos nós (o GSAP usa a mesma `ease` nos tweens e no `stagger`).

- **Oráculo da cascata:** o `stagger()` do **anime.js** (MIT) tem `grid: [cols, rows]`, `from: 'first'
  | 'center' | 'last' | índice | 'random'`, `axis: 'x' | 'y'`, `ease` sobre a distribuição e
  `start`/valor total — é a mesma pergunta do GSAP (`stagger: {grid, from, axis, ease, each|amount}`)
  sob licença aberta. Corre-se por `node` sobre as NOSSAS grelhas; a saída é fixtura com cabeçalho.
  ⛔ O GSAP não é oráculo: a licença dele (não-OSI, desde a compra pela Webflow) restringe o uso em
  ferramentas que concorram com o construtor visual do Webflow — a conferir no texto da licença na W0,
  mas o anime.js responde a mesma pergunta sem a dúvida.
- **Oráculo das curvas:** as equações de Penner (BSD) já portadas em `ph2d-anim/src/easing.rs` (de
  `simple_easing`, MIT); `cubic-bezier` = a definição do CSS Easing Level 1 (W3C); `steps(n, jump-*)`
  idem; `Spring` = o oscilador harmónico amortecido em forma fechada (o `spring()` do CSS Easing L2 /
  o do Motion One).

## §3 — O desenho (a porta ÚNICA de cada pergunta)

### §3.1 — A tabela de easing: uma crate folha, duas línguas, uma fonte

- **Hoje:** `motion.stagger` tem a sua tabela `f32` sem transcendentais (Sine/Expo/Elastic fora por
  HR-5 e pela paridade bit-exacta do dispositivo) + uma cópia WGSL à mão (`kernel.rs`, `sg_ease_in`);
  o `ph2d-anim` tem outra em `f64` com as 11 famílias; o `map-range` e o `ph2d-curve` têm as suas.
- **Desenho:** crate folha nova `ph2d-ease` — `ease(family, dir, t) -> f32` e o `WGSL_LIB` (texto)
  gerados da MESMA tabela; o `motion.stagger` e o `value.time` passam a ler dela; o `ph2d-anim` prova
  por teste que as suas curvas são as mesmas (tolerância declarada, `f64` vs `f32`). As famílias
  transcendentais entram no dispositivo por **polinómios escritos igual nas duas línguas** (minimax
  de grau fixo, avaliado por Horner) — a resposta da CPU e a do dispositivo continuam iguais ao bit,
  que é a lei da casa; a curva difere do `sin` de biblioteca em `< 1e-6`, medido no gate.
- **Famílias apendadas** (a escada `ease_curve` é wire format — só cresce no fim; `8 = Custom` fica):
  `9 Sine` · `10 Expo` · `11 Elastic` · `12 Steps` (com `steps` e o salto início/fim) · `13 Spring`
  (`stiffness`, `damping`: forma fechada, sub/criticamente/sobre-amortecida) · `14 Bezier` (as duas
  alças do CSS, resolvido por Newton + bissecção, polinomial).
- **Quebra:** nada — as famílias `0..8` ficam ao bit (gate: tabela antiga × nova sobre `4 097`
  pontos, `==`).

### §3.2 — A ordem: do índice ao ESPAÇO e à GRELHA, sem olhar para o vizinho

- **Hoje:** `Index` · `From Center` · `Random` — todos função de `(i, n)`; `By X/Y` recusado porque
  pedia um POSTO (ordenação) e derrubava o nó para a CPU (`order.rs`, cabeçalho).
- **Desenho** (apendado à escada `order`; todos por-elemento ⇒ ficam no dispositivo):
  - `3 Grid` — a linha `i` vira `(col, row)` por `cols` (o anime.js/GSAP `grid`); a origem é uma
    CÉLULA (`from`: primeira · centro · última · índice); `axis`: ambos (distância euclidiana na
    grelha) · só `x` · só `y`; normaliza pela maior distância POSSÍVEL na grelha (fechada em `cols`,
    `rows`, origem — sem redução).
  - `4 From Point` — a distância da POSIÇÃO `P` da linha a um ponto (`origin_x`, `origin_y`, no mundo),
    normalizada por `reach` (o alcance onde a onda chega a `1`; acima disso, `1`). `axis` idem.
    ⚠️ É o «By X / By Y» que o ciclo 2 recusou, **sem** o posto: a pergunta deixa de ser «que lugar
    ocupo» e passa a ser «a que distância estou» — por-elemento.
  - `amount` vs `each` (GSAP): o `value.time` ganha `spread` — o atraso TOTAL distribuído pela
    ordem (`amount`) — ao lado do `stagger` de hoje (`each`, por índice). Default `spread = 0` ⇒ o de
    sempre ao bit.
- **A porta única:** `ph2d-ease::order` (CPU) + o mesmo trecho no `WGSL_LIB` — lida pelo
  `motion.stagger` e pelo `value.time`. ⛔ Nada de cópia por nó.

### §3.3 — O `value.time` vira o stagger NO TEMPO

`value_i = playhead · rate + offset + i · stagger` ganha `− spread · ease(raw_i)` com a ordem de §3.2:
a onda deixa de ser só uma rampa de VALOR (o `motion.stagger`) e passa a ser um ATRASO real — cada
linha começa a sua animação mais tarde, a partir do centro, do canto ou do ponto. Isto é o que o
GSAP chama *stagger* e o que o artista pede quando diz «um de cada vez».

## §4 — Waves

| wave | conteúdo | gates (red-first) |
|---|---|---|
| **W0** | auditoria §2 escrita; oráculo anime.js corrido (`docs/Motion Nodes/ferramentas/animejs_stagger_oracle/`) sobre 3 grelhas nossas; fixtura com cabeçalho | a fixtura existe e o cabeçalho diz versão, licença, comando |
| **W1** | crate `ph2d-ease`: tabela + `WGSL_LIB`; o `motion.stagger` passa a ler dela; famílias `9..14` | `0..8` ao bit; CPU × dispositivo ao bit nas 15 famílias; endpoints exactos; Spring/Bezier contra a forma fechada/CSS |
| **W2** | a ordem `Grid` / `From Point` + `axis` + params no cartão (i18n) | o oráculo anime.js por passo; censo do cartão (todo param alcançável); dispositivo ao bit |
| **W3** | `value.time`: `order` · `ease` · `spread` pela mesma porta | `spread = 0` ⇒ byte-idêntico; atraso = oráculo |
| **W4** | `ph2d-anim` prova a mesma tabela; `map-range` documenta (ou adopta) | paridade `f64`/`f32` com tolerância declarada |
| **W5** | medir (CPU · dispositivo · `loadavg`); cena de smoke com FORMAS + SIMULAÇÃO; fotografar; tutorial 10 em PDF | tabela no §5 deste doc; as duas fotos (com e sem) diferem |

## §5 — Medições

### §5.1 — Correcção (todas pelos gates, nesta máquina: RTX 5060 Ti)

| o quê | medido | gate |
|---|---|---|
| famílias `0..7` contra a tabela do ciclo 2 | **ao bit** em 4 097 pontos × 3 direcções | `ph2d-ease` `the_eight_families_of_cycle_2_are_bit_identical` |
| `sin`/`exp2` polinomiais contra `f64` | `≤ 4e-7` / `≤ 2,5e-7` relativo | `the_polynomial_sin_and_exp2_hold_their_measured_error` |
| Sine/Expo/Elastic contra Penner `f64` · contra o anime.js | `< 1e-6` · `< 2e-6` | `sine_expo_elastic_match_penner_in_f64` · `tests/it` |
| a grelha contra o anime.js | 20 casos, `< 0,006` (a régua do arredondamento dele) | `the_grid_wave_matches_the_oracle_step_by_step` |
| a mola fechada contra a integrada (RK4 `f64`) | `< 2e-5`, `ζ ∈ {0,1 … 3}` | `the_closed_form_spring_follows_the_integrated_one` |
| timeline (`ph2d-anim`, `f64`) contra os nós (`f32`) | pior **`3,2e-7`**, 11 famílias × 3 | `ph2d-anim` `a_mesma_tabela` |
| `motion.stagger` placa × CPU, 6 famílias novas + Back × 3 direcções, Grid/From Point × 3 canais | pior **`9,5e-7`** | `stagger_kernel_matches_the_cpu_in_the_cycle_17_families_and_orders` |
| `value.time` com `delay`, 5 ordens × 6 famílias | pior **`9,5e-7`** | `value_time_delay_matches_the_cpu_in_every_order` |

⭐⭐ **O defeito que a paridade achou: a DIVISÃO do WGSL não é a do IEEE.** `1599/1599` dava
`0,99999994` no dispositivo (a especificação dá `2,5 ULP` à divisão `f32`), e numa família com SALTO
(o `Expo` de Penner vale `0` em `t = 0` e `2⁻¹⁰` logo depois; o `Steps` muda de degrau) um ULP vira
`0,3 %` do curso — `instance 1599: cpu 7,045 vs gpu 7,042`. Nas curvas contínuas do ciclo 2 nunca se
via. ⇒ a lib WGSL divide por `pe_div` (o refinamento de Markstein com `fma`: recíproco corrigido,
quociente, resíduo exacto, correcção) e a paridade fechou a `9,5e-7` nas 7 famílias × 3 direcções e
nas ordens com `Steps`.

### §5.2 — O relógio do grupo (`measure_the_rhythm_group`, `--release`, load `2,46`)

`grid 181² → oscillator → X → output`, 32 761 linhas, mínimo de 31 quadros intercalados:

```text
  caso                   │   linhas │         disp ms │          CPU ms │ ns/linha
  (a base, sem X)        │    32761 │   0.103 / 0.108 │   0.103 / 0.112 │     3.14
  stagger Linear         │    32761 │   0.120 / 0.122 │   0.360 / 0.375 │     3.65
  stagger Quad           │    32761 │   0.120 / 0.125 │   0.365 / 0.385 │     3.66
  stagger Bounce         │    32761 │   0.117 / 0.122 │   0.390 / 0.399 │     3.56
  stagger Sine           │    32761 │   0.118 / 0.122 │   0.417 / 0.431 │     3.59
  stagger Expo           │    32761 │   0.119 / 0.123 │   0.454 / 0.464 │     3.63
  stagger Elastic        │    32761 │   0.121 / 0.126 │   0.516 / 0.526 │     3.70
  stagger Steps          │    32761 │   0.116 / 0.121 │   0.384 / 0.396 │     3.54
  stagger Spring         │    32761 │   0.118 / 0.121 │   0.888 / 0.904 │     3.61
  stagger Bezier         │    32761 │   0.149 / 0.176 │   5.141 / 5.212 │     4.55
  stagger Grid           │    32761 │   0.117 / 0.121 │   0.612 / 0.621 │     3.58
  stagger From Point     │    32761 │   0.117 / 0.120 │   0.380 / 0.391 │     3.56
  time delay Index       │    32761 │   0.126 / 0.130 │   0.367 / 0.376 │     3.84
  time delay Point+Spr   │    32761 │   0.123 / 0.129 │   0.432 / 0.450 │     3.77
```

⇒ **No dispositivo as famílias e ordens novas custam o mesmo que as antigas** (`0,116–0,126 ms`
contra `0,117–0,120`); só o `Bezier` sobe (`+0,03 ms`, as 24 bissecções). Os 14 casos ficam na placa
(a sonda reprova se um cair). **Na CPU o `Bezier` custa `5,1 ms`** (157 ns por linha: 24 iterações
em cadeia de dependência) — é o caminho de reserva (§0.0: ele só tem de dar a mesma resposta); um
Newton com a mesma contagem nas duas línguas cortaria isto, e fica nomeado aqui, não feito.

### §5.3 — A cena `=132` e o que as fotos acharam

Duas galáxias de 256 estrelas (simulação: `Falloff → Vortex (velocidade-alvo) → Attractor →
Integrate`), iguais menos o `Order` do `Time` e do `Stagger`. O gate mede pela bomba do produto, NA
SAÍDA: a `1,5 s` a faixa da laranja (`Index`) ocupa **2 fileiras** e atravessa **2,14 m**; o anel da
azul (`From Point`) tem **0,24 m** e corta **7 fileiras**; a galáxia gira (a estrela que mais anda
`1,84 m` em 3 s) sem fugir nem colapsar (extensão `3,39 → 3,38 m`).

As fotos (`target/prova/c17/fotos/`, perfil `smoke`, placa e `PH2D_GPU_COOK=0`) acharam TRÊS
defeitos da cena que os gates (que liam o `Stagger`) não viam:
1. ⛔ **O `Duplicator` deixa a forma mandar no tamanho** (`point_scale = 0` por omissão) — as 800
   estrelas do mesmo tamanho com o `Stagger` a variar. ⇒ a cena liga `point_scale = 1`, e o gate
   passou a ler a SAÍDA (o que se desenha), não o nó do meio.
2. ⛔ **Girar `72°` uma estrela de cinco pontas devolve a mesma estrela** — a onda de rotação era
   invisível por simetria. ⇒ `GIRO = 36°`.
3. As galáxias encostavam-se e as fichas caíam fora do quadro ⇒ 16×16 com vão `0,32 m`, fichas por
   baixo.

E uma lição de régua: a onda só ocupa parte de cada ciclo de 4 s, e duas das três primeiras fotos
caíram no intervalo sem onda — *uma foto num instante prova o instante*. O PDF usa a do `8 s`.

### §5.4 — O tutorial

`tutoriais/10_o_ritmo.pdf` (fonte `src/10_o_ritmo.html`, figuras recortadas das fotos do app). Gates:
cada linha que o texto manda mexer está no cartão `Time` NO ESTADO em que o passo a pede
(`every_row_the_rhythm_tutorial_names_is_on_the_card`), as figuras existem e o texto abre a `=132`.
