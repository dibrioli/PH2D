# Handoff de integração — line/PainterWatercolor, a seleção, as camadas e o ritmo (12/11, n.º 4)

Leitor: o agente integrador. Os handoffs [MISTURA_E_ENVIO](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_MISTURA_E_ENVIO.md),
[LUZ_E_CAMADAS](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_LUZ_E_CAMADAS.md) e
[DESEMPENHO_TOTAL](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md) **CONTINUAM a valer** (superfície, riscos,
decisões do dono, abertos). Aqui está só o que esta janela acrescentou: `4cc0e272fb..HEAD` (22 commits; a §12 acrescenta `63f5e2ca23..116f9c8072`, +19 commits, HEAD `116f9c8072`). **Nada foi integrado nem enviado.**

## 1. Identidade

- Branch `line/PainterWatercolor`; merge-base com o main `bc4a6ce930`. A linha **NÃO foi rebaseada**.
- Tocado: `ph2d-tool-painter`, `ph2d-render` e `ph2d-vector` (só a §12), `ph2d-app-painter` (4 ficheiros, mais `painter_gpu_preview.rs`, `lib.rs` e testes na §12: `painter_bridge_selection_overlay.rs`, `painter_bridge_queries.rs` e dois
  testes), `ph2d-wet-paint` (`solver/`) e `docs/`. **Ficaram intocados:** `shells/desktop`, os `Cargo.toml`/`Cargo.lock` e os contratos do §6
  (conferido pelo `git diff --stat`, também em `63f5e2ca23..116f9c8072`).
- Commits (cada mensagem traz o mecanismo e a tabela de medição):
  - B20.11 `d15a917f44` · B01.A1 `7bfcae90f6` · B52.R13 `5ba1c8aa66` · B40.A5 `56bba7ce79` · B40.A4 `9f32dc19f8` · B40.A3 `26a9cc8a68`
    (docs `513f086915`);
  - B13.6 `61330c6e8d` · B22.1 `5ead7e7ad0`, `19764e2d78`, `986aab640d` · B19.10 `740d798d56`, `8587dd5f26`, `56262508d7`
    (docs `3d07604e81`, `112cbf6797`);
  - gates herdados `8a00073ea0`, `a6258f908f`, `24a1371f68`; fmt/clippy `4482855510`, `0eb6ccc203`; matriz `65b12e2595`.

## 2. Superfície de colisão

- **O compositor é de todo o Painter:** `compositor/compose.rs`, `barreira.rs`, novos `resumo.rs` e `cache.rs` (tiras de cache, salto α=0 nos 21
  modos, resumo por ladrilho de 64 px). Linha que mexa na composição **colide**.
- **`LayerPixelSource::layer_plane`** novo no trait (`compositor/mod.rs`; padrão `None`). Implementar o trait noutro sítio = herdar o padrão.
- **`PaintState` ganhou campos** (`state.rs`/`state_default.rs`): `traco_relogio`, `movido_desde_o_passo`, `canvas_wet_relogio`; mais o ritmo
  em `tool/paint/ritmo.rs` (passos fixos de 1/60 s, até 8 por quadro). O que o tique faz por quadro mudou.
- **`mask.rs` partido:** a porta da proteção vive em `mask_gate.rs` (só mover, B52.R13). Quem tocar `mask.rs` rebaseia à mão.
- Seleção: `selection.rs`, `selection_formigas.rs` (novo), `selection_offset.rs`, `selection_raster.rs`, `selection_shapes.rs`,
  `stamp_route/mascara.rs`; `layers/estrutura.rs` e `layers/mod.rs`.
- `ph2d-wet-paint`: `solver/advect_saida.rs` (novo), `advect_fundida.rs`, `advect_jacobi.rs`.
- Chave nova de A×B (só sonda/gate): `PH2D_RITMO_FIXO`.

## 3. Números que se CONTAM (DELTA)

`PROJECT_SCHEMA` **+0** · ADR **+0** · registos 0 · pacotes 0 · nenhum formato gravado mudou (as curas são de relógio, composição e cópia; os
campos novos do `PaintState` não são serializados).

## 4. O que entra (régua = sonda no mesmo binário, smoke, mínimos)

