# HANDOFF de integração — `line/GameEngineFuncional02`, 08/11 (opção B: a forma de colisão é um objeto FILHO do corpo)

Leitor: o agente integrador. **Este handoff é para uma integração FUTURA da ponta da linha**, depois de o `line/GameEngineFuncional02-ponto` (`175d7debe2` = `b24dc55400` + cherry-pick `bac19f4e23`) já estar no `main`. O `-ponto` NÃO tem a opção B; não o toque. Faça rebase da ponta sobre o `main`: `bac19f4e23` já está no `main` pelo `-ponto` (cherry-pick) e deve **cair como vazio** no rebase. Handoff da física anterior: [FISICA](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-08_FISICA.md). Mecanismo: [continuação B](HANDOFF_CONTINUACAO_2026-11-08_OPCAO_B.md) · [ADR-0199](../../architecture/decisions/0199-a-forma-de-colisao-e-um-objeto-filho-do-corpo.md) · [plano](../componentes/f2_contorno_e_formas_plano.md).

## 1. Identidade

Branch `line/GameEngineFuncional02`, HEAD `1afcd7077d`. Merge-base com o `main` de hoje: `652b506b58` (o `-ponto` ainda não entrou; o merge-base MUDA com o rebase). Opção B = tudo depois de `b24dc55400`: **43 commits**, 249 ficheiros, +5734 −1324 (`git diff --shortstat b24dc55400..HEAD`). Nada integrado, nenhum push. **Suíte completa NÃO correu** (regra do dono 23/10).

## 2. O que entra (por onda; `git log --oneline b24dc55400..HEAD`)

- **B1 motor** — `af721f6960` main_shape (forma principal = 1.ª da ordem da árvore) · `1722b33775` motor aceita a forma numa filha · `4446ad504b` `shape_pose` por libm (caminho do hash; gate `no_std_transcendental`).
- **B2 migração, schema 219 → 220** — `ea4476e22a` `migra_formas_para_filhos` · `018698763c` timeline `realveja_trilhas_da_forma` · `cbf8aba897` PROJECT_SCHEMA 220 (`migra_formas_no_retrato` em `project_load_read.rs`: um ≤ 219 abre com a filha «Collision Shape») · `4aaa7cdcf9` · `f9e1756738`.
- **B3 portas** — B3a `9b07186ea0` `nasce_com_forma` · `ef834629e2` · `af68cecfcc` (sai `own_body_note`) · `9b24d488df` todo corpo do editor nasce com filha · `222ccdae65` catraca das crates app (−3 components, −4 game). B3b `a93cc31c62` `torna_corpo`/`tira_corpo` · `ed04b0860c` Add Body/Make Independent + gate `nenhuma_porta_do_editor_poe_a_forma_no_corpo` · `45c4d9563f` · `45d2a0881c` Remove leva a «Collision Shape». B3c `dcbf5fcc45` campo de forma no corpo redireciona (`dono_do_campo`/`forma_para_editar`); `move_forma_para_filha`. Docs: `bac19f4e23` (fix, já no main), `c0d997a3c3`, `0f977f3c8e`, `c643652860`, `3d7e13629d`, `7c08d9ac70`, `9a0eec6a75` (ADR-0199).
- **B4 Inspector** — `3fffc42c22`: corpo só com o cartão Body; a filha mostra «Shape of», Shape + Collision Layers; «Move shape to a child» no corpo antigo. `InspectorPhysicsInfo` ganha `shape_on_child`, `shape_of`, `collides_override`, `collides_mixed`, `collides_differs`.
- **B5 canvas** — alças só com a forma selecionada; a caixa de seleção de entidade só-collider = AABB do collider (`collider_box_or`); `6b9dc4c567` a forma selecionada abraça a própria forma.
- **B6** — `1fa83686e0` zona sopra no frame do COLLIDER, efetor por collider (peças com área são zonas) · `a54813f89b` pontos de colagem compõem o ângulo da forma · `74e6383ba9` `ordered_children_iter` (não aloca) · `c06f9fd90e` corte de prosa histórica (tecto de LOC; sem código).
- **B7 cenas** — `847f41adfe` cena 120 + `monta_v219` (fixture), fotos dos cartões, fábrica, lava · `3f6bbb0977` as 23 cenas antigas de `ph2d-app-components` migram por `migra_formas_para_filhos` no fim de cada builder; instância-mestre · `323008c4c9` macros reproduzem `move_forma_para_filha` (aplica/palco_fisica/palco); forma solta passa a «Loose Shape» · `c372fd89e5` fotos.
- **B8 olho** — `17d762f8af`: olho da Hierarquia numa forma = esconde o contorno E não colide (inclui ancestral escondido); formas nascem com o olho aberto; desligar uma forma não apaga a outra.
- **B9 exceção por forma, schema 220 → 221** — `b14808575a` `CollisionOverride{include,exclude}` (decisão do dono 08/11, idioma Unity). Regra `pair_collides`: exclude de qualquer lado ⇒ nunca; senão matriz OU include de qualquer lado. Ganchos de filtro de par + bits do `user_data`; truque da linha vazia em `groups_for`. `384936b74b` consultas levam a exceção de quem pergunta · `531a8b7a41` cartão com multisseleção (`CollidesWith{layer,on}` absoluto, `UseLayerDefault`, chips mistos, resource `LiveLayerRows`, `estado_da_selecao`) · `778fb2fa58` o teste v220 junta-se ao v219 (ficheiro temporário partilhado).
- **Fecho** — `056463c1dc` (olho de ancestral desliga a forma; edição/chave vão à forma desligada; Add Body mantém o tipo; zonas no checkpoint; anel limpo ao ligar/desligar peça; guarda de entidade morta) · `d292c0160f` · `1afcd7077d` (fmt, clippy, tecto).

