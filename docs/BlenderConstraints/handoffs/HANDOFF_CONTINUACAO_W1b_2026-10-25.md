# Continuação — `line/BlenderConstraints`, onda W1 (2.ª janela → 3.ª), 25/10/2026

> Anterior: [`HANDOFF_CONTINUACAO_W1_2026-10-25.md`](HANDOFF_CONTINUACAO_W1_2026-10-25.md) (o mapa do cartão e
> as armadilhas medidas continuam válidas). Plano: [`01_plano.md`](../01_plano.md) · ADR:
> [`0192`](../../architecture/decisions/0192-a-pilha-de-constraints-e-um-componente-por-entidade-avaliada-da-pose-autorada.md).

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4`. HEAD =
o commit deste handoff, árvore limpa. Binário `target/smoke/ph2d-host-desktop` compilado (perfil smoke).

## Decidido nesta janela

- **IK na posição da pilha** (o dono delegou: «descubra o melhor e faça»): plano §1/§2.2, ADR-0192, memória
  `project_constraints_ik_runs_at_its_stack_position_2026_10_25`. Não perguntar de novo.
- **O `look_at` do Motion NÃO chama a lei da constraint** — medido: aproximação de Rajan f32 com gémeo WGSL
  (mesmo bit CPU/GPU) × `atan2` f64 exacto do oráculo; alvo = dono diverge (0+offset × pose intacta). ADR-0192 §5.
  O item 6 da lista anterior está FECHADO assim.

## Feito (commits `442cdbd669` · `b90a76959d` · `215ad758bc`)

| Peça | Onde | Prova (por nome, verde) |
|---|---|---|
| Família `ph2d-app-constraints`: `quadro()` (encena `PH2D_CONSTRAINT_SMOKE` 1×, drena a curva, avalia); a shell trocou `evaluate` → `quadro` | `src/lib.rs`, `smoke.rs` | `the_shell_links_exactly_the_registered_families`, `no_two_families_claim_the_same_router`, staleness |
| Cenas 2 (olho: Copy Location + Limit Distance, seta Look At) e 3 (ponteiros: limite 170°→45°, meia cópia → 180°) | `smoke.rs` | `the_pupil_never_leaves…`, `the_limited_pointer…`; fotos conferidas; **smoke do dono OK** |
| Canal `CONSTRAINT = 8` (FORCA=1, LIGADA=2; `i` = bits baixos do `Item::id`, `j` = altos — o grupo parte-se por `i`) | `ph2d-ecs/src/canal_constraint.rs`, `ph2d-app-constraints/src/canais.rs` (aplicar dentro de `quadro`, publicar em `fase_autokey`) pelo ledger `PreviewDrive.fields` | `a_forca_animada` (2), `canal_constraint` (2) |
| Cartão: vocabulário `constraint_edits.rs` (8 tipos no menu, `MAX_LINHAS = 32`), painel (`ids/inspector_constraint.rs`, populate/event/sync/`sections/constraint.rs`), família `inspector.rs` (info, `apply_all`, `escolher_alvo`), losango `marcas.rs`, eyedropper do palco generalizado na shell | ver commit `215ad758bc` | `seam_constraints` (6), `o_cartao` (4), `todo_campo_pintado_com_losango…`, `architecture_panel_wiring_parity`, `the_shell_only_shrinks` (189 021 ≤ 189 041) |
| Pegar e largar um Child Of sem salto: ligar re-acerta o desvio contra a pose VISTA; largar insere em `Adotar` e a fase não conduz o dono 1 quadro (o `settle` esquece-o, a pose vista vira autorada) | `inspector.rs::liga`, `ph2d-constraints-ecs/src/avalia.rs` | `a_child_of_picks_up_and_drops_without_a_jump` |
| ADR-0192 | `docs/architecture/decisions/` | — |

## Falta na W1 (por esta ordem)

1. **⛔ REPORT DO DONO no smoke (25/10):** *«em Limit Rotation, onde estão configuradas as limitações?»* — o
   cartão não mostra os parâmetros de cada tipo. Acrescentar por item, debaixo da força: **Limit Rotation**
   Min/Max em graus · **Limit Distance** distância + Inside/Outside/Surface · **Copy** espaço World/Local e
   Replace/Add · **Look At** desvio · **Child Of** botão «Reset offset». Cada número com canal (família 8:
   campos novos por APPEND, ex. `LIMITE_MIN=3`, `LIMITE_MAX=4`, `DISTANCIA=5`, `DESVIO=6`) e losango; vocabulário
   em `ConstraintFieldEdit` (append). Seam test + `o_cartao` para cada um.
2. **Gizmos no palco**: linha ao alvo; arco do Limit Rotation com as duas pontas ARRASTÁVEIS (arrastar edita o
   campo do item 1, e o campo move o arco); círculo do Limit Distance arrastável. Molde
   `ph2d-app-skeleton/src/bone_limit.rs:107` + `bone_pose.rs:65` (`drag_edge`); o gizmo da luz
   (`ph2d_app_light2d::gizmo`, chamado em `fase_selection_highlight.rs:147`) é o molde da chamada na shell.
3. **Chave de compensação ao largar com Auto Key**: largar escreve a pose vista como autorada no quadro f; com
   Auto Key ligado falta a chave do Transform em f−1 com a pose ANTERIOR (senão, um objeto sem curva fica com uma
   chave só e a cena inteira mostra a pose largada). Depois disto, a **cena 1** (mão pega a espada no 24, larga
   no 48: salto 0 px medido pela sonda) e o tecto do roteador fica verde
   (`every_level_up_to_the_router_ceiling_answers` está VERMELHO de propósito até lá).
4. **Bake** de um intervalo para chaves (porta do Auto Key) + força 0 no intervalo; gate
   `the_bake_equals_the_live_stack_at_every_frame` (48 quadros, ao bit).
5. **`PROJECT_SCHEMA`**: decidir pelo teste de ida-e-volta se o componente novo pede o degrau `193 → 194`
   (precedente: a Luz 2D registou componentes SEM degrau; o `LuauScript` subiu). Se subir: escada +
   tripla em `project_schema_tests.rs:318` (três sítios).
6. Provas de mutação (`blend` arco curto, `ordem`, ciclo, `base` autorada, `Adotar`, `chave` i/j) e
   `/pd-onda-fechar`.

## Armadilhas desta janela

- `bind_on_create` com um `PreviewDrive` VAZIO parte da pose viva: é o certo para um item que nasce no FIM da
  pilha (vê o que os de cima deram); para re-alvejar um item do MEIO da pilha a pose viva é a saída da pilha
  inteira, não a entrada dele — aberto, sem caso de produto ainda.
- O guarda R5 lê um `for t in …; cargo-test-narrow $t` como «crate inteira»: corra cada teste numa chamada.
- O fish não parte `$files` em palavras: commits de muitos caminhos por `--pathspec-from-file`.
- A shell tem folga de 20 linhas (189 021 / 189 041); toda linha nova continua a pagar-se.
