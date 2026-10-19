# 52 — Os pincéis por fazer, medidos contra o Rebelle (2026-10-19)

> Continuação do [18](18_mapa_das_midias.md) e do [51](51_medicao_spacing_continuous.md). Pedido do dono (19/10): *adiar os 4
> abertos do P51 e «ver quantos pincéis faltam»*; depois, *«vamos nisso»* = medir. Nenhum preset foi criado nem gravado: é a lista do
> que a composição de HOJE **já exprime**. Saídas da régua em [`52_regua/`](52_regua/) (as três passadas, os vizinhos de fábrica, as
> receitas em JSON, a prancha das 24 fotos).

## §0 — A lista de partida: 19 pincéis

A lista é o plano de arranque do [02](02_presets_para_wet_painter.md) §6 (escrito **antes** de «só o Wet Paint evolui», 07/10) contra os 20 presets
de fábrica: 22 faltavam. Saem 3: **Spray 2** = o nosso Airbrush (afinado contra a foto `airbrush` = Default/Spray 2); **Dry Thin** = o nosso
Soft Pastel (afinado contra `pastel` = Pastel Dry/Dry Thin); **Metallic Flat** não existe no Rebelle 8
([`rebelle_presets_281.json`](fixtures/rebelle_presets_281.json)). O «Marker Effect» do 02 é a família **Marker/Effect** do Rebelle (7: Burn,
Color, Dodge, Hue, Luminosity, Overlay, Saturation). **Ficam 19.**

## §1 — Os dados

- **24 fotos novas** pela porta WebSocket (`ferramentas/fotos_lote.sh` + `lote_19.txt`; tela virtual por `gotas_inicia.sh`, fecho por
  `gotas_fecha.sh`). O teste do Rebelle tinha **18 dias** a 19/10 ⇒ expira ~06/11: foto nova é para agora.
- **16 entraram** em [`fixtures/rebelle/`](fixtures/rebelle/README.md) (estado A = B, salvo as duas tintas: ~900 px de diferença), com
  cabeçalho e linha no README: granulation · flatoily · filbert · knife · rake · dabber · pencil2h · pencilhb · pencil2b · pencil4bcolor ·
  scratch · squarehard · waxscrape · fountainsquare · reedpen · markerchisel.
- **Sem foto:** Spatter (quase transparente sob o Wine a 40 px, a 80 px e com water 60/opacity 100: o papel do Rebelle não se desenha sob o
  Wine, e a régua não acha traço); os 7 Marker/Effect (a cena «Marker Bullet + efeito por cima» pintou **0 px** do efeito amarelo sobre o
  traço seco — por tentar sem o `DRY` entre base e efeito).
- **Granulation tem foto, mas a textura é do papel** (`tex` = nan na régua) ⇒ sem oráculo medível da identidade.
- ⚠️ **Armadilha:** um `SET_BRUSH` com preset inválido deixa diálogos de erro **em fila** no Rebelle (18 fechados à mão), que estragam as fotos
  seguintes. `inj.sh "rect e"` antes de um lote. O `fotos_lote.sh` distingue agora a base `TOOL:Grupo/Nome` dos argumentos `--…` do gerador.

## §2 — A régua

`ferramentas/avalia_19.py` = a barra do [18 §2](18_mapa_das_midias.md): uma métrica **conta** se |C−R| > 3× o ruído do Rebelle; passa quem
fica do lado do Rebelle do ponto médio (R–C); a largura passa a ± 2 px. Sonda `sonda_midias` com receitas por JSON a partir do vizinho de
fábrica (`base`), **seca** (`PH2D_SONDA_SECA=1320`), perfil `smoke`. A sonda passou a aceitar **qualquer knob do registo** numa receita com
`base` (antes só os que o preset grava). Custo: ~3 min por ~100 receitas (load ~5).

## §3 — As três passadas (série justificada: cada uma escolhe a seguinte)

