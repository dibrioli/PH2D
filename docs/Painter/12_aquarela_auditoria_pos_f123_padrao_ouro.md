# 12 — Aquarela: auditoria sistemática pós-F1/F2/F3 vs padrão-ouro EXTERNO (2026-07-07)

> **Missão (handoff da linha Painter):** auditoria sistemática do motor watercolor ATUAL
> (pós-chão-real F1 + escala/soak F3 + Wet Mix F2, HEAD `65f426d2`) comparado com o padrão-ouro
> da indústria. **Regra do Enio (ordem explícita da sessão): padrão-ouro é conferido em fontes
> EXTERNAS** — artigos publicados e manuais de outros apps, fetchados de verdade; docs internos
> não contam como fonte. Restrição de produto mantida: **sem física real** (ADR-0096) — toda
> recomendação aqui é Tier-2 (óptico/stateful leve).
>
> **Método:** workflow multi-agente (47 agentes): 6 lentes de auditoria (óptica, bordas,
> rewet/mixer, granulação, determinismo, perf-medida) + 2 de painel/UX, cada achado passando por
> refutação adversarial independente (2 céticos p/ P0/P1, 1 p/ P2/P3, lentes evidência +
> materialidade). **28 achados sobreviveram, 0 refutados** (vários ajustados — o texto abaixo já
> incorpora as correções dos céticos). Perf foi MEDIDA (sondas release @2048²), não confiada.

## §0 — Veredito em 7 linhas

1. **A fundação óptica está certa e NÃO deve ser tocada:** o film Beer–Lambert per-canal em luz
   linear é exatamente o caso S=0 do Kubelka–Munk de Curtis 1997 (glaze não-espalhante:
   `R = T²·R_base`) — qualquer F4 que "migre o film pra KM" degradaria o caminho mais correto
   do motor. O F4 é só para os caminhos de **mistura**.
2. **O buraco nº 1 de cor é a mistura naive-RGB nos caminhos de matiz:** o deposit do Wet Mix é
   lerp em sRGB reto (o defeito literal que o paper do Mixbox condena: azul+amarelo→cinza) e o
   `ryb_mix` colapsa complementares em cinza puro (magenta+amarelo → 128/128/128 por trace).
   Correção barata (~40-60 LOC, LUTs já existem). É o pivot ratificado do ADR-0096, ainda não
   cumprido no código.
3. **O buraco nº 1 de textura é o espectro do papel:** o PaperCold do preset Aquarela tem ~90-98%
   da energia em λ>32 px (blotch, não tooth) — **medido por FFT** — e a granulação é modulação
   instantânea (tier NPR/Bousseau), não deposição acumulada nos vales (Curtis/Corel/Rebelle).
   Juntos, são a causa direta do look "digital/mottled" reportado.
4. **O buraco nº 1 de comportamento:** pen-up seca instantâneo (traços nunca fundem num wash),
   o gesto canônico do backrun (água limpa em wash úmido) é **inalcançável por construção**
   (Dilution=1 → cobertura 0 → rewet nem roda), e o Charge não depleta (a assinatura nº 1 do
   Procreate — "the brush runs out of paint" — não existe).
5. **Determinismo/HR-5: limpo.** Zero transcendental de libm no hot loop (tudo LUT + integer
   hash); os `.sqrt()` per-pixel são IEEE-754 exatos (bit-determinísticos, precedente pervasivo
   na crate). Achados menores: soak é frame-rate-dependente (P2) e `transmittance()` não lerpa
   o LUT (P3).
6. **Perf: claims do doc 11 CONFIRMADOS por medição independente** (todos <2×; suite 33/33
   verde). Gap real: o gate `incremental≡full` nunca exercita wet>0/ds>1 — o invariante que
   legitima o fix de Spread alto está garantido só por comentário (P1 de teste, não de runtime).
7. **Fences ratificadas que esta auditoria NÃO manda mexer:** core_r cap (fix do blob, Enio
   2026-07-07), retenção assimétrica do mixer + exit-bleed em Pull 0 (Enio 2026-07-07), rewet
   complementar ao mixer (doc 11 F2). Só corrigir os DOCS que ficaram stale sobre elas (§7).

## §1 — Tabela-resumo priorizada (28 achados, severidade pós-verificação)

| # | Sev | Lente | Achado | Custo estimado |
|---|-----|-------|--------|----------------|
| OPT-1 | P1 | óptica | Wet Mix deposita/acumula por lerp sRGB reto (naive-RGB do Mixbox) | ~15-25 LOC |
| OPT-2 | P1 | óptica | `ryb_mix` colapsa complementares em cinza exato (vs KM/Mixbox) | ~40 LOC |
| EDGE-1 | P1 | bordas | Sem umidade persistente entre traços — washes nunca fundem | ~1 buffer u8 + decay |
| EDGE-2 | P1 | bordas | Backrun/bloom clássico inexiste; gesto canônico inalcançável (Dilution=1 → cov 0) | ~30-60 LOC |
| MIX-1 | P1 | mixer | Charge não depleta ao longo do traço (assinatura nº 1 do Procreate) — **LANDOU take 4** (vide §W-C: 4 takes + 1 revert no caminho) | ~20 LOC |
| MIX-2 | P1 | mixer | Doc-drift: doc 11 promete pickup da base "já-liftada"; código lê base PRISTINA + tripla contagem no regime Wet+Charge altos | ~5-15 LOC |
| GRAN-1 | P1 | granulação | Granulação = modulação instantânea/simétrica (e sinal invertido vs Curtis), não deposição nos vales | ~40-60 LOC |
| GRAN-2 | P1 | granulação | Espectro do PaperCold = blotch de baixa freq (FFT: ~90-98% em λ>32px); claim interno "crisp tooth" falso | ~10 LOC (rota 1) |
| PERF-1 | P1 | perf | Gate incremental≡full nunca exercita wet>0 / ds>1 | ~45 LOC de teste |
| OPT-3 | P2 | óptica | Pigmento branco invisível / opacos sem body (S acoplado a K) | ~20 LOC gated |
| OPT-4 | P2 | óptica | Viabilidade F4 fechada: KS por álgebra + 1 sqrt IEEE (ou LUT 16k); drift byte→preto só em loop iterativo | análise pronta |
| OPT-5 | P2 | óptica | Alvo do F4 no doc 11 não discrimina (azul+amarelo→verde JÁ passa com RYB) | ~15 linhas doc |
| EDGE-3 | P2 | bordas | Rim aditivo sem conservação — interior não empalidece no modo seco | ~1-5 LOC (muda look) |
| EDGE-4 | P2 | bordas | Rim geometricamente uniforme — ignora água/dwell/carga local | ~10 LOC |
| MIX-3 | P2 | mixer | Reservatório zera a cada traço — sem workflow "pincel sujo" (PS) | ~20 LOC gated |
| MIX-4 | P2 | mixer | Pickup 5-tap fixo a r/2 — subamostra brush grande; sem knob de raio | ~25 LOC |
| MIX-5 | P2 | mixer | spec.rs (wet_pull): "no carry" em Pull 0 é FALSO (half-life ≈5,4 dabs, decisão deliberada) — corrigir DOC | ~2 LOC doc |
| GRAN-3 | P2 | granulação | Fallback built-in = 1 octave 5px mono-escala (lado "digital" do default) | ~10 LOC |
| GRAN-4 | P2 | granulação | Granulação por-pigmento inexistente; pool do rewet granula idêntico ao fresco | 2 degraus (10/60 LOC) |
| GRAN-5 | P2 | granulação | Fallback cego a Paper Size; tooth não ancora no tamanho do documento | ~15 LOC |
| DET-1 | P2 | determinismo | Soak frame-rate-dependente (floor 1 byte/tick + truncamento mata o fade do rim) | ~15-30 LOC |
| PERF-2 | P2 | perf | Sonda não cronometra o pen-down (composite_below full-canvas invisível) | ~5 LOC teste |
| EDGE-5 | P3 | bordas | core_r capa a largura do rim no raio do pincel — FENCE ratificada, só anotar | Enio-gated |
| MIX-6 | P3 | mixer | Assimetria load/unload = divergência deliberada da indústria — manter + teste direcional | ~5 LOC teste |
| MIX-7 | P3 | mixer | Inventário Procreate: falta Wetness Jitter (barato); Attack/Grade = defers ok; "Wetness boost" não existe | ~12 LOC |
| DET-2 | P3 | determinismo | `transmittance()` sem lerp no LUT (os gêmeos absorbance/exp_mag lerpam) | 1 linha |
| DET-3 | P3 | determinismo | LUTs de runtime dependem de libm (1 ULP cross-platform) — caveat p/ golden-hash futuro | nota de doc |
| PERF-3 | P3 | perf | Claims doc 11 confirmados; guard ratio-based E #[ignore] — nenhuma sentinela roda no CI | ~2 LOC |

