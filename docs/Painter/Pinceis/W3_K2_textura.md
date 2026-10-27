# W3 K2 — a textura que acompanha o tamanho do pincel (27/10)

Agente K2 da MISSÃO W3 do Óleo Real. Família «textura fixa em px de tela» (CLAUDE.md §0.10), três casos da W2:
Dabber 2 (buracos), Dabber 3 (lascas), Metallic Splats (pó + `spray_count > 1` que não pousava).

## O mecanismo

Dois knobs do Wet (`Tuning › Stroke`), o par do `embossSizeExp/Ref`:

- `textureSizeExp` (γ, 0–1,5, **0 = a de sempre, ao bit**) e `textureSizeRef` (px, 41).
- O factor é `(D ÷ ref)^γ`, com `D` = o diâmetro do PINCEL (`2 · clamped_radius`, o do traço). ⛔ Não o do dab: o
  grão `Tiled` está preso à tela, e um factor que segue a pressão re-faseia o padrão inteiro a cada oscilação.
- Lido pelo hospedeiro (`wetpaint/lote.rs::escala_da_textura`) e aplicado a:
  - o grão de mapeamento **Tiled** (`escala_o_grao`: o `Size` multiplica a coordenada, divide-se por ele) — View/Random
    já é relativo ao dab, o Flow ao traço, o Stencil ao gizmo;
  - o **pó solto** (`po.rs`: `po_px × factor`).
- Campo ausente no preset = o default = neutro. Nenhum preset de fábrica o usa (censo: 62 ficheiros).

## O `spray_count > 1` que não pousava — a causa

Medido na bancada (Splats, pigmento 2000, tinta integrada na faixa de 200 px): `spray_count` 10 com `particle` 1 →
6 181; com `particle` 0,1 → **54** (o pincel sem spray: 4 719). `paperGate` 0 → 8 502: o portão do papel comia a marca.
Sondado por dab (debug temporário em `dab_route`, retirado): `sil = 1,000` no centro e o **`texv` do tapete de cerdas
0,02–0,05** — a marca pequena era um recorte do tapete das cerdas (≈ 94 % feltro perto de zero com `bristleStrength`
0), e `stamp − (1 − dente) · portão` ficava ≤ 0. ⛔ A 1.ª hipótese (o peso por célula de uma marca sem cápsula, como o
`po`) foi MEDIDA e recusada: com peso 28× a tinta ficou 54 (as marcas têm cápsula; o depósito era zero antes do peso).

Cura (`dab_route.rs`): a marca MENOR que o pincel (`particula_no(spray_count, raio) < 1`) é um respingo **liso** — não
lê o tapete de cerdas. Depois: **54 → 9 059**; as outras sete variantes da sonda ficaram iguais à tinta de antes.
A marca de 0,5 px (o traço de 5 px) continua a não pousar: é o portão do papel sobre um disco de meia célula.

## Régua: tamanho do grão ÷ diâmetro, nós × Rebelle

`manchas.py` (scratchpad do agente): diâmetro equivalente médio PONDERADO PELA ÁREA dos buracos (papel cercado de tinta)
ou, no Splats, das manchas de tinta, ÷ a largura medida D, nas faixas 40 e 200 da cena `tamanhos` (foto A, sem óleo,
escalas da W2: Dabber 2 0,561 · Dabber 3 0,729 · Splats 0,65). Escala livre = razão 200÷40 de 1; preso à tela ≈ 0,2.

| caso | ÷D a 40 | ÷D a 200 | 200÷40 |
|---|---|---|---|
| **Dabber 3** Rebelle | 0,068 | 0,089 | **1,30** |
| antes (Stucci 45, γ 0) | 0,202 | 0,102 | 0,51 |
| depois (Stucci 160, γ 1,2) | 0,083 | 0,099 | **1,20** |
| **Dabber 2** Rebelle | 0,078 | 0,043 | **0,55** |
| antes (sem grão) | 0,067 | 0,023 | 0,34 |
| depois (Stucci 120, γ 0,7) | 0,080 | 0,046 | **0,58** |
| **Splats** Rebelle (tom médio 0,15 · 0,25) | 0,091 | 0,032 | **0,34** |
| antes (`po_px` 1,5, γ 0; tom 0,21) | 0,105 | 0,025 | 0,24 |
| depois (γ 0,3, `po_opacidade` 0,3; tom 0,16) | 0,084 | 0,028 | **0,33** |

Olhado (crop 200 px, Rebelle | antes | depois): o Dabber 2 passa a abrir buracos a 200 px que crescem com o pincel; o
Dabber 3 tem o grão fino a 40 e o grosso a 200; o Splats tem os pontos maiores a 200.

## O que fica para a etapa P (valores, não motor)

- Dabber 2: a FRACÇÃO de buraco está ~6× alta (0,164 × 0,026 a 200) — `paperGate`/`grain_depth`/o Size do Stucci.
- Dabber 3: as lascas do Rebelle são FLOCOS angulosos e alongados; o Stucci é esponja — a forma é do tipo de textura
  (fila de motor «carimbo de lascas»), a escala já segue.
- Splats: o Rebelle é bimodal (pontos grandes e escuros + pó fino) e escurece a 200 (0,25); o nosso é unimodal e de tom
  plano. O `po_px`/`po_opacidade`/`po` e um 2.º grão por spray (agora possível: a marca pousa) são valores da P.
- Valores dos candidatos acima: provados por variantes da bancada (`PH2D_EXAME_VARIANTES`); os ficheiros dos
  candidatos NÃO foram regravados.

## Gates

- Neutro: `presets_de_fabrica::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (a fábrica inteira ao byte) verde.
- `tests/it/textura_do_tamanho.rs`: `o_grao_tiled_cresce_com_o_pincel` (γ 0 escrito = sem a chave ao byte; 96÷24 do
  tamanho do grão 1,85× o de γ 0), `o_po_cresce_com_o_pincel`, `a_marca_pequena_do_spray_pousa`.
- Tuning: `every_visible_knob_has_a_row` 85 → 87 (Stroke 7 → 9).
- Mutações, todas VERMELHAS sobre o verde (ficheiro restaurado e conferido por sha256): γ 0 não neutro (factor 1,25)
  → a miniatura da fábrica · toda marca lisa (`< 1` → `<= 1`) → a miniatura da fábrica · a marca com cerdas →
  `a_marca_pequena_do_spray_pousa` · o grão não escala → `o_grao_tiled_cresce_com_o_pincel` · o pó não escala →
  `o_po_cresce_com_o_pincel`.

## ⛔ Recusas MEDIDAS

| tentativa | medido | porquê não |
|---|---|---|
| factor pelo raio do DAB | — (por desenho) | o grão Tiled re-faseia a cada oscilação da pressão (a lição do Flow, `texture.rs`) |
| γ = 1 com o Stucci 45 (Dabber 3) | 0,205 · 0,240 ÷D | os buracos a 200 px ficam 2,7× os do Rebelle: o Size de base tem de subir com γ |
| peso de grão (`1 ÷ FRACAO_DO_FINO`) na marca do spray | 54 → 54 | a marca morria no portão antes do peso |
| γ = 0,6 no pó do Splats | 200÷40 = 0,54 (Rebelle 0,34) | o Rebelle cresce os pontos ~D^0,3 (γ 0,3 dá 0,34) |
