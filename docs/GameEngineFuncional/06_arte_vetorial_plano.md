# A arte do jogo de plataforma — vetor plano sobre o Kenney Pixel Platformer (plano, 25/10)

> Decisões do dono (25/10, depois da N1): o jogo do smoke ganha arte de verdade.
> 1. **Referência de conteúdo:** [Kenney Pixel Platformer](https://kenney.nl/assets/pixel-platformer) — CC0 1.0
>    (200 ficheiros, azulejo 18×18; a imagem do próprio pacote diz «CC0 1.0 … commercial projects»).
>    Recusados antes: Sunny Land (o dono preferiu o Kenney), Fantasy Warrior, Pixel Adventure, etc.
> 2. **Estilo:** vetor PLANO como a ilustração de animais que o dono mostrou (formas geométricas
>    arredondadas, cor chapada, UM tom mais escuro para volume, olhos brancos com pupila, sem contorno).
>    É REDESENHO peça a peça (silhueta, proporção e paleta do Kenney), nunca decalque de píxeis.
> 3. **Animação:** *«alta qualidade, embora muitíssimo simples»*. O dono autorizou uma **2.ª referência em
>    Blender 3D** para não errar os quadros: um boneco simples com rig, os ciclos (parado, correr, saltar,
>    cair) animados e renderizados de lado em ortográfica, quadro a quadro, como guia de pose e de tempo
>    para os quadros 2D.
> 4. **Começa por uma AMOSTRA** que o dono aprova antes do resto: o boneco verde de capacete nas 4
>    poses + blocos de chão com relva + moeda + bandeira, a correr no `PH2D_GAME_SMOKE=2`.

## O que já existe (medido)

| Peça | Onde |
|---|---|
| SVG → documento vetorial do PH2D (lei dos eixos Y↓/Y↑) | `crates/ph2d-vec-svg` (usvg 0.48) |
| SVG → píxeis (rasterizar) | `crates/ph2d-imageio-svg` (resvg 0.48) |
| Folha de animação a tocar por estado | N1: `Animated Sprite`/`Character` (`SpriteGrid` + `SpriteAnimations` + `CharacterAnimations`), boneco de fábrica `HERO_SHEET_KEY` (procedimental, 4×6, 64 px) |
| Blender | `/usr/bin/blender` — corre na placa: SEMPRE `PH2D_GPU=1 bash scripts/ph2d-run.sh blender -b …` (o guarda recusa o resto) |

## O caminho proposto (a confirmar ao medir)

1. **Triagem de licença por artefacto** (CLAUDE.md §0.9): descarregar o zip para pasta própria, ler o
   `License.txt`; Blender é GPL (corre-se como ferramenta; o que ele renderiza sobre entradas nossas é nosso).
2. **Referência Blender** (script Python versionado em `docs/GameEngineFuncional/ferramentas/`, corrido por
   `blender -b -P`): rig mínimo (corpo, cabeça/capacete, 2 braços, 2 pernas), ciclos com tempo de animação
   clássico (antecipação no salto, *squash & stretch*, contacto/passagem no correr), render ortográfico de
   lado por quadro → PNG de referência com cabeçalho. A contagem de quadros e o tempo saem DAQUI.
3. **Desenho vetorial** (SVG versionado, uma peça por ficheiro ou por pose): por cima da referência; cores do
   Kenney reinterpretadas no estilo plano.
4. **No jogo:** o SVG abre-se no PH2D como vetor (editável) e assa-se para folha (resolução alta, margem por
   célula) para o `Animated Sprite`; o boneco de fábrica pode passar a ser esta arte.
5. **Foto lado a lado** (Kenney · Blender · vetor · no jogo) de cada peça antes de mostrar ao dono.

## A amostra — feita (25/10), à espera do smoke do dono

Foto: [`arte/amostra_lado_a_lado.png`](arte/amostra_lado_a_lado.png) (Kenney · Blender · vetor · no jogo) e
[`arte/referencia_blender.png`](arte/referencia_blender.png) (os 22 quadros da referência).

| Passo | O que ficou | Medido |
|---|---|---|
| Licença | `License.txt` do zip `kenney_pixel-platformer.zip` (1.2, sha256 `d01a196d…`): **CC0**, uso comercial livre, crédito opcional | — |
| Paleta | histograma dos PNG do pacote (`tile_0000` boneco, `0002` chão, `0151` moeda, `0111` bandeira) | 4 verdes + vidro + `#434A5F` |
| Blender | [`ferramentas/boneco_blender.py`](ferramentas/boneco_blender.py): hierarquia raiz→anca→tronco→cabeça, coxa→canela; chaves Bézier a 30 q/s; perna de APOIO por IK de 2 ossos com o pé plantado (desliza para trás no passo); render Workbench ortográfico + [`ferramentas/boneco_poses.json`](ferramentas/boneco_poses.json) (juntas por desenho, com cabeçalho) | sola do contacto a < 0,5 px do chão |
| Tempos | parado 6 × 167 ms · correr 8 × 67 ms (2 passos em 0,53 s) · saltar 100/133/167/133 ms **uma vez** (fica no ápice) · cair 4 × 100 ms | gate `os_tempos_sao_os_da_referencia_do_blender` |
| Vetor | [`ferramentas/arte_svg.py`](ferramentas/arte_svg.py) lê as poses e escreve `crates/ph2d-render/src/atlas/fabrica/*.svg`: só `fill` em caminhos absolutos | gate `a_arte_de_fabrica_abre_como_vetor_sem_perdas` (1 forma por `<path>`, zero notas) |
| No jogo | `FACTORY_ART` (ph2d-render): rasterizada 1× por processo (resvg) nas chaves `u32::MAX-2..-7`; o nível 2 veste os blocos (azulejo × `UvTransform` + `Repeat`), as moedas (gira, `moeda.luau`) e a bandeira (ondula); céu `#C2E3E8`; câmara a 8 m | gates `o_nivel_2_*` |

**Desvios conscientes do desenho do Kenney:** pernas de 0,14 → 0,20 da célula (com as do pixel a corrida não se
lê de lado); olhos brancos com pupila (o estilo do dono, o Kenney tem dois píxeis escuros); sem braços (o
Kenney também não tem). A cápsula de colisão subiu para o topo do capacete (`half_height` 0,07 → 0,15) e
ficou com o raio 0,2 — mais larga não paira numa rampa de 45° (`fitted_float`).

**Dois defeitos do «repetir textura» que a amostra mostrou e que ficaram curados** (valem para toda sprite com
`UvTransform` + `Repeat`/`Mirror`):
1. **Risco cinzento a cada azulejo** — onde o `fract` dá a volta, a derivada implícita salta um azulejo e a
   placa escolhe o mip mais pequeno. Cura: `sample_tiled` no `sprite.wgsl` amostra com o gradiente do UV
   contínuo; quem não repete continua no `textureSample` (byte a byte). Gate de placa
   `um_azulejo_repetido_nao_tem_costura` (vermelho sem a cura: 101 texels, alfa 0,54 em x = 16/32/48).
2. **Fio claro fino a cada azulejo** — o mip mistura a borda da região com o vizinho no atlas (transparente;
   a «gutter padding» que o `atlas/mod.rs` já nomeava). Cura na arte, não no atlas: um azulejo entra em 3
   cópias (3 × 3 a terra) e a sprite pinta a do MEIO (`FactoryArt::middle_frame`). Gate
   `um_azulejo_repete_o_mesmo_desenho_em_cada_celula`; a foto confirma.

**Blender:** o `/usr/bin/blender` do sistema (5.2.3) não arranca — `libosdGPU.so` (opensubdiv) pede
`libcudart.so.13` e o `cuda` não está instalado. A referência correu no Blender **portátil oficial** 5.2.3
(`download.blender.org`, sha256 `78486c01…` conferido) — o mesmo número de versão; instalar o `cuda` é decisão
do dono, não da linha.

## Aberto da N1 (continua)

- **Veredito do smoke N1 do dono — POR RECEBER** (menu Add com as 5 peças + dicas ilustradas; nível 2;
  Auto Animate no Inspector). Handoff de continuação da N1 só depois dele.
- R1 (vista dupla ao vivo), View ▸ Focus mode morto na UI nova, sem `ph2d.disconnect`, fecho da linha.
