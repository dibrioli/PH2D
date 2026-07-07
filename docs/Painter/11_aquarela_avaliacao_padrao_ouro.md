# 11 — Aquarela: avaliação vs padrão-ouro da indústria + plano (SEM física real)

> **Pedido (Enio, 2026-07-06):** avaliar a implementação watercolor, comparar com o padrão-ouro,
> diagnosticar (a) a rediluição imperfeita da tinta assentada no wet-on-wet e (b) o puxão indevido
> pro bege do papel virtual, e traçar o caminho pra padrão-ouro. **Restrição explícita: "não
> queremos física real por enquanto"** — nada de solver/fluido (coerente com ADR-0096); o alvo é o
> topo do tier **óptico/stateful-leve** (Procreate/Krita/Photoshop), não o tier sim (Rebelle/MoXi).
>
> Estudo apenas — **nenhum código alterado.** Los alvos de aceitação abaixo são as
> ASSERÇÕES-VERMELHAS (DIRETIVA §3/§4) que cada fase futura deve satisfazer ANTES de fechar.

## §0 — Veredito em 5 linhas

1. O render-path óptico (Beer–Lambert por canal em luz linear, LUTs HR-5) é **acima da média** do
   tier não-físico — a maioria dos apps Tier-2 só faz truques de alpha. A fundação está certa.
2. O **bege é um bug real e mensurável**: a base óptica usa um papel virtual cromático
   (`PAPER=[239,233,220]`) onde a camada é transparente, e o bake **assa uma fração `T·film_a` de
   bege nos pixels** — sobre um fundo branco real, o traço puxa pro creme. Matemática em §3.
3. A **rediluição é imperfeita por construção**: presença/lift são heurísticas RGB **relativas ao
   bege global** (tinta clara é invisível ao rewet; o lift caminha pro creme, não pro chão real), e
   não existe estado de pincel (charge/dilution/recentness) — a rediluição não é progressiva nem
   direcional.
4. Correção-mãe: **o chão óptico deve ser o backdrop REAL** (composite das camadas abaixo; onde
   ainda transparente, uma **cor de papel do documento** escolhível no painel — default branco).
   Resolve o bege E conserta a referência do rewet de uma vez. As duas metades da pergunta do Enio
   estão certas — é "pixels reais abaixo" **e** o campo de cor de papel (pro caso 100% transparente).
5. O "feel" de rediluição padrão-ouro sem física = **mixer state do doc 07 §4**
   (charge/dilution/pull/recentness, Procreate-verbatim) lendo a **base congelada já-liftada** —
   corrige as 3 causas da tentativa retirada (reservoir Dulling).

## §1 — O modelo atual (o que existe, com file:line)

Tudo em `crates/ph2d-tool-painter/src/tool/paint/`:

- **Arquitetura wet_edges** (`watercolor_render.rs:1-37`): cobertura + cor por-traço
  (`watercolor_accum.rs`), reconstrução óptica por frame sobre base congelada no pen-down
  (`stroke_lifecycle.rs:57-59`), bake 1× no pen-up (`stroke_lifecycle.rs:190-194`). Dirty-rect
  incremental (~0,22 ms/frame @2048², sonda `watercolor_perf_frame_cost_probe`).
- **Óptica**: `D = (cover·fill + edge)·gran`, `Tᵢ = pigᵢ^(D·depth)` em luz linear via LUTs
  (`watercolor_field.rs:17-95`); edge = unsharp da cobertura (rim no front recuante); warp fractal;
  papel/grain como height-fields (`watercolor_render.rs:275-312`).
- **Wet = rewet per-pixel stateless** (`watercolor_render.rs:148-208, 325-352, 392-406`): presença
  = escurecimento vs `PAPER` (dead-zone 14→50), lift log-space (cap `REWET_LIFT=0.85`), dissolve =
  box-blur one-shot raio `edge_spread`, pool no rim (`REWET_POOL=0.35`), redistribuição do próprio
  wash (`WET_THIN/WET_EDGE_BOOST/WET_RAGGED`), mix RYB `max(pigment_mix, wet·presença)`.
- **Smudge = true smear** da base congelada (`watercolor_smudge.rs`); o reservoir Pickup (Dulling)
  foi **tentado e retirado** (cadence-bound, self-feeding, perceptualmente fraco — 2026-07-06).
