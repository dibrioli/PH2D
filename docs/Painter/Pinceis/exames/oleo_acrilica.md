# Exame da família `oleo_acrilica` (J4, catálogo G — óleo e acrílica)

Receitas e portões: `tests/it/familias/oleo_acrilica.rs`. Pranchas `pranchas/oleo_acrilica/` (cegas e chaves em `cegas/`,
`chaves/`). Oráculo Rebelle `fixtures/rebelle_bateria/OIL_AND_ACRYLIC` e `EXPRESS_OIL`; tamanho = `volume_params.size` do
preset do Rebelle. Régua `exame_regua.py` (✓/454 com as fotos A e B). Custo com load 16–25: o relógio NÃO vale.

## 1.ª ronda (resumo; o detalhe está no git, `10e78d325a`)
Filbert **aprovado** (refeito: a pegada redonda que segue o traço, fios em dedos ρ 1,6, pigmento 250; 376/454; quatro
revisores tomaram o nosso pelo original). Oil Bristle, Express Oil e Dabber **curados, não aprovados** (Oil Bristle: «pente
regular demais», 3 revisores; Express Oil: translúcido sobre o vermelho e pontas desfiadas; Dabber: a 0,55 do 8B do próprio
Rebelle — decisão para a J5). Flat Bristle, Palette Knife, Rake, Thick Impasto **reprovados** (em estudo). Acrylic Round
fundida no Hard Round. Dry Brush e Fan sem candidato. Portões `cada_oleo_tem_a_sua_identidade`, `os_reprovados_do_oleo_…`.

## 2.ª ronda (J4, 21/10)

**Motor do fio (Wet Paint, aditivo, desligado por omissão; `a_lei_da_tinta` pinta os outros 19 ao bit)** — `140169b17f`:
- **(a) O VEIO** (`Knob::BristleStreak`/`…Light`/`…Length`/`…Cut`, `bristles_fios::Fios::veio`): a troços raros, com
  princípio e fim, um fio pesa (luminosidade HSLuv −) ou falta (+); o resto do fio fica na sua cor. Medido no recto de
  200 px (luminância alisada 7 px ao longo, desvio à mediana da linha > 25): o Rebelle `Bristle Flat` 0,0154 escuro em
  corridas de 84 px, 0,0074 claro; o nosso antes 0,0000. Portões red-first: `o_veio_e_raro_…` (wet-paint) e
  `os_fios_do_oil_bristle_aparecem_e_somem` (controlo sem veio 0). Mutação (`veio_do_fio` → 0): SANGRA (0,0000).
- **(b) O fio de pouca carga em grão: NÃO era motor.** Medido no Flat Bristle (mesma receita, 15 variantes): `visualGrain`,
  `paperVisibility`, `paperFibres/Grooves`, `paperContrast`, `pickup`, `wetLift/rewet`, `extStaining`, `gateSaturation`
  não mexem no grão (o mesmo md5); `capillary` 0 dá a risca lisa translúcida do Rebelle. A capilaridade espalha o pouco
  pigmento píxel a píxel; a tinta grossa satura e esconde-o. Sem feltro (0) aparece o pontilhado do tapete (aliasing).
- **(c) O chato pequeno esmaece** (`Knob::ThinStrokeFade`/`ThinStrokeRef`, `dab_route`): a carga do dab × (D ÷ ref)^γ
  abaixo de ref. O `Flat` do Rebelle a 5 px tem 0,228 da tinta do de 40 (o `Bristle Flat` 0,952). γ 0,6/0,9/1,2/1,6 →
  cobertura do 5 px 0,33/0,20/0,12/0,04 (Rebelle 0,14); fica 1,05. Portão `o_chato_pequeno_esmaece` (0,23; controlo 0,95;
  o Oil Bristle sem a lei 0,95). Mutação (o factor preso a 1): SANGRA (0,948).
- O portão existente `o_traco_fino_pesa_por_area_…` é de um lado só (o fino não pode ser FRACO demais): não via o defeito.

