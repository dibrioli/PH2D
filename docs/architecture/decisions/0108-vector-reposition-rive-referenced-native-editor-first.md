# ADR-0108 — Reposicionamento do Vector: reimplementação nativa (ECS/Vello) do modelo Rive, editor-first; boolean edit-time; sem boolean em runtime

- **Status:** ACEITO (Enio, 2026-07-05).
- **Data:** 2026-07-05.
- **Supersede:** a **ambição operacional** da linha vetorial ambiciosa —
  [ADR-0056](0056-vector-network-data-model.md)..[ADR-0068](0068-mobile-core-tier.md) (o plano de 20 waves /
  ~102 tasks / 32 crates: SDF-hybrid GPU boolean, diffusion-curve Poisson, runtime+física Rapier, LLM/ML nodes,
  variable-font-as-network, CRDT). **Mantém como histórico** — não apaga os ADRs; ficam como registro da
  pesquisa. O que sobrevive conceitualmente (data model Bézier, boolean via linesweeper, render Vello) é
  **re-fundado** nos termos deste ADR, não herdado como está.
- **Retira o contrato congelado do Vector** (`VectorOp≤16` / `Vertex`SmallVec32 / `Segment`64 / `Region.segments`16 /
  `MAX_SPIRAL_TURNS`/`MAX_POLYGON_SIDES`/`MAX_VERTICES_PER_LLM_GEN` + gate `architecture_vector_contract_surface`
  em `ph2d-vector-doc`). Este ADR é o **veículo exigido** (CLAUDE.md §0.2/§6, DIRETRIZ §4) para descongelar essa
  superfície; a nova superfície nativa será congelada de novo **quando estável** (não neste doc-step).
- **Não afeta:** contratos congelados de **Nodes** ([ADR-0039](0039-nodegraph-contract-freeze-w2t4.md)) e **Tools**
  ([ADR-0040](0040-tool-as-isolated-feature-crate.md)) — intocados. Não afeta Painter, sprite, render de sprite.

## 1. Contexto

Duas avaliações independentes (2026-07-05) fecharam o diagnóstico que motiva a virada:

1. **O plano ambicioso é "estratosfera", igual à saga do Painter.** 20 waves / ~102 tasks / 32 crates / 13 ADRs
   *Accepted*, com features "primeiro do mundo", **três** motores de boolean, diffusion-curve research-grade,
   física, nós LLM/ML — e **zero kill-criteria**, validado só por auto-nota de IA (9.7/10). É exatamente a doença
   ("alvo irrefutável", DIRETIVA §5) que custou as semanas do Painter.
2. **A base construída é CPU-residente — fundação errada para a meta do Enio.** ~34 crates / ~26k LOC / 578 testes.
   Os 5 tools de desenho (Pen/Pencil/Shape/Select/Direct) + fill + undo **funcionam** no app; mas o grafo procedural
   (18 nós + boolean) só roda atrás de env-flags (`PH2D_VECTOR_GRAPH=1`…), **sem canvas de node-graph**, e
   **animação não existe** (só o tipo `AnimValue` + `sample(t)`). O data model (`VectorNetwork` AoS + `SmallVec`),
   o boolean (linesweeper) e os tools re-stampam na CPU; só o *render* (Vello) é GPU.

**Meta do Enio (2026-07-05):** módulo vetorial **básico, apto a grande número de objetos com muitas operações
animadas em tempo real, GPU-first** — animação (timeline) fica para o futuro; a **arquitetura** já nasce apta.
Prioridade explícita e lição do Painter: **implementação mais simples e mais garantida, ancorada em referência
open-source provada — não reinventar a estratosfera.**

