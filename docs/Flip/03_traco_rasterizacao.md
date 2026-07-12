# Flip §03 — O traço: rasterização, o tripé, a mordida e o fix

> **Este é o doc definitivo do rasterizador de traço do Flip.** Ele consolida: a saga das 7
> rodadas (HANDOFF_flip_impl), a análise linha-a-linha do shader do GP 5.2, a evidência da
> comunidade Blender, o estado da arte externo (`04_alem_do_blender.md` §1), e a análise
> adversarial (3 lentes) do fix. **Quem for mexer em `ph2d-flip-render` lê este doc INTEIRO
> antes.** Arquivos nossos: `crates/ph2d-flip-render/src/shaders/flip.wgsl` · `pipeline.rs` ·
> `tests/gpu_render.rs`. Referência GP: `~/Downloads/blender-5.2-grease-pencil-ref/`.

Sumário: §1 A arquitetura e o invariante · §2 O GP por dentro (o que cada peça faz) ·
§3 A mordida (mecanismo provado) · §4 O fix, ranqueado e especificado · §5 O oráculo
corrigido + plano de testes · §6 Execução em 1 iteração + kill-criteria · §7 Antialiasing ·
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

## §4 — O fix, ranqueado e especificado

### 4.1 — 1º: fragment com janela de vizinhos p0/p3 (RECOMENDADO agora)

**Ideia:** a cobertura do fragment passa a ser a **união local das 3 cápsulas** — a do próprio
segmento e as dos dois vizinhos. Como o perfil é monótono decrescente na distância,
`min-distância ⇔ max-cobertura`:

```
dn = min(dn_prev, dn_own, dn_next)   →   mask = hardness_mask(dn, hardness, aa)
```

**Consistência entre vencedores (o coração):** na sobreposição da quina em `p2`, a janela de A
é `{prev(A), A, B}` e a de B é `{A, B, next(B)}` — **a interseção `{A, B}` contém os dois campos
relevantes do disco da junção**. Enquanto só A e B alcançarem o pixel, ambos computam o MESMO
`min` e o first-wins volta a ser invisível. Casos: quina quebrada isolada = **exato**; quina
mitrada = mata o resíduo de taper; hairpin de 3 pontos = exato; caps = sentinela `p0==p1`
(cápsula degenerada ignorada); closed = o vertex já faz wrap (a costura ganha a janela de
graça); traço reto = **byte-idêntico ao atual com largura uniforme** (goldens intactos).

**⚠️ A correção OBRIGATÓRIA da spec (achada 3× na verificação adversarial): `dn_own` TEM de
usar a MESMA função de cápsula dos vizinhos.** O `dn` atual vem do varying `thickness`
interpolado sobre o quad ESTENDIDO (afim na coordenada axial, extensões incluídas); as cápsulas
vizinhas usariam `mix(ra, rb, t_clampado)` analítico. Com **largura por-ponto** (pressão de
tablet — o caso normal!) são funções DIFERENTES do mesmo segmento → o invariante quebra de novo
e a mordida encolhe em vez de morrer (contra-exemplo numérico: taper 5→2 sobre ext=r dá 25% de
divergência em `dn`). **Uma função, três chamadas:**

```wgsl
// distância NORMALIZADA à cápsula a→b com raios lerp'ados pelo t clampado.
// SENTINELA: cápsula ausente (p0==p1 / p3==p2, flat varyings → igualdade exata).
fn capsule_dn(frag: vec2f, a: vec2f, b: vec2f, ra: f32, rb: f32) -> f32 {
    let ab = b - a;
    let len_sq = dot(ab, ab);
    if (len_sq < 1e-6) { return 1e9; }
    let t = clamp(dot(frag - a, ab) / len_sq, 0.0, 1.0);
    let d = length(frag - a - t * ab);
    return d / max(mix(ra, rb, t), 1e-4);   // 0 = centro, 1 = borda
}
```

Varyings novos (o vertex JÁ busca `sp`/`sn` do storage para o miter — só passa a exportá-los;
+2 fetches para os raios vizinhos, custo nulo):

```wgsl
@location(6) @interpolate(flat) ss_p0: vec2<f32>,  // sentinela: == ss_p1 (aberto sem prev)
@location(7) @interpolate(flat) ss_p3: vec2<f32>,  // sentinela: == ss_p2
@location(8) @interpolate(flat) radii: vec4<f32>,  // (r_p0, r_a, r_b, r_p3) em px

// vs_main:
out.ss_p0 = select(sa, sp, prev_gp != gp);
out.ss_p3 = select(sb, sn, nn_gp != next_gp);
out.radii = vec4(r_prev, r_a, r_b, r_next);
```

