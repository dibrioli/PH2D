# Flip — Plano de implementação (waves + tasks)

> **Decisão:** [ADR-0114](../architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md) ·
> **Visão/nomes/reference:** [`00_README.md`](00_README.md) ·
> **Algoritmos Blender 5.2 (consultar SEMPRE):** [`02_referencia_algoritmos_blender_5.2.md`](02_referencia_algoritmos_blender_5.2.md) ·
> **O traço (bug aberto + fix):** [`03_traco_rasterizacao.md`](03_traco_rasterizacao.md) ·
> **Estado da arte além do Blender:** [`04_alem_do_blender.md`](04_alem_do_blender.md).
>
> **Estado (2026-07-19 — reconciliado contra o código; ver a tabela "O que landou" abaixo).**
> Tudo de WT..W7 **+ Edit Mode + Colorize C1 (Trap) + Região por curvas** está **integrado ao
> main**, com os smokes **APROVADOS pelo Enio**. O Flip é um app de ANIMAÇÃO **usável ponta-a-ponta**:
> desenhar → frames/hold/ciclos/ghost → tween → balde → reshape → **editar/selecionar/transformar** →
> **instância/pose de quadro** → **multiframe** → salvar. Docs por wave: WT [`03`](03_traco_rasterizacao.md) ·
> W3 [`05`](05_frames_ghost_tween.md) · W4 [`06`](06_fill_balde.md) · W5 [`07`](07_reshape_escultura.md) ·
> W6 Edit Mode [`08`](08_edit_mode_selecao.md) · Colorize [`09`](09_colorize.md) · Região por curvas [`10`](10_regiao_por_curvas.md).
> **A wave Colorize FECHOU as três fatias, todas smokadas** (C1 Trap · C2 LazyBrush ·
> C3 onion fill — [`09_colorize.md`](09_colorize.md) §7). A integração
> com a **timeline principal** segue **ADIADA** (Enio 2026-07-12) e o **export** é deferido de
> propósito. Fila e backlog verificado:
> [`../HANDOFF_line_FLIP_CONTINUACAO_2026-07-19.md`](../HANDOFF_line_FLIP_CONTINUACAO_2026-07-19.md) §3.
>
> **A wave COLORIZE integrou ao main em 2026-07-21.** A jornada seguinte (2026-07-22) é o
> **Tween v2** — doc [`11`](11_tween_v2.md), **construída, gateada e SMOKE APROVADO pelo Enio
> (2026-07-22)** (`PH2D_FLIP_TWEEN_SMOKE=1` — o boneco de palito: braço mantém o comprimento no
> arco, tronco casado não desliza, chapéu órfão desvanece só com Fade); handoff de integração:
> [`../HANDOFF_line_FLIP_INTEGRACAO_tween_v2_2026-07-22.md`](../HANDOFF_line_FLIP_INTEGRACAO_tween_v2_2026-07-22.md).

## O que landou DESDE o snapshot de 2026-07-12 (não reconstruir — tem doc + código)

> Esta seção existe porque a versão anterior deste plano parou em 2026-07-12 e **listava como
> ABERTOS itens que já estão no código** — o modo de falha que faz a próxima LLM reconstruir o que
> existe (o módulo de áudio já pagou por isso). Cada linha diz onde o código VIVE.

