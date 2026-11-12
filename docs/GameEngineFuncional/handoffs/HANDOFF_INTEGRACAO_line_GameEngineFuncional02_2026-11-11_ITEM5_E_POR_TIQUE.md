# HANDOFF de integração — `line/GameEngineFuncional02`, 11/11 (Look da forma de colisão · física fluida sob animação · canais por tique · item 5 herdado · janela do jogo à frente)

Leitor: o agente integrador. Anterior (já no `main`): [OUTLINE_E_F3](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-09_OUTLINE_E_F3.md). Mecanismo de cada onda, nos handoffs de continuação linkados em §2 — não repetido aqui.

## 1. Identidade

Branch `line/GameEngineFuncional02`, HEAD `104dbc5461`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02`. Merge-base com o `main`: `bc4a6ce930` (o `main` do começo desta linha). **61 commits**, 266 ficheiros, +7 983 −2 094 (`git diff --shortstat bc4a6ce930..HEAD`). Nada integrado, nenhum push. **Suíte completa e `nextest-impacted` NÃO correram** (regra do dono 23/10: são do integrador). Árvore: só `.log`/`.png` de fotos por rastrear em `fotos_fisica/` (lixo do `fotografa_cena.sh`; comite ou apague).

## 2. O que entra (por onda; `git log --oneline bc4a6ce930..HEAD`)

- **Onda 5** — [ONDA5](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA5.md). «Set Scale to 1» conserta a cadeia (forma + pais esticados) num clique, um passo de undo `3bb9f4e91c` (`InspectorPhysicsInfo::estica` `Option<u64>` → `Vec<u64>`; `PhysicsFieldEdit::ScaleToOne`). Sai o «Create this action» do cartão do Trigger; só o «Create and set key» cria `cab51710a9`.
- **Onda 6** — [ONDA6](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA6.md). Bind da janela Input Map sobre a escuta do cartão cancela e não re-arma (`input_map_listen_cancelled_by_down`: `bool` → `Option<ActionId>`) `43593d8140`; tabelas do Outline podadas às formas vivas (`sweep_outlines` no `prepare`) `f486a4e8e2`; canais da JUNTA falam GRAUS como o cartão `ee202ff305`; `every_physics_component_is_authorable` sai da shell `279d4900d9`.
- **Onda 7** — [ONDA7](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA7.md). Física fluida sob animação: juntas de mundo no referencial do objeto (`joint_desc(.., lado_b_mundo)`), formas reafinadas no lugar e por tique (`drive_shapes`), `retune_body` acorda sempre `f7b233f13b`; cena 66 animada `PH2D_PHYSICS_SMOKE_ANIMA=1` `b9a15ad2d6`.
- **Onda 8** — [LOOK](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_LOOK.md) · [SMOKE_E_LOOK_B](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_SMOKE_E_LOOK_B.md) · [ITENS_2_A_4](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_ITENS_2_A_4.md) · [LOOK_TRACO](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_LOOK_TRACO.md). `ShapeLook { pivot, tint, opacity, … }` no lado-forma, cartão Look, ferramenta Pivot do canvas `e28c5da10d`..`f16259aadd`; Color/Opacity pintam o contorno `35e5d16d76`; retorno do smoke `c22f3377cf`; Look ao associar volta ao padrão do corpo `eb702f0a17`; Dash e largura do stroke `92fb36208c`; rótulo que não cabe = reticências `80670dbced`. Penetração na rampa (`reshape_push`) `ff04dccc4a`; Width/Height por tique `757d483a8b`; motor da junta em graus no cartão `3a9bdae8eb`; livro do quadro `be48b89680`; cena 122 (`PH2D_PHYSICS_SMOKE=122`, caixa que cresce).
- **Item 5 + por tique** — [ITEM5_E_POR_TIQUE](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ITEM5_E_POR_TIQUE.md). 5.1 censo de sinais ouve o painel do artista `5b8fc8ca46` · 5.3 contorno acima do tecto desiste após o 1.º corte (`trace_fitted` 256² 1,18 s → 40 ms) `8e150966ef` · 5.4 nav/impressão contam todas as formas do corpo `b0c883aa53` · 5.5 macro não troca contorno com pontos animados `bb5aa69d2c` · 5.6 painel clássico apaga os travados `d509c7100a`. JUNTA e MUNDO chegam ao solver por tique (`SceneAtTick::mundo`, `PhysicsBridge::poe_a_cena`, `Livros{drive,fisica,mundo}`) `fe7ed896ec`; o livro da junta guarda o documento (`Driven::com_outra_mao`, fusão de 3 vias campo a campo) `e55a3a90bd`.
- **Onda 10 (pós-fecho, 11/11)** — a janela do jogo vem à frente e o editor parado mostra um aviso `c8ccc1b6b7` (+ foto `104dbc5461`). Ver §2.1.
- **Fecho (esta janela, 11/11)** — cápsula (a) e caixa de sinal (d) MEDIDAS e não confirmadas (§6); `FOTO_LAYOUT` do `fotografa_cena.sh` `1ece3815cb` (`FOTO_LAYOUT` ausente quebrava o heredoc sob `set -u`); fmt e clippy `a9d6da935f` `ace77ee00f`; cortes de comentário `200f104a9a` `9e7b28957c`.

### 2.1 Onda 10 — «app travou» não era travamento

Relato do dono 11/11 («app travou» ao simular). Diagnóstico por perf no pid vivo: não é travamento nem a forma de colisão; o Play com o palco «Window» abriu o jogo ATRÁS do editor maximizado (KWin/Wayland ignora `focus_window()` sem ficha) e `da_outra_janela` descarta os eventos do editor ⇒ cara de congelado. Pré-existente desde `bc3bf2ea58`. Dono: as duas — janela à frente + aviso sobre o editor. `game_stage.rs` (`passo_da_janela`, `QUADROS_SEM_FICHA = 30`, 4 testes, 7/7 mutantes), `screens/hero/aviso_do_jogo.rs`, `janela_do_jogo.rs` (ficha xdg-activation; clique no editor → `request_user_attention(Critical)`), i18n `shell.janela_do_jogo.running`, `undo.rs::pedido_da_janela`. Gates `the_shell_only_shrinks`/`no_magic_numeric`/clippy verdes; a fiação da shell não tem teste headless. Mecanismo e foto: [ONDA10](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-11_ONDA10_JANELA_DO_JOGO.md).

## 3. Superfície de colisão (colada de `bash <worktree>/scripts/collision-surface.sh`, 11/11 — REFERÊNCIA; o integrador lê o `main` no ficheiro)

```
SUPERFÍCIE DE COLISÃO — line/GameEngineFuncional02 contra main
  merge-base bc4a6ce930   ·   61 commit(s)   ·   266 arquivo(s)
  ⚠ PROJECT_SCHEMA                        233   (base: 231)
  ⚠   └ tripla do gate               (233, 13, 29)   (base: (231, 13, 29))
    VEC_SCENE_SCHEMA 29=29 · FLIP_SCHEMA 13=13 · DOC_VERSION (timeline) 25=25
  REGISTRO: ph2d-ecs 121 (base 121)
  ✗ ph2d-render / ph2d-script (espelho = ecs + K)  —  (base: —)
  CONTRATO CONGELADO: node.rs intocado · tool.rs intocado
  ADR: último no disco 0200, próximo livre 0201; esta linha não cria ADR
  Cargo.lock: nenhum '+name' externo novo (arestas internas: ph2d-shape-handles→ph2d-anim/libm/ph2d-inspector-ordering, ph2d-app-macros→ph2d-shape-handles, ph2d-panel-physics)
  MARCADORES DE CONFLITO: nenhum
  ✗ TETOS DE LOC: 703/700 editor-core/src/screens/hero/paint.rs · 997/600 shells/desktop/src/main.rs · 601/600 shells/desktop/src/undo.rs
  ✗ SONDA CEGA em ph2d-render, ph2d-script (bug de caminho do script, «—» também na base; não é desta linha)
