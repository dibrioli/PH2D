# Plano — Timeline geral do app (UI): dope-sheet + transporte + curvas

**Data:** 2026-07-08 · **Status:** EM CONSTRUÇÃO (por etapas, a pedido do Enio) · **Linha:** `line/anim`
**Base já pronta e integrada:** dado (`ph2d-anim`: Track/Clip/curvas/easing/RationalTime) + fio do tempo
(`ph2d-core::Playhead` + transporte por teclado) + bindings/apply (`ph2d-timeline`:
`SpriteAnimation`→`Transform` no Playhead). **Falta:** a UI (este plano).

> Escopo: a **timeline GERAL do app** — transporte + dope-sheet que anima QUALQUER propriedade
> (sprite/layer do painter/param de motion node/vetor). NÃO é a timeline do módulo Motion Nodes
> (essa é deferida, `docs/Motion Nodes/01_plano` §Decisões #3). Referência visual: theatre.js.
> Este doc cresce por etapas; cada etapa vira um tópico abaixo.

---

## Tópico 1 — Timelines mais amadas pelos usuários (pesquisa de campo, 2026-07-08)

**Objetivo:** descobrir o que faz uma timeline ser AMADA, pra mirar nisso por construção.

### As mais amadas + o motivo-âncora

| Timeline | O que os usuários amam (a essência) |
|---|---|
| **After Effects** | **Graph editor**: controle fino do que acontece *entre* keys — tangentes bézier, overshoot, slam, ease custom. É o padrão-ouro de *timing*. |
| **Blender** (dope sheet + graph) | **Dope sheet = visão aérea** da cena; keys como blocos que move/escala/duplica em massa (loops triviais). **Separação timing (dope) × valor (graph).** |
| **Spine** (2D game) | Mesma dupla, mas dope sheet mostra **muitas propriedades de uma vez** só com timing — legível quando o graph fica poluído. |
| **Cavalry** | Mental model de AE mas **curva de aprendizado bem menor** + procedural. Precisão de motion **sem rig**. |
| **Rive** | **State machine sobre timelines** — casa com como designer de UI pensa; **editor = runtime** (WYSIWYG real). |
| **Procreate Dreams** | **Timeline por gestos**, touch-first — anima com o filme tocando, sem keyframing complexo. Muito **menos intimidante**. |
| **theatre.js** | **Editor visual DENTRO do app**, sobre objetos reais; UI de timeline + controle por código juntos. |
| **Final Cut** (magnetic) | **Sem trilhas/sem colisão** — clips se afastam sozinhos; foco na história, não na mecânica. **Remove atrito estrutural.** |

### Os 6 padrões transversais (= o que "amado" significa; alvo do design)

1. **Dope-sheet + graph editor = as duas vistas canônicas** (timing × valor). As duas → amado; só uma → frustra. **Inegociável.**
2. **Curvas bézier de 1ª classe** (tangentes editáveis) — motion "vivo" vs robótico. *Já temos o núcleo (`AnimCurve`+bézier).*
3. **Baixa intimidação / curva suave** — Cavalry/Dreams/Rive ganham por **acessibilidade**, não por poder. Gesto direto > setup de keyframe.
4. **WYSIWYG: editor = runtime** (Rive, theatre.js) — animar sobre o objeto REAL, ver na hora. *É a direção do `apply` no Playhead.*
5. **Manipulação em massa de keys** (mover/escalar/duplicar/loop no dope-sheet) — Blender/Spine.
6. **Remover atrito estrutural** — a ferramenta some, a intenção fica.

**Anti-padrões (o que odeiam):** graph poluído com N propriedades; spec ambígua (usar **ms/tempo real**, não só frames); animação desalinhada do que roda de fato (gap editor↔implementação).

### Leitura pra PH2D

A timeline amada = **dope-sheet (timing) + graph editor (curvas bézier)**, WYSIWYG sobre o objeto
real, com baixa intimidação. Isso **valida o que já está pronto** (curvas bézier, apply no Playhead)
e fixa as **2 vistas** que a UI precisa ter. As próximas etapas do plano derivam disso.

**Fontes:** [School of Motion (AE graph editor)](https://www.schoolofmotion.com/blog/graph-editor-after-effects) ·
[Blender Manual — Dope Sheet](https://docs.blender.org/manual/en/latest/editors/dope_sheet/index.html) ·
[Spine — Dopesheet](http://en.esotericsoftware.com/spine-dopesheet) ·
[SuperRenders — Cavalry 2026](https://superrendersfarm.com/article/cavalry-motion-design-review-2026) ·
[UX Collective — Figma+Rive](https://uxdesign.cc/how-i-create-animation-for-interfaces-7183b3b6482f) ·
[Procreate Dreams](https://procreate.com/dreams) · [Theatre.js](https://www.theatrejs.com/) ·
[Frame.io — FCPX Magnetic Timeline](https://blog.frame.io/2017/10/16/fcpx-magnetic-timeline/) ·
[NN/g — Animation duration](https://www.nngroup.com/articles/animation-duration/)

---

## Próximas etapas (a preencher, uma por vez)

- **Tópico 2 —** (a definir com o Enio): provável recorte de UX/ondas a partir do Tópico 1.
- Decisão de design registrada desde já: **unificar os dois transportes** (o `Playhead` geral como
  relógio único que o Motion Nodes também consome) — evitar dois tempos divergentes.
