# EXAME da família `tintas` (J3, catálogo C)

Receitas e portões: `familias/tintas.rs`. Pranchas: `pranchas/tintas/` (`*_o2` = 2.ª onda; cegas e chaves ao lado).
Réguas, custo e a régua do borrão: `exames/tintas/`. Bancada: `PH2D_EXAME_DIR=/tmp/ph2d_exame_tintas`. A 1.ª onda inteira
(receitas, os 19 revisores, as fotos refeitas) está no commit `d28b9d7768`.

## 2.ª onda (20–21/10) — vereditos

| pincel | veredito | porquê (revisor cego NOVO, cruzado com a chave) |
|---|---|---|
| C-1 Dip Pen (existente) | ⛔ fica com a cura | oráculo certo 6/6; abaixo: «o que falta» |
| C-2 Ink Bullet (existente) | ⛔ fica com a cura | escolheu o NOSSO como original 6/6, mas as diferenças são visíveis |
| C-3 Ink Brush (existente) | ⛔ fica com a cura | oráculo certo 6/6 |
| C-9 Reed Pen | ⛔ reprovado → SAI (pendente) | o NOSSO como original 6/6 (2.ª vez), diferenças visíveis; o `git rm` espera a `caligrafia_marcadores` (mede-o em `MEDIDOS`) |
| C-15 Wood Stick | ⛔ saiu (`612a1046ae`) | oráculo certo 6/6 |
| C-12 Gloop Pen | ⛔ saiu | ArtRage certo 3/3 |
| C-13 Ink Bleed | ⛔ saiu | a capilaridade pelas fibras não existe (medido, abaixo) |
| C-14 Gesinski Ink (doc.) | ⛔ saiu | cumpre translúcido/escurece/borda limpa; falta a densidade viva (motor) |

## O motor: o BORRÃO do cruzamento — a ponta de tinta líquida é um RESERVATÓRIO (`4d70916e0d`, `51acfb80e7`)

**A pista da parede (J2, `levelingOwnStroke`) não era a causa:** a lei é por pincel e só o Oil Bristle a liga; as tintas
nunca a tiveram. **Medido no Rebelle** (`exames/tintas/borrao.py`, mistura = G < 0,65 do azul limpo e R < 110; fixture
`borrao_rebelle.json`): a mistura passa a faixa SECA 11–38 px acima / 5–29 abaixo e a MOLHADA 11–50 / 2–29; é ESCURA (R
26–39 — o produto, nunca o roxo); os marcadores 0; os óleos não (o Bristle Flat 0). Escurece **um raio ANTES** da faixa
(pela pegada inteira) e também sobre a SECA. O nosso: 0 px acima. Mecanismo: a ponta do Wet guarda a cor por TEXEL (só o
ponto do bico que tocou), recolhe por janela com `Pickup` ≤ 0,2 (o carregador corta acima), pela média (roxo), e o produto
não apanha o assentado (`pickup_assentado` 0).

**A lei** `Knob::TipReservoir` (`WP/trail/transfer.rs`; 0 = ao bit): a fracção `r` da tinta debaixo da ponta entra a cada
DAB (`k^dabs`); o pigmento de OUTRA tinta absorve na da ponta (produto; `outra_tinta` 0 = a mesma não escurece); a tinta
líquida re-molha a seca; a pegada passa à média. Receitas: Dip/Bullet/Reed `r` 0,4 e Glaze → 0,5 (a 0,9 o cruzamento ia
a L 11, Rebelle 32); Ink Brush `r` 0,2 + `tipClean` 0,05 (sem limpeza o traço inteiro ficava preto), Glaze 0,4.
**Portão** `o_borrao_da_tinta_transborda_a_faixa` (por tinta, ≥ 0,4× o Rebelle na seca, ≥ 0,25× na molhada, R ≤ 60, o
Marker Bullet 0): vermelho antes (0 px), verde (Dip 13/14 · Bullet 20/21 · Brush 8/11 · Reed 15/14 molhada/seca acima).
**Mutação**: o assentado fora · a média em vez do produto · sem a média da pegada · por janela em vez de por dab — as
quatro sangram. **Custo** (`custo_o2.txt`, load 4–22): p95 a 200 px Dip 15,0 · Bullet 14,9 · Reed 10,2 ms; Ink Brush
26,6 (sem tecto; era 63,7).
**O tecto** (decisão 8): `size_max_px` = 40·(largura a 200 ÷ a 40) do Rebelle — Dip 91 · Bullet 93 · Reed 95 (Max Size
100 pinta ~0,93); o Ink Brush não tem (183 a 200). O 200 px deixa de ser o dobro.
**Recusas medidas (a subida A MAIS da molhada — Dip 45 e Reed 40 contra 21 e 26 na seca — e o alastrar para dentro do
vermelho molhado, 13–21 px):** `leveling` 1,5–2 (+5 px no Dip só com a régua larga, 0 com a estrita), `capillary` 0,6–1,
`viscosity` 0, `extDiffusion` 0,25 com `waterPerDab` 2–4 e evaporação 0,05–0,2 (o filme da tinta ≤ 0,1: o `diffusion_pass`
salta; mais água espalharia no papel, que o Rebelle não faz). `r` 0,5–0,8: +2 px e o cruzamento ao preto. ⇒ **motor**:
a troca de pigmento entre duas tintas MOLHADAS que não molha o papel seco.