```

Tetos de LOC (leitura): `main.rs` 1 031 (base) → 997, só encolheu; `paint.rs` 700 → **703** (+3, onda 10) e `undo.rs` 599 → **601** (+2, onda 10) passam do teto do script — o integrador confirma se o gate os pega; a cura é cortar comentário nesses dois, nunca subir o número.

**Contadores como DELTA** (o integrador RECONTA: `python3 scripts/schema-recount.py`):

- `PROJECT_SCHEMA` **+2: 231 → 233** em `shells/desktop/src/project_schema.rs:432` (`PROJECT_SCHEMA: u32 = 233`). Degrau 231→232 `e28c5da10d` (componente `ShapeLook` no registo); 232→233 `92fb36208c` (`ShapeLook` ganha `largura`, `tracejado`, `traco`, `traco_proprio`). A tripla `(233, 13, 29)` NÃO vê estes degraus (escada e tripla são ficheiros irmãos). O `main` estava em 231 no merge-base e ainda em 231 ao fechar (`git show main:shells/desktop/src/project_schema.rs`). Se outra linha subir antes, reconte pela escada, nunca escolha. Sem leitor de formato velho (§0.11).
- Registo `ph2d-physics-ecs` **+1** (48 → 49, `ShapeLook`; `assert_eq!(reg.len(), 49)`); espelhos `ph2d-app-macros/src/mundo/portas.rs` e `ph2d-component-desc/src/catalog/physics.rs` já alinhados (confira com o gate do registo). Registo `ph2d-ecs` 121 = 121.
- Cenas de smoke **+1** (a `PH2D_PHYSICS_SMOKE=122`; a 66 ganhou uma env, não cena; a 48 mudou de crate). ADR 0. Contratos §6 0.
- `unidade_mundo::FIXOS` 52 → 54 (ONDA8_LOOK).

**Pontos quentes de linhas vivas.** `ph2d-app-components` (PixelArt) 5 ficheiros (`trigger_smoke*.rs` + 3 de testes; `action_trigger_inspector.rs` não tocado). `ph2d-timeline` 5 ficheiros (`apply_prop`/`read_prop_kind` com graus do motor). `ph2d-editor-core` 18 ficheiros (`input_map_*`, `action_bus.rs`, `store_*`). `ph2d-i18n` 4. `shells/desktop` 15 ficheiros, +84 −36 nos `src`.

## 4. O que um merge pode partir

- Literais `InspectorPhysicsInfo { estica: … }` (agora `Vec<u64>`; `look: ShapeLookFacts::NONE` é campo novo); `match` exaustivo sobre `PhysicsFieldEdit` (`ScaleToOne`, `Look*`); `Tinta::Camada` ganhou `tom: Tom` (`tom: TOM_NEUTRO`).
- `take_listen_cancelled_by_down()` agora `Option<ActionId>`. Removidos: `EditorAction::CreateInputAction`, `PedidoDoMapa::Criar`, `INSP_TRIGGER_CREATE_ACTION`, `INSP_SHAPE_{TINT,OPACITY}_RESET`, `PhysicsFieldEdit::LookReset*`.
- `joint_desc(.., lado_b_mundo)` (5.º parâmetro); `JointView` de pino de mundo vem no referencial do OBJETO; `retune_body` acorda sempre; `ControllerMemory` +2 campos (`formas`, `pecas`).
- **Unidades de canal gravadas mudaram SEM migração:** canais da JUNTA em graus (limites angulares, eixo Rotation do Custom, motor da roldana) e `JointMotorTarget/Speed` em graus (rotação) / metros (curso). Fixtures de outra linha com chaves em rad nestas props partem (§0.11: apague).
- `TimelineScene::put` aplica escritas FISICA e JUNTA/MUNDO; `dispatch` chama `scene.put` antes de `prepare`; `TimelineScene.livro = Livros{drive,fisica,mundo}`, `dispatch` recebe `&mut self.physics.mundo`. Quem reordenar `drive_shapes`/`drive_joint_params` nos laços de dispatch/rewind perde o bit-exacto do scrub.
- `PhysicsBridge::prepare` chama `sweep_outlines`; outra tabela de outline fora do `OutlineCache` fica sem poda.
- `cards_sprite_pivot::grade` mudou de assinatura (`nome: Option<(NodeId, bool)>`). `ph2d-app-macros` agora depende de `ph2d-shape-handles`.
- `shells/desktop/tests/it/main.rs` perdeu um `mod` (teste movido `279d4900d9`): conflito nessa linha para quem a editar.
- **Catracas** (`find crates/<c> -name '*.rs' | xargs cat | wc -l`): `ph2d-app-physics` **37 750 / 37 798**; `ph2d-app-macros` 14 598 / 14 630; `ph2d-app-components` **80 869 / 80 869 (no tecto, intocado em LOC)**; `physics_apply.rs` 594/600; shell: líquido −1 na onda 10 (o gate `the_shell_only_shrinks` verde). Cura é sempre MOVER ou cortar comentário, nunca subir o número.

## 5. Prova de fecho (na linha, 11/11)

- `cargo fmt` workspace verde após `a9d6da935f`; onda 10: rustfmt limpo, clippy `-D warnings` em `ph2d-editor-core`, `ph2d-i18n`, `ph2d-host-desktop` (§2.1).
- `cargo clippy -D warnings` verde em 13 pacotes após `ace77ee00f` (as crates tocadas até o fecho, incluindo `ph2d-app-physics`, `ph2d-timeline`, `ph2d-host-desktop`).
- Gate de LOC `the_shell_only_shrinks_and_every_app_crate_too` (`ph2d-editor-core --test it`) verde.
- `cargo machete` nas 3 crates com `Cargo.toml` tocado (`ph2d-app-macros`, `ph2d-physics-smoke-scenes`, `ph2d-shape-handles`): nada não usado.
- Cada cura das ondas tem teste nascido vermelho e mutação sangrada (números nos handoffs de continuação).
- `scripts/agent-loop-profile.sh`: paralelismo 1,15 ✗, test:check 2,9× ✗, contexto/passo 252 mil ✗; as outras réguas ✓.
- **NÃO correram (do integrador):** suíte completa, `nextest-impacted`, `ship.sh`, `censos-da-arvore-combinada.sh`, cruzamento `aarch64-apple-darwin` (nenhum `#[cfg(target_os)]` novo foi escrito de propósito; confirme na combinada).

