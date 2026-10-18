# 49 — PROMPT: a W3 — o RELEVO de cada fio, a oleosidade e a luz (18/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo; `bash scripts/btrfs-health.sh`. Leia SÓ: este prompt ·
o [handoff P48](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P48_W2_FIOS.md) §2–§4 e §6 · o
[48_medicao](48_medicao_os_fios.md) §2, §6 e as Recusas · o [05](05_plano_da_linha_e_o_antes.md) §2 (a luz do relevo:
`impasto_light.rs` é método do estado do Impasto ⇒ porta livre append-only num ficheiro irmão) e §3 (a linha da **W3** e a
barra dela) · o [02](02_presets_para_wet_painter.md) §2.4 (Add/Remove impasto, `paint_struct`, `remove_impasto_strength`
por população). Regras de 13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no fecho, edição provada), R8
(sondas em `--profile smoke`), §0.9 (o Rebelle corre-se, nunca se lê — o teste dele acaba ~06/11; a bancada:
`ferramentas/gotas_inicia.sh` + `tilt_lapis_rebelle.sh` como molde).

## De onde vimos

W2 fechada e **smoke do dono OK (18/10)**: os fios com física própria (`ph2d-wet-paint/src/bristles_fios.rs`, 12 knobs
`Hidden` por nome) no Oil Bristle (cor e pressão por fio: carris 0,00 → 3,03, Rebelle 3,32) e no Express Oil (pressão
quase constante por fio: textura 3,2 → 7,8, Rebelle 9,0). **O que ficou para esta onda (lista 26 C34):** a textura do
Oil Bristle é 3,85 contra 5,86 — os fios ESCUROS do Rebelle são a SOMBRA do relevo de cada fio (o Bristle Flat dele tem
`particle_thin_remove_impasto_strength` 0,65: metade dos fios escava, metade constrói). A régua mostrou-o: a textura do
óleo é quase toda ATRAVÉS do traço (resíduo 5×5 através 5,48 %, ao longo 0,98 %), e a pressão por fio não a pode dar
porque o óleo satura (pigmento 2 000: só abriria falhas brancas — recusado, medido).

## Pedido — o produto final, não um MVP

1. **Medir primeiro, um lote só (§0.10).** A barra da W3 (05 §3) é de 08/10: «óleo borda/miolo 0,8–0,9 (hoje 0,52)» — o
   portão das mídias já mede o Oil Bristle com Borda 0,872: confira o «hoje» antes de acreditar nele. Re-meça nos três
   óleos (Oil Bristle, Acrylic Round, Express Oil) à largura da foto e a 40 px: textura, a textura ATRAVÉS × AO LONGO,
   borda/miolo, carris, e a ALTURA do relevo por fio (o campo do motor, não o píxel). Pergunte ao código onde o relevo nasce
   hoje (o `emboss`, o `height` do Wet Paint, o tapete) e se ele já segue os fios da W2 — confira antes de construir.
2. **O relevo de cada fio** (05 §3, a linha da W3): fios que CONSTROEM e fios que ESCAVAM (Add/Remove), a altura pela
   pressão do fio da W2, a oleosidade como viscosidade, e a luz do Impasto por uma porta livre (o corpo dela não muda).
   Em ficheiros irmãos (`bristles_*.rs`; `trail.rs`/`solver.rs` no tecto). Cada knob POR NOME no preset, omissão = a lei de
   hoje ao byte. A barra é a foto do Rebelle de cada óleo (o vale, com o controlo = o preset sem o relevo novo).
3. **Gates**: as barras da W3 vermelhas antes, ao lado de `tests/it/os_fios.rs` (a sonda `sonda_a_barra_da_w2` e
   `cena_seca_com_rampa` servem); prova de mutação nas leis que importam; a identidade das 22 cenas de `a_lei_da_tinta`, as
   miniaturas e os outros meios ao byte onde a lei não muda (onde muda, regere a fábrica no mesmo commit e diga-o).
   ⚠️ Testes que usam um pincel de fábrica como VEÍCULO de outra lei tiram o que não é deles
   (`verde_no_soltar_tests::sem_os_fios`, o Emboss) — a bateria da W2 achou dois.
4. **Fecho**: suíte afetada + clippy (`PH2D_SUITE=fecho`); handoff ≤ 8 KB; o 05 §3 com a W3 marcada; smoke ao dono em
   passos numerados (o Oil Bristle ao lado do Rebelle; leve × forte; a luz do relevo).

## Por decidir do dono (pergunte no fim, não bloqueie a onda)

- **B15** — o Acrylic Round liso como o do Rebelle? (os carris dele são o tapete esparso afinado pelo dono em 11/10;
  nenhuma lei dos fios os cura). Se a resposta chegar SIM antes do fecho e for o mesmo bloco (a acrílica é da W3: «largura
  A = B»), junte-a.

## Na fila (do dono — NÃO junte a esta onda)

Lista [26](26_ABERTOS.md): **A13** apagar um pincel do utilizador · **A14** repor pincéis · **A15** a prévia do pincel em
TODOS os modos. Da caneta: C29 (pad), C30 (dedo), C32 (X11/Windows). C33 (o Dry Brush sem régua). Depois da W3: W4
(papel), W5 (mistura), W7 (sem água), W8 (o painel — inclui os 12 knobs dos fios).
