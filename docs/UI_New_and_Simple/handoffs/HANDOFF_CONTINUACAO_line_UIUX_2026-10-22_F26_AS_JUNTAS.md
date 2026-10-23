# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 22/10 — F26: as juntas (todo campo da junta e da roldana grava no Auto Key)

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `100d34c8e0` (não andou). Anterior:
  [F25](HANDOFF_CONTINUACAO_line_UIUX_2026-10-21_F25_O_BLOCO_2.md) (§7: smoke de 22/10 do dono, *«algumas
  propriedades das juntas ainda não gravam keys»*; §6 item 10 = esta onda).
- Plano: [`35_plano_F24_o_auto_key_grava_tudo.md`](../vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md).

## 1 — Commits desta onda

| commit | o quê |
|---|---|
| `4b359d7f10` | F26: 42 ficheiros (21 de código/teste + fotos/logs `F26_cena_autokey_{16..26}.{png,log}` em `docs/UI_New_and_Simple/vetor_nova_ui/fotos/`), +1503/−92 |
| (este) | este handoff |

## 2 — Como foi construído

Inventário: cada campo pintado na §12 (junta, 9 tipos) e na §13 (roldana) do Inspector, contra o que grava. Três mecanismos de falha e dois achados, curados no mesmo bloco.

### 2.1 (A) Quatro canais fora do `AUTOKEYED`

`JointMotorTarget`, `JointMotorSpeed`, `JointRestLength`, `JointMaxLength` (`PropKind` 9–12) não estavam em `PropKind::AUTOKEYED` (nota antiga «a porta é o K»): a edição do painel voltava à curva no quadro seguinte. Cura: `AUTOKEYED` 11→15 (`crates/ph2d-timeline/src/prop.rs`); `PoseSample` cresce (tipo derivado do `len`, nenhum literal).

Achado na cena: o `max_length` de uma POLIA é re-derivado pela ponte quando a rota muda (`reseat_wheel_geometry` → `anchored=false` → L0) e passou a ser gravado como mão (trilha de 1 chave na Pulley). Cura em `shells/desktop/src/render_loop/autokey_amostra.rs`: salta `JointMaxLength` quando `ph2d_app_physics::canais_junta::e_polia` (o K manual continua).

### 2.2 (B) Campos sem vocabulário

Tipo, Active, Collide Connected, Anchor B (Object/World) e os 8 campos da roldana não tinham canal nem losango; `joint.rs`, `joint_pair_rows.rs` e `wheel.rs` estavam fora do ESCOPO do gate. Vocabulário (só APPEND, `crates/ph2d-ecs/src/canal_junta.rs`, gate `o_vocabulario_da_junta_nao_renumera`):

- `TIPO` 50 · `ATIVA` 51 · `COLIDE_PAR` 52 · `PONTA_NO_MUNDO` 53 (marcador `JointWorldAnchor`).
- Roldana: `RODA_RAIO` 60 · `RODA_RAIO_SAIDA` 61 · `RODA_WESTON` 62 (marcador `WestonAxle`) · `RODA_ORDEM` 63 (0-based) · `RODA_LADO` 64 · `RODA_MOTOR` 65 (rad/s) · `RODA_QUEBRA_LIGADA` 66 · `RODA_QUEBRA_FORCA` 67.
- Rótulos em `ph2d-i18n/src/canais.rs`.

`crates/ph2d-app-physics/src/canais_junta.rs` reescrito: publica junta + marcador e roldana; aplica pelo funil do painel (`joint_with_edit`, `wheel_with_edit`), TIPO primeiro na ordem, marcadores pelas portas `ph2d_app_physics::joint::anchor_to_world` (a shell `fase_physics_edits.rs` passou a chamá-la: −10 linhas) e `joint_wheel::weston_axle`; rota mudou ⇒ `reseat_wheel_geometry` e a corda declara-se. `ph2d-preview-drive`: `Driver`/`Driven::PontaNoMundo(bool)` e `Roldana{roda, weston}`. `marcas_inspector.rs` (`ph2d-app-components`): 28 ids novos (`INSP_JOINT_KIND[0..9]`, `ACTIVE`, `COLLIDE`, `ANCHOR_B`, `INSP_WHEEL_*`).

### 2.3 (C) Tipo igual re-semeava as âncoras

