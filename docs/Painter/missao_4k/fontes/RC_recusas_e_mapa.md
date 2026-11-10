# RC — Recusas medidas, mapa do pipeline e docs a atualizar (Painter 4K)

Fonte: `PH2D_Estudos/Painter_2026-11-09/secoes/14_recusas_e_docs.md` (§14.a, §14.c, §14.d) e `secoes/02_mapa.md` (§2.3).
Data: 10/11/2026. Extraído por agente; endereços conferidos na worktree `Worktrees/line-PainterWatercolor` (grep em 10/11; o `02_mapa` era de `22d0c6a651`, o HEAD hoje é `cd02ac77ef`; linhas que andaram vão marcadas).

Abreviaturas de caminho (todas sob `docs/Painter/` do repo): `25`=`25_avaliacao_gpu.md` · `26`=`26_plano_performance_procreate.md` · `28`=`28_otimizacoes_o_que_funcionou.md` (`A28` = o arquivo/anexo do doc 28) · `29`..`33` = `29_offthread_sim.md`, `30_plano_multiresolucao.md`, `31_handoff_watercolor_perf.md`, `32_aquarela_o_que_custa_hoje.md`, `33_plano_gpu_do_carimbo.md` · `43`=`43_a_pilha_do_composite_brush_ficou_rapida.md` · `44`=`44_a_fila_depois_da_linha.md` · `RS/`=`49_realshader/` · `J13`=`Arquitetura/J13*` · `ESTUDO`=`ESTUDO_CUSTO_WET_PAINT_2026-11-07.md` · `B`=`BUGS_painter.md` · `UR`,`TV`,`O2`,`MP` = handoffs em `docs/Painter/handoffs/` (nomes no cabeçalho do `achados/A2_historia_recusas.md`) · `ADR-n` = `docs/architecture/decisions/`. Os números `:linha` são os do estudo (docs), não reconferidos.
Legenda da coluna final: — não reabrir · 🔁 reabrir SÓ com nova MEDIÇÃO (facto novo) · ⚠️ outra coisa com nome parecido.

## RECUSAS MEDIDAS

Formato: `RC-n | recusado | medição | porquê | doc de origem [reabrir?]`

### Carimbo, eventos e tiles
RC-1 | Frente T: tiles na contabilidade do sujo (`dirty_rect`), construída e REVERTIDA | grade reduz só ~1,4×; relógio +12–14 % em 2 gestos | o excesso mora nos chamadores do `mark_dirty` (bbox por segmento) | 26:310-341 [— ⚠️ tiles para ARMAZENAR com COW (T1.1/G1-05) e tiles que dormem na água (D_WET b1) são outra população]
RC-2 | Tiles 64/128/256 da literatura | perdem para o bbox: 10,40× contra 1,66× | o tile útil ≈ diâmetro do pincel | 26:332-334 [—]
RC-3 | Derivar a janela do undo do `mark_dirty` (S1) | revertida | `mark_dirty` diz onde a imagem mudou, não onde se escreveram bytes | 28:763-767; A28:702 [—]
RC-4 | Frente C: juntar dabs de um quadro para poupar trabalho | 1,00×; +86 % dabs ⇔ +89 % tempo | `stamp_dabs` percorre a pegada de cada dab de qualquer forma | 26:540-579; 28:68-81 [🔁 parcial: lote ≥ 2 dabs dá à rota em bandas (`stamp_banded.rs`) algo a paralelizar; só pincel pequeno r ≲ 28 (D_DIGITAL E2, SH-03)]
RC-5 | «O custo é por evento» (Composite Brush) | 1440 eventos ≈ 45 eventos ≈ 245–263 ms | o custo é por PÍXEL percorrido | 43:76-80 [—]
RC-6 | Aumentar o espaçamento no preview | redundância 10× → 5×, reprovado na tela | muda o look (lei produto-por-dab) | 33:128-133 [—]
RC-7 | Portar `save_region` / micro-otimizar o kernel CPU do carimbo | 1,48 ms = 7 %; 1,06 ns/visita | não é gargalo; CPU no teto | 33:27-30 [—]
RC-8 | Banda do depósito no `rayon` em vez de `std::thread::scope` (linha Y) | passo 2: 4,41→4,27; passo 8: 11,39→11,51 ms/quadro (load 27–29, 1024²) | ganho zero; os 49,6 % em `pthread_create` eram o `gdb` | 43:262-280,325; ADR-0172 [🔁 só FATIAS PEQUENAS: `smear_field.rs:291-296` mediu 227→152 µs por pingo r128,8; a 4K r ≥ 200 a área por dab é 10–50× maior. «rayon com as MESMAS 9 fatias grandes» NÃO]
RC-9 | Otimizar `Arc`/upload do preview por camada | `take_preview_arc` ≤ 6,5 ms | não é o penhasco | handoffs/HANDOFF_per_layer_color_perf_artifacts.md:39,210 [—]

### Pen-down, alocação e undo
RC-10 | Reusar a capacidade dos planos (`clear()+resize`) | pen-down 17,6→47,5 ms | memset explícito de 235 MB; `vec![0;n]` já é `alloc_zeroed` | 26:409-414; 28:128-134 [— (Liquify `disp` 66 MB: cura = guardar o «antes» por ladrilhos)]
RC-11 | «O reuso mata a falta de página» | cópia 11,68 de 12,35 ms | a alocação vale 5 % do pen-down | 26:493-496 [—]
RC-12 | «O U1 cura o pen-down» | 16,49→16,07 ms | 2.ª referência ao canvas força `Arc::make_mut` | 26:483-501 [— (cura = T1.1/G1-05: capturar o «antes» por região)]
RC-13 | Cap de undo fixo de 512 MB | 204/62/17 traços a 1024²/2048²/4096² | promessa «>300 traços» falsa | 26:503-513 [— (substituído pelo teto em bytes)]
RC-14 | Paralelizar a extração da janela do undo | ~470 MB de banda | o custo é o SCAN, já paralelo | 28:745-748; A28:647 [—]
RC-15 | Meia-versão barata do journal (só Ctrl+Z) | recusada | caminho de falha deixa o documento sem pixels | 28:790-793 [—]
RC-16 | Material neutro = «plano ausente» / «preencher só a janela» | recusas no código | pincel de fábrica tem Shine 0,7; papel tem presença 1 em toda a parte | `crates/ph2d-tool-painter/src/plane_copy.rs:153-158` [⚠️ NEUTRAL = bytes zero (nascer por `alloc_zeroed`) não é nenhuma das duas]

