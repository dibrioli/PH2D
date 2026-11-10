# HANDOFF de continuação — `line/GameEngineFuncional02`, 10/11, onda 8 item 1 (o Look da forma de colisão)

Leitor: o próximo agente da linha (ou o integrador). Anterior: [ONDA7](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA7.md) (a §7 lista a onda 8). Memória: [decisões do Look da forma](../../../project-memory/project_collision_shape_look_2026_11_10.md).

## 1. Identidade

Branch `line/GameEngineFuncional02`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02`, 10 commits sobre `c6a53ee0e9` (`e28c5da10d..ee7156bd10`). **`PROJECT_SCHEMA` 231 → 232** (+1, `ShapeLook` no registo; escada + tripla). Registo `ph2d-physics-ecs` 48 → 49. ADR 0, contratos §6 0. Nada integrado, nenhum push. Smoke do dono do item 1: **OK** (10/11), com o retorno curado em `c22f3377cf`/`ee7156bd10` (ainda sem novo smoke).

## 2. Decisões do dono (10/11)

- O Look da Sprite na forma de colisão; a ferramenta Pivot do canvas sobre a forma.
- Retorno do smoke: Opacity mínima **25 %**; a Color **substitui** a cor da camada (não multiplica); **reset** na Color e na Opacity; sem o «…», o «Pivot from centre» sempre à vista, X e Y empilhados ao lado da grade; «animar o pivot fez a shape balançar — veja como se cura na Sprite».

## 3. O que mudou (lei e onde)

- **Dado:** `ph2d_physics_ecs::ShapeLook { pivot, tint, opacity, cor_propria }` (`shape_look.rs`, lado-forma, catálogo oculto `OtherEditor` com 4 campos ⇒ canais genéricos da Timeline). O pivô é FRAÇÃO da meia-extensão (0 = centro, ±1 = bordas) — por isso a forma cresce em volta dele com Width/Height pelo cartão, pela curva e por tique, sem estado escondido.
- **Porta única do centro:** `forma_resolvida(world, e, col)` = o `Collider` com `offset − pivô ⊙ meia-extensão`. Leem por ela: `bridge/bodies.rs`, `bridge/parts.rs` (solver, `drive_shapes`), `joint_create` (emenda), overlay `outline`/`joint_ghost`/`probes`, `ph2d-shape-handles::Medida::da_forma` (alças), `collider_box_or` (a caixa do gizmo; achado pela foto). A escrita das alças volta por `offset_para_centro`.
- **Cartão e canvas:** `ph2d-shape-handles::look` (`fatos`, `aplica`, `forma_arrastada`, `e_forma`, `anchor_half`, `contem`, `arrasta_pivo`) — mudar o pivô deixa a forma no mundo (a origem anda `R·(S⊙d)`, o Offset dobra-se, os filhos ficam). Cartão `ph2d-panel-inspector/src/cards_forma_look.rs` (na lista do topo, `cards_objeto::do_topo`), `sections/look.rs::paint_cor_da_forma`, grade partilhada `cards_sprite_pivot::grade`. `PhysicsFieldEdit::Look{PivotPoint,PivotX,PivotY,Opacity,Tint,ResetColor,ResetOpacity}`. Shell: 3 chamadas (+7 linhas) no clique/âncora/arrasto da ferramenta Pivot.
- **Desenho:** `tinta_canvas::Tom { cor, alfa }` em `Tinta::Camada` (a cor própria substitui o RGB da camada; a Opacity multiplica o alfa, contorno e sombra).
- **Balanço do pivô animado:** `ph2d_sprite_edit::pivot_chave::e_ancora` reconhece o pivô do `ShapeLook` como a âncora da Sprite (a cura GEF 23 `612022a7bc`: ritmo linear e chave da trajetória em canto).

## 4. O que um merge pode partir

- `Tinta::Camada` ganhou `tom: Tom` (literais fora desta linha não compilam: `tom: TOM_NEUTRO`).
- `InspectorPhysicsInfo` ganhou `look: ShapeLookFacts` (literais completos: `look: ShapeLookFacts::NONE`).
- `cards_sprite_pivot::grade` mudou de assinatura (`nome: Option<(NodeId, bool)>`).
- Schema e registo somam entre linhas: reconte (`python3 scripts/schema-recount.py`). `unidade_mundo::FIXOS` 52 → 54.
- `ph2d-shape-handles` ganhou `libm` e `ph2d-inspector-ordering` (sem crate nova no lock). `physics_apply.rs` 594/600. Catraca `ph2d-app-physics` verde (o Look saiu dela).

## 5. Prova (na linha; suíte completa e `nextest-impacted` NÃO corridos: do integrador)

Testes por nome, todos nascidos vermelhos: `forma_pivot` (3), `shape_look` (3), `physics_apply_look`→`look_tests` (7), `o_cartao_look_da_forma` (7), `o_look_da_forma_tinge_o_contorno_e_o_pivo_o_move`, `a_collider_only_entity_box_is_the_collider_aabb` (caso do pivô), `o_pivo_da_forma_de_colisao_anda_no_ritmo_da_posicao`, `o_piso_da_opacidade_e_um_so`. Mutação: 11 mutações, todas sangraram (uma sobreviveu à 1.ª — a rotação do clique — e o teste ganhou o ponto rente à borda). Censos tocados por nome verdes (registo, lado-forma, catálogo, canais, marcas, LOC, `the_shell_only_shrinks`). **Clippy das crates tocadas NÃO corrido** (fecho).

## 6. Smoke (o retorno, ainda por ver pelo dono)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_PHYSICS_SMOKE=120 PH2D_PHYSICS_SMOKE_SELECT="Loose Shape" cargo run --profile smoke -p ph2d-host-desktop` (`PH2D_PHYSICS_SMOKE_PIVO_NO_CANTO=1` nasce com o pivô no canto). Fotos: [01](fotos_fisica/onda8/01_look_da_forma.png) · [02](fotos_fisica/onda8/02_pivo_no_canto.png) · [03 retorno](fotos_fisica/onda8/03_look_retorno.png). Ver: amostra Color mostra a cor da camada; os dois resets; Opacity não desce de 25; X/Y ao lado da grade; Auto Key com 3 chaves de pivô e posição: a forma não balança.

## 7. ABERTO (onda 8, itens 2–5, por ordem; o dono já decidiu)

2. **Penetração na rampa** (0,6–1,0 cm medidos na onda 6 quando a altura cresce em volta do centro). Medir primeiro: sonda tique a tique (o `caixote` + `EscalaAnimada` de `tests/it/forma_animada.rs`), tabela ao lado. Hipótese a medir (não construída): ao reafinar a forma de um corpo com contactos, corrigir a posição pelo deslocamento do ponto de contacto ao longo da normal antes do passo (exacto no chão rígido, só nos tiques que reafinam). ⛔ Não mexer na mola (`solver_params.rs`, 120 Hz/ζ 5 medidos — a tabela das recusas está lá).
3. **Medir** Width/Height dos canais FISICA 1×/quadro contra por tique; curar só se medir.
4. **Graus** no motor da junta (`JointMotorTarget/Speed`) na Timeline, no referencial do objeto.
5. **Herdados** do [OUTLINE_E_F3](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-09_OUTLINE_E_F3.md) §11.5, cada um com teste nascido vermelho.

O fecho da linha (clippy, handoff de integração) e a integração continuam só por ordem do Enio.
