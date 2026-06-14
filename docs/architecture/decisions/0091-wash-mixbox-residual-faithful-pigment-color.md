# ADR-0091 — Wash: residual Mixbox no K–M (cor escolhida fiel, mistura espectral)

- **Status:** ACEITO (Enio 2026-06-14), implementado.
- **Contexto:** [ADR-0086](0086-watercolor-minimal-core-wash.md)/[0087](0087-wash-integration-parallel-watercolor-mode.md)/[0089](0089-wash-dual-field-faithful-color-and-synchronous-undo.md). Enio (test strip): no modo **Pigment** as cores colapsavam — vermelho/laranja/amarelo viravam o MESMO laranja, azul-claro/escuro o MESMO azul. Pediu o estado-da-arte, não escolha pessoal.
- **Supersede:** o composite K–M do [ADR-0089 §2.2](0089-wash-dual-field-faithful-color-and-synchronous-undo.md) (matiz da razão de concentrações a uma magnitude fixa `K_REF`). Mantém o campo DUPLO (Linear/dye continua), o transporte, o undo (ADR-0090) e a mistura espectral.

## 1. Problema — a "K–M ingênua" descarta o VALOR

O 0089 normalizava **toda** cor para `Σc = K_REF` (unmix) e relia a razão a `K_REF` no composite, tirando a luminosidade só da cobertura. Consequência: a dimensão **VALOR/saturação** da cor escolhida some. Dois azuis que diferem em valor → mesmo azul; e a `K_REF = 3.0` (fundo no gamut) espremia vermelho/laranja/amarelo num laranja comum. Pesquisa (Sochorová & Jamriška, *Practical Pigment Mixing for Digital Painting*, **SIGGRAPH Asia 2021**, usado no **Rebelle**) identifica exatamente essa "K–M ingênua" como impraticável: o requisito é *"handle all RGB colors without clipping or distortion"* — nunca distorcer uma cor sozinha.

## 2. Decisão — o residual Mixbox

Cada cor RGB vira um **latente** = concentrações de pigmento **+ residual RGB aditivo**:
- **encode** `F(rgb) = (c, r)`, `c = unmix(rgb)` (NNLS, **sem** `K_REF`), `r = rgb − mix(c)`.
- **decode** `G = mix(c) + r`. Como `mix(c) + (rgb − mix(c)) = rgb`, **uma cor sozinha reproduz EXATA** (identidade por construção). Só a MISTURA wet-on-wet de cores diferentes (média ponderada dos latentes pela física do solver) mostra o pigmento espectral (azul+amarelo→verde).

O campo carrega o latente acumulado, mass-weighted: o composite lê `c̄ = pig/mass`, `r̄ = res/mass` (massa = `dye.w`) e mostra `mix(c̄) + r̄` sobre o backdrop, cobertura de `mass`. Validado no Metal: vermelho `[0.7,0.1,0.1]` → sRGB `(218,89,89)` (a própria cor); azul+amarelo → verde (`green-excess 53` vs `−6` Linear).

> Implementamos a **técnica** do paper, não a **lib** Mixbox (licença não-comercial). Pigmentos: os 4 do núcleo (CMY+K, ADR-0086) — o branco/valor vem da cobertura sobre o papel (aquarela), não de um pigmento branco.

## 3. Implementação

- **`km.rs`:** `unmix` (refino em espaço de cor, **sem** projeção a `K_REF`) + `pigment_residual` (encode) + `compose_km_mixbox` (decode). O caminho `K_REF` antigo fica como histórico até a limpeza.
- **Solver:** novo canal `res` (signed premul-RGB) — `res_a/res_b`, bindings step 8/9 + splat 5, `Dab.res`, `upload_res`/`read_res` (**os dois gêmeos**, ADR-0090), `clear`, copy-back. Para caber no **limite de 8 storage-buffers/stage**, o binding `paper` (ignorado pelo gate desde o B5) foi **removido** do step — granulação v1.1 re-adiciona. **Cap unificado por massa** (escala pig/dye/res pelo mesmo fator) p/ `c̄`/`r̄` consistentes no overlap pesado.
- **Shaders:** `res` transportado (`wash.wgsl`, mesmo `face()` gather, signed/sem clamp), depositado (`splat.wgsl`), decodificado (`composite.wgsl`: `mix(c̄)+r̄`).
- **Bridge:** deposita o residual por dab (`pigment_residual`); o `FieldSnap` do undo inclui `res` (os 3 campos dinâmicos pig+dye+water+**res**; `paper` é estático).

## 4. O seletor de cor (WYSIWYG)

A fidelidade é **da engine**: a cor pintada = a cor escolhida. Logo o seletor já é WYSIWYG — a amostra mostra o que sai no papel sem transformação extra (o decode `mix(c)+r` de uma cor sozinha é identidade). Não há colapso a corrigir no picker; a discrepância "picker vívido × pigmento turvo" do test strip some com a engine.

## 5. Consequências

- **+** Cor escolhida fiel nos dois modos; mistura espectral preservada; estado-da-arte (Mixbox/Rebelle), não ajuste pessoal.
- **+** Undo segue completo (res entra no snapshot, regra "restaure todo o estado dinâmico" do ADR-0090).
- **−** +1 canal de campo (memória ~+33% no campo GPU) e o `paper` saiu do step (granulação v1.1 re-adiciona — era inerte).
- **−** O caminho `K_REF` (`rgb_to_concentrations`/`compose_km_display`) vira código morto até a limpeza; gates de paridade GPU↔CPU do K–M passam a mirar `compose_km_mixbox`.
