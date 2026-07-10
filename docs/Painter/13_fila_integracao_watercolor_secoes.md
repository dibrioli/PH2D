# 13 — Fila: integração das seções do painel ao sistema Watercolor

> **Origem (Enio, 2026-07-07):** pesquisa mapeou quais seções do painel Painter fluem (ou não) para o
> render-path watercolor. O ponto de bifurcação é `stamp_route.rs::stamp_dabs` — com watercolor ativo o
> batch desvia para `accumulate_wet_*` ANTES de todo o roteamento normal. O que sobrevive é o que já vem
> dentro do `Dab` (gerado pela engine de stroke antes do desvio): pressão (radius/coverage), Strength,
> Randomize Color, Jitter Scale, **Symmetry** (espelhada na engine), Stabilizer, métodos básicos de Stroke.
> Grain é re-propositado (vira o mapa de granulação). O resto é ignorado silenciosamente → esta fila.

## ✅ Feito

- **Seleção + máscara de proteção** (2026-07-07, `0eaa3501`): o watercolor pintava através de seleção
  ativa e região protegida. Fix em 3 camadas keyed no mesmo `keep` (`splat_keep`): splat gates (o wash
  não FORMA em texel gateado → rim/bleed reagem na borda, look de masking-fluid), keep-lerp no composite
  (semântica de restore exata, à prova do vazamento por warp) e snapshot/restore da base no Smudge.
  Teste: `watercolor_respects_selection_and_protection_masks` (com Ragged Edge ligado).

## ✅ #1 — Shape "Automatic" (LANDOU 2026-07-07, `5000decc`)

Implementado conforme a spec abaixo: checkbox na Shape (modo aquarela, default marcado =
byte-idêntico, provado por teste de continuidade via o seam real), `Falloff::Watercolor` novo
(wire 10, bit-idêntico ao feather — avaliado no `t` original, round-trip `1−p` perdia bits),
engine `WetShapeStamp` nos dois splats com RNG replay-sync entre os passes. Hardness participa via
`falloff_weight`. Deferidos anotados: Shape Tone ramp no stamp (item #7) e `dab_flatten`.
Jitter Rotate (#6) resolvido de brinde. **Round 2 (`1d4ecc36`, pós-smoke):** Flatten/Rotate
integrados ao envelope (fp.falloff_t quando não-identity; identity mantém dn bit-exato) +
**normalização da ponta** (1/max_lum por traço): cobertura watercolor é geometria de molhado
max-blend que precisa SATURAR (cw→1 corpo, inner→1 rim) — luminância tonal crua deixava centro
pálido e rim morto. Ponta cinza uniforme == branca byte-a-byte; textura relativa sobrevive.
**Round 3 (`57639e65`):** ponta TEXTURIZADA mantém a aquarela típica — split molhado/pigmento:
wet = envelope saturado (imagem via rampa TIP_WET 0.03→0.20; procedural = só falloff) + density =
a textura, acumulada em `stroke_density` (per-stroke, max-blend) e multiplicada no fill do
composite. Corpo molhado + rim no contorno EXTERNO + textura como variação de pigmento; pigmento
0 = "só água". `watercolor_render` re-split pro teto LOC (RewetFields+consts → field).

## 🎯 spec original — Shape "Automático" (Enio, 2026-07-07)

**Modo aquarela apenas**, na seção **Shape**: um checkbox **"Automatic"**, **marcado por default**.

- **Marcado (default):** exatamente o comportamento atual — silhueta procedural própria da aquarela
  (disco feather 2-segmentos 1,0→0,92@0,62→0, `watercolor_accum::feather`, + endurecimento SS0/SS1 +
  warp fractal). **Visual byte-idêntico ao de hoje — inegociável.**
- **Desmarcado:** abre os itens COMPATÍVEIS da seção Shape para configurar a silhueta da aquarela:
  - **Falloff** — como não existe preset equivalente ao feather da aquarela, **criar um Falloff novo
    nas opções** (ex.: "Watercolor" — a curva 2-segmentos 0,62/0,92 exata) que vira o selecionado ao
    desmarcar (transição contínua: desmarcar sem mexer em nada = mesmo visual). NOTA: `Falloff` é enum
    do brush (`MAX_FALLOFF` — verificar se há gate de contrato na superfície antes de estender).
  - **Shape image** (silhueta custom) + rotação **Angle/Rake/Random** + **Jitter Rotate** — via
    `sample_unit` + `dab_basis` por dab (o padrão da rota de Smear). Rake = pincel chato de aquarela.
  - **Hardness** — mapear para o endurecimento/feather se fizer sentido; senão fica inativo (dim).
- **Onde muda:** só o carimbo de cobertura (`accumulate_wet_coverage` + `accumulate_wet_color`, que
  compartilham o feather) — TODA a dinâmica (rim, bleed, thinning, granulação, rewet) vive rio abaixo
  da cobertura e segue qualquer silhueta automaticamente.
- **Cuidados mapeados na pesquisa:** endurecimento SS0/SS1 é calibrado pro range do feather (silhueta
  de máximo < 0,60 afinaria o wash → renormalizar); pontas esparsas fazem o edge-darkening contornar
  furos internos (provavelmente bonito, exige smoke); custo por-dab cacheável via `StampMask`
  (bake silhueta-only); o mixer (5-tap) e o depósito por prioridade seguem funcionando (amostram, não
  desenham).

## 🎯 LOTE DA JORNADA 2026-07-11 (ordem do Enio, 2026-07-10)

> A próxima jornada trabalha ESTE lote (tabela abaixo + bloco "Investigações 2026-07-10"):
>
> **UI/feature aquarela:** **#9** Botão Dry (secar já) · **#10** Botão Wet (molhar canvas) ·
> **#11** Slider de tempo de secagem (default novo: 10 s — `CANVAS_WET_DRY_PER_S = 25.5`,
> Enio 2026-07-10: 60 s era "extremamente alto") · **#12** preview de umidade on-canvas +
> secagem PAULATINA em tempo real influenciando a mescla.
> UI pelos 4 sites de registro de painel + Widget Gallery
> ([[feedback_docked_panel_registration_four_sites]]).
>
> **Investigações:** **#13** retângulos + re-estilo da união ao mudar paper/grain (hipótese
> forte já mapeada — começar estendendo o guard aos params não-capturados) · **#14** retângulos
> do Per-Layer Color no brush comum (método BUGS #8) · **#15** perf do Per-Layer Color
> (checklist de 6 otimizações da aquarela) · **#16** pesquisa traço-3D (height-map + lighting
> vs Per-Layer Color, doc de design com medição).

