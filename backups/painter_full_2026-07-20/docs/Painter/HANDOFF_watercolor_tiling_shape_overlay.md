# HANDOFF — Watercolor Tiling: shape overlay/edit na costura + fila pendente

> Escrito 2026-07-11 (Enio pediu handoff após o problema do overlay de forma no tiling persistir).
> **Modo L** (worktree isolado `Worktrees/line-Painter`). **TUDO É COMMIT LOCAL — nada pushado/integrado.**
> Base de integração = `1c7c9a22` (origin/main). Não integre nem faça ship sem ordem EXPLÍCITA do Enio
> (CLAUDE.md §0.7; memória `feedback_integration_only_enio_command_end_of_all_lines`). Você fecha, entrega
> este handoff e PARA.

---

## 0. Estado da linha (commits locais desta jornada, do mais novo pro mais velho)

> **INTEGRADO ao `main` em 2026-07-11** — todos os commits abaixo estão no main, e **TODO o smoke foi
> aprovado pelo Enio** (o último pendente, o default de Size do Paper, foi aprovado pós-integração).
> Nada aqui está em aberto. A fila que RESTA está no §2.

```
e3ff4f27 feat(watercolor): Paper procedural default de Size fino (~21px, não blobs de 256) ← ✅ smoke OK
<novo> test(watercolor): guard all-kinds Paper==Grain params (fecha a classe do bug)
<novo> fix(watercolor): Paper reseta params ao trocar kind (Voronoi casa Grain)
<novo> fix(watercolor): grain casa a ESCALA do brush (ViewPlane→canvas por radius)
<novo> feat(watercolor): Dots/Scales tilam seamless via hash-wrap (#2 Fase 2 completa)
<novo> feat(watercolor): patterns analíticos tilam seamless sob Tiling (#2 Fase 2)
<novo> feat(watercolor): Smudge (TRUE SMEAR) wrapa na costura do Tiling (#2 fu-a)
d9df426f fix(watercolor): overlay some além da costura — tiles do Repeat Image cobriam o chrome (resolve §1)
4fa05563 feat(watercolor): overlay editável CONTÍNUO (uma cópia), sem partir nos tiles
49216a01 feat(watercolor): edit-in-tile multi-shape — badges tiladas + grab de qualquer forma
27bcd421 feat(watercolor): overlay 3×3 nos tiles p/ Ellipse/Polygon/Line (paridade visual)
1633274a fix(watercolor): edit-in-tile por offset FIXO no gesto (corrige 2 bugs)
daa7e864 feat(watercolor): forma editável nos tiles embrulhados (Free Hand/Curve)
1d97b3f5 fix(watercolor): formas dinâmicas atravessam a barreira do tiling
7f6079e0 feat(watercolor): wash molhado re-renderiza ao vivo ao mudar textura (todos os tiles)
bbc174fa docs(watercolor): #2c lattice any-size seamless no tracker da fila
77671244 style(watercolor): fmt/clippy skew nos testes #3/#4
356ee6bf feat(watercolor): #2c lattice procedurals tile seamlessly at any size
6384aca2 feat(watercolor): #2b Paper presets (Cold/Rough/Hot) seamless com Tiling
210bcb3c feat(watercolor): #2(b) texturas de SLOT imagem seamless com Tiling
0f3287ed feat(watercolor): #4 esconde o Brush Blend dropdown em modo aquarela
```

**Gates rodados por commit:** `cargo check -p`, `cargo test -p ph2d-tool-painter/-brush --lib`, `rustfmt
--edition 2024` (pin 1.95), `clippy -p`. **NÃO rodei o ship completo** (fmt-all/machete/deny/typos/
nextest-impacted escapam — o integrador drena latentes, orce 2-4 iterações; memória
`project_integrator_ship_catches_latents_budget_iterations`).

