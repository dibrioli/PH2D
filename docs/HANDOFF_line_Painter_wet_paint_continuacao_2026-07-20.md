# HANDOFF — line/Painter: Wet Paint, W0 FECHADO → continuação (W1..W3)

> Continuação de [`HANDOFF_line_Painter_wet_paint_2026-07-20.md`](HANDOFF_line_Painter_wet_paint_2026-07-20.md)
> (a tarefa, as regras do Enio, o mapa de integração — **releia o §4 e o §5 de lá antes do W1**).
> Protocolo de troca: `cd Worktrees/line-Painter && pwd && git branch --show-current` ANTES de tudo.

## §1 — O QUE ESTÁ FEITO (2026-07-20, HEAD `b35bb036`)

- **[ADR-0134](architecture/decisions/0134-wet-paint-fluid-sim-returns-cpu-first-parity-tested.md)**
  aceito: supersede o ADR-0096 NESTE ponto; fixa nome (crate `ph2d-wet-paint`, prefixo
  `wetpaint_`, rótulo de UI "Wet Paint"), contrato de neutralidade (OFF byte-idêntico), lei de
  integração e kill criteria. ⚠️ **Tem uma EMENDA pendente de veto do Enio**: a barra da sim viva
  foi re-derivada da MEDIÇÃO (tabela no ADR) — sessão representativa pior classe ≤ 2 ms (medido
  0,84) · flood §18 pior classe ≤ 12 ms (medido 8,3–9,4; o 8 original era palpite pré-medição).
  O solver é **serial POR SEMÂNTICA** (o brake lê `wet` VIVO escrito no mesmo passe; o drying lê
  o vizinho esquerdo pós-update) ⇒ ADR-0109 (bandas bit-idênticas) é inaplicável — não re-derive.
- **W0 — a crate `ph2d-wet-paint`**: porte 1:1 de `docs/Painter/ph2d_wet_paint/js/engine/`,
  módulo a módulo (16 módulos; leia o doc de `lib.rs` pra lei do porte: **aritmética f64,
  storage f32**, semântica JS só via `jsmath`, transcendental só via `libm =0.2.16`).
  - **Suite §18 completa e VERDE** (`tests/acceptance.rs` §18.1–.10, `tests/acceptance_budget.rs`
    §18.11–.12 + integral §7 — os orçamentos *binding* de massa/água/cobertura nas DUAS bitolas
    passam). `tests/perf.rs`: gates de wall-clock são `#[ignore]`, rodam com
    `cargo test -p ph2d-wet-paint --release --test perf -- --ignored --nocapture`
    (metodologia: **mediana por classe de cadência** — max de amostra única é ruído de scheduler).
  - **`tests/fingerprint.rs`**: fingerprint de sessão pinado — TODA reescrita de hot loop se prova
    byte-idêntica contra ele (o pin tem histórico comentado; só move com justificativa semântica).
  - **W0-verify (auditoria adversarial JS↔Rust, workflow 16 pares + verificadores)**: 6
    divergências reais achadas e corrigidas (ordem de soma do advect · lattices do papel eram
    Float32Array · guard falsy do std · `spacing||2` · `reset_group` engolia as notificações de
    rebuild → `Engine::reset_knob_group` é a porta do painel · ToInt32 pleno no opacity). A
    **família NaN é divergência ACEITA e documentada no doc do `jsmath`** (exige estado já
    envenenado; o Rust recupera onde o JS fica envenenado) — não "conserte".
- **Sem runtime JS na máquina** (node/deno/bun ausentes) — irrelevante: a suite Rust É a
  aceitação; o app de referência roda no browser (`python3 -m http.server`) pro smoke visual.

## §2 — W1 (PRÓXIMO): o modo no painter — o blueprint reconhecido

- **`PaintMode::WetPaint`** em `ph2d-tool-painter/src/tool/paint/paint_mode.rs` — variant novo,
  `slot() = 11`, `PAINT_MODE_COUNT = 12` (slot próprio de `BrushSpec`, o padrão que a Faca
  acabou de estabelecer). Chip na lista de tools? — é decisão do painel (W3); o modo primeiro.
- **Master switch**: `BrushSpec::wetpaint` OFF por default + **gate de fingerprint OFF
  byte-idêntico** (padrão `impasto_off_is_byte_identical`). Os smokes aprovados são o contrato.
- **Pendura em `stamp_dabs_inner`** (`stamp_route.rs:250`) com early-return ANTES das rotas de
  cor, como o Sculpt faz (`:276`) — o engine é dono do depósito de pigmento. Isso dá
  Symmetry/Tiling/shape editors/pressão/Jitter de graça.
- **O tick**: `Tool::on_tick` (`trait_impls.rs:541` — o watercolor já seca o papel por ali) +
  acumulador de 40 Hz fixos (clamp 5 passos, SPEC §5). A sim PAUSA com pointer down (menos blow).
