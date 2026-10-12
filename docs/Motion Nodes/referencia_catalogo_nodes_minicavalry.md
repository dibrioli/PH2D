> **PROVENIÊNCIA:** cópia de `/home/enio/Documentos/Recursos/Nodes/DOCS/_NODES.md`, trazida
> pro repo em 2026-07-09 a pedido do Enio. É o catálogo comportamental do protótipo
> MiniCavalryV2 (JS proprietário fica FORA do repo — clean-room: só comportamento). Vale como
> referência de VOCABULÁRIO e UX de autoria (nota da seção Physics: forças em cadeia linear,
> "drag depois pra estabilizar", "PinConstraint por último" — zero fios de retorno). Os
> MECANISMOS internos do protótipo (estado oculto por nó, dt wall-clock) foram REJEITADOS
> pelo PH2D — racional em `02_dinamica_m2_pesquisa_decisoes.md` e `03_reentrada_integrate_estudo_padrao_ouro.md`.

# Mini Cavalry — Referência de Nós (autor)

> História verbatim: [arquivo](../archive/motion-nodes-2026-10-12/referencia_catalogo_nodes_minicavalry.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| Mecanismos internos do protótipo MiniCavalryV2 (estado oculto por nó, `dt` wall-clock) | Rejeitados pelo PH2D: este catálogo vale só como vocabulário e UX de autoria (clean-room: só comportamento) | vivo, proveniência (racional em `02_dinamica_m2_pesquisa_decisoes.md` e `03_reentrada_integrate_estudo_padrao_ouro.md`) |
| Fios de retorno nas forças em cadeia | A nota da secção Physics: forças em cadeia linear, «drag depois», «PinConstraint por último» — zero fios de retorno | vivo, Physics |
| (sem recusa — corte de tamanho) Categorias Color · Focus · Characters · Animation · Filters/FX · Utility · Output | Só referência de vocabulário do protótipo; arquivadas verbatim para caber em 40 KB | [arquivo](../archive/motion-nodes-2026-10-12/referencia_catalogo_nodes_minicavalry.md) |

> Catálogo interno para escrita de tutoriais. **Não é tutorial em si** — é o índice mestre de:
> o que cada nó faz, portas, params essenciais, casos de uso e combos naturais.
> Atualize quando um nó novo entrar ou um param mudar.

## Arquitetura (resumo)

- **Loader:** `mini-cavalry.html` carrega `core/state.js` → `registry.js` → `helpers.js` → `editor.js` → `engine.js`, depois cada `nodes/**.js` chama `registerNode(key, def)`.
- **Avaliação:** `engine.evaluateNode` é DFS reverso a partir do `renderNodeId`, com cache por frame (`_evalCache`), suporte a Group (proxy in/out) e `processTime` (override de tempo upstream — usado por TimeRemap).
- **Tipos de socket (7):** `shape`, `point`, `value`, `color`, `gradient`, `pulse` (`{value, edge, t}`), `skeleton` (`{joints, bones}`). Conversões implícitas via `convert(value, from, to)` cobrem a maioria dos gaps (ver `helpers.js`).
- **Instâncias renderizáveis:** `{x, y, rotation, scale, size, color, type, index, count}` + flags opcionais (`_alpha`, `_additive`, `_filterBlur`, `_glowBlur`, `_glowColor`, `anchorX/Y`).
- **Falloff:** modifiers downstream respeitam `inst.falloffStrength` (escalar 0–1) — campo de influência espacial atenua transformações.

## Categorias (ordem da sidebar)

1. Generators (Geradores)
2. Distribute (Distribuir)
3. Compose (Composição)
4. Transform (Transformar)
5. Behaviours (Comportamentos)
6. Physics (Físicas)
7. Vary (Variar)
8. Color (Cor)
9. Focus (Foco)
10. Characters (Personagens)
11. Animation (Animação)
12. Filters/FX (Filtros/FX)
13. Utility (Utilidades)
14. Output (Saída)

---

# Generators (Geradores)

### shape — Shape (Forma)
- **Tipo:** source
- **Função em 1 linha:** Emite UMA instância renderizável de uma forma geométrica (circle/square/ellipse/rectangle/polygon/star/heart) na origem.
- **Inputs:** —
- **Outputs:** out(shape)
- **Params principais:** type `circle` (circle/square/ellipse/rectangle/polygon/star/heart); size `50` (10–200); sides `6` (polygon); points `5` + innerRatio `0.45` (star); cornerRadius `12` + frameThickness `0` (rectangle); innerSize `0` (furo no círculo).
- **Quando usar:** Base de qualquer composição — partícula de combate, ícone de loader, item de UI, cabeça de mascote, pétala duplicada N vezes.
- **Combina bem com:** Duplicator (instanciar em pontos), Paint (colorir), CloneLinear (criar fila/onda), Spin (rotação contínua), Morph (transformar A→B).
- **Gotchas:** Dropdown `type` muda quais sliders aparecem. Sempre devolve 1 instância — pra muitas, encaminhar via Distribute+Duplicator.

### text — Text (Texto)
- **Tipo:** source
- **Função em 1 linha:** Renderiza string como letras/palavras/linhas indexadas, centralizado H/V.
- **Outputs:** out(shape) — N instâncias type='text' com `glyph`.
- **Params principais:** text `'VIBE'` (textarea); mode `char` (char/word/line); size `80`; tracking `0`; lineHeight `1.2`.
- **Quando usar:** Logo letra-por-letra, intro com text reveal, mensagem de loader, kinetic typography.
- **Combina bem com:** Stagger (revelar com rampa), Oscillator (wave por letra), Wiggle (texto tremendo), Spring (bouncy), NumberRangeToColor (gradiente).
- **Gotchas:** Cada letra é instância com index — todos behaviours indexados funcionam.

### image — Image (Imagem)
- **Tipo:** source
- **Função em 1 linha:** Emite UMA instância renderizando uma imagem (file picker, URL ou base64).
- **Outputs:** out(shape) — uma instância type='image' com `src`.
- **Params principais:** src `''` (file picker/URL/base64); size `120`.
- **Quando usar:** Logo/branding, mascote bitmap, avatar com efeitos, sprite-base, intros com asset.
- **Combina bem com:** Duplicator (imagem em pontos), Spin/Wiggle (animar logo), Falloff, Distribute (mosaico).
- **Gotchas:** Limite 10MB no upload local. Sem src devolve []. PNG/JPG/GIF/WebP/SVG.

### lsystem — L-System
- **Tipo:** source
- **Função em 1 linha:** Reescreve axiom N vezes aplicando regras, depois interpreta como turtle 2D — gera fractais.
- **Outputs:** out(shape) — pontos isPoint com tangente, centralizados.
- **Params principais:** axiom `'F'`; rule `'F=F+F-F-F+F'` (`;` separa); iterations `3` (0–6); angle `90°`; length `12`.
- **Quando usar:** Snowflake/Koch, planta/árvore, Dragon curve, intro generativa.
- **Combina bem com:** Duplicator + Shape (forma em cada vértice), Paint, NumberRangeToColor, Spin.
- **Gotchas:** Limites MAX_LEN=30k chars, MAX_POINTS=4k. Símbolos: F/G avança+emite, f avança sem emitir, +/- giram, [/] push/pop.

### particleEmitter — Particle Emitter (Emissor de Partículas)
- **Tipo:** source
- **Função em 1 linha:** Emite partículas contínuas com taxa, vida, velocidade, gravidade, cone, fade-out.
- **Outputs:** out(shape) — partículas type='circle'.
- **Params principais:** rate `30` p/s; lifespan `2s`; speed `120`; gravity `80`; direction `-90°`; spread `60°`; originX/Y `0/150`; particleSize `12`; fadeOut `true`.
- **Quando usar:** Fogo/fumaça/faísca, explosão de combate, sparkles atrás de cursor, fontes, chuva, neve, confetti.
- **Combina bem com:** Paint, NumberRangeToColor (gradiente por idade), Falloff, Wiggle (turbulência), Mixer.
- **Gotchas:** Limite hard 500 partículas. Time scrub > 1s ou negativo limpa pool.

### dataSource — Data Source (CSV/JSON)
- **Tipo:** source
- **Função em 1 linha:** Parseia JSON ou CSV e gera instâncias com campos reconhecidos (x, y, scale, rotation, size, color, type).
- **Outputs:** out(shape) — N instâncias com campos da fonte.
- **Params principais:** format `json` (json/csv); data `'[...]'` (textarea ou file picker); scaleField `1`.
- **Quando usar:** Visualizações de dados, layouts pré-definidos, mapa de pontos, importar exports de Figma/Excel.
- **Combina bem com:** NumberRange (mapear colunas), NumberRangeToColor (heatmap), Paint, Spin.
- **Gotchas:** Tem cache por `format|raw|scale` — não reparseia a 60 FPS.

---

# Distribute (Distribuir)

> Todos os Distribute têm **dois outputs**: `out` (shape, com isPoint) e `points` (point, conecta no input `points` do Duplicator).

### distributeGrid — Distribute Grid (Distribuir Grade)
- **Tipo:** source
- **Função em 1 linha:** Gera grade rows×cols centrada na origem.
- **Outputs:** out(shape); points(point)
- **Params principais:** cols `5`, rows `5` (1–20); gapX `50`, gapY `50` (0–200).
- **Quando usar:** Wallpaper de ícones, dashboard de cards, mosaico, matriz LED de pixel art.
- **Combina bem com:** Duplicator + Shape, Stagger (offset Y = ondulação), Falloff, Wiggle.

### distributeCircle — Distribute Circle (Distribuir Círculo)
- **Tipo:** source
- **Função em 1 linha:** Distribui N pontos em círculo/arco/espiral.
- **Outputs:** out(shape); points(point)
- **Params principais:** count `12`; radius `120`; startAngle `0°`; arc `360°`; voltas `1` (espiral se >1); radiusGrowth `0`; useRotation `false` (radial).
- **Quando usar:** Anel de loading, pétalas, ponteiro de roleta, espiral hipnótica, halo de partículas.
- **Combina bem com:** Duplicator, Spin (rotação real), Pivot (radial), Stagger (cores por índice).
- **Gotchas:** `voltas > 1 + radiusGrowth > 0` = espiral viva.

### distributePath — Distribute Path (Distribuir Caminho)
- **Tipo:** source
- **Função em 1 linha:** Distribui N pontos uniformemente ao longo do comprimento real (arc-length) de uma curva paramétrica.
- **Outputs:** out(shape); points(point)
- **Params principais:** shape `sine` (line/sine/figure8/heart/rose); count `30`; width `400`; height `120`; useTangent `false`; tangentOffset `0`.
- **Quando usar:** Texto seguindo onda, notas musicais em senoide, coração de partículas, lemniscata, rosácea.
- **Combina bem com:** Duplicator (forma na curva), LookAt + useTangent (apontar), Spin, CloneLinear.
- **Gotchas:** Pesado — tem cache por params. figure8/heart/rose são fechadas.

### distributeFibonacci — Distribute Fibonacci
- **Tipo:** source
- **Função em 1 linha:** Padrão filotático (Vogel/girassol) — raio = scale*√i, ângulo = i*angle (137.5° = áureo).
- **Outputs:** out(shape); points(point)
- **Params principais:** count `80` (1–500); scale `12`; angleDeg `137.5` (girassol; outros dão padrões distintos).
- **Quando usar:** Girassol, pinha, miolo de flor, padrão orgânico denso, mandala procedural.
- **Combina bem com:** Duplicator + Shape, NumberRangeToColor (gradiente radial), Paint, Spin. Experimentar 90, 144, 222.5.

### distributeRandom — Distribute Random
- **Tipo:** source
- **Função em 1 linha:** N pontos aleatórios em W×H (determinístico por seed), opcionalmente com Poisson-disk via `minDist`.
- **Outputs:** out(shape); points(point)
- **Params principais:** count `50`; width `500`, height `350`; seed `1`; minDist `0` (>0 ativa Poisson).
- **Quando usar:** Estrelas, partículas espalhadas, confetti, debris, fundo de menu.
- **Combina bem com:** Duplicator, Random/NumberRangeToColor, Wiggle.
- **Gotchas:** Com `minDist>0`, pode devolver menos que count se área lotar.

### poissonDisk — Poisson Disk
- **Tipo:** source
- **Função em 1 linha:** "Blue noise" — pontos com distância mínima garantida (Bridson dart-throwing).
- **Outputs:** out(point) — pontos isPoint.
- **Params principais:** width `500`, height `350`; minDist `30`; k `30` (tentativas); seed `1`.
- **Quando usar:** Árvores em top-down, estrelas/confete espaçados, marcadores em mapa, sample points pra fluido.
- **Combina bem com:** ConvexHull, VoronoiPattern, Replicator/Duplicator, ShapeEdges.
- **Gotchas:** Cacheado por `W|H|minDist|k|seed`. Sem garantia de count exato.

### voronoiPattern — Voronoi Pattern
- **Tipo:** source
- **Função em 1 linha:** Seeds de células Voronoi via grid jittered (não calcula polígonos — só centros).
- **Outputs:** out(point) — centros.
- **Params principais:** count `30`; width `500`, height `350`; jitter `0.7` (0=grid, 1=caos); seed `1`.
- **Quando usar:** Padrão orgânico de fundo, pontos de spawn, mosaico abstrato, "pedras" decorativas.
- **Combina bem com:** Duplicator + Shape (visualiza), ConvexHull, MotionTrail, Twist/Bend.
- **Gotchas:** Não calcula polígonos. Count exato não garantido.

### shapeEdges — Shape Edges
- **Tipo:** modifier
- **Função em 1 linha:** Distribui N pontos no perímetro de cada shape de entrada (usa `shapeToPoints`).
- **Inputs:** in(shape)
- **Outputs:** out(point) — pontos por shape.
- **Params principais:** resolution `32` (4–200) — pontos por shape.
- **Quando usar:** Texto como nuvem de pontos, círculo → N anchors, wireframe, shape → ConvexHull/Voronoi.
- **Combina bem com:** Text/Shape sources, Duplicator, ConvexHull, Bend/Twist/Lattice.
- **Gotchas:** Resolução total = N × shapes de entrada.

---

# Compose (Composição)

### duplicator — Duplicator (Duplicador)
- **Tipo:** modifier
- **Função em 1 linha:** Instancia a forma de `shape` em cada ponto de `points`, somando posição e rotação.
- **Inputs:** shape(shape); **points(point)** — porta nomeada, não-default
- **Outputs:** out(shape) — N×M instâncias.
- **Quando usar:** Estrela em cada nó de grade, partículas em pontos de path, mesma forma em cada perna do Walker, mascotes.
- **Combina bem com:** Todos Distribute (porta `points`), Shape, Tentacle/Walker.
- **Gotchas:** Porta `points` é não-default — conecte EXPLICITAMENTE. Sem points, vira pass-through.

### cloneLinear — Clone Queue (Clonar Fila)
- **Tipo:** modifier
- **Função em 1 linha:** Fila de N cópias num eixo (X/Y/angle), com taper opcional de scale e rotation.
- **Inputs:** in(shape)
- **Params principais:** count `5`; gap `60`; axis `x` (x/y/angle); angleDeg `0`; scaleTaper `1` (lerp final); rotTaperDeg `0`.
- **Quando usar:** Trilho de notas, cobra que diminui, escada de cards, fila de ícones, foguete com chama (taper).
- **Combina bem com:** Shape, Stagger (variar por índice), Spin (cada cópia girando), Falloff.
- **Gotchas:** Centraliza em torno da origem. Taper é lerp do 1º ao último.

### group — Group (Grupo)
- **Tipo:** modifier
- **Função em 1 linha:** Encapsula sub-grafo num único nó (1 in / N out via OutputProxies).
- **Inputs:** in(shape); **Outputs dinâmicos** baseados em OutputProxies filhos.
- **Quando usar:** Organizar pipelines complexos, componentes reutilizáveis, esconder complexidade.
- **Combina bem com:** InputProxy/OutputProxy (obrigatórios), qualquer nó dentro.
- **Gotchas:** Duplo-clique no header pra entrar. Engine trata tipo `group` diretamente.

### inputProxy / outputProxy — Group I/O (hidden na sidebar)
- Criados automaticamente ao entrar num Group.
- `label` define o nome do pino no Group pai.

### mixer — Behaviour Mixer
- **Tipo:** modifier
- **Função em 1 linha:** Combina até 4 streams ponderadamente — modo `avg` (suaviza) ou `add` (empilha).
- **Inputs:** a, b, c, d (shape)
- **Params principais:** mode `avg`/`add`; wa/wb/wc/wd `1,1,0,0`; geomFrom `auto` (auto/a/b/c/d).
- **Quando usar:** Combinar wiggle+oscillator+spring na mesma forma, blend entre poses, crossfade de layouts, somar forças em enxame.
- **Combina bem com:** Wiggle/Oscillator/Spring/Noise (cada um numa lane).
- **Gotchas:** Cor sempre em média ponderada (somar RGB satura). `geomFrom: auto` pega lane com maior peso.

### morph — Morph (Morphing A→B)
- **Tipo:** modifier
- **Função em 1 linha:** Mistura duas streams — `vertex` (perímetros amostrados e lerpados), `switch` (troca abrupta), `crossfade` (ambas com scale lerpado).
- **Inputs:** a(shape); b(shape)
- **Params principais:** t `0.5`; easing `linear`; mode `vertex`; resolution `64`; threshold `0.5` (switch).
- **Quando usar:** Logo se transformando, ícone A→B, play→pause, mascote mudando expressão.
- **Combina bem com:** Shape (A e B), Oscillator (animar t via Modulate/NumberRange), Paint.
- **Gotchas:** Vertex faz fallback pra retângulo em text/image. Crossfade emite 2N instâncias.

### combineStreams — Combine Streams
- **Tipo:** modifier
- **Função em 1 linha:** Concatena até 4 streams shape (A+B+C+D), opcionalmente reindexando.
- **Inputs:** a, b, c, d (shape)
- **Params principais:** reindex `true`.
- **Quando usar:** Juntar fumaça + chispas + brilhos num render, HUD + cena, merge de grids pra Boids, ícones de fontes diferentes.
- **Combina bem com:** Duplicator/Grid (gerar sub-streams), GroupTag (taggear antes), Switch (alternar).
- **Gotchas:** Com `reindex=false`, índices podem colidir.

---

# Transform (Transformar)

### move — Move (Mover)
- **Tipo:** modifier
- **Função em 1 linha:** Soma offset X/Y constante (×falloffStrength).
- **Params principais:** x `0`; y `0` (-300 a 300).
- **Quando usar:** Sair da origem, centralizar grupo num lado, ajustar baseline, offset de UI.
- **Combina bem com:** Shape, Falloff (mover só região), Stagger (deslocar progressivamente).

### pivot — Pivot (Pivô)
- **Tipo:** modifier
- **Função em 1 linha:** Define ponto de rotação/escala local (normalizado: -1=borda, 0=centro, 1=borda oposta).
- **Params principais:** anchorX `0` (-2 a 2); anchorY `0`. UI com 9 botões cardeais.
- **Quando usar:** Conectar tentáculos pela ponta (anchorX=-1), ponteiro de relógio (anchor na base), flip-card pivotar lateral, escalar do canto.
- **Combina bem com:** Spin (pivô = eixo), CloneLinear (taper diferente), Shape.
- **Gotchas:** Engine aplica offset DEPOIS de rotate/scale. anchorX=-1 é o truque clássico pra cadeias/tentáculos.

### cloneLinear — ver Compose acima

### snapGrid — Snap to Grid
- **Tipo:** modifier
- **Função em 1 linha:** Força x/y/rotation a múltiplos de uma grade.
- **Params principais:** snapPos `true`, snapRot `false`; gridX/gridY `20/20`; snapRotDeg `15`.
- **Quando usar:** Pixel art, composições geométricas, layouts modulares, chunky/retro, ângulos iso (30/45).
- **Combina bem com:** Wiggle/Noise (chunky noise), Falloff (snap em região), DistributeRandom (organizar caos).
- **Gotchas:** Honra falloff. SnapPos e SnapRot independentes.

### bend — Bend Deformer
- **Tipo:** modifier
- **Função em 1 linha:** Curva instâncias ao longo de eixo com rotação proporcional à distância do centro.
- **Params principais:** axis `horizontal` (H/V); angle `90°`; center `0`; range `200`.
- **Quando usar:** Texto distribuído em arco (banner curvado), wave de cards inclinando, bandeira tremulando, títulos arco-íris.
- **Combina bem com:** ShapeEdges, Replicator/Grid (linha de elementos), Falloff, TimeRemap.
- **Gotchas:** Rotaciona posição E rotation. Clamp em ±1 do range.

### twist — Twist Deformer
- **Tipo:** modifier
- **Função em 1 linha:** Rotaciona ao redor de um centro com ângulo proporcional à distância — cria espirais e torções.
- **Params principais:** mode `radial` (R/H/V); strength `360°/200px` (-720 a +720); centerX/Y `0/0`.
- **Quando usar:** Espiral de partículas (feitiço), título em "vortex" de entrada, pétalas torcidas, transição entre cenas.
- **Combina bem com:** Replicator/Grid/PoissonDisk, Bend (combo), TimeRemap, Falloff.
- **Gotchas:** Strength é graus por 200px — não é volta completa.

### fourPointWarp — Four Point Warp
- **Tipo:** modifier
- **Função em 1 linha:** Mapeia retângulo source nos 4 cantos destino via interpolação bilinear.
- **Params principais:** srcWidth/Height `400/300`; tlX/Y, trX/Y, blX/Y, brX/Y — 4 cantos.
- **Quando usar:** Mockup de tela de celular, letreiro no chão, perspectiva forçada em poster, corrigir/distorcer quad.
- **Combina bem com:** Grid/Rectangle, Lattice, Bend, ExportPNG.
- **Gotchas:** u,v clampados em [0,1]. É warp bilinear, não homografia.

### lattice — Lattice Deformer
- **Tipo:** modifier
- **Função em 1 linha:** Aplica offsets procedurais (sin/cos no tempo) numa grade N×M e interpola bilinear.
- **Params principais:** cols `5`, rows `5`; width `500`, height `400`; amplitude `30`; frequency `1`; speed `0.5`.
- **Quando usar:** Texto "água"/"gelatina", TV antiga com warp, fundo abstrato animado, jelly bounce em UI.
- **Combina bem com:** Grid/Rectangle/Poisson, ShapeEdges, TimeRemap (loop), Bend.
- **Gotchas:** Offsets procedurais (não editáveis ponto-a-ponto). Fora de W×H passa intacto.

---

# Behaviours (Comportamentos)

### spin — Spin (Girar)
- **Tipo:** modifier
- **Função em 1 linha:** Rotaciona continuamente — Local (em si) e/ou Órbita (em centro).
- **Params principais:** speed `2` rad/s; local `true`; orbital `false`; centerX/Y `0`.
- **Quando usar:** Spinner de loading, planetas orbitando, hélice, ícone de carregamento, halo girando.
- **Combina bem com:** Pivot (eixo do Local), Shape, Render, DistributeCircle, Stagger.
- **Gotchas:** speed em rad/s. Local+Orbital juntos = girar enquanto orbita.

### wiggle — Wiggle (Tremor)
- **Tipo:** modifier
- **Função em 1 linha:** Tremor 2D suave (Perlin) na posição x/y, seed independente por instância.
- **Params principais:** amount `30` (0–200) px; speed `5` (0–20).
- **Quando usar:** Personagem nervoso, fogo crackling, idle animation, texto tremendo, partículas vivas.
- **Combina bem com:** Shape, Falloff (tremer perto do mouse), Spring (suavizar), Mixer.

### oscillator — Oscillator (Oscilador)
- **Tipo:** modifier
- **Função em 1 linha:** Onda periódica num canal com defasagem por índice; também emite value e pulse.
- **Outputs:** out(shape); value(value); pulse(pulse) — zero-crossing.
- **Params principais:** channel `y`; wave `sine` (sine/square/triangle/saw); amplitude `50`; frequency `1` Hz; phaseStagger `0.5`.
- **Quando usar:** Onda em fila, breathing, heartbeat, blink (square), zigzag, equalizer fake.
- **Combina bem com:** CloneLinear/Distribute (índices), Mixer, Spring, Stagger.
- **Gotchas:** Aditivo ao canal. Pulse dispara `edge='enter'` no zero-crossing+.

### stagger — Stagger (Escalonamento)
- **Tipo:** modifier
- **Função em 1 linha:** Rampa min→max num canal por índice, com easing — ADITIVO.
- **Params principais:** channel `y` (x/y/rotation/scale/size); min/max `-100/100`; easing `linear`; reverse `false`.
- **Quando usar:** Wave em fila, queda escalonada de notas, escada subindo, domino, text reveal letra-por-letra.
- **Combina bem com:** CloneLinear/Text/Distribute, Mixer, Falloff.
- **Gotchas:** Aditivo (soma). Sem instâncias indexadas, sem efeito.

### delay — Delay (Atraso em Cascata)
- **Tipo:** modifier
- **Função em 1 linha:** Amostra valores PASSADOS de um canal — cada instância vê o valor de `index*perIndex` segundos atrás.
- **Params principais:** channel `y`; perIndex `0.05s`; maxDelay `2s`.
- **Quando usar:** Onda viajando em fila (cobra), motion trail, eco de animação, snake reveal, hair flow.
- **Combina bem com:** Oscillator/Wiggle/Noise (algo precisa mudar upstream), CloneLinear, Spring.
- **Gotchas:** Sem movimento upstream, nada acontece. Limite 256 amostras/instância.

### noise — Noise (Ruído Perlin)
- **Tipo:** modifier
- **Função em 1 linha:** Perlin 2D num canal — valor coerente baseado em (x, y) e tempo.
- **Params principais:** channel `y`; amplitude `60`; scale `0.01` (0.001–0.1); timeScale `0.5`; seedOffset `0`.
- **Quando usar:** Topografia em grade, plantas balançando, terreno orgânico, brilho coerente, deformação de fila.
- **Combina bem com:** DistributeGrid (terreno), Stagger, Mixer, VectorField.
- **Gotchas:** Depende da posição (x, y) — sem variação espacial, mesmo valor pra todos.

### spring — Spring (Mola)
- **Tipo:** modifier
- **Função em 1 linha:** Suaviza mudanças num canal simulando mola (tensão + atrito) — overshoot/bounce.
- **Params principais:** channel `y`; tension `8` (0.5–60); friction `1.5` (0.1–20).
- **Quando usar:** Mascote seguindo mouse com bounce, UI popando, peso em transitions, balanço, elastic reveal.
- **Combina bem com:** LookAt, FollowTarget, Oscillator (amortece picos), qualquer behaviour que muda canal.
- **Gotchas:** SÓ AGE sobre alvos que mudam. State por índice. Reset em time regression.

### lookAt — Look At (Olhar Para)
- **Tipo:** modifier
- **Função em 1 linha:** Rotaciona pra apontar pra alvo (cursor ou fixo), com damping opcional.
- **Outputs:** out(shape); angles(value); target(point)
- **Params principais:** mode `mouse`/`fixed`; targetX/Y `0/0`; offset `0`; smoothing `0` (0–0.99).
- **Quando usar:** Olhos seguindo cursor, ponteiro/seta, torreta de tower defense, girassóis seguindo sol, agulhas de bússola.
- **Combina bem com:** Shape, Duplicator+Distribute (vários olhares), Spring (suavizar).

### followTarget — Tentacle (Tentáculo)
- **Tipo:** source
- **Função em 1 linha:** Cadeia de N segmentos perseguindo alvo — Livre ou Ancorado (FABRIK), com física Verlet opcional.
- **Outputs:** out(shape) — pontos isPoint com tangente.
- **Params principais:** count `14`; segLength `26`; target `mouse`/`fixed`; anchor `false`; damping `0.25`; maxAngleDeg `180`; inertia `0` (>0 = jiggle); friction `0.1`, gravityY `0`.
- **Quando usar:** Tentáculo de polvo, cabelo/cauda, braço de robô IK, planta ao vento, corda pendurada, cobra perseguindo.
- **Combina bem com:** Duplicator + Shape, Pivot (anchorX=-1 conecta), Modulate (cor por segmento), Stagger (taper de scale).
- **Gotchas:** É source. Devolve pontos isPoint — pra renderizar plugue Duplicator+Shape. Time scrub reseta.

### proceduralWalker — Procedural Walker
- **Tipo:** source
- **Função em 1 linha:** Personagem que segue mouse/fixo com pernas que dão passos (IK FABRIK por perna, gait alternado/wave).
- **Outputs:** out(shape) — `legs × segsPerLeg` pontos isPoint.
- **Params principais:** body `mouse`; legs `4`; segsPerLeg `4`; legLength `110`; legSpread `25`; restRadius `80`; stepDistance `45`; stepDuration `0.22s`; stepArc `30`; gait `alternating`/`wave`.
- **Quando usar:** Mascote andante (aranha, polvo, robô), criatura procedural reativa, IK demo, character em landing, NPCs.
- **Combina bem com:** Duplicator + Shape, Modulate (`divisor=segsPerLeg`), Paint, Spin.
- **Gotchas:** Saída flatten — use `i % segsPerLeg` pra identificar segmento.

### audioReact — Audio React (Áudio Reativo)
- **Tipo:** source
- **Função em 1 linha:** Microfone via Web Audio + AnalyserNode → N pontos horizontais com Y proporcional à amplitude FFT.
- **Outputs:** out(shape) com `amplitude`; value(value) média; bass(value) primeiras 4 bandas.
- **Params principais:** bands `32`; smoothing `0.7`; gain `1.5`; width `600`, height `200`.
- **Quando usar:** Music visualizer estilo equalizer, mascote dançando, intros reativas, VJ live, react-to-claps.
- **Combina bem com:** Duplicator + Shape, Paint/NumberRangeToColor, Spring, bass → Modulate/NumberRange.
- **Gotchas:** Precisa gesto do usuário ("Ativar microfone"). Antes disso, placeholder.

### vectorField — Vector Field (Campo Vetorial)
- **Tipo:** modifier
- **Função em 1 linha:** Desloca instâncias seguindo flow gerado por Perlin (ângulo do ruído).
- **Params principais:** amount `40`; scale `0.008`; timeScale `0.3`; turns `1`; seedOffset `0`.
- **Quando usar:** Enxames (peixes, pássaros), redemoinho de fumaça, vento em pétalas, fluxo em grade, campos magnéticos.
- **Combina bem com:** DistributeGrid/Random, Paint, ParticleEmitter, Noise.
- **Gotchas:** Não integrado no tempo (cada frame é amostragem independente).

---

# Physics (Físicas)

> Em geral, ordem importa: campos aplicam FORÇA (drag depois pra estabilizar). PinConstraint sobrescreve — coloque por último.

### attractorField — Attractor Field
- **Tipo:** modifier
- **Função em 1 linha:** Move instâncias em direção a um ponto-alvo com falloff radial.
- **Inputs:** in(shape); target(point) — no mode=input.
- **Params principais:** mode `mouse`/`fixed`/`input`; strength `80` (negativo=repel); radius `200`; falloffPower `2`; repel `false`; targetX/Y `0`.
- **Quando usar:** Orbe que puxa folhas até varinha, buraco negro, escudo repulsivo, planeta puxando lua (combo com vortex).
- **Combina bem com:** Point/Mouse como target, VortexField (órbita), DragField (estabilizar), CollisionEvent.

### vortexField — Vortex Field
- **Tipo:** modifier
- **Função em 1 linha:** Força tangencial perpendicular ao raio — orbita em torno do centro.
- **Params principais:** mode `mouse`/`fixed`; strength `80`; radius `200`; clockwise `true`; centerX/Y `0`.
- **Quando usar:** Tornado, espiral de fim de fase, portal mágico, redemoinho em ralo, turbilhão de folhas.
- **Combina bem com:** AttractorField no mesmo centro (espiral pra dentro), DragField, CurlNoiseField, pulse trigger.
- **Gotchas:** Sem modo input. Força tangencial pura — mantém distância.

### windField — Wind Field
- **Tipo:** modifier
- **Função em 1 linha:** Força direcional uniforme com rajadas senoidais Perlin.
- **Params principais:** angle `0°`; strength `30`; noise `0.3`; gust `1`.
- **Quando usar:** Vento em árvore (galho softBody), bandeira (verletRope), chuva oblíqua, neve, cabelo em corrida.
- **Combina bem com:** BuoyancyField (balão lateral), CurlNoiseField (turbulência), VerletRope/SoftBody, DragField.
- **Gotchas:** Mesma rajada pra todas instâncias no mesmo frame.

### dragField — Drag Field
- **Tipo:** modifier
- **Função em 1 linha:** Reduz velocidade implícita (delta entre frames) — frição.
- **Params principais:** coefficient `0.15` (0–1).
- **Quando usar:** Projéteis perdendo energia, partículas desacelerando, gelo com pouca fricção, freio depois de Attractor.
- **Combina bem com:** Attractor/Vortex/Wind upstream (drag freia). NÃO com Boids (já tem clamp) nem SoftBody (já tem friction).
- **Gotchas:** Estima velocidade via `inst.index` no `node._prev` — index estável é crítico. Coloque DEPOIS dos campos que aceleram.

### curlNoiseField — Curl Noise Field
- **Tipo:** modifier
- **Função em 1 linha:** Flow field divergence-free (curl do Perlin) — turbulência tipo fluido.
- **Params principais:** strength `30`; scale `0.005`; timeScale `0.2`; seed `0`.
- **Quando usar:** Fumaça subindo, correnteza, névoa mística, rastros de vapor, poeira em redemoinhos.
- **Combina bem com:** WindField (bias direcional), BuoyancyField, particle spawner, DragField.
- **Gotchas:** Densidade preservada (divergence-free) — partículas não se acumulam.

### buoyancyField — Buoyancy Field
- **Tipo:** modifier
- **Função em 1 linha:** Empuxo vertical constante + oscilação lateral senoidal por instância.
- **Params principais:** strength `60` (-200..200; <0 afunda); wobble `15`; wobbleSpeed `2`.
- **Quando usar:** Bolhas em refrigerante, balões, fagulhas subindo, folhas afundando.
- **Combina bem com:** WindField (vento horizontal), DragField, CollisionEvent (bolhas estouram).
- **Gotchas:** Wobble depende de `inst.seed ?? i`.

### boids — Boids (Flocking)
- **Tipo:** modifier
- **Função em 1 linha:** Agentes seguindo regras de Reynolds (separação, alinhamento, coesão).
- **Params principais:** perception `80`; separation `1.5`; alignment `1.0`; cohesion `0.8`; maxSpeed `80`; maxForce `50`.
- **Quando usar:** Cardume, enxame de morcegos, bando de pássaros, swarm de inimigos.
- **Combina bem com:** AttractorField (líder a perseguir), CurlNoiseField (turbulência), shape triângulo (rotation aponta cabeça).
- **Gotchas:** O(N²) — pesado >100 agentes. Spawn quebra continuidade.

### collisionEvent — Collision Event
- **Tipo:** modifier (sensor)
- **Função em 1 linha:** Detecta sobreposições por `size` e emite pulse + contact + count.
- **Outputs:** out(shape) pass-through; pulse(pulse); contact(point); count(value).
- **Params principais:** threshold `1.0` (>1 detecta antes do contato).
- **Quando usar:** Bolas de bilhar (som ao bater), partículas explodindo no impacto, contador de pinball, faíscas no contato.
- **Combina bem com:** pulse → trigger SFX, contact → spawner, simuladores upstream (softBody, verletRope, boids).
- **Gotchas:** O(N²). NÃO resolve colisão (sem bounce físico).

### verletRope — Verlet Rope
- **Tipo:** source
- **Função em 1 linha:** Corda Verlet+PBD de N partículas com gravidade e âncoras.
- **Outputs:** out(shape) — pontos isPoint com rotation tangente.
- **Params principais:** count `20`; segLength `18`; anchorX/Y `0/-150`; gravityY `200`; pinFirst `true`; pinLast `false`; iterations `8`; friction `0.02`.
- **Quando usar:** Corda de ponte, rabo de gato, lasso (pinFirst+anchor=mouse), teia, cabelo simples, antena.
- **Combina bem com:** PinConstraint (override pontas), AttractorField (puxar meio), CollisionEvent, Duplicator + Shape.
- **Gotchas:** É source. Não tem visual — combine com Duplicator+Shape. Reinit se count mudar.

### softBody — Soft Body XPBD
- **Tipo:** modifier
- **Função em 1 linha:** Mesh deformável com distance + bend constraints (XPBD), gravidade, elasticidade.
- **Params principais:** topology `ring`/`chain`; gravityY `300`; friction `0.02`; stretchStiff `1.0`; bendStiff `0.3`; iterations `8`.
- **Quando usar:** Slime que pula, massa de pão, bandeira (chain + pin + wind), blob mascote, bolha de água.
- **Combina bem com:** PinConstraint (ancorar bordas), WindField/CurlNoise, shape source pra rest pose, CollisionEvent.
- **Gotchas:** Rest length capturado no 1º frame ou quando N muda. Aplica gravidade INTERNAMENTE.

### pinConstraint — Pin Constraint
- **Tipo:** modifier
- **Função em 1 linha:** Trava instâncias selecionadas (por índice) em posição alvo.
- **Inputs:** in(shape); target(point) — no mode=input.
- **Params principais:** which `first`/`last`/`both`/`all`/`indices`; indexList `'0'`; mode `fixed`/`mouse`/`input`; targetX/Y `0`.
- **Quando usar:** Ponta de corrente no teto, bandeira em dois mastros, marionete seguindo mouse, bordas de softBody fixas.
- **Combina bem com:** VerletRope (override pontas além de pinFirst/Last), SoftBody (fixar bordas), Point.
- **Gotchas:** Sobrescreve COMPLETAMENTE — coloque DEPOIS do simulador. Teleporte instantâneo.

---

# Vary (Variar)

### random — Random (Aleatório)
- **Tipo:** modifier
- **Função em 1 linha:** Soma valor aleatório fixo (determinístico por seed+index) num canal.
- **Params principais:** channel `y`; min/max `-50/50`; seed `1`.
- **Quando usar:** Estrelas com tamanhos variados, partículas com rotação inicial, jitter em grade, alturas variadas em fila.
- **Combina bem com:** DistributeGrid/Random, CloneLinear, Falloff, Modulate.
- **Gotchas:** Aditivo. Sem index ≠ 0, todas variam igual.

### numberRange — Map Range (Remapear Faixa)
- **Tipo:** modifier
- **Função em 1 linha:** Remapeia canal-fonte para canal-destino com easing por potência.
- **Params principais:** sourceChannel `index` (index/x/y/rotation/scale/size); destChannel `y`; sMin/sMax `0/1`; dMin/dMax `-120/120`; easing `1` (power).
- **Quando usar:** X→scale (zoom progressivo), index→tamanho (gradient), Y→rotação, gráfico de dados.
- **Combina bem com:** Distribute (fonte de index), Stagger, NumberRangeToColor (mesma lógica pra cor), DataSource.
- **Gotchas:** SOBRESCREVE (não soma). `sourceChannel: index` usa 0..1 normalizado.

### modulate — Modulate (i mod N)
- **Tipo:** modifier
- **Função em 1 linha:** Atribui N valores diferentes baseado em `index mod divisor` — padrões cíclicos.
- **Params principais:** channel `scale`; divisor `2` (2–12); values `[1, 0.5]` (cresce com divisor).
- **Quando usar:** Xadrez (pares grandes, ímpares pequenos), alternância A/B/C, pernas de walker (`i mod 2`), notas musicais.
- **Combina bem com:** Distribute, CloneLinear, ProceduralWalker (`divisor=segsPerLeg`), ColorArray.
- **Gotchas:** SOBRESCREVE. Array `values` cresce automaticamente com divisor.

### clamp — Clamp (Limite)
- **Tipo:** modifier
- **Função em 1 linha:** Limita canal entre [min, max].
- **Params principais:** channel `x`; min/max `-200/200`.
- **Quando usar:** Conter partículas, limitar tremor pra não escapar, segurar scale, evitar overshoot de spring.
- **Combina bem com:** Wiggle/Noise (atenuar amplitude), Spring (limitar overshoot), ParticleEmitter (boundary).
- **Gotchas:** Honra falloff (lerp). Se min>max, troca automaticamente.

---

# Combos canônicos (atalhos para inspiração de tutos)

| Combo | O que produz |
|---|---|
| `Shape → Spin → Render` | Estrela giratória — primeiro tuto, hello world. |
| `DistributeCircle.points → Duplicator + Shape → Render` | Anel de ícones / halo. |
| `DistributeCircle → Duplicator → Spin + Stagger` | Loader em onda circular. |
| `Text → Stagger(channel=y) → Wiggle → Render` | Logo letra-por-letra com idle. |
| `Skeleton → IkFabrik(mouse) → SkeletonRender` | Tentáculo IK seguindo cursor (debug visual). |
| `ParticleEmitter → CurlNoiseField → Paint → Render` | Fumaça orgânica. |
| `VerletRope + AnchorMouse → Duplicator + Circle → Render` | Lasso que segue cursor. |
| `ProceduralWalker → Duplicator + Shape → Modulate(divisor=segsPerLeg) → Render` | Mascote andante com pernas coloridas. |
| `DistributeGrid → Duplicator + Shape → Noise(channel=y) → Render` | Terreno topográfico animado. |
| `LFO → Counter → SampleHold → ColorArray` | Cor que troca a cada beat. |
| `Falloff(followMouse) → Wiggle → Render` | Tremor local no hover. |
| `AudioReact → Duplicator + Rectangle + NumberRangeToColor → Render` | Equalizer reativo. |
| `Shape A → Morph(t=LFO) ← Shape B → Render` | Logo se transformando em outro em loop. |
| `softBody(ring) + WindField + PinConstraint(which=first)` | Bandeira tremulando. |

# Padrões importantes a lembrar ao escrever tutos

- **`points` é porta não-default** no Duplicator — sempre destacar a conexão `Distribute.points → Duplicator.points`.
- **Stagger/Random/Modulate são aditivos** — Shape sozinho não muda. Precisa de instâncias com `index` (CloneLinear, Distribute, Text…).
- **`numberRange` SOBRESCREVE** — diferente do par (aditivo).
- **Falloff só aparece se algo downstream o respeitar** — Move/Wiggle/Paint/Spin etc. Falloff sozinho não muda nada visível, só desenha overlay rosa quando o nó é selecionado.
- **Pivot só tem efeito se há rotação ou escala downstream** — Pivot + forma estática = invisível.
- **Spring/Delay/MotionTrail/SlitScan precisam de movimento upstream** — formas estáticas → efeito invisível.
- **TimeRemap muda upstream, não downstream** — coloque-o ENTRE o que você quer remapear e o Render.
- **Filtros multiplicadores** (DropShadow ×2, Mirror ×2/4, RgbSplit ×3, Bloom ×passes, MotionTrail ×length, SlitScan ×samples) compõem rápido → cuidado com performance.
- **Group precisa de OutputProxy** dentro pra emitir; cada OutputProxy vira uma porta nomeada.
- **Conversões implícitas** funcionam pra maioria dos pares — não vale a pena listar adaptadores manuais a menos que o tuto exija (ver `CONVERSIONS` em helpers.js).
- ⚠ **Os "campos" de Physics (Buoyancy/Wind/Attractor/Vortex/CurlNoise/Drag) NÃO são forças integradas** — todos computam `output = input + offset_por_frame` e não acumulam estado entre frames. Aplicados em ParticleEmitter/VerletRope (que mantêm posição interna e ignoram inputs downstream), o offset vira **deslocamento visual constante**, não aceleração. Componentes time-varying (Buoyancy wobble, Wind noise, Curl) animam por terem `sin(time)`/`perlin(time)` dentro, mas não causam drift contínuo. Pra movimento real de subida/queda: use a **Gravidade do próprio Particle Emitter** (negativa = sobe). Pra perseguição de mouse: promova `anchorX/Y` do simulador a socket e ligue um MousePosition. DragField é exceção parcial: tem `node._prev` próprio e só "funciona" se houver movimento upstream pra dampear.
