# Estudo: o que custa caro no Wet Paint, e o que cortar (07/11)

Estudo por leitura de código e por medições já gravadas. **Nada foi compilado nem corrido, e nada no produto mudou.**
Decisão de cortar é do dono: cortar muda pincel.

## 1. Resumo para o dono

**O que mais pesa, nesta ordem:**

1. **A água da aguada depois de soltar a caneta.** Em tela 4K com pincel grande, ela ocupa de 11 a 14 núcleos, sem parar enquanto houver água, e a tela atualiza 0 a 1 vez por segundo. O óleo, parado, gasta zero.
2. **A luz e a sombra do óleo (Real Oil) durante o traço.** Com pincel de tamanho normal, é cerca de 60 % do trabalho. Ela é o que dá a cara do óleo, e o senhor disse que é o mais importante.
3. **O tamanho real da ponta nos pincéis chatos.** O Flat Bristle, a tamanho 240, pinta com uma lâmina 4,6 vezes mais comprida que a do Dabber: 8 vezes mais tinta calculada por marca. É por isso que ele leva cerca de 1 segundo por quadro em 4K. Isto não é um efeito ligável ou desligável; é o formato da ponta. A explicação ainda é conta, não medição.
4. **As cerdas pesam menos do que parecia.** Tirar tudo delas ganha cerca de 10 % a tamanho 51 e 20 % a tamanho 200 (uma corrida por caso). O relevo das cerdas sozinho responde por 12 %. A variação de cor e a de pressão por cerda, cada uma sozinha, não se distinguem do ruído da medida.

**O que eu cortaria primeiro:**

| Ordem | Corte | O que se perde | Ganho |
|---|---|---|---|
| 1 | **Nada se perde: tirar contas repetidas nas cerdas** (ver 3.3) | Nada; a pintura fica igual | Ainda não medido |
| 2 | **`Bristle relief` a 0 nos 11 pincéis que o usam** (hoje 0,75). O senhor já pôs a 0 e gostou | A escovada longa do Dabber a tamanho grande; a 200 ele fica mais granulado | ~12 % no traço (uma corrida) |
| 3 | **Fazer a aguada parar mais cedo** | O escorrer lento depois de alguns segundos | Grande, mas ainda não medido; é a próxima medição a fazer |
| 4 | **Reformar a ponta dos chatos** (Flat Bristle, Tilted Shade, Filbert Dry) | Precisa de foto; pode mudar a marca do pincel | Possivelmente o maior no traço grande; ainda não medido |

**O que eu NÃO cortaria:** Real Oil, Ragged tips, Bristle width, Bristles follow brush size. E **remover os controles que nenhum pincel usa não acelera nada**: desligados, eles já quase não custam (conta em 3.1).

**O conflito que precisa da sua decisão:** o senhor disse em 05/11 «adoro os pincéis como estão». Os itens 2, 3 e 4 mudam pincéis aprovados. O item 1 não muda nada.

**Sobre a meta:** a tamanho 30 em 1080p, o traço do Dabber já cabe em um quadro de 60 Hz (8,5 ms); o Flat Bristle passa um pouco (20 ms). O problema maior aparece no pincel grande, em tela grande e na aguada parada. O tamanho de fábrica (46 a 51) não foi medido nesta rodada.

---

## Notação

**M** = medido, com fonte e condições. **D** = deduzido do código (contagem de operações; [não é perfil](../../project-memory/feedback_an_operation_count_is_not_a_profile_and_the_build_profile_decides_the_number.md)). **N** = não medido.

Caminhos abreviados: `FIOS` = `crates/ph2d-wet-paint/src/bristles_fios.rs` · `POUSA` = `crates/ph2d-tool-painter/src/tool/paint/wetpaint/pousa_o_dab.rs` · `LOTE` = `crates/ph2d-tool-painter/src/tool/paint/wetpaint/lote.rs`.

**Fontes de medição:**

| Sigla | Fonte | Condições |
|---|---|---|
| **H0711** | [handoff de 07/11](handoffs/HANDOFF_CONTINUACAO_2026-11-07_undo_redo_e_lote.md), item 2 | Perfil `smoke`, relógio do app, 60 Hz, ferramenta nova por corrida, mínimo de 3 rodadas (2 no Size 30), carga ~3–9, `rayon=16` |
| **LOG-V** | `target/lote/velocidade.log`, `velocidade30.log` (não versionados) | A mesma corrida. A CPU por fio citada aqui é da **rodada 0**, sem resumo |
| **LOG-E** | `target/lote/dab.log`, `dab2.log`, `dab6.log` (não versionados) | `sonda_a_escovada_do_dabber`, `smoke`. ⚠️ **Uma corrida por variante, em série, carga não gravada.** O mesmo caso varia ~3 % entre logs (5 665 × 5 810 ms) |
| **M50** | [doc 50 §4](NewFeaturesWetPaint/50_medicao_o_relevo_sem_escada.md) | `release`, 1024², Oil Bristle (hoje fora da fábrica) a 103 px, traço rápido, mínimo de 7, carga < 3,2. ⚠️ **Anterior à lei de 19/10**, que levou esse caso de 13,7 a 7,3 ms ([doc 51 §6](NewFeaturesWetPaint/51_medicao_spacing_continuous.md)). As proporções podem ter mudado |
| **J13** | [J13](Arquitetura/J13_fps_do_real_oil.md) | 1024², carga 9–17; cena sem tela para as fases, perf com símbolos |
| **B** | [B_nucleos](49_realshader/B_nucleos.md), [SECAGEM](49_realshader/SECAGEM_diagnostico.md) | Wash 4096×2560, carga 3–9. Antes de cortes posteriores: as partes valem como ordem de grandeza |

