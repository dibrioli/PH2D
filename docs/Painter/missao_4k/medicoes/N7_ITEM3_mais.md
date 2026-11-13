# N.º 7 item 3 — o «+» a 4K sem diff (autorizado pelo dono 13/11: «sim»)

## Antes (handoff n.º 6 §1, não re-medido aqui)

«+» a 4K: 55–65 ms = `add_raster_layer` 51 ms. Dentro dele, `commit_structural_edit` leva 15–30 ms, dos quais
`UndoEntry::split` leva 14 ms a comparar a tela velha com a nova zerada. Sobre a base opaca a entrada saía
`Whole{A, Z}`: segurava a tela nova `Z`, e por isso o 1.º traço na camada nova copiava a tela inteira (33 MB).

## O que mudou

- `StoredPlane::Nova { before }` (`undo_delta.rs`): guarda a tela que saiu (`A`) inteira, sem diff. O lado
  `after` é todo neutro e não se segura. O refazer recria-o (`vec![0; len]`, só no refazer do «+»).
  `reach` = `Whole` (repinta tudo, como antes), `copiado_bytes` = 0, `heap_bytes` = `len`.
- `UndoController::tela_nova_neutra` é um sinal de uso único. Acende-o `add_raster_layer_com` quando a camada
  nasce transparente (`rgba = None`); `add_raster_layer_with_pixels` não o acende. `record_structural_hinted`
  apaga-o.
- `PlaneDeltas::tira_a_tela_nova` age antes do `split`: dá aos dois endpoints o MESMO plano vazio (o `split`
  lê `Unchanged` sem varrer). Depois do `split`, `poe_a_tela_nova` grava o `Nova`. Recusa (e cai na rota
  antiga) com tela elidida, comprimentos diferentes ou o mesmo `Arc`. Em debug confirma que a tela nova é toda
  zero.
- A chave `MAIS_SEM_DIFF` (`undo_planes.rs`, por fio, ligada por omissão): desligada, volta a rota antiga. A
  sonda ganhou `PH2D_SONDA_MAIS=ab`.

## A contabilidade: quantos passos de desfazer se perdem a 4K

Plano RGBA 4K = 3840×2160×4 = **33,18 MB**. A entrada do «+» pesa agora `Nova` (1 plano) + `OnlyAfter` da
imagem da camada que saiu (1 plano) = **66,4 MB**, seja qual for a camada de baixo.

| «+» sobre… | rota antiga | nova | diferença |
|---|---|---|---|
| camada opaca (a base da imagem aberta) | `Whole` × 2 + imagem = 99,5 MB | 66,4 MB | **−33,2 MB** (lembra MAIS) |
| camada rala (poucos traços) | `Patch` da janela × 2 + imagem ≈ 33,2 MB + 2·janela | 66,4 MB | até **+33,2 MB** |

No pior caso (um «+» sobre camada rala) perde-se **≈ 1,5 traço** (22,45 MB por traço de lado a lado a 4K) por cada
«+» desses que esteja no histórico, tanto no piso de 509 MiB (23 traços) como na máquina do dono (3 943 MiB,
176 traços). O 1.º «+» sobre a base opaca ganha o mesmo tanto. O tecto conta o `before` pelo tamanho inteiro,
embora seja o mesmo `Arc` da imagem viva (conservador, como o `Whole`).

## Gates (`tool/paint/mais_sem_diff_tests.rs`, 520×390)

- `o_mais_sem_diff_da_o_bit_no_desfazer_e_no_refazer_wb`: base opaca, 2 traços, «+», 2 traços, «+» sobre a
  rala, 1 traço. Desfaz tudo e refaz tudo. A cada passo compara ao bit com a rota antiga (a chave desligada)
  no mesmo processo, e com os estados gravados. Assere o «+» = 2 planos (antigo 3 sobre opaca, menos sobre a
  rala). No pen-down do 1.º traço da camada nova, ninguém segura a tela além da viva e do cursor: chave = 0
  donos, controlo antigo = 1 (a entrada).
- `…_wet`: o mesmo no Wet Paint. Duas ferramentas Wet NÃO dão o mesmo bit entre si: medido com a chave igual
  nos dois lados, diverge no «+» (o motor corre fora do fio). Por isso cada rota confere-se contra o PRÓPRIO
  histórico gravado; os donos e os bytes asserem-se como no WB.
- `o_tecto_conta_o_mais_pelo_tamanho_real`: grava o peso de cada passo nas duas rotas e põe um tecto que cabe
  tudo menos o 1.º «+» antigo. A profundidade retida é igual à simulação do `cap` nas duas rotas (antiga 8,
  nova 6). Controlo: a nova < a antiga.
- Mutações: (M1) o refazer do `Nova` devolve a tela velha → 2 vermelhos; (M2) `tira_a_tela_nova` sempre `None`
  → 3 vermelhos.
- Vizinhos por nome, verdes: `camadas_integridade`, `duplicate`, `o_undo_e_o_redo_dao_o_bit`,
  `watercolor_vidro_journal_tests` (inclui `a_camada_nova_e_o_vidro_elidido_dao_o_bit` e
  `quem_segura_o_vidro_no_segundo_pen_down`), `every_plane_of_a_snapshot`, mais os filtros `undo`, `layers`,
  `camada`. `cargo check --profile smoke` compila.

## Próximo passo

1. O coordenador corre `N7_ITEM3_sonda.sh` pela fila pesada (A×B intercalado, WB e Wet Wash, 4K). Esperado:
   o «+» perde os ~14 ms do split, e o 1.º traço da camada nova perde a cópia da tela (≈ 33 MB · ~6 ms).
2. O resto do «+» fica por curar: `flush_active_to_images` 6,5 ms e `a_tinta_que_adormeceu_muda` 7,5 ms.
3. O Wet não é bit-determinístico entre duas ferramentas no «+»: isto existia antes desta mudança. Fica
   registado para quem fizer um oráculo cruzado do Wet.
4. ⚠️ Vizinho vermelho que NÃO é deste item: `undo_confine_tests::o_ctrl_z_de_um_traco_no_papel_e_confinado_em_cada_meio`
   (sem «+»). O controlo dele espera `v=WHOLE` no 1.º traço do Watercolor; com o `Aparece` do ADR-0204 (item 2) o
   vidro que aparece guarda só a janela (`v=win`). Ou seja, o controlo ficou velho com o item 2: a cura é atualizar o
   controlo para `v=win`. Fica para o item 2 / coordenador.
