# HANDOFF de continuação — `line/GameEngineFuncional02`, 10/11, onda 8 itens 2–4 (rampa, canais por tique, motor em graus) + mapa do item 5

Leitor: o próximo agente da linha (ou o integrador). Anterior: [ONDA8_LOOK](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_LOOK.md) (a §7 lista os itens 2–5). Herdados: [OUTLINE_E_F3](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-09_OUTLINE_E_F3.md) §11.

## 1. Identidade

Branch `line/GameEngineFuncional02`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02`, 3 commits sobre `784b00d898`: `ff04dccc4a` (8.2) · `757d483a8b` (8.3) · `3a9bdae8eb` (8.4). **`PROJECT_SCHEMA` inalterado (232).** ADR 0, contratos §6 0. Nada integrado, nenhum push. O smoke do retorno do Look (ONDA8_LOOK §6) **continua por ver pelo dono**.

## 2. Decisões do dono

As de [ONDA8_LOOK §2](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_LOOK.md), sem mudança. Item 4: dono 10/11, «corrija para graus».

## 3. O que mudou (lei e onde)

### 8.2 — penetração na rampa (item 2)

`crates/ph2d-physics/src/world/reshape_push.rs`, chamado de `copia_colisor` (`world/body_build.rs`, ou seja `retune_body` e `retune_part`). Quando a forma de um corpo dinâmico é reafinada **em contacto ativo**, o corpo é transladado pelo quanto a forma NOVA afunda a MAIS que a velha em cada vizinho rígido (fixo/cinemático): `parry query::contact`, Gauss-Seidel sobre os pedidos; duas normais opostas = espremido, fica para o solver; eixos travados respeitados. Vizinho dinâmico empurrado para cima/de lado é levado um nível; vizinho dinâmico ABAIXO segura como chão (quem cresce sobe). **Nenhuma velocidade injetada; a mola (`solver_params.rs`) intocada; custo só nos tiques que reafinam.**

Medido antes → depois (caixote 1→2 m em volta do centro, afundamento máximo): rampa 120 tiques 0,30 → 0,002 cm · 40: 0,83 · 20: 1,54 · 5: 5,0 → 0,002 · canto (chão+parede) 7,4/7,6 cm → 0 · caixa dinâmica em cima afundava 45 cm e saltava 12 cm → 0,004 · forma que cresce sobre caixa dinâmica empurrava-a 2,34 cm no chão (8,39 se empurrada para baixo) → 0,002. Pivô na base (`ShapeLook` pivô `[0,-1]`) já dava 0.

Testes (`crates/ph2d-physics-ecs/tests/it/forma_animada.rs`): `a_forma_que_cresce_no_chao_nao_o_atravessa`, `a_forma_que_cresce_no_canto_sai_do_chao_e_da_parede`, `a_caixa_em_cima_sobe_com_a_forma_que_cresce`, `a_forma_que_cresce_em_cima_nao_mete_a_de_baixo_no_chao`, `a_forma_que_cresce_nao_salta_a_trava_do_eixo`; unitário `a_translacao_satisfaz_cada_pedido_e_recusa_o_espremido`. Mutação 9/9 sangraram.

### 8.3 — Width/Height FISICA por tique (item 3)

Medido (caixote, Height 1→2→1→2 pela Timeline; quadro do app = `apply_from_doc` + canais `aplicar` + `dispatch`): 3 tiques/quadro contra 1 divergia 4,3 cm; o scrub já igualava o Play. Cura em duas metades:

- (a) `TimelineScene::put` (`crates/ph2d-app-physics/src/bake.rs`) também chama `canais_corpo::aplicar_do_tique` (mesmo funil, registo estático próprio dos componentes §11, livro de autoria descartável — o livro do quadro guarda o 1.º valor da corrida por `ConducaoDosCanais::conduziu` `or_insert`) → 0,65 cm.
- (b) `crates/ph2d-physics-ecs/src/bridge/dispatch.rs` pede à cena o 1.º tique devido ANTES de `prepare`/`reconcile` quando `target > last_stepped` (o quadro aplicara a curva no ALVO; o `reconcile` reafinava para o fim do quadro e o laço voltava atrás — ida e volta que o `reshape_push` não desfaz) → 0.

Testes (`crates/ph2d-app-physics/src/canais_por_tique_tests.rs`): `a_curva_da_forma_da_a_mesma_corrida_com_varios_tiques_por_quadro`, `o_scrub_da_curva_da_forma_da_a_pose_do_play`. Mutação 2/2. Bateria nomeada verde (physics-ecs kinematic/rewind/scrub/replay/bake/player/tape/contact…, app-physics bake/scrub/canais…, host-desktop physics/bake/scrub).

**Escopo:** só a família FISICA por tique. Os canais de JUNTA e de MUNDO seguem 1×/quadro (JUNTA precisa do `PreviewDrive`, MUNDO da ponte) — **não medidos** (§7).

### 8.4 — motor da junta em graus (item 4)

`PropKind::JointMotorTarget/Speed` guardam agora o número do CARTÃO (graus e graus/s numa rotação; metros num curso; Custom pelo eixo do motor), no referencial do objeto (o componente já o guarda desde a onda 7). Porta única `PhysicsJoint::motor_no_cartao`/`motor_do_cartao` + `alvo_no_cartao`/`velocidade_no_cartao` (`crates/ph2d-physics-ecs/src/joint/ports.rs`). `ph2d-timeline` `apply_prop` `JointField` converte no get/set; `read_prop_kind` passou a `pub` e reexportada; o shell `timeline_bridge_keys.rs` lê os quatro parâmetros da junta por ela (shell −10 linhas); o cartão `joint.rs` usa a porta (`motor_out`/`motor_in` removidos). Cena de smoke 78: chaves −66°/49° (servo), 630°/s (roda). Teste `crates/ph2d-timeline/tests/it/chave_do_motor.rs` (feature `physics`) `a_chave_do_motor_e_o_numero_do_cartao`; mutação 4/4. O herdado «canais de JUNTA em radianos»: medido, já estava curado; só o comentário de `RODA_MOTOR` foi corrigido.

## 4. O que um merge pode partir

- `TimelineScene::put` agora aplica escritas FISICA; `dispatch` chama `scene.put` antes de `prepare`.
- `read_prop_kind` é `pub`.
- **Valores de chave de `JointMotorTarget/Speed` são graus**: fixture de outra linha com chaves em rad nestas props parte (sem projetos antigos, §0.11).
- Catraca `ph2d-app-physics` à beira (~20 linhas de folga depois da 8.4; o gate conta a pasta INTEIRA, `tests/` incluído).

## 5. Prova (na linha; suíte completa, `nextest-impacted` e clippy NÃO corridos: fecho/integrador)

Só testes por nome, listados na §3, todos nascidos vermelhos, com as mutações indicadas.

## 6. Smoke — NÃO preparado nesta janela

A janela seguinte tem de **montar e fotografar** a cena antes de dar o smoke ao dono (`docs/Components/ferramentas/fotografa_cena.sh`; imagem ao dono = link clicável relativo à raiz do repo principal). Alvos:

- (a) retorno do Look: comando em [ONDA8_LOOK §6](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_LOOK.md);
- (b) caixote que cresce no chão por Height na Timeline, Physics armada: não afunda;
- (c) pista do motor do servo em graus: cena `PH2D_PHYSICS_SMOKE=78` (`crates/ph2d-app-physics/src/physics_smoke_joint_anim.rs`; confirmado o número no cabeçalho do ficheiro). Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_PHYSICS_SMOKE=78 cargo run --profile smoke -p ph2d-host-desktop`.