### Impasto
RC-17 | Fundir varreduras de pigmento e de altura | leitura de código | premissa falsa: um é disco, o outro cápsula | 26:697-704; 28:83-88 [—]
RC-18 | Gatear o AA por raio | 105 660 bytes diferentes, pior delta 62 (r=100) | muda a arte | 26:706-711; 28:115-118 [—]
RC-19 | AA com 5 amostras, em cruz, ou colapsado em 3 leituras | 56,67 níveis; 4,949 e 5,344 contra 0,060 | `Constant` erra 2/9 exato em todo raio | 26:713-747; 28:90-113,313-348 [—]
RC-20 | Encolher a LUT para caber no cache | N=512..16384 ≈ 110 ms | indiferente | 26:921-924; 28:120-126 [—]
RC-21 | `#[inline(never)]` ou closure por texel | 2,53 contra 2,57 | não era o custo | 28:276-285 [—]
RC-22 | «Superaditivo / localidade de cache» | 2–9 %, r100 não-monótono | desmanchou com mediana de 5 | 26:667-675 [—]
RC-23 | Materializar o relevo da tela INTEIRA por quadro (`impasto_fields`) | 3,9→2,1 ms a 2048² | medido e recusado no doc-comment | `crates/ph2d-tool-painter/src/tool/paint/impasto_light.rs:148-152` [⚠️ REGRESSÃO: o ramo Gaussiano/Nitidez voltou a fazê-lo (`impasto_light.rs:270-283`, `AdjustWindow::full`), R_F10 F10-01]
RC-24 | `invalidate_composite` em `sync_relief_flags` | 225 ms a 4096² | derruba cache e recompõe a tela inteira só para publicar um booleano | `impasto_live.rs:396-424` (comentário confere) [⚠️ `refresh_live_relief` (`impasto_live.rs:426`) ainda chama `invalidate_composite` a cada tique do slider Body]
RC-25 | Média deslizante f32 no `impasto_settle` | deriva | — | R_F16 [⚠️ soma INTEIRA u32 (F16-14) é exata: não é a recusada]

### Placa (GPU)
RC-26 | Portar o depósito do Digital para a placa | 1,15 ms/move; exigiria canvas residente (211 refs a `canvas_rgba` em 51 ficheiros) | custo de arquitetura | 25:256-265,288; 26:295 [—]
RC-27 | Impasto na pista da GPU | 2,61 ms GPU contra 2,20 CPU a 4096² | pista fazia TUDO em tela inteira | 26:89-91 [🔁 re-medir com composite por região (`api.rs:158-170`) e ramo parcial (`painter_gpu_preview.rs:312-321`), nunca tomados (G1 T3.1/G1-02, D_IMPASTO S1)]
RC-28 | Carimbo na placa sem piso | 0,22× (figura 1900) → `MIN_REDUNDANCY = 4,0` | 4,5× mais lento em figura grande | 33:115-120; `tool/paint/stamp_device.rs:222-242` [— (medido só r≈155 a 4096²; pincel pequeno nunca)]
RC-29 | S4: canvas residente na VRAM durante o gesto | fronteira 2,80 contra 18,30 ms de CPU | deixaria `canvas_rgba` obsoleto para ~25 leitores | 33:122-126 [—]
RC-30 | Re-derivar o fold das camadas no shader | — | seria 2.ª resposta para «como camadas se empilham» | 25:337-339; 26:297-299 [—]
RC-31 | Premissa «o Procreate é GPU-residente» | — | teto de camadas do Procreate depende da RAM | 26:80-91 [—]
RC-32 | Placa do Wet ligada por omissão (`PH2D_WET_GPU`) | 78,6 contra 6,7–8,7 ms/passo; subir+descer ~0,45 s/traço; RTX travou o PC 2× (Xid 62/109) | Radeon no app 7,5–10× mais lenta que a CPU | RS/LIGACAO_PLACA.md:89-93,127-129; RS/PLACA_degrau6.md:3-7; `wetpaint/offthread/placa.rs:25` [🔁 só DEPOIS de curar (passadas na folha inteira, hash da grade inteira a cada chegada, descida por linhas inteiras com espera) e medir a RTX no app (nunca medida); ⚠️ a placa só anda a água DEPOIS do traço]
RC-33 | Double-float/Kahan na secagem da placa | massa 2,0e-5→1,9e-5; ms 5,15→5,45 | não pegou na RADV | RS/PLACA_degrau5.md:81-85,109-124 [—]
RC-34 | Despachar `Rect::full` em todas as passadas da placa | 41 k workgroups; caixa era 29 % da folha | custo à toa | RS/PLACA_degrau4.md:57; PLACA_degrau6.md:19,63-65 [— (é a cura pendente, não a recusa)]

### Wet: solver na CPU
RC-35 | Paralelizar o Gauss-Seidel inteiro | 34 % serial | 4 passes seriais por semântica (ADR-0134) | A28:1995,2119; 29:113-120 [— (resolvido por Jacobi, ADR-0147)]
RC-36 | Igualar a cadência da água na CPU | pior caso 7 Hz | muda a física | A28:2330 [⚠️ cadência POR TILE (D_WET b1) é parente: mostrar ao dono o tempo de secagem lado a lado]
RC-37 | Encolher pool do rayon, espiral, hipóteses 5.47–5.64 | várias | refutadas | A28:3425,3512,3711,5213 [—]
RC-38 | Secagem TOL, FIXA, UMA (atalhos do Newton) | máx 4,7–21,6 níveis; ganho ≤ 7 % | erro acumula sob Glaze | RS/SECAGEM_diagnostico.md:75-88 [—]
RC-39 | Memo do `levanta`; `#[inline(always)]`; cortar o Newton do levantar | 70,9→70,4 ms | custo eram cópias `[T;3].map` | RS/MEDIDA_31_10_RODADA4.md:44-46; RODADA5.md:77-79 [—]
RC-40 | Fundir `edge_rows`; `flow_at_point` em linha; escrever `flow_x/y` só quando muda | 34,8×35,8; 52,8×52,7 ms | ~1 ms / ruído / quase nunca igual | RODADA5.md:76; RODADA6.md:54; RODADA9.md:46-51 [—]
RC-41 | SIMD à mão no `transport_rows` | 14,3 contra 11,6 ms | já a ~1,2× do chão de banda | RS/MEDIDA_31_10_RODADA6.md:55 [—]
RC-42 | Advecção fundida com bloco 64/32/8/4; `momentum` dentro do bloco; fundida em grade < 4 M células | 54,23/55,48/51,52/51,27 contra 50,30 (bloco 16) | halo recalculado, L2 disputada | RODADA7.md:103-113; RODADA8.md:11-40; `advect_fundida.rs:56` [—]
RC-43 | Stores não-temporais | — | exigem `unsafe`, proibido | RODADA8.md:42-48 [—]
RC-44 | Zerar `div/prs` dentro das passadas de escrita | −16 ms em ±50 de ruído | inconclusiva sob carga | RS/FLUXO_cortes.md:45-48 [re-medir só em rodada calma]
RC-45 | Planos mais estreitos (f16/bf16/u16 em `vel`, `film`, `susp`; `susp_rgb` u16; dither) | p99.9 de 4,47 a 11,9; 1 ulp em `vel` já dá máx 8 | visível na cor | RS/MEDIDA_31_10_RODADA10.md:24-73; `estreito.rs` [— (abertos e NÃO medidos: `susp_rgb` pré-multiplicado e u16 estocástico, RODADA10:53-61)]
RC-46 | Solver em f32 | ganho ≤ 4 %; cor infinita em 1–1,6 % das células | instável | RS/SONDAS_C_f32_v3.md:21-40 [—]
RC-47 | `target-cpu=x86-64-v3` (AVX2) | −8,5 % só na rota serial; adotado o v2 | exige `unsafe`; rayon não herda `#[target_feature]` | SONDAS_C_f32_v3.md:29-35; `.cargo/config.toml:15-27` [—]
RC-48 | 32 fios dentro de um teto de 16 núcleos | 127,4 contra 85,0 ms (+50 %) | throttle do cgroup | RS/MEDIDA_31_10_RODADA3.md:48-50 [—]
RC-49 | `grid_ratio` ligada ao Size; protótipos T (textura), B (fios), E (relevo); carimbo de grade | Size 200: Flat 19,6→6,9 s, Wet Round 1,42→0,26 s | razão é da SESSÃO: mudar mata a tinta viva, borra o Thick, fura o Dabber. DONO 07/11: «deixar como está» | handoff UR:110-132; ESTUDO:169 [— (decisão do dono)]