| wave | o que é | landou | onde no código |
|---|---|---|---|
| **W5 (costura)** | modo **Sculpt** plugado no shell (tool + painel modal + gesto) | 2026-07-13 `930f4c9b` | `flip_reshape.rs`, `flip_smooth.rs` |
| **W6 — Edit Mode** | selecionar traço/**ponto**/**segmento** + realce + **transformar (girar/escalar)** a seleção; aposenta o "alvo vivo" | 2026-07-13 | doc [`08`](08_edit_mode_selecao.md); `flip_select*.rs`, `flip_selection_gizmo.rs` (518 LOC), `flip_transform.rs`, `flip_edit_gesture.rs` |
| **W7 — Multiframe** | **multi-seleção de chaves na tira** (`strip.selected_keys()`) · **Instance** / linked-duplicate (`FLIP_KEY_INSTANCE` + marcador `INSTANCE_DOT`) · **pose de quadro** · **régua de scrub** · **falloff temporal** visível na tira · modo **`Selected`** dos fantasmas | 2026-07-13..15 (W7.1–W7.4) | `flip_multiframe.rs`, `flip_strip.rs`, `flip_pose_gizmo.rs`; `onion.rs` (`OnionMode::Selected`) |
| **Colorize C1 — Trap** | *trapped-ball*: o balde para de vazar por vão estreito (chip `FLIP_TRAP`) | 2026-07-18 `a6e8277a` | **em `ph2d-flip-fill`** (`ball.rs`/`edt.rs`, `Trap`), **não** na crate `ph2d-flip-colorize` que o [`09`](09_colorize.md) previa |
| **Região por curvas** | a malha do fill nasce dos vértices das linhas; o donut (buraco = componente conexa); R3 revogada por medição | 2026-07-18 | doc [`10`](10_regiao_por_curvas.md) §11–§12; BUGS #19–#22; `flip_fill_curve_route.*`, `flip_fill_dilate.*` |
| **Tween v2** | a correspondência de traços deixa de ser ordinal (custo + atribuição ótima) e o inbetween percorre o **arco** em vez da corda (espiral logarítmica) — mais o `Ease`/`Fade` da barra, que fecham a T3.7 | 2026-07-22 | doc [`11`](11_tween_v2.md); `tween_match.rs`, `tween_spiral.rs`, `tween.rs`; smoke `PH2D_FLIP_TWEEN_SMOKE=1` |
| **Tween v2 — correção de pares** | a lição CACAni: o toggle **Pairs** abre um overlay da correspondência e o clique re-pareia; o Add commita corrigido | 2026-07-22 | doc [`11 §8`](11_tween_v2.md); `flip_tween_correct.rs`, `tween_match.rs::repair`; smoke `PH2D_FLIP_TWEEN_PAIRS_SMOKE=1` |
| **Tween v2 — fase da costura** | dois anéis fechados com o ponto 0 em lugares diferentes deixam de torcer no meio: correlação circular sobre a **virada** alinha o pareamento antes da espiral | 2026-07-22 | doc [`11 §9`](11_tween_v2.md); `tween_phase.rs`; smoke `PH2D_FLIP_TWEEN_PHASE_SMOKE=1` |

## Regras permanentes (valem em TODA task)

1. **Consulte o Blender 5.2 antes de cada tópico** (`~/Downloads/blender-5.2-grease-pencil-ref/`;
   índice no README; versão pinada = recorte 5.2, exceções anotadas no `02`). Leia o algoritmo no
   `02`, vá ao fonte, **reimplemente do zero** — clean-room, nunca copie código GPL.
2. **Padrão-ouro sem custo** (§0.6 do CLAUDE.md): a melhor opção técnica vence cronograma. Gaps
   in-scope fecham na wave.
3. **Isolamento (Modo L):** worktree própria; foundational que você criar projete pra isolamento;
   anote ids/consts novos no handoff. **NÃO integre nem pushe** — feche, handoff (DIRETRIZ §1.5.9), PARE.
4. **UI canônica:** zero hex, zero f32-literal, zero string hardcoded — tokens + i18n; labels em
   **inglês** (memória `feedback_app_ui_english_only`).
5. **Widgets pela Widget Gallery**; painel = idioma do Inspector/Painter.
6. **LOC cap 700/arquivo (crates), 600 (shell), 200/fn** — transborde = módulo-irmão; `fmt` antes de medir.
7. **Inner loop = `cargo check -p`**; teste/clippy/gate 1× no fechamento da wave.
8. **Ready-to-smoke:** toda feature nasce com exemplo auto-play no doc demo.
9. **HR-5 (determinismo):** transcendentais só onde o gate permite; preferir formas polinomiais
   (binomial > gaussiana-exp; espiral-log = 1 sincos POR STROKE, nunca por vértice).
