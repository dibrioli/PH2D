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