### O que LANDOU e está SÓLIDO (não mexer sem motivo)
1. **#2c lattice any-size** (`356ee6bf`): família value-noise (Noise/Clouds/DistortedNoise/Musgrave/
   Stucci/Grain + Voronoi) tila contínuo em qualquer Size. Foundational em
   `crates/ph2d-painter-brush/src/texture/patterns.rs`: `hash2w`/`wrapi` + `value_noise_t`/`fbm_g_t`/
   `warp_uv_t` (período em células; octave escala por freq potência-de-2) + `sample_kind_t(period)` +
   `lattice_tileable()`; `sample_kind` delega `[0,0]` = byte-idêntico. `sample_tiled_rot_wrapped` snapa
   span→P inteiro, **gated a SEM rotação**. Splits LOC: `patterns/math.rs` + `texture/tiled.rs`. Testes:
   `slot_lattice_tiles_seamlessly_under_tiling`, `lattice_wrap_is_byte_identical`.
2. **Wash molhado re-renderiza ao vivo ao mudar textura** (`7f6079e0`): o ÚLTIMO wash fica re-renderável
   enquanto molhado (até próximo traço / Dry). `wet_editable_{base,backdrop,region,tex}` em `PaintState`;
   captura no `paint_end`; `rerender_editable_wash` + `clear_wet_editable` em `watercolor_backdrop.rs`;
   hook por-frame em `paint_tick` (compara `brush.texture`/`paper` via PartialEq). Teste:
   `watercolor_texture_size_rerenders_the_wet_wash_and_all_tiles`.
3. **Formas atravessam a costura do WASH** (`1d97b3f5`): `stamp_drag_preview_watercolor` replica os dabs
   (`tiling::tiled_dabs`) + footprint full-axis no eixo com tiling. Teste:
   `watercolor_shape_wash_crosses_the_tiling_seam`.
4. **Edit-in-tile por offset FIXO** (`1633274a` + `49216a01`): o wrap do ponteiro pra editar formas dos
   tiles. **ISSO É BOM E DEVE FICAR** (independe do overlay). Ver §1.

---

## 1. ~~PROBLEMA ABERTO~~ **RESOLVIDO 2026-07-11 (`d9df426f`) — ✅ SMOKE APROVADO + integrado ao main** — overlay na costura do tiling

