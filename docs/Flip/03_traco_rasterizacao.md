# Flip §03 — O traço: rasterização, o tripé, a mordida e o fix

> **Este é o doc definitivo do rasterizador de traço do Flip.** Ele consolida: a saga das 7
> rodadas (HANDOFF_flip_impl), a análise linha-a-linha do shader do GP 5.2, a evidência da
> comunidade Blender, o estado da arte externo (`04_alem_do_blender.md` §1), e a análise
> adversarial (3 lentes) do fix. **Quem for mexer em `ph2d-flip-render` lê este doc INTEIRO
> antes.** Arquivos nossos: `crates/ph2d-flip-render/src/shaders/flip.wgsl` · `pipeline.rs` ·
> `neighbors.rs` · `pack.rs` · `tests/gpu_render.rs`. Referência GP:
> `~/Downloads/blender-5.2-grease-pencil-ref/`.
>
> ## 🟩 ESTADO: a mordida está MORTA (wave WT fechada 2026-07-12; pendente o smoke do Enio)
>
> A cobertura do traço é agora a **UNIÃO GLOBAL da polilinha**, num único passe. O que a
> spec previa (janela `p0`/`p3`) fechou a classe *quina quebrada*, mas o smoke sintético
> revelou uma **segunda classe** que a análise tinha subestimado (a "auto-aproximação
> não-adjacente" — §4.2): o fix final tem **quatro peças**, não uma. Resultado: 15 testes GPU
> verdes (debug e release), **5 mutações provadas**, custo real de 1.7 ms para um traço de
> 4000 pontos. O que mudou em relação à spec original está marcado com **[REVISÃO]**.

Sumário: §1 A arquitetura e o invariante · §2 O GP por dentro (o que cada peça faz) ·
§3 A mordida (mecanismo provado) · §4 O fix IMPLEMENTADO (4 peças) · §5 O oráculo de
aparência + a bateria · §6 O que sobrou e os kill-criteria · §7 Antialiasing ·
§8 Refinos futuros do traço.

---

## §1 — A arquitetura e o invariante central

O traço é uma **fita de quads por segmento** expandida no vertex shader (vertex pulling de
storage buffers; sem geometry shader), com a forma final decidida no fragment por **cobertura
analítica**: a distância do pixel à linha-de-centro do segmento, clampada, alimenta o perfil
de hardness. O estado do pipeline é o **tripé do GP 2D** (validado nas rodadas 5–7 contra 4
classes de artefato — bead, spike/bowtie, escamado, acúmulo):

1. **Fita conectada por miter + `miter_break`** — segmentos adjacentes compartilham o vértice
   de junção (abutam, não sobrepõem). Virada > 120° (`-dot(dir_in, dir_out) > 0.5`) NÃO
   mitra: o offset fica na perpendicular do próprio segmento e o quad **estende `r` ao longo
   da linha** — nunca dobra sobre si (fim do bowtie), e a extensão cobre o disco da junção.
2. **Depth por-STROKE + GREATER estrito + write-depth** (clear = 0.0) — a 2ª face no mesmo
   pixel é DESCARTADA, não misturada → zero acúmulo premult. Default do GP: *"the stroke
   cannot overlap itself"* (`gpencil_vert.glsl:96`).
3. **Discard de fragmento com `alpha < 0.001`** (`gpencil_frag.glsl:549`) — fragmento
   ~transparente não escreve depth, senão o canto vazio de um quad fura a geometria que
   chega depois (era o "escamado").

**O invariante não-documentado que faz tudo isso funcionar** (a descoberta central do estudo):

> *Onde dois quads do mesmo traço cobrem o mesmo pixel, ambos DEVEM computar a MESMA
> máscara* — porque o depth first-wins escolhe um vencedor arbitrário (o primeiro no buffer),
> e isso só é invisível se tanto faz quem vence.

O GP constrói a máscara para satisfazer o invariante em quase todo lugar: no caso ROUND, a
distância clampada além do endpoint vira distância radial ao **ponto compartilhado** (campo
independente do segmento — contínuo através da bissetriz); nos tipos BEVEL/MITER, as "cunhas"
com p0/p3 fazem os dois lados computarem o MESMO plano de corte (`draw_grease_pencil_lib.glsl:115-142`).
**O único lugar onde o invariante quebra é ROUND + quina quebrada + hardness < 1 — a mordida (§3).**

Papel exato do discard (não é otimização): com hardness = 1 a máscara é binária; onde o campo
do vencedor é 0, o discard impede a escrita de depth e o perdedor pinta em seguida — **no caso
binário, first-wins + discard produzem a união exata** (a menos da franja de AA de ~1px, cujos
alphas intermediários escrevem depth — resíduo sub-pixel que o MSAA/SMAA do GP dilui). O
discard é o mecanismo de correção geométrica do sistema; removê-lo ou mudar o threshold sem
casar com o `0.999` do perfil binário quebra caps/quinas/bevels (provado por mutação na
rodada 7: desvio 254 no oráculo).

## §2 — O GP por dentro: o que cada peça faz (mapa de consulta)

Referência viva: `source/blender/draw/` no recorte. Números = arquivo:linha do 5.2.

| Peça | Onde | O que importa |
|---|---|---|
| Expansão da fita | `draw_grease_pencil_lib.glsl:669-727` | miter compartilhado = bissetriz exata; `miter_break` a 120° com clamp do limite a `cos(60°)` (esticão máx 2r — o "miter infinito" de SVG não existe no GP); extensão `line*x` em break E round-caps (1 bit de sinal por cap) |
| Máscara analítica | `lib:65-146` | cápsula clampada; `both_round` (default!) **ignora p0/p3** e retorna o campo do próprio segmento; cunhas BEVEL/MITER usam p0/p3 p/ consistência |
| Perfil de hardness | `lib:29-41` | `hard>0.999 → step(1e-8, d)`; senão `smoothstep(0,1, d^mix(0,10,1-hard))` |
| Depth 2D | `gpencil_vert.glsl:81-97,144` | por-stroke `(sid+2)·2e-7`; **fill em `(sid+1)` — 1 quantum atrás do próprio traço**; flag `GP_STROKE_OVERLAP` troca p/ por-PONTO (auto-overlap com acúmulo — opção de material) |
| Estado do passe | `gpencil_cache_utils.cc:447-453` + `gpencil_engine_c.cc:821` | `WRITE_DEPTH \| BLEND_ALPHA_PREMUL \| DEPTH_GREATER` estrito; **depth clear = 0.0** (2D). Clear 1.0/LESS no 3D — inverter só um dos dois = tela vazia |
| Discard e ordem | `gpencil_frag.glsl:549-582` | alpha → scene-depth → mask → `gl_FragDepth` constante. A ordem importa (mask depois do depth-write escreveria depth onde a máscara obliterou). `frag_depth` mata early-Z (custo aceito) |
| Fade sub-pixel | `gpencil_frag.glsl:534-535` | `color *= smoothstep(0,1, thickness_unclamped)` — traço < 1px perde opacidade em vez de afinar; mata o aliasing de linha fina sem MSAA. **Barato; vale portar** |
| Sentinelas | 3 camadas | `mat=-1` no CPU → NDC degenerado no vertex → `p0==p1` como flag geométrica no fragment (teste `dist² < 1e-6` em px², EXATO porque os varyings são FLAT) |
| Cíclico | `lib:494-510` + cache | o padding de adjacência guarda o `stroke_id` da ponta oposta — o mesmo fetch de vizinho funciona pra aberta e fechada sem branch |
| Ponto único | `lib:512-517` | vira dot redondo à força — um port que descarte segmentos degenerados **apaga pontos isolados do desenho** |
| Dots/squares | `lib:283-355` + `frag:337-516` | 1 shape por segmento + dots resolvidos ANALITICAMENTE no fragment (interseção de cápsula desigual + loop com early-out em alpha>0.999) — o esquema "Ciallo" (§8) |

**O GP tem a mordida? SIM — latente, e convive com ela.** Três evidências independentes:

1. **Análise do shader:** o caminho quente (corner ROUND, default `GP_CORNER_TYPE_ROUND_BITS=0`)
   retorna a cápsula do próprio segmento ignorando p0/p3 (`lib:88-91`); na quina quebrada os
   quads estendidos se sobrepõem com o MESMO depth; first-wins pinta a queda radial do
   primeiro sobre o núcleo do segundo. Nada no pipeline evita.
2. **Issues da comunidade:** [#140075 "Grease Pencil Softness Artifacts"](https://projects.blender.org/blender/blender/issues/140075)
   (ABERTO, "Worked: Never"; dev do módulo: *"current limitation of how strokes are generated
   and drawn with transparency"*) · [#102927](https://projects.blender.org/blender/blender/issues/102927)
   (fechado como limitação: *"no clean corners with lower opacity"*) ·
   [#94252](https://projects.blender.org/blender/blender/issues/94252) (aberto desde 2021,
   soft pencil + self-overlap off = artefatos de borda).
3. **A resposta estrutural do próprio Blender (2025) ratifica o nosso caminho:** os **Corner
   Types** ([PR 143688](https://projects.blender.org/blender/blender/pulls/143688), Round/Sharp/Flat
   por ponto) resolvem a junção **no fragment com p0/p3** — sem fechar o caso ROUND macio.

Por que ninguém vê no Blender: hardness **default = 1.0** (caso binário = união exata fora da
franja de AA), viradas > 120° são raras em traço à mão densificado, a região é ≤ ~1 raio, e o
SMAA borra o *tell*. **No Flip o pincel macio é o caso comum — o esconderijo não serve. Aqui
divergimos do Blender de propósito, na direção que o próprio shader dele aponta.**

## §3 — A mordida: mecanismo provado

Numa virada > 120° em `p2` (segmentos A = `p1→p2`, B = `p2→p3`), os dois retângulos estendidos
se sobrepõem ~`r` ao redor de `p2` — de propósito, para cobrir o disco da junção. O fragment de
cada quad mede a distância clampada ao **PRÓPRIO** segmento: na zona de extensão de A (além de
`p2`), o campo de A é **radial centrado em `p2`**. Parte dessa zona está **sobre o núcleo de B**
(cobertura correta ≈ 1). Mesmo `sid` → mesmo depth → GREATER estrito: **A, desenhado primeiro,
vence todo pixel compartilhado cujo alpha sobreviva ao discard** — e pinta ali a SUA queda
radial intermediária.

```
                         p3
                        ↗
              núcleo de B (cobertura correta ≈ 1)
                      ↗
        ┌────────────╱━━━━━━━━┐ ← quad A, estendido r além de p2
        │        ···╱ ·  ▒▒▒  │
  p1 ═══╪═════════ p2 ·· ▒▒▒▒ │   ▒ = MORDIDA: pixels no núcleo de B,
        │  núcleo   `· ,  ▒▒  │       vencidos por A, que pinta a
        └───────────── ' ─────┘       SUA queda RADIAL (isolinhas ···,
                 isolinhas radiais       centradas em p2)
                 de A (t1 clampado)
```

Precisões que a verificação adversarial cravou (não confie na intuição aqui):

- **A fronteira visível da mordida é a iso-linha do DISCARD de A** (alpha = 0.001), não a
  aresta do quad: reta onde o corpo de B dobra sobre a fita de A, **arco centrado em `p2`** na
  zona de extensão. Coincide com a aresta do quad só em hardness/opacity altos. (Importa pro
  diagnóstico no smoke: com hardness→0 a fronteira fica bem DENTRO do quad.)
- **hardness = 1 esconde porque o caso binário é união exata fora da franja de AA (~1px)** —
  a franja ainda escreve depth com alphas intermediários; resíduo sub-pixel, invisível na prática.
- **Assimetria diagnóstica:** quem morde é sempre o segmento de índice menor — a mordida
  **troca de lado se a direção de desenho inverte**.
- **NÃO existe "pinch" na quina MITRADA com largura uniforme** (refutado com geometria de
  Voronoi: a aresta de miter compartilhada É a bissetriz dos dois raios a partir de `p2`; todo
  pixel do quad de A satisfaz `dist_A ≤ dist_B` → a cobertura mitrada JÁ é a união exata hoje).
  O resíduo mitrado real é de 2ª ordem e só aparece com **largura por-ponto** (taper), pela
  divergência de parametrização de raio — ver D1 no §4.

## §4 — O fix IMPLEMENTADO: a cobertura é a UNIÃO GLOBAL da polilinha

**A ideia central** (validada; era o coração da spec original): a cobertura de um fragmento
deixa de ser a distância ao *próprio* segmento e passa a ser a distância à **polilinha** —
`dn = min(dn_i)` sobre as cápsulas que alcançam o pixel. Como o perfil de hardness é monótono
decrescente, **min-distância ⇔ max-cobertura**: os quads que se sobrepõem num pixel passam a
computar o **mesmo** valor, e o depth first-wins volta a ser invisível (o invariante do §1 é
restaurado).

O que a análise adversarial tinha previsto em parte, e o oráculo provou por inteiro: **fazer
isso direito exige quatro peças.** As duas primeiras estavam na spec; as duas últimas são
**[REVISÃO]** — vieram do vermelho dos testes.

### 4.1 — Peça 1: a janela de sequência (`p0`/`p3`)

O vertex já busca os vizinhos de sequência para o miter — agora **exporta-os** como varyings
FLAT (`ss_p0`, `ss_p3`, `radii`), e o fragment inclui as duas cápsulas vizinhas no `min`.
Fecha a classe **quina quebrada** (`miter_break`): na sobreposição em `p2`, a janela de A
(`{prev,A,B}`) e a de B (`{A,B,next}`) contêm ambas `{A,B}` → mesmo mínimo → sem mordida.

Sentinela de borda: sem prev/next, o vizinho **coincide** com o próprio extremo; como os
varyings são FLAT (sem interpolação), a igualdade é exata e a cápsula degenerada é ignorada
(`len_sq < 1e-6 → +∞`). Um port que interpole esses varyings quebra a sentinela.

### 4.2 — Peça 2 [REVISÃO]: os vizinhos GEOMÉTRICOS (a classe que faltava)

**O oráculo mostrou que a janela ±1 NÃO basta.** No zigzag do smoke, o pixel (43,36) tinha a
GPU pintando `2` onde a união pede `254`: o **segmento 2 passa por baixo da borda macia do
segmento 0** — não-adjacentes. O quad de 0 cobre aquele pixel, sua máscara ali é fraquíssima
(`0.0046`, ou seja 1/255) mas **sobrevive ao discard, escreve depth e bloqueia o segmento 2**,
cujo núcleo (cobertura ~1) deveria pintar. Ou seja: **a borda quase-invisível de um segmento
apaga o miolo opaco de outro.** É a mesma doença, com alcance maior — e é o que mais salta aos
olhos no zigzag do Enio.

Chamar isso de "teto aceitável" (o K4 da spec) seria errado: **acontece em todo traço que volta
sobre si mesmo** — zigzag apertado, laço, letra, hachura, qualquer rabisco.

**A solução, num único passe:** a lista de vizinhos geométricos, pré-computada na CPU
(`neighbors.rs`), consumida pelo fragment.

- **CPU (no `pack`, que é cacheado por desenho):** para cada segmento, quais segmentos
  NÃO-adjacentes podem alcançar os pixels do seu quad. Critério **conservador, sem
  falso-negativo**: um pixel do quad de `i` está no máximo a `2·r_i` do eixo de `i` (o esticão
  do miter é limitado a 2×), e `j` só o influencia se `dist(pixel, j) < r_j`; pela desigualdade
  triangular basta `dist(seg_i, seg_j) < 2·r_i + r_j`. **O teste é ASSIMÉTRICO** — o raio do
  dono do quad entra DOBRADO — e essa assimetria é load-bearing no grid (pad de inserção `r_j`,
  pad de consulta `2·r_i`; usar o mesmo pad dos dois lados perde vizinhos mais GROSSOS que o
  dono, e a mordida volta em silêncio naqueles pixels).
- **GPU:** `seg_extra_range[gp]` (por segmento, via varying flat) → `(offset, count)` na lista
  `seg_extras` de pares `(a,b)` de pontos; o fragment soma essas cápsulas ao `min`. `count == 0`
  na esmagadora maioria dos traços ⇒ **custo zero**. Os storage buffers `points` e `seg_extras`
  ficam visíveis ao FRAGMENT (mudança de BGL em `pipeline.rs`).

**Custo (medido, release):** traço longo NORMAL (onda de 4000 pontos, não volta sobre si) =
**1.7 ms**; rabisco browniano patológico de 4000 pontos = 14 ms, limitado pelo `PAIR_BUDGET`.
O grid é ~linear (um teste prova equivalência exata com o par-a-par `O(n²)`).

**Degradações declaradas** (as duas são determinísticas e ficam onde não doem):
- `MAX_EXTRAS_PER_SEGMENT = 16` — num rabisco denso, dezenas de segmentos cruzam o mesmo; os
  16 mais próximos entram. **Desempate por índice é obrigatório** (não só por distância):
  dezenas empatam em distância 0, e sem o desempate o corte dependeria da ordem de descoberta
  → o mesmo desenho geraria buffers diferentes (determinismo/replay-hash).
- `PAIR_BUDGET` — teto de trabalho por traço; além dele os segmentos restantes ficam sem lista
  e voltam ao first-wins do GP. Só é atingido pelo borrão sólido, onde a mordida é invisível.

### 4.3 — Peça 3 [REVISÃO parcial]: uma única `capsule_dn` (o defeito D1, confirmado)

A verificação adversarial apontou (3 lentes independentes) e o **teste do taper provou**: se o
segmento próprio usar o `thickness` interpolado sobre o QUAD (que inclui as extensões) enquanto
os vizinhos usam a cápsula analítica, então **com largura por-ponto** (pressão de tablet, o caso
normal!) os dois quads que cobrem o mesmo pixel normalizam por raios diferentes — o invariante
quebra de novo e a mordida sobrevive em 2ª ordem. **Uma função, três (ou mais) chamadas**:

```wgsl
// raio efetivo = o interpolado pelo `t` CLAMPADO da cápsula (não pelo quad!)
fn capsule_dn(frag: vec2f, a: vec2f, b: vec2f, ra: f32, rb: f32) -> f32 {
    let ab = b - a;
    let len_sq = dot(ab, ab);
    if (len_sq < 1e-6) { return 1e9; }          // cápsula degenerada (sentinela)
    let t = clamp(dot(frag - a, ab) / len_sq, 0.0, 1.0);
    return length(frag - a - t * ab) / max(mix(ra, rb, t), 1e-4);
}
```

O varying `thickness` sobrevive, mas só para o fade sub-pixel (§4.4) — **não** entra mais na
máscara.

### 4.4 — Peça 4 [REVISÃO]: o par clamp+fade, e o AA de cobertura

Ao portar o **fade sub-pixel** do GP (`mask *= smoothstep(0,1, thickness_px)`) descobrimos que
ele **sozinho não faz nada**: uma linha de 0.35 px não cobre o centro de nenhum pixel e some
por completo (alpha 0). O GP tem um **par**: clamp de largura mínima (~1.3 px, usado na
geometria E na máscara) + fade pela espessura **não-clampada**. Juntos: a linha fina não afina
mais — ela **desbota**, preservando energia e matando o pisca/serrilhado ao mover e ao dar zoom.
Implementado como `MIN_WIDTH_PX = 1.3` no vertex (raios clampados) + `thickness` cru no varying.

E, no caminho, um bug de AA que estava lá desde o W1: a forma antiga
(`edge = 1 - smoothstep(1-aa, 1, dn)`) **subestima a cobertura** quando o traço é fino
(`aa = fwidth(dn) > 1`) — a linha de 1 px saía 10× mais fraca do que devia. A forma correta é a
**fração do pixel coberta**:

```wgsl
let edge = clamp(0.5 + (1.0 - dn) / aa, 0.0, 1.0);   // em dn=1 dá 0.5 = meio pixel
```

### 4.5 — O que NÃO mudou (o tripé intacto)

Vertex (miter/`miter_break`/extensões/caps/cíclico), depth por-stroke + **GREATER estrito**,
`discard` de `alpha < 0.001`, blend premult, o compositor por-camada, o `TessCache` do shell.
Uma blindagem nova: `safe_dir` no miter — um **ponto duplicado** (tablet repete amostra; o
smooth funde dois) fazia `normalize(0)` = NaN e **rasgava o traço**; agora o vizinho degenerado
é tratado como "sem vizinho".

**Descoberta sobre o discard:** com a união global, ele **deixou de ser load-bearing** para a
correção (a mutação "sem discard" não sangra mais). A razão: o fragmento que cobre o núcleo de
outro segmento agora tem máscara ALTA, não ~0 — a classe "fragmento transparente escreve depth
e fura o vizinho" desapareceu. Ele permanece porque (a) protege a degradação do cap/budget e
(b) evita escrever depth à toa. **Não afrouxe** — é barato.

### 4.6 — Alternativas, e por que a escalada NÃO foi necessária

A spec previa, como escalada, o **scratch com blend MAX** (Stencil-then-Cover contínuo). Ela
resolveria a mesma classe — mas custa **2 render passes por traço**: com ~300 traços num frame
são ~600 passes (~3 ms de CPU só de encoding), e o cache de frame não salva porque o traço
rasteriza em screen-space (zoom/pan invalidam). A janela geométrica entrega a **mesma união
global** com **zero passes extras** e custo O(1) por fragmento. A escalada fica registrada como
plano B se algum dia um caso patológico exigir (§8) — mas hoje ela seria mais lenta e mais
complexa, sem ganho de qualidade.

As demais (correção de alpha do Ciallo, dab-based, Vello stroke-expansion, polar stroking,
airbrush integral, depth por-ponto) seguem rejeitadas pelas razões do estudo — ver a tabela
em `04_alem_do_blender.md` §1.

## §5 — O oráculo de APARÊNCIA e a bateria (o que garante que não volta)

**A lição da rodada 7** (memória `feedback_oracle_must_model_appearance_not_implementation`):
o oráculo antigo modelava o first-wins — a IMPLEMENTAÇÃO — e ficou **verde com a mordida na
tela**. O novo (`gpu_render.rs`) modela o **OBJETO**:

> Um traço macio É a união dos discos varridos ao longo da polilinha. A cobertura num pixel é
> o perfil de hardness aplicado à MENOR distância normalizada às cápsulas de **todos** os
> segmentos (com o raio mínimo rasterizável e o fade sub-pixel — que também são aparência).
> Nada nele sabe de quads, depth, ordem de desenho ou discard.

Pixels ambíguos (a faixa de AA da borda e o limiar do discard) são pulados; **todo o resto do
alvo** é comparado, fundo incluso. Qualquer classe de artefato — mordida, bead, escama, spike,
acúmulo, buraco de junção, rasgo por NaN — é uma divergência da união, e portanto vermelha.

**Sequência obrigatória** (foi seguida; repita-a em qualquer mudança futura):
1. Troque/estenda o oráculo **primeiro** e prove que ele fica **VERMELHO** no código atual.
   (Foi: 4 testes vermelhos com desvio ~250/255 — a GPU pintava 2 onde a união pede 254.)
2. Só então mexa no shader, até o verde.
3. **Prove as mutações.** O oráculo só vale se elas sangram.

**A bateria (15 testes GPU, verdes em debug e release):**

| Teste | O que prova |
|---|---|
| `a_soft_broken_corner_matches_the_polyline_union` | **o zigzag do smoke** — quina quebrada + o segmento que volta por baixo |
| `a_tapered_broken_corner_matches_the_union` | o mesmo, com **largura por-ponto** (pega o defeito D1 — nenhum outro teste o vê) |
| `a_soft_hairpin_matches_the_polyline_union` | virada ~175°: o extremo da sobreposição A∩B |
| `a_closed_soft_star_seam_matches_the_union` | traço FECHADO com quinas afiadas: a costura do wrap ganha a janela |
| `a_soft_mitered_corner_matches_the_union` | arco mitrado: **regressão** (já era união — verde antes e depois; não é discriminante) |
| `soft_round_caps_are_unchanged_by_the_neighbor_window` | 2 pontos: a janela toda-sentinela não estraga as tampas |
| `a_duplicated_point_does_not_tear_the_stroke` | ponto repetido não vira NaN/rasgo (`safe_dir`) |
| `a_subpixel_thin_stroke_fades_instead_of_flickering` | o par clamp+fade: a linha fina desbota, não some nem pisca |
| `a_stroke_crossing_itself_is_a_clean_union_without_accumulation` | auto-cruzamento: união sem acúmulo premult |
| `a_soft_stroke_has_no_bead_at_the_joints` · `a_sharp_corner_is_a_round_join_without_an_outward_spike` · `newer_stroke_draws_over_older_at_crossing` · `filled_closed_stroke_renders_fill_under_stroke` · `hardness_controls_edge_falloff` · `straight_stroke_paints_a_band…` | o **tripé** e o resto do contrato — nenhum foi afrouxado |

**As 5 mutações provadas** (cada uma sangra):

| Mutação | Resultado |
|---|---|
| sem os vizinhos geométricos (`count = 0`) | **3 vermelhos** (zigzag, taper, duplicado) |
| sem a janela `p0`/`p3` (só o próprio segmento) | **5 vermelhos** |
| `GreaterEqual` no depth (o acúmulo) | **6 vermelhos** |
| sem o fade sub-pixel | 1 vermelho |
| sem o clamp de largura mínima | 1 vermelho |

Além da GPU: `neighbors.rs` tem **6 unit tests**, incluindo o que compara o grid com o
par-a-par `O(n²)` num rabisco de 180 segmentos (pegou 2 bugs reais durante a implementação:
o pad assimétrico e o desempate não-determinístico), e `pack_perf.rs` guarda o custo por ORDEM.

## §6 — O que sobrou (teto honesto) e os kill-criteria

**O que a união global NÃO cobre:**
- **Além do cap/budget** (rabisco patológico): os segmentos sem lista voltam ao first-wins do
  GP. Determinístico, e num borrão sólido a mordida é invisível.
- **A COR em auto-cruzamento** continua first-wins (o vencedor pinta com a cor DELE). A
  *cobertura* é a união (o buraco morreu), mas num traço com gradiente de cor por-ponto que
  cruza a si mesmo, o trecho cruzado mostra a cor do segmento de índice menor. É a semântica do
  GP; mudá-la exige o caminho de 2 passes (§4.6) e só vale se alguém reclamar.
- **Auto-sobreposição com acúmulo** (tinta que escurece ao passar duas vezes) **não existe** —
  é a flag *Self Overlap* futura (§8), não um bug.

**Kill-criteria para qualquer mudança futura no traço:**
- **K1 (oráculo):** a mudança precisa de um teste que fique **vermelho antes**. Sem isso, não
  comece.
- **K2 (tripé):** qualquer um dos testes de tripé vermelho ⇒ reverta e re-analise.
- **K3 (smoke ≠ harness):** oráculo verde e artefato na tela ⇒ o harness reproduz o mecanismo,
  não o contexto (memória `feedback_harness_reproduces_mechanism_not_context`): instrumente no
  app real (dump da janela + do `dn` no pixel apontado) ANTES de escrever código.
- **K4 (perf):** `pack_perf` é o guard de ordem. Se o preview travar num rabisco longo, o
  próximo passo é o **pack incremental** (o traço em curso cresce; só a cauda muda — recomputar
  só a janela ativa), não afrouxar o broadphase.

## §7 — Antialiasing (a decisão, com a história do GP como aviso)

O GP **precisa** de pós-AA (SMAA 1x, preset HIGH) porque o traço hardness=1 dele produz borda
`step` dura. A história é um aviso: o "AA Threshold" da UI é na verdade um **ganho do luma**
(threshold real fixo em 0.1); edge-detect em scene-referred linear perde bordas de baixo
contraste (#74938); shimmer temporal de linhas finas está aberto há 5 anos (#90321); e a
solução definitiva deles foi **SSAA no render** (4.5+), não mais SMAA.

**Decisão Flip:**

1. **Viewport: AA analítico como caminho primário — NÃO portar SMAA agora.** ✅ FEITO, e
   corrigido: a cobertura de borda é `clamp(0.5 + (1-dn)/fwidth(dn), 0, 1)` — a **fração do
   pixel coberta** (em `dn=1` dá 0.5). A forma antiga (`1 - smoothstep(1-aa, 1, dn)`)
   subestimava a cobertura em traço fino: a linha de 1 px saía 10× mais fraca (§4.4).
2. **Fade sub-pixel + clamp de largura mínima.** ✅ FEITO — e são um PAR: o fade sozinho não
   salva a linha fina (ela não cobre o centro de pixel nenhum e some). `MIN_WIDTH_PX = 1.3`
   na geometria/máscara + `thickness` cru (não-clampado) no fade. Teste:
   `a_subpixel_thin_stroke_fades_instead_of_flickering`.
3. **Export/render: o esquema de ACÚMULO do GP é a joia barata** — Halton(2,3) → gaussiana
   σ=0.284 truncada em ±0.93 (o `sqrt(0.284)` é contrato empírico, "needed to match EEVEE") →
   translação ortográfica sub-pixel → média corrente em `log2(c+0.5)` com snap de alpha opaco.
   8–16 amostras = SSAA gaussiano exato que resolve TUDO (fills, interseções). Em 2D-orto o
   jitter é uma translação exata — trivial.
4. **SMAA 1x como resolve opcional futuro** para fills/composição — **do reference
   implementation público (github.com/iryoku/smaa, licença MIT, LUTs incluídas)**, nunca da
   árvore GPL do Blender. Se portar: edge-detect com alpha no métrico (tinta escura sobre
   transparente é invisível pro luma da cor!), resolver os DOIS buffers com os mesmos pesos,
   filtro linear nas LUTs (nearest = quebrado silencioso).
5. **FXAA e MSAA: rejeitados** (FXAA erode linha fina; MSAA quantiza gradiente e multiplica
   bandwidth dos alvos fp16 por camada).
6. **Buffer de acumulação de borda macia precisa de canal alpha decente** — R11G11B10 tinge
   (amarelo) o acúmulo de borda macia (#80038); nosso caminho premult RGBA16F está correto.
7. **Determinismo de varying:** fator que deve ser exatamente 1.0 interpolado entre vértices
   pode chegar ≠1 conforme o hardware (#156278, fix = clamp perto da identidade) — passar
   fatores exatos como `flat` ou clampar.

## §8 — Refinos futuros do traço (backlog qualificado)

- **Pack INCREMENTAL do traço em curso** (o gatilho: se o preview travar num rabisco muito
  longo). O traço cresce ponto a ponto e só a cauda muda (o `active_smooth` congela o resto):
  recomputar só a janela ativa + os segmentos afetados, em vez do broadphase inteiro por frame.
  Hoje: 1.7 ms para um traço normal de 4000 pontos; 14 ms no rabisco patológico (limitado pelo
  `PAIR_BUDGET`).
- **Escalada de 2 passes (scratch + blend MAX)** — só se um caso patológico exigir a união
  ALÉM do cap, ou se a semântica de COR em auto-cruzamento incomodar (§6). Custa ~2 render
  passes por traço; hoje seria mais lenta e mais complexa, sem ganho de qualidade (§4.6).
- **Flag *Self Overlap* por pincel/traço** (1 bit + 1 linha: depth por-PONTO em vez de
  por-stroke) — auto-sobreposição com acúmulo, para marker/build-up expressivo. É a resposta
  canônica ao "parte nova por cima" do smoke da 3ª rodada.
- **Corner types Round/Sharp(miter-limit)/Flat por ponto** (paridade SVG/GP-2025): as cunhas
  BEVEL/MITER do `segment_mask` com p0/p3 — a mesma janela do fix (§4.1) já carrega os dados.
- ~~**Pincel pontilhado (dots/squares) estilo Ciallo/GP**~~ — **LANDOU 2026-07-25**
  (`PH2D_FLIP_TIP_SMOKE=1`): `FlipStroke::tip` = `Continuous`/`Dots`/`Squares` + `dot_spacing`
  (MUNDO). O fragment recorta a cobertura por uma MÉTRICA — a distância normalizada ATRAVÉS
  (`dn`) ganha um termo AO-LONGO do arco (`da`), e a conta é um DISCO (`√(dn²+da²)`, Euclidiano)
  ou um QUADRADO (`max(dn, da)`, Chebyshev). Espaçamento por ARC-LENGTH (buffer `arc_len`
  cumulativo por-ponto, binding 6; o vertex lê o início e soma `|b−a|`), imune à densidade de
  input E ao zoom. `Continuous` **não toca `dn`** ⇒ byte-idêntico. **A depth é por-TRAÇO** (o
  tip não a muda), então contas sobrepostas numa quina são first-wins (união), nunca acumulam
  — o oposto da armadilha do *Self Overlap*. Seletor **Tip** + slider **Spacing** na seção
  Brush (Draw). `FLIP_SCHEMA` 8→9, `PROJECT_SCHEMA` 29→30. Gates GPU
  (`dots_carve_gaps_that_a_continuous_line_does_not` red-first mutação-provado +
  `squares_cover_more_area_than_round_dots`).
- **Pincel airbrush analítico (Ciallo):** falloff por integral em forma fechada
  `A(y) = 1 − exp(−2αc·sqrt(R²−y²))` — semântica de acúmulo físico; casa com a flag Self
  Overlap. (Fórmulas do paper/tutorial; código do CialloResearch é GPL-3 — só comportamento.)
- **Variante SDF do caminho (b)** — hardness re-editável por uniform (scratch de distância +
  MIN). Interessante quando houver "estilo de traço" re-aplicável.
- **Budget de depth:** quantum 2e-7 ≈ 5M índices por alvo. O Flip rasteriza POR CAMADA/frame
  (sids re-baseiam a cada pack) — inatingível na prática; documentado como custo aceito junto
  com early-Z desligado.
- **Perf de referência:** a lição das 3 ordens de magnitude do GP pré-2.83 (T57829) foi
  **1 batch por objeto, nunca estado por-stroke** — a nossa arquitetura (1 upload SoA por
  drawing + draw por camada) já nasce do lado certo; manter.
