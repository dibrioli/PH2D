# HANDOFF de continuação — `line/GameEngineFuncional02`, 10/11, onda 6 (4 suspeitas medidas · poda do Outline · testes fora da shell · junta em graus)

Leitor: o próximo agente da linha (ou o integrador). Anterior: [ONDA5](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA5.md). Último handoff de integração (já no `main`): [OUTLINE_E_F3](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-09_OUTLINE_E_F3.md). Memória nova: [física fluida sob animação](../../../project-memory/project_physics_fluid_under_animation_2026_11_10.md).

## 1. Identidade

Branch `line/GameEngineFuncional02`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02`. Onda 6 = 6 commits sobre `6721e6cc9b` (handoff da onda 5). `PROJECT_SCHEMA` = 231, **inalterado**; registos, ADR, contratos congelados: 0. Nada integrado, nenhum push.

| SHA | O quê |
|---|---|
| `43593d8140` | Item A: as 4 suspeitas da §7 anterior medidas; 1 real, curada |
| `f486a4e8e2` | Item B: poda das tabelas do Outline |
| `5171e5c0a6` | foto `onda6/01_rock_outline.png` |
| `279d4900d9` | Item C: teste sai da shell (folga da catraca) |
| `ee202ff305` | Item D: canais da JUNTA em graus |
| `bd9fafbc20` | foto `onda6/02_portoes.png` |

## 2. Decisões do dono (10/11)

Smoke dos itens A, B, D = OK. No smoke de D (cena `PH2D_PHYSICS_SMOKE=66`, Pin A, Auto Key no Limit min −30→−60) o dono reportou: «a simulação não obedece à animação dos limites; parece que entra em repouso quando toca no limite», e «animar os tamanhos dos collision shapes também dá resultados ruins» (com UMA forma só). **Quer a física FLUIDA** com collision shapes e joint limits animados → é a onda 7 (§7).

## 3. O que mudou

**A — `43593d8140` (7 ficheiros, +104 −16).** Medido:
- (a) cápsula raio+altura no mesmo quadro: **NÃO real** (aplicar ordena por `FieldKey`, RAIO 21 < CAPSULA 24; escrever refaz o retrato). Teste que pina: `raio_e_altura_da_capsula_no_mesmo_quadro_dao_a_altura_escrita` (`ph2d-app-physics/src/canais_corpo_tests.rs`).
- (b) «Delete animation» 2× Ctrl+Z: **NÃO real por leitura** (`apagar_num_passo` + `empurra` ligam por `liga_gesto_da_cena`); não corrido ponta-a-ponta.
- (c) **REAL, curado:** cartão do Trigger à escuta de X + «+»/Bind da janela Input Map em X → o Down desarmava o cartão e o Click re-armava como «acrescentar». `input_map_listen_cancelled_by_down` passou de `bool` a `Option<ActionId>`; `take_listen_cancelled_by_down()` devolve `Option<ActionId>`; `screens/hero/chrome/input_map/apply.rs` trata `== Some(aid)` como cancelar; `event_action_trigger.rs` usa `.is_some()` (também `input_map_ops.rs`, `store_core.rs`, `store_fields.rs`). Testes `o_bind_da_janela_sobre_a_escuta_do_cartao_cancela_e_nao_rearma` (nasceu vermelho) e controlo `o_bind_de_outra_accao_arma_a_janela_depois_de_desarmar_o_cartao` (`ph2d-editor-core/tests/it/o_cartao_do_trigger_escolhe_a_tecla.rs`).
- (d) «N listen» contar a própria caixa: **NÃO real** (Trigger é Emit; o censo não o lê).

**B — `f486a4e8e2` (7 ficheiros, +169 −8).** Medido com 40 Detail + 40 escalas: tabelas [peças Q4, conversões, mundo] = [40, 79, 41] (vivo [1,1,1]); peça arrastada [41,41,3] e, podando só por chave, [13,13,2]. Cura: `crates/ph2d-physics-ecs/src/bridge/outlines.rs` `sweep_outlines`, chamado no `prepare` após `reconcile_parts`; chaves vivas = `rest.shape` de corpos e peças; `OutlineCache::needs_sweep` sem alocar; `sweep` poda mundo (`PhysicsWorld::retain_outlines`, novo `outline_count`), conversões por geração marcada no resolve, peças Q4. Acessor `PhysicsBridge::outline_tables()` (`#[doc(hidden)]`). Testes `arrastar_detail_e_escala_nao_acumula_pecas`, `a_peca_arrastada_nao_acumula_e_a_forma_principal_fica` (`ph2d-physics-ecs/tests/it/outline_bridge.rs`). Foto: [`onda6/01_rock_outline.png`](fotos_fisica/onda6/01_rock_outline.png).

