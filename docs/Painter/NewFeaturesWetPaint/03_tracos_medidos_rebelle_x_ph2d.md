# 03 — Os mesmos gestos nos dois programas, medidos com a mesma régua (2026-10-07)

> **Pergunta:** *porque é que o traço do Rebelle é superior em algumas características?* Responde-se
> EXECUTANDO (método §1): os cinco gestos da nossa sonda (`crates/ph2d-tool-painter/tests/it/sonda_gestos.rs`)
> foram enviados ao Rebelle pela porta oficial WebSocket (eventos Motion IO com pressão) e pintados pelos
> nossos quatro meios; as duas pranchas foram medidas por [`ferramentas/mede_traco.py`](ferramentas/mede_traco.py).
> As leis que isto fundamenta estão no [01 §2](01_o_que_o_rebelle_tem_e_nos_nao.md); aqui ficam a bancada,
> os números e o que NÃO se pode concluir deles.

## §1 — A bancada

| | Rebelle 8.3.4 (Pro, teste) | PH2D (`main` 2a925859d) |
|---|---|---|
| tela | `NEW_ARTWORK` 1000×640 px, papel `Default/EX00 White Simple` branco, `SET_RANDOM_SEED 7` | `set_source` 1000×640 branco |
| gestos | os mesmos, gerados por [`gera_eventos.py`](ferramentas/gera_eventos.py): recto 500 px a 3 px/evento · curva em S (amp. 25 px, 1,5 períodos) · rampa de pressão 0,15→1→0,15 · recto RÁPIDO a 25 px/evento · recto + cruzamento vermelho + recto vermelho | idem, `gesto()` na sonda; `on_tick(8 ms)` por evento; 8 s de relógio no fim |
| pincel | ferramenta + preset de fábrica, `size_px 40` (reenviado em `SET_BRUSH` antes de CADA gesto — medido: só vale para o 1.º traço), pressão 100, azul `#1765b3`, vermelho `#bf0012` | o pincel de fábrica de cada meio, `set_brush_size_px(28)` (dá ~40 px de largura no Digital), `strength 1`, as mesmas cores |
| imagem | fotografia da janela a zoom 100 % (`foto.sh`), recorte da tela 1000×640 em (480, 220) — `SAVE` ainda não existe por WebSocket | `run_full()` → PPM → PNG |
| estados | A = depois dos traços + `SIMULATION 240`; B = depois de `DRY` (iguais em tudo o que se mediu: a aquarela já estava seca ao fim da simulação) | um só (8 s de ticks) |

**Pranchas:** [`prancha_rebelle.png`](prancha_rebelle.png) (12 painéis: óleo Bristle Flat · Express Oil Hairy ·
Watercolor Flat 2 (opacidade do painel, 100) · Ink Bullet · Pencil 8B · Pastel Dry Thin · Marker Liner · Airbrush
Spray 2 · Watercolor Flat 2 a opacidade 50/água 30 (seca) · Watercolor Round a opacidade 60/água 80 · Pencil 8B
inclinado 55° · Bristle Flat com Dirty Brush em Paint & Mix) · [`prancha_ph2d_size28.png`](prancha_ph2d_size28.png)
(Digital · Watercolor · Impasto · Wet Paint) · [`prancha_ph2d_size40.png`](prancha_ph2d_size40.png).

**A régua** (por recorte: recto = linha 1 esquerda; rápido = linha 2 direita; rampa = linha 2 esquerda):
largura mediana por coluna (50 % do máximo) · taper = largura a 5/50/95 % do comprimento · **orla** = transição
10→90 % na borda (px) · **CV** = variação da cobertura ao longo do traço · **período** = pico do espectro do
perfil (e a fracção da potência que leva — o «colar de contas») · **textura** = desvio do resíduo 5×5 no miolo,
em % da cobertura · **borda/miolo** = cobertura na orla de 3 px sobre a do miolo (> 1 = borda escura) ·
**cobertura** do miolo · cor média do miolo (sRGB).

## §2 — Os números

### Rebelle (estado A; o preset entre parênteses)