## §2 — Lente 1: modelo óptico (Beer–Lambert × Kubelka–Munk)

**O que está CERTO (e fica):** `out = sb·T + pig·(1−T)` com `T = pig^(D·depth)` em luz linear
(watercolor_render.rs:582-588, LUTs em watercolor_field.rs) é o limite S=0 do KM de Curtis 1997
§5.2 (compositing `R = R1 + T1²R2/(1−R1R2)` com R1→0 ⇒ `R = T²·R_base` — Beer–Lambert puro; o
fator 2 do caminho duplo é absorvível no `depth`). O film óptico é o pedaço mais correto do
motor; **F4 não deve tocá-lo** (OPT-5).

- **OPT-1 (P1) — deposit do mixer em sRGB reto.** `watercolor_mixer.rs:100-103` (reservoir:
  running-average em sRGB reto premul) e `:108-116` (deposit: lerp sRGB puro);
  `sample_surface:163-179` compõe bytes sem linearizar. É a mistura que o abstract do Mixbox
  (Sochorová & Jamriška 2021, TOG) nomeia: *"blue and yellow make gray instead of green... the
  software is built around the RGB representation, which models the mixing of colored lights"*.
  É o principal caminho de **mistura de matiz** sem forma subtrativa (o dissolve usa ln-space,
  render.rs:534-541; o pigment-mix usa RYB) — nota: o smear do Smudge e o source-over do accum
  também são sRGB reto, mas são deslocamento de tinta, não mistura de duas cores por design.
  **Correção:** mover deposit+reservoir pra absorbância reusando as LUTs `lnl`/`exp_mag`
  existentes (mesmo padrão do dissolve) — per-DAB (5 taps), zero custo per-pixel, HR-5 ok.
  **Assert discriminante (validado numericamente pelo cético):** pincel azul (0.1,0.2,0.8) +
  poça amarela, Charge 0.3 → G > R e G > B no deposit (lerp sRGB atual dá R≈G = khaki).
- **OPT-2 (P1) — `ryb_mix` colapsa complementares.** Trace manual verificado linha-a-linha
  (blend.rs:203-253): magenta(1,0,1) + amarelo(1,1,0) em t=0.5 → RYB [1,0,1]+[0,1,0] → mix
  [.5,.5,.5] → `ryb_to_rgb` devolve **exatamente (0.5,0.5,0.5)** — cinza sem matiz; KM
  single-constant dá vermelho saturado (~1, 0.01, 0.01). Consumido em render.rs:611-624 (o
  caminho que dispara **sempre que Wet>0 sobre tinta** — "o segredo" do wet-on-wet) e
  blend.rs:191. **Correção (o F4 real):** KM single-constant per-canal em luz linear nos 3
  sites (pigment-mix, blend_over_pigment, deposit do mixer): `KS=(1−R)²/(2R)`, mistura linear
  em KS, inversão `R = 1+KS−sqrt(KS²+2KS)`. Alvos que o RYB **reprova** (os discriminantes):
  magenta+amarelo → R>0.8/G<0.35; croma de tint com branco preservada. Manter azul+amarelo→verde
  só como regressão (já passa hoje — OPT-5).