## Fila (demais gaps, sem ordem decidida)

> ✅ **RESOLVIDO 2026-07-09 (rota a+b combinadas):** tabela de estilos por-traço da sessão
> (`WetSessionStyles` em `watercolor_field.rs` — fill/depth/edge_gain/wet/granulation/warp/
> pigment_mix/cor-fallback, capturados com os clamps EXATOS do composite no pen-down) + mapa u8
> de DONO por-pixel (recência, casa com o source-over da cor; splat no coverage pass). O composite
> resolve os params do dono por pixel (leitura warpada; warp usa o dono pré-warp); geometria/
> campos (pad, rewet build, soaked) usam MÁXIMOS da sessão (conservador). Owner 0 / sem estilos =
> caminho antigo bit-idêntico (491/491 verdes). Fica global por composite (documentado): core_r/
> spread do blur, fonte da textura de grain, spread_thin, paper. Teste refutável:
> `watercolor_session_keeps_each_strokes_style` (wash 1 byte-exato pós-união; FAIL sem o fix).
> Bloco de granulação extraído p/ `granulation_factor` no field (teto LOC).
>
> ~~**1º DA FILA (ordem do Enio, 2026-07-09): BUG da sessão molhada — parâmetros por-traço.**~~
> Traço 1 com Concentration 1.0 + traço 2 com 0.3 na mesma sessão ⇒ no pen-up o re-bake da
> união re-estiliza o traço 1 com 0.3 (o composite lê os params CORRENTES do brush pro conjunto;
> era o caveat documentado do EDGE-1 take 2 — confirmado no smoke, não é aceitável). Rotas a
> avaliar amanhã: (a) dobrar Concentration (e candidatos: fill/depth) no **mapa por-pixel de
> reserva** no splat (o `stroke_deplete` já multiplica fill+edge — carrier natural, cuidado com o
> caminho byte-idêntico do default); (b) mapa u8 de índice-de-traço + snapshot de params por
> traço da sessão (geral, mais estado); (c) params divergentes ⇒ encerra sessão (barato, mas
> perde a fusão — último recurso). Escolher a que preserva o caráter POR TRAÇO mantendo a fusão.
>
> ~~**Sintoma adicional (Enio 2026-07-09):** QUALQUER mudança no brush com poças úmidas propaga
> pelas poças dentro da área RETANGULAR de ação do brush~~ — **RESOLVIDO 2026-07-09**, junto com
> o retângulo-que-clareia-sem-mudar-params (raiz mais funda que os params do #1: o composite não
> era função PURA do estado da sessão — campos de rewet lidos do base per-stroke envenenado,
> settle na flag do frame, soak zerado por traço, core_r/spread_thin do brush vivo, px de água
> sem dono). Doc 12 §"Reprodutibilidade da sessão". Testes:
> `watercolor_session_rerender_reproduces_the_bake_byte_exact` +
> `watercolor_session_brush_changes_do_not_touch_baked_washes`.