⚠️ **Armadilha de nome:** «Dabber sem relevo» em H0711 é **Real Oil desligado** (`set_wet_oleo_real_ligado(false)`, `sonda_lote_tests.rs:173`), **não** `Bristle relief` 0. São coisas diferentes.

**Como contei os pincéis:** decodifiquei os 56 `assets/brushes/WetPainter/**/*.ph2d-brush.png` (chunk `iTXt`, que os testes geram das receitas de `tests/it/familias/`) e contei valor > 0 na chave. Chave ausente = neutro. Base: **46 de pintura** + 10 ferramentas (5 Smear, 4 Blend, 1 Water). Os outros 22 dos 78 de fábrica são Digital e Watercolor, que não passam por aqui.

---

## 2. Tabela das features caras

| Feature | Knob(s) | Quando paga | Custo | Evidência | Paga desligada? | Pincéis | Benefício e aprovação | Veredito |
|---|---|---|---|---|---|---|---|---|
| **Água depois de soltar** | `Evaporation`, Dilution, grade | Por célula molhada, por passo, em outro fio | Parado: 4K Size 240 **11–14 núcleos**, 0–1 imagem/s; 1080p 7–9 núcleos, 20–29 imagens/s; óleo **0** | **M** H0711. Hipótese do gargalo (compositor na fila do mesmo pool): **N** | Não: sem água, o passo sai cedo (`sim.rs:492`) | 29/46 têm água no volume. Nos óleos (0,1) ela acaba em ≤ 2 s | O escorrer é o coração da aguada. Sem foto aprovando o escorrer tardio em particular | **Simplificar**: parar mais cedo; medir antes |
| **Real Oil: luz no traço** | `realOil`, `OilDepth`, `OilGloss`, material | Por célula da caixa, por imagem, fora do fio da tela | Traço, p50 sem → com: 1080p Dabber 12 → 34 ms; 4K Dabber 68 → 130. CPU do pool: Size 30 ~**60 %** dele; 4K Size 240 Dabber 60 %, Flat 26 % | **M** H0711, LOG-V (rodada 0) | **Sim:** a grade que viu óleo fica com a óptica (J13 §5) | 14/46 | **Aprovado:** W1c e W2, [«Muito bom!», 27/10](../../project-memory/project_oleo_real_mission_w2_2026_10_27.md); [o mais importante, 26/10](../../project-memory/project_oleo_real_priority_relief_shader_light_2026_10_26.md) | **Manter**. Só simplificar a prévia durante o traço, se a medição pagar |
| **Real Oil: imagem ao assentar** | idem | Uma imagem ao soltar e depois | 222–1252 ms | **M** H0711 | Parado: 0 | 14/46 | idem | **Manter** |
| **Geometria da lâmina** | `dab_flatten`, `dab_quina`, `width_table` | Por célula, por sub-dab | Flat a Size 240: raio do motor ~807, ~781 mil células por marca; Dabber ~176, ~98 mil. Traço 4K: **1037 × 130 ms** | Tempo **M** H0711; raio e células **D** (tabela invertida, 3.2); causa **N** | É o formato, não um efeito | Muito caros: Flat Bristle, Tilted Shade, Filbert Dry | Dono escolheu o Flat em [19/10](Pinceis/01_CATALOGO.md); «adoro» geral 05/11 | **Medir primeiro**; simplificar se a foto aprovar |
| **Sub-dabs (Continuous)** | `continuous_spacing`, `capsuleMaxPx` | Por sub-dab: o carimbo inteiro | Dabber a cada 2 px, Flat a cada 4, aguadas ~2,5 % do diâmetro. A lei de 19/10 reduziu os dabs à metade: Oil Bristle 14,5 → 7,3 ms | **M** [doc 51 §6](NewFeaturesWetPaint/51_medicao_spacing_continuous.md); Size grande **N** | Só com a opção ligada | 54/56 | **Vitória do dono**: [«não desfazer», 01/11](../../project-memory/project_painter_panel_e10_owner_decisions_2026_11_01.md) | **Manter**. ⛔ Tecto de 3 px no Oil Bristle dá escada 4,71 ([recusa 51](NewFeaturesWetPaint/51_medicao_spacing_continuous.md)). Só testar o tecto a tamanho grande |
| **Relevo das cerdas** | `bristleHeight`, `bristleRemove` | Por célula, por dab | Sem ele: **−12 %** a Size 51 e 200. Oil Bristle: 13,7 → 11,4 ms | **M** LOG-E (n=1), M50 | **Não** (`POUSA:308`); o campo do óleo ainda lê o plano vazio | 11/46 | Dono pôs 0 e gostou (informado neste pedido; **não achei em `project-memory`**) | **Cortar** nos 11, com o relatório de pincéis |
| **Pressão por cerda** | `bristlePressure`, `bristleLift`, `bristleNoiseLength` | Tabela por dab; consulta por célula | Sozinha: −2,3 % e −2,4 %, **dentro do ruído** | **M** LOG-E | Não, se todos os fios estiverem neutros | 17/46 (`Lift` 10) | Faixas do Express Oil, «mão leve». Aprovação só no «adoro» | **Manter** (é a base do Ragged, 3.4) |
| **Cor por cerda (HSLuv)** | `bristleNoiseSat`, `Light`, `Hue` | Tabela por dab + mistura por célula | Sozinha: +0,4 % e −3,2 %, ruído. Oil Bristle: 1,5 ms de 13,7 | **M** LOG-E, M50 | Não | 15/46 (`Hue` 0) | Os fios claros e escuros. Dono em 01/11: riscas e cor por cerda moram no Shape | **Deixar opcional**; ganho pequeno |
| **Todos os fios juntos** | os cinco do Dabber | | **−9,5 %** Size 51, **−20 %** Size 200; Oil Bristle 13,7 → 8,0 | **M** LOG-E, M50 | | 17/46 | | Não cortar em bloco |
| **Ragged tips** | `bristleTipRagged` | 16 leituras por dab; **4 por célula** | **N**. Candidato a parte do custo do Flat | **D** `ponta_das_cerdas.rs:55,111` | Não (`rho = 0` não entra) | 7/46; só em chatos e leque | **Dono valoriza** | **Manter**; medir |
| **Jitter de posição** | `brush.jitter` | Por dab | Sem ele: −17 % Size 51, −22 % Size 200 | **M** LOG-E | Não | Dabber 0,3 e outros | ⛔ **Recusa medida**: jitter 0 vira carris (0,72); é o que dá largura ([W3_P_Dabber](Pinceis/W3_P_Dabber.md)) | **Manter** |
| **Smooth Edges** | `smoothEdges` | Plano por imagem; caixa maior | Retângulo 300²: 0,13 → 1,4 ms **no máximo**, versão antiga da tela, carga 28. Memória do 0,3 no Dabber: pico **305 → 368 MiB** (raio 300), 1 610 → 1 802 (raio 1000) | **M** [doc 14 §5](NewFeaturesWetPaint/14_smooth_edges_a_borda_suave.md), [handoff 05/11](handoffs/HANDOFF_CONTINUACAO_2026-11-05_noite.md). Tempo da versão por traço: **N** | Não (`composite_otica.rs:139`) | 15/46 | **Ordem do dono:** Liner e Wash a 0,12 ([smoke 05/11](../../project-memory/project_smoke_2026_11_05_owner_verdicts.md)); 0,3 nos 13 óleos com buracos ≥ 5 % ([handoff](handoffs/HANDOFF_CONTINUACAO_2026-11-05_bordas_e_smooth_oleo.md)) | **Manter**; medir o tempo do 0,3 |
| **K–M, depósito** | `pigmentMixing` (ausente = ligado) | Por célula que recebe segunda tinta | ~3,8 ms de 13,7 em Oil Bristle | **M** M50, com a ressalva | Só quando a célula tem cor própria | 53/56 ligados | [Ordem do dono 20/09](../../crates/ph2d-wet-paint/src/colorops.rs): os meios misturam igual | **Manter**; é a cor que ele conhece |
| **K–M, advecção** | idem | Por célula em movimento | **N** hoje; a porta da medida existe (`measure_wetpaint_step.rs:377`) | **N** | Sim, com K–M | 53/56 | idem | **Manter**; medir |
| **Secagem (`assenta`)** | `Evaporation`, `Rewet` | Por célula molhada, 1 passo em 6 | Era 46–55 % do passo; depois cortada ao bit em −36 % | **M** B | Sem água, não roda | Todos com água | Parte da lei | **Manter**. ⛔ TOL, FIXA e UMA recusados (até 22 níveis) |
| **Tapete de cerdas** | `bristleCount`, `Strength`, `Size`, `felt` | Por célula: consulta; por mudança: construir | Construir: 0,21 ms (Size 2), **3,73** (16), **7,29** (16, 4000) | **M** H0711, LOG-E. Consulta: **N** | Não: Shape sem Bristles é liso | 26/46 entram pelo caminho (10 têm grão por cima; Liner e Wash ficam fora pelo filme) | Parte da textura. Dabber D4: «tapete sozinho não pinta» | **Manter**. ⚠️ Bristle Size 16 custa 18× ao mudar |
| **Memória de cerdas (`SilMemo`)** | ligada pelo relevo, lâmina ou grão | Por sub-dab: cria e preenche um vetor | ~6 MB por marca no Flat a 240 | **D** `sil_memo.rs:29-31` (conta 3.3); tempo **N** | Não sem relevo, lâmina ou grão | ≥ 11 | Nenhum visual: pura contabilidade | **Reaproveitar o vetor** |
| **Memória da tinta viva** | sempre, no Wet | Parado | ~140 B/célula; 4096×2560 Dabber: **5 081 → 500 MB** comprimida | **M** [TINTA_VIVA](TINTA_VIVA_2026-11-06.md) | Sim, na sessão | Todos | **Decisão do dono, 06/11** | **Manter** |
| **Undo vivo** | sempre | Pen-down, mais registros | Pico do Dabber 367 → 490 MiB (raio 300); pen-down 4K ~52–68 ms | **M** [memória](../../project-memory/project_first_stroke_undo_memory_cut_next_round_2026_11_07.md), [handoff](handoffs/HANDOFF_CONTINUACAO_2026-11-06_tinta_viva_onda2.md). Causa: leitura | Sim | Todos | **Pedido do dono**. Corte da caixa do traço já ordenado para depois do CI | **Simplificar** como já ordenado |
| **Controles sem uso** | Wave, Hue, Streak (0/46); Span, Oscillation (1: Fan) | Só comparações | ~0 | **D** `FIOS:97,233,357,374` | ~0 | Ver ao lado | [Config. avançada «não se perde», 01/11](../../project-memory/project_painter_panel_e10_owner_decisions_2026_11_01.md) | **Não cortar por velocidade** |