## 3. Superfície de colisão (colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02/scripts/collision-surface.sh`, 08/11; REFERÊNCIA, o integrador lê o `main` no ficheiro)

```
merge-base 652b506b58 · 178 commit(s) · 629 arquivo(s)   (contra o main de hoje, SEM o -ponto)
PROJECT_SCHEMA 221 (base 217) · tripla (221, 13, 29) (base (217, 13, 29))
VEC_SCENE_SCHEMA 29=29 · FLIP_SCHEMA 13=13 · DOC_VERSION 25=25
ph2d-ecs registo 120 (base 120); sondas render/script «cegas» (const mudou de ficheiro; não é desta linha)
node.rs / tool.rs intocados · ADR criado: 0199 (próximo livre no disco 0200)
Cargo.lock: +ph2d-outline-trace, +ph2d-shape-handles (internos)
marcadores de conflito: nenhum · LOC: shells/desktop/src/main.rs 1023/600 (pré-existente, isenção da shell)
```

**Contadores como DELTA sobre o `main` do dia** (o integrador RECONTA, §5.0):
- `PROJECT_SCHEMA`: **DELTA +2 desta linha na opção B** (220 = formas em filhas; 221 = `CollisionOverride`), por cima do 219 que o `-ponto` leva ao `main`. A tabela acima mede contra o `main` SEM o `-ponto` (217). Reconte: `python3 scripts/schema-recount.py <valor que o degrau passa a ter>`; escada e tripla em ficheiros irmãos (`project_schema*.rs`, `project_schema_tests.rs`: três sítios).
- Registo de componentes (`ph2d-physics-ecs/src/lib.rs`, `assert_eq!(reg.len(), …)`): **46 → 47** (`CollisionOverride`; DELTA +1).
- ADR: `0199` foi escrito com o `main` em `0198`: reconte (outra linha pode tê-lo tomado); renomeie o ficheiro e os links (`grep -rn 0199 docs CLAUDE.md`).
- Outline (planeado, não feito): o schema do Outline conta-se 222 depois desta linha.

**Contratos congelados (§6) NÃO tocados.** Prova: `git diff --stat b24dc55400..HEAD -- crates/ph2d-nodegraph crates/ph2d-vector-doc crates/ph2d-editor-core/src/tool.rs` → vazio. Gates verdes no fecho: `architecture_contract_surface` 3/3 (ph2d-nodegraph), `architecture_tool_contract_surface` 4/4, `architecture_vector_contract_surface` 11/11 (ph2d-vector-doc).

## 4. Foundational tocado, e por que é aditivo

| Onde | O que | Aditivo porque |
|---|---|---|
| `ph2d-ecs` | `ordered_children_iter` (nova); `row_flag` inalterado | função nova, mesma ordem de `ordered_children` |
| `ph2d-physics` | módulo novo `layer_override`; o checkpoint passa a incluir efetores e zonas (`056463c1dc`); truque da linha vazia em `groups_for` | byte-idêntico para cena sem exceção |
| `ph2d-physics-ecs` | fns públicas novas (`main_shape`, `shape_pose`, `nasce_com_forma`, `torna_corpo`, `tira_corpo`, `move_forma_para_filha`, `dono_do_campo`, `forma_para_editar`, `migra_formas_para_filhos`, `realveja_trilhas_da_forma`); `LADO_FORMA`; componente `CollisionOverride`; `BodyDesc.shape_pose` e `.collide` ACRESCENTADOS ao fim | campos novos: quem constrói `BodyDesc` à mão precisa deles (grep `BodyDesc {`) |
| `PhysicsFieldEdit` | variantes `MoveShapeToChild`, `CollidesWith`, `UseLayerDefault` | `match` exaustivo noutra linha precisa dos braços |
| `ph2d-app-physics`, `-components`, `-macros`, `-game`, `ph2d-panel-inspector`, `ph2d-panel-registry-init`, `ph2d-component-desc`, `shells/desktop`, `shells/game-scenes` | as portas, o Inspector, as cenas e a macro | consumidores |

