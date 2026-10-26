# 49 — O óleo do Rebelle 8 (RealShader · SoftShadows · NanoPixel 2): estado da arte, medição e especificação

> Pedido do dono (25–26/10): *«realmente realista e extraordinário — faz o nosso impasto parecer brincadeira
> de criança. Descubra os segredos a todo custo, compare com o que temos, escreva especificação e plano.»*
> Vídeos: `youtube.com/shorts/ajLV2fTJr0I` (SoftShadows II) e `youtube.com/shorts/CvP7KqQbN0Y` (Martin
> Surovček, sénior da Escape Motions, sobre NanoPixel 2 + RealShader). Plano de implementação:
> [`plano.md`](plano.md).
>
> ⛔ **Oráculo que se CORRE, nunca fonte que se lê** (CLAUDE.md §0.9): nada do binário, dos shaders ou dos
> mapas de reflexo do Rebelle foi lido ou copiado. Tudo abaixo é (a) o que a Escape Motions publicou e
> (b) o que o produto devolve a pinceladas nossas, fotografadas.

## 1. O que a Escape Motions diz (fontes públicas)

| Peça | O que é, nas palavras deles | Fonte |
|---|---|---|
| **RealShader** | «mapas de ambiente a partir de FOTOS reais» — iluminação baseada em imagem (IBL) como no 3D; *Environment 1* (reflete mais), *Environment 2* (brilho mais subtil), *Legacy* (= Rebelle 7 sem NanoPixel). Um dos mapas nasceu da foto de uma **esfera pintada de cinza**; o dev pintou pinceladas de óleo reais e fotografou-as na mesa dele — «por isso se vê a minha oficina reflectida no óleo». | blog Escape Motions; legendas do short `CvP7KqQbN0Y` |
| **SoftShadows** | «ray tracing em tempo real»: cada pincelada projecta sombra macia na tinta por baixo; «tons claros reflectem mais, escuros absorvem e fazem sombra mais funda». Direcção da luz **não ajustável** («not possible at the moment», staff nos comentários). | blog; Digital Production 11/07/2025 |
| **NanoPixel 2** | super-resolução por *deep learning* (rede treinada com **140 milhões de imagens**; o dev re-treinou porque apareciam costuras nas pinceladas), mapas de textura maiores e mais nítidos; exportação até **20×** (era 16×/4×), ≤ 32 000² px (TIFF sem tecto). Corre em OpenGL na placa. | legendas; blog R5/R8; 80.lv |
| Edição | RealShader, SoftShadows, NanoPixel = **Pro**. | blog |

## 2. O que o produto devolve — MEDIDO (26/10, Rebelle 8.3.4 Pro trial, NVIDIA real)

**Método.** Rebelle aberto no ecrã REAL (a placa é obrigatória: no `kwin --virtual` o render é por software
e o RealShader/SoftShadows **não aparece** — as fotos de 25/10 da tela virtual eram iguais ao nosso
impasto e foram descartadas pelo dono). Pinceladas-sonda por WebSocket (`ferramentas/tracos.py`:
espátula lisa horizontal e vertical, *Knife Textured*, *Flat Oily*, *Thick*, cúpula em espiral, em azul,
branco e azul-escuro), foto só da janela (`ferramentas/foto0.sh`). Os parâmetros que o WebSocket aceita
(`realshader`, `gloss`, `impasto_depth`) variam AO VIVO sobre a mesma pintura; os que só existem no
`settings.ini` (`raytracer_shadows`, `raytracer_lightAltitude`, `mat_environmentSphere`,
`mat_oilShadows`) exigem reabrir o app — a pintura é refeita com a mesma semente; a *Knife Textured*
sai igual entre aberturas (M11), a cúpula em espiral não — por isso as réguas usam o crop da *Knife
Textured* e o perfil da espátula vertical.

Mapa UI → `settings.ini` (lido no ficheiro do dono depois de ele mexer nos sliders):
`SoftShadows 10` → `raytracer_shadows=10.0`; `Shadow Altitude 10` → `raytracer_lightAltitude=42°`
(o valor de fábrica do ficheiro era `84°` com sombra `7.5`); `Gloss 10` → `mat_oilShininess=1.33`;
`mat_oilShadows=0.33`; `Environment 2` → `mat_environmentSphere`.