| item | antes | depois |
|---|---|---|
| B20.11 formigas da Seleção, 4K | refeitas e 33 MB à placa por QUADRO (2,4 ms CPU) | 10 reconstruções/s (passo 100 ms), 0 entre passos; a Seleção parada deixa de segurar o laço a 60 q/s |
| B01.A1 `apply_mask` 4096² | 16,56 ms | 3,89 |
| B40.A3 arrasto de forma da seleção 4096² | 27,2 ms | 5,2 (tique do Offset 1,38 → 0,78) |
| B40.A4 proteção, pico do journal por traço | 33 177 600 B | 131 072 (relógio já baixo, 0,1–0,25 ms) |
| B40.A5 Mask + seleção, traço de 12 lotes 4096² | 1 402,7 ms | 8,5 |
| B19.10 composição 4K, 16 camadas vazias | 216,7 ms | 48,3 |
| B19.10 idem, com traços | 231,5 | 86,4 |
| B19.10 ruído espalhado ¼ α=0 | 281,4 | 309,2 (+4–10 %, ABERTO) |
| B13.6 passo da água 4K, 3 traços, 1 fio (advect) | 124–139 ms | 117–131 (−6 %); a 32 fios no ruído |

- B19.10: `mede_as_tiras_de_cache_4k`, `--test-threads=1`, mínimo de 3, load 20–32. A chave do resumo é a identidade `Weak` do `Arc` do plano;
  também no arrasto de ajustes (`56262508d7`).
- B20.11: mutação 6/7; a 7.ª (níveis fora da chave) é equivalente.
- B13.6: o resto é o piso serial do advect (ADR-0145).
- B22.1: secagem, encharque, tique do traço, settle e aerógrafo em passos fixos de 1/60 s dão o mesmo bit a 30/60/144 Hz. Gates
  `o_mesmo_gesto_a_30_60_e_144_hz_da_a_mesma_tela` e `o_traco_que_anda_e_para_seca_igual_a_30_60_e_144_hz` (red 83·0·542 / 49·0·530 →
  0·0·0).
- B22.4 **RECUSA argumentada:** a caneta já é drenada no topo do quadro antes do `on_tick`; pôr o tique depois do submit atrasaria +16,7 ms.
- Gates herdados consertados: `cada_campo_do_paint_state_e_do_traco_ou_partilhado` (`mask_spacing_saved`, de `39168824c3`);
  `ate_255_tracos_o_dono_u16_e_o_u8_antigo_dao_o_mesmo_bit` e `o_256_o_traco_nao_reescreve_o_estilo_do_255_o` perderam o fenómeno em
  `060b3fe257` (o pen-up só relê traço ⊕ pad); agora correm com `COMMIT_PELA_UNIAO`; mutação u8 → vermelho.

## 5. Prova de fecho (12/11)

- **fmt** ✓ e **clippy** `PH2D_SUITE=fecho -D warnings` ✓ nas 3 crates.
- **21 testes por nome ✓:** `selection_formigas`, `apply_mask`, `mask_gate`, `mascara_selecao`, `a_recomposicao_da_selecao`,
  `as_tiras_de_cache`, `o_resumo_por_ladrilho`, `o_arrasto_de_ajustes_com_resumo`, `ritmo_fixo`, `o_traco_que_anda_e_para`,
  `macro_track::` (87 verdes), `ate_255_tracos`, `o_256_o_traco`, `cada_campo_do_paint_state`, `watercolor_dry_tests`, `selection_editing`;
  em `ph2d-wet-paint` `advect` e `a_saida_por_bloco`; em `ph2d-app-painter` `as_formigas_da_selecao` e `envio_fundido`; em
  `ph2d-editor-core` `the_shell_only_shrinks`.
- **Alarmes de pincel:**
  - largura: só Oil Wash e Smear Wet vermelhos (como antes);
  - miniaturas: **75** vermelhas = as 74 do começo da janela com o MESMO fnv, MAIS três mudanças deliberadas:

    | pincel | efeito | fnv novo | causa |
    |---|---|---|---|
    | Watercolor Basic | 63 px | `0xbdbf06f44afd2928` → `0x56bcf726d060e8b2` | B22.1: o banco de miniaturas tica a 8 ms/Move, não múltiplo de 1/60 s |
    | Brush Pen | 7 321 → 7 756 px | `0xdce74b0a9bcb86ca` | `19764e2d78` (velocidade medida por passo) |
    | Mapping Pen | 258 → 325 px | `0x7117c1b2b0b756b0` | `19764e2d78` |

  **Nada regravado:** é decisão do dono pelo relatório `target/relatorio-pinceis/index.html`.
