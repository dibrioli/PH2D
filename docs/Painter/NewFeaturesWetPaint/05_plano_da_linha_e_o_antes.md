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
- 🔎 **3.ª ronda, pelo smoke do dono** (08/10, foto ampliada ~3×: *«borda inferior pixelada/quadriculada
  assim como escorrimento… isso não acontecia antes»*). O fio: perfil transversal contínuo com o centro ao
  décimo de célula (`SOFT_EDGE`), anti-serrilhado por construção — feito. A borda: é a barreira firme da
  linha de contacto, à célula; antes de hoje a água corria livre e a borda era difusa (e fazia os degraus).
  ⛔ **Recusa MEDIDA:** uma faixa de transição (fração do fluxo para o papel seco de fora, em quatro formas:
  12 % a todas; só a 1.ª fila vizinha de água de verdade; com tecto de película; a faixa a contar como
  seca) inunda a folha em riscos verticais — 106–127 mil px de tinta fora da mancha na cena do dono, contra
  0 com a barreira firme. ABERTO: suavizar a borda sem a abrir (p. ex. difusão lateral do pigmento
  SUSPENSO ao longo da orla, `ExtDiffusion` do solver, ou um dente da borda à meia-célula) — decisão do
  dono se vale a 3.ª tentativa.
- ✅ **4.ª ronda — o escorrimento natural de volta** (dono, 08/10: *«vc aniquilou aquele escorrimento
  bonito e natural de todos os traços com essa barreira??»*). **Create Drips desligado = o motor de antes
  da W1b, AO BYTE** (sem linha de contacto, sem gotas, sem a força da lei): gate
  `sem_create_drips_e_o_escorrimento_natural_de_antes_ao_byte` contra as impressões que a bancada mediu
  com o código de antes (`0x7cc9…6d0e` a meio, `0x0747…3d4e` no máximo). A força da tela da lei passou a
  ser `DRIP_LAW_GRAVITY = 4×` sobre o `Knob::Gravity` do motor (`0,005`, de volta ao boot) e SÓ com
  Create Drips ligado — a calibração da lei não mudou (0,005 × 4 = 0,02). O esmaecido: Create Drips só
  não age com a tela direita; Easy/Size/Length só agem com a lei ligada e a tela além do ângulo mínimo.
  A prova dos degraus corre agora com as gotas ligadas e conta só corridas largas (≥ 8 px) fora da
  mancha. Mutações: a barreira sem a caixa ✗ natural; a força da lei a 1× ✗ 4 da bancada.
  ⚠️ Pendente do dono: qual nasce ligado — hoje Create Drips nasce LIGADO (a lei do Rebelle).
- ✅ **5.ª ronda — a orla de baixo sem brechas** (dono, 08/10: *«vamos tentar corrigir a borda
  inferior»*). A causa, vista de perto: os buracos da franja junto à orla têm papel seco por baixo, e a
  linha de contacto tratava-os como FORA da mancha — a faixa escura ficava cortada por cunhas claras
  verticais. Cura: uma brecha estreita (água dos dois lados na transversal, a até `NOTCH_W` = 4 células)
  enche — a tensão superficial; debaixo de uma encosta (água de um lado só) ou de um arco (abertura
  larga) continua fechada, logo os degraus não voltam (o gate da forma continua a 0 px). Gate
  `a_orla_de_baixo_e_continua_sem_brechas` (colunas com um pixel claro no NÚCLEO da faixa): 4 / 12 agora,
  23 / 26 com `NOTCH_W = 0` (mutação ✗), barra 17. ⛔ Recusa medida: a difusão do pigmento
  (`ExtDiffusion` 0,05 e 0,15) não muda a orla (o pigmento da faixa já está ASSENTE; a difusão só mexe
  no suspenso). `TRAIL_FILM` 0,1 → 0,07: com as brechas a encher a orla dá menos água à gota e o funil
  10/10 da água 0,7 parava a 394 px (Rebelle 465). Custo a 4096² (load 46, inflado): estágio das gotas
  p99 1,34 ms, um pico de 2,1 ms.
- 🔎 **Auditoria da borda** (08/10, pedido do dono: *«nada feito. auditoria com agentes»*): 2 lentes +
  o oráculo do S a 1:1 (fixtures `s_*.png`). 5 achados de correção (L1) e 9 de costura (L2), com o
  mecanismo, a reprodução e o gate que faltava — no prompt da janela seguinte
  [08_PROMPT_janela_W1c_borda.md](08_PROMPT_janela_W1c_borda.md). Dono: os controlos das gotas FICAM no
  Paper; «cuidado para não piorar nossa engine» ⇒ as quatro cercas (incluindo o escorrimento natural ao
  byte) do princípio ao fim.
- ✅ **W1c — a borda e as gotas pela auditoria** (08/10, prompt [08](08_PROMPT_janela_W1c_borda.md); commits
  `4ffbc7432` física · `b6d0b21e0` costuras · `6abfef287` censo). A régua: `ferramentas/borda.py`, portada para
  `tests/it/borda_do_s.rs` (`mede_borda`, `fios_do_s`) — a descida do S (x 440–632), tudo em proporção ao corpo
  com a tela DIREITA do mesmo motor (Rebelle ~102, Wet Round ~213: pigmentam diferente, compara-se a forma).
  Reproduz o README das fixtures (orla do Rebelle 4 / 1,10 · 5 / 0,92 · 8 / 2,0).
  - **L1-1 — a causa da escada não é a força, é a POÇA.** Aos 3 s o nosso S já é o do Rebelle (corpo claro,
    rampa, linha fina); de 3 a 30 s o pigmento SUSPENSO espreme-se atrás da linha de contacto numa faixa de
    cor pura (o nosso pigmento já está a 0,92 da saturação com a tela direita) cujo topo serrilhado é a escada.
    Cura: a gota escoa a orla da sua bacia até `DripLaw::recede` e o canal CONTINUA a escoar depois de o fio
    acabar (`drain_after`) — a água sai, o pigmento fica na linha (o anel). A força fica 4×.

    | S, tela no máximo | degraus | faixa ≥ B0 | rampa | corpo | orla salto / rms | fios a y 330 |
    |---|---|---|---|---|---|---|
    | antes (W1b) | 84–88 % | 11–12 px | 14 | 0,43 | 5 / 0,9 | 2 (1–2 px) |
    | força ×2 · ×1,5 · ×1 | 39 · 23 · 3 % | 12 · 12 · 0 | 16 · 17 · 20 | 0,56 · 0,62 · **0,76** | — | — |
    | `recede` 0,003, sem canal | 31 % | 7 | 19 | 0,43 | 5 / 1,0 | — |
    | **W1c** (`recede` 0,001, `feed` 0,5, canal) | **8 %** | **3** | **23** | 0,44 | 7 / 1,1 | **12 (3 px)** |
    | Rebelle `s_inclinado` | 8 % | 0 | 28 | 0,25 | 4 / 1,1 | 11 (3–4 px) |

    ⚠️ ABERTO: o corpo não clareou para o do Rebelle (0,44 contra 0,25) — é a carga de pigmento do Wet Round e
    a textura das cerdas, fora desta onda.
  - **L1-3 — gotas de toda a orla.** Saiu a regra «só o ponto baixo» (`LOW_SLACK`). A bacia de uma gota é a orla
    que recolhe `basin_water` = 3000 (película × px²) atrás de si — um alcance FIXO não dá os dois oráculos (9 px
    por Size: 9 fios no S e 3 por traço recto; o Rebelle dá 10–11 e 1). Fio ≥ 3 px (`trail_min_px`), funil
    escuro (`neck_dark` 2: raiz ÷ fio 1,69, Rebelle ~2, antes 1,1). A reserva de tinta do fio é a MAIOR orla
    que a bacia teve (escoada, a orla seca aos bocados e o fio clareava: água 0,7 a 189 px < 250).
    S: 12 fios, 9 em encostas (Rebelle 6–8), ~280 px (228–430). Bancada: 259 / 258 px, 1 fio por traço, 10/10
    464 / 464 — todos os portões de antes verdes sem mexer na barra.
  - **L1-2 — `NOTCH_W` fica.** Orla recta: brechas 0 / 10 com ele, 110 / 110 sem (barra 17). Encosta do S, com a
    lei nova: inerte (37 contra 40 colunas com um claro na linha da orla; patamares 1 contra 2) — a
    regularização da escada que a auditoria mediu (2 → 9 patamares) era da escada, que já não existe.
  - **L1-4 — um predicado**, `Folha::por_fora` (edge.rs), para a linha de contacto e a orla de baixo
    (`is_front(e)` ⟺ a célula de baixo está presa). Teste-propriedade com a corrida de 8, o fundo da folha, o
    buraco da franja e a brecha — visto vermelho: «corrida de 8 secas: orla = true, presa = false». A linha de
    contacto ficou a mesma conta; fora da folha conta como seco nos dois (a água cai dela).
  - **L1-5 — a lei em px**: `DripLaw` em dados (`Sim::drip_law`; a calibração varia-a no mesmo processo por
    `PainterTool::set_wet_drip_law`, `#[doc(hidden)]`), convertida por `Sim::drip_px_per_cell` (a razão do tool,
    posta ao nascer a sessão). Grade 2: fios de 275 px contra 280 na grade 1 (antes 352 contra 296, e só não
    mais porque batia no fundo da tela).
  - **L2-2** — o Wet Drip de fábrica: água 0,49 → 0,7 (0,49 → 0 px; 0,6 → 260; 0,7 → 259; 1,0 → 278); a fábrica
    regerada — só o Wet Drip mudou de bytes (com a tela direita a lei não age, as miniaturas são iguais).
  - **L2-1, L2-3..L2-9** (costuras, agente próprio): set_source repõe as gotas e a selecção (censo que
    destrutura `StashedDoc` sem `..`); a dica de Create Drips diz o anel da lei (`wet_primeiro_anel_das_gotas`);
    Create Drips esmaece pela gravidade EFECTIVA; o Tilt tem a dica que aponta o Paper; o Reset do Paper repõe as
    gotas; o Ctrl+Z desfaz as gotas (`ModelSnapshot.gotas`, um passo por clique, arrasto coalescido); os campos
    Drip Size/Length arrastam inteiros (`set_number_integer`); nenhum id da Gravity é encaminhado. 21/21
    mutações sangram. ⚠️ Aberto: as gotas de uma sprite desmontada vivem só na sessão (persistir pedia degrau).
  - **O censo dos controlos** arma o Length no mínimo no anel 8: com a água da bacia a gota corre o Length
    inteiro e, a 128², 5 e 8 davam fios além do fundo da tela («esmaecida=false mas a tinta NÃO mudou»).
  - **Portões novos** (vistos VERMELHOS no `bb2bb1bf4`, numa worktree própria): `a_borda_do_s_e_a_do_rebelle`
    (87 % de degraus), `as_gotas_saem_de_toda_a_orla_do_s` (1 fio), `a_grade_2_da_os_mesmos_fios_em_px` (51 %
    de degraus), `o_wet_drip_de_fabrica_pinga_na_tela_no_maximo` (0 px). Reescrito DE PROPÓSITO:
    `com_a_tela_inclinada_a_mancha_guarda_a_forma_e_nao_faz_pente` — a barra 1–6 fios era a regra do ponto
    baixo, que contradiz o oráculo; agora 4–17 (hoje 14; o pente era 20).
  - ⛔ **Recusas MEDIDAS (W1c):** baixar a força da lei (tira os degraus e o corpo deixa de clarear: 0,76) ·
    assentamento do suspenso na água parada (`sediment` 0,02–0,2: degraus 0 % mas o corpo congela escuro,
    0,76–1,05) · `EdgeDarkening` 100 / 200 (degraus 88 %) · gotas de toda a orla SEM o canal (a poça volta,
    degraus 84 %) · funil `neck_dark` ≥ 3 (brechas 0 → 21 na orla recta) · alcance fixo da bacia (ver L1-3).
  - **Mutações (9 de 10 sangram, controlos verdes antes; agente numa worktree própria):** `drain_after` falso ✗
    27,5 % de degraus · `recede` = `hold` ✗ 86,5 % (e a grade 2, 46,6 %) · `basin_water` 1e9 ✗ 2 fios no S ·
    `trail_min_px` 1 ✗ fios de 1 px · `neck_dark` 0 ✗ funil 1,10 · `Cells::c` sem a razão ✗ grade 2 a 398 px
    contra 280 · a reserva = a última orla ✗ água 0,7 a 194 px · `is_front` sem a brecha ✗ propriedade ·
    `notch_w` 0 ✗ 110 brechas. Sobreviveu `FRONT_DRY` 8 → 7 nos DOIS lados do predicado (a consistência
    fica, e nenhuma cena mede 7 contra 8): constante da W1b, onde 8 → 1 sangrava — tolerância, não lei nova.
  - **Custo a 4096²** (`measure_drip_pass_cost`, `smoke`, Size/Length 10/10):

    | tentativa | três poças: estágio das gotas p50 / p99 / máx | tela molhada máx | load |
    |---|---|---|---|
    | W1b final | 0,27 / 0,74 / 0,76 ms | 1,49 | 6,6 → 11 |
    | W1c 1.ª (bacia por volume, soma o alcance inteiro) | 0,94 / 1,53 / **141 ms** ✗ | 1,93 | 6,5 → 14 |
    | W1c 2.ª (bacia nos dois lados até juntar a água) | 0,91–0,93 / 1,27–1,49 / **3,9–4,3 ms** ✗ | 1,6–3,0 | 3,6 → 13 (3 corridas) |

    ⛔ **Kill-criterion atingido** (> 2 ms depois da 2.ª tentativa): PARADO e reportado ao dono antes da 3.ª.
    O pico é raro (p99 ≤ 1,5 ms) e repete-se no mesmo tique das poças (544 gotas vivas, 3× a W1b). Hipótese
    para a 3.ª, NÃO medida: o `na_bacia` do `spawn` compara cada candidata com TODAS as gotas (O(orla × gotas))
    — um índice das gotas pela coordenada transversal tira-o.
  - Pranchas: [antes · depois · Rebelle, a 3×](prancha_borda_w1c.png) · [o S inteiro](prancha_s_w1c.png).