---

## 3. As cerdas em detalhe

### 3.1 O que acontece a cada dab

Um dab só toca nos fios se algum estiver ligado (`FIOS:97-104`, `POUSA:306-309`). Em aguada sem cerdas o custo é zero, salvo algumas contas por dab.

| Peça | Custo por dab, por fio | Custo por célula | Medido sozinho (Dabber, %; 51 / 200) | Se sair, cai |
|---|---|---|---|---|
| **Pressão** (`bristlePressure`) | 1 Perlin composto (6 hashes), `FIOS:300-309` | Multiplicação e consulta | −4,4 / −2,3 | Faixas fortes e fracas ao longo do traço; **e o Ragged**, se nada mais mantiver os fios |
| **Levanta** (`bristleLift`) | Junto do ganho | Nada a mais | −2,8 / −2,4 | Mão leve mais texturada |
| **Cor** (`Sat`, `Light`) | ⚠️ **3** Perlins sempre, mesmo com matiz 0, `FIOS:347-348`; HSLuv ida e volta, `bristles_cor.rs:50-59` | Consulta; depois a mistura K–M | −4,2 / +0,4 (Sat); −4,8 / −3,2 (Light) | Os fios claros e escuros |
| **Matiz** (`Hue`) | Idem | | 0/46 | ⛔ «berrante» já a 0,08 ([recusa 48](NewFeaturesWetPaint/48_medicao_os_fios.md)) |
| **Onda** (`Wave`) | 0 quando 0 (`FIOS:374,390`); 2 Perlin simples por célula quando ligada | Até 2 Perlin | 0/46 | Nada na fábrica |
| **Span** | 0 quando 0 (`FIOS:233`) | Deslocamento | Só Fan | Leque |
| **Oscilação** | 1 Perlin por dab | Soma | Só Fan | Leque |
| **Veio** (`Streak`) | 0 quando 0 (`FIOS:357`) | | 0/46 | Nada na fábrica |
| **Relevo** (`bristleHeight`) | 2ª tabela de ganhos, com os mesmos Perlin (`LOTE:428-430`) | Cosseno + potência, em `painter/relevo.rs:53-63` | **−11,9 / −11,9** | A escovada; em Dabber a 200, coerência 0,471 → 0,349 |
| **Ragged** | Hash fixo por fio | **4 leituras de silhueta** (`ponta_das_cerdas.rs:110-114`) | não medido | Ponta reta e lisa dos chatos |
| **Tapete** | | `bristles_at` + `bristle_tile` | não medido | Poros; o tapete é ~94 % feltro |
| **Bristle width** | Fios ∝ 1/largura | Não muda | −6,1 / −2,3 (de 1,5 para 3) | ⛔ fios de 2,5 px perdem a faixa de 3 px |
| **Follow brush size** | Menos fios a tamanho grande | Não muda | Com γ 1: −1,0 / −6,9 | Fios mais largos; ⛔ não resolve as riscas |

