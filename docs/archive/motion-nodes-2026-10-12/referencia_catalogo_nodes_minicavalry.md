# ARQUIVO — referencia_catalogo_nodes_minicavalry.md (história, 505 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`referencia_catalogo_nodes_minicavalry.md`](../../Motion%20Nodes/referencia_catalogo_nodes_minicavalry.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-535,1041-1072` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

# Color (Cor)

### paint — Paint (Pintar)
- **Tipo:** modifier
- **Função em 1 linha:** Substitui cor de cada instância por cor fixa (respeita falloff via lerp).
- **Params principais:** color `#f472b6`.
- **Quando usar:** Tingir logo, cor base antes do Render, cor única em cluster, mascote (corpo).
- **Combina bem com:** Shape, ColorArray (alternar por índice), Falloff, NumberRangeToColor.

### colorArray — Color Array (Paleta)
- **Tipo:** modifier
- **Função em 1 linha:** Cicla cores de uma paleta por índice.
- **Outputs:** out(shape); color(color) array; gradient(gradient).
- **Params principais:** colors `['#ff6b6b','#4ecdc4','#ffe66d','#a8e6cf']`.
- **Quando usar:** Pontos em arco-íris, ícones de status, notas com cor por nota, pétalas multicolor.
- **Combina bem com:** Distribute, CloneLinear, Modulate, Falloff.
- **Gotchas:** TRÊS outputs. Ciclam por `idx % N`.

### numberRangeToColor — Range to Color
- **Tipo:** modifier
- **Função em 1 linha:** Remapeia canal-fonte pra gradiente — cada instância amostra cor na sua posição.
- **Outputs:** out(shape); colors(color) array.
- **Params principais:** sourceChannel `index`; sMin/sMax `0/1`; stops `['#1e3a8a','#3b82f6','#facc15','#f97316']`; easing `1`.
- **Quando usar:** Heatmap, gradiente em arco, gradiente de profundidade, índice → cor temperatura.
- **Combina bem com:** DistributeCircle/Grid/Random, Stagger, NumberRange (irmão escalar).

### colorPicker — Color Picker
- **Tipo:** source
- **Outputs:** out(color); alpha(value).
- **Params:** color `#ff2d92`; alpha `1`.
- **Quando usar:** Cor de marca, splash, UI loader com alpha, paleta de mascote.

### hsvColor — HSV → Color
- **Tipo:** modifier
- **Função em 1 linha:** Converte HSV → hex com inputs sobrescrevendo params.
- **Inputs:** h, s, v (value).
- **Params:** h `0`, s `0.8`, v `1` (0–1).
- **Quando usar:** Arco-íris animado (LFO em hue), estado de saúde, destaque pulsante, gradiente vivo.
- **Combina bem com:** LFO/Time (drive hue), Lag (suavizar), DistanceFromAnchor.normalized (modular V).
- **Gotchas:** Hue faz wrap automático em [0,1).

### sampleGradient — Sample Gradient
- **Tipo:** modifier
- **Função em 1 linha:** Amostra cor de gradient em t (stream) com modo de borda.
- **Inputs:** gradient(gradient); t(value).
- **Params:** tConst `0.5`; repeat `clamp`/`repeat`/`pingpong`.
- **Quando usar:** Heatmap, trail por idade, barra de saúde gradiente, loader por progresso.
- **Combina bem com:** DistanceFromAnchor.normalized, Time/LFO (t animado), StaggerValue.

---

# Focus (Foco)

### falloff — Falloff (Atenuação)
- **Tipo:** modifier
- **Função em 1 linha:** Campo de influência espacial — instâncias dentro têm `falloffStrength`≈1, fora 0. Modifiers downstream respeitam.
- **Params principais:** shape `circle`/`rect`/`linear`; size `250`; curve `smoothstep`/`linear`/`quad`; invert `false`; followMouse `false`.
- **Quando usar:** Hover effect (mascote acorda perto do mouse), spotlight, ripple ao redor de ponto, atenuar wiggle longe do centro.
- **Combina bem com:** Move/Spin/Wiggle/Noise/Paint (respeitam falloffStrength), FalloffBoolean (combinar 2), Render (overlay rosa).
- **Gotchas:** MULTIPLICATIVO — falloffs em cadeia se intersectam.