## Os outros abertos de motor (medidos)

- **A entrada clara** (Bullet 0,94 → 1,00 em 60 px no Rebelle; Dip 0,957): a 2000 de pigmento a tinta satura; um taper
  de opacidade 1 em 1,5–3 diâmetros só clareia os primeiros ~10 px (0,16 → 1,0). Falta uma carga na opacidade VISÍVEL.
- **A capilaridade pelas fibras** (CAP-14): `extFingering` 1, `capillary` 1, `paperFibres`/`paperGrooves` 3,
  `capillaryGate` 100, água 3–4, evaporação 0,05–0,2 — a borda nunca sai da pegada.
- **O grão sob o tecto do traço** (Gesinski): Grain Size 48–128 a profundidade 0,6–1 e `flow` 0,6 = a mesma imagem (os ~40
  dabs de cada píxel somam ao tecto). A 2.ª ponta com grão só morde a orla. Falta o grão a modular o TECTO do plano
  (`traco.rs::compoe` + `traco_plano.rs` + o campo no preset, painel e macro — não é pequeno).
- **A orla granulada do Gloop** (salpicos a 200 px no ArtRage): não tentada a fundo (o Gloop saiu pela pressão e pelo tom).
- **A borda serrilhada do Rebelle** (Dip, Wood: ±3 px em trechos de 10–20): `paperGate` 0,3–0,9 com grumos 0–0,08 deixa
  a borda LISA; os grumos (tamanho por dab) só dão contas regulares.

## Por pincel (2.ª onda)

**C-1 Dip Pen** — régua 430 → 438/454; vizinho Ink Brush 0,839; oráculo 1.º. Revisor (oráculo certo 6/6): «(1) a
imitação não reage à tinta molhada: o original incha numa bolha roxa além do traço e faz nódoas azul-sobre-azul (2) borda
serrilhada × fila de círculos lisos (3) 5 px e pressão baixa pontilhados». Falta: o alastrar da molhada (motor), a
serrilha, a ponta leve (a entrada/saturação), o azul sobre azul (o Glaze 0,5 baixou-o: 1,05 contra 1,18).

**C-2 Ink Bullet** — régua 412 → 418/454; vizinho Dip 0,942; oráculo 1.º. Revisor: o NOSSO como original 6/6 (média);
o que ele leu no Rebelle: «início translúcido (o loading), o escuro que passa a faixa em discos e a cauda cinza, borda
lisa». Falta: a entrada clara (motor), o borrão ainda metade (20 contra 40 na molhada), o tom.

**C-3 Ink Brush** — régua 282 → 296/454; vizinho Dip 0,839; oráculo 2.º. Revisor: «(1) mais escuro e saturado, grão de
spray × estrias de cerda (2) a pressão do original comanda a OPACIDADE (rampa que nasce transparente) e a nossa a largura
(3) o original alastra em trapézio no molhado e distingue seco/molhado; o nosso confina e é igual». Falta: o tom, as
estrias a 200 px, a pressão → opacidade (não é a decisão 9: é o canal, não o ponto de entrada), o alastrar (motor).

**C-9 Reed Pen** — régua 405 → 418/454; vizinho Wood 0,980. Revisor: o NOSSO como original 6/6 pela 2.ª vez (média), mas
lista: o borrão redondo maior do Rebelle e as nódoas azul-azul, o corte a esquadro e os degraus da borda, o 200 px mais
estreito, a ponta leve do Rebelle lisa × o nosso grão. Reprovado (§3: as diferenças são do nosso lado reconhecível).

**C-15 Wood Stick** — régua 390 → 384/454 (sem o nivelamento a entrada volta clara: 0,88; o portão de identidade a pede).
Revisor: «contas × serrilha fibrosa; a fresta clara no vermelho ao lado do azul; o disco transparente da entrada». Saiu.

**C-12 Gloop Pen** — ArtRage: composição NORMAL a ~0,65 (azul sobre vermelho (74,81,133), azul sobre azul 1,38×) ⇒ Multiply
→ Mix, tecto 0,65 (1,35×); o Gesinski a 1,135 (era 0,884). Régua 138/227. Revisor: «a pressão (o ArtRage quase não
afina, cabeça redonda), a orla granulada com salpicos, o tom lavado». Saiu.

**C-14 Gesinski Ink** — tecto 0,55 → 0,7 (o «aguado»); vizinho Hard Round 1,099. Revisor (identidade escrita, 2 colunas
sem rótulos, `chaves/gesinski_ink_o2.json`): cumpre translúcido, escurece e borda limpa; «(1) forma vetorial chapada,
sem tom nem grão (2) pressão e velocidade só mexem na largura (3) a sobreposição é uma conta de opacidade; pastel a 200
px». Saiu (o grão sob o tecto é motor).

**C-13 Ink Bleed** — sem revisor novo: a falta do 1.º (não sangra pelas fibras) é a CAP-14, medida acima. Saiu.

## 1.ª onda (resumo)

Fotos do Rebelle refeitas (`foto_estavel.py`); o ArtRage Gloop fotografado. O C-8 Fountain fundiu-se no Ink Bullet. O
motor da ponta ROMBA do taper (`bf73f3a58b`, `Taper::fade_blunt`). O portão `cada_tinta_tem_a_sua_identidade` (agora
Dip, Bullet, Brush, Reed). Custo da 1.ª: load 16–94, sem valor.