10. **Oráculo modela a APARÊNCIA, não a implementação** (memória
    `feedback_oracle_must_model_appearance_not_implementation`): o expected de todo teste visual
    deriva da definição do objeto (união, blend canônico), roda VERMELHO antes do fix, e as
    mutações têm de sangrar.

## Decisões CRAVADAS (não re-litigar; contexto nos docs citados)

| Decisão | Onde está o porquê |
|---|---|
| Espessura do brush = **px de TELA** (absoluta; `fold_model × mean_scale` no gizmo) | Enio 2026-07-11; HANDOFF_flip_impl |
| Espaço de cor: rasteriza premult/linear/16F → resolve → compositor 8-bit sRGB do Painter | W1, ratificado (blend byte-idêntico ao Painter) |
| Undo = fila global `ProjectState` (sem undo próprio); **Arc-CoW nos drawings antes de dup/hold em escala** | 02 §1/§8; lição GPv3 (undo 6.6×) |
| Traço = **UNIÃO GLOBAL da polilinha** num passe: janela `p0`/`p3` + **vizinhos geométricos** (broadphase no pack) + `capsule_dn` única + clamp/fade sub-pixel. Escalada de 2 passes NÃO foi necessária | 03 §4 |
| Auto-cruzamento: a COBERTURA é união (sem mordida); a **cor** segue first-wins (GP default). Acúmulo de tinta = flag *Self Overlap* futura | 03 §4.2/§6/§8 |
| AA: cobertura analítica `0.5+(1-dn)/fwidth` + par clamp(1.3px)/fade sub-pixel; acúmulo Halton no export; SMAA só do reference MIT (futuro) | 03 §7 |
| Onion: default RELATIVE (por-desenho), verde/azul do GP como **tokens**, fade 1/Δ piso 0.1, **some no play** | 02 §8; 04 §4 |
| Tween W3 = GP literal (índice+padding+auto-flip) com fator POR CAMADA; v2 = matching+espiral (04 §2) | 02 §3; 04 §2 |
| Fill W4 = pixel solver do GP + fechamentos PERSISTENTES (Harmony) + Paint/Unpainted/Unpaint + Grow/Shrink; Delaunay = v2 | 02 §6; 04 §3 |
| Fill lê camada(s) de referência (linha) — contrato desde o W4 | 04 §4 (workflow linha/cor) |
| Ciclos = pre/post behavior por camada (None/Loop/PingPong/Hold) no sampler do playhead | 04 §4 |
| Sem sistema de materiais: cor/gradiente = por-stroke; modo/placement/randomização/texturas = brush preset | 02 §9 |
| Multiframe editing: **LANDOU** (W7) — seleção de chaves na tira, dedup por desenho, falloff temporal (`05 §8`) | 02 §11 |
| VFX, modifiers, lineart, armature/vertex-groups, bake, SVG/PDF export, trace de imagem: **fora de escopo** (VFX = referência adormecida no 02 §10) | não-objetivos |
| Budgets de perf (workstation, `--release`): traço vivo ≤ 1 ms/frame de pack+upload; playback 60 fps com ≥ 200k pontos visíveis + 4 ghosts; flip prev/next < 16 ms | WT/W3 benches |

---

# WT — O traço (FECHADA 2026-07-12 · smoke APROVADO)

**Objetivo (batido):** a cobertura do traço é a **união global da polilinha** — a mordida
morreu em todas as suas formas. Detalhe completo em
[`03_traco_rasterizacao.md`](03_traco_rasterizacao.md) §4-§6.

- [x] **WT.1 — Oráculo de APARÊNCIA.** `expected_alpha` deixou de modelar o first-wins e passou
      a modelar a união da polilinha. Provado VERMELHO no código antigo (4 testes, desvio ~250).