**Investigação Rive (source-level, MIT).** O `rive-runtime` (C++) e `rive-rs` (Rust) são **MIT** (©2023 Rive); o
**editor é fechado**. O runtime é um **player** (carrega `.riv` autorado, faz playback). Achados que dirigem a
decisão: (a) a deformação por bones da foto do Enio é **Linear Blend Skinning clássico na CPU** (`src/bones/`:
`skin.cpp`/`weight.cpp`/`tendon.cpp`; 4 influências/vértice byte-packed; `boneTransforms = boneWorld × inverseBind`);
(b) ela deforma os **pontos de controle Bézier** — âncora **+ handle-in + handle-out** de cada cúbica
(`cubic_vertex.cpp`) → o **path continua exato e editável, NÃO vira mesh** (mesh triangulada é caminho à parte, só
para imagem/textura); (c) o custo real em escala **não é o skinning** (microssegundos), é o **re-encode/upload da
cena Vello por frame** → escala se resolve por **dirty-tracking**, não por compute-shader; (d) **o Rive não faz
NENHUMA operação boolean** — nem runtime nem editor (feature pedida e não entregue, feedback #198/#310); só
clipping/fill-rule em render-time.

## 2. Decisão

**D1 — Estacionar o plano de 20 waves e descongelar o contrato Vector.** A ambição de 0056..0068 vira backlog de
pesquisa (histórico), não compromisso. A superfície congelada em `ph2d-vector-doc` é retirada; a nova é definida
pela reimplementação e re-congelada quando estável.

**D2 — Norte: reimplementação NATIVA do MODELO do runtime Rive, sobre a stack do PH2D.** Rive é a **fonte da
verdade / blueprint provado** e a **fonte dos algoritmos auto-contidos** (portados com atribuição MIT — §3). Mas a
fundação roda **nativa em ECS + kurbo + Vello**, **editor-first**. Explicitamente **rejeitado**: vendorizar o
runtime OOP do Rive como fundação literal, depender da crate `rive-rs`, ou adotar o formato `.riv` (§6, alternativas).

**D3 — "GPU-first" esclarecido: render GPU-residente + operações CPU baratas + dirty-tracking.** O render vive na
GPU (Vello). As operações de geometria (skinning, boolean, offset) são computadas na CPU — são baratas, e é assim
que **até o Rive** faz — e alimentam a cena Vello. O ganho de escala para "muitos objetos" vem de **dirty-tracking
no re-encode** (só re-encodar a sub-árvore que animou), **não** de rodar skinning/boolean em compute shader (que o
Rive não faz e não é o gargalo). GPU-resident/GPU-skinning fica como passo **futuro medido**, se a medição exigir.

**D4 — Boolean só edit-time; sem boolean em runtime.** União/interseção/subtração são **ações de edição** (CPU,
~ms, via linesweeper / fallback `path-bool` do Graphite) que produzem um path editável; o **resultado** anima
(transform/skinning). **Não** haverá boolean recomputado por-frame — sem precedente no Rive, fora de escopo.

**D5 — Skinning: portar o LBS do rive-runtime (MIT) sobre pontos de controle Bézier.** Cada vértice carrega até 4
`(bone_index, weight)`; por frame monta-se os world-transforms dos bones (hierarquia `Transform` do ECS) e aplica-se
a soma ponderada afim às **âncoras + handles das cúbicas**, emitindo o `kurbo::BezPath` deformado → Vello. O path
permanece **exato/editável**; nada vira mesh. O ponto pristino nunca é mutado (deformado em buffer separado).

**D6 — Data model editor-first (mutável, undo-ável); playback é a mesma data vista de outro ângulo.** Não construir
um runtime playback-only e "bolar edição depois" (é justamente o problema do editor-fechado do Rive). As estruturas
nascem editáveis; a avaliação de playback lê a mesma cena.

**D7 — Faseamento honesto.** **Near-term:** desenhar/editar vetor + boolean edit-time + rig de bones + deformação
por skinning (interativa, arrastando o bone). **Futuro:** timeline/keyframe (dirige o rig; modelo de dados espelha
Lottie/Rive), depois state machine / constraints / interatividade (portar a lógica do runtime Rive, MIT).

## 3. Licença / atribuição

- **Rive runtime = permissivo (MIT).** Portar/adaptar **com atribuição** é permitido — **não exige clean-room**
  (diferente dos casos GPL/proprietário Blender-Texture-Paint e Rebelle/Rebecca). Todo código derivado de
  `rive-runtime`/`rive-rs` preserva o aviso de copyright MIT (©2023 Rive) num `NOTICE` da crate correspondente.
- **Sem `.riv`, sem GPL.** Não adotamos o formato `.riv` (governado pelo editor fechado). Referências GPL
  (Inkscape/Blender Grease Pencil) seguem valendo **só como comportamento**, nunca porte literal.
- Deps permissivas já na stack e mantidas: **kurbo**, **vello**, **peniko** (MIT/Apache), **linesweeper**
  (MIT/Apache — repo arquivado jan/2026 + beta; fallback mapeado = `path-bool` do Graphite, Apache-2.0).

## 4. Contrato / superfície congelada

- Este ADR **retira** o gate `architecture_vector_contract_surface` e os caps associados em `ph2d-vector-doc`. A
  **remoção/rework do gate acontece nas fases de implementação**, não neste doc-step (docs primeiro; DIRETIVA §5).
- As 34 crates vetoriais existentes são **aposentadas** (decisão from-scratch do Enio). **Salváveis como
  building-blocks provados** (lift verbatim, sem herdar a arquitetura): o cubic-fit de Levien, a Hobby spline, o
  wrapper de boolean sobre linesweeper (`ph2d-vector-kurbo::boolean_paths`) e a estratégia de schema versionado
  depth-bounded do `postcard_schema`. Isso não é "manter a ambição" — é não re-derivar math resolvida.
- A **nova** superfície nativa (data model editável, rig/skin, cena Vello) será congelada com gate próprio **quando
  estável**, num ADR de follow-up.

## 5. Kill-criteria (DIRETIVA §5 — antídoto do alvo irrefutável)

- **Escala GPU-residente (two-strikes):** o **N** alvo (objetos vetoriais riggados animando @ 60 FPS / resolução-alvo)
  é **fixado por spike de medição** na Fase 0 (medir antes de prometer — não inventar constante). Se, após a **2ª**
  tentativa de arquitetura de dirty-tracking, N não sustentar 60 FPS na GPU-alvo, o modelo GPU-residente-em-escala é
  **reprovado** — PARA e prova o modelo antes da 3ª.
- **Conjunto de aceitação do MVP (concreto/congelado):** definido no plano ([18_plano_reposicionamento_rive_native.md](../../Vector%20Module/18_plano_reposicionamento_rive_native.md))
  — cada fase fecha por **teste comportamental de seam (`ph2d-ui-testkit`) verde + smoke do Enio**, não por
  compile-verde (DIRETIVA §3/§5).
- **Skinning:** paridade numérica do LBS portado contra a referência Rive (mesmos inputs → mesma deformação) é um
  **entregável de teste**, não um "parece certo".

## 6. Alternativas consideradas (e por que rejeitadas)

- **(a) Reaproveitar a base CPU-residente existente.** Rejeitada: é a fundação errada para "GPU-first, apto a
  escala"; reformá-la (AoS→SoA, CPU→GPU-resident, playback→editor-first) custaria mais que refundá-la.
- **(b) Vendorizar o runtime Rive (crate `rive-rs`) como fundação literal.** Rejeitada: (i) é um **player**, não um
  editor — o editor (o produto, 70% do trabalho) é exatamente o que o Rive mantém **fechado**; (ii) OOP por herança
  (`Core→Component→Drawable`) bate de frente com o **norte ECS** (ADR-0075); (iii) acopla ao formato `.riv`
  governado pelo editor fechado; (iv) estruturas read-optimized para playback lutam contra edição mutável/undo-ável;
  (v) a crate é beta. Adotar o *modelo* (D2) captura o valor sem herdar o passivo.
- **(c) Manter o plano de 20 waves.** Rejeitada: sem kill-criteria, alto risco de repetir a saga do Painter.

## 7. Consequências

- **Menos ambição, mais entrega.** O alvo deixa de ser "sucessor procedural-animável do Illustrator" e passa a ser
  um **editor vetorial direto com rig/skinning e boolean edit-time**, GPU-renderizado, animation-ready — cada peça
  com gabarito Rive provado por trás.
- **Norte arquitetural preservado.** Nativo em ECS/kurbo/Vello, drop-crates, sem subsistema OOP estrangeiro.
- **Churn de código real** nas fases de implementação (aposentar 34 crates, novo scaffold) — orçado no plano, não
  neste ADR.
- **Descongelar o contrato Vector é uma decisão do Enio**, registrada aqui como exige o processo (§4).