### G-1 Oil Bristle — CURADO, NÃO APROVADO (perto)
- Receita: o veio 0,9 · 0,7 · 2000 px · corte 0,35; o relevo 0,8 → **0,6** com o expoente 0,4 → **0,5** (o veio também é
  contraste: com o relevo de antes `o_relevo_nao_cresce_com_o_pincel` lia 14,8 contra 11,5 ± 2 e a escada da curva 2,62
  contra 2,55; agora 12,8 · razão 0,793 ≤ 0,824). O controlo sem relevo de `o_relevo_nao_tem_escada` tira também o veio
  (com ele o controlo lia 11,5, colado ao Rebelle 12,1). ⛔ Medido e recusado: a água por dab 0,5 (o contraste dos troços
  do Rebelle, ~42, mas a textura 6,87 e o emboss 14,2 saem do vale dos portões; a 0 o traço granula, cv 0,007 → 0,021).
- Régua **422/454** (era 422), origem **2.º dos 228 (0,717)**, vizinho fora da família Flat Block-in 1,05.
- Revisão cega (o2, o2b ×2; as chaves em `chaves/`): os **três revisores tomaram o NOSSO pelo original** em gestos e tamanhos (4 das 6
  linhas; r-o2b-A média, os outros baixa) e já ninguém vê o «pente regular»; o que listam do lado do Rebelle: a rampa que
  entra opaca (decisão 9), o 5 px com nódulos, pontas mais curtas. Nas duas linhas «sobre» os dois revisores do o2b acharam
  o Rebelle (média e baixa) e **repetem**: o azul horizontal do Rebelle **ondula na largura** (o nosso é recto e constante)
  e (média) os nossos verticais deixam um **halo claro/rosado** dos lados sobre o vermelho. Pela decisão 12 é diferença
  visível ⇒ não aprovado.
- Revisor o2c (a receita final, relevo 0,6, novo, **média**): achou o Rebelle nas 6 linhas. Repete o o2b: o gesto do
  Rebelle **ondula** (largura e direcção; 3 revisores) e as pontas dele são arredondadas contra as nossas em garfo/degrau
  (2 revisores); novo: as nossas riscas escuras «poucas, regulares, quase pretas» contra as finas e esbatidas dele (o veio
  com o relevo mais baixo fica à vista — afinar o escuro do veio para baixo de 0,9). A rampa (decisão 9) e o 5 px com
  nós do Rebelle também listados. Fila: a ONDULAÇÃO do traço (motor: o caminho que oscila devagar na normal — o
  `bristleOscillation` só desloca o tapete, não a silhueta), as pontas redondas, o escuro do veio.
- Pranchas `oil_bristle_o2.png` (água 0,5), `_o2b` (relevo 0,8), `_o2c` (final).

### G-6 Flat Bristle — CURADO, NÃO APROVADO (em estudo; ficheiro em `fixtures/presets_teste/`)
- Receita: `capillary` 0, a carga quase constante (força 0,8, ruído 2000 px), a ponta 0,3, `thinStrokeFade` 1,05.
- Régua 364 → **367/454**; **1.º dos 228 do Rebelle (0,838)**; vizinho Oil Bristle 1,07 (claramente longe).
- Revisor cego (o2c, novo): achou o Rebelle nas 6 linhas (**baixa**), o nosso a imitação, o vizinho «claramente outro».
  Diferenças: o vermelho da base **opaco e saturado** (cobertura 0,92 contra 0,72 do Rebelle — o mesmo pincel, a mesma
  carga: o vermelho do K–M a 200 de pigmento cobre mais que o azul, que casa 0,72 × 0,67; não é do preset), quadrados mais
  escuros onde o azul cruza o azul, a rampa que começa no zero (decisão 9), pontas desfiadas contra ovais, a estria fina e
  regular. O grão e o 5 px firme da 1.ª ronda já não aparecem.