- **O Grid do engine** dimensionado ao canvas da camada ativa do painter; o preview compõe o
  `render_region` do engine sobre a base congelada (o modelo do watercolor: composite sobre base).
- **A pressão sintética do SPEC §8 é SUBSTITUÍDA** pela pressão real do stroke engine do painter;
  o trail/§10 do engine CONSOME os dabs do painter (o `stroke.rs` da crate segue existindo pros
  testes §18, mas o produto alimenta `trail::accumulate_paint` direto com dabs reais).
- **Silhueta**: compor `silhouette_at` (dab.rs) com a bristle (fator, como o Grain) — a bristle
  do SPEC §7 é o *default* do modo; Grain do artista substitui.
- **Undo**: a sessão molhada (grid do engine) entra no `ModelSnapshot` **no mesmo commit** que
  criar o estado (lição §10.4 do impasto). O que capturar: `snapshot_grid` já existe na crate.
- Smoke: cada wave entrega cena `PH2D_WETPAINT_SMOKE=N` auto-play.

## §2.4 — W1 FECHADO (2026-07-20, HEAD `13009e54`) — AGUARDANDO SMOKE DO ENIO

**Smoke: `PH2D_WETPAINT_SMOKE=1 cargo run --release -p ph2d-host-desktop`** — canvas branco
1024², brush já armado em Wet Paint (azul, 24 px): escolha o Painter e arraste; solte e ESPERE
(a água segue nivelando/sangrando/secando a 40 Hz). ⚠️ O smoke arma o MODO em código de
propósito e documentado: até o W3 não existe chip/painel que selecione Wet Paint.

O que entrou (commits `2437a0cc..13009e54`):
- **Portas de produto no engine** (`822f7ae4`): `dispatch_pressure_dab` (o §9 com pressão REAL +
  raio REAL; `dispatch_dab` vira wrapper — fingerprint prova delegação byte-idêntica) ·
  `begin/segment/end_direct_stroke` (traço alimentado por dabs reais, sem history do engine) ·
  `render_pigment_only_region` (o full render virou wrapper da região — um corpo por célula).
  Gates: região==full dentro do rect + sentinela fora; depósito + gating do sim.
- **O modo no tool** (`8a89de7a`, `tool/paint/wetpaint.rs`): sessão = engine + base congelada
  (`Arc`) + **guard de identidade de canvas** (o `wet_session_canvas` do watercolor, EAGER — no
  dab E no tick, porque o sim composita sem pen-down; undo/fill/troca de layer matam a sessão
  por `Arc::ptr_eq`). **Display-state, não document-state**: o composite escreve `canvas_rgba`
  por dirty-rect, **encerrar a sessão É o bake** (os pixels já estão lá), e o grid fica FORA do
  `ModelSnapshot` — um `GridSnapshot`/passo seria ~235 MB a 2048² (ADR-0117). Consequência
  honesta, pro smoke julgar: **undo de um traço wet devolve o LOOK e mata a água** (o redo não
  ressuscita a sessão). Rota em `stamp_dabs_inner` ANTES do passe de altura (relevo de tinta que
  ESCOA seria errado 2×); tick 40 Hz clamp 5 em `paint_tick`; pen-up fecha o traço direto.
  Célula 1-based: pixel `p` → célula `p+1` (o `view.toCell` do reference).
- **Depósito só em gesto VIVO + métodos CUMULATIVOS** (Dots/Airbrush/Space): `live_gesture`
  armado no `paint_begin` — ⚠️ o sinal NÃO pode ser `paint.stroke` (o lifecycle `mem::take` o
  stroke durante o stamp; foi o 1º vermelho dos gates). DragDot/Anchored/Line re-carimbam por
  frame e o depósito de fluido NÃO é idempotente (I2) — recusados até o W2 desenhar (idem shape
  editors, que nem chegam ao `paint_begin`).
- **4 gates mutation-tested** (inline em `wetpaint.rs`; 3 mutações sangram: rota · guard do
  tick · teardown de modo) + suíte inteira do tool verde (743).
- **LOC**: `solver.rs` 858→539 (meu débito do W0; split `solver/advect.rs`+`solver/project.rs`,
  fingerprint prova byte-identidade) · `paint.rs` 713→**700 exatos** (re-ancoragem de
  doc-comments pra caber `mod`+campo). ⚠️ **7 ofensores HERDADOS seguem no gate
  `workspace_src_files_under_loc_cap`** (das waves AA/rake anteriores da linha, pré-meus
  commits): `watercolor_render` 751 · `sculpt_tests/w3` 743 · `sculpt` 715 · `watercolor_field`
  709 · `dab` 708 · `spec` 701 — fecham no gate batched do fechamento da linha.

