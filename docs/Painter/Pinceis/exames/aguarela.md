# Exame da família `aguarela` (J4–J5, catálogo I — aguarela e sumi-e)

Receitas: `crates/ph2d-tool-painter/tests/it/familias/aguarela.rs` (`WET`; os reprovados em `EM_ESTUDO`, sem ficheiro);
portões em `familias/aguarela/{portoes,molhado}.rs`; estudos `familias/aguarela/estudo.rs` (J5: lê também as receitas do
guache e do óleo). Pranchas `pranchas/aguarela/` (+ `cegas/`, `chaves/`). Oráculo: Rebelle `rebelle_bateria/WATERCOLOR/`.
⚠️ **O histórico da J4 (I-1…I-13, o motor do filme, as três rondas, as recusas medidas) está em `96868e656e`**
(`git show 96868e656e:docs/Painter/Pinceis/exames/aguarela.md`) — consulte as recusas lá antes de propor uma cura.

**Estado (J5, 22/10):** aprovados nenhum · curados não aprovados **Wet Liner, Wet Wash** (com a lei nova), Wet Flat,
Wet Round, Wet Drip, **Wet Dry Brush (fica — não se funde, abaixo)** · reprovados em estudo Mop, Bristle Watercolor,
Sumi-e, Granulation, Fringe Bloom, Spatter Watercolor, Optical Wash.

## Motor J5 — molhado × molhado (`Knob::WetOpen` + `Knob::WetOpenPush`, `crates/ph2d-wet-paint/src/trail/molhado.rs`)

**A medição** (cena `sobre`, foto A; régua `blot.py` do scratchpad, a mesma conta no portão `mancha()`): a largura da
mancha no meio da faixa molhada ÷ a do vertical no papel, e a escuridão do miolo ÷ a do azul no papel.

| | Rebelle | nosso ANTES (parede) | sem parede | **nosso DEPOIS** |
|---|---|---|---|---|
| Liner | 1,42 · 1,06 · [98,123,175] | 1,00 · 1,27 · [75,97,155] | 1,16 · 1,24 | **1,51 · 1,06 · [106,124,173]** (0,42 · 0,5) |
| Wash Round | 1,17 · 1,30 | 1,00 · 1,84 | 1,07 · 1,78 | 1,39 · 1,40 (0,15 · 0,4; a régua conta o halo) |
| Mop | 1,44 · 1,06 | 0,98 · 1,68 (estudo) | — | 1,75 · 1,13 (0,42 · 0,8) |
| Gouache Round | 1,25 · 1,00 | 1,27 · 1,11 (anel 1,14) | — | não serve (abaixo) |

A composição NÃO o exprimia (medido: `extDiffusion` 0,25, `extBackrun` 2, as duas, sem a parede — o quadrado fica).

**A lei** (desligada por omissão; os presets que não a ligam ficam ao bit — fingerprint `ph2d-wet-paint` e
`a_lei_da_tinta` verdes): (1) a cada dab a sonda olha `r + 2` à frente da pegada; na 1.ª vez que cai na tinta molhada de
OUTRA cor (`outra_tinta`), mede pela linha do traço a faixa `[ta, tb]` e guarda o disco no meio dela, de raio
`min(meia faixa, r·(1 + WetOpen))`; e empurra pela normal a fracção `WetOpenPush·(1 − (s/0,8·meia)²)` da tinta de baixo
para a orla molhada fora do disco (um destino por píxel até 2 × o raio do disco, pesos em triângulo, bilinear); (2) cada janela que
pousa dentro do disco estica o seu pigmento e a água pela normal (8 destinos, cauda em triângulo, tecto 1,3 × a meia).
⛔ **Recusas medidas nesta janela:** destinos discretos da orla (anéis concêntricos); orla de 1–4 px (crescente escuro);
esticar sem tecto (o Mop: um olho de 95 px contra 59); pesos em trapézio até ao disco (núcleo rectangular, tampas
pálidas — o pouso é pela opacidade da massa que chega); esticar na GRADE depois de o traço passar (pela massa o vermelho
pesado engole o azul: 1,05; pela opacidade 1,30); deixar a água no lugar (manchas e contorno).
**Portão** `familias::aguarela::molhado::o_azul_abre_uma_mancha_redonda_no_vermelho_molhado` (o Wet Liner de fábrica;
controlo `wetOpen` 0 — < 1,1 e escuro > 1,18). Red-first e mutação: M0 lei desligada → VERMELHO (mancha), M1 sem
empurrão → VERMELHO (escuro somado), M2 sem esticão → VERMELHO (mancha); restaurado, verde. Custo: Liner p95 4,67 ms ·
Wash 5,90 ms a 200 px, RSS ≤ 100 MiB (load 11–5; smoke).

