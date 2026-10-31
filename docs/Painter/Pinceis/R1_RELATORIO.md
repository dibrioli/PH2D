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

**Receitas** (`familias/aguarela.rs`, `WET`): Wash = orla 40 + `edgeSoftMass 4` + domo 0,5 + tampas 1 + K-M off + `pontas_esbate 1` (2 = pontas finas cortadas a direito, tubo; 0 = contorno na tampa larga);
Liner = pigmento 20 + orla 200 (fica: a 200 px 100 e 200 dão o mesmo aro ~0,9; a 40 px a 100 tirava o fio escuro do Rebelle, 1,247 → 1,07) + `wetOpenEdge 28` + domo 0,3 + tampas 1 + K-M off. `CURAS_J6` esvaziada (aplicada).

**Lei nova — a borda GRADUADA** (`Knob::EdgeSoftMass`, «Edge darkening fades with the paint», grupo Water, 0..8,
autorizada pelo coordenador): a tampa do Wash ainda tinha um contorno fino irregular que o Rebelle não tem. Causa: a
secagem conta vizinhos com `susp > 10` BINÁRIO ⇒ o aro nasce na isolinha 10 mesmo onde a tinta esbate. Com `k > 0` cada
vizinho conta `min(susp / (10·(1+k)), 1)`; `k = 0` = o caminho de hoje AO BIT e sem custo (a janela escolhe-se pelo tipo
fora do laço, `EdgeGather`/`EdgePlane` monomorfizados). A lei vive em `drying/edge_window.rs` (`EdgeWindowSoft`,
`soft_plane` para a rota Jacobi num `thread_local`). Testes (`ph2d-wet-paint`): `a_borda_graduada_a_zero_e_a_secagem_de_antes_ao_bit`
(hash GS `0x20ba9f7b236930f9` · Jacobi `0xe8e3174d523a6d13`, gravados antes da lei), `a_borda_graduada_esbate_a_rampa_sem_degrau`
(2.ª diferença do `sett` ÷ média: k=2 1,03 → 0,09; vermelho antes da lei), `the_soft_sliding_window_is_the_direct_gather`.
**Linhas de `drying.rs` tocadas** (para o merge com o agente de performance): L148 `count: u32` → `nb: f64`; L173
`e = nb`; L294–309 `drying_pass` escolhe `dry_gs::<EdgeWindowSoft|EdgeWindow>` (o corpo antigo sob `fn dry_gs<W: EdgeGather>`);
L325 `W::seed(.., soft_den(k))`; L337 `win.frac()`; L476–483 k > 0 ⇒ `soft_plane(.. dry_jacobi::<SoftPlane> ..)`;
L485–486 `dry_jacobi::<HardPlane>(.., &[])`; L488–496 cabeçalho `fn dry_jacobi<P: EdgePlane>`; L551 `P::nb(edge, soft, i)`.
Nada movido nem reformatado fora delas. Calibração no Wash (tampa ampliada + perfil): k 0/1 = contorno; k 2 aro 1,38; k 4
tampa limpa, aro 1,41 ⇒ **k 4 + orla 40: aro 1,33 · +4 0,87 · +20 0,79 · +30 0,85**, tampa sem contorno.

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

## Portões e fecho

- 4 portões J4/J6 da família usavam o preset de fábrica como CONTROLO e a fábrica passou a ter as curas: os instrumentos
  prendem agora o estado que medem (`as_tampas_do_filme_largo_esbatem` — o Wash da J6, `foto_a_pincel`;
  `o_filme_do_traco_e_liso` — o filme chato, domo 0; `a_borda_da_mancha_molhada_esbate` — o Liner da J6 nos dois braços:
  pigmento 17, sem domo/tampas, K-M desligado — dá os números da J6, 12,5 · 1,034 · 1,45, controlo 2,8 · 1,162);
  ⚠️ com o Liner de fábrica R1 a mancha molhada mede 1,67–1,73 × a largura (Rebelle 1,42): fila (pigmento 20 × 17).
  `o_liner_tem_o_fio_escuro_da_borda` reprovou a orla 100 (1,066) ⇒ o Liner volta a 200: 1,141 (Rebelle 1,247).
- Verdes por nome: `familias::aguarela` (14), `the_factory_presets_are_counted`, `ph2d-wet-paint` `a_borda_graduada` (3) e `drying`.
- Finais (fábrica regerada): Wash 200 px aro 1,29 · +20 0,77 · +30 0,83; Liner 40 px orla 1,13. Pranchas `r1_Wet_Wash.png`,
  `r1_Wet_Liner.png` (Rebelle · antes · R1), `r1_Mop.png` (Rebelle · M2 · M4), `r1_Smudge_Finger_alcance.png`.
- ⚠️ Vermelhos ANTERIORES ao R1 (verificados na base `6ba5d06493`, não curados aqui): `ph2d-panel-wet-tuning`
  `every_visible_knob_has_a_row` (base 94 × 98; com o knob novo 95 × 99) e `os_escondidos…` (9 knobs `Hidden` fora da
  lista: extGranulation, extRakeLight, extStaining, extWetSoften, glaze, rewetDrag, rewetPaint, solubility, wetCover);
  `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (Blender Dots/Hard/Soft/Textured, Ink Brush… — o ESTADO J11 já
  dizia «miniaturas (24 mudam)»); clippy `collapsible_if` em `assenta/ponto.rs:38` e fmt em `solver/advect_*`,
  `oleo_por_traco_tests.rs` — ficheiros que o R1 não toca.

## Commits

- `a8df93dfed` Wet Wash e Wet Liner por valores (orla, domo, tampas, K-M off), fábrica regerada; Smudge recusado.
- `94df46567d` a lei `EdgeSoftMass` (borda graduada), Wash k 4 + orla 40 + pontas 1, Liner orla 200, portões presos, pranchas.

## Fila (não feito)

1. Mop: o anel escuro + rasto branco no cruzamento molhado (água muita + parede) — motor.
2. Smudge Finger: juntar as duas paredes do fio num só traço (o eixo amostra o papel do início do traço) — outra lei.
3. Liner: a mancha molhada de fábrica 1,67 × a largura (Rebelle 1,42) — pigmento 20 × 17 a decidir pela cega/dono.
4. Item 3 da missão (Water Brush, Fan Brush, Gouache Dry/Wash) — não começado.
