# HANDOFF de continuação — line/JointEditor, W4h–W4j feitas (15/11/2026)

Leitor: a próxima LLM da linha. Ramo `line/JointEditor`, worktree `Worktrees/line-JointEditor`, base = main `7b3dd983f7`, sem rebase.
Não é handoff de integração (a linha não fechou). Plano: [01_plano.md](../01_plano.md) · números: [02_gabaritos_medidos.md](../02_gabaritos_medidos.md) ·
anterior: [HANDOFF_CONTINUACAO_W3_2026-11-12.md](HANDOFF_CONTINUACAO_W3_2026-11-12.md) · memória `project-memory/project_joint_editor_version_b_2026_11_12.md`.

Commits desta jornada (`git log --oneline main..HEAD`):

| sha | onda | o quê |
|---|---|---|
| `5fa41a0b24` | W4h | a montagem é a PASTA das peças; cada junta mora no corpo A; porta de pose de mundo em `ph2d-physics-ecs/src/joint_pose.rs` |
| `bd506d6462` | W4i | olho/cadeado da pasta valem para o conteúdo; «Open in Joint Editor» no bloco Joints; View ▸ Joint Editor; smokes 10/11 |
| `248b6c813d` | W4j | a ponte (Bridge, id 8) passa à família Pin |
| `7529a3832b` | W4j | «Custom» por família (ids 19–27) e lista Template em grelha de três |
| `7d676760dc` | W4j | alças do motor, servo com «Target», «Pin to the scene» abre a montagem |
| `d59eba2bd9`, `519688feea` | W4j | `cargo fmt` em ficheiros da W4h/W4i |
| `d369627091` | W4j | fotos das cenas 1, 3, 5, 12, 13 |

O dono fez o smoke da W4h e da W4i: OK. A W4j ainda não tem smoke do dono (§5).

## 1. O que a W4j entregou

### 1.1 Bridge é da família Pin (`248b6c813d`)

- O gabarito Bridge (id 8) estava na família Rope, mas as juntas são Pins `one_chain` (medição no doc 02): a aba mostrava «Joint: Rope» sobre
  cartões de Pin. Passou à família Pin. Lei nova **`a_template_family_is_the_kind_of_its_joints`** (`ph2d-joint-templates`
  `tests/it/catalog.rs`; vermelha com Bridge em Rope, verde curada). Atualizados: chamadores `by_name`, `MACHINES` da cena 123, `01_plano`, `02`.
- `joint_assembly_tests.rs` (763 linhas, tecto 700, fora do fmt, herdado da W4h) partido: os 8 testes da PASTA foram para o filho
  `joint_assembly_folder_tests.rs` (591 + 257 linhas).

### 1.2 «Custom» por família (`7529a3832b`)

- Ids **19–27 no FIM do ALL** (`crates/ph2d-joint-templates/src/catalog/custom.rs`), `Template::hand_made`: duas peças («Part A»/«Part B»;
  ou Body/Wheel; Rail/Part B) e UMA junta do tipo com os números de fábrica do tipo (Seam nos de um ponto, Ends nos de duas com `AS_PLACED`
  na corda/haste, eixo no trilho/roda). A família Custom passa a ter gabarito (entra no seletor Joint).
- Leis: ids fixados +9; **`every_family_ends_with_its_hand_made_custom`** (substitui `every_family_but_custom_offers_a_template`);
  `every_template_assembles_its_factory_parts_and_holds` e `every_joint_of_every_template_lives_in_its_home` verdes com os nove.
- Aba: lista Template em grelha (`widgets::MENU_COLS = 3`, `MENU_CELL_MIN_W = 200`; maquete B1 tem 180 px com miniatura 64, a nossa 76;
  alarga para o palco). Custom = caixa tracejada com `IconId::Plus` e «You draw the parts»; «1 template» no singular.
- Costura **`a_lista_template_e_uma_grelha_de_tres_com_o_custom_por_ultimo`** (mutação `grid_cols=1` sangra).
- Smokes novos 12 (lista do Pin aberta) e 13 (Custom do Pin por fazer).

### 1.3 Alças do motor, servo, «Pin to the scene» (`7d676760dc`)

