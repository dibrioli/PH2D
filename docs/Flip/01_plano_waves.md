# Flip — Plano de implementação (waves + tasks)

> **Decisão:** [ADR-0114](../architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md) ·
> **Visão/nomes/reference:** [`00_README.md`](00_README.md) ·
> **Algoritmos Blender 5.2 (consultar SEMPRE):** [`02_referencia_algoritmos_blender_5.2.md`](02_referencia_algoritmos_blender_5.2.md) ·
> **O traço (bug aberto + fix):** [`03_traco_rasterizacao.md`](03_traco_rasterizacao.md) ·
> **Estado da arte além do Blender:** [`04_alem_do_blender.md`](04_alem_do_blender.md).
>
> **Estado (2026-07-12):** W0+W1+W2 **integrados ao main** · **WT (o traço) FECHADA** (smoke
> APROVADO pelo Enio) · **W3 (Frames · Ghost · Tween) FECHADA** — o Flip virou app de
> ANIMAÇÃO (tira com exposição, ciclos, Ghost Frames, autokey por-tool, flip por desenho,
> tween). Doc: [`05_frames_ghost_tween.md`](05_frames_ghost_tween.md). Pendente o smoke.
> **A wave atual é a W4 (Fill).** A **W6 (timeline global) está ADIADA** até a timeline
> principal ficar pronta (Enio 2026-07-12).

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
| Multiframe editing: modelo preparado (lista `(drawing, frame, falloff)`), UI deferida | 02 §11 |
| VFX, modifiers, lineart, armature/vertex-groups, bake, SVG/PDF export, trace de imagem: **fora de escopo** (VFX = referência adormecida no 02 §10) | não-objetivos |
| Budgets de perf (workstation, `--release`): traço vivo ≤ 1 ms/frame de pack+upload; playback 60 fps com ≥ 200k pontos visíveis + 4 ghosts; flip prev/next < 16 ms | WT/W3 benches |

---

# WT — O traço (FECHADA 2026-07-12 · pendente o smoke do Enio)

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
- [ ] **WT.6 — Smoke do Enio** (zigzag hardness alto/baixo, curvas densas, laço, linha fina com
      zoom out). Kill-criteria K1-K4 no 03 §6.

---

# W3 — Frames · Ghost Frames · Tween (FECHADA 2026-07-12 · pendente o smoke)

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
      **Parcial e declarado:** o *picker de easing* e o toggle de *fade-in dos órfãos* NÃO estão
      na UI (o motor suporta os dois — `TweenOptions`); hoje a UI usa Linear + sem fade.
      Carry-over de UI, não de motor.
- [ ] **T3.8 — Cache de playback.** **NÃO foi feito, e por escolha:** a tesselação já é cacheada
      por DESENHO (T1.8) e é ela o custo real da troca de quadro; o ring de texturas COMPOSTAS só
      compensa se o composite virar o gargalo — e isso se **mede** antes (memória
      `feedback_measure_perf_symptom_scale`). Fica como carry-over COM bench: ring keyed por
      (frame, escala), invalidação por (camada, desenho) sujo, drop de frame no relógio (04 §4).
- [ ] **T3.9 — Marcadores fixos (light table)** — carry-over explícito (o passe de ghost já
      aceita a lista; falta a UI de marcar).

**Carry-overs da W3 (conscientes, não esquecidos):** drag de célula/borda na tira (mover chave e
esticar hold por arrasto — hoje pelos botões ◀/▶ e pela caixa Hold) · multi-seleção de chaves
(destrava o modo `Selected` dos fantasmas, já pronto no modelo) · light table.

**Gate W3:** smoke — 2 desenhos-chave, ghosts ligados, Add Tween, play com loop; goldens do
onion; bench do cache.

---

# W4 — Fill (balde)

**Objetivo:** balde robusto para line-art com Gap Closure interativo, resultado = GEOMETRIA.
Referências: `02` §6 (pipeline exato + constantes) e `04` §3 (upgrades decididos).

- [ ] **T4.1 — Fill como geometria.** `fill_id` por-stroke (fills com buracos multi-curva,
      02 §1) + `hide_stroke`; render no passe de fill existente (depth `sid+1` — 02 §2b).
      Aceite: fill de N curvas com furo renderiza.