- ⛔ Portão `a_pressao_de_cada_preset_e_a_da_sua_origem`: **0,202** (barra 0,18). A distância é a entrada tardia do Rebelle
  (nada abaixo de p 0,2: a largura do decil 2 da afunila é 0 nele, 0,95 em nós). Medido: curva da carga 1/1,5/2/2,5/3 →
  0,35/0,29/**0,20**/0,23/0,25; a largura pela pressão piora (0,21–0,60). Só um piso a copia — a decisão 9 proíbe-o.
  **Decisão para o coordenador**: o Flat do Rebelle fica fora da barra (como os Flats do dono a 0,17–0,18), ou o pincel
  não entra. O Flat Bristle está no `leis_rebelle.json` (Flat/Flat, medido) e no `ORIGEM`: entra na fábrica ⇒ o portão vê-o.

### G-2 Acrylic Round — RETIRADA (ordem do coordenador; fundida no Hard Round)
Preset apagado; `MIDIAS`; `a_lei_da_tinta` (21 → 20 cenas); `os_fios::FIOS` (18 → 17); `o_relevo_dos_fios::OLEOS`,
`sonda_spacing_continuo`, `sonda_shape_casa` passam ao **Filbert** (o óleo de pegada redonda que ficou); `leis_rebelle.json`
e `exame_matriz.ORIGEM` trocam-na pelo **Flat Bristle** (`Flat/Flat`, as leis medidas por `leis_oraculo.leis` — o mesmo
código dá o Filbert ao byte). Portões tocados verdes por nome (28/29: o que falta é o Wet Liner da família aguarela).

### G-12 Wet Blend Round — NÃO ENTRA (o Soft Round já o pinta)
Os dois oráculos (Krita `i__Wet_Paint`, MyPaint `tanda/acrylic-04-with-water`) **não mostram o «puxar» na bateria**: nos dois
o azul cobre o vermelho sem o arrastar (`pranchas/oleo_acrilica/soft_round_wb_krita.png`). O do Krita é um redondo macio
com a entrada a esbater (o pincel começa carregado de papel): o **Soft Round** de fábrica dá **366/454** contra ele (o Wet
Round 192, o Express Oil 214). Sem alvo que meça a troca de pigmento, o motor «molhado × molhado» (deste dono) fica na
fila sem pincel que o prove; o candidato natural é o Wet Mix da Watercolor (`wet_charge`/`wet_pull`, transparente).

### G-14 Rigger — FUNDIDO no Hard Round (decisão 13)
O Hard Round contra o `Round/Liner` do Rebelle: **404/454** (contra o `rigger_brush` do MyPaint 308/432); a prancha
(`hard_round_rigger_liner.png`) mostra a mesma largura, o mesmo afilar pela pressão e a mesma cobertura — só a orla de
fiapos de cerdas difere, que é o Filbert. O «fio longo e fino que dura» é o Hard Round pequeno.

### G-13 Nikko Rull — SEM ALVO
O manual do Procreate não descreve o pincel (só os conjuntos); sem foto/vídeo lido não há alvo visual. Fila.

### G-11 Dry Brush, G-8 Fan, G-9 Rake, G-7 Palette Knife — não tocados nesta ronda
O Flat Dry do Rebelle são 4–6 tufos largos com papel entre eles, grão dentro e orla escura nítida (outro desenho que o (b));
o Fan pede a dispersão da família texturas.

### Custo
p95 a 5/40/200 px (smoke, load 21–24 — o relógio NÃO vale; a J5 re-mede calma): **Oil Bristle 2,7 · 9,0 · 17,3 ms**
(1.ª ronda 10,9 · 12,1 · 28,5 a load 21), RSS 79–101 MiB; **Flat Bristle 1,3 · 18,5 · 16,0 ms**, 78–100 MiB. O veio custa
um Perlin por FIO por dab (na tabela do dab), o esmaecer uma potência por lote: nenhum por célula.

### O que fica para a fila
- Motor: a ONDULAÇÃO lenta do traço (Oil Bristle, 3 revisores); a troca de pigmento molhado × molhado sem oráculo que a
  mostre (G-12); o vermelho do K–M mais opaco que o azul à mesma carga (Flat Bristle; afecta todos os Wet ralos).
- Decisão: o Flat Bristle e a barra 0,18 da pressão contra a decisão 9 (acima). O Dabber fica como está (coordenador).
- Por fazer: G-11 Dry Brush, G-8 Fan, G-9 Rake, G-7 Palette Knife, G-13 Nikko Rull (sem alvo).
