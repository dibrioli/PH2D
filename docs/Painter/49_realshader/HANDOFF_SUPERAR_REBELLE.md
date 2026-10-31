# HANDOFF — «superar o Rebelle» (linha PainterWatercolor, 30–31/10)

Leitor: próxima LLM / integrador. Tabela viva: [ESTADO_REBELLE_MEDIDO.md](ESTADO_REBELLE_MEDIDO.md). Continuação: [PROMPT_CONTINUACAO_SUPERAR_REBELLE.md](PROMPT_CONTINUACAO_SUPERAR_REBELLE.md). Anterior: [HANDOFF_REBELLE_MEDIDO.md](HANDOFF_REBELLE_MEDIDO.md).
**NÃO integrado no main, sem push.** HEAD `d070dcaf85` (ramo `line/PainterWatercolor`).

## 1. Superfície de colisão (para o integrador)
- Crates: `ph2d-wet-gpu` (nova, água na placa), `ph2d-wet-paint` (advecção/secagem/COW/dryCover/tuftArc/waterDrag, `tuning`), `ph2d-tool-painter` (receitas, painel Wet), `ph2d-app-painter` (ligação da placa, PH2D_WET_GPU).
- Ficheiros novos: `crates/ph2d-wet-paint/src/bristles_arco.rs`, `src/tools/arrasto_agua.rs`, `trail/transfer/cobre.rs`; `assets/brushes/WetPainter/{Water/Clean,Acrylic/Fan,Watercolor/Gouache}`.
- Toca `drying.rs` (R1 edgeSoftMass × 46199a6a7c, já juntado). `PROJECT_SCHEMA`: não conferido nesta fase — o integrador reconta (`python3 scripts/schema-recount.py`).

## 2. Contadores (DELTA, confira por `git grep`)
- `KNOB_COUNT` 140→143 (dryCover, tuftArc, waterDrag). Fábrica 74→77 (contada).
- Painel: Tuning › Bristles 20→21; linhas do Tuning 95→96; `PAINTER_WETPAINT_FIELDS` 35→37; `KNOBS_NO_CARTAO` 17→18; `KNOBS_DA_FERRAMENTA` 5→6; `DEPOSITO_MOLHADO` 20→21→20.
- Memória por ponto 428→177 B/px; passo 4K CPU 128→100,5 ms (rodada calma pendente); placa 4K: 9,24 ms RTX (secagem 1 em 6) / Radeon 80 ms após `1d88fb1b77`. Ver [LIGACAO_PLACA.md](LIGACAO_PLACA.md), PLACA_degrau1..5, [MEDIDA_31_10_RODADA2.md](MEDIDA_31_10_RODADA2.md), [D2_rebelle_passo_agua.md](D2_rebelle_passo_agua.md) (Rebelle 37,3 ms).

## 3. Esta fase (31/10 madrugada)
- **Motor novo** (cada um a 0 = bit-idêntico, portões ao bit verdes): `dryCover` `4cce472f3e` (`cobre_o_assentado`, afasta o assentado de outra cor pela massa; CPU e placa iguais; constante 0,25 NÃO medida em grelha) · `tuftArc` `8317139b72` (crescente no referencial do dab; tufos laterais só com pressão a·s²) · `waterDrag` `560bce5a22` (transporte conservativo do suspenso no sentido do gesto, sem escurecer).
- **Painel**: `68f5c856c1` (Dry cover no cartão Wet Paint; Drag na linha da ferramenta Wet); `48c186b5f9` (dryCover escondido de outra superfície).
- **Pincéis** `d30872a44d`: Water Brush (ferramenta Wet sem pigmento, wetLift 0,7, waterPush 0,35, waterDrag 0,8), Fan Brush (tufos Express Oil, tuftArc 0,8), Gouache Opaque (dryCover 1).
- **Censo** `d070dcaf85`: `cada_ferramenta_da_agua_so_ve_o_que_le` estava vermelho desde `f6fd6e4f88` (Pigment mixing escondido também da Blow; a K-M é da simulação). Nova `LinhaDaFerramenta::MisturaDoPigmento`. O censo só corre em `--profile smoke` (~5–7 min; debug > 900 s).
- **Marker Chisel** `13c264ac48`: glaze 1→0,5 (clareava o papel de cor, BUGS #54). Cura de raiz (caixa das tintas no assenta) descrita, não feita (mexe 25 miniaturas).
- **Fecho de qualidade**: clippy/fmt de `ph2d-wet-gpu`, `ph2d-wet-paint`, `ph2d-tool-painter` (com e sem `--all-features`) verdes após `ac01f7c653`. Mutação nos portões novos: 24/36 sangraram; sobreviventes fechados por testes em `96df9bc5f1` (borda graduada k>0, Gauss-Seidel/Jacobi), `7480217f5e` (advect_jacobi: rotas de commit e faixa exacta), `f05808adb8` (historia_cow: fronteira de tile e fluxo transitório). Nenhum defeito real.
- **Achado**: `flow_x/flow_y` são transitórios (zerados no restaura); o desfazer não rebobina `sim.frame` (cadências n%4) — por desenho.
- **Placa**: paredes entre traços `2fe3400624`, uma espera por passo `0f3fa93d0c`, subir/descer só linhas sujas `c2460bc455`.

## 4. O que um merge pode partir
- Contagens acima (painel, KNOB_COUNT, fábrica) soma-se com outras linhas: recontar, nunca escolher.
- `cada_preset_de_fabrica_e_a_sua_receita`, `the_factory_presets_are_counted`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`: **VERMELHOS, pendentes do coordenador** (miniaturas).

## 5. Miniaturas (pendente do coordenador)
11/22 em `6579e829ee`, 4 em `5400a16388`. Faltam 7: Blender Dots/Hard, Smear Flat/Shattered/Soft/Spikey/Wet (~20–60 min cada, pico 20 GB; hoje `M` na árvore: Blender Dots/Hard). Depois regerar Water Brush, Fan Brush, Gouache Opaque, Marker Chisel (`PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="…" bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test it presets_de_fabrica::portoes::gera_os_presets_de_fabrica -- --ignored --nocapture`, no worktree da linha). O coordenador actualiza depois.

## 6. RTX
Xid 109 CTX SWITCH TIMEOUT às 03:36 de 31/10 («GPU is probably locked»): o Vulkan NVIDIA falha até reboot (acção do dono). Lição: despacho de ecrã inteiro 4K longo pode bater no watchdog — antes de a placa ser omissão, partir o trabalho em despachos mais curtos. Medição in-app da RTX pendente. Placa continua opt-in (`PH2D_WET_GPU=1`).

## 7. ABERTO
1. Rodada calma A/B/C do `nucleos_wash` com o HEAD (ms/passo CPU após todos os cortes; método em MEDIDA_31_10_RODADA2).
2. Célula Glaze 4K (1,0055 > 1 nível; PLACA_degrau5 ronda 4).
3. Juntar as 33 leituras de descida numa espera.
4. Mop e Smudge Finger precisam de motor. Lista não-promovida do R2.
5. Cura de raiz BUGS #54. Constante 0,25 do dryCover por medir em grelha.
6. Miniaturas (§5) e os 3 gates vermelhos. Smoke do dono não feito (missão autónoma).