| # | Medição (crop da *Knife Textured* e perfil à direita da espátula vertical) | Resultado | Leitura |
|---|---|---|---|
| M1 | Fenda (2 % mais escuros) vs mediana, RealShader ON (`A_base`) | fenda `(0,43,113)` · mediana `(82,181,240)` · razão por canal `R 0 · G 0.24 · B 0.47` | o escuro **não é cinza**: satura no tom da tinta |
| M2 | O mesmo em linear: `dark = mid · s · albedoᵏ` | `k ≈ 2,1`, `s ≈ 0,25` | a fenda é a tinta **multiplicada por si mesma ~2×** — luz que ressalta dentro da cavidade (inter-reflexão) e uma oclusão de ~75 % |
| M3 | RealShader OFF (`A_rs0`) | fenda `(2,114,189)`, razão `G 0.62 · B 0.79` | o modo antigo (≈ o nosso): fendas rasas e lavadas |
| M4 | Gloss 0 / 5 / 10 | Gloss 0 mantém fendas e volume, perde só os brancos; 10 = brancos duros, alongados no sentido das cristas | o brilho é um termo **somado**, separado do volume |
| M5 | Impasto depth 2 / 5 / 10 | fenda `(3,120,195)` → `(0,43,113)` → `(0,31,101)` | a profundidade escala a altura; a lei da cor (M2) é a mesma |
| M6 | Sombra no papel à direita da espátula, passos de 5 px de ecrã (1 px tela = 1,167 px ecrã) | sombra 10: `39 · 14 · 5 · 1 · 0`; sombra 3: `12 · 4 · 2 · 0`; OFF: `0` | sombra **macia de decaimento ~exponencial (~4 px de tela por e-fold)**, intensidade **linear** no slider; cai NO PAPEL, não só na tinta |
| M7 | SoftShadows OFF (`B_base`) | fendas quase iguais (`(0,50,121)`); mediana mais clara `(98,197,249)`; brilhos >250: 3,7 % → 9,1 % | **o escuro das fendas vem do SHADER (M2), não da sombra**; a sombra escurece o meio-tom e apaga brilhos de cristas à sombra |
| M8 | Environment 1 / Legacy vs 2 | 1 e Legacy: superfície mais lisa e vidrada, fendas mais claras (`(0,63,130)`, `(0,70,138)`); 2: micro-relevo nítido | o ambiente muda a **leitura do micro-relevo** — o 2 é o que o dono fotografou |
| M9 | Bordas da espátula vertical (`A_borda`) | lado esquerdo aceso, lado direito escuro + sombra no papel à direita; fio de luz no topo da crista | luz de cima-esquerda; a crista tem *rim light* do ambiente |
| M10 | Luz `84°` (`C_base`) vs `42°` (`A_base`) | `84°`: sombra no papel `12 · 0` (1 passo); fenda `(0,70,141)`, mediana `(94,191,245)` · `42°`: `39 · 14 · 5 · 1` | luz mais baixa ⇒ sombra mais LONGA e fendas mais fundas: é obstrução ao longo da direcção da luz sobre o campo de altura (marcha de raio), não um desfoque fixo. `15°`: **não existe no Rebelle** — o mínimo é 42° e 15° derruba o app (W0(b), §2b) |
| M11 | `mat_oilShadows=0` (`H_base`) | estatística da *Knife Textured* idêntica à de `A_base` ao dígito | sem efeito visível com SoftShadows ligado (o slider da UI manda) |

Capturas: [`capturas/`](capturas/) (`A_*` = mesma pintura com parâmetros ao vivo; `B..G` = reaberturas
com `settings.ini` diferente; `*_textura.png`/`*_borda.png` = crops ampliados).

## 2b. W0 — medir antes de limitar (26/10)

### W0(a) — custo do passe na placa

Sonda `crates/ph2d-render/examples/mede_oleo_real.rs` (shader de protótipo inline, fiel em CUSTO, não em
aspecto): Sobel 3×3 + AO por horizonte (`N` direcções × `K` passos LINEARES de 1 px, `textureLoad` do
texel mais próximo) + sombra macia (`K` passos na direcção da luz, penumbra `min(c·Δh/t)`); `N`, `K` e o
modo são `override` do WGSL, as 16 variantes no MESMO processo, aquecidas, 7 rondas intercaladas de
blocos de 10 dispatches, relógio = timestamps da placa. Campo: 5 cristas até 14 px + ruído, 0–20 px.

