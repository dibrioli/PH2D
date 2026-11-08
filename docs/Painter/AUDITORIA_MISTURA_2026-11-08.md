# Auditoria da mistura Wet blend (08/11) — os defeitos #73–#80 (CURADOS no mesmo dia)

> ✅ **CURADOS em 08/11** — `0604fd6f5c` (#73–#78) e `6b0522894b` (#79–#80); commit e teste de cada em [`BUGS_painter.md`](BUGS_painter.md) (§ Bugs #73–#80) e o fecho da onda no [handoff](handoffs/HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_CURAS_DA_MISTURA.md). O texto abaixo é a medição ANTES da cura (mecanismo e linhas de 08/11 de manhã); a seção «Outros abertos» continua viva.

> Pedido do dono (08/11): *«conferir como os novos pincéis com o novo color mixing reagem com paper, grain, color
> shape, color grain, assim como o composite brush e o sistema layer per color. Auditoria pesada.»* Duas lentes
> independentes, só medição (nenhum código mudado). Sondas ignoradas, corridas à mão (perfil smoke):
> [`sonda_auditoria_papel_grao.rs`](../../crates/ph2d-tool-painter/tests/it/sonda_auditoria_papel_grao.rs) (cena
> 600×300 vazio · branco · listras; 13 pincéis × 20 variantes) e
> [`sonda_auditoria_composite_camadas.rs`](../../crates/ph2d-tool-painter/tests/it/sonda_auditoria_composite_camadas.rs).
> Material: [`material_2026-11-08_auditoria_papel_grao/`](handoffs/material_2026-11-08_auditoria_papel_grao/) e
> [`material_2026-11-08_auditoria_composite_camadas/`](handoffs/material_2026-11-08_auditoria_composite_camadas/)
> (`sonda_saida.txt`, `sonda_lote2.txt`). Pincéis: Round Mixing, Gouache, Dry Gouache, Thin Gouache, Gouache Blender,
> Oil Paint, Paint And Apply, Thick Oil Paint, Crayon, Pastel, Chalk, Finger Tip, Pointillism (valores de 08/11:
> [`material_2026-11-08_mistura/`](handoffs/material_2026-11-08_mistura/)). ⚠️ Os números de linha são de 08/11
> (commit b2693186b8); confira-os antes de editar.

## O caminho da mistura (o que a auditoria leu)

Por carimbo (`stamp_route/mistura.rs`, `mistura_opaca.rs`): (1) a pegada (falloff, Shape, Grain/papel-como-grão)
pinta-se a BRANCO numa máscara e só o alfa é usado (`stamp_dab_textured_masked` com `branco`); (2) amostra UMA cor
média da tela de antes do traço + a película «velha» do traço, pesada por pegada × alfa; (3) reservatório por
distância (Color stretch); (4) tinta = `lerp(reservatório, pincel, a_eff)`, `a_eff = 1 − (1 − amount)·res.carga`;
(5) taxa = pegada sem grão × Density × Flow × `corpo`, tecto = cobertura × grão; (6) a película compõe-se sobre a tela
de antes. A mistura é decidida em `stamp_route.rs:520` e SAI CEDO: as rotas depois dela nunca correm com ela ligada.
Amostra: a camada ATIVA como estava no início do traço (no Composite, a cópia do undo); nunca o composto visível.

## Defeitos (por gravidade) — todos de MOTOR

| # | Defeito | Medição | Mecanismo | Cura (decisão do dono quando a há) |
|---|---|---|---|---|
| **#73** | Composite: a camada **Erase PINTA** com Wet blend | pilha [Erase, Brush] no bloco: Off α 0; Wet `[95,120,233,255]` | a porta `stamp_route.rs:520` só olha `!self.stroke_erases()` (o interruptor da ferramenta); a camada Erase troca `brush.blend` para EraseAlpha (`composite_acumulado.rs` ~302, `composite_pilha::aplica_borracha_do_traco`) e o núcleo da mistura não lê o blend | a porta recusa também `brush.blend == EraseAlpha` (ou o tipo de acumulação da camada) |
| **#74** | Composite: camadas Brush partilham UM reservatório/película → **o byte depende da cadência dos eventos** (fere «o mesmo traço dá o mesmo byte») e a camada de cima apanha a tinta da de baixo | 2 Brush azuis: Wet Δ médio 3,5 máx 138 vs 1 camada (Off Δ 0,34); 60 × 120 eventos: Δ 0,43–0,72 máx 20–34 nos 4 pincéis (sem Composite ou sem mistura: mesmo byte); camada de cima amarela sai `[163,169,183]` (Off: amarelo puro) | um só estado por traço (`mistura.rs:150-160`, `:345-349`); o reservatório salta entre camadas por lote; a película é lida do plano da 1.ª camada (`mistura_opaca.rs:220-262`) e escrita por todas | **dono: cada parte mistura SÓ a tela de ANTES do traço** → um estado de mistura por posição da pilha, trocado em `acumula_camada` como `composite_mask`/`rng_camada`. Prova: 60 × 120 eventos = mesmo byte |
| **#75** | Composite (receita do dono Blur 1 / Smear 0,25 / Brush 0,10): **Strength da camada Brush não age** com Wet blend | Strength 0,10 × 1,0: Wet Δ 0,000; Off Δ 8,4 máx 229; Wet pinta `[100,125,234]`, o alvo (Off) é `[229,233,251]`. Pilha de 2 Brush a 0,1: a Strength age (Δ 7) | o núcleo (`mistura_opaca.rs:336-470`) não lê `spec.strength`: taxa = densidade × Flow × corpo | multiplicar a taxa pela Strength; antes, confirmar porquê a pilha de 2 Brush a respeita |
| **#76** | Esbatedores (Amount 0: Finger Tip, Gouache Blender) **pintam a cor do pincel** no rasto que sai das cores para o vazio | Finger Tip no vazio `(75,185,115)` α 1,0; Gouache Blender `(76,185,116)` α 0,93; Round Mixing a Amount 0 `(72,184,113)` α 0,8; a cor apanhada no branco era `(232,212,210)`. No vazio puro α 0 (certo) | `a_eff = 1 − (1 − amount)·res.carga`: no vazio a carga decai, `a_eff → 1`, a tinta vira o pincel e `corpo = carga` ainda pousa | **dono APROVOU:** pesar pela quantidade, `P = (res·c·(1−a) + pincel·a) / (c·(1−a) + a)` (a=0 → reservatório puro; c=0 → pincel; c=1 → igual a hoje). Muda a cor a meio da carga em todos os pincéis com mistura → alarmes + relatório ao smoke |
| **#77** | **Controlos mortos** com Wet blend: Grain Colors, Shape Color, Per-Layer Color (e, pela leitura, a ponta de pixel) | Grain Colors = só grão (mesmo hash) nos 13; Shape Color = base; Per-Layer Color Δ 0 máx 0 (Round Mixing, Oil Paint, Finger Tip). Controlo: Off, as rampas pintam (2 250/2 250 texels) | `stamp_route.rs:520-525` sai antes do encaminhamento das rampas (`:592-626`, `color_ramp_owner`) e da ponta de pixel (528) / Per-Layer (~535); a rota só lê o alfa da máscara. Painel: nenhuma regra em `inercia.rs`, nenhum aviso | a cor por píxel da rampa/camada entra como a cor do pincel: `tinta(x) = lerp(res.cor, rampa(x), a_eff)` (pintar a máscara com a rampa em vez de `branco` e ler o RGB). No mínimo, um `Dependente` com dica |
| **#78** | Pincéis que espalham ficam **cheios** com a mistura (Pointillism, Pastel, Dry Gouache) | textura do alfa no vazio, sem grão: Pointillism 0,067 (Off 0,196), Dry Gouache 0,098 (0,128); cobertura Pointillism 0,71 (Off 0,38–0,46), Pastel 0,36 (0,16) | `spec/queries.rs:395` corta o espaçamento a 3 % (`PASSO_DA_MISTURA`): o `k_passo` normaliza a opacidade mas não o nº de pontos espalhados; o tecto que só sobe (`px[3] = s_livre.min(tecto.max(s0))`) acumula as cerdas que rodam | amostrar a 3 % sem CARIMBAR a 3 % nos pincéis que espalham; ver junto com o Fixed dab size (65eccb563f) |
| **#79** | A mistura **só vê a própria camada** | camada nova vazia sobre cores: Gouache Blender e Finger Tip não fazem nada (bloco `[200,0,0]` intacto); Round Mixing Δ 1,0 | a amostra é a camada ativa (`stamp_route/mistura.rs:145-175`) | **dono SIM:** opção da mistura «Sample: This layer · All visible layers» (inglês, no cartão do Wet blend); «All visible layers» de fábrica no Gouache Blender e no Finger Tip |
| **#80** | Com «Same as Paper» ligado e Paper None, a **Depth do Grain** não esmaece (não tem o que ler) | foto `same_as_paper_nao_muda_o_paper.png`; `GraoDoPapelSemPapel` não lista `PAINTER_BRUSH_GRAIN_DEPTH` | pôr o id na regra fez o censo da Aquarela reprovar (lá a Depth também nada faz com o Same as Paper da granulação) | uma regra própria para a Depth (Aquarela incluída), não a mesma lista |

## Conferido e bom (não refazer)

O grão NÃO vira tinta arrastada (a amostra é uma cor por carimbo) e sobrevive no arrasto (textura do alfa 0,105 Wet ×
0,107 Off); Paper como grão: Wet ≈ Off (0,041); os esbatedores não pintam o papel vazio; mesmo traço = mesmo byte
(13 repetidos) e, sem Composite, 60 × 120 eventos = mesmo byte; 1.º e N-ésimo carimbo no vazio = a cor do pincel, sem
NaN nem saturação; undo/redo voltam ao byte (tela direta, Composite [Smear, Brush], camada sobre camada); mistura Off
ignora Amount/Density/Stretch; Composite de uma camada = sem Composite; Amount contínuo no limiar 0,25
(`AMOUNT_QUE_COBRE`: R 146/143/140 em 0,24/0,25/0,26); a cor apanhada no branco escurece com o raio (coerente com a
memória em diâmetros); o Multiply do Wet Wash Mix continua a agir com a mistura. **Baixo, não é defeito:** a ordem das
camadas do Composite muda o resultado (Δ 1–3, máx 197) — é a lei do Composite.

## Outros abertos da linha (08/11)

- `o_pincel_guardado_pinta_o_mesmo_traco` (lib `ph2d-tool-painter`, `wetpaint_preset/tests.rs`) reprova — já
  reprovava em de47355fc3, antes do Fixed dab size (medido num worktree à parte). Causa por achar.
- Wet Wash Mix: o grão em imagem (CC0) não aparece no traço e deixa-o ~4× mais lento — motor (`dab::grain_at`, o grão
  por carimbo some na soma de dezenas de carimbos macios; hipótese não medida).
- Pastel fraco: tecto de cobertura do spray com grão, não a mistura; e o Pastel tem ponto fixo sem referência (não
  cresce com o Size) — mudar de fábrica é decisão do dono.
- Watercolor Splash: no tamanho de fábrica as manchas espalham-se pela cena toda (o dono olha).
- O filme do Óleo Real (`grao_no_filme.rs`) ainda usa o papel de reserva com Paper None (a regra do Same as Paper não
  foi estendida para lá sem ordem).
- Fixed dab size: tecto `SPRAY_FIXO_MAX = 4096` medido sem o Wet blend armado.