- 🔎 **Auditoria da W1c, 09/10** (smoke do dono sobre `f216fd97c`: *«borda pixelada e artificial. veja o
  escorrimento. Que app pro faria isso?»* — [capturas](fixtures/smoke_dono_w1c/), prompt
  [09](09_PROMPT_janela_auditoria_W1c.md)). Uma rodada: a régua NOVA da cobertura do pixel e da raiz
  (`ferramentas/borda_cobertura.py`, portada em `tests/it/auditoria_borda_w1c.rs` com a sonda
  `sonda_auditoria_w1c`, 9 cenas no mesmo processo, load 3–7), o Rebelle a 400 % (`s_zoom400.png`).
  - **A escada é o CONTEÚDO da tela, não o ecrã.** O dono olha a ~8,5× (cada degrau = 1 px da tela; a
    rampa de cada degrau mede 8 px de ecrã = o filtro Linear, `ImageFilterMode::Smooth` por omissão); o
    Rebelle a 400 % também amplia suave. A orla com a lei ligada é **0/1 em 72,5 %** das colunas da descida
    (meio-tom 9,8 %) — com Create Drips desligado 9,8 / 43,5, com a tela direita 16,6 / 67,4, na grade 2
    (o composite reconstrói por smoothstep) **0 / 68,9**; Rebelle 5,7–19,7 / 67–89. A linha de contacto
    (`pin_downhill`, fluxo 0 na célula `por_fora`) faz a orla binária e nada no motor nem no composite
    (cobertura = `pa`) lhe dá meio-tom.
  - **E o contraste multiplica-a:** o degrau orla → papel vale 1,04·B0 = 222 de escuro num pixel
    (Rebelle 0,44·B0 = 45: **4,9×**); a faixa de ≥ 88 % da cor pura tem **4 px** na bancada e no azul do dono a
    100 px (a captura do dono: 4–5 px; Rebelle 0). O portão da W1c (faixa ≥ B0 ≤ 6) aprova-a.
  - **A raiz:** a orla NÃO muda de altura no fio (degrau 0,7 px contra 1,3 longe dos fios; Rebelle 1,8 / 1,3;
    `drain_after` e `recede` não mudam nem uma raiz). O que o dono vê é o fio RECTANGULAR de topo plano
    (3 px desde 1 px abaixo da orla; Rebelle 5 → 4 → 3, um funil) colado por baixo de uma orla OBLÍQUA de
    cor pura: o lado de cima faz o canto. O «funil» da W1b é só tinta (`neck_dark`), não forma. O fio é
    vertical também no Rebelle (desvio 0).
  - **Os pontinhos** por baixo da orla existem com a tela DIREITA (72 por 100 colunas; Rebelle 2–57): vêm
    do carimbo do pincel (hipótese por leitura: o rabo do falloff × as cerdas × `pigmentPerDab` 600, sem
    piso), não das gotas — e mexer ali muda as duas cercas «ao byte».
  - Portões novos, VERMELHOS e `#[ignore]` (a cura tira-o): `a_orla_do_s_tem_meio_tom_como_a_do_rebelle`
    (72,5 > 25 %) · `o_fio_alarga_para_a_orla_como_o_funil_do_rebelle` (3,5 → 3 contra ≥ 1,5 px) · e a guarda
    `a_raiz_do_fio_nao_faz_degrau_na_orla` (verde hoje). Os achados inteiros e a cura:
    [10_PROMPT_janela_W1d_cura.md](10_PROMPT_janela_W1d_cura.md) · [prancha a 8×](prancha_auditoria_w1c.png).
  - **O «Smooth Edges» do Watercolor avaliado** (pedido do dono, 09/10, antes da cura: *«talvez ajude a
    nossa própria solução… smooth edges»*; e *«não sei se essa imagem do rebelle serve… não tem muito
    pigmento nas bordas»* — a régua do meio-tom é relativa ao pico LOCAL, não depende da carga; o contraste
    sim). A lei (`watercolor_aa.rs::aa_coverage`: 3×3 amostras a ±0,667 texel, `alpha = média ÷ máx` linear
    sobre o pixel acabado, nada de água nem pigmento movido) aplicada FORA do produto às telas da sonda, com a
    mancha 0/1 como campo de cobertura (`ferramentas/smooth_edges_no_wet.py`, emulação):

    | descida do S | binária % | meio-tom % | salto/B0 | último px ÷ puro | faixa pura (12 px acima) | pontinhos |
    |---|---|---|---|---|---|---|
    | azul do dono 100 px, hoje | 72,5 | 9,8 | 1,04 | 1,00 | 5 | 94 |
    | + Smooth Edges | **14,0** | **61,7** | **0,47** | 0,46 | 4 | 38 |
    | azul do dono 60 px, hoje | 57,0 | 23,8 | 0,97 | 1,00 | 3 | 18 |
    | + Smooth Edges | 31,6 | 41,5 | 0,44 | 0,67 | 3 | 4 |
    | tela direita, hoje · + SE | 16,6 · 1,6 | 67,4 · 82,4 | 0,57 · 0,23 | — | — | 73 · 27 |

    ⇒ cura o PIXEL da orla (A1) e apaga boa parte dos pontinhos (A4), mas NÃO a faixa de cor pura (A3) nem o
    rectângulo da raiz (B2) — [prancha a 8×](prancha_smooth_edges.png): a escada fica de cantos macios, a faixa
    fica. Aplicado em toda a mancha muda as cercas «ao byte» (tela direita: 4 204 px; Create Drips off: 6 532).