- **Matriz do app:** [DEPOIS_N4.md](../missao_4k/medicoes/DEPOIS_N4.md) (load 9–90: não compara com o ANTES; a «regressão da aquarela» foi
  desmentida por A×B, `986aab640d`).
- **Não corrido:** a suíte completa (é do integrador).

## 6. Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke`
2. Abra uma imagem grande. Escolha a ferramenta **Selection** e faça uma seleção; deixe-a parada. As formigas andam calmas, na mesma
   velocidade em qualquer monitor. Arraste a forma da seleção e mexa no slider **Offset**: tem de acompanhar sem engasgar.
3. Na aba **Layers** crie várias camadas quase vazias e pinte: tem de acompanhar a caneta.
4. Pinte com **Watercolor Basic**, **Brush Pen** e **Mapping Pen** e diga se o traço está como ontem. O relatório
   `target/relatorio-pinceis/index.html` mostra as três miniaturas antes/depois.
5. Errado: travar, formigas a correr ou a parar, cor diferente.

## 7. Decisões do dono

As **3 miniaturas** (Watercolor Basic, Brush Pen, Mapping Pen): aprovar e regravar, ou mandar voltar. Ficam as anteriores (W5, orçamentos,
vermelhos por decisão).

## 8. ABERTO, por impacto

1. **B19.10, ruído espalhado ¼ α=0** (281,4 → 309,2): medir com calma (load < 5).
2. **Matriz do app** a re-medir com load < 5 (o soltar da aquarela 27,9 → 35–40 ms nos dois binários: anterior a esta janela ou carga).
3. **B20.6, a parte da placa** (`write_texture`; não há StagingBelt).
4. **B13.6** além do piso só com paralelo novo + ADR.
5. **B22.4** recusado (§4). **B23.2** por atacar.

## 9. Riscos para o integrador

- O compositor (§2) é de todo o Painter; se outra linha mudou composição/camadas, re-correr os gates `as_tiras_de_cache` e
  `o_resumo_por_ladrilho`.
- O ritmo fixo muda o que o tique faz por quadro: `macro_track::` (87 verdes) é o gate; qualquer gravação de macro com tempo de tique merece
  olhar.
- `PaintState` ganhou 3 campos; um construtor literal noutra linha não compila (cura: `..Default`/`state_default.rs`).
- **Medições:** a régua é o A×B no mesmo binário (ou 1 fio, `RAYON_NUM_THREADS=1`); a carga foi de 9 a 90, por isso o relógio absoluto vale
  pouco.

## 12. Depois do fecho: os três defeitos do dono (12/11 à noite)

Intervalo `63f5e2ca23..116f9c8072` (19 commits; 53 ficheiros, +3412/−121). Novo fora do Painter: `ph2d-render` (`layer_compositor/compositor/{api,buffers,mod}.rs`, `layer_compositor/mod.rs`, teste `a_camada_alem_do_orcamento.rs`) e `ph2d-vector/src/scene.rs` (+22). `fila_da_caneta{,_tests}.rs` MOVIDA de `ph2d-app-painter` para `ph2d-tool-painter` (catraca verde). `ph2d-wet-paint`: `assenta/memo.rs`, `solver/active_region.rs`, `advect_commit.rs` + 3 `examples/sonda_*_4k.rs`. `shells/desktop`, `Cargo.toml`/`Cargo.lock` e contratos §6 intocados. Colisão: o compositor/`ensure_array` de `ph2d-render` (quem mexa em camadas na placa) e `scene.rs`.

**Defeito 1: «a linha pontilhada da seleção fica invisível a depender do zoom» (B20.11b; `099d47d773`, `906b303443`, `eea0f91ea0`).** As formigas eram texels de 1 px de imagem amostrados por vizinho-mais-próximo. Cura: contorno 128 da máscara em polilinhas (`selection_contorno.rs`, guardado por identidade do `Arc`) traçado em ESPAÇO DE ECRÃ a 1 px (branco + tracejado preto, offset pelo passo); linhas ciano do Offset idem. Acima do orçamento de 20 000 cantos: caminho raster em ecrã (`selection_formigas_raster.rs`). Medido: varinha 4K, 5,26 M lados, 632–939 ms → pior quadro 11,1 ms, passo seguinte 6,9 ms, `bin_data` 10/262 144; 20 k lados = 0,62 ms de encode/quadro. Gates `as_formigas_*` (4 bordas a zoom 0,37/0,5/1/3,3, marcha), `o_contorno_nos_casos_limite`, `o_orcamento_escolhe_o_caminho_dos_dois_lados`. Hang de teste curado (`signum(0.0)` = 1).

