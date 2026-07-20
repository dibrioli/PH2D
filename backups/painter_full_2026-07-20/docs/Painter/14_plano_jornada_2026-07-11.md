# Plano da jornada 2026-07-11 — aquarela UI (#9-#12) + investigações (#13-#16)

> Fila-fonte: [`13_fila_integracao_watercolor_secoes.md`](13_fila_integracao_watercolor_secoes.md)
> (lote 2026-07-11). Este doc ancora cada item em file:line reais (mapeados 2026-07-10) para o
> implementador não recomeçar do zero. **Método obrigatório das investigações:** o do
> [`BUGS_painter.md`](BUGS_painter.md) #8 — instrumentar o app real → bissectar → perfil 1D →
> sondas → só então fix com RED→GREEN. Regra da casa (Modo L): commit local, **sem push/integração**.

## Ordem recomendada + racional

1. **#11 slider de secagem** — FUNDAÇÃO (const → campo de runtime; abre o plumbing de controle
   canvas-level). Barato.
2. **#9 + #10 Dry / Wet** — reusam o teardown atômico + um pour novo; mesma seção do painel. Barato.
3. **#12a overlay de umidade** — accessor + véu em `draw_overlays`. Barato-médio, visual.
4. **#13 substrato não-capturado** — RED primeiro; fix provável = encerrar sessão na troca de
   substrato. Médio.
5. **#14 retângulos do Per-Layer Color** — REPRODUZIR + bissectar antes de tudo (handoff diz
   "resolvido"; a foto diz que não). Investigação, médio.
6. **#16 pesquisa traço-3D** — estratégico: pode **substituir** o Per-Layer Color e dissolver a
   dívida de perf de #15. Vem ANTES de sobre-investir em #15.
7. **#15 perf Per-Layer (confirmar, não sobre-investir)** + **#12b secagem paulatina → mescla** —
   conforme fôlego; #15 já teve a maior parte do trabalho feito (ver abaixo), #12b é design fundo.