### falloffBoolean — Boolean Falloff
- **Tipo:** modifier
- **Função em 1 linha:** Combina falloffStrengths de 2 streams — union (max), intersect (min), subtract (A-B), invert (1-A).
- **Inputs:** a(shape); b(shape).
- **Params:** mode `union`.
- **Quando usar:** Dois hotspots, recortar área (subtract), inverter máscara, interseção precisa.
- **Combina bem com:** Falloff (gerar máscaras), modifiers downstream (Move/Wiggle/Paint/Spin).
- **Gotchas:** Falloffs em cadeia já fazem intersect natural. Este nó adiciona union/subtract/invert.

---

# Characters (Personagens)

### skeleton — Skeleton (Esqueleto)
- **Tipo:** source
- **Função em 1 linha:** Cadeia FK paramétrica de N bones (joints igualmente espaçados a partir de raiz).
- **Outputs:** out(skeleton).
- **Params principais:** count `6`; length `40`; rootX/Y `0`; rootAngle `-90°`; bend `0°` (curvatura acumulada).
- **Quando usar:** Ponto de partida obrigatório de QUALQUER pipeline de personagem.
- **Combina bem com:** `skeleton.out` → ik2bone/ikFabrik/skeletonRender/skinDeformer/rubberHose.
- **Gotchas:** Source puro. count=1 gera 2 joints (mínimo). bend alto + count alto = loops.

### ik2bone — IK 2-Bone (Analítico)
- **Tipo:** modifier
- **Função em 1 linha:** Lei dos cossenos pra braço/perna humano (2 bones).
- **Inputs:** in(skeleton) com 3 joints; target(point).
- **Params:** mode `mouse`/`fixed`/`input`; targetX/Y `100/0`; bendDir `down`/`up`.
- **Quando usar:** Braço de mascote seguindo cursor, perna apoiada, mão de robô, garra mecânica.
- **Gotchas:** Exige EXATAMENTE 3 joints. Alvo é clampado em [|L1-L2|, L1+L2].

### ikFabrik — IK FABRIK (Iterativo)
- **Tipo:** modifier
- **Função em 1 linha:** FABRIK (forward-and-backward) pra cadeia de N bones.
- **Inputs:** in(skeleton); target(point).
- **Params:** mode `mouse`/`fixed`/`input`; targetX/Y; iterations `8`; pinBase `true` (true=ombro fixo; false=chicote livre).
- **Quando usar:** Tentáculo/cauda longa, cabelo/coluna, pescoço de NPC, chicote.
- **Combina bem com:** skeleton/point → ikFabrik → skeletonRender/skinDeformer/rubberHose.
- **Gotchas:** pinBase=false faz raiz desgrudar. Iterations baixas = subresolvido.

### rubberHose — Rubber Hose Limb
- **Tipo:** modifier
- **Função em 1 linha:** Skeleton → N pontos amostrados numa bezier quadrática (membro borracha Cuphead/Mickey).
- **Inputs:** in(skeleton).
- **Outputs:** out(shape) — pontos com isPoint+rotation.
- **Params:** samples `20`; curvature `50`; follow `auto`/`perp`/`elbow`; useTangent `true`.
- **Quando usar:** Braço/perna cartoon sem articulações, tentáculo elástico Cuphead, cauda macia, mangueira.
- **Combina bem com:** skeleton/IK → rubberHose → Duplicator + shape pequeno.
- **Gotchas:** Output é shape, não skeleton — não plugue de volta em IK.

### skinDeformer — Skin Deformer
- **Tipo:** modifier
- **Função em 1 linha:** Vincula cada shape ao bone mais próximo (rest pose) e reprojeta a cada frame.
- **Inputs:** in(shape); skeleton(skeleton).
- **Params:** Re-bind button (auto-rebind quando N de instâncias ou bones muda).
- **Quando usar:** Vestir mascote com sprites/shapes (cabeça, mão, pé) seguindo IK, personagem Spine-like.
- **Combina bem com:** skeleton/IK → skinDeformer.skeleton; Duplicator(shapes) → skinDeformer.in.
- **Gotchas:** 1 bone por instância (sem weights). Rest pose capturada na 1ª frame que vê o skeleton — se já estiver deformado, clique Re-bind.