| traço | larg. px | taper 5/50/95 | orla px | orla/larg | CV | período px (% pot.) | textura % | borda/miolo | cobertura | cor sRGB |
|---|---|---|---|---|---|---|---|---|---|---|
| Oils (Bristle Flat) recto | 41 | 41/41/0 | 2 | 0,05 | 0,011 | 25,8 (9 %) | 5,9 | 0,87 | 0,91 | 23,102,179 |
| Oils rápido | 42 | 42/44/41 | 1 | 0,02 | 0,008 | 37,0 (17 %) | 6,1 | 0,84 | 0,91 | 23,101,177 |
| Oils rampa | 39 | 35/42/33 | 1 | 0,03 | 0,038 | 16,2 (7 %) | 7,4 | 0,82 | 0,90 | 24,104,180 |
| Express (Hairy) recto | 42 | 40/42/30 | 1 | 0,02 | 0,008 | **6,0 (47 %)** | 9,0 | 0,80 | 0,70 | 76,136,196 |
| Express rápido | 40 | 34/40/36 | 5 | 0,12 | 0,017 | 6,0 (34 %) | 9,9 | 0,62 | 0,65 | 88,144,200 |
| Express rampa | 37 | 29/39/24 | 2 | 0,05 | 0,166 | 20,6 (16 %) | 8,2 | 0,76 | 0,57 | 109,158,207 |
| Watercolor (Flat 2, opac. 100) recto | 42 | 41/42/0 | 1 | 0,02 | 0,001 | 7,0 (23 %) | 1,0 | 0,83 | 0,91 | 24,102,179 |
| Watercolor rápido | 42 | 42/41/41 | 1 | 0,02 | 0,001 | 7,0 (18 %) | 0,9 | 0,83 | 0,91 | 24,102,179 |
| Watercolor rampa | 42 | 32/42/33 | 1 | 0,02 | 0,055 | 13,5 (17 %) | 1,2 | 0,80 | 0,81 | 49,118,187 |
| Watercolor (Flat 2, opac. 50 · água 30) recto | 41 | 41/42/1 | 1 | 0,02 | 0,011 | 2,3 (9 %) | 2,3 | **1,04** | 0,43 | 145,182,219 |
| idem rampa | 42 | 34/42/36 | 1 | 0,02 | 0,072 | 2,8 (15 %) | 3,0 | 1,01 | 0,39 | 156,189,222 |
| Watercolor (Round, opac. 60 · água 80) recto | 31 | 30/30/9 | 1 | 0,03 | 0,045 | 2,1 (4 %) | 3,0 | 0,84 | 0,38 | 157,190,223 |
| idem rápido | 30 | 31/31/28 | 1 | 0,03 | 0,057 | 2,3 (10 %) | 3,0 | 0,83 | 0,37 | 160,192,224 |
| idem rampa | 25 | 13/28/14 | 1 | 0,04 | 0,094 | 2,4 (3 %) | 3,3 | 0,82 | 0,33 | 171,199,228 |
| Inks (Bullet) recto | 42 | 42/42/27 | 1 | 0,02 | 0,002 | 2,0 (7 %) | 0,4 | 1,00 | 0,86 | 35,109,183 |
| Inks rampa | 37 | 28/41/28 | 1 | 0,03 | 0,014 | 32,8 (5 %) | 0,7 | 0,99 | 0,85 | 37,110,184 |
| Pencils (8B) recto | 30 | 28/30/6 | 7 | 0,23 | 0,037 | 24,8 (7 %) | **9,1** | **0,32** | 0,80 | 52,120,188 |
| Pencils rampa | 28 | 20/36/20 | 8 | 0,29 | 0,117 | 19,3 (10 %) | 9,8 | 0,30 | 0,73 | 70,132,194 |
| Pencils 8B **inclinado 55°** recto | **37** | 29/36/38 | 5 | 0,14 | 0,025 | 16,0 (4 %) | 5,2 | 0,33 | 0,86 | 36,110,183 |
| idem rampa | 33 | 20/39/15 | 6 | 0,18 | 0,138 | 29,4 (14 %) | 6,3 | 0,32 | 0,78 | 56,123,190 |
| Pastels (Dry Thin) recto | 17 | 17/18/0 | 6 | 0,35 | 0,042 | 8,2 (3 %) | 7,2 | 0,44 | 0,75 | 63,128,192 |
| Pastels rampa | 16 | 8/16/9 | 6 | 0,38 | 0,053 | 9,4 (3 %) | 8,1 | 0,47 | 0,73 | 68,131,194 |
| Markers (Liner) recto | 18 | 17/18/14 | 2 | 0,11 | 0,001 | 4,0 (24 %) | 2,1 | 0,72 | 0,91 | 23,101,179 |
| Markers rampa | 16 | 12/17/12 | 2 | 0,12 | 0,019 | 52,7 (7 %) | 2,1 | 0,70 | 0,91 | 23,101,179 |
| Airbrushes (Spray 2) recto | 26 | 21/22/17 | 12 | 0,46 | 0,088 | 17,8 (5 %) | 22,7 | 0,54 | 0,38 | 158,191,223 |
| Airbrushes rampa | 21 | 22/25/18 | 13 | 0,62 | 0,329 | 7,9 (4 %) | 23,7 | 0,48 | 0,33 | 172,200,228 |
| Oils Bristle Flat **Dirty Brush, Paint & Mix** recto | 41 | 41/41/0 | 2 | 0,05 | 0,011 | 25,8 (9 %) | 5,8 | 0,87 | 0,91 | 23,102,180 |

