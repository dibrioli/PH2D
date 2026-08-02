//! **Os presets de dispositivo da moldura** — a tabela, e a resolução id → tamanho.
//!
//! Uma tabela, DOIS leitores: o painel PINTA a partir dela (rótulo + id) e a shell RESOLVE por ela
//! (id → w/h). Duas listas divergiriam no dia em que alguém acrescentasse um aparelho a uma só, e
//! o sintoma seria um botão que pinta e não faz nada.
//!
//! # ⚠️ A unidade foi MEDIDA, e a medição derrubou a leitura óbvia
//!
//! A leitura óbvia — *"uma unidade de documento é um pixel de desenho"*, que é o que o Figma faz —
//! **não é utilizável neste editor hoje**, e o número que a mata é da câmera:
//! `Camera2d::ZOOM_MAX_HEIGHT_WORLD = 100.0` (e a câmera abre em `height_world = 10.0`). Um
//! telefone de 844 pontos seria **8,4× mais alto do que a maior distância a que se pode afastar**:
//! o artista veria 12% da moldura e nunca a moldura.
//!
//! Então a tabela guarda **os pontos REAIS do aparelho** (auditáveis, e é o que a exportação vai
//! querer) e [`DevicePreset::size`] os converte para unidades de documento com o lado maior em
//! [`LONG_SIDE`] — **8 unidades, medidas contra a câmera**: 80% da altura visível na abertura, e
//! ainda 12× dentro do afastamento máximo se o artista quiser contexto. O **aspecto é exato**, que
//! é a propriedade que o preset de facto entrega.
//!
//! ⚠️ Isto é uma conversão de UMA porta, e é o lugar onde a decisão muda quando ela for tomada: no
//! dia em que a exportação (W8) disser o que uma unidade vale em pixels, é esta função que passa a
//! dizê-lo — e não quatro literais espalhados.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;

/// O lado maior de qualquer preset, em unidades de documento. **Medido contra a câmera**, não
/// escolhido: `Camera2d` abre em `height_world = 10.0` e não afasta além de
/// `ZOOM_MAX_HEIGHT_WORLD = 100.0`, então 8 põe a moldura inteira à vista no primeiro quadro e
/// deixa uma ordem de grandeza de folga para trás.
pub const LONG_SIDE: f64 = 8.0;

/// Um preset: o botão, o aparelho, e o que ele escreve.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DevicePreset {
    /// O id do botão no painel.
    pub id: NodeId,
    /// O rótulo. Segue o precedente do `SymmetryKind::label()` deste mesmo vocabulário: literal
    /// em inglês na crate do vocabulário, e não uma chave i18n — metade destes rótulos são
    /// medidas e a outra metade nomes de classe de aparelho.
    pub label: &'static str,
    /// A largura do aparelho, em **pontos lógicos publicados**. É o número auditável; não é o que
    /// o campo W recebe (ver [`DevicePreset::size`]).
    pub pts_w: f64,
    /// A altura do aparelho, em pontos lógicos.
    pub pts_h: f64,
}

impl DevicePreset {
    /// O tamanho em unidades de DOCUMENTO: o aspecto do aparelho, com o lado maior em
    /// [`LONG_SIDE`].
    #[must_use]
    pub fn size(self) -> (f64, f64) {
        let long = self.pts_w.max(self.pts_h);
        // A tabela é const e nenhum preset é degenerado (há gate); a guarda protege contra uma
        // linha nova escrita com zero, que de outro modo devolveria NaN em silêncio.
        if long <= 0.0 {
            return (LONG_SIDE, LONG_SIDE);
        }
        let k = LONG_SIDE / long;
        (self.pts_w * k, self.pts_h * k)
    }
}

/// Os quatro presets. **Quatro, e não quarenta**: o campo W/H alcança qualquer tamanho, então
/// esta lista não é um teto — é o atalho para as quatro proporções que se desenham todo dia.
pub const DEVICE_PRESETS: &[DevicePreset] = &[
    DevicePreset {
        id: ids::VECTOR_FRAME_PRESET_PHONE,
        label: "Phone",
        // iPhone 14/15/16.
        pts_w: 390.0,
        pts_h: 844.0,
    },
    DevicePreset {
        id: ids::VECTOR_FRAME_PRESET_TABLET,
        label: "Tablet",
        // iPad Pro 12,9" em retrato.
        pts_w: 1024.0,
        pts_h: 1366.0,
    },
    DevicePreset {
        id: ids::VECTOR_FRAME_PRESET_DESKTOP,
        label: "Desktop",
        pts_w: 1920.0,
        pts_h: 1080.0,
    },
    DevicePreset {
        id: ids::VECTOR_FRAME_PRESET_SQUARE,
        label: "Square",
        pts_w: 1080.0,
        pts_h: 1080.0,
    },
];

/// O preset que este id pede, se algum.
#[must_use]
pub fn device_preset(id: NodeId) -> Option<DevicePreset> {
    DEVICE_PRESETS.iter().copied().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dois presets com o mesmo id fariam um deles inalcançável — e o botão morto seria o de
    /// baixo, que ninguém suspeitaria.
    #[test]
    fn every_preset_has_its_own_id() {
        for (i, a) in DEVICE_PRESETS.iter().enumerate() {
            for b in &DEVICE_PRESETS[i + 1..] {
                assert_ne!(a.id, b.id, "{} e {} partilham id", a.label, b.label);
            }
        }
    }

    /// A resolução é a INVERSA da tabela: o que a tabela pinta é o que o clique resolve.
    #[test]
    fn the_table_and_the_lookup_agree() {
        for p in DEVICE_PRESETS {
            assert_eq!(device_preset(p.id), Some(*p));
        }
        assert_eq!(device_preset(ids::VECTOR_MODE_FRAME), None);
    }

    /// **O ASPECTO é exato** — é a propriedade que o preset entrega, e a única que sobrevive à
    /// conversão de unidade.
    #[test]
    fn the_aspect_survives_the_conversion() {
        for p in DEVICE_PRESETS {
            let (w, h) = p.size();
            let want = p.pts_w / p.pts_h;
            assert!(
                (w / h - want).abs() < 1e-12,
                "{}: aspecto {} != {want}",
                p.label,
                w / h
            );
        }
    }

    /// ⚠️ **A moldura tem de CABER no que a câmera mostra.** O número que este gate afirma é o da
    /// `ph2d_render::Camera2d`: ela não afasta além de 100 unidades de altura, então um preset em
    /// pontos crus (844) seria uma moldura que o artista nunca vê inteira.
    #[test]
    fn every_preset_fits_inside_what_the_camera_can_show() {
        // O teto da câmera, escrito aqui em literal de propósito: `ph2d-tool-vector` não depende
        // de `ph2d-render`, e o valor é o que o gate está a afirmar sobre o mundo.
        const CAMERA_MAX_HEIGHT_WORLD: f64 = 100.0;
        for p in DEVICE_PRESETS {
            let (w, h) = p.size();
            assert!(w > 0.0 && h > 0.0, "{} é degenerado", p.label);
            assert!(
                w.max(h) <= CAMERA_MAX_HEIGHT_WORLD,
                "{}: {}x{} nao cabe no afastamento maximo da camera",
                p.label,
                w,
                h
            );
            assert!(
                (w.max(h) - LONG_SIDE).abs() < 1e-12,
                "{}: o lado maior tem de ser LONG_SIDE",
                p.label
            );
        }
    }
}
