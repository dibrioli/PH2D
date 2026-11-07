# Continuação: Physics Body, Static Body, Collision Shape (`line/GameEngineFuncional02`), 07/11/2026

A linha continua: isto fecha a ONDA 4. Ficha: [`../componentes/f1_physics_static_shape.md`](../componentes/f1_physics_static_shape.md). Fotos: [`fotos_fisica/onda4/`](fotos_fisica/onda4/).

## Estado

As quatro ondas da ficha estão feitas; smoke do dono em 07/11 aprovou 12 de 13 passos, e a forma solta passou a aparecer no canvas, tracejada (decisão dele). O `main` não andou durante a onda: sem rebase. Schema continua em 218. HEAD: o último commit `docs(GEF02)` desta onda (`git log -1`).

## O que a onda 4 fez

- **Esc a meio com Auto Key:** forma, Offset, chaves e base voltam, sem passo. `TimelineHistory::desiste` + `AutokeyState::desiste_do_gesto`, chamado de `forma_esc` só com arrasto armado. Medido no app: `forma_esc_chave` k=8 curvas 4 → k=20 curvas 0, passos 0.
- **Clique parado no miolo:** `ph2d_shape_handles::clique_parado` + `App::clique_para_a_selecao`, a mesma porta da câmera (extraída de `camera_frame_up`). O clique no anel ou na imagem do próprio objecto fica com ele, como a câmera. Medido: `forma_clique` sai com a Platform High selecionada e a edição fechada.
- **Cursor:** o app tem cursor por contexto (`update_eyedropper_cursor`). `cursor_em` decide pela direcção da alça em ecrã; `forma_cursor` liga antes do da câmera.
- **Lista da camada:** o slot passou a levar a camada e o popover usa `lista_da_camada`; a linha escolhida fica realçada (foto `fisica_camadas`). Sem teste automático: a prova é o par de fotos onda 2 × onda 4.
- **Prova ponta a ponta:** `docs/GameEngineFuncional/ferramentas/prova_alcas.sh` corre o app real e reprova com exit 1. Não há harness de gesto dentro da `App` em `cargo test`: ela segura uma surface de janela (nota em `bone_undo_probe.rs`). Falta para isso: separar `pointer_button` e `post_frame_undo` da surface, trabalho grande, não feito.
- **Cena `PH2D_PHYSICS_SMOKE=120`** (`CENAS` = 120, em `PAUSED_SCENES`): corpos em `ph2d-physics-scene/src/ficha.rs`, porque `ph2d-app-physics` não tinha folga no tecto; `physics_smoke_ficha.rs` só pede vista, regras e seleção. Moeda em Region (em Still o cartão avisava «Still shows all of it» com uma célula só no canvas).
- **Uso no jogo:** gate `a_plataforma_alargada_por_width_segura_o_player_e_ele_anda_nela` (controlo: sem alargar, cai) e `PH2D_GAME_SMOKE=4` conduzido no app: fotos `jogo4_02`, `jogo4_larga_02` (Player em cima da Platform Low com Width 4).
- **Catraca da shell:** paga por corte de comentário em `physics/physics_smoke.rs` e `bone_undo_probe.rs`. Tecto intacto; folga final de 2 linhas: a próxima costura tem de mover código.
- **Clippy:** apanhou dívida das ondas anteriores (`clone` de `Copy`, `drop` sem efeito, `.into()` inútil, tipo complexo), corrigida.

## Camadas da cena 120

Ground: Player, Props · Player: Ground · Props: Ground, Props, Zones · Zones: Props. Por isso o Player atravessa o Crate e não sente a zona. Isto é da cena, com gate.

## Números medidos

Bola 2 × 1 com `ZONE_FORCE = 40`: oscila entre y ≈ 3 e 7 m, sem nunca chegar ao chão (`probe_cena_120`). Sem a força, chega. Salto do Player: pico y ≈ 1,82, pousa a 1,55 na One-way (topo 1,10 + 0,45).

## Onda 6, «Collides with» (07/11)

Feita, à espera do smoke do dono. Quadros no cartão Shape, «Edit matrix…», curas a–d da onda 5. Detalhe, medidas e provas: fim da [ficha](../componentes/f1_physics_static_shape.md). Fotos: [`fotos_fisica/onda6/`](fotos_fisica/onda6/). Tectos: shell 189 039 de 189 041; `ph2d-app-physics` 55 511 de 55 515. Quem somar linhas ali tem de mover antes.

## Onda 7, cartão «Collision Layers» (07/11)

Feita, à espera do smoke do dono. As camadas saíram do cartão Shape para um cartão próprio, com os mesmos ids e edições. Detalhe e provas: fim da [ficha](../componentes/f1_physics_static_shape.md). Fotos: [`fotos_fisica/onda7/`](fotos_fisica/onda7/). Na largura de fábrica o Crate dobrado resume só `Props`: a fila inteira não cabe ao lado do título.