## 6. Fecho do ABERTO herdado (OUTLINE_E_F3 §11 — tudo fechado)

- **11.1** «Set Scale to 1» em cadeia, um clique, um undo `3bb9f4e91c`. **11.3** tabelas do Outline `f486a4e8e2`. **11.4** «Create this action» saiu `cab51710a9`.
- **11.2(a)** cápsula (raio + altura com chave no mesmo quadro): **não confirmada**; teste `raio_e_altura_com_chave_dao_a_capsula_do_tique_no_solver` (`canais_por_tique_tests.rs`), mutação vermelha `a4a841bff6`. **(b)** «Delete animation» = 1 Ctrl+Z: não confirmada. **(c)** Bind do Input Map sobre a escuta: confirmada e curada `43593d8140`. **(d)** censo contar a própria caixa: não confirmada, `nenhuma_caixa_e_a_sua_propria_outra_ponta` `f261f173f6`.
- **11.5** herdados: 5.1 `5b8fc8ca46` · JUNTA em graus `ee202ff305` · 5.3 `8e150966ef` · 5.4 `b0c883aa53` · 5.5 `bb5aa69d2c` · 5.6 `d509c7100a` · JUNTA/MUNDO por tique `fe7ed896ec` + livro `e55a3a90bd`.

## 7. Smokes (dono) — o dono JÁ fez; o integrador pode repetir