- `Handle::MotorSpeed`: a ponta da seta mostra até onde o motor leva a peça em `widgets::MOTOR_PREVIEW_S = 0,25 s` (carro −360°/s = quarto
  de volta; o arco cabe nos ±180° da alça, logo arrasto até ±720°/s; acima disso digita-se). Roda/pino: arco do topo, `Geo::Spin`; trilho:
  seta reta `Geo::Linear` com escala × 0,25.
- Servo (`MotorMode::Position`, o Flipper): o cartão dizia «Speed», que o servo não lê (controlo morto). Agora `Field::MotorTarget`
  «Target» + ponteiro até o alvo com `Handle::MotorTarget`.
- Defeito curado: «Pin to the scene» num gabarito por fazer criava a montagem mas a aba ficava no rascunho (`lib.rs frame()` só abria com
  `made.zip(line)` e `PlaceWorld` não tinha frase) ⇒ a peça seguinte caía noutra montagem («Flipper 1 (1)») e a junta não nascia. Agora abre
  sempre; `PlaceWorld` tem a frase `joint_editor.placed_world`.
- O entre-eixos já anda em mundo desde a W4h (só no Wheel de duas rodas, por desenho).
- Costuras: **`a_ponta_da_seta_do_motor_muda_a_velocidade`** (valor exato pela geometria; mutações sem dividir / outro campo / alça fora do
  arco sangram, 3/3) e **`o_servo_do_flipper_mostra_e_arrasta_o_alvo`** (mutações alvo→velocidade, cartão sem modo, foco sem a nova
  sangram, 3/3).
- `smoke.rs`: as faixas `n >= 10` e `n >= 8` viraram exatas (as cenas 12/13 caíam na aba Scene / painel mínimo).

## 2. Provas (verdes nesta onda)

- `ph2d-app-joint-editor`: 31/31. Catálogo: 5 por nome. Montagem: `every_template_…`, `every_joint_…`, folder 8, bridge 2.
  `ph2d-app-physics`: `physics_smoke_joint_templates`.
- clippy `-D warnings` limpo em `ph2d-joint-templates`, `ph2d-joint-assembly`, `ph2d-app-joint-editor`, `ph2d-i18n`,
  `ph2d-physics-smoke-scenes`. `cargo fmt --all` limpo. Binário smoke compilado (2.ª corrida: Finished em 0,23 s).

## 3. Contadores como DELTA e superfície

- `PROJECT_SCHEMA`: a W4j não soma degrau. Registro de componentes **+0**. Ids de gabarito **+9** (19–27). Crates **+0**.
- Colisão da W4j: `ph2d-joint-templates` (`catalog/custom.rs`, `catalog` ids, `tests/it/catalog.rs`) · `ph2d-app-joint-editor` (`widgets.rs`,
  `paint*.rs`, `lib.rs`, `smoke.rs`, `apply`/`doc`) · `ph2d-i18n` (`joint_editor_tab.rs`) · `ph2d-joint-assembly` (testes partidos) ·
  `ph2d-physics-smoke-scenes` (`physics_smoke_joint_templates.rs`, `MACHINES`). Um merge pode partir: a numeração dos ids de gabarito (outra
  linha que acrescente ao ALL), a lista de chaves `joint_editor.*`, e a faixa de smokes do `smoke.rs` (agora 1..13).
- ⚠️ **VERMELHO conhecido, vai no lote do fecho:** `the_shell_only_shrinks_and_every_app_crate_too` — `ph2d-app-joint-editor` **4 891** contra
  tecto 4 749 e `ph2d-joint-assembly` **2 416** contra 2 175. Já estavam acima no `bd506d6462` (4 798 e 2 331; main 4 748 e 2 135); esta onda
  somou +93 e +85. Cura = cortar/mover (comentário histórico longo em `joint_info.rs`, 133 linhas de comentário; `joint_create.rs`, 105),
  nunca subir o número.

## 4. ABERTO (por ordem)

1. **W4 «Make the parts»** (plano 01 §W4): objetos reais nomeados (`part_name`), forma de colisão filha por `nasce_com_forma` (ADR-0199),
   aparência simples, no centro da vista, selecionados, aviso «Made N objects: …», UM passo de desfazer; Bridge/Chain: N peças de UMA solta
   com o campo «Planks». Reaproveitar o construtor dos testes (`joint_assembly_tests.rs` `body`/`parts_of`) e o da cena 123
   (`ph2d-physics-smoke-scenes` `physics_smoke_joint_templates.rs` `block`/`parts`) — uma só porta.
