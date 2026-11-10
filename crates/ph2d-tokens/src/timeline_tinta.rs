//! ⭐ **As tintas da Timeline nos cartões** — a harmonia da maquete B1 (dono 09/11: «imitemos TODAS as
//! cores dela»), irmã do [`crate::color`] (no tecto de 700 linhas).
//!
//! Cada tinta é **um token base da paleta do cartão + o desvio OKLab que a maquete mede a partir dele**.
//! A maquete foi desenhada sobre a paleta do cartão do tema de fábrica (`KanbanList` = o fundo dela,
//! `KanbanLabelInk` = o texto do objecto, `KanbanBlueBold` = a chave escolhida, ao bit). No tema de
//! referência ([`REFERENCIA`]) a tinta é a cor AMOSTRADA na maquete; nos outros temas, o mesmo desvio
//! sobre a base daquele tema — a relação, não o número, viaja. Amostras (sRGB, moda de bloco, captura
//! chrome headless de `critica_ph2d.html#timeline`) no handoff da onda 2 (2g).

use crate::{Color, ColorToken, Theme, srgb_to_oklch};

/// O tema em que a maquete foi amostrada: a paleta do cartão dele é a da maquete.
pub const REFERENCIA: Theme = Theme::Forge;

/// A claridade OKLab acima da qual a base é de tema claro.
const CLARO: f64 = 0.6;

/// Uma superfície ou tinta da Timeline nos cartões.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimelineTinta {
    /// A fita sob as propriedades (sem zebra: a maquete separa por fio).
    FitaPropriedade,
    /// A fita sob o cabeçalho de um objecto.
    FitaCabecalho,
    /// O fio entre duas linhas da fita.
    FitaFio,
    /// A régua, por cima da fita.
    FitaRegua,
    /// O traço da régua.
    ReguaTraco,
    /// A coluna de nomes sob as propriedades.
    NomesPropriedade,
    /// A coluna de nomes sob o cabeçalho de um objecto.
    NomesCabecalho,
    /// O fio entre duas linhas na coluna de nomes.
    NomesFio,
    /// O nome do objecto.
    TextoCabecalho,
    /// O nome da propriedade.
    TextoPropriedade,
    /// O valor da propriedade (e os números da régua).
    TextoValor,
    /// A barra de uma chave suavizada (curva): ciano.
    BarraCurva,
    /// A barra de uma chave linear: violeta.
    BarraReta,
    /// A barra de uma chave segura (tracejada) e o ponto do objecto.
    Ponto,
    /// A chave.
    Chave,
    /// A chave escolhida (e o azul de destaque da maquete).
    ChaveEscolhida,
    /// O chip escolhido da barra (o «jump» da maquete).
    ChipEscolhido,
    /// A célula do nome do Summary (forma C do dono, azul da maquete).
    SummaryCelula,
    /// A fita do Summary.
    SummaryFita,
    /// O cursor de tempo.
    Cursor,
    /// A borda da folha com o Auto Key a gravar.
    Gravando,
    /// A moldura da área das trilhas em repouso: o traço da régua da maquete, neutro e sutil.
    Moldura,
    /// A cor de uma peça (strip) de clipe no Arrange/Containers, pela ordem do clipe: [`PECAS`].
    PecaA,
    /// Segunda cor de peça de clipe (o «walk» da maquete).
    PecaB,
    /// Terceira cor de peça de clipe (o «jump»).
    PecaC,
    /// Quarta cor de peça de clipe (o «swing»).
    PecaD,
    /// Quinta cor de peça de clipe (o «shake»).
    PecaE,
    /// A peça de um container, a barra dele na lista e o seu destaque (o laranja da maquete).
    PecaContainer,
    /// A pílula do peso de uma faixa (o «1.0» da coluna de nomes).
    PesoPilula,
    /// O olho de uma faixa muda (fechado, avermelhado e meio apagado).
    OlhoMudo,
    /// A faixa fina do laço no topo da régua (o `.c13lp` da maquete B: o verde da [`Self::PecaA`]).
    LacoDaRegua,
    /// O losango `Mudado` do Inspector e o valor do campo mudado (maquete B1: `#FF9F5A`, o laranja do
    /// Spine/Blender; a mesma amostra da [`Self::PecaContainer`], outro significado).
    Mudado,
    /// O contorno do campo mudado (maquete B1: o laranja a ~58 % sobre o cartão, amostra da moda).
    MudadoContorno,
}

/// As cores das peças de clipe, na ordem da maquete; o clipe `i` veste `PECAS[i % PECAS.len()]`.
pub const PECAS: [TimelineTinta; 5] = [
    TimelineTinta::PecaA,
    TimelineTinta::PecaB,
    TimelineTinta::PecaC,
    TimelineTinta::PecaD,
    TimelineTinta::PecaE,
];

