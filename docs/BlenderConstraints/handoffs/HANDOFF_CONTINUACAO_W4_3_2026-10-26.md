# Continuação — `line/BlenderConstraints`, onda W4 (CAD no vetor), 26/10/2026 — W4.2 FEITA, abre a W4.3

> Anterior: [`W4 a meio`](HANDOFF_CONTINUACAO_W4_2026-10-26.md). Doc da onda: [`04_W4_cad.md`](../04_W4_cad.md) — §3.1 diz o
> que a W4.2 entregou e o que mudou no caminho; §4 marca os gates ✓. **Não duplicado aqui.**

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4` (o `main` não andou
até 26/10 12h). Commits da W4.2: `2bcf0377bb` fixture v23 · `9d3b42c514` núcleo (`VertId`, `VEC_SCENE 24`) · `f2a0d537a3`
projeto 195 + consumidores + Copy Point · `04e5edc42d` Cargo.lock · `76bea14f72` fmt · `3400a70456` herança pela posição. Árvore
limpa. `PROJECT_SCHEMA` desta linha = **195** (reconta-se na integração: três sítios, `scripts/schema-recount.py`).

## Feito na W4.2 (não refazer)

- `VecVertex.id: VertId` e `VecPath.next_vert` no FIM; `normaliza_ids` (push_path) e **`normaliza_ids_desde(anterior)`** no
  gancho `settle_tree_before_capture` com o `undo_baseline` — um vértice `NOVO` herda o id do mesmo lugar na fotografia
  anterior se livre (as formas re-geradas guardam os ids; sem isso, passo de undo espúrio por quadro com entrada).
- Leitura do antigo: um âmbito só, `WireLayout::enter(ver)` (v22 traço+geometria, v23 geometria); projeto por
  `fio_do_projeto(ver)`; blobs do Envelope e da pele por `migra_fio(mundo, fio)`.
- Fixtures sobre BYTES do código antigo: `vec_scene_v23_geometria.bin` (cena) e `projecto_v194_geometria.ph2dproj` (projeto,
  gravado numa worktree temporária em `db09f4bd45`, já removida) — ⛔ nunca regenerar.
- `PontoDeForma { vert: VertId }`; `Pontos` = `(VertId, âncora)` com `subpaths`; id sumido ⇒ `SemAlvo`.
  `ph2d-constraints`/`-ecs` dependem de `ph2d-vec-scene`.
- Verde por nome: `ph2d-vec-scene` inteira (motivo formato gravado, 578), `vertex_ids_through_edits` (2), `vec_v23_fixture`,
  `stroke_v22_fixture`, `o_ponto`, `poderosas::a_point_of_a_shape_is_a_target`, `oraculo_w3`, `traco_v22_tests` (4),
  `geometria_v194` (3), `traco_v189`, tripla, `zorder_fixpoint` (7), `zorder_late_writers`, `the_shell_only_shrinks`,
  `the_net_knows_every_derived_writer`. `cargo check --workspace --tests` = 0 erros, 0 avisos.

## Aberto da W4.2 (conhecido, não bloqueia)

- Os sítios de `ph2d-vec-edit` e `ph2d-vec-boolean` não têm caso no gate: o censo (explorador, 26/10) leu-os — todos copiam o
  vértice inteiro (K) ou re-geram a forma (N, coberta pela herança). Se a W4.4 os tocar, junte um caso por sítio lá.
- O passo «apagar um vértice tira as regras que o citam no MESMO passo de undo» é da W4.4 (as regras ainda não existem).

## Próximo passo — W4.3 (§3 da tabela; abra pelo gate vermelho)

1. Crate NOVA `ph2d-vec-constraints` (pura): `kcl-ezpz =0.2.18`; a tradução regra → `kcl_ezpz::Constraint` está pronta em
   `docs/BlenderConstraints/ferramentas/bancada_solvers/src/main.rs` `mod ez` (`monta`, `tangente_orientada`, `uma_vez`) —
   porte-a, endereçando pontos por `(VecPathId, VertId)`. Arrasto = `Fixed` do agarrado + palpite da solução anterior; mover
   a âncora leva as duas alças pelo mesmo delta.
2. **Decisão técnica a tomar lendo o código** (delegada, padrão-ouro): o que é «círculo» no vetor para `Tangente`/`Raio` — a
   forma viva Elipse (`ph2d-app-vec/src/shape_live.rs`) com `rx = ry`, como datum centro+raio que se re-escreve depois de
   resolver (guardando os ids)? Escreva a escolha e o porquê no §3 antes do código.
3. Gates do §4 da W4.3: `a_well_constrained_sketch_lands_where_freecad_puts_it` (fixture `bem.*`, 1e-6),
   `the_drag_keeps_every_rule_at_every_step` (escada 202 / misto 200, 60 passos, 1e-4 px), `the_drag_is_deterministic`.
4. REF = teste de posto nosso (jacobiano; `faer` já vem com a ezpz); medir o custo de compilação da família `faer` (crate nova
   ⇒ `PH2D_SUITE="motivo: crate NOVA"`).

## Armadilhas

- `cargo-test-narrow.sh` dentro de um `for` é recusado pelo guarda (R5) — escreva cada chamada por extenso.
- O `poe_campo.py` desta jornada (scratchpad, não versionado) casava só o nome curto do tipo nos erros do compilador; o nome
  qualificado (`ph2d_vec_scene::VecVertex`) passou mudo na 1.ª ronda — confira a contagem de erros depois de cada ronda.
- `lib.rs` da `ph2d-vec-scene` está a ~680 de 700 linhas: métodos novos da cena vão para o módulo do assunto (`vert_id.rs`).
- Gravar uma fixture com o código ANTIGO: `git worktree add --detach <scratchpad>/wt <sha>` (target próprio), teste `#[ignore]`
  temporário, `git worktree remove --force` no fim.