```
bash scripts/ph2d-run.sh env PH2D_GPU=1 __GL_SHADER_DISK_CACHE=0 cargo run --profile smoke -p ph2d-render --example mede_oleo_real
bash scripts/ph2d-run.sh env VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json \
  __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json __GLX_VENDOR_LIBRARY_NAME=mesa \
  cargo run --profile smoke -p ph2d-render --example mede_oleo_real
```

ms de placa por dispatch, **mínimo (mediana)**, tela inteira. RTX = `NVIDIA GeForce RTX 5060 Ti` (Vulkan,
615.78.08) · Radeon = `RADV RAPHAEL_MENDOCINO` (iGPU do Ryzen 9 9950X, Mesa 26.2.5). loadavg 4,8→4,3
(Radeon), ~6,0 (RTX; mínimo = mediana, a carga não mexeu no número).

| variante | RTX 1080p | RTX 4K | Radeon 1080p | Radeon 4K |
|---|---|---|---|---|
| só cópia (mesmas bindings) | 0,025 (0,025) | 0,112 (0,113) | 0,255 (0,257) | 1,032 (1,039) |
| `ImpastoLightPass` de hoje (4 lâmpadas) | 0,123 (0,123) | 0,462 (0,463) | 3,029 (3,040) | 12,211 (12,273) |
| K=8 N=4 | 0,217 (0,217) | 0,841 (0,880) | 5,716 (5,729) | 22,736 (22,780) |
| K=8 N=8 | 0,365 (0,395) | 1,427 (1,475) | 9,693 (9,698) | 38,702 (38,716) |
| K=16 N=4 | 0,406 (0,424) | 1,635 (1,645) | 10,670 (10,675) | 42,727 (42,782) |
| K=16 N=8 | 0,830 (0,874) | 3,384 (3,389) | 18,411 (18,419) | 74,032 (74,115) |
| K=32 N=4 | 0,901 (0,927) | 3,671 (3,690) | 21,081 (21,104) | 82,962 (83,025) |
| K=32 N=8 | 1,653 (1,657) | 6,585 (6,614) | 36,539 (36,559) | 144,886 (144,908) |

**Leitura.** O custo é linear em `K·(N+1)` amostras e 4K ≈ 4× 1080p. Na RTX o caso mais caro (K=32 N=8,
4K, tela INTEIRA) cabe em 6,6 ms — **o tecto de `K`/`N` não é da RTX**, é da Radeon (20–25× mais lenta:
K=8 N=4 a 1080p já são 5,7 ms). ⇒ `K`, `N` não são constantes: são knobs de qualidade cujo tecto se
escolhe por placa (recurso = ms de placa por quadro, só na região suja). Limite inferior: amostragem
nearest; marcha bilinear/sub-pixel (se W3/W4 a pedirem) mede-se de novo.

### W0(c) — escala de altura do Wet Paint

Sonda `crates/ph2d-tool-painter/src/tool/paint/wetpaint_preset/medida_oleo_real.rs` (dois `#[ignore]` que
imprimem): `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_oleo_real_altura -- --ignored --nocapture`.
As 8 pinceladas de `ferramentas/tracos.py` (1200×800, pressão 1, passo 3 px), 1 px de tela = 1 célula.

⛔ **O Wet Paint não tem espátula.** Só 3 presets de fábrica têm relevo (todos com altura de fios 0,75):
`Oil/Dabber/Dabber` (emboss 0,5), `Oil/Flat/Filbert` (0,4), `Oil/Bristle/Oil Bristle` (0,6). As facas
foram pintadas com o Dabber. O Rebelle tem 10 (`Oil/Knife`: Smooth, Smooth 2, Smooth 3, Textured, Oily,
Dry, Dry 2, Grainy, Soft, Stain) ⇒ **as espátulas do Wet Paint entram nesta linha** (plano, W0b).

