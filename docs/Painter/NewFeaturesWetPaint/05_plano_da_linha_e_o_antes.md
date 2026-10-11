# 05 — Plano da linha «Wet Painter» e o «antes» medido (2026-10-07)

> Linha `line/PainterWatercolor` reaberta sobre `main` 2a925859d (decisão do dono 07/10: só o Wet Paint
> evolui). Este doc é o **tracker** da linha: as ondas do [04](04_PROMPT_linha_wet_painter.md) com o ponto
> de código onde cada uma entra, e a barra de cada gate tirada do VALE entre o nosso «antes» e o Rebelle
> ([03 §2](03_tracos_medidos_rebelle_x_ph2d.md)) — nunca de um número escolhido.

## §1 — O «antes» (W0), reproduzido

`PH2D_SONDA_SIZE=28|40` sobre 2a925859d, duas corridas: **sha256 iguais nos quatro meios** (a sonda é
determinística ⇒ a cerca «byte-idêntico» é possível). A tamanho 28 os números batem **exactamente** com o
03 §2 (mesma régua, `ferramentas/mede_traco.py`).

| Wet Paint hoje | larg. px | taper 5/50/95 | orla px | CV | período (% pot.) | textura % | borda/miolo | cob |
|---|---|---|---|---|---|---|---|---|
| size 28 recto | 28 | 28/29/15 | 6 | 0,036 | 19,4 (5 %) | 14,4 | 0,61 | 0,78 |
| size 28 rampa | 20 | 8/28/8 | 6 | 0,066 | 17,6 (6 %) | 13,0 | 0,65 | 0,79 |
| size 40 recto | **44** | 46/45/41 | **14** | 0,031 | 2,0 (**16 %**) | 12,0 | 0,52 | 0,77 |
| size 40 rápido | 44 | 36/45/24 | 14 | 0,033 | 2,0 (14 %) | 12,3 | 0,52 | 0,77 |
| size 40 rampa | 28 | **9/46/4** | 9 | 0,064 | 2,0 (8 %) | 13,8 | 0,55 | 0,75 |

Digital/Watercolor/Impasto a 40: 58 / 68 / 72 px de largura (o mesmo número, quatro larguras).

### §1.1 — A lei raio → largura do Wet Paint (sonda `tests/it/sonda_largura.rs`, 07/10)

⚠️ O «Size» do painel é o **RAIO** (`brush_settings.rs:45`, `set_brush_size_px` grava `radius_px`) e o
chip mostra o valor NORMALIZADO 0..1 (`paint_brush.rs:176` → `number_chip.rs:149 format_number`), não px;
o anel do cursor desenha `radius_px` (`dab_spec.rs:155 cursor_dab`). ⇒ a promessa do dono («o tamanho é a
largura que se vê») lê-se: **largura medida = 2 × raio = o diâmetro do anel**, e o volume *Size px* da W1
mostra esse diâmetro. A régua em Rust (`sonda_gestos::largura_mediana`) é a de `mede_traco.py` — conferida
nas mesmas imagens (wetpaint 28 / 140, digital 28, impasto 90: iguais).

| raio | 2 | 4 | 7,8 | 12,2 | 15,3 | 19,1 | 23,8 | 29,8 | 46,6 | 72,8 | 114 | 178 | 278 | 434 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| largura (t = 0 = 8 s = 20 s) | 1 | 3 | 6 | 10 | **10** | **18** | 26 | 30 | 52 | 79 | 127 | 208 | 327 | 524 |
| largura / (2·raio) | 0,25 | 0,38 | 0,38 | 0,41 | 0,33 | 0,47 | 0,55 | 0,50 | 0,56 | 0,54 | 0,56 | 0,59 | 0,59 | 0,60 |

- **Não muda com o tempo** (a água não alarga o traço — o mesmo que o Rebelle, 03 §3.6).
- **Degrau 12,5 → 16 px de raio** (largura 10 → 16, larguras 11–15 INALCANÇÁVEIS). O perfil transversal
  mostra porquê: o traço é um **miolo denso (~25 % do diâmetro) + uma franja rala de cerdas, ASSIMÉTRICA**
  (maior de um lado); a largura a 50 % salta quando a franja cruza metade do máximo. A assimetria é a da
  transferência por janela (Gauss-Seidel dependente do sentido — doc 44 §1, mantida pelo dono em 29/09:
  não se reabre).
- ⇒ **A largura de hoje é uma propriedade da FRANJA DAS CERDAS**, e as cerdas são o que a W2 substitui. Uma
  tabela invertida agora deixaria o buraco 11–15 e seria regravada pela W2. Pela §0.10 (mesma família), **a
  promessa da largura e o seu gate entram COM a W2**, medidos sobre o traço novo; a W1 guarda no preset a
  tabela raio → largura que a sonda gera (o motor muda ⇒ a tabela regera-se com o corpus de miniaturas).