| Passada | Receitas | O que mudou | Saída |
|---|---|---|---|
| 1.ª | 377 (`receitas_19.py`) | grelhas por família a partir do vizinho | [passada1](52_regua/passada1_377_receitas.txt) |
| 2.ª | 87 (`receitas_19b.py`) | lápis à **largura corrigida** (a lei raio→largura da sonda lê o traço molhado e a régua o seco com grão: 3–5 px a menos) + grelhas novas no eixo que reprovou | [passada2](52_regua/passada2_87_receitas.txt) |
| 3.ª | 55 (`receitas_19b.py --c`) | Scratch, Dabber, Flat Oily, Knife | [passada3](52_regua/passada3_55_receitas.txt) |

## §4 — O veredito, pincel a pincel

«Passa» = todas as métricas que contam + largura. «Fábrica» = o melhor vizinho **sem mudança** ([vizinhos_de_fabrica](52_regua/vizinhos_de_fabrica.txt)).
Receitas completas nos `receitas_19*.json` (o nome é o do lote); `tex` Noise = `brush.texture.kind Noise` com o `size` indicado.

**A. NOVOS de verdade — só com valores (11).** Nenhum vizinho de fábrica os passa.

| Pincel | Vizinho | Receita que passa | Métricas que contam | Passa | Fábrica sem mudança |
|---|---|---|---|---|---|
| 2H | Graphite 8B | `lapisb_027`: pigmentPerDab 40, paperGate 0,3, Noise 16 (alvo 28,8) | cob · tex · borda · orla · cruz · larg | 6/6 | 4/6 (Graphite 8B) |
| HB | Graphite 8B | `lapisb_028`: pigmentPerDab 40, paperGate 0,3, Noise 32 (alvo 30,3) | idem | 6/6 | 3/6 |
| 2B | Graphite 8B | `lapisb_032`: pigmentPerDab 40, paperGate 0,65, Noise 64 (alvo 28,8) | idem | 6/6 | 3/6 |
| 4B Color | Graphite 8B | `lapisb_041`: pigmentPerDab 80, paperGate 0,65, Noise 64 (alvo 30) | idem | 6/6 | 4/6 |
| Scratch | Charcoal | `lapisc_053`: pigmentPerDab 20, paperGate 0,5 (alvo 32) | cob · tex · borda · orla · cv · cruz · larg | 7/7 | 3/7 |
| Square Hard | Soft Pastel | `sq_007`: pigmentPerDab 60, paperGate 1, hardness 0,2, dab_quina 1, Noise 100 (alvo 31) | cob · tex · borda · cv · cruz · larg | 6/6 | 3/6 (Oil Pastel) |
| Wax Scrape | Soft Pastel | `wx_007`: pigmentPerDab 150, paperGate 1, hardness 0,2, dab_quina 0, Noise 100 (alvo 36) | cob · tex · borda · cruz · larg | 5/5 | 3/5 (Oil Pastel) |
| Filbert | Acrylic Round | `filbert_016`: bristleSize 2, pigmentPerDab 500, hardness 0,5 (alvo 40) | cob · tex · borda · larg | 4/4 | 3/4 (Oil Bristle) |
| Dabber | Oil Bristle | `dabberc_032`: bristleSize 2, emboss 1,2, bristleStrength 2, jitter 0,5, hardness 0,6 (alvo 36) | cob · tex · borda · orla · larg | 5/5 | 3/5 |
| Flat Oily | Acrylic Round | `laminac_018`: bristleStrength 0, felt 1, pigmentPerDab 1000, dab_flatten 0,82, hardness 0,9 (alvo 10) | cob · tex · borda · cv · cruz · larg | 6/6 | 4/6 (Dip Pen) |
| Knife | Acrylic Round | `laminac_012`: bristleStrength 0, felt 1, pigmentPerDab 500, dab_flatten 0,82, hardness 0,9 (alvo 10) | idem | 6/6 | 5/6 (Dip Pen) |

