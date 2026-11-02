# HANDOFF DE CONTINUAÇÃO — Pixel Art P7a (pincéis e dinâmica: oráculo + núcleo) — 02/11

> Linha `line/PixelArt`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt` (`cd` + `pwd` +
> `git branch --show-current` antes de ler). Plano: [P7_plano.md](../P7_plano.md) (o §2 é o desenho; o §5 as rodadas).
> Anterior: [reabertura 02/11](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-02_REABERTURA.md).
> **Dono 02/11:** os pincéis guardados ficam NO COMPUTADOR (`~/.ph2d/pixel_brushes/`), memória
> `project_pixel_brushes_live_on_the_computer_2026_11_02.md`. Nada no `PROJECT_SCHEMA`.

## 1. Commits desta jornada (mais antigo primeiro)

`48106f005a` reabertura (pulldown-cmark 0.13.6) · `c4cfc838a5` plano P7 + decisão do dono · `cbe9a4052f` rodada 1 do
oráculo · `bf9d2b4769` **PatternBrush no núcleo** · `ed0a90ebbc` **dither Bayer** · `f148711f5d` **crate `ph2d-pen-law`** (a
corda) · `327d4648f8` rodada 2 do pincel de padrão · (+ o commit da rodada 2 da dinâmica, §4).

## 2. O que JÁ existe (não reconstruir)

- `ph2d-pixel-core/src/pattern.rs`: `PatternBrush::from_selection(sprite, key, &Mask)`, `anchor() = (⌊w/2⌋, ⌊h/2⌋)` do
  RECTÂNGULO da selecção (nada se apara), `offsets()` = células da máscara, `value(align, k, at, start)`;
  `PatternAlign { Source (omissão), Destination, Paint }`.
- `Brush` (dono único, `brush.rs`) deixou de ser `Copy`: `pattern: Option<Arc<PatternBrush>>`, `pattern_align`,
  `image_color` (a borracha põe `false` = apaga com a FORMA). `Paint` também deixou de ser `Copy`.
- `Stroke` (`stroke.rs`) guarda o valor por célula (`src`, o último carimbo cobre) e o `start` (o Destination).
- `PixelOptions.pattern_align: PixelPatternAlign` (editor-core) → `gesture::paint(o, doc, pincel)`; `PixelApp.pincel`.
  **Ainda ninguém o ESCREVE** (falta o Ctrl+B e o seletor: §5.1) — não é órfão por desenho, é o próximo passo.
- `ph2d-pixel-core/src/dither.rs`: `Dither { None, Bayer2, Bayer4, Bayer8 }`, `level(x, y)` preso ao canvas, `reaches(x, y, t)`.
- Crate NOVA `ph2d-pen-law` (sem deps): `pulled_string_step` (a corda). `ph2d-painter-brush::pen` re-exporta-a.
- Gates: `ph2d-pixel-core` `brush_pattern` (13 casos da rodada 1, grelha 64×48 inteira + a âncora sem apara) e
  `dither_laws` (a ordem medida + permutação). Mutações vistas vermelhas: âncora `(w−1)/2` (2 ✗), Bayer transposta (✗).
- Ferramentas: `docs/PixelArt/ferramentas/trial_pinceis.py`, `trial_dinamica.py`, `pixelorama_pressao/` (extensão MIT que
  injecta pressão; `corre.sh`). Fotos `docs/PixelArt/aseprite/50_..69_*`.

## 3. Leis MEDIDAS do pincel de padrão (Trial 1.3.18.6, `oracle_trial_brush_pattern.json`, 31 casos)

- Âncora e alinhamentos: ver §2 (gate verde). Transparente nunca pinta; selecção elíptica = só o interior.
- Ctrl+B **sem** selecção = modo «Select brush bounds | Right-click to cut» (arrastar define o pincel).
- Depois do Ctrl+B a selecção NÃO recorta o traço (carimbos fora dela pintaram) ⇒ o Ctrl+B tem de largar a selecção.
- Cada Ctrl+B cria um ESPAÇO novo (Alt+1, Alt+2…), destrancado, só com **Brush Type + Image Color** ligados.
  «Save Brush Here» cria um espaço TRANCADO com Type/Size/Angle/Image Color/Ink Type/Opacity/Shade (Foreground,
  Background, Pixel-Perfect desligados). Parâmetro desligado = fica o valor actual (provado por clique: Size, Angle).
- O modo do padrão por omissão é **Source**; a escolha persiste entre Ctrl+B.
- **Escolher uma cor** depois do Ctrl+B: o padrão passa a pintar na cor de FRENTE — EXCEPTO o pixel (0,0) do pincel, que
  mantém a sua (provado com GRB/RCR → GBB/BBB). ⚠️ Parece defeito do Aseprite: **pergunta ao dono** antes de o copiar
  (recomendação: tudo na cor de frente). Recuperar o espaço (Alt+n) volta às cores da imagem.
- **Não existe flip/rotate do pincel no Trial** (menus Edit/Select e barra, fotos 65/68/69). O plano P7 pedia-o
  (Pixelorama tem) — **pergunta ao dono** se o queremos mesmo assim.
- Tinta: Alpha Compositing a 50 % mistura ~0,5 com o destino; Lock Alpha só pinta sobre opaco. Buraco não pinta.
- Por medir: quadrado 5 px a 30° guardado carimbou 3×3; 50 % = 127 ou 128; (0,0) transparente + cor de frente.

## 4. Dinâmica (`oracle_trial_dynamics_velocity.json`)

- Popup (foto 55): Stabilizer (caixa + valor; 16 ao ligar) · grelha Size/Angle/Gradient × Pressure/Velocity (uma
  coluna por linha) · Min/Max por linha (Size Max segue o tamanho do pincel) · Gradient = dropdown No Dithering /
  Bayer 2×2 / 4×4 / 8×8, legenda «BG > FG» · Sensor Threshold = uma barra de DUAS pegas por sensor activo ·
  «Same in all Tools».
- **Velocidade (rodada 1):** o tamanho NÃO segue a velocidade instantânea — cresce aos degraus ao longo do arrasto e
  não satura em 142 células; px de ECRÃ (o zoom não ajuda); jitter ±1. Gradiente: só as duas cores, Bayer preso ao
  canvas, densidade cresce com o arrasto. **Stabilizer 16 = corda de ~120–128 px de ecrã** (independente do zoom e do
  pincel; o traço acaba ATRÁS do cursor).
- **Rodada 2 — a LEI da velocidade** (ajuste sobre as colunas lidas, erro médio 0,12 de tamanho; um α fixo por evento
  dá 0,63, a lei por tempo ganha): `passo` = deslocamento em px de ECRÃ desde o evento anterior (NÃO se divide por dt);
  `v_f += (1 − exp(−dt/τ))·(passo − v_f)` a cada evento, `τ ≈ 50 ms`, `v_f = 0` ao premir; `tamanho = floor(Min +
  (Max − Min)·clamp((v_f − lo)/(hi − lo)))`. As pegas do limiar são `lo`/`hi`: omissão 0,10/0,89 da barra ≈ 6/60 px por
  evento (≈ 66 × fracção, de 3 posições). Rampa medida (eventos a 8 ms): passo 8→1, 12→2, 16→3, 20→4, 24→6, 28→7, 32→8,
  40→10, 48→12, 56→14, 64→16. Passo 16 dá 3 com eventos de 2 a 128 ms (125–8 000 px/s): a velocidade em px/s NÃO conta.
  Parado o filtro não corre (o 1.º evento depois da pausa vale quase sozinho); de passo 64 para 8 volta a 1 em ~90 ms;
  **inverter o sentido recomeça em 1** (o filtro age no VECTOR — não isolado). O xdotool encadeado da rodada 1 enchia a
  fila do Trial: era isso o «cresce ao longo do arrasto». Estabilizador: **k ≈ 8 px de ecrã por unidade** (4→3, 7→6,
  32→29 células de atraso da borda a 400 %; 16→128 px). Fixtura: `round2_size_velocity_rows`, `round2_law_candidate`,
  `stabilizer_round2`; script `trial_dinamica.py` modos `r2rows`/`r2` (XTest por ctypes; a docstring do topo ainda não os
  descreve). **Não medidos:** Angle ← Velocity, o Gradient até saturar, px de ecrã × px de interface (screen_scale 2),
  a escala exacta da barra do limiar, o filtro sobre o vector.
- Pressão: o Trial não a recebe. O Pixelorama 1.2.3 (MIT) mede-a por extensão: rampa LINEAR entre limiares
  (`remap(p, min, max) → lerp(tamanho, tamanho+incremento)`, arredonda; alpha linear). A prova final é a Wacom do dono.

## 5. Próximo (na ordem do plano §8)

1. **App:** Ctrl+B (`input_teclas.rs`, com o Pixel a ter o teclado; `B` sozinho continua o lápis) → `PatternBrush::from_selection`
   na camada activa → `app.pincel`, largar a selecção; sem selecção = modo de recorte. Acrescentar ao gate `brush_pattern`
   os casos `rb6_*` da rodada 2.
2. **Núcleo da dinâmica** (`dynamics.rs`): `Channel { sensor, threshold, min, max }`, `Channel::map` (UMA rampa para
   pressão e velocidade), o filtro da velocidade pela lei da rodada 2, `Sample { pos, pressure, t_ns }` do
   `ph2d_pen::porta::{tinta, evento_ns}` lido no `View::of` (sem crescer a shell), a corda via `ph2d-pen-law` em px de ecrã.
   O `Stroke` precisa de carimbo de tamanho/ângulo/valor POR PONTO (hoje `offs` é fixo por traço).
3. **Lista de pincéis:** `ph2d-pixel-io::brush_file` + `~/.ph2d/pixel_brushes/` (padrão de `PixelApp.paletas`).
4. **UI:** menu do tipo de pincel (espaços + «Save Brush Here» + parâmetros com cadeado), seletor do padrão, popup
   Dynamics com a barra de duas pegas (widget NOVO, com o gate dele), seam tests.
5. Cena 18 (`max_level` 17 → 18), sonda headless, fotos, mutações, handoff de integração.

## 6. Decisões do dono (02/11, depois das rodadas)

1. **Escolher uma cor depois do Ctrl+B pinta o pincel INTEIRO na cor escolhida** — o pixel (0,0) do Aseprite NÃO se copia
   (defeito dele). Recuperar o espaço guardado volta às cores da imagem (`image_color`).
2. **O PH2D TEM virar (H/V) e girar 90° o pincel de padrão** (o Aseprite não tem; o Pixelorama tem): botões no menu do
   tipo de pincel; a lei é a das transformações do núcleo (`transform.rs`) sobre o `PatternBrush` (valores, máscara,
   `origin` do Source ajustado), nunca uma cópia.
