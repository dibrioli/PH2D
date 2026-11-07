# O smoke do dither

Cabeçalho de `shells/desktop/src/dither_smoke.rs`, movido para cá em 07/11 pela catraca da shell.

```text
`PH2D_DITHER_SMOKE` — **as faixas e a cura, na MESMA imagem**.

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
  && env PH2D_DITHER_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

**Uma** sprite, partida ao meio na horizontal. As duas metades saem do **mesmo** degradê de 16
bits, descido para 8 bits pelos dois caminhos:

| metade | porta | o que se vê |
|---|---|---|
| **cima** | `rgba16_to_rgba8` (fiel) | faixas verticais, com arestas duras |
| **baixo** | `rgba16_to_rgba8_dithered` (W6.1) | liso |

# Por que UMA sprite partida, e não duas lado a lado

⚠️ **A primeira versão eram duas sprites separadas, e o Enio não viu diferença nenhuma.** Duas
metades adjacentes valem muito mais que duas imagens vizinhas: em cada coluna, cima e baixo
partem do **mesmo** valor, e as arestas da metade de cima **param a meio da imagem**. O olho é
excelente a detectar uma descontinuidade que atravessa uma fronteira partilhada, e péssimo a
comparar dois quadrados separados por um vão — sobretudo quando a diferença é de **um** código,
que é o passo mais pequeno que existe.

# A causa MEDIDA de as duas primeiras versões saírem lisas

⚠️ **O filtro de textura do projeto é `Smooth` por omissão** (bilinear + anisotropia 16×,
[`ph2d_host::ImageFilterMode`]). Em **ampliação**, o bilinear interpola entre texels vizinhos — e
um degrau de um código transforma-se numa rampa suave. *Aproximar o zoom não mostrava o defeito:
apagava-o.* A instrução «dê zoom até ver os pixels» estava exactamente ao contrário.

A cura é a sprite trazer o seu próprio [`ph2d_ecs::TextureFilter`] em `Nearest`: assim o que
aparece no ecrã são os bytes, a qualquer zoom, e a cena deixa de depender de uma preferência de
projeto que o artista pode ter mudado.

# O que se vê a cada zoom, e as duas leituras são diferentes

- **Zoom normal (a cena abre assim):** em cima, faixas verticais largas; em baixo, liso. É a
  leitura de **produto** — é isto que o defeito é.
- **Zoom fundo:** em baixo aparece um xadrez fino. ⚠️ **Não é ruído — é o mecanismo.** A aresta
  dura de cima foi trocada por uma mistura dos dois tons vizinhos; a 100% ela é fina demais para
  se ver, e o que fica é o degradê liso.

# O que este smoke NÃO mostra, e é honesto dizê-lo

⛔ **Não há uma metade «RGBA16» para comparar.** Ela mostraria faixas *na mesma*: o ecrã é de
8 bits e a descida final **não** leva dither — foi medida e recusada (W6.2, folga de 0,0283 LSB
numa placa em que meio passo precisa de 0,431). Uma terceira metade ensinaria a coisa errada:
que 16 bits não serve para nada. O que ele serve é para **não requantizar entre edições**, e isso
não se vê num ecrã — vê-se ao fim de cinco ferramentas.
```
