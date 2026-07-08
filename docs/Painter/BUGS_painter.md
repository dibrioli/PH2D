# Bugs do módulo Painter — registro + soluções

> Log vivo de bugs **não-triviais** do Painter (sintoma → causa-raiz → tentativas que falharam → solução).
> O objetivo não é listar todo fix (isso o git já faz), mas registrar os bugs cuja **causa enganava** —
> aqueles em que a aparência levou a vários rounds na pista errada. Cada entrada termina em **lições
> generalizáveis** pra não repetir o erro de diagnóstico.

| # | Bug | Área | Estado | Data |
|---|---|---|---|---|
| [1](#bug-1--offset-de-curva-as-quinas-não-ficavam-paralelas-nem-cruzavam) | Offset de curva — quinas (não-paralelas, depois não-cruzavam) | Stroke shape-editor (Curve/Circle/Polygon/Free Hand) | ✅ Resolvido | 2026-06-29 |
| [2](#bug-2--per-layer-color-fps-despenca--artefatos-retangulares-retângulo-virtual) | Per-Layer Color — FPS despenca + artefatos retangulares ("retângulo virtual") | Stamp path (CPU) + GPU preview slot | ✅ Resolvido | 2026-06-29 |
| [3](#bug-3--queda-de-fps-warp--shapes-booleanas--todo-arraste-interativo) | Queda de FPS (Warp · Shapes booleanas · todo arraste interativo) | Bridge preview + selection recompose + warp mesh | ✅ Resolvido em CPU (2 rodadas 2026-07-04: Transform smoke-OK; per-layer texture-color + overlay booleanas fechados na 2ª — pendente smoke) | 2026-07-04 |
| [4](#bug-4--simplify-curve-degenerava-o-schneider-fit-não-fecha-loops) | Simplify Curve degenerava (curva → 2 pontos idênticos / triângulo) | Selection curve simplify (`fit_curve` fechado) | ✅ Resolvido (DP fechado + Catmull-Rom corner-aware) | 2026-07-05 |
| [5](#bug-5--offset-de-curva-densa-amontoava-os-pontos-após-convert) | Offset amontoava os pontos de uma curva densa após Convert (perda de perfeição) | Stroke offset (movia os pontos de controle) | ✅ Resolvido (offset DRAWING-ONLY, modelo da Seleção) | 2026-07-05 |
| [6](#bug-6--simplify-quase-bom--offset-arredondava-as-quinas--refit--vértice-reconstruído) | Simplify "quase bom" + offset arredondava as quinas | Simplify/Merge (`curve_refit`) | ✅ Resolvido (refit Schneider corner-split + vértice por interseção de bordas) | 2026-07-05 |
| [7](#bug-7--aquarela-grave-queda-de-fps--build-profile--composite-2frame--loops-seriais-não-os-algoritmos) | Aquarela: "grave queda de FPS" — build profile + composite 2×/frame + loops seriais, NÃO os algoritmos | Watercolor render-path + dev profile + heartbeat do shell | ✅ Resolvido (60fps em release com todos os knobs; validado pelo Enio via frame profiler) | 2026-07-07 |

---

## Bug #7 — Aquarela: "grave queda de FPS" — build profile + composite 2×/frame + loops seriais, NÃO os algoritmos

**Sintoma (Enio 2026-07-07, em 3 rodadas):** (1) brush >200px com recursos de aquarela = queda severa de
FPS; (2) após os primeiros fixes, "ainda lento"; (3) **mesmo brush pequeno** (16px) com Bleed/Ragged
Edge/Rewet/Smudge/Pigment no máximo = queda grave. A aparência acusava "algoritmo de aquarela pesado" —
e essa pista estava **quase toda errada**.

**Os VERDADEIROS culpados (por ordem de impacto, todos medidos antes de corrigir):**

1. **O build profile — o maior de todos.** O smoke rodava via `cargo run` = **dev opt-0**; o motor
   custava **10,9 ms/frame em debug vs 2,9 ms em release** (~4×) com os knobs no máximo, e o resto do
   app (Vello/compositor/painel) somava o próprio overhead de debug ⇒ estourava os 16,7 ms e o vsync
   amplificava (perde o vblank → 22-28 fps). O brush comum não sentia porque é ~memcpy; a aquarela é
   matemática por-pixel real (LUTs, blurs, paper procedural) — exatamente o que opt-0 massacra. **Fix:**
   `[profile.dev.package.*] opt-level=2` só nos 4 crates de paint-math (idiom do `ci-test`; dev
   10,9→3,75 ms) + smoke de feel SEMPRE em `--release` (`058dabf0`).
2. **Composite 2×/frame (achado da instrumentação do shell).** Durante o gesto, o Move flush compunha a
   janela E o heartbeat (`on_tick`) compunha DE NOVO — o `grow_wet_soak` forçava `stamped=true` todo
   tick. O frame profiler mostrou `stamps` e `tool-tick` carregando ~4-5 ms CADA. **Fix:** `stamped |=
   parked` — movendo, o soak só folda no dirty (o próximo composite ≤1 frame o inclui); parado (quando o
   bleed crescer sob a ponta É o efeito), segue ao vivo. Bake byte-idêntico; tick em movimento
   3,75→0,01 ms (`4e00e8e8`).
3. **Loops seriais O(janela) num caminho embaraçosamente paralelo.** O composite por-pixel, o `box_blur`
   e o fill dos campos rewet são funções puras por-pixel/linha sem redução — rodavam single-thread por
   disciplina "sem rayon" (que existe pro replay determinístico). **Fix:** exceção sancionada
   **ADR-0109** (3 invariantes: sem redução entre pixels · sem estado mutável compartilhado · sem RNG ⇒
   bit-idêntico independente de nº de threads): composite ∥ (`d775c31c`), box_blur ∥ por transposição
   (`93c14b94`), fill dos campos rewet ∥ (`8ac5be35` — o Rewet era o último knob furando 60fps porque
   com Bleed ≤12 os campos rodam em resolução CHEIA, ds=1). R220 "tudo": frame max 51→10 ms, commit
   238→44 ms.
4. **Paper procedural recomputado todo frame.** PaperCold = ~28 hashes inteiros/pixel, canvas-anchored
   (mesmo pixel ⇒ mesmo valor o traço inteiro) — recalculado a cada composite. **Fix:** memoização
   `wet_substrate` (f32/px, NaN=não-computado, reset no pen-down — o papel não muda no meio do traço,
   então não existe invalidação in-stroke pra errar). `+paper` virou grátis após o 1º toque
   (`2e19c9a0`).
5. **(Não-bug, mas o fato que explica o "mesmo brush pequeno"):** com Rewet+soak o custo por frame é
   **pad-dominado** — a janela recomposta = dirty + 2·(2·Bleed + Ragged) ≈ ±144 px por lado com os
   knobs a 48, **independente do raio do brush**. É a física pedida pelos knobs (1 dab influencia
   ±144 px), não desperdício: o algoritmo já era assintoticamente correto (dirty-rect incremental +
   downsample ds + bake 1×).

**Pistas falsas REFUTADAS por medição (tão importantes quanto os fixes):**
- **"É churn de alocação"** (17 Vecs/frame ∝ janela): implementado o reuso de buffers, medido — **zero
  ganho** (números idênticos dentro do ruído). Os spikes eram compute-bound; o alocador já reciclava
  bem. Revertido no ato — mudança que não se paga não fica.
- **"É a GPU / o upload / o painel":** frame profiler provou GPU 0,8-4 ms (inocente), upload já parcial
  (dirty-bbox), `hero-paint` estável ~1-3 ms.
- **"É o mixer (Charge/Dilution/Pull)":** ablação mostrou ~0,4 ms — desprezível.
- **"O motor está lento" (rodada 3):** ablação reversa com a config exata do Enio provou o motor a
  **2,9 ms/frame em release** — dentro do orçamento com folga; o problema era o item 1.

**Ferramentas que resolveram (e ficam):** sonda de **ablação reversa** (tudo-no-máximo, desligando 1
knob por vez — isola o culpado em uma rodada) · **frame profiler do shell** estendido com
`tool-tick`/`stamps`/`hero-paint` (`PH2D_FLUID_PROFILE=1`, `07d079b9`) — foi ele que pegou o composite
2× que nenhuma sonda unitária pegava, porque só o app real tem a ordem Move-flush→tick do frame ·
sondas temporárias em `--release` sempre revertidas após medir.

**Estado final (validado pelo Enio no app, release):** ~60 fps com todos os recursos; Rewet-only medido
1,1-2,2 ms/frame no motor (R16-R60 × Bleed 7-48). Restante conhecido: pen-down carrega o
`build_wet_backdrop` full-canvas (~15-25 ms 1×/traço @2048² — o S4/backdrop-regional da auditoria doc
12 é o fix mapeado, byte-idêntico, se o soluço de início de traço incomodar).

**Lições generalizáveis:**
1. **"App lento" começa no build profile, não no algoritmo.** Confira `opt-level` ANTES de otimizar
   código — 4× de graça. Corolário: feel-test é em `--release`; e per-package `opt-level` no dev
   profile dá smoke realista sem custar o build inteiro.
2. **Sonda unitária mede o motor; só instrumentar o SHELL pega interação entre fases.** O composite
   2×/frame era invisível em qualquer teste da crate — existia na ordem real Move→tick do frame. Quando
   o probe diz "rápido" e o app diz "lento", instrumente o pipeline inteiro e deixe o split apontar.
3. **Meça para REFUTAR, não só para confirmar.** A teoria da alocação era plausível e caiu em uma
   medição; sem ela, teríamos complexidade permanente (thread-locals) por zero ganho. Fix que não se
   paga, reverte.
4. **Paralelismo byte-idêntico existe e é auditável:** função por-pixel pura + fatias disjuntas + zero
   redução ⇒ bit-igual em qualquer nº de threads. Mas política de determinismo se fura por ADR com
   cerca explícita (ADR-0109), nunca por conveniência local.
5. **Custo pad-dominado engana:** "até brush pequeno é lento" parecia bug e era a janela de influência
   dos knobs (2·Bleed+Ragged). Entender O QUE escala com O QUÊ (ablação por knob) evita otimizar o
   termo errado.

---

## Bug #5 — Offset de curva densa amontoava os pontos após Convert

**Sintoma (Enio 2026-07-05):** depois do **Convert to Curve** (que passou a gerar curvas densas de múltiplos
pontos, ~16px de espaçamento, no P6), aplicar **Offset para dentro** amontoava as âncoras (uma elipse de 24
âncoras encolhida pra raio ~10 ficava com âncoras a **2.5px** umas das outras) → pontos sobrepostos + artefatos
+ "perda da perfeição da curva". Regressão introduzida pela densificação do Convert (antes o Convert dava
poucos pontos, que não amontoavam).

**Causa-raiz:** o offset do stroke **movia os pontos de controle** — `offset_curve_refined` reconstruía a
curva inteira (densify adaptativo + join CAD) e o overlay/fill exibiam a curva OFFSETADA. Num offset pra
dentro de uma curva densa, as âncoras encolhem proporcionalmente ao raio e se amontoam; o offset em si é fiel
(o spine bate no raio certo), mas os pontos de controle exibidos ficam sobrepostos.

**Tentativa que falhou (e a lição):** re-fluir a curva offsetada pra densidade uniforme quando *bunched*.
Corrige o amontoado, mas (a) a 16px de espaçamento os handles Catmull-Rom sub-estimam um círculo pequeno e
encolhem 26%; a 8px fica OK — mas (b) fundamentalmente é **remendo do sintoma**: os pontos ainda se movem e a
curva editável é reconstruída a cada frame. Enio apontou o caminho certo: **"veja como é feito em Seleção —
offset sem mover os pontos, movendo apenas o desenho."**

**Tentativa #2 que falhou (importante):** offsetar o **spine** como polilinha via `line_offset::offset_polyline`
(miter). Mantém os pontos pristinos, mas a curva de desenho fica **imperfeita** — o offset por miter numa
polilinha não é a curva paralela verdadeira (Tiller–Hanson é exato só pra reta+círculo). Enio: *"o resultado
ficou pior, o offset produz curvas imperfeitas. temos documentado como conseguir a curva perfeita."* A **curva
perfeita** já existia: `offset_curve_refined` (reconstrução CAD adaptativa sub-pixel, Levien 2022) — o header
do `curve_offset.rs` a documenta.

**Solução final (drawing-only puro, modelo da Seleção):** o EDITOR inteiro fica na curva **PRISTINA** — âncoras
+ handles + gizmo + a **linha-guia** (`curve_overlay.spine` = flatten da curva pristina, SEM offset) — e **só
o desenho pintado** (os dabs pretos, `curve_fill`/re-stamp de parked) sofre o offset. O desenho usa a curva
paralela PERFEITA: `offset_curve_spine` roda `offset_curve_refined` (CAD adaptativo) e guarda só o spine
achatado. `bake_curve_offset` virou **no-op** (o offset nunca materializa nos pontos; Apply-&-Keep só acumula
o valor). Resultado: nada no editor se move ou amontoa (ponto e linha parados na fonte da verdade) + a pintura
= a parametrização paralela exata. Como a linha-guia é pristina, o "bico"/artefato que aparecia no guia
offsetado **some** (o amontoado e o cruzamento só existiam nas âncoras/guia offsetados, nunca na fonte).

**Lição generalizável (a saga inteira):** três tentativas, uma pista decisiva. (1) re-flow das âncoras
offsetadas = remendo do sintoma. (2) offset por miter da polilinha = rebaixou a curva perfeita. (3) certo:
**a fonte da verdade (âncoras + linha) fica pristina; só o resultado renderizado sofre o offset** — exatamente
como a Seleção offseta a máscara e não a curva. A pista do Enio ("offset movendo apenas o desenho, ponto e
linha parados") era literal: o problema nunca foi o ALGORITMO de offset (o CAD já era perfeito no spine
pintado), foi **exibir/editar sobre a geometria reconstruída**. Quando um offset "move os pontos" gera
artefatos, não troque o offset — pare de mover a fonte.

**Coda — o "bico nos pontos convertidos" (2026-07-05, agentes):** mesmo com o offset drawing-only, o Convert
ainda **assava o offset na geometria** — `stroke_state_to_curve_state` (multi-shape, ramo Curve) rodava
`offset_curve_refined_kinds` e gravava o resultado direto nos pontos de controle; num canto côncavo/over-offset
o join CAD **divide o vértice em dois pontos que se cruzam** (de propósito — o Trim corta a "orelha" no
DESENHO), mas gravado como âncora vira um V auto-cruzado editável. `bake_ellipse_offset` no Convert single
também assava (limpo, mas assava). Enio: *"por que a Seleção faz a mesma coisa e sai perfeita?"* — porque a
Seleção **nunca assa offset nas âncoras** (o offset dela é na máscara). Prova: converter elipse (mesmo
excêntrica rx=120/ry=20), círculo e polígono é **byte-perfeito** (0 cruzamentos, desvio < 0.0004) — o bico só
vinha do assado. **Fix:** o Convert agora produz a geometria **PRISTINA** e o offset **persiste** como
transform de desenho (nada assado, slider não reseta) — `offset_curve_refined_kinds` deletada. Toda âncora
fica exatamente na forma. (Método: 2 agentes — um comparou Stroke vs Seleção linha-a-linha e isolou o ramo
Curve; a densify/split_cubic/ellipse-math provou-se idêntica e correta nos dois. Nota operacional: o agente de
worktree vazou tests `dbg_` no arquivo principal — limpos manualmente.) Bônus: botão **Simplify** estava
escondido em curva convertida (gate só `FreeHand || added_point`); agora aparece em qualquer curva fechada
editável.

**Coda 2 — o "bico" no MERGE (2026-07-05):** o Convert já pristino, mas o **Merge** ainda cuspia bicos nas
cinturas côncavas (peanut). Causa: o Merge (`merge_open_shapes_to_curves` / `selection_merge_curves`) fitava o
contorno traçado com `to_closed_curve_precise` (Schneider tight erro 1.0 + densify 16px) → **muitos** pontos
que, numa cintura pinçada, viravam uma agulha auto-cruzada (turn ~1.94 = quase 180°). O próprio Enio achou o
fix: *"Simplify resolve; reduzir os pontos gerados melhora."* Novo helper `to_closed_curve_smooth` roda o
mesmo redutor robusto do Simplify (`simplify_closed_smooth` — DP + Catmull-Rom corner-aware): âncoras a menos
(62→32 no pinch), zero auto-cruzamento, cintura limpa. Ambos os Merges (stroke + seleção) usam. Lição:
tracing-de-máscara + fit-tight amplifica o staircase do contorno em bicos; o redutor DP-fechado é a saída.
Merge usa tolerância própria `MERGE_SIMPLIFY_TOL_PX=1.0` (mais densa que Simplify).

---

## Bug #6 — Simplify "quase bom" + offset arredondava as quinas → REFIT + vértice reconstruído

> Desenho final consolidado (algoritmos, constantes, testes-âncora):
> [`09_curvas_convert_merge_simplify_offset.md`](09_curvas_convert_merge_simplify_offset.md).

**Sintoma (Enio 2026-07-05):** "não ficou bom o simplify… faça pesquisa de como gerar curvas simplificadas
perfeitas, com números de pontos bem reduzidos. Descubra os tipos de handles adequados." E depois do fix do
Simplify: "os vertex das quinas fazem um offset ruim (arredonda as quinas no offset)."

**Simplify FINAL — refit por mínimos quadrados (pós-pesquisa):** duas iterações anteriores
(decimação DP + handles Catmull-Rom; depois Visvalingam-20% + kinds Symmetric/Vector derivados) produziam
curvas "quase" — porque decimar pontos e derivar handles genéricos **não é fit**. A pesquisa (Schneider 1990,
o pipeline do Inkscape/paper.js `simplify()`; Levien 2023) manda: (1) **detectar quinas** (cusps) no spine
denso (janela de ±3px de arco, giro ≥ ~70°, supressão não-máxima > 2×janela — senão UMA quina vira duas);
(2) **fit cúbico por mínimos quadrados** (Schneider, `fit_curve`) em cada trecho ABERTO entre quinas (aberto
= zero risco do colapso do Bug #4); (3) **kinds do fit**: junção suave = **Aligned** (braços colineares de
comprimentos independentes — o fit carrega a forma nos comprimentos; Symmetric os igualaria e distorceria),
quina = **Free** (braços fitados independentes; Vector os apontaria pros vizinhos e mataria a curvatura de
aproximação). Anel sem 3 cusps ganha seams artificiais nos terços (re-suavizados pra Aligned). Progressivo:
cada aperto escala a tolerância (0.5px ×1.7…) até ~20% das âncoras caírem. Módulo novo `curve_refit.rs` — o
funil único de Simplify E Merge. Resultado medido: círculo denso 16 âncoras → **3 Aligned** (spine a <1.5px
do círculo real); pentágono 15 → **5 Free exatas**. Lições: (a) simplificar curva = REFIT, nunca decimação;
(b) o raio de supressão do detector de quinas tem de exceder o span de resposta (2×janela); (c) anel fechado
precisa ≥3 seams — um cúbico engole meio-anel dentro da tolerância e o assembly degenera.

**Quinas × offset (2026-07-05, follow-up):** com o Merge refitado, o offset **arredondava as quinas**. Causa:
o trace da máscara é SUAVIZADO (média móvel), então a ponta da quina chega ~2px arredondada; o fit ancorava a
quina EM CIMA da ponta arredondada com tangentes estimadas nos primeiros samples (borrados) — e **o offset
amplifica o arredondamento por |d|** (ponta raio 2px offsetada 20px = arco visível de raio 22px). No merge
denso antigo, vários pontos na região reproduziam a quina apertada — por isso "estava bom antes". **Fix
(`CORNER_TRIM_PX`/`corner_vertex` no `curve_refit`):** apara ~3px de arco de cada lado da quina REAL (a ponta
suavizada) e re-ancora os runs do fit na **interseção das duas retas de borda** (medidas num baseline limpo de
3-9px) — vértice-navalha na curva, miter exato no offset. Medido: quadrado com pontas arredondadas → quinas
reconstruídas a <1.2px do vértice verdadeiro; offset de 12px alcança o ápice do miter a <1.5px (arredondado
ficaria ~5px aquém). Lição: quando um consumidor AMPLIFICA erro (offset × curvatura de ponta), a fonte tem de
reconstruir a geometria ideal, não reproduzir fielmente o dado degradado (o trace suavizado).

---

## Bug #4 — Simplify Curve degenerava: o Schneider fit NÃO fecha loops

**Sintoma (Enio 2026-07-05):** ao pedir **Simplify Curve** numa seleção convertida (ex.: um retângulo →
curva densa de 8 pontos), a curva **colapsava**: uma região virava 2 pontos idênticos (`[[58,58],[58,58]]`),
outra virava um triângulo de 3 pontos. A cobertura da seleção sumia. O `Simplify` antigo só rodava com UMA
curva (`selection_shapes.len() == 1`), então isso nunca fora exercido no caminho denso do P6.

**Causa-raiz (o que enganava):** o *spine* achatado estava **perfeito** — 25 pontos formando um quadrado
limpo, `spine[0] == spine[24]`. O problema é o **fitter**: `ph2d_painter_brush::fit_curve` (Schneider,
Graphics Gems 1990) fita **polilinhas ABERTAS** — preserva os extremos e estima as tangentes das pontas por
`p1−p0` / `p_{n-2}−p_{n-1}`. Num **loop fechado** onde `start == end` (ou start≈end, mesma aresta), as duas
tangentes brigam no ponto de costura e o least-squares/reparam de Schneider **colapsa a curva inteira num
único cubo degenerado**. É por isso que o Free Hand funciona (a captura da caneta é **aberta** e só depois é
marcada `closed`) e o Offset/Apply-&-Keep "funcionava" (alimenta contornos **densos** traçados, muitos pontos
→ o fit sobrevive exceto num segmento na costura). Uma curva já-fechada limpa e esparsa (saída do Convert)
não tem pontos suficientes pra mascarar o colapso.

**Tentativas que falharam:** (a) achatar **aberto** (sem duplicar a costura) e re-fechar → ainda degenera:
start e end na mesma aresta a ~9px, o fitter aproxima o quadrado inteiro por 1 cubo. (b) densificar antes do
fit → a degeneração é de **tangente na costura**, não de densidade; não resolve pro caso esparso.

**Solução (`curve_geom::simplify_closed_smooth`):** trocar o fitter por um redutor **closed-loop-correto**:
achatar pro spine denso → **Douglas–Peucker fechado** (`selection_trace::simplify_closed`, tolerância 3px →
âncoras precisas + poucas) → atribuir a cada âncora sobrevivente uma tangente **Catmull-Rom** (⅓ da corda
adjacente por lado), **colapsando pra quina dura** quando o giro local ≥ 60° (`dot(dir_in, dir_out) ≤ 0.5`). Um
retângulo volta a ser 4 quinas afiadas exatas; um laço orgânico fica tão suave quanto um Free Hand.
Transcendental-free (dot + sqrt). O `Simplify` agora roda em **TODAS** as curvas Freehand da lista (antes ou
depois do Merge), não só quando há exatamente uma.

**Lição generalizável:** um fitter de curva "de alta qualidade" pode ser **estruturalmente incapaz** de fechar
loops — o Schneider assume extremos distintos. Antes de reusar um fitter aberto em geometria fechada, cheque
o caso `start==end`; a densidade do input pode **mascarar** o colapso (Offset passava; Convert-esparso pegava).
Para curvas fechadas, DP-fechado + tangentes por vizinhança é robusto e preserva quinas por design.

---

## Bug #3 — Queda de FPS: Warp, Shapes booleanas, e TODO arraste interativo

**Crates/arquivos:** [`shells/desktop/src/render_loop/painter_bridge.rs`](../../shells/desktop/src/render_loop/painter_bridge.rs),
[`tool/paint/selection_shapes.rs`](../../crates/ph2d-tool-painter/src/tool/paint/selection_shapes.rs) +
[`selection_raster.rs`](../../crates/ph2d-tool-painter/src/tool/paint/selection_raster.rs),
[`tool/paint/warp/transform_mesh.rs`](../../crates/ph2d-tool-painter/src/tool/paint/warp/transform_mesh.rs).
**Método:** auditoria de performance **multi-agente, 4 lentes** (Warp · Shapes booleanas · Composite/GPU ·
Alocação), medida em `--release`, correções cruzadas verificadas.

### Sintoma
Queda séria de FPS ao (a) arrastar o gizmo do **Warp**, (b) editar **múltiplas shapes de seleção com
operações booleanas**, e — latente — em qualquer arraste de pintura. Bench-verde escondia tudo.

### Causa-raiz (as 4 lentes convergiram)

- **★ Transversal (afeta Warp, pintura, seleção): deep-copy do canvas inteiro por-move.** O bridge do desktop
  segurava um `Arc::clone(canvas_rgba)` **entre frames** e a detecção de upload GPU era chaveada no
  **ponteiro do Arc** → o `Arc::make_mut(canvas_rgba)` do tool via `strong_count == 2` e **copiava o canvas
  inteiro** (16,8 MB @ 2048², **escala com o CANVAS**, não com a região editada) TODO move. Invisível aos
  benches (que não seguram o Arc entre moves) — o clássico bench-vs-live gap. Também penalizava o Per-Layer
  Color (Bug #2), num eixo que o harness §1.R nunca exercitou.
- **Shapes booleanas:** cada Move do gizmo **re-rasterizava TODAS as N shapes** no canvas inteiro (O(N·A),
  só uma mudou) **e** chamava `invalidate_composite()` → **derrubava o composite + upload GPU do canvas
  inteiro** por-move — apesar de a máscara de seleção **não** entrar no composite (compositor sem nenhuma
  referência a seleção; a marquee é overlay por-frame).
- **Warp:** a grade **pristina** era re-subdividida (Catmull-Rom) todo move (constante durante o arraste).

### Smoke Enio 2026-07-04 (noite) — o eixo TRANSFORM está fechado; per-layer/booleanas NÃO
- **Transform whole-image (P1): ✅ RESOLVIDO, smoke OK** — as bandas paralelas do composite
  (6,4–8,5 ms/move medidos) resolveram a lentidão em todos os 4 sub-modos.
- **Per-Layer Color (P3): causa live achada e fechada em CPU (2ª rodada, mesma noite).** O harness
  antigo setava cor custom em TODAS as camadas → media só o caminho CACHED. O uso real (camadas
  capturadas SEM pick) é **Texture Color, o default** → roteava o kernel DINÂMICO serial
  (`accumulate_shape_layer_rgba`): medido **354 ms/move (N3) e 1,87 s/move (N16)** @2048²·r100
  (`per_layer_perf_live`) — o "FPS 60→10". Três fixes: (1) `take_preview_arc` recompunha o bbox da
  shape inteira por-frame no stack não-trivial — `composite_region_linear` + `encode` agora em bandas
  paralelas (31 → 5 ms); (2) kernel dinâmico batched+banded+layer-fused (354→39 N3 · 1874→181 N16);
  (3) **rota nova `stamp_dabs_cached_color_rgba`**: Texture Color com orientação constante (sem
  Rake/Random/Jitter/Randomize/grain canvas-fixed) assa cada camada num stamp premul-RGBA COM a cor
  (o `render_color_stamp_mask` já sabia) e blita 4-canais fused/banded → **13,1 ms/move (N3, 27×) ·
  54,8 ms (N16, 34×)**; recomposite RGBA compartilhado com o dinâmico, também em bandas. N16×r100
  segue caso-GPU (documentado). Teste de comportamento novo:
  `per_layer_texture_color_paints_each_layers_own_rgb`.
- **Booleanas multi-shape (P4): causa live achada e fechada.** Não era o recompose (cacheado, 5 ms,
  coalesced): era o **overlay de marching-ants** — `selection_overlay_rgba` reconstrói um RGBA do
  canvas INTEIRO **todo frame** (o `phase` anima as ants ⇒ nunca cacheia; roda até parado) =
  **9,9 ms/frame serial @2048²·8 shapes**. Agora em bandas paralelas (per-pixel puro, bit-idêntico):
  **1,35 ms/frame** (7,3×). Harness: `perf_selection_overlay_frame`.

### Regressão 2026-07-04 ("booleanas multi-shape lentas DE NOVO") + fix definitivo
O cache por-shape (`a914a772`) estava INTACTO (re-medido: 5,0 ms/move cache vs 34,0 full). A lentidão live
era **entrega por-evento bruto**: o modo Selection nunca entrou no coalescing por-frame
(`coalesces_canvas_motion` só olhava o stroke method), então um mouse de alta Hz pagava o recompose de
~5 ms VÁRIAS vezes por frame — a mesma tempestade do Bug #2, no eixo da seleção. **Fix:** Selection
coalesce por-frame (gizmo/Rectangle/Ellipse/Automatic agem só na última posição; **Freehand lasso fica de
fora** — captura o path e precisa de todo evento). Guard estendido em
`coalesces_canvas_motion_is_true_only_for_restore_based_fill_methods`.

**No mesmo dia, o eixo Transform/Warp (que o revert do Fix A devolveu ao estado lento) foi fechado por
outro caminho:** medição com o Arc retido (`perf_transform_whole_image_table`) mostrou whole-image 2048² =
**188–218 ms/move** com o loop de gather = ~99% e o deep-copy do Arc = só ~1,3 ms — ou seja, o Fix A mirava
1% do problema (por isso "estritamente melhor" no papel e irrelevante na prática). Fix real: bandas de
linhas paralelas + fast-paths exatos no `over` + strips fora do `affected` viram memcpy + cache da
subdivisão pristina do Warp → **6,4–8,5 ms/move** (29×), byte-idêntico (bandas disjuntas), SEM tocar o
bridge. Mesma alavanca aplicada ao kernel per-layer (95 → 7,9 ms/move; ver
`HANDOFF_per_layer_color_perf_artifacts`). Lição nova: **um custo por-move IGUAL em máquinas muito
diferentes (M2 8GB vs 9950X 128GB) = trabalho serial O(canvas)** — paralelize antes de teorizar sobre
caches/uploads.

### Solução
- **Bridge (`2c64ba80`) — ❌ REVERTIDO (`461dcafd`, 2026-07-04).** A ideia era: `needs_upload` do sinal
  `preview_dirty` em vez do ponteiro do Arc + soltar o clone após o upload → `make_mut` in-place. **Smoke do
  Enio mostrou o oposto: regrediu Warp E Per-Layer Color JUNTOS.** Dois tools não relacionados piorando em
  sincronia = mudança no caminho de display compartilhado, e essa era a única edição local no
  `painter_bridge.rs`. O ganho in-place nunca foi confirmado visualmente e na prática *piorou* — revertido
  por inteiro. **Lição atualizada abaixo (nº 5).** O eixo Per-Layer Color vai pra **GPU** (não mais CPU) —
  ver [`HANDOFF_per_layer_color_perf_artifacts`](../HANDOFF_per_layer_color_perf_artifacts.md) §4.2.
- **Seleção (`a914a772`) — ✅ mantida.** **cache por-shape** da cobertura (chaveado por valor da geometria, auto-validante;
  `Raster` por `Arc::ptr_eq`) → um arraste re-rasteriza **só a shape que moveu** — **medido 34,3 → 5,1 ms/move
  (6,8×)** com 8 shapes em 2048². E **removido o `invalidate_composite()`** da derivação da máscara (o
  composite é comprovadamente independente da seleção) → sem drop de composite/upload por-move.

### Lições
1. **Bench-verde ≠ live-green (o bench-vs-live gap é literal aqui):** o custo dominante (deep-copy do canvas)
   só aparece quando um clone do Arc é retido **entre frames** — exatamente o que o bridge faz e o harness
   não. Sempre modele o retentor real (ver o bench `perf_anchored_drag_per_move_cost` com `hold_preview`).
2. **Detecção de mudança por ponteiro é frágil + load-bearing:** chavear upload no `Arc::as_ptr` fazia o
   `make_mut` (que troca a alocação) parecer "mudou" — o desperdício estava sustentando a correção. Use o
   sinal semântico explícito (`preview_dirty`), não a identidade do Arc.
3. **Invalidação estrutural (`invalidate_composite`) num edit que NÃO toca o composite** = full upload grátis
   por-frame. Antes de invalidar, prove que a saída depende do que mudou (`grep` no compositor fechou isso).
4. **Multi-agente por lentes convergiu na mesma causa raiz** vista de 3 ângulos (Warp/Boolean/Alocação todos
   apontaram o deep-copy) — a triangulação deu confiança pra mexer no caminho de display. **MAS** (ver nº 5)
   convergência de análise estática ≠ prova; o benefício era teórico.
5. **★ Otimização de análise-estática sem smoke visual do caminho de display = aposta.** O bridge fix parecia
   estritamente melhor no papel (mata uma cópia de 16 MB/move) e ainda assim regrediu 2 tools. **Regra:**
   qualquer mudança no caminho de display **compartilhado** (`painter_bridge.rs`, upload GPU, lifecycle do
   Arc de preview) exige smoke visual **por-tool** (Warp *e* pintura *e* seleção) ANTES de considerar
   landada — o commit até se auto-marcou "NEEDS VISUAL SMOKE / revert is one commit", e foi exatamente isso.
   Dois tools piorando em sincronia ⇒ suspeite PRIMEIRO do caminho compartilhado, não de cada tool.

---

## Bug #1 — Offset de curva: as quinas não ficavam paralelas (nem cruzavam)

**Crates/arquivos:** [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/) →
[`tool/paint/curve_offset.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_offset.rs),
[`tool/paint/curve_join.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_join.rs) (novo),
[`tool/paint/curve_trim.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_trim.rs),
[`tool/paint/curve.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve.rs).
**Feature:** o slider **Offset** (card Offset + checkbox **Trim**) do editor de traço — gera a curva paralela
de Curve / Circle-convertido / Polygon-convertido / Free Hand, pra fora e pra dentro, aberta e fechada.

### Sintoma (e como ele evoluiu — o próprio sintoma enganou)

O Offset funcionava nos **trechos curvos** mas falhava nas **quinas**, e a descrição do sintoma mudou a cada
round porque cada fix resolvia uma camada e expunha a próxima:

1. *"O Polygon convertido em curva não offseta direito; parece o algoritmo antigo."*
2. *"Funciona com lados retos; só ao criar ponto novo e curvar é que piora."*
3. *"Lados curvos ficam mais distantes que os lados retos."* (quinas **encurtadas** vs. curvas)
4. *"O handle Free/Aligned/Symmetric piora; Auto/Vector é melhor."*
5. *"As quinas ficam pontudas e **não se cruzam**."* (sintoma final, decisivo)

### Causa-raiz (a verdadeira, achada só no fim)

Havia **duas** causas, em camadas:

- **Camada A — undershoot da quina.** O offset deslocava cada âncora pelo **normal médio normalizado** (unitário)
  × `d`. Num vértice suave isso é exato, mas numa **quina** (descontinuidade de tangente — handle colapsado/Free)
  a curva paralela verdadeira fica na **interseção das duas arestas offsetadas**, a `d / cos δ`, não a `d`.
  Resultado: a quina ficava **mais curta** que os trechos curvos por um fator `cos δ`. Isso explica os sintomas
  3 e 4 (Auto/Vector mantêm tangente contínua → sem descontinuidade → sem undershoot; Free/Aligned criam a
  descontinuidade → undershoot).

- **Camada B — a quina nunca cruzava.** Mesmo corrigindo a distância (miter na interseção), o algoritmo ainda
  produzia **um único vértice por quina**. E **um ponto único nunca se auto-cruza.** O padrão-ouro CAD é
  **offset-then-trim**: cada aresta é offsetada de forma independente, numa quina **côncava** as duas arestas
  **ultrapassam** uma a outra (cruzam), e um passo de **Trim** corta a orelha. Fundir a quina num ponto (mesmo
  na distância certa) **evita** justamente o cruzamento que o resultado pro precisa. Esse é o sintoma 5.

### Tentativas que falharam — e por quê (as lições estão aqui)

| # | Tentativa | Por que pareceu certo | Por que falhou |
|---|---|---|---|
| 1 | Offsetar âncoras ao longo da **tangente** Bézier (não do chord) | A teoria do "chord dá distância desigual" era correta | **Nenhuma mudança visível**: o `offset_curve` já roda sobre pontos **densificados**, onde tangente≈chord. O fix estava num lugar que já era no-op. |
| 2 | **Polyline offset** (offset por segmento de reta + miter join) | Deu "distância correta" | Perdeu as âncoras Bézier/pontos visíveis (Enio rejeitou) **e** ainda artefatava nas quinas. |
| 3 | Restaurar **densificação CAD** com pontos visíveis | Resolveu "ver os múltiplos pontos" nas curvas | Não tocava nas quinas: a densificação refina **dentro** de spans suaves; a quina é uma **junção entre** segmentos. |
| 4 | **Miter** simétrico: `vertex_normal` devolve `(n₁+n₂)/(1+n₁·n₂)` (a interseção) com miter-limit | Corrigiu o undershoot (Camada A); zero regressão em suave/círculo | Ainda **um vértice único** → continuava pontudo, sem cruzar (Camada B intacta). |
| 5 | Miter **assimétrico** (convexa clampa, côncava alcança a interseção sem clamp) | Distâncias 100% corretas em todos os casos | Ainda **um vértice único** por quina → **não cruzava**. Um ponto não se auto-cruza, ponto final. |

**A lição-mãe:** "distância visualmente correta" **não** é prova de que o algoritmo está certo. As tentativas 4 e 5
acertavam a distância e ainda assim estavam erradas na **topologia** (sem cruzamento). Só o sintoma reformulado
pelo Enio — *"não se cruzam"* — revelou que o problema era de **estrutura de saída** (1 ponto vs. 2), não de
posição. Ver [feedback_measure_perf_symptom_scale](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_measure_perf_symptom_scale.md)
e [feedback_tool_unit_green_integration_dead](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_tool_unit_green_integration_dead.md).

### A solução final (offset-then-trim, padrão CAD)

A arquitetura **já estava pronta** para o cruzamento e ninguém tinha percebido: em
[`curve.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve.rs) o **Trim age só no *spine* pintado**
(`trim_offset_spine`), deixando as **âncoras livres pra cruzar** (comentário explícito: *"the anchors may
cross; the crossed loop just isn't painted"*). Faltava o `offset_curve` **produzir** o cruzamento.

Criei [`curve_join.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_join.rs) (irmão do `curve_trim`),
onde o novo `offset_curve` decide **por quina**:

- **Vértice suave** (`n_in ≈ n_out`, dot > `SMOOTH_DOT`): **1 âncora** no normal unitário → círculos / Auto /
  Vector ficam byte-idênticos (sem regressão).
- **Quina convexa** (lado de **fora** da curva — um gap): **1 âncora** no **miter** `(n₁+n₂)/(1+n₁·n₂)`,
  clampada a `MITER_LIMIT` (um espinho convexo não se auto-cruza, então o Trim não o limparia → tem que ser
  limitado).
- **Quina côncava** (lado de **dentro** — as arestas se sobrepõem): **DIVIDE em 2 âncoras** `P_in = V+d·n_in`
  e `P_out = V+d·n_out`. Na côncava elas caem em **lados opostos** do vértice → as duas arestas offsetadas
  **ultrapassam** → o `flatten_spine` gera um spine **auto-cruzado** → o **Trim corta a orelha**. O conector
  reto (handles colapsados) entre `P_in` e `P_out` é exatamente a orelha.

Convexo vs. côncavo é decidido pelo sinal do giro vezes o sinal de `d`:
`côncavo ⇔ (n₁×n₂)·d < 0` (com gate `dot < SMOOTH_DOT` pra um bend suave nunca fragmentar).

**Plumbing do `remap`.** Como uma quina côncava agora vira **2 âncoras**, a saída do `offset_curve` tem tamanho
variável. Ele devolve um `remap: Vec<usize>` (índice de saída → índice de entrada; o split mapeia 2 saídas →
1 entrada), e o [`offset_curve_refined`](../../crates/ph2d-tool-painter/src/tool/paint/curve_offset.rs) **compõe**
esse `remap` com o `origin` da densificação, pra o **bake** continuar carregando handle-kinds + seleção através
do split. O bake materializa o cruzamento na curva editável — o usuário **vê** os pontos cruzados.

**HR-5 (transcendental-free):** tudo é produto vetorial + a rotação complex-multiply do `SegXform`. Nada de
`atan2`/`sin`/`cos`. Não puxa kurbo (usa transcendentais; e há o gate `vello_kurbo_only_in_ph2d_vector`).

### O que NÃO era a causa (red herrings registrados)

- **Miter-join no convexo.** Necessário e correto, mas **insuficiente** — só corrige a distância (Camada A),
  não a topologia (Camada B).
- **Mais densidade de pontos na quina.** Não resolve: a quina é uma **junção**, não falta de amostras. Mais
  pontos só agrupa amostras perto do vértice (errado). Confirmado na literatura (Levien: subdivisão e junções
  são problemas **separados**).
- **Tipo de handle.** Free/Aligned "pioravam" só porque criavam a descontinuidade de tangente que disparava o
  undershoot; não era um bug do handle.

### Arquivos e commits (ordem cronológica da saga)

| Commit | O que fez |
|---|---|
| `3a3f6071` | (tentativa 1) offset ao longo da tangente — no-op por causa da densificação |
| `803a7c76` | (tentativa 2) polyline offset — revertido (perdia Bézier; artefatos) |
| `d9e6e5ab` | (tentativa 3) densificação CAD com pontos visíveis; sem simplificação automática |
| `c6e600ab` | (tentativa 4) miter simétrico corrige o undershoot da quina |
| `7d7d7a7d` | (tentativa 5) miter assimétrico: convexo clampa, côncavo alcança a interseção |
| `99f3aef0` | **solução** — `curve_join.rs`: côncava **divide em 2 âncoras** → spine cruza → Trim corta |

### Verificação

Testes em [`curve_join.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_join.rs) (`cargo test -p ph2d-tool-painter --lib curve_join`):
`a_concave_corner_splits_into_two_overshooting_anchors` (prova: 4 âncoras, `remap=[0,1,1,2]`, `P_in`/`P_out`
em lados opostos; convexo no mesmo canto = 3 âncoras), `a_convex_corner_stays_one_sharp_miter_anchor`,
`offsetting_a_circle_stays_concentric...` (suave nunca fragmenta), `the_convex_miter_reaches_the_true_distance_then_clamps`,
`a_smooth_vertex_miter_stays_unit`, `side_normals_follow_the_handle_tangent_not_the_chord`.
**Smoke do Enio (2026-06-29):** "Perfeito tanto para fora quanto para dentro! Tanto curvas como quinas! Curvas
abertas ou fechadas!"

### Lições generalizáveis

1. **Reformule o sintoma antes de iterar.** O salto de "quinas curtas" para "quinas não cruzam" mudou a classe
   do problema (posição → topologia). Cada round na pista errada custou um commit.
2. **"Parece certo" ≠ "está certo".** Distância visualmente correta escondeu um defeito topológico por 2 fixes.
3. **Junção ≠ amostragem.** Offset de traçado = problema de *stroking*: a parte suave é subdivisão; a quina é
   uma **junção** (miter/round/bevel ou split-and-trim). São mecanismos distintos.
4. **Cheque o que a arquitetura já permite.** O Trim-só-no-spine já deixava as âncoras cruzarem; a correção era
   *upstream* (produzir o cruzamento), não mexer no Trim/dispatch.
5. **Saída de tamanho variável precisa de `remap`.** Ao trocar 1↔N na saída de uma função no meio de um
   pipeline, propague um mapa índice→origem pra os consumidores a jusante (bake/seleção/kinds) não quebrarem.

---

## Bug #2 — Per-Layer Color: FPS despenca + artefatos retangulares ("retângulo virtual")

**Crates/arquivos:**
- **Perf:** [`ph2d-painter-brush/src/stamp_color/accumulate.rs`](../../crates/ph2d-painter-brush/src/stamp_color/accumulate.rs)
  (kernel **fundido** `accumulate_color_stamps_fused`), [`tool/paint/stamp_color_cache.rs`](../../crates/ph2d-tool-painter/src/tool/paint/stamp_color_cache.rs);
  coalescing de ponteiro no shell ([`input_dispatch/painter_canvas_input.rs`](../../shells/desktop/src/input_dispatch/painter_canvas_input.rs)
  + [`render_loop/mod.rs`](../../shells/desktop/src/render_loop/mod.rs)) + `StrokeMethod::coalesces_canvas_motion`.
- **Artefato:** [`ph2d-render/src/individual.rs`](../../crates/ph2d-render/src/individual.rs) —
  `clear_all_mips_transparent` em `create_entry_empty` (clear-on-alloc do slot de preview).

**Feature:** Per-Layer Color (camadas-como-pincel) — N camadas capturadas como Shape, cada uma com sua cor,
compostas em z-order e estampadas ao longo do traço.

Dois sintomas reportados **juntos**, com **causas-raiz diferentes** (essa foi a primeira armadilha):

1. **FPS despenca** ao desenhar (9 FPS) **e** o contador **"Raw" SOBE enquanto o FPS cai** (paradoxo).
2. **Artefatos retangulares:** fatias da imagem do brush **aparecem e somem** em "cantos de retângulos invisíveis".

### Problema A — Perf (estrutural, bem-comportado)

**Medição primeiro** (harness `per_layer_perf` em [`tool/paint/tests.rs`](../../crates/ph2d-tool-painter/src/tool/paint/tests.rs),
`--release`): split de fases revelou **um único kernel = 96.5%** do custo por-Move — `accumulate_color_stamp_coverage`,
`O(D·N·S)` (D dabs × N camadas × footprint (2r)²), refeito pra forma inteira a cada pointer-Move. **Refutou** a
teoria do handoff (que culpava bbox/recompose/upload — D/H≈1.0 provou que **não** é bbox-bound). O "Raw sobe" é a
assinatura: as estampas rodam **fora** da janela de encode que o Raw mede, então `frame_cpu_ms` (Raw) **cai**
enquanto o wall-clock total (FPS) **sobe**.

**Fix:** (1) **kernel fundido alpha-only** — todos os N stamps compartilham `size`, então as coords bilineares
são computadas **1×/pixel** (não ×N) e só o canal alpha é amostrado (o caminho descarta o RGB) → **3.2–4.5×**,
byte-idêntico (gate `fused_per_layer_accumulate_is_bit_identical_to_sequential`). (2) **Coalescing de ponteiro
por-frame** dos métodos de forma (Curve/Line/Circle/Polygon) — colapsa o storm de re-estampa por-evento bruto em
1/frame (incrementais resamplam o segmento, ficam de fora por design). **Limite aceito:** com pincel grande × N16
× canvas grande uma estampa única ainda é ~110 ms — o caso extremo fica para a **migração GPU** do accumulate
(decisão do Enio: sem mitigações CPU de spacing/pincel/camada).

### Problema B — Artefatos (a causa que enganou por ~5 rounds)

A **descrição do sintoma evoluiu** e cada reformulação reposicionou a causa:

1. *"Listras retangulares ao desenhar."* → suspeita: upload parcial de GPU (`preview_upload_bbox`).
2. *"Persiste com `PH2D_PAINT_FULL_UPLOAD=1`."* → upload parcial **descartado**; reclassifiquei como **tearing
   por perf** (§3-D) — **errado** (tearing seria persistente, não "primeiras vezes").
3. *"Fatias da forma, transientes, só nas primeiras vezes; depois nunca mais."* (mockup do Enio) → re-suspeita:
   base stale no cache `composited` CPU. Implementei `reseed_preview_base` (full recompose no início de cada
   sessão de forma). **Não resolveu.**
4. *"Existe um **retângulo virtual** onde o traço é feito; ele sofre o artefato só na **PRIMEIRA vez** que aquela
   região é desenhada na sprite; depois fica limpo pra sempre, mesmo redesenhando."* (a observação decisiva).

A observação 4 é a assinatura inequívoca de **leitura de memória GPU não-inicializada**: garbage até a região ser
escrita a 1ª vez; uma vez escrita, válida para sempre. E **imune ao FULL_UPLOAD e ao reseed** porque ambos mexem
em buffers **CPU já semeados** — e se o stack é GPU-elegível, o `gpu_owns_preview` **desliga o caminho CPU inteiro**.

### Causa-raiz (a verdadeira) + a saga do falso-negativo

A assinatura é **leitura de memória GPU não-inicializada** (retângulo virtual; garbage só na 1ª vez que a região é
desenhada; limpo pra sempre depois — e **não-determinístico**: memória não-inicializada às vezes calha de ser
transparente/preta, às vezes lixo visível). Trace exaustivo: **todos** os buffers semeados **EXCETO um** — o slot do
[`IndividualTextureStore`](../../crates/ph2d-render/src/individual.rs) (a textura que o sprite amostra via
`PreviewOverride`) era criado em `create_entry_empty` **sem clear** (texturas wgpu nascem com lixo). O caminho
GPU-preview adquire esse slot **vazio** (`acquire_empty`) e o preenche por **cópia de região** depois → uma região
amostrada antes da 1ª cópia lê garbage. Retângulo = a região; primeira-vez = antes do 1º write; limpo-pra-sempre = a
textura persiste escrita.

**O falso-negativo que custou 3 rounds.** O clear-on-alloc do slot foi a 1ª hipótese certa — mas o teste do Enio logo
após disse *"alarme falso, ainda existe"*, o que me fez **descartar** a hipótese e caçar `out`/premul (que verifiquei
limpos) e reprodução runtime. **Era um binário stale**: o `play.command` daquele momento rodou um build **sem o clear
compilado** (ou pegou um cache), então o artefato (não-determinístico) ainda aparecia. Num **rebuild limpo** (`play.command`
sem env, depois do ship), o clear-on-alloc está ativo e o artefato **não voltou em vários testes**.

### Tentativas / a ordem real (incluindo o falso-negativo)

| # | Passo | Resultado |
|---|---|---|
| 1 | `PH2D_PAINT_FULL_UPLOAD` (upload full do slot CPU) | Persistiu → não é cobertura do upload; e no stack GPU-elegível o `gpu_owns_preview` desliga o caminho CPU. |
| 2 | Reclassificar como **tearing por perf** (§3-D) | Errado — "primeiras vezes, depois nunca" contradiz tearing (seria persistente). |
| 3 | `reseed_preview_base` (full recompose por sessão de forma) | Re-semeia o `composited` **CPU**; defensivo correto, mas não era o buffer (GPU). |
| 4 | **Clear-on-alloc do slot** (`clear_all_mips_transparent`) — **O FIX** | Falso-negativo (binário stale) me fez achar que falhou → descartei. |
| 5 | Verificar `out`/premul (shaders) | Limpos (escrevem todo texel) — não eram a fonte. Confirmou que o pipeline todo estava semeado **menos o slot do passo 4**. |
| 6 | Rebuild limpo + re-teste (Enio) | **Artefato resolvido.** O passo 4 era o fix o tempo todo. |

### A solução final (clear-on-alloc)

`clear_all_mips_transparent` ([`texture_clear.rs`](../../crates/ph2d-render/src/texture_clear.rs), chamado em
`individual.rs::create_entry_empty`): render-pass `LoadOp::Clear(TRANSPARENT)` sobre **todos** os níveis de mip (o
sampler trilinear lê qualquer nível e `regen_mips` só roda após o 1º upload — então cada nível precisa nascer limpo,
não só o 0). Custo: uma vez na alocação do slot. Agora qualquer amostragem-antes-do-write mostra **transparente** (e
deterministicamente), não lixo.

### O que NÃO era a causa (red herrings registrados)

- **Upload parcial de GPU / `preview_upload_bbox`** (§3-A): cobertura provada consistente; FULL_UPLOAD descartou.
- **Tearing por perf** (§3-D): contradito pelo "primeiras vezes, depois nunca".
- **Cache `composited` CPU stale / drag-preview restore:** auto-consistentes (trail-freedom verdes).
- **`out`/premul (compositor GPU):** `cs_flat` parte de `acc=vec4(0)` e escreve todo texel; `cs_main` (premul) idem,
  canvas inteiro. Ambos totalmente escritos cada frame — não eram a fonte.

### Verificação

- **Perf (✅):** harness `per_layer_perf` (`#[ignore]`, `--release`) + gate de paridade byte
  `fused_per_layer_accumulate_is_bit_identical_to_sequential`; **3.2–4.5×**.
- **Artefato (✅):** guard `acquire_empty_slot_reads_back_transparent_not_garbage` (slot vazio lê all-zero, antes
  garbage); 6/6 `individual_readback` verdes.
- **Smoke do Enio (2026-06-29):** "Testei várias vezes, o bug/artefato não voltou a aparecer" (`play.command`, rebuild limpo).

### Lições generalizáveis

1. **Verifique um REBUILD LIMPO antes de declarar um fix morto.** O falso-negativo ("ainda existe") foi um binário
   stale — eu **descartei o fix certo** e gastei 3 rounds caçando o buffer errado. Bug não-determinístico + build
   incremental = "ainda aparece" pode ser só o binário antigo. Force o rebuild (toque o crate / `--release` limpo)
   antes de abandonar a hipótese.
2. **"Não mudou" não autoriza reclassificar a causa** — só prova que **aquele** buffer/build estava ok. Vale dobrado
   quando o sintoma é não-determinístico (memória não-inicializada).
3. **Texturas wgpu nascem com lixo.** Toda textura amostrável-antes-do-1º-write-completo precisa de clear-on-alloc;
   limpe **todos** os níveis de mip (o `regen_mips` só roda depois do 1º upload).
4. **"Primeira vez, depois nunca" = leitura não-inicializada.** Escrito-uma-vez-fica-válido aponta direto pra um
   buffer sem clear-on-alloc (foi a pista que cravou o slot).
5. **Meça antes de culpar (perf).** O split de fases (96.5% num kernel) refutou a teoria do handoff em uma medição.
   Ver [feedback_measure_perf_symptom_scale](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_measure_perf_symptom_scale.md).

---

## Como adicionar um bug aqui

Uma seção `## Bug #N — <título>` + linha na tabela do topo. Foque nos bugs cuja **causa enganou** (vários rounds
na pista errada); fix trivial fica só no git. Sempre termine em **lições generalizáveis**.
