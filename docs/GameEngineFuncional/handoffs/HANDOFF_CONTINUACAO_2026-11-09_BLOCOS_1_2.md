# HANDOFF DE CONTINUAÇÃO — line/GameEngineFuncional02, 09/11 — blocos 1 e 2 feitos, bloco 3 (Outline) por começar

> NÃO é handoff de integração: o dono proibiu fechar a linha. Leitor: a próxima LLM desta linha.
> Anterior (integrado): [OPCAO_B](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-08_OPCAO_B.md) (ADR-0199).

## 1. Estado

Linha reaberta de `main f254e42bf1` (09/11), depois da integração da opção B. Commits da linha:

| hash | o quê |
|---|---|
| `6a3c6e5c9a` | nomes únicos das formas + exceção «Collides with» DENUNCIADA (dono 09/11) |
| `0ad295bd30` | BLOCO 1: corpo de formas todas desligadas FICA no solver (smoke 120 do dono reprovou) |
| `af0cbabde4` | mesma família, num bloco (§0.10): leitores que EDITAM usam `forma_para_editar` |
| `9bf303f451` | BLOCO 2: «Collides with» com 2 formas = UM passo; Ctrl+Z devolve as duas |
| `9c0eca9c81` | env `PH2D_PHYSICS_SMOKE_OLHO=<nome exato>` + fotos do olho |

Suíte completa: só na integração. Nada empurrado, nada integrado.

## 2. Bloco 1 — o olho de uma forma tirava o corpo inteiro

**Mecanismo — UMA raiz, dois consumidores.** `main_shape` (primeira forma LIGADA) devolvia `None` quando
todas as formas do corpo estavam desligadas. (1) `bridge/bodies.rs` fazia `continue` → corpo removido do
rapier → não caía. (2) `inspector/body.rs` lia `main_shape` → `has_body` falso → o cartão Physics Body
era substituído. Hipótese (a) «o olho escreve no corpo» REFUTADA (o gate afirma que `Visibility` cai na
forma). (b) massa zero: no rapier 0.36.1 a força da gravidade é `gravity*mass`
(`rigid_body_components.rs:1055`) e collider desligado não conta massa (`:457`) — relevante para a CURA.
(c) = consumidor 2.

No comando do dono, `PH2D_PHYSICS_SMOKE_SELECT="Collision Shape"` no main apanhava a forma do PLAYER
(Dynamic, uma forma) por ordem de consulta — medido por sonda temporária. Hoje, com nomes únicos,
esse nome casa a forma do CHÃO (nome exato), não a do Player.

**Lei (decidida):** corpo com TODAS as formas desligadas fica no solver, não toca em nada, o Dynamic cai.
Massa `MASS_WITHOUT_SHAPE_KG = 1.0` (Box2D ResetMassData / Godot / Unity) salvo `MassOverride`.
`BodyDesc.shape_off` → collider `.enabled(false)`. Lei do PlatformPlayer saltada para corpo `shape_off`
(o raio da perna o prendia ao chão). `zone_effect` recusa zona desligada. O Inspector lê `forma_para_editar`.

**Gate RED-first:** `shells/desktop/src/render_loop/fase_bus_hierarchy_tests.rs ::
o_olho_da_unica_forma_do_player_nao_o_tira_do_solver_nem_do_inspector` (cena 120; olho via
`EditorAction` → `fase_bus_hierarchy_tree` → `row_flag::aplica`; 90 ticks).
RED: y 0,450→0,450 e `has_body` falso. GREEN: y 0,450→−10,350 e `has_body` verdadeiro.

**Mutações (todas sangraram):** M1 `continue` na ponte · M2 `main_shape` no Inspector · M3 sem 1 kg ·
M4 collider ligado · M5 sem a exceção do player.

**Porque o portão antigo `olho_da_forma` estava verde:** `desligar_uma_forma_nao_desliga_a_outra` tinha
SEMPRE duas formas (`main_shape` nunca `None`) e um `||` fraco; `o_olho_da_forma_desliga_e_liga_o_collider`
AFIRMAVA o defeito (`body_pose(duo).is_none()`, um tick, queda não medida); nenhum passava pelo caminho
da Hierarquia, pelo construtor do Inspector nem por um PlatformPlayer. Reescritos para a lei nova, mais
`as_formas_todas_desligadas_atravessam_o_chao` e `a_unica_forma_desligada_cai_com_a_massa_de_um_corpo_sem_forma`
(1 kg / 5 kg com MassOverride).