**C — `279d4900d9` (5 ficheiros).** `every_physics_component_is_authorable` movido da shell para `crates/ph2d-panel-inspector/tests/it/` (caminhos `src/render_loop` → `../../shells/desktop/src/render_loop`). Folga de `the_shell_only_shrinks` **+25 → +181**. Docs `Components/20_plano_ray_sensor.md` e `Physics/00_plano_waves.md` com o caminho novo. **NÃO movidos, e porquê:** `physics_gesture_tests`/`_zone`/`inspector_player_tests*` atravessam de propósito a porta de produção da shell (`init::build_component_registry` + `component_attach`); `render_loop/*_tests` são filhos por `super::*`; `a_tecla_do_gatilho_nao_e_reclamada` é gate SOBRE o teclado da shell e `ph2d-app-components` não tem folga (81 007 vs tecto 80 869 com ele).

**D — `ee202ff305` (4 ficheiros, +55 −31).** Canais da JUNTA em GRAUS como o cartão: ler converte (`limit_out` pelo tipo; `to_degrees` no eixo que não é em metros; `to_degrees` no `RODA_MOTOR`); `edicao`/`edicao_roda` passam o valor cru. Doc de `ph2d-ecs/src/canal_junta.rs` corrigido. Teste `o_canal_e_a_edicao_do_painel_dao_a_mesma_junta` reescrito em graus (nasceu vermelho); `canais_donos_tests.rs::escrever_pela_junta_muda_o_que_o_catalogo_le` converte o catálogo (radianos) nos campos angulares. Foto: [`onda6/02_portoes.png`](fotos_fisica/onda6/02_portoes.png). **FORA de propósito:** `PropKind::JointMotorTarget/Speed` seguem no valor do componente (`ph2d-timeline` `apply_prop` `JointField` + `shells/desktop/src/timeline_bridge_keys.rs:105`; linha Timeline viva; a unidade depende do tipo) → §7.2.

## 4. O que um merge pode partir

- Quem lê `take_listen_cancelled_by_down()` como `bool` (agora `Option<ActionId>`) ou o campo `input_map_listen_cancelled_by_down`.
- Quem lê/escreve canais da junta em **radianos** pelo catálogo de canais (`canais_junta`): agora graus (limites angulares, eixo Rotation do Custom, motor da roldana). Linha Timeline: `JointMotorTarget/Speed` NÃO mudaram.
- `OutlineCache`/`PhysicsWorld` ganham API (`needs_sweep`, `sweep`, `retain_outlines`, `outline_count`); o `prepare` da ponte chama `sweep_outlines` — qualquer linha que reordene o `prepare` ou crie outra tabela de outline fora do cache fica sem poda.
- `shells/desktop/tests/it/main.rs` perdeu um `mod` (o teste movido); quem o editar/rebasear conflita nessa linha. Quem citar o caminho antigo nos docs de plano: atualizado.
- **Catraca:** `physics_apply.rs` continua em **599/600** — não precisou crescer; qualquer linha nova reprova, cura = MOVER. `the_shell_only_shrinks` tem agora folga +181.
- Contadores como DELTA: schema 0 · registos 0 · cenas de smoke 0 · ADR 0 · contratos §6 0.

## 5. Prova de fecho (na linha; suíte completa e `nextest-impacted` NÃO corridos: do integrador)

- Testes por nome, verdes: os citados em §3. Mutação: **A 4/4 · B 5/5 · C 1/1 · D 6/6**, todas vermelhas.
- `cargo fmt --all` limpo. `clippy -D warnings` limpo em `ph2d-editor-core`, `ph2d-app-physics`, `ph2d-panel-inspector`, `ph2d-physics-ecs`, `ph2d-physics`, `ph2d-ecs`.