| # | Item | Estado hoje | Nota |
|---|---|---|---|
| 2 | **Tiling** | ignorado (a replicação `tiled_dabs` vive em `stamp_dabs_routed`, depois do desvio) | replicar os dabs antes do accumulate; o composite/dirty-rect precisa do wrap também |
| 3 | **Stroke shape-editors** (Curve/Circle/Polygon/Free Hand) | deliberadamente plain (stampam sem lifecycle → sem base congelada) | dar lifecycle/óptica aos bakes dos editors |
| 4 | **Blend dropdown** | nunca consultado (depósito source-over + óptica própria) | decisão de design: suportar × esconder/dim em modo aquarela (honestidade da UI) |
| 5 | ~~**Composite Brush**~~ | ✅ escondido em modo aquarela (`a7712f45`, decisão Enio); Strength não some junto | — |
| 6 | ~~**Jitter Rotate**~~ | ✅ resolvido com o #1 (silhueta orientável, `5000decc`) | — |
| 7 | **Shape Tone ramp / Per-Layer Color** | ignorados | avaliar semântica em aquarela (tone da silhueta?) |
| 8 | **Alpha-lock da camada** | não aplicado no bake | mesma família do fix de Seleção; avaliar keep = alpha existente |
| 9 | **Botão "Dry" (secar rápido)** (Enio 2026-07-09) | ✅ LANDOU 2026-07-11 (`82924b7c`, `dry_session_now`) | zera o `canvas_wet` + encerra a sessão molhada na hora; "Dry the layer" do Rebelle. Card **Wetness** |
| 10 | **Botão "Wet" (molhar canvas)** (Enio 2026-07-09) | ✅ LANDOU 2026-07-11 (`82924b7c` + **#3 smoke** `wet_canvas_now` reabre sessão sobre a tinta existente + rewet forçado `WET_CANVAS_REWET=0.8`) | "Wet the layer" do Rebelle: traços feitos depois LEVANTAM/fundem a tinta seca mesmo com Rewet do pincel em 0. Enio 2026-07-11 escolheu "tornar significativo" (a versão só-moisture não afetava tinta seca) |
| 11 | **Slider de tempo de secagem** (Enio 2026-07-09) | ✅ LANDOU 2026-07-11 (`82924b7c`, `dry_rate_per_s` + slider Drying Time 2–60 s) | knob de calibração da janela de fusão do EDGE-1 |
| 12 | **Preview de umidade + secagem paulatina** (Enio 2026-07-09) | (a) ✅ LANDOU (overlay neutro + slider `d9dda7ec`/`7b198f51`); **umidade AO VIVO** (pour por-frame, resolve "só no mouse up" + os retângulos) + **secagem EDGES-TO-CENTER** (erosão da fronteira, `WET_ERODE_GAIN`) ✅ LANDOU 2026-07-11; **(c) secagem influenciando a MESCLA = DEFERIDO** | (a)+erosão feitos. (c) gatear o rewet-lift pela umidade local foi tentado e REVERTIDO: o mapa de umidade vem de `stroke_coverage` (deposito de pigmento) → água limpa/diluída subrepresenta a molhabilidade → o gating matava a reativação por água (quebra `clean_water_backrun`). Precisa de um sinal de "molhabilidade" separado da cobertura de pigmento (água vs pigmento). Fica pra quando o modelo de água for revisitado |
| 13 | **Retângulos + re-estilo da união ao mudar paper/grain** (Enio 2026-07-10) | ✅ LANDOU 2026-07-11 (`e3cdf551`, substrato por-dono) | vide bloco "Investigações 2026-07-10" abaixo — hipótese forte já mapeada (params NÃO capturados no `WetStrokeStyle`) |
| 14 | **INVESTIGAÇÃO: retângulos do Per-Layer Color no brush comum** (Enio 2026-07-10, foto) | bug aberto (handoff `HANDOFF_per_layer_color_perf_artifacts.md`) | aplicar o MÉTODO do BUGS #8 (bissecção + perfil + sondas) |
| 15 | **INVESTIGAÇÃO: perf do Per-Layer Color — otimizações da aquarela não aplicadas** (Enio 2026-07-10) | lento (handoff aberto) | checklist BUGS #7 + stamp-cache + ADR-0109 — vide bloco abaixo |
| 16 | **PESQUISA: traço de aspecto 3D — como Procreate/Rebelle/Painter fazem** (Enio 2026-07-10) | design | height-map + lighting pass como alternativa barata ao Per-Layer Color — vide bloco abaixo |
| 17 | **BUG: cores claras (amarelo/azul-claro) quase não aparecem** (Enio 2026-07-11, foto) | ✅ LANDOU 2026-07-11 (rota (a): slider **Opacity** = pigment body/hiding power) | vide bloco "#17" abaixo — resolução no fim do bloco |

**Blur do Wet Mix: exposto (`a7712f45`) e REVERTIDO no smoke** (Enio: "funcionava melhor quando
ele não era configurável") — o pickup do mixer fica FIXO em r×0,5 (cerca de Chesterton anotada no
`sample_surface`; não re-expor sem novo smoke).

> Perf/cor (outra dimensão, não-UI): waves W-A..W-D da auditoria em
> [`12_aquarela_auditoria_pos_f123_padrao_ouro.md`](12_aquarela_auditoria_pos_f123_padrao_ouro.md).


---

## Investigações enfileiradas 2026-07-10 (Enio — "coloque na fila para amanhã")

> Contexto colhido HOJE pra o agente de amanhã não recomeçar do zero. Método obrigatório: o do
> **BUGS_painter.md #8** (instrumentar o app real → bissectar → perfil 1D → sondas por-pixel →
> só então fix com RED→GREEN). Não iterar mecanismo sem reprodução fechada.

