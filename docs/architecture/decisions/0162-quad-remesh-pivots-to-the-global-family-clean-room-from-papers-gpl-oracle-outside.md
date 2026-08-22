# ADR-0162 — O quad remesh PIVOTA para a família GLOBAL: clean-room a partir dos papers, oráculo GPL fora da árvore

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
| `ph2d-crossfield` | Bommes 2009 (MIQ) + QuadWild §5 | ⚠️ **paridade no CAMPO só em malha BEM DISTRIBUÍDA** — 8 numa grade `uv`, **194** na mesma esfera com distribuição irregular (⛔ célula corrigida em 21/08; PLAN §4-octies) |
| `ph2d-quantize` | **Bi-MDF 2023 §3 e §4.4** | ⭐ **fecha com o ótimo DEMONSTRADO** em todos os layouts fechados do oráculo (PLAN §4-quater) |
| `ph2d-trace` | QuadWild 2021 §6 | ⭐ **a cadeia fecha**: o layout deixou de vir do oráculo e o F4 quantiza-o com prova (PLAN §4-quinquies) |
| `ph2d-quadfill` | QuadWild 2021 §7–§8 | ⭐⭐ **a MALHA**: 100 % quads, χ exata, irregulares de **39,7 % → 0,5 %** — ~85× (PLAN §4-sexies) |

⚠️ **A fronteira jurídica aguentou as cinco.** Nenhuma linha traduzida; o que atravessa a fronteira do
oráculo são **saídas** — e desde o F4 elas atravessam já convertidas em números por `layout.py`, que
vive na bancada: **nenhum formato do oráculo (`.patch`, `.corners`, `.rosy`) tem parser dentro da
engine.**

⭐ **Desde 2026-08-20 o oráculo deixou de ser NECESSÁRIO para a cadeia correr** — ele volta ao papel
que este ADR sempre lhe deu: **régua**, não fornecedor. As fases F1..F4 consomem, hoje, só o que o F3
produz.

⚠️ **E o `ph2d-quadflow` (porte BSD do Instant Meshes) FICA** — ele é o backend de *preview* do F7, e
a sua licença permissiva é o que torna isso possível. O pivô não o revoga; repõe-no no lugar certo.

## Emenda 2 (2026-08-20) — a tese do ADR está MEDIDA no produto final

O corpo deste ADR justificou o pivô com uma tabela de **duas ordens de grandeza** na fração de
vértices irregulares. Com a cadeia F1..F5 completa, a mesma tabela, nas mesmas malhas:

| malha | o motor que o ADR condenou | **a cadeia nova** | oráculo |
|---|---|---|---|
| esfera 96×144 | 68,7 % quads · **39,7 %** irreg. | **100 % · 0,5 %** | 100 % · 0,2 % |
| toro | 64,9 % · **48,9 %** | **100 % · 0,6 %** | 100 % · 0,0 % |
| esfera 98 k | 82,7 % · **21,2 %** | **100 % · 1,1 %** | 100 % · 0,2 % |

⭐ **A decisão deste ADR está PAGA**: a fração caiu ~85× e passou a ser da mesma ordem do oráculo.
⚠️ **E não é o chão.** Uma esfera admite **8** irregulares; o oráculo fica praticamente nele, nós
ficamos em **21**. O resto vem dos ~2× patches a mais do F3 — nomeado, medido, e não escondido.

## Emenda 3 (2026-08-21) — o gargalo mudou de fase, e a tese do ADR não muda

A cadeia deixou de tropeçar na topologia e passou a tropeçar no **campo**.

1. ⭐ **A não-variedade que travava o `cube` era do FLIP da `ph2d-mesh`**, não do
   F1: duas trocas da mesma rodada criavam a mesma diagonal, e nenhuma a via na
   adjacência de entrada. Curada por uma quarta recusa, com dois gates e duas
   provas de mutação. Custo medido: **uma troca em 9 968**. Ela destravou o `cube`,
   a esfera sacudida (**> 20 min → 1,6 s**) e a ruidosa (PLAN §4-septies).
2. ⛔ **E expôs que o F2 só chega ao ótimo em malhas bem distribuídas** — 8
   singularidades numa grade `uv` de 13 682 vértices, **194** na mesma esfera
   remalhada isotropicamente a 10 251. A causa é o **lote** do rounding guloso
   (`194 → 24` ao apertá-lo), que por sua vez existe porque a re-resolução é um CG
   do zero em vez de uma fatoração com *update* (PLAN §4-octies).
3. ⭐ **E a RÉGUA do F2 estava errada**: faltava o defeito angular `K_v` na fórmula
   do índice. Em malha uniforme `K_v ≈ 4π/N` é minúsculo e o erro passava por ruído
   numérico; em malha com triângulos de tamanhos muito diferentes o arredondamento
   ficava em **empate** (0,4999) em milhares de vértices e a soma saía `−147` onde
   a topologia exige `+8`. Corrigida, Poincaré–Hopf vale **exactamente** em todo o
   corpus, e o `cube` — a última malha com a soma errada — fechou (PLAN §4-nonies).
   ⚠️ **O gate da invariante existia e era VERDE**: as quatro fixturas dele são
   todas bem distribuídas, e a soma fechava por cancelamento.

⚠️ **Nada disto toca a decisão deste ADR** — a família global continua a ser a
certa, e a tabela da Emenda 2 continua a valer nas malhas em que foi medida. O que
muda é **onde está o trabalho**: a porta no shell desce na fila, porque ligar o
botão hoje é ligar o pior caso.

⚠️ **E a fronteira jurídica não foi tocada por nada disto**: a cura é da nossa
crate de malha, e a tabela de rounding saiu de medir o nosso próprio solver.

## Papers (o que é permitido)

✅ Ler, executar o binário, inspecionar SAÍDAS. ⛔ Traduzir/transcrever fonte GPL.
Em mãos: QuadWild 2021 · Bi-MDF 2023 · MIQ 2009 · Instant Meshes 2015. **Falta QEx 2013** (paywall
ACM) — item aberto no plano.
