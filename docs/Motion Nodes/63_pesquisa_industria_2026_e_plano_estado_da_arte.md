# 63 — Pesquisa profunda da indústria + PLANO: Motion Nodes ao estado da arte

**Data:** 2026-07-24 · **Linha:** `line/motion-nodes` (Modo L) · **Status:** pesquisa FECHADA, plano PROPOSTO (aguarda ordem do Enio por wave)
**Método:** 6 pesquisas paralelas sobre docs oficiais — **Houdini+MOPs** (sidefx.com, wiki MOPS) · **Cavalry 2.7.2** (cavalry.studio/docs) · **C4D MoGraph+Fields** (help.maxon.net) · **Blender Geometry Nodes 4.5** (docs.blender.org + release notes 4.1–4.5 + workshops 2025) · **TouchDesigner/Nuke/Fusion/Substance/Notch** (docs de cada) · **Niagara/Unity VFX Graph/Stardust/Autograph/Rive** — mais verificação direta minha dos fatos decisivos (ex.: Dependency Graph do Cavalry é **totalmente editável** — confirmado na doc oficial).
**Dumps completos** (catálogos e tabelas de parâmetro por app, com URLs): os 6 `referencia_pesquisa_*.md` nesta pasta — este doc é a síntese e o plano; os dumps são a evidência.
**Baseline local medida:** 87 nós reais · **318 params** (média 3,7/nó) · widgets `Slider·IntSlider·Angle·Toggle·Seed·Color·Enum·Text` · 7 categorias de UI · o editor já tem backdrops+paleta, knife, probe+sparkline, smart-connect, readouts inline (`Cook::peek`), véu/marcha/massa, influência, postage stamps, subgrafos, busca no add-menu, waypoints/reroute, params dirigidos (docs 35–62).

---

## §0 — Sumário executivo: as 12 decisões que a pesquisa dita

| # | Decisão | Origem |
|---|---|---|
| **D1** | **O falloff vira uma FAMÍLIA componível** (fontes espaciais + remap por-fonte + nó de combinação com blend modes + modificadores temporais com estado), não um nó de 6 params. O `motion.falloff` atual é o "falloff legado" que a Maxon teve de QUEBRAR no R20 — não repetir a migração deles: construir já componível. A composição é por NÓS (o idioma do nosso grafo, = MOPs), não por pilha-widget (C4D). | C4D Fields (spec §C do dump) · MOPs (`mops_falloff` sustenta ~90 nós) |
| **D2** | **`ParamWidget::Curve` e `::Gradient`** — curva/rampa como TIPO de parâmetro, serializado no canal de **text param** (o padrão canônico p/ não-f32, doc 32; contrato intacto). Destrava: contour do falloff, Graph do stagger, custom wave do oscillator, curvas over-life, remap por curva. Lição Blender: **projetar promovível a parâmetro de subgrafo desde o dia 1** (a limitação "curve não entra em group interface" é dor de anos lá). | Houdini ramps · Blender Float Curve · Cavalry Graph attribute · C4D Contour |
| **D3** | **Cluster de NOISE padrão** (type·amplitude·freq·offset·octaves·lacunarity·roughness·speed/pulse·seed·**loop**) como módulo compartilhado CPU+WGSL, embutido nas forças (amplitude 0 = desligado, o default Houdini) e nos nós de noise. Hoje cada nó tem um subconjunto diferente. | Houdini (o MESMO cluster em POP Force/Wind/Curl/AttribNoise/CHOP Noise) |
| **D4** | **Contrato congelado NÃO se move**: todo canal novo é side-metadata no registry (7º, 8º canal… — precedente 6×) + text channel p/ não-f32. Nenhuma task deste plano bumpa `NodeOp=2/OpResolver=1/NodeManifest=8`. | política da casa, re-validada |
| **D5** | **Evento→spawn filho** (partícula que morre/colide/pulsa EMITE noutro sistema) — o buraco estrutural que Niagara (GPU events), VFX Graph, Stardust (Aux) e Particular (parent/child) têm e nós não. Desenho stateless: o filho é função dos eventos que o cook do pai registra — casa com o emitter-função-do-playhead. | Niagara/VFXG/Stardust |
| **D6** | **Áudio-reativo agora** — `ph2d-audio-spectral` JÁ tem FFT; falta só a ponte (bandas→colunas de valor, via shell bridge — FFT nunca entra no cook). C4D Sound Effector (probes no espectro) é a referência de UX; Cavalry Sound (bandas→índices) a de mapeamento. | C4D · Cavalry · MOPs Audio Falloff |
| **D7** | **Params essenciais NO nó** (1–3 rows por nó, declarados por side-metadata `register_ui`) + **proveniência por-param com cor** (constante/expressão/dirigido — os "4 modos" do TD, storage sem destruição) + gesto **"drive by…"** (menu no param cria e liga `value.lfo`/`expression`/`noise` — o Dynamic Input do Niagara com o NOSSO grafo como verdade). | TD Parameter Dialog · Niagara Dynamic Inputs · Autograph modifier-stack |
| **D8** | **Spreadsheet de colunas + custo inline**: tabela viva instância×coluna (nossas colunas já são colunas — barato) + cook-time por nó pintado no grafo (toggle). O probe mostra DADO; isto mostra TUDO e CUSTO. | Niagara Attribute Spreadsheet · Blender Spreadsheet · Notch perf inline · Houdini MMB info |
| **D9** | **Gizmos de canvas bidirecionais** para falloff/attractor/curvas (arrastar o centro/raio NA TELA). Lição Blender pinada: *a posição do gizmo tem de depender do valor que ele controla*, senão salta ao soltar. Cavalry mostra o falloff como gizmo colorido arrastável — é o gesto que faz o mograph clássico ser 5 gestos. | Blender Gizmo nodes 4.3 · Cavalry · MOPs Preview Falloff |
| **D10** | **Presets serializam o grafo inteiro** — o formato textual v2 JÁ é a serialização; falta o browser + pasta de user presets + **exemplo carregável por nó** (o F1 do Houdini instala a cena de exemplo). É onboarding, marketing e teste de regressão de graça. | Stardust Smart Presets · Houdini galleries/examples |
| **D11** | **O benchmark de UX vira smoke permanente**: o mograph clássico (grid de clones + falloff arrastável + stagger com delay) em **≤5 gestos**, cronometrado como cena de smoke. Cavalry faz em 5; C4D em 2 (auto-wire). Cada wave de UI se mede contra ele. | Cavalry §D · C4D §D |
| **D12** | **Defaults vivos**: nó recém-criado FAZ algo visível (C4D: effector nasce com P.Y=50; Houdini: scatter nasce com 1000 pts). Auditar os 87 na wave de params: nada nasce inerte, `Enabled=off ⇒ neutro-1` (falloff desligado devolve 1, nunca mata a cena — Cavalry). | C4D · Houdini · Cavalry |
| **D13** | **PRODUTO FINAL, não MVP — cada nó carrega o conjunto COMPLETO de params que os apps pro expõem para o SEU tipo**, conferido nos `referencia_pesquisa_*.md` (o catálogo, **não a memória**) ANTES de fechar o nó E ao TOCÁ-LO de novo. Subconjunto mínimo = dívida disfarçada de "v1". Caso que forçou a regra: shipei `field.box` **axis-aligned** e os pros TODOS giram — e o catálogo ainda disse **COMO** (per-field + gizmo, nunca nó de transform), corrigindo os dois reflexos de uma vez. Detalhe: §0.1. | Enio 2026-07-24 |

