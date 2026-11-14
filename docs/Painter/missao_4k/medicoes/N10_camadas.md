# N.º 10 — família 5 (camadas e compositor), agente F5 — 14/11

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B18.1 | feito | — | `ecb212fead` (no main) | `LayerImage.rgba8` é `Arc<Vec<u8>>` |
| B18.2 | feito | — | `05f682e924` (no main) | `PaintedDocument` partilha os planos |
| aberto 2a (troca pura com a tinta viva do Wet) | parcial | sonda 64² (papel sim/não, 4 trocas): **0 píxeis** da figura mudam; marcava 64×44 (a janela da tinta inteira) → nada marcado | este commit | adormecer/acordar não escreve píxel; a figura só lê píxeis; quem escreve marca o seu. Chave `PH2D_TINTA_MARCA_A_JANELA=1` = antes. Gate `a_troca_pura_com_a_tinta_viva_nao_marca_nada_e_a_figura_fica` (figura = recomposição inteira, undo/redo; controlo `TINTA_MARCA_SEMPRE` marca e dá a mesma figura); vermelho com a chave ligada (mutação = guarda fora). `a_troca_com_a_tinta_viva_muda_o_relevo_so_onde_a_placa_o_redobra` passa a exigir «nada a drenar» na troca pura. ⏳ O gate da placa `a_troca_com_a_tinta_viva_dobra_na_placa_so_o_relevo_ao_bit` (ph2d-app-painter): telas ao bit, mas o controlo `DOBRAS_PELA_FIGURA ≥ 1` cai (a dobra só da figura já não acontece) — falta exportar `TINTA_MARCA_COMO_ANTES` e correr esse gate com ela a 1 (cobre o item 3) e a 0 (n = 0). App 4K não medido. |
| B18.3/4/6/7/8/10/12 · B19.10/12/14 · B20.6 · aberto 2b (Ctrl+Z na nova) · aberto 3 (Impasto duplicar) | não tocados (prazo) | — | — | — |