### skeletonRender — Skeleton Render
- **Tipo:** modifier (sink visual)
- **Função em 1 linha:** Bones viram retângulos finos arredondados, joints viram círculos.
- **Inputs:** in(skeleton).
- **Params:** boneThickness `8`; jointSize `12`; boneColor `#fb923c`; jointColor `#ec4899`; showJoints `true`.
- **Quando usar:** Debug visual de qualquer rig, mascote stick-figure, wireframe, visualização técnica/educacional.
- **Gotchas:** Output é shape — terminal visual, não plugue de volta em IK.

### stateMachine — State Machine
- **Tipo:** modifier
- **Função em 1 linha:** FSM de N estados que avança a cada pulse — cycle/bounce/random.
- **Inputs:** trigger(pulse); reset(pulse).
- **Outputs:** out(value) índice; pulse(pulse) na transição.
- **Params:** states `4`; mode `cycle`/`bounce`/`random`; initial `0`.
- **Quando usar:** Orquestrar animações (idle/wave/blink/talk com clique), trocar pose por timer/beat, alternar expressões, ciclar poses de luta.
- **Combina bem com:** pulse source (clique, timer, beat, keyboard) → trigger; out → Switch que escolhe pose/IK target.
- **Gotchas:** Só conta `edge='enter'` com value>0.5. Time scrub pra trás reseta.

---

# Animation (Animação)

### autoAnimate — Auto-Animate
- **Tipo:** modifier
- **Função em 1 linha:** Detecta mudança e tween automático até o novo valor.
- **Params:** channel `all`/x/y/rotation/scale/size; duration `0.4s`; easing `easeInOut`.
- **Quando usar:** Suavizar slider mudando, transição entre tamanhos quando seed muda, repositionamento de cards.
- **Gotchas:** Thresholds embutidos por canal (x/y=0.5px, rot/scale=0.005, size=0.1).

### celAnimation — Cel Animation (Frame-a-Frame)
- **Tipo:** modifier
- **Função em 1 linha:** Alterna entre 2–8 inputs como frames em FPS configurável.
- **Inputs:** frame0..frame7 (shape).
- **Outputs:** out(shape); frame(value) índice.
- **Params:** frames `4`; fps `12`; mode `loop`/`pingpong`/`once`.
- **Quando usar:** Sprite sheet (idle/walk/run), GIF de personagem piscando, respiração, UI com 3 estados.
- **Combina bem com:** ExportLottie/WebM (empacotar), LoopSequencer (sincronia).

### loopSequencer — Loop Sequencer
- **Tipo:** source
- **Função em 1 linha:** Step sequencer (4/8/16 botões on/off) com pulse + value sincronizados.
- **Outputs:** out(value); pulse(pulse); step(value).
- **Params:** steps `8`; stepDuration `0.25s`; pattern `[1,1,1,...]`.
- **Quando usar:** Loop rítmico de luzes, spawner no beat, color flash sincronizado, pisca-pisca de HUD.
- **Combina bem com:** Trigger/Burst em ParticleEmitter, Switch/Mix consumindo value, ExportWebM.

### motionTrail — Motion Trail
- **Tipo:** modifier
- **Função em 1 linha:** Histórico de pose → N fantasmas pra trás com atenuação de escala/opacidade/HSL.
- **Params:** length `12`; spacing `2`; scaleFade `0.6`; includeOriginal `true`; opacityMax/Min `1/0`; hueShift `0°`; satMax/Min `1/1`.
- **Quando usar:** Rastro de cometa, trail de cursor, partícula com cauda colorida, sprite echo.
- **Combina bem com:** Particle/Boids/Force, Replicator + Noise, ExportWebM/MP4.
- **Gotchas:** Depende de `inst.index` estável. Reseta em time-scrub.

