# HANDOFF DE INTEGRAÇÃO · `line/motion-value` · **bloco Z** — 2026-08-23

> **A linha NÃO integrou e NÃO pushou** (`CLAUDE.md` §0.7). Oito commits locais, à espera de ordem
> explícita do Enio.

**Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` · **branch:**
`line/motion-value` · **base:** `main` em `35f937cb2`.

---

## §1 — O que entrou, em uma frase

**Todo teto deste catálogo passa a dizer de que RECURSO ele é** (`CLAUDE.md` §0.0) — 27 params
curados em 11 crates, os dois grampos de integração medidos, e a legenda das cenas de smoke
mudou-se para o canvas. Conferência 89: **89 → 82 P2** (7 células, 4 folhas).

Registro completo: [`docs/Motion Nodes/91_os_tetos_que_ninguem_mediu.md`](../91_os_tetos_que_ninguem_mediu.md).

---

## §2 — Os commits (ordem de aplicação)

| sha | o que |
|---|---|
| `311176ba8` | folha 04: o `falloff` do `motion.kaleidoscope` REFUTADO por medição |
| `1d1634cdb` | a porta que promete um CAMPO e lê só o elemento 0 (`spline_wrap`, `lattice`) |
| `e6e324509` | `motion.wave`: a altura escolhe o canal (`height_channel`) |
| `029a939be` | cena `=83` — o campo que era um número |
| `079305096` | **bloco Z**: 25 tetos de precisão + o `rate` + o ângulo + os dois `MAX_DT` |
| `ef97264af` | a legenda das cenas de smoke vai para o CANVAS |
| `e947b6c98` | o teto de paradas do gradiente ganha derivação (e a medição confirmou o 8) |
| `e04f092b5` | o doc 91 + as 7 células fechadas + as contagens reconciliadas |

---

## §3 — Superfície de colisão (o que o integrador tem de saber)

### §3.1 — ⚠️ **Um TOKEN novo** (o único item foundational-ish)

`docs/design/tokens.json` ganha **`chrome.panel-min-w: 220`**, re-exportado como
`ph2d_tokens::PANEL_MIN_W_PX`. O valor **já existia** — era um `const MIN_W` privado *dentro* de
`clamp_panel_rect`, e o comportamento não muda um pixel.

⚠️ **Se outra linha tocou `tokens.json`, este é o ponto de merge.** É um acréscimo de uma linha
num bloco ordenado; a lista de re-export em `ph2d-tokens/src/lib.rs` é alfabética e o nome novo
entra entre `PANEL_HEAD_PAD_PX` e `PANEL_RADIUS_PX`.

### §3.2 — Crates tocadas (11 crates-nó, 3 de infra, 1 shell)

`ph2d-node-{field-box, field-radial-sweep, field-remap, force-attractor, force-buoyancy,
force-vortex, force-wind, motion-emitter, motion-four-point-warp, motion-integrate, motion-move,
motion-spherize, motion-voronoi, sim-spawn, sim-step, value-lfo, registry-init}` ·
`ph2d-{tokens, editor-core, panel-motion-params, gpu-cook}` · `shells/desktop`.

⚠️ **Quase toda a edição é APENDAR uma `static PARAM_HARD_MAX`/`PARAM_HARD_MIN` e uma linha de
`register_*`** — colisão de mesmo-símbolo é improvável, e um merge textual que perca uma das duas
metades é apanhado pelo gate (que exige a entrada E o número).

### §3.3 — Dois splits por HR-18

- `ph2d-node-field-remap/src/params.rs` (novo) — hints/gates/grupos/teto saem do `lib.rs` (686 → 503)
- `ph2d-node-motion-four-point-warp/src/bounds.rs` (novo) — os 16 limites dos 8 cantos (681 → 693)

### §3.4 — Uma dev-dependency nova

`ph2d-node-registry-init` ganha `ph2d-core` em `[dev-dependencies]` (só o gate dos tetos, que mede
à cadência de `DEFAULT_HZ`). **`Cargo.lock` mexe.**

---

## §4 — MUDANÇAS DE COMPORTAMENTO (leia antes de integrar)

### §4.1 — ⚠️ O `MAX_DT` dos dois integradores: `0,1` e `0,05` → **`0,03`**

**Em regime é byte-idêntico** (o tique fixo é `1/60 = 0,0167`, muito abaixo do grampo, e o
`FixedStep` da casa entrega um tique por cozedura mesmo num ecrã lento). O que muda é **quanto de
um SCRUB o sim absorve** — e absorver é a resposta certa ali.

A medição (doc 91 §5): a `0,1`, o laço fechado real com uma força que se alcança **arrastando**
atira uma grelha nascida em raio `1,0` a **127,19**; o irmão `sim.step`, a `0,05`, segurava a mesma
cena em `2,49`.

⛔ **`motion.spring`, `motion.boids` e `motion.wave` NÃO foram tocados.** O `spring` deriva do
`MAX_DT` dele **três** tetos medidos e é wave própria; os outros dois ficam registados como dívida.

### §4.2 — ⚠️ Três testes deixaram de estar pinados a literais

`motion.integrate` tinha três gates com `dt` escrito à mão (`0.1`, `0.05`, `0.1`) que reprovavam
**sobre produto correcto** quando o grampo desceu. Passam a derivar do `MAX_DT`.

### §4.3 — ⚠️ A fixtura do mar da paridade CPU/GPU: `density 40 → 90`

Com o grampo mais apertado, dois tiques deixaram de mover o campo acima do piso de **vacuidade**
(`0,0923` contra `0,1`) — e esse piso é o que impede o gate de comparar dois campos congelados.
⛔ **Alargar o PERCURSO está REFUTADO por medição** (4 tiques dão `0,00526 > ε`, 3 dão `0,00266`):
aquela fixtura é caótica **em número de passos**, e o comentário dela já o dizia. A cura é a FORÇA
(paridade final: `2,3e-4`, 8× de folga sob o ε).

### §4.4 — Nada de UI muda de aparência

Os `ParamHardMax`/`ParamHardMin` só alargam o campo **digitável**; nenhum `ParamUiHint` foi tocado,
então **todo arrasto é idêntico**. A legenda das cenas de smoke é chrome novo e é **no-op** quando
nenhuma cena publicou (todo arranque normal do editor).

---

## §5 — Gate de fecho (batched, 1×)

| | |
|---|---|
| `cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast` | **17.865 testes · 17.863 ✓ · 2 ✗** |
| clippy `--all-targets --all-features`, alvo DERIVADO do diff (25 crates) | **0** |
| `cargo fmt --all` | limpo |
| `typos crates/ shells/ docs/Motion Nodes/` | **0** |
| `file_loc_caps` (shell) · `architecture_widget_loc_cap` · `architecture_panel_loc_cap` | ✓ |
| `placar_conferencia.py` | verde · **82 P2 · 4 P1 · ✅ 222** |
| paridade CPU/GPU de sim (`--ignored`, RTX) | **29/29 ✓** |
| `doc-index.sh` | regenerado |

### ⚠️ Os 2 ✗ são FLAKES pré-existentes, em crates que esta linha não toca

| teste | crate | sozinho |
|---|---|---|
| `the_mask_stroke_cost_does_not_follow_the_canvas` | `ph2d-tool-painter` | **4 de 5 ✓** (é flaky mesmo sozinho) |
| `apply_from_doc_is_zero_alloc_steady_state` | `ph2d-timeline` | **3 de 3 ✓** |
| `a_wet_move_costs_what_the_footprint_costs...` | `ph2d-tool-painter` | **3 de 3 ✓** (já listada no §5.0) |

As duas primeiras foram **acrescentadas ao `CLAUDE.md` §5.0** como a 5.ª e a 6.ª — a de zero-alloc
é espécie nova naquela lista.

⚠️⚠️ **E há um achado de PROCESSO ali:** a primeira corrida (com fail-fast) parou em **11.240 com
1.007 testes por correr**. *Um vermelho de flake esconde o resto da suíte.* O `--no-fail-fast` é o
que torna o gate batched uma medição em vez de uma amostra — está escrito no §5.0 agora.

---

## §6 — Prova de mutação (gates novos)

| gate | mutação | RED |
|---|---|---|
| `every_scene_labels_both_halves_on_opposite_sides` | as duas fichas do mesmo lado | ✓ |
| `every_caption_is_chip_sized` | a frase inteira do terminal numa ficha (70 chars) | ✓ |
| `every_precision_bound_param_types_to_the_measured_ceiling` | apanhou, ao vivo, um erro MEU de sinal (`-A - B` em vez de `-(A - B)`) na reescrita dos literais | ✓ |

⚠️ **O terceiro não é uma mutação encenada: é o gate a fazer o trabalho dele durante este bloco.**
Ao trocar os literais truncados pela aritmética exacta (`2_097_152.0 - 0.125`), o `replace` deixou
os pisos negativos como `-2_097_152.0 - 0.125`, que é outro número. *Um piso simétrico escrito como
literal é um sinal à espera de se perder;* a forma segura é a do `bounds.rs`, com um `const REACH` e
`-REACH`.

---

## §7 — O que fica ABERTO (dívida nomeada)

1. ⏳ **`motion.boids::MAX_DT` e `motion.wave::MAX_DT`** continuam a `0,1` **por medir** — copiaram
   o número sem derivação, como os dois que este bloco curou. A sonda `excursion` já está escrita
   (`integrator_ceilings.rs`) e serve-lhes com outro grafo.
2. ⏸️ **`motion.spring`** — mexer no `MAX_DT` dele move **três** tabelas medidas (`friction`,
   saturação do sub-passo, `tension`). Wave própria, com o oráculo de três braços dele.
3. **`MAX_CURVE_POINTS = 8`** (o irmão do teto de paradas, no mesmo arquivo) continua sem
   derivação: *"matches the field.remap text param's practical ceiling"*. O editor de curva tem o
   MESMO `GRAB_R = 9,0`, então a conta é a mesma — só não foi feita.
4. **A legenda no canvas cobre 2 cenas** (`=82`, `=83`). Cena nova = uma `captions()` pura + uma
   linha no `publish`. A lista está em `motion_demo_legend_tests.rs::scenes()`.
5. ⚠️ **Sinal pré-existente:** `conferencia_vs_manifesto.py` sai vermelho na metade *"já existe no
   manifesto"* com 4 células. Lidas uma a uma, são **falsos positivos**: a ferramenta casa o nome
   do param mencionado na CURA proposta (*"um 9º `ease_curve = Custom`"*), não um param que já
   fecharia a célula. A metade das CONTAGENS está verde (127 nós).

---

## §8 — O smoke, para o Enio

Ele já smokou a `=83`. O que este bloco lhe dá para conferir é **o que estava trancado**:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=13 cargo run -p ph2d-host-desktop --release
```

1. Clique no nó **Spherize** (a lente).
2. No painel, ache **Radius**. Ele diz **320**.
3. Arraste o slider: ele salta para **20 ou menos**, e daí não volta.
4. Escreva `320` na caixa e dê Enter. **Antes deste bloco a caixa recusava** (parava em 20); agora
   ela aceita, e a lente volta ao que a cena tinha.

**Deu errado se** a caixa recusar o 320, ou se aceitar e a lente não voltar ao que era.

E a legenda nova:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=83 cargo run -p ph2d-host-desktop --release
```

Cada figura tem agora **uma ficha em cima dela** dizendo o que é. **Deu errado se** as fichas não
aparecerem, ou se aparecerem sobre a figura errada.
