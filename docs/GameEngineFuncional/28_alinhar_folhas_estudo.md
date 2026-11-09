# 28 — Alinhar folhas no editor de animações (estudo, 09/11)

> Pedido do dono no smoke 09/11 (#4): *«settar o pivot em Animated está confuso porque se trata de sheets diferentes
> com tamanhos de quadros diferentes… precisaremos de uma tool especial e completa para settar pivot, scales, position
> e rotation com opção por sheet, por animação ou por quadro, tudo dentro do editor de animações»*. Ele aprovou estudar;
> **nada construído** até ele aprovar este plano.

## O que existe hoje (inventário 09/11)

- Por folha: só `size` e `anchor` (`ExtraSheet`, `crates/ph2d-ecs/src/sprite_sheets.rs:20`; a folha 0 no `Sprite`).
  `sheet_sprite` (`crates/ph2d-render/src/sprite/sheets.rs:32`) copia-os; escala e rotação são do `Transform` do objecto,
  iguais para todas as folhas.
- Por quadro: nada geométrico. `FrameRef{sheet,cell}` é só índice; `FrameMarker` é sinal; o tempo tem `per_frame_ms`.
- Sem onion skin no editor (`ph2d-anim-editor`); o único onion é o das malhas do esqueleto (`ph2d-render`).
- O Pivot por folha e o encaixe com Ctrl (centro, cantos, meios, centro do desenho) já existem (`ph2d-sprite-edit/src/pivot.rs`).

## Como os outros fazem (referência de idioma, não de código)

| App | Onde vive o acerto | Lição |
|---|---|---|
| Unity Sprite Editor | Pivot por fatia (= por quadro) | por quadro resolve tudo, mas é trabalho manual quadro a quadro |
| Aseprite | posição de cada *cel* + onion skin | o onion skin é o que torna o acerto possível — sem ver o anterior não se alinha |
| Godot AnimatedSprite2D | um `offset` só, do nó | folhas de tamanhos diferentes saltam; queixa recorrente |
| Spine / Spriter | transformação por quadro-chave | poderoso, mas é animação de recortes, outro produto |

## Proposta

1. **Modo «Align» dentro do editor de animações:** uma prévia grande com
   - o quadro atual **por cima** do anterior (ou de um quadro de referência, de qualquer folha) **translúcido** — onion skin;
   - a cruz do Pivot e uma **linha de chão** (a altura dos pés) que se arrasta.
2. **Dois níveis, não três:**
   - **Folha** (base): posição (deslocamento), escala e rotação da folha inteira, além do Pivot que já existe.
     Resolve 90 %: «a run é maior e está mais alta que a idle».
   - **Quadro** (exceção): um deslocamento em pixels por quadro, com setas do teclado (1 px; Shift = 10 px).
     ⚠️ **Mora na CÉLULA da folha, não na animação:** o desenho é o mesmo em qualquer animação que o use,
     logo o acerto também. Quadro acertado ganha uma marca na faixa.
   - **Por animação: não.** Uma animação usa células de folhas; o acerto da folha e o da célula já valem nela.
     Um terceiro nível faria o artista perguntar «onde é que eu mexi?».
3. **Ajudas automáticas:** «Alinhar pelos pés» (o Pivot de cada folha, ou de cada quadro, ao meio da base da caixa opaca)
   e «Alinhar pelo centro». Um clique, e desfaz-se com Ctrl+Z.
4. **Dados:**
   - folha: `offset`, `scale` e `rotation` no `ExtraSheet` e o equivalente para a folha 0;
   - célula: um mapa esparso `cell → [dx, dy]` em px.
   - Sobe o `PROJECT_SCHEMA` (regra 11: sem compatibilidade).
   - O render aplica `Transform` do objecto × transformação da folha × deslocamento da célula.
   - A caixa do gizmo, o encaixe e a colisão leem a mesma geometria (`sheet_geometry`).

## Ondas sugeridas

- **A — dados + render:** os campos, o render, a caixa e o encaixe a lerem-nos; gates de geometria.
- **B — a prévia «Align»:** onion skin, quadro de referência, cruz do Pivot, linha de chão; folha: arrastar, escalar, rodar.
- **C — por quadro:** setas, marca na faixa, «Alinhar pelos pés/centro».
- **D — cena de smoke** com duas folhas de tamanhos diferentes desalinhadas de propósito, e fotos.

## Perguntas ao dono antes da onda A

1. A rotação por folha é precisa? Ela entra pelo mesmo caminho da escala; por quadro, a proposta é só deslocamento.
2. O onion skin mostra por omissão o quadro anterior, ou o primeiro quadro da animação «idle» (a referência dos pés)?