### timeRemap — Time Remap
- **Tipo:** modifier
- **Função em 1 linha:** Modifica o `time` passado à sub-árvore UPSTREAM (não a si mesmo) — loop/freeze/pingpong/reverso/escala.
- **Params:** mode `scale`/`loop`/`pingpong`/`freeze`/`reverse`; scale `1`; offset `0s`; duration `2s` (loop/pingpong).
- **Quando usar:** Loop curto de boids longo, freeze pra still PNG, câmera lenta no impacto, reverse de explosion.
- **Combina bem com:** Particle/Noise/Oscillator (qualquer source temporal), ExportLottie/WebM pra fixar timing.
- **Gotchas:** Único nó deste lote com `processTime`. Encadear multiplica efeitos.

---

# Filters/FX (Filtros/FX)

### glow — Glow (Brilho)
- **Tipo:** modifier
- **Função em 1 linha:** Halo via `shadowBlur` (1×, NÃO multiplica instâncias).
- **Params:** intensity `30`; useShapeColor `true`; color `#ffffff`.
- **Quando usar:** Logo neon, badge "selecionado", projétil/laser, partículas mágicas, halo de personagem.
- **Combina bem com:** Bloom (estouro longe), DropShadow (cor escura embaixo), RgbSplit (vaporwave).
- **Gotchas:** Único filtro que NÃO multiplica. Coloque por ÚLTIMO se quiser mandar (Bloom/DropShadow sobrescrevem `_glowBlur`).

### bloom — Bloom (Brilho HDR)
- **Tipo:** modifier
- **Função em 1 linha:** Shapes acima de threshold de luminância viram N cópias borradas e glow atrás.
- **Params:** threshold `0.5`; intensity `1.2`; passes `3`; spread `16`.
- **Quando usar:** Logo neon piscando, sol/luz forte, badge "level up", fogo/lava.
- **Combina bem com:** Levels antes (empurrar cor), Glow (firmar halo próximo), Vignette depois (contrastar).
- **Gotchas:** Multiplica por `passes` × shapes brilhantes — pesado. Depende totalmente da `color`.

### blur — Blur (Desfoque)
- **Tipo:** modifier
- **Função em 1 linha:** Blur Gaussiano por instância via `canvas.filter` (GPU).
- **Params:** amount `6` px.
- **Quando usar:** DoF estilizado, motion blur, suavização de partículas, fundo desfocado pra destacar UI.
- **Combina bem com:** Glow/Bloom (spread mais cremoso), DropShadow.
- **Gotchas:** Custa fillrate. >30 px em muitos shapes derruba FPS. Blur DEPOIS de Glow some com brilho nítido.

### dropShadow — Drop Shadow
- **Tipo:** modifier
- **Função em 1 linha:** Duplica shape como sombra colorida, deslocada e borrada, atrás.
- **Params:** offsetX/Y `8/8`; blur `12`; opacity `0.5`; color `#000`.
- **Quando usar:** Texto/UI legível sobre fundos variáveis, cards flutuantes, separar foreground, sombra colorida estilizada.
- **Combina bem com:** Levels (contraste no original), Blur (objeto fofo + sombra fofa), text/UI sources.
- **Gotchas:** Dobra `count` (downstream vê 2N). Sombra é silhueta borrada, não quadrado.

### levels — Levels (Níveis)
- **Tipo:** modifier
- **Função em 1 linha:** Ajusta brilho/contraste/gamma/black/white-point da `color`.
- **Params:** blackPoint `0`; whitePoint `1`; gamma `1`; brightness `0`; contrast `1`.
- **Quando usar:** Colorgrade global (quente/frio/washed), levantar shapes pro Bloom, padronizar cores, variantes dia/noite.
- **Combina bem com:** Bloom (empurra brilho), Vignette (look cinematográfico).
- **Gotchas:** Mexe SÓ na `color` (não gradient/texture).