### §1.2 — A tabela 02 §6.9 em unidades do motor (W1, a fábrica)

A tabela mistura escalas. **Pigment/dab, Paper Gate, Evaporation, Bristle Size/Count e Water** estão nas
unidades do motor, e o «Wet Round» bate AO NÚMERO com o `WetKnobs::DEFAULT` (os cinco ajustes do dono de
22/07). **Pickup, Edge Darkening e Gravity** estão em 0–1 e não cabem nas faixas do motor (Pickup 0,5 >
máx 0,2; Gravity 0,5 > máx 0,05; Edge 0,8 contra o padrão 50 de 0–200). Lei adoptada: **o Wet Round É o
padrão do dono, e essas três colunas são RAZÕES ao Wet Round** (Wash: Pickup ×0,6, Edge ×0,75, Gravity ×0,6;
Dry Brush ×1,6 / ×1,125 / ×0,4; Drip ×0,8 / ×1 / ×2). Tamanho e opacidade não estão na tabela: a fábrica
NÃO os fixa (fica o do artista) — nada inventado. Receitas em `tests/it/presets_de_fabrica.rs::RECEITAS`.

## §2 — Onde o código entra (o mapa que decide a cerca)

- O gesto vira carimbos no motor PARTILHADO (`stroke_lifecycle` → `dynamics`/`spec` → `stamp_route.rs:340
  stamp_dabs_inner`); o braço Wet Paint é **`wetpaint/dab_route.rs:24 stamp_dabs_wetpaint(dabs, brush)`**,
  que já recebe `radius_px`, `stroke_radius_px` e a pressão de cada carimbo. ⇒ **largura em px, piso/tecto
  de pressão, smoothing e inclinação em dois eixos entram AQUI** (re-mapeiam o carimbo), nunca em
  `dynamics.rs`/`spec.rs`.
- Fios: `ph2d-wet-paint/src/brush.rs:40 BristleKnobs` + `build_bristle_texture` (tile 128² fixo, semente
  `BRISTLE_SEED`) — o tile é ARRASTADO pela lane do traço inteiro: daí os «carris». `trail.rs` (598 l.) e
  `solver.rs` (553 l.) estão perto do tecto de 700 ⇒ os fios com física vão para ficheiros irmãos
  (`bristles_*.rs`).
- Knobs: `wetpaint_settings.rs:26 WetKnobs` + `ph2d-wet-paint/src/tuning.rs KNOB_DEFS`. Um preset do Wet
  Painter = `WetKnobs` + o sub-conjunto do `BrushSpec` que o braço lê + os campos novos das ondas.
- Luz do relevo: `impasto_light.rs:285 apply_impasto_light(&self, …)` é método do estado do Impasto, não
  função livre ⇒ W3 precisa de uma porta livre append-only num ficheiro irmão (o corpo não muda).
- Ponteiro: `ph2d-editor-core/src/tool.rs:230 CanvasPointer {pos, pressure, tilt:[f32;2], phase}` — tilt já
  tem os dois eixos; **não há barrel**. ⚠️ Pôr o barrel é mexer em contrato congelado (§6) ⇒ decisão do
  dono antes da W6 (ver §4).
- Restrições herdadas: determinismo ao bit (ADR-0134/0147), pior tick ≤ 2 ms (ADR-0134), Gauss-Seidel do
  arrasto fica (doc 44 §1), o vidro da aguada (`wetpaint/composite.rs`, `vidro_por_cima_tests.rs`) continua.

## §3 — As ondas, o que entregam e a barra (do vale)