⚠️ A Size 51, variantes sem relação com as cerdas já dão −3 a −5 % (`felt` −4,2, `pigmentPerDab` −5,1, `paperGate` −3,2). Isso é ruído ou efeito de aquecimento: leia as colunas só como «não se destaca». Entre as peças das cerdas, a única que se destaca é o relevo.

### 3.2 Por que o Flat Bristle é o mais caro (conta, não perfil)

O Size é a **largura prometida**: `dab_spec.rs:80-81` procura na `width_table` do pincel o raio do motor que entrega essa largura. O Flat tem `dab_flatten` 0,7 e `dab_quina` 1, e a largura sai do eixo menor. Então, para pintar 480 px, o motor precisa de um retângulo de ~807 de raio.

Tabela invertida (`cel/dab` ≈ `(4q + π(1−q))·r²·(1−flatten)`, ignora rotação e jitter):

| Pincel | r a Size 30 | r a Size 240 | mil cél/dab a 30 | mil cél/dab a 240 | sub-dab a 240 |
|---|---|---|---|---|---|
| Dabber | 25 | 176 | 1,9 | **98** | 2 px |
| Thick Impasto | 25 | 198 | 2,0 | 123 | 4 px |
| Filbert | 31 | 246 | 3,1 | 190 | 4 px |
| Filbert Dry | 64 | 493 | 7,1 | 424 | 4 px |
| Tilted Shade | 89 | 695 | 9,9 | 604 | 4 px |
| **Flat Bristle** | **101** | **807** | **12,2** | **781** | 4 px |
| Knife Smooth 3 | 30 | 240 | 0,7 | 46 | 2 px |
| Wet Round | 49 | 467 | 7,5 | 685 | ~23 px |
| Wet Wash | 31 | 246 | 3,0 | 190 | ~12 px |