- [x] **WT.2 — Janela de sequência (`p0`/`p3`)** + **`capsule_dn` única** (o defeito D1 — raio
      por-ponto — era real; o teste do taper o pegou).
- [x] **WT.3 — Vizinhos GEOMÉTRICOS** *(não estava na spec)*: a janela ±1 não bastava — todo
      traço que volta sobre si mesmo tinha a mordida de longo alcance. Broadphase por grid no
      `pack` (`neighbors.rs`, cacheado por desenho) + loop no fragment. **União global, 1 passe,
      zero render passes extras.**
- [x] **WT.4 — `safe_dir`**: ponto duplicado fazia `normalize(0)` = NaN e RASGAVA o traço.
- [x] **WT.5 — Fade sub-pixel + clamp de largura mínima** (são um PAR) + **AA de cobertura
      correto** (a forma antiga subestimava traço fino em 10×).
- [x] **Gate:** 15 testes GPU + 18 unit + 2 composite verdes (debug e release), **5 mutações
      provadas**, fmt/clippy/LOC limpos, suite do shell verde. Perf: 1.7 ms para um traço real
      de 4000 pontos.
- [x] **WT.6 — Smoke do Enio APROVADO** (zigzag hardness alto/baixo, curvas densas, laço, linha
      fina com zoom out; kill-criteria K1-K4 no 03 §6). O traço FECHOU e todas as waves seguintes
      foram construídas sobre ele.

---

# W3 — Frames · Ghost Frames · Tween (FECHADA 2026-07-12 · smoke APROVADO)

**Entregue:** o modelo de tempo completo (vão/exposição/ciclos), os Ghost Frames como função
pura + passe de silhueta tingida, o autokey POR TOOL (a borracha sempre duplica), o flip por
desenho, o tween com auto-flip, e a **tira** (`ph2d-panel-flip-frames`). Detalhe e gotchas:
[`05_frames_ghost_tween.md`](05_frames_ghost_tween.md).

**Objetivo (batido):** a tira de frames própria (não a timeline global), transporte com ciclos, Ghost
Frames e Tween. Nomes intuitivos (README §nomes). Referências: `02` §1 (frames/invariantes),
§3 (tween), §8 (onion EXATO, autokey, primitivas); `04` §2 (tween v2) e §4 (UX dos apps).

**Padrão-ouro:** Procreate Animation Assist / Callipeg (tira visual, onion por default,
add/duplicate/hold por gesto) + os aprendizados TVPaint/Harmony (04 §4: flip como inner loop,
células de exposição, pre/post behavior).

- [x] **T3.1 — Tira de frames.** Célula por keyframe (nº de exposições visível na célula, à
      TVPaint), quadro atual destacado, Add/Duplicate/Delete/Reorder, drag de **Hold**.
      Respeitar os invariantes do mapa (02 §1 — tabela; delete-vira-sentinel, duplicate
      transacional). Aceite: manipular quadros pela tira; testes tabelados dos invariantes.
- [x] **T3.2 — Transporte + ciclos.** play/pause/FPS + **pre/post behavior por camada**
      (None/Loop/PingPong/Hold) como wrap-mode do sampler (04 §4). Aceite: loop e pingpong
      reproduzem sem duplicar frames.
- [x] **T3.3 — Ghost Frames.** Port EXATO do `get_frame_id` (02 §8: RELATIVE default,
      ABSOLUTE opcional, filtro por tipo, before-first Δ++, wrap SHOW_LOOP corrigido p/
      `first..last`) como **função pura testável** no `ph2d-flip`; tint = silhueta 100%
      recolorida (tokens `FlipGhostBefore`/`After` verde/azul GP) + alpha `1/|Δ|` piso 0.1;
      1 draw + 2 uniforms por ghost no passe existente; **some no play**; flag por camada.
      Aceite: goldens dos 3 modos + smoke com 2/2 ghosts.
- [x] **T3.4 — Autokey por-tool** (02 §5): desenhar sem chave no frame → cria em branco (ou
      duplicata com "Additive"); **borracha/reshape → SEMPRE duplicata**. Aceite: teste do
      trio needs_new + smoke.