## Os pincéis (ronda do motor novo; revisores cegos `sonnet`, um por ronda)

- **Wet Liner** — receita J4 + `MANCHA_DO_LINER` (0,42 · 0,5). Régua 419 → **430/454**; vizinho Hard Round **0,725**
  (≥ 0,596). Revisor m1 (`cegas/wet_liner_j5.png`): tomou o NOSSO pelo original nos gestos (baixa), achou o Rebelle nos
  tamanhos e no «sobre» (4/6) e lista: a 200 px a borda difusa e o miolo carregado dele contra o nosso chapado de borda
  dura (FILA 2), os anéis das voltas da espiral, a nossa mancha do cruzamento mais escura, redonda e com halo (a dele mais
  clara, de cantos quadrados), o vermelho seco mais denso. Já não lista o arrasto nem o quadrado. ⇒ **CURADO, NÃO
  APROVADO** (a lei fica na receita: a régua e o cruzamento melhoraram).
- **Wet Wash** — receita J4 + `MANCHA_DO_WASH` (0,15 · 0,4). Régua 373 → **384/454**; vizinho Glaze Round 1,16. Revisor
  m2 (`cegas/wet_wash_j5.png`): o Rebelle 5/6 (baixa/média): o nosso contorno escuro (orla 100), a rampa que entra tarde
  (decisão 9), a nossa mancha nítida em losango contra a difusa dele, as pontas em cápsula. ⇒ **CURADO, NÃO APROVADO**.
- **Mop** (estudo; receita + 0,42 · 0,8) — régua 390 → 383/454; vizinho Wet Wash 0,797. Revisor m3 (`cegas/mop_j5.png`):
  o nosso tomado pelo original nos gestos e nos tamanhos (3/6); o Rebelle no «sobre» (alta): a nossa mancha redonda e
  escura de aro nítido («um carimbo») contra o véu difuso dele que arrasta o vermelho; a faixa vermelha dele manchada.
  ⇒ **REPROVADO ainda** (a lei NÃO entra no `EM_ESTUDO`: a régua desce).
- **Wet Dry Brush → NÃO se funde no Wet Flat.** Medido na matriz: o 1.º vizinho é o Airbrush a **1,236** (Wet Flat
  longe; nenhum pincel da biblioteca o pinta); só a CURA para o Rebelle `Flat Dry` (308/454) virava o Wet Flat (F1 6/6).
  Fundir tiraria o único pincel de riscos soltos e mexia em 10 ficheiros partilhados. Fica como estava, curado não
  aprovado contra a origem (régua 220/454), com a razão de uso: o pincel seco de riscos soltos.
- Flat Bristle (óleo): ver `exames/oleo_acrilica.md` — a lei não se aplica (o Rebelle `Flat` não abre mancha, 0,86).

## Motor J6 (28/10) — a borda da mancha e o domo do filme (`8394f62eed`, `5f6ae47051`)

Réguas (scratchpad J6 `borda.py`/`perfil.py`; as mesmas contas nos portões `borda()`/`domo()`): na cena `sobre` A, y 145–155,
o **degradê** = células entre 80 % e 20 % da distância de cor do miolo da mancha à faixa longe, o **aro** = escuridão
máxima na banda da borda ÷ miolo; no recto de 200 px da `tamanhos` A, a escuridão a 5/10/20 % da largura ÷ o meio.

| | Rebelle | nosso antes | **nosso J6** |
|---|---|---|---|
| Liner — degradê · aro · mancha | 12,2 · 1,10 (o vermelho empurrado, por fora) · 1,42 | 2,8 · 1,16 · 1,51 | **12,5 · 1,035 · 1,46** (pena 28) |
| Wash Round — degradê · aro | 40,7 · 1,03 | 2,0 · 1,28 | pena 80: 45,7 · 1,09, mas ver abaixo |
| Liner 200 px — 5/10/20 % | 0,89 · 0,91 · 0,94 (40 px chato) | 0,98 · 0,99 · 1,00 | **0,88 · 0,92 · 0,96** (domo 0,3), 40 px chato |
| Wash 200 px | 0,79 · 0,85 · 0,92 | 0,95 · 0,99 · 1,00 | **0,76 · 0,86 · 0,93** (domo 0,3) |