**O modelo contra o relógio** (4K, Size 240, traço de ~4 900 px, CPU do pool na rodada 0 de LOG-V, Real Oil desligado):

| Pincel | Visitas a células (milhões) | CPU (s) | ns/visita |
|---|---|---|---|
| Wet Round | 146 | 6,9 | **47** |
| Wet Flat | 88 | 6,5 | 74 |
| Wet Wash | 76 | 8,5 | 112 |
| Dabber | 239 | 47,3 | **198** |
| Flat Bristle | 957 | 537,3 | **561** |

Leitura: o Flat paga **mais células E mais por célula**. As células explicam 4× (Dabber → Flat); sobra ~2,8× por célula, onde Ragged, rotação por dab e o `SilMemo` são suspeitos.

⚠️ **O modelo falha a tamanho pequeno:** a Size 30 ele prevê 3,3× entre os dois, e a CPU medida é 2,1×. Lá a conta dá ~1 000 ns/visita, porque composição e água dominam. Também: a CPU do pool inclui água e composição, então estes ns são **tetos**, não custo do carimbo. Este desacordo [é um achado](../../project-memory/feedback_an_operation_count_is_not_a_profile_and_the_build_profile_decides_the_number.md): perfilar antes de mexer.

Dado medido à parte (LOG-V): no mesmo gesto, o Flat deixa massa em **6,07 M** células, o Dabber em **2,24 M**, e a área composta é 133 × 43 M px. Com largura «prometida» igual, isso sugere tinta fraca fora da largura visível, ou a lâmina comprida varrendo de lado na curva. **Não confirmado.**

### 3.3 Contas repetidas (nada muda na pintura)

Tudo **D**, ganho **N**. Primeiro da fila porque a régua é o byte igual.

| # | O que | Onde | Conta |
|---|---|---|---|
| 1 | `SilMemo` cria e preenche um vetor novo a cada sub-dab | `sil_memo.rs:29-31`, `POUSA:358` | Flat a 240: ~(1617 × 487) × 8 B ≈ **6 MB por sub-dab**, ~10 sub-dabs por quadro em 4K. Hipótese: page faults em alocação grande. Reaproveitar |
| 2 | 3 Perlin compostos por fio mesmo com matiz 0 | `FIOS:347-348` | 18 hashes por fio; 6 a 12 a menos com um ou dois canais a 0 (Hue é 0 em todos; Light é 0 no Flat). Pular os canais a 0 |
| 3 | 2 tabelas por dab com o mesmo ganho | `POUSA:386` + `LOTE:428-430` | A do relevo recalcula o Perlin que a dos fios já tem |
| 4 | `bristle_tile`: `RefCell` + `Rc::clone` + comparar chave **por célula** | `texture/bristles.rs:31-44,91` | Resolver uma vez por dab ou fila |
| 5 | Potência `det_powf` (software) por célula no pente, quando `passos ≠ 1` | `bristles_relevo.rs:77-81` | O expoente é do dab, a base é da célula. Com sub-passo inteiro o expoente pode ser inteiro, e aí há atalho ao byte; conferir antes, porque o caminho vem do arco. ⛔ O **cosseno** já foi medido e não é o custo (doc 50 §4) |
| 6 | 16 leituras de silhueta por dab para achar a corda | `ponta_das_cerdas.rs:55-62` | Pequeno ante as células; só no Ragged |
| 7 | `tile` por fio do pool (`thread_local`) | `texture/bristles.rs:33` | Reconstrói por fio: Bristle Size 16 × 16 fios |

