# HANDOFF DE INTEGRAÇÃO — `line/Painter`: lag do Rake + remoção do Random Angle (2026-07-19)

> Para o **agente integrador** (por ordem EXPLÍCITA do Enio — a linha NÃO integra sozinha).
> Duas mudanças, dois commits, sobre `origin/main` integrada. **Pendente de smoke do Enio.**

## 0. Estado

| | |
|---|---|
| Branch | `line/Painter`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` |
| Base | `c2ddb6e0` (= `origin/main` + o doc de takeover), rebaseado no início da jornada |
| Ahead of `origin/main` | **3 commits**: o takeover doc (`c2ddb6e0`) + os 2 abaixo |
| Árvore | limpa · `cargo check --workspace --all-targets` 0 · clippy 0 · LOC cap verde |

```
a3b54853 feat(painter): remove o Random Angle por-slot (Shape + Grain) -- o Jitter Rotate do Stroke cobre
427e580b fix(painter): o Rake segue o traco -- Dab::dir vem dos CENTROS dos dabs, nao do EMA que atrasa
```

Suítes (as 4 crates tocadas): **1907 verdes, 0 falhas**. Arch gates verdes
(`architecture_panel_wiring_parity`, `node_id_collisions`, `*_loc_cap`, `interactive_crate_has_behavioral_test`).

## 1. O pedido do Enio (2026-07-19)

> *"Shape:Texture:Rake e Random Angle foram quebrados e não funcionam. As mesmas funções em Grain estão
> ativas (embora Rake em Grain esteja ruim). … Avalie se faz sentido ter Rake e random em grain e paper. Se
> não fizer sentido vamos deixar apenas em Shape. Outra coisa: temos Jitter rotation no Stroke. Avalie se faz
> sentido deixar Random Angle nos slots de textura."*

**Achado da investigação (importante):** Shape Rake/Random **não eram código morto** — provei o depósito
(imagem E procedural), o seam real (`Click(PAINTER_SHAPE_RAKE/RANDOM)` vira o flag) e rendeziei a rotação
de 90°. O que o Enio via como "quebrado" era **o LAG do Rake** (mais visível no Shape, que tem orientação
forte) e o Random mascarado pelo overlap denso. Decisões confirmadas pelo Enio: **(A)** consertar o lag e
manter Rake em Shape+Grain; **(B)** remover o Random Angle por-slot dos dois slots (o Jitter Rotate cobre).

## 2. Commit `427e580b` — o Rake segue o traço (o LAG)

**Causa:** `Dab::dir` era a EMA suavizada do heading (`heading::advance`, `smooth_len = 0.1·diâmetro`), cujo
atraso **escalava com o pincel**: medido **5,9° no raio 8 → 51,7° no raio 60** (meio ângulo reto). É o mesmo
defeito que o Sculpt Chisel curou em 2026-07-18, e a nota dele dizia "os 4 leitores de `Dab::dir` ficam
intocados" — o Rake de Shape/Grain eram dois desses.

**Fix (engine, mínimo):** `dab_at` passa a carimbar `Dab::dir` com a direção entre **CENTROS de dabs
consecutivos** (`heading::from_centers(last_emit_pos, pos, self.heading)`), atraso ~meio espaçamento em
qualquer pincel (**6,9° no raio 60**). Novo campo `Stroke::last_emit_pos` (reset em `begin`; os shape editors
criam um `Stroke` fresco por re-stamp, então reseta sozinho lá).

⚠️ **Byte-idêntico para o Push e o Chisel, e é a espinha do fix:** o 1º dab cai de volta na EMA settled (o
`from_centers` devolve `smoothed` quando não há predecessor), e **Push e Chisel só leem `d.dir` no 1º dab**
(o Push como predecessor sintético em `impasto.rs:341`; o Chisel como fallback do `path_axis` em
`sculpt_session.rs:289`) — para os dabs ≥2 eles usam o próprio center-diff. Witnesses passando:
`the_raked_groove_runs_parallel_to_the_stroke`, `the_knife_carries_the_body_...`, `the_rim_is_a_fact_...`,
`the_trench_is_a_fact_...`. Simetria: `push_symmetric` já espelha `dir` (correto). Tiling: as cópias herdam
`dir` (translação preserva direção).

**Gate red-first:** `the_rake_heading_does_not_lag_the_stroke_on_a_large_brush` (`stroke/tests.rs`, raio 60,
tol 15°) — **RED em 51,7°** revertendo `dab_at` para `dir: self.heading`; **GREEN em 6,9°**. O irmão antigo
`arc_stroke_heading_tracks_the_tangent` só cobria raio 8 com tol 25°, ficava verde sobre o bug. Render-and-look
confirmado (traço em L: a ponta direcional segue as duas pernas, sem ghost de atraso).

## 3. Commit `a3b54853` — remoção do Random Angle por-slot

**Removido:** o campo `TextureSettings::random_angle` + o ramo random de `dab_basis` (e `random_unit`, morto);
o termo `random_angle` das 5 gates de cacheabilidade/rotação; os toggles `toggle_brush_{shape,texture}_random`;
os campos de snapshot `{shape,texture}_random`; as linhas de UI "Random Angle" (Shape + Grain); os ids
`PAINTER_SHAPE_RANDOM` / `PAINTER_BRUSH_TEXTURE_RANDOM`; o wiring de `populate`/`event`/`brush_fallback`.

**Intactos:** Random **Offset** (`TextureMapping::Random`) e **Jitter Rotate** (`BrushSpec::jitter_rotate`) —
são coisas diferentes. `BrushSpec` não é serde ⇒ nenhum save antigo carrega o campo. Paper já não tinha.
Byte-idêntico para pincéis sem random_angle (o ramo só era tomado com o toggle ligado).

**Gate/decisão:** `jitter_rotate_is_the_grains_random_spin_now_that_per_slot_random_angle_is_gone` (irmão do
de Shape) pina que a capacidade não se perdeu. `grain_rake_and_random_are_inert_under_the_wash` →
`grain_rake_is_inert_under_the_wash`. Os testes de tiling-share e do stream de rng do impasto passaram a usar
**Random Offset** (o draw por-dab que sobra). Lista `GATED` do painel: 8→7 (sem RANDOM).

## 4. Notas de integração (DIRETRIZ §1.5.9)

- **Ids removidos:** `PAINTER_SHAPE_RANDOM`, `PAINTER_BRUSH_TEXTURE_RANDOM` (hashes de `painter_brush.shape_random`
  / `painter_brush.texture_random`). Não colide com nada — só remoção.
- **Campo removido de `TextureSettings`** (foundational-ish `ph2d-painter-brush`): `random_angle: bool`. Uma
  linha paralela que tenha adicionado campo a `TextureSettings` pode conflitar textualmente no struct/Default —
  resolver mantendo os dois conjuntos de campos (menos o `random_angle`).
- **`Dab` ganhou dependência de `Stroke::last_emit_pos`** mas a forma pública do `Dab` **não mudou** (o campo
  `dir` continua; só a semântica dele — agora center-diff, não EMA). Nenhum `PROJECT_SCHEMA`/contrato bumpou.
- **Docs históricos** (`docs/Painter/05,06,PLAN_*`, `HANDOFF_shape_grain_dual_texture`, `HANDOFF_painter_texture_section`)
  ainda citam `random_angle` — são snapshots de projeto, deixados como estão. CLAUDE.md §5 foi atualizado.

## 5. Pendente de smoke do Enio (o veredito é CONDICIONAL)

Rodar o app e conferir com os próprios olhos:
1. **Rake não atrasa:** Shape com ponta direcional (ou Grain Stripes) + Rake ligado, pincel **grande**, traço
   em **curva** — a ponta/grão segue a tangente, não aponta pra trás. Antes (r60) atrasava 52°; agora ~7°.
2. **Random Angle sumiu** das seções Shape e Grain, e o **Jitter Rotate** (seção Stroke) faz o giro aleatório
   por-dab (do stamp inteiro, com slider).
3. **Grain Rake** deixou de estar "ruim" (mesmo conserto de lag).

## 6. ⛔ NÃO integrei nem pushei (protocolo §0.7 / §0.2)

Fechei, escrevi este handoff, **PAREI**. Integração e ship só por ordem explícita do Enio, via agente
integrador dedicado.
