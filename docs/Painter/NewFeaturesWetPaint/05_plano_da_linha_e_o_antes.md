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
| **W2** | fios com pressão Perlin, largura ondulante, oscilação, Thin/Thick/Edges, Random Span, ruído HSLuv — e a **largura prometida** (§1.1: o raio do motor sai da tabela do preset, em `stamp_dabs_wetpaint`) | **largura = 2·raio ± 2 px** em raios fora da tabela, de 2 a 300 (hoje 0,25–0,60 × e o buraco 11–15) · a 40 px: **5 ≤ textura ≤ 10 %** (hoje 12–14 ✗; Rebelle óleo 5,9, express 9) · **CV ≤ 0,02** (hoje 0,031 ✗; dele 0,011) · **período < 15 %** (hoje 16 % ✗; dele 9 %) · controlo: knobs a 0 ⇒ PNG de hoje ao byte |
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

- ✅ **Cerca** (07/10): `tests/it/os_outros_meios_nao_mudam.rs` (FNV dos pixels da sonda a 28; perfis
  `test` e `smoke` iguais; 0,7 s) + `ferramentas/cerca.sh` (ficheiros protegidos só ganham linhas).
  Mutações: `impasto_light.rs:48 DEPTH_UNIT_PX` 16→17 ✗ impasto · `watercolor_render.rs:40 SS0` 0,12→0,13 ✗
  watercolor · `ph2d-painter-brush/src/falloff.rs:115` Smooth 3,0→3,1 ✗ digital · controlo
  `ph2d-wet-paint/src/brush.rs:21 BRISTLE_SEED` +1 ⇒ cerca VERDE com o Wet Paint mudado (fnv
  `0x2c12…` → `0xaa36…`). `cerca.sh`: linha alterada ✗, só acrescentar ✓.
- A sonda passou para o binário único `tests/it/` (era `tests/*.rs` solto); os gestos vivem em
  `tests/it/sonda_gestos.rs`; os bytes dos quatro meios não mudaram com a mudança de sítio.