### 3.4 O que depende do quê (⚠️ cortes que derrubam outro)

- **Ragged precisa dos fios:** `PontaDasCerdas::nova(relevo_dab.or(fios_dab), …)` (`POUSA:325-332`). Sem relevo e com os fios neutros, a ponta desfiada **some em silêncio**. Nos 7 pincéis com Ragged, `Bristle relief` 0 é seguro enquanto `bristlePressure` > 0 (todos têm). Cortar relevo + pressão + cor junto **mata o Ragged**.
- **`bristleLaneWidth` é só largura de fio:** a Dabber a 200 ela não muda o tempo além do ruído. Deixá-la como o dono quiser é barato.
- **Relevo a 0 no Dabber a 200:** fica granulado, a cara que [o handoff de 07/11](handoffs/HANDOFF_CONTINUACAO_2026-11-07_undo_redo_e_lote.md) busca, com gradiente um pouco mais plano (12,8 × 16,3, medido a 0,12). Valores intermediários 0,25 e 0,12 já foram testados: tempo 5 505 e 5 446 × 5 665 ms, ou seja, **quase todo o ganho de tempo só vem em 0**, quando o pente deixa de correr.
- **Os portões `o_relevo_dos_fios` e `o_oleo_tem_o_relevo_de_cada_fio`** esperam o relevo: mudam junto, com a régua trocada por ordem do dono.

---

## 4. O resto do Wet Paint

| Tema | O que se sabe | Evidência | Leitura |
|---|---|---|---|
| **Solver: a água** | Passos: 1080p 40/s; 4K Size 240 só 22–25/s, worker 96–98 %. Razão de fluxo 1×1 e 1×2 sem diferença material | **M** H0711 | «O fluxo não é o custo» |
| **Solver: partes** | Secagem 46–55 %, advecção 34 %, fluxo 3 %, projeção 2 %, em um wash; depois −36 % na secagem | **M** B | Secar e transportar pigmento são o grosso. ⛔ [Recusas](49_realshader/SECAGEM_diagnostico.md) |
| **Grade da água (`G`)** | Size 200: Flat 19,6 → 6,9 s, Thick 4,8 → 1,2, Wet Round 1,42 → **0,26**; Wet Wash com detalhe 1,01, Wet Round 0,74–0,78 (igual a hoje). Mas: borra Thick, fura Dabber, xadrez em Knife, a zoom 100 % | **M** H0711, carga ~12 | ⭐ **Em aguada, quase de graça; em óleo, não.** ⛔ `G` por tamanho de pincel **trava**: a razão é da sessão, e mudar mata a tinta viva. ⛔ O protótipo do carimbo foi recusado pelo dono em 07/11 |
| **Óleo: o campo e a óptica** | Por composição, 555 mil células: campo 12,1 ms · pigmento 28,0 · sombra+grão 14,8 · 55,6 no total. Perf: `visibilidade_e_ceu` 19 %, `rebote_no_papel` 12,6 %, `luz_directa` 9,7 %, `Sectores` 7,5 %, libm ~10 % | **M** J13 §2 | Só leem altura e albedo. Por célula: **72 leituras** de horizonte (4 fatias × 2 lados × 9, `oleo_real.rs:186-187`) e 24 passos de sombra (:182) |
| **Óleo: fora da tela** | Quadro parado 72–82 → 16,5–17 ms. Mas: óptica ainda 0,4–66 ms por imagem, em outro fio | **M** [J13 §6](Arquitetura/J13_fps_do_real_oil.md) | Já curado na tela; o trabalho continua |
| **Óleo: borda** | `oleo_borda.rs`: `borda_suave`, `sem_entalhes`, desfoques; parte dos 12 ms do campo | **D**; sem parte separada **N** | Sem dado para cortar |
| **Óleo: horizonte e pontinhos** | `oleo_pontinhos.rs` é **régua**, não custo de produto. Horizonte acima | **D** | Nada a cortar nos pontinhos |
| **Óleo: inerte que paga** | Uma vez com óleo na grade, aguada por cima paga a óptica inteira: «a grade fica com a óptica» | **M** J13 §5. Parte evitável: **N** | ⭐ Bom candidato: Wet Drip sobre Dabber. ⛔ Cache de luz por célula já **recusada**: mais lenta em todo ε (24,3 × 19,6 ms) |
| **Smooth Edges** | Ver tabela. Com óleo, também alarga a janela do campo (`oleo_campo.rs:36`) e amacia a silhueta dele | **D** `composite_otica.rs:173-183` | Só com o valor > 0; `raio_do_teto` |
| **Adormecer logo após o traço** | Dabber em 4K: 1721 → 0,5 ms. Wet Round **não curado**: 4K 245/1282 ms. Compor por linhas: ⛔ não chega | **M** [handoff](handoffs/HANDOFF_CONTINUACAO_2026-11-06_tinta_viva_onda2.md) | Pendente do dono |
| **Tinta adormecida** | Outros modos: **sem** custo acima do ruído (28,27 × 29,06 ms; ociosa 71 µs); comprimir 2,5 s e descomprimir 1,0 s em fio à parte, a 4K | **M** [TINTA_VIVA](TINTA_VIVA_2026-11-06.md) | Passou no critério. **Não cortar** |
| **Undo vivo** | 4K: 4 → 28 passos curtos, fotos 480–560 → ~150–180 MB; desfazer 199/351 ms; refazer 206/298 | **M** idem | Pedido do dono. Falta só a caixa do traço |
| **Memória dos planos** | 140 B/célula; `campo_cache` ~28 B (230 MB a 4K, só com óleo); cor morta após Dry canvas 19,4 MB | **M** TINTA_VIVA | ⭐ «Cor morta» é cura de graça: `canvas_ops.rs:71-73` |
| **Composição** | Parado na aguada: óptica mede 15 ms, mas saem 0–1 imagens/s a 4K | **M** H0711. Fila no pool: **N** | Provável ganho **sem** mudar pintura: separar pools |
| **Primeiro traço** | Pen-down 10–14 ms em 1080p, 34–38 ms em 4K (fluxo 2). Óleo 30–64 ms antes de P8 | **M** H0711, J13 §6 | Aberto |
| **Placa de vídeo** | A óptica é CPU. Submissões fatiadas 93,3 → 22,6 ms na Radeon; ⛔ a NVIDIA travou o PC 2× | **M** [PLACA_degrau6](49_realshader/PLACA_degrau6.md) | Fora deste estudo |