2. Perguntar ao dono que ajustes o painel ainda precisa (13/11).
3. Decisão do dono: o cadeado de uma pasta COMUM também tranca o conteúdo? (hoje só a montagem.)
4. Fecho da linha: corte de LOC (§3); provas de mutação da W4h/W4i (`joint_home` a escolher B, `place_part` sem `adopt`, `#[require]` fora,
   `locked_by_a_folder` sem `Locked`, `is_off_canvas` só da própria entidade, registo do `INSP_JOINT_OPEN_EDITOR`); clippy/fmt das crates da
   W4h/W4i (`ph2d-physics-ecs`, `ph2d-ecs`, `ph2d-entity-visibility`, `ph2d-app-physics`); censos; handoff de integração §1.5.9 e a linha do §5
   do CLAUDE.md (smokes agora `PH2D_JOINT_EDITOR_SMOKE=1..13`).

## 5. Smoke do dono (W4j)

Fotos: [ponte Pin](Worktrees/line-JointEditor/docs/JointEditor/fotos/w4j_ponte_pin.png) ·
[cena 1](Worktrees/line-JointEditor/docs/JointEditor/fotos/w4j_cena_1.png) ·
[cena 5](Worktrees/line-JointEditor/docs/JointEditor/fotos/w4j_cena_5.png) ·
[cena 12](Worktrees/line-JointEditor/docs/JointEditor/fotos/w4j_cena_12.png) ·
[cena 13](Worktrees/line-JointEditor/docs/JointEditor/fotos/w4j_cena_13.png)

**A. Ponte (cena 3)**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && PH2D_PHYSICS_SMOKE=123 PH2D_JOINT_EDITOR_SMOKE=3 ./target/smoke/ph2d-host-desktop`
2. Abre a aba com a ponte. O topo diz «Joint: Pin» e embaixo estão os cartões Limits, Motor e Breaks.
3. Deu errado se o topo diz «Rope».

**B. Lista de gabaritos e Flipper (cena 12)**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && PH2D_PHYSICS_SMOKE=123 PH2D_JOINT_EDITOR_SMOKE=12 ./target/smoke/ph2d-host-desktop`
2. A lista «Template» já vem aberta: 6 gabaritos em 2 linhas de 3, o último é «Custom» com um «+».
3. Escolha «Flipper» e clique «Pin to the scene». Arraste um objeto da Hierarchy para a forma «Paddle».
4. A junta nasce na MESMA montagem; o cartão Motor diz «Target» (não «Speed») e há um ponteiro verde que se arrasta.
5. Deu errado se aparecer uma segunda montagem («Flipper 1 (1)»), se a junta não nascer ou se o cartão disser «Speed».

**C. Motor no desenho (cena 1)**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && PH2D_PHYSICS_SMOKE=123 PH2D_JOINT_EDITOR_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Em «Car 1» há um arco verde na roda de trás com uma bolinha na ponta. Arraste a bolinha: o número «Speed» do cartão muda junto.
3. Deu errado se a bolinha não existe ou o número não acompanha. Ctrl+Z desfaz num só passo.

**D. Custom (cena 13)**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && PH2D_PHYSICS_SMOKE=123 PH2D_JOINT_EDITOR_SMOKE=13 ./target/smoke/ph2d-host-desktop`
2. A aba mostra o Custom do Pin por fazer: duas caixas tracejadas, «Part A» e «Part B».
3. Deu errado se faltar uma das caixas ou se a lista não tiver o Custom.

## 6. Prompt de continuação (cole na janela nova)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && pwd && git branch --show-current
```
Confirme `line/JointEditor`. Leia `docs/JointEditor/01_plano.md`, a memória `project-memory/project_joint_editor_version_b_2026_11_12.md`
e este handoff. W1–W4j feitas. Comece o §4.1 («Make the parts»); o fecho da linha = §4.4 e o corte de LOC do §3. Inner loop =
`cargo-check-narrow.sh`; testes por nome; sem push.