> **DIAGNÓSTICO FECHADO (a pergunta-chave abaixo foi respondida):** a geometria **CRUZA a borda**
> (x > iw) — já era provado pelo teste verde `shape_in_sprite_grab_drags_past_the_seam_without_wrapping`
> (âncora em x=80 com iw=64) e o freehand empurra pontos crus — e o overlay contínuo (`4fa05563`)
> desenhava certo, un-clipped. O que "parava na borda" **não era clamp nem clip: era Z-ORDER** —
> `draw_repeat_image` rodava **DEPOIS** de `draw_overlays` na **MESMA** `vector_scene`
> (`painter_bridge.rs`, dispatch), então os 8 blits full-canvas **opacos** dos tiles vizinhos pintavam
> por cima de todo chrome além da borda (overlay do editor, brush ring, marching ants). Dentro da
> sprite o conteúdo real está no pipeline (SOB a cena vetorial) → overlay visível; fora, tile por cima
> → "cortado na borda". **Retro-explica o "editor partido" das cópias 3×3** (tentativa 1): as cópias
> também eram desenhadas antes dos tiles, então só sobrevivia o pedaço da cópia ±iw que caía DENTRO da
> célula central.
>
> **Fix (`d9df426f`):** `draw_repeat_image` desenha **PRIMEIRO** (tiles = conteúdo de canvas; chrome
> por cima) — ordem agora: repeat tiles → selection overlay → draw_overlays. Bônus: brush ring e
> marching ants voltam a aparecer sobre os tiles vizinhos (a hit-region 3×3 já era pintável).
> **Asserção-vermelha:** gate novo `shells/desktop/tests/repeat_image_tiles_draw_under_the_editing_chrome.rs`
> (lê o fonte do dispatch; reordenar de volta = RED). Gates rodados: check/clippy `-p ph2d-host-desktop`
> verdes, LOC caps verdes, 530 lib-tests do tool + brush verdes.
>
> **Smoke pendente (Enio):** aquarela + Tiling X + Repeat Image, Free Hand cruzando a borda direita →
> o overlay (caixa + spine + handles) deve seguir CONTÍNUO por cima do tile vizinho; brush ring visível
> sobre os tiles. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && cargo run -p ph2d-host-desktop`
>
> O texto original do problema segue abaixo (histórico do diagnóstico).

### Sintoma (Enio, smoke 2026-07-11)
Free Hand (e Line/Ellipse/Polygon) em aquarela + **Tiling** + **Repeat Image**, forma desenhada
**cruzando a fronteira da sprite**: o WASH tila lindo (contínuo), mas o **overlay editável** (caixa do
gizmo + spine + handles) **para na borda da sprite** — não acompanha a forma além dela. A última foto do
Enio: a caixa verde e o spine cobrem só a metade ESQUERDA da forma (até a linha vertical = borda direita
do sprite); a metade direita do wash não tem overlay. "editor ativo é partido, não tem continuidade."

### A tensão de FUNDO (o que torna isso difícil — leia antes de tentar)
- O **wash é RASTER** e tila via **Repeat Image**, que desenha o conteúdo **JÁ EMBRULHADO** da sprite
  (toroidal) 3×3. Uma forma cruzando a borda aparece contínua em cada tile porque o raster embrulha.
- O **overlay é GEOMETRIA VETORIAL** (caixa/spine/handles). Vetor **não embrulha toroidalmente de graça**.
- Os dois desejos do Enio — overlay **contínuo** E **visível/editável em todos os tiles** — **conflitam**
  pra um overlay vetorial. Cada tentativa bateu num lado do conflito.

### O que já foi tentado (e por que cada um falhou)
1. **Cópias 3×3 do overlay** (`daa7e864` curve, `27bcd421` ellipse/polygon/line, `49216a01` op-badges):
   desenhar o overlay inteiro em cada offset `±iw/±ih` (helper `overlay_tile_offsets`). **Falhou:** quando
   a forma cruza a borda, as cópias PARTEM (offset 0 mostra um lado, `-iw` mostra o outro; não embrulham
   como o raster). "editor partido."
2. **Overlay único contínuo** (`4fa05563`, ATUAL): reverti pras cópias, desenho UMA vez na geometria,
   sem clip (deveria estender além da sprite). Helper `overlay_tile_offsets` → `[(0,0)]` (o "switch"
   único); curve espelha inline. **Falhou também:** a foto mostra o overlay AINDA parando na borda.

### DIAGNÓSTICO QUE FALTA (faça ISTO primeiro — não chute, foi o que me derrubou)
A pergunta-chave **não resolvida**: a geometria da forma **cruza a borda** (pontos com x > iw) ou está
**dentro de [0, iw]** e o que aparece do outro lado é a **cópia embrulhada** do wash?
- Já verifiquei (leitura estática) que `curve_move` empurra o ponto **CRU não-clampado**
  (`crates/ph2d-tool-painter/src/tool/paint/curve.rs:~186`), que o Move não é gateado (só o Down, em
  `shells/desktop/src/input_dispatch/painter_canvas_input.rs:~367`), que o overlay não tem clip, e que o
  viewport do render é a janela inteira (`render_loop/mod.rs:~863`). **Isso diz que o overlay DEVERIA
  desenhar além da borda** — mas a foto contradiz. Falta observar em runtime.
- **Instrumente** (funcionou muito bem nesta sessão — env-gated `eprintln`): logue no `draw_curve_overlay`
  (shell) os `overlay.points`/`overlay.spine` (min/max x,y) e o `iw`; logue no `curve_move` (tool) o
  `pos` recebido e o range dos `model.points`. Rode `env PH2D_WC_LOG=1 cargo run -p ph2d-host-desktop`,
  desenhe cruzando a borda, cole as linhas. Aí você SABE se os pontos passam de `iw` ou não.

### Caminhos candidatos (escolha DEPOIS do diagnóstico; o Enio já opinou uma vez)
- **Se a geometria está DENTRO de [0,iw]** (wash embrulha as cópias): o overlay contínuo está correto na
  geometria; o "outro lado" é cópia. Pra ter overlay nas cópias **sem partir**, só há o caminho
  **TOROIDAL**: embrulhar o overlay vetorial (split do spine/bézier na fronteira + dobrar os handles em
  mod iw/ih), desenhando o resultado embrulhado. **Complexo** (split de bézier na borda). O Enio já
  marcou isso como "opção futura" no switch `overlay_tile_offsets`.
- **Se a geometria CRUZA a borda** (x > iw): então algo está **clampando/clipando** o overlay OU a
  geometria — ache e conserte (deveria estender além, un-clipped). Mais simples. Suspeitos: o
  `transform_gizmo.bbox` (`curve_gizmo::inflated_bbox`) pode estar limitado; ou o Down-gate impede
  **editar/desenhar** além da borda (aí a geometria nunca passa de iw). O Enio ofereceu antes "faz
  aparecer fora das fronteiras da sprite original" — se a geometria cruza, esse é o alvo.
- **Alternativa de UX** (se toroidal for caro demais): **manter a geometria SEMPRE dentro de [0,iw]**
  (embrulhar os pontos no commit, toroidal) → o overlay fica sempre coeso dentro da sprite e o wash tila.
  Custo: uma forma desenhada cruzando dobra pra dentro (o traço "pula" na costura ao desenhar).

### O que MANTER independente da decisão
- **`shape_edit_wrap` / `shape_edit_tile_offset`** (`stroke_multi.rs`, `1633274a`+`49216a01`): o offset
  FIXO por-gesto que faz um grab/drag numa cópia de tile editar a forma ORIGINAL (multi-shape aware:
  considera a ativa + todas as parqueadas). Corrigiu 2 bugs reais (salto do Ellipse/Polygon ao cruzar a
  borda; "cria outra curva" do Free Hand). Testes: `shape_editable_from_a_wrapped_tile_under_tiling`,
  `shape_in_sprite_grab_drags_past_the_seam_without_wrapping`. **Não reverta.**
- `CurveEditor::is_drawing_freehand()` (`curve.rs`) — suprime o wrap durante o DESENHO do Free Hand (a
  captura crua fica contínua, sem pular na costura).

### Arquivos-chave do overlay
- `shells/desktop/src/render_loop/painter_bridge_overlays.rs` — helper `overlay_tile_offsets` (o SWITCH,
  hoje `[(0,0)]`) + draws de ellipse/polygon (ainda em loop sobre o helper, iteram 1×).
- `shells/desktop/src/render_loop/painter_bridge_curve_overlay.rs` — curve/free-hand (desenho único).
- `shells/desktop/src/render_loop/painter_bridge_line_overlay.rs` — line (loop sobre o helper).
- `shells/desktop/src/render_loop/painter_bridge_op_badges.rs` — badges das formas parqueadas.
- `crates/ph2d-tool-painter/src/tool/paint/stroke_multi.rs` — `route_shape_pointer_multi` (aplica o
  `shape_edit_wrap`), `shape_edit_tile_offset`, `shape_state_bbox`, `on_active_centre_square`.

---

## 2. FILA PENDENTE (doc 13 `13_fila_integracao_watercolor_secoes.md` — itens ABERTOS)

Os LANDOU estão marcados ✅ no doc; abaixo só o que falta:

| # | Item | Nota / onde |
|---|---|---|
| **2 fu** | **Follow-ups do Tiling** | ~~(a) `smear_wet_base` (Smudge>0) sem wrap~~ **LANDOU 2026-07-11** (wrapa via `tiled_offsets_into` + lift toroidal; teste `watercolor_smudge_wraps_across_the_tiling_seam`, RED verificado). ~~patterns ANALÍTICOS (Fase 2)~~ **LANDOU 2026-07-11** — 10 kinds (Checker/Diamonds/Stripes/Grid/Crosshatch/Waves/Chevron/Weave/Bricks/Gradient) via `analytic_tile_period` + `snap_slot_size` per-eixo (ZERO mudança de sampler — são exatamente periódicos, só o span é alinhado). Excluídos (gate refutável): turbulência (Marble/Magic/Wood), irracional (Triangles/Hexagons). ~~Dots/Scales~~ **LANDARAM 2026-07-11** via hash-wrap (`dots_t`/`scales_t` embrulham o `hash2` no período da célula, gated por `analytic_needs_hash_wrap`; Scales exige `pv` par → snap garante). Teste `slot_analytic_pattern_tiles_seamlessly_under_tiling` (12 kinds), RED verificado. **RESTA:** caso **rotacionado** (todo grid — limitação fundamental); **papel procedural on-the-fly** ("Fase 3"). Ver o bloco #2 do doc 13. |
| **7** | **Shape Tone ramp / Per-Layer Color em aquarela** | ignorado; avaliar semântica (tone da silhueta?) no wash. |
| **12(c)** | **Secagem influenciando a MESCLA** | DEFERIDO. Gatear o rewet-lift pela umidade local foi tentado e REVERTIDO: o mapa de umidade vem de `stroke_coverage` (pigmento) → água limpa subrepresenta molhabilidade → quebra `clean_water_backrun`. **Precisa de um sinal de "molhabilidade" separado da cobertura de pigmento.** Fica pra quando o modelo de água for revisitado. |
| **14** | **INVESTIGAÇÃO: retângulos do Per-Layer Color no brush COMUM** | bug aberto; handoff `docs/HANDOFF_per_layer_color_perf_artifacts.md`. Aplicar o MÉTODO do BUGS #8 (bissecção + perfil + sondas). |
| **15** | **INVESTIGAÇÃO: perf do Per-Layer Color** (otimizações da aquarela não aplicadas) | lento; handoff aberto. Checklist BUGS #7 + stamp-cache + ADR-0109. |
| **16** | **PESQUISA: traço de aspecto 3D** (Procreate/Rebelle/Painter) | design; height-map + lighting pass como alternativa barata ao Per-Layer Color. |

**Perf/cor (outra dimensão, não-UI):** waves W-A..W-D da auditoria em
[`12_aquarela_auditoria_pos_f123_padrao_ouro.md`](12_aquarela_auditoria_pos_f123_padrao_ouro.md).

**Não re-expor sem novo smoke** (cercas de Chesterton): o **Blur do Wet Mix** (pickup fixo em r×0,5) e o
**Paper Colors ramp** (revertido 2026-07-06, papel volta ao grayscale — memória
`project_aquarela_paper_ramp_broken`).

---

## 3. Como continuar (Modo L)

1. **Ambiente:** trabalhe SEMPRE em `Worktrees/line-Painter` (prefixe `cd` absoluto em TODO comando Bash —
   sem isso o cwd cai no repo MAIN e `PH2D/target` dá "Not a directory"). fmt = `rustup run 1.95 rustfmt
   --edition 2024 <arquivos>` (o crate é edition 2024, let-chains). O grep do Bash MANGLA alguns
   identificadores na saída — confie no Read pra nomes reais.
2. **Inner loop:** só `cargo check -p <crate>`; teste/clippy/fmt 1× no fechamento.
3. **Fechamento:** rode `cargo test -p ph2d-tool-painter -p ph2d-painter-brush --lib`, clippy `-p`,
   fmt, e o gate de LOC (`cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap`).
   Commit LOCAL (`git commit --no-verify -m ... -- <paths>`). **PARE.** Ship/integração = ordem do Enio.
4. **Doc 13** é o tracker vivo — atualize o item quando fechar. Este handoff cobre o overlay + a fila.