Smoke do dono 11/11: **OK.** ⚠️ A dica do primeiro smoke usou `PH2D_CARDS=0` (painel clássico) e o dono viu regressões que eram o painel antigo. **Smoke corre sempre na UI de cartões, a omissão.** Fotos: [cartões 120](../smoke/onda9/cartoes_120.png) · [alta](../smoke/onda9/cartoes_120_alta.png) · [inspector baixo](../smoke/onda9/insp_baixo.png).

1. `cd /home/enio/Documentos/Projetos/PH2D && PH2D_PHYSICS_SMOKE=120 PH2D_PHYSICS_SMOKE_OUTLINE=Rock cargo run -p ph2d-host-desktop --profile smoke` (na árvore combinada; sem `PH2D_CARDS`).
2. Selecione a Rock e abra o cartão Outline e o cartão Look da forma; mexa em Detail, Pivot, Color, Opacity, Dash.
3. Tem de acontecer: o contorno e a forma acompanham; mudar o Pivot não move a forma no mundo; Ctrl+Z desfaz um passo.
4. Deu errado se: um campo travado do Outline não apaga, o pivô move a forma, ou aparece o painel clássico (então `PH2D_CARDS=0` ficou no ambiente).

### Smoke da onda 10 — PENDENTE (dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_GAME_SMOKE=1 ./target/smoke/ph2d-host-desktop` (compile antes com `cargo build -p ph2d-host-desktop --profile smoke`).
2. Clique em Play (▶) na barra de cima.
3. Tem de acontecer: a janela «PH2D · Game» abre NA FRENTE; o editor fica escurecido com o aviso «Game running in its own window — close it to stop»; clicar no editor escurecido traz o jogo de volta; ao fechar a janela do jogo o aviso some e o editor responde.
4. Deu errado se: o jogo abre atrás sem aviso, o aviso fica depois de fechar o jogo, ou o clique só faz o ícone piscar na barra de tarefas (então a reativação no Wayland não funciona, §8).