## 5. Símbolos que podem colidir (grep na integração)

`CollisionOverride` · `PhysicsFieldEdit::{MoveShapeToChild,CollidesWith,UseLayerDefault}` · `InspectorPhysicsInfo.{shape_on_child,shape_of,collides_override,collides_mixed,collides_differs}` (sem `Default`: quem o constrói à mão ganha 5 campos; sai `own_body_note`) · `BodyDesc.{shape_pose,collide}` · `pair_collides` · `LiveLayerRows` · `ordered_children_iter` · nomes «Collision Shape» / «Loose Shape» · gate `nenhuma_porta_do_editor_poe_a_forma_no_corpo` · fixture `monta_v219` · cena `PH2D_PHYSICS_SMOKE=120` (e `PH2D_PHYSICS_SMOKE_SELECT`, `PH2D_GAME_SMOKE_FOTO=fisica_forma_caixa|fisica_camadas`).

## 6. Catracas de LOC (SOMAM entre linhas; o integrador reconta, cura = MOVER, nunca subir)

Depois desta linha: `ph2d-app-physics` **55 515/55 515** (folga 0; curada por prosa histórica encurtada em `c06f9fd90e`, sem código), `ph2d-app-components` **80 869/80 869**, `ph2d-app-macros` **14 630/14 630**, `ph2d-app-vec` ~1, shell no tecto (`the_shell_only_shrinks`; teste v220 fundido no v219, `778fb2fa58`). Se a linha irmã gastou folga nas mesmas crates, a soma estoura.

## 7. O que um merge pode partir

- Cenas de componentes de outra linha que construam corpos à mão: precisam de `migra_formas_para_filhos` no fim do builder (as 23 daqui já têm).
- Retrato salvo ≤ 219 abre pela migração; teste que compare bytes de gravação com o formato antigo muda (220/221 grava a forma em filha).
- Zonas somam agora na ordem de handle do COLLIDER (antes: do corpo): zonas sobrepostas podem mudar um fixture gravado.
- Conflito provável: `ph2d-physics-ecs/src/lib.rs` (registo, `reg.len()`), `project_schema*.rs`, `ph2d-panel-inspector` (cartão de colisão), `Cargo.lock` (`cargo metadata`, nunca à mão).

## 8. Prova de fecho (medida em 08/11 na linha; suíte completa NÃO corrida)

`cargo fmt --all --check` ✓ · `clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings` exit 0 (`1afcd7077d`). Gates: `file_loc_caps` 4/4 · `the_shell_only_shrinks` 2/2 · `architecture_panel_loc_cap` 4/4 · `arch_safe_clamp_only` 2/2 · `architecture_claude_md` 3/3 · `architecture_tool_contract_surface` 4/4 · `architecture_no_dependency_climbs_a_layer` 3/3 · `registers_every` 3/3 · `workspace_file_loc_cap` ✓ (depois de `1afcd7077d`) · `architecture_contract_surface` 3/3 · `architecture_vector_contract_surface` 11/11 · `no_std_transcendental` 2/2.

Testes da linha por nome (todos verdes; contagens):
- `ph2d-physics-ecs`: `forma_filha` 5 · `olho_da_forma` 3 · `zona_forma_filha` 2 · `migra` 4 · `shapes` 8 · `compound` 24 · `parts` 5 · `rewind` 38 · `determin` 6 · `override` 4 · `effector` 5.
- `ph2d-physics`: `layer_override` 9 · `zone` 29 · `checkpoint` 12 · `query` 14.
- `ph2d-app-physics`: `physics_tests` 25 · `inspector` 9 · `canais_` 33 · `alcas` 6 · `f1` 5 · `joint_rig` 22 · `outline` 29.
- `ph2d-panel-inspector`: `o_cartao_da` 83 · `os_quadros_da_colisao` 11 · `seam_physics` 43.
- `ph2d-game-scenes`: `jogo_object_add` 8 · `fisica_b4` 3 · `colide_` 1. `ph2d-host-desktop`: `physics_gesture` 12 · `um_v219_abre` 1 · `schema` 1 · `v194` 4. `ph2d-physics-scene`: `ficha` 6.
- `ph2d-app-macros`: `mundo` 35 · `aplicar_mundo` 9 · `cena_do_mundo` 3. `ph2d-app-components`: `nav_smoke` 21 · `vida` 59 · `instance` 160 · `factory` 13. `ph2d-ecs`: `sibling_order` 10. `ph2d-panel-registry-init`: `o_inspector_armado` 3. `ph2d-component-desc`: `catalog` 19.