Front-load a UI barata que o Enio pediu direto (#11/#9/#10/#12a, baixo risco) e depois as
investigações; #16 antes de #15 porque pode obsoletar o esforço de perf.

---

## BATCH A — UI / lifecycle da aquarela

### Recipe comum do painel (mapeado 2026-07-10, vale p/ #9/#10/#11)

Painel `crates/ph2d-panel-painter-layers/`. Adicionar widget à seção Watercolor **existente NÃO
toca os 5 sites de painel-novo** (registry-init/shell/z-order/visibility/forwarding). Toca só:

- **id:** `crates/ph2d-editor-core/src/ids/chrome/painter_watercolor.rs` — novo `const
  PAINTER_WATERCOLOR_*` + **bumpar o array**: `PAINTER_WATERCOLOR_FIELDS: [NodeId; 24]`
  (`:153`, para slider) ou `PAINTER_WATERCOLOR_CLICKS: [NodeId; 4]` (`:141`, para botão). O
  `[NodeId; N]` fixo faz omissão virar **erro de compilação**, não passe silencioso. Auto-exportado
  como `core_ids::X` (`ids/chrome/mod.rs:60`).
- **paint:** `src/paint_watercolor.rs` (cards Wash/Brush/Water) ou `src/paint_watercolor_paper.rs`
  (Paper/Grain). Slider = `card_row(..., ID, valor, min, max, step, casas)` (`paint_watercolor.rs:334`
  → `number_field::chip`). Botão = template do reset (`paint_brush_top.rs:104/149`) ou checkbox
  (`paint_checkbox_row`, `paint_brush_top.rs:75`).
- **populate:** `src/populate.rs` — slider entra no loop que encadeia `PAINTER_WATERCOLOR_FIELDS`
  (`:128`); botão no loop de Button (`:275-363`, push `:350-355`).
- **event:** `src/event.rs` — botão via `PAINTER_WATERCOLOR_CLICKS.contains` (`:432` → Click
  `:452`); slider via `is_param_field` (`:533` → SetValue). `is_param_field` lê
  `PAINTER_WATERCOLOR_FIELDS` (`number_field.rs:47`) — automático ao bumpar o array.
- **tool route:** `crates/ph2d-tool-painter/src/tool/paint/watercolor_settings.rs` —
  `route_brush_watercolor_event` (Click arm ~`:23-42`, SetValue arm ~`:94`). Novo setter + espelhar
  em `reset_brush_watercolor` (`:313`).
- **gate automático:** o seam test `crates/ph2d-panel-painter-layers/tests/seam.rs:427/454` itera os
  DOIS arrays → cobre o id novo e **FALHA** se faltar o forward em `event.rs` ou o arm no route.
  (Esse é o alvo irrefutável do item — não "compila logo funciona".)

### #11 — Slider de tempo de secagem (FUNDAÇÃO)

**Meta:** expor o tempo de secagem como slider (default 10 s; hoje `CANVAS_WET_DRY_PER_S = 25.5`
const em `watercolor_backdrop.rs:213`, usada num único ponto `:304`).

**DECISÃO DO ENIO (2026-07-10): propriedade de CANVAS/SESSÃO** (não de brush). Campo
`paint.dry_rate_per_s: f32` (default 25.5) em `PaintState` (`state_default.rs`), lido em
`dry_canvas_wet` no lugar da const `CANVAS_WET_DRY_PER_S` (`watercolor_backdrop.rs:213/304`). O
painel lê via um snapshot dedicado minúsculo (thread-local tipo `CURRENT_BRUSH`, publicado 1×/frame
em `painter_bridge.rs` junto do brush snapshot). (Razão: `brush_by_mode` daria uma taxa por modo de
pincel — sem sentido para a secagem do papel.)

**Mapeamento UI:** o slider mostra SEGUNDOS (2–60), o setter converte `rate = 255 / clamp(seg, 2, 60)`
(2 s→127.5, 10 s→25.5, 60 s→4.25). **`∞ = nunca seca` NÃO entra no v1** (Enio não pediu; fica como
follow-up = checkbox → rate 0, com o caveat de que a sessão nunca teardown sozinha + buffer não cai).

**Passos:** (1) campo em PaintState + default; (2) trocar a const por `self.paint.dry_rate_per_s`
em backdrop.rs:304; (3) snapshot p/ o painel; (4) id `PAINTER_WATERCOLOR_DRY_TIME` + array bump; (5)
card_row no Wash/Brush card; (6) populate + event + setter `set_dry_time_s` (converte seg→rate).
**Byte-identity:** default 25.5 = comportamento atual. **Teste:** seam auto-cobre o id;
adicionar um unit `set_dry_time_s(10.0) ⇒ dry_rate_per_s ≈ 25.5` e `(60.0) ⇒ ≈ 4.25`.
**Risco LOC:** conferir `paint_watercolor.rs` / `watercolor_settings.rs` (perto do teto? medir
com `fmt` antes) — se estourar, sibling module, nunca allowlist.

### #9 — Botão "Dry" (secar já) + #10 — Botão "Wet" (molhar canvas)

**Meta #9:** encerrar a sessão molhada AGORA (bake vira definitivo) — o "Dry the layer" do Rebelle.
**Meta #10:** re-molhar o papel SEM depositar pigmento — próximo traço funde/bloom sobre a pintura
existente ("Wet the layer" do Rebelle).

**Mecânica #9 (barato — o teardown já existe):** o bloco atômico de teardown já está em
`dry_canvas_wet` (`watercolor_backdrop.rs:325-340`: zera canvas_wet/rect/carry + session_base +
buffers de união + soak + wet_cum_dirty). **Extrair** esse bloco para `fn dry_session_now(&mut self)`
e chamá-lo do route do botão. Guarda: só faz sentido com sessão viva (`canvas_wet_rect.is_some()`);
com stroke aberto, o mesmo cuidado do `:325` (`stroke.is_none()`) — ou negar o clique enquanto pinta.

**Mecânica #10 (novo pour):** `fn wet_canvas_now(&mut self)` que preenche `canvas_wet` = 255 no
canvas inteiro (ou numa região) SEM tocar `stroke_color`/pigmento, e arma `canvas_wet_rect` = canvas
todo. É o inverso do `pour_canvas_wet` (`:245`) mas com moisture cheia e sem depender de
`stroke_coverage`. Cuidado: precisa também garantir que a próxima sessão CONTINUE
(`wet_session_continues` exige `stroke_coverage`/`stroke_color` dimensionados + `wet_session_base` +
`Arc::ptr_eq` do canvas) — o botão Wet cria o estado de "papel molhado" mas o primeiro traço
seguinte é quem congela a base; validar que `wet_session_continues` aceita "molhado sem base ainda"
ou ajustar a condição (o Wet-then-paint deve fundir). **É o ponto sutil do #10 — testar no smoke.**

**UI:** 2 botões `PAINTER_WATERCOLOR_DRY` / `PAINTER_WATERCOLOR_WET` (array CLICKS 4→6), template do
reset, labels "Dry"/"Wet" (inglês, HR-15 ok — memory `app_ui_english_only`). Route: 2 Click arms →
`dry_session_now` / `wet_canvas_now`. **Byte-identity:** ações puras, nenhum caminho default muda.
**Teste:** unit `dry_session_now` esvazia `canvas_wet` + `wet_session_continues()==false`;
`wet_canvas_now` seguido de um traço funde (mede que o 2º traço lê base molhada).

### #12a — Preview de umidade on-canvas (véu de umidade, estilo Rebelle "show wetness")

**Meta:** overlay que mostra onde o papel está úmido (o `canvas_wet` já existe e decai por byte no
heartbeat).

**Mecânica:** (1) accessor `pub fn canvas_wet_view(&self) -> Option<(&[u8], u32, u32)>` no
PainterTool (hoje `canvas_wet` NÃO é exposto ao shell — grep confirmou zero `pub`); (2) desenhar o
véu em `shells/desktop/src/render_loop/painter_bridge_overlays.rs::draw_overlays` (`:16`), um tint
semitransparente (brilho/frio) proporcional ao byte de umidade, só onde `> 0`. Gate: só em modo
aquarela + com sessão viva. **Byte-identity:** read-only, não toca o composite. **Perf:** desenhar
só o `canvas_wet_rect`. **Risco:** tokens de cor (zero hex — usar ColorToken; pode faltar um token
de "umidade", criar via design system). **Teste:** seam do overlay se houver; senão smoke visual.

### #12b — Secagem PAULATINA influenciando a mescla (design fundo, deferir se faltar fôlego)

**Meta:** hoje a fusão da sessão é BINÁRIA (molhado enquanto `canvas_wet_rect` vivo). Usar o valor
LOCAL de `canvas_wet` para atenuar progressivamente a fusão/derretimento do rim conforme seca
(meio-seco = meio-rim), casando com o settle do GRAN-1.

**Por que é fundo:** acopla ao modelo EDGE-1 de sessão (`wet_session_continues` é boolean; o
composite não lê umidade local hoje). Precisa: (a) o composite ler `canvas_wet[px]` como peso de
fusão por pixel; (b) reconciliar com o guard de reprodutibilidade
(`watercolor_session_rerender_reproduces_the_bake_byte_exact` — a umidade decai por tempo, então o
re-bake precisa ler o MESMO campo, não um decaído). Provável necessidade de congelar o
`canvas_wet` no bake (como o soak já é congelado). **Entregável:** design curto + RED antes de
mexer. Não fazer junto com #12a.

---

## BATCH B — Investigações

### #13 — Retângulos + re-estilo da união ao mudar paper/grain (aquarela)

**Sintoma (Enio):** traço 1 → mudar QUALQUER prop do brush watercolor (Amount do grain, Same as
Paper, textura do papel) → traço 2 sobre poça úmida ⇒ retângulos ao redor do brush + o brush novo
aplicado a TODA a área úmida (mesmo onde não tocou).

**Diagnóstico CONFIRMADO por leitura (2026-07-10):** `WetStrokeStyle::capture`
(`watercolor_field.rs:535-555`) captura fill/depth/edge_gain/wet/granulation/warp/pigment_mix/color/
geometria — mas **NÃO** captura `paper` (TextureSettings), `paper_depth`, `granulation_use_paper`
(Same as Paper) nem o Grain slot (`brush.texture`). O composite lê esses GLOBAIS do brush vivo
(`watercolor_render.rs` ~L230-250: `paper_tex`/`paper_active`/`paper_img`/`gran_tex`/`gran_own_map`/
`gran_img`/`paper_depth`) ⇒ a união re-renderiza inteira com o substrato novo = "aplica a tudo". Os
retângulos = janelas incrementais do frame re-renderizando com globais novos contra o bake velho
(classe do Δ2 do take 7, amplificada de sub-visível pra gritante pela troca de substrato).

**Complicador (o que torna isto mais fundo que o fix de campo do take 10):** o cache
`wet_substrate` (`watercolor_render.rs`, memoiza `paper_h` por pixel de canvas, invalidado no
pen-down em `watercolor_backdrop.rs:52-60`) **assume UM papel por sessão**. E paper/grain são
IMAGENS (`paper_image`/`texture_image` em PaintState), não só escalares — capturar substrato
por-dono exigiria armazenar imagens por-dono + um cache de substrato por-dono. Caro.

**DECISÃO DO ENIO (2026-07-10):** mudar config de pintura no painel **NÃO encerra a sessão NEM
aplica aos traços já pintados** — a sessão continua (fusão preservada) e o param novo vale só pro
traço NOVO. Isso é o fix POR-DONO (mesmo padrão do "molhado" no take 10), **NÃO** o session-break.
O "aplica a tudo" é o bug; a cura é capturar o substrato no estilo por-dono, não reiniciar a sessão.

**Passo 1 (RED imediato):** estender `watercolor_session_brush_changes_do_not_touch_baked_washes`
para variar paper-kind / granulation_use_paper / Amount / grain entre o traço A e o B → deve dar
RED (o bake de A muda com o substrato novo). Confirma o diagnóstico na árvore.

**Passo 2 (fix = captura por-dono, preservando a fusão):**
1. **Estender `WetStrokeStyle`** (`watercolor_field.rs:517`) com os campos de substrato:
   `paper` (TextureSettings), `paper_depth`, `granulation_use_paper`, e o Grain slot
   (`brush.texture`) — capturados em `capture()` (`:535`) com os clamps exatos do composite.
2. **Imagens (paper_image / texture_image):** são Arc-backed em PaintState → capturar o
   **Arc handle** por-dono é barato (refcount). Se não forem Arc, envolver em Arc uma vez. Cobre o
   caso "carregar textura de papel nova no meio da sessão".
3. **Composite resolve por-dono** (`watercolor_render.rs` ~L230-250): hoje lê `paper_tex`/
   `paper_active`/`gran_*` GLOBAIS do brush vivo → passar a resolver pela `style_at(owner)` (como o
   `wet_field` do take 10 já faz para os termos wet-driven). O paper_h por pixel usa o papel do
   DONO daquele pixel.
4. **Cache `wet_substrate`** (memoiza paper_h por pixel assumindo UM papel): **fast-path
   single-substrate** = todos os donos compartilham o substrato ⇒ cache como hoje, **byte-idêntico**
   (o caso comum, e o guard de byte-identity exige isto). **Slow-path multi-substrato** = donos com
   substratos distintos ⇒ desliga o cache e computa paper_h por pixel pelo papel do dono (mais lento,
   correto). Detectar com um flag "sessão tem >1 substrato distinto" setado no push_capture.

**Verificar com o RED→GREEN:** o fix remove o "aplica a tudo" (A mantém seu papel, B usa o novo) E
os retângulos? Se um retângulo transiente sobrar no 1º traço pós-troca (janela incremental), é o
resíduo Δ2 conhecido — mesma classe dos `#[ignore]` do take 7, avaliar separado.

**Custo/risco:** `WetStrokeStyle` cresce (hoje 12 campos escalares → +settings +2 Arc); a tabela é
por-sessão (≤255 estilos), ok. O composite ganha ramos por-dono no caminho de paper/grain — cuidar
LOC de `watercolor_render.rs` (699/700! provável extração pro `watercolor_rewet_px.rs`) e a
byte-identity do single-substrate.

### #14 — Retângulos do Per-Layer Color no brush comum (foto do Enio 2026-07-10)

**MUDANÇA DE QUADRO (crítica):** o handoff `HANDOFF_per_layer_color_perf_artifacts.md` declara o
artefato retangular **RESOLVIDO** em 2026-06-29 — era **leitura de textura GPU não-inicializada**
(slot `acquire_individual_empty` sem clear → fix `clear_all_mips_transparent`). Confirmei que o clear
**ainda está no lugar** (`individual.rs:795` chama, dentro de `create_entry_empty:767`). Logo a foto
de ontem é **classe nova OU regressão de outra coisa** — NÃO o bug de rect da CPU que o §6 do handoff
já REFUTOU (teoria do "coverage-map sujo" — não re-investigar).

**Plano (método BUGS #8):**
1. **Reproduzir** o cenário exato da foto no app (brush comum + Per-Layer Color, traço que mostra os
   retângulos).
2. **Bissectar** com `PH2D_PAINT_FULL_UPLOAD=1` (confirmado ativo em `painter_bridge.rs:516-521`):
   - Some com o toggle ⇒ é a lane de upload PARCIAL (o bbox de `take_preview_upload_bbox` /
     `extract_region`) — uma região subindo antes do seed cheio, ou um bbox fora de sincronia.
   - Persiste com o toggle ⇒ é o buffer CPU de preview OU um slot GPU não-inicializado que NÃO passa
     por `create_entry_empty` (o toggle força upload cheio, não recompose cheio). Suspeitos: os rects
     axis-aligned de `save_region`/`restore_region` (`region.rs:35-57`, ambos `mark_dirty`), ou o
     mitigador `reseed_preview_base` (handoff L70-76) não cobrindo o caso da foto.
3. **Perfil/sondas** conforme o resultado da bissecção; fix só com RED→GREEN nos params reais (dump
   por eprintln 1-linha, padrão `[wet-diag]`).

**Nota:** manter DISTINTO de #13 — "retângulo" aqui é GPU/preview-slot; lá é composite-CPU com
substrato não-capturado. Mesma aparência, raízes diferentes.

### #15 — Perf do Per-Layer Color (CONFIRMAR o teto, não sobre-investir)

**Estado real (mapeado 2026-07-10 — a maior parte das lições da aquarela JÁ foi aplicada):**
- **Profile de build:** `ph2d-tool-painter` ESTÁ na lista `[profile.dev.package.*] opt-level=2`
  (`Cargo.toml:33-40`, junto de painter-brush/color/rayon) → o stamp per-layer é opt-2 já em dev.
  MAS o shell/`painter_bridge` NÃO está (opt-0) → **medir em `--release`** (BUGS #7 raiz 1).
- **Composite 2×/frame** (BUGS #7 raiz 2): é caminho de aquarela (heartbeat) — Per-Layer Color não
  tem heartbeat compositando; provável N/A. Confirmar.
- **Paralelização** (ADR-0109): já paraleliza via `std::thread::scope` row-banded
  (`stamp_color_cache.rs:357`, `stamp_color_dynamic.rs:382`, gate `PARALLEL_MIN_AREA`) — não é rayon,
  mas já é paralelo. Migrar p/ rayon é opcional, ganho incerto.
- **Stamp cache** (`project_painter_texture_brush_stamp_cache`): o caminho comum JÁ cacheia
  (`ensure_color_stamp_cache`, `stamp_color_cache.rs:379`, `ColorStampMask` por camada keyed por
  appearance). SÓ o caminho DINÂMICO (Rake/Random/Jitter/Randomize Color,
  `stamp_dabs_per_layer_dynamic:26`) re-amostra silhueta×Grain×RGB por pixel/dab — candidato restante.
- **Re-stamp incremental** (só dabs novos): AVALIADO e explicitamente NÃO implementado no handoff
  (L22-26 — uma linha de 2 pontos re-stampa tudo de qualquer forma; o row-banding cobriu todos os
  casos com menos risco).

**Conclusão honesta:** o teto que sobra é o accumulate **O(D·N·S)** (96,5% do move — handoff
L181/200), FUNDAMENTAL ao design "N camadas como pincel". Pouca fruta baixa na CPU. O plano de record
do handoff é migração GPU-residente (§4.2).

**#15 é CONTINGENTE a #16 (decisão do Enio):** se #16 confirmar o traço-3D nativo mais leve, o
Per-Layer Color é **inativado** e #15 fica MOOT (não otimizar o que vai sair). Portanto: **fazer #16
ANTES**, e só voltar a #15 se #16 NÃO substituir o Per-Layer Color. Se voltar: **entregável de #15**
= medir em release + confirmar o checklist acima (documentar o que já está aplicado p/ não re-fazer)
+ concluir que o ganho real vem de migração GPU (§4.2), não de mais CPU.

### #16 — PESQUISA: traço de aspecto 3D sem N camadas (o objetivo do Per-Layer Color)

**Insight que conecta a #15:** Per-Layer Color é O(bbox·N) PORQUE compõe N camadas pra FALSIFICAR
3D. A técnica-padrão da indústria (**height-map acumulado + 1 passe de iluminação por gradiente**) é
O(pixels)×1 — não precisa de N camadas. Então #16 não é só "uma alternativa": é o candidato a
**substituir** o Per-Layer Color e dissolver o teto de #15.

**Pesquisa (verificar fontes ANTES de afirmar — `no_industrial_claims_without_verification`):**
Corel Painter Impasto, Rebelle 7 (impasto/metallic), ArtRage (oil thickness), Krita (bump do brush),
e o que Procreate faz em brushes "dimensionais" (blend modes + shading normal-like? vs 3D Materials,
que é pintar EM modelo 3D — coisa diferente). Confirmar a família height+lighting por
grep/WebFetch/docs, não de memória.

**Esboço técnico (a validar):** o traço acumula um canal de ALTURA `h` (buffer u8/f16 irmão do
coverage); um kernel por-pixel deriva a normal do gradiente de `h` (Sobel/forward-diff) e aplica
iluminação direcional + specular. Custo O(pixels do dirty-rect)×1 passe (gradiente + dot product) —
ordens abaixo de O(bbox·N). Casa com o pipeline (ADR-0109: kernel puro por-pixel; HR-5: diferenças
finitas = sem transcendental salvo o specular, tabelável).

**DECISÃO DO ENIO (2026-07-10):** *"se souber como fazer os brushes de aspecto 3D como o Procreate
e for mais leve, podemos inativar o Per-Layer Color."* → aprovação CONDICIONAL de implementar +
inativar. O caminho de #16 vira: (1) pesquisar/CONFIRMAR a técnica; (2) protótipo + medir; (3) SE
mais leve E der o aspecto 3D → implementar o traço-3D nativo e **inativar o Per-Layer Color**.

**Ressalva honesta (Procreate especificamente):** não afirmar de memória que o Procreate usa
height+lighting — o Procreate NÃO tem um "3D stroke" marquee como o Impasto do Painter; o aspecto de
volume dele pode vir de textura+grain+blend, ou de shading normal-like no brush studio (≠ "3D
Materials", que é pintar EM modelo 3D). **Passo 1 é descobrir o que o Procreate REALMENTE faz**
(WebFetch/docs/brush studio) + confirmar a família Impasto (Painter/Rebelle/ArtRage/Krita bump) por
fonte, não palpite (`no_industrial_claims_without_verification`).

**Entregável:** (a) pesquisa confirmando a técnica; (b) protótipo height+lighting MEDIDO num cenário
fixo (ms/frame por knob) vs Per-Layer Color no mesmo cenário; (c) se mais leve + convincente →
implementar nativo + inativar Per-Layer Color (drop-in: o canal `h` é buffer irmão do coverage, o
lighting é 1 kernel por-pixel). Se NÃO for mais leve / não convencer → doc registra o resultado e
Per-Layer Color fica (aí #15 volta a importar).

---

## Decisões do Enio (2026-07-10) — RESOLVIDAS

1. **#11:** ✅ taxa de secagem = propriedade de **canvas/sessão** (`PaintState.dry_rate_per_s`). `∞`
   fora do v1 (slider 2–60 s).
2. **#13:** ✅ **NÃO** encerrar a sessão nem aplicar aos traços já pintados ao mudar config no painel
   → fix por-DONO (captura do substrato no `WetStrokeStyle`), preservando a fusão. (Session-break
   REJEITADO.)
3. **#16:** ✅ aprovação CONDICIONAL: se o traço-3D nativo (height+lighting) for **mais leve** e der
   o aspecto 3D do Procreate → implementar e **inativar o Per-Layer Color**. Confirmar a técnica por
   fonte antes (o que o Procreate faz de fato). #15 fica MOOT se #16 substituir.

## Riscos / gates transversais

- **LOC caps:** `paint_watercolor.rs`, `populate.rs`, `event.rs`, `watercolor_settings.rs` — medir
  com `fmt` ANTES; card/rows/route novos podem estourar → sibling module, nunca allowlist
  (`feedback_loc_cap_split_not_allowlist`).
- **Arrays de id fixos** (`[NodeId; N]`): bumpar N ao adicionar id (erro de compilação protege).
- **Seam test** (`seam.rs:427/454`) itera os arrays → cobre id novo e falha sem o forward/route
  (alvo irrefutável — não confiar em "compilou").
- **HR-15:** labels "Dry"/"Wet"/"Drying Time" em inglês (memory `app_ui_english_only`); zero hex
  (ColorToken); zero `→` em string literal (gate `no_tofu_glyphs`).
- **Byte-identity:** todo campo novo default = comportamento atual; todo fix novo entra com RED
  provado na árvore (DIRETIVA).
- **cwd reseta pro main a cada turno** — mutação SEMPRE por caminho absoluto do worktree
  (`feedback_sed_relative_path_hits_primary_cwd`).