### Wet: tinta viva, óptica, cerdas
RC-50 | Cache da luz por célula (ε) | 19,58 sem · 22,4–24,3 ms com | refaz ≥ 69 % da caixa | J13:52-69,111 [⚠️ cache do DENTE do papel (estático, sem ε) não é esta]
RC-51 | Cache exata da óptica | poupa só ~35 % | 60–70 % da janela refaz | J13:49-50,110 [—]
RC-52 | Óptica no worker entre passos | +50 ms no passo seguinte; teto ~16 imagens/s | — | Arquitetura/J13b_plano_otica_fora.md:9-12 [—]
RC-53 | Compor o Wet Round por linhas depois de soltar | previsto 4K 541/1160 ms, não alcança | a óptica é 78–94 % | handoff TV:66; O2:203 [—]
RC-54 | Tirar `marca_tudo()` de `land_direct_remainders` | continua +117 MiB | não muda nada | project-memory/project_first_stroke_undo_memory_cut_next_round_2026_11_07.md:12 [⚠️ outros `marca_tudo` (`pousa_grao_solto`, `deposit_mask`, `placa.rs:57-58`) não medidos]
RC-55 | Marcar `preview_dirty` com a mão parada; adiar a aguarela do pen-down para o tique | 89 ms continua (Wet Wash 3240², sob carga) | sem conclusão | B:658 [—]
RC-56 | Recompor a tela inteira depois de um undo com células enterradas | muda 3 862 px (orla re-ilumina) | muda a imagem | O2:15 [—]
RC-57 | Jitter 0, matiz das cerdas, teto de 3 px no sub-dab | carris (0,72) / berrante a 0,08 / escada 4,71; jitter custa −17–22 % | look | ESTUDO:63,69,93 [—]
RC-58 | Pente com cobertura ×4–16; jitter no mesmo px; «T» no Wet Round | tijolinho / carris / estria | look | UR:147-153 [—]
RC-59 | Amostra viva pura do Color Mixing; Density como teto | G-Pen não misturava; véu chapado | look | UR:32 [—]
RC-60 | Desligar controlos sem uso para acelerar | ~0 ms | desligados quase não custam | ESTUDO:24,78 [—]
RC-61 | Lei nova da tinta (Pigmento, E11) | Wet Liner 1,5→4,2 ms; placa 5,7→9,2 ms/passo | ARQUIVADA pelo dono 05/11; 2–3× por evento | Arquitetura/LEI_NOVA_ARQUIVADA.md:1-9; RS/PLACA_E11_tinta.md [— (medições E11 não valem para o produto)]

### Watercolor
RC-62 | Cortar taps do warp | 226 degraus → 0 com warp 48 | o warp É a cura da borda serrilhada | 31:101-105; A28:241 [⚠️ tabela de `hash2` por composite (D_WATERCOLOR E4) não corta taps]
RC-63 | Fatorar os 2 eixos do warp; aproximar dentro do texel | 1,20× na função; 0,12–0,17 ms em ±0,13 | dentro do ruído; mesma classe do AA | 31:107-131; 28:880-887 [—]
RC-64 | Janela deslizante na secagem | 1,02×, revertida | cria dependência entre linhas | 32:88 [⚠️ R6 (paralelizar só as cópias) é o contrário]
RC-65 | Piso da erosão; retângulo da humidade a encolher; `box_blur` reusando o prefixo | ~1,00×; 0,2 ms | sem ganho | 32:89-91 [⚠️ a «variante pequena» do G1 T6.1 (partilhar prefixos `near`/`far`) pode ser esta: conferir 32:89-91 antes]
RC-66 | Pour pelo retângulo do quadro | — | muda a tinta (`dry_canvas_wet` do mesmo tique decai o alvo) | 32:92 [🔁 condicional: só nos quadros em que o secador NÃO deu passo (~58 %); gate ao byte de traço longo (D_WATERCOLOR R8)]
RC-67 | A LUT do impasto na aquarela | — | estrutural, não transfere | A28:135 [—]
RC-68 | Paralelizar o gerador do tile do papel | — | ordem do RNG é a impressão digital do motor | 44:333-342 [—]
(NÃO recusada por medição: Possibilidade B do Rewet — cache de `pres/wr/wg/wb`; 32:142-174 adiou-a por memória (268 MB a 4096², 133–265 MB a 4K) e falta de gate.)

