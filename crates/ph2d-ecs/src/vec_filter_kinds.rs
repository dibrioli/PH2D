//! **O CATÁLOGO dos tipos de degrau** da pilha de FX raster — o que cada `kind` É, numa tabela.
//!
//! Irmão de [`super::vec_filter`] pelo teto de LOC, e o corte é por responsabilidade: aquele
//! arquivo diz **o que um degrau e uma pilha SÃO** (os campos, os acessores, os defaults), este diz
//! **o que cada TIPO é** — e é este que cresce a cada wave, uma linha de tabela de cada vez.

use crate::FxOp;

/// **O que um tipo de degrau É** — a tabela que decide os controles, a margem e o número de
/// passes. UMA tabela, e todos os consumidores a leem: o painel (que rows oferecer), o passe da
/// GPU (quanto espalhar, quantos dispatches) e o próprio `FxOp` (`tints`/`displaces`).
///
/// ⚠️ **É por isto que ela existe.** A W2 tinha três tipos e o painel decidia por `kind == 2` /
/// `kind != 0` espalhado pelo `paint`. Com sete tipos essa aritmética apodrece na primeira adição
/// — e o modo de falha é um knob morto (ou um knob que falta) que nenhum gate vê.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FxKindSpec {
    /// O nome que o painel mostra (e o rótulo do card).
    pub name: &'static str,
    /// O rótulo do raio, ou `None` se o tipo não tem raio nenhum (o Color Overlay é pontual).
    /// `Some` também significa *este op custa DOIS dispatches* (a Gaussiana separável).
    pub radius_label: Option<&'static str>,
    /// Os rótulos do par de offset, ou `None` se o tipo não desloca nada. ⚠️ **É um par de nomes,
    /// não um booleano:** o mesmo vetor é *para onde a sombra cai* na Drop Shadow e *de onde a luz
    /// vem* no Bevel, e um rótulo errado é um knob que mente sobre o que faz.
    pub offset_labels: Option<(&'static str, &'static str)>,
    /// O rótulo da cor, ou `None` se o tipo não tinge (o Blur reusa os pixels que recebeu).
    pub color_label: Option<&'static str>,
    /// **Espalha para FORA da silhueta que recebeu?** É esta a pergunta da MARGEM da textura: um
    /// halo externo cresce a imagem, um efeito de DENTRO não cresce nada.
    pub grows: bool,
    /// O halo nasce do alfa **INVERTIDO** e é mascarado pela forma — ou seja, mora DENTRO dela.
    pub inner: bool,
    /// Os MODOS que este tipo oferece, na ordem dos códigos, ou vazio se ele não tem escolha a
    /// fazer. O painel pinta um chip por modo e o **índice no slice É o código** — como a `SPECS`,
    /// a lista é a tabela.
    pub modes: &'static [&'static str],
    /// **A COR deste degrau pousa sobre conteúdo que já existe?** É esta a pergunta do BLEND: um
    /// modo de mistura responde *como a minha cor se combina com a que está aí*, e só faz sentido
    /// onde há uma cor lá para se combinar.
    ///
    /// ⚠️ **Campo próprio, e não `!grows`** — hoje as duas listas coincidem, e coincidem por
    /// ACIDENTE: `grows` pergunta *preciso de margem na textura?* e este pergunta *a minha cor
    /// encosta na de baixo?*. Um tipo futuro que espalhe para fora E tinja por dentro (um *Satin*)
    /// responde SIM às duas, e derivar uma da outra o teria feito nascer sem o controle, em
    /// silêncio.
    ///
    /// ⚠️ **Quem NÃO toma, e por quê** (medido — ver `the_blend_of_an_outer_halo_only_reaches_the_
    /// antialiased_fringe` na `ph2d-render`): um halo EXTERNO entra **por baixo** da entrada
    /// (`over + halo·(1−a)`), então onde ele aparece não há nada com que se misturar (`a = 0`) e
    /// onde há algo ele não aparece (`1−a = 0`). O que sobra é o produto `a·(1−a)`, que pica em
    /// **0,25 exatamente na rampa de anti-aliasing** — um controle cujo efeito inteiro é uma orla
    /// de 1 px lê como quebrado. O `Blur` e o `Feather` não têm cor própria nenhuma: a saída deles
    /// É a entrada transformada.
    pub takes_blend: bool,
    /// **Este degrau lê um campo de RUÍDO PROCEDURAL?** — e, se lê, como se chamam os três knobs
    /// dele: `(tamanho, detalhe, semente)`.
    ///
    /// ⚠️ **Um campo só para os três, e não três campos**, porque eles não existem em separado:
    /// *que tamanho têm as ondulações*, *quantas oitavas somam* e *qual das infinitas realizações
    /// é esta* são a MESMA pergunta (*qual ruído?*) em três tempos. Oferecer um sem os outros é um
    /// controle que não descreve nada. É o idioma do [`Self::offset_labels`], que também é um par
    /// de NOMES em vez de um booleano — pela mesma razão: o rótulo é parte do fato.
    pub noise_labels: Option<(&'static str, &'static str, &'static str)>,
    /// O rótulo do **crescimento**, ou `None` se este tipo não engorda nem afina nada.
    ///
    /// ⚠️ **Campo próprio, e não o [`Self::radius_label`]** — os dois são comprimentos de MUNDO e
    /// param aí a semelhança. Um raio é o desvio de uma Gaussiana: não-negativo por definição, e o
    /// slider mapeia `0..MAX`. Este número tem **SINAL** (positivo engorda, negativo afina), e o
    /// mapa track→valor de um slider é registado por LINHA, não por tipo — a linha muda de tipo em
    /// runtime, então uma régua bipolar emprestada ao raio faria o Blur ler metade da faixa dele
    /// como negativa. Duas grandezas, duas réguas.
    pub grow_label: Option<&'static str>,
    /// **Este degrau AJUSTA a cor que recebeu?** — e, se ajusta, como se chamam os três knobs
    /// dele: `(matiz, saturação, brilho)`.
    ///
    /// ⚠️ **Um campo para os três, e não três campos** — é o idioma do [`Self::noise_labels`] e
    /// pela mesma razão: *que matiz*, *quanto croma* e *quão claro* são a MESMA pergunta (*como
    /// esta cor é ajustada?*) em três eixos, e é assim que Photoshop / AE / Krita / Blender
    /// desenharam a ficha. Oferecer um sem os outros deixaria a arte CINZENTA sem nenhum knob
    /// vivo (matiz e saturação não movem um pixel sem croma — só o brilho move).
    pub adjust_labels: Option<(&'static str, &'static str, &'static str)>,
}

/// Os modos de QUEDA. Duas leis diferentes sobre *o que é "perto da borda"*, e a diferença é
/// visível em qualquer forma com reentrância.
///
/// ⚠️ Chamavam-se `INNER_MODES` porque só os degraus de dentro os ofereciam — e o nome era um
/// acidente de quem chegou primeiro, não uma propriedade da escolha: a pergunta *"perto da borda é
/// pouco fora por perto, ou é pouca DISTÂNCIA até ela?"* é a mesma para um halo externo.
pub const FALLOFF_MODES: [&str; 2] = ["Proximity", "Contour"];

/// Os modos do RUÍDO — as duas leis de soma de oitavas, e são exactamente os dois `type` do
/// `feTurbulence` do SVG.
///
/// - **Smooth** (`fractalNoise`): a soma COM SINAL, `Σ n_i/2^i`. Ondulações macias, tipo nuvem.
/// - **Creased** (`turbulence`): a soma dos MÓDULOS, `Σ |n_i|/2^i`, recentrada. Onde cada oitava
///   cruza o zero fica um VINCO — é a textura de fumaça/mármore, e o que dá aspecto de rasgado.
///
/// ⚠️ Os nomes do SVG (`fractalNoise`/`turbulence`) não sobem à UI de propósito: o degrau já se
/// chama *Turbulence*, e um modo com o mesmo nome do tipo não distingue nada.
pub const NOISE_MODES: [&str; 2] = ["Smooth", "Creased"];

/// **A tabela dos tipos.** Indexada pelo `kind`; a ordem É a dos códigos acima (há gate).
pub(crate) const SPECS: [FxKindSpec; FxOp::KINDS] = [
    FxKindSpec {
        name: "Blur",
        radius_label: Some("Radius"),
        offset_labels: None,
        color_label: None,
        grows: true,
        inner: false,
        modes: &[],
        takes_blend: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Glow",
        radius_label: Some("Radius"),
        offset_labels: None,
        color_label: Some("Color"),
        grows: true,
        inner: false,
        // O halo externo faz a MESMA escolha que os de dentro: a silhueta borrada (Proximity)
        // ou uma banda de largura constante ao longo do contorno (Contour). Numa forma com
        // reentrância elas desenham coisas visivelmente diferentes — a ponta de uma estrela
        // brilha nas duas, o vão entre pontas só na segunda.
        modes: &FALLOFF_MODES,
        takes_blend: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Drop Shadow",
        radius_label: Some("Radius"),
        offset_labels: Some(("Offset X", "Offset Y")),
        color_label: Some("Color"),
        grows: true,
        inner: false,
        modes: &[],
        takes_blend: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Inner Shadow",
        radius_label: Some("Radius"),
        offset_labels: Some(("Offset X", "Offset Y")),
        color_label: Some("Color"),
        grows: false,
        inner: true,
        modes: &FALLOFF_MODES,
        takes_blend: true,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Inner Glow",
        radius_label: Some("Radius"),
        offset_labels: None,
        color_label: Some("Color"),
        grows: false,
        inner: true,
        modes: &FALLOFF_MODES,
        takes_blend: true,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Outline",
        // ⚠️ "Width", não "Radius": o contorno se estende EXATAMENTE este tanto a partir de uma
        // aresta reta (o corte duro mora no nível `Φ(−1)` da Gaussiana — há gate que mede).
        radius_label: Some("Width"),
        offset_labels: None,
        color_label: Some("Color"),
        grows: true,
        inner: false,
        modes: &[],
        takes_blend: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Feather",
        radius_label: Some("Feather"),
        offset_labels: None,
        color_label: None,
        grows: true,
        inner: false,
        modes: &[],
        takes_blend: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Bevel",
        radius_label: Some("Depth"),
        offset_labels: Some(("Light X", "Light Y")),
        color_label: Some("Shadow"),
        grows: false,
        inner: true,
        modes: &[],
        takes_blend: true,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Color Overlay",
        radius_label: None,
        offset_labels: None,
        color_label: Some("Color"),
        grows: false,
        inner: false,
        modes: &[],
        takes_blend: true,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Turbulence",
        // ⚠️ "Amount", não "Radius": este número é *quão longe um pixel anda*, e é ele que
        // define a margem (o alcance de um deslocamento É a amplitude dele). Chamá-lo de raio
        // prometeria um borrão.
        radius_label: Some("Amount"),
        offset_labels: None,
        // Não tinge: a saída É a entrada deformada, como o Blur e o Feather. Logo também não
        // tem lei de mistura a aplicar — não há cor DELE que encoste na de baixo.
        color_label: None,
        // Empurra pixels para fora da silhueta que recebeu, até `Amount`.
        grows: true,
        inner: false,
        modes: &NOISE_MODES,
        takes_blend: false,
        noise_labels: Some(("Size", "Detail", "Seed")),
        grow_label: None,
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Grow / Shrink",
        // Sem RAIO: este tipo não borra nada. O comprimento que ele tem é o `grow_label`, e é
        // com sinal — ver o campo.
        radius_label: None,
        offset_labels: None,
        // Não tinge: a saída É a entrada engordada, e a cor da orla nova vem da borda mais
        // próxima (a mesma extensão que o Feather já faz). Logo também não tem lei de mistura.
        color_label: None,
        // Cresce até `grow`, e SÓ quando ele é positivo — quem responde é o `op_reach`, que
        // olha o número; aqui a resposta é *pode crescer*.
        grows: true,
        inner: false,
        modes: &[],
        takes_blend: false,
        noise_labels: None,
        grow_label: Some("Amount"),
        adjust_labels: None,
    },
    FxKindSpec {
        name: "Color Adjust",
        // Pontual como o Color Overlay: sem raio, sem vizinho, um dispatch, margem zero.
        radius_label: None,
        offset_labels: None,
        // ⚠️ **Não tinge, e a distinção é o verbo:** o Color Overlay SUBSTITUI a cor por uma que o
        // artista escolhe; este AJUSTA a que já está lá, relativamente. Uma swatch aqui seria a
        // segunda porta para *"recolorir"*.
        color_label: None,
        grows: false,
        inner: false,
        modes: &[],
        // Sem cor própria, logo sem lei de mistura a aplicar — o argumento do Blur e do Feather:
        // a saída DELE é a entrada transformada.
        takes_blend: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: Some(("Hue", "Saturation", "Brightness")),
    },
];