**`af0cbabde4`** — leitores que EDITAM migrados para `forma_para_editar`/`tem_forma`: `excecao.rs`
`forma_da_excecao`; `physics_apply` Mass→Manual (semeia MassOverride ou 1 kg); `inspector/player.rs`
(semente); `joint_create` (pivô); `nav_inspector` (`has_shape`); sprite-anim `tiled.rs`; app-game
`smoke.rs`/`smoke_moedas.rs`. Mantidos (simulação): `bridge/bodies.rs`, impressão digital do `palco_fisica`
nas macros. Testes novos: `a_excecao_de_um_corpo_de_formas_desligadas_mora_na_forma`,
`manual_num_corpo_de_formas_desligadas_semeia_1_kg` (ambos vistos RED por mutação).

## 3. Bloco 2 — «Collides with» com duas formas

`colide_com_duas_formas_e_um_passo_e_ctrl_z_devolve_as_duas` em
`shells/desktop/src/render_loop/fase_bus_inspector_preso_tests.rs`: dreno real `fase_bus_physics_sections`,
seleção [a,b], aplica `pd.physics_edits` com `apply_physics_edit` (`inspector_commits.rs:418`), um passo de
`ProjectUndo` restaura. **VERDE DE NASCENÇA** (a shell já espalha num dreno; o undo é o diff por quadro,
`post_frame_undo_steps`). Provado por mutação: empurrar só o primário · `take(1)` · `skip(1)` → todos RED.

## 4. Nomes únicos e exceção denunciada (`6a3c6e5c9a`)

- Nomes via `ph2d-unique-name`: «Collision Shape», «Collision Shape (1)»… (`unique_names_in` novo no `World`
  cru). Duplicar renomeia formas copiadas que não são peça de receita. O dono pediu sufixo «002»; a lei
  única do app « (n)» ficou e o dono foi avisado.
- Exceção DENUNCIADA: chips fixados + «Differs from layer» em Warn/WarnSoft (`EstadoDaSelecao.fixos` →
  `InspectorPhysicsInfo.collides_pinned`).
- Aba Collision Layers: objetos «Crate › Collision Shape» (`ObjetoDaCamada.dono`); secção «Differ from this
  layer» com um botão por forma (`PHYSICS_LAYER_EXCEPTION`, seleciona) e o que difere («Ground off · Props
  off»); ponto Warn na célula de par fixado (`pinned_rows`); contagem no rodapé (`exceptions_on_pair`).
- Fotos: `PH2D_PHYSICS_SMOKE_EXCECAO=1` e `PH2D_PHYSICS_SMOKE_ABA=excecao`.

## 5. Smokes PENDENTES do dono (blocos 1–2, nomes únicos, exceção)

Reconstruir ANTES, na ponta: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`

- Olho: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_PHYSICS_SMOKE=120 PH2D_PHYSICS_SMOKE_SELECT=Player ./target/smoke/ph2d-host-desktop`
  Fechar o olho da forma do Player e dar Play: ele cai pelo chão; o cartão Physics Body continua.
