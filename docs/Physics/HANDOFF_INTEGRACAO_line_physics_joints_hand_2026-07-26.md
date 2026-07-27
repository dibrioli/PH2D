# HANDOFF DE INTEGRAÇÃO — `line/physics`, jornada de 2026-07-26

> **Para o agente INTEGRADOR.** A linha está FECHADA. Todos os smokes foram
> aprovados pelo Enio. Este documento é a única coisa que você precisa ler para
> integrar; o detalhe por-wave vive no tracker
> ([`HANDOFF_line_physics.md`](HANDOFF_line_physics.md)) e o mapa em
> [`00_plano_waves.md`](00_plano_waves.md).

## §0 — GO / NO-GO

**GO.** Ordem explícita do Enio em 2026-07-26: *"smoke OK. Antes de continuar
vamos integrar ao main."*

- Branch: **`line/physics`**, worktree em `Worktrees/line-physics`.
- Tip: **`c26a25fa7`** (+ um commit de docs marcando os smokes aprovados).
- Base: **`0afc6bb28`** — e o `main` **NÃO andou** desde o fork
  (`git rev-list --count $(git merge-base main HEAD)..main` = **0**).
- **21 commits**, **146 arquivos**, +27.317 / −1.594.

⚠️ Como o `main` não andou, a integração é `--ff-only` **limpa**. Se ela deixar de
ser (outra linha entrou primeiro), leia a §5, que é onde está tudo o que pode
conflitar.

## §1 — O que a linha entrega

Duas frentes, na ordem em que foram construídas.

### (A) O plano 02 — JOINTS: UI/UX + os tipos que faltavam

`docs/Physics/02_plano_joints_ui_authoring.md`, **as nove linhas fechadas**:

| wave | o quê | cena |
|---|---|---|
| W-JointParams | tunar param de joint AO VIVO (2 causas: gate `at_rest` obsoleto + fila sem flush) | `=42` |
| W-J1 | **o joint DESENHA o que ele é** — glifo por tipo, não um segmento genérico | `=43` |
| W-J2 / W-J2b | a âncora tem **DUAS alças** + ímã (snap CTRL aos 9 pontos do collider) | `=44` |
| W-J3 | **pose, não digite**: limite e comprimento autorados no canvas | `=45` |
| W-J4 / W-J4b | **criar onde se olha** (press em A, arrasta, solta em B) | `=46` |
| W-J5 / W-J5b | **o TRILHO** (Slider/prismatic), o 5º tipo | `=47` |
| W-J6..W-J6d | **o SERVO e o GUINCHO** (o motor ganha modo Position e chega ao Slider/Rope) | `=48` |
| W-J7 / W-J7b | **o joint que PARTE sob carga** + o readout de carga | `=49` |
| W-J8 | **a higiene do PAR** (Active · Collide · Swap A↔B · o nome) | `=50` |
| W-JG | **o grupo carrega o rig** (Alt+arrastar move o componente conexo INTEIRO) | `=51` |

### (B) A ferramenta de INTERAÇÃO com a cena rodando

| wave | o quê | cena |
|---|---|---|
| W-Grab | **a MÃO**: pegar um corpo no PLAY (mola via solver, não teleporte) | `=52` |
| W-Hand | **a seção da FERRAMENTA**: 3 modos de segurar · explosão · campo de atração · **o bug do collider fantasma** | `=53` |

## §2 — Números que se CONTAM (confira, não copie)

| pin | no fork | no tip | por quê |
|---|---|---|---|
| `PROJECT_SCHEMA` | 31 | **34** | v32 W-J6 (`motor_mode`+`motor_target`) · v33 W-J7 (`break_*`) · v34 W-J8 (`active`+`collide_connected`) — três campos APENDADOS ao `PhysicsJoint`, postcard posicional |
| registro `ph2d-ecs` | 21 | **21** | nenhum componente novo nesta jornada |
| `physics_ecs_c9` | — | **`c9d4baee…`, 87 corpos** | debug ≡ release; **byte-idêntico** entre a W-Grab e a W-Hand |
| ids de gizmo numéricos | ≤**964** | **965, 966, 967, 968** | o 964 (`GIZMO_JOINT_ANCHOR`) **já está no `main`** desde a integração de 07-25; esta jornada acrescenta `GIZMO_JOINT_ANCHOR_B` (965) · `GIZMO_JOINT_LIMIT_{MIN,MAX}` (966/967) · `GIZMO_JOINT_LENGTH` (968). **Próximo livre: 969** |

⚠️ **O `PROJECT_SCHEMA` é o item nº 1 do seu checklist.** Ele já foi renumerado
uma vez nesta linha (o v30 desta linha e o v30 da `line/FLIP` na mesma janela
viraram 30 e 31). Se outra linha bumpar na mesma janela, **o valor certo não está
em nenhum dos dois lados do conflito — conte**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]), e reescreva os
parágrafos `v3X` do `project.rs` na ordem em que os bumps de fato empilharam.