### Aparência, máscara e outras (regra do dono ou do olho)
RC-69 | Esconder/baratear a textura durante o arrasto | — | DONO 22/06: a textura TEM de ficar visível no arrasto — baratear o trabalho, não o look | project-memory/project_painter_texture_brush_stamp_cache.md [—]
RC-70 | Proxy de baixa resolução à Krita (Instant Preview), Wavelet Turbulence, upsample do pigmento | popping; mau encaixe | look | 30:112-136 [— (G1 §8 confirma)]
RC-71 | Máscara: envelope (§13.6), teto por época (§13.7), envelope Wash do Krita (§13.9), pull-back pela base (§13.11.2) | 0,886→0,992 vira 0,667→0,141 | linhas brancas; vazou para o pincel normal; «péssimo» (dono) | 25:431-452,462-471,510-517 [—]
RC-72 | Perseguir a cauda de 100 ms lendo código | — | a ferramenta é o `perf` | 26:300-302 [⚠️ `perf_event_paranoid=2`: usar `docs/Painter/ferramentas/amostra_gdb.py`]
RC-73 | Relief linear no Digital; «curar» o relevo de volta ao oráculo Cycles | — | Paper Kind inerte; dono aprovou 2,24 | handoff RT:14; MP:95 [—]
RC-74 | Teto do transporte do esfregão (2,0 raios), passo de volta pelo arco (`arco_do_caminho.rs`) | — | só limitar o módulo cura e o dono recusou | B:287-358; `TECTO_MEDIDO_E_RECUSADO_EM_RAIOS` [— (R_F12 F12-20)]
RC-75 | Cura simulada do #54; Over Oil / lei J8; «variar a borda pelo papel» (#62); #78 (sortear a pegada) | 25 miniaturas mudam até 10 níveis; revertido `702e98f913` | inconclusiva; revogado pelo dono 08/11 | B:496-507,833,931 [—]
RC-76 | Manter o campo denso do Liquify e animá-lo | 128 MB/pose a 4096² | memória | ADR-0157 (R_F18) [—]
RC-77 | Cache por versão dos planos do sculpt | — | luz velha invisível | handoffs/HANDOFF_line_Painter_gpu_light_2026-07-18.md (~l.258) [—]
RC-78 | Fifo por omissão, Immediate só opt-in | — | no Metal não há Mailbox | docs/perf/mouse-stutter.md:66-92 [🔁 no Linux/KWin o Mailbox existe e `desired_maximum_frame_latency` nunca foi medido (R_SHELL SH-02)]
RC-79..RC-125 | As 47 recusas indexadas do doc 28 | — | — | 28:907-967 → A28 (a tabela do §14.a cobre-as por tema; detalhe lá) [—]

### Recusas de MÉTODO (§14.d: o que não vale reutilizar)
RC-M1 | Medidas antigas a 4096²/4096×2560 como 3840×2160 | só 4 medidas antigas são a 3840×2160 (Rebelle, 20 facas, passe de luz do Óleo Real, teste do orçamento de camadas) | escalar ×0,49 (de 4096²) / ×0,79 (de 4096×2560) só vale para o que percorre a TELA; pisos mudam de regime: `MIN_CELULAS_FUNDIDA=4M`, `PARALLEL_MIN_AREA`, `MIN_REDUNDANCY=4` | 14.d1
RC-M2 | Absolutos sob carga > ~5 | rodadas load 20–210 | só a RAZÃO entre variantes intercaladas no mesmo processo | CLAUDE.md §5.0; medicoes/A3,M1
RC-M3 | Perfil não dito / `release` | 14,4×11,1 ms na pilha do Composite; ~1,8× no Newton | dono pinta em `smoke` (cgu 16); `ph2d-tool-painter` compila com 16 unidades; só `ph2d-painter-brush` e `ph2d-wet-paint` têm cu=1; rodadas 3–5 (cu=16) ≠ 6–9 (cu=1) | Cargo.toml:192-212
RC-M4 | Sonda externa ≠ app | fatia de 16 fios vs 32; sem feature `gpu`; builds `cfg(test)` pagam journal e «espia» do esfregão (8 MB/lote; 66 MB a 4K) | `cargo test` a 4K mede a espia | M1/A3; R_F12 F12-17
RC-M5 | Diâmetro passado como raio nas sondas (`measure_relief_systems.rs:211,481,595`; `measure_route_cost.rs:131`; `measure_undo_cost.rs:289,338`; `measure_wetpaint_stamp.rs:296,449`) | 4× a área por dab | `set_brush_size_px` toma o RAIO | R_F15 F15-01
RC-M6 | Sondas do Wet GS vs Jacobi (`measure_wetpaint_passes.rs`, `measure_wetpaint_flow.rs`) | descrevem passo que o produto não dá | produto corre Jacobi | R_F15 F15-07
RC-M7 | Luz do relevo acesa 2× nos gates (`medida_oleo_real_fenda.rs:289-301`, `medida_paper_press.rs:40-49`) | papel nu leva `shade²` | âncora 2,24 (faixa 1,84..2,67) mede imagem que o app não mostra; re-ancorar a faixa, nunca repintar | R_F22 F22-11
RC-M8 | Sondas «com a sim a correr» (Live Water nasce falso) | razão ~1,00× por construção | `agua_viva` falso | R_F15 F15-06
RC-M9 | Placa do Wet na sonda (`PH2D_WET_GPU` lido 1×/processo; recusa de `placa_sabe` silenciosa) | pode medir CPU sem saber; 9,24 ms da RTX é residente fora do app | — | M1-07
RC-M10 | Fork isolado | 3,26 ms sonda × 21–27 ms no app | faltas de página, `Arc` com 2 donos | 44:315-330; B:654
RC-M11 | Sonda por MOVE | não viu véu 220 ms, secagem 28 ms | mede o evento, não o quadro | 31:58-66 × 32:23-29
RC-M12 | L0 `EVENTO→FRAME` | para no fim do quadro | fila da swapchain (latência 2) fora da conta | 26:350-352
RC-M13 | «O mínimo» nas rodadas da água; 1.ª tentativa de 6 rondas num wash | rodadas não são réplicas; wash barateia 530→17 ms | régua certa = 1 wash NOVO por ronda | J13:108-109; RS/MEDIDA_31_10_RODADA3.md:16-21
RC-M14 | Tabelas a grade 1×1 (07/11) | geometria do dono é fluxo 2, 4K: pen-down 34–38 ms | — | UR:49-53
RC-M15 | Memória do Wet antes de 06/11 | 577 MB eram 5 081 MB | `bytes_retidos` contava 0 com motor `Away` | O2:31
RC-M16 | Comparações com o Rebelle (4096×2560 vs 3840×2160 dele; 6,4 s dos 8 s são ritmo de envio) | históricas; Rebelle deixou de ser referência (31/10) | — | RS/D2_rebelle_passo_agua.md:25; D_rebelle_4k.md:34-35
RC-M17 | «CPU calma» da advecção (4 valores 20–38 · 71,7/54,6 · 23,2 · 31,5 ms); escada parte de 85,0 e não de 100,5 | qualquer ganho «placa × CPU» | — | RS/PLACA_degrau1.md:40-43; ESTADO_REBELLE_MEDIDO.md:75; RODADA3.md:106-108
RC-M18 | Modelo células × ns/visita do ESTUDO | falha a Size 30 (prevê 3,3×, mede 2,1×) | é teto/hipótese | ESTUDO:136
RC-M19 | `mede_o_dab` no Digital | lê 0 dabs | — | A3 `A3_ferramentas.md:324-330`
Régua a usar antes de qualquer cura (A2-01): 4 modos × 3840×2160 × `--profile smoke` × traço fixo (Size 30/120/240), p50/p95 do quadro no fio da tela + INPUT por evento + pen-down; variantes no MESMO processo, intercaladas, o mínimo (salvo rodadas da água), load anotado.

