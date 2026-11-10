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

## Fatos medidos que mudam a ordem

- 4K papel: quadro de ~100 ms igual em TODOS os pincéis no 1.º movimento, ~0,5 s depois de soltar e depois do Ctrl+Z
  (1080p ~35 ms) — fora do pincel. Traço 4K p95 21–26 ms (Filbert 37). Toque Water/Watercolor Basic ~155 ms.
- Parado: 48–70 % de um núcleo (o app redesenha sem entrada) → B23 sobe.
- Wet com Real Oil: a água cai de 11,9 para 0,6 img/s em 6 rodadas.
