# W2 E4 — as 10 facas do Wet Paint contra o Rebelle (Óleo Real W2, 27/10)

Oráculo: `fixtures/rebelle_bateria/OIL_AND_ACRYLIC/knife_knife_*` (6 fotos por faca). Bancada `04_BANCADA.md`, revisão
cega `05_REVISAO_CEGA.md`, custo `REGRAS_DE_CUSTO.md` (2 rondas, 1 revisor `sonnet` por ronda cobrindo as 10).

## Como se refaz

`python3 -I docs/Painter/49_realshader/ferramentas/E4_Knife_facas.py <raiz> docs/Painter/49_realshader/ferramentas/E4_Knife_ronda.json`
→ `cand/<Nome>.ph2d-brush.png` (base = o Dabber de fábrica + os valores abaixo; o escritor de PNG é
`E4_Knife_mk_preset.py`). Exame: `bash ferramentas/exame.sh <cand> fixtures/…/knife_knife_<slug>` com
`PH2D_EXAME_DIR` próprio. Pranchas `pranchas/W2_Knife_<slug>.png` (cegas e chaves idem), réguas `exames/W2_Knife_<slug>_regua.txt`.

## Os valores (ronda 2)

Comum: `falloff` Constant, `hardness` 1, `dab_quina` 1, `jitter` 0, `pressureOpacityCurve` 1, `pressureSizeCurve` 1,
`levelingOwnStroke` 1. Geometria = a lei L3 (`06_LEIS_DAS_LACUNAS` §1, `leis_lacunas.py lamina` em cada `gestos_A`):
fixa com o eixo longo do Rebelle, `dab_flatten ≈ 1 − fino/longo`; seguidora `dabRotation` 1 a 90°.

| faca | lâmina | ponta | `knifeRidge` | carga · papel | textura · forma | emboss · pressão→tamanho |
|---|---|---|---|---|---|---|
| Knife Smooth | fixa 0° · flatten 0 · quina 1 | hard 0,5 Smooth | 3 | ppd 600 · gate 0,8 | Noise 6 · — | 0,1 · 0,4 |
| Knife Smooth 2 | segue · 0,8 · quina 0,5 | 1 Constant | 8 | ppd 2000 · gate 0 | — · — | 0 · 0,4 |
| Knife Smooth 3 | segue · 0,8 · quina 1 | 1 Constant | 8 | ppd 600 · gate 1,2 · contraste 2,2 | Noise 6 · — | 0 · 0,4 |
| Knife Textured | fixa 86° · 0,87 · quina 0,5 | 1 Constant | 4 | ppd 400 · gate 1,0 | Noise 60×2 · — | 0,1 · 0,6 |
| Knife Oily | fixa 172° · 0,69 | hard 0,5 Smooth | 3 | ppd 150 · gate 0 | — · Gradient (carga da lâmina) | 0,1 · 0,6 |
| Knife Stain | segue · 0,8 | 1 Constant | 0 | ppd 600 · gate 1,5 · contraste 2,5 | Noise 5 · — | 0,1 · 0,3 |
| Knife Soft | fixa 178° · 0,84 | hard 0,3 Smooth | 2 | ppd 500 · gate 1,0 | Noise 6 · — | 0,1 · 0,6 |
| Knife Grainy | segue · 0,8 | 1 Constant | 2 | ppd 600 · gate 1,2 · contraste 2,0 | Clouds 20 · — | 0,1 · 0,3 |
| Knife Dry | fixa 170° · 0,74 | hard 0,4 Smooth | 0 | ppd 400 · gate 1,0 | Noise 6 · — | 0,1 · 0,6 |
| Knife Dry 2 | fixa 10° · 0,79 | hard 0,4 Smooth | 0 | ppd 300 · gate 1,2 | Noise 6 · — | 0,1 · 0,6 |

Medido no caminho: as riscas internas da 1.ª ronda eram o `emboss` 0,5 do Dabber (a 0: chapado); a textura do pincel
só aparece com `paperGate` > 0 e a carga abaixo da saturação do portão (`pigmentPerDab` 2000 = cheio); `extDryBrush`
não mexe na faca; `pressureSizeCurve` 0,25 do Dabber deixava a rampa de pressão com largura constante.

## Régua (dentro de 3 ruídos, /454) e revisão cega