Fragment (a única mudança de comportamento):

```wgsl
let dn_own  = capsule_dn(frag, in.ss_p1, in.ss_p2, in.radii.y, in.radii.z);
let dn_prev = capsule_dn(frag, in.ss_p0, in.ss_p1, in.radii.x, in.radii.y);
let dn_next = capsule_dn(frag, in.ss_p2, in.ss_p3, in.radii.z, in.radii.w);
let dn = min(dn_own, min(dn_prev, dn_next));
let aa = max(fwidth(dn), 1e-4);            // dn é contínuo (campos coincidem na troca de dono)
let mask = hardness_mask(dn, in.hardness, aa);
// discard < 0.001: INALTERADO — 3ª perna do tripé; onde a união local ainda dá ~0,
// não escreve depth e um segmento FORA da janela pinta depois.
```

O que **não** muda: vertex (geometria/miter/break/depth), `pipeline.rs`, discard, blend,
compositor por-camada, `pack.rs` (nenhum atributo novo — tudo já está nos buffers). Diff
confinado a `flip.wgsl`. **Nota de fronteira:** os storage buffers têm `visibility: VERTEX`
apenas (`pipeline.rs`) — o fragment recebe TUDO por varying; qualquer tentação de "buscar
`points[]` no fragment" muda a BGL (deixar um comentário-cerca no shader).

Efeitos colaterais aceitos e documentados:
- **Goldens de caps/extensões com taper mudam ligeiramente** (o raio na extensão hoje é
  poluído pelo comprimento da extensão — o valor NOVO é o mais correto).
- **Cor/opacity por-ponto:** o vencedor pinta com os varyings DELE. Na junção ambos convergem
  para os valores do ponto compartilhado (sem costura de COR); no flanco da sobreposição há
  um degrau teórico de opacity limitado por `Δ·r/(len+r)` — resíduo documentado como teto,
  coberto pelo teste 9 (width) e anotado para opacity.

**Limites conhecidos (o teto de (a)):** a janela é ±1. Sobreposição com `i±2, i±3…` (tablet
denso + pincel gordo em curva fechada apertada — o miter interno dobra) e **auto-cruzamento
não-adjacente** (laço) permanecem first-wins. O laço é **semântica pinada de propósito**
(default do GP; a alternativa com acúmulo é a futura flag *Self Overlap*, §8). O `i±2` é o
gatilho objetivo da escalada (K4, §6).

### 4.2 — 2º: dois passes com blend MAX (a ESCALADA exata, se (a) bater no teto)

Passe 1: os mesmos quads numa scratch single-channel (`R16Float`; `R8Unorm` chega), **sem
depth**, `BlendOperation::Max`, o fragment escreve `mask` — a união `max_i(mask_i)` emerge por
hardware, **globalmente correta** (junções, i±2+, hairpins, laços). Passe 2: composite com a cor.

- **Correção obrigatória (D4):** o passe 2 redesenha os quads com o depth GREATER de hoje para
  a seleção de cor, **descartando pelo mask PRÓPRIO** (`< 0.001`, recomputado) e tomando só o
  **alpha** do scratch — sem isso, o canto de A com alpha-da-união alto venceria o depth e
  pintaria a COR de A sobre o núcleo de B (pior que hoje).
- Escopo: a união só vale DENTRO de um traço → scratch→composite→clear **por traço** (dirty-rect),
  ou rotear pro caminho lento só traços com `hardness < 1` E auto-sobreposição. Para o traço
  AO VIVO é 1 scratch incremental — desprezível. Para replay de frame com centenas de traços
  macios o custo de passes é real (mitigação: cache do frame composto, que a arquitetura de
  camadas já favorece).
- Pedigree: é o *Stencil-then-Cover* (NV_path_rendering) contínuo; a semântica do Krita Wash e
  das patentes de ink da Microsoft (`04_alem_do_blender.md` §1).
- **Variante SDF** (anotar, não implementar): scratch de DISTÂNCIA com blend MIN + falloff no
  composite → hardness re-editável por uniform sem re-render.
- Zoom/pan invalidam cache de frame (o traço rasteriza em screen-space) — o custo por-pass cai
  no caminho interativo; por isso (b) é escalada com gatilho, não default.

### 4.3 — Rejeitadas (com razão registrada)

