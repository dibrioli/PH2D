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
| M10 | Luz `84°` (`C_base`) vs `42°` (`A_base`) | `84°`: sombra no papel `12 · 0` (1 passo); fenda `(0,70,141)`, mediana `(94,191,245)` · `42°`: `39 · 14 · 5 · 1` | luz mais baixa ⇒ sombra mais LONGA e fendas mais fundas: é obstrução ao longo da direcção da luz sobre o campo de altura (marcha de raio), não um desfoque fixo. `15°` não capturado (a sessão falhou duas vezes) — fica para W0 |
| M11 | `mat_oilShadows=0` (`H_base`) | estatística da *Knife Textured* idêntica à de `A_base` ao dígito | sem efeito visível com SoftShadows ligado (o slider da UI manda) |

Capturas: [`capturas/`](capturas/) (`A_*` = mesma pintura com parâmetros ao vivo; `B..G` = reaberturas
com `settings.ini` diferente; `*_textura.png`/`*_borda.png` = crops ampliados).

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