## 6. Smokes (dono: A, B, D OK em 10/11)

Prefixo: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && … cargo run --profile smoke -p ph2d-host-desktop`.

1. **Poda do Outline (B).** `PH2D_PHYSICS_SMOKE=120 PH2D_PHYSICS_SMOKE_OUTLINE=Rock`. Arrastar Detail e a escala da Rock muitas vezes; o jogo não engasga nem a forma desfaz. Errado: forma da Rock some após arrastar. Foto: [`onda6/01_rock_outline.png`](fotos_fisica/onda6/01_rock_outline.png).
2. **Junta em graus (D).** `PH2D_PHYSICS_SMOKE=66`. Limites e motor da junta mostram os mesmos graus no cartão e no canal. Foto: [`onda6/02_portoes.png`](fotos_fisica/onda6/02_portoes.png). **Este smoke é onde o dono achou o defeito da §7.1** (limite mínimo animado não obedece).
3. **Bind sobre a escuta (A).** Cartão do Trigger à escuta de X; na janela Input Map «+»/Bind em X: cancela e não arma «acrescentar».

Armadilha das fotos: `docs/Components/ferramentas/fotografa_cena.sh` — env vars TODAS no 1.º argumento; casos de física com prefixo `fisica_`.

## 7. ABERTO (por ordem)

**1. Onda 7 — física fluida sob animação** (pedido do dono, §2). Investigação feita na ponte com sondas já apagadas:

- *Junta.* A curva chega ao componente e ao rapier por tique (`reconcile_joints` `bridge/joints.rs:509-545`; `drive_joint_params` `bridge/joint_drive.rs:49-78` via `dispatch.rs:198`; `retune_joint` `world/joints.rs:239-247`, `get_mut(handle,true)` acorda). **CAUSA:** referencial. No pino de mundo `body_a` = portão, `body_b` = mundo, e o rapier mede B relativo a A — o portão a −30° encosta no limite MÁXIMO (+30); `joint_desc.rs:60` passa `[limit_min, limit_max]` sem trocar o sinal. Animar o mínimo mexe no batente onde nada encosta (medido: mínimo −30→−60 fica em −30,00°; máximo +30→+60 desce suave até −60,00° em t370). O arco do overlay (`overlay/joint_glyphs.rs:476-503`) está do lado errado no pino de mundo (não visto na tela).
- *Formas.* Forma principal cresce sem salto (`retune_body` `world/body_build.rs:169-310` → `set_shape`; ring não limpo, 0 recriações); altura que cresce penetra 0,6–1,0 cm durante a rampa (cresce em volta do centro); o corpo acordou mas sem razão identificada (`set_shape` do rapier não acorda). PEÇA (2.ª forma filha): `bridge/parts.rs:226-245` faz `ring.clear` + detach + attach a CADA tique (60 recriações). Voltar a 0 depois da curva recria o corpo uma vez (retune sobrescreve `b.rest` em `bodies.rs:185-193`; idem `j.rest` `joints.rs:542`). Formas só relidas 1×/quadro, sem drive por tique como o das juntas. A curva passa por `shells/desktop/src/render_loop/fase_timeline_drain.rs:12-41` (≤ 1 quadro atrasada; lido, não medido no app).
- **O caso do dono (UMA forma) NÃO foi reproduzido com defeito grande na ponte.** 1.º passo = reproduzir no APP (Width/Height pelo cartão com Auto Key? Scale? Play com sleep ligado: Speed 0,05, Delay 2 s) e MEDIR antes de curar.
- *Cinco curas num bloco (§0.10):*
  1. Limite no referencial do objeto quando o lado B é o mundo: passar `[−max, −min]` em `joint_desc.rs:60`/`reconcile_joints` e no arco do overlay — teste `o_limite_minimo_do_pino_de_mundo_e_o_do_objeto` (+ controlo do máximo). Reconferir a cena 66.
  2. Reafinar peças no lugar em `parts.rs` (função gémea de `retune_body` para `ColliderHandle`, acorda o pai; ring sobrevive; provar contra scrub) — `a_peca_animada_nao_limpa_o_ring`.
  3. `wake_up(true)` explícito no `retune_body` quando a forma muda (`body_build.rs:286`) — `a_forma_que_cresce_acorda_o_corpo_que_dorme`.
  4. Drive de formas por tique ao lado do `drive_joint_params` (`dispatch.rs:198` e laço de replay) — `o_scrub_fora_do_passo_do_ring_da_a_forma_do_play`.
  5. Não sobrescrever o `rest` no retune (estado vivo à parte) — `voltar_a_zero_depois_da_curva_nao_recria_o_corpo`.
  - Opcional: a penetração de ~1 cm (Box2D/Godot fazem igual). Estruturais que continuam a recriar: trocar tipo de forma/junta, dono, sensor, Dynamic↔Static. `physics_apply.rs` não precisa crescer. Estado da arte: rapier `get_mut(…, wake_up)` acorda, `set_shape` não; Box2D v3 `SetLimits` acorda e `SetPolygon` refaz contactos; Godot PinJoint2D limites relativos ao `node_a`.

**2.** `PropKind::JointMotorTarget/Speed` em graus (toca `ph2d-timeline` e shell; coordenar com a linha Timeline; a unidade depende do tipo de junta).

**3.** Herdados (§11.5 do handoff OUTLINE_E_F3): censo de sinais, O(pontos²) do traço, ponte de navegação, macro que muda o nº de pontos sem modal, painel `PH2D_CARDS=0`.

**Riscos:** `physics_apply.rs` 599/600. Ficheiros `.log` de fotos por rastrear em `docs/GameEngineFuncional/handoffs/fotos_fisica/` (lixo do `fotografa_cena.sh`): NUNCA `-A`; comite por path. Suíte completa + `nextest-impacted`: do integrador. Integração só por ordem do Enio; smoke do dono não é aprovação.

## 8. Prompt de continuação (colar numa janela nova)

```
/pd-linha-assumir line/GameEngineFuncional02

