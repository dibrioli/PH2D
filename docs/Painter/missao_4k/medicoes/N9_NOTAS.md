# N.º 9 — notas de medição (13/11)

## Item 2 — o custo da cura do vidro do escorrido (`69158c3890`)

Sonda `watercolor_vidro_custo_sonda` (ignorada, `--profile smoke`): 4096², papel de cor, 4 aguadas, Wet por cima com
30 ticks de água entre traços; o pen-down seguinte e o Ctrl+Z. Três telas no MESMO processo, rondas rodadas, o
MÍNIMO de 6. ⚠️ load 10–22 em todas as corridas (outro agente a compilar): leia DIFERENÇAS dentro da ronda.

| rota (ms, mínimo) | pen-down | Ctrl+Z |
|---|---|---|
| `antes` — o vidro elidido não testemunha (a resposta ERRADA de antes de `69158c3890`) | 7,5 | 35,7 |
| `segurada` — a cura de `69158c3890`: o cursor segura o vidro do reset | 45,9 · 53,0 · 59,2 · 64,2 | 71,3 · 80,4 · 89,9 · 93,1 |
| **`composta`** — o delta do topo composto com o journal, nas caixas dos dois | **14,2** (1.ª versão 25,2) | **35,6** (1.ª versão 55,0) |

Partes da `segurada` (uma ronda): `vidro_no_reset` (clone 58 MB + rewind) ~20 ms · detector ~28 · materializar
~10 · re-split ~28–31. A `composta` tira as três varreduras do vidro e, com elas, duas da TELA:

1. **O vidro** compõe-se em `StoredPlane::composto_com_o_journal` (`undo_delta_composto.rs`): as caixas da grade de
   128 que o topo ou o journal tocam; `before` = topo onde ele fala, senão o journal, senão o vivo. Ali mesmo sai se
   o escorrido mudou algum byte (o `from_journal` separado custava 13 ms a mais).
2. **A tela**: o `absorbed` barato recusava o topo `Tiles` (B17.5) — a tela inteira era copiada (3,5 ms) e re-partida
   (7,6 ms). Agora aceita `Tiles` (o mesmo `Patch` exacto; o gate campo-a-campo `absorb` verde). Com a tela que o
   escorrido não tocou, o delta dela é o do topo tal qual.
3. **A montagem** dos dois lados é paralela por caixa.

O que fica (~7 ms no pen-down acima da rota errada) é o tamanho do dado: nesta cena o escorrido escreve o vidro em
352–418 das 1024 caixas e 210–254 mudam — o passo guarda ~57 MB, como a segurada já guardava.

Gate: `o_vidro_composto_da_o_documento_do_vidro_segurado` (composta × segurada ao bit em cada traço, desfazer e
refazer; controlos: as duas dispararam). Mutação: o `before` do journal trocado pelo vivo ⇒ vermelho.

⏳ Re-medir em calma (load < 5) junto com o item 1.

### Vermelhos do filtro `wet` (13/11, depois de `8a1714c8d0`) — nenhum é desta mudança

Vermelhos SOZINHOS e também na base `88ac5f8653` (restaurada por `git restore --source`): `a_primeira_marca_do_wet_refaz_a_imagem_inteira`,
`com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte`, `a_secao_desligada_e_o_traco_de_hoje_ao_byte`,
`cada_preset_de_fabrica_mapeia_por_funcao`, `cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao`,
`o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde` (+ `receita_e4`, já no prompt n.º 9). Verdes sozinhos, vermelhos
só em lote (o flake de isolamento da contagem de leituras): `a_silhueta_alem_da_pegada_nao_e_avaliada…`,
`o_motor_nao_le_a_ponta_alem_da_pegada…`. Os filtros `undo` (187), `vidro` (34) e `absorb` (17): verdes.