**Auditoria (2 lentes):** achados corrigidos em `056463c1dc`; o item «Remove spread sobre corpo + filha» não se reproduziu (guarda mantida); simetria da matriz verificada segura.

## 9. Provas de mutação

12 mutações sobre 7 leis, cada guarda vista VERDE antes, árvore restaurada do `HEAD` depois de cada uma. **12/12 sangram.**

| Lei | Mutação | Guarda (por nome) |
|---|---|---|
| Nenhuma porta põe a forma no corpo | `nasce_com_forma` insere o `Collider` no corpo · `torna_corpo` sem `move_lado_forma` | `nenhuma_porta_do_editor_poe_a_forma_no_corpo` |
| idem | `move_forma_para_filha` sem `move_lado_forma` | `b4_move_shape_to_a_child_leva_a_forma_do_corpo_antigo` (a guarda acima não a apanha — sobreviveu a ela) |
| Olho fechado = não colide | `forma_desligada` sempre `false` · sem a subida aos antepassados | `olho_da_forma` |
| Exceção por forma | `pair_collides` sem o `exclude` · sem o `include` | `layer_override`, `query` |
| Forma filha ≡ forma no corpo | `shape_pose` zerada em `bridge/bodies.rs` | `forma_filha` / `zona_forma_filha` |
| Zonas no checkpoint | `checkpoint()` sem `effectors` · `restore()` sem `effectors` | `uma_zona_tirada_depois_do_checkpoint_volta_com_o_restore` |
| Edição chega à forma de olho fechado | `forma_para_editar` salta desligadas | `nenhuma_porta_…`, `olho_da_forma` |
| Remove leva a «Collision Shape» | `tira_corpo` sem `despawn` | `removing_a_body_detaches_both_components` |

## 10. Smokes (dono; passos)

Build: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && bash scripts/ph2d-run.sh cargo build --profile smoke -p ph2d-host-desktop`.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_PHYSICS_SMOKE=120 ./target/smoke/ph2d-host-desktop` (com `PH2D_PHYSICS_SMOKE_SELECT="Floor"` abre no corpo; `"Collision Shape"` na forma).
2. Clique no corpo «Floor» na Hierarquia: o Inspector mostra só o cartão Body (e «Move shape to a child» se for corpo antigo). Errado: forma ou camadas aparecem no corpo.
3. Clique na filha «Collision Shape»: aparecem «Shape of», Shape e Collision Layers, e as alças no canvas. Errado: alças com o corpo selecionado.
4. **PENDENTE (feito depois do smoke 1–6 do dono):** o olho da Hierarquia na forma — esconde o contorno E ela deixa de colidir (Play: o objeto atravessa); e **«Collides with»** da forma com multisseleção (duas formas, chips mistos, «Use layer default»). Mesmos comandos; cena das camadas: `PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=fisica_camadas`.

Smoke do dono 08/11: passos 1–6 da opção B **OK**. Fotos em [`fotos_fisica/opcao_b/`](fotos_fisica/opcao_b/): `01_corpo_floor.png` (`PH2D_PHYSICS_SMOKE_SELECT="Floor"`), `02_forma_do_floor.png` (`…_SELECT="Collision Shape"`), `03_forma_do_floor_alcas.png`, `04_corpo_sem_forma.png`, `05_foto_forma_caixa.png` (`PH2D_GAME_SMOKE_FOTO=fisica_forma_caixa`), `06_foto_camadas.png` (`PH2D_GAME_SMOKE=4` + `PH2D_GAME_SMOKE_FOTO=fisica_camadas`). **Foto do cartão da exceção: NÃO tirada.**

## 11. ABERTO

- Zonas somam em ordem de handle do collider (era do corpo): ver §7.
- O anel é limpo todo tick para uma peça cuja pose autorada é animada no play (o scrub fica mais lento, não errado).
- `PH2D_PHYSICS_SMOKE_SELECT` escolhe a 1.ª «Collision Shape» pelo nome.
- Sondas do player perguntam com a exceção do 1.º collider sólido do corpo.
- Um include de camada é nulo se TODAS as 32 linhas estiverem vazias.
- Auto Key da exceção não é chaveável (escondida do catálogo).
- Um único passo de undo do espalhar da multisseleção NÃO está provado por teste.
- Ondas do Outline W0/W2/W4–W6 não começaram (Outline = schema 222).
- ADR-0199: renumerar se preciso (§3). Mutações: §9 pendentes. Suíte completa: só a do integrador.
