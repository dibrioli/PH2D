# W3+ — O plano: um jogo de plataforma é o smoke, a essencialidade é a ordem

> Decisões do dono (25/10), sobre o estudo [`02_godot_2d_catalogo.md`](02_godot_2d_catalogo.md):
> 1. **Nomes simples** (inglês, HR-15). Ao passar o rato: **não** o nome da Godot, mas uma **explicação completa
>    com ilustração**, como as do Fusion 360 (tooltip rico: o que o objecto é, para que serve, uma imagem).
> 2. **A janela de jogo: as duas** — janela separada E dentro do canvas em ecrã cheio.
> 3. **A ordem:** começar por uma **janela de runtime rudimentar** que melhora ao longo da obra; depois manda a
>    **essencialidade** (o que outro precisa vem antes: `Node` antes de `Node2D`) e **a necessidade do jogo**:
>    *«nosso smoke será um jogo funcional completo de relativa complexidade […] um game de plataforma de mediana
>    complexidade»*.

## O jogo (o smoke que cresce a cada onda)

Um plataforma de complexidade média, um nível completo, feito SÓ no editor da PH2D (sem código Rust de cena):

| # | O que o jogo tem | O que a engine precisa (catálogo 02) |
|---|---|---|
| J1 | Uma cena abre numa **janela de jogo** e fecha de volta ao editor intacto | runtime rudimentar (Play/Stop), ciclo de vida do `Node` |
| J2 | O jogador é uma **sprite** que anda e salta, com gravidade, sobre o **chão** e plataformas | `Node2D`, `Sprite`, `Collision Shape`, `Static Body`, `Character` (andar/saltar, `on_floor`), entrada (setas/espaço) |
| J3 | Animações **parado / correr / saltar / cair** | `Animated Sprite` a tocar no jogo, trocado pelo estado do personagem |
| J4 | A **câmara** segue o jogador, presa aos limites do nível; **fundo em parallax** | `Camera` (seguir, limites), `Parallax` |
| J5 | **Moedas** que se apanham (som + contador no HUD) | `Area` + sinal «entrou», `Audio Player`, HUD (`Label`), destruir objecto |
| J6 | **Inimigos** que patrulham e viram na borda; tocar = perder vida; saltar em cima = derrotá-los | `Raycast` (borda), sinais de colisão por objecto, vida/dano |
| J7 | **Plataformas móveis** e **espinhos** | corpo animado (`AnimatableBody`), `Path`/`Path Follow`, `Timer` |
| J8 | **Checkpoint**, vidas, **game over** e **reinício** | estado do jogo, mudar/recarregar cena |
| J9 | **Menu inicial**, pausa e **porta de fim de nível** | interface (`Button`, contentores), mudar de cena, pausa |
| J10 | **Exportar** o jogo e jogá-lo sem o editor | export |

Cada onda fecha com o dono a jogar o jogo até onde ele chegou. O nível usa sprites e formas vetoriais
para o cenário; o mapa de tiles fica no projecto `docs/Tilling` (dono, 16/09) até o jogo o pedir.

## A ordem de construção (essencialidade × o jogo)

1. **R0 — a janela de runtime rudimentar**: Play abre a cena numa janela (e em ecrã cheio no canvas), Stop
   devolve o editor ao bit; o jogo corre sobre uma CÓPIA do mundo (o documento não muda ao jogar).
2. **N0 — o `Node`**: o ciclo de vida no script (`ready`, `process`, `physics`, `exit`), grupos, sinais por
   objecto — a base que todos os outros usam.
3. **N1 — os objectos do J2/J3**: o menu Add ganha os objectos (Sprite, Animated Sprite, Static Body, Character,
   Collision Shape), cada um um conjunto fixo de componentes, com Inspector e **tooltip ilustrado**.
4. Depois, J4 → J10 pela tabela acima; cada objecto novo entra quando o jogo o pede, e entra completo (nome
   simples, tooltip ilustrado, Inspector, funções e sinais no script).

⛔ Recusado à partida: copiar a árvore de herança da Godot no menu Add (spec/06 §5 — temos ~20 tipos, não 150);
o tooltip com o nome da Godot (decisão 1).
