# CONTINUAÇÃO — `line/MiroClone` W23: bloco 5 «fidelidade» fechado (B9, A11) — 2026-10-20

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_BLOCO4](HANDOFF_CONTINUACAO_W23_BLOCO4_2026-10-20.md) (smoke OK
> 20/10) · lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md). Base = main `100d34c8e0`.
> Sem integração nem push: só por ordem do Enio.

## 1. Feito

- **A11** (`b3897f9330`) — o ciclo da escolha do «Send to board…» (alvos, apagado → «Deleted object», sem nome → nome do
  tipo) passou da fase da shell para `ph2d_app_board::send_to_board::gather(world, clicked, selected, kind, read)`; a shell
  só lê o conteúdo por tipo. Gate de comportamento sobre um mundo ECS real:
  `the_choice_reads_the_scene_counts_the_deleted_and_names_the_nameless` (mutações «sem o apagado» e «sem a selecção
  inteira» sangram). O gate da costura (`shells/desktop/tests/it/o_send_to_board_leva_a_seleccao_com_a_pose.rs`) deixou de
  procurar a linha e prende só que a fase chama `gather` com o mundo da cena e não repete a decisão. `ph2d-app-board` ganhou
  a dependência `ph2d-ecs` (fundação; nenhuma família depende de outra). Shell encolheu.
- **B9** (`5fa249933b`) — o 0.18.1 do Excalidraw re-liga as nossas setas no losango (vivo e redondo) e no rectângulo
  redondo pela LEI DAS PEÇAS, medida no oráculo (110 casos, 28 188 setas, ≤ 0,0022 px, determinista): vértice do losango
  em ⌊w/2⌋+1, cantos cúbicos (losango) / quadráticos (rectângulo) com o raio `R()` do tipo 3, cada canto transladado
  (g·V/|V| no losango; g·(hw+g, hh+g)/‖·‖ no rectângulo), ≥ 2 pontos → o mais perto, 1 → o ponto do foco, 0 → fica.
  `legacy_focus.rs` escreve o `gap` inverso (bissecção). O nosso ficheiro no 0.18.1: losango 3,875 → 0,0000 px, rectângulo
  redondo 3,513 → ≤ 0,0009 (≤ 0,12 com as duas pontas presas — efeito do teste que empurra tudo), ponto fixo dentro do lado
  de cá 4,70 → 0,0000; do lado de lá continua sem se exprimir (rect. redondo 85,6 → 12,6). O next (o do dono) não muda.
  Gates `w23_the_pieces_law_reproduces_the_0181_sweep`, `the_written_pair_lands_on_our_tip_by_the_pieces_law`,
  `an_inside_point_is_reached_where_the_law_can_and_nearest_where_not`, `xv7_*` apertados (≤ 4 → ≤ 0,12). Mutação 10/10.
  Detalhe: [07_fidelidade_excalidraw.md](../07_fidelidade_excalidraw.md) XV7 e a onda W23 B9; oráculo README §W23.

## 2. O que um merge pode partir

- `ph2d-app-board/Cargo.toml` + `Cargo.lock` (dep `ph2d-ecs`); `send_to_board::gather` novo (aditivo).
- `ph2d-board-excalidraw`: `roundness_type` partilhado entre `write.rs` e `legacy_focus.rs`; os ficheiros do oráculo W8/W9/W10
  regenerados (o escritor mudou) — `w8_sondas.json` voltou ao da W8 como o README manda; deriva de 0,003 px num rótulo da W8
  vinda de `fe0025ed8d` (curva do texto), não deste bloco.
- Sem formato, `PROTOCOL`, i18n nem cena nova.

## 3. Prova

Gates por nome verdes (acima + `ph2d-board-excalidraw` inteira 48 · 4 ignorados, corrida pelo agente; a catraca da shell;
`o_send_to_board` 3). ⚠️ clippy `--all-targets` e as suítes inteiras NÃO corridas (fecho da linha, `PH2D_SUITE=fecho`).

## 4. ABERTO (nesta ordem)

1. ~~Smoke do dono do bloco 5 (§5)~~ ✅ OK 20/10.
2. Bloco 6 (B12 laço, B13 desenho inteligente, B14 menções/notificações/voto anónimo, **B15 de um quadro para outro — pedido do dono 20/10**; com `/pd-feature`) ⇒ **janela nova**.
3. `textAlignVertical` (top/bottom do Miro): por medir, só existe na API.
4. Perguntas ao dono sem resposta: YouTube Premium; «Rascunho/Final» e a letra importada.
5. Nomeados, não feitos: rectângulo vivo e elipse não ganharam a busca do «menor erro» para o ponto fixo do lado de lá
   (70,12 / 50,9 px no 0.18.1; não se exprime — doc 07 §3); o caminho de longe do rascunho para preenchimento às riscas (doc 16).

## 5. Smoke ao dono (curto)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && ./target/smoke/ph2d-host-desktop` (recompile antes:
   `cargo build --profile smoke -p ph2d-host-desktop`).
2. Na lista **Hierarchy**, seleccione duas linhas (uma imagem e um vector; Ctrl+clique), botão direito numa DELAS (na
   lista — a cena não tem menu de botão direito para objectos) → «Send to board…» → escolha um quadro.
3. Tem de acontecer: os dois aparecem no quadro, no meio da vista, seleccionados.
4. Deu errado se só um aparecer, se aparecer um aviso sem nome («Left out: , …») ou se nada acontecer.
