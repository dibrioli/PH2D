# Doc-comments órfãos cortados do `main.rs` da shell para baixo do tecto (integração 2026-11-13)

HR-18: `shells/desktop/src/main.rs` estava em 1 008 > 984 depois da cadeia de 13/11. Estes `///` estavam colados ao item ERRADO (o módulo que descreviam já não está por baixo deles — saiu para uma crate ou foi reordenado), e um doc órfão documenta o vizinho com a frase de outro. Verbatim, com a linha onde começavam e o item que estavam a documentar por engano.

## `main.rs:80` (colado a `mod brush_corner_smoke;`)

    /// ⭐ **DAR e TIRAR o traço de uma forma** (plano 34) — a porta da caixa *Stroke* do painel. Existe
    /// porque o `restyle_selected_strokes` recusa quem não tem traço, e essa recusa está CERTA: ele
    /// corre por quadro, e criar ali vestiria toda forma selecionada sem ninguém pedir.

## `main.rs:112` (colado a `/// ⭐ O PRÓLOGO das cenas da família das instâncias — o invólucro que traduz `&mut App` para a`)

    /// O gesto que cria um conector (Down numa forma, Up noutra).

## `main.rs:119` (colado a `#[cfg(test)]`)

    /// Conectores vivos: a linha que gruda em duas formas e as segue (re-cook por frame).

## `main.rs:129` (colado a `mod cut_smoke;`)

    /// **O Width Tool** — as alças de largura na curva (plano 25 W2, ADR-0148).

## `main.rs:134` (colado a `/// `PH2D_EMISSIVE_SMOKE` — a sprite como fonte de luz (plano `docs/Sprite_projeto/18` W8).`)

    /// O canal da **DOAÇÃO de forma** para a tinta do Painter — plano de normais + o tamanho do canvas.
    /// Sem `cfg`, de propósito: o que atravessa é `Vec<f32>`, nunca um tipo do módulo 3D.
    /// ⭐⭐⭐ **As FAIXAS de desenho** (ADR-0154 Fase 2) — a lei que põe vetor e sprite na MESMA ordem
    /// total, e parte essa ordem nas passagens que o presente desenha.

## `main.rs:271` (colado a `/// **Pattern Along Path** — o cozimento vivo do `VecPatternPath` (plano 23), irmão do `offset_live`.`)

    /// A SONDA do drift de pan (`PH2D_PAN_DIAG=1`) — report do Enio de 2026-08-25.

## `main.rs:281` (colado a `/// **A conversão de precisão dos pixels de uma sprite** — o que os botões `RGBA8 / RGBA16` do`)

    /// **O import de uma folha hand-packed** (`folha.png` + `folha.json`) — irmão do
    /// `image_import`, e o primeiro consumidor que o `parse_atlas_meta` tem desde 2026-05-12.
    /// **A folha como OBJETO** (plano `docs/Sprite_projeto/17` §7) — criar uma a partir da seleção e
    /// arranjar as peças dentro dela. Quase nada aqui é código novo: a folha é um retângulo vivo que
    /// ganhou um componente, como a moldura.
    /// **Um contêiner não rouba o clique dos próprios filhos** — a lei que faz um filho de moldura
    /// (ou de folha) ser agarrável no canvas. Pura, e por isso testada.
    /// As fronteiras da folha — confinar uma peça, e contar o que está mal.

## `main.rs:292` (colado a `/// ⭐⭐⭐ **As duas SAÍDAS do palco do prefab** (`Done`/`Cancel`) — a ponte que fica porque o `Cancel``)

    /// **As ferramentas que só movem pixels preservam a precisão** — veja os docs do módulo.

## `main.rs:315` (colado a `/// ⭐⭐ **A TAXONOMIA da biblioteca dentro do ficheiro** (plano 07, A3) — blob auto-versionado.`)

    /// **A arte dos padrões dentro do ficheiro de projecto** (plano 33, W4).

## `main.rs:329` (colado a `/// **As settings do PROJETO viajam no arquivo** (doc 88, D3) — a escala do mundo e a`)

    /// A metade ARQUIVADA da escada do `PROJECT_SCHEMA` (v2..v79) — irmã por LOC (HR-18).
    /// A escada arquivada de `v83` a `v98` — o corte por idade, a 2.ª vez (ver o cabeçalho dela).

## `main.rs:415` (colado a `/// A cena de smoke do **Trim** — `PH2D_BUILD_SMOKE=80` (plano 38).`)

    /// ⭐⭐⭐ **A ferramenta TRIM** (plano 38) — a costura entre o ponteiro e a lei da crate.

## `main.rs:420` (colado a `/// A cena de smoke da **HIERARQUIA** de estados (`PH2D_BUILD_SMOKE=64`) — irmã de `build_smoke`.`)

    /// ⭐ O smoke da UI VIVA (`PH2D_UI_MOTION_SMOKE`) — o carácter e a corda.

## `main.rs:428` (colado a `/// A BOOLEANA dentro do modo de PREVIEW — irmão dos gates do preview por LOC (HR-18).`)

    /// A cena de smoke dos **ESTADOS DE UI** (`PH2D_BUILD_SMOKE=61`) — irmã de `build_smoke`.
    /// ⭐ A cena da BOOLEANA VIVA dentro de um ESTADO de UI (`PH2D_BUILD_SMOKE=74`) — a troca de
    /// operação que MORFA em vez de saltar, com os operandos a mover-se ao mesmo tempo.

## `main.rs:465` (colado a `/// **O papel de cada forma dentro de uma booleana viva** — a porta única de *"que verbo é o`)

    /// ⭐ Os gates de ROTA da booleana viva até os ESTADOS (auditoria de 2026-08-23): *com uma
    /// booleana em mãos, o artista chega às poses dela?* — a pergunta que "o widget existe e o clique
    /// chega ao barramento" não faz.