### mirror — Mirror (Espelho)
- **Tipo:** modifier
- **Função em 1 linha:** Duplica espelhando posição (e rotation invertida) em torno de um centro.
- **Params:** mode `horizontal`/`vertical`/`both`/`kaleido4`; centerX/Y `0/0`.
- **Quando usar:** Caleidoscópio musical, simetria de logo, padrão decorativo, Rorschach, abertura simétrica.
- **Combina bem com:** Sources de partículas, Glow/Bloom depois, RgbSplit (glitch espelhado).
- **Gotchas:** kaleido4 = 4× instâncias. Combinar com outros multiplicadores escala MUITO rápido.

### rgbSplit — RGB Split
- **Tipo:** modifier
- **Função em 1 linha:** Triplica em R/G/B com offsets opostos, blend aditivo (`_additive: true`).
- **Params:** mode `simple`/`radial`; offsetX `6`; offsetY `0`; intensity `0.7`.
- **Quando usar:** Glitch de erro, cyberpunk, distorção CRT/VHS, "tela quebrando", vaporwave em texto.
- **Combina bem com:** SlitScan (glitch hardcore), Glow antes, Levels (clarear), Mirror.
- **Gotchas:** ×3 SEMPRE. `_additive` precisa fundo escuro pra ver. Modo `radial` precisa shapes longe da origem.

### slitScan — Slit Scan
- **Tipo:** modifier
- **Função em 1 linha:** Histórico de pose por instância → N fantasmas de tempos passados, deslocados por axis.
- **Params:** axis `x`/`y`/`index`; delaySpan `1.5s`; samples `30`; fade `true`.
- **Quando usar:** Rastro de personagem, echo estilizado, bullet-time, distorção temporal, arte generativa.
- **Combina bem com:** Sources com movimento real, RgbSplit (glitch+cromático), Glow.
- **Gotchas:** SEM movimento upstream = invisível. `samples` alto = MUITAS instâncias.

### vignette — Vignette (Vinheta)
- **Tipo:** modifier
- **Função em 1 linha:** Escurece cor por distância radial ao centro (smoothstep).
- **Params:** centerX/Y `0/0`; radius `300`; softness `0.4`; intensity `0.8`.
- **Quando usar:** Look cinematográfico, focar centro (HUD/título), simular lente vintage, transição "fim de level".
- **Combina bem com:** Levels (colorgrade completo), Bloom/Glow (centro brilha, bordas afundam), DropShadow em UI central.
- **Gotchas:** Mexe SÓ na cor (não overlay) — sem shape numa área, vinheta não aparece. Coloque por ÚLTIMO no chain.

---

# Utility (Utilidades)

> Sources de valor escalar/cor/ponto + operadores genéricos. Foco em **driving** outros nós via params promovidos ou sockets dedicados.

### time — Time (Tempo)
- **Tipo:** source
- **Função em 1 linha:** Tempo global, opcionalmente transformado (sine/cosine/loop/pingpong).
- **Outputs:** out(value).
- **Params:** mode `linear`; scale `1`; offset `0`.
- **Quando usar:** Rotação contínua de spinner, pulsar de logo (sine), cronômetro em loop, ping-pong.
- **Combina bem com:** AnimationCurve (curvar t), MapRange, ComposeXY, Math.

### lfo — LFO (Oscilador Multi-onda)
- **Tipo:** source
- **Função em 1 linha:** Onda configurável (sine/cosine/triangle/saw/square/spike) + pulse por ciclo.
- **Outputs:** out(value); pulse(pulse) — edge='rise' a cada novo ciclo.
- **Params:** wave `sine`; frequency `1` Hz; amplitude `1`; offset `0`; phase `0`.
- **Quando usar:** Spinner (sine em rotação), piscar de notificação (square), batimento (triangle), metrônomo (spike).
- **Combina bem com:** Counter (ciclos via pulse), MapRange, AnimationCurve, SampleHold.

### noiseValue — Noise Value (Escalar)
- **Tipo:** source
- **Função em 1 linha:** Perlin 1D animado com fBm (até 6 oitavas).
- **Outputs:** out(value) — sempre 1 elemento.
- **Params:** frequency `1`; amplitude `1`; offset `0`; octaves `1`; seed `1`.
- **Quando usar:** Wobble orgânico de chama, tremor "vivo" de mascote, câmera shake sutil, respiração de UI.
- **Combina bem com:** MapRange, ComposeXY (2 noise → ponto), Lag, SpringDriver.