### #13 — Retângulos ao redor do brush + re-estilo da união inteira ao mudar paper/grain (aquarela)

**Sintoma (Enio):** traço 1 → mudar QUALQUER propriedade do brush watercolor (ex.: Amount do
grain, "Same as Paper", textura do papel) → traço 2 sobre a poça ÚMIDA ⇒ (a) retângulos nas
áreas ao redor do brush; (b) **o brush novo é aplicado a TODA a área úmida da união, mesmo onde
o traço 2 não tocou**.

**Hipótese forte (já verificada por leitura hoje):** o `WetStrokeStyle`
(`watercolor_field.rs:517-531`) captura fill/depth/edge_gain/wet/granulation(Amount)/warp/
pigment_mix/color/spread_thin/core_r/spread_px — mas **NÃO captura** `paper` (TextureSettings),
`paper_depth`, `granulation_use_paper` (Same as Paper) nem o Grain slot (`brush.texture`).
Esses são lidos **globalmente do brush VIVO** no `apply_watercolor`
(`watercolor_render.rs` ~L230-250: `paper_tex`/`paper_active`/`paper_img`/`paper_rot`/
`gran_tex`/`gran_own_map`/`gran_img`/`gran_rot`/`paper_depth`) — a união re-renderiza INTEIRA
com o substrato novo ⇒ o "aplica a tudo". Era um caveat DOCUMENTADO da rota a+b de 2026-07-09
("Fica global por composite: … fonte da textura de grain, paper"). O guard
`watercolor_session_brush_changes_do_not_touch_baked_washes` passa porque só varia params
capturados — **estender o guard aos não-capturados deve dar o RED imediato** (começar por aí).
Os retângulos = janelas incrementais do frame (frame dirty + pad) re-renderizando com globals
novos contra o entorno do bake velho — a classe do resíduo Δ2 do take 7, amplificada de
sub-visível pra gritante pela mudança de substrato. Atenção também ao cache
`wet_substrate` (memoização do paper_h por pixel do canvas): invalidação na troca de
paper/rotação — cache meio-velho/meio-novo = retângulos por si só.

**Direção de fix a avaliar:** capturar os 4 no `WetStrokeStyle` (resolver substrato POR DONO —
custo: paper_h por estilo, o cache `wet_substrate` vira por-estilo ou cai) × encerrar a sessão
na troca de substrato (barato, perde a fusão — último recurso, mesma régua do item (c) da
rota antiga).

### #14 — Retângulos do Per-Layer Color no brush COMUM (foto do Enio 2026-07-10)