⚠️ **2H e HB diferem pouco** no Rebelle (cob 0,595 / 0,561): a melhor receita de cada um passa no outro (`lapisb_028` é a 2.ª do 2H e a 1.ª do HB).
Na 3.ª passada o Flat Oily e o Knife convergem à mesma família (10 de 18 receitas passam o Knife): é uma LÂMINA chata sem cerdas.

**B. JÁ COBERTOS por um vizinho na régua (4).** Um preset com este nome hoje seria «um nome, não uma mídia» ([18 §4.3](18_mapa_das_midias.md)).

| Pincel | Vizinho de fábrica, sem mudança | Passa | O que a régua do recto não vê |
|---|---|---|---|
| Rake | Oil Bristle (`rakev_OilBristle`) | 5/5 | os dentes do ancinho |
| Fountain Square | Ink Brush (`fountainv_InkBrush`) | 4/4 | a ponta chata, lida de lado |
| Reed Pen | Ink Bullet e Dip Pen | 5/5 e 5/5 | a ponta chata |
| Marker Chisel | Marker Liner e Marker Bullet | 6/6 e 6/6 | o Chisel lê **16 px** no recto a size 40: a forma é a ponta |

A identidade destes é a **forma da ponta** (chata, lida de lado), e a régua do recto não a mede: falta a métrica da ponta/curva.

**C. Precisa de MOTOR (1 família).** **Marker/Effect (7).** O Wet Paint não tem modo de mistura do pincel; há um campo `blend` em `CAMPOS_DO_PINCEL`
— por medir se o Wet Paint o lê.

**D. Sem oráculo medível (2).** **Granulation** (textura do papel) e **Spatter** (sem traço sob o Wine). O motor tem `extGranulation` e `spray_count`:
por medir contra outro oráculo ou contra fotos do dono.

**E.** **Sumi-e** — já medido a 11/10 ([18 §3](18_mapa_das_midias.md), linha do nanquim, foto `sumi.png`): FORA, a borda macia sai
manchada (orla 8–25 contra 9, textura 11–24 contra 3,4). Não varrido de novo; a forma por inclinação dele (02 §6.1) é motor.

> **Contagem dos 19:** 11 (A) + 4 (B) + 1 família Marker/Effect (C; 7 presets do Rebelle) + Granulation e Spatter (D) + Sumi-e (E).

## §5 — ⛔ Recusas MEDIDAS

| Recusa | Medido |
|---|---|
| Os 7 efeitos na **mesma cor** da base | Hue/Color/Saturation de azul sobre azul não mudam nada; ~1 000 px de diferença entre eles |
| Spatter a 80 px e com mais tinta | a régua continua sem achar traço (papel do Rebelle não se desenha sob o Wine) |
| A lâmina **com cerdas** (Acrylic Round chata com cerdas) | textura 6–8 contra 0,8–1,0 do Rebelle; passou só **sem cerdas, felt 1** |
| Aceitar Rake/Fountain/Reed/Chisel como «novos» | passam pelo vizinho: seria um nome sem mídia (18 §4.3) |

## §6 — ⚠️ Limites e o que fica ABERTO

**Limites:** uma foto por pincel, a 40 px, pressão 100, sem inclinação; o recto + o cruzamento. As receitas passam **esta** régua, não a mão do dono.

**Aberto:**
1. Criar os 11 presets: o gerador `presets_de_fabrica` + `receitas_de_fabrica.rs`, `PRESETS_DE_FABRICA` contado, o portão `cada_preset_parece_a_sua_midia` por foto nova.
2. A métrica da **forma da ponta** (a ponta/curva) para decidir os 4 cobertos.
3. A cena do Marker/Effect (tentar sem o `DRY` entre base e efeito) e a pergunta do campo `blend`.
4. O oráculo do Granulation/Spatter.
5. Os 4 abertos do P51, **ADIADOS pelo dono** ([51 §8](51_medicao_spacing_continuous.md)).