### randomValue — Random Value
- **Tipo:** source
- **Função em 1 linha:** N valores aleatórios determinísticos (LCG) em uniform/gaussian/bernoulli.
- **Params:** distribution; seed `1`; count `1` (1–100); min/max; mean/stddev; probability.
- **Quando usar:** Variações fixas em grid de cards, offsets pra estrelas, moedas de probabilidade pra spawn, tabela de delays.
- **Gotchas:** Determinístico — mesmo seed = sempre mesmos valores.

### constant — Constant
- **Tipo:** source
- **Função em 1 linha:** Literal tipado único (Value/Position/Color).
- **Params:** mode `value`/`point`/`color`; value/x/y/color.
- **Quando usar:** Default de slot promovido, offset fixo em ComposeXY, cor padrão de fallback, anchor fixo.
- **Gotchas:** Tipo do output muda dinamicamente — trocar `mode` quebra conexões incompatíveis.

### colorPicker — ver Color
### hsvColor — ver Color
### sampleGradient — ver Color

### mousePosition — Mouse Position
- **Tipo:** source
- **Outputs:** out(point); x(value); y(value).
- **Params:** scale `1` (pode inverter); offsetX/Y `0/0`.
- **Quando usar:** Cursor customizado de mascote, parallax leve, joystick virtual, spotlight de portfolio.
- **Combina bem com:** Lag, DistanceFromAnchor, ComposeXY, MapRange.

### viewportSize — Viewport Size
- **Tipo:** source
- **Outputs:** out(point); w(value); h(value); aspect(value).
- **Params:** mode `logical`/`physical`/`normalized`.
- **Quando usar:** Layout responsivo de HUD, margem proporcional, reescalar grid pra caber, demo adaptativa.

### counter — Counter
- **Tipo:** modifier
- **Inputs:** trigger(pulse); reset(pulse).
- **Outputs:** out(value).
- **Params:** step `1`; initial `0`; maxValue `0` (>0 ativa wrap).
- **Quando usar:** Contador de pontos, índice de slide, ticker de notificações, marcador de batidas.
- **Combina bem com:** PulseOnChange/Threshold (gerar triggers), LFO (pulse de batida), Switch (índice N-way).
- **Gotchas:** Só `edge='enter'`. Time-scrub pra trás reseta.

### sampleHold — Sample and Hold
- **Tipo:** modifier
- **Inputs:** in(value); trigger(pulse).
- **Outputs:** out(value) — último capturado.
- **Params:** initial `0`.
- **Quando usar:** Congelar high score, freeze de preço no clique, capturar timestamp no beat, lock-in de cor.
- **Gotchas:** Só `in[0]`, só `edge='enter'`. Time-scrub reseta.

### pulseOnChange — Pulse on Change
- **Tipo:** modifier
- **Inputs:** in(value) — lê `[0]`.
- **Outputs:** out(pulse).
- **Params:** tolerance `0.001`.
- **Quando usar:** Disparar SFX quando contador varia, flash de UI no valor de bolsa mudar, reset de animação no seed mudar.
- **Gotchas:** Só primeiro elemento. Só edge `enter`/`idle`.

### threshold — Threshold (Limiar com Schmitt)
- **Tipo:** modifier
- **Inputs:** in(value).
- **Outputs:** out(value) 0/1; pulse(pulse) enter/exit.
- **Params:** threshold `0.5`; hysteresis `0.05`.
- **Quando usar:** Detector ON/OFF com noise, trigger de SFX no pico, gate de shake detection, binarizar sensor.
- **Gotchas:** Estado único compartilhado entre elementos do stream.

### compare — Compare
- **Tipo:** modifier
- **Inputs:** a, b (value).
- **Outputs:** out(value) 1/0.
- **Params:** op `gt`/`lt`/`eq`/`neq`/`lte`/`gte`; constant `0` (B fallback).
- **Quando usar:** Pulse quando pontos>100, trocar cor quando time>5s, destacar quando mouseX<anchor, gate de power-up.
- **Combina bem com:** Boolean, Switch, PulseOnChange.
- **Gotchas:** `eq`/`neq` usam eps=1e-6. B vazio cai em `constant`.

