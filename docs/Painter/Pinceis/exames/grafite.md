# EXAME da família `grafite` (J3, catálogo B: B-1 HB · B-2 2H · B-3 2B · B-4 8B · B-5 Mechanical · B-11 Erodible)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/grafite.rs`. Pranchas: `pranchas/grafite/` (cegas e chaves
ao lado; `_antes`, `_krita`, `_mypaint` são da 2.ª ronda). Régua: `exames/grafite/`. SHAs 1.ª–2.ª ronda: `07bf02f7dc` ·
`8f8b1db5ab` · `8d182780bc`. **3.ª ronda**: `31f4ae9b13` · `601d9a6cd1` · `cffaa23d57` · `674f87212c` · `2763b29dd1` · `2a53a8dd09`.

## 3.ª ronda (20–21/10) — dois recursos de motor, os quatro recurados, o exame

**Motor 1 — o TECTO de tamanho (decisão 8).** `BrushSpec::size_max_px: Option<f32>` (o DIÂMETRO do anel; `None` = só o do
slider), no mapa `brush` do preset só quando fixado (`CAMPOS_DO_PINCEL` ⇒ os 4 meios; ausente = sem tecto, nunca o do pincel
anterior). O tool não deixa o raio passar de metade dele (`raio_maximo` em `set_brush_size_px`; o pincel que sai do preset
também, para o Size herdado), salvo na lei CRUA do Wet (a tabela raio do motor → largura: com o tecto ela achatava-se).
Painel: `BrushSettings::size_max_px`, a faixa do volume «Size px» do Wet acaba nele. `a_largura_e_a_prometida` cumpre
`min(pedido, tecto)`. **Medido** (fotos `tamanhos`, recto de 200 com `max_brush_size` 80): o Rebelle pinta 2H 59 · HB 60 ·
2B 59 · 8B 69 px; ÷ o recto de 40 = 1,97–2,00 ⇒ pinta ao 80 dele, mas o Size dele pinta 0,75 do que diz (30 a 40; o 8B
0,87). ⇒ **a régua do Rebelle NÃO mede 80 no nosso anel: mede 60 (lápis) e 69 (8B)**; o «~41 px» da decisão 8 não se
reproduz (a ½ do máximo: 47–48; a ¾: 34). Portão red-first `o_tecto_do_preset_pinta_o_grafite_ao_tamanho_do_rebelle`:
200 px → 196–199 antes, 60 · 61 · 60 · 70 depois (Rebelle 59–69); o painel 60, o Wet Round 200, o Size herdado 60; o
controlo sem o campo 188. Mutações: tirar o tecto do setter (sangra: 195–200, painel 1024) · desligar o recorte do preset
(sangra: o herdado 200). Custo: um `min` por mudança de Size — zero no traço. Os outros presets: byte-idênticos (o campo
não se grava). Como usar: `04_BANCADA.md` §9.

**Motor 2 — a 2.ª ponta do dual À FRENTE da 1.ª, no arco do caminho.** O rápido (25 px/evento) pousava 7–12 % mais que o
lento (3 px) — HB 0,669 → 0,714; sem o dual 0,749 · 0,746; o Rebelle 0,627 · 0,627. Causa: a máscara só chegava a `p` e a
1.ª carimba até `p + raio`; no lento cada dab perdia a frente. Cura: `dual_estende` estende a 2.ª ponta até `p + raio`
pelo ARCO da curvatura medida (em linha recta o miolo da espiral saía branco). Portões red-first
`o_rapido_pousa_o_que_o_lento_pousa` (1,07–1,12 → 0,99–1,03) e `a_espiral_pinta_o_proprio_caminho` (o caminho no 1.º
quarto ÷ o resto 0,69–0,72 → 0,92–0,98; Rebelle 1,00). Mutações: frente 0 (1,11) · curvatura 0 (0,69–0,72). Só os
grafites levam dual.

**As receitas** (a revisão cega via o Rebelle MACIO, manchado, de borda que ondula; o nosso salpicado de píxel): orla do
lápis 0,45 × 3, através 0,3, espalhar 0,25 (2H 0,2), dureza 0, forma Noise 32 a 0,3; perfil Smoother; tufos (Shape Noise
`Random` Size 4) HB 0,4 · 2H 0,05 · 2B 0,1; grão de píxel HB 0,4 · 2H 0,3, o 2B Size 128 a 0,56; cargas HB 100 · 2H 58 ·
2B 120. 8B: orla 0,6 × 2, através 0,4, espalhar 0,4; papel 0,4 (franja macia), grão Size 128 a 0,5, tufos Size 8 a 0,9,
carga 280. ⚠️ A folga do ac a 5 px do portão da família subiu 0,15 → 0,27 (o chão do Wet a 5 px sem orla nem tufos é 0,68;
o 0,34 da 2.ª ronda vinha do defeito do Motor 2; a forma da orla a 0,8 dava 0,33 mas tirava a Textura da mídia do vale).

**Verdes por nome** (cada um depois do último preset): a família (onda 0,069–0,076; cv5 0,082–0,133; tom 2H ÷ HB 0,80),
a mídia (os quatro), o grão do lápis, o lápis deitado (8B ao longo 1,30, Rebelle 1,24), o rápido = o lento, a espiral, o
tecto, a largura prometida, miniatura, byte, `a_lei_da_tinta` (o 8B regravado). Vermelhos de OUTRAS famílias no caminho:
Ink Bullet/Dip Pen/Ink Brush/Marker na `a_lei_da_tinta` e na pressão.

### Os quatro — exame (Rebelle), revisão cega NOVA, cruzamento

