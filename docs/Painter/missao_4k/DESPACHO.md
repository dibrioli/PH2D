# Despacho da janela principal (missão Painter 4K) — o que já foi lançado

> A janela principal só decide e confere. Se a conversa compactar: este ficheiro + `git log` + os estados do
> [`BACKLOG.md`](BACKLOG.md) e do [`BACKLOG_AUDITORIA.md`](BACKLOG_AUDITORIA.md) são a verdade.

| hora | o quê | resultado |
|---|---|---|
| 07:10 | 11 extratores dos estudos → `fontes/` | feito (`5693d690a4`) |
| 07:15 | medição de base no app (B08 «antes») | feito (`2bf12269c1`), parcial: sem fundo branco (falta `PH2D_COMPOSITE_FUNDO`), blocos só 1024², Real Oil 1440 por medir, trocar de camada por medir. **Placa: forçar a Radeon integrada** (sob Xvfb o app abria a RTX) |
| 07:20 | BACKLOG único: 47 blocos, 313 itens, 12 ondas | feito (`5693d690a4`) |
| 07:22 | pente fino AU01–AU15 (só leitura) | feito (`7e11fbcb6f`) |
| 07:47 | achados AU → `BACKLOG_AUDITORIA.md` | em curso |
| 07:48 | **onda 1:** B01 camadas · B03 traço preso fora da janela · B07 réguas/lacunas · B11 Wet a tempo + espelho · diagnóstico do quadro de ~100 ms comum a 4K e do CPU parado 48–70 % | B01 B03 B07 B11 em curso · diagnóstico feito (`1569fcc8ab`) |
| 07:51 | achados AU → `BACKLOG_AUDITORIA.md` | feito (`29c21ea0ea`): 74 achados, B48–B52 novos |
| 08:00 | **B10.3** (folha ao vivo só à vista) + B10.2 (plano do dente em cache) — promovidos pelo diagnóstico | em curso |
| 08:12 | B11 1.ª volta | parcial (`1c45478ab4`): agenda pura, gate `a_agua_no_tecto_recebe_imagem_a_tempo`; continuação lançada (B11.2 espelho estreito, B11.3/4, AU02-2/3, medir no app) |
| 08:13 | B01 | feito salvo B01.5 (`5651d764d4`); B01.5 → agente do B05; poda das adormecidas → B02 |
| 08:14 | **onda 2:** B02 tinta Wet presa ao documento · B04 aguada 256 · B05 ajustes/undo + B01.5 | em curso |
| 08:25 | B03 | feito salvo AU-AU13-7 (`a5b636178c`) |
| 08:27 | **Workflow `painter-4k-missao`** (run `wf_d8a3ff7d-587`): ondas adaptativas (planear → executar com 2 continuações → verificar/curar → registar aqui) até o BACKLOG fechar, depois o FECHO (matriz depois, pranchas, gates, mutação, docs, handoff, smoke). Script: `~/.claude/projects/-home-enio-Documentos-Projetos-PH2D/5b44abf9-…/workflows/scripts/painter-4k-missao-wf_d8a3ff7d-587.js`; retomar com `resumeFromRunId` | em curso |
| 08:40 | B11-cont | B11.2 feito (`292af7d534`): cópia do espelho 30,35 → 3,89 ms a 4K, ao bit; falta B11.A2 (recorte do véu `col0`), AU02-2/3, medir no app, B11.3–5 |
| 08:42 | B10 | B10.3 feito (`abc82a8fa3`): Digital 4K toque/+1 s/Ctrl+Z 98/102/99 → 31/26/31 ms; Impasto ainda 56/82/106 (outro custo: perf `omissão:impasto`); falta B10.2 (plano do dente, oráculo `o_plano_do_dente_e_o_dente_amostrado_ao_bit`) e B10.5 |
| 08:44 | B05 | quase feito (`966e12940e`): `ajuste_mudou` porta única, arrasto = 1 passo, censo de 28 setters; B01.5 curado mas o verde por confirmar (`cargo-test-narrow.sh ph2d-tool-painter camadas_integridade_tests`); falta B05.3 (12 setters de `texture.rs`) |
| 08:46 | **B02 — ⚠️ POR COMMITAR NA ÁRVORE** | cura + gate escritos, verde por confirmar (a crate não compilava pelos `watercolor_*` do B04 em curso). Ficheiros: `tool/paint/wetpaint/adormecida.rs`, `…/adormecida/enterro.rs` (novo), `…/adormecida/documento_tests.rs` (novo), `tool/documents.rs`, `tool/paint.rs`, `tool/layers/undo.rs`, `tool/layers/estrutura.rs`, `tool/paint/camadas_integridade_tests.rs`, BACKLOG B02.1. Próximo: quando compilar, `cargo-test-narrow.sh ph2d-tool-painter adormecida` + `camadas_integridade` + `documents`; mutação: tirar `wet_poda_as_adormecidas` do `estrutura.rs` ⇒ vermelho; commit por caminho; depois B02.2–5, B02.A1–A3 |
| 08:58 | B07 | 5/7 feitos (`827b9e883b`); tecto do custo do Wet Wash Mix DESCEU 1,3 → 0,75 (produto 0,55–0,60; rota dobrada 0,89–1,12) — ⚠️ medido só no perfil dev: o FECHO confere no perfil `ci-test` (o da integração); falta B07.3 (contadores do véu 4K) e B07.7 (censo de `elapsed()` em `assert!`) |
| depois de B03 | **B23.2** quadro a pedido (`main.rs:1021` `ControlFlow::Poll` + `present_title.rs:66` `request_redraw` incondicional) — mesmo `main.rs` que o B03 | por lançar |
| onda 1 | B23 | parcial (a4cca61a4c): B23.2 passo 2, a lei `quadro::Laco` (foco, entrada, smoke nunca dorme) e `quer_quadro` do Painter; gates vermelhos sem os produtores, 5/5 verdes. Falta ligar na shell (tecto 189 041) e B23.3 |
| onda 1 | B48 | feito (2b6f72bcb8): o tique do balde reusa o scratch e custa pela região, não pela tela; 4096² 11,3→3,2 ms, t4096/t2048 ×0,95–1,09; 90 080 B por tique nos dois tamanhos; 65 fill e 105 selection verdes |
| onda 1 | B06a | feito (13f483823f): B06.1 com Wet, tinta viva e morta; altura e cobertura escalam pela razão alfa; red 105,97 → ≤ 1 %; 3 gates novos verdes, borracha 79/79, alarmes verdes; BUGS #70 curado |
| onda 1 | B08b | parcial (ce62ec168d): B08.1 feito, a matriz mede grupo, luz do Impasto e abrir doc; régua SIGSTOP 250 ms red→green; 1080p branco Round Mixing p95 84→12,9. Falta B08.2 (10 pincéis lentos, Oil Wash B36) |
| onda 1 | B14a | parcial (e8d383f451): sonda `sonda_a_caixa_do_passo_e_a_que_mudou` acha o mecanismo (`step_simulation` suja a caixa do fluido com 0 células mudadas); sem cura. Falta o corte em painter.rs, composite_otica.rs e o gate |
| onda 1 | B17a | feito (ac36447bef): o collect do par_clone copia em blocos de 256 KiB; tela 4K 7,72→1,67 ms, 4096² 3,43, u8 4K 0,97→0,29; limiares → 7,5 MiB; gates red→green, igualdade ao bit verde |
| onda 1 | verificação | 3 alvos sem prova: `o_tique_do_balde_a_4k_e_dez_vezes_o_antigo` e `sonda_o_corpo_sob_a_borracha_no_wet` não existem (0 correram); `prancha_b06a_a_borracha_leva_o_corpo` é só `#[ignore]` |
| onda 2 | B02-cont | parcial (3ea1c16e2f): B02.5: o registo de cada passo guarda a caixa dos seus dabs; desfazer além do tecto enterra só ali (+32). Gate casco_tests: sem a caixa 55 694 valores longe a zero (red) → 0. Falta B02.2, B02.A3 e a prancha. |
| onda 2 | B41a | feito (5454a67f2c): Inpaint: se a placa falha num nível, o pedido todo passa ao inpaint_cpu, igual ao byte (antes 4 expect fechavam o programa). diag → stamp_banded/diag.rs, 636→506 linhas. Red por injeção nos 5 → green. |
| onda 2 | B23-cont | parcial (5a5aac2f52): catraca painter 24 760→24 382 (tecto 24 429); novos produtores (voo do Morph, som, folha devida); shell 189 054→189 009. CPU parado: pedido 3 % × contínuo 57–70 %. Falta janela oculta não apresentar. |
| onda 2 | B10a+B15.6 | parcial (8d5fadff95): Ctrl+Z 4K no app (Água, Radeon): ciclos 1920/1714/1981→904/955 M (−52 %), toque 4437/3835/4341→3665/3521 M. Ao bit: 4 gates, 2 mutações. Falta a libc (14–20 % do quadro): reusar ImpastoPlanes. |
| onda 2 | B14a-cont | parcial (03510a42ef): medido no app (2880²): 12,0→0,8 img/s, óptica 60→953 ms; a caixa mudada é 0,92 da união (faixas anteriores seguem vivas). Falta a óptica por várias caixas (caixa_do_passo.rs, composite_otica.rs). |
| onda 2 | B16a+B16.9 | parcial (7c4bd3cc7a): B16.9: pen-down repõe o NaN do substrato só na caixa preenchida (4K, 3 camadas 1152,8→1132,4 ms); B16.A2: chão uniforme guardado por cor e tamanho. Gates red→green. Falta partir apply_watercolor_inner e B16.2. |
| onda 2 | verificação | verde |
| onda 3 | B02-cont2 | feito (0c4c98f0dd): B02.A3 cont2: o mudou vem só da caixa que a óptica reescreve; wet_congela reusa a congelada; base em par_chunks_mut. Visitas 77 473 → 12 100; paridade ao bit; mutação vermelha. |
| onda 3 | B23-cont2 | feito (1c8b183d14): régua anterior errada (o salta apresentava). Cura: acorda dá SEM_PRESENT (16,67 ms). Oculta: CPU 97–100 % → 44–46 %, 60 q/s; gate 250 → 60; shell −1. B23.6 aberto: winit sem Occluded no Wayland. |
| onda 3 | B10c | parcial (e8dc100b53): B10.5: leitura só pela caixa depois do 2.º traço, nos 4 meios; pixel_clock na chave; 2 mutações vermelhas. Falta medir a sonda em ms, o Ctrl+Z do Impasto e partir compoe_a_saida. |
| onda 3 | B16-cont2 | parcial (3a8b2654c7): B16.2: pen-up da aguada 4K 48,48 → 1,95 ms (−96 %), texels 4 657 444 → 48 884. Falta curar o dono do halo (style_at) com prancha e alarmes, e tirar o guarda de estilo uniforme. |
| onda 3 | B14-cont2 | parcial (52db80f6cd): só medição: óptica 59 → 457 ms por faixa viva; margem não é a causa (85–88 % das células mudam ao bit, 18 % em u8); limiar recusado. Falta B14.2 (luz simplificada no gesto) e listas de faixas. |
| onda 3 | B15-cont | feito (38cd706fcd): B15.A1: box_blur do settle com 1 buffer, ladrilhos 64×64, ao bit; sonda 4K r4 30,7–43,5 → 13,8–22,2 ms. Planos 34,7 → 18,7 ms; dobra do Ctrl+Z 12,3 → 9,9 ms. Resta o undo não confinado. |
| onda 3 | verificação | verde |
| onda 4 | B24 | feito (2528829629): B24.1/B24.2 prancha (709c41f60a): borda escura do Blur em camada transparente some. B24.5: Caixa r=512 9,0→9,9 ms/dab, Binomial 90→498; tecto fica no binomial. Gate 38 colunas, verde; alarmes verdes. |
| onda 4 | B52.R3+B19.11 | feito (48adf8c0b9): blend_window pula o apply de camada Normal com α≤0 quando o resultado é o mesmo bit; gate das faixas refeito (Noise + controlo). 4K vazias 38,1→19,2 ms; traços 34,3→16,9; ruído branco sem ganho. |
| onda 4 | B10c-cont | parcial (a2a5f9302c): R4 sem mudar comportamento: undo.rs 692→633 (undo_consulta.rs), undo_delta.rs 693→564 (undo_delta_diff.rs, sha256 igual). Verdes por nome. Falta confinar Impasto e Wet e medir a sonda. |
| onda 4 | B14b | parcial (c77035d6f0): prancha gesto × repouso em pranchas/B14b (Real Oil, luz 12°): ~25 % dos píxeis diferem na tinta, média 10,6–12,5 níveis; repouso igual à Cheia ao byte. Falta medir no app (load 29–38). |
| onda 4 | B16c | parcial (4584d861ef): B16.12 só diagnóstico: toque do papel 154,5 ms (composição CPU 94,7 + envio 18,7 + Down 22,5); a 1.ª aguada cria o vidro e passa o produtor à CPU. Branco/blocos 41 ms. Cura por fazer. |
| onda 4 | B08b-cont | feito (a3821cf1a0): B08.2 completo: os 10 pincéis lentos nos 3 fundos que faltavam, matriz 10 × 6 = 60/60, mesmo binário, Radeon, load 26–44. Nenhum dos 10 passa em fundo nenhum. Sem mudar desenho nem gate. |
| onda 4 | verificação | verde |
| onda 5 | B23-cont3 | parcial (0692ff3bab): B23.6 diagnóstico: ph2d-gpu diz o adaptador e avisa se o dispositivo se perde; a causa do pânico no KWin virtual é a WSI. Falta compositor headless com GL; q/s minimizado não medido. |
| onda 5 | B10c-cont2 | feito (34a963a3df): Ctrl+Z do 1.º traço Wet confinado à janela do traço (confined_region por camada); invalidate_composite põe gpu_lane_stale. Gate red→green, 7/7 + 187 vizinhos verdes. App 4K não medido. |
| onda 5 | B16c-cont | feito (db4ce86030): papel_pede_a_cpu: CPU desde o armar do Watercolor sobre papel; a placa que larga o documento faz a CPU compor a tela. App 4K: toque 186,3 → 32,0 ms (alvo ≤41). Gates red→green. |
| onda 5 | B16-cont3 | feito (060b3fe257): B16.2 cont3: watercolor_render/dono.rs dá ao texel sem dono o mais próximo até pad; style_at e o desloca leem-no. Sonda 4K raio misto 118,65 → 3,14 ms. Prancha: 0 píxeis mudam. |
| onda 5 | B12a | parcial (58f6a1cde7): B12.4 veneno do rascunho do solver (cfg(test)): grade igual ao bit à do rascunho limpo; mutação vermelha, sem ela verde. Falta B12.1 marca_dab no pouso, B12.3 tile esparso, B52.R11. |
| onda 5 | B14-cont3 | feito (f00ea17138): B14.4 o emboss da tinta sem óleo segue o Light angle do cartão (Sim::luz_do_relevo); 225° igual ao bit. Gate o_emboss_segue_a_luz_do_cartao red→green; mutação red. Alarmes verdes. |
| onda 5 | verificação | verde |
| onda 6 | B11-cont2 | feito (ffc2605130): B11.A1: worker morto em pânico → perde_o_motor nas 3 portas, motor vazio e SimWorker.perdido; a sessão acaba sem tocar a tela. Gate worker_morto_acaba_a_sessao_sem_tocar_a_tela red→green; regressão verde. |
| onda 6 | B29a | feito (f981cf4a89): R26 parte: bandas do carimbo movidas p/ stamp_color/bandas.rs (accumulate_batch 510→336), ao bit; A2 nível da pirâmide por carimbo; A3 GraoCache (serial 1200→300). Alarmes verdes. |
| onda 6 | B14c | parcial (1eddf51ce4): silhueta base da ponta guardada à parte (SilMemo::nova_base): Flat Bristle 240 a 4K, 7,11 M → 1,89 M avaliações por dab, ao bit; gate red→green. Falta A/B no app, caixa do memo, pow, Drag. |
| onda 6 | B10c-cont3 | recusa medida (3614bf29ea): só BACKLOG. B10.4 Wet 2880² na Radeon: quadro por relógio de parede sem forma comum entre rodadas; CPU por janela constante, sem acúmulo. Sem defeito nem gate; B10.5 já feito. |
| onda 6 | B06b | parcial (473854fefa): B06.A5: risca da malha curada (homografias em coordenadas locais; folga em distância 1e-3 texel); texels sem dono 3 → 0; 54 testes do warp verdes. B06.7 bloqueado; falta resto de relevo e estrias. |
| onda 6 | B23-cont4 | feito (aa64a921ee): B23.1 minflt por fase na linha [quadro]; shell 189 015 → 189 015. 4096²: 1 camada a traçar 1,6 faltas/quadro, p50 42 ms; 12 camadas 0,0, 16 ms. Gates verdes; commit levou linhas B06.2/B06.3 do BACKLOG. |
| onda 6 | verificação | verde |
| onda 7 | B15a | feito (e892702f1c): B15.5-cont: com Gaussiano/Nitidez, planos da placa só da janela cortada, ao bit; sonda 4K (load 56, só o sentido): planos 434 → 3,0 ms. B15.8: cobertura clampada 1×; gate red→green. |
| onda 7 | B17b | parcial (58d017b8d7): StoredPlane::changed_tiles guarda só os tiles que mudaram; undo de diagonal a 4K: Impasto 265,4 → 32,0 MB, Digital 10,8 → 8,4 MB. Gate novo. Falta B17.5 (recontar só tiles restaurados). |
| onda 7 | B12b | parcial (14cb07ed06): CampoEsparso<T> em ph2d-wet-paint (pedaços de 4096 células): dab r30 a 4K escreve ~26 MB contra 897 MB; 3 gates ao bit. AINDA não ligado à Grid; falta Ctrl+Z a 4K no app. |
| onda 7 | B14c-cont3 | parcial (543e7ca42e): A/B no mesmo binário do corte da silhueta, Flat Bristle 240 a 4K: 562,4 → 534,7 G instruções (−4,9 %), hash igual. A silhueta já não é a alavanca: ~89 G/dab fora dela; falta perfilar. |
| onda 7 | B16b | parcial (12ddb9b8cf): RewetCache com planos esparsos por ladrilho 64²: 3,1 MB contra 201 MB (Spread 12); 4096²: 37,8 contra 738 MB. Mas o traço fica mais lento (9,1–10,0 → 12,6–12,8 ms); falta caminho híbrido. |
| onda 7 | B14b-cont | parcial (11a7100349): Dabber na cena Composite já liga o Real Oil, sem .rs; wet6 ganhou gesto longo e --ciclos a tabela. A × B, load 10–28: nenhum ganho decisivo. Falta log da óptica por quadro (.rs). |
| onda 7 | verificação | verde |
| onda 8 | B15b | feito (fb010927be): Placa: o knob da lâmpada não redobra a tela; dobra do tique 4K 21,6 → 0,01 ms (load 18–26, só o sentido). Gate novo red → green, mutações vermelhas; Radeon verde. |
| onda 8 | B12a-cont | parcial (070d3480dc): B12.1 anel: a grade nova nasce com o anel de dreno selado; gate do undo do 1.º traço red → green, 20+37 verdes. Falta pen-down Wet na Radeon e B12.2 (Ctrl+Z Wet 4K). |
| onda 8 | B17b-cont | parcial (7f7c1813a3): 2.ª cópia de tela do Ctrl+Z atribuída (undo_journal.rs:180) e removida: Digital undo 11,6 → 7,8 ms, Impasto 20,0 → 12,6; cópias 2 → 1 / 5 → 4. Falta B17.4 (dono único da tela). |
| onda 8 | B17c | feito (d6c5c94513): B17.6: Blur e carimbo de cor assado passam a pegada própria; journal 512² Tiling 1 048 576 → 262 144 B. Dois gates novos red → green; alarmes de pincel verdes. Ganho só em debug/test. |
| onda 8 | B11-cont3 | parcial (84d9650d36): B11.3 passo 2: o compositor da óptica corre o laço de píxel e o fio da tela só cola (sem Real Oil, razão 1). otica_no_tique 10/10. Nada medido no app; falta Smooth Edges/razão > 1. |
| onda 8 | B14b-cont2 | feito (4fce3b6b3b): Fecho medido no app, Radeon: luz Simples × Cheia −13 % / −9 % em ciclos de casa (load 14–44, contagem); óptica 0 imagens nos 27 gestos; aguada 4K 11–26 img/s. Sem .rs. |
| onda 8 | verificação | verde |
| onda 9 | B25a | feito (b078044bb7): B25.1: a_ordem_e_da_pilha_e_nao_da_taxa_do_rato e a_macro_da_o_mesmo_byte rodam a 1/2/32 fios, ao byte do serial; porta antiga diverge 270 bytes a 32 fios, a nova 0. Prancha B25.2: 0 píxeis mudam. |
| onda 9 | B40a | feito (c307063bd7): B52.R16: paste_patch.rs 687→286 linhas (compose, input), 13 testes verdes. B40.A1: guarda_da_escrita_direta no Color Fill, Cut e Colar; red 6/6 → 7/7 verdes pelo oráculo CAMINHO_ANTIGO. Prancha 12 PNG. |
| onda 9 | B17b-cont2 | parcial (5a5ab43746): cura B17.4: journal da tela em release, ModelSnapshot.canvas_elided; 3 donos → 1 no pen-down (gate at_pen_down_every_canvas_plane_has_one_strong_owner), undo ao byte. Falta medir 4K e a rede audit_journal (known=0). |
| onda 9 | B12b-cont | feito (8dbd53c477): Grid::aparencia em CampoEsparso (pedaços de 4096): tela 3840×600, dab r30 64,8 MB → ≤6,8 MB. Gate novo vermelho no Vec denso, verde no esparso; ao bit: historia, espelho, desfaz, caixa, w3b, Radeon. |
| onda 9 | B19a | parcial (9471ab7518): B19.2: janela suja sob ajuste espacial compõe a área alargada pelos halos; Bloom r>16 tela inteira; CA no modelo da placa. janela_tests vermelho 32/32 → verde. Falta B19.4 (Bloom cor>1 CPU×GPU); B19.5 schema. |
| onda 9 | B06c | parcial (71138ad644): B06.A2: casos Uniform+sel e Distort+sel; mutação li dx→rx red→green. B06.A4: levante 4096² 66,8–77,4 → 14,3–18,9 ms; begin_transform 17,9 ms (alvo 60). Falta a sonda do relevo e B06.A3 (apply.rs). |
| onda 9 | verificação | 1 alvo sem prova: `between_gestures_the_canvas_still_has_two_owners_and_this_is_its_number` não casou nenhum teste (0 correram); o nome não existe, só `at_pen_down_every_canvas_plane_has_one_strong_owner` (undo.rs:377), verde. |
| onda 10 | B16b-cont | feito (b1e5eda70b): B16.7: o arrasto de seleção só reescreve a máscara e o Up invalida 1× (antes, tela recomposta por Move). Gate 8 Moves×4 modos: 8 recomposições → 0, máscara e prévia ao byte. |
| onda 10 | B19b | parcial (c25a32cfbb): arrasto de ajuste em faixas sem espacial, quadro reciclado no CompositorCache. 4K ABBA mín.: 1149,5→133,9 ms (load ~30), 611,0→90,1 (load ~13). Falta B19.6 barreira espacial e B19.12. |
| onda 10 | B12c | feito (9e1a58c783): os 4 vermelhos da óptica na tool-painter são carga (load 26–34), não o B12c: sozinho 23/23 e conjunto 92/92 no HEAD. B12.3 explicado, B12.4 feito. Só BACKLOG. |
| onda 10 | B20a | parcial (77b7014395): premul da prévia só na região tocada (run_region) e mips por retângulo (run_rect). Gates da Radeon vermelhos no caminho inteiro, verdes com a cura; ms não medidos. Falta interruptor e medir. |
| onda 10 | B17b-cont3 | bloqueado (1f4d83ec45): B17.5, 2 tiles opostos no 4K publicam 8 294 400 px sujos contra 30 720 restaurados (270×); número fixado em gate. Bloqueio: dirty é 1 rect, leitor é a lane GPU (B20a) e dreno.rs alheio. |
| onda 10 | B08b-cont2 | feito (779d578163): medido a 4K na Radeon (load 10–81, só tecto). W9 p95 308→380 ms; W14 30→53 ms; W1, W7, W13 sem custo; W17 sem veredito (load 80). Só medição, sem gate. |
| onda 10 | verificação | verde. |
| onda 11 | B41b | feito (6fd9e4b23d): B41.5 fechado: macro_estado_aplica lê as texturas antes de escrever; textura ilegível dá ErroDoEstado::Texturas e a ferramenta fica intacta. Gate red→green, 9 testes macro_estado verdes. |
| onda 11 | B11-cont4 | feito (e1a29ca661): B11.5 papel trocado com a sessão viva suja espelho e tela (+0,8 ms a 4K). B12.A1/B11.6: motor longe, o tique pede sem esperar; gate de relógio 4,03 ms → 620 ns por quadro. |
| onda 11 | B16d | parcial (03a854ee0c): B16.6 `aparencia` lê lut.s2l[k] (mesmo bit); 4096² r100 move+tique 3,530 → 3,429 ms. 2 gates red→green, alarmes verdes. Falta: tabela de hash2 por composite (ganho < 0,1 ms = recusa). |
| onda 11 | B25b | feito (d338b49547): B52.R12 grupo Grain/papel saiu de dab_spec (679 → 486 linhas, verbatim). B25.4 porta lê o pincel passado (red→green). B25.3 aguada limpa dab_groups (468 → 0 bytes). Alarmes verdes. |
| onda 11 | B19b-cont | feito (36a8563534): B19.7 resto: seis passadas em faixas passam do SO ao pool rayon; gate de fios 72/128/96 → 0, mutação vermelha, ao bit verde. Ms não medidos (load 20–38). |
| onda 11 | B20a-cont | feito (92460d48e1): interruptor PH2D_PREVIA_INTEIRA=1. Quadro p50 4096² 1 camada 38,8→10,2 ms, 4 camadas 47,4→10,6. Pen-up 4K: toque 32,6→16,0, traço 32,0→14,6, Ctrl+Z 38,1→17,5 ms. |
| onda 11 | verificação | verde. |
| onda 12 | B35.A1+B50a | feito (a2cc8fc1f2): B52.R5 fecha o bloco: json.rs + meio.rs viram UMA tabela Campo{nome, ler, escrever, grava}. Paridade ao byte (67 757 B, 4 pincéis). Gates e 2 alarmes verdes. |
| onda 12 | B40a | feito (d9de1bf431): B40.2 Trim só com Offset ≠ 0; B40.7 tecto da captura max(4096, 2(w+h)); B40.A2 Feather em janela deslizante nas faixas do pool, 4096² r=64: 11 121 → 16,8 ms, mesmo byte. |
| onda 12 | B13a | parcial (14d244c0ae): B13.2 medido sem cura: vão seco custa 5,9 ms/quadro (88,6 vs 17,8 ms, 5,9× células). Vale a cura (spans da máscara de tiles 64²); código do produto intacto. |
| onda 12 | B17b-cont4 | parcial (31babe408d): Ctrl+Z diagonal Impasto 4K na Radeon, A×B em 3 rodadas: 50,2 → 26,9 ms (−46 %); toque 27,3 → 13,9. Só medição; falta B20.7 (sujo por união, gate a_textura_e_o_espelho). |
| onda 12 | B49 | bloqueado (cf846e49de): passo 2 do B49.1 (pó por ladrilho 256²) mexe em stamp_cache.rs e stamp_ramp_routes.rs, com mudanças do B26a por commitar. Pen-down 4K já 12,6 → 5,8 ms. Planeador decide. |
| onda 12 | B26a | parcial (7d6ef9b53d): B26.A2 o t do dente do papel lido de memória própria do GraoCache: 0,149 → 0,045 ms/dab (3,31×), ao byte. Falta B26.1 (RLE da máscara) e a rota stamp.rs:644. |
| onda 12 | verificação | verde. |
| onda 13 | B13a-cont | parcial (9f9028a31b): secagem e sedimenta andam por span_runs, ao bit; longe 65,1 → 22,1 ms/12 quadros, vão 0,99 → 0,76. Falta a grade de fluxo (frow_tiles). Smear Wet já vermelho antes. |
| onda 13 | B17b-cont5+B20.7 | bloqueado (nenhum): último fad04b99be; verde não visto, pois dormente.rs:140 não compilava (campos novos do Grid). Falta correr a_textura_e_o_espelho e medir Ctrl+Z 4K. |
| onda 13 | B49-cont | feito (3f843b81b1): B49.1 fechado na crate real; pó por ladrilho 256², pen-down 4K Pastel 6,65 → 2,62 ms. Prancha B49-cont; alarmes de pincel vermelhos (46 presets), não regravados. |
| onda 13 | B33 | recusa medida (618996fd16): B33.2 recusado; o Wet do app usa o papel comum (sample_tiled_rot_wrapped), o ladrilho 512 só existe no motor isolado. Só linha do BACKLOG, sem código. |
| onda 13 | B35a | parcial (87034db1d0): B35.A3 feito; o cartão do Tuning não mede texto em quadro parado (cabe_memo). Gate o_tuning_nao_mede_texto_em_quadro_parado: 16 → 8 medidas. Falta B35.3, .5, .6, .7. |
| onda 13 | B12a-cont2 | recusa medida (d3b11719a0): só medição, 2.ª rodada A·B 4K. Ctrl+Z Wet: Round 54 → 30 ms, Wash 59 → 46; B12.2 feito. B12.1 recusado: bring_home/COW fora do topo do perfil. |
| onda 13 | verificação | verde: ph2d-painter-brush passou no check; o alvo ph2d-paint-brush do pedido não existe (erro de nome, não do código). |
| onda 14 | B17b-cont5+B20.7 | feito (628fac4546): sem código novo; dormente.rs compila desde 9f9028a31b; a_textura_e_o_espelho vermelho no fad04b99be, agora verde. Ctrl+Z diagonal Impasto 4K Radeon A/B: mín. 33,5 → 30,0 ms; a união não custa. |
| onda 14 | B13a-cont2 | parcial (88001a09fb): gate de tiles com cena de óleo; advect anda por span_runs (take(1) vermelho). Pânico de borda do backrun em rf>1 curado (serial e Jacobi), gate novo vermelho→verde. Não medido; falta a jan da ladeira_do_oleo. |
| onda 14 | B15.2 | feito (5d62bd7df8): sem código; remedido em 4K (load 30, só tectos): 1.º quadro de tela nova 376,9 → 39,9 ms; tique+dreno r200 18,76 → 1,5 ms; quadro p50 r200 21,7 → 4,4. Resto é um composite 4K, fora do bloco. B15.7 não começou. |
| onda 14 | B27 | parcial (d2e9c788c2): B27.3 afim/ECS por quadro: recusa medida. mesh_uv 4,2 µs/evento com 1 sprite, 34,9 com 1000; o carimbo custa 2,7–4,4 ms, no máximo 1 %. Sem cache. Falta medir INPUT/p95 no app com load < 5; B27.2 após o portão dos lotes. |
| onda 14 | B40b | feito (b6edd8dc8c): prancha em pranchas/B40b, 12 PNG (3 fundos, 1080p e 4K). Line com Mirror X: o 1.º dab agora espelha; Ellipse no Mask: o contorno protege. «Antes» por oráculos no mesmo processo. Gates verdes; devolveu 3 linhas do BACKLOG. |
| onda 14 | B22.2 | feito (5272823994): prancha em pranchas/B22.2, 3 fundos × 1080p/4K. 30, 60 e 144 q/s iguais à vista; a 30 a nuvem cresce na borda (240→285/s). Gate o_antes_a_30_qs_e_o_motor_novo_a_8_periodos_por_tique ao bit. Sem código de produção. |
| onda 14 | verificação | verde. |
| 11/11 06:43 | **janela pós-missão** (`e81e56bb1f..1d40807b12`, 82 commits): ondas 15–20 + fecho M1–M5/C1–C3/F1–F2. Onda atribuída pela ordem do `git log` | [handoff](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-11_POS_MISSAO_4K.md) |
| onda 15 | rustfmt + B06.A2 | feito (bfe7c348e5 · ddd54e93fa): 24 fontes do tool-painter formatadas; sonda do Transform com relevo 2048²/4096². |
| onda 15 | B17.5 D | parcial (9842534a52): upload da camada activa remenda só as caixas do Ctrl+Z (`LayerPixels.dirty_parts`); gate red→green. Falta A×B no app. |
| onda 15 | W15-A (mutação) | feito (58982865d9): `funde_as_caixas` acima do tecto e folga B25.1 fechados; os 2 sobreviventes sangram. |
| onda 15 | B13.2 cont7 | parcial (9edf92c364): a ladeira do óleo anda pelos tiles 64 da grade de fluxo, ao bit. |
| onda 15 | B35.3 · B35.5 | feito (5590ac584d · 2d4bf06535): linha com pai desligado esconde; uma só fábrica do Wet (`WetKnobs::fabrica`). |
| onda 16 | B36.1 Oil Wash | feito (74ffdd0c46 · efe15c5e63 · b8218ba570): preset novo pela lei E10g (Dilution 0,6), prancha B36. Largura sem aprovação (alarme). |
| onda 16 | B13.2 cont8 | feito (7d1a08a84e): memset do outflow e zero do project só nos troços dos tiles, ao bit. |
| onda 16 | B16d hash2 | feito (6962541dfc): warp do Ragged Edge lê os hash2 de uma grelha por composite. |
| onda 16 | B35.6 · B35.7 | feito (03df88ea8c · e079c342fd · 36416d5b36): `KnobDef::na_lei`; interruptores = caixa; Pickup/Edge/Gravity em %. |
| onda 16 | B16.8 | recusa medida (2186a13e01): a água da Dilution não é inerte (backrun); vermelho herdado com dois causadores aprovados. |
| onda 16 | B28.3 | parcial (ade7d9c95a): Color mixing r100 2048² 21,59 ms/q → pegada em paralelo + amostra nos pares; falta ~4 ms/q. Prancha B28.3. |
| onda 17 | B13.3 sono da água | parcial (dd0d231470 · b86207ad85 · 90f2e93bae): mecanismo e sonda; nasce DESLIGADO (ε=0 não é ao bit; não ganha a 4K). |
| onda 17 | B34.1 · B34.1b | parcial (602f277f35 · 551a86db79): fator de aparência do documento, PROJECT_SCHEMA 232→233; papel à escala. Falta sombra do óleo. |
| onda 17 | B29.3 | feito (d7f3af061a): sob a película do Impasto o grão é tecto; prancha B29. |
| onda 18 | B31.1 · B31.2 | B31.1 feito (c898d4d62a · 47ec5ad159): sem aro branco no papel; B31.2 parcial (d950e98544): realce sem anéis 15,8×→1,18×. |
| onda 18 | B34 relevo | feito (47ec5ad159): a luz lê declive × fator; dente 4K 4,561 → 9,318 (1080p 8,555). |
| onda 18 | B32.1 | feito (00f3ca1751 · 6eed7cffb6): Wet pela espectral de 10 bandas (`PH2D_WET_MISTURA=km` volta); 20 miniaturas mudam; prancha B32. |
| onda 18 | B35.8 | feito (d4e2700f0b · c8ead87301): ↺ e duplo clique voltam ao pincel que abriu. |
| onda 18 | B18.Y | feito (e1da1e9f35 · 102adcec00): 3 vermelhos herdados = mudanças de lei (B29.3 ×2, B33.1); nenhum gate afrouxado. |
| onda 18–20 | B29.1 tecto largo | feito (03c1d8cf0e … 9a7337ac68, 7 commits): u16 + recomposição; 10 pincéis mudam ≤ 4 níveis; prancha B29.1. |
| onda 19 | B38.1–.3 | feito (59b02eea28 · 25f15170ae): Clone/Heal pré-multiplicados; Heal placa = CPU ao byte (SSD u32, Q16). Prancha B38. |
| onda 19 | B32.2–.4 | feito (6b2b376e0b · b50b3731d3): uma lei de mistura no Digital/Impasto; pastel claro é tinta. |
| onda 19 | B35.9–.12 | feito (74f47da27e · fbb0aa9212 · afca5db3d5 · e512b49f27 · 553e222290): Erase Strength único; Base evaporation = 2; «Tooth fill»; Opaque/Paint drag/Emboss no cartão. |
| onda 19 | B39.1–.3 | feito (f3e0d71e13 · 5ffb92228b · ac93071017): Channel Mixer Monochrome, «−» do Falloff, seletor fecha com o Painter. |
| onda 19 | B42.1 · B42.2 | feito (9f505bba98 · 96e06618af · d308636a7e): 64 camadas 4K na integrada, orçamento ao medido; device pede os limites do adaptador. |
| onda 19 | B43.1 · B43.5 | feito (7c3fe5ca3d · 66f6034d67): `det_powi`; premissa «v3 contrai» corrigida no ADR-0174; portão do `RUSTFLAGS`. |
| onda 19 | tectos | feito (603abe3de9 · c4bab596fb): `budget.rs`, `pass_collect.rs`. |
| onda 20 | B39.4–.6 | B39.4/.5 feito (de290cd45e · d7939aa4ba · c4fd0cd418); B39.6 parcial (816daad4d9): prévia do papel = recorte do motor. |
| onda 20 | B42.3–.5 | parcial (32ba879a1f · 6b502e3e55 · 138cce774b): undo pela RAM; 40 Hz e Spread de uma fonte; raio 512 medido. |
| onda 20 | B43.6 | parcial (f8dc48c3de): cgu1 custa ~4× a edição; vai por drop-crate. |
| onda 20 | B35.13 | parcial (892acec6f9): as 17 cerdas no cartão Shape; falta dustDrop/wetOpen* junto dos pais. |
| fecho M1–M3 | B08c «depois» no app | feito (3df3206f86 · da1854fb75): 41 células; regressões Filbert soltar (= luz Cheia no repouso, B14.2) e Ctrl+Z Impasto. |
| fecho C1 | Ctrl+Z Impasto 4K | feito (2121ecd849 · 23afcc19e0): 42,8 → 34,3 ms (= caminho antigo 34,1). |
| fecho C2–C3 | soltar Filbert 4K | parcial (6f11399c70 · f983140019): 47,8 → 34,0 ms; Cheia no gesto daria 19,1 (DECISÃO DO DONO, B14.2). Defeito: metade do traço ficava Simples. |
| fecho F1 | fmt/clippy, tectos, os_cartoes, B50.1, alarmes | feito (9cdcbb95b7 · 7c01a2a029 · 32d4989ccf · 5c326dabb5 · a41c9b5a8e): largura 2 · miniatura 74 vermelhos, não regravados. |
| fecho F2 | B47.6 · B47.7 | feito (ba252b036f · 1d40807b12): presets no schema 2 sem píxel; secção Bristles vazia sai do Tuning. |

## Fatos medidos que mudam a ordem

- 4K papel: quadro de ~100 ms igual em TODOS os pincéis no 1.º movimento, ~0,5 s depois de soltar e depois do Ctrl+Z
  (1080p ~35 ms) — fora do pincel. Traço 4K p95 21–26 ms (Filbert 37). Toque Water/Watercolor Basic ~155 ms.
- Parado: 48–70 % de um núcleo (o app redesenha sem entrada) → B23 sobe.
- Wet com Real Oil: a água cai de 11,9 para 0,6 img/s em 6 rodadas.
