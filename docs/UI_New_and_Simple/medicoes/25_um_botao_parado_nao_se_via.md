# 25 — Um botão parado não se via, e havia DUAS leis para a cor dele

> **Report do dono, 2026-09-20, com duas fotos:** *«a aparência deste tipo de botão precisa mudar:
> veja como não se pode saber que é um botão pois só aparece o nome. **Todo o app tem essa
> aparência ruim** desse tipo de botão.»*

## §1 — O que a foto mostra, e o que ela já separa

Na primeira foto, o painel `Sculpt 3D`:

| o que se vê | lê-se como |
|---|---|
| `S \| B` (Reference) · `Basic \| Pro` · `Move \| Rotate \| Scale` | **botões** — têm superfície |
| `Draw` · `Filter Collisions` · `Apply to all tools` · `Filter Whole Mesh` | **legendas** — só texto centrado |
| `X` · `Y` · `Z` (Symmetry) · `Dynamic Topology` · `Transform Free Part` | **legendas** |

⭐ **A foto já contém o diagnóstico:** o que tem superfície são os **chips segmentados**; o que não
tem são os que passam pelo **`Button`** canónico. *Dois widgets, dois dialectos.*

## §2 — A causa: duas leis, e o doc de uma afirmava ser a outra

```
flat_button_surface(state)        →  repouso = Bg2        (5 sítios de pintura, os chips)
Button::bg_token(Default, Normal) →  None                 (o Button canónico)
```

⛔⛔ **E o doc da primeira diz**, textualmente:

> *«repouso `Bg2`, Hovered/Focused `BgElev`, Pressed `AccentSoft` — **as mesmas superfícies que o
> `Button` canónico usa**»*

**Não eram.** ⇒ havia duas respostas a *«que cor tem um botão parado?»*, e o doc de uma **afirmava
ser** a outra. Nenhum gate podia ver isso: cada lei tinha os testes dela, e os dois passavam.

## §3 — E a tabela de design desta linha já declarava a resposta

[`pesquisa/08 §7.16`](../pesquisa/08_modelos_com_codigo_para_seguir.md), escrita quando o dono
reprovou o contraste dos cartões:

> *«Subir o cartão para o `Bg2` estava fora por medição: **um botão em repouso PINTA `Bg2`**
> (`flat_button_surface`), e os botões dentro do cartão desapareceriam.»*

| tema | painel | cartão | **botão (`Bg2`)** |
|---|---|---|---|
| Dark | `#131313` | `#1f1f1f` | **`#292929`** |

⇒ a cura não escolhe cor nenhuma: põe o `Button` a obedecer à escada que já foi medida e aprovada.

## §4 — ⛔ A cura NÃO é devolver a moldura, e a razão está no pintor

O `paint_button` filtra o traço de repouso por
`ph2d_tokens::visuals::Widgets::of(theme).inactive.bg_stroke.is_visible()` — **falso** num tema
moderno (o Godot só as traça com *Draw Extra Borders*), e o redesenho tirou-as de propósito
(*«moldura zero, raio 4»*).

⚠️⚠️ **E o `border_color` do mesmo ficheiro já escrevia o report inteiro, dois anos-luz antes dele:**

> *«Secondary / ghost (`Default`) buttons **always** carry a `Border` outline so they read as
> buttons even with no fill — **without it a Normal-state Cancel / Reset is bare text
> indistinguishable from a label**.»*

⇒ **uma promessa escrita num sítio e desligada noutro.** A palavra *always* daquele doc era falsa
desde o dia do tema moderno, e o que sobrou foi exactamente a frase que ele usa para descrever o
defeito.

⭐ *A afordância de um botão plano é a SUPERFÍCIE dele, não um contorno* — que é o que o Godot
Modern faz (um `Button` tem `StyleBoxFlat` com preenchimento; só o `flat` é nu).

## §5 — O que fica gateado

| gate | afirma |
|---|---|
| `default_normal_paints_its_surface` | repouso = `Bg2`, **e** repouso ≠ hover (senão pintar o hover no repouso passaria) |
| `an_icon_only_chip_stays_frameless_at_rest` | o chip de ícone da fila **continua** sem superfície — uma grelha densa de ícones com fundo vira um tabuleiro de xadrez |

⚠️ O gate anterior chamava-se `default_normal_has_no_bg` e afirmava `is_none()`: era **o defeito
escrito como lei**. Ele foi reescrito com a morte da premissa à vista no diff.

**Mutação:** devolver `None` ao repouso do `Default` ⇒ sangra.
