# CONTINUAÇÃO — `line/MiroClone` W23: bloco 4 «velocidade e memória» fechado — 2026-10-20

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_B5_4](HANDOFF_CONTINUACAO_W23_B5_4_2026-10-19.md) ·
> lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md). Base = main `100d34c8e0`.
> Sem integração nem push: só por ordem do Enio. Smoke da cena 27 e a pergunta «Delete board» (fica): resolvidos a 20/10.

## 1. O bloco 4 — medido ANTES, tudo em [16_bloco4_velocidade.md](../16_bloco4_velocidade.md)

- **B6 FEITO.** Abaixo de 11 px no ecrã uma forma em rascunho (preenchimento sólido) desenha-se final com o contorno ×2,5
  (`Detail::sketch_final_below_px`, `SKETCH_FINAL_BELOW_PX = 11`, `SKETCH_FAR_STROKE = 2.5`). 10 mil formas a 8 px: CPU −33 %,
  placa −55 %. ⛔ O candidato da W5 (o final simples) foi RECUSADO pela régua: vê-se (perde o contorno que o traço duplo do
  rough.js dá) — razão 1,5–2,3 contra o controlo de ½ px em todos os tamanhos.
- **B7 FEITO.** `FracKey::prefix()` + `live_in_z_order` ordena `(prefixo u64, índice)` e desempata pelo texto;
  `live_in_paint_order` põe os vídeos por cima por partição. 100 mil de z baralhado: 21,7 → 9,3 ms; na ordem natural igual.
- **B10 e B11 já estavam feitos na W16** (a lista-mãe herdou-os de um handoff anterior): conferido no código e nos números
  gravados — doc 16 §3.

## 2. O que um merge pode partir

- `ph2d-board-render::Detail` ganhou o campo `sketch_final_below_px`: quem constrói um `Detail { .. }` literal sem
  `..Detail::default()` deixa de compilar (feitos: `ph2d-board-export/src/lib.rs`, `tests/it/measure_detail_cost.rs`).
- `ph2d-board-model`: `FracKey::prefix()` (novo, aditivo); `live_in_z_order`/`live_in_paint_order` mesma assinatura e mesma
  ordem (gate com z baralhado, empates, chaves de cauda longa e vídeos).
- Sem formato, `PROTOCOL`, i18n nem cena de smoke novos. Shell não tocada.

## 3. Prova

- Gates por nome verdes: `sketch_tests::a_far_sketched_shape_is_drawn_final_and_a_near_one_by_hand` (+ os 13 `sketch`),
  `frac_tests::the_prefix_orders_like_the_key`, `board_tests::the_paint_order_is_the_key_order_on_a_shuffled_board`
  (+ `ph2d-board-model the_`, 73).
- Mutação (20/10): 9 mutações; 7 sangraram à 1.ª, M2 e M7 sobreviveram e mataram-se reforçando os gates; M9 equivalente —
  tabela no doc 16 §4.
- Régua `measure_bloco4` (`ph2d-board-render/tests/it`), saída em `../ferramentas/bloco4/saida_2026-10-20.txt`; imagens da
  visibilidade conferidas ampliadas 4×.
- `cargo fmt --check` limpo nas 3 crates tocadas. ⚠️ clippy `--all-targets` e as suítes inteiras NÃO corridas (cadência: no
  fecho da linha, `PH2D_SUITE=fecho`).

## 4. ABERTO (nesta ordem)

1. Smoke do dono do bloco 4 (§5).
2. Bloco 5 (B9, A11) · bloco 6 (B12, B13, B14 com `/pd-feature`) ⇒ janela nova.
3. `textAlignVertical` (top/bottom do Miro): por medir, só existe na API — sem UI nem oráculo.
4. Perguntas ao dono sem resposta: YouTube Premium; «Rascunho/Final» e a letra importada.
5. Não medido, nomeado: o caminho de longe para formas com preenchimento às riscas (`fill_style` ≠ sólido) — ficam à mão.
6. O chão de 100 mil que resta é a passagem pelo `BTreeMap` + `collect` (doc 16 §2) — mudar o contentor dos elementos, fora
   do bloco.

## 5. Smoke ao dono (curto)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=24 ./target/smoke/ph2d-host-desktop`
2. O quadro abre com o mapa mental desenhado à mão (Rascunho) e, por baixo dele, duas caixas ligadas por uma seta. Com o
   rato sobre as duas caixas, role a roda para AFASTAR, devagar, até elas ficarem do tamanho de grãos. (O mapa, dentro
   de um frame, some antes — de longe um frame mostra só o título; é o de antes.)
3. Tem de acontecer: as duas caixas continuam com o mesmo contorno claro à volta enquanto encolhem — nenhuma muda de
   aspecto de repente.
4. Deu errado se: num certo ponto do afastar as caixas perdem o contorno, ficam mais escuras ou «piscam» de um desenho para
   outro.