**Decisões registradas p/ Enio validar no smoke:** cor por-TRAÇO (Randomize per-dab = W2) ·
knobs = defaults §16 do reference (painel = W3) · seção Watercolor ligável em modo WetPaint
ainda não é escondida (incompatível; W3, lei #3) · gatilho de commit da sessão = mutação alheia /
troca de modo (secagem completa NÃO encerra sozinha — decisão aberta de produto).

## §2.5 — W1 (histórico do andamento; superado pelo §2.4)

- **Inc.1 COMMITADO (`c329d126`)**: `PaintMode::WetPaint` (slot 11, `PAINT_MODE_COUNT` 12) + as
  DUAS portas de wire-string (`set_paint_tool_mode("wetpaint")` / `active_paint_mode_id()`).
  Nada seleciona o modo ainda (sem UI, sem smoke) — em modo WetPaint hoje os dabs caem nas rotas
  normais de cor; a rota própria é o próximo incremento.
- **Decisão de desenho (documente no gate)**: SEM `BrushSpec::wetpaint` — o modo É o switch
  (precedente Knife; um flag por cima do modo seria 2ª porta pra mesma pergunta). O contrato
  "OFF byte-idêntico" vira: (a) arch-gate "nenhum outro modo alcança o engine wet" (irmão de
  `no_other_paint_mode_touches_the_relief`) + (b) fingerprint de pintura normal antes/depois.
- **O modelo de display é o do watercolor, generalizado** (`watercolor_render.rs:47` é o molde):
  sessão congela a base (`Arc` dos pixels pré-sessão, padrão `wet_session_base`); cada frame
  recompõe SÓ o dirty rect — para o wet paint: `render_pigment_only` do engine (RGBA straight
  sobre transparente) alpha-over a base, escrito em `canvas_rgba`. Commit da sessão derruba a
  base (dentro da transação de undo). A sessão do wet paint atravessa TRAÇOS (a água segue viva)
  — decidir o gatilho de commit: secou-completamente / troca de layer / troca de modo / Apply.
- **`Dab` do painter** (`ph2d-painter-brush/src/stroke.rs:27`): `center [f32;2]` · `radius_px` ·
  `coverage` (strength×pressão) · `color [f32;3]` (Randomize já resolvido!) · `rotation` ·
  `dir` · `arc_len` · `stroke_radius_px`. Mapeamento pro engine: center→(x,y), radius_px→r,
  coverage→intensity (o §9 do SPEC vira: pressão REAL já embutida em coverage), color→cor do
  dab (o trail do engine usa a cor por-célula — Randomize de graça).
- **A sessão mora em módulo IRMÃO novo** (`tool/paint/wetpaint.rs` + `wetpaint_session.rs` se
  crescer): `paint.rs` está a 713 linhas com teto congelado — só cabe o campo
  `wetpaint: WetPaintState` no struct (e talvez precise re-ancorar um doc-comment pra caber).
- **Rota**: em `stamp_dabs_inner` (`stamp_route.rs:250`), logo após o braço do Sculpt:
  `if matches!(self.paint.paint_mode, PaintMode::WetPaint) { self.stamp_dabs_wetpaint(dabs, &brush); return; }` —
  antes das rotas de cor. O tick: `paint_tick` (`stroke_lifecycle.rs:215`) ganha
  `self.wetpaint_tick(dt_s)` ao lado do `dry_canvas_wet` (acumulador 40 Hz, clamp 5 passos).
- **Undo §10.4**: o grid da sessão entra no `ModelSnapshot` NO MESMO commit que criar o estado
  (o bug do `mats` se escondia na tela vazia — teste onde o fato pode ser CONTRADITO).

## §3 — W2/W3 (depois)

W2 = integração total recurso a recurso com gate de seam cada (Shape/Grain/Paper/Falloff/Blend/
Randomize/Selection/alpha-lock/Symmetry/Tiling/stroke methods). Paper: os 3 presets do SPEC §4
viram fontes do slot `BrushSpec::paper` — leia `docs/Painter/19_relevo_do_papel_investigacao.md`
ANTES (a extração de substrato quer ADR; o Wet Paint é o 2º consumidor que a justifica).
W3 = seção "Wet Paint" do painel (espelho de `paint_watercolor.rs`): ~meia dúzia de knobs curados
(a tabela §16 do SPEC é a fonte; o resto vira constante nomeada), incompatíveis ESCONDIDOS com
gate de presença+ausência. O reset de grupo do painel chama `Engine::reset_knob_group` (nunca
`Tuning::reset_group` direto — ela devolve os defs mudados e o caller TEM de reagir).

## §4 — Avisos operacionais

- ⚠️ O cwd do shell RESETOU pro primário no meio da sessão sem aviso — toda mutação por caminho
  ABSOLUTO com `/Worktrees/line-Painter/` (a memória `sed_relative_path` salvou esta sessão);
  `pwd` antes de cargo/git.
- ⚠️ 5 subagentes do workflow morreram em "session limit" — verificações deles foram feitas à mão
  (advect confirmado por inspeção; oráculos de teste endurecidos). Economize subagentes.
- O commit do W0 é `75606759`; os fixes do verify `b35bb036`; ADR `f36a533a` + `40a023b3` (lock).