| pincelada (preset) | massa `sett+susp` p50/p90/p99/max | `altura` p50/p90/p99/max (px) | \|emboss\| p50/p99/max | tecto ±40 morde |
|---|---|---|---|---|
| faca H azul (Dabber) | 2589/3853/4642/6004 | 0,16/0,63/0,84/1,10 | 3,6/16,0/27,8 | 0 % |
| faca V azul (Dabber) | 2591/3804/4575/6026 | 0,15/0,59/0,82/1,07 | 2,3/10,9/19,6 | 0 % |
| oily azul (Filbert) | 2106/2901/3187/4518 | 0,32/0,88/1,12/1,22 | 1,4/8,4/12,8 | 0 % |
| thick azul (Oil Bristle) | 3572/5575/7178/9226 | 0,15/0,84/1,09/1,23 | 5,1/17,8/24,4 | 0 % |
| faca H branca / escura | ≈ a azul | ≈ a azul | ≈ 3,7/16,5/28,9 | 0 % |

Perfil à direita da faca vertical (Dabber, y=240): a massa cai de ~4 000 a 0 em **~15 px sem degrau**; a
`altura` dos fios é ASSINADA e pequena (±1,3 px) e passa ~6 px além da tinta. **Leitura:** hoje não há
borda de espessura (o «degrau» que faz a sombra do Rebelle, M6/M9); a massa e os fios contribuem ~igual
para o emboss, e o tecto ±40 nunca é atingido — o «teto de contraste» de §3 item 4 não é o tecto, é o
ganho (`0,008`, `LUZ=20`).

### W0(b) — sombra × altitude da luz no Rebelle

Sessões `ferramentas/sessao.sh L<alt> raytracer_lightAltitude=<alt> raytracer_shadows=10` (ecrã real, RTX,
autorização do dono 26/10), régua `ferramentas/regua_sombra.py` (escurecimento do papel à direita da
espátula vertical, mediana de ~350 linhas, contra o papel a +100..+120 px; passos de 5 px de ECRÃ =
4,3 px de tela). Capturas recortadas na folha: `capturas/L25|L35|L42|L60_base.jpg`.

| altitude (`settings.ini`) | sombra no papel por passo de 5 px de ecrã | leitura |
|---|---|---|
| SoftShadows OFF (`B_base`) | `5,2 · 0` | o fio de borda (antialias), não é sombra — é o zero da régua |
| 84° (`C_base`) | `19,6 · 1,8 · 0` | |
| 60° (`L60`) | `33,3 · 7,6 · 0,8 · 0` | |
| 42° (`L42`; `A_base` dá `85,5 · 20,8 · 7,6 · 2,1`) | `50,5 · 16,8 · 6,4 · 1,4 · 0` | |
| 35° (`L35`) · 25° (`L25`) | **idêntico ao 42° à décima** | o Rebelle **trava em 42°** |
| 15° (`L15`) | — | o Rebelle **morre** na 1.ª pincelada (4/4 corridas, log pára em `SET_RANDOM_SEED`) |

⛔ **42° é o MÍNIMO do Rebelle**: o slider *Shadow Altitude* 10 (o que o dono usou) já é o fundo da
escala; abaixo o valor é ignorado (25° e 35° saem iguais ao 42° na espátula; a tela só difere na
cúpula, que não se reproduz entre aberturas — M11), e 15° derruba o app. **M10 a 15° não existe no
oráculo**: abaixo de 42° a lei é NOSSA (física: comprimento ∝ `cot(alt)`), sem régua do Rebelle — vantagem
declarada, como a direcção.

**A espessura que a sombra implica.** O comprimento visível (até ~1 nível de cinza) é ~20 px de ecrã
(≈17 px de tela) a 42° e ~12 px de ecrã (≈10 px de tela) a 60°; `h ≈ L·tan(alt)` dá **≈ 15–17 px de
tela** nas duas, com `impasto_depth 8`. O decaimento a 42° é de ~4,4 px de tela por e-fold no 1.º passo e
~2,8 no 2.º (mais duro na ponta: é o fim do degrau, não um desfoque). O nosso campo de hoje: a massa da
faca (Dabber) tem p50 ≈ 2 600 e os fios ±1,3 px (W0(c)) ⇒ **ponto de partida da escala: ~0,006 px de
relevo por unidade de massa a «profundidade 8»**, a ajustar pela régua M6 em W4 — não é constante final.

### W0(b′) — a folha das 10 espátulas (`F42`)

`ferramentas/tracos_facas.py` (uma pincelada horizontal de 450 px por espátula, tamanho 90, azul
`(20,150,230)`, pressão 1, luz 42°, sombra 10) e régua `ferramentas/regua_facas.py capturas/F42_base.jpg`.
Coluna da esquerda: Smooth, Smooth 2, Smooth 3, Textured, Oily; direita: Dry, Dry 2, Grainy, Soft, Stain.

