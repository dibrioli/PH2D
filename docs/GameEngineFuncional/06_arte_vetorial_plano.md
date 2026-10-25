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

## Aberto da N1 (continua)

- **Veredito do smoke N1 do dono — POR RECEBER** (menu Add com as 5 peças + dicas ilustradas; nível 2;
  Auto Animate no Inspector). Handoff de continuação da N1 só depois dele.
- R1 (vista dupla ao vivo), View ▸ Focus mode morto na UI nova, sem `ph2d.disconnect`, fecho da linha.