- [x] **T3.5 — Flip de desenho (atalhos).** Prev/next **por DESENHO** (pula holds; F/G do
      Harmony) + por frame; cels vizinhas residentes na GPU (latência zero — é o inner loop
      do animador, 04 §4). Aceite: flip instantâneo em doc com holds.
- [x] **T3.6 — Tween (dados).** GP literal (02 §3): pareamento por índice/seleção + auto-flip
      (cruzamento + desempate 15°) + `sample_curve_padded` (padding ao MAX preservando pontos
      originais — NÃO reamostragem uniforme) + lerp não-clampado com fator POR CAMADA + easing
      (`ph2d-anim::Interp`) + BREAKDOWN kind + `exclude_breakdowns` (re-tween idempotente).
      Fills POR FATIA (bug do original — 02 §3). Aceite: testes tabelados (extremos
      pixel-idênticos em t=0/1; auto-flip nos 3 ramos; órfãos estáveis).
- [x] **T3.7 — Tween (UI).** Caixa de contagem + botão **Add Tween** na tira: gera N inbetweens
      entre a chave atual e a seguinte, auto-flip ligado, fator por posição absoluta.
      **FECHADA em 2026-07-22 (Tween v2, doc [`11`](11_tween_v2.md) §5):** o chip **Ease**
      (Linear/In/Out/In-Out, nos rótulos da timeline) e o toggle **Fade** entraram na barra,
      pela porta única `FlipStrip::tween_options()`. A família do easing é fixa em `Quad` de
      propósito — o picker completo já existe no menu de curvas da timeline.
- [ ] **T3.8 — Cache de playback.** **NÃO foi feito, e por escolha:** a tesselação já é cacheada
      por DESENHO (T1.8) e é ela o custo real da troca de quadro; o ring de texturas COMPOSTAS só
      compensa se o composite virar o gargalo — e isso se **mede** antes (memória
      `feedback_measure_perf_symptom_scale`). Fica como carry-over COM bench: ring keyed por
      (frame, escala), invalidação por (camada, desenho) sujo, drop de frame no relógio (04 §4).
- [ ] **T3.9 — Marcadores fixos (light table)** — carry-over explícito (o passe de ghost já
      aceita a lista; falta a UI de marcar).

**Carry-overs da W3:** **multi-seleção de chaves — LANDOU (W7.3)**: destravou o modo `Selected` dos
fantasmas (`onion.rs` · `strip.selected_keys()`). **AINDA ABERTO:** drag de célula/borda na tira
(mover chave e esticar hold por arrasto — hoje pelos botões ◀/▶ e pela caixa Hold) · **light table**
(T3.9 — o passe de ghost já aceita a lista; falta a UI).

**Gate W3:** smoke — 2 desenhos-chave, ghosts ligados, Add Tween, play com loop; goldens do
onion; bench do cache.

---

# W4 — Fill (balde) — FECHADA 2026-07-12 · smoke APROVADO (âncora 2026-07-12; Região por curvas / BUGS #22 2026-07-18)

**Objetivo:** balde robusto para line-art com Gap Closure interativo, resultado = GEOMETRIA.
Referências: `02` §6 (pipeline exato + constantes) e `04` §3 (upgrades decididos).

- [x] **T4.1 — Fill como geometria.** `fill_id` por-stroke (fills com buracos multi-curva,
      02 §1) + `hide_stroke`; render no passe de fill existente (depth `sid+1` — 02 §2b).
      Aceite: fill de N curvas com furo renderiza.
- [x] **T4.2 — Pipeline raster.** Fit-to-bounds (margem 20px, Precision, mín 128², zoom ≤5×) →
      render offscreen com **`radius_scale = 0.5`** + threshold `r ≥ 1/255` → span fill com
      **leak filter cruzado 3px** → Moore trace (buracos = contornos separados) → **RDP ε≈1.25px
      + fit Schneider** (upgrade sobre o smooth 20× do GP) → stroke cíclico com fill. Buffer de
      flags dedicado `Vec<u8>`. Falha total ao tocar a borda (+ modo invert). Aceite: clicar
      dentro de forma fechada preenche; goldens do trace.