- [ ] **T4.2 — Pipeline raster.** Fit-to-bounds (margem 20px, Precision, mín 128², zoom ≤5×) →
      render offscreen com **`radius_scale = 0.5`** + threshold `r ≥ 1/255` → span fill com
      **leak filter cruzado 3px** → Moore trace (buracos = contornos separados) → **RDP ε≈1.25px
      + fit Schneider** (upgrade sobre o smooth 20× do GP) → stroke cíclico com fill. Buffer de
      flags dedicado `Vec<u8>`. Falha total ao tocar a borda (+ modo invert). Aceite: clicar
      dentro de forma fechada preenche; goldens do trace.
- [ ] **T4.3 — Gap Closure.** Extend (pontas + **quinas mid-stroke por curvatura**) com corte
      por colisão (2 passes, 3 exclusões) + Radius (círculos-guia SÓ nos gaps pendentes;
      linhas centro-a-centro); ajuste modal ao vivo (scroll). **Fechamento bem-sucedido vira
      stroke INVISÍVEL persistente** (twist do Harmony — o re-fill sobrevive). Aceite: fechar
      forma com abertura; helpers visuais; re-fill de frame vizinho reaproveita.
- [ ] **T4.4 — Semântica de balde de animação.** Modos **Paint / Paint Unpainted
      (paint-behind) / Unpaint** + **Grow/Shrink** por offset CAD do polígono (+2px default) +
      **Precision**. Fill lê camada(s) de referência (linha) — o contrato linha/cor. Aceite:
      colorir sem tocar a linha; grow mata o halo do AA.
- [ ] **T4.5 — Fill multiframe** (roda POR FRAME selecionado — N fills independentes). Aceite:
      pintar a mesma região em 3 quadros de uma vez.

**Gate W4:** smoke — line-art com gaps, preencher com preview dos helpers, Grow/Shrink,
paint-behind, multiframe.

---

# W5 — Reshape (escultura de traço)

**Objetivo:** remodelar traços com pincéis de raio+força+falloff. Referência: `02 §7`
(os 9 pincéis com TODAS as constantes — a W5 está lá, não aqui).

- [ ] **T5.1 — Trait `ReshapeBrush`** (3 callbacks, 02 §7) + infra: influence =
      `alpha·pressure·falloff(dist, raio)·multi_frame_falloff` (falloff curve do Painter);
      invert por Ctrl; **auto-masking congelado no down** (seleção/camada ativa; threshold
      20px). Aceite: seam headless do pipeline de influence.
- [ ] **T5.2 — Smooth** (binomial iterations=2, influence = mistura; projeta TODOS os pontos).
      Aceite: alisar traço trêmulo sem encolher pontas.
- [ ] **T5.3 — Push + Grab** (push = delta·influence por sample; grab = máscara+pesos
      CONGELADOS no down, pressure=1). Aceite: os dois com a distinção de UX correta.
- [ ] **T5.4 — Thickness + Strength** (aditivos: ±0.001 no raio [na NOSSA unidade: px de
      tela], ±0.125 com clamp na opacity). Aceite: engrossar/apagar gradual.
- [ ] **T5.5 — Pinch + Twist** (pinch `inf²/25`; twist 1°·influence em tela). Aceite: ambos.
- [ ] **T5.6 — Randomize** (hash splitmix64 por sample, perpendicular ao movimento) — 2º corte
      se apertar. **Clone = comando** (paste posicionado), não brush — fora da W5.
- [ ] **T5.7 — Reshape multiframe** (o falloff já está na assinatura desde T5.1; UI de seleção
      de frames + curva com ativo em 0.5). Opcional/carry-over.

**Gate W5:** smoke — smooth, push, grab, engrossar; constantes com a "sensação GP".

---

# W6 — Integração com a Timeline principal (ADIADA — Enio 2026-07-12)

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
- **Tween v2:** matching espacial + espiral logarítmica + UI de correção de pares (04 §2).
- **Colorize (wave própria):** trapped-ball ("colorir tudo") → LazyBrush/CTG com onion-fill
  (04 §3) — a feature de produção que só o TVPaint tem.
- **Ghost extras:** light table (marcadores fixos) + Shift & Trace (transform por ghost +
  F1/F2/F3) (04 §4).
- **Edit Mode** (seleção de traço/ponto/segmento + transform): o modelo de seleção está no
  02 §11; é um pacote próprio pós-W5.
- **2.5D multiplane** (parallax_factor por camada — ADR-0114 §Decisão 3).
- **Instância de drawing na UI** (o modelo já suporta; entra JUNTO com o gesto + marcador
  visual na tira — lição GPv3, 04 §5).
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