Outras cenas desta linha: `PH2D_PHYSICS_SMOKE=122` (caixote que cresce, Height 1↔2 m), `PH2D_PHYSICS_SMOKE=66 PH2D_PHYSICS_SMOKE_ANIMA=1` (portão com limite e largura animados), `PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=fisica_esticada`, `PH2D_TRIGGER_SMOKE=1`. Todas já aprovadas pelo dono em 10/11.

## 8. ABERTO após a linha

- **Smoke do dono da onda 10 pendente** (§7). **Medir `QUADROS_SEM_FICHA = 30`** (raciocinado, não medido). **Reativação no clique em Wayland NÃO verificada em KWin real** (no Wayland pede ficha na superfície do jogo; se falhar, só pisca na barra). Cosmético conhecido: toasts de arranque pintam acima do escurecimento; o ícone do gizmo da câmera fica logo abaixo do cartão.
- **Lição:** relato de «travou» → primeiro olhe a janela/palco do jogo (abriu atrás?) antes de perfilar.
- **Nada do §11.** Do que os handoffs de continuação listam: tudo tem commit acima.
- **Lacuna conhecida, inofensiva** (`e55a3a90bd`): se a curva da junta fica parada depois do quadro 1 não há chamada `driven`, logo nenhuma fusão até ela mudar; o autorado continua o documento.
- **Limite 5.1:** o censo só vê as linhas que o painel do artista pinta agora (ao vivo se visível, senão o compilado).
- **Recusa MEDIDA (não reconstruir):** a regra «só a pose no tique 0» e um campo `inicio` para a forma animada (onda 7, mutação não sangrou).
- Para o integrador: a suíte inteira, o `ship.sh` e a árvore combinada; depois o §5 do CLAUDE.md já aponta para este handoff.