**Sintoma:** os mesmos retângulos (bordas retas de janelas/discos no corpo do traço — vide foto
anexada na sessão de 2026-07-10) no brush normal (não-aquarela) com **Per-Layer Color** ativo.
Handoff aberto: `docs/HANDOFF_per_layer_color_perf_artifacts.md` ("listras retangulares";
teoria do coverage-map sujo REFUTADA lá — não re-investigar essa rota). BUGS #2 resolveu uma
família disso em 2026-06-29; a que sobrou nunca passou pelo método do #8.

**Plano:** (1) bissecção com `PH2D_PAINT_FULL_UPLOAD=1` (o toggle já existe no bridge —
separa upload-parcial-do-shell × conteúdo do canvas CPU); (2) se CPU: perfil 1D + sondas nos
buffers do per-layer (`per_layer_stroke`, restore/re-stamp do drag-preview — o restore-region
retangular do `stamp_drag_preview` é suspeito natural pra bordas retas); (3) fix só com
RED→GREEN nos params reais (dump via eprintln 1-linha, padrão [wet-diag]).

### #15 — Perf do Per-Layer Color: otimizações da aquarela NÃO aplicadas (checklist)

O handoff aberto diagnostica: re-stamp da forma INTEIRA por-move × N≤16 camadas + recomposite
O(bbox·N). Lições da aquarela que provavelmente NÃO foram portadas (verificar uma a uma —
BUGS #7 + memórias):

1. **Profile de build** (BUGS #7 raiz 1): `[profile.dev.package.*] opt-level=2` cobre os crates
   de paint-math — o per-layer roda nesses crates? Medir em `--release` ANTES de otimizar.
2. **Composite 2×/frame** (BUGS #7 raiz 2): o per-layer também compõe no Move flush E no tick?
3. **ADR-0109** (BUGS #7 raiz 3): os loops do re-stamp/recomposite per-layer são puros
   por-pixel (sem redução/RNG/estado compartilhado)? → rayon sancionado.
4. **Stamp cache** ([[project_painter_texture_brush_stamp_cache]]): o per-layer re-amostra
   falloff×textura por dab? O `StampMask` cacheado foi a cura do textured-brush.
5. **Incremental de verdade:** re-stampar só os dabs NOVOS (não a forma inteira por-move) +
   dirty-rect por camada (não bbox global × N).
6. **Medir a ESCALA primeiro** ([[feedback_measure_perf_symptom_scale]]): fixar o nº em ms
   por-frame por-knob antes de escolher o alvo.

### #16 — PESQUISA: traço de aspecto 3D sem N camadas (o objetivo do Per-Layer Color)

**Pergunta do Enio:** Procreate e outros produzem traços com aspecto 3D — como? Dá pra
implementar aqui mais barato que o Per-Layer Color (que existe PRA isso mas pesa)?

**Ponto de partida da pesquisa (verificar fontes antes de afirmar — zero claim sem
grep/WebFetch, [[feedback_no_industrial_claims_without_verification]]):** a técnica-padrão da
indústria pra "3D stroke" 2D é **height-map + lighting em 1 passe**, não N camadas: o traço
acumula um canal de ALTURA (h) junto do pigmento; um passe screen-space deriva a normal do
gradiente de h (Sobel/forward-diff) e aplica iluminação direcional/specular — Corel Painter
(Impasto), Rebelle 7 (impasto/metallic layers), ArtRage (oil thickness), Krita (experimentos
de bump do brush) seguem essa família; Procreate: investigar o que o motor faz em brushes
"dimensionais" (Blend modes + normal-like shading?) e os 3D Materials (outra coisa — pintura
EM modelo 3D). Custo esperado: O(pixels do dirty-rect) × 1 passe (gradiente + dot product) —
**ordens abaixo de O(bbox·N camadas)** do Per-Layer Color, e casa com nosso pipeline (o canal
h é um buffer u8/f16 irmão do coverage; o lighting é um kernel por-pixel puro → ADR-0109;
HR-5: normal por diferenças finitas = sem transcendental). Entregável: doc de design
comparando (a) Per-Layer Color otimizado (#15) × (b) height+lighting nativo, com medição de
ambos num cenário fixo, e recomendação. Decisão de manter/deprecar Per-Layer Color é do Enio.

### #17 — BUG: cores CLARAS quase não aparecem (amarelo, azul-claro) — Beer-Lambert sem body

**Sintoma (Enio 2026-07-11, foto):** MESMAS configs de brush, cores diferentes — vermelho forte,
**azul-claro e amarelo quase invisíveis**. Não é config; é o modelo óptico.

**Diagnóstico CONFIRMADO por leitura (`watercolor_render.rs:577-582`):** o depósito é Beer-Lambert
PURO por canal — `t = transmittance(pig[c], od)`, `lin = base·t + pigment·(1−t)`. Para um canal com
pigmento CLARO (perto de branco, `pig[c] ≈ 255`) a absorbância é ~0 → **`t ≈ 1` qualquer que seja a
densidade `od`** → o canal fica na cor do PAPEL. Amarelo `[255,255,50]` só absorve no AZUL → sobre
papel branco vira branco-levemente-menos-azul = amarelo pálido invisível. Vermelho `[255,50,50]`
absorve em G+B → aparece forte. Azul-claro idem (2 canais altos). **É a assinatura correta de
aquarela TRANSPARENTE (washes claros SÃO fracos), mas a magnitude está extrema** — aquarela real
mostra cores claras porque a tinta tem CORPO (opacidade/scattering), não só absorção.

**Rotas de fix (exige decisão — perceptual × físico):**
- **(a) Body/opacidade (barato, pragmático):** somar um termo de COBERTURA que alpha-blenda a cor do
  pigmento sobre a base independente da reflectância do canal — `alpha = fill·cover·k`. O `film_a`
  (opacidade perceptual, já existe, `:586`) hoje só dirige o mix; usá-lo (ou um irmão) como piso de
  depósito faz a cor clara registrar. Cuidado com o caminho byte-idêntico default (k=0 ⇒ atual).
- **(b) Densidade perceptual (médio):** escalar `od` inversamente à luminância do pigmento — cores
  claras ganham mais densidade pra compensar a baixa absorbância. Menos físico, calibração a olho.
- **(c) Kubelka–Munk / scattering (grande, o "certo"):** o pivot já previsto (CLAUDE.md, Mixbox/KM)
  — absorção + scattering K/S dá corpo às cores claras nativamente. É o caminho de estado-da-arte,
  mas é reescrita do modelo de pigmento.
- Recomendação: (a) como fix imediato (body term gated, byte-idêntico no default), (c) como norte.

**Método:** RED com uma cor clara (amarelo) medindo o depósito sobre papel branco (Δ do byte deve
ser >> hoje); fix; verificar que vermelho/escuros ficam byte-idênticos (o body só levanta o piso das
claras). Precisa do smoke pra calibrar `k`.

**✅ RESOLUÇÃO (2026-07-11) — rota (a), slider Opacity:**
- **Modelo:** `body_cov = opacity·(1 − e^{−k·od})` (`BODY_OD_GAIN k=6`, `watercolor_lut::Luts::body_cov`);
  o composite deita a cor do PRÓPRIO pigmento sobre o resultado Beer-Lambert por `body_cov` — value-
  independent, então o amarelo aparece na sua matiz em vez de `Tᵢ≈1`. **`opacity=0` ⇒ fold no-op, byte-
  idêntico** (prova: `x + (…)·0.0 = x`). Default `0.4` (só ativo com Watercolor ON → brush default intacto).
- **Alpha in-gamut (crítico):** o body derruba o canal mais-absorvido abaixo do piso `ground·(1−a)` do
  un-premultiply → `L` clampava (erro de 22 bytes no bake de camada transparente). Fix: `cov_a =
  max(1−t_min, a_body)` onde `a_body = max_c gamut_alpha(app_c, ground_c)` (o alpha mínimo que mantém
  `L∈[0,1]` nos dois lados). `a_body=0` quando body off → `cov_a` byte-idêntico.
- **Split LOC:** o campo novo transbordou `watercolor_render.rs`/`watercolor_field.rs` (ambos a 1 linha do
  teto 700 no main) → extraído `watercolor_lut.rs` (LUTs + helpers ópticos), re-exportado por glob.
- **Testes:** `watercolor_opacity_gives_light_pigments_body` (RED provado neutralizando o fold: amarelo dá
  `[255,253,197]` idêntico com/sem opacity → GREEN com o fold). `signed_rim`/`wet_lift` isolados com
  `opacity:0` (o body é o filme, ortogonal ao rim/lift). 508 testes verdes, clippy 0.
- **UI:** slider **Opacity** no card **Wash** (após Concentration), `PAINTER_WATERCOLOR_OPACITY`.
- **Aberto:** calibrar `k`/default no smoke; rota (c) Kubelka–Munk segue o norte (supersede o body term).