| espátula | largura (px tela) | cobertura % | fenda RGB | mediana RGB | razão fenda/mediana G, B | sombra por baixo /5 px de ecrã |
|---|---|---|---|---|---|---|
| Knife Smooth | 19 | 99 | (1,51,145) | (29,154,233) | 0,34 · 0,63 | 36 · 9,0 · 1,2 · 0 |
| Knife Smooth 2 | 91 | 100 | (1,39,115) | (27,154,233) | 0,25 · 0,49 | 81 · 31 · 15 · 7,2 · 3,2 · 0,6 |
| Knife Smooth 3 | 88 | 99 | (3,33,106) | (41,159,233) | 0,21 · 0,46 | 126 · 75 · 43 · 27 · 18 · 12 (orla clara por baixo, não só sombra) |
| Knife Textured | 87 | 99 | (13,119,187) | (90,184,240) | 0,65 · 0,78 | 6,2 · 0 |
| Knife Oily | 5 | 92 | (6,84,139) | (89,183,240) | 0,46 · 0,58 | 17,9 · 1,1 · 0 |
| Knife Dry | 33 | 94 | (1,101,182) | (26,149,222) | 0,68 · 0,82 | 18,7 · 11 · 7,2 · 4,6 · 2,4 · 0,6 |
| Knife Dry 2 | 32 | 88 | (22,147,224) | (66,176,242) | 0,84 · 0,93 | 0,8 · 0 |
| Knife Grainy | 87 | 86 | (2,31,108) | (68,160,220) | 0,20 · 0,49 | 73 · 32 · 17 · 10 · 6,0 · 3,6 |
| Knife Soft | 15 | 99 | (1,63,143) | (24,150,227) | 0,42 · 0,63 | 26 · 11,4 · 4,4 · 1,2 · 0 |
| Knife Stain | 85 | 48 | (2,16,94) | (34,129,198) | 0,13 · 0,48 | 59 · 24 · 12,4 · 6,4 · 3,2 · 1,4 |

**Leitura (para W0b).** No mesmo tamanho 90, a largura vai de **5 px (Oily) a 91 px (Smooth 2)**: o
tamanho não é a largura da lâmina, cada preset tem a sua. A espessura também varia: Smooth 2/3, Grainy
e Stain fazem sombra longa (tinta alta) e fenda funda (verde `0,13–0,25`, a lei M1/M2); Textured, Dry e
Dry 2 quase sem sombra e com fenda rasa (`0,65–0,84`). ⚠️ A *Knife Textured* a tamanho 90 sai RASA
(`0,65`), ao contrário da mesma faca a 120 nas capturas `A_*` (`0,24`, M1): a altura da tinta depende do
tamanho — medir em W0b antes de copiar a forma (máscara da régua inclui a fenda escura; corrigida 26/10).

## 3. O que o PH2D tem hoje

| | Impasto (`ph2d-render/src/shaders/impasto_light.wgsl`) | **Wet Paint** (`ph2d-wet-paint/src/render.rs:385-450`) — o único modo que evolui (dono 07/10) |
|---|---|---|
| Altura | plano `relief` por pixel | massa `sett+susp` + `altura` dos fios (`bristles_relevo`) |
| Normal | diferença central × `DEPTH_UNIT_PX=16` | gradiente da massa, vertical pesado 2× |
| Luz | Blinn-Phong RELATIVO, até 4 lâmpadas, `AMBIENT=0.35`, razão ≤ 2 | **emboss cinza SOMADO** à cor, `±40/255`, luz fixa topo-esquerda (`LIGHT_X=LIGHT_Y=-1/√2`) |
| Sombra projectada | ✗ (doc 27 W1, plano nunca construído) | ✗ |
| Oclusão / fenda | ✗ (doc 27 W2) | ✗ |
| Ambiente (IBL) | ✗ | ✗ |
| Cor da fenda | `albedo · ≥0.35` (cinza-lavado) | `albedo − 40` (acinzenta) |
| Espaço de cor | sRGB | sRGB |

