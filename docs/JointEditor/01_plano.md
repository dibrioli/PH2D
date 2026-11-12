# Joint Editor — plano da linha `line/JointEditor` (12/11/2026)

Leitor: a LLM que implementa. Pedido e decisões do dono: [memória](../../project-memory/project_joint_editor_version_b_2026_11_12.md).
Desenho de referência (maquetes A/B/C, fatos, pesquisa em 21 apps): `/home/enio/Documentos/GameEngineSamples/critica_ph2d.html#juntas`,
fonte `fonte_critica/onda3c_juntas.py`, fatos `fatos_juntas.md`, pesquisa `pesquisa_juntas.json`. **Escolhida: B.** «Faça melhor que no doc.»

## Decisões do dono (12/11) que limitam tudo

1. Aba central de juntas com **gabarito**: tipo de junta → montagem → arrastar cada objeto para o papel dele.
2. **O Inspector NÃO perde as juntas — melhora.** Ficam: cartão Joint, Join Selected Bodies, Draw Joint on Canvas, Chain N, Rig N Parts.
3. Sem monociclo.
4. **Objetos prontos sim, plenamente configuráveis e expostos; objeto mágico que surge do nada, não.** Todo objeto criado é real, nomeado, na Hierarchy, selecionado e anunciado.

## O que já existe (mapa, explorador 12/11 — confira antes de confiar)

| peça | onde |
|---|---|
| `PhysicsJoint` (Component, serde posicional; body_a/b = `stable_name_id`; entidade própria com `Transform` = âncora; aparece na Hierarchy) | `crates/ph2d-physics-ecs/src/joint.rs:117`, `:1-30`; `ph2d-editor-core/src/hier_types.rs:122` |
| 9 tipos; Wheel = Generic(LIN_Y) com mola em LinX e motor em AngX | `ph2d-physics-ecs/src/joint/kind.rs`; `ph2d-physics/src/world/joints.rs:143-149, 385-397` |
| Unidades: limites rad, motor rad/s ou m/s, distâncias m; a UI da Wheel da polia mostra °/s | `joint.rs:125-160`; `ph2d-app-physics/src/joint_wheel.rs:278` |
| Criar junta reutilizável: `create_joint`, `create_joint_at(..., at)`, `join_chain` | `ph2d-app-physics/src/joint_create.rs:34,58`; `joint_draw.rs:284` |
| Cartão Joint | `ph2d-panel-inspector/src/sections/joint*.rs`; modelo `ph2d-editor-core/src/screens/hero/inspector_model_joint.rs`; edição `ph2d-app-physics/src/canais_junta.rs:183-210` |
| Botões de criar | `ph2d-panel-inspector/src/sections/physics_join_rows.rs:138-247` |
| Aba central (molde) | `ph2d-app-input-map/src/{lib,surface}.rs` (`DocumentSurface`, `AppFamily`), `ph2d-app-registry-init/src/lib.rs:76`, `shells/desktop/src/init.rs:390`, teclado `input_dispatch/keyboard.rs:102` |
| Arrastar objeto → alvo | `DragPayload::Object` (`ph2d-editor-core/src/interaction/drag_payload.rs:30`), `DropTarget` (`ph2d-app-components/src/asset_drop.rs:37-75, 259-266`). Não há alvo de junta. |
| Testes | física: `ph2d-physics-ecs/tests/it/joint_motor.rs`, `joint_live_edit.rs`; costura: `ph2d-panel-inspector/tests/it/seam_joint.rs` |
| Schema | `PROJECT_SCHEMA = 237` (`shells/desktop/src/project_schema.rs:459`); escada + tripla em ficheiros irmãos — **contar**, nunca escolher |

## Ondas

### W1 — Gabaritos e montagem, por baixo (sem tela)
- Crate nova `ph2d-joint-templates` (dados puros): cada gabarito = papéis (nome, forma-guia, quantos) + juntas entre papéis (tipo, regra de âncora: centro do papel / borda / ponto de contato, eixo, números) + números de fábrica justificados no doc (medidos na física, não inventados).
  Catálogo inicial: Wheel {Car, Motorbike, Custom} · Pin {Door, Seesaw, Pendulum, Flipper} · Rope {Chain, Bridge, Hanging} · Slider {Elevator, Piston, Sliding door} · Spring {Trampoline, Soft platform} · Weld {Glue, Breakable} · Rod {Rigid pendulum} · Pulley {Lift} · Custom.
- Montagem = componente `JointAssembly` (template, papel → `StableId`, «iguais entre si» por grupo) numa entidade própria, nomeada («Car 1»), visível na Hierarchy; cada junta ganha `assembly` + `role` (campos no FIM, serde posicional). Sobe `PROJECT_SCHEMA` (§0.11: sem migração). ADR novo para o modelo.
- `apply(template, papéis)` usa `create_joint_at` (não duplica criação).
- **Prova:** testes de física headless — o carro de 3 corpos com motor atrás anda para a frente no chão; a porta gira e para no limite; a ponte de N tábuas aguenta; salvar → abrir devolve a mesma montagem; um Ctrl+Z desfaz a montagem inteira.

### W2 — Inspector melhor (os gestos de hoje ficam)
- Título do cartão pelo papel («Front wheel · Car 1»); rótulos por tipo e com unidade (Wheel: Suspension Stiffness/Damping, Travel up/down (m), Drive, Spin speed (°/s)); some o controle morto (Stiffness/Damping na Rod).
- «Same as the rear wheel» (caixa): editar uma junta do grupo edita todas.
- Cartão **Joints** no Inspector de cada corpo: as juntas em que ele está, editáveis, e «Open in Joint Editor».
- Body A/B aceitam arrastar da Hierarchy/canvas (`DropTarget::JointBody`).
- **Prova:** testes de costura dirigindo o clique/arraste real → o `PhysicsJoint` muda; caixas, nunca toggles (dono 10/11).

### W3 — A aba Joint Editor (a B)
- Crate `ph2d-app-joint-editor` no molde do Input Map: lista de montagens; seletores Joint e Template com miniaturas; desenho do gabarito com formas tracejadas por papel; arrastar um objeto para a forma (acende e diz o que vai acontecer antes de soltar); soltar cria a junta (undo); alças no desenho (entre-eixos, curso, motor, limite); painel de baixo com «Both wheels · Rear · Front». Abre por menu e pelo botão do Inspector.
- **Prova:** costura (arrastar → junta criada com o papel certo), cena de smoke `PH2D_JOINT_EDITOR_SMOKE=1..n` fotografada (foto ao lado da maquete B, acabamento incluído).

### W4 — Peças prontas, sem mágica
- «Make the parts» em cada gabarito: cria objetos reais e nomeados (Car Body, Wheel Back, Wheel Front com forma de colisão e aparência simples) no centro da vista, já selecionados, com aviso «Made 3 objects: …», tudo editável.
- Bridge/Chain: N peças a partir de UMA solta, com o campo «Planks» à vista; mudar o número refaz as cópias.
- **Prova:** teste (contagem e nomes na Hierarchy, undo desfaz tudo) + foto.

### Fecho
Gate batched 1× (testes por nome, fmt/clippy das crates tocadas, mutação na lei do `apply`), README do módulo, ADR, entrada §5 do CLAUDE.md, handoff §1.5.9, smoke em passos numerados ao dono.
