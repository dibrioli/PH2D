# ADR-0154 — As formas do Motion são VETOR VIVO na GPU, não tiles assadas

**Status:** Aceito (proposto na linha `line/motion-value`). ⚠️ **Número PROVISÓRIO** — um ADR escolhido numa linha paralela renumera na integração (precedente: 0130→0131, 0134→0140, 0145→0148/0149/0150). O `main` do dia manda.

**Data:** 2026-08-03 · **Contexto do pedido:** um nó de formas "como o MiniCavalry V2, mas melhor" (dropdown de círculo/quadrado/elipse/retângulo/polígono/estrela/coração).

## Contexto

- **O estado da arte é unânime.** Cavalry (o de verdade), After Effects shape layers, Blender Geometry Nodes e Rive tratam uma forma como **geometria vetorial VIVA** que flui pelo grafo e renderiza **resolução-independente na GPU**. O MiniCavalry V2 (o protótipo de referência) usa `ctx.fill` de canvas 2D **raster** — é a aproximação de quem não tem um renderizador vetorial. **Nós temos Vello** (GPU-compute, kurbo+peniko), e o módulo Vector já desenha `VecPath` nítido por ele.
- **O render sink de Motion, porém, só desenha quads texturizados.** `RenderInstance` (`ph2d-render/src/sprite/instance.rs`) = `texture_id + uv_rect + size + basis + tint`; não há campo de geometria. `Domain::Vector` + `CookValue::Opaque` existem no substrato (`ph2d-nodegraph`) mas **nenhum nó emite e o sink ignora** — um beco sem saída.
- **O doc 86 abriu a porta certa, no eixo raster:** `texture_id` virou **convenção de stream** (lida por `lower_to_instances`, ausente ⇒ id 0 ⇒ byte-idêntico) e o `source.object` **assa** um `VecPath` numa textura individual (`motion_object_bake.rs`, cache por conteúdo, `IndividualTextureStore`). Esta ADR é a **mesma porta, um eixo acima**: geometria em vez de textura.
- **E o passe já existe.** `VelloPass` (`ph2d-render/src/vello_pass.rs`) renderiza um `VectorScene` no frame **todo present** (`present.rs:302`, o caminho da ferramenta Vector). `ph2d-vec-render::draw_path(path, transform, &mut VectorScene)` **anexa comandos de encode à cena — sem abrir passe de GPU novo** (doc-comment do crate), com custo **medido de 10k formas = 1,323 ms/frame** de re-encode (`encode_cost_by_n`).

## Decisão

**Uma instância de Motion pode carregar `geometry_id`** — uma **convenção de stream**, o gêmeo exato do `texture_id`: um handle para um `VecPath` cozido num **`VecPathStore`** (cache por conteúdo, o gêmeo do `IndividualTextureStore`). No present, cada instância vetorial é **desenhada nítida** via `ph2d-vec-render` no `VectorScene` que o `VelloPass` **já** renderiza — sem passe de GPU novo, sem pipeline novo.

⇒ Uma forma vinda de um nó (`source.shape`) é **vetor vivo**: nítida em qualquer zoom, sem re-bake, e — porque a geometria é **dado** — componível por boolean/trim/offset/deform/morph rio-abaixo (Fase 2).

**O contrato congelado fica INTACTO.** `geometry_id` é convenção de stream (como `texture_id`), **não** campo do `NodeManifest` — `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` seguem. O nó `source.shape` é **leaf** (deps: `ph2d-nodegraph` + `ph2d-node-registry`).

## Por que não bake-to-tile (a alternativa medida e rejeitada)

O `source.object` do doc 86 **assa** o `VecPath` numa textura. Para uma forma **animada** — o caso criativo — isso re-rasteriza uma **textura Vello INTEIRA por frame** (mais caro que só desenhar o path) e produz **pixels mortos**: nada rio-abaixo pode re-moldar uma tile (boolean/trim/deform/morph ficam impossíveis, e o zoom extremo estoura o teto de 2048 px). Bake-to-tile só vence em **100k+ instâncias ESTÁTICAS de uma forma só** (nicho) — e esse caso ganha um *bake fallback* futuro **se** for medido, não por default.

Criar **entidades ECS Live Shape** por instância também foi rejeitado: churn de spawn/despawn + poluição de undo (o doc 86 avisa contra exatamente isso).

## Consequências

- **Stores e convenções paralelos:** `VecPathStore` ‖ `IndividualTextureStore` · `geometry_id` ‖ `texture_id` · a **MESMA porta única de key** (o descritor da forma, content-addressed — formas idênticas compartilham geometria).
- **Z-order na Fase 1: vetor desenha SOBRE os sprites** (a ordem atual do compositor: `sprite_pass` escreve a surface, `vello_pass` escreve por cima). Z-interleave **por-instância** entre forma-vetor e sprite-texturizado é **Fase 2** — o mesmo problema de segmentação de passe que o compositor do Painter já resolve; nomeado, não escondido.
- **`ph2d-vec-render` ganha uma porta pública de desenho de um `VecPath` avulso** (com transform + paint/tint) — hoje só `draw_path_isolated` sobre uma `VecScene`. Append-only, não toca contrato nenhum.
- **Modelo de perf = custo de ENCODE** (linear no nº de formas × complexidade da forma), não de bake. Nítido em qualquer zoom, **zero re-bake churn**. O teto é o do encode Vello (10k formas ≈ 1,3 ms), não o de 2048 px por tile.
- **Composição em nível de geometria destravada como Fase 2** (deform/boolean/trim sobre a `VecPath`) — a geometria é dado, então isso "cai de graça" no idioma do módulo Vector.
- **Schema intocado:** `PROJECT_SCHEMA`/`VEC_SCENE`/`DOC_VERSION` não mudam (o grafo viaja como texto e carrega a própria versão; nó+convenção novos são aditivos — precedente doc 86).

## Faseamento

- **Fase 1 (esta wave):** a convenção `geometry_id` + o `VecPathStore` + a porta de desenho + a fiação no present + o nó `source.shape` (~10 primitivas de `ph2d-vec-scene`). Entrega formas **nítidas, resolução-independentes, z-ordenadas entre si**.
- **Fase 2 (nomeada):** z-interleave por-instância (segmentação de passe) · deform/boolean/trim em nível de geometria · `source.shape_edges` (distribuição de perímetro pela MESMA `VecPath` — porta única).
- **Fase 3 (só se medido):** *bake fallback* para contagens de instância extremas.