## 7. ABERTO

**Item 5** (herdado do OUTLINE_E_F3 §11.5; cada um com teste nascido vermelho + mutação; mapa do código por levantamento):

- 5.1 **Censo de sinais:** botões de painel autorados (`Signal::from_control`, `shells/desktop/src/render_loop/authored_intents.rs:45-108`, `ui_preview_gesture.rs:110`) faltam em `crates/ph2d-signal-census/src/lib.rs` `CAMPOS` (~:60; só `UiButton` em :138) → uma Logic a ouvir a chave de um botão mostra «ninguém envia» (`panel.inspector.signal.nobody_sends`, `crates/ph2d-panel-inspector/src/signal_hint.rs:99-101`).
- 5.2 **Feito** (JUNTA em radianos: já curado, ver 8.4).
- 5.3 `crates/ph2d-outline-trace/src/lib.rs`: `fits()` (:148-153) chama `trace()` antes de checar o teto `MAX_POINTS` (:131, :153); `trace_fitted` (:195) bisseciona chamando `fits` repetidas vezes; a busca de orelha é quadrática (README :139).
- 5.4 `crates/ph2d-physics-ecs/src/bridge/nav_custo.rs:45-80` itera só `self.bodies` (sem partes, ao contrário de `nav_malha.rs:118-145`); `crates/ph2d-app-macros/src/mundo/palco_fisica.rs:211` tira a impressão só de `main_shape`.
- 5.5 Macro que muda o nº de pontos do contorno ignora o modal da regra B: `canais_corpo.rs` `aplicar_pontos` → `op::aplicar` (`crates/ph2d-physics-outline/src/canais_pontos.rs:190-197`, `regra_b`); a guarda `travados::numero_travado` só está em `physics-outline/src/lib.rs:435` e `merge.rs:27` — **confirmar se `op::aplicar` checa travados** antes de escrever o teste.
- 5.6 Painel clássico (`PH2D_CARDS=0`) não acinzenta os campos travados do Outline (Detail, Alpha cutoff, Ignore specks under, com custom + shape edit desligado): caminho dos cartões `physics_cartao_contorno.rs:27/:74`; o clássico `populate_physics.rs:258-283` só regista a dica.

**Também aberto:** canais de JUNTA e de MUNDO por tique (medir primeiro, como na 8.3); restante do OUTLINE_E_F3 §11.1–11.4 sem mudança.

O fecho da linha (clippy, handoff de integração) e a integração continuam só por ordem do Enio.