**Defeito 2: «o primeiro traço da nova camada em imagem grande com Wet Paint tem delay» (B18.4).** Três causas, todas curadas:
- (a) o pen-down derrubava a composição inteira quando a marca molhada virava: `1beae71478` (307 200 → ~3 300 texels reivindicados; diag `fc0270453e`);
- (b) o «+» recompunha a tela: `66c5dcbca7`, `abdb2e67db` (só a caixa da tinta que adormece). «+» Watercolor Basic 73 → 17–31 ms, Wet 89 → 35–47 ms;
- (c) a placa: `regiao_suja` compunha a tela e enviava a camada nova inteira no 1.º toque. `c39130ac3e`: a camada transparente nasce limpa na placa (`LayerCompositor::nasce_limpa`). `116f9c8072`: `ensure_array` limpava o cache ao crescer a capacidade e re-enviava TODAS as residentes; agora copia as fatias velhas na placa (gate `o_array_que_cresce_guarda_as_fatias_ao_bit`). `o_primeiro_toque_da_camada_nova_envia_so_a_caixa_ao_bit`: 257 956 B → 27 556 B.
- APP (NOVA=1, matriz 3840×2160 papel, Radeon, 3 rodadas intercaladas, mínimo; antes = `28ee89016d`): 1.º toque na camada nova Wet **51,7 → 23,5 ms** (2.º toque ~17–22); Watercolor Basic 1.º toque 66,9 → 63,9 (sem mudança, ABERTO).

**Defeito 3: «líquido escorrendo em vários traços consecutivos fica lento embora o FPS não mostre queda» (B13.12).** A sim corre fora da UI e não acompanhava o relógio. Medido (`sonda_n_tracos_em_tempo_real`): sim-s/s 1,00/1,00/0,95/0,70/0,57/0,49 para 1..6 traços a 4K. Curas ao bit: `memo::potencia` (`422d809eb1`, secagem 63,1 → 50,8 ms); commit do advect pela rota que move menos bytes (`d8ba3eb20c`, estágio 3 19,5 → 16,7 ms); saia do `rebuild_active_region` por colunas em paralelo (`74346c7354`, estágio 0 8,9 → 3,8 ms). Sob carga: 6 traços 0,49 → ~0,89, 5 → 0,98, 1–4 → 1,00. RECUSAS medidas: sono por tile (`8cbfed0328`: 90–98 % dos tiles mudam por passo; os candidatos mudariam o bit) e sono por célula (B13.3). O advect está na banda de memória (~3,6 ns/célula); premissa dos «30» refutada (`3cccc6bcaf`: secagem 4,4 ns/célula).

**Determinismo (B18.5, `713650f15c`).** O acordar da tinta adormecida dependia do agendamento dos fios (anterior à janela: `48becb76ca`, 06/11). Com relógio fixo/fita/ritmo o acordar espera o fio; no relógio de parede o app não espera. Gate `undo_os_gemeos_da_camada_adormecida_dao_o_mesmo_bit_em_qualquer_largura_do_pool`.

**ABERTO, por impacto:**
1. A sim a 6 traços a ~0,89: bytes por célula no advect (momentum no bloco, active/uv relidos, dst_rgb) e a cura 1b da secagem (x/y escalares no resolve/assenta); re-medir sim-s/s com calma.
2. Watercolor Basic, 1.º toque na camada nova ~64 ms (~64 MB de memória a mais, por achar).
3. O nascer da sessão Wet: 10–14 ms no mouse-down (27 MB).
4. A matriz do app com load < 5; o ruído do B19.10; B20.6 placa.

## Binário do smoke

Compilado em `target/smoke/ph2d-host-desktop` (HEAD da linha, depois do fecho; `target/*/incremental` reclamado, 21 GB). 2.ª corrida de `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`: zero `Compiling`, `Finished `smoke` profile [optimized] target(s) in 0.21s`.