### PH2D (`set_brush_size_px 28`)

| traço | larg. px | taper 5/50/95 | orla px | orla/larg | CV | período px (% pot.) | textura % | borda/miolo | cobertura | cor sRGB |
|---|---|---|---|---|---|---|---|---|---|---|
| Digital recto | 40 | 40/40/26 | **10** | 0,25 | 0,000 | 28,1 (13 %) | 0,7 | 0,24 | 0,79 | 55,122,189 |
| Digital rápido | 40 | 38/40/36 | 10 | 0,25 | 0,000 | 7,0 (10 %) | 0,7 | 0,22 | 0,79 | 54,121,189 |
| Digital rampa | 30 | 12/40/8 | 8 | 0,27 | 0,039 | 32,3 (8 %) | 1,3 | 0,28 | 0,81 | 50,119,188 |
| Watercolor recto | 48 | 50/51/41 | 3 | 0,06 | 0,039 | 25,1 (11 %) | 6,9 | **1,23** | 0,40 | 153,199,232 |
| Watercolor rápido | 47 | 47/50/53 | 3 | 0,06 | 0,034 | 32,2 (17 %) | 6,7 | 1,24 | 0,40 | 153,199,232 |
| Watercolor rampa | 36 | 13/50/10 | 2 | 0,06 | 0,108 | 21,2 (12 %) | 7,4 | 1,21 | 0,44 | 143,194,230 |
| Impasto recto | 51 | 51/51/44 | 3 | 0,06 | 0,007 | **5,6 (42 %)** | 1,8 | 0,72 | 0,89 | 28,96,165 |
| Impasto rápido | 51 | 51/51/49 | 3 | 0,06 | 0,007 | 5,6 (41 %) | 1,5 | 0,73 | 0,89 | 27,95,163 |
| Impasto rampa | 39 | 17/51/13 | 2 | 0,05 | 0,040 | 5,6 (8 %) | 2,7 | 0,74 | 0,88 | 30,96,162 |
| Wet Paint recto | 28 | 28/29/15 | 6 | 0,21 | 0,036 | 19,4 (5 %) | **14,4** | 0,61 | 0,78 | 57,123,190 |
| Wet Paint rápido | 28 | 29/28/22 | 6 | 0,21 | 0,040 | 17,4 (6 %) | 14,3 | 0,60 | 0,78 | 57,123,190 |
| Wet Paint rampa | 20 | 8/28/8 | 6 | 0,30 | 0,066 | 17,6 (6 %) | 13,0 | 0,65 | 0,79 | 54,121,189 |

(A `size 40` nossa, para referência: Digital 58 px / orla 13, Watercolor 68 / 3, Impasto 72 / 3,5, Wet Paint 44 / 14 —
ficheiro `prancha_ph2d_size40.png`. ⚠️ O mesmo número de tamanho dá quatro larguras nos nossos meios; o Rebelle
dá 41–42 px nos quatro meios «de tinta» — é um defeito de coerência nosso, a corrigir na W1.)

## §3 — O que os números dizem (e o que não dizem)

1. **Fios (L1).** Óleo Bristle Flat: textura 5,9 % com CV 0,011 — texturado E uniforme. O nosso Impasto: 1,8 % / 0,007
   (liso); o nosso Wet Paint: 14,4 % mas os fios são carris fixos (ver prancha). O Express *Hairy* dele dá 9 % de
   textura **mas com período 6 px a 47 %** — um pincel por carimbo com cerdas desenhadas na forma TAMBÉM mostra o
   carimbo; a cura dele é o motor de fios, não a forma.
