---
name: project-clip-composition-not-blender-nla
description: Composição de clips (o ex-"NLA") NÃO porta o strip-stack do Blender — o próprio Blender o abandonou, e no 2D o idioma é nesting
metadata:
  type: project
---

**Não porte o NLA do Blender.** Pesquisa de 5 frentes (2026-07-12, a pedido do Enio: *"pesquisa padrão
ouro antes de portar; Blender nem sempre é o melhor"*) **inverteu** o desenho. Resultado congelado em
[ADR-0115](../docs/architecture/decisions/0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md)
+ [plano](../docs/Timeline/02_plano_composicao_clips.md).

**Por quê:**
1. **O Blender está saindo do próprio NLA.** Projeto Baklava/Layered Actions: blend+influence migram do
   **strip** pra **CAMADA**, 5 modos viram 2 (Replace+Combine), tweak mode é o penhasco que eles querem
   matar. Fase 1 (Slotted Actions) shipou no 4.4; Layered Animation em WIP; strips-na-Action é 2027+.
   Veredito do próprio módulo A&R: *"not a pleasure to work with"*.
2. **Nenhum runtime da indústria usa strip-stack** (Rive/Unity/Godot/Unreal/Spine: grafos de mixers ou
   pilhas de trilhas). Strip-stack sobrevive só como ferramenta de **autoria**.
3. **Os sequenciadores convergiram no gesto que falta ao Blender:** *sobrepôs = cruzou* (Unity "Mix mode",
   Unreal "intersecting sections", Maya "the more they overlap, the longer the crossfade"). No Blender,
   strips na mesma faixa **não podem** se sobrepor — você digita blend_in/blend_out na mão.
4. **No 2D, "empilhar e blendar" NÃO é o idioma.** Animate/Harmony/AE **não têm blend de animação nenhum**
   ("blend" neles = compositing de pixel). Moho resolve overlap de Actions por **canais disjuntos**.
   Cavalry (única 2D com blend real) escolheu **camadas por-atributo**. O idioma 2D de reuso é **nesting**
   (símbolo/precomp/artboard aninhado) + hierarquia + ciclo — **e o PH2D tem ZERO nesting** (a lacuna real).

**As 3 armadilhas que a pesquisa expôs no NOSSO código (valem mesmo sem o ADR):**
- `remapped_time` lê `doc.active_clip()` — sob uma pilha, *qual clip dá o relógio da entidade?* é
  **indefinido**. Regra: strip mapeia timeline→clip; o `TimeRemap` **daquele clip** mapeia clip→fonte
  (precomp do AE). Mesma classe de bug de [[feedback-derived-coordinate-seed-must-match-sample]].
- **O apply já é O(bindings²)** (`remapped_time` re-varre a lista por binding). Um laço de strips aninhado
  ingenuamente vira **cúbico** — hoistar é pré-requisito **medido**, não bônus.
- `TranslationX` é posição **ABSOLUTA**: "blend-to-default" (regra do Blender/Godot) **joga o sprite na
  origem do pai**. Eles não sofrem disso porque osso é rest-relative. Daí `rest` **capturado** por binding
  — é o **Capture Base State** do Rive e a **Base Pose** do Unreal, que os dois precisaram adicionar depois.

**Insight guardado p/ o norte node-centric:** *blend-por-parâmetro e frame-pick são a MESMA UX* — "um número
escolhe a pose"; um interpola (canal contínuo), o outro salta (canal discreto). É o Rive Blend 1D = Smart
Bone do Moho = Frame Picker do Animate. Casa com os Motion Nodes **e** com o Flip (desenhos não blendam).

**Why:** portar uma referência sem checar se ela ainda é a resposta certa importa a dívida dela junto.
**How to apply:** antes de portar QUALQUER referência publicada, cheque o roadmap do autor e o idioma do
DOMÍNIO (2D ≠ animação de personagem 3D). Ver [[feedback-perfection-no-deferrals]].