- Exceção: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_PHYSICS_SMOKE=120 PH2D_PHYSICS_SMOKE_EXCECAO=1 ./target/smoke/ph2d-host-desktop`
  (`+ PH2D_PHYSICS_SMOKE_ABA=excecao` abre a aba em Props).
- Fotos do olho (`PH2D_PHYSICS_SMOKE_OLHO=<nome exato>`, `ficha.rs olho_para_as_fotos`): a forma do Player é
  «Collision Shape (7)» (aspas internas ao passar ao `fotografa_cena.sh`). Em
  `docs/GameEngineFuncional/handoffs/fotos_fisica/olho_09_11/`: 01 pausado olho fechado · 01b cartão Physics
  Body inteiro (1930×2772) · 02 a tocar, T 4,47 s Player fora por baixo · 03 T 0,75 s Player abaixo do Floor.

## 6. ABERTO

- Ponte de navegação (`nav_custo.rs`, obstáculos em `nav_malha.rs`) ignora `shape_off`: corpo com todas as
  formas desligadas ainda corta áreas de nav — decisão de PRODUTO do dono.
- Impressão digital do `palco_fisica` só vê a forma principal (anterior a esta linha).
- SEM mutação: o ramo `was_off` de `retune_body` (tira o 1 kg quando o olho reabre) e a recusa `shape_off`
  de `zone_effect`.
- Corpo sem NENHUM filho forma ainda sai do solver — fora de escopo, pergunta ao dono.
- Warn na pele padrão é creme pálido (#D4C79E): o dono pode querer realce mais forte.
- Smoke do dono dos blocos 1–2 e dos nomes únicos/exceção: PENDENTE. Suíte completa só na integração.

## 7. Bloco 3 — Outline: o que falta e a ordem

**Não começado.** Feito só a W1 (`b83680070d`, crate `ph2d-outline-trace`, oráculo Godot em
`docs/GameEngineFuncional/oraculo_contorno/`, `tests/oraculo.rs`); ZERO consumidores no app.
Plano: [f2_contorno_e_formas_plano.md](../componentes/f2_contorno_e_formas_plano.md) (decisões do dono 08/11
no topo: um contorno = união dos quadros; reentrância sólida; buraco fechado fica buraco; Detail, Alpha
cutoff, Ignore specks under; pontos editáveis; Follows the picture × Custom size + Fit to picture; Auto Key
grava Detail e Alpha cutoff).

Sob a opção B: `ColliderOutline` mora na forma FILHA; `source` = o ancestral mais próximo com imagem, até ao corpo.

Ordem:
1. **W0** — MEDIR o custo das peças (`measure_outline_pieces`, `--profile smoke`; ainda NÃO existe, escrever).
   Escrever o teto com a tabela da medição ao lado (§0.0; acima dele o Detail sobe, nunca corta a arte).
2. **W2** — motor + formato: compound de `ConvexPolygon` no rapier, rewind, espelho (escala com sinal e
   inverter o enrolamento quando det<0 — `scale.rs` usa `abs()`). Os números de schema do plano («220/222»)
   estão VELHOS: o main está em 225; o degrau CONTA-SE com `python3 scripts/schema-recount.py`, nunca se
   escolhe. Sem compatibilidade com projeto antigo (§0.11).
3. **W4** — cartão + Auto Key.
4. **W5** — canvas (pontos do Outline; alças só com a forma selecionada — resposta 3 do dono à B).
5. **W6** — cena 120 (contorno, anel com buraco, corpo com 3 formas) + fotos com
   `docs/Components/ferramentas/fotografa_cena.sh` ANTES do smoke + smoke do dono em passos numerados.

Restrições: código novo fora de `ph2d-app-*` sempre que der (catraca de LOC da família); UI do app só em
inglês; sem push/integração/fechar a linha nem handoff de integração sem ordem do dono.

## 8. Prompt de continuação

```
/pd-linha-assumir GameEngineFuncional02

Worktree: /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 (branch line/GameEngineFuncional02).
Faça `cd` para lá e confira `pwd` + `git branch --show-current` ANTES de ler.

Leia, nesta ordem: docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_2026-11-09_BLOCOS_1_2.md e
docs/GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md.

NÃO refaça: a opção B / ADR-0199, as decisões do dono sobre o contorno (topo do plano), nem os blocos 1 e 2
(olho da forma, «Collides with» com duas formas) — estão feitos, com portão e mutação.

Tarefa: BLOCO 3 (Outline), nesta ordem W0 → W2 → W4 → W5 → W6.
- W0: MEÇA primeiro o custo das peças (`measure_outline_pieces`, --profile smoke, ainda por escrever) e
  escreva o teto com a tabela da medição ao lado (acima dele o Detail sobe, nunca corta a arte).
- W2: motor + formato (compound de ConvexPolygon no rapier, rewind, espelho: escala com sinal e inverter o
  enrolamento quando det<0). O degrau do schema CONTA-SE com `python3 scripts/schema-recount.py` (os 220/222
  do plano estão velhos: o main está em 225); sem compatibilidade com projeto antigo.
- `ColliderOutline` mora na forma FILHA; `source` = o ancestral mais próximo com imagem, até ao corpo.
- W4: cartão + Auto Key (Detail e Alpha cutoff). W5: canvas (pontos; alças só com a forma selecionada).
- W6: cena 120 (contorno, anel com buraco, corpo com 3 formas); FOTOGRAFE com fotografa_cena.sh antes de
  mandar o smoke; smoke ao dono em passos numerados (comando com `cd` · onde clicar · o que acontece · como
  saber que errou), UM por onda.
- Todo portão nasce VERMELHO primeiro, e cada um leva prova de mutação.
- Código novo fora de ph2d-app-* sempre que der (catraca de LOC). UI do app SÓ em inglês.
- Inner loop = cargo-check-narrow; teste por NOME; suíte completa só na integração.
- NÃO faça push, NÃO integre, NÃO feche a linha nem escreva handoff de integração sem ordem EXPLÍCITA do Enio.
```
