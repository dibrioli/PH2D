# Handoff de continuação — GEF onda 19 (ordem do desenho e do clique), 01/11

Linha `line/GameEngineFuncional`, worktree `Worktrees/line-GameEngineFuncional`, sobre o main `422fec2027`
(o main não andou até 01/11). Plano e decisões: [19_ordem_e_clique_plano.md](../19_ordem_e_clique_plano.md);
decisões do dono em `project-memory/project_hierarchy_order_rules_drawing_and_click_2026_11_01.md`.
A onda 18c (miniatura) foi aprovada no smoke de 31/10; o dono mandou NÃO fechar a linha ainda.

## Feito (não reconstruir)

| etapa | commits | o quê |
|---|---|---|
| E1 | `7d703c2d79` | `FrameOrder::enrol`/`stack_key`; `pick_objects_at` ordena pela pilha do desenho; lâmpada/câmera à frente SEM retorno cedo; `is_locked_for_edit` em toda família; `pick_order::cycle_step` (1 clique/degrau, `pending_single_replace` no Up) |
| E2 | `c2d460075d` | `Family::Pixel`; arte do Pixel na cena da faixa, overlays do Draw no chrome; `DRAWN` só do passe do editor |
| E3 | `06494dff4e`, `cc02d5e412` | `Family::Motion` + rank do `MotionGenerator`; placa colada POR FAIXA (`motion_shape_placa_faixa.rs`, `Colagem::cola_a_faixa`, `cola_o_topo`); desenho da placa antes de `draw_lower_bands`; `motion_na_placa` (sem placa = rota Vello, por cima, de propósito) |
| E5 | `b1c99a7bbf`, `1db4cf3d50` | gate `a_seccao_ordering_existe_em_todo_tipo_do_add` (20 tipos, todos mostram o Z Index; nada a curar); fotos `fotos_19/inspector_{skeleton,flip}.png` |
| E6 | `4bf8ac8efe` | `ph2d_app_macros::icone_no_canvas` (glifo `IconId::Macro`, «Icon Size»), no lugar do anel; à frente no clique; smoke `PH2D_OBJECT_ADD_PICK=<chave>` |
| E7b | `26c6e5c2b3` | repetição de camada escalada: tile local × \|escala × dolly\| |

## ABERTO, por ordem

1. **E4 — Flip na ordem** (`Family::Flip`): `present_flip_fx` (`present.rs:~378`) por subconjunto de objectos
   (`pass::render` recebe `models`), faixa no padrão dos sprites (HDR → tonemap → acumulador), fantasmas/peek só em
   autoria, miniatura incluída. Mesmo molde das E2/E3 (`git show c2d460075d cc02d5e412`).
2. **Restos da E3:** (a) palcos das macros ainda por cima — o dono do palco é a entidade Macro
   (`PalcoEmCena.em_cena.dono()`), falta dar-lhe rank e uma `Colagem` por palco; (b) camada filtrada + lente quando
   o topo Motion cai no passe de baixo correm logo depois da última faixa de sprites — cura: mover
   `motion.sprites_da_camada` (`present.rs ~240`) para depois de `draw_lower_bands`; (c) imagens de grupo (sem
   `fonte`) vão sempre na faixa Motion de cima.
3. **O clique nas formas do Motion:** `hover_highlight.rs` não testa objectos Motion — clicar numa forma do Motion
   pega o que está por baixo. Medir e, se for assim, entrar na lista do clique pelo `stack_key` (que já os conhece).
4. **E7a** — o que o editor mostra da paralaxe fora do Play: o código conduz pela câmera do jogo em todo quadro
   (`fase_paralaxe.rs:41`), os handoffs 11/12 dizem «pose autorada»: FOTO (`PH2D_GAME_SMOKE=2`) e decidir.
   **E7c** — gate dos limites do nível 1 (`smoke.rs:285-325`; limites = paredes, medido no código).
5. **Trava × grupo:** com a E1, um filho de `GroupedChildren` deixou de se pegar em TODA família (o vetor e o Flip
   já o recusavam). Ninguém redirecciona o clique para o grupo — avaliar se o clique num filho agrupado deve pegar o
   grupo.
6. **Prova no app real antes do smoke:** fotos do clique (o `smoke_mao` conduz gestos sem rato:
   `crates/ph2d-app-game/src/smoke_mao.rs`), do Pixel/Motion sobrepostos a uma imagem nas duas ordens da Hierarchy,
   do ícone da Macro. Provas de mutação em lote (`mutacao`): o `sort_by_key` do pick, `cycle_step`, `stack_key`,
   `cola_a_faixa`, o arm Motion do passe de baixo.
7. **Pergunta ao dono:** no modo Draw do Flip a coluna da direita abre na aba Flip, não no Inspector (o Z Index fica
   a um clique). A decisão de 24/10 fala de seleccionar e de sair de um modo, não de entrar — perguntar.
8. **Smoke do dono** (passos numerados, cena com FORMAS do Motion + imagem + Pixel + Flip + câmera + macro, nas duas
   ordens) → depois, só por ordem dele, fecho da linha (§1.5.9): gate batched sobre `422fec2027..HEAD`, fmt+clippy
   das crates tocadas com `PH2D_SUITE=fecho`, **ph2d-app-macros inteira** (deps novas: `motivo: deps`),
   `agent-loop-profile.sh`, `rm -rf target/*/incremental`, handoff de INTEGRAÇÃO das ondas 18, 18b, 18c e 19.

## Riscos medidos/não medidos

- Testes de placa gráfica (verificador, 01/11, `PH2D_GPU=1 … --profile smoke -- --ignored`, load 8–18): verdes
  `sondas_p4_tests` (3), `gpu_tests::intercalada::sonda_intercalada`, `pousa_gpu_tests` 3 de 5, e os 4 gates da
  shell da placa/palcos/miniatura. **Vermelhos determinísticos** (isolados também):
  `o_scene_com_imagens_pela_cpu_guarda_a_ordem_das_linhas` (`motion_shape_placa_pousa_gpu_tests.rs:459`) e
  `o_scene_com_imagens_pelo_dispositivo_segue_a_faixa_deslocada` (`:529`) — `TextureAtlas::insert` devolve
  `AtlasFull { 4, 4 }` num atlas de 64; o MESMO pânico em
  `motion_bridge_gpu_mistura_imagens_tests.rs:66`, fora do que esta onda tocou. Suspeita: a subida do stack de 30/10
  (`0341c1704d`, wgpu 30). ⇒ medir no main com o MESMO comando antes de culpar a onda (item 0 abaixo).

## Item 0 (antes do E4)

Correr os três testes do `AtlasFull` no main (worktree limpo do main, mesmo comando). Vermelhos no main ⇒ herdado
da subida do stack: registar no BUGS do Motion e seguir. Verdes no main ⇒ é desta linha: bissectar
`422fec2027..HEAD`.
- Uma faixa Motion partida repinta a camada uma vez por segmento por faixa; com uma faixa só, os mesmos bytes.
- O Pixel nas faixas recebe a Luz 2D (antes, no chrome, não recebia).