| | régua | fora (d) · família | origem | revisor: oráculo · imitação = nosso · 3.º outro |
|---|---|---|---|---|
| HB | 430/454 | Rough Sketch 0,819 · 2H 0,57, 2B 0,41 | 5.º 0,719 | 0/6 (tomou o 2B) · 6/6 · **não** («parente próximo») |
| 2H | 328/454 (o tom, decisão 10) | Rough Sketch 0,825 · HB 0,57 | 3.º 0,739 | 0/6 (o NOSSO por original, o HB por imitação) · — · sim |
| 2B | 400/454 | Oil Pastel 0,798 · HB 0,48, 4B 0,56 | ✓ 1.º 0,657 | **6/6 · 6/6 · sim** |
| 8B | 426/454 | Oil Pastel 0,801 · 4B 0,52 | ✓ 1.º 0,541 | **6/6 · 6/6 · sim** |

- **8B — ⛔ REPROVADO, um defeito de fora da decisão 9.** Revisor: (1) a rampa entra a ⅓ e enche de repente — decisão 9;
  (2) **a borda do Rebelle em lóbulos/contas e o recto de 200 px com bossas de ±10 px; a nossa quase lisa**; (3) o azul
  do Rebelle tapa o vermelho (33/101/174), o nosso deixa um violeta (43/97/164); o de 5 px nosso mais claro. Medido: as
  bossas são o carimbo do Rebelle (a forma oval-IMAGEM girada a cada 0,2 do diâmetro) — com a regra da fábrica (Spacing
  0,1 + Continuous, dono 18/10) a cápsula alisa o contorno; a orla do dual (0,8 × 1, 0,6–0,8 × 2–3, dureza 0,6–0,8) e o
  jitter 0,2–0,4 testados: bossas pequenas, ou o «ao longo» deitado sai do vale (1,55), ou o traço parte-se em blocos.
  **Para a fila**: decidir se os grafites saem da regra do Spacing (carimbo esparso de imagem) — decisão de produto.
- **HB — ⛔ REPROVADO.** O revisor tomou o 2B (o vizinho, de grão) pelo original e o Rebelle por «parente próximo» — o
  traço é já o do Rebelle, mas ele ainda vê: as pontas do Rebelle em tampa REDONDA (as nossas afinam, a rampa forte abre
  em forquilha); o recto de 5 px do Rebelle mais escuro e contínuo (régua: cobertura 0,58 × 0,50, textura 9,6 × 22,8 à
  escala 0,75); o cruzamento sobre o vermelho chapado no Rebelle, manchado no nosso.
- **2H — ⛔ REPROVADO.** Pares iguais pela semente (o nosso 2H e o HB têm o mesmo contorno; o revisor usou-o). O Rebelle:
  manchas de contorno mais nítido e grão mais marcado que o nosso (que lê «esbatido»). ⚠️ Conflito medido: a Textura da
  foto do 2H é 4,98 (a do HB 6,15) e o vale da mídia vai a 6,41 — com o grão 0,4 o 2H dava 7,27. Fica para o coordenador.
- **2B — ⛔ REPROVADO, já identificado (6/6 · 6/6 · o HB «claramente outro»).** O revisor ainda vê: o nosso MAIS ESCURO
  (cobertura da mídia 0,606 contra 0,544 — dentro do vale, mas lê-se; o cruzamento azul-marinho opaco, o do Rebelle
  arroxeado); o grão aos montes onde o Rebelle é um salpico miúdo uniforme; o recto de 5 px mais claro e falhado; uma
  «bolota» no fim forte da rampa. Próxima cura (receita, não motor): a carga 120 → ~105, o grão do 2B mais fino
  (Size 160–192), e a lei do traço fino (abaixo).

**Comum aos três lápis (a fila da próxima ronda):** o traço de 5 px (à escala da bancada, 3,75 px) — o nosso falha pelo
grão de píxel (Textura 22,8 contra 9,6 no HB), o do Rebelle é contínuo: o grão não deve partir o traço fino (03_MOTORES
§7.2, «o grão não escala para baixo»). E as pontas: o Rebelle fecha em tampa redonda; a nossa rampa forte abre em
forquilha (as marcas da orla espalhadas através no fim do traço). Revisões cegas: os nossos dois presets da mesma orla têm
o MESMO contorno (a mesma semente) e os revisores usam-no para os emparelhar — vale semear a orla pelo preset.

## Custo (`--custo`, smoke, 60 Hz; p95 ms 5 · 40 · 200; RSS 200; `load` 1 min) — COM o tecto

| HB 4,16 · 7,77 · 6,82 · 86 MiB · 11–13 | 2H 2,46 · 8,48 · 6,77 · 87 MiB · 14–15 | 2B 1,01 · 3,81 · 4,07 · 86 MiB · 10–12 (antes da cura 3c) | 8B 1,04 · 3,15 · 3,58 · 86 MiB · 8–10 |
|---|---|---|---|

O 2B a 200 px (16,9 ms a load 14–18 na 2.ª ronda) agora 4,07: o tecto pinta-o a 60 px. ⚠️ load 8–15 (outras famílias): o
relógio é indicativo; todos ≤ 8,5 ms, longe do orçamento.

## Rondas 1–2 (resumo; o detalhe no git, `32c84ee113`)

A orla do dual (a borda que ondula), o grão fino fraco dos duros, o dual que se deita com a caneta (motor), o 2H no tom de
fábrica (decisão 10), **B-5 Mechanical 0.5 fundido no Hard Round** (d 0,289; o Pentell do Rebelle é uma ponta dura fina),
**B-11 Erodible NÃO SAI** — motor GRANDE (L17: estado da ponta entre traços + «Sharpen Tip», a forma que perde os cantos,
a dureza por dab; 12 ficheiros em 5 crates).