- ✅ **Smooth Edges no Wet Paint** (dono 09/10: *«opção B - vamos tentar»* — em todos os traços, ligado de
  fábrica). Onde vive: `wetpaint/smooth_edges.rs` (a lei é a do Watercolor, `aa_coverage`, chamada — não
  copiada; o campo é a mancha `pa ≥ SS0`, o pé 3×3 lido do plano de pigmento; a fração entra na COBERTURA do
  pigmento antes do `over`, como o Watercolor escala o alfa acrescentado — misturar a cor straight com a
  base escurecia a orla numa camada transparente, apanhado por `papel_nasce`/`vidro_por_cima`); o
  checkbox **Smooth Edges** abaixo de Paper (`PAINTER_WETPAINT_SMOOTH_EDGES`, `WetPaintState::smooth_edges`,
  o Reset volta a ligá-lo, o preview vivo segue-o); o rectângulo do composite cresce 1 px (o papel ao lado
  da orla). Nada de água nem pigmento se move: o motor fica ao byte.
  - **Cercas:** `com_a_tela_direita…` e `sem_create_drips…` correm com o Smooth Edges DESLIGADO e dão as
    MESMAS impressões de antes (o motor não mudou); gate novo
    `smooth_edges_so_toca_a_silhueta_e_desligado_e_o_over_de_antes` (só pixels de pé misto mudam; o clique
    desliga e volta ao `over` de antes, 0 bytes). `os_outros_meios_nao_mudam` e `cerca.sh` verdes.
  - **Medido** (sonda `sonda_auditoria_w1c`, S na tela no máximo; antes → agora):

    | | binária % | meio-tom % | salto/B0 | pontinhos /100 col. | faixa pura (12 px acima) |
    |---|---|---|---|---|---|
    | bancada | 72,5 → **10,9** | 9,8 → **72,0** | 1,04 → **0,44** | 83 → 35 | 4 → 4 |
    | azul do dono 100 px | 72,5 → 10,4 | 9,8 → 72,0 | 1,04 → 0,44 | 94 → 49 | 5 → 4 |
    | azul do dono 60 px | 57,0 → 22,3 | 23,8 → 58,5 | 0,97 → 0,41 | 18 → 4 | 3 → 3 |
    | Rebelle | 5,7–19,7 | 67–89 | 0,32–0,44 | 2–57 | 0 |

    `a_orla_do_s_tem_meio_tom_como_a_do_rebelle` saiu do `#[ignore]` (verde; vermelho antes). Continuam
    vermelhos e ignorados, para a W1d: a faixa de cor pura (4 px) e o funil. ⚠️ A régua `faixa_pura` contava
    a corrida que COMEÇA na orla e leu «0» com o Smooth Edges (o último pixel fica com meio-tom) — passou a
    maior corrida nos 12 px acima; a «faixa ≥ B0» da W1c (`mede_borda`) tem o MESMO defeito e lê agora 0
    em todas as cenas: não a use para a faixa.
  - **Réguas ajustadas, com o porquê no código:** o fio da bancada mede-se a MEIA ALTURA
    (`fios_a_meia_altura`; tudo > 25 contava a franja de ~⅓ de tom: 5 px); o piso do papel branco do Wet
    Paint é 1 nível (arredondamento do alfa da fração numa camada transparente: pior 1, 0 bytes > 1).
    Miniaturas de fábrica regeradas (`PH2D_REGERA_PRESETS=1`).
  - **Custo** (`medida_do_smooth_edges`, 4096², desligado/ligado intercalados, o mínimo de 7; load 6–13,
    inflado): 1.ª forma (máscara da região em série) tela inteira 10 → **106 ms** ✗; pé 3×3 local 9 → 39;
    máscara por linha 10 → 15; máscara da região em PARALELO + linhas sem borda saltadas **11,3 → 15,3 ms**
    (a recomposição inteira: o clique, a tela toda molhada) e **0,14 → 0,31 ms** no rectângulo de um traço.
  - [prancha a 8×: Rebelle · antes · agora](prancha_smooth_edges_wet.png).
- ⏪ **DESFEITO — as gotas, a linha de contacto e a água de fábrica delas** (dono, 09/10: *«parece que o
  traço perdeu qualidade desde o momento que vc introduziu os limites e o sistema de drips. Consegue
  desfazer isso tudo?»* → escolheu «voltar tudo menos o Smooth Edges»). O CÓDIGO (80 ficheiros fora de
  `docs/`) voltou ao de `69b4f1a75` (08/10, o preview vivo, antes da W1b): sem `drips*`, sem Create/Easy
  Drips nem Drip Size/Length no Paper, sem `DripLaw`/`DRIP_LAW_GRAVITY`, a Gravity e a água dos 4 pincéis de
  fábrica as de antes, o dial Tilt como era, o degrau do schema de novo **187** (sem o 188 nem a migração
  `ProjectFileV187` — ⚠️ um projecto gravado durante os testes das gotas, a 188, não abre nesta build). Ficou
  SÓ o **Smooth Edges** do Wet Paint, como OPÇÃO **desligada de fábrica** (o checkbox abaixo de Paper;
  desligado = o composite de antes ao byte, gate `smooth_edges_so_toca_a_silhueta_e_desligado_e_o_over_de_antes`).
  - Ficaram em `docs/` (sem efeito no programa): os docs 06–10, as fixtures do Rebelle (`rebelle_gotas/`,
    `s_zoom*.png`), as capturas do dono e as réguas Python (`borda.py`, `borda_cobertura.py`,
    `smooth_edges_no_wet.py`). Os testes Rust das gotas e da auditoria (`borda_do_s.rs`,
    `auditoria_borda_w1c.rs`, `gotas_pela_lei_do_rebelle.rs`, as sondas) saíram com o código; vivem no
    histórico (`8d3e8f68a`).
  - ⛔ **Para quem voltar às gotas:** o dono julgou que a BARREIRA (a linha de contacto que prende a água no
    papel seco) e o que veio com ela pioraram o TRAÇO, não só a borda. Antes de reconstruir, compare o traço
    normal (tela direita e inclinada, os pincéis de fábrica) contra este estado, com o dono, e leia a
    tabela de recusas da W1b/W1c. O prompt [10](10_PROMPT_janela_W1d_cura.md) está OBSOLETO.
  - Verificado depois de desfazer: `check --workspace --all-targets` ✓ · clippy `-D warnings` (6 crates) ✓ ·
    suítes de ph2d-tool-painter / panel-painter-layers / i18n / wet-paint / panel-wet-tuning ✓ · testes do
    projecto da shell (98) ✓ · `cerca.sh` ✓ (0 ficheiros protegidos no diff).
- 🔎 **Depois de desfazer (09/10): o escorrimento, o papel e o Tilt** — ordens do dono e medições no
  [11_PROMPT_janela_W1e_agua_papel_tilt.md](11_PROMPT_janela_W1e_agua_papel_tilt.md): o motor escorre em PENTE
  (7–10 dedos por traço) onde o Rebelle não escorre (a meio) ou faz UM fio (no máximo); o Wet Paint tem dois
  papéis (o do painel quando armado, o procedural do motor sem Kind — o dono: obedecer ao painel); o pincel
  seco não pega nos picos com o papel do motor (picos ÷ vales 1,06; com o do painel 1,81); o preset não guarda
  o Tilt; a MISTURA: azul sobre vermelho dá azul (molhado ≈ seco), a água só corre depois do Up. ✅ Feito: o
  menu de presets lista cada pincel uma vez (`3d8e40be1`). Sondas:
  `tests/it/sonda_tilt_do_preset.rs`, `wetpaint/sonda_papel_seco.rs`, `wetpaint/sonda_mistura.rs`, régua `ferramentas/tilt_rebelle.py`.
