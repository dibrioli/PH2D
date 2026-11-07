# Continuação: Physics Body, Static Body, Collision Shape (`line/GameEngineFuncional02`), 07/11/2026

A linha continua: isto fecha a ONDA 4. Ficha: [`../componentes/f1_physics_static_shape.md`](../componentes/f1_physics_static_shape.md). Fotos: [`fotos_fisica/onda4/`](fotos_fisica/onda4/).

## Estado

As quatro ondas da ficha estão feitas e **à espera do smoke do dono**. O `main` não andou durante a onda: sem rebase. Schema continua em 218. HEAD: o último commit `docs(GEF02)` desta onda (`git log -1`).

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

## O que falta

1. Smoke do dono (rascunho no relatório desta onda).
2. **Polígono pelo contorno:** item próprio, a seguir. Mexe no motor e no formato salvo.
3. Decisão de produto: a forma solta não tem contorno no canvas (só o anel; com a edição acesa, sim). O «Hits» dela já saiu: ver a ficha.

## Não provado

- A ligação do cursor no app: a foto não mostra o cursor; só a lei tem teste.
- As teclas na cena 120 (←/→/↑) não foram conduzidas no app; os gates dão o dedo direto à ponte. No nível 4 o condutor prime as setas reais.
- A cena 120 sob Run ▸ Play: ela não tem câmera de jogo. O smoke usa o Play da timeline (Espaço).
- `suite-completa` e `nextest-impacted`: são do integrador.

## Como retomar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && pwd && git branch --show-current
git cherry main HEAD | wc -l && git log --oneline HEAD..main | wc -l
bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
PH2D_PHYSICS_SMOKE=120 ./target/smoke/ph2d-host-desktop
bash docs/GameEngineFuncional/ferramentas/prova_alcas.sh
```

Testes: `ph2d-physics-scene ficha::` · `ph2d-shape-handles tests::` · `ph2d-host-desktop esc_a_meio` · `ph2d-host-game-scenes a_plataforma_alargada`. Fecho da linha: `/pd-linha-fechar`.
