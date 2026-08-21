# ADR-0161 — O quad remesh PIVOTA para a família GLOBAL: clean-room a partir dos papers, oráculo GPL fora da árvore

Status: **Aceito** (2026-08-20) · Supersede o **plano** do [ADR-0160](0160-quad-remesh-is-a-native-cross-field-port-quadriflow-referenced.md) (o porte permanece, como backend de PREVIEW)
Decisor: Enio · Plano vivo: [`docs/3D/quad-remesh/PLAN.md`](../../3D/quad-remesh/PLAN.md)

## O que se decidiu

1. A retopologia deixa de ser **só** a família local (Instant Meshes / QuadriFlow, ADR-0160) e passa a
   ter um segundo backend da família **GLOBAL**: campo cruzado com singularidades otimizadas →
   decomposição em patches → **quantização inteira global** → quadrangulação por patch.
2. O núcleo global é **clean-room a partir dos PAPERS**. ⛔ Nenhuma linha traduzida do
   `quadwild-bimdf`, `CoMISo`, `vcglib` ou `libQEx` entra nesta árvore — eles são **GPL3**, e a PH2D é
   proprietária de venda única.
3. O binário GPL vive **fora do repositório**, em `/home/enio/Documentos/Projetos/ph2d-quadbench/`,
   invocado por CLI, como **oráculo de bancada**. Nunca linkado, nunca traduzido.
4. O porte do **Instant Meshes** (BSD-3-Clause, ADR-0160) **fica** e vira o backend de **preview**.

## Por que — o número que decidiu

O corpus inteiro, medido lado a lado em 2026-08-20 (`ph2d-quadbench/metrics.py`):

| malha | nosso, quads | nosso, **vértices irregulares** | oráculo, quads | oráculo, **irregulares** |
|---|---|---|---|---|
| esfera 96×144 | 68,7 % | **39,7 %** | **100,0 %** | **0,2 %** |
| esfera-com-bico (o diagnóstico) | 70,5 % | **40,5 %** | **100,0 %** | **0,3 %** |
| esfera amassada | 83,3 % | **23,2 %** | **100,0 %** | **0,2 %** |
| toro | 64,9 % | **48,9 %** | **100,0 %** | **0,0 %** |
| esfera 98 k | 82,7 % | **21,2 %** | **100,0 %** | **0,2 %** |

⭐ **Duas ordens de grandeza na grandeza que o artista vê.** Uma grade tem singularidades — oito numa
esfera. **Nós entregamos 21 a 49 % dos vértices irregulares**; o oráculo entrega 0,2 %. Isso não é
afinação: é o que separa *"a grade tem defeitos"* de *"não há grade"*.

⚠️ **E não é implementação: é a CLASSE.** A família local decide o alinhamento por vizinhança e nunca
negocia globalmente — as singularidades caem onde o campo local se confunde. A quantização inteira
global é o passo que as põe onde a topologia exige e mais nada. O ADR-0160 já registrava esse defeito
como conhecido da literatura; o que mudou é a medição de **quanto** ele custa.

## O que este ADR NÃO decide

- ⛔ **Não remove o ADR-0160.** O porte do Instant Meshes é BSD, está medido, e é a única coisa que
  responde em sub-segundo — ele é o backend de **preview** enquanto o artista pinta.
- ⛔ **Não promete paridade bit-a-bit com o oráculo** — seria juridicamente indesejável num clean-room.
  A meta é **igualar ou superar** nas métricas da §9 do briefing.
- ⛔ Não decide UX de strokes, nem a ordem das fases: isso é o plano vivo, que muda com a medição.

## As três premissas do briefing que a preparação REFUTOU

⚠️ **Elas mudam o custo do plano, e por isso estão aqui e não só no plano.**

1. **«Guide strokes com pressão do stylus»** — **não há pressão** nesta engine. O `winit` escreve
   `force: None` nos três backends de desktop, e existe um gate a afirmá-lo
   (`shells/desktop/tests/the_desktop_shell_has_no_pen_pressure.rs`). O stroke de densidade por
   pressão **não é uma fase do remesher**: é uma camada de tablet que ainda não existe.
2. **«O remesher é um nó do DAG não-destrutivo, como tudo na PH2D»** — o módulo Sculpt **não tem
   DAG**. Ele é snapshot + pilha de undo (`StrokeUndo`). O grafo de nós (`ph2d-nodegraph`, ADR-0032)
   é dos Motion Nodes. Um remesher re-executável e invalidável **é infraestrutura nova**.
3. **«f64 no núcleo»** — a `Mesh` da engine é `f32` de ponta a ponta (posições, normais, octree,
   buffers de GPU). O núcleo do remesher pode ser `f64` internamente; a **fronteira** custa conversão
   e o determinismo cross-platform tem de ser afirmado onde ele de facto vive.

⇒ As três estão **precificadas separadamente** no plano (§Risco), e nenhuma delas bloqueia F0..F5.

## Trilha B — o oráculo, e a fronteira

`ph2d-quadbench/` (fora do repo): `oracle/` (clone GPL, compilado), `corpus/` (10 malhas, incluindo os
sculpts do diagnóstico), `ref/` (saídas do oráculo), `ours/` (saídas nossas), `metrics.py`,
`run_oracle.sh`, `docs/papers/`.

⚠️ **A porta do corpus está DENTRO da engine de propósito** — as fixturas de escultura são desenhadas
com os verbos do produto e não existem fora dele (`shells/desktop/src/sculpt3d_corpus.rs`,
`#[cfg(test)]`, escreve para um caminho **absoluto** fora da árvore). A F0 do plano move isto para o
harness.

## Emenda 1 (2026-08-20) — as três crates clean-room, e o que cada uma provou

| crate | paper-fonte | o que a medição diz |
|---|---|---|
| `ph2d-remesh-iso` | QuadWild 2021 §4 | ⚠️ **NÃO é a alavanca** — cura o `cube` e move a agulha nos dois sentidos no resto (PLAN §4-bis) |
| `ph2d-crossfield` | Bommes 2009 (MIQ) + QuadWild §5 | ⭐ **paridade no CAMPO**: 8 singularidades numa esfera, o ótimo topológico (PLAN §4-ter) |
| `ph2d-quantize` | **Bi-MDF 2023 §3 e §4.4** | ⭐ **fecha com o ótimo DEMONSTRADO** em todos os layouts fechados do oráculo (PLAN §4-quater) |

⚠️ **A fronteira jurídica aguentou os três.** Nenhuma linha traduzida; o que atravessa a fronteira do
oráculo são **saídas** — e desde o F4 elas atravessam já convertidas em números por `layout.py`, que
vive na bancada: **nenhum formato do oráculo (`.patch`, `.corners`, `.rosy`) tem parser dentro da
engine.**

⚠️ **E o `ph2d-quadflow` (porte BSD do Instant Meshes) FICA** — ele é o backend de *preview* do F7, e
a sua licença permissiva é o que torna isso possível. O pivô não o revoga; repõe-no no lugar certo.

## Papers (o que é permitido)

✅ Ler, executar o binário, inspecionar SAÍDAS. ⛔ Traduzir/transcrever fonte GPL.
Em mãos: QuadWild 2021 · Bi-MDF 2023 · MIQ 2009 · Instant Meshes 2015. **Falta QEx 2013** (paywall
ACM) — item aberto no plano.