| Opção | Por que não |
|---|---|
| Correção de alpha do Ciallo `1−sqrt(1−A)` | pressupõe alpha constante + exatamente 2 camadas = o caso hardness=1, que o tripé já resolve |
| Dab-based (Krita/Painter) | re-stamp por frame de animação; casa mal com edição posterior; o Flip é traço vetorial re-renderizável |
| Vello stroke-expansion / polar stroking / stroke→fill | união exata SÓ para alpha constante — nenhum trata falloff; referência p/ futuro pincel "ink" duro |
| Airbrush integral (Ciallo) | semântica de ACÚMULO físico (laço escurece) ≠ união GP; excelente pincel FUTURO |
| Depth por-ponto (`GP_STROKE_OVERLAP`) | muda a semântica p/ acúmulo, não corrige — é a futura flag de material *Self Overlap* |
| Join-fan geométrico sem sobreposição (pygfx) | sofre a mesma doença (campo radial do fan vs núcleo do vizinho); e o recuo exato do vértice interno com raio variável é espinhoso |

## §5 — O oráculo corrigido + plano de testes

**A lição da rodada 7** (memória `feedback_oracle_must_model_appearance_not_implementation`):
o oráculo modelava first-wins e ficou **verde com o bug na tela**. O expected deriva da
APARÊNCIA:

- **Cenas de junção** (sem auto-sobreposição não-adjacente): `expected = hardness_mask(min_i
  dn_i)` sobre **TODOS** os segmentos (a distância à polilinha = a união). Nessas cenas,
  união global e janela ±1 coincidem por construção → o oráculo é irrefutável E discriminante.
- **Cenas com laço** (cruzamento não-adjacente): expected = first-wins (o oráculo atual),
  **pinando a semântica escolhida** — documentado no próprio teste.
- Manter: containment de quad (pixels fora de todo quad = 0), pulos de faixa de aresta/limiar
  de discard/AA, `checked > 500`.

**Migração dos testes legados (não é opcional — foi um achado adversarial):** 2 dos 9 testes
GPU atuais usam o `expected_alpha` first-wins (`a_sharp_corner_does_not_accumulate_color` e
`a_smooth_curve_matches_the_analytic_coverage`). Com o fix, o first-wins antigo fica ERRADO
(o GPU pinta a união, maior) → **migram pro oráculo-união junto com o fix**. Atenção: o teste
do arco (r=8, segmentos ~12px, o miter interno dobra) é exatamente o cenário `i±2` — se ficar
vermelho pós-fix APENAS em pixels de sobreposição i±2, isso é o **teto documentado de (a)**
(K4), não um bug: ou skip window-aware documentado só nesses pixels, ou promover a escalada
(b). **NUNCA afrouxar a tolerância pra "passar"** — é a 4ª causa da DIRETIVA.

**Bateria (nomes + o que afirmam):**

1. `a_soft_broken_corner_matches_the_polyline_union` — o zigzag do smoke (viradas 135–170°),
   hardness 0.8: todo pixel = união. **TEM de estar VERMELHO antes do fix** (é a mordida).
2. `a_soft_mitered_corner_matches_the_union` — virada ~30°, gordo, hardness 0.7. **Verde
   ANTES do fix com largura uniforme** (cobertura mitrada já é união — D2): é REGRESSÃO, não
   discriminante. Não espere vermelho aqui.
3. `a_soft_hairpin_matches_the_polyline_union` — 3 pontos, virada ~175° (o extremo da
   sobreposição A∩B).
4. `a_closed_soft_star_seam_matches_the_union` — fechada com viradas > 120° (**estrela, não
   quadrado** — quinas de 90° são mitradas e não discriminam).
5. `soft_round_caps_are_unchanged_by_the_neighbor_window` — 2 pontos (janela toda sentinela):
   paridade com o comportamento antigo (regressão de cap).
6. `a_soft_self_crossing_keeps_first_wins_semantics` — o X macio: pinos explícitos de (i) sem
   acúmulo (`cross ≈ arm`), (ii) primeira-parte-por-cima, (iii) tolerância documentando o
   resíduo da borda de A sobre o núcleo de B. **É o contrato do teto de (a).**
7. **`a_tapered_broken_corner_matches_the_union` (teste 9 — OBRIGATÓRIO):** quina quebrada
   macia com **largura por-ponto** (ex.: 4→16px). É o teste que pega a divergência de
   parametrização de raio (D1) — sem ele, o gap do `dn_own` passa em silêncio (todos os
   outros usam largura uniforme).