- [x] **T4.3 — Gap Closure.** Extend (pontas + **quinas mid-stroke por curvatura**) com corte
      por colisão (2 passes, 3 exclusões) + Radius (círculos-guia SÓ nos gaps pendentes;
      linhas centro-a-centro); ajuste modal ao vivo (scroll). **Fechamento bem-sucedido vira
      stroke INVISÍVEL persistente** (twist do Harmony — o re-fill sobrevive). Aceite: fechar
      forma com abertura; helpers visuais; re-fill de frame vizinho reaproveita.
- [x] **T4.4 — Semântica de balde de animação.** Modos **Paint / Paint Unpainted
      (paint-behind) / Unpaint** + **Grow/Shrink** por offset CAD do polígono (+2px default) +
      **Precision**. Fill lê camada(s) de referência (linha) — o contrato linha/cor. Aceite:
      colorir sem tocar a linha; grow mata o halo do AA.
- [x] **T4.5 — Fill multiframe — LANDOU (W7):** a multi-seleção de chaves na tira existe e o balde
      a consome via `flip_multiframe::targets(…, strip.selected_keys(), …)` (`flip_fill.rs:405`).

**Carry-overs da W4:** **fill multiframe — LANDOU (W7)** (`flip_fill.rs:405`) · **Colorize C1 (Trap)
— LANDOU** em `ph2d-flip-fill` (2026-07-18); **C2 (LazyBrush) + C3 (onion-fill) = a wave ABERTA**
([`09`](09_colorize.md) §7). **AINDA ABERTO:** ajuste modal ao vivo do Gap Closure (scroll + helpers
nos vãos pendentes — o `closures()` já devolve os segmentos, falta o overlay) · modo **Radius** do
Gap Closure — ⚠️ **candidato a APOSENTADORIA** (o Trap responde melhor à mesma pergunta; handoff §3.3).

**Gate W4:** smoke — line-art com gaps, preencher com preview dos helpers, Grow/Shrink,
paint-behind, multiframe.

---

# W5 — Reshape (escultura de traço) — FECHADA 2026-07-12 · costura Sculpt no shell 2026-07-13 (`930f4c9b`) · smoke APROVADO

**Objetivo:** remodelar traços com pincéis de raio+força+falloff. Referência: `02 §7`
(os 9 pincéis com TODAS as constantes — a W5 está lá, não aqui).

- [x] **T5.1 — Trait `ReshapeBrush`** (3 callbacks, 02 §7) + infra: influence =
      `alpha·pressure·falloff(dist, raio)·multi_frame_falloff` (falloff curve do Painter);
      invert por Ctrl; **auto-masking congelado no down** (seleção/camada ativa; threshold
      20px). Aceite: seam headless do pipeline de influence.
- [x] **T5.2 — Smooth** (binomial iterations=2, influence = mistura; projeta TODOS os pontos).
      Aceite: alisar traço trêmulo sem encolher pontas.
- [x] **T5.3 — Push + Grab** (push = delta·influence por sample; grab = máscara+pesos
      CONGELADOS no down, pressure=1). Aceite: os dois com a distinção de UX correta.
- [x] **T5.4 — Thickness + Strength** (aditivos: ±0.001 no raio [na NOSSA unidade: px de
      tela], ±0.125 com clamp na opacity). Aceite: engrossar/apagar gradual.
- [x] **T5.5 — Pinch + Twist** (pinch `inf²/25`; twist 1°·influence em tela). Aceite: ambos.
- [x] **T5.6 — Randomize** (hash splitmix64 por sample, perpendicular ao movimento)
      se apertar. **Clone = comando** (paste posicionado), não brush — fora da W5.