### boolean — Boolean (Lógico)
- **Tipo:** modifier
- **Inputs:** a, b (value).
- **Outputs:** out(value) 0/1.
- **Params:** op `and`/`or`/`not`/`xor`/`nand`/`nor`.
- **Quando usar:** Combinar "mouse no hero" AND "scroll<100", inverter HitTest, debouncer lógico.

### math — Math
- **Tipo:** modifier
- **Outputs:** out(value).
- **Params:** op `add`/`sub`/`mul`/`div`/`mod`/`pow`/`min`/`max`/`lerp`/`sin`/`cos`/`abs`/`sqrt`/`neg`; constant `0`.
- **Quando usar:** Inverter sinal de Y, somar offset em barra, lerp pra fade, sin de tempo pra wobble simples.
- **Gotchas:** `div`/`mod` por 0 devolvem 0. `sqrt` de negativo = `sqrt(0)`. `lerp(a, b, k=constant)`.

### vectorMath — Vector Math
- **Tipo:** modifier
- **Inputs:** a, b (point).
- **Outputs:** out(point); scalar(value).
- **Params:** op `add`/`sub`/`scale`/`dot`/`cross`/`length`/`normalize`/`lerp`.
- **Quando usar:** Vetor de mira (sub), steering de boid (normalize), câmera-look (lerp), ângulo (cross 2D).
- **Gotchas:** `scale` usa |B|. `lerp` é FIXO em 0.5. `dot`/`cross`/`length` poluem `out.x` — use porta `scalar`.

### composeXY — Compose XY
- **Tipo:** modifier
- **Inputs:** x, y (value).
- **Outputs:** out(point).
- **Params:** constX/Y `0/0` (fallback).
- **Quando usar:** Trajetória senoidal (LFO em Y, Time em X), tooltip seguindo mouseX com Y fixo, joystick virtual.
- **Gotchas:** Stream menor estica pelo último valor.

### extractXY — Extract XY
- **Tipo:** modifier
- **Inputs:** in(point).
- **Outputs:** x, y (value).
- **Quando usar:** mouseY → volume, X de joystick → rotação, Y de partícula → cor por altura, debug.

### mapRange — Map Range
- **Tipo:** modifier
- **Inputs:** in(value).
- **Params:** inMin/inMax `0/1`; outMin/outMax `0/100`; clamp `true`.
- **Quando usar:** normalized [0..1] → scale [0.5..2], mouseY → volume, LFO [-1..1] → opacity [0..1].
- **Gotchas:** Pode mapear range invertido.

### animationCurve — Animation Curve (Easing)
- **Tipo:** modifier
- **Inputs:** t(value).
- **Outputs:** out(value).
- **Params:** easing `easeOutCubic`; clamp `true`; autoTime `false`; duration `1s`.
- **Quando usar:** Easing de loader, curva de modal entrando, rebound de mascote ao clicar, slide de onboarding.
- **Gotchas:** `autoTime` é cíclico, não one-shot.

### lag — Lag (Suavizar)
- **Tipo:** modifier
- **Inputs:** in(value).
- **Params:** responseTime `0.2s`.
- **Quando usar:** Suavizar tremor do mouse, easing de barra de progresso, smoothing de joystick, estabilizar scroll.

### springDriver — Spring Driver
- **Tipo:** modifier
- **Inputs:** in(value) "Target".
- **Outputs:** out(value).
- **Params:** stiffness `100`; damping `10`; mass `1`.
- **Quando usar:** Bounce físico de menu, arraste com inércia em card, rebound de botão, cursor com peso.
- **Gotchas:** Sub-stepping interno. Estado por índice.

### staggerValue — Stagger Value
- **Tipo:** source
- **Outputs:** out(value) — N valores 0→1 escalonados.
- **Params:** count `5`; delay `0.1s`; duration `1s`; easing `easeOutCubic`.
- **Quando usar:** Onboarding em cascata, menu escalonado, wave de leds, revelar letras.
- **Gotchas:** Sem reset — após duração+delay total, todos em 1 (não loopa).