**As leis** (aditivas; 0 = ao bit — a cena `a_lei_da_tinta` dá o mesmo hash com e sem o motor, A/B no mesmo processo):
`Knob::WetOpenEdge` (pena, células): a célula da borda do tubo (`(s/r)²`) reparte-se em 15 destinos por uma rampa de
largura `pena` CENTRADA na borda (recua `min(pena/2, 0,45·raio)`), com a fase dos destinos sorteada por célula do grid;
o levantar do vermelho alarga a rampa do mesmo tanto. `Knob::StrokeFilmDome`: o alvo do filme de cada dab é
`1 − d·(1 − 20/r)⁺·u²` (r em células; até 20 o filme é chato). ⛔ Recusas medidas: pena para os DOIS lados (a metade de
dentro soma-se ao miolo: dois riscos escuros); só para FORA (mancha 2,0 × a largura); 5 destinos (riscas a 6 células);
fase fixa (riscas ao longo do traço, 1,56 contra 0,84). Portões: `a_borda_da_mancha_molhada_esbate`,
`a_pena_larga_nao_risca`, `o_filme_largo_tem_o_miolo_carregado_e_os_lados_em_degrade`; mutações (pena 0, levantar
sem pena, RECUO 0, fase fixa, domo 0, raio chato 0) todas VERMELHAS. Censos W3b verdes (os knobs não mudam tinta pousada).

## Os pincéis J6 (receitas em `CURAS_J6` e no `EM_ESTUDO`; NÃO aplicadas à fábrica até à revisão cega)

Biblioteca fixa (65 presets pintados uma vez em `/tmp/claude-1000/j6_wet/biblioteca` — a chave do `exame.sh` muda a cada
edição dos outros agentes nas crates e o Óleo Real leva ~6 min por preset). Pranchas `pranchas/J6_<nome>{,_cega}.jpg`,
chave `J6_<nome>_chave.txt`.

- **Wet Liner** — + pena 28 + domo 0,3. Régua 429/454 (J5 430); vizinho Soft Pastel **0,853**; 1.º dos oráculos
  (Liner 0,677). Fica: as pontas do 200 px (o Rebelle esbate 60–120 px nas duas pontas; o domo não chega lá) e um leve
  clarão do levantar abaixo da faixa.
- **Wet Wash** — só o domo 0,3 (régua 384/454 = J5). ⛔ A pena NÃO serve ao Wash: com a orla 100 a frente da água
  esticada desenha fios escuros ondulados na mancha (pena 28: aro 1,46) e a pena larga levanta o vermelho numa caixa
  branca (pena 60–80). A mancha difusa dele (que passa a faixa) é outro motor. Vizinho Scratch 1,43.
- **Mop** (EM_ESTUDO) — + mancha 0,42 · 0,8 + pena 28 + domo 0,1: régua 382/454 (J5 383); 2.ª ronda (empurrão 0,5,
  domo 0,3) 371 — fica a 1.ª. Vizinho Wet Wash 0,771. Na prancha a mancha ainda tem a barra escura nos bordos da faixa.
- **Custo** (200 px, bateria, smoke) só sob load 18–35: p95 Liner 60 · 17 ms contra o de fábrica 54 · 56 na MESMA
  ronda intercalada — sem diferença mensurável; RSS 102 MiB. O relógio calmo fica por medir (J5: 4,7 ms).

**2.ª ronda (cega sonnet: o Rebelle achado nos tamanhos, «borda esfumada; a imitação chapada, contorno duro»):** a pena
não entra num traço sobre papel; através o domo já casava; o que diferia era a TAMPA → `Knob::StrokeFilmEnds` (`5cd611289a`):
Wash 200 px ponta 0,94 → 0,83 do meio (Rebelle ~0,5), o corpo igual; no Liner quase nada (a tinta dele satura). Receitas
Liner + tampas 0,8 (régua 429), Wash domo + tampas 0,8 (384). Fica: o fio da orla 100 do Wash (orla 1,6 contra 1,15) e a
rampa da ponta que no Rebelle entra ~60 px no corpo. Pranchas cegas refeitas (nova ordem e chave). Mop: REPROVADO.

## Para a fila (coordenador)

1. As pontas do traço de 200 px do filme: o Rebelle esbate 60–120 px nas DUAS pontas (Liner 0,70 → 0,99 do meio); a de
   saída pede saber onde o traço acaba (motor). O domo (J6) só dá os lados e a tampa.
2. A mancha do Wash e do Mop: a frente da água esticada com orla (fios escuros) e o véu difuso que passa a faixa.
3. As voltas da espiral do Liner; o recto de 5 px do Mop; o Sumi-e (cerdas em pente); granulação; franja; escorrido.
4. O Wet Round com pigmento 125 (régua 297 → 419): só com o `painter_default_knobs` e o controlo das MIDIAS.