- [x] **T5.7 — Reshape multiframe — LANDOU (W7):** o reshape consome
      `flip_multiframe::targets(…, strip.selected_keys(), …)` (`flip_reshape.rs:145`); o falloff
      temporal está visível na tira (W7.4).

**Gate W5:** smoke — smooth, push, grab, engrossar; constantes com a "sensação GP".

---

# Timeline — integração com a timeline principal (ADIADA — Enio 2026-07-12)

> ⚠️ **Colisão de número resolvida:** o rótulo **"W6" foi reatribuído ao Edit Mode** (doc
> [`08`](08_edit_mode_selecao.md), landou 2026-07-13). Esta wave — plugar os frames do Flip na
> `ph2d-timeline` — segue **ADIADA** e **NÃO integrou** (nenhum binding flip↔dope-sheet no código; o
> Flip roda pela própria tira sobre o `Playhead` global). As tasks T6.1–T6.5 abaixo ficam **sem
> número fixo** até reabrir.
>
> **A timeline principal ainda está em desenvolvimento.** A integração espera ela ficar pronta;
> até lá a tira do W3 é a UI de tempo do Flip (e o playhead JÁ é o global — não há relógio a
> reconciliar quando a hora chegar).


**Objetivo:** plugar os frames do Flip na `ph2d-timeline`/dope-sheet/`Playhead` globais.
**Coordenar com o dono da timeline** (`PropKind` é enum fechado).

- [ ] **T6.1 — Bind frames↔timeline** (faixa/keys no dope-sheet; keyframe kinds → cores).
- [ ] **T6.2 — Playhead unificado** (o transport local do W3 vira atalho; scrub global dirige o Flip).
- [ ] **T6.3 — Autokey do Flip × autokey global** — reconciliar os DOIS toggles homônimos
      (decisão de UX explícita; hoje: autokey do Flip é por-tool, 02 §5).
- [ ] **T6.4 — Markers/loop** integrados.
- [ ] **T6.5 — Handoff de integração** (DIRETRIZ §1.5.9) e **PARAR**.

---

## Deferidos explícitos (backlog qualificado — cada um com spec pronta nos docs)

- **Traço:** flag *Self Overlap* · corner types por-ponto · pincel dots/squares (Ciallo-style)
  · pincel airbrush analítico · variante SDF da escalada (tudo: 03 §8).
- ~~**Tween v2:** matching espacial + espiral logarítmica~~ — **LANDOU 2026-07-22, SMOKE APROVADO**
  (doc [`11`](11_tween_v2.md)). ~~a **UI de correção de pares** (o overlay + o re-par manual —
  a lição CACAni)~~ — **LANDOU 2026-07-22, SMOKE APROVADO** (`PH2D_FLIP_TWEEN_PAIRS_SMOKE=1`;
  doc [`11 §8`](11_tween_v2.md)): toggle **Pairs** na barra → overlay da correspondência (linhas
  por confiança verde/vermelho/âmbar + anéis de órfão), clique re-pareia, o Add commita com o
  plano corrigido. ~~o **alinhamento de FASE da costura** em traço fechado~~ — **LANDOU
  2026-07-22, pendente de smoke** (`PH2D_FLIP_TWEEN_PHASE_SMOKE=1`; crate nova `tween_phase.rs`,
  correlação circular sobre a **virada** e não as posições — a espiral tira o rígido depois, então
  a fase tem de ser invariante à rotação; doc [`11 §9`](11_tween_v2.md)). **ABERTO de lá:** a
  torção em rotação grande (Sederberg 1992 / Alexa 2000 — a correspondência era o pré-requisito
  dos dois).