8. Os testes de tripé existentes (bead/spike/escamado/acúmulo/cruzamento entre strokes/fill)
   permanecem verdes — nenhum é afrouxado.
9. **Mutações provadas** (asserção-vermelha real, como na rodada 7): remover `dn_prev/dn_next`
   do min → teste 1 vermelho; `GreaterEqual` → hairpin vermelho; sem discard → vermelho.
   O oráculo só vale se as mutações sangram.

**Gotcha de dados (R3):** ponto DUPLICADO no meio do traço degenera a cápsula-vizinha
(sentinela acidental) e a janela não vê o segmento seguinte — e o código atual JÁ é hostil a
duplicados (`normalize(0)` no miter). Garantir dedup no ingest (`flip_draw`/pack) e registrar
como invariante do buffer (`debug_assert`).

## §6 — Execução em 1 iteração + kill-criteria

Sequência (a ordem importa — alvo irrefutável ANTES do fix, DIRETIVA):

1. **Trocar o oráculo** → rodar → o teste 1 **TEM de estar vermelho no código atual**.
2. Implementar o fragment (§4.1, com `capsule_dn` unificada) → suite completa `--ignored` em
   debug E `--release` → migrar os 2 legados (política do arco: acima).
3. Smoke do Enio no mesmo zigzag, hardness alto e baixo + tablet denso se possível.

Kill-criteria:

- **K1 (oráculo):** teste 1 não fica vermelho antes do fix → o oráculo modela mecanismo, não
  aparência — invalida a iteração; consertar o oráculo primeiro.
- **K2 (tripé):** qualquer teste de tripé vermelho após o fix → o fix interagiu com
  discard/depth de forma imprevista → reverter e re-analisar.
- **K3 (smoke vs harness):** oráculo verde mas a mordida visível no app → o harness reproduz o
  mecanismo, não o contexto (memória `feedback_harness_reproduces_mechanism_not_context`):
  instrumentar no app real (dump da janela p0/p3 e do `dn` no pixel apontado) ANTES de código novo.
- **K4 (teto → escalada):** quinas limpas mas mordida residual com tablet denso + pincel gordo
  em curvas fechadas, ou laço macio incômodo → NÃO iterar em (a); promover (b) (§4.2), que
  reutiliza a mesma máscara — só troca quem faz o `max` (o hardware de blend).
- **Perf (não-kill):** +2 cápsulas/fragment e +8 escalares flat (dentro do budget de 16 vec4
  inter-stage) — abaixo de ruído; medir antes de atribuir qualquer regressão a isto.

## §7 — Antialiasing (a decisão, com a história do GP como aviso)

O GP **precisa** de pós-AA (SMAA 1x, preset HIGH) porque o traço hardness=1 dele produz borda
`step` dura. A história é um aviso: o "AA Threshold" da UI é na verdade um **ganho do luma**
(threshold real fixo em 0.1); edge-detect em scene-referred linear perde bordas de baixo
contraste (#74938); shimmer temporal de linhas finas está aberto há 5 anos (#90321); e a
solução definitiva deles foi **SSAA no render** (4.5+), não mais SMAA.

**Decisão Flip:**

1. **Viewport: AA analítico (fwidth) como caminho primário — NÃO portar SMAA agora.** Para o
   traço, o analítico é estritamente superior (cobertura exata, estabilidade temporal, custo
   ~zero). Fechar a mordida vem antes de qualquer pós-AA.
2. **Portar o fade sub-pixel do GP** (`color *= smoothstep(0,1, thickness_px_unclamped)`) —
   barato, mata o pisca de traço < 1px (o hack validado min-width ~1.3px + modulação de opacidade).
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

- **Flag *Self Overlap* por pincel/traço** (1 bit + 1 linha: depth por-PONTO em vez de
  por-stroke) — auto-sobreposição com acúmulo, para marker/build-up expressivo. É a resposta
  canônica ao "parte nova por cima" do smoke da 3ª rodada.
- **Corner types Round/Sharp(miter-limit)/Flat por ponto** (paridade SVG/GP-2025): as cunhas
  BEVEL/MITER do `segment_mask` com p0/p3 — a mesma janela do fix (§4.1) já carrega os dados.
- **Pincel pontilhado (dots/squares) estilo Ciallo/GP:** dots sintetizados NO fragment
  (interseção de cápsula desigual + early-out em saturação) — zero geometria extra, espaçamento
  independente da densidade de input. Arquitetura pronta no GP (`frag:337-516`).
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