### Reabertura legítima (🔁 FACTO NOVO) — resumo
RC-4 (lote por quadro p/ pincel pequeno) · RC-8 (rayon com fatias pequenas) · RC-27 (Impasto na GPU com região) · RC-32 (placa do Wet depois de curar) · RC-66 (pour pelo retângulo nos quadros sem secador) · RC-78 (Mailbox/latência no KWin).

## O QUE FUNCIONOU (não desfazer) — §14.b
Compositor na placa 66–107× (885× no HSB) 25:50-57 · orçamento 16/32 camadas a 4K `layer_compositor/mod.rs:123-180` · undo por janela U1 67,8→2,36 MB/passo `undo_shape_state.rs:146-149` · `target-cpu=x86-64-v2` −20–23 % · L0 `PH2D_PAINT_PERF` · carimbo device com `wants_device` ≥4 · `fork_par` no pen-down 10,3→3,9 ms · Composite: 1×/quadro + linhas rayon + `codegen-units=1` (240→68–86 ms/traço) · Per-Layer Color 354→13,1 ms · Watercolor: cadência 1/quadro, pen-down sem 268 MB, Smudge sem fork 49,6→5,1, véu 220,8→16,6, secagem/pour paralelos · Impasto: LUT do AA (−28 %), fold do relevo paralelo + janela na placa (232,7→1,1–1,3 ms), luz na placa, prewarm no bind, journal do undo · Wet: move plano, água fora do fio (`EngineSlot`, 15→33,4 Hz), Jacobi + rayon, secagem cortada ao bit (passo 128→50,3 ms), undo COW 64×64, óptica em fio próprio (só 1024²), P8, 20 facas, adormecer, tile do papel 1× (7,41→0,013 ms), placa opt-in com guarda `TETO_MS=25`.

## MAPA DO PIPELINE

Abreviaturas: `tp/`=`crates/ph2d-tool-painter/src/tool/paint/` · `tl/`=`crates/ph2d-tool-painter/src/tool/` · `ap/`=`crates/ph2d-app-painter/src/` · `wp/`=`crates/ph2d-wet-paint/src/` · `sh/`=`shells/desktop/src/`. ⚠️ = trabalho em TELA INTEIRA (W×H; 8,29 Mpx a 3840×2160). ✔ = linha conferida por grep em 10/11; (doc) = do `02_mapa`, não reconferida.

### Tronco comum (todos os meios)
- caneta/rato → `sh/input_dispatch.rs:150 pointer_moved` → `painter_canvas_input.rs:115 painter_canvas_move` (doc); geometria ecrã→pixel refeita a cada evento (`painter_canvas_input.rs:270-419`, doc)
- `PainterTool::on_canvas_pointer` `tp/canvas_pointer.rs:11` ✔ (corpo `:37`; Down→`paint_begin` `:129` ✔)
- `paint_begin` `tp/stroke_lifecycle.rs:25` ✔ · `paint_extend` `:305` ✔ · `paint_end` `:485` ✔ · `paint_tick` `:388` ✔ · `paint_tick_do_traco` `:419` ✔
- `Stroke::extend` `crates/ph2d-painter-brush/src/stroke.rs:389` (doc)
- roteador do carimbo `tp/stamp_route.rs:425 stamp_dabs_inner` ✔ → `stamp_dabs_wetpaint` `tp/wetpaint/dab_route.rs:32` ✔ · `stamp_dabs_height` `tp/impasto.rs:61` ✔ · rotas de cor `tp/stamp_cache.rs`/`stamp_banded.rs` (doc)
- `mark_dirty` → `dirty_rect` (união das caixas) → ramo final

### Ramo final comum (por quadro, corre SEMPRE: `ControlFlow::Poll`, `sh/main.rs:1013`)
- `sh/render_loop/mod.rs:486 run_render_frame` (doc) → `ap/painter_bridge.rs:60 dispatch` ✔
- decisão: `ap/painter_gpu_preview.rs:166 gpu_eligible` ✔ (recusas: pilha trivial :238, Repeat Image :247, máscara viva :262, papel de cor+aguada :268, ajuste sem op GPU :271 — doc)
- PISTA GPU: `try_drive` `painter_gpu_preview.rs:125` ✔ → `drive` `:195` ✔ ⚠️ composite N camadas `Region::full` (:313 doc) + premul W×H (:509) + cópia GPU→GPU W×H (:343) em todo quadro sujo (a única chamada de produção passa a TELA)
- PISTA CPU: `tl/runtime.rs:402 take_preview_arc` ✔ → `drena_o_estado :437` ✔ → TrivialFast `:503` ✔ (O(bbox)) · Partial (composite_region(bbox)) · Full ⚠️ (composite W×H, acumulador `[f32;4]` 133 MB alocado por chamada, `tl/compositor/compose.rs:282` doc)
- upload: `ap/painter_bridge_upload.rs:76 upload_cpu_preview` ✔ → `:197 plan_upload` ✔; Partial = bbox; Full ⚠️ = clone 33 MB + premul W×H + upload 33 MB (seed; máscara viva TODO quadro, `runtime.rs:553-565` doc)
- ⚠️ `regen_mips` da textura INTEIRA (12 níveis a 4K, 1 passe+submit por nível) a cada escrita no slot: `crates/ph2d-render/src/individual_read.rs:189,268` ✔ (chamadas; `individual.rs:162-173`, `mipgen.rs:140-197` doc)
- ⚠️ apresentação: `sh/render_loop/present.rs:27` (doc) — painel+chrome+Vello reconstruídos todo quadro (O(JANELA), mesmo parado); `request_redraw()` incondicional `present_title.rs:66` (doc)

