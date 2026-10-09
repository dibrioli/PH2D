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

*(a preencher na wave W0, com fonte por afirmação e o oráculo CORRIDO — §0.9)*

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

*(W5)*