| onda | entrega | gate (barra = vale «antes» ↔ Rebelle) |
|---|---|---|
| **cerca** | `the_other_media_are_untouched` (diff vs merge-base) + fixture `fixtures/sonda_28/{digital,watercolor,impasto}.ppm` com cabeçalho | sha256 iguais; mutação num dos três meios reprova |
| **W0** ✅ | as 12 fotos do Rebelle em `fixtures/rebelle/` com cabeçalho; `mede_todos.sh` sem caminho de scratchpad; a régua em Rust; a lei raio → largura medida e diagnosticada (§1.1). `PH2D_SONDA_PRESET` passa para a W1 (antes dela não há presets) | o gate de largura passa para a W2 (§1.1) |
| **W1** ✅ | crate `ph2d-brush-presets` (`*.ph2d-brush.png`), fábrica em `assets/brushes/WetPainter/`, navegador + os volumes VIVOS (Size px, Water; Opacity/Pressure/Length sem leitor ⇒ fora) | round-trip ao byte; presets contados; campo desconhecido não falha; miniatura = a do motor; o pincel guardado pinta o mesmo traço ao byte |
| **W1b** ✅ (08/10) | **as gotas pela lei do Rebelle** ([06](06_gotas_rebelle_x_ph2d.md)): a gravidade sai do pincel e vai para a TELA (o dial de inclinação, **desligado** de fábrica); Create Drips · Drip Size · Drip Length passam a ser do PAPEL (a obra); a água de fábrica dos presets desce para a faixa do Rebelle; o MECANISMO da gota no motor (ficheiro irmão do solver) | a bancada do 06 (`sonda_gotas` + `ferramentas/gota*`): tela direita ou inclinação a meio ⇒ **0**; inclinação máx. ⇒ gota **≥ 250 px** nos traços de água ≥ 0,7 e **0** nos de 0,4 e 0,2, **1–2 gotas por traço, ≤ 4 px de largura** (Rebelle 249–283 px, 1–3 px); Create Drips off ⇒ 0 e a orla de baixo escura; Size/Length 10/10 ⇒ ≥ 450 px em funil (Rebelle 463–465) · nenhum preset leva `gravity` · controlo: knobs das gotas no neutro e inclinação desligada ⇒ o traço de hoje ao byte |
| **W2** ◐ (a largura ✅ 11/10, §4b) | fios com pressão Perlin, largura ondulante, oscilação, Thin/Thick/Edges, Random Span, ruído HSLuv — e a **largura prometida** (§1.1: o raio do motor sai da tabela do preset, em `stamp_dabs_wetpaint`) | **largura = 2·raio ± 2 px** em raios fora da tabela, de 2 a 300 (hoje 0,25–0,60 × e o buraco 11–15) · a 40 px: **5 ≤ textura ≤ 10 %** (hoje 12–14 ✗; Rebelle óleo 5,9, express 9) · **CV ≤ 0,02** (hoje 0,031 ✗; dele 0,011) · **período < 15 %** (hoje 16 % ✗; dele 9 %) · controlo: knobs a 0 ⇒ PNG de hoje ao byte |
| **W3** | campo de altura no grid (fios Add/Remove), oleosidade = viscosidade, luz do Impasto por porta livre | óleo: borda/miolo **0,8–0,9** (hoje 0,52 ✗; dele 0,87) e textura na banda da W2; acrílico: largura A = B |
| **W4** | lei do papel do fluido (força × pressão, contraste, scratch, fundo que fura, `PaperBehaviour` × 5 papéis) | lápis 8B: textura **≥ 8 %**, borda/miolo **< 0,5** (dele 9,1 / 0,32; não temos lápis) |
| **W5** | Paint / Paint&Mix / Paint&Blend / Blend / Erase por pressão e limiar da queda; Loading+Length; Dirty Brush; Glaze | 1.º a bancada do Rebelle (03 §4, redondo a cruzar + 3.º traço fora); depois o nosso na mesma direcção |
| **W6** | piso/tecto/smoothing de pressão, Pulled String, tilt em 2 eixos, rotação {None, Follow, PenTilt[, PenRotation]} | rampa: pontas **≥ 60 %** do miolo (hoje 9/46/4 = 20 % / 9 % ✗; dele 35/42/33) · 8B a 55°: largura **≥ 1,2×** e textura menor (dele 37/30, 5,2 vs 9,1) |
| **W7** | caminho água 0 sem solver (lápis, carvão, pastel, giz, marcador, airbrush); Water como volume | a prancha dos 8 meios dentro do vale nas 7 métricas (barra com os dois lados) |
| **W8** | painel: browser + volumes + Stroke/Bristles/Paint/Paper + Tilt + Blow, i18n, tokens | censo sem controlo morto; cada knob com mutação que «morde» |

## §4 — Decisões do dono (07–08/10)

1. ✅ **A largura prometida vale para TODAS as mídias, secas incluídas** (dono: «SIM»). O Rebelle, a
   `size_px 40`, entrega 41–42 px nos meios de tinta mas **30 (lápis 8B), 17 (pastel), 18 (marcador), 26
   (airbrush)**: a textura e a orla comem a largura. Nós não: o número no painel é a largura medida ± 2 px.
   ⇒ a comparação da W7 com o Rebelle nos secos faz-se a largura MEDIDA igual, não ao número.