impl TimelineTinta {
    /// `(base na paleta do cartão, a cor amostrada na maquete)`.
    const fn par(self) -> (ColorToken, [u8; 3]) {
        use ColorToken as T;
        match self {
            Self::FitaPropriedade => (T::KanbanList, [7, 9, 19]),
            Self::FitaCabecalho => (T::KanbanList, [12, 15, 27]),
            Self::FitaFio => (T::KanbanList, [11, 13, 23]),
            Self::FitaRegua => (T::KanbanList, [16, 20, 37]),
            Self::ReguaTraco => (T::KanbanList, [37, 41, 56]),
            Self::NomesPropriedade => (T::KanbanList, [20, 24, 40]),
            Self::NomesCabecalho => (T::KanbanList, [25, 29, 45]),
            Self::NomesFio => (T::KanbanList, [32, 36, 51]),
            Self::TextoCabecalho => (T::KanbanLabelInk, [222, 228, 234]),
            Self::TextoPropriedade => (T::KanbanLabelInk, [150, 159, 171]),
            Self::TextoValor => (T::KanbanLabelInk, [100, 108, 120]),
            Self::BarraCurva => (T::KanbanBlueBold, [82, 165, 211]),
            Self::BarraReta => (T::KanbanBlueBold, [131, 113, 222]),
            Self::Ponto | Self::Cursor => (T::KanbanLabelInk, [169, 174, 193]),
            Self::Chave => (T::KanbanLabelInk, [220, 226, 245]),
            Self::ChaveEscolhida => (T::KanbanBlueBold, [87, 157, 255]),
            Self::ChipEscolhido | Self::SummaryCelula => (T::KanbanList, [34, 54, 86]),
            Self::SummaryFita => (T::KanbanList, [20, 36, 68]),
            Self::Gravando => (T::KanbanList, [196, 116, 114]),
            Self::Moldura => (T::KanbanList, [37, 41, 56]),
            // As cinco `--c13-*` e o `--c13-cont` do CSS da maquete (onda 3), ao bit.
            Self::PecaA | Self::LacoDaRegua => (T::KanbanBlueBold, [73, 224, 174]),
            Self::PecaB => (T::KanbanBlueBold, [255, 207, 77]),
            Self::PecaC => (T::KanbanBlueBold, [183, 156, 255]),
            Self::PecaD => (T::KanbanBlueBold, [108, 203, 250]),
            Self::PecaE => (T::KanbanBlueBold, [255, 122, 168]),
            Self::PecaContainer | Self::Mudado => (T::KanbanBlueBold, [255, 159, 90]),
            Self::MudadoContorno => (T::KanbanList, [162, 107, 71]),
            Self::PesoPilula => (T::KanbanList, [33, 37, 52]),
            Self::OlhoMudo => (T::KanbanList, [137, 83, 87]),
        }
    }

    /// A tinta neste tema: a amostra no [`REFERENCIA`], o mesmo desvio OKLab sobre a base nos outros.
    #[must_use]
    pub fn resolve(self, theme: Theme) -> Color {
        let (base, alvo) = self.par();
        let lab = |c: Color| {
            let (l, ch, h) = srgb_to_oklch(c.r, c.g, c.b);
            let h = h.to_radians();
            (l, ch * h.cos(), ch * h.sin())
        };
        let a = lab(Color {
            r: alvo[0],
            g: alvo[1],
            b: alvo[2],
            a: u8::MAX,
        });
        let r = lab(base.resolve(REFERENCIA));
        let b = lab(base.resolve(theme));
        // Num tema claro (base acima do meio) o degrau de claridade inverte: «mais fundo» na maquete
        // escura é «mais fundo» lá também, sem saturar no branco.
        let sentido = if (b.0 > CLARO) == (r.0 > CLARO) {
            1.0
        } else {
            -1.0
        };
        let (l, x, y) = (
            b.0 + (a.0 - r.0) * sentido,
            b.1 + a.1 - r.1,
            b.2 + a.2 - r.2,
        );
        if theme == REFERENCIA
            || (b.0 - r.0).abs() + (b.1 - r.1).abs() + (b.2 - r.2).abs() < f64::EPSILON
        {
            return Color {
                r: alvo[0],
                g: alvo[1],
                b: alvo[2],
                a: u8::MAX,
            };
        }
        Color::from_oklch(
            l.clamp(0.0, 1.0),
            x.hypot(y),
            y.atan2(x).to_degrees().rem_euclid(360.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No tema de referência cada tinta é a amostra da maquete, ao bit (a ida e volta OKLab não pode
    /// arredondar a amostra para o lado).
    #[test]
    fn no_tema_da_maquete_a_tinta_e_a_amostra() {
        let t = TimelineTinta::FitaPropriedade.resolve(REFERENCIA);
        assert_eq!((t.r, t.g, t.b), (7, 9, 19));
        let t = TimelineTinta::BarraReta.resolve(REFERENCIA);
        assert_eq!((t.r, t.g, t.b), (131, 113, 222));
        let t = TimelineTinta::PecaContainer.resolve(REFERENCIA);
        assert_eq!((t.r, t.g, t.b), (255, 159, 90));
    }

    /// As seis cores de peça (cinco de clipe + a do container) leem-se diferentes em TODO tema: uma peça
    /// de `walk` não pode passar pela de `jump`. Piso medido na maquete (Forge): o par mais perto, swing ×
    /// idle, a 0,14 em OKLab.
    #[test]
    fn as_cores_das_pecas_leem_se_diferentes_em_todo_tema() {
        let lab = |c: Color| {
            let (l, ch, h) = srgb_to_oklch(c.r, c.g, c.b);
            let h = h.to_radians();
            (l, ch * h.cos(), ch * h.sin())
        };
        let mut todas: Vec<TimelineTinta> = PECAS.to_vec();
        todas.push(TimelineTinta::PecaContainer);
        let mut pior = f64::MAX;
        for theme in Theme::ALL {
            for (i, a) in todas.iter().enumerate() {
                for b in &todas[i + 1..] {
                    let (p, q) = (lab(a.resolve(theme)), lab(b.resolve(theme)));
                    let d =
                        ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) + (p.2 - q.2).powi(2)).sqrt();
                    pior = pior.min(d);
                    assert!(d >= 0.10, "{theme:?}: {a:?} × {b:?} a {d:.3}");
                }
            }
        }
        assert!(pior < 1.0, "a régua mediu alguma coisa: {pior}");
    }
}
