# 48 — PROMPT: de volta ao plano — o resto da W2, os FIOS e a textura do traço (18/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo; `bash scripts/btrfs-health.sh`. Leia SÓ: este prompt ·
o [05](05_plano_da_linha_e_o_antes.md) §2 (onde o código entra) e §3 (a linha da **W2** e a barra dela) · o
[handoff P47](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P47_W6.md) §2 e §6 · o
[05d](05d_estado_mistura_II_e_presets.md) «Presets de todas as mídias» (a Fase B: a largura da W2 e o que ficou por fazer) ·
o [27](27_plano_shape_casa_dos_pinceis.md) (as cerdas moram no SHAPE desde 13/10; ⛔ a decisão W2.4 «o Grain substitui as
cerdas» não se reabre sem a medição D7) · o [03](03_tracos_medidos_rebelle_x_ph2d.md) §2–§3 (a régua e os números do Rebelle).
Regras de 13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no fecho, edição provada), R8 (sondas em
`--profile smoke`), §0.9 (o Rebelle corre-se, nunca se lê — o teste dele acaba ~06/11; a bancada:
`ferramentas/gotas_inicia.sh` + `tilt_lapis_rebelle.sh` como molde).

## De onde vimos

W6 fechada e **smoke do dono OK (18/10)**: a ponta deitada do lápis (medida no Rebelle a 0–63°), a curva do tamanho, o
piso/tecto, a suavização, a corda e a rotação do carimbo; a C31 medida (o KDE não aplica curva). ⛔ **O dono REVOGOU o piso
de pressão que dava as pontas do Rebelle** («apertar muito e pouco variam pouco a espessura»): a mão dele vai de 0,24 a
0,86 — qualquer lei que mexa na resposta à pressão mede-se nessa faixa
(`docs/Painter/ferramentas/caneta/medicoes_2026-10-17/pressao_por_toque.txt`), não só na rampa sintética.

## Pedido — o produto final, não um MVP

1. **Medir primeiro, um lote só (§0.10).** A barra da W2 (05 §3) foi escrita a 08/10; desde então mudaram as cerdas (no
   Shape), a largura (a W2 de 11/10), a família C, o pó, as formas. Re-meça HOJE, em todos os pincéis de fábrica com
   cerdas ou grão, a 40 px e à largura da foto do Rebelle de cada um: **textura 5–10 %** (Rebelle óleo 5,9, express 9) ·
   **CV ≤ 0,02** (dele 0,011) · **período < 15 % da potência** (dele 9 %) — a régua de `mede_traco.py`/`regua_midia`
   (o período e o CV já estão na régua Rust? confira antes de portar). Com a tabela na mão, diga o que já passa e o que
   falta; o resto da W2 é o que falta.
2. **Os fios com física própria** (05 §3, a linha da W2): pressão Perlin ao longo do fio, largura ondulante, oscilação,
   Thin/Thick/Edges, Random Span, ruído HSLuv — onde os números do passo 1 os pedirem, em ficheiros irmãos (`bristles_*.rs`;
   `trail.rs`/`solver.rs` estão no tecto). Os «carris» (o tile 128² arrastado pela lane) são o defeito-alvo: a cura é a
   franja determinística dos fios, não outro tile. Cada knob POR NOME no preset, omissão = a lei de hoje ao byte.
3. **Gates**: as barras da W2 como portões (vermelhos antes), no portão das mídias (`midias_de_fabrica`, o vale do
   Rebelle) ou ao lado; prova de mutação nas leis que importam; a identidade das 22 cenas de `a_lei_da_tinta`, as
   miniaturas e os outros meios ao byte onde a lei não muda (onde muda, regere a fábrica no mesmo commit e diga-o).
4. **Fecho**: suíte afetada + clippy (`PH2D_SUITE=fecho`); handoff ≤ 8 KB; o 05 §3 com a W2 marcada; smoke ao dono em
   passos numerados (o óleo e o express lado a lado com o Rebelle; leve × forte).

## Na fila (do dono — NÃO junte a esta onda)

Lista [26](26_ABERTOS.md): **A13** apagar um pincel do utilizador · **A14** repor pincéis (um, ou todos) · **A15** a prévia
do pincel em TODOS os modos (é também por onde os knobs da caneta chegam aos outros 4 meios). Da caneta: C29 (pad), C30
(dedo), C32 (X11/Windows). Depois da W2: W3 (altura/oleosidade), W4 (papel), W5 (mistura), W7 (sem água), W8 (o painel).