### Fase 1 — toque / 1.º dab (pen-down)
- `paint_begin` `stroke_lifecycle.rs:25` ✔ → ⚠️ Digital `fork_canvas` `tp/plane_fork.rs:176` ✔ / `fork_par_raw :246` ✔: copia a tela (33,2 MB) se há 2.º dono forte (o «antes» do undo); SERIAL a 4K porque 33 177 600 B < `PAR_MIN_BYTES = 32<<20` (`crates/ph2d-tool-painter/src/plane_copy.rs:35` ✔); 3,3–13,1 ms; repete-se após traço com bbox ≥ 50 % (undo Whole)
- ⚠️ Impasto: 5 planos de traço do tamanho da tela (h f32, p f32, g u8, f u8, r f32 = 14 B/px, 116 MB) `tp/impasto.rs:160-185` (doc), do pool `relief.take_planes`; 1.º traço do doc aloca novo
- ⚠️ Watercolor: `freeze_watercolor_ground` `tp/watercolor_backdrop.rs:66` ✔ → `composite_below` em SÉRIE, acumulador f32 133 MB (`tl/compositor/compose.rs:169-175` doc) + `wet_substrate = vec![0.0;n]` + fill(NaN) f32 + 1.º carimbo texturizado aloca `stroke_density/deplete/deplete_prox` (`tp/watercolor_accum.rs:397-427`, backfill serial)
- ⚠️ Wet: motor montado FORA do pen-down (`tp/wetpaint/preparo.rs:82,195`, grade 1 célula/px = 8,31 M células); `sess.base` (Arc da tela, strong 2) 1×/sessão; `try_bring_home` `tp/wetpaint/offthread.rs:245` ✔ (bloqueia ≤ `TICK_WAIT` 4 ms)
- ⚠️ Composite Brush: pré + 1 plano de tela por camada viva; Color mixing «Wet blend» pegadas do tamanho da tela

### Fase 2 — traço por quadro
- por evento: `stamp_dabs_inner :425` ✔ — O(pincel); Digital em bandas rayon se lote ≥ 2 dabs, senão serial com kernel dividindo dabs ≥ 3 232 px (`crates/ph2d-painter-brush/src/dab/bands.rs:32,49` doc); rota device `tp/stamp_device.rs` (readback síncrono, `ph2d-paint-gpu/src/lib.rs:270`); Impasto `stamp_dabs_height` antes da cor + `rebuild_live_layer_relief` `tp/impasto_live.rs:196` ✔ + `commit_stroke_height :21` ✔; Wet `stamp_dabs_wetpaint` O(r²) células
- por quadro: `paint_tick :388` ✔ (via `sh/render_loop/fase_fixed_step_clocks.rs:46-49` doc)
  - Watercolor: `dry_canvas_wet` `tp/watercolor_dry.rs:23` ✔ (bbox da poça) · `apply_watercolor` `tp/watercolor_render/diag.rs:26` ✔ (janela = dabs do quadro + pad; Rewet>0 ⇒ `build_rewet_fields` `tp/watercolor_field.rs:264` ✔, 10 box blurs em resolução cheia na janela, 8,74 ms a 4096²) · ⚠️ `pour_canvas_wet` `tp/watercolor_backdrop.rs:340` ✔: bbox CUMULATIVA, cresce com o comprimento do traço
  - Wet: `wetpaint_tick` `tp/wetpaint/ciclo.rs:122` ✔ → `wet_recebe_imagens` `otica_no_tique.rs:94` ✔ (laço de píxel OVER sess.base na CAIXA de toda a água viva + cópia `agua_antes`, por imagem) → `hand_off_sim` `offthread.rs:374` ✔
  - worker `ph2d-wet-sim`: `worker_loop` `tp/wetpaint/offthread_laco.rs:64` ✔ → `sim_step_stage` `wp/sim.rs:487` ✔ (40 Hz; faixa viva por linha; sem estado «parado»); ⚠️ espelho `wp/painter/espelho.rs:104-193` copia serial, linhas inteiras, TODAS as camadas (só com folga ⇒ a 4K ~1 imagem/7–9 s); `PH2D_WET_GPU=1` sobe/desce linhas (`placa.rs:25`)
  - Impasto/GPU: ⚠️ `impasto_gpu_planes_in` `tp/impasto_gpu.rs:173` ✔ (fold CPU: janela 0,38 ms, TELA 14,55 ms a 4096²) + ImpastoLightPass `Region::full` + premul + cópia W×H todo quadro sujo; se GPU recusa: ⚠️ `apply_impasto_light` `tp/impasto_light.rs:327` ✔ laço SERIAL ~0,8–0,95 s no quadro Full a 4K (hipótese)
  - véu de humidade (Show Wet): `ap/painter_bridge_wetness.rs:17`, `build_veil :184` serial ~6 ms a 4096² (doc)
- 1 camada Digital/Watercolor: TrivialFast+bbox; ≥ 2 camadas ou Impasto: ⚠️ pista GPU na TELA

### Fase 3 — soltar / pen-up
- `paint_end` `stroke_lifecycle.rs:485` ✔: Digital commit do undo 0,13–1,27 ms · Watercolor `apply_watercolor(true)` com janela = o TRAÇO inteiro + pad (pen-up ~180 ms a 4K na sonda) · Wet `wetpaint_stroke_end` `tp/wetpaint/ciclo.rs:63` ✔ (sessão continua; worker simula até secar) · Impasto: ⚠️ undo dos 3 planos de relevo (`commit_stroke_height`)
- `mark_dirty` final + ramo final comum

### Fase 4 — parado (sem caneta)
- `run_render_frame` corre SEMPRE: ⚠️ painel+chrome+Vello+present todo quadro (O(JANELA)) · `paint_tick` 1×/quadro: Watercolor secagem/aguada/pour; Wet: tique + worker 40 Hz até `has_fluid=false` (dorme 4 ms, IDLE_SLEEP; tique ocioso ~70 µs) · óptica fora do fio: `ph2d-wet-otica` `offthread_otica.rs:250` (aplica imagens no tique); worker RECUSA a cópia sem folga (~170 recusas/imagem a 4K) · véu de humidade se Show Wet · máscara viva ⇒ upload FULL todo quadro ⚠️ · adormecer: `wet_adormece_ou_acaba` `tp/wetpaint/adormecida.rs:253` ✔ (zstd; acordar O(W·H) 1×)