**Vermelhos que já vinham das ondas 5 e 6, por curar:** `quantas_entradas_tem_cada_painel` (`physics`, 4 contra 3 e 1636 px contra 1236) e `no_magic_numeric` (`ph2d-panel-physics/src/paint/matrix.rs:63`, `paint.rs:41`). O integrador reprovaria: cure-os antes de fechar a linha.

## Onda 8, o modal «Collision Layers» (07/11)

Feita, à espera do smoke do dono. A matriz inteira das 32 camadas saiu do painel para um modal central (`ph2d-panel-physics/src/modal.rs`), com nomes à esquerda e em cima, destaque cruzado, duplo clique para renomear (modal e chip da Layer do Inspector), Enter = um passo, Esc desiste. **Os três vermelhos curados** (carga 4 → 3; altura 1 636 → 1 064, catraca escrita; `no_magic_numeric` verde). Detalhe, provas, mutações e fotos: fim da [ficha](../componentes/f1_physics_static_shape.md). Fotos: [`fotos_fisica/onda8/`](fotos_fisica/onda8/).

**Para o integrador (partilha com a linha irmã `line/GameEngineFuncional`):** o editor de animações dela é uma ABA do centro (`AnimDocs`, `DocumentSurface`, `ph2d-app-sprite-anim/src/editor/`), não um diálogo; o nosso modal é `widget::Modal` + `camada_das_listas::por_cima`. Parecidos: título no cabeçalho, fundo plano escuro, células/miniaturas em grelha com `Radius::Sm`, o «editor próprio» aberto por um botão «Edit …» do cartão do Inspector. Se o dono quiser o MESMO idioma, o caminho é um `DocumentSurface` «Collision Layers» com a pintura de `modal.rs` (a geometria `Geometria` já é independente da moldura). Tocámos fora da crate: `widget/modal.rs` (`sem_rodape`, campo `rodape`), `interaction/dispatch/key.rs` (`esc_fecha`), `i18n/cartoes.rs`, e a isenção `POR_FUNCAO` no gate dos canais.

**Falta da onda 8:** rolagem dentro do modal com rótulos fixos (janela baixa); o campo de edição mais largo que o rótulo; a faixa da linha pairada por cima das células (hoje só o nome acende); o duplo clique no chip `Dropdown` conduzido no app.

## O que falta

1. Smoke do dono: aprovados 1–9 e 11–13. **Passo 10 por repetir:** o achado e a proposta estão na ficha; o rótulo é decisão dele. Smoke das ondas 5, 6 e 7.
2. Curar os vermelhos acima.
3. **Polígono pelo contorno:** o próximo item. Mexe no motor e no formato salvo.

## Não provado

- A ligação do cursor no app: a foto não mostra o cursor; só a lei tem teste.
- As teclas na cena 120 (←/→/↑) não foram conduzidas no app; os gates dão o dedo direto à ponte. No nível 4 o condutor prime as setas reais.
- A cena 120 sob Run ▸ Play: ela não tem câmera de jogo. O smoke usa o Play da timeline (Espaço).
- `suite-completa` e `nextest-impacted`: são do integrador.
- Onda 7: a fila inteira do Crate no título (só gate, com Inspector largo); `Rename…` só por gate, sem foto; arrastar o cartão para reordenar e o alfinete no app (gates).
- Onda 6: o projeto inteiro em disco e o `autokey_pass` com as 528 curvas; a roda real (a tela virtual não recebe rato); o estado pintado da caixinha, só por foto.