---

## 5. Ordem de medição proposta

Um único lote ([§0.10](../../CLAUDE.md)), uma compilação `smoke`, regras de [MEDIR_VELOCIDADE](../DevOps/MEDIR_VELOCIDADE.md): variantes por override no mesmo processo, em blocos rodados, o mínimo, e o `loadavg` ao lado. Sem esperar calma. **Não escrevi nenhuma destas sondas.**

⛔ Restrições já pagas:
- **Real Oil × sem Real Oil:** ferramenta nova por corrida (J13 §5: a grade contamina), e cada rodada é uma réplica.
- **Comparação pelo mínimo** só em réplica: nunca sobre água acumulada.
- **Traço rápido no fio da tela não vê** o carimbo no HUD: use o relógio do handler (doc 50 §4).

| # | Sonda (nome sugerido) | Variantes | Pincéis e tamanhos | O que comparar |
|---|---|---|---|---|
| **1** | `sonda_custo_das_cerdas` | Fábrica · `Height` 0 · `Sat`+`Light` 0 · `Pressure`+`Lift` 0 mantendo os fios · `Ragged` 0 · `Quina` 0 · `Flatten` 0 · jitter 0 · todos · repetir fábrica como controle | Dabber, Flat Bristle, Filbert, Thick Impasto, Knife Smooth 3; Size 30, factory, 120, 240; 1080p e 4K | p50/p95 do quadro, CPU por fio, sub-dabs e **visitas a células** (contador `cfg(test)`, acusa quando a variante não exerceu); px mudados e foto |
| **2** | Perfil do Flat | `perf` com símbolos (`target/perfsym`), Flat a Size 240 e 46, com e sem Real Oil | Flat, Dabber | Decide 3.2: células, Ragged, memo ou janela do rasto. Se discordar da conta por > 2×, o relógio ganha |
| **3** | `sonda_otimizacoes_ao_byte` | Flag para cada item de 3.3 (memo reusado; canais a 0; tabela única; tile por dab; `x²`) | Dabber e Flat, 120 e 240 | Tempo e **hash da imagem igual**: sem hash igual, descarta |
| **4** | `sonda_aguada_parada` | `Evaporation` ×1, ×2, ×4 · parar após 3, 5, 10 s · compositor em pool próprio · grade fixa 2 na sessão | Wet Round, Wet Wash, Wet Drip, Wet Flat; 4K Size 240 e 30, 1080p 120 | CPU total por segundo, imagens/s, tempo até zerar e **foto aos 3, 10 e 30 s** para o dono. Mede a hipótese do pool |
| **5** | `sonda_otica_inerte` | Aguada em tela **sem** óleo × tela com óleo longe × com óleo debaixo | Wet Drip sobre Dabber | Tempo de óptica por imagem: quanto é evitável com hash igual |
| **6** | `sonda_otica_leve_no_traco` | `FATIAS` 4 e 2 · `HORIZONTE` 9 e 5 · `PASSOS_SOMBRA` 24 e 12. Exige passar de `const` a override | Dabber, Flat; 30 e 120 | Tempo, p95 e foto parada × em movimento. Só faz sentido com o dono vendo |
| **7** | `sonda_km_na_agua` | Porta existente `measure_wetpaint_step` K–M on e off, mais `pigmentMixing` no depósito | Wet Round, Dabber | ms/passo e a foto |
| **8** | `sonda_suave_por_traco` | `smoothEdges` 0, 0,12, 0,3 | 13 receitas + Liner, Wash; 30 e 120 | Custo do valor atual, nunca medido nessa versão |