### switch — Switch (Alternar)
- **Tipo:** modifier
- **Inputs:** cond(value); a..h(shape).
- **Outputs:** out(shape).
- **Params:** mode `binary`/`index`; threshold `0.5`; count `2` (até 8 em N-way).
- **Quando usar:** Mascote happy/sad por estado, splash/login/main por step, tema claro/escuro, HUD por fase.
- **Combina bem com:** Counter (cond=índice), Compare, Threshold, CombineStreams.

### combineStreams — ver Compose
### groupTag — Group Tag
- **Tipo:** modifier
- **Função:** Anexa `groupId` (0–32) a cada instância (sem mudar geometria).
- **Params:** tag `0`.
- **Quando usar:** Separar boids aliados/inimigos, marcar fumaça vs faísca, rotular leds por status.

### hitTest — Hit Test
- **Tipo:** modifier
- **Inputs:** in(shape).
- **Outputs:** out(value) 1/0 por instância; pulse(pulse) global enter/exit.
- **Params:** radius `50`.
- **Quando usar:** Hover de ícones de menu radial, detector de toque, spotlight ao passar mouse, tooltip de mascote.

### distanceFromAnchor — Distance from Anchor
- **Tipo:** modifier
- **Inputs:** in(shape); anchor(point).
- **Outputs:** out(value) distância; normalized(value) `d/radius` em [0..1].
- **Params:** source `param`/`mouse`/`input`; x/y `0`; radius `200`.
- **Quando usar:** Halo de mouse iluminando ícones, spotlight de seleção, campo gravitacional, falloff de hover em grid.

---

# Output (Saída)

### render — Render (Renderizar)
- **Tipo:** sink
- **Função em 1 linha:** Desenha as instâncias no canvas — saída final do grafo.
- **Inputs:** in(shape).
- **Quando usar:** Sempre — todo projeto precisa de pelo menos um Render no fim.

### exportPNG — Export PNG
- **Tipo:** sink
- **Função:** Botão "Capturar canvas" → `toDataURL('image/png')` → download.
- **Quando usar:** Thumbnail, print pra portfolio, key art com TimeRemap freeze, referência rápida.

### exportMP4 — Export MP4
- **Tipo:** sink
- **Função:** Botão "Gravar MP4" → MediaRecorder com fallback (mp4/h264 → webm/vp9).
- **Params:** duration `5s`; fps `30`; bitrate `8 Mbps`.
- **Quando usar:** Instagram/TikTok, entrega cliente, reel de feature, teaser de game.
- **Gotchas:** Em Chrome/Firefox cai pra WebM (extensão muda). Sem áudio. Duração travada.

### exportWebM — Export WebM
- **Tipo:** sink
- **Função:** Botão "Gravar (clique)" → MediaRecorder VP9/WebM.
- **Params:** duration `5s`; fps `30`; bitrate `5 Mbps` (fixo, sem UI).
- **Quando usar:** GIF leve pra web, preview pra Discord/Slack, reel com transparência, render rápido.

### exportLottie — Export Lottie
- **Tipo:** sink
- **Função:** Amostra N frames pausando o engineLoop, baixa JSON Lottie/bodymovin v5.7.
- **Params:** duration `3s`; fps `30`; width/height `800/600`.
- **Quando usar:** Logo intro pra LottieFiles, ícone pra Figma/AE, spinner ultra-leve em SVG-data, UI motion pra mobile.
- **Gotchas:** Pausa engine pra não corromper state. Todos shapes viram elipses no JSON (perde detalhe). Só não-points.

### exportWebComponent — Export Web Component
- **Tipo:** sink
- **Função:** Gera HTML standalone com `<mini-cavalry-scene>` + grafo serializado inline.
- **Params:** width `800`; height `600`; autoplay `true`.
- **Quando usar:** Embed em site cliente, preview estático, portfolio autocontido, landing hero animado.
- **Gotchas:** **PLACEHOLDER** — o HTML gerado mostra só texto + contagem; NÃO executa o grafo. Roadmap pra single-file bundle.

---