- **Rede de testes**: ~17 `watercolor_*` em `paint/tests.rs` pinam dirty-rect, incremental≡full,
  smear, lift/bleed, hue sem Pigment, inércia, convergência Pigment ON/OFF.

## §2 — Padrão-ouro da indústria (verificado)

**Tier 1 — simulação física** *(EXCLUÍDO por decisão: "sem física real por enquanto" + ADR-0096)*:
Curtis 1997 (shallow-water + settle/lift), MoXi/Expresii (LBM), **Rebelle** (dois estados
suspended/settled + rewet físico + secagem), Corel Painter Real Watercolor (difusão dirigida pela
luminância do grão do papel; pigmento assenta ao secar). O clean-room `docs/Painter/ph2d_wet_paint/SPEC.md`
especifica esse tier inteiro (rewet `lift = sett·rewetBase·(1+excess·50)`, edge darkening por
evaporação diferencial, fingering) — fica como **norte futuro**, não deste ciclo.

**Tier 2 — óptico/stateful sem fluido** *(o nosso tier; o alvo é o topo dele)*:
- **Procreate Wet Mix** (Handbook, doc 07 §4 verbatim): estado de mixer por traço —
  **Charge** (reservatório que depleta), **Dilution**, **Pull** (taxa de reamostragem),
  **recentness** (gate de quando reamostrar o destino), Blur (disco de pickup). É o que torna a
  rediluição *progressiva e direcional* sem física.
- **Krita Color Smudge** (Smearing/Dulling) — nosso true-smear já é o Smearing.
- **Photoshop Mixer Brush** (wet/load/mix wells) — mesmo espaço de design do Charge/Dilution.
- **Cor do papel**: no **Rebelle** é propriedade do DOCUMENTO escolhível pelo usuário (qualquer cor
  da paleta; cada papel tem um default; manual "Select Canvas"). No **Corel Painter** o papel
  (grão + cor) é propriedade do documento. **Ninguém usa uma constante cromática de pincel** — e a
  própria demo de referência (`wet_edges_paint.html`) **não tem papel virtual**: ela PINTA o creme
  como pixels reais no `committed` e a base do composite são esses bytes; a constante bege só é
  usada como referência do gate RYB. Nosso `PAPER` const é um artefato do port, não do modelo.
- No SPEC clean-room, o papel modula só **luminância** perto do branco (`base=255+(pap·30−30)`,
  render-only) — nunca uma cromaticidade que a óptica enxerga.

