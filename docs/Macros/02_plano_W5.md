# Macros — plano da W5: ETAPAS (2026-10-14)

> Pedido do dono depois do smoke de 14/10 ([handoff W4 §DONO](handoffs/HANDOFF_CONTINUACAO_W4_COMPONENTE_2026-10-13.md#-dono-1410-depois-do-smoke-as-duas-faltas-que-definem-a-w5)):
> (1) cada ETAPA é uma chave na Timeline que se arrasta para re-cronometrá-la (e às seguintes); (2) um painel
> lateral de etapas NOMEADAS com as propriedades de cada uma editáveis. Decisão do dono (14/10, «siga» sobre a
> recomendação): arrastar uma etapa para mais tarde abre uma **ESPERA** — o relógio continua a correr (a água
> escorre, o vento sopra) —, nunca câmara lenta.

## 1. O que é uma etapa (medido)

Uma etapa é um GESTO do artista; o relógio entre gestos não é etapa. A segmentação é uma função PURA da trilha,
por família (só a família lê a sua carga, `crates/ph2d-macro/src/doc.rs:35`); nada se grava para a decidir.

| Família | Etapa | Relógio (não é etapa) |
|---|---|---|
| Motion | um `Passo::Grafo` (um lote do dreno, `macro_track/mod.rs:255-281`; um arrasto de nós `BeginDrag..EndDrag` = uma) · uma SESSÃO de param = `Params` do mesmo `(nó, param)` em quadros SEGUIDOS (não há início/fim de arrasto gravável: `motion_bridge_params_edit.rs:105-137` só abre sessão para o conta-gotas) · `Escritas`/`Documento` = «edição direta» (gizmo, desfazer) · `Recusada` = etapa que mostra o porquê | `Quadro`, `Toque` |
| Painter | uma pincelada `Ponteiro Down..Up` (`passo.rs:104-120`); as suas PROPRIEDADES são o pincel no toque (cor, tamanho), venham de `Cor`/`TamanhoPx`/`PresetAplica`/`Painel` · `Recusada` | `Tique`, `Agua`, os drenos (`Preview*`, `TiraCommit`…) |

**Nome:** derivado e traduzido no momento de mostrar (`ph2d-i18n`: «Add {nó}» com o nome do catálogo
`node_catalog.rs`, «{a} → {b}», «{param}: a → b», «Pincelada {n}» + a amostra da cor). Só o nome que o artista
escreve é conteúdo gravado.

## 2. Re-cronometrar — a escrita é no `MacroDoc`

⛔ **Recusado: uma chave LINEAR por fronteira de etapa na faixa «Macro Time».** (1) Estica o relógio DENTRO do
intervalo: os `Tique`/`Quadro` gravados espalham-se por mais tempo de cena ⇒ câmara lenta, não espera; (2) sem
ripple (`MoveSelectedKeys` move só as selecionadas, `ph2d-timeline/src/intent_apply.rs:145-156`); (3) as chaves
cruzam-se (`ph2d-anim/src/track.rs:397`) e o Bézier só limita o x (`curve.rs:82-88`) ⇒ tempo não-monótono ⇒
recuo a cada quadro (`em_cena.rs:59-89`); (4) segmento parado congela a água (sem tempo de macro não há `Tique`);
(5) duas verdades (chave × passo) que desalinham ao apagar/editar uma etapa; (6) chave «roving» perde o tempo
arrastado (`intent_settle.rs:25`). A faixa fica como está — identidade `(0,0)→(D′,D′)` — e serve a câmara lenta
DELIBERADA.

**Escrita:** deslocar a etapa k em +Δ desloca-a e a todas as seguintes; o intervalo que se abre recebe relógio
SINTETIZADO na cadência do último gravado (Motion: `Quadro{tempo+dt, a_tocar, dt, sujou:false}` e os `tempo`
seguintes +Δ; Painter: `Preview` + `Tique(dt)` por quadro). −Δ tira só relógio e pára no fim da etapa anterior.
O clipe (chave D′, `length_override`, laço — `clamp_loops_to_lengths` não corre no arrasto, `doc_loops.rs:93-112`)
ressincroniza-se na mesma transação.

## 3. Editar propriedades — MEDIDO (W5a, sondas `#[ignore]`, perfil smoke, load 17–28)

`crates/ph2d-tool-painter/tests/it/sonda_etapas.rs` e `crates/ph2d-app-motion/src/macro_track/etapas_sonda_tests.rs`.
A água é RE-SIMULADA na reprodução: a fita só guarda a agenda das respostas do worker (`wetpaint/fita.rs:17-46`);
o tocador anda o motor local até cada marca (`offthread.rs:378-397`).

**Painter, pincelada 2 editada** (512²; texels ≠ original no fim; 2 corridas iguais ao bit em TODAS as linhas):

| edição | gravada no worker, pela fita | worker, relógio fixo desde a etapa | Splash (relógio fixo) |
|---|---|---|---|
| cor | ✅ 21 379 | ✅ 38 205 | ✅ 58 188 |
| tamanho | ✅ 178 930 | ✅ 179 574 | ✅ 116 423 |
| caminho (+8 % x) | ✅ 28 295 | ✅ 45 991 | ✅ 91 858 |
| espera 0,5 s antes | ❌ `Divergiu(PerguntaInesperada)` | ✅ 43 205 | ✅ 95 835 |
| controlo: sem edição | — | 33 258 (só por largar a fita) | 0 |

⇒ **Lei:** uma edição de VALOR mantém a fita (o diff mínimo: a água correu os mesmos passos entre as mesmas ações);
uma edição de TEMPO (espera, deslocamento) passa a água ao relógio fixo desde a etapa deslocada
(`TocadorDoPainter::passa_ao_relogio_fixo`: o worker corre `STEP_S` de sim por `STEP_S` de parede, a cadência que
o relógio fixo idealiza). Se uma edição de valor divergir na reprodução de conferência (a água secou antes de uma
marca, `MarcaInalcancavel`), a edição grava o relógio fixo desde a sua etapa — decidido UMA vez, ao editar.

**Motion** (cena `=2`, 14,5 s, 894 eventos, 16 lotes do grafo): editar o valor de `size`/`gap_x` e abrir 1 s de
espera antes do gesto 6 ou 15 — **sem erro nos dois modos**, antes da edição igual ao original, fim diferente. O
Fiel só confere a CONTAGEM de marcas (`tocar.rs:276-283`). ⇒ «Fiel» continua a conferir; o cartão diz «editada
desde a etapa N». Custo do zero ao fim: Fiel 11 ms, Explicativo 53 ms (CPU) — o Motion não precisa de instantâneos.

**Custo de tocar do zero** (mínimo de 5 rondas intercaladas; mediana ao lado na sonda):

| | macro | do zero ao fim | por s de macro | até ao fim de cada pincelada |
|---|---|---|---|---|
| Splash 1024² fixo | 4,47 s | 1 094 ms | 245 ms | 52 · 116 · 205 · 306 |
| Splash 2048² fixo | 4,47 s | 2 399 ms | 537 ms | 127 · 266 · 442 · 662 |
| worker 1024² fita | 2,33 s | 482 ms | 207 ms | 35 · 127 · 201 · 261 · 349 · 412 |
| worker 1024², fixo desde 0 | 2,33 s | 469 ms | 201 ms | — |

⇒ Sem instantâneos, editar a etapa k de uma macro de 60 s a 2048² custaria ~32 s. **Instantâneo do tocador do
Painter no início de cada etapa** (W5c): editar a k recomeça do instantâneo k; scrub e recuo do laço usam o mesmo.
O `PainterTool` e o `Engine` não são `Clone` (`tool/mod.rs:116`, `ph2d-wet-paint/src/painter.rs:85`): o tamanho e
o custo da cópia medem-se na W5c, ANTES de qualquer tecto de memória.

### 3.1 Os instantâneos — MEDIDO (W5c, 14/10, smoke)

**A cópia** = `PainterTool::instantaneo` (`tool/instantaneo.rs`, padrão sem `..`): pixels das camadas por `Arc` (a
1.ª escrita do original desdobra UMA camada), o resto `derive(Clone)` (52 tipos); à mão só o recurso vivo —
`EngineSlot` (`Away` pânico nomeado), `WetSession` (o worker não viaja), a fita (cópia PROFUNDA), recusa a gravar /
com worker / com ponte de dispositivo. O `SolverScratch` é «derivado, não estado» (`grid/scratch.rs`): a cópia
leva-o VAZIO.

| | retido por instantâneo (dhat) | B/px | declarado ao razão | retido ÷ declarado | tirar um, água molhada |
|---|---:|---:|---:|---:|---:|
| Splash 1024², cópia do scratch (1.ª versão) | 85,0 MB | 85 | — | — | — |
| **Splash 1024²** | **60,2 MB** | 60 | 55,3 MB | 1,089 | **3,9 ms** |
| **Splash 2048²** | **228,3 MB** | 57 | 218,2 MB | 1,046 | **14,3 ms** |
| worker 1024² (fita) | 64,4 MB | 64 | 60,9 MB | 1,057 | — |
| worker 512² (fita) | 17,3 MB | 69 | 16,0 MB | 1,082 | — |

**O tecto** é o razão da RAM, `ph2d_core::razao` (a lei do doc 132 §7 P2 passou do Motion para o core: `1/32` da
RAM física por FAMÍLIA de cache, partilhado pelo processo) — `macro_track::instantaneos::razao()`; reserva sem
leitura da RAM = 256 MiB (o `1/32` de 8 GiB, cabe um a 2048²). Nesta máquina (123 GiB) ≈ 3,85 GiB ⇒ ~64 instantâneos
a 1024², ~17 a 2048²; a 8 GiB: 4 a 1024², 1 a 2048². Passou do tecto, a macro DOBRA os seus (larga um em cada dois e
tira de `2·passo` em `2·passo` etapas): a cobertura fica uniforme.

**O preço e o ganho** (mínimo de 5 rondas intercaladas, `sonda_instantaneos`, load 13–17): tocar o Splash até ao
fim COM instantâneos 1 158 ms × sem 1 169 ms (1024²), 2 603 × 2 481 ms (2048²) — dentro do ruído; editar a ÚLTIMA
pincelada: do instantâneo 866 ms × do zero 1 150 ms (1024²), 1 990 × 2 446 ms (2048²). O ganho no Splash é pequeno
porque ~4 s de água a secar vêm DEPOIS da última pincelada e re-simulam-se sempre; numa macro longa o ganho é o
prefixo inteiro. O que resta do custo é a água (W5e: a reprodução para editar pode saltar as composições).

**Reeditar** (`MacroEmCena::reedita`, o palco da cena quando o `Macro` muda): ficam os instantâneos cujos passos
despachados são prefixo comum das duas trilhas (mesma inicial e alvo) E cujas respostas do worker já consumidas
são as mesmas; a fila da fita refaz-se da trilha nova. Uma espera ENCURTADA tira relógio antes da etapa: o
instantâneo dela cai e recomeça-se do anterior (correcto, uma etapa a mais de reprodução).

## 4. Formato

`events` = a gravação bruta, IMUTÁVEL (uma verdade). Etiqueta nova por trilha `edicoes`: deslocamentos, trocas
de carga, nomes, relógio fixo desde — indexados pelo índice do 1.º evento da etapa NA GRAVAÇÃO (estável). O doc
efetivo é função pura, calculada ao ler. Etiqueta `exige` no cabeçalho: um leitor recusa com frase uma
funcionalidade que não conhece. MINOR 0 → 1. ⚠️ Um leitor 1.0 (o `main` de 14/10) tocaria a gravação sem as
edições, calado — hoje vazio (nenhum ficheiro de utilizador: só as cenas criam macros); o `exige` fecha os
seguintes. Suíte com `motivo: formato gravado`.

## 5. UI

- **`ph2d-panel-macros`** (DIRETRIZ §3.B.1): Gravar ▸ Parar (as duas famílias; `macro_abre` do Painter passa de
  INTERNA a pública com decisão no censo), a lista de etapas (molde `ph2d-panel-hierarchy`: `paint.rs:319,388`,
  `event_rows.rs:16-112`), as propriedades da selecionada (`labeled_number_field` + `set_number_range`, molde
  `ph2d-panel-skeleton/src/paint.rs:70-80`; a amostra de cor de `hierarchy/row.rs:235`), renomear, apagar, o erro
  (5.6). Clicar numa etapa = `TimelineIntent::Scrub(L)` com L = o FIM da etapa no tempo local do clipe
  (`apply_one.rs:117-122`: com Autoplay o tempo local é `clip_local_time(t)`).
- **A linha de etapas na Timeline:** campo novo no snapshot (hoje só o clipe ativo tem `tracks`,
  `snapshot.rs:~268`; os markers são da cena, `snapshot.rs:264`); losangos pintados como `paint_lane_keys`
  (`ph2d-panel-timeline/src/tracks.rs:285`); arrastar emite um evento lido pela família (ADR-0075), nunca uma
  chamada.
- **Undo:** o `Macro` entra pelo diff do mundo (`undo_app.rs:334`); o clipe pelo histórico PRÓPRIO da Timeline
  (`BeginEdit/EndEdit`, `intent_apply.rs:348`) — duas pilhas: medir na W5d o que o Ctrl+Z faz com as duas.
- **Cartão Macro no Inspector** (molde `a777f2f541`, Motion Input): modo, «editada desde a etapa N», erro.

### 5.1 W5d — o que se mediu e decidiu (14/10)

- **Undo medido (antes):** o clipe não entrava em pilha nenhuma; o Ctrl+Z global tirava a Macro e deixava o clipe;
  com o painel da Timeline aberto o Ctrl+Z vai à pilha dela primeiro (`keyboard_timeline.rs:41-49`), antes do
  Painter e do Global. **Decisão** (lei do doc ObjectsAndComponents §12: o undo dos clipes é o da Timeline, uma
  verdade só): o clipe é DERIVADO e o Parar (criar/re-cronometrar o clipe) é UM passo da pilha da Timeline; a
  Macro é do diff do mundo. Estados intermédios inofensivos (clipe sem macro não toca; macro sem clipe não corre).
- **Duração do clipe derivada:** `Macros::sincroniza` re-cronometra a cada quadro enquanto o clipe tiver a forma
  do Parar (`TimelineDoc::macro_clip_identity_end`: faixa Macro Time identidade (0,0)→(D,D) linear, duração = D,
  tolerância relativa ao f32); câmara lenta do artista não é re-cronometrada (gate com controlo). D = a duração da
  trilha QUE TOCA (`instala::segundos` pela `tocavel`). ⚠️ `MacroDoc::duration` lê a gravação imutável: uma espera
  não a muda — o gate apanhou (o clipe ficava em 0,87 s).
- **Conferência adiada:** a edição de VALOR do Painter corre o `assenta` no 1.º quadro sem edição nova (o selector
  manda uma cor por quadro; conferir custa tocar a macro inteira). O custo numa macro longa NÃO está medido (W5e).
- **Gravadora:** `ph2d_macro::Gravadora` (abre/passos/para), implementada pelo `PainterTool` (alvo 0 = a sprite
  da Macro) e pelo `MotionState` (o objecto em edição). `macro_abre/passos/para` do Painter passaram de internas
  a PORTAS com `Decisao::Ambiente` no censo (controlam a gravação, não são passo da obra). O Parar chega sempre
  ao Painter (`Familias.painter`), mesmo com outra ferramenta activa; só o armar exige `painter_activo`.
- **Recusa de escala:** arrastar a escala de uma etapa sem faixa — a família recusa e o painel diz.
- **Apagar uma etapa:** decidido pelo dono em 15/10 — «o tempo dela fecha-se»; feito na W5e (ver §7).

## 6. Ondas

| Onda | O quê | Gates (vermelho primeiro + mutação) |
|---|---|---|
| W5a ✅ | medição (§3) + `passa_ao_relogio_fixo` | sondas `#[ignore]` |
| W5b ✅ (`9b40399255`..`95ca6db68e`) | núcleo + Painter + Motion; mutação 21/21 | [handoff W5](handoffs/HANDOFF_CONTINUACAO_W5_ETAPAS_2026-10-14.md) |
| W5b | segmentação + nomes + `edicoes`/`exige` + deslocamento com relógio sintetizado + edição de valor | cena `=2` = N etapas, Splash = 4; antes da etapa editada = original ao bit; espera = a água/o vento continuam; recuar == do zero com edições; formato ida-e-volta, 1.0 lê |
| W5c ✅ (`ff001e1bc7`..`8c92e9cd19`) | instantâneos do Painter por etapa (§3.1), razão da RAM no core, o palco reedita | continuar = do zero ao bit; recuo e edição = palco novo ao bit; dobra; fita reeditada; dhat retido ÷ declarado ∈ [1; 1,15] |
| W5d ✅ (`d21a072d6b`..`a3475fb6fb`) | painel, linha de etapas na Timeline, Gravar ▸ Parar, cartão, undo; mutação 18/18 (§5.1) | `ph2d-app-macros --test it gate7` (4): Gravar → pincelar → Parar ⇒ Macro + clipe + N etapas; arrastar ⇒ etapa move; editar cor ⇒ só aquela pincelada; undo das duas pilhas |
| W5e ✅ (`024b153672`..`dc3d49d001`) | apagar etapa; composição 1× por quadro + subida parcial; conferência fora do quadro, do instantâneo; os dois riscos do smoke; cenas com etapas (§7) | `ph2d-macro --test it etapas::apagar_uma_etapa_fecha_o_tempo_dela` · `ph2d-tool-painter --lib macro_track::etapas_tests::apagar_uma_pincelada_e_grava_la_sem_ela`, `macro_track::composicao_tests` · `ph2d-tool-painter --test it macro_etapas::conferir_do_instantaneo_do_palco_decide_o_mesmo_que_do_zero` · `ph2d-app-motion --lib macro_track::etapas_tests::apagar_um_gesto_e_grava_lo_sem_ele`, `…leitor::apagar_um_arrasto_fecha_os_quadros_dele_e_o_relogio_de_dentro_nao_salta` · `ph2d-app-painter --lib macro_track::palcos_tests::a_ranhura_recebe_so_o_que_mudou_e_fica_igual_a_tela` · `ph2d-app-macros --test it gate7` · `ph2d-panel-macros --test it seam_macros::apagar_a_etapa_escolhida_chega_como_pedido_e_larga_a_escolha` |

## 7. W5e — o que se mediu e decidiu (15/10)

**Apagar** (dono, 15/10: o tempo fecha-se): `EdicaoDeEtapa.apagada` (codec tag 7, dentro de «etapas» 1.1 — nenhum ficheiro 1.1 saiu da linha). `efetiva` salta os eventos da etapa (o relógio de dentro também); o que vem depois recua `t[fim]−t[inicio]`; sai da gravação ali. As outras edições da etapa apagada deixam de valer; inserções de OUTRAS etapas num evento apagado tocam onde ele fechou. As etapas vivas levam o `k` da GRAVAÇÃO (uma apagada não renumera; o Painter numera «Pincelada n» pela gravação).
- Motion: `LeitorDaFamilia::sem_o_gesto` — o `Quadro` do instante do fim de uma etapa apagada passa a `sujou:false` (sem isso o Fiel diverge: medido). `tempo_deslocado` encaixa o `tempo` na grelha do `dt` quando o desvio cabe no arredondamento a µs (fechar 4 quadros tira 66 667 µs: 0,33 µs de deriva, o gate apanhou).
- UI: botão «Delete step» no painel MACROS (`MACROS_DELETE`), `MacroRequest::Apaga`; undo pelo diff do mundo, o clipe encurta pela sincronização.

### 7.1 Composição e subida — MEDIDO

Sondas `custo_w5e`, `sonda_n_palcos_do_painter`, `sonda_subir_a_tela`; perfil smoke, mínimo de rondas intercaladas; load antes 23–29, depois 4–55. ⚠️ A sprite branca do PRODUTO abre com papel branco (`abre_a_sprite`) ⇒ composição completa; o Splash de antes era `set_source` (pilha trivial, composição grátis) — `roteiro::grava_no_papel` grava pela porta do produto e a cena `=1` passou a usá-la.

Antes: cada `Preview` da reprodução compunha (parcial) e a leitura da cena forçava uma composição INTEIRA (`composto_da_cena` sem retângulo). Agora: `PainterTool::drena_sem_compor` (os mutadores do dreno — papel, composição adiada do Composite, vidro — correm igual; o retângulo sujo acumula) e a cena compõe UMA vez sobre o retângulo acumulado; `caixa_da_tela`; o palco sobe só esse retângulo (`RanhurasDaCena::substitui_regiao`).

| | antes | depois |
|---|---|---|
| Splash no papel 2048², cena: segue + compor por tela nova | 12,8 + 18,2 ms | 10,1 + 8,7 ms |
| Splash no papel 1024², idem | 5,3 + 3,5 ms | 4,2 + 1,9 ms |
| tocar do zero ao fim, Splash no papel 1024² / 2048² | 2 043 / 3 622 ms | 1 102 / 2 441 ms |
| tocar do zero, longa 61 s no papel 1024² / 22 s a 2048² | 26,6 / 17,7 s | 22,9 / 13,5 s |
| subir à placa 2048² (inteira / 25 % / 5 %) | 8,53 / 2,49 / 0,49 ms | — |
| subir à placa 1024² (inteira / 25 % / 5 %) | 1,73 / 0,53 / 0,26 ms | — |
| palco no papel com a placa, 2048² N=1 (segue + sobe) | — | 4,0 + 3,8 ms |

Área média sujada por pré-visualização (antes): 28–47 %; 48 de 111 subidas no gate do palco são inteiras (a ferramenta invalida a cache). Verdade dos gates: a composição de uma vez = a completa ao bit em cada quadro; a ranhura parcial = a tela premultiplicada em cada quadro. ⚠️ A cena mostra o estado no FIM do quadro (depois do `Tique`); o editor mostrava-o drenado ANTES do tique do quadro — uma régua «a cena = o que o editor mostrava» diverge 368 texels no quadro 26 COM O CÓDIGO ANTIGO TAMBÉM (medido): não é a régua desta mudança.

### 7.2 Conferência fora do quadro — MEDIDO

Antes tocava a macro inteira DENTRO de um quadro (3,6 s no Splash no papel 2048²; 26,6 s na longa de 61 s). Agora `conferencia.rs` (app-macros): DERIVADA da `Macro` (tique de mudança + hash dos bytes), num fio «macro-confere», no 1.º quadro sem mudança nova, a partir do instantâneo do palco em cena (`MacroEmCena::partida`, `limite_da_partida`: nunca depois da 1.ª etapa editada); escreve só se a `Macro` ainda for a conferida; uma mudança a meio pára o fio; a decisão fica pelos bytes e os mesmos bytes que voltam (desfazer/refazer) recebem-na já. O undo suprime o passo enquanto há conferência («uma macro a conferir», molde do Colorize).

| conferir a cor | do zero | do instantâneo | tirar o instantâneo (no quadro) |
|---|---|---|---|
| longa 61 s, 1024²: pincelada 49/96 | 25,5 s | 16,3 s | 6,6 ms |
| longa 61 s, 1024²: pincelada 96/96 | 24,5 s | 3,0 s | 7,9 ms |
| longa 22 s, 2048²: 17/32 | 16,3 s | 11,3 s | 20,6 ms |
| longa 22 s, 2048²: 32/32 | 19,4 s | 7,6 s | 16,8 ms |

O que sobra é a água (a simulação re-corre: `Tique` = 70–80 % de tocar do zero).

### 7.3 Os dois riscos do smoke (investigados)

1. No extract o palco vence a pré-visualização do editor (os palcos antes do `do_editor` em `fase_extract_inputs`; `sim_extract` fica com o 1.º) — uma 2.ª tomada sobre uma sprite com macro mostrava a tomada antiga enquanto o artista pintava ⇒ `Macros::sprite_gravada` + `sobe(.., cede)`: a sprite que o Painter grava mostra o traço vivo; o palco nunca escreve no documento nem partilha ranhura com a pré-visualização. Pintar SEM gravar depois do Parar continua escondido enquanto o clipe toca (por desenho: a sprite mostra a macro).
2. O Motion grava um `Quadro` por quadro do editor e a testemunha apanha escritas fora das portas, mas a Sessao drenava-o na `fase_macros`, ANTES de o Motion correr ⇒ cada passo levava o relógio do quadro seguinte ⇒ `Macros::depois_do_motion` depois da `fase_motion_bridge`.

### 7.4 ⛔ Recusas MEDIDAS

| Recusado | Porquê |
|---|---|
| «a cena = o que o editor mostrava» como régua da composição | fase do tique; diverge igual no código antigo (368 texels, quadro 26) |
| conferir dentro do quadro | 3,6–26,6 s num quadro |
| conferência disparada só pelo pedido de edição (`por_assentar`) | um desfazer/refazer devolvia bytes não conferidos — agora derivada dos bytes |

### 7.5 Gates W5e (todos verdes, por nome)

Os da coluna Gates da §6 (W5e). Cobrem: apagar fecha o tempo (núcleo, Painter, Motion, leitor); composição 1× por quadro; conferir do instantâneo = do zero; a ranhura recebe só o que mudou (+ o palco cede só a sprite gravada); `gate7` (apagar encurta o clipe; conferida fora do quadro; a decisão é dos bytes; os passos do Motion no relógio do quadro; `sprite_gravada`); o painel manda `Apaga` e larga a escolha.

**Prova de mutação W5e (15/10, agente `mutacao`, 3 rondas): 1.ª ronda 15/20; fechadas → todas vistas a
sangrar.** Sobreviventes e cura: **M13** (refazer a fila da reserva na 2.ª volta — equivalente: a marca de
saída não é resposta do worker ⇒ saiu do código); **M14** (a guarda de hash no colhe — equivalente: a mudança
da `Macro` pára o fio na leitura dos tiques ⇒ saiu); **M12/M22** (o limite da partida — NÃO redundante: uma
edição cujo desvio aparece pinceladas depois deixa o palco com instantâneos de depois da etapa editada, e a 2.ª
volta arrancaria de um estado que não é o da trilha com a marca ⇒ o limite passou para DENTRO de `confere`,
gate com uma partida tardia); **M8** (o estado do dreno ⇒ UMA função `drena_o_estado` com dois chamadores;
sangra em `substrate_relief_tests::raising_the_relief_repaints_without_a_stroke`); **M10** (o recorte da
subida ⇒ `recorta` com gate fora do canto). Achado no caminho: um NOME numa etapa anterior tirava-a da gravação
(`primeira_que_toca`; gate em `macro_etapas`). G2 (Painter) não vê o tempo nem a marca de saída nos píxeis —
o núcleo (G1) cobre-os.

Tutorial «fazer» (balão, realce, cena `=3`): W6.