`JointFieldEdit::Kind` com o MESMO tipo re-semeava as âncoras (`anchored=false`): a curva do tipo declarava a junta conduzida em TODO quadro, e a população do autokey salta o conduzido (`preview.drives`) — a volta ao Pin no q24 da cena `=26` não gravava (medido com `PH2D_AUTOKEY_LOG=1`: a entidade sumia da população). Cura em `joint.rs`: `Kind(mesmo tipo)` = no-op (o clique no chip já escolhido também deixou de largar as âncoras). Teste `escrever_o_mesmo_tipo_nao_mexe_na_junta` (`canais_junta_tests.rs`).

### 2.4 (D) Números da roldana só seguiam na troca de seleção

`sync_wheel_fields` só corria na troca de seleção (o mesmo defeito que F25 2.4(c) curou nas juntas): agora a cada quadro, com o guarda de foco/arrasto (`sync_physics.rs`, `sync.rs`). Teste `seam_wheel::the_wheel_numbers_follow_the_document_while_the_wheel_stays_selected`.

### 2.5 (E) O gate

`architecture_every_editable_field_is_a_channel`: ESCOPO + `joint.rs`, `joint_pair_rows.rs`, `wheel.rs`; censo novo `nenhum_campo_de_junta_fica_fora_do_escopo` (todo `sections/*.rs` que nomeia `ids::INSP_JOINT_`/`INSP_WHEEL_` está no ESCOPO; controlo ≥ 6 ficheiros, hoje 7). O leitor `SEM_COLUNA` confundia `IconId::Eyedropper` com `IconId::Eye` (fn casa): os conta-gotas da §12/§13 liam-se como campos sem coluna.

### 2.6 Classificação

Nenhum campo da §12/§13 ficou `NaoAnimavel`. Botões (Swap, Copy/Paste, Delete, conta-gotas, Add Wheel, Unmount) não são campos.

### 2.7 Cenas `PH2D_AUTOKEY_SMOKE=16..26` (`crates/ph2d-app-vec/src/autokey_smoke_tipos.rs`; despachante `1..26`)

10 bancadas (Pin, Spring, Rope, Weld, Slider, Rod, Wheel, Pulley, Custom, Type Change) + Pulley Wheel; tudo selecionado; muda no q12, volta no q24, pára no q18. `=16..23` Pin..Pulley · `=24` a roldana · `=25` Custom · `=26` Type Change. Todas as trilhas 1×2 chaves (log q24/q18). Valor no q18:

| bancada | muda |
|---|---|
| Pin | modo Position, Target 0,6 rad (34,377°), Collide On |
| Spring | Rest Length 0,9, Active Off |
| Rope | Max Length 1,5, Anchor B World |
| Weld | Soft, Stiffness 110 |
| Slider | limites ±1,0 m, motor 1,25 |
| Rod | Max Length 1,15, Breakable On 1250 |
| Wheel | Stiffness 250, motor 1,5 |
| Pulley | Breakable On 1250 |
| Pulley Wheel | raio 0,45, Out 0,075, Wrap Over, motor 1 rad/s (57,296°/s), Axle Breaks On 1000 |
| Custom | eixo X Limited ±1,0, motor-eixo Rotation |
| Type Change | = Slider |

A Weston saiu da cena: com uma roldana só a talha não tem por onde voltar e a rota degenera («no route» no canvas, foto conferida); o canal dela prova-se em `canais_junta_tests`. A polia monta as pontas penduradas dos dois lados e a corda nasce na ponta A.

## 3 — Superfície de colisão (um merge pode partir)

- **Vocabulário:** `canal_junta.rs` (append; gate de renumeração cobre). Sem `PROJECT_SCHEMA`, sem `DOC_VERSION`, sem contrato §6.
- **`PropKind::AUTOKEYED` 11→15** (`ph2d-timeline`, muitas crates dependem; `PoseSample` cresce). Outra linha que mexa na lista colide: reconta-se pelo código.
- **`ph2d-preview-drive`:** enums `Driver`/`Driven` +2 variantes (`match` exaustivo fora da nossa área quebra).
- **`joint.rs`:** `Kind` do mesmo tipo = no-op (comportamento).
- **Shell:** `fase_physics_edits.rs` (−10 linhas), `autokey_amostra.rs`; `the_shell_only_shrinks` verde.
- **Gate do editor-core:** ESCOPO + censo novo; outra linha que adicione `sections/*.rs` de junta/roldana cai nele.
- Contadores como delta: `AUTOKEYED` +4; ids de canal da junta +12; ids de marca do Inspector +28; testes `canais_junta` 11; cenas de smoke +11 (16..26).

