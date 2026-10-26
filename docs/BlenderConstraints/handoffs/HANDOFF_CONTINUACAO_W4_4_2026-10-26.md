# Continuação — `line/BlenderConstraints`, onda W4 (CAD no vetor), 26/10/2026 — W4.3 FEITA, abre a W4.4

> Anterior: [`W4.3`](HANDOFF_CONTINUACAO_W4_3_2026-10-26.md). Doc da onda: [`04_W4_cad.md`](../04_W4_cad.md) — §3.2 (as decisões
> da W4.3: círculo = geometria, a porta é uma folha, REF pelo nosso jacobiano) e §3.3 (o que ficou feito, medido e mutado);
> §4 marca os gates ✓. **Não duplicado aqui.**

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4` (o `main` não andou
até 26/10 à tarde). Commits da W4.3: `ede2f7ab12` decisão (§3.2) · `21573c19b1` crate + gates · `43c81fd66b` tecto de LOC
herdado da W4.2 (`corner_live.rs` 703→699, `ph2d-vec-edit/src/lib.rs` 702→686) e fmt · `5ce85c725f` gates dos
sobreviventes da mutação · o commit deste handoff. Árvore limpa. `PROJECT_SCHEMA` desta linha = **195** (reconta-se na
integração), `VEC_SCENE_SCHEMA_VERSION = 24` (ainda SÓ nesta linha ⇒ campos novos da `VecScene` entram no MESMO degrau).

## Feito na W4.3 (não refazer)

- `ph2d_vec_scene::cad`: `PontoCad::{Vert(path, VertId), Centro(path)}`, `SegCad`, `Lei` (13 leis, inclui `Fixo` e
  `Angulo`), `RegraCad { lei, referencia }` (serde, pronto para gravar).
- `ph2d-vec-constraints` (pura, `kcl-ezpz =0.2.18`): `Folha::new([(&VecPath, Xform local→folha)])`;
  `resolve(&Folha, &[RegraCad], Option<Arrasto>, Option<&Solucao>) -> Resultado { solucao, falta, sem_alvo }`;
  `Solucao::{da_folha, ponto, raio, aplica(&mut VecPath, Xform) -> bool}`; `estado(&Folha, &[RegraCad]) -> Estado {
  regras: Vec<EstadoRegra{Ok,Ref,Redundante,Conflito,SemAlvo}>, pontos: BTreeMap<PontoCad, Liberdade> }`;
  `orienta_tangente` (o centro à ESQUERDA, a convenção da ezpz); `circulo_de`.
- Verde por nome (14): os 9 do commit `21573c19b1` + os 5 do `5ce85c725f`; a crate inteira 1× (motivo crate NOVA, 9/9
  antes dos 5); `workspace_src_files_under_loc_cap`, `no_dependency_climbs_a_layer`, `every_member_inherits_the_workspace_lints`,
  `the_stack_versions_page_matches_the_lockfile`, `no_source_file_in_the_workspace_is_left_out_of_the_build`,
  `cargo deny check licenses bans`, os dois testes da junção da caneta (`ph2d-vec-edit`), `every_corner_rounds`.
- Mutação 13/13 (§3.3).

## Próximo passo — W4.4 (§3 da tabela; abra pelo gate vermelho)

1. **Onde guardar:** `VecScene.regras: Vec<RegraCad>` apendado ao FIM (postcard posicional), no degrau 24; o leitor v23
   dá `vec![]`. Leia `ph2d-vec-scene/src/schema.rs` e o `WireLayout` antes.
2. **Apagar um vértice tira as regras que o citam no MESMO passo de undo** (aberto da W4.2); idem path apagado. Um
   censo por sítio do `ph2d-vec-edit`/`ph2d-vec-boolean` só se os tocar (o explorador já os leu: copiam o vértice ou
   re-geram a forma).
3. **Arrasto:** em `ph2d-vec-edit/src/pen_drag.rs` / grab (`lib.rs`), DEPOIS do snap; a folha = os paths citados pelas
   regras com o `Xform` de cada um (`VecXforms`); `palpite` = a solução do passo anterior; `Solucao::aplica` em cada
   path; um círculo vivo actualiza a receita (`resize_recipe`, `w = h = 2r`) e o `Transform`. Undo = 1 passo por gesto.
4. **Inferência:** a guia acesa ao largar (`ph2d-app-vec/src/snap.rs:148`, `ph2d-vec-edit/src/snap.rs:329`) cria a regra;
   seam `draw_with_a_guide_creates_the_rule` (`ph2d-ui-testkit`). Medida a mais ⇒ `estado` diz `Redundante` ⇒ nasce
   `referencia: true`. Conflito ⇒ lista as regras `Conflito`. `estado` fora do fio do quadro (só quando a ESTRUTURA muda).
5. Cores por token (zero hex), textos por `ph2d-i18n`; nada na shell (`the_shell_only_shrinks`).
6. **Medir** o pior passo de `resolve` no app (escada 202 / misto 200) com a tabela ao lado — nunca medido ainda.
7. Depois W4.5 (medida animada, canal `CONSTRAINT = 8`, cena 6 com smoke do dono — §5 do 04).

## Armadilhas

- `git commit -- <paths>` não leva ficheiro por rastrear: na W4.3 o 1.º commit da crate saiu sem `circulo.rs`/`posto.rs`
  e foi emendado — `git add` antes, e `git show --stat HEAD` depois.
- `cargo test` cru é recusado pelo guarda: `bash scripts/ph2d-run.sh cargo test …` ou `cargo-test-narrow.sh` (um nome por
  chamada).
- Os índices de conflito das fixtures são das regras da BANCADA; a tradução junta `Fixo` e `Raio` no fim — compare só
  `[..n_regras_da_cena]` (`tests/it/estado.rs` `com`).
- O tecto de LOC de 700 morde os ficheiros que a W4.2 engordou: conte antes de comitar (`wc -l`), cure cortando.
