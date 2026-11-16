# HANDOFF de continuação — line/JointEditor, W4k–W4l feitas (15/11/2026)

Leitor: a próxima LLM da linha. Ramo `line/JointEditor`, worktree `Worktrees/line-JointEditor`, base = main `7b3dd983f7`, sem rebase,
sem integrar. Anterior: [HANDOFF_CONTINUACAO_W4j_2026-11-15.md](HANDOFF_CONTINUACAO_W4j_2026-11-15.md) (W4h–W4j). Plano: [01_plano.md](../01_plano.md).

## 1. Commits desta jornada (`git log --oneline 10b202a97c..HEAD`)

| sha | o quê |
|---|---|
| `294ba66281` | W4k: os campos do painel = os do cartão do Inspector. A: `compose` (Commit do IME) chegava a `false` → `type_char`. B: `Drag { field, value, by }` + `doc.live(f)`; a alça anda no grão do campo (`step_of`). C: `Hit::Scrub(f)` + `field_scrub::FieldScrub`; o valor pintado vai a `SpinButton.value` (a11y). Costuras `tests/it/campos.rs` |
| `267778beeb` | W4k: a seta do motor é uma espiral sem tecto (`MOTOR_SPIRAL_GROWTH`); `Geo::value_near` conta as voltas no arrasto. Costura `a_espiral_do_motor_passa_da_meia_volta` |
| `f6bdc13f15` | W4l: «Make the parts». Porta única `ph2d_joint_assembly::joint_factory` (`make_parts`, `block`, `centre`/`centre_at`, `shift`, `middle`, `count_of`, `free_suffix`) — usada pelo botão, pela cena 123 e pelo `parts_of` dos testes. Campo «Planks»/«Links» (`Field::Count`) no rascunho; `Edit::Make` → peças no meio da câmara (`hero.grid.view`), `assemble`, todas selecionadas, aviso no pé e num toast (`frame(sim, hero, toasts)`). O poste preso à ÚLTIMA da fila anda (`shift`), no Make e no desenho. Nomes: sufixo da máquina inteira; «Bridge 2». Cena de smoke 14 (ponte por fazer) |
| `21dd51221c` | `doc.rs` volta a 600 linhas |

Smoke do dono: os 3 defeitos e a espiral **OK** (15/11). «Make the parts»: smoke entregue (cena 14), sem resposta ainda.
Fotos: `docs/JointEditor/fotos/w4k_*.png`, `w4l_a_ponte_no_editor.png`, `w4l_b_ponte_na_cena.png`. Fotos de gesto real:
`docs/JointEditor/ferramentas/fotos_gesto.sh` (Xvfb; `move/down/up/type/key/foto`).

Provas verdes: crates inteiras (código movido) `ph2d-joint-assembly` 22, `ph2d-app-joint-editor` 37, `ph2d-physics-smoke-scenes` 108;
`ph2d-app-physics joint` 180. Mutações: W4k 4/4 + espiral 3/3 + W4l 9/9 sangram.

## 2. ABERTO (por ordem)

1. ⭐ **Pedido do dono (15/11, depois do smoke): os parâmetros da junta no painel de CRIAÇÃO.** Hoje o rascunho mostra só «Planks» e
   «Make the parts». Fazer: o rascunho mostra os MESMOS cartões da montagem feita (Limits/Motor/Breaks/…, `panel::groups_of`) com os
   números de fábrica (`JointRule::joint(0.0)`, como `paint_stage` já faz com `fab`) e o MESMO «Applies to» (`scope_options`: «Plank»
   = todas as juntas repetidas da fila; «All N»); as edições guardam-se no rascunho por regra e o `Edit::Make` leva-as (aplicar às
   juntas com `joint_with_edit` logo depois do `assemble`). Vale para todo gabarito com papel de várias (ponte, corrente). Prova: costura
   que muda um número no rascunho com «Plank» e confere as 11 juntas da ponte feita; foto; smoke.
2. **«Planks» numa montagem JÁ feita refaz as cópias** e «N peças a partir de UMA solta» (plano §W4, 2.º ponto): não feito.
3. **Fragilidade medida:** a ponte da cena 123 cede (caixa de 10 kg no chão) se a 7.ª tábua mudar 1e-6 m (3,539999 → 3,54; a ordem da
   conta). `joint_factory::centre_at` mantém a ordem antiga para o gate `the_template_scene_shows_four_named_machines_that_work`; a causa
   física (a corrente `one_chain` à beira do limite?) está por investigar — não silenciar.
4. Decisão do dono: o cadeado de uma pasta COMUM tranca o conteúdo? (hoje só a montagem).
5. Fecho da linha: corte de LOC (`the_shell_only_shrinks_and_every_app_crate_too`: `ph2d-app-joint-editor` e `ph2d-joint-assembly`
   acima do tecto — cresceram mais nesta jornada; cura = cortar/mover); provas de mutação da W4h/W4i (lista no handoff W4j §4.4);
   clippy/fmt das crates tocadas; censos; handoff de integração §1.5.9 e a linha do §5 do CLAUDE.md (smokes `PH2D_JOINT_EDITOR_SMOKE=1..14`).

## 3. Prompt de continuação (cole na janela nova)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && pwd && git branch --show-current
```
Confirme `line/JointEditor`. Leia este handoff (`docs/JointEditor/handoffs/HANDOFF_CONTINUACAO_W4l_2026-11-15.md`) e o anterior.
Comece o §2.1 (parâmetros da junta no painel de criação, «Applies to: Plank» = um ajuste para todas as repetidas), depois §2.2. Inner
loop = `cargo-check-narrow.sh`; testes por nome; foto de gesto real com `fotos_gesto.sh`; smoke em passos numerados; sem push.