Worktree: /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02
Antes de ler: cd lá, `pwd`, `git branch --show-current` (tem de dar line/GameEngineFuncional02).
HEAD = o commit do handoff docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA6.md
(confira com `git log --oneline -3`). Leia esse handoff (§7 ABERTO) e CLAUDE.md.

Estado: onda 6 feita; smoke do dono OK em 10/11 nos itens A, B, D (suspeitas medidas, poda do
Outline, junta em graus), mas o dono achou que a física NÃO fica fluida com limites de junta e
tamanhos de forma animados. Nada integrado, nada com push. PROJECT_SCHEMA = 231.

Missão (onda 7): física fluida sob animação. COMECE por reproduzir no APP o caso de UMA forma do
dono (tamanho animado: Width/Height pelo cartão com Auto Key, Scale, Play com sleep ligado) e MEDIR
antes de curar. Depois as 5 curas da §7.1 num bloco: (1) limite no referencial do objeto no pino de
mundo (o «mínimo» passa a ser o do objeto; reconferir a cena PH2D_PHYSICS_SMOKE=66), (2) reafinar
peças no lugar sem limpar o ring, (3) acordar o corpo quando a forma muda, (4) drive de formas por
tique, (5) não sobrescrever o rest no retune. Cada uma com teste por NOME nascido vermelho e mutação.

Regras de sempre: UI só em inglês, por tokens/i18n (zero hex, zero string solta); teste por NOME,
nascido vermelho, com prova de mutação; inner loop = cargo-check-narrow; comando pesado por
scripts/ph2d-run.sh; commit só por path (`git commit --no-verify -m … -- <paths>`), nunca -A,
não comite os .log de fotos; physics_apply.rs está em 599/600 — cure por MOVER; suíte completa é do
integrador; SEM push, integração ou fecho sem ordem do Enio; smoke ao Enio em passos numerados com
foto tirada antes via docs/Components/ferramentas/fotografa_cena.sh (env vars todas no 1º argumento,
casos de física com prefixo fisica_) e imagem como link markdown relativo à raiz do repo principal
(Worktrees/line-GameEngineFuncional02/docs/…png). Respostas ao Enio: curtas, sem jargão.
```
