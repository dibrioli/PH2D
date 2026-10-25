# Continuação — `line/BlenderConstraints`, onda W1 (3.ª janela → 4.ª), 25/10/2026

> Anteriores: [`W1b`](HANDOFF_CONTINUACAO_W1b_2026-10-25.md) (estado até o cartão) · [`W1`](HANDOFF_CONTINUACAO_W1_2026-10-25.md)
> (mapa do cartão e armadilhas). Plano: [`01_plano.md`](../01_plano.md) · ADR
> [`0192`](../../architecture/decisions/0192-a-pilha-de-constraints-e-um-componente-por-entidade-avaliada-da-pose-autorada.md).

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4` (o `main`
não andou nesta janela: `git log HEAD..main` = 0). HEAD = o commit deste handoff, árvore limpa. Binário
`target/smoke/ph2d-host-desktop` compilado. Shell **189 032 / 189 041** (folga 9).

## Smoke do dono nesta janela: **OK** (cenas 2 e 3)

Reports dele, todos fechados: *«em Limit Rotation onde estão as limitações?»* (item 1) · *«Distance em px mas o
valor é metros»* e *«há um seletor px/metros nas configurações e precisa ser obedecido»* (`834db91b38`) ·
*«Inside e Surface na mesma posição»* (correcto: com a bola FORA do olho os dois dão a borda — sonda
`inside_and_surface_differ_only_with_the_ball_inside_the_eye`) · *«Outside, a pupila fica sempre na bola»* →
ele próprio concluiu *«não entra no círculo, talvez esteja certo»* (sonda `outside_pushes_the_pupil_to_the_edge…`).

## Feito (commits `109255e156` · `4937245088` · `d55894fb57` · `140f3827a0` · `834db91b38`)

| Peça | Onde | Prova (por nome, verde) |
|---|---|---|
| **Ajustes por tipo** no cartão: Limit Rotation Min/Max + Space · Limit Distance Distance + Mode · Copy Mix + Space · Look At Offset · Child Of «Reset Offset» (= offset identidade, cola ao alvo) | porta única `ph2d-app-constraints/src/campos.rs` (que campos, ler/escrever na unidade do cartão, chave serde no item); vocabulário `AjusteNumero/AjusteEscolha/Unidade{Graus,Metros}`, `ConstraintFieldEdit::Campo/ZerarDesvio`; painel `sections/constraint.rs::ajustes`, `event_constraint.rs`, `sync_constraint.rs`; ids LITERAIS `INSP_CONSTRAINT_NUMERO/OPCAO/ZERAR` | `seam_constraints` (8), `o_cartao` (5), `todo_campo_pintado_com_losango…` (entrada POR_FUNCAO `constraint.rs::ajustes`), `node_id_collisions` (14) |
| **Canais** por APPEND na família 8: `LIMITE_MIN=3 LIMITE_MAX=4 DISTANCIA=5 DESVIO=6` (contínuos, graus/metros) · `MODO=7 ESPACO=8 MISTURA=9` (Opcoes); a curva conduz qualquer campo pelo ledger (`items[i]/kind/<Variante>/<campo>`) | `ph2d-ecs/src/canal_constraint.rs`, `canais.rs` | `canal_constraint` (2), `a_curve_drives_a_setting…` |
| **Unidades do projecto**: Distance px/m, ângulos graus/rad — o painel converte (`no_ecra`/`do_ecra` lêem `host.project()`: a pintura semeia ANTES de pôr as thread-locals); `unidade_e_passo` | `sync_constraint.rs` | `os_numeros_obedecem_a_px_metros…` |
| **Gizmo no palco**: linha a cada alvo, ARCO do Limit Rotation com 2 pontas arrastáveis, CÍRCULO do Limit Distance com a pega do raio (lado OPOSTO ao dono); `pecas` = a única conta (desenho e clique); `gesto(Premir/Mover/Largar)` = a única porta da shell; estado em recurso `Segurando` | `ph2d-app-constraints/src/gizmo.rs`; shell: `despacho_clique_select.rs` (Down e Up), `despacho_mover.rs`, `despacho_metodos_picks_e_arrastos.rs::constraint_gizmo`, desenho em `fase_vector_overlays.rs` (por cima das formas, doc 109 §6) | `o_gizmo` (2), `the_warp_gizmo_is_wired_to_the_pointer` (11), `the_shell_only_shrinks`; fotos cenas 2 e 3 |
| Cenas abrem com o sujeito ESCOLHIDO e o Inspector à vista (`smoke::escolhido`, consumido 1× em `fase_skeleton_drives.rs`) | `smoke.rs` | fotos |

## Falta na W1 (por esta ordem)

1. **Chave de compensação com Auto Key** (pegar E largar — o mesmo problema nos dois lados):
   - pegar no quadro f (força 0→1): sem chave em f−1 a curva da força tem UMA chave (1) e a espada fica presa à
     mão desde o quadro 0. ⇒ FORCA: f−1 = valor anterior, f = novo.
   - largar em f (1→0, `Adotar`): FORCA f−1 = 1, f = 0; **Transform** f−1 = pose autorada de ANTES (a curva em
     f−1, ou o `Transform` autorado sem curva), f = a pose largada (vista).
   - Mapa medido nesta janela (explorador): o Auto Key é por COMPARAÇÃO, sem gesto — `fase_autokey.rs:41-73`
     publica (`canais::publicar` :60) e corre `autokey_pass::run` (`armed = painel timeline aberto && flags.auto_key`,
     `autokey_pass.rs:78`); `ph2d-timeline/src/autokey_campos.rs:29` compara com a curva; sem curva só CRIA canal
     se o valor mudou desde o quadro anterior (`allow_create`). Chaves = `TimelineIntent::AddKey { entity, prop,
     t: RationalTime, value: AnimValue, interp }` (`intent.rs:86`, aplicado em `intent_apply.rs:130`), drenadas
     por `self.timeline_intents` (`app_state.rs:125`); Transform = `PropKind::{TranslationX,TranslationY,Rotation,
     ScaleX,ScaleY}` (`prop.rs:21-33`); `t` em tempo de CLIP (`apply.rs:334,347` `key_home/key_time`);
     `default_interp` e `chave_da_k` em `timeline_bridge_keys.rs:~315/380`. Precedente: nenhum (o
     `ph2d-app-flip/src/reshape.rs:33` documenta uma compensação RECUSADA — ler o porquê antes).
   - Desenho proposto: a família devolve `Vec<TimelineIntent>` (como `marcas::clique`) de uma função
     `compensar(doc, sim, edits, inst)` chamada onde o commit do cartão corre; ⚠️ medir se o `autokey_pass` da mesma
     passada já grava f e só falta f−1 (e se o `Mudado` do item interfere).
   - **Cena 1** (mão pega a espada no 24, larga no 48) no molde dos smokes «conduzidos»
     `ph2d-app-components/src/autokey_smoke_componentes.rs` (passo por quadro com `sim, hero, tl, ph`; liga
     `tl.flags.auto_key`, `ph.seek`, mexe, mede) — gancho de 1 linha no `.or_else` de
     `render_loop/fase_object_mode.rs:37`. Sonda: salto 0 px no 24 e no 48. Fecha o vermelho
     `every_level_up_to_the_router_ceiling_answers` (subir `max_level` para cobrir o 1 — hoje 3 com o 1 mudo).
2. **Bake** de um intervalo para chaves (porta do Auto Key) + força 0 no intervalo; gate
   `the_bake_equals_the_live_stack_at_every_frame` (48 quadros, ao bit).
3. **`PROJECT_SCHEMA`**: decidir pelo teste de ida-e-volta (precedente: Luz 2D sem degrau; `LuauScript` subiu).
4. Provas de mutação (`blend` arco curto, `ordem`, ciclo, `base` autorada, `Adotar`, `chave` i/j, `campos::escrever`
   Min≤Max, `gizmo::arrastar` arco curto) e `/pd-onda-fechar`.

## Armadilhas desta janela

- O censo `node_id_collisions` recusa ids gerados por `const fn` (forma não-literal) — tabelas LITERAIS, geradas por
  script com `assert` de contagem.
- O gate do losango lê id numa VARIÁVEL como sem canal: entrada `POR_FUNCAO` com classe `Canal` e os nomes das tabelas.
- A shell conta comentários: o gizmo (~35 linhas) pagou-se condensando dois blocos de histórico
  (`fase_selection_highlight.rs`, `fase_vector_overlays.rs`, `despacho_*`). Folga agora: **9**.
- `fase_selection_highlight` corre ANTES da arte das formas do Motion (no Vello fica por baixo) — gizmo de família
  vai na `fase_vector_overlays`.
- `ColorToken::Selection` lê-se azul-escuro sobre o cinza do palco: o gizmo usa `Accent`/`AccentSoft`.
- ⚠️ Visto e NÃO mexido (outro módulo): o cartão da Luz 2D fixa `Unit::Meters` (`sections/luz.rs:230/265`) e não
  obedece ao seletor px/metros — anotar para a linha da Luz 2D.
- O guarda R5 lê `for t in …` como crate inteira; um teste por chamada. O fish não parte `$files`: commits por
  `--pathspec-from-file` ou `git add -A -- <pastas>`.