- ✅ **W1** (07/10, `5fcf7d826` + `c8824cd35`) — onde vive cada coisa:
  - **Formato** — crate `ph2d-brush-presets` (só formato + biblioteca, não conhece o motor): PNG à mão,
    parâmetros num `iTXt ph2d-brush` comprimido (⚠️ `iTXt` e não o `zTXt` do 02 §5: o `zTXt` é Latin-1 e os
    nomes são UTF-8), origem num `tEXt`, texturas NOSSAS num chunk privado `phTx` (PNG cinzento). zlib por
    `miniz_oxide` e **não** `flate2`: no lock o flate2 tem dois motores e a unificação de features escolhe-o
    pelo conjunto do build — o mesmo preset sairia com bytes diferentes num `-p` e num `--workspace`.
  - **Preset do Wet Painter** — `ph2d-tool-painter/src/tool/paint/wetpaint_preset.rs`: os knobs por CHAVE
    do `KNOB_DEFS` + `erase`, os 28 campos do `BrushSpec` que o traço lê (`CAMPOS_DO_PINCEL`), as máscaras da
    forma e do grão, o modo e os volumes; os outros 66 campos em `CAMPOS_FORA` com o porquê (o gate lê o
    `spec.rs`: campo novo sem destino reprova). ⭐ O gate `o_pincel_guardado_pinta_o_mesmo_traco` (TODOS os
    campos mexidos, pelo ficheiro, ao byte) foi visto vermelho por um defeito REAL: 3 153 px — `paper_depth`
    estava fora e o traço lê-o (é a «Paper Texture Strength» por pincel do Rebelle).
  - **Miniatura** = o S da bancada (rampa 0,15→1→0,15, 2 s de água) a 256², no relógio FIXO (renderização
    fora do ecrã; o relógio fixo deixou de ser só `test-support`). Igual no perfil `test` e no `smoke`.
  - **Fábrica** — `assets/brushes/WetPainter/Watercolor/Fluid/` (4, §1.2), gerador
    `presets_de_fabrica::gera_os_presets_de_fabrica` (`PH2D_REGERA_PRESETS=1`); gates contados · ao byte ·
    miniatura = a que o motor pinta hoje (o corpus de regressão: a W2 vai pô-lo vermelho e REGERÁ-LO de
    propósito) · duas miniaturas nunca iguais. Tabela raio → largura por preset (a do Wet Round = §1.1).
  - **Painel** — `ph2d-panel-painter-layers/src/paint_wetpaint_presets.rs`: no Wet Paint a fileira «Preset»
    dos meios dá lugar ao navegador (Favoritos · Recentes · grupos), Favorite, miniatura, Save as New ·
    Save as Default · Reset, os volumes VIVOS **Size px** (= diâmetro; o Size normalizado do topo sai no Wet
    Paint) e **Water** (saiu da secção para o topo) com valores rápidos, e a política dos volumes.
    ⛔ **Opacity fora**: a força não tem leitor no Wet Paint (dono 12/08; gate
    `a_opacidade_nao_tem_leitor_no_wet_paint` fica vermelho quando tiver ⇒ o volume entra). Pressure e
    Length entram com W6/W5.
  - **Disco** — `ph2d-app-painter/src/painter_bridge_presets.rs` (feature do painel): o tool regista o
    `PedidoPreset`, a ponte executa (biblioteca, `estado.json`, política, Save/Reset, avisos visíveis).
  - **Sonda** `PH2D_SONDA_PRESET=<nome>|todos` em `tests/it/sonda_tracos_por_meio.rs`.
  - **Mutações (13/13 sangram, controlos verdes antes):** `serde(flatten)`→`skip` no `extra` ✗ campo
    desconhecido · `NIVEL_ZLIB` 9→8 ✗ ao byte · utilizador não sobrepõe ✗ biblioteca · `paper_depth` não
    escrito ✗ 3 153 px · máscara da forma não instalada ✗ 4 550 px · Size sem ½ ✗ volumes + 2 seams ·
    `QUADROS_DEPOIS` 120→119 ✗ miniatura · guarda do clique do navegador ✗ seam · decoder da política
    trocado ✗ seam · popovers não pintados ✗ 3 seams · ponte sem «lembrar» ✗ política · Reset sem apagar ✗
    ponte · fábrica com 3 ✗ contagem.
  - **Portão batched** (07/10): cerca ✓ · fmt ✓ · clippy `-D warnings` ✓ (apanhou `chunks_exact` e um tipo
    complexo — curados) · nextest das 8 crates tocadas ✓ (apanhou a secção do «On preset change», que
    elidia a 220 px ⇒ «On switch», e «Watercolor»/«User» no HR-15 ⇒ excepção declarada: são PASTAS) ·
    `check --workspace --all-targets` ✓ · `architecture_*` ✓.
- ✅ **Preview VIVO do pincel** (08/10, pedido do dono: *«não se atualizam em tempo real… não consideram o
  paper»*, `b1cc63d2f`): a faixa do painel deixou de ser a miniatura do FICHEIRO e passou a ser o pincel
  de AGORA sobre a obra de AGORA.
  - **Cena** (`wetpaint_preset/preview.rs`): o `BrushSpec`, os knobs, as máscaras, o papel de cor do
    documento, o papel do pincel e a imagem dele, o relevo do substrato, inclinação, grade, mistura K–M, o
    papel visível. **Chave** = os mesmos ingredientes, imagens pela VERSÃO. **Saída** = a do ecrã
    (`take_preview_arc`: papel de cor, vidro, luz do relevo) — o papel aparece porque é a mesma porta. A
    miniatura dos ficheiros é o último quadro do mesmo pintor sobre a obra neutra (bytes da fábrica
    iguais, sem regerar).
  - **Custo medido** (`sonda_custo_da_miniatura`, `smoke`, load 2): 33–80 ms por S ⇒ **thread própria**
    (`ph2d-preview-pincel`, só o pedido mais recente conta) e os quadros **tocados** a 33 ms (a 80 ms a
    animação não se via). Lei: mudança isolada anima já; mudanças a menos de 180 ms (um arrasto) mostram o
    traço FINAL ao vivo e animam uma vez ao sossegar; nunca pisca para o vazio.
  - **Mutações 9/10 sangram**; a que sobreviveu (apagar `tool = Paint` na cena) era EQUIVALENTE — a cena
    não leva a ferramenta molhada — e a linha saiu; o gate do Smear fica contra quem a meter na cena.