⚠️ **Os quatro ids de gizmo NOVOS são numéricos e sequenciais** — a única
superfície desta jornada que colide por VALOR e não por nome. Se outra linha
reivindicou 965-968, renumere OS DESTA (os nomes diferem, então o git funde limpo
e ninguém percebe: é o padrão que já apareceu 3× no repo).

**Nenhum ADR novo.** Tudo cai sob o ADR-0131 já aceito.
**Nenhum contrato congelado (§6) foi tocado** — conferido por grep, não por
auto-relato.

## §3 — O gate de fechamento, como ele estava aqui

Rodado no tip, tudo verde:

```
cargo fmt --all --check                                        # OK
cargo clippy -p ph2d-physics -p ph2d-physics-ecs \
             -p ph2d-panel-physics -p ph2d-host-desktop --all-targets   # zero warning
cargo test -p ph2d-physics -p ph2d-physics-ecs -p ph2d-panel-physics \
           -p ph2d-panel-inspector -p ph2d-editor-core -p ph2d-host-desktop
                                                               # 178 blocos ok, 0 failed
typos                                                          # OK
cargo run -p ph2d-physics-ecs --bin physics_ecs_c9             # c9d4baee…, 87
cargo run --release -p ph2d-physics-ecs --bin physics_ecs_c9   # idem
```

⚠️ **Quatro gates que um `cargo test -p` filtrado NÃO alcança** e que esta linha
já viu nascerem vermelhos-latentes noutras jornadas — rode-os por nome:

```
cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap
cargo test -p ph2d-host-desktop --test file_loc_caps
cargo test -p ph2d-editor-core --test architecture_panel_wiring_parity
cargo test -p ph2d-editor-core --test arch_safe_clamp_only
cargo test -p ph2d-editor-core --test no_magic_numeric
```

(Os dois de LOC moram em crates diferentes e cobrem árvores diferentes —
`crates/` e `shells/`; o `arch_safe_clamp_only` já ficou vermelho-latente por uma
wave inteira nesta linha.)

## §4 — Foundational tocado (26 arquivos fora de `ph2d-physics*`)

Tudo **aditivo**. Em ordem de risco de conflito:

- **`ph2d-editor-core/src/ids/`** — `chrome/physics.rs` (a família `PHYSICS_*` da
  seção Interaction, slugs hasheados) · `inspector.rs` + `inspector_joint.rs`
  (`INSP_JOINT_*`, `INSP_PHYS_*`) · `mod.rs` (re-exports) · `gizmo/hit.rs` (os 4
  ids NUMÉRICOS novos — §2).
- **`ph2d-editor-core/src/gizmo/`** — `point.rs`/`point_tests.rs`/`mod.rs`: o
  `PointGizmoView` ganhou `PointHandleKind` (A/B/limite/comprimento). Módulo
  IRMÃO, ponto de extensão do W-JointAnchor.
- **`ph2d-editor-core/src/screens/hero/`** — `inspector_model_physics.rs` (os
  tipos das §11/§12, o arquivo desta linha desde o W8) · `paint.rs`/`state.rs`
  (fiação do gizmo de ponto).
- **`ph2d-panel-inspector/`** — §11 e §12 (`sections/physics*.rs`,
  `sections/joint*.rs`, `event_*`, `populate*`, `sync.rs`) + os 2 seams.
- **`ph2d-i18n/src/lib.rs`** — **+18 chaves** `panel.physics.*` (Interaction) e as
  do §12. **Só ADIÇÕES**, no meio do bloco `panel.physics.*`: se conflitar, é
  texto puro e a resolução é manter os DOIS lados.
- **`ph2d-panel-physics/`** — a crate é desta linha; a seção Interaction é
  `interact.rs` + `paint/interact.rs` (arquivos NOVOS).

⚠️ **Nada em `ph2d-ecs`, `ph2d-render`, `ph2d-script`** ⇒ o contador triplo de
componentes (que já ficou vermelho-latente duas vezes no repo) **não é problema
desta integração**.

## §5 — O que provavelmente vai conflitar, e como resolver

1. **`shells/desktop/src/render_loop/mod.rs`** — a chamada de
   `physics_overlay::draw` cresceu para **23 argumentos** (as 3 marcas da
   ferramenta) e ganhou vizinhos novos (`pointer_world`,
   `crate::body_grab::age_blast_flash`). Se outra linha mexeu no mesmo bloco,
   resolva pelos **ESTÁGIOS do índice**, nunca pelos marcadores
   ([[feedback_resolve_conflicts_from_index_stages_not_markers]]).
