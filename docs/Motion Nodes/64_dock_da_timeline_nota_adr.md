# 64 — O dock da timeline (W4.T4) (nota-ADR)

> **Status:** implementado (linha `line/motion-value`, 2026-07-14). Contrato congelado **intocado**.

## 1. Não faltava sistema. Faltava **geometria**.

O W4.T4 estava na fila há duas jornadas como *"coordene com a linha `anim`"*. A auditoria mostrou
que **quase tudo já estava pronto**:

| | |
|---|---|
| **Relógio** | **ÚNICO.** O `MotionTransport` morreu no W4.T7 — o grafo cozinha no tick do `Playhead`, e o transporte da timeline **já dirige esse mesmo `Playhead`**. |
| **Bridge** | `timeline_bridge::run` roda **todo frame**, visível ou não. |
| **Snapshot** | publicado **todo frame**, incondicionalmente. |
| **Registro do painel** | os 5 sítios **já feitos** (a timeline é um painel global). |
| **Visibilidade** | existe (tecla `L`). |

O que faltava: o `motion_timeline_slot` era um `Rect` de **altura ZERO que ninguém lia** — um seam
reservado desde o M0.T4 —, o `motion_graph` ia até o chrome, e o `timeline` era a mesma faixa
inferior de sempre. **Os dois ocupavam OS MESMOS PIXELS**, e a timeline, desenhada depois, pintava
**por cima** do editor de nós.

## 2. A decisão: não se move um painel, **carva-se uma banda**

`HeroLayout::dock_timeline_into_motion()` — o grafo cede a borda de baixo, e o `layout.timeline`
**vira** essa banda.

Consequência que é o ponto todo: **o painel da timeline não muda uma linha.** Ele já desenha em
`ctx.layout.timeline`; o `ctx.layout.timeline` é que passou a ficar noutro lugar. **Um rect, decidido
num lugar só.** Um painel que tivesse de perguntar *"em qual rect eu estou hoje?"* seria um segundo
lugar para os dois discordarem.

- **Altura:** 200 px (menor que o dock livre de 240), **capada em 45% do grafo** — numa janela baixa,
  200 px fixos deixariam o editor de nós um filete, e um dock que come o hospedeiro é um dock que
  ninguém quer.
- **Por que 200 basta:** debaixo do Motion o dope-sheet está **quase vazio de propósito** — as tracks
  dele bindam a **objetos** do ECS, e um param de Motion não é um (keyframá-los está **deferido**). O
  que o artista precisa aqui é **transporte + régua + scrub**, e isso cabe.
- **A timeline entra COM a tool.** Ela já estava rodando e já dirigia este relógio; só não estava na
  tela a menos que o artista tivesse apertado `L`. **Uma tool que dá auto-play e esconde o transporte
  é uma tool que te pede pra scrubar às cegas.** Sair da tool **não** a esconde de novo: é a timeline
  global, e tirar da tela um painel que o artista está vendo não é nosso.

## 3. As duas armadilhas (e a segunda é uma lição)

**(a) A string era o vetor.** O bridge do shell **escreve** `panel_visibility["motion_graph"]` e o
paint do hero **lê** pra decidir se carva. Uma string digitada nos dois lados são **duas portas para
a mesma pergunta**, e duas portas divergem — **em silêncio**, porque uma chave ausente lê como
`false` e a feature simplesmente **nunca acontece**. Viraram **consts compartilhadas**
(`PANEL_MOTION_GRAPH` / `PANEL_TIMELINE`), e o typo morreu no tipo.

**(b) A mutação que SOBREVIVEU.** Apaguei o **call-site** do dock no `paint.rs` e **os quatro gates
de layout ficaram verdes**. Claro que ficaram: eles testam a *função*, e a função continuava certa. O
que morreu foi a *feature*.

> É [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] na veia: a mutação sobrevivente não
> queria dizer "os gates estão frouxos", queria dizer **"falta um gate"** — ninguém dizia *"e alguém
> tem que CHAMAR isto"*.

O hero não é um `Panel` e não tem seam headless, então o gate lê **o código-fonte do produto** —
exatamente como o arch-gate de ordem-de-frame da projeção de z faz — e exige três coisas: a chamada
existe · está **dentro** do guard de visibilidade · e o guard usa as **consts**, não literais
re-digitados. Com ele, a mesma mutação **morre**.

## 4. Superfície

| Arquivo | O quê |
|---|---|
| `ph2d-editor-core/src/screens/layout.rs` | **`MOTION_TIMELINE_H` (200)** + `MOTION_TIMELINE_MAX_FRAC` (0.45) + **`HeroLayout::dock_timeline_into_motion()`** (pura) + 4 gates |
| `ph2d-editor-core/src/screens/hero.rs` | **`PANEL_MOTION_GRAPH` / `PANEL_TIMELINE`** (consts públicas — o seam) |
| `ph2d-editor-core/src/screens/hero/paint.rs` | chama o dock quando os DOIS painéis estão visíveis |
| `ph2d-editor-core/tests/the_hero_paint_docks_the_timeline_into_motion.rs` | **arch-gate novo** (§3b) |
| `shells/.../motion_bridge.rs` | a timeline **abre com a tool** (mesma borda que já dava `play()`) |
| `shells/.../motion_bridge_dock_tests.rs` | seam: o "antes" (eles se sobrepunham) e o "depois" |
| **`ph2d-panel-timeline`** | **ZERO mudanças.** É o resultado, não uma omissão. |

**Aberto (e é a linha `anim` que decide):** keyframar **params de Motion** exigiria estender
`PropKind`/`TargetBinding` com um alvo `(NodeId, param)` — o redesign que a
[memória de 2026-07-09](../../project-memory/project_motion_keyframes_deferred_timeline_integration.md)
antecipou. **Fora do W4.T4 por decisão do Enio.**