### Ctrl+Z / Ctrl+Y
- `undo_last` `tl/runtime.rs:48` ✔ → `undo_last_corpo :54` ✔ (`flush_shape_txn` → `absorb_foreign_writes_now` → `snapshot_model` → `undo.peek_confined_region` → `wet_solta_para_o_undo` → `crates/ph2d-tool-painter/src/undo.rs:457 undo` → `restore_model_confined` `tl/layers/undo.rs:110` ✔ → `wet_volta_do_undo`); `redo_last :114` ✔ (`undo.rs:483`)
- ⚠️ Digital/Impasto: Ctrl+Z re-dobra a tela; pior caso dispatch 71,2 ms a 4096² (F15-09); região confinada = janela; se não confinado ⇒ cache composto derrubado ⇒ quadro seguinte Full ⚠️ O(TELA·N) (Impasto: luz serial ~0,9 s se GPU recusa); troca GPU→CPU zera cache (`tl/layers/preview.rs:120`, doc)
- Wet: undo vivo por registo + foto em tiles f32 (`agua_antes` da caixa); recompor tela inteira após undo com células enterradas foi recusado (RC-56)

### Trocar camada
- `select_layer` `tl/layers/mutate.rs:511` ✔ → `set_active_layer :461` ✔: `snapshot_model` (undo_before) → `flush_active_to_images` (⚠️ copia a camada ativa, W×H×4) → remove/aloca buffer da nova (⚠️ `vec![0u8; w*h*4]` se vazia) → `replace_canvas` → `commit_structural_edit` → `wet_acorda_da_camada` (⚠️ descomprime zstd e compara com a tela, O(W·H)) → `invalidate_composite` (`tl/layers/cache.rs:28` ✔: derruba `composited` e `dirty_rect` ⇒ ⚠️ próximo quadro = recomposição COMPLETA)

### Slider de luz / material (Impasto)
- `set_impasto_light_angle/elevation/intensity/color` `tp/impasto_settings.rs:381,387,393,400` ✔ e `set_impasto_shine :451` ✔ → `invalidate_composite()` ⇒ ⚠️ quadro seguinte Full: pista GPU recompõe a TELA + ImpastoLightPass `Region::full` + premul + cópia + mips da TELA; na CPU `apply_impasto_light` serial
- Slider Body/Depth/Smoothing: `refresh_live_relief` `impasto_live.rs:426` ✔ → `rebuild_live_layer_relief :196` ✔ + `invalidate_composite` ⚠️ (225 ms a 4096² medido para o equivalente em `sync_relief_flags :396`, que foi curado — este ainda paga)
- Wet: `Gaussiano/Nitidez` no relevo ⇒ `AdjustWindow::full` em `impasto_light.rs:270-283` ⚠️ (regressão RC-23)

### Abrir documento 4K
- `bind_document` `ap/painter_bridge_phases.rs:112` ✔ → `tl/documents.rs:111 bind_document` ✔ → `bind_document_corpo :130` ✔ (guarda o doc anterior via `stash_current_doc`; `restore_doc` ou `abre_a_sprite_nova`) · `prewarm` `ap/painter_gpu_preview.rs:46` ✔ (compila 3 pipelines, tirou 28 ms do 1.º traço)
- ⚠️ semente: 1.º quadro = `drive(...seed_full=true)` na TELA / upload FULL (clone 33 MB + premul W×H + upload 33 MB) + mips + 1.º composite Full; memória típica ~100 MB RAM + ~44 MB VRAM por camada Digital; Impasto +250–315 MB; Watercolor ~265 MB; Wet 1,5–2,7 GB/camada

### Duplicar / apagar camada
- `duplicate_layer` `tl/layers/mutate.rs:200` ✔: `snapshot_model` → `flush_active_to_images` ⚠️ → clone de `images[id].rgba8` (⚠️ W×H×4, 33 MB) → `layers.duplicate` → copia `vidros` (7 B/px) e `tintas_molhadas` → invalida
- `delete_layer` `tl/layers/mutate.rs:163` ✔: `snapshot_model` → `layers.remove` → limpa `images` das mortas → `invalidate_composite` ⚠️ (recomposição completa); camada base é permanente
- ambos entram no undo como `ModelSnapshot` (snapshot de planos por `Arc::ptr_eq`, sem cópia quando não mudaram)

### Tabela de escala (§2.4): o que cresce com a TELA
Cópia no 1.º toque (todos) · planos do meio nascem (Watercolor, Impasto, Wet) · pilha ≥ 2 camadas e Impasto 1 camada (placa) · luz do relevo · upload com máscara viva/seed/Full · mips do slot (todos, por escrita) · Full CPU 133 MB · Impasto luz serial ~0,9 s. Cresce só com a JANELA: painel+chrome+apresentação (todo quadro).

## DOCS A ATUALIZAR (§14.c)