**Encadeamento:** 2 depende de 1 só para focar. 3 pode correr junto. Depois do lote, uma folha para o dono com cada corte que ganhar > 10 %, em zoom 100 %: a [folha reduzida já escondeu defeitos](handoffs/HANDOFF_CONTINUACAO_2026-11-07_undo_redo_e_lote.md).

---

## 6. Limites e conflitos

**O que não consegui confirmar:**

- **Nada aqui foi perfilado.** As contagens são leitura de código. Os ns/visita são estimativas com CPU que inclui água e composição.
- **LOG-E tem n = 1 por variante e carga não gravada.** Eu aceitaria o −12 % do relevo e o −20 % de todos os fios como ordem de grandeza; os demais, como «não se destaca». **M50 e LOG-E discordam** (−42 % × −20 % sem fios): cena, Size e versão são diferentes, e M50 é anterior à lei de 19/10. Não dá para dizer qual vale hoje a Size 51 no traço rápido.
- **LOG-V, rodada 0:** os valores absolutos de CPU são da fatia de 16 fios, não do app. O envio à placa não está aí.
- **Causa do custo do Flat: não medida.** A parte «lâmina» vem da tabela de largura; a área é aproximada.
- **Janela do rasto e transferência** para a grade (`trail/transfer.rs`): crescem com o raio e não têm medida separada. Podem ser parte relevante do Flat.
- **Aprovações:**
  - «Pôs `Bristle relief` a 0 e achou bom» e «os 4 que valoriza» vêm **do pedido desta rodada**; não há registro em `project-memory/`.
  - **`smoothEdges` 0,3 nos 13 óleos:** os handoffs de 05/11 dizem «ordem do dono», e a [memória de 04–05/11](../../project-memory/project_smooth_edges_per_brush_owner_decision_2026_11_04.md) ainda diz «os demais a 0». Fico com os handoffs, por serem posteriores; a memória está defasada.
  - Veredito individual por pincel de cerdas: só achei o «adoro» geral e os citados.
- **Efeito visual dos cortes:** não vi imagem nova. O que digo do visual vem dos docs e das folhas de 07/11, e a coerência é régua que [só compara consigo mesma](handoffs/HANDOFF_CONTINUACAO_2026-11-07_undo_redo_e_lote.md).
- **Tabelas de recusa:** consultei `50`, `48`, `51`, `J13 §5`, `SECAGEM`, `W3_P_Dabber`, `W3_K3b`, `ponta_das_cerdas.rs`, handoff 07/11 e TINTA_VIVA. **Não reli as 47 do [doc 28](28_otimizacoes_o_que_funcionou.md)**, só o índice; são sobretudo da aquarela antiga. Quem for implementar o item 3.3 deve lê-las.

**Conflitos com o que o dono aprovou, à vista:**

| Corte proposto | Choca com | Como fica |
|---|---|---|
| `Bristle relief` 0 em 11 pincéis | «Adoro os pincéis como estão» ([05/11](../../project-memory/project_brushes_as_they_are_are_the_reference_2026_11_05.md)); «o mais importante é o Relevo realista» (26/10); facas W2 aprovadas | O relevo do óleo fica; sai só o dos fios. Alarme e relatório; só ele regrava. Sugiro decidir **por pincel**, olhando facas à parte |
| Aguada parar mais cedo | «Adoro»; a lei velha é a referência ([05/11](../../project-memory/project_new_paint_law_for_all_brushes_2026_11_05.md)) | Mostrar foto aos 3, 10 e 30 s |
| Ponta dos chatos | «Adoro»; Flat escolhido em 19/10 | Só com foto a 100 %; largura e alarmes |
| Grade da água em tamanho grande | **Ele já recusou a variante de 07/11** («deixar como está»); razão por sessão choca com a tinta viva | Não reabro por óleo. Em aguada, com grade fixa na sessão, só se ele pedir: é a única variante com ganho de 5× medido e detalhe igual |
| Remover controles | «A configuração avançada é importante» (01/11) e o ganho é ~0 | Fica. Não proponho |
| `bristleSize` até 16 | Acaba de ser pedido | Fica; o preço é só ao mudar (3,73–7,29 ms, por fio do pool) |

**Recusas que este estudo respeita, e não propõe de volta:** jitter 0 · Spacing mais denso · `capsuleMaxPx` 3 no fio · cache de luz · compor por linhas · matiz por fio · TOL/FIXA/UMA do Newton · lei nova · massa em meia precisão · grade ligada ao Size.