- 🔎 **Gotas: a lei do Rebelle medida** (08/10, pedido do dono) — [06](06_gotas_rebelle_x_ph2d.md): no
  Rebelle a gravidade NÃO é do pincel (0 de 281 presets); o escorrimento é da TELA (Tilt, desligado por
  omissão) e a forma das gotas é do PAPEL (Create Drips/Size/Length); a água de fábrica da aquarela é 9–49.
  O nosso: Gravity por pincel, dial ligado por omissão, pente de dedos curtos em vez de 1–2 gotas longas; os
  knobs escondidos `extFingering`/`extRivulets` não fazem gotas. ✅ **Decidido** (§4.3): onda W1b.
- ✅ **W1b — as gotas pela lei do Rebelle** (08/10) — onde vive cada coisa:
  - **Motor** — `ph2d-wet-paint/src/drips.rs` (ficheiro irmão; o solver não cresceu): `DripPaper`
    {create, size, length} (1–10, 5), a **linha de contacto** `pin_downhill` (entre o `prepare_rows` e o
    `outflow_rows` do advect de Jacobi: célula ativa SECA perde a componente de descida do `uv` — a massa
    fica exacta porque saída e transporte leem o mesmo `uv`) e a **gota** `drip_pass` (no estágio final do
    passo, o da cadência: o teste dos seis pontos de retoma fica). `Sim::gotas: Option<DripPaper>` —
    `None` no boot (o port 1:1: fingerprints intactos), o tool empurra sempre `Some`. Sem gravidade as
    duas peças são INERTES ao bit. `Grid::drips` viaja no snapshot; `Grid::drip_dirty` junta-se ao sujo
    nas DUAS portas do passo (`step_simulation`, `step_stage` — `merge_drip_dirty`).
  - **A lei da gota (medida, constantes com a tabela no `drips.rs`)**: a orla de baixo é célula molhada com
    8 células SECAS a jusante (não um buraco da franja); a gota solta-se quando a tela passa o ângulo
    mínimo `SLIDE` (meio geométrico entre a meia e a máxima: anéis 6–8) **e** `gravidade × água da orla >
    HOLD = 0,039` (meio geométrico entre 0,7 = `0,1405` e 0,4 = `0,0109` na máxima). ⛔ Um só limiar de
    peso NÃO separa: na meia inclinação a água 1,0 junta MAIS na orla (`0,180`) que a 0,7 na máxima
    (`0,141`) — o Rebelle só pinga na máxima ⇒ o ângulo é uma condição própria. A gota bebe 25 % do
    excesso das orlas da SUA bacia (`walk_edge`: a orla seguida coluna a coluna, ±`15·Size` px, até 3
    colunas sem orla), desce a 1 px/passo na máxima, e o rasto é `270·(Length/5)^1,6·(bacia/HOLD)^0,1` px;
    cada célula do rasto assenta `0,045 ×` o pigmento da orla onde a gota nasceu (o fio é PÁLIDO e
    uniforme — Rebelle: 40–52 % do escuro do traço; com o pigmento da gota espalhado o nosso dava 82–95 %
    e o funil escurecia ao estreitar), a cabeça espera pela tinta da fileira enquanto a orla ainda dá, e
    quando deixa de dar o resto reparte-se igual pelo que falta (clareia até à ponta, nunca se corta); a
    largura é `0,08·Size²·(falta/total)²` (o funil). Uma bacia não volta a pingar no mesmo episódio molhado, e nada nasce DENTRO da bacia de uma
    gota (os buracos de uma poça a escoar: eram centenas de gotas natimortas).
  - **Tela** — o dial é a porta ÚNICA: nasce DESLIGADO (o tool e o Reset; o motor continua a nascer ligado
    e o reconcile empurra o desligado — `the_tool_default_reaches_the_engine_exactly` reescrito de
    propósito). A gravidade da tela é `Knob::Gravity = 0,01` no `WetKnobs::DEFAULT` (o anel 8 = ×2 = o
    Rebelle no máximo: a orla sem gotas dá baixo/cima **3,03 / 2,11** contra **3,07 / 2,28** dele; a
    `0,005` de antes dava 1,64 / 1,24). A fileira Gravity saiu da secção e da tabela de ajuste
    (`KNOBS_DA_OBRA` em `wetpaint_preset.rs`: não se captura nem se aplica; o painel de ajuste filtra-a).
  - **Papel** — `PainterTool::gotas` ao lado do `papel` (`papel.rs`: `gotas()`/`set_gotas()`), na secção
    Paper só no Wet Paint (`paint_watercolor_paper.rs::paint_drip_rows`; ids
    `PAINTER_WETPAINT_DRIPS_CREATE` em `PAINTER_WETPAINT_CLICKS`, `_DRIP_SIZE`/`_DRIP_LENGTH` em
    `PAINTER_WETPAINT_FIELDS`). Persistem com o documento: `PaintedDocument.gotas` (último campo, 3 bytes,
    `serde(remote)` — o motor não tem serde), degrau **187 → 188** com migração (`ProjectFileV187`,
    `PaintedDocumentSemGotas`: um 187 abre com as gotas de fábrica). O preview vivo segue-as
    (`CenaDoPreview`/`ChaveDoPreview`).
  - **Fábrica** — a água do Rebelle (Wet Round 0,29 · Wash 0,24 · Dry Brush 0,09 · Drip 0,49), sem
    `gravity`; regerada (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` ficou vermelho e voltou verde
    no mesmo commit). ⚠️ O Wet Drip a 0,49 fica no vale (0,4 não pinga, 0,7 pinga): decisão do dono pendente
    se deve pingar com a tela no máximo.
  - **A bancada como gate** (`tests/it/gotas_pela_lei_do_rebelle.rs`, 1000×640, 30 s, ~11 s as cinco):

    | caso | água 1,0 | 0,7 | 0,4 | 0,2 | Rebelle |
    |---|---|---|---|---|---|
    | tela direita | 0 | 0 | 0 | 0 | 0 — e o traço de antes AO BYTE (fnv `0xafa3…b7fa`) |
    | meia inclinação | 0 | 0 | 0 | 0 | 0 |
    | máxima | **296** px, 2 fios de 1 px, pálidos e uniformes | **260**, 2 de 1 px | 0 | 0 | 283 · 249 · 0 · 0 (1–3 px) |
    | Create Drips off | 0 · orla baixo/cima 3,0× | 0 · 2,1× | 0 | 0 | 0 · 3,1× · 2,3× (direita: ~1×) |
    | Size/Length 10/10 | **464** (funil 7 px → 1) | **464** | 0 | 0 | 463 · 465 |

  - **Custo a 4096²** (`ph2d-wet-paint/tests/it/drips.rs::measure_drip_pass_cost`, `smoke`, as duas
    variantes intercaladas no mesmo processo, Size/Length 10/10):

    | cena | estágio das gotas p50 / p99 / máx | advect com / sem a linha de contacto (p50) | load |
    |---|---|---|---|
    | 1.ª tentativa (varredura em CAIXA por gota), três poças | 6,1 / 13,5 / **15,5 ms** ✗ | — | 8,5 |
    | 2.ª (a orla seguida, `walk_edge`), três poças, 30–36 gotas | 0,24 / 0,78–0,87 / **0,99–1,62 ms** ✓ | 3,7 / 3,9 ms | 4,1–5,6 |
    | 2.ª, a tela INTEIRA molhada (16 M células, sem gotas) | 0,002 / 2,2 / 4,3 ms | 74,6 / 73,4 ms | 17,8 → 41 (INVÁLIDO) |
    | 2.ª final (tinta por célula), três poças, 30 gotas | 0,27 / 0,74 / **0,76 ms** ✓ | 3,55 / 3,77 ms | 6,6 → 11 |
    | 2.ª final, a tela INTEIRA molhada | 0,001 / 1,49 / **1,49 ms** ✓ | 70,4 / 68,9 ms | 6,6 → 11 |

    ✅ O pior tick das gotas fica ≤ 2 ms a 4096² na 2.ª tentativa (a varredura das orlas e a linha de
    contacto correm por linhas em paralelo; a bacia é a orla seguida, não uma caixa). Os 4,3 ms eram a
    carga de outras linhas (load 17–41). A linha de contacto custa ~1,5 % do advect.
  - **Mutações (17 sangram de 19; controlos verdes antes):** sem o ângulo mínimo ✗ meia · sem linha de contacto
    ✗ Create Drips off + motor · `HOLD` 0,005 ✗ água 0,4 · `FRONT_DRY` 1 ✗ funil 10/10 · bacia desligada ✗
    1–2 fios · sem `merge_drip_dirty` no `step_stage` ✗ · sem `drip_pass` ✗ bancada + determinismo · tool
    a nascer inclinado ✗ `the_tool_default…` · Reset inclinado ✗ · `knob_da_obra` falso ✗ · sem o
    reconcile das gotas ✗ · chave do preview sem gotas ✗ · sem o clique Create Drips ✗ seam · rows não
    pintadas ✗ seam · tabela de ajuste sem o filtro ✗ · lei `GotasDaTela` falsa ✗ censo · `TRAIL_TINT`
    0,2 ✗ fio pálido (0,93). Sobreviveram duas, e as duas mudaram o código: a forma ANTIGA do rasto
    (`pigmento·película/água`) passava em todos os gates de comprimento — a medição mostrou o fio a 82–95 %
    do traço contra 40–52 % do Rebelle, e o rasto passou a ser a tinta por célula com o gate do fio pálido
    (que a `TRAIL_TINT` 0,2 sangra); e o tecto «o fio não escurece», EQUIVALENTE (sem beber,
    `pigmento / células` é constante por construção) — saiu.
  - ✅ **Easy Drips** (pedido do dono 08/10: *«deixe as duas versões funcionando com um checkbox»*):
    `DripPaper::easy`, a caixa na secção Paper ao lado de Create Drips. Ligada, o papel segura menos
    (`HOLD_EASY = 0,00277`, o meio geométrico entre o traço de água 0,2 da bancada, `0,0014`, e o Wet
    Round, `0,0055`) e absorve menos o fio (a película na mesma razão): o Wet Round de fábrica pinga
    **255 px** na máxima (Rebelle 302), nada a meio, o Dry Brush nunca. ⚠️ O Wet Wash pinga também — ele
    junta MAIS na orla (`0,0062`) que o Round, e nenhum limiar separa os dois (no Rebelle o Wash Round não
    pinga). Gate `com_easy_drips_o_wet_round_de_fabrica_pinga_como_o_round_do_rebelle` (mutação «o hold
    ignora `easy`» ✗ 0 px). Persistido no mesmo degrau 188 (o campo entra no `GotasNoFio`).
  - **Censo dos controlos**: as três rows esmaecem com a tela abaixo do ângulo mínimo (`GotasDaTela`) e
    Size/Length com Create Drips off (`FormaDaGota`); a linha «o ângulo do carimbo não muda a tinta da
    água» saiu — com a tela direita ele MUDA (a água a correr apagava a diferença).
- ✅ **2.ª ronda das gotas, pelo smoke do dono** (08/10, duas fotos: *«ficou invertido: sem easy drips
  escorre… a tinta se assenta formando bases retas e degraus… o escorrimento fino parece artificial
  demais»*). A cena dele vive em `tests/it/sonda_gotas_do_dono.rs` (o S e o L, Wet Round a 140 px, água
  1,0). Os quatro defeitos e a cura de cada um:
  - **Degraus e bases planas** — a 1.ª linha de contacto cortava só a DESCIDA no papel seco, e a água
    entrava de LADO por baixo das encostas (26 433 px de tinta fora da mancha na cena do dono). Agora o
    papel seco POR FORA da mancha (seco a jusante, `FRONT_DRY`) não recebe água de lado nenhum; os buracos
    de DENTRO (a franja, com água por baixo) deixam-na passar. ⛔ Fechar também os de dentro prendia a
    água (orla 1,0×, nenhuma gota). Gate `com_a_tela_inclinada_a_mancha_guarda_a_forma_e_nao_faz_pente`:
    0 px fora da mancha (barra 300), mutação da 1.ª forma ✗ 24 848.
  - **O pente** (20 fios) — cada degrau de uma encosta era uma orla. A gota nasce só no PONTO BAIXO com
    água da bacia (`LOW_SLACK`, a caminhada com `EDGE_SEARCH` 12), e a bacia é ±`30·Size` px (Size 5 ⇒ 1
    fio num traço de 150, o Rebelle). A cena do dono: 4 fios (gate: 1–6; a 1.ª forma ✗ 8).
  - **Invertido** — no papel que segura menos a gota nascia cedo e esvaziava a orla rasa, e a tinta dela
    vinha da orla (16–22 de escuro contra 76–116). A tinta do fio é agora uma fração da OPACIDADE do
    traço (`TRAIL_ALPHA` 0,4, pela curva do motor: imune ao amontoado da orla e à franja) e SAI da orla
    e do traço acima dela fileira a fileira; a orla só dá água. Na cena do dono: 4 fios com e sem Easy, ~93
    de escuro nos dois.
  - **Artificial** — o fio ganhou ombros macios (0,35), um desvio lento (0,7 px, onda de 70 px, fase pelo
    hash da origem), um pescoço (3 células a afinar em 12 px) e a contra-gota (disco de raio 2).
  - **Recalibrado pela linha de contacto final** (a gota já não incha para fora da mancha): gravidade da
    tela `0,02` (orla 3,03 / 2,12 contra 3,07 / 2,28 do Rebelle), `SLIDE 0,0283` (meio entre a meia e a
    máxima), `HOLD 0,337` (meio entre 0,7 = 0,91 e 0,4 = 0,125), `HOLD_EASY 0,012` (meio entre o traço de
    0,2 = 0,006 e o Wet Round = 0,025). Bancada: 265 / 258 px na máxima, 464 / 464 no 10/10, 0 a meio, o
    Wet Round com Easy 258 px. Custo a 4096² (load 32–56, inflado): pior 1,13 ms (poças) / 1,51 ms (a tela
    inteira molhada).
- **Falta (só com ordem do dono):** o smoke dele e o fecho da linha (`/pd-linha-fechar`, handoff).
  ⚠️ Para o integrador: o degrau 187 → 188 tem migração (`ProjectFileV187`, braço `187 =>` em
  `project_load_read.rs`) — se o main passar de 187 antes, os dois movem-se à mão (o `schema-recount.py`
  só reconta a escada e a tripla).

## §5 — A próxima janela (a W2)

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