**O que já é nosso e ninguém tem** (o plano AMPLIA, não alcança): deformers/grade espacial/scan/reduce **100% GPU-resident** (nenhum app 2D tem; Cavalry e C4D são CPU) · scrub **bit-exato** GGPO · emitter **stateless** (função pura do playhead — o dos outros re-simula) · véu/marcha/massa + influência (inteligibilidade que nem Houdini pinta) · determinismo cross-OS gateado · LLM co-autora (MCP) no horizonte do design v1.

---

## §0.1 — A regra de processo (D13): revisar os params PRO a cada nó

**Não é o MVP; é o produto final.** Toda task de nó — **nova OU revisão** — abre por:
1. Abrir o `referencia_pesquisa_*.md` dos apps que têm aquele tipo de nó e **LISTAR** os params que expõem (o catálogo verbatim, não a lembrança).
2. Implementar o **superset** — ou, se um param for deliberadamente fatorado noutro nó / diferido, **escrever por quê** no doc-comment do nó (senão vira o buraco da rotação: "esqueci" vestido de "v1").
3. Conferir **Coordinates** (position/**rotation**/scale), o neutro byte-idêntico (D12) e a rota GPU.

**O alvo da família `field.*` — a decomposição, ancorada no catálogo (não em palpite).** Cada *field object* pro tem as abas **Coordinates · Field · Remapping · Color Remap · Direction**, e — verificado nos três dumps (D13) — **C4D, Cavalry E Houdini/MOPs põem o transform (position+rotation+scale) NO campo, com um GIZMO de canvas**, nunca num nó de transform separado (nó-de-transform é idioma TouchDesigner/DAG-geral, **não** mograph). **Enio confirmou per-field + gizmo (2026-07-24).** Então, no nosso idioma de nós:
- **Coordinates** = params do próprio field: `center_x/y` + **`rotation`** (✓ box; **todo spatial field ganha os dois**) + size. O **gizmo de canvas (D9)** é a UI ergonômica — arrastar a alça mexe NESSES params, e é a resposta ao *"não quero caçar a rotação"*. **PRIORIZADO.**
- **Field** = os params de forma (Box: width/height/soft; Radial: setor + repetições + ângulos; Linear: comprimento/direção).
- **Remapping** (Inner Offset/Contour/Min-Max/Clamp/Invert) = o nó **`field.remap`** a jusante — fatorado de propósito (D1: composição por nós, não aba embutida). **Documentar essa fatoração no doc-comment de cada field**, senão parece gap.
- **Color Remap / Direction** (os 3 canais value/color/direction) = **diferido** (§6.3; v1 = só o canal value/`falloff`).
- ⚠️ **`index_range` NÃO tem Coordinates** — mascara por **RANK** (`i/(n-1)`), não por posição; rotação/posição não têm sentido num posto. É **categoria** (spatial vs rank/data), **não gap**.

**Estado (2026-07-24):** 3 nós da família landaram — `field.index_range` (rank) · `field.box` (espacial, **com rotation**) · `field.combine` (composer) — todos GPU-resident **bit-exatos** (paridade `0e0` a 25.6k na RTX), smokes `=17/=18/=19`. Próximo, pela D13: o **gizmo dos fields** (D9, priorizado) · `field.radial_sweep` (já nasce com Coordinates) · `field.remap` + `ParamWidget::Curve`.

---

## §1 — Onde estamos (a régua honesta)

87 nós / 318 params. A UI do grafo já cobre 6 das 8 camadas de inteligibilidade do design v1 (docs 35–62). Os vãos estruturais, vistos da indústria:

1. **Falloff raso** — 1 nó, 1 forma, 6 params, 1 canal (escalar). A indústria compõe campos como camadas, com remap por fonte e 3 canais (valor/cor/direção).
2. **Params rasos nos nós certos** — as forças têm 1–7 params onde a referência tem 10–15 (o cluster de noise é o grosso da diferença); emitter sem shape/burst/inherit; oscillator sem BPM/custom-wave; stagger sem Graph.
3. **Zero pontes** — áudio (módulo inteiro existe!), eventos de colisão→ação, física ECS↔grafo, dados externos (CSV), texto.
4. **Zero widgets de curva/gradiente** — o tipo de parâmetro mais usado da categoria.
5. **Param exposto NO nó: nenhum** — tudo vive no painel lateral.
6. **Sem evento→spawn filho, sem repeat-N, sem gather entre instâncias** (`evaluate_at_index`/`sample_index`).
7. **Sem presets/exemplos** por nó.

---

## §2 — ASPECTO 1: catálogo de nós que FALTAM (por família, com origem e prioridade)

P0 = diferencial imediato de motion design · P1 = completa a família · P2 = valioso, decisão/dependência própria. ✕ = cross-line (fora desta linha, nomeado em §8). Detalhe de params de cada um: dumps `referencia_pesquisa_*.md`.