- **OPT-3 (P2) — branco invisível / opacos sem body.** `lnl[255]=0 ⇒ T≡1` para qualquer
  densidade (field.rs:36-39,73-77): branco titânio não deposita nada; cádmio opaco sobre base
  escura fica preto nos canais byte-255. Curtis 1997 §5.1 mantém K e S independentes (*"Opaque
  paints... have high scattering in the same wavelengths as their color"*). **Correção Tier-2
  gated:** slider "Body" (surrogate de S): segundo lookup `T_s = exp(−D·body)` na MESMA LUT,
  default 0 byte-idêntico — **com o ajuste do cético:** `cov_a` precisa incluir o termo de body
  (senão `1−t_min = 0` pro branco e o early-out descarta o pixel em camada transparente).
- **OPT-4 (P2) — viabilidade transcendental-free do F4: FECHADA.** Forward KS = álgebra pura
  (1 div). Inversão = 1 `sqrt` — **IEEE-754 correctly-rounded, bit-exato cross-platform**
  (≠ exp/sin de libm), com precedente pervasivo no próprio caminho (accum.rs:90,148,
  backdrop.rs:135, falloff/stroke/smear da crate brush); alternativa 100% LUT: 16k em
  `u=KS/(KS+1)` (~0.7 byte de erro). O claim do SPEC "quantizar a bytes deriva pra preto" é
  matematicamente verdadeiro **para loops iterativos** (re-mix milhares de vezes; Jensen sobre
  KS convexo) mas NÃO se aplica ao nosso composite one-shot que re-deriva dos bytes pristinos
  por frame — float é obrigatório só em estado acumulativo (reservoir, já f32).
- **OPT-5 (P2) — recalibrar o card F4 do doc 11 ANTES de implementar** (alvo irrefutável,
  regra-mãe da DIRETIVA): o alvo escrito ("azul+amarelo → verde vibrante") **já passa hoje**
  com RYB (`ryb_mix((0,0,1),(1,1,0),0.5) = (0,0.5,0)`); e registrar que single-constant KM não
  melhora glazing (sem espessura) nem granulação 2-pigmentos (exige γ por-pigmento, Curtis
  Fig. 5) — escopo = só os caminhos de mistura.

Fontes: [Curtis et al. 1997 (PDF)](https://grail.cs.washington.edu/projects/watercolor/paper_small.pdf) ·
[Mixbox paper TOG 2021 (abstract verbatim via Semantic Scholar)](https://api.semanticscholar.org/graph/v1/paper/DOI:10.1145/3478513.3480549?fields=title,abstract,year,authors,venue) ·
[scrtwpns.com/mixbox](https://scrtwpns.com/mixbox/).

## §3 — Lente 2: wet edges / rim / backruns

- **EDGE-1 (P1) — pen-up seca instantâneo.** Toda umidade é por-traço (lifecycle.rs:49,60,208;
  backdrop.rs:31-33); nenhum timestamp sobrevive. O rewet existente (Wet>0) lifta/dissolve/mixa
  tinta velha, mas é **atemporal** (igual segundos ou horas depois) e **cego ao rim**: o edge
  term do traço novo lê só a própria coverage (render.rs:413) → segundo traço desenha rim
  COMPLETO cruzando o wash vizinho (contornos duplos em pinceladas sobrepostas rápidas — o
  artefato mais visível da lente). Padrão publicado: Curtis §3-4 (wet-area mask persistente;
  washes que se tocam molhados fundem); DiVerdi/Adobe TVCG 2013 §4 (*"wet map cells are set to
  255 when wetted... they take 8.5 seconds to dry"*). **Correção Tier-2:** wet map u8
  canvas-wide que sobrevive ao pen-up com decay no heartbeat (`on_tick` já existe); no bake,
  despejar a cobertura no wet map; no composite seguinte, atenuar o edge term onde o wet map
  ainda está molhado sob a borda nova (rim some na junção). NÃO é o "Wet the Layer" do Rebelle
  (excluído no doc 11 §6 — aquele é comando de usuário; este é decay automático de estado
  visual, mesma classe do soak).
- **EDGE-2 (P1, 2× CONFIRMED) — backrun inalcançável por construção.** O "bloom" atual
  (REWET_POOL, render.rs:510) deposita no rim liso do próprio traço (mesma forma `(1−inner)`);
  sem serrilhado/couve-flor. Pior: água limpa = Dilution 1 → `flow=0` → `peak=0` → cobertura
  zero → `cw≤0 → continue` (render.rs:403-410) — **todo o caminho rewet é pulado**; o gesto
  que produz o backrun real não é executável. Curtis §2.2: *"water tends to push pigment along
  as it spreads, resulting in complex, branching shapes with severely darkened edges"*.
  **Correção:** (1) separar canal de ÁGUA do de pigmento — Dilution alta continua depositando
  wet-coverage (reusar o disco do soak) e o composite gateia o rewet em wet-coverage>0 em vez
  de cw>0 (o lift existente já entrega o "whitened wake" de graça); (2) serrilhar o contorno
  do pool com jitter integer-hash de célula média (~8-16 px) — HR-5 ok.
- **EDGE-3 (P2) — rim aditivo sem conservação (modo seco).** `density = (cw·fill_px + edge)·gran`
  (render.rs:463) soma o edge por cima do fill pleno; o thinning do interior é gated em
  `wet>0` (:457-459) → no default o traço lê como "wash uniforme com CONTORNO", não como wash
  seco com pigmento migrado; subir Edge desloca o tom médio. Curtis §4.3.3: *"the pigment
  MIGRATES from the interior... leaving a dark deposit at the edge"* (MovePigment é
  conservativo). **Correção candidata (1-5 LOC, MUDA O LOOK DEFAULT — smoke gate):** unsharp
  assinado `density = (cw·fill + gain·(cw − inner)).max(0)·gran` — o lobo negativo (hoje
  descartado) empalidece o interior perto da borda; integral ~zero-sum.
- **EDGE-4 (P2) — rim uniforme ao longo da borda.** Amplitude idêntica pra mesma geometria;
  o campo `wet_soak` (dwell por-pixel) EXISTE mas nunca modula o edge. **Correção (~10 LOC,
  zero reads novos):** `gain_px = edge_gain·(1 + k1·soak_raw)·(0.5 + 0.5·color_alpha)` — rim
  forte onde poolou/demorou, fraco no rabo seco do traço; o rim passa a contar a história do
  gesto.
- **EDGE-5 (P3) — core_r: FENCE.** O cap `min(spread, radius/2)` limita a largura do rim ao
  pincel (dead-zone do Spread em pincéis pequenos; no publicado a largura é escala do MEIO).
  Decisão ratificada (fix do blob, doc 11 §5.2) — **não tocar sem decisão do Enio**; se o smoke
  acusar, o caminho é separar as duas escalas (cap na saturação do core, band próprio pro edge).

Fontes: Curtis 1997 (acima) ·
[DiVerdi et al., "Painting with Polygons" TVCG 2013 (PDF)](https://raw.githubusercontent.com/pkuwwt/tvcg-papers/master/2013/05/Painting%20with%20Polygons%20-%20A%20Procedural%20Watercolor%20Engine.pdf) ·
[handprint.com — backruns](https://www.handprint.com/HP/WCL/tech23.html) ·
[Bousseau et al. 2006 (PDF)](https://artis.inrialpes.fr/Publications/2006/BKTS06/watercolor.pdf).

## §4 — Lente 3: rediluição / lift / mixer direcional

- **MIX-1 (P1) — Charge não depleta.** `pickup = 1 − wet_charge` constante por traço
  (mixer.rs:82; nenhum estado de depleção no `WetMix`). O Charge do Procreate (Handbook,
  fetchado verbatim): *"Like a real paintbrush, the longer you drag your stroke out, the more
  paint it will leave behind... As the brush runs out of paint, the trail of color it leaves
  will become fainter"*. O nosso é semanticamente o Color Rate do Krita / Mix do PS, não o
  Charge. O doc 07 §4.2 mapeava `s = 1 − charge·depletion(t)` — o `depletion(t)` nunca foi
  implementado. **Correção (~20 LOC, O(1)/dab):** `travel: f32` no WetMix (acumula
  ‖dab_i−dab_{i−1}‖) + `fresh = charge·max(0, 1 − travel/(K·radius·(0.5+charge)))`, atuando na
  INTENSIDADE depositada (peak/flow), ativo só atrás do gate `wet_charge < 1` (default
  byte-idêntico). **VEREDITO 2026-07-08: LANDOU no take 4** — houve revert integral no meio
  ("achei esse mix 1 ruim") e retorno a pedido do Enio com a curva perceptual. Ver §W-C.
- **MIX-2 (P1) — doc-drift do pickup + tripla via no regime alto.** Núcleo confirmado: doc 11
  §F2 promete 2× *"o pickup lê a base congelada JÁ-LIFTADA (sb pós-lift)"* como defesa contra
  self-feeding — o código amostra a base **PRISTINA** (mixer.rs:98; o lift só existe no
  composite, nunca persiste). Com Wet e Charge<1 altos, a mesma superfície colore a saída por
  3 vias paralelas (deposit do mixer → `pig`; dissolve tinge `pig`; mix RYB wet-driven contra
  `sb`) — as vias (ii)/(iii) são decisões ratificadas ("knobs independentes", cerca de
  Chesterton), mas a soma no regime alto over-pulla o matiz. **Correção mínima (~3-15 LOC):**
  atenuar o presence do sample pelo lift esperado (`pres × (1 − REWET_LIFT·wet)`) quando o
  mixer está ativo — realinha código↔doc 11 sem tocar as vias ratificadas.
- **MIX-3 (P2) — pincel sempre limpo.** `reset_wet_mix()` incondicional no pen-down
  (lifecycle.rs:54). PS Mixer persiste o reservoir entre traços (helpx/ProEdu: "Clean The
  Brush After Each Stroke" é opção). Default nosso está CERTO (Procreate recarrega por traço).
  **Correção gated:** checkbox "Clean brush each stroke" (default ON = atual byte-idêntico).
- **MIX-4 (P2) — pickup 5-tap fixo.** `sample_surface` = centro + 4 a r/2 (mixer.rs:136-143),
  independente do raio — brush grande subamostra (flicker de reservatório sobre detalhe fino);
  Krita expõe Smudge Radius, Procreate expõe Blur/Blur Jitter. **Correção:** anel determinístico
  de 8 taps (offsets ±0.707 pré-computados, sem trig) acima de r>16 px + slider "Pickup Radius"
  (% do brush, padrão Krita). Custo por DAB.
- **MIX-5 (P2, rebaixado de P0 pelos céticos) — doc stale no contrato.** spec.rs:226-230 diz
  que Pull 0 = *"no carry; the deposit tracks the local surface"* — falso: RETAIN_UNLOAD_MIN
  =0.88 dá half-life ≈5,4 dabs a Pull 0, **por decisão deliberada** (exit-bleed espelha a
  entrada, Enio 2026-07-07, comentário do mixer). **Corrigir o DOC, não o comportamento**
  (cerca de Chesterton) + pinar o half-life num teste.
- **MIX-6 (P3) — assimetria load/unload: manter.** Nenhum engine fetchado usa taxa dupla
  (MyPaint/Krita: taxa única simétrica; PS separa eixos mas sem assimetria temporal) — é
  inovação local na direção que o próprio Procreate descreve pro Charge. Fence de comentário
  já existe; falta só o assert direcional (entrada satura ≤2 dabs; saída retém ≥50% após 5).
- **MIX-7 (P3) — inventário Procreate.** Set real (Handbook): Dilution, Charge, Attack, Pull,
  Grade, Blur, Blur Jitter, Wetness Jitter ("Wetness boost" NÃO existe). Faltam: Attack
  (dobrado em charge/flow — defer ok), Grade (defer ok), Blur (=MIX-4), **Wetness Jitter**
  (barato: `flow_dab = flow·(1 − wj·rand01(splitmix64))` por dab, ~12 LOC, infra do jitter.rs
  já existe).

Fontes: [Procreate Handbook — Brush Studio](https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings) ·
[Krita — Color Smudge engine](https://docs.krita.org/en/reference_manual/brushes/brush_engines/color_smudge_engine.html) ·
[Adobe — Mixer Brush](https://helpx.adobe.com/photoshop/using/painting-mixer-brush.html) (timeout no fetch direto; confirmado por fontes secundárias) ·
[MyPaint smudge via Drawpile docs](https://docs.drawpile.net/help/draw/mypaint.html).

## §5 — Lente 4: granulação e papel (a causa do "mottled")

- **GRAN-2 (P1, MEDIDO) — espectro do PaperCold é blotch, não tooth.** FFT 1024² sobre
  reimplementação bit-fiel do gerador: **~90-98% da energia espectral em λ>32 px** (pico
  ~512 px; mid-band 8-32 px ≈ 2%; fino 2-8 px ≈ 0.2%), uniformidade péssima (std das médias de
  blocos 32 px = 91,7% do std do campo), 4,2% dos pixels clampados em 0. O claim interno
  "crisp tooth (not a soft blob)" é falso **por construção**: o "high-pass" do gerador subtrai
  noise NÃO-correlacionado (soma energia grave em vez de remover). Como o preset Aquarela usa
  PaperCold como papel E mapa de granulação (Same-as-Paper default), multiplicar a densidade
  por esse campo produz exatamente manchas grandes de densidade variável = **"mottled"**.
  Padrão-ouro: Bousseau 2006 usa scanned papers / soma de gaussianas multi-escala; Rebelle usa
  imagens 1024². **Correções:** rota 1 (~10 LOC, só constantes) — re-escalar base 5→~28,
  high-pass real (subtrair a própria octave-abaixo), reduzir contrast; rota 2 (padrão-ouro,
  ~40 LOC + asset 64KB) — pré-cozer num tile grayscale 256² e amostrar bilinear (1 fetch/px —
  hot loop fica MAIS BARATO que as 4 octaves+Worley atuais).
- **GRAN-1 (P1) — granulação instantânea, simétrica e com sinal invertido.** `gran = 1 +
  (h−0.5)·2·amount` multiplica D no instante do composite (render.rs:451,463): contraste pleno
  no traço fresco, não cresce com água/permanência, não acumula entre passadas — e o sinal é
  **invertido vs Curtis**: nossos PICOS (h alto) escurecem; no Curtis §4.5 o depósito é máximo
  nos VALES (`δ_down = g(1 − h·γ)ρ`, one-way ratchet). Em amount alto os texels de h BAIXO
  clampam gran em 0 → speckle branco furando o wash. **Correção Tier-2 (~40-60 LOC, zero
  buffer novo):** dividir D em glaze + depósito valley-gated
  `D = base·(1−k) + base·k·(1 − h·γ)` com `k = granulation·(a + b·wet + c·soak)` lendo o
  `wet_soak` existente, aplicando o peso cheio só no BAKE do pen-up (o "momento de secagem") —
  granulação que assenta na secagem e cresce com a água, a assinatura nº 1 da coisa real.
- **GRAN-3 (P2) — fallback mono-escala.** Sem Paper ativo: 1 octave de value noise a 5 px
  (field.rs:154-156). Medido: mid-band 77,5% (uniforme, isotrópico — bom), mas mono-escala
  full-range lê "digital". **Correção:** 2ª octave (2.5 px, peso 0.35) ou apontar pro tile
  pré-cozido da GRAN-2 rota 2.
- **GRAN-4 (P2) — γ global, sem identidade de pigmento.** Um único `granulation: f32`; o
  pigmento redissolvido pelo Wet granula idêntico ao fresco — a "separation" (Curtis §2.2,
  Fig. 1d) é inexpressável. **Degrau 1 (~10 LOC):** γ_dissolve próprio pro termo do pool do
  rewet (redissolvido re-assenta MAIS granulado — separação barata visível nos blooms).
- **GRAN-5 (P2) — escala.** Slot Paper honra Size/Angle/Offset e o zoom é canvas-anchored
  (correto, igual Painter/Rebelle); mas o fallback é 5 px fixo (cego a Size) e
  TEX_TILE_BASE_PX=256 em px de canvas → mesmo preset dá tooth 8× mais fino num canvas 4096
  vs 512. **Correção (~15 LOC):** fallback lê `paper.size` mesmo com kind=None; preset ancora
  size no tamanho do documento.

Fontes: Curtis 1997 §4.5 (TransferPigment, verbatim extraído do PDF) · Bousseau 2006 (acima) ·
[Rebelle 8 — Visual Settings](https://escapemotions.com/products/rebelle/manual/8/interface/panel-visual-settings/) ·
[Corel Painter — Watercolor and paper texture](https://product.corel.com/help/Painter/540219480/Main/EN/Win-Documentation/Corel-Painter-Watercolor-and-Paper-Texture.html).

## §6 — Lente 5: determinismo / HR-5 (interna)

**Veredito: hot loop genuinamente limpo.** Zero transcendental de libm per-pixel (tudo LUT +
integer hash + soma/mult); RNG semeado (splitmix64/tex_rng por traço); sem HashMap/rayon no
caminho watercolor. Os `.sqrt()` per-pixel (accum.rs:90,148, backdrop.rs:135) são IEEE-754
correctly-rounded — bit-exatos cross-platform, não são transcendentais de libm; precedente
pervasivo na crate. Três achados:

- **DET-1 (P2) — soak frame-rate-dependente.** `add = (127.5·dt).clamp(1.0, 255.0)`
  (backdrop.rs:107): acima de ~127 fps o floor de 1 byte/tick faz a taxa efetiva = fps bytes/s
  (240 Hz despeja 2× o soak/s de 60 Hz); e `(add·w) as u16` trunca — a 60 fps (add=2) todo
  pixel de rim com w<0.5 recebe 0 PARA SEMPRE (o "fading to the rim" vira degrau {0,1,2}).
  **Correção (~15-30 LOC):** acumular com fração preservada (Q8.8 ou dither determinístico
  hash2) e remover o floor.
- **DET-2 (P3) — `transmittance()` sem lerp.** field.rs:73-77 indexa `exp_neg` cru (bin ≈1.55%
  de T) enquanto os gêmeos `absorbance`/`exp_mag` lerpam explicitamente "pra matar o banding".
  Pior caso real ~1.5-2 bytes/degrau em wash raso de gradiente suave. **Correção: 1 linha**
  (`transmittance = exp_mag(−lnl[c]·od)`).
- **DET-3 (P3) — LUTs de runtime ← libm.** powf/ln/exp na construção (1× por processo):
  determinístico intra-plataforma, mas conteúdo pode variar 1 ULP entre Linux/macOS/Windows —
  caveat SÓ se um dia houver golden-hash cross-platform do painter (hoje não há). Nota de doc.

## §7 — Lente 6: perf — MEDIDO (release, @2048², workstation Linux)

Números crus da sonda (`watercolor_perf_frame_cost_probe`, 440 moves):

```
wet 0: first-40 0.237 ms · last-40 0.237 ms · max 0.546 ms · commit  7.893 ms
wet 1: first-40 0.476 ms · last-40 0.459 ms · max 0.796 ms · commit 13.061 ms
guard spread alto (watercolor_high_spread_frame_cost_bounded): verde (razão hi/lo dentro do 8×)
suite watercolor: 33 passed / 0 failed / 2 ignored (sondas)
```

- **PERF-3 (P3):** claims do doc 11 §5.1/§5.2 **confirmados** (<2× em tudo; first-40 == last-40
  comprova custo constante = dirty-rect incremental funcionando). Ressalvas: o guard é
  ratio-based **e** `#[ignore]` — nenhuma sentinela absoluta roda no CI; e o número
  "wet 1,32 ms @spread48" do doc 11 não tem linha de sonda que o reproduza (adicionar
  `live_ms(48, false)` ao guard, ~2 LOC).
- **PERF-1 (P1, 2× CONFIRMED) — gate incremental≡full não cobre o regime do fix.** O único
  teste de equivalência roda com spread=12/wet=0 → nunca constrói RewetFields nem alcança
  ds>1 (`REWET_DS_SPREAD=12` foi definido "acima de todo Spread de teste" — ou seja, o
  downsample NUNCA é exercitado por teste). O invariante que legitima o fix de perf do Spread
  alto (grid global-alinhado) é garantido só por comentário. **Correção (~45 LOC de teste):**
  variante com wet=1, spread=48 (ds=4) + `paint_tick` no meio (ativa soak/far-fields),
  tolerância ±2 (bilinear).
- **PERF-2 (P2) — pen-down não medido.** O `build_wet_backdrop` (composite_below FULL-CANVAS,
  1×/traço) fica fora do Instant da sonda — latência de encostar a caneta em documento grande
  /N camadas é invisível. **Correção (~5 LOC):** cronometrar o Down na sonda; só discutir
  composite preguiçoso SE o número passar de ~5 ms.

## §8 — O que NÃO mexer (fences ratificadas, cerca de Chesterton)

1. **core_r cap** (render.rs:216-220) — fix do "blob chato" @Spread>24, medido e ratificado
   (doc 11 §5.2). Mexer = Enio-gated (EDGE-5 dá o caminho SE o smoke reclamar).
2. **Retenção assimétrica + exit-bleed em Pull 0** (mixer.rs:34-42) — decisão perceptual
   deliberada (Enio 2026-07-07); a indústria usa taxa única mas a nossa direção é a que o
   Procreate descreve. Só corrigir o doc stale (MIX-5) + assert direcional (MIX-6).
3. **Rewet ⊥ mixer como knobs independentes** (doc 11 F2 §Interação) — as "3 vias" do MIX-2
   não são bug em si; a correção é só o presence-atenuado prometido pelo próprio doc 11.
4. **Film Beer–Lambert** — é KM S=0 correto (Curtis §5.2); F4 fica FORA do film.

## §9 — Plano recomendado (ordem de ataque, todas Tier-2)

| Wave | Conteúdo | Achados | Por quê primeiro |
|------|----------|---------|------------------|
| W-A "cor" | Mistura subtrativa nos 3 sites (KM single-constant ou absorbância-LUT) + recalibrar card F4 | OPT-1, OPT-2, OPT-5 | Pivot ADR-0096 literal; barato; maior ganho de matiz |
| W-B "papel" | Espectro do papel (rota 1 ou 2) + deposição valley-gated + fallback 2-oct | GRAN-1, GRAN-2, GRAN-3 | Mata o "mottled" reportado; é o look |
| W-C "comportamento" | Charge depletion (✓ take 4) + wet map persistente + água limpa/backrun + rim assinado/modulado | MIX-1 ✓, EDGE-1, EDGE-2, EDGE-3, EDGE-4 | O feel; EDGE-3 muda o default → smoke gate |
| W-D "higiene" | Docs stale (MIX-5, OPT-5, Spread "1..24"→48 em painter_watercolor.rs:27) + testes (PERF-1/2/3, MIX-6) + DET-1/2 | resto | Barato, fecha claims falsos e buracos de gate |

Extras candidatos da lente UX (decisão junto com o redesign do painel, doc à parte): Wetness
Jitter (MIX-7), Pickup Radius (MIX-4), Clean-brush toggle (MIX-3), Body/opacos (OPT-3),
Dryness/dry-brush e Water-brush (EDGE-2 destrava).

## §10 — Rastreabilidade

- Workflow `watercolor-gold-audit` (47 agentes, 0 erros): 6 lentes + painel + UX, verificação
  adversarial 2×(P0/P1)/1×(P2/P3); fact-check das fontes de apps (8 URLs re-fetchadas — nenhum
  nome de parâmetro inventado; 5 ajustes menores incorporados).
- Rebaixamentos/ajustes dos céticos incorporados: MIX-5 P0→P2 (doc, não comportamento);
  GRAN-2 P0→P1 (perceptual, não blocker); MIX-2 reframed (vias ratificadas ≠ bug); OPT-1
  ("único caminho" → "principal caminho de mistura de matiz").
- Código lido inteiro pelo orquestrador: watercolor_render/field/accum/mixer/backdrop/smudge/
  settings + stroke_lifecycle + paint_watercolor(_paper).rs + spec.rs + ryb_mix.
- Medições: sondas release @2048² (números crus em §7); FFT do PaperCold (GRAN-2).
- Fontes externas (todas fetchadas na sessão): Curtis 1997 · Bousseau 2006 · DiVerdi TVCG 2013 ·
  Mixbox TOG 2021 (+ site) · Procreate Handbook · Krita docs · Rebelle 7/8 manual (4 páginas) ·
  Corel Painter docs (2 páginas) · Adobe helpx (indireto) · MyPaint/Drawpile · handprint.com.

---

## Landing notes

### W-A · OPT-1 — LANDOU 2026-07-08 (mistura subtrativa no Wet Mix)

Conferido ANTES por sonda (ordem do Enio "confira se já funciona"): azul × poça amarela (Charge
0,3) depositava **(128,128,115)** — o cinza R≈G previsto. Fix: reservatório + depósito em
**absorbância** per-canal (LUTs `lnl`/`exp_mag`, float end-to-end). Medido DEPOIS:
**(128,139,68)/(101,119,81)** na saída da poça — verde G-dominante, batendo a previsão numérica da
verificação adversarial. Gate `carried_colour_is_saturated_not_watery` re-pinado JUNTO (roxo
azul×vermelho vira púrpura-marrom R-dominante, pigmento real; B−G 40→15). Discriminante permanente:
`watercolor_wet_mix_blue_over_yellow_deposits_green`. Segue em aberto na W-A (opcional): o mesmo
upgrade no `smear_dab` do Smudge; OPT-4 (lerp no `transmittance`) fica pra smoke próprio (muda
todos os bytes do watercolor por natureza).

### W-B · GRAN-1/2/3 — LANDOU 2026-07-08 (papel mid-band + deposição nos vales)

**GRAN-2 rota 2:** presets Paper = tiles 256² seamless pré-cozidos (periódico por construção,
oitavas 2-32px + high-pass real = box-blur 32px do próprio campo subtraído). Métrica da auditoria
razão bloco32/campo: **0,917 → <0,35** (pinado por teste + costura). Hot loop = 1 fetch bilinear
(mais barato). **GRAN-1:** deposição valley-gated `(1 − k·h·γ)`, γ=0,9 — sinal Curtis correto, sem
speckle; settle: 3 takes no mesmo dia (Enio 2026-07-08) — (1) pleno-só-no-bake popava na soltura; (2)
WYSIWYG live==bake matou a secagem; (3) **final: secagem REAL no bake (settle 1.0) + preview vivo
QUASE-seco** (BASE 0.80 + headroom de água 0.12/0.12, capado no seco) → a soltura lê como
assentamento sutil fiel à física. Knobs de calibração nomeados (GRAN_SETTLE_*, loop de smoke com
o Enio). Amount 0 =
byte-idêntico. **GRAN-3:** fallback 2 oitavas (5px + 2.5px·0,35). Gotcha de teste: `sample_image`
usa convenção dab-space (`u·0,5+0,5` → 1 unidade de tile = meia imagem). Segue aberto: GRAN-4
(γ por-pigmento / dissolve), GRAN-5 (escala do fallback), W-C.

### W-C · MIX-1 — LANDOU 2026-07-08, take 4 (4 takes + 1 revert integral no mesmo dia)

**Estado final:** depleção como **mapa u8 por-pixel** `stroke_deplete` multiplicando `fill + edge`
no composite (cobertura/água intocada), `fresh` começando em 1.0 com `span = 120·r·charge/(1−charge)`,
carry pesado pela absorbância real do reservatório, rampa de 15% do raio no splat, e **decaimento
QUADRÁTICO em densidade** (`(1−u)²`, take 4): o Beer–Lambert comprime o extremo escuro, então
linear-em-densidade lia "despenca no fim" (Enio); o quadrático adianta a perda onde o olho não vê
e pousa suave onde vê — taxa percebida constante. Após o take 3 o Enio pediu revert integral
(`154cead4`, "achei ruim") e depois o retorno com esta curva ("vamos voltar com o mix mas com
decaimento ainda mais linear"). Histórico dos takes:

- **Take 1** (`fresh = charge·(1−travel/span)` escalando a COBERTURA): Charge <0,93 matava a borda
  — cobertura sub-saturada deixa `inner < 1` no interior INTEIRO e o edge term (`cw·(1−inner)·gain`)
  inunda o centro (slab opaco); cruzar poça pálida explodia pigmento (`depl = max(fresh, t)` com
  `t` = peso de MISTURA, que salta pra ~1 em qualquer poça).
- **Take 2** (depleção como mapa u8 por-pixel `stroke_deplete` multiplicando `fill + edge` no
  composite, cobertura intacta; `fresh` começando em 1.0 com `span ∝ charge/(1−charge)`; carry
  pesado pela absorbância real do reservatório): fisicamente coerente, MAS decaimento rápido
  demais e costuras duras pixeladas no cruzamento (mapa binário 255/0 na borda do disco ×
  nearest-sample warpado).
- **Take 3** (span 3× + rampa de 15% do raio no splat): resolveu as costuras; reprovado no feel →
  revert integral (`154cead4`). **Take 4** (retorno a pedido): curva `(1−u)²` = fade perceptual
  constante. Calibração: `MIX_DEPLETE_SPAN = 120` (sobe = fôlego maior), `MIX_CARRY_FULL_ABSORB
  = 2.0` (sobe = smudge mais tímido), expoente da curva (2 = atual; maior = fim ainda mais suave).
- **Lições que FICAM** (valem pra qualquer feature futura): (a) **cobertura é GEOMETRIA DE ÁGUA
  saturada** — nunca a escale; "enfraquecer" o traço = modulação de densidade por-pixel depois do
  rim derivar da cobertura intacta (mesma lição do grey-tip normalise, §Shape); (b) qualquer mapa
  por-pixel novo lido pelo composite precisa de taper na borda do dab (nearest + warp transforma
  degrau em escada); (c) intensidade de smudge deve pesar o pigmento REAL do reservatório
  (absorbância), nunca o peso de mistura `t`.


### W-C · EDGE-1 — LANDOU 2026-07-09, take 2: SESSÃO MOLHADA (take 1 reprovado no smoke)

**Take 1 (atenuação de edge, 2026-07-08):** `edge *= 1 − umidade` suprimia só o rim NOVO — o
smoke mostrou que o contorno duplo é dominado pelo **rim já assado** do traço anterior visível
através do wash de cima ("acho que não funcionou"). Atenuar não basta: tem que FUNDIR.

**Take 2 (final): enquanto o papel está molhado, traços consecutivos são UMA sessão molhada.**
- `canvas_wet` u8 persiste pós-pen-up (pour da cobertura ENDURECIDA smoothstep SS0/SS1, pós-bake,
  max-blend + rect vivo); seca no heartbeat a `CANVAS_WET_DRY_PER_S = 30` bytes/s ≈ 8,5 s
  (DiVerdi/Adobe TVCG 2013); seco ⇒ dropa mapa + sessão (fast path, custo ocioso zero).
- **Sessão** = buffers do traço (coverage/color/density/deplete + cum rect) NÃO zeram no pen-down
  enquanto `wet_session_continues()`: o composite re-renderiza a **UNIÃO sobre a base da sessão**
  (`wet_session_base`, congelada no 1º traço) — um wash, UM rim ao redor da união; o rim interno
  do traço anterior **derrete no re-bake**. Cruzar rápido NÃO escurece (cobertura max-blend =
  wash único); esperar secar e repassar = glazing escurece — igual aquarela real.
- **Guarda de invalidação sem hooks:** `wet_session_canvas` = o Arc exato do canvas que o NOSSO
  bake produziu; qualquer mutação alheia (undo, troca de camada, fill, resize, outro tool) troca
  o Arc ⇒ `Arc::ptr_eq` falha ⇒ sessão nova. **Undo por traço continua exato** (o snapshot
  pré-traço já contém a união anterior assada).
- `watercolor_base` (re-congelada por traço, contém a união assada) segue servindo **mixer
  pickup + rewet** — cruzar o vizinho na mesma sessão capta a tinta dele normalmente.
- **Gotchas resolvidos no caminho:** (a) smudge muta a base DA SESSÃO (o `make_mut` bifurcava os
  dois Arcs e o composite lia a base não-esmeada) + re-share no 1º traço; (b) mapa de depleção é
  da sessão → ao dimensionar, **backfill 255 sob a cobertura pré-existente** (senão o re-bake
  multiplicava as poças por 0 e elas SUMIAM) e traço mixer-off splatta 255 com o mapa vivo;
  (c) alpha da cor volta a × `depl` (exato, do mixer) — pincel esgotado não re-tinge poça molhada
  em prioridade cheia (na união a poça re-renderiza com a cor contaminada).
- Atenuação de edge do take 1 REMOVIDA (na união ela derreteria o próprio rim no re-bake).
- ~~Caveat documentado: trocar parâmetros do brush no MEIO da sessão re-estiliza a união~~ —
  **PROMOVIDO A BUG (Enio smoke 2026-07-09):** Concentration 1.0 → 0.3 entre traços da sessão
  converte o traço 1 pra 0.3 no pen-up. **1º da fila de amanhã** (doc 13, topo): params por-traço
  na união (candidato: dobrar no mapa de reserva por-pixel).
- Teste: `watercolor_touching_wet_washes_merge_without_double_rim` (junção molhada: rim de B
  ausente E rim interno de A derretido, ~100 G mais claro que seco-primeiro; mapa seco dropado).
  Knob: `CANVAS_WET_DRY_PER_S` (desce = janela de fusão maior).

### W-C · EDGE-2 — LANDOU 2026-07-09 (backrun/bloom de água limpa)

O gesto canônico era **inalcançável por construção** (Dilution 1 → `flow = 0` → cobertura zero →
`cw ≤ 0` pulava todo o caminho). Agora:
- **Canal de água próprio** `stroke_water` (u8, session-scoped como os buffers da união; SEPARADO
  do soak de dwell — reusar o `wet_soak` fez o anel nascer sob o pincel parado e quebrou o teste
  do dwell): Dilution poura por dab (`água·wgt·keep`, max-blend) independente do pigmento; os
  splats de pigmento/dono são gateados em `peak > 0` (água não rouba ownership nem reserva).
- **Composite**: pixels `cw ≤ 0` mas com água > 0 seguem vivos (early-out só quando ambos secos);
  `water` lido em coordenada SERRILHADA (`water_at`: value-noise célula 12 px, ±5 px — o contorno
  couve-flor; Curtis §2.2). Lift `max(wet·cw, água)` (whitened wake), dissolve `max(wet, água)`,
  e o **anel** `(água − halo)⁺` (halo = blur 2× do canal, novo campo `water_halo` nos
  RewetFields) deposita densidade no contorno.
- **Concentração do anel** (`BACKRUN_CONC = 1.5`): Beer–Lambert satura NA cor do pigmento — só
  densidade nunca fica mais escuro que o wash de origem; o backrun real é pigmento CONCENTRADO,
  então o anel aprofunda a absorbância do pigmento dissolvido (o "severely darkened edge").
- Física emergente: gota pequena (raio ≲ blur 12 px) = bloom todo-anel; poça grande = interior
  lavado + anel no contorno. Água em papel em branco = invisível (tudo ∝ presença de tinta).
- Knobs: `BACKRUN_POOL = 2.0` (força do anel), `BACKRUN_CONC = 1.5` (escurecimento),
  `BACKRUN_JAG_CELL = 12` / `BACKRUN_JAG_PX = 5` (serrilhado).
- Default byte-idêntico (Dilution 0 = inerte); 492/492. Teste:
  `watercolor_clean_water_backrun_blooms_on_wet_wash` (interior clareia · anel escurece ·
  branco intacto). LOC: bloco rewet por-pixel extraído p/ `watercolor_rewet_px.rs` (verbatim).
- Nota: Dilution segue TAMBÉM diluindo a cobertura do pigmento (`flow`), como antes — a água
  extra agora existe em paralelo. Rebalancear os dois é calibração de smoke.

### W-C · EDGE-3 + EDGE-4 — LANDOU 2026-07-09 (SMOKE-GATE: mudam o look default do watercolor)

**EDGE-3 — rim assinado com conservação (Curtis §4.3.3):** o edge vira unsharp ASSINADO
`gain·(cw − inner)` — o lobo negativo (franja, `inner > cw`), antes clampado fora, EMPALIDECE: o
pigmento que escurece a borda migrou do interior. Junto, `inner` passou a ser blur da cobertura
**endurecida** (smoothstep SS0/SS1) em vez da crua: o platô do feather (~0,93) deixava resíduo
`+0,07·gain` no interior INTEIRO — "subir Edge desloca o tom médio", a reclamação literal da
auditoria. Perfil medido (fill 0,5/depth 2/gain 3): interior 39→39 (tom não desloca) · rim 39→26
· franja 205→255 (empalidece). Teste 3-propriedades: `watercolor_signed_rim_pales_the_fringe`.

**EDGE-4 — o rim conta a história do gesto:** `gain_px = edge_gain·(1 + 0,5·soak)·(0,5+0,5·alpha)`
— mais forte onde a água poolou/demorou (dwell), mais fraco onde o depósito foi tênue (cauda seca
de um traço depletado). Zero reads novos (soak + alpha da cor já existiam por-pixel; hoisted e
reusados pelo settle/pigment). Knob: `EDGE_SOAK_BOOST = 0.5`. Teste:
`watercolor_rim_strengthens_where_the_brush_dwelled` (rim em meio-tom — no escuro o Beer–Lambert
comprime o boost). Gotcha de teste: edge_gain alto satura o clamp (≤1) e esconde qualquer
modulação.

LOC: `style_at`/`paper_h_px` extraídos p/ os siblings (render/field exatos em 700). 494/494.
**Ambos mudam o look default do modo aquarela** — vetos revertem individualmente
(EDGE-4 = só o `gain_px`; EDGE-3 = fórmula + blur endurecido).

### W-C · Reprodutibilidade da sessão — o retângulo que clareava a poça vizinha (LANDOU 2026-07-09)

**Bug (Enio smoke):** com uma poça úmida na tela, o segundo traço fazia sua área retangular de
ação aparecer como artefato em tempo real, CLAREANDO a poça vizinha antes de tocá-la (some no
mouse-up). Raiz: o composite da união NÃO era função pura do estado da sessão — o re-render da
janela viva produzia bytes diferentes do bake para pixels já assados. Cinco vazamentos:

1. **Campos de rewet lidos do base per-stroke** (recongelado a cada pen-down ⇒ contém a poça 1
   assada): no bake da poça os campos eram zero (canvas virgem); na janela do traço 2 viravam
   presença > 0 ⇒ o dissolve tingia o pigmento pra cor PÁLIDA da aparência assada (o
   clareamento), o pool dobrava o rim, o wet_paint ligava o mix RYB. **Fix:** campos ← base da
   SESSÃO (o seco embaixo da sessão inteira); vizinho de sessão é água viva, fundido pela união
   (EDGE-1), não "tinta velha".
2. **Settle da granulação gated pela flag `commit` do frame:** a poça assada (settle 1.0)
   re-renderizava viva com ~0.80 — a textura mudava dentro do retângulo e voltava no mouse-up.
   **Fix:** `settled = commit || dono-do-pixel já commitado`.
3. **Soak zerado a cada pen-down:** o dwell da poça 1 sumia ⇒ o rim EDGE-4 dela re-renderizava
   mais fraco. **Fix:** o soak PERSISTE na sessão (zera só em sessão nova; o teardown limpa).
4. **Geometria do brush VIVO aplicada a pixels alheios:** `core_r` (raio do blur do `inner`) e
   `spread_thin` eram globais ⇒ trocar Size/Spread re-blurava o rim da poça assada (o "qualquer
   mudança no brush propaga", doc 13). **Fix:** ambos na `WetStrokeStyle` por-dono; o `inner`
   ganha um blur por raio DISTINTO da sessão (`inner_blur_set`, normalmente 1 mapa); janela e
   raios de campo usam os MÁXIMOS da sessão (fold único sobre a tabela).
5. **Pixels só-água sem dono** resolviam pro estilo do brush vivo (depth/cor do anel derivavam
   com o traço seguinte). **Fix:** o splat de água reivindica dono em pixel VIRGEM (nunca rouba
   pixel com pigmento).

**EDGE-2 preservado por campos UNION (`UnionFields`, watercolor_rewet_px.rs):** o anel/tinta do
backrun sobre wash de sessão (ainda molhado) agora lê os próprios buffers da união — cobertura
endurecida × alpha depositado + cor crua, mascarados dono ≠ traço vivo (sem auto-anel em traço
com Dilution) — com blur near/far espelhando os campos secos. Água ESVAZIA o wash que redispersa
(`lift_wash` multiplica a densidade ANTES do pool: o interior empalidece e a massa volta como
anel). Sobre pintura SECA (sessão nova) nada muda: base da sessão ≡ base per-stroke no 1º traço.

Extra da mesma família: `stroke_density` criado no MEIO da sessão (trocar pra tip texturizado)
agora backfilla 255 sob a cobertura existente (gêmeo do backfill do deplete — os washs
anteriores liam densidade 0 e o interior deles sumia).

Testes (refutáveis, FAIL provado pré-fix):
`watercolor_session_rerender_reproduces_the_bake_byte_exact` (probe byte-exato vivo E pós-Up,
com dwell + granulação + Bleed) · `watercolor_session_brush_changes_do_not_touch_baked_washes`
(brush radicalmente trocado entre traços) · guarda verde-antes-e-depois:
`watercolor_clean_water_backrun_blooms_on_wet_session_wash` (EDGE-2 sobre wash MOLHADO).
497/497 · clippy 0 · render 700 / field 694 / paint 600.

Residual de 2ª ordem (documentado, aceito): os raios de blur dos CAMPOS (secos e union) usam o
máximo da sessão ⇒ um traço de Spread maior entrando numa sessão de GLAZE (campos ≠ 0) pode
derivar levemente o tint re-renderizado; e o anel de um traço Dilution+pigmento pode
re-renderizar diferente sob um 3º traço (o `bp_union` exclui só o traço VIVO). Ambos estreitos
vs o estado anterior, que derivava em TODA re-render.

SMOKE-GATE (muda o look de casos já vistos): (a) traço com Bleed cruzando/margeando poça da
MESMA sessão não "re-molha" mais a vizinha — fusão pura EDGE-1 (o comportamento antigo era o
próprio bug); (b) o anel do backrun sobre wash molhado vem do pigmento CRU da união (mais
saturado que a aparência assada de antes) e o interior esvazia por `lift_wash`. Sobre pintura
seca, byte-idêntico ao aprovado.

#### Take 4 — reversão + knobs + o ajuste cirúrgico de custo zero (Enio: "não resolveu e caiu FPS")

Ordem do Enio: reverter takes 2/3 inteiros (d42349dd — sai StyleFields/difusão, claim por
dominância, water_soft, CONC_CAP e os builds por-frame que derrubaram FPS; o retângulo visto na
build do take 3 morre junto — os guards byte-exatos do 2e11f691 seguem verdes) e atacar por
CALIBRAÇÃO. Knobs (3c863f65): **CANVAS_WET_DRY_PER_S 30→15** (janela 8,5s→17s — 8,5s expirava
entre traços e o traço seguinte GLAZEAVA com silhueta dura by-design; hipótese-chave do "não
resolveu") · BACKRUN_JAG_PX 5→3 · BACKRUN_POOL 2.0→1.6 · CONC mantido 1.5 (1.2 clareava o anel
aprovado além do guard).

Smoke seguinte (cruz rápida, mesma sessão, MESMOS params + Dilution): costura fina seguindo a
fronteira de roubo de dono do traço novo. Forense harness (V-primeiro/H-último, coluna pela
junção): degrau 29 bytes/px na linha do dono. Mecanismo: os DOIS reads não-rampados do bloco
d'água — `lift_wash` multiplicava a presença union CRUA (lp_u, cliff de 1 px na cobertura
endurecida + máscara de dono) e o deepen do CONC entrava full-strength no gate. **Fix de custo
ZERO** (nenhum buffer/blur novo — 8f6…): `lift_wash` lê a presença BORRADA `bp_u` (JÁ amostrada
pro anel) e `backrun ×= bp_ring` (um multiply). Medido: 29→9 bytes/px (refutável, stash-provado).
Teste permanente: `watercolor_water_junction_owner_line_is_smooth` (≤15). 498/498 · clippy 0.