2. **Colar de contas (L2).** O nosso Impasto: 5,6 px a **42 %** da potência (o carimbo vê-se no perfil). Óleo dele:
   25,8 px a 9 %; aquarela dele (opaca): 7 px a 23 % (é o grão). O Digital nosso: 28 px a 13 % — o período aparece
   porque o falloff suave nosso soma em ondas longas.
3. **Orla.** O Rebelle entrega **1–2 px** de transição a 40 px de largura em óleo, aquarela, tinta, marcador; o nosso
   Digital **10 px** (falloff de fábrica suave: é o carimbo Blender, não uma lei de meio), o Impasto/Aquarela 3 px.
   ⇒ o preset de fábrica do Digital deve nascer com `hardness` alta; é o item mais barato de todos.
4. **Papel (L4).** Lápis 8B dele: textura 9,1 % e **borda/miolo 0,32** (o grão abre na orla, onde a pressão efectiva é
   menor, e fecha no miolo); pastel 7,2 % / 0,44. Nós não temos lápis — o Digital dá 0,7 % / 0,24 (a borda baixa é o
   falloff, não o papel).
5. **Inclinação (L6).** 8B a 55°: **37 px contra 30** direito, textura 5,2 % contra 9,1 % (o lado do grafite é mais
   largo e mais liso), e a rampa muda de forma (20/39/15 vs 20/36/20). Nós: o tilt só escala o raio.
6. **Aquarela (L7).** Flat 2 a opacidade 50 / água 30, seca: cobertura 0,43, **borda 1,04** (escurece ao secar), orla
   **1 px**. A nossa: cobertura 0,40, borda **1,23** (a orla escura nossa é mais forte — ok), orla 3 px. Round a água
   80: cobertura 0,37, borda 0,83, cruzamento vermelho-sobre-azul **arroxeado** (prancha, linha 4); a nossa aquarela
   K–M dá o vermelho sobre azul quase igual ao vermelho fora (94/54/68 vs 94/53/68: não mistura no cruzamento).
   ⚠️ **Refutado nesta bancada:** «a água alarga o traço» — a 80 de água a largura é 31 px nos três gestos; o que a
   água muda é a transparência e a mistura, não a largura (uma corrida anterior, com volumes não honrados no 2.º
   traço, tinha dado 85 px e enganou).
7. **Rampa de pressão.** Rebelle: 35/42/33 (óleo), 32/42/33 (aquarela), 28/41/28 (tinta) — o traço **nunca
   desaparece** nas pontas (pisos de 60–80 %: `pressure_size_curve (0,0,6…0,9)→(1,1)`); o nosso 12/40/8 e 17/51/13 vai
   a zero. Com mouse e pressão 1,0 o Rebelle ainda dá taper no fim (`41/41/0`) porque o `POINTER_RELEASE` leva pressão 0.
8. **Pincel sujo (L5) — NÃO medido.** O gesto de cruzamento é um flat de lado (4 px) e a amostra não distingue
   `oildirty` de `oil` (80/37/40 nas duas). A lei vem do manual e dos presets (136/281 em Paint & Mix); a bancada
   que a mede ainda não existe: um traço redondo a cruzar outro, e depois um terceiro traço fora, lendo a cor.
9. **Água como volume que corre, gotas, blow, tilt da tela** — não medidos (gestos horizontais, sem tilt de tela).

## §4 — O que medir a seguir (a bancada que falta, por onda do 01 §4)

| onda | fixture a acrescentar | régua |
|---|---|---|
| W2 smudge entre carimbos | o traço recto a 40 px com `splat_smudge` on/off (preset editado por `EDIT_BRUSH_PRESET`) | fracção do período |
| W3 pares por pressão | rampa 0→1 em óleo Round Hard | textura ao longo do traço em janelas de 50 px |
| W4 papel | o mesmo lápis em `HP01 Hot Pressed` vs `RH05 Rough` | textura e borda/miolo |
| W5 mistura/sujo | cruzamento com pincel REDONDO + terceiro traço fora | cor do miolo do 3.º traço |
| W6 fios | óleo Bristle Flat vs Bristle Round vs o nosso | textura · CV · período |
| W7 água | `SET_CANVAS_TILT`, `BLOW`, água 20/50/80 | largura ao longo do tempo (A vs B), gotas |

Tudo pela porta do [01 §6](01_o_que_o_rebelle_tem_e_nos_nao.md): mesmo papel, mesma semente, mesmos gestos.
