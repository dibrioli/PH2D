# Continuação — `line/BlenderConstraints`, W5.0 FECHADA (vermelhos e ABERTOS curados), o código da W5 por escrever

> Anterior: [`W5`](HANDOFF_CONTINUACAO_W5_2026-10-26.md). Desenho da W5: [`05_W5_diferenciais.md`](../05_W5_diferenciais.md)
> (§0 decisões do dono, §6 ordem do código). Fecho dos ABERTOS: [`04_W4_cad.md`](../04_W4_cad.md) §3.8. **Não duplicado aqui.**

## Identidade

Mesmo worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4`.
`PROJECT_SCHEMA` = 195 (ainda; reconta-se na integração), `VEC_SCENE_SCHEMA_VERSION = 24`, família de canal `CAD = 9`,
cena de smoke 7 «Caixa com tampa»; **a próxima cena é a 8** («Braço articulado», 05 §5).

## Feito nesta janela (não refazer)

Commits desde `cbf0a45ff2`:

- `12ca761771` — as fixturas de `ph2d-app-components` levam `register_constraint_components`; porta `constraints`
  armada/desarmada em `o_inspector_armado.rs`; os scanners seguem o código movido (agulhas de
  `architecture_the_ik_chain_is_drawn.rs`, `os_verbos_do_osso…`, `the_pencil_owns_its_whole_gesture` com
  `self.vec.pen.on_press_encaixe`); `envelope_live`: a fonte entra pela cena (`push_path` numera `1..=n`) ⇒ a expectativa
  normaliza os ids.
- `55ce0b0982` — `ids/inspector_constraint.rs` (1327 LOC) partido em `inspector_constraint.rs` (470) +
  `inspector_constraint_opcao.rs` (OPCAO_0..2) + `inspector_constraint_opcao_b.rs` (OPCAO_3..4), mesmos slugs;
  `("constraints", …)` na PORTAS. Os testes v189–v192 da shell leem bytes REAIS gravados pelo código v193
  (`c3662cd8e4`): `shells/desktop/tests/fixtures/projecto_{marcas_v191,marcas_v192,opacidade_v191,scripts_v189,scripts_v190}.ph2dproj`.
  ⛔ Nunca regenerar (o `VecScene` vivo ganhou regras/`next_regra` e o `VecPath` ids). Shell encolheu ~50 linhas.
- `6372320ace` — ABERTOS 1, 2, 3 e 7 da W4.6 (cura e gates em `04_W4_cad.md` §3.8). 4/4 mutações sangraram.
- `b973f4cd9d` — `05_W5_diferenciais.md`.
- `00fa889981` — smoke do dono 27/10 («as linhas das shapes piscam»): `CadCores` (`ph2d-app-vec/src/cad_cores.rs`) mostra
  a última análise da MESMA estrutura até a nova chegar. Smoke do dono OK (os 14 vermelhos, os abertos e o piscar).

## Próximo passo

**W5 — o código**, na ordem de `05_W5_diferenciais.md` §6: (1) ler a porta do gizmo de objecto e o undo do gesto ·
(2) oráculo (triagem de licença → script → fixture) · (3) crate `ph2d-constraints-cola` com gates, red-first · (4) o
`quadro()`, o gesto, `AlvosDasFamilias`, gravar/abrir (+1 schema), ADR · (5) UI, cena 8, kill da cola e kill da pilha na
MESMA rodada. As decisões do dono estão no §0 daquele doc; perguntar só o que for produto.

## Portões por nome verdes

`o_registo_das_fixturas_e_o_do_produto` · `every_buildable_authored_component_is_reachable_from_some_palette` ·
`the_authored_source_survives_in_each_child` · `panel_files_under_loc_cap` · `changing_the_chain_recaptures_the_bend_side` ·
`the_shell_only_shrinks` · `o_campo_do_desvio_chega_ao_componente_em_radianos` ·
`o_espelho_pergunta_a_porta_e_nao_reconstroi_a_conta` · `the_pencil_press_runs_before_the_pen_and_the_shape` ·
`the_scanner_finds_what_it_scans_for` · `toda_porta_do_inspector_e_armada` · `toda_porta_do_inspector_e_desarmada` ·
`a_tabela_de_portas_cobre_todas_as_portas` · `o_inspector_abre_dentro_da_dobra` · `toda_semeadura_de_dobra_passa_pela_porta` ·
os 5 de `node_id_collisions` · `um_v191_e_um_v192_abrem_sem_marcas` · `um_v191_abre_com_a_opacidade_da_forma_nas_poses` ·
`a_v189_…`/`a_v190_project_with_a_script_path_imports_it_and_runs` · `as_marcas_do_artista_atravessam_o_arquivo` ·
`a_v190_project_keeps_the_script_text_and_the_reference` · os 6 dos ABERTOS e os 2 do `CadCores`.

Herdados do `main` (do integrador, NÃO curar): `shell_files_respect_hr18_loc_cap`, `shell_functions_respect_their_ceiling`.
⚠ **NÃO** se re-correu o `nextest-impacted` inteiro (cadência): corre no fecho da W5 com `PH2D_SUITE=onda`.

## Armadilhas

Herdadas da W5:
- `git commit -- <paths>` não leva ficheiro por rastrear: `git add` antes, `git show --stat HEAD` depois.
- `rustfmt --check` no ficheiro da shell antes de comitar; um teste por chamada no `cargo-test-narrow`.
- O hook recusa comandos cujo texto nomeia programas de placa (`kwin_wayland --virtual`): `ps -eo pid,comm | grep`.
- A entidade do path tem de existir antes de gravar chaves (o `step` da cena espera-a).
- A população do Auto Key é seleção + mãos + `AlvosDasFamilias`: uma tool que edita o que não está selecionado publica
  os alvos ANTES do gesto.
- Sonda temporária no app real (env var + `fotografa_cena.sh` com `FOTO_RAJADA`): comitar o WIP antes, `git checkout` depois.
- Tecto da shell: folga ~50 depois desta janela (reconfirme com `the_shell_only_shrinks`); a cola não põe linhas na
  shell — vive em `quadro()` de `ph2d-app-constraints`.

Novas:
- Ficheiro antigo para teste de degrau = bytes REAIS do código de então (worktree temporária no commit antigo, despejar os
  bytes do próprio teste, `git worktree remove --force` depois); nunca o tipo vivo com número velho.
- `mod x;` dentro de `ids/inspector_constraint.rs` procuraria `ids/inspector_constraint/x.rs`: os irmãos declaram-se em `ids.rs`.
- Análise assíncrona chaveada pelo VALOR pisca sob animação: chaveie a ESTRUTURA para mostrar, o valor para pedir.
- Para a W5 (05 §1/§2.4): a pilha é de um sentido (ADR-0192), ciclo ⇒ `Estado::Ciclo` em `ordem.rs:60-82`; a porta que
  grava o `Transform` no arrasto do gizmo de objecto NÃO foi achada (`input_dispatch/gizmo_drag.rs:45`
  `advance_gizmo_drag` é o driver) — 1.º passo do código.
- O kill-criterion da pilha (0,5 ms / 1 000) nunca foi medido: medir na mesma rodada do da cola.
- Oráculo FreeCAD 1.1.4 Assembly (Fixed/Revolute, `assembly.solve()`): `libOndselSolver.so` sem LICENSE no pacote —
  triagem antes de correr. `kcl-ezpz` tem Distance, PointsCoincident, Fixed, LinesAtAngle e prioridade (`solve_with_priority`).