Fontes: [Rebelle 7 Manual — Visual Settings](https://escapemotions.com/products/rebelle/manual/interface/panel-visual-settings/) ·
[Rebelle — Working with Water](https://www.escapemotions.com/products/rebelle/manual/starting-painting/water/) ·
[Rebelle 7 — Art Canvases](https://www.escapemotions.com/products/rebelle/manual/7/starting-painting/art-canvases/) ·
[Corel Painter — Watercolor layer](http://product.corel.com/help/Painter/540215550/Main/EN/Win-Documentation/Corel-Painter-Watercolor-Layer.html) ·
[Corel Painter — Watercolor and paper texture](https://product.corel.com/help/Painter/540219480/Main/EN/Win-Documentation/Corel-Painter-Watercolor-and-Paper-Texture.html) ·
doc 07 §4 (Procreate Handbook verbatim) · `ph2d_wet_paint/SPEC.md`.

## §3 — Diagnóstico A: o puxão pro bege (confirmado, matemática)

`PAPER` entra em **quatro** papéis distintos hoje (`watercolor_render.rs`):

| Site | Papel do PAPER | Linha |
|---|---|---|
| Base efetiva `sb` do Beer–Lambert onde a camada é transparente | chão óptico | 384-391 |
| Referência da presença do rewet (`d = max(PAPERᵢ − cᵢ)`) | "o que é tinta" | 168-182 |
| Alvo do lift (`sb → paper_lin` na curva log) | "pra onde a tinta sai" | 392-406 |
| Comentário/gate RYB herdado da demo (PAINT_LO/HI) | gate perceptual | 182 |

**O defeito do bake.** Para um pixel de camada transparente (`ab=0`): `sb = paper_lin`,
`out_rgb = sb·T + pig·(1−T)`, `out_a = film_a`. O compositor então faz
`final = out_rgb·film_a + backdrop·(1−film_a)` — o resultado final contém
**`T·PAPER·film_a` de bege** que não existe no mundo real quando o backdrop é branco. Um wash fino
(T alto) é o pior caso: quase todo o corpo do traço é papel-virtual assado. É exatamente o sintoma
relatado ("se pinto em canvas branco há momentos que a pintura puxa para o bege").

**O defeito da referência.** A presença lê "quanto este pixel escurece vs o CREME global":
- tinta **mais clara que o creme** (amarelo pálido, cinza claro) → presença ≈ 0 → **invisível ao
  rewet** (não redissolve, não sangra, não mistura);
- o lift caminha pro **creme**, não pro chão real — sobre fundo branco ou cinza, rewet "suja" na
  direção errada;
- a dead-zone 14→50 (o hack anti-flood de 2026-07-06) só é necessária porque a referência é uma
  constante global errada; com referência local correta ela degenera no caso trivial.

## §4 — Diagnóstico B: por que a rediluição "não é perfeita"

1. **Referência errada** (§3) — a metade dos casos ruins de rediluição É o bege.
2. **Stateless**: a liftabilidade é constante (`REWET_LIFT=0.85` pra sempre) — não há distinção
   fresco/seco nem qualquer progressividade: esfregar 5× redissolve o mesmo que passar 1×.
3. **Sem pincel-como-agente**: a tinta redissolvida não entra no pincel nem é transportada — o
   dissolve é um blur one-shot de raio fixo (`edge_spread`), simétrico, sem direção do gesto. No
   padrão-ouro Tier-2 (Procreate/Photoshop), o pincel **carrega** o que redissolve (charge) e o
   **deposita adiante diluindo** (dilution) até esgotar — é isso que o olho lê como "rediluir".
4. **Escala**: a demo de referência roda com `S=2` em HiDPI (todos os params em px ×S — grain
   5·S, warp 22·S/8·S, spread·S); o port Rust não mapeia S ⇒ spread/warp/grain efetivos são a
   **metade** do look de referência. O bleed curto agrava a leitura de "rediluição fraca".
5. (Menor) O pool do dissolve entra só no rim — o interior recebe tint mas o corpo do wash não
   ganha a "nuvem" de pigmento redissolvido; aceitável, revisitar depois de 1-4.

## §5 — Plano (fases, cada uma com alvo irrefutável ANTES de codar)

### F1 — Chão real: backdrop composite + campo "Paper color" *(mata o bege; pré-requisito de tudo)*

**Mudança.** No `paint_begin`, materializar `wet_backdrop`: composite das camadas **abaixo** da
ativa (o compositor CPU já tem `composite_region`/cut-point cache — `compositor/compose.rs:30,50`);
onde o backdrop ainda for transparente, a **cor de papel do documento** (novo campo no painel
watercolor/canvas: swatch, default **branco**; presets de papel podem setar seu default à la
Rebelle). Substituir `PAPER` nos 4 papéis do §3: `sb` compõe camada-sobre-backdrop-local; presença
= escurecimento vs backdrop local; lift caminha pro backdrop local; gate RYB idem.

**Bake sem contaminação.** O pen-up resolve o straight-alpha **contra o backdrop conhecido**:
`out_rgb = (aparência_alvo − backdrop·(1−film_a)) / film_a` (clamp de gamut), de modo que
`flatten(camada sobre backdrop) == aparência pintada`. Zero bege assado; camada continua RGBA8
source-over normal.

**Custo.** Composite do backdrop 1× por traço (região-preguiçosa se necessário); hot loop troca
uma constante por um read — re-rodar a sonda de perf (alvo: ≤0,3 ms/frame @2048², hoje 0,22).

**Alvos (asserções-vermelhas):**
- **T1 (bege morto):** camada ativa transparente sobre camada branca opaca — o flatten do traço
  bakeado difere ≤2 bytes/canal do MESMO traço pintado direto numa base branca opaca. (Hoje: delta
  = cast bege mensurável, dominado por `T·(PAPER−branco)`.)
- **T2 (tinta clara redissolve):** amarelo pálido `(250,240,150)` seco sobre branco → presença no
  rewet > 0.3 (hoje ~0 nos canais r/g).
- **T3 (lift na direção certa):** wet=1 sobre vermelho seco em fundo branco → o lift clareia
  **preservando o hue na curva do próprio pigmento** (rosa), sem componente na direção do creme;
  repetir sobre fundo cinza-médio (lift caminha pro cinza). Gate existente
  `watercolor_wet_lift_stays_in_hue_without_pigment` é atualizado JUNTO (decisão documentada aqui).
- **T4 (default intacto):** watercolor OFF byte-idêntico; wet=0 + camada opaca byte-idêntico.
- Gates que assumem o bege (lift/bleed/hue/convergência) mudam **junto com esta decisão** — nunca
  silenciosamente (regra do handoff).

### F2 — Rediluição Tier-2: charge/dilution/pull/recentness (doc 07 §4, já projetado)

**Mudança.** Estado de mixer por traço `{rgb, a, recentness, charge}` premul-linear (doc 07 §4.1):
reamostragem do destino gateada por `recentness/pull`; depósito = `mix(brush, state, s)` com
`s = f(charge_depletion, dilution)`; o resultado alimenta a cor dos dabs → `stroke_color` → o
render-path óptico existente pinta tudo. Sliders novos na seção Wet (Charge, Dilution, Pull; Blur
reusa `edge_spread`).

**Por que NÃO repete o fracasso do reservoir retirado** (as 3 causas nomeadas, cada uma fechada):
(a) *self-feeding* → o pickup lê a **base congelada já-liftada** (`sb` pós-lift), nunca o composite
vivo; (b) *cadence-bound* → `recentness/pull` gateiam a reamostragem por distância/pull, não por
frame; (c) *perceptualmente fraco* → a assinatura visível vem da **depleção** (o rastro morre) e da
**diluição** (o depósito enfraquece), que o reservoir antigo não tinha.

**Alvos:**
- **T5 (transporte direcional):** traço wet cruzando uma banda vermelha seca em fundo branco → a
  jusante da banda há vermelho depositado, decaindo monotônico com a distância, <10% após o
  comprimento de charge.
- **T6 (progressivo):** 2 passadas sobre o mesmo ponto redissolvem mais que 1 (Δ mensurável).
- **T7 (default intacto):** Charge/Dilution/Pull zerados → byte-idêntico ao pós-F1.

### F3 — Escala e feel (barato, alto impacto perceptual)

- ~~**Mapeamento S da demo**~~ — **REFUTADO POR MEDIÇÃO (2026-07-06, na implementação):** a demo
  escala o **raio do dab** por S também (`wet_edges_paint.html:445`, `radius = size·(0.5+0.5·p)·S`),
  então raio/spread/warp/grain escalam juntos — as proporções são DPR-invariantes e o app a 100%
  de zoom já bate a referência. Dobrar às cegas teria DESVIADO da demo. O gap real: o cap fixo de
  24 px deixava pincéis grandes "secos" → **Spread/Warp 24→48** (setter + painel + render clamp).
- **Bleed que cresce com a permanência (soak):** implementado via campo per-stroke `wet_soak`
  (u8/px) — o heartbeat (`paint_tick`) derrama dwell sob a ponta (satura em ~2 s; a poça
  **alarga** com a saturação do centro, até 2× o raio); o composite lê o soak em 2 formas: RAW
  (contato → aprofunda o lift, `SOAK_LIFT`, cap 0.95) e HALO (blur 2×spread → o dissolve lerpa
  entre o blur normal e um de raio 2×, `SOAK_DISSOLVE` dobra o tint em soak pleno). Parado, o
  tick recomposita ao vivo (o bleed cresce visível sob a ponta). **Alvo revisto com a decisão**
  (o de "raio ≥1,5×" era threshold-granular no primeiro-cruzamento): lift sob a ponta mais fundo
  (ΔG > +6) **e** massa de tint além da borda ≥1,15× (medido +21% nos knobs default) — teste
  `watercolor_soak_deepens_and_widens_the_dissolve_while_parked`. Knobs nomeados = superfície de
  tuning pro smoke.

### F4 (opcional, decisão do Enio) — Mistura K–M single-constant no lugar do RYB

O pivot do ADR-0096 já aponta Kubelka–Munk/Mixbox pro blend de pigmento. O SPEC §14 dá a forma
barata e determinística: `KS(R)=(1−R)²/(2R)`, mistura linear em K/S, inversão fechada — **em
floats, nunca quantizar a bytes** (deriva pra preto). Substituiria `ryb_mix` nos caminhos
dissolve/mix. Alvo: azul+amarelo → verde vibrante (assert de canal); 1000 re-mixes sem drift >2
bytes. Não é física — é uma fórmula de mistura melhor.

**Sequência:** F1 → F3 (juntas cabem numa jornada) → smoke → F2 → smoke → F4 se aprovado.
F1 é pré-requisito de F2 (o pickup precisa do chão certo pra não carregar bege).

## §5.1 — Registro da implementação F1+F3 (LANDOU 2026-07-06, gate-verde local)

- **F1 chão real:** `wet_backdrop` congelado com a base no pen-down (`watercolor_backdrop.rs`) via
  `compositor::composite_below` (walk root→grupo, abaixo do anchor; máscara ancora no PARENT).
  `PAPER` const **morto** — os 4 papéis (base `sb`, presença, alvo do lift, mix base) leem o chão
  local per-pixel. Presença virou distância **simétrica** ao chão (tinta mais clara que o chão
  também redissolve — segura porque o chão local zera exato no não-pintado). Lift bidirecional
  (converge no chão pelos dois lados, log-space). **Paper color** = swatch na seção Paper
  (picker compartilhado, id `PAINTER_WATERCOLOR_PAPER_COLOR_THUMB`, default BRANCO; tool-global,
  persistência por-documento = follow-up).
- **Bake un-premultiply:** o composite resolve `L = (aparência − chão·(1−a))/a` em luz linear;
  **achado de gamut**: `a = film_a` (luminância) estourava o solve (erro 59 bytes no flatten) —
  o alpha de cobertura correto é `1 − min_c T_c` (o canal mais absorvido); `film_a` fica só como
  força do mix. Base opaca ⇒ byte-idêntico ao caminho antigo.
- **Decisão semântica (Rebelle-consistente):** um canvas opaco não-branco É tinta da camada sobre
  o papel — Wet redissolve um fill cinza rumo ao papel; quem quer "o cinza é meu papel" seta o
  Paper color (teste `watercolor_wet_reads_no_paint_on_a_paper_colored_ground`).
- **Perf (sonda @2048², release):** wet=0 0,23 ms/frame (inalterado) · wet=1 sem dwell 0,45 ms
  (campos far/halo gateados por `wet_soak_active`) · com dwell ~1,2 ms · bake 13-25 ms 1×.
- **Testes novos:** T1 `watercolor_ground_is_the_real_backdrop_not_a_virtual_cream` (flatten ≤2
  bytes vs pintar direto) · T2 `watercolor_wet_lifts_paint_lighter_than_the_old_cream` · T3-cinza
  (acima) · soak (§5 F3) · seam Paper color em `panel_events_drive_watercolor_state`. 472/472 na
  crate + 758/758 editor-core + 40 panel; clippy 0; LOC caps ok (split `event/picker.rs`).

## §6 — Fora de escopo (explícito)

- **Física real** (shallow-water, dois-layers suspended/settled, secagem temporal, fingering,
  evaporação diferencial): excluída por decisão do Enio (2026-07-06) + ADR-0096. O
  `ph2d_wet_paint/SPEC.md` permanece o norte se/quando essa decisão mudar.
- Estado persistente de umidade por camada ("Wet the Layer" do Rebelle): exige estado fora do
  traço → primeiro degrau do tier físico; não entra agora.
- Migração GPU do composite: ortogonal (o modelo per-pixel é GPU-amigável; decidir depois do feel).

## §7 — Rastreabilidade

Código lido inteiro: `watercolor_render.rs` (469), `watercolor_accum.rs` (156),
`watercolor_smudge.rs` (72), `watercolor_field.rs` (292), `stroke_lifecycle.rs` (210),
`spec.rs` (campos watercolor), `compositor/compose.rs` (entry points). Docs:
`10_aquarela_render_path_preset_papers.md`, `07_rendering_modes_wet_mix.md` §4,
`ph2d_wet_paint/SPEC.md` (§2/4/5/6/13/14/16/17/18), `wet_edges_paint.html` (modelo completo,
constantes L188-197). Testes pinados: 17 `watercolor_*` em `paint/tests.rs:10498-11323`.
