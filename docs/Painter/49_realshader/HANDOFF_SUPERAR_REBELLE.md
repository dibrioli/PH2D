# HANDOFF — «superar o Rebelle» (linha PainterWatercolor, 30–31/10)

Leitor: próxima LLM / integrador. Tabela viva: [ESTADO_REBELLE_MEDIDO.md](ESTADO_REBELLE_MEDIDO.md). Continuação: [PROMPT_CONTINUACAO_SUPERAR_REBELLE.md](PROMPT_CONTINUACAO_SUPERAR_REBELLE.md). Anterior: [HANDOFF_REBELLE_MEDIDO.md](HANDOFF_REBELLE_MEDIDO.md).
**NÃO integrado no main, sem push.** HEAD `aed309a84e` (ramo `line/PainterWatercolor`); §3–§7 descrevem `d070dcaf85`, a **§8 é o que veio depois** (`2c10300493..HEAD`, 26 commits) e prevalece onde diverge (§4 miniaturas/gates, §5, §6 RTX, §7 aberto).

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

## 8. Depois de `2c10300493` (31/10 manhã, HEAD `aed309a84e`)
**Máquina.** O PC travou às 07:24 (RTX Xid 109 «GPU is probably locked»; a placa já falhava desde 02:43 com Xid 62); reboot pelo dono. ⛔ Nada de testes na NVIDIA até o passo partido (§8.4) ser medido com o PC livre e `journalctl -k | grep -i xid` vigiado.

**8.1 Miniaturas e gates (fecha §4/§5).** Todas regeradas e comitadas: `026ab2ae87`, `975aa286b6`, `bfe9b3793e`, `10d55f174d`. Fábrica **77**. Verdes: `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`, `cada_preset_de_fabrica_e_a_sua_receita`, `the_factory_presets_are_counted`, `volta_ao_byte`, `cada_ferramenta_tem…`, `aplicar_um_preset_da_agua…` (esta após `e6cd58b2fb`). Regerar rápido com a máquina livre (15 em 43 min); lento sob carga.

**8.2 Defeitos do smoke do dono, curados.** Grumos do óleo sobreposto `611591d423` (espalha) · undo/redo matava a tinta em tela grande `fcc3c46112` (histórico por blocos ligado ao app) · borda branca amarelo/azul nas tintas OPACAS `a1d8b75767` + `e5efa2d206` (`Knob::Opaque`, mistura K-M na célula opaca; Gouache Flat/Opaque/Fan Brush opacos; o marcador fica de fora; sonda `ef1026bdcd`) · óleo derretia o traço ao cruzar `3570226c1e` (ladeira pesa pela fracção assente) · pontos pretos nas fendas `a6796fd570` (piso do ganho do céu) · tinta a mais no Size 64 `4279ba26eb` (sub-passo natural da cápsula) · Pigment mixing escondido da Blow `d070dcaf85` · célula de óleo 16 B `26f87c39b2` · censo `composite_pendente` `fc56456c43` · Water do topo por slot `e6cd58b2fb`.

**8.3 Performance CPU 4K (ms/passo, 16 fios; Rebelle 37,3).** 85,0 (rodada 3, `042137034f`) → 63,5 memo do assentar `aeb67ca906` → 54,9 levanta + memo em linha de cache `8b629bdc21` → 50,3 advecção fundida em blocos de 16 linhas `fb8ff6eec7`/`5b356e4ff4` → 50,58 projecção `9a4480553f` (ao bit, hash 6/42/78). Advecção na banda de memória (transporte 14,3 contra chão 11,6 ms; `9ef95d36a7`). `f126dd0dc7`: smoke com codegen-units=1 no `ph2d-wet-paint`. **Recusados medidos:** planos estreitos f16/u16 (régua invisível, `b1d456636a`, `tests/it/regua_estreita.rs`), stores não-temporais (unsafe proibido), inline do `flow_at_point`, fundida abaixo de 4 M células. Docs: MEDIDA_31_10_RODADA3..10.md.

**8.4 Placa.** `aed309a84e`: passo partido em submissões ≤ 40 000 grupos — maior submissão 93,3 → 22,6 ms na Radeon (~2,8 ms estimado na RTX), guarda `TETO_MS` (25 ms) devolve a água à CPU, descida suja numa espera (33 → 1). Ver [PLACA_degrau6.md](PLACA_degrau6.md). A placa continua opt-in e é a ÚNICA alavanca que passa o Rebelle (9,24 ms isolado).

**8.5 ABERTO (substitui §7).**
1. Medir na RTX com o PC livre (Xid vigiado); partir o `outflow_pass` por faixas ou despachar só a área da tinta; tirar a 2.ª espera da secagem; depois decidir a placa por omissão.
2. Smoke: o Thick Impasto a raio 24–32 (Size 48–64) ainda encaroça menos estriado que a raio 50 (sonda `sonda_oleo_estria.rs`, régua cn8). O dono ainda NÃO testou o build novo: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke`.
3. Vermelhos: `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` no toggle Real Oil (de `776a9ff173`, decisão da família do óleo); `os_fios_do_oil_bristle_aparecem_e_somem` (14,3<15, anterior); 3 de óleo pré-existentes (`o_traco_seguinte…`, `a_aguada_re_molha…`, Blender×Cycles). A fenda forte está verde.
4. Do §7 seguem: célula Glaze 4K (1,0055 > 1 nível), Mop e Smudge Finger (motor), BUGS #54 de raiz, constante 0,25 do dryCover por medir. (Itens 1 e 3 do §7 cumpridos: rodadas 3–10 e descida numa espera.)