2. ✅ **Sem rotação da caneta (barrel)** (dono: «por enquanto ficamos sem rotação»). O `CanvasPointer`
   (contrato congelado) não muda; a W6 entrega None / FollowTrajectory / PenTilt. A mesa de teste do dono
   é uma **Wacom Intuos5 Touch** (pressão 2048 níveis + inclinação; sem barrel — a única caneta Wacom com
   barrel, a Art Pen, está descontinuada; Huion/XP-Pen não o têm). Reabrir = ADR próprio.
3. ✅ **As gotas seguem a lei do Rebelle, as quatro partes do 06 §5** (dono, 08/10: «vamos seguir esse
   caminho»): a gravidade sai do pincel para a TELA, que vem **direita de fábrica** (não escorre até se
   ligar a inclinação); as gotas são do PAPEL (criar sim/não, tamanho, comprimento); a água de fábrica dos
   pincéis desce para a do Rebelle; e o motor ganha o mecanismo da gota fina e longa. ⇒ onda **W1b**,
   ANTES da W2 (mesma família: a água; §0.10 — as quatro partes num bloco, uma bancada).

## §4b — Estado

> Partido em 2026-10-11 (`scripts/doc-split.py`, sha256; o doc tinha 136 KB). As entradas vivem em quatro
> pedaços por ordem cronológica, verbatim; entrada nova vai para o último.

| pedaço | datas | o que tem |
|---|---|---|
| [05a](05a_estado_W1_ate_smooth_edges.md) | 07–09/10 | a cerca, a W1 (onde vive cada coisa), o preview vivo, a W1b e as 5 rondas das gotas, a W1c e a auditoria da borda, o Smooth Edges |
| [05b](05b_estado_W1e_W1f_W1g.md) | 09–10/10 | o DESFEITO das gotas, a W1e (água, papel, Tilt), a W1f (achatamento), a W1g (opções desligadas de fábrica) |
| [05c](05c_estado_mistura_e_claridade.md) | 10/10 | a mistura amarelo sobre azul seco, os 4 pincéis de fábrica, assentar não muda o que se vê, a claridade é o PIGMENTO |
| [05d](05d_estado_mistura_II_e_presets.md) | 11/10 | a Mistura II (a representação da cor), os presets de todas as mídias (W2 e 1.º lote), a cor real, o relevo, o 2.º lote |

## §5 — A próxima janela (a W2)

> ⭐⭐ **11/10, depois do 17:** a W2 (a largura) e o 1.º lote (o Wet Liner) fecharam (§4b «Presets de todas as mídias»).
> A próxima janela é a do bloco que o dono escolher no [18 §5](18_mapa_das_midias.md) (A: a lei da tinta por preset — caneta,
> nanquim, marcador · B: o caminho sem água + o papel — lápis, carvão, pastéis, aerógrafo · C: os fios + a altura — óleo, acrílica).
> O texto abaixo é o plano da W2 de 08/10, histórico.
>
> ⭐ **11/10 — o prompt da próxima janela é o [17](17_PROMPT_janela_presets_todas_as_midias.md)** (pedido do
> dono: os presets de TODAS as mídias): o mapa das mídias medido contra o Rebelle, a W2 abaixo (a largura
> prometida) como base, e o 1.º lote de presets que o motor já pinta como a mídia. O texto abaixo é o plano
> da W2 de 08/10 e continua válido como a Fase B do 17.

A W1b fechou (§4b; o prompt dela: [07](07_PROMPT_janela_W1b_gotas.md)). Pendente do dono: o Wet Drip a
0,49 pinga ou não com a tela no máximo. A W2:

1. Fase 0 do [MODELO_TROCA_DE_AGENTE](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md), a
   cerca e `os_outros_meios_nao_mudam` verdes antes de tocar em produto.
2. W2 pelo §3: fios com física própria em ficheiros irmãos `bristles_*.rs` (trail/solver no tecto), e a
   **largura prometida** — o raio do motor sai da `width_table` do preset em `stamp_dabs_wetpaint`, com o
   gate «largura = 2·raio ± 2 px» de 2 a 300. Os campos novos dos fios entram no preset (`CAMPOS_*` ou no
   mapa `engine`) e na tabela.
3. A W2 muda a lei do motor ⇒ `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` fica vermelho: é o
   esperado; regere a fábrica (`PH2D_REGERA_PRESETS=1`) no mesmo commit da lei e diga-o no handoff.
4. Barra das medições da W2 no §3 (textura 5–10 %, CV ≤ 0,02, período < 15 %), numa rodada só (§0.10).

## §6 — Ritmo

Uma janela por onda (CLAUDE.md §2). Janela 1: cerca + W0. Janela 2: W1. Medições de cada onda numa
rodada só, intercaladas (§0.10); o oráculo só se reabre para as bancadas novas (W5 cruzamento, W4 papéis,
W6 tilt), todas numa sessão do Rebelle.