## Como retomar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && pwd && git branch --show-current
git cherry main HEAD | wc -l && git log --oneline HEAD..main | wc -l
bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
PH2D_PHYSICS_SMOKE=120 ./target/smoke/ph2d-host-desktop
bash docs/GameEngineFuncional/ferramentas/prova_alcas.sh
```

Testes: `ph2d-panel-inspector o_cartao_das_camadas` · `ph2d-physics-scene ficha::` · `ph2d-shape-handles tests::` · `ph2d-host-desktop esc_a_meio` · `ph2d-host-game-scenes a_plataforma_alargada`. Fecho da linha: `/pd-linha-fechar`.

## Onda 9a — a aba central «Collision Layers» (07/11)

**Commits:** `6bf69c9501` (a aba) · `632fc50a0f` (Inspector: «Open in tab…», «Default») · `55cc2b14e0` (a família mora na `ph2d-panel-physics`; isenção do losango; nomes das fotos pela tabela; teste «Layer 3») · `cada_objeto_conta…` (teste da contagem) · o commit das fotos e docs.

**Como está montada:** `ph2d-panel-physics::aba::familia::AbaDasCamadas` é um `DocumentSurface` de UMA aba (instalado em `shells/desktop/src/init.rs` junto das outras famílias): dá título, ×, `take_focus_request` e a ÁREA (`aba::poe_area`). Quem pinta e regista alvos é o painel Physics (`aba/pinta.rs`, na camada de cima, como o modal da onda 8 fazia) — por isso os ids das células, o despacho de `event.rs`, undo, Auto Key, arrasto e losango não mudaram. Os cliques chegam pelo índice de cliques antes da porta do quadro (a superfície devolve `false`). ⚠️ Limite herdado da onda 8: o conteúdo pinta-se no `paint` do painel Physics (`pinta_com_a_aba_atras`); se o painel não estiver no layout de todo, a aba fica vazia — a 9b pode passar a pintura para a família (precisa do store no `DocumentSurface::paint`). A família ficou na `ph2d-panel-physics` (deps novas `ph2d-host`, `ph2d-text`, `ph2d-vector`) porque a catraca `the_shell_only_shrinks_and_every_app_crate_too` reprovava +106 linhas na `ph2d-app-physics`.

**Para o integrador — onde se parece com o editor de animações da linha irmã** (`line/GameEngineFuncional`, aba «Animations — Anim Strip»): mesmo mecanismo de aba central (`DocumentSurface` + `install`), mesmo tom em cartões (o mapeamento `PanelBg→KanbanList`, `Bg1→KanbanCard`, `Bg2→CardField`, `Bg3→KanbanCardRaised`, `Text*→KanbanCardFg/CardLabel` via `CardInk::of_list`, copiado do `tom` do editor de animações do NOSSO main, não da linha irmã — a macro `tom!` em `aba/pinta.rs`), lista à esquerda com linha selecionada `Bg3` e botões `Bg2` `Radius::Sm`, `TypeToken::Sm`. Nenhum código da linha irmã foi copiado; se ambas as linhas trouxerem um `tom` próprio, um helper partilhado em `ph2d-editor-core` cura a duplicação.

**Provas (por nome, todas verdes no fim):** `ph2d-panel-physics a_aba_das_camadas` (10: porta abre e pede foco, só as visíveis, célula (4,1) com as duas metades CRUAS; Inspector abre na camada 2 com título «Enemy · 3 objects»; lista conta e clique abre; pairar em ECRÃ acende linha [4] e coluna [1] e muda a frase; duplo clique na lista e nas duas bordas com Enter/Esc/vazio e largura do campo; Default/Layer n; «+ Add» → `LayerShow{5}` e a frase; face clássica; arrasto; losango) · seam: `every_matrix_cell_answers_a_real_click`, `a_matriz_pinta_o_vivo`, `arrastar_na_matriz_pinta` · `os_cartoes` · `every_word_this_panel` · Inspector `seam_physics` (43), `o_cartao_das_camadas`, `os_quadros_da_colisao` (o «3» esperado era velho desde a onda 8: «Layer 3»), `every_word` · `quantas_entradas` · editor-core `architecture_panel_wiring_parity`, `architecture_every_editable_field_is_a_channel` (isenção movida para `aba/pinta.rs::paint`), `no_magic_numeric`, `the_shell_only_shrinks`, `workspace_file_loc_cap`, `no_dependency_climbs`, `architecture_panel_loc_cap` · `ph2d-physics-ecs cada_objeto_conta_uma_vez_na_sua_camada`.
**Mutações (as três sangraram):** par trocado na frase do pé → `pairar_acende…` vermelho («Player × Player Shots» ≠ «Player Shots × Player»); linha que não acende sob o ponteiro → vermelho (`[]` ≠ `[4]`); contagem que conta o sensor de quem tem Collider → `cada_objeto_conta…` vermelho (`(3,1,1,1)` ≠ `(2,1,0,1)`).

**Fotos** ([`fotos_fisica/onda9/`](fotos_fisica/onda9/)), `PH2D_PHYSICS_SMOKE=120` + `PH2D_PHYSICS_SMOKE_ABA=abre|pairar|nome|default`: `aba_abre`, `aba_pairar` (Player Shots × Player), `aba_nome` («Enemy Bats» em edição na lista), `aba_default` (camada 0 sem nome = «Default»). `folha_comparacao.png` = a nossa × maquete B estado 1 × referência. **Diferenças que restam:** (1) a grade fica pequena no topo de um cartão alto e vazio (a maquete é compacta; a referência também deixa vazio) — a 9b enche-o com a camada aberta; (2) sem as bolinhas de cor por camada da maquete (não há cor de camada no modelo); (3) sem «Remove» (9b); (4) a frase do pé não põe o par em negrito; (5) a diagonal é contornada a cheio, não tracejada; (6) célula desligada `Bg3` mais escura que a da maquete.

**A 9b tem de fazer:** a camada aberta inteira — frase, «Bumps into» (as caixinhas = a linha da grade, mesmo valor) e «Objects on this layer» com botões que selecionam na cena e «Select all»; «Remove» que pergunta para onde vão os objetos («Move and remove» / «Cancel»; a 0/Default não se apaga); projeto novo só com «Default» na lista (estado 4 da maquete: «Default · 17 objects»); talvez levar a pintura para a família (ver o limite acima).
