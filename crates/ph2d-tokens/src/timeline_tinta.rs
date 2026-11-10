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
    /// A listra da zona depois do fim.
    Listra,
    /// O cursor de tempo.
    Cursor,
    /// A borda da folha com o Auto Key a gravar.
    Gravando,
    /// A moldura da área das trilhas em repouso: o traço da régua da maquete, neutro e sutil.
    Moldura,
}

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
            Self::Listra => (T::KanbanList, [9, 12, 24]),
            Self::Gravando => (T::KanbanList, [196, 116, 114]),
            Self::Moldura => (T::KanbanList, [37, 41, 56]),
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
    }
}
