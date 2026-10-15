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

## 6. Ondas

| Onda | O quê | Gates (vermelho primeiro + mutação) |
|---|---|---|
| W5a ✅ | medição (§3) + `passa_ao_relogio_fixo` | sondas `#[ignore]` |
| W5b ✅ (`9b40399255`..`95ca6db68e`) | núcleo + Painter + Motion; mutação 21/21 | [handoff W5](handoffs/HANDOFF_CONTINUACAO_W5_ETAPAS_2026-10-14.md) |
| W5b | segmentação + nomes + `edicoes`/`exige` + deslocamento com relógio sintetizado + edição de valor | cena `=2` = N etapas, Splash = 4; antes da etapa editada = original ao bit; espera = a água/o vento continuam; recuar == do zero com edições; formato ida-e-volta, 1.0 lê |
| W5c | instantâneos do Painter por etapa; relógio fixo decidido ao editar | instantâneo == tocar do zero ao bit; editar a k só muda o que vem depois |
| W5d | painel, linha de etapas na Timeline, Gravar ▸ Parar, cartão | gate 7 (ui-testkit): Gravar → pincelar → Parar ⇒ componente + clipe + N etapas; arrastar ⇒ etapa move; editar cor ⇒ tela muda |
| W5e | cenas `=1`/`=2` com etapas, fotos com a máquina calma, perf (compor 1× por quadro, premultiplicar/subir), fecho | fecho batched, handoff, binário do smoke |

Tutorial «fazer» (balão, realce, cena `=3`): W6.