Docs em `docs/` (caminho · o quê):
1. `docs/Painter/25_avaliacao_gpu.md` §7 (332-339): diz Onda 3 = depósito/bake/wash do Impasto/Watercolor em compute; vale `26` §6 e `33`: NÃO portar o depósito, pista GPU perdeu (2,61×2,20), só ganha com redundância ≥ 4 (`MIN_REDUNDANCY`).
2. `25_avaliacao_gpu.md` §3–4: diz que a placa recusa máscara e clipping e teto de 8 camadas a 4K; código: máscara/clipping são ops (só recusa GRUPO mascarado e ajuste espacial mascarado), 16/32 camadas a 3840×2160 (`compositor/gpu_ops.rs:19,95-133`; `layer_compositor/mod.rs:123,154`).
3. `25_avaliacao_gpu.md:91`: tique ocioso do Wet 16,1 ms a 4096² («o pior achado») — curado: ~70 µs (`wetpaint/offthread.rs:58-83`).
4. `25_avaliacao_gpu.md` §8 (353-356): «Procreate é GPU-residente» — `26` §2: teto depende da RAM.
5. `docs/architecture/decisions/` ADR-0146 «PROPOSTA… construção não começou»: `ph2d-wet-gpu` existe (7 011 linhas) e liga com `PH2D_WET_GPU=1`.
6. `docs/Painter/49_realshader/ESTADO_REBELLE_MEDIDO.md:79`: «placa por omissão é a única que passa o Rebelle (9,24 ms)» — 9,24 ms é FORA do app, estado residente; no app Radeon 7,5–10× mais lenta; RTX nunca medida; K–M de fábrica manda 53/56 pincéis à CPU.
7. `49_realshader/*` leitura implícita «a placa do Wet resolve o 4K do Wet»: a placa só anda a água DEPOIS do traço (`wet_diag.rs:323-331`; `offthread_laco.rs:116`).
8. `docs/Painter/44_a_fila_depois_da_linha.md:315-330`: fork da tela 3,26 ms «fecha por medição» — BUGS #72b: 21–27 ms no app.
9. `docs/Painter/32_aquarela_o_que_custa_hoje.md:31` (e `BUGS_painter.md:653`): «dono pintou 4096 com fluidez» — Watercolor Basic de hoje (com Rewet) não cabe nos 60 Hz nem a 1080p (20–22 ms); sonda do estudo: p50 17–24 ms e pen-up ~180 ms a 3840×2160 (load 20–77).
10. `31_handoff_watercolor_perf.md:176-181` §2: «não é problema de performance hoje» (sonda por MOVE) — D32 achou custos por QUADRO.
11. `43_a_pilha_do_composite_brush_ficou_rapida.md:300-307`: decidir x86-64-v3; o v2 foi adotado, v3 recusado (`.cargo/config.toml:15-27`).
12. `49_realshader/B_nucleos.md:40-54`: secagem 46–55 % do passo — depois dos cortes advecção 56 %, secagem ~14 % (`MEDIDA_31_10_RODADA6.md:8-45`).
13. `49_realshader/MEDIDA_31_10_RODADA5.md:281-283`: `codegen-units=1` no `ph2d-wet-paint` «fica ao integrador» — já aplicado (`Cargo.toml:207-212`); rodadas 3–5 (cu=16) não se comparam com 6–9 (cu=1).
14. `49_realshader/` compara com Rebelle a 4096×2560 — foi medido a 3840×2160; dono usa 3840×2160 (`D_rebelle_4k.md:7`).
15. `26_plano_performance_procreate.md` §4 (plano T→L→U→R): T revertido, U1 não cura o pen-down (§7); §9.6 projeta 31 %, §9.6.5 mede 18 %.
16. `28_otimizacoes_o_que_funcionou.md` §4.1→§4.8.2: quatro explicações sucessivas do atraso do 1.º traço — só a última vale (fold de 201 ms).
17. `30_plano_multiresolucao.md` topo «as seis fases fecharam» × tabela F2–F6 em branco; `29_offthread_sim.md:3-5` «pendente de smoke» — fechadas; smoke feito (`A28:2196`).
18. `29_offthread_sim.md:127`: smoke do dono é `--release` — CLAUDE.md §2: smoke = `--profile smoke`.
19. Handoff Onda 1 da tinta viva (577 MB): eram 5 081 MB (`O2:31`); descartar números de memória do Wet anteriores a 06/11.
20. `BUGS_painter.md:709` «#70 CURADO»; `O2:64` «Wet Round resolvido»; `TV:44` «`enterra_celulas` só retém a caixa» — restos do #70 abertos (B:715, O2:25); Wet Round NÃO curado (O2:203, TV:66); memória do 1.º traço NÃO feita (+117 MiB, MP:157).
21. `crates/ph2d-tool-painter/src/lib.rs:356-359` (`MANIFEST.memory_budget` «~64 MB 4K» = uma camada RGBA8 a 4096², não o custo real) e `layer_compositor/mod.rs:150-154`, `undo_shape_state.rs:146-149` (orçamentos «do dispositivo» são literais).
22. `ESTUDO_CUSTO_WET_PAINT_2026-11-07.md:180/200` e `UR:86-88,240`: aguada parada mostra 0–1 imagem/s «porque o compositor espera no mesmo pool» — o worker RECUSA a cópia antes de pedir a imagem (`offthread_otica.rs:306-311`); separar pools não muda a taxa.
23. `docs/perf/mouse-stutter.md:66-92`: Fifo por omissão porque «não há Mailbox» — era Metal; no Linux/KWin há Mailbox e `best_nonblocking_mode` prefere Immediate (`ph2d-gpu/src/surface.rs:162-165`).
24. `25_avaliacao_gpu.md:419-424` §10.6: seis ajustes sem porta GPU continuam abertos; Color Balance sem Preserve Lum. já é Curves (`gpu_codes.rs:20-45`).

Comentários de código que contradizem o código (corrigir junto): `height_film_lut.rs:233` («a crate não tem rayon» — tem, `ph2d-painter-brush/Cargo.toml:27-49`) · `ph2d-wet-paint/src/par.rs:47-49`, `painter/espelho.rs:5` («rayon exige ADR») · `wetpaint/composite.rs:292-297` («solver serial por semântica», agora Jacobi+rayon) · `wetpaint/offthread_otica.rs:29` («48 ms voltam em 4 passos» só com folga) · `wetpaint/offthread.rs:13-18,60-80,141` (tabela do `TICK_WAIT`, regime velho) · `wetpaint.rs:353,366,377,416,520` (comentários colados ao `mod` errado) · `wetpaint/offthread/placa.rs:171-177` («omissão passa a placa quando verde») · `ph2d-wet-gpu/src/passo.rs:14-33` (`placa_sabe`/`EdgeSoftMass`) · `watercolor_render.rs:20-21` («no transcendental in the hot loop» × `aparencia.rs:369` 3 `powf`) · `ph2d-painter-brush/src/blur_peso.rs:86-89,228-229,263-266` (premultiplicado × recto) · `ph2d-painter-effects/src/adjustments/spatial.rs:17` («sem chamador» — `compose.rs:554`) · `impasto_light.rs:148-152` × `:270-283` · `warp/field.rs:203-206,340-345` × `warp/apply.rs:535-560` · `substrate_relief.rs:412-415` · `ph2d-tool-painter/src/tool/mod.rs:10-13` («motor apagado») · `brush_core_settings.rs:134-154`, `ph2d-brush-presets/src/modelo.rs:74-79`, `substrate_smoke.rs:171-174` (raio × diâmetro) · `watercolor_reserve.rs:263` (`RESERVE_R_MAX` 1024 é DIÂMETRO) · `ph2d-wet-paint/src/grid/material.rs:2`, `ph2d-painter-brush/src/stroke/walk.rs:35-37`, `material.rs:50,309` (32 B/52 B/«4 bytes» → 7 B) · `material.rs:112-118` × `spec_default.rs:224` (`Material::NEUTRAL`) · `wetpaint_settings.rs:178-184,236-237` («Enio's five» → sete) · `solver/gota.rs:23,51` · `smooth_edges/diag_w3b.rs:297-299` · `taper.rs:18-47`/`paint_taper.rs:235-238` · `paint_wetpaint.rs:496`, `paint_stroke.rs:3-4`, `paint_shape_layers.rs:38-39`, `paint_rows_relief.rs:643-646` · `paint_impasto_rig.rs:13-14` · `paint_layer_list.rs:32-33` · `state.rs:532-534` × `:74` · `spec.rs:39,353` × `ids/painter_shape.rs:252` · `sculpt.rs:106,132-137,190-199,245-247,306-319` · cabeçalhos de smokes/sondas com `--release` (`thread_probe.rs:8` e outros; guarda R8 recusa).
Nota: `PH2D_PAPEL_SEM_DEGRAU` (memória/`BUGS_painter.md:668`) não existe no código desta linha (git grep só acha docs).
