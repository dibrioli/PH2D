# Continuação — `line/BlenderConstraints`, onda W4 (CAD no vetor), 26/10/2026 — W4.4 FEITA, abre a W4.5

> Anterior: [`W4.4`](HANDOFF_CONTINUACAO_W4_4_2026-10-26.md). Doc da onda: [`04_W4_cad.md`](../04_W4_cad.md) — **§3.4** (o que
> a W4.4 fez, a tabela medida no editor e o ABERTO), §4 (gates ✓), §5 (cena 6). **Não duplicado aqui.**

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4` (o `main` não andou
até 26/10 à tarde). Commits da W4.4: `8bf2b5a9ea` regras na cena + poda · `6c8f2e3e94` CAD na caneta · `55f7726b75` shell +
círculo vivo · `d96f9d095d` cores e ficha do conflito · `fec70812d5` sonda · `dcfe52cdf0` posto por componente ·
`04b255bd81` sobreviventes da mutação · o commit deste handoff. `PROJECT_SCHEMA` = **195** (reconta-se na integração),
`VEC_SCENE_SCHEMA_VERSION = 24` (só nesta linha; `VecScene.regras` entrou no MESMO degrau).

## Feito na W4.4 (não refazer)

Ver §3.4. Em uma linha: `VecScene.regras` + poda no assentamento dos ids · `SnapAxis.dono` · `PenTool::on_press_encaixe`/
`on_drag_encaixe` (solver depois do snap, regra da guia acesa ao soltar) · `junta_regra` (porta única; medida a mais nasce
REF, geométrica a mais não entra) · `ph2d_vec_entities::receita` (círculo vivo assenta receita + `Transform`) ·
`ph2d_app_vec::cad_cores`/`cad_marks` (cores fora do quadro, ficha «Conflict — delete one of: …»).
Mutação: 1.ª ronda 12/18 (+M12 sobrevive por desenho: só perf); M7 era equivalente (o palpite saiu do `CadGesto`); M10,
M11, M15, M16, M17 ganharam o caso (`04b255bd81` + o commit do M16) e **re-provados: 5/5 sangram**. Total **17/18** (M12 = só perf, exacto).

## Portões por nome (verde no fim da W4.4)

Os 18 de §4 marcados W4.4, `the_shell_only_shrinks` (o teste da shell encolheu: tecto apertado — cada linha nova paga-se),
`workspace_src_files_under_loc_cap`, `no_dependency_climbs_a_layer`, lints, `the_stack_versions_page_matches_the_lockfile`,
`cargo deny`, i18n (app-vec e shell), hex, `the_net_knows_every_derived_writer…`, `the_tree_settles_before_the_capture`,
`the_node_ops_are_wired`, v23/v194. ⚠️ **Vermelhos HERDADOS, não desta linha** (já no `d727ca64fb`):
`shell_files_respect_hr18_loc_cap` (`despacho_metodos_picks_e_arrastos.rs` 611, `project_tests.rs` 604),
`shell_functions_respect_their_ceiling` (`fase_snapshots_publish`, `fase_vector_overlays` 201) — do integrador.
`no_magic_numeric_in_widget_or_screens` reprovava em `ph2d-panel-inspector/src/sync_constraint.rs` (W3.2, desta linha):
**curado** no commit deste handoff (razão escrita por linha: conversão por cento e passo em segundos).

## Próximo passo — W4.5 (§3 linha W4.5, §4, §5)

1. **Medida animada:** a medida de uma `Lei::{Distancia, Angulo, Raio}` é um canal da família `CONSTRAINT = 8` (`campo` =
   valor, `i:j` = id do item — decida e escreva o id durável da regra: o índice em `regras` parte ao apagar; candidato:
   id por regra apendado a `RegraCad` NO MESMO degrau 24). Chaves pela porta `ph2d_app_constraints::chaves::gravar` (um
   passo de undo). Gate vermelho primeiro: `an_animated_dimension_plays_and_scrubs_to_the_same_bit` (3 chaves, quadros
   0→40→12→40).
2. **A pose parte SEMPRE do desenho AUTORADO** em cada quadro (não do quadro anterior) ⇒ tocar = arrastar a timeline ao
   bit. Onde o resultado vive (preview ≠ documento, `ph2d-preview-drive`) — leia antes de escrever.
3. **Cena 6 «Régua de desenho»** (§5; o roteador `PH2D_CONSTRAINT_SMOKE` responde até 5, `ph2d-app-constraints/src/smoke.rs`):
   perfil de dentes com H/V, larguras iguais e uma medida animada; nome `smoke_*`; **fotografe antes** (memória: nunca
   `spectacle` em `kwin --virtual`; `import -window`). Números antes da mensagem: violação ≤ 1e-4 px em todos os quadros,
   play × scrub = 0, pior passo do arrasto no app (§3.4).
4. ABERTO da W4.4 (§3.4): juntar uma regra numa componente de 200 pontos custa 17–33 ms no quadro em que a guia acende —
   cura desenhada: base do posto em cache pela análise que já corre fora do quadro.
5. Ainda sem UI para criar uma regra À MÃO (seleccionar dois pontos → H/V/medida) e para apagar uma regra da ficha de
   conflito: a cena 6 precisa de pelo menos «apagar a regra» para o dono resolver um conflito — confirme no plano
   (`01_plano.md` §7 W4) se é W4.5 ou onda própria.

## Armadilhas

- `git commit -- <paths>` não leva ficheiro por rastrear: `git add` antes, `git show --stat HEAD` depois.
- `cargo test` cru e `cargo fmt` da shell: o fmt reformatou `vec_tree_settle.rs` (W4.2) e a shell crescia 3 linhas — a
  linha foi reescrita para caber (`b.vec.clone()`); corra `rustfmt --check` no ficheiro da shell antes de comitar.
- Um teste por chamada no `cargo-test-narrow` (nunca num `for`); sonda de velocidade em `--profile smoke` via
  `ph2d-run.sh` (`cad_medida_tests.rs`, `#[ignore]`).
- `ph2d-ui-testkit` não conduz o canvas: seam do canvas = teste na crate com o MESMO snap + gate de texto na shell.
