# O smoke `PH2D_ANIM_SMOKE`: a §11 Animation a andar

Cabeçalho de `shells/desktop/src/anim_smoke.rs`, movido para cá em 07/11 pela catraca da shell.

```text
`PH2D_ANIM_SMOKE` — **a §11 Animation a andar** (spec Sprite 08).

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
  && env PH2D_ANIM_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

Uma sprite com uma **tira de 8 células** — cada uma com um quadrado a subir e a mudar de cor,
para que o frame que está no ecrã se leia de relance — e **três animações** sobre a mesma
tira, que é a tese do modelo (§8.2: *frames são um pool, animações são intervalos nomeados*):

| animação | células | direção | o que prova |
|---|---|---|---|
| `idle` | 0-3 | Ping-Pong | vai e volta sem repetir as pontas, respira (`hold`), **hesita numa célula** (§8.12) e **grita a cada volta** (§8.10) |
| `walk` | 0-7 | Forward | a tira inteira, em ciclo — **uniforme e CALADA**, que é o default |
| `attack` | 4-7 | Forward, 1 volta | **pára no ÚLTIMO frame** — e **anuncia o fim**, uma vez |

⚠️ **As três partilham as MESMAS células.** É isto que o modelo do Aseprite compra e que o
`SpriteFrames` do Godot (N arrays separados) não tem: `idle` e `walk` sobrepõem-se em 0-3 sem
um único pixel duplicado.

# O gesto

A sprite nasce selecionada e a tocar `walk`. Com a seção **Animation** aberta:

1. A barra **Frame N / 8** anda sozinha, e o desenho muda com ela.
2. Clicar em **`attack`** na lista → ela toca uma vez e **fica na última célula**. Clicar em
   `walk` volta ao ciclo.
3. **Speed** a `-1` toca ao contrário; a `0` pausa sem perder o sítio.
4. Clicar em **`idle`** → vai e volta entre as quatro primeiras, com uma pausa no fim da volta.
   ⚠️ **Repare que ela HESITA na 3.ª célula** (460 ms contra 80 das outras): é a *antecipação*
   que um artista desenha à mão, e o caso que um `frame_ms` único não sabia exprimir — uma
   animação importada saía com o tempo **achatado** (spec §8.12). Abaixo do campo `Frame ms`
   aparece a linha *«this animation has per-frame timing»*; na `walk` ela não aparece.
   ⚠️ E a cada volta sai um **sinal** (`idle_cycle`) como um aviso na tela; o `attack` manda um
   (`attack_done`) ao acabar, e a `walk` — a que toca por omissão — **não diz nada**. O silêncio
   é o default (spec §8.10).
5. **Direction override** força uma direção sobre a que a animação declara; **Inherit** devolve.
6. **Rewind** devolve a imagem à primeira célula da animação escolhida.
7. Depois de o `attack` acabar, **Playing** volta a tocá-lo do princípio — num clique só.
8. **Arrastar a barra `Frame N / 8`** move a célula debaixo do dedo — e **pausa** a reprodução.
9. Na seção **Sprite Sheet**, marcar **«Show sheet on canvas»**: a tira inteira abre-se ao lado
   da célula viva, esmaecida, com as **linhas dos cortes** por cima e a célula que está no ecrã
   contornada. É como se vê se o `H Frames` corta onde a arte espera.

10. Com a sprite selecionada, entre no **Painter**: a folha **abre-se inteira**, cada quadro no
    lugar dele e no tamanho certo — e uma **célula extra, acima dela, toca a animação** enquanto
    se pinta, **mesmo com o transporte pausado**. ⚠️ Antes disto a tira saía esmagada 8:1 dentro
    de uma célula (report do Enio, com foto), e o ponteiro tinha o mesmo esmagamento: as duas
    contas eram a mesma, e ambas erradas.
11. Com **Show sheet on canvas** marcado, a **caixa de seleção envolve a folha inteira** — não
    uma célula no meio de oito. Arrastar uma alça escala a folha toda, porque as células saem do
    tamanho do sprite.
12. Com **Show sheet on canvas** marcado **e** o Painter aberto, as **linhas caem em cima dos
    cortes** da arte. ⚠️ Elas caíam desalinhadas (2.º report, com foto): a folha pintada
    centra-se no pivô e a grelha dispunha-se à volta da célula viva — duas âncoras, e as linhas
    seguiam a errada.

⚠️ **Os passos 2, 6 e 7 são a auditoria de 2026-08-23** ([doc 21](../../../docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md)):
até ela, o passo 2 deixava a sprite muda (escolher outra não retomava a que se esgotara), o
*Rewind* repunha contadores sem mexer na imagem, e a caixa precisava de dois cliques.

⚠️ **Como saber que está errado:** a barra parada com `Playing` marcado, ou o `attack` a voltar
à primeira célula em vez de ficar na última.
```