- **Colorize:** **C1 (Trap) LANDOU** em `ph2d-flip-fill` (2026-07-18). **C3 (onion fill)
  LANDOU 2026-07-21, smoke APROVADO** — com chaves marcadas na tira um Apply colore todas; o
  que ela acrescenta NÃO é o range (esse é do W7) e sim a **SEMENTE**: o rabisco é autorado em
  MUNDO sobre as poses empilhadas e semeia cada quadro, que resolve sozinho porque a linha se
  move ([`09 §5.2`](09_colorize.md)). **C2 LANDOU (2026-07-19,
  pendente smoke): MOTOR + FATIA** — o motor headless (crate `ph2d-flip-colorize`: `flow.rs` BK +
  `colorize()`; corte hugga a tinta, vão não vaza) **e** o **modo Colorize clicável no shell** (7º
  `FlipMode`: rabiscar → **Apply** → regiões, com **overlay ao vivo dos rabiscos**; smoke
  `PH2D_FLIP_COLORIZE_SMOKE=1`). ~~multiframe~~ · ~~Apply live~~ · ~~C3~~ — os três landaram.
  **ABERTO:** **pré-segmentação por regiões** (perf a 4K, [`09`](09_colorize.md) §7.1) ·
  ~~o re-Apply ao vivo viola o kill-criterion de 16 ms~~ — **FECHADO 2026-07-21**: medido em
  **304 ms/tique** (3 quadros da C3, escala do produto) e **1,45 s** com zoom; o split refutou
  o cache (solve 76%, raster 4%), então o corte saiu para um `Job` com **um em voo** e o undo
  passou a tratar recálculo pendente como gesto em andamento · o `trap_px`
  não sobrevive ao clamp de `MAX_SIDE`. ~~o `fill_at` do BALDE tem o mesmo buraco de quina~~ —
  **FECHADO 2026-07-21** ([BUGS #23](BUGS_flip.md)): era pior que no Colorize (o balde
  **RECUSAVA** a partir da precisão 80, com o toast mandando subir o Gap Closure sobre uma
  quina fechada na tela); a `weld.rs` mudou de casa para a `ph2d-flip-fill` e serve os dois
  consumidores por uma porta só. **Aberto de lá:** o `reach` do Gap Closure precisa de **4× o
  vão** para fechá-lo, e o slider é rotulado pelo alcance — medido, nomeado, não perseguido.
- **Ghost extras:** light table (marcadores fixos) + Shift & Trace (transform por ghost +
  F1/F2/F3) (04 §4).
- ~~**Edit Mode**~~ (seleção de traço/ponto/segmento + transform): **LANDOU (W6, 2026-07-13, doc
  [`08`](08_edit_mode_selecao.md))** — `flip_select*.rs` + `flip_selection_gizmo.rs` + `flip_transform.rs`.
- **2.5D multiplane** (parallax_factor por camada — ADR-0114 §Decisão 3).
- ~~**Instância de drawing na UI**~~: **LANDOU (W7.1)** — botão `FLIP_KEY_INSTANCE` + marcador
  `INSTANCE_DOT` na tira (linked duplicate).
- **Export/render com acúmulo** Halton+gaussiana (03 §7.3).
- **SMAA opcional** (reference MIT) p/ fills/composição (03 §7.4).
- **Congelar o contrato do `ph2d-flip`** (gate de superfície) quando o modelo assentar.

## Não-objetivos (declarados — não perguntar de novo)

VFX do GP (referência adormecida: 02 §10) · modifiers/geometry-nodes · lineart · armature/
vertex groups/rig (ADR-0114 §Gaps) · bake de animação · import/export SVG-PDF · trace de
imagem (potrace) · materiais como sistema (mapeamento: 02 §9) · viewport 3D (ADR-0114).

## Definition of Done (por wave)

1. Smoke real no app (a wave faz algo visível) — **e o oráculo/teste que prova a APARÊNCIA
   roda vermelho-antes/verde-depois** quando aplicável.
2. `cargo test -p <crates>` + arch-gates relevantes verdes.
3. LOC caps ok (fatiar antes de medir; `fmt` no pin).
4. Zero hex/f32-literal/string hardcoded; labels em inglês.
5. Ready-to-smoke atualizado no doc demo.
6. Commits locais; no fim da linha: handoff de integração (§1.5.9) e PARAR.