- ✅/⚠️ **W1e (09/10, prompt [11](11_PROMPT_janela_W1e_agua_papel_tilt.md))** — `2e92420be` (F6) · `0b64130fa`
  (F2+F3) · `c91dade9c` (F1+F4+F5). Rebelle 8.3.4 pela porta WebSocket (`ferramentas/mistura_papel*.{py,sh}`),
  réguas `ferramentas/velatura.py` (γ em luz linear) e `ferramentas/fios.py`. Pranchas 1:1 e 8×:
  [escorrimento](prancha_w1e_escorrimento.png) · [8×](prancha_w1e_escorrimento_8x.png) ·
  [mistura](prancha_w1e_mistura.png) · [8×](prancha_w1e_mistura_8x.png) · [pincel seco](prancha_w1e_papel_seco.png).

  | | antes | depois | Rebelle |
  |---|---|---|---|
  | **F6** seco: γ durante · depois (0 = por cima, 1 = multiplicar) | 0,17 · 0,06 | **0,47 · 0,58** | 0,42–0,49 · 0,44–0,51 |
  | F6 molhado: γ G/B durante · sangria no vermelho a 15 px | 0,48/0,31 · +7 | 0,47/0,42 · **+19** | 0,67/0,49 · +12…+21 |
  | F6 seco: sangria · pincel sujo (rasto − azul) | +6 · (5, 0, −5) | **−3…0** · (6, −1, −7) | +1…+2 · ≈ 0 |
  | F6 a água corre com o traço (quadro a pintar, 4096², poça pesada, pior) | 1,6 ms, água parada | **11,7 ms** (sem pintar: 11,1) | corre |
  | **F2** fontes de papel do Wet Paint | 2 (motor sem Kind · painel) | **1** (sem Kind = o Cold de fábrica do painel) | 1 (o papel) |
  | F2 controlos de papel fora da secção Paper | 6 (grupo PAPER do Tuning + «Paper») | **0** («Paint Texture» na secção Paper) | — |
  | **F3** Wet Dry Brush sem Kind: picos ÷ vales · r(escuro, altura) | 1,06 · +0,05 | **1,34 · +0,28** | sem barra (ver abaixo) |
  | F3 molhados (Round · Wash · Drip) picos ÷ vales | 0,95 · 0,98 · 0,94 | 0,95 · 0,98 · 0,94 | — |
  | **F1** Wet Round, bancada: tela direita · meia · máxima (fios ≥ 20 px) | 0 · 0 · 1 | **0 (ao bit) · 0 · 0** | 0 · 0 · 1 fio fino de 250–300 px |
  | F1 orla de baixo ÷ de cima a meia (água 1) | 0,76 | **1,30** | 1,6 |
  | F1 pincéis de fábrica, meia (Round · Wash · Dry · Drip) | 0 · 0 · 0 · 4 | 0 · **4** · 0 · **0** | — |
  | F1 pincéis de fábrica, máxima | 1 · 4 · 0 · 6 | **0 · 1–6 · 0 · 3** (dentinhos < 20 px no Round) | Round 1 fio |
  | **F5** gravidade dos pincéis (Round · Wash · Dry · Drip) | 0,005 · 0,003 · 0,002 · 0,01 | **0,005 em todos** | não existe por pincel |
  | **F4** o preset guarda o Tilt | não | **sim** (preset antigo não mexe; Round/Wash/Dry direitos, Drip no máximo) | a tela nasce direita |

  - **F6:** `colorops::velatura` — a junção «por cima» ganha o termo de Beer–Lambert na fração γ = 0,47
    (`VELATURA_MEDIDA`), × a distância entre as cores (a MESMA tinta não se vela: sem o factor um traço de
    uma cor escurecia 202 → 156). Em todas as junções (pouso sobre suspenso, assentamento, Dry canvas,
    re-wet, render). `Sim::sim_while_painting`: a água corre com o traço; o lote que chega com o motor na
    simulação espera numa fila e pousa no tick (sem ela o quadro esperava o estágio: 19 ms). Duas fugas
    que a água viva expôs, curadas: o push do Impasto ligava o aquecimento do motor de traço no Wet Paint;
    um carimbo redondo rodava o último bit do raio. ⛔ γ = 1 escurece demais (0,80–0,91) e mata a sangria.
  - **F2/F3:** sem Kind, o motor é semeado pelo `papel_de_fabrica()` do painel (o Cold que o Relief arma e
    acende) pela mesma lei do Kind armado; o papel procedural do motor deixou de ser fonte no produto e o
    grupo PAPER do Tuning saiu inteiro; o «Paper» do Wet Paint é a **Paint Texture** da secção Paper (o nome
    do Rebelle, `paint_texture`); o Tooth aparece sem Kind no Wet Paint. Relief/Roughness já iluminavam o Wet
    Paint (a luz é da tela; o censo dos controlos o prova). Custo: semear o papel a 4096² 11 ms (re-assar o
    do motor: 1 ms), uma vez por sessão. ⛔ Recusa medida: o ruído de recurso da Aquarela como papel sem
    Kind (o Tooth deixava de ser monótono: halo 81 / 60 / 91).
  - ⚠️ **F3 sem barra do Rebelle:** pela porta, sob o Wine, o papel não se desenha (branco com desvio 0), o
    tamanho do Flat Dry não pega depois de um `SET_ENGINE_PARAMS` (saiu um fio de 12 px numa faixa de 30 —
    os «73 % de papel branco» eram faixa vazia) e o registo da altura do papel é ambíguo (|r| 0,20 em duas
    posições de sinais opostos). A barra ficou a lei física (seco > 1 nos picos, molhados < 1). ⛔ Não
    reabra por esta porta: `SET_PAPER` derruba o 8.3.4 («bad optional access»).
  - **F1 (PARCIAL):** `solver::Gota` — numa célula SECA (a advecção é por recolha: é ela que puxa), a
    componente que desce leva `min(1, 2·hill(h/h_c))`, `hill = q⁸/(1+q⁸)`, `h` = a conta logo acima,
    `h_c = 0,2/|g|` (`ALTURA_DA_GOTA`); nos construtores travado e suavizado. Tira o pente do Round e do Drip
    a meia inclinação e escurece a orla de baixo como o Rebelle; uma poça de 4 passadas continua a correr.
    ⛔ **Recusas medidas** (bancada Round, `sonda_gota`): a lei só no FREIO (morde 1 quadro em 4: nada muda) ·
    pela altura da própria célula seca (zero — uma barreira, prende tudo) · o fluxo INTEIRO da célula seca
    (8–9 dedos e riscos brancos no topo do traço) · `q⁴` (a conta do Wash, 17,7 contra a crítica 40, fugia
    7 %) · mais nivelamento (2 dedos mas largos, e muda a tela direita) · `extFingering` está a 0 nos 4
    pincéis (não é a origem do pente).
  - **Aberto (F1, a próxima onda):** o fio ÚNICO, fino e longo do Rebelle no máximo não sai de nenhuma lei
    local — 3 topologias medidas, parei pela regra dos dois tropeços. Precisa de um mecanismo NÃO local (a
    conta de baixo a equalizar a pressão ao longo da orla, para o primeiro fio drenar os outros). E o Wet
    Wash e o Wet Drip (água de fábrica 1,0; o Rebelle usa ~0,24) fazem contas de 40–60 de altura e ainda
    abrem 3–6 dedos no máximo — o Wash também a meia, desde que a F5 lhe subiu a gravidade de 0,003 para
    0,005. A água de fábrica foi revertida com a W1b; decidir com o dono.
- 🔎✅ **W1f — auditoria do ACHATAMENTO (10/10, prompt [12](12_PROMPT_janela_auditoria_achatamento.md))**
  (smoke do dono sobre `a9ce1a4c6`: *«áreas retas, parece que retângulos virtuais influenciam»* —
  [captura](fixtures/smoke_dono_w1e/achatamento_s_vermelho.png)). Cena: o S do oráculo a 1:1 (Wet Round
  100 px, água 1, 30 s; `wetpaint/sonda_achatamento.rs`), tela direita · meia · máxima · diagonal (raio 2).
  Régua NOVA `ferramentas/recto.py`, portada em `wetpaint/achatamento_tests.rs` (a mesma conta): a orla de
  baixo plana mais longa (±0 px) e a fracção de fios LARGOS (≥ 8 colunas) com a ponta cortada a direito
  (≥ 70 % das colunas na mesma última linha ±1). Calibrada no Rebelle (5 telas `s_*.png`): plano 19–22,
  pontas quadradas 0 (os fios dele têm 3–7 colunas); a captura do dono: plano **62** (e 33, 27, 24 nos
  três sítios marcados).
  - **Causa (medida célula a célula, `sonda_ponta`): a lei da gota da W1e.** Na ponta de um fio a água
    está no MEIO (filme 10–19 no centro, 0–3 nas bordas), mas todas as colunas descem juntas, uma linha de
    cada vez: (1) a altura crítica (`h_c` 40–80) quase nunca é atingida — a frente avança porque a célula
    «seca» destrava com `film > 1e-3` (um degrau binário); (2) a lei só trava a componente que DESCE, a
    de lado fica livre — a linha nova enche-se de lado até à largura do fio antes de a seguinte abrir.
    Resultado: fundo plano, cantos a 90°, paredes verticais. Aos 8–15 s as pontas ainda são redondas; o
    corte aparece no rastejar final (a 30 s).

    | H | teste | resultado | veredicto |
    |---|---|---|---|
    | **H1** a lei da gota | S com `drip_head` 0 × 0,2 | com a lei: meia plano 33 (com a água no traço) / 20, pontas quadradas 3 de 3; máxima 20 de 22. Sem ela: 13 · 0; 10 · 1 de 24 | **a causa** |
    | H1 «ĝ arredondado» | fonte bilinear ao longo do fluxo · máximo dos 3 de cima · máximo de 4 na coluna | meia plano 33 / 33 / 20, pontas iguais | ⛔ não é o arredondamento |
    | **H2** recorte por eixo | `maxVelocity` 3 (e + `levelClamp` 1) | 33 (76 com o nivelamento) — pontas quadradas iguais | ⛔ morta |
    | **H3** rectângulos reais | `spans_enabled = false`; ecrã × recomposição inteira; `rf` | byte-idêntico; 0 px de diferença em 60 cenas; `rf = 1` de fábrica | ⛔ morta |
    | **H4** o fio que parou | a ponta ao longo do tempo (`PH2D_SONDA_X`) | redonda a correr, cortada no rastejar (o mecanismo acima) | ✅ sintoma da H1 |

  - **⛔ Recusas MEDIDAS (curas da lei que mantêm a trava na entrada do seco — ~40 variantes, 6 rodadas):**
    altura crítica modulada pelo papel (`h_c·(1 ± 0,5…1)`: plano 33 → 23, pontas quadradas iguais) ·
    limiar de «seca» relativo (`0,05–0,25·h_c`: a orla vira ESCADA, cantos 49–50) · conta presa abaixo de
    `0,6·h_c` (plano 36) · o fluxo INTEIRO da célula seca com `h_c/cos θ` (escadas; em todas as direcções
    os riscos brancos e 9–10 fios no máximo da bancada) · transição seco→molhado contínua (escadas e fios
    na MEIA) · a lei na GRAVIDADE em vez da entrada (piso 0: nada se move, igual à tela direita mesmo no
    máximo; piso 0,25–0,5: a orla de baixo deixa de escurecer, 0,76–0,93). ⇒ uma porta célula a célula na
    entrada do papel seco imprime a grelha; não há variante dela sem rectângulos ou escadas.
  - **Decisão (pelo número): a lei sai do produto** (`drip_head = 0` no nascimento da sessão; o código
    fica no motor como instrumento das sondas, o porquê no doc de `ALTURA_DA_GOTA`). Portão novo
    `o_escorrimento_do_s_nao_se_corta_em_rectangulos` (meia e máxima; controlo DENTRO: com a lei reprova;
    mutação «a lei de volta no produto» ⇒ VERMELHO). `a_meia_inclinacao_nao_escorre_e_a_orla_de_baixo_escurece`
    passou a `a_meia_inclinacao_nao_escorre` (o produto, ≤ 20 px; controlo: G 0,01 no máximo escorre).

    | S / bancada | antes (W1e) | depois (W1f) | Rebelle |
    |---|---|---|---|
    | S meia: orla plana · pontas quadradas | 33 · 2 de 3 (com a água no traço) | **13 · 0 de 3** | 19–22 · 0 |
    | S máxima: pontas quadradas | 19 de 21 | **1 de 24** (redondas) | 0 (fios de 3–7 px) |
    | bancada Round: direita · meia · máxima (fios ≥ 20 px) | 0 · 0 · 0 | **0 · 0 · 1** (21 px) | 0 · 0 · 1 (250–300 px) |
    | ⚠️ orla de baixo ÷ de cima a meia (água 1) | 1,30 | **0,77** | 1,6 |
    | pincéis de fábrica, meia (Round · Wash · Dry · Drip), fios ≥ 20 px | 0 · 4 · 0 · 0 | 0 · 5 · 0 · 0 | — |
    | pincéis de fábrica, máxima | 0 · 1–6 · 0 · 3 | 1 · 5 · 0 · 3 | Round 1 |

    ⚠️ **O que se perdeu** (aberto, a decidir com o dono): a orla de baixo a meia já não escurece (a lei
    prendia a água na orla; sem ela rasteja, como no port). E o S no MÁXIMO continua com ~20 fios
    grossos (antes e depois; o Rebelle 10–11 finos e longos) — o «fio único» da W1e continua aberto.
  - **A água só corre depois do Up (dono, 10/10: *«prefiro tudo em tempo real, mas precisamos pintar
    imagens de até 4k»*).** Medido a 4096² (poça pesada, `measure_a_painting_frame_with_the_water_running`,
    7 blocos intercalados, load 25–40): o quadro a pintar com a água parada p50 1,2–1,6 ms · pior 2,1–4,0;
    com ela a correr no traço p50 4,1–4,7 · pior **14,1–44,6 ms** ⇒ `sim_while_painting` desligado no
    produto. A mistura continua a ver-se a pintar (o pouso do dab vela), mas o render do filme MOLHADO
    sobre o seco velava demais sem a água a assentar no traço (seco durante γ 0,76): ganhou constante
    própria, `VELATURA_DO_FILME` = 0,20 (`Sim::glaze_filme`; varrida em `sonda_velatura_do_filme`):
    seco γ durante · depois 0,76 · 0,62 → **0,55 · 0,43** (Rebelle 0,42–0,49 · 0,44–0,51), molhado
    sangria +25 (Rebelle +12…+21). `azul_sobre_vermelho…` verde.
  - Pranchas: [1:1](prancha_w1f_achatamento.png) · [8× nas pontas](prancha_w1f_achatamento_8x.png)
    (antes · depois · Rebelle · a captura do dono).