### 2.1 Campo / Falloff (a família nova — D1)

| Nó novo | O quê | Origem | Pri |
|---|---|---|---|
| `field.linear` | rampa ao longo de um eixo (comprimento+direção) | C4D Linear · MOPs Shape | **P0** |
| `field.radial_sweep` | setor ANGULAR (radar; start/end angle + repetições) | Cavalry Sweep · C4D Radial | **P0** |
| `field.box` | retângulo com falloff por borda | todos | **P0** |
| `field.index_range` | falloff por FAIXA DE ÍNDICES (não-espacial) | Cavalry Range Falloff | **P0** |
| `field.spline` | distância a curva OU posição-ao-longo → peso | MOPs Spline Falloff | **P0** |
| `field.noise` | o noise como fonte de campo (cluster D3) | MOPs Noise Falloff · C4D Random Field | **P0** |
| `field.combine` | composita 2 campos com blend modes (add/sub/mul/screen/min/max/overlay/normal) + strength | MOPs Combine · C4D layer blending | **P0** — a peça-chave |
| `field.remap` | inner_offset (platô) · contour (linear/quad/step/quantize/**curva**) · min/max · clamp · invert · **probability+seed** (campo→máscara binária) | C4D Remapping (verbatim no dump) · Cavalry Probability | **P0** |
| `field.delay` | o campo chega com atraso/mola/decay — modificador TEMPORAL com estado | C4D Delay/Decay layers · MOPs | **P1** (estado: infra checkpoint existe) |
| `field.freeze` | congela o campo no pulso (sample&hold espacial) | C4D Freeze layer | **P1** |
| `field.spread` | "infecção": cresce de sementes por vizinhança (frente animável + largura) | MOPs Spread Falloff | **P1** — assinatura mograph |
| `field.shape` | forma vetorial como campo (dentro=1 / borda decai) | Cavalry Shape falloff · C4D Volume | **P1** (consome vetor) |

### 2.2 Forças e simulação

| Nó novo | O quê | Origem | Pri |
|---|---|---|---|
| `force.gravity` | preset −g massa-aware (hoje se finge com wind) | Niagara/VFXG | **P0** |
| `force.speed_limit` | clamp min/max de \|v\| — o estabilizador universal | POP Speed Limit · Niagara Limit | **P0** (barato) |
| `force.curve` | o "rio": tangente + atração + espiral ao longo de uma curva | POP Curve Force | **P0** |
| `force.line_attract` | atração ao ponto mais próximo de um segmento | Niagara Line Attraction | **P1** |
| `force.follow` | perseguir/fugir de um alvo com antecipação | POP Follow · Attract/Predict | **P1** |
| `force.conform` | sucção+aderência a forma/curva (attract+stick) | VFXG Conform to SDF | **P1** |
| `force.vector_field` | campo vetorial de imagem/campo dirigindo força (flow field) | VFXG · Cavalry Flow Field · MOPs Velocity Field | **P1** |
| `sim.kill_zone` | mata dentro/fora de forma (invert) | Niagara/VFXG Kill | **P1** |
| `sim.replicate` | **evento→spawn filho** (D5): morte/colisão/pulso do pai emite no filho, com payload | Niagara GPU events · Stardust Aux | **P0** (desenho stateless próprio) |
| `sim.collision_pulse` | colisão da sim → pulso/atributo (ponte p/ `pulse.*`: cor no impacto, impulso, sticky) | Cavalry Collision Events | **P1** |
| `motion.push_apart` | de-overlap determinístico com modos **Push/Scale/Hide** | C4D Push Apart · MOPs Relax | **P1** (collide cobre o modo físico) |
| `motion.smooth` | relax/suavização de posições/atributo (iterações+força) | SOP Smooth · CHOP Filter | **P0** |
| `motion.lag` | lag+overshoot com clamp de slope/aceleração (irmão barato do spring) | CHOP Lag | **P0** |

### 2.3 Spawn / emissão

| Nó novo / param | O quê | Origem | Pri |
|---|---|---|---|
| emitter: **shape** | point/circle/rect/**curva/perímetro de forma** + inward/outward | Cavalry · Niagara Location | **P0** (param, não nó) |
| emitter: **burst** | N no tempo T + periódico + probability | Niagara Spawn Burst | **P0** |
| `motion.spawn_per_unit` | emite por DISTÂNCIA percorrida (com clamp anti-teleporte) | Niagara Spawn Per Unit | **P1** |
| emitter: **inherit velocity** | herda velocidade do emissor (função do playhead — nosso stateless torna exato) | Niagara/Cavalry | **P1** |

### 2.4 Valor / dados / ordem

| Nó novo | O quê | Origem | Pri |
|---|---|---|---|
| `value.random` | nó dedicado: Float/Int/**Bool(probability)**, min/max, ID+Seed separados | Blender Random Value · Cavalry Random | **P0** |
| `value.curve` | Float Curve — mapeia valor por CURVA (widget D2) + Factor | Blender · Cavalry Graph | **P0** |
| `value.gather` | `evaluate_at_index` — lê coluna de OUTRO elemento (lookup por índice) | Blender Evaluate at Index | **P0** |
| `value.sample_stream` | lê coluna de OUTRA stream por índice/proximidade (inter-stream) | Blender Sample Index/Nearest | **P1** |
| `value.accumulator` | acumula no tempo (Σ v·dt) | Cavalry Accumulator · C4D Time effector | **P1** |
| `value.index_switch` | N entradas por inteiro | Blender 4.1 | **P1** |
| `value.stat` | reduce completo: mean/median/**stddev/variance**/range + **Selection** (o nosso `Sum/Min/Max` GPU ganha os irmãos) | Blender Attribute Statistic | **P1** |
| scan: **Group Index** | prefix-sum SEGMENTADO (bins) + saídas Leading/Trailing/Total | Blender Accumulate Field | **P1** (refina o scan GPU) |
| `pulse.adsr` | trigger com envelope Delay/Attack/Sustain/Release | CHOP Trigger | **P1** |
| `value.int_math` / `bool_math` | famílias dedicadas (hash já temos) | Blender 4.3/4.5 | **P2** |

### 2.5 Distribuições / geometria de instâncias

| Nó novo | O quê | Origem | Pri |
|---|---|---|---|
| `motion.connect` | **plexus**: liga pontos por raio/nearest/índice com limites por ponto — a estrela visual da categoria | Cavalry Connect Shape (params no dump) | **P0** |
| `motion.distribute_honeycomb` | grade deslocada (tijolo/colmeia) com form square/circle | C4D Honeycomb | **P1** |
| `motion.distribute_rose` | curva rosa paramétrica | Cavalry Rose | **P2** |
| distribution: **mask/intersections** | remove pontos fora de forma · pontos nas interseções de paths | Cavalry | **P1** (mask consome vetor) |
| `motion.shuffle` | embaralha IDs (distribution modifier) | Cavalry Shuffle | **P1** |
| `motion.points_to_curves` | agrupa instâncias em curvas por Group ID + ordena por peso (trails de partículas viram GEOMETRIA) | Blender | **P1** |
| `motion.clone` v2 | multi-fonte (**iterate/random/blend/sort**) + **time offset por clone** + step cumulativo | C4D Cloner (o time offset é o retiming canônico do Duplicator) | **P0** (upgrade, não nó novo) |

### 2.6 Tempo / retiming

| Nó novo | O quê | Origem | Pri |
|---|---|---|---|
| `motion.time_offset` | desloca o RELÓGIO por instância, dirigível por campo/índice (o par Stagger→Shape Time Offset do Cavalry; MOPs Delay = falloff espacial vira onda temporal) | Cavalry · MOPs · C4D Time Offset | **P0** — o retiming como material |
| `motion.inheritance` | clones assumem pose/ANIMAÇÃO de um objeto líder, defasada por índice | C4D Inheritance | **P2** |
| `motion.sequencer` | liga/desliga instâncias em sequência temporal (visibility sequence) | Cavalry | **P1** (step/strobe cobrem metade) |

### 2.7 Aparência / estilo (dentro do grafo, sobre instâncias)

| Nó novo | O quê | Origem | Pri |
|---|---|---|---|
| `motion.squash_stretch` | S&S automático por velocidade (o clássico de animação — barato e amado) | Cavalry Squash and Stretch | **P0** |
| `motion.motion_stretch` | esticão na direção do movimento | Cavalry Motion Stretch | **P1** |
| `motion.color_ramp` v2 | gradiente com N stops via `ParamWidget::Gradient` (hoje: 2 cores + preset) | todos | **P0** (upgrade) |
| `motion.number_to_color` | valor→cor por gradiente (separado do ramp por posição) | Cavalry | **P1** |
| `motion.skew` / `pinch` | deformers que faltam na família GPU | Cavalry | **P1** |
| `motion.ripple` | ondas concêntricas decaindo (wave é grade; ripple é radial sobre instâncias) | SOP Ripple | **P1** |

### 2.8 Áudio (D6)

| Nó novo | O quê | Origem | Pri |
|---|---|---|---|
| `audio.bands` | N bandas de frequência → colunas de valor (freq scale log/mel · smoothing · A-weighting · **Use Index Context**: banda i → instância i) | Cavalry Sound (params no dump) | **P0** |
| `audio.probe` | UX C4D: retângulos desenhados no espectro (freq×loudness) → valores; sampling Peak/Average/**Step** (o medidor de LED) | C4D Sound Effector | **P1** |
| `field.audio` | bandas como CAMPO (Audio Falloff) | MOPs | **P1** |

**Contagem do plano:** ~35 nós P0/P1 novos + 6 upgrades de nós existentes ⇒ ~125+ nós ao fim, com as famílias que definem a categoria completas — sem contar as frentes cross-line (§8).

---

## §3 — ASPECTO 2: parâmetros — o gap nó-a-nó dos 87 EXISTENTES

### 3.1 Clusters padrão (definir UMA vez, reusar em todo nó — a lição estrutural)

| Cluster | Campos | Quem usa |
|---|---|---|
| **NOISE** (D3) | `type` (perlin/simplex/worley-F1/F2−F1/value/cellular…) · `amplitude` · `frequency`+`scale_xy` · `offset_xy` · `octaves` · `lacunarity` · `roughness` · `speed`/`pulse` · `seed` · **`loop_period`** (loop perfeito — Cavalry Looping) | noise, wiggle, forças (embutido, amp=0 default), field.noise |
| **REMAP** | `inner_offset` (platô) · `contour` (linear/quad/step/quantize/curve) · `min/max` · `clamp` on/off · `invert` · `probability+seed` | field.*, falloff, stagger, luminance |
| **RANGE** | `min/max` + modos (positive/zero-centered/min-max) em vez de só `amplitude` | oscillator, noise, random |
| **STRENGTH/weight** | todo modificador multiplicável por campo de peso (a coluna-contrato de §6.2) | todos os "effectors" |
| **SEED** | int + re-roll (✓ temos widget) + **`use_node_as_seed`** (evita gêmeos ao duplicar — Cavalry Use Layer as Seed) | todo estocástico |
| **TIME local** | `speed`·`offset` por nó (conveniência; `time_remap` segue sendo o escopo por sub-árvore) | temporais |

### 3.2 Gap por nó existente (os que a referência expõe mais fundo)

| Nó (params hoje) | ADICIONAR (da referência) |
|---|---|
| `force.wind` (angle·strength·gust·gust_freq·seed) | **modelo alvo-velocidade** (`airresist·(targetv−v)` — não explode; POP Wind) · cluster NOISE completo · per-axis |
| `force.drag` (coefficient) | `directional_scale` (anisotrópico) · `goal_velocity` (arrasta rumo a v≠0) · `rotational_drag` (se rotação ganhar velocidade angular) |
| `force.attractor` (target·strength·radius·curve·repel) | `kill_radius` (mata ao chegar) · `reversal_distance` (inverte perto — não orbita) · `peak_force_distance` (platô) · modos **Follow/Predict Intercept** · janela min/max · `swirl` |
| `force.curl` (strength·scale·speed·octaves·seed) | `lacunarity`·`roughness`·`offset`·`pulse` (cluster) · *collision avoidance* (contorna `sim.collide` — P2, caro) |
| `force.vortex` (center·strength·radius·clockwise) | **`lift` + `suction`** (o Axis Force do Houdini unifica órbita+elevação+sucção com UM falloff) · `soft_edge` · `inner/outer_strength` · `treat_as_wind` |
| `motion.noise` (channel·amplitude·scale·octaves·roughness·type·speed·seed) | `lacunarity` · `offset` · modos de RANGE · **`loop_period`** · transform do campo (pos/rot/scale) · controles cellular (jitter, métrica euclid/manhattan) · `stagger` (defasa por índice) |
| `motion.oscillator` (channel·wave·amplitude·frequency·phase_stagger·offset·phase) | **Time Mode: Seconds/BPM** · **custom wave por CURVA** (D2) · min/max (RANGE) · `strength_fade_to_zero` (entra do zero, não do min) |
| `motion.stagger` (channel·min·max·ease_curve·ease_dir·reverse) | **Graph (curva D2)** no lugar do ease enumerado · `offset` · auto-invert min>max |
| `motion.wiggle` (channel·amplitude·frequency·seed) | `octaves` · `loop_period` · separate channels |
| `motion.spring` (channel·tension·friction) | modos **Average/Blend/Spring** (o Delay do C4D generalizado: qualquer propriedade chega com atraso/mola) · input effect Position/Force (CHOP Spring) |
| `motion.emitter` (rate·life·speed·angle·spread·x·y·seed·max·size) | **shape** (point/circle/rect/curva/perímetro) · direction inward/outward · **burst** (count/time/period/probability) · `probability` · `inherit_velocity` · duration/interval · emissão por distância (§2.3) |
| `motion.falloff` (shape·curve·center·radius·invert) | → dissolve na família `field.*` (D1); o nó atual vira alias/compat |
| `motion.clone` (count·distance·angle·center) | multi-fonte iterate/random/blend/sort · **time_offset por clone** · step cumulativo (espiral) |
| `motion.trail` (length·fade·shrink) | limite por tempo · path type (pontas/linha contínua — ribbon) · time offset |
| `value.map_range` (in/out·clamp) | **interpolation**: linear/stepped(+steps)/smoothstep/smootherstep (Blender) · curva (D2) |
| `motion.grid` (rows·cols·gap) | `form` (círculo/forma recorta a grade) · `fill` (casca) — C4D Grid Array |
| `motion.look_at` (offset) | `strength` (peso 0..1) · up/flip |
| `motion.sort` (key·descending·center·seed) | `group_id` (ordena DENTRO de grupos) · peso-como-coluna (Sort Elements) |
| `motion.mixer` (mode) | peso POR entrada |
| `motion.color_ramp` (2 stops) | N stops via Gradient (D2) · interpolações por stop |
| `sim.spawn` (rate·scatter·seed) | burst · probability (espelha o emitter) |
| `motion.distribute_poisson` (radius·width·height·seed) | `density` como campo (Blender Density Max × field) |

**Auditoria D12 junto:** para cada nó da tabela, conferir o DEFAULT (nada nasce inerte) e o neutro (`enabled=off`/strength=0 ⇒ identidade exata, byte-idêntica — a lei que a rack de áudio já pratica).

---

## §4 — ASPECTO 3: a decoração do nó e a UI (o que temos ✓ · o que roubar)

### 4.1 Anatomia-alvo do nó (spec)

```
╭─ ▓▓ Noise ──────────────────── 3.2µs ─╮   header: cor da categoria ✓ + nome ✓ + cook-time (novo, toggle)
│ ◇ in                          out ◇  │   sockets: ◇ = stream/coluna · ○ = valor uniforme (novo: FORMA por tipo)
│ amplitude  1.00 ━━●━━      [≈]       │   1–3 params ESSENCIAIS no nó (novo, D7) — slider mini + chip
│ scale      0.40 ━●━━━      [·]       │   [·]=constante [≈]=dirigido/expressão (proveniência com cor, D7)
│ ▸ Fractal · ▸ Time                   │   painéis colapsáveis DENTRO do nó (Blender node panels)
│ Readout: 96 inst · media 0.42 ✓      │   readouts inline ✓ (Cook::peek, doc 43)
│ ⣿ postage stamp ✓ (política de custo)│   stamp ✓ — ganhar modo static-frame (lição Nuke)
╰── [B][S][V] ──────────────────────────╯   pips de borda (novo): Bypass · Solo · Viewer (TD flags)
```

### 4.2 Tabela roubar/temos (com custo; ranking cruzado dos 6 dumps)

| Padrão | Origem | Status | Custo |
|---|---|---|---|
| Header cor por categoria · véu inerte · marcha (dashes) · massa · influência · probe+sparkline · readouts · stamps · backdrops+paleta · reroute/waypoints · knife · busca no add · subgrafos · smart-connect | — | **✓ temos** (docs 35–62) | — |
| **Params essenciais no nó** (1–3 rows, declarados no `register_ui`) | Cavalry DG rows · Blender | FALTA | Médio — **maior alavanca visível** |
| **Proveniência por-param com cor + storage sem destruição** (constante/expressão/dirigido) | TD 4 modos | mecanismo ✓ (params dirigidos doc 58, expression) — falta a UI | Médio |
| **Gesto "drive by…"** no param (cria+liga lfo/expression/random) | Niagara Dynamic Inputs | FALTA (1 clique sobre mecanismo existente) | Baixo |
| **Value ladder** (MMB: degraus 0.001→100, grosso+fino num gesto) | Houdini/TD/Nuke | FALTA | Baixo |
| **Forma de socket** = coluna (◇) vs valor uniforme (○); link ilegal = vermelho | Blender | FALTA (temos cor por tipo?) — conferir `paint_wire` | Baixo |
| **Pips de borda** Bypass/Solo/Viewer clicáveis (alvos escalam com zoom — anti-padrão TD anotado) | TD flags | FALTA | Médio |
| **Cook-time inline** sob o nome (toggle de profiling) | Notch · Houdini MMB | FALTA (Cook já mede — doc 53 mediu fps) | Baixo |
| **Drop-no-fio com highlight** + **shake-to-disconnect** (fio se cura) | Nuke | FALTA (knife ✓) | Baixo/Médio |
| **Criação com modificadores**: Shift=branch · Ctrl=replace · `A`,Enter=repete último | Nuke | FALTA (busca ✓) | Baixo |
| **Ctrl+F find-navega** (mesma busca do add, apontada aos nós existentes) | Notch | FALTA | Baixo |
| **Zone strap** — a "cinta" visual do par `sim.zone` in/out abraçando o miolo | Blender zones | PARCIAL (sim.zone existe; a cinta é desenho) | Baixo |
| **Inspection Index** — qual passo/iteração o probe mostra dentro de zona | Blender | FALTA | Baixo |
| **Spreadsheet de colunas** (painel: instância×coluna, filtros, estado cru/cozido) | Niagara/Blender | FALTA (D8) | Médio |
| **Static-frame stamps + toggle global** (política de custo de preview) | Nuke (a lição: stamps vivos matam o script) | FALTA | Baixo |
| **Gizmo de canvas bidirecional** (falloff/attractor/curvas arrastáveis NA TELA) | Blender 4.3 · Cavalry | FALTA (Camada 6 do design v1 previa overlays; o gizmo é o passo além: ESCREVE) | Médio/Alto |
| **Heatmap por instância** do campo selecionado (azul→vermelho no canvas) | C4D Fields Color · Camada 4 design v1 | FALTA | Baixo (1 tint por coluna) |
| **Expose de subgrafo com dialog** (widget/range/grupo/`visible_if`/batch) | Substance | FALTA (subgrafo ✓) | Alto — a fundação de assets |
| **Menu/enum autorável** em subgrafo (Menu Switch) | Blender 4.1 | FALTA | Médio |
| **Minimap opt-in domável** | Notch/Fusion | FALTA | Médio |
| **Warning node + propagação** (subgrafo avisa pra cima, erro borbulha) | Blender 4.3 · Houdini | PARCIAL (erros por nó?) | Médio |

### 4.3 Anti-padrões pinados (dos dumps — NÃO repetir)

1. **Stamps/preview sem política de custo** (Nuke, Substance): todo preview no nó nasce com modo estático + toggle global.
2. **Min/max que não clampa o dado** (Blender): se a UI diz max, ou clampa ou o rótulo diz "soft" — nunca surpresa silenciosa (nosso `ParamHardMax` já distingue; manter a lei).
3. **Toggle de painel que não gateia os knobs** (Blender Panel Toggle): painel desligado ⇒ knobs inertes DE VERDADE (a lei do knob-morto da casa já cobra).
4. **Controle destrutivo no corpo do nó** (Blender escondeu o Skip): pausar/limpar sim não é checkbox clicável por acidente.
5. **Prioridade numérica manual** (XPresso −499..499): nosso cook topológico é superior — jamais importar.
6. **Duas semânticas no mesmo desenho de fio** (Notch): se um dia houver fio de hierarquia vs dado, inconfundíveis.
7. **Contrato implícito de ordem de elementos** (Blender criou Geometry Randomization pra CAÇAR isso): nossos gates de determinismo já cobrem — manter id-estável, nunca índice-dependente.
8. **Escape-hatch lento sem aviso** (Blender For-Each): se um contêiner por-elemento nascer, o custo vem escrito no nó.

---

## §5 — ASPECTO 4: intuitivo e poderoso para artistas

1. **O benchmark dos 5 gestos (D11)** — cena de smoke permanente: retângulo → clone/grid → falloff arrastável na TELA → stagger→time_offset. Meta: ≤5 gestos, zero digitação. Cada wave de UI re-roda o benchmark.
2. **Auto-wire ao criar** (C4D: effector nasce conectado ao cloner selecionado): criar nó com um nó selecionado já liga (✓ smart-connect?) — estender: criar um `field.*` com um modificador selecionado liga no peso dele.
3. **Defaults vivos (D12)** + **neutro exato**: strength=0/off = byte-idêntico (gate por nó, o padrão da rack de áudio).
4. **Presets & exemplos (D10)**: browser de presets (serialização v2) + **exemplo por nó** (help do nó instala cena — Houdini F1) + tooltips por param (i18n, já temos infra HR-15).
5. **Retiming como material**: `motion.time_offset` dirigível por campo/índice — o "delay em cascata" vira um fio, não uma expressão.
6. **Scrub everywhere**: value ladder + arrastar no chip (✓ chips) + gizmos de canvas (D9).
7. **Ver é entender**: heatmap de campo · spreadsheet · cook-time · véu/influência (✓) — o grafo confessa dado, custo e alcance.
8. **Rig = params promovidos** (Cavalry Control Centre): o expose de subgrafo (§4.2) + painel "controles do grafo" com os params marcados — o artista rigga sem bones.
9. **Onboarding**: os 3 tutoriais clássicos (grid+falloff · áudio-reativo · plexus) como cenas de exemplo carregáveis.

---

## §6 — Consequências de arquitetura (antes das waves)

1. **Contrato (D4):** zero mudança em `NodeOp/OpResolver/NodeManifest`. Novos canais de side-metadata prováveis: `register_ui` ganha *params essenciais no nó* (lista de até 3) e *gizmo spec*; `KernelResolver` ganha o que os kernels novos pedirem (7º canal…). Curve/Gradient = **text params** serializados (formato: lista de stops/pontos com interp — versionado no próprio texto, como o header v2).
2. **A coluna de PESO como contrato** (o `mops_falloff` nosso): os `field.*` escrevem numa coluna transiente `weight` (irmã da `accel`); todo modificador multiplica seu efeito por ela quando conectado (`STRENGTH` cluster). Efeito = `lerp(identidade, efeito, weight)` — a regra MOPs, gateada por byte-identidade em weight=1/0.
3. **Fields multi-canal (C4D §C):** v1 = canal ÚNICO (peso). Cor e direção ficam nomeados como extensão — direção conversa com `accel` (um campo alimentando força), cor com `tint`. Não construir os 3 de uma vez; deixar o desenho aberto.
4. **Estado em modificador de campo** (`field.delay/freeze`): são Temporais — usam o MESMO desenho de sim (função do playhead onde possível; onde não, o par checkpoint/ring já existe). Nada de estado fora do modelo.
5. **GPU:** todo nó novo declara a rota — forças/fields/deformers = WGSL pelos canais existentes (map/reduce/scan/grade); `field.combine/remap` são map puros; `connect` usa a grade espacial (vizinhos) já portada; o censo `motion_gpu_coverage` cobra a frontier (e ganha os docs novos — ele já foi cego uma vez).
6. **Áudio (D6):** FFT NUNCA entra no cook. A shell bridge (padrão painter/physics) computa bandas por frame via `ph2d-audio-spectral` e publica como INPUT do grafo (colunas/valores) — determinismo: bandas são função do arquivo+playhead ⇒ scrub-exato.
7. **Determinismo (HR-5):** cluster de noise transcendental-free onde a política manda (hash-based value/simplex; worley por grade — JFA já existe); `libm` pinado é o precedente para o resto.
8. **Perf:** o benchmark de 5 gestos ganha uma variante de CARGA (10k instâncias + 3 fields + 4 forças) com orçamento medido antes de cada wave fechar (§0.0 do CLAUDE.md: medir antes de limitar).

---

## §7 — O PLANO: waves com tasks

> Ordem recomendada: **W-A → W-B → W-C** (linguagem → campo → forças = os multiplicadores), depois **W-I** (UI do nó — pode intercalar), **W-D/E/F** (eventos/dados/distribuições), **W-G/H** (tempo/áudio), **W-J** (assets). Cada wave fecha com: gate batched + smoke próprio + entrada no handoff. Nada aqui integra sozinho (regra da linha). **E cada TASK de nó ABRE com a revisão de params PRO (D13/§0.1) — produto final, não MVP: o superset do que os apps pro expõem, conferido no catálogo.**

### W-A — A LINGUAGEM DOS PARAMS (fundação, tudo consome)
| Task | Entregável | Aceitação |
|---|---|---|
| A1 | `ParamWidget::Curve` — widget de curva (pontos+interp por ponto) no painel de params, valor serializado em text param; API de avaliação `curve_eval(t)` CPU+WGSL (LUT baked p/ GPU) | red-first: nó de teste com curva muda o cozido; round-trip serialização byte-exato; mutação no eval sangra |
| A2 | `ParamWidget::Gradient` — irmão de cor (stops RGBA + interp), mesmo canal | idem; `color_ramp` v2 consome (§3.2) |
| A3 | **Value ladder** (MMB nos chips numéricos: degraus 0.001→100) | seam test dirigindo o gesto real; funciona em TODO chip numérico do app (widget compartilhado — beneficia os outros painéis) |
| A4 | **Proveniência por-param**: indicador colorido na row (constante/dirigido/expressão) lendo o estado REAL (params dirigidos doc 58 + text params) | gate presença+ausência; dirigir um param muda a cor sem clicar |
| A5 | **"Drive by…"**: menu no param → cria `value.lfo`/`expression`/`value.random` já ligado ao param | seam: clique real cria nó + edge + o param mostra proveniência; undo = 1 passo |
| A6 | Auditoria **D12** dos 87: defaults vivos + neutro byte-idêntico (tabela §3.2) | 1 gate paramétrico por família (strength=0 ⇒ stream intacta ao byte) |

### W-B — FIELDS: o falloff que compõe (D1)
| Task | Entregável | Aceitação |
|---|---|---|
| B1 | Coluna-contrato `weight` + cluster STRENGTH: modificadores multiplicam por ela (`lerp(id, efeito, w)`) | w=1 byte-idêntico ao hoje; w=0 = identidade; mutação (ignorar w) sangra |
| B2 | Família de fontes: `field.linear/box/radial_sweep/index_range/noise` (+ o circular herdado do falloff atual, que vira alias) | golden por forma; GPU map por kernel; gizmo overlay (B6) |
| B3 | `field.remap` (REMAP cluster completo, contour por **Curve** de A1, probability+seed) | quantize/step/curve golden; probability determinística por id |
| B4 | `field.combine` (blend modes + strength) | tabela de modos vs referência (C4D verbatim no dump); comutatividade onde deve |
| B5 | `field.spline` + `field.shape` (consomem curva/forma — porta template como o kaleidoscope) | distância assinada correta nos 2 modos (borda/dentro) |
| B6 | **Gizmo de canvas** do field selecionado (centro/raio/eixo arrastáveis; lição D9: posição deriva do valor) + **heatmap por instância** (toggle no nó) | seam de canvas: arrastar o gizmo escreve o param; heatmap = cena de smoke |
| B7 | `field.delay/freeze/spread` (os temporais/estado) | scrub bit-exato via infra checkpoint; freeze == sample_hold espacial provado |

### W-C — FORÇAS COMPLETAS (D3 + §3.2)
| Task | Entregável |
|---|---|
| C1 | Módulo `noise_cluster` compartilhado (CPU+WGSL) + adoção em `force.wind/curl` e `motion.noise/wiggle` (params novos com defaults neutros — grafos salvos intactos) |
| C2 | Upgrades §3.2: attractor (kill/reversal/peak/janela) · vortex (lift/suction/soft_edge) · drag (anisotrópico/goal) · wind (modelo alvo-velocidade como MODO, default preservado) |
| C3 | Nós novos: `force.gravity` · `force.speed_limit` · `motion.lag` · `motion.smooth` |
| C4 | `force.curve` (o rio) + `force.line_attract` + `force.follow` |
| C5 | `motion.squash_stretch` (+ `motion_stretch`) — leem velocidade, escrevem scale/rot |
| Aceitação | cada força: golden CPU + paridade GPU (canal existente) + neutro exato; perf da cena de carga re-medida |

### W-D — SPAWN & EVENTOS (D5)
| Task | Entregável |
|---|---|
| D1 | Emitter v2: shape (point/circle/rect/curva/perímetro) + burst + probability + inherit_velocity — mantendo **stateless** (tudo função do playhead) |
| D2 | `sim.replicate` — evento→spawn filho com payload (desenho próprio: filho = função dos eventos do cook do pai; nota-ADR dedicada antes de codar) |
| D3 | `sim.collision_pulse` + `sim.kill_zone` + `motion.spawn_per_unit` |
| D4 | `pulse.adsr` (envelope no trigger) |
| Aceitação | replicate: scrub bit-exato (o teste que mata o desenho errado); burst determinístico por seed |

### W-E — DADOS & ORDEM
`value.random` · `value.curve` (consome A1) · `value.gather` · `value.sample_stream` · `value.accumulator` · `value.index_switch` · `value.stat` completo (+Selection) · scan com Group Index · upgrades `map_range` (interp modes) e `sort` (group_id). Aceitação: gather/sample com fixture de ordem embaralhada (id-estável, nunca índice — anti-padrão §4.3.7); stat vs oráculo NumPy-style de referência.

### W-F — DISTRIBUIÇÕES & CONNECT
`motion.connect` (a estrela — usa grade espacial GPU p/ vizinhos; params Cavalry no dump) · honeycomb · mask/intersections · shuffle · `points_to_curves` · clone v2 (multi-fonte + time offset + step cumulativo) · push_apart (modos scale/hide). Smoke: cena plexus 5k pontos + cena colmeia.

### W-G — TEMPO
`motion.time_offset` (por instância, dirigível por weight/índice — o retiming como material) · sequencer/visibility · upgrades stagger (Graph por curva) e oscillator (BPM + custom wave). Smoke: o dominó (falloff→delay temporal em cascata — MOPs Delay).

### W-H — ÁUDIO-REATIVO (D6)
| Task | Entregável |
|---|---|
| H1 | Shell bridge `audio_bands` (ph2d-audio-spectral → colunas por frame; função de arquivo+playhead ⇒ scrub exato) — FFT fora do cook, fronteira gateada (padrão `no_codec_reaches_the_mixer`) |
| H2 | `audio.bands` (nó consumidor: N bandas, freq scale, smoothing, index-context) |
| H3 | `field.audio` + `audio.probe` (UX C4D: caixas no espectro) — o probe pede widget de espectro no painel (usar o desenho do audio-editor) |
| Smoke | 3 cenas: barras de LED (Step) · pulso no beat (bands→scale) · campo de áudio movendo grid |

### W-I — UI DO NÓ 2.0 (intercalável com B/C)
| Task | Entregável |
|---|---|
| I1 | Params essenciais NO nó (side-metadata: até 3 rows; mini-slider+chip reusando widgets) |
| I2 | Pips de borda (Bypass·Solo·Viewer) — alvos escalam com zoom |
| I3 | Socket shapes (◇ coluna / ○ valor) + link ilegal vermelho |
| I4 | Cook-time inline (toggle) + zone strap do `sim.zone` + Inspection Index |
| I5 | Drop-no-fio com highlight + shake-to-disconnect; criação Shift=branch/Ctrl=replace/repeat-último |
| I6 | Ctrl+F find-navega; minimap opt-in |
| I7 | **Spreadsheet de colunas** (painel novo `motion-spreadsheet`: instância×coluna, filtros, cru/cozido) |
| I8 | Política de stamps: static-frame + toggle global |
| I9 | **Editor de gradiente** para `motion.color-ramp` (fila — descoberto 2026-07-25 na auditoria "mesmo problema da curva A1", ordem do Enio). Hoje: 6 sliders crus `0..1` p/ **2 stops fixos**, sem swatch/preview/stops arrastáveis — o **único análogo real** do editor de curva. Reusa o primitivo `InteractiveState::CurvePoint` (posição do stop, só `x`) + o swatch OKLCH (`register_picker_swatch`, já reusado pelo vetor); stops MULTI, não 2 fixos. Painel-only, como o A1-ui. Candidato menor e DIFERENTE (não este): `value.attribute` = nome de coluna livre → quer dropdown populado em runtime (precisa o `ParamsSnapshot` enxergar o stream upstream). NÃO mexer: `motion.expression` (fórmula É texto legítimo, idioma de planilha). |
| Aceitação | benchmark 5-gestos re-cronometrado a cada task; seams clicam tudo (a lei das 4 condições da física vale aqui) |

### W-J — SUBGRAFO = ASSET (D10)
Expose dialog (widget/range/grupo/visible_if/batch) · Menu/enum autorável · browser de presets (serialização v2) + pasta user · exemplo carregável por nó (help) · Warning propagável de subgrafo. Aceitação: um subgrafo "DominoWave" salvo, re-instanciado e dirigido só pelos params promovidos.

---

## §8 — Fora desta linha (nomeado, não contrabandeado)

| Frente | Por quê fora | Dependência |
|---|---|---|
| **Pós-FX raster como nós** (blur/glow/levels… — 54 filtros do Cavalry) | os ALGORITMOS já existem em `ph2d-painter-effects`; o que falta é um estágio de render por-instância/grupo no pipeline — decisão de renderer, linha própria | ADR próprio |
| **Texto + strings procedurais** (Text Shape, animação por caractere via seleção de sub-mesh) | o texto vive no módulo vetor (`ph2d-text`, VecTextPath); a família `string.*` só faz sentido sobre ele | cross `line/Vector` |
| **Export Lottie/vídeo** | módulo de export próprio (o precedente é o audio-encode); Lottie do Cavalry mostra o que baka e o que não | linha própria |
| **Inputs interativos** (mouse/keyboard/gamepad) + **state machine** (Rive) | é o contexto Logic do design v1 (D2: v2) e/ou camada da Timeline | decisão Enio |
| **Física↔grafo** (Forge do Cavalry ≈ nosso physics ECS) | pontes (posições de corpos como stream; collision events→pulse) tocam a `line/physics` | cross-line |
| **CSV/Sheets → colunas** (data-driven, Dynamic Rendering) | IO no cook é sujo; o desenho limpo é asset via shell (como o áudio) — pequeno, mas merece nota própria | decisão Enio (P2 in-line possível) |

---

## §9 — Fontes

Dumps completos com URLs por afirmação: `referencia_pesquisa_houdini_mops.md` · `referencia_pesquisa_cavalry.md` · `referencia_pesquisa_c4d_fields.md` · `referencia_pesquisa_blender_gn.md` · `referencia_pesquisa_ui_editores.md` · `referencia_pesquisa_niagara_stardust.md` (esta pasta). Verificação direta: Cavalry Dependency Graph (cavalry.studio/docs — editável, confirmado 2026-07-24). Baseline local: inventário extraído dos MANIFESTs em 2026-07-24 (87 nós/318 params).