## 4 — Smoke do dono (em linguagem simples)

Binário: `target/smoke/ph2d-host-desktop` já compilado sobre `4b359d7f10` — 2.ª build `Finished smoke profile [optimized] target(s) in 0.87s`, zero «Compiling».

1. Rode `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=16 ./target/smoke/ph2d-host-desktop`. A cena mexe sozinha e pára no quadro 18 (a meio da volta).
2. Na Hierarquia, clique em cada junta, uma de cada vez: Pin, Spring, Rope, Weld, Slider, Rod, Wheel, Pulley, Pulley Wheel, Custom, Type Change. No Inspector, cada campo da lista abaixo tem de mostrar o valor certo e ter o losango:
   - Pin: modo Position, Target 0,6 rad (34,377°), Collide On.
   - Spring: Rest Length 0,9, Active Off.
   - Rope: Max Length 1,5, Anchor B World.
   - Weld: Soft, Stiffness 110.
   - Slider: limites ±1,0 m, motor 1,25.
   - Rod: Max Length 1,15, Breakable On 1250.
   - Wheel: Stiffness 250, motor 1,5.
   - Pulley: Breakable On 1250.
   - Pulley Wheel: raio 0,45, Out 0,075, Wrap Over, motor 1 rad/s (57,296°/s), Axle Breaks On 1000.
   - Custom: eixo X Limited ±1,0, motor do eixo Rotation.
   - Type Change: o tipo é Slider.
3. Troque à mão qualquer campo de uma junta, com o Auto Key ligado, num quadro sem chave: tem de nascer o losango e uma linha na Timeline.
4. Errado = campo sem losango; valor diferente do da lista; linha que falta na Timeline; «no route» vermelho na polia; o campo volta sozinho ao valor antigo no quadro seguinte à sua edição.

## 5 — Verificação

- `cargo fmt` ✓. `clippy -D warnings` nas 10 crates tocadas ✓.
- Testes por nome, verdes: `ph2d-ecs` `canal_junta`; `ph2d-app-physics` `canais_junta` (11) e `joint` (172); `ph2d-timeline` inteira 569/569 (`PH2D_SUITE=motivo: AUTOKEYED`); `ph2d-app-components` `marcas_inspector`; `ph2d-panel-inspector` `seam_wheel` (13) e `seam_joint` (33); `ph2d-preview-drive`; `ph2d-host-desktop` `autokey` (56).
- Gates: `the_shell_only_shrinks` e `architecture_every_editable_field_is_a_channel` (6 testes) verdes.
- **Suíte impactada NÃO corrida** (cadência: < 10 commits desde a última verde). Fica para o fecho da LINHA.

## 6 — ABERTO, por ordem

1. **Smoke do dono** da F26 (§4).
2. **O ESCOPO do Inspector é uma lista:** outras secções (câmera, nav, vida, HUD, …) estão fora do gate e fora de `DIVIDA_CRATE` — o mesmo buraco que escondeu as juntas, por família (Componentes).
3. `IkBendSide` (chip do esqueleto) continua fora do `AUTOKEYED` de propósito — rever à luz de «grava tudo».
4. **Rope Length de uma Pulley** não se grava por Auto Key (só K), porque a ponte o re-deriva (2.1).
5. A troca de tipo pela curva não passa pelo re-seed de limites quando escrita direto no componente (a cena escreve direto; o canal passa pelo funil).
6. Os itens 2–9 e 11 do §6 do [handoff F25](HANDOFF_CONTINUACAO_line_UIUX_2026-10-21_F25_O_BLOCO_2.md) continuam (o 10 está fechado por esta onda).

## 7 — Smoke do dono (22/10)

**Cenas 16–26: OK** — palavras do dono: *«smoke parece ok»*. Próximo passo por ordem dele: integrar a linha ao `main` (fecho da linha + integração por agente integrador).