- ✅ **W1g — OPÇÕES NOVAS, desligadas de fábrica (10/10, prompt [13](13_PROMPT_janela_W1g_opcoes.md))**
  (dono 10/10: *«se vc sabe como melhorar sem danificar e sem provocar limitações e sim criando novas
  opções»*). Duas opções entregues na secção Wet Paint, uma recusada pelo número.
  - **A cerca: desligadas = o traço de hoje AO BYTE.** `opcoes_tests::com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte`
    fixa o FNV do S e da bancada nas três inclinações, gravados em `fcc685e43` ANTES do primeiro
    commit (verde antes e depois). Miniaturas de fábrica (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`),
    `os_outros_meios_nao_mudam`, `cerca.sh`, a suíte de `ph2d-wet-paint` (impressões do port) e o censo
    dos controlos: verdes antes e depois.
  - **O1 «Live Water»** (caixa, `PAINTER_WETPAINT_LIVE_WATER`): a água corre com o traço aberto
    (`Sim::sim_while_painting`, levado ao motor vivo pelo `reconcile_facts`). Dica no painel com o custo
    MEDIDO na W1f (4096²: pior quadro 2–4 ms → 14–45 ms). ⚠️ **A velatura do filme depende do modo**:
    com a água ao vivo o `VELATURA_DO_FILME` 0,20 sangrava o azul `+3,2` sobre o vermelho seco (barra
    ≤ 3) ⇒ `VELATURA_DO_FILME_VIVA` = 0,35 (tabela no doc da constante, `colorops.rs`), trocada pelo
    reconcile com a opção.

    | mistura (seco γ durante · depois · sangria; molhado sangria) | água parada (fábrica) | Live Water | Rebelle |
    |---|---|---|---|
    | velatura 0,20 | 0,55 · 0,43 · +3 · +26 | 0,42 · 0,41 · **+3,2 ✗** · +28 | 0,42–0,49 · 0,44–0,51 · +1…+2 · +12…+21 |
    | velatura 0,35 | 0,67 · 0,54 · +2 · +25 | **0,57 · 0,51 · +2,9** · +28 | idem |

    Portões: `com_a_live_water_a_agua_corre_com_o_traco` (pelo autorado: fábrica 0 passos com o traço
    aberto, ligada > 0) · `com_a_live_water_a_mistura_continua_na_barra_do_rebelle` (controlo DENTRO: a
    velatura da água parada com a água ao vivo reprova). ⚠️ Aberto: a melhoria do custo (simular só a
    janela perto do pincel durante o traço) é um item à parte, não medido.
  - **O2 «Heavy Edge»** (linha `0..1`, `PAINTER_WETPAINT_HEAVY_EDGE`, logo abaixo do disco do Tilt;
    esmaece com a dica «Acts when Tilt is on» — `Dependente::BordaSemTilt`): **o pigmento desce mais que
    a água** — `solver::sedimenta` (filho de `solver/`), um passo depois da advecção: a fracção
    `a·|ĝ_eixo|` do pigmento EM SUSPENSÃO passa à vizinha de baixo, pesada por `min(f_c, f_d)/f_c` (a
    água das duas a tocar-se; seca = 0 ⇒ o pigmento acumula-se na última água da orla). Não move ÁGUA,
    nenhuma decisão por célula (contínua no filme), conservativa (gather), independente de ordem;
    `a = 0` ou `|g| = 0` não entra. `a = k·|g|`, `k = slider × SEDIMENTA_DA_BORDA` (10).
    Rodada única (`sonda_opcoes`, bancada Wet Round + S, água parada × ao vivo), orla de baixo ÷ de cima
    do traço de água 1:

    | `k` no motor | 0 (fábrica) | 1 | 2 | **5 (slider 0,5)** | 10 (slider 1) | 20 | 40 |
    |---|---|---|---|---|---|---|---|
    | meia | 0,77 | 0,85 | 1,07 | **1,56** (ao vivo 1,81) | 4,14 | 5,08 | 5,78 |
    | máxima | 0,78 | 0,97 | 1,30 | 2,88 | 2,95 | 3,49 | 0,72 (drena) |
    | direita | 0,81 | = | = | = | = | = | = |
    | S meia: plano · fios · quadrados | 13 · 3 · 0 | 13 · 3 · 0 | 13 · 3 · 0 | 13 · 3 · 0 | 14 · 3 · 0 | 12 · 3 · 0 | 20 · 6 · 0 |
    | S máxima: plano · fios · quadrados | 10 · 24 · 1 | 10 · 23 · 1 | 10 · 23 · 3 | 10 · 23 · 2 | 10 · 22 · 2 | 10 · 22 · 2 | 11 · 21 · 1 |

    Rebelle a meia: 1,6. Portão `a_heavy_edge_a_meio_pesa_na_orla_de_baixo_como_no_rebelle` (slider 0,5
    pelo autorado: barra 1,3–1,6; controlo DENTRO: a fábrica 0,77 reprova; a régua do recto verde no S a
    meia e no máximo; a tela DIREITA com o slider a 1 = o hash de hoje). ⚠️ Os traços de água 0,4 descem
    a 0,75 com k 5 (o pigmento sai do topo E da orla de baixo, que é fina) — a barra do Rebelle é a da
    água 1.
  - **Painel e preset:** as duas no painel (i18n + tokens), no Reset (voltam ao neutro), no preset
    (`Parametros::live_water` / `heavy_edge`, opcionais: um preset de antes delas NÃO mexe — a regra do
    Tilt; `um_preset_sem_as_opcoes_da_w1g_nao_mexe_nelas`, e `o_pincel_guardado_pinta_o_mesmo_traco`
    agora com as duas ligadas). Costura: `tests/it/seam_wetpaint_opcoes.rs` (presentes só armado, vivas
    sob o ponteiro, o clique real liga/desliga na ferramenta, a edição vira `SetValue` e prende-se a
    `0..1`, as duas dicas).
  - **⛔ O3 «fios finos e compridos no máximo» — NÃO ENTREGUE** (a regra do prompt: nenhuma lei passou
    (a)–(c) numa rodada). Antes de construir lei, medida a COMPOSIÇÃO: 14 variantes dos knobs que o
    motor já tem (`sonda_fios`), S a meia e no máximo, régua nova `ferramentas/fios_s.py` (fios = colunas
    cuja tinta passa a base da orla por ≥ 20 px), calibrada no Rebelle: `s_inclinado` 11 fios, largura
    mediana 5 (máx 7), comprimento mediano 306; `s_inclinado_1` 8 · 5 · 273; `s_zoom100` 8 · 5 · 292;
    `s_plano` 0.

    | variante | máxima: fios · largura · comprimento · finos-e-longos | meia: fios (fábrica 3) | veredicto |
    |---|---|---|---|
    | fábrica | 22 · 10 · 51 · 0 | 3 | a referência |
    | `extFingering` 0,3 · 1,0 · 1,0 + `extRivulets` 8 / 40 | 22–23 · 10 · 48–51 · 0 | 2–3 | ⛔ não afina nem alonga |
    | `leveling` 0,2 · 0,1 | 25 · 10 · 53 · 0 / 33 · 8 · 48 · 0 | **10 / 16** | ⛔ fios a mais a meia |
    | `levelClamp` 0,05 | 30 · 8 · 42 · 0 | **16** | ⛔ fios a mais a meia |
    | `viscosity` 0,05 | 23 · 10 · 60 · 0 | 3 | ⛔ não afina |
    | `capillary` 0,6 · `maxVelocity` 2 | 22 · 10 · 51 · 0 | 3 | ⛔ nada muda |
    | `brake` 0,5 | 24 · 10 · 66 · **1** | **10** | ⛔ 1 fio fino, fios a mais a meia |
    | fingering 1 + rivulets 10 + leveling 0,2 · leveling 0,1 + viscosity 0,05 | 25 · 10 · 46 · 0 / 36 · 8 · 45 · 0 | 7 / 19 | ⛔ |

    Nenhuma variante passa de 1 fio fino e longo (Rebelle 8–11), e todas as que afinam acrescentam fios
    a meia. Confirma a W1e: o fio único/longo é NÃO LOCAL (a conta de baixo a equalizar a pressão ao
    longo da orla) — um mecanismo novo, a medir numa onda própria; os vetos continuam (canal, linha de
    contacto, Create/Easy Drips, DripLaw, trava célula a célula na entrada do seco).
  - Pranchas: [1:1](prancha_w1g_opcoes.png) · [8×](prancha_w1g_opcoes_8x.png) (cada opção desligada ·
    ligada · Rebelle; a linha da Live Water é o S no instante do Up — o Rebelle não tem foto no Up, a
    coluna dele é 1 min depois). Gerador `ferramentas/prancha_w1g.py`.
  - **Smoke do dono (10/10): OK**, com dois achados e um pedido, curados na mesma janela:
    - ⛔ **O preview do painel ignorava a Heavy Edge** (e a Live Water): `CenaDoPreview`/`ChaveDoPreview`
      copiam o estado do fluido para o pintor do preview, e as opções novas ficaram de fora — a família
      «copiar um componente deixa os irmãos». Cura: `opcoes` na cena e na chave. Red-first:
      `a_chave_muda_com_cada_parametro_e_o_traco_tambem` com os casos novos deu «heavy edge: a chave não
      viu» antes da cura.
    - **O Undo com as opções** (`medida_undo`, 4096², quatro ferramentas no MESMO processo, 7 rodadas
      de ordem rodada, mínimo · mediana, load ~37 — o relógio absoluto vale pouco, a comparação vale):

      | | Ctrl+Z | 1.º quadro do traço seguinte | undo guardado |
      |---|---|---|---|
      | fábrica | 55,7 · 69,4 ms | 63,5 · 72,3 ms | 3,1 MB |
      | Heavy Edge 1 | 48,5 · 68,5 | 64,3 · 67,4 | 3,1 MB |
      | Live Water | 53,5 · 66,4 | 66,4 · 78,8 | 3,1 MB |
      | as duas | 49,2 · 70,8 | 62,5 · 82,5 | 3,1 MB |

      ⇒ as opções não pesam no Undo (o Undo não passa por elas: restaura a tela e a água renasce no
      traço seguinte). ⚠️ O Ctrl+Z de ~50 ms a 4096² é anterior à W1g — um item à parte, por medir sem
      carga.
    - **Smooth Edges com intensidade** (pedido do dono: «ajustar a intensidade deste smooth»; ⚠️ a LEI
      abaixo foi substituída no 2.º smoke — «no máximo não se vê» — pela borda suave por raio): a caixa
      virou uma linha `0..1` (o mesmo id, de `CLICKS` para `FIELDS`). A lei: a cobertura premultiplicada
      do pixel da borda entre o desligado (o `dele`) e o ligado, `d + (l − d)·força`; `0` não monta a
      silhueta (o desligado de antes) e `1` é o ligado sem conta nenhuma — os dois AO BYTE contra o FNV
      gravado com o código do checkbox em `362938a99` (`smooth_edges_nos_extremos_e_o_checkbox_de_antes_ao_byte`,
      razão 1 e 2); `smooth_edges_a_meio_fica_entre_o_desligado_e_o_ligado` (a 0,5 cada byte entre os
      dois, nenhum muda onde eles concordam). O Smooth Edges do Watercolor não muda (a cerca). ⚠️ Mais
      forte que o ligado de antes (um pé maior que 3×3) seria outra lei — não pedida, não construída. Mutação «o
      slider vira interruptor» (`forca > 0` = ligado) ⇒ o gate do meio VERMELHO. Como linha numérica o
      nome corta no mínimo do dock (220 px): a catraca `cada_nome_deste_painel_cabe_na_coluna_da_seccao`
      sobe 12 → 13 pelo precedente de 23/09 (linha que ENTRA na população; de 245 px para cima nada muda).
  - **2.º smoke do dono (10/10), três pedidos:**
    - **O preview tem de ser EXATO** («não escorre como o brush real»). O preview pintava num quadrado de
      256 px, o diâmetro preso a 96 px, pressão a subir e a descer, e só **2 s** de água — o escorrimento
      real leva dezenas de segundos (medido, `medida_preview`, perfil smoke, até o motor ficar sem
      fluido: Wet Round 33 s, Wet Wash 60 s; custo 1–8 s numa thread). O preview vivo (`pintar_preview`)
      passou a: o pincel no tamanho REAL numa tela 2:1 de `clamp(10·d, 512, 1024)` px (1:1 com a obra),
      o S no terço de cima (por baixo cabem ~2,9 d de fios: na tela 1:1, Tilt no máximo, água 1, os fios
      descem 0,7–0,8 d no Wet Round e 2,4–2,8 d no Wet Wash), pressão 1 (a do rato), a água até PARAR
      (tecto 120 s) e cada quadro reduzido pela média a 384×192. A receita antiga ficou só para a
      miniatura dos ficheiros (`pintar_miniatura`). Portão `o_preview_escorre_como_a_tela` (Wet Round
      60 px, Tilt máximo: 444 px de fios no último quadro; controlo: a receita de 2 s, 0 px). ⚠️ A
      sonda tinha dado 0 porque o preset traz a tela DIREITA (F4) — o portão liga o Tilt.
    - **O Smooth Edges no máximo não se via.** A 1.ª lei (o 3×3 do Watercolor) é um anti-serrilhado de
      1 px; a 2.ª tentativa (o raio a alisar a FORMA da mancha) também quase não se via — no S a
      silhueta já é lisa; o que se vê duro é o contorno ESCURO, por dentro da mancha. Lei entregue
      (`Suave`): a mancha desfocada por uma tenda de meia largura `raio = slider × 6` dá a faixa da orla
      (`B ∈ (0,1)`); nela a tinta (premultiplicada) é desfocada pela mesma tenda com peso `4·B·(1−B)`. No
      S: ¼ tira a escadinha, ½ amacia, 1 dá uma borda suave. Portões: o zero ao byte (razão 1 e 2);
      `mais_smooth_edges_e_uma_borda_mais_suave` (a aspereza — soma das diferenças entre vizinhos — cai
      27 % → 37 % → 45 % a ¼ · ½ · 1; a tinta total dentro de 2,3 %; mutação «raio fixo» ⇒ VERMELHO); só
      a orla muda (o pé do alcance). ⛔ Recusada: o raio a alisar só a forma (contorno −55 % mas sem efeito
      visível no S). **Custo** (`medida_do_smooth_edges`, perfil smoke, no máximo, mínimo de 7): a tela
      4096² inteira (a recomposição ao mexer no slider) 9,5 → 82 ms na 1.ª versão (load 42) → 45 ms
      saltando as linhas longe da tinta (o desfocado ali é exatamente zero) → **31 ms** guardando o
      pigmento da região num plano (load 9,9); o retângulo de 300² (o quadro normal) 0,13 → **1,1 ms**.
    - **«Paper: None» e Relief 0 ainda mexiam no traço:** sem Kind o motor era semeado com o Cold de
      fábrica (F2), e ligar o Relief arma um papel — então None nunca era «nenhum». Rodada de leis
      (`prancha_w1g_sem_papel.png`; orla de baixo a meia · tinta da bancada · mistura):

      | sem Kind | aspeto | orla meia | tinta | molhado sangra | veredicto |
      |---|---|---|---|---|---|
      | plano a 1 | liso | 2,36 | +65 % | — | ⛔ deixa passar tudo (o Solid e o «pincel menor que a célula» mudam) |
      | plano à média (0,5) | liso | 0,98 | +1,4 % | — | ⛔ o Solid ganha 3 buracos (o limiar come os toques fracos) |
      | plano + lei ESPERADA (depósito e humidade) | liso, vêem-se as cerdas | 3,2 | −8 % | **−5** | ⛔ a água deixa de sangrar |
      | relevo do Cold só na água + lei esperada | idem | 3,2 | −8 % | −4 | ⛔ não é a capilaridade: é o limiar por célula |
      | Cold (o de antes) | as manchas do papel | 0,77 | — | +26 | o pedido do dono |
      | **as alturas do Cold ao acaso por célula** | grão fino, sem desenho | **1,45** | −1 % | **+13** | ✅ **escolha do dono** |

      Entregue: sem Kind, `seed_paper_with` com `alturas_do_papel_de_fabrica()` (64 quantis de uma janela
      512² do Cold) em `celula_ao_acaso` (misturador de 64 bits, determinístico); o Tooth não age sem papel
      (`Dependente::ToothSemPapel`, a dica «Acts when a Paper is chosen»); o «Texture on Paint» só imprime
      um papel escolhido. Portão `sem_papel_o_traco_nao_tem_o_desenho_de_um_papel` (correlação com o
      vizinho: Cold 0,96 · sem papel 0,001; o Tooth não muda um byte; tinta 1,017 da do Cold). O motor
      não mudou (só `paper::quantis_do_papel`).
      **Consequências medidas:** a mistura com a água parada continua na barra (γ 0,59·0,42, sangria seca
      +2,7, molhada +13); a orla a meia sem papel nasce a 1,45 (Rebelle 1,6) — a Heavy Edge foi calibrada
      com o Cold escolhido (0,77 → 1,56) e sem papel leva 1,45 → 2,99; o Solid deixa um bolso de 3×3 na
      FRANJA da orla fora do polígono do rabisco (21, 105) — o portão do Solid passou a contar só dentro
      do polígono (regra não-zero), que é a promessa dele. ⚠️ **Aberto:** com a Live Water e sem papel a
      sangria no seco fica em +3,4…+5 para qualquer velatura (Rebelle +1…+2; com o Cold +2,9) — a água ao
      vivo corre sobre o grão; o portão exige a barra inteira com o Cold e, sem papel, tecto +5.
      As cercas de bytes (`opcoes_tests`, o zero do Smooth Edges) e as 4 miniaturas + tabelas de largura
      de fábrica foram regravadas UMA vez por esta ordem; os presets de fábrica não fixam a Live Water nem
      a Heavy Edge.
      Testes antigos que liam o Cold escondido: `paper_visual_prints_grain…` passa a escolher o Cold; o S
      dos rectângulos corre a régua no Cold (com o controlo) E sem papel (meia 21·2·0, máxima 10·15·1 —
      verde); `the_artists_paper_drives_the_wet_tooth`: o Checker rejeita os vales a 0,602 desde sempre e
      o controlo passou de ~1,01 a 1,002 — a barra 0,6× estava na beira, ficou 0,65×. Na Aquarela
      (`watercolor_papel_nos_vales.rs`, protegido) os dois testes do Tooth do WET PAINT perdem o caso sem
      Kind — 2 linhas, a cerca ganhou `ferramentas/cerca_isencoes.tsv` (o caminho, o número EXATO, a
      razão: uma linha a mais volta a reprovar). O censo dos controlos: o Texture on Paint sem papel
      esmaece com o Tooth (`Dependente::SemPapel`); saiu a exceção «escolher o Cold a partir de None não
      muda a tinta» (agora muda).
  - **3.º e 4.º smoke do dono (10/10):** o preview demorava até 10 s — o 1.º quadro sai em 0,01–0,05 s e
    o traço em 0,2–0,4 s; era a apresentação a esperar a água parar num arrasto (agora mostra o mais
    recente; `o_preview_nao_espera_a_agua_parar`). O Smooth Edges («maravilhoso») documentado em
    [14](14_smooth_edges_a_borda_suave.md), e estendido às bordas SOBRE tinta (o degrau de cor dentro da
    mancha — a lei e a rodada dos limiares no 14 §2–§3). 5.º smoke: as linhas escuras da Heavy Edge por
    dentro da tinta — o detetor de LINHA COMPRIDA (14 §2 passo 5).
  - **Smoke do dono sobre `711513d42`: OK** — e um defeito novo para a PRÓXIMA janela: a mistura de pigmentos
    (amarelo sobre azul seco dá PRETO e nunca cobre; [captura](fixtures/smoke_dono_w1g/amarelo_sobre_azul_seco.png)).
    Prompt: [15](15_PROMPT_janela_auditoria_mistura.md) — auditada e curada na entrada «Mistura» abaixo.
- 🔎✅ **Mistura — amarelo sobre azul seco (10/10, prompt [15](15_PROMPT_janela_auditoria_mistura.md)).**
  Sonda `wetpaint/mistura_camadas_tests.rs::sonda_camadas` (a cena da bancada: referência amarela no branco,
  mancha azul (30, 100, 220) de 9 traços + Dry canvas, 10 passadas de amarelo (245, 205, 25) em serpentina,
  molhado = 0,5 s entre passadas, seco = Dry canvas entre passadas; miolo x 285–315, y 185–215; «escuro» = 2.º
  percentil da luminância em x 240–360, y 155–245; «→amarelo» = fracção azul → tinta em luz linear, R e G).
  Oráculo: Rebelle 8.3.4 pela porta WebSocket, a MESMA cena (+200, +120): `ferramentas/camadas.py`,
  `camadas_tela.sh`, `camadas_regua.py`; fotos em [fixtures/rebelle_camadas/](fixtures/rebelle_camadas/).
  - **O que o Rebelle faz:** nunca escurece; molhado o miolo vai (201,203,134) → (235,206,56) à 3.ª → a cor
    da tinta (245,205,25) à 9.ª; seco (camada a camada) (214,217,169) → (244,225,127) à 3.ª → creme
    (249,226,120) estável desde a 5.ª. O amarelo COBRE o azul por volta da 3.ª passada nos dois modos.
  - **A causa do PRETO é a «Pigment mixing (K-M)» do Tuning ligada.** O preto da captura é (14,14,14) — o
    chão de reflectância da K–M, que a sonda alcança — e os fios verdes (41,180,12) são a cor da K–M. Com a
    mistura de fábrica NENHUMA das 8 condições (tela direita/meia/máxima, Heavy Edge 1, Live Water, esfregar a
    60 px, tudo junto) põe o miolo ou o 2.º percentil abaixo do azul. Com a K–M + a cena do dono (Live
    Water, Tilt meia, Heavy Edge 1, Smooth Edges 1, 60 px, 3 idas por traço) o assentado do miolo vai
    (12,98,43) → (12,80,18) → (12,12,13) → **(2,3,12)** — e o próprio traço amarelo no BRANCO acaba (86,122,19).

    | hipótese | medida (molhado salvo dito) | veredicto |
    |---|---|---|
    | **H1** a velatura compõe-se | K–M, cena do dono: desligar a velatura SÓ na junção «assenta» (drying) → (237,203,25); só no pouso, só no re-molha ou só no render → o mesmo preto. De fábrica nunca desce abaixo do azul | ✅ **sob a K–M**: a K–M já é subtractiva, a velatura por cima conta a absorção duas vezes e o ciclo assenta ↔ re-molha realimenta-a |
    | **H2** sRGB sem tecto | a velatura com tecto no mínimo dos dois por canal = o produto ±2 níveis; a média geométrica em luz linear também | ⛔ não é a causa |
    | **H3** o pincel sujo | de fábrica, simples: o suspenso (189,177,75) contra a tinta (245,205,25); com o pincel a apanhar só o MOLHADO (244,205,25), o miolo à 10.ª 0,63 → **0,81**; cena do dono 1.ª passada 0,53 → **0,79** | ✅ **é o «nunca cobre»**: o bico apanhava o azul ASSENTADO (peso `0,5·α(sett)`) a cada dab |
    | **H4** a cobertura | a massa azul do miolo 1984 → 1707 em 10 passadas (a re-molha tira 14 %); sem re-molha o miolo de fábrica sobe só +7 níveis; com a K–M é decisivo (seco final 0,35 → 0,98 com Staining 1) | ✅ residual, só sob a K–M — é a lei do **Staining** (doc 23: «quão permanente é a tinta seca», 0,5 de fábrica) |
    | **H5** o render | sem a velatura do render o ecrã molhado sobe (157 → 182 no 1.º) mas o ASSENTADO preto é o mesmo | ⛔ o preto é pigmento, não render |

  - **A cura (duas leis, as duas no produto; o port e a impressão digital não mudam):**
    1. **Com a K–M não há velatura** — `WetSession::reconcile_facts` troca `sim.glaze` com a caixa
       (`VELATURA_MEDIDA` ↔ 0), como a W1g troca a do filme com a Live Water. A velatura do render fica (não
       se guarda, não realimenta).
    2. **O pincel sujo só apanha tinta MOLHADA** — `Sim::pickup_assentado` (o peso do assentado no que o
       bico recolhe, `trail/transfer.rs::tip_rows`): port 0,5 (ao bit: a mesma multiplicação), produto 0.
       O Rebelle já o dizia na W1e (rasto − azul ≈ 0 sobre o seco).

    | miolo, passadas 1 · 3 · 10 · seco (→amarelo) | antes | depois | Rebelle |
    |---|---|---|---|
    | de fábrica, molhado | (157,157,79) · (173,165,62) · (199,177,44) · (180,166,53) — 0,39 · 0,48 · 0,63 · 0,50 | (197,177,32) · (206,181,26) · (224,190,27) · (205,178,36) — 0,62 · 0,68 · **0,81** · 0,66 | (201,203,134) · (235,206,56) · (245,205,25) · (245,205,26) |
    | de fábrica, cena do dono | 0,53 · 0,82 · 0,96 · 0,97 | **0,79 · 0,99 · 1,00 · 1,00** | idem |
    | de fábrica, seco | (128,138,73) · (229,192,25) · (245,205,25) — 0,24 · 0,84 · 1,00 | (149,148,28) · (244,203,24) · (245,205,25) — 0,33 · **0,98** · 1,00 | (214,217,169) · (244,225,127) · (249,226,120) |
    | K–M, cena do dono | (98,151,35) · (102,140,20) · (99,92,17) · **(2,3,12)** | (161,182,21) · (223,200,23) · (243,205,25) · (244,205,25) — 0,54 · 0,87 · **0,99** · 0,99 | — |
    | K–M, molhado simples | (148,158,75) · (97,162,40) · (57,153,22) · (30,134,23) — escurece a cada passada | (175,180,26) · (131,182,23) · (133,185,24) · (94,173,27) — 0,57 · 0,47 · 0,49 · 0,35, **nunca abaixo do azul** | — |
    | K–M, seco | (63,135,64) · (180,181,21) · (227,202,24), orla 2.º percentil 0,10 < azul | (138,192,25) · (243,205,25) · (245,205,25) — 0,54 · 0,99 · 1,00 | — |

  - **Portão** `n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto` (6 cenas: de fábrica e K–M ×
    molhado simples · cena do dono · seco). Barra do Rebelle: (a) o miolo nunca abaixo da luminância do azul
    e o 2.º percentil nunca abaixo de 0,8× dela; (b) à 10.ª passada `≥ 0,70` do caminho para o amarelo e
    não recuou da 1.ª. **Nasceu VERMELHO** no produto de hoje («de fábrica, molhado: 0,63»; «K–M, cena do
    dono: (2,3,12)»; «K–M, seco: 2.º percentil 0,10»). Controlos DENTRO: a velatura por cima da K–M na
    cena do dono e o pincel que apanha o assentado (de fábrica, molhado) reprovam. Mutações no produto
    (restauro por cópia + `touch`): «a K–M volta a levar a velatura» (`reconcile_facts`) ⇒ VERMELHO (K–M,
    cena do dono, miolo (104,103,17) à 3.ª); «o pincel volta a apanhar o assentado» (`pickup_assentado`
    0,5 no nascimento da sessão) ⇒ VERMELHO (de fábrica, molhado: 0,63); restaurado ⇒ verde.
    ⚠️ Custa ~86 s no perfil smoke (9 cenas de 600×400 a 10 passadas) — o portão mais caro da família.
  - **Cercas, verdes depois:** `cerca.sh` · `os_outros_meios_nao_mudam` + miniaturas de fábrica · a suíte de
    `ph2d-wet-paint` (as impressões do port) · o censo dos controlos (4 features) · a ponte do preview
    (`--features panel-painter-layers`) · `--lib wetpaint` inteiro (158, com `opcoes_tests` — os bytes do
    S e da bancada NÃO mudaram: num traço de uma cor o bico apanha a própria cor, e as miniaturas idem) ·
    `mistura_tests` (a barra de uma camada: γ seco 0,59·0,42, sangria +2,7 · molhado +13 — igual com e sem
    as curas, medido no mesmo processo). «Antes»: as cercas fecharam verdes na W1g (`711513d42`) e o rebase
    sobre `782f90b69` não tocou no Painter; não as re-corri antes da cura.
  - ⛔ **Recusas medidas:** o tecto da velatura no mínimo dos dois por canal (= produto, não cura nada) · a
    velatura pela média geométrica em luz linear (100 % e 200 %: = produto de fábrica) · zerar a re-molha no
    produto (de fábrica +7 níveis; é a lei do Staining, o controlo do artista desde o doc 23, e mexer-lhe
    move os lavados molhados) · desligar a velatura de fábrica (o seco da W1e volta a 0,06).
  - **Aberto — decisão do dono:** com a K–M ligada, a tinta MOLHADA sobre o azul seco re-dissolve ~14 % dele e
    a K–M (um pouco de azul domina o amarelo) segura o miolo verde-amarelado (0,35–0,49, nunca abaixo do azul);
    com a Live Water ou camada a camada cobre (0,99). **Staining 1** faz cobrir também ali (0,98). Subir o
    Staining de fábrica quando a K–M liga seria uma lei nova — não pedida, não construída.
  - Pranchas: [1:1](prancha_mistura_camadas.png) · [8× no miolo](prancha_mistura_camadas_8x.png) (Rebelle ·
    de fábrica antes · depois · K–M cena do dono antes · depois; colunas azul · 1 · 2 · 3 · 5 · 10 · seco).
    Gerador `ferramentas/prancha_camadas.py`.
- 🔎 **Conferência dos 4 pincéis de fábrica (dono 10/10: «o Wet Wash é seco mesmo? o nome confere?»).**
  A bancada de `gotas.py` (4 traços de 150 px a 40 px) dos dois lados: Rebelle `--sem_motor` (Round · Wash
  Round · Flat Dry · Round água 100; [fixtures/rebelle_presets/](fixtures/rebelle_presets/)) e a sonda
  `wetpaint/sonda_presets.rs` (Wet Round · Wet Wash · Wet Dry Brush · Wet Drip, cada um numa tela sua, 30 s);
  régua `ferramentas/presets_regua.py` (tinta = mediana de `255 − min(rgb)` no centro; orla = borda ÷ centro);
  [prancha](prancha_presets_conferencia.png).

  | plano, 30 s (inclinado: escorre) | tinta | largura (40) | orla | papel a aparecer | escorre inclinado |
  |---|---|---|---|---|---|
  | Wet Round · Rebelle Round | 210 · 54 | 25 · 32 | 0,87 · 0,92 | 0 · 0,01 | 12 · 340 |
  | **Wet Wash · Rebelle Wash Round** | **214 · 39** | 30 · 41 | **0,90 · 1,41** | 0 · 0 | **115 · 0** |
  | Wet Dry Brush · Rebelle Flat Dry | 160 · 102 | 19 · 34 | 0,64 · 1,00 | **0,20 · 0** (estrias) | 1 · 1 |
  | Wet Drip (o Tilt dele) · Rebelle Round água 100 | 210 · 60 | 30 · 32 | 0,92 · 0,91 | 0 · 0 | 70 · 344 |

  ⇒ **O nome não confere no Wet Wash:** a aguada do Rebelle é a MAIS clara, lisa, a mais larga, com a orla
  fina e escura ao secar, e não escorre (água 24); a nossa carrega tanta tinta como o Round e é a que MAIS
  escorre. Round, Dry Brush e Drip conferem com o nome (o Dry Brush é fios partidos onde o do Rebelle é
  estriado e cheio; os fios do Drip são grossos e curtos — o «fio único» aberto da W1e). ⚠️ Os quatro
  carregam 3–4× a tinta do Rebelle e saem 25–30 px a 40 (a largura prometida, W2). Decisão do dono pendente.
- ✅ **Assentar não muda o que se vê (dono 10/10: «Pigment mixing + Glaze layering pintando sobre tinta seca
  ainda tem uma aparência em 2 etapas — isso não pode acontecer»).** Sonda e portão em
  `wetpaint/mistura_camadas_tests.rs` (`duas_etapas`: uma passada de amarelo sobre o azul seco, o miolo com o
  rato em baixo contra +1 · +30 · +120 · +600 ticks e o Dry canvas).
  - **Causa medida:** o ecrã mostra o filme molhado sobre o assentado por UMA lei (o `lerp` + a velatura do
    filme, ou a camada K–M com o «Glaze layering»), e o motor, ao assentar e ao re-molhar, recalculava a cor
    por OUTRA (a junção mistura + velatura, ou a K–M com o «Pigment mixing»). Pior Δ (níveis sRGB), antes:
    fábrica 29–35 · K–M 81 · Glaze 28 · as duas **126** (o miolo (230,194,45) → (106,180,26)); com a Live
    Water 45 · 49 · 37 · 103. O «Glaze layering» nem chegava ao motor (só ao ecrã).
  - **Cura** (`crates/ph2d-wet-paint/src/assenta.rs`): as transferências VERTICAIS (assentar, Dry canvas,
    re-molhar — o mesmo ponto, a mesma coluna de tinta) resolvem a cor nova pela inversa da lei do ecrã
    (`assenta::cor`, que é agora a porta do render: ⛔ uma só). Palpite físico (a parte que assenta é uma
    camada do filme por baixo do filme que fica — exacto para o `lerp`), afinado por Newton com derivada
    numérica onde o ecrã vê a cor (sensibilidade ≥ 0,1); a camada K–M por fórmula fechada (assentar) e pela
    raiz da quadrática (re-molhar). `Sim::lei` (port `Porto` ao bit; produto `PorCima`, `Camada` com a caixa,
    levada pelo reconcile como facto novo `km_glaze`). A mistura fica onde acontece: na água (o pouso, a
    advecção). ⛔ Duas armadilhas medidas: assumir a derivada `1 − a` dava passos 4× grandes e o miolo ia a
    PRETO; corrigir onde o filme grosso esconde o assentado escrevia uma cor ESCONDIDA que aparecia quando o
    filme afinava (+24 de verde ao secar) — por isso o limiar.
  - **Depois:** com a Live Water ≤ 7 nas 4 combinações (fábrica 6 · K–M 6 · Glaze 5 · as duas 5); sem ela a
    água só nivela depois do Up (a escolha da W1f para a 4K) e todas mudam ~20 nos primeiros 0,5 s — a
    mesma em todas, não é a mistura. A K–M deixa de mudar o que quer que seja ao secar (linhas iguais).
  - **A velatura do filme passou a valer também depois** (o seco fica com a cor do ecrã): `VELATURA_DO_FILME`
    0,20 → **0,25** (seco 0,62 · 0,50, sangria +2,9; a 0,20 a sangria seca dava +3,1 ✗; tabela no doc da
    constante). Live Water 0,35 continua (com o Cold verde; sem papel +4,7, tecto +5).
  - **Portão** `o_traco_nao_muda_de_aparencia_ao_secar` (Live: ≤ 12 nas 4; sem Live, as duas da K–M ≤ 30),
    CONTROLO dentro (a lei de antes com as duas caixas reprova). O portão das camadas ganhou o controlo das
    DUAS leis de antes (cada cura sozinha já tira o preto). Unidade `assenta::tests` (≤ 1,5 níveis em 7
    estados, incluindo filme a 99 % e opaco). Cercas verdes: suíte de `ph2d-wet-paint` (impressões),
    `--lib wetpaint` 159 (bytes do S e da bancada, `mistura_tests`, Live Water), miniaturas, ponte do preview.
- **Falta (só com ordem do dono):** o fecho da linha (`/pd-linha-fechar`, handoff).
  (O degrau 187 → 188 das gotas foi desfeito com elas: a linha não muda o schema.)

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
