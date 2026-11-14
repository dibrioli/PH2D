# N10 — família 4: a água do Wet e o Óleo Real (agente F4, 14/11)

| id | estado | antes → depois | sha | porquê / próximo passo |
|---|---|---|---|---|
| B13.8 | feito | 2.ª passagem com 100 leis de célula (vela por célula, 3 cores): red no antigo (as leis além da 64.ª nunca acertam: ≤ 192 de 300; o número exacto do antigo não foi impresso) → 296 de 300 | cf381d4854 | a lista de 64 leis por fio ficava cheia para sempre (65.ª lei ⇒ memo desligado no fio até um `memo_esquece`). Agora tabela de leis associativa (1 024 lugares, 4 vias), ids nunca repetidos (entrada de lei expulsa nunca casa), a geração do `memo_esquece` recomeça-os. Ao bit por construção (o teste compara com `assenta` a cada chamada). Gate `o_memo_acerta_com_mais_de_64_leis` (os 4 restantes são colisões da tabela de contas, 300 chaves em 32 Ki). `assenta` 30 verdes, `drying` 11. Velocidade da secagem NÃO medida (procura da lei: era linear até 64 × 48 B, agora ≤ 4 vias): `sonda_secagem_4k` A×B por fazer. load 8 |
| B13.11 | parcial | — | este commit | feito: `solver::regua::Regua::poe(&STATIC, v)` repõe o valor no `Drop` (também no pânico); gate `a_regua_repoe_o_valor_mesmo_quando_o_teste_reprova` (red: o padrão `store`…`store(0)` vaza a rota 2 depois do pânico). Falta: trocar os pares `store(x)`/`store(0)` pela régua em `crates/ph2d-tool-painter/src/tool/paint/wetpaint/{sobe_a_ladeira_porta,queda_na_ladeira,remolha,escorre_contorna,chao_forca,agua_no_oleo}_tests.rs` e `wetpaint_preset/{sonda_relevo_oleo,sonda_oleo_furos,sonda_oleo_derrete}.rs` (também os `CHAO_*`), e correr esses testes por nome |
| B11.2 | não tocado | — | — | resta o véu `veu_da_caixa` por linhas inteiras (B11.A2) |
| B11.3 | não tocado | — | — | resta o laço com Smooth Edges/razão > 1 e a medição na Radeon a load ≤ 5 |
| B13.4 | não tocado | — | — | muda aparência: prancha 3 fundos antes de curar (cadência por tile, `sim.rs:579` / `drying.rs:122`) |
| B13.5 · B13.12 | não tocados | — | — | o passo (B13.6); 0,91 sim-s/s a 6 traços, o resto pede a `assenta` ou AVX2 |
| B13.7 · B13.9 · B13.10 | não tocados | — | — | B13.10 muda aparência (raro): prancha |
| B14.1 | não tocado | — | — | custo por faixa (`sonda_o_campo_custa_pela_janela` 2 354 × 66) e listas de faixas no `servido`/dívida |