2. **`shells/desktop/src/input_dispatch.rs`** — três inserções: o intercept modal
   das ferramentas de ponto (ANTES do picking), o `poke_press`, e o release no
   topo do `on_mouse_input`. **A ORDEM é load-bearing e há arch-gate afirmando-a**
   (`the_point_tools_are_intercepted_before_the_canvas_pick`): se o merge mover o
   intercept para depois do pick, o gate fica vermelho — é ele funcionando.
3. **`shells/desktop/src/app_state.rs` + `main.rs`** — dois campos novos
   (`interaction`, `blast_flash`) e dois `mod`. Listas compartilhadas: **só
   ADICIONE** ([[feedback_a_shared_list_is_merged_against_todays_main]]).
4. **`crates/ph2d-i18n/src/lib.rs`** — ver §4.
5. **`shells/desktop/src/physics_smoke.rs`** — o dispatch de cenas ganhou `"53"`.
   Lista compartilhada com nada mais; append puro.

**Depois de CADA commit resolvido:** varra marcadores de conflito
(`git grep -nE '^(<<<<<<<|=======|>>>>>>>)'`) e rode `cargo check --workspace` —
um merge textual limpo pode estar semanticamente quebrado
([[feedback_clean_text_merge_can_be_semantically_broken]]).

## §6 — Mudanças de COMPORTAMENTO que alguém pode notar

Três, e todas foram smokadas:

1. **Um corpo ESTÁTICO arrastado com o relógio ANDANDO agora move o collider
   junto.** Antes, o desenho ia e o collider ficava (o bug reportado). Se algum
   gate de outra linha assumia o comportamento antigo, ele agora é *o gate certo
   falhando pelo motivo certo*.
   ⚠️ **A cena 52 teve o passo 5 REESCRITO** por isso: ele afirmava que arrastar o
   muro estático *"não faz nada"*, o que esta wave tornou FALSO.
2. **O default do motor de joint mudou** (W-J6: `MotorMode` nasce em `Velocity`,
   e o motor passou a existir no Slider e na Rope). Saves v31 são **recusados** —
   é o que o bump de schema promete.
3. **Alt+arrastar um corpo jointado em repouso arrasta o rig inteiro** (W-JG).
   Sem Alt, o gesto é o de sempre.

## §7 — Os smokes, para a re-verificação pós-merge

Todos `--release`. **A jornada inteira foi aprovada; rode os dois últimos como
sanidade** (são os que tocam o maior número de arquivos):

```
cd <árvore combinada>
env PH2D_PHYSICS_SMOKE=52 cargo run -p ph2d-host-desktop --release   # a MÃO
env PH2D_PHYSICS_SMOKE=53 cargo run -p ph2d-host-desktop --release   # a FERRAMENTA + o bug do estático
```

A cena 53 **imprime o roteiro de 9 passos com os números medidos**; se essa linha
não aparecer no terminal, pare — o resto não significa nada (a lição que a cena do
Colorize pagou).

As cenas dos joints (`=42`..`=51`) só precisam ser re-rodadas se o merge tocou
`ph2d-panel-inspector` ou `gizmo/`.

## §8 — Depois da integração

- Atualize **`CLAUDE.md` §5** (o bloco de Física): a jornada acrescenta as 12
  waves da §1, o `PROJECT_SCHEMA` **34**, o c9 **87 corpos** e as cenas **42..53**.
  ⚠️ O texto de lá afirma hoje `PROJECT_SCHEMA` **29** e **28 cenas** — os dois
  números estão desatualizados desde a integração de 07-25, e **um número falso é
  pior que um ausente** (foi a lição que a própria §5 registrou em 07-23).
- Atualize **`docs/SESSION_ACTIVE.md`** (quem possui o quê).
- **Não faça `git push`** — o push é 1× por jornada e é ordem do Enio (§0.7 do
  `CLAUDE.md`).

## §9 — Aberto na linha (não bloqueia a integração)

- A explosão **não impõe torque** (decisão medida; um estouro que gira exigiria
  escolher um ponto de aplicação).
- O campo **repelindo arremessa para fora de quadro** (medido: −20 N abre a nuvem
  para 9,23 m em 1 s). É força sustentada sem freio fora do alcance.
- **Rigid e Rope atravessam parede** — inerente à rigidez infinita, nomeado no
  doc, no enum e no smoke.
- Soltar a mão deixa **um passo de undo** (pré-existente de QUALQUER clique no
  play; a cura mora no roteador de undo, outro domínio).
- O resto do horizonte do plano 02 §8 (IK multibody · params keyframáveis ·
  Wheel preset · Rod/soft weld · copiar-colar propriedades) segue **não
  escalonado**, aguardando decisão do Enio.