| faca | r1 | r2 | cega r1 (achou o oráculo / a imitação era o nosso) | cega r2 | veredicto |
|---|---|---|---|---|---|
| Smooth | 362 | 344 | 6/6 · 6/6 | 2/6 · 6/6 (nosso tomado por original em 4) | **curado-não-aprovado** |
| Smooth 2 | 366 | 356 | 4/6 · 4/6 | 6/6 · 6/6 | **curado-não-aprovado** |
| Smooth 3 | 252 | 332 | 6/6 · 6/6 | 0/6 (nosso = original nas 6) | **aprovada** (fábrica) |
| Textured | 368 | 348 | 0/6 (nosso = original) | 6/6 · 6/6 | **curado-não-aprovado** |
| Oily | 290 | 318 | 6/6 · 6/6 | 1/6 (nosso = original em 5) | **aprovada** (fábrica) |
| Stain | 262 | 250 | 0/6 (nosso = original) | 0/6 — a «imitação» era o VIZINHO (Wax Scrape) | **curado-não-aprovado** |
| Soft | 382 | 358 | 6/6 · 6/6 | 0/6 (nosso = original) | **curado-não-aprovado** (ver Dry 2) |
| Grainy | 316 | 270 | 6/6 · 6/6 | 0/6 (nosso = original) | **aprovada** (fábrica) |
| Dry | 342 | 360 | 6/6 · 6/6 | 0/6 (nosso = original) | **aprovada** (fábrica) |
| Dry 2 | 262 | 314 | 6/6 · 5/6 | 0/6 (nosso = original) | **curado-não-aprovado** |

Todas as escolhas da r2 com confiança BAIXA; o vizinho foi «claramente outro» em todas as linhas (as 10 ficam longe
da biblioteca: d 0,78–1,73 × limiar 0,59). Dry 2 e Soft: o revisor viu nas três pranchas Dry · Dry 2 · Soft «o mesmo
trio» — a régua não as separa do Dry, e o dono já recusou facas iguais entre si (P53) ⇒ fora da fábrica até se
distinguirem (no Rebelle as três também são quase a mesma lâmina: 170°, 10°, 178°).

### As diferenças que ficam (r2, ditas pelo revisor; ORIGINAL = Rebelle)

- **Smooth**: o Rebelle quebra em tracejado a 5 px e afina a rampa quase a um fio; o nosso é rugoso, contínuo, com
  mancha no começo. E pinta ~3× a largura do Rebelle no mesmo Size (escala 0,33): Rebelle 9/12 px a 40.
- **Smooth 2**: o nosso tem aro escuro, anéis na espiral, estrias finas na rampa, cor mais saturada; o Rebelle é
  macio e manchado. **No Óleo Real** (folha abaixo) o nosso sai em tubo com sombra funda (massa do Dabber + crista 8);
  o Rebelle é chapado — a faca alisa.
- **Textured**: o Rebelle é translúcido e manchado, afina em ponta triangular na rampa; o nosso é chapado, com lascas
  brancas verticais, rampa rectangular (a pressão quase não estreita a lâmina fixa).
- **Stain**: o nosso é uma camada rendada com riscos brancos (lê-se como o Wax Scrape); o Rebelle é pontilhado ralo,
  de borda dura, com mais papel à vista.

## Óleo Real ligado (`PH2D_EXAME_OLEO=1`, gestos A, escala 1)

[`49_realshader/out/E4_Knife_oleo.jpg`](../49_realshader/out/E4_Knife_oleo.jpg). A revisão cega correu no render
clássico: com o Óleo Real a bancada custa ~130–260 s por foto na CPU (a biblioteca inteira com ele não acabava em
30 min). Visto: sombra no papel à volta de todas as facas (o Rebelle não tem); Smooth/Smooth 2 em tubo; Soft, Dry e
Dry 2 a escala 1 mais finas que o Rebelle (escala da régua 1,5–1,75).

## Fica (fila, sem começar)

- **Largura ao Size**: a tabela de largura vem do Dabber; Smooth pinta 3× o Rebelle, Soft/Dry/Dry 2 ~0,6×. A
  receita no gerador (`familias/oleo_acrilica.rs`) regera a tabela — o integrador/coordenador.
- **Motor**: a faca que ALISA (a massa empilhada sob a lâmina vai a um planalto, sem o tubo no Óleo Real); a lâmina
  fixa que estreita com a pressão (rampa triangular do Textured/Dry); o friso branco vertical a meio do recto
  (Stain, Smooth 3, Textured: o mesmo x em todas — textura do papel?); a crista na sombra (`E4_lamina.md` §Rebordo).
- **Contagens partilhadas**: 4 ficheiros novos em `assets/brushes/WetPainter/Oil/Knife/` sem receita ⇒
  `the_factory_presets_are_counted`, `cada_preset_de_fabrica_e_a_sua_receita`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`,
  `a_contagem_por_meio`, `a_largura_e_a_prometida` e o número da fábrica (45 → 49), lista e tutorial — NÃO editados aqui.