**Porque o nosso «parece brincadeira» — a lista, por peso:**
1. **Fenda cinza em vez de fenda saturada** (M1–M2): somar/multiplicar um cinza nunca dá `albedo²`.
2. **Sem oclusão nem sombra** (M6–M7): o relevo lê-se como inclinação, nunca como obstrução.
3. **Brilho de lâmpada pontual** (ponto) em vez de **reflexo de ambiente** (faixas que seguem a crista, M4/M9).
4. **Teto de contraste**: o Wet Paint satura em `±40`; o Impasto em `0.35..2×`.
5. **Tudo em sRGB**: a luz multiplica valores gama — os escuros ficam lamacentos.

## 4. Especificação — o «Óleo Real» do PH2D

Um passe de **óptica** novo, na placa, sobre o campo de altura que JÁ existe (o fold continua na CPU —
doc 27 §3.1), servindo o **Wet Paint** primeiro e o Impasto pela mesma porta. Ordem do pixel:

1. **Linear.** Albedo `sRGB→linear` à entrada, `linear→sRGB` à saída.
2. **Normal** por Sobel 3×3 sobre `h` em unidades de pixel (o ganho é o slider *Profundidade*, = M5).
3. **Reflexo de ambiente (RealShader).** Mapa de ambiente equiretangular HDR **nosso** (CC0, p.ex. Poly
   Haven; ou foto do dono de uma bola cinza pintada — o mesmo método deles), pré-filtrado offline em:
   (a) irradiância difusa (SH de 9 coeficientes) e (b) *mip* especular por rugosidade (split-sum + LUT BRDF).
   Dieléctrico: Fresnel Schlick `F0=0.04` (óleo/ligante). Vista ortográfica `(0,0,1)`. **Rotação do
   ambiente = direcção da luz** (o Rebelle não roda; nós rodamos — vantagem).
4. **Oclusão de cavidade (AO por horizonte)**, `N` direcções × `k` passos (a medir em W0), e a **lei da cor
   da fenda** medida em M2: `difuso = albedo · E_irr · lerp(1, s·albedoᵏ, ao)` com `k≈2`, `s≈0,25` como
   ponto de partida a ajustar contra as capturas.
5. **Sombra macia (SoftShadows)**: marcha na direcção da luz dominante do ambiente (altitude + azimute),
   penumbra por cone (`min(k·Δh/t)` estilo SDF) — mede-se o decaimento contra M6 (~4 px por e-fold a 42°,
   altura da espátula). **Cai no papel** (excepção explícita à lei «papel nu não recebe nada»: só o termo
   sombra; o difuso continua mudo).
6. **Brilho**: `Gloss` escala só o termo especular (M4), nunca o volume.
7. **Neutro byte-idêntico**: *Óleo Real* desligado ⇒ o render de hoje, ao byte (gate).

Controlos (cartão do Wet Paint, i18n, tokens): **Óleo Real** (liga) · **Ambiente** (Estúdio · Janela ·
Clássico) · **Profundidade** · **Brilho** · **Sombra** · **Altura da luz** · **Direcção da luz** (nosso a mais).

## 5. NanoPixel 2 — o que se faz em vez da rede de 140 M imagens

O deles é super-resolução treinada. O nosso caminho candidato é **determinístico**: o Wet Paint conhece a
pincelada (o motor é uma simulação por células) — exportar a N× = **re-simular/re-amostrar o campo de
altura em resolução maior** e só depois iluminar; a luz (§4) é independente da resolução. ⚠️ Não medido:
depende de o motor guardar o traço reproduzível em escala (ver `plano.md` W5, abre com medição, cancela
se a reprodução não fechar). Detalhe de textura fina: um *detail normal* tirado de FOTOS de tinta real
(como eles fizeram), nunca dos mapas deles.

## 6. ⛔ Recusas

| Recusa | Porquê |
|---|---|
| Copiar `Reflection Maps/` ou qualquer textura do instalador | propriedade da Escape Motions; EULA de trial pessoal |
| Treinar uma rede de super-resolução agora | 140 M imagens e anos; o caminho determinístico (§5) mede-se primeiro |
| Fotos de oráculo tiradas na tela virtual | render por software: o RealShader não corre lá (medido 25/10, o dono rejeitou) |
| Reabrir o Rebelle para comparar pixel a pixel | a mesma semente não reproduz o traço entre aberturas (cúpula diferente) — compara-se estatística |
