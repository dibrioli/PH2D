# R1 — relatório (janela reserva 01, ramo `line/PainterWatercolor-r1`, base `6ba5d06493`, 31/10)

Missão (coordenador `ph2d-f2`, «superar o Rebelle»): (1) borda molhada esbatida — Wet Liner, Wet Wash, Mop; (2) fio do
arrasto — Smudge Finger; (3) se sobrar: Water Brush, Fan Brush, Gouache Dry/Wash. Método: simular antes de construir,
cena `tamanhos`/`sobre`/`gestos` A da bateria, oráculo `fixtures/rebelle_bateria/WATERCOLOR`, máx. 4 variantes por ronda.

## 1. Borda molhada — Wet Wash e Wet Liner: CURADOS por valores (fábrica regerada)

**Diagnóstico medido** (ampliações em `pranchas/r1_Wet_Wash.png`): a 200 px o Rebelle tem a borda LATERAL nítida com um
aro fino e pontilhado, e a TAMPA esbate ~40 px SEM aro; o nosso desenhava um aro contínuo (1,34–1,84 × o miolo) e fios
escuros ondulados na tampa. Com `edgeDarkening 0` a tampa fica igual à do Rebelle mas o aro lateral some ⇒ o aro vem do
escurecimento da borda; a orla baixa (50) dá o aro do Rebelle. O cruzamento azul × vermelho molhado saía QUASE PRETO e
quadrado: é o K-M ligado de fábrica (J11) — com `pigmentMixing 0` no preset sai a mancha lilás macia do Rebelle.

Perfil através do recto de 200 px (cena `tamanhos` A, colunas 40–60 %, ÷ miolo; aro · +4 · +6 · +20 · +30 px):

| | Rebelle | fábrica antes | **R1** |
|---|---|---|---|
| Wash Round | 1,15 · 0,71 · 0,74 · 0,84 · 0,89 | 1,34 · 0,88 · 0,92 · 0,99 · 1,00 | **1,27 · 0,73 · 0,72 · 0,81 · 0,85** (orla 50, domo 0,5) |
| Liner | 1,06 · 0,89 · 0,87 · 0,92 · 0,93; miolo 0,479 | — chapado, aro duro; miolo 0,43 | domo 0,15: 0,91 · 0,85 · 0,89 · 0,97; miolo 0,461 → fica o domo 0,3 da J6 |

Variantes medidas (4 rondas): orla 0/30/50/100 × domo 0,3/0,5 × tampas 0,8/1 × `pontas_esbate` 0/1/2 × K-M; ⛔ recusadas:
orla 30 (sem aro: 0,89) · orla 100 com tampas (fios escuros na tampa) · domo 0,5 no Liner (0,65 junto da borda) ·
`wetOpenEdge 28` no Wash (quadrados de borda no cruzamento, J6 confirmado) · `pontas_esbate` no Liner (a 40 px o Rebelle
é cheio, J6t).

**Receitas** (`familias/aguarela.rs`, `WET`): Wash = orla 50 + domo 0,5 + tampas 1 + K-M off + `pontas_esbate 2`;
Liner = pigmento 20 + orla 100 + `wetOpenEdge 28` + domo 0,3 + tampas 1 + K-M off. `CURAS_J6` esvaziada (aplicada).

**Fica (motor, PEDIDO ao coordenador):** a tampa do Wash ainda tem um contorno fino irregular que o Rebelle não tem. Causa:
`drying/edge_window.rs` conta vizinhos com `susp > 10` BINÁRIO ⇒ o aro nasce na isolinha 10 mesmo onde a tinta esbate.
Lei proposta (a «transição de ~10 px», ao bit com 0): fator de borda GRADUADO, cada vizinho conta `min(susp/limiar, 1)`.
Toca `drying.rs` (reservado ao agente de performance).

## 2. Mop — REPROVADO de novo (motor)

Rondas M1–M4 (receita `EM_ESTUDO` + tampas 1, K-M off, pigmento 7,3 — o miolo 0,166 → 0,250, Rebelle 0,235 — empurrão
0,8/0,3/0, mancha 0,42/0,2): no cruzamento sobre o vermelho molhado fica um ANEL escuro e um rasto branco no traço azul
que nenhum valor tira (com empurrão 0 e pena 0 igual) — é o cruzamento molhado da água muita (`waterPerDab 1,5` + parede),
não a borda. O Rebelle: manchas redondas macias e o traço largo mosqueado. Fila de motor.

## 3. Smudge Finger — tecto do alcance SIMULADO e RECUSADO

Prancha `pranchas/r1_Smudge_Finger_alcance.png` (sobre B, coluna x 180–330; ampliações da língua e da faixa de baixo).
⚠️ O diagnóstico herdado («a língua do Rebelle é CHEIA, a nossa um U oco») não se confirma na ampliação: o Rebelle TAMBÉM
abre um sulco branco na faixa e as paredes convergem num fio. A diferença real é o FIO: o do Rebelle é UMA linha cheia,
o nosso DUAS linhas finas com o meio branco (massa 1,25 × 2,14, J6). Construído e medido (o dono autorizou o campo via
coordenador): `BrushSpec::smear_alcance` (tecto do `disp` por pincel) + a tinta arrastada POR CIMA da que lá estava
(`over`, para não abrir entalhe). Alcance 4/6/8/12 raios: o sulco acaba num fundo CHATO abrupto e o fio entre as faixas
corta-se em pedaços — pior que hoje. ⇒ revertido inteiro (nenhum ficheiro do pincel mudou). O que falta é juntar as duas
paredes do fio num só traço (o eixo amostra o papel do início do traço) — outra lei, por medir.

## Commits

(preenchido no fecho)
