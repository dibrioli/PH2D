//! **O fator de aparência do documento** (B34, decisão 6 do dono 10/11): as escalas que o motor escreve em
//! píxeis absolutos (período do papel, orla da aguada, warp, relevo, sombra do óleo…) foram aprovadas numa
//! tela 1080p; numa tela maior ficam relativamente menores (a 4K, à metade). O fator diz quantas vezes a
//! tela é maior que a de referência e viaja COM o documento (stash, undo-free, disco).
//!
//! Nasce do tamanho quando o documento nasce (`set_source`) e nunca mais muda: redimensionar a tela não
//! re-escala a tinta que já está nela. Até 1080p o fator é `1` — o motor de sempre, ao bit.
//! O papel lê-o por [`PainterTool::papel_na_escala`] (B34.1); as outras escalas entram item a item (B34.2..B34.7,
//! `docs/Painter/missao_4k/BACKLOG.md`).

use crate::tool::PainterTool;
use ph2d_painter_brush::TextureSettings;

/// O lado CURTO da tela de referência, em píxeis: os pincéis foram aprovados a 1920×1080.
pub const LADO_CURTO_DE_REFERENCIA: u32 = 1080;

/// O fator de uma tela `w × h`: lado curto ÷ 1080, nunca abaixo de 1 (uma tela pequena pinta como sempre).
/// 1920×1080 → 1 · 2560×1440 → 1,33 · 3840×2160 → 2.
#[must_use]
pub fn do_tamanho(w: u32, h: u32) -> f32 {
    (w.min(h) as f32 / LADO_CURTO_DE_REFERENCIA as f32).max(1.0)
}

/// O papel `p` à escala de um documento de fator `fator`: o Size do papel é FREQUÊNCIA (células por
/// `TEX_TILE_BASE_PX` píxeis), logo o dente fica `fator`× maior em píxeis quando o Size DIVIDE pelo fator — a 4K
/// o papel cobre a mesma fração da tela que a 1080p. Fator 1 devolve `p` intacto (ao bit).
#[must_use]
pub fn papel_a_escala(mut p: TextureSettings, fator: f32) -> TextureSettings {
    if fator != 1.0 {
        p.size = p.size.map(|s| s / fator);
    }
    p
}

impl PainterTool {
    /// O fator de aparência do documento ligado (`1` = a tela de referência).
    #[must_use]
    pub fn fator_aparencia(&self) -> f32 {
        self.fator_aparencia
    }

    /// (Sondas e pranchas) Força o fator do documento ligado — o «antes» de uma tela grande é o fator 1.
    #[doc(hidden)]
    pub fn fixa_fator_aparencia_para_sonda(&mut self, fator: f32) {
        self.fator_aparencia = fator;
    }

    /// ⭐ A porta ÚNICA do papel em píxeis (B34.1): todo leitor que amostra o papel na tela passa por aqui.
    #[must_use]
    pub(crate) fn papel_na_escala(&self, mut p: TextureSettings) -> TextureSettings {
        // B33.1 (o papel em duas leituras) só no Wet Paint; os outros meios leem o papel do main (BUGS #115).
        p.lei.leitura_dupla = self.paint_media() == crate::PaintMedia::WetPaint;
        papel_a_escala(p, self.fator_aparencia)
    }
}

impl crate::BrushSettings {
    /// O papel como o motor o amostra neste documento (B39.6): o slot do Paper à escala do fator publicado — o que a
    /// prévia do cartão passa a [`ph2d_painter_brush::render_tiled_preview`].
    #[must_use]
    pub fn papel_como_o_motor_o_le(&self) -> TextureSettings {
        let p = TextureSettings {
            kind: ph2d_painter_brush::TextureKind::from_u8(self.paper_kind),
            angle_deg: self.paper_angle,
            offset: self.paper_offset,
            size: self.paper_size,
            params: self.paper_params,
            lei: ph2d_painter_brush::GrainLaw {
                leitura_dupla: self.wetpaint,
                ..ph2d_painter_brush::GrainLaw::NEUTRA
            },
            ..TextureSettings::default()
        };
        papel_a_escala(p, self.paper_fator.max(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_fator_e_o_lado_curto_sobre_1080_nunca_abaixo_de_1() {
        assert_eq!(do_tamanho(64, 64), 1.0);
        assert_eq!(do_tamanho(1920, 1080), 1.0);
        assert_eq!(do_tamanho(3840, 2160), 2.0);
        assert_eq!(do_tamanho(2160, 3840), 2.0, "retrato: o mesmo lado curto");
        assert!((do_tamanho(2560, 1440) - 4.0 / 3.0).abs() < 1e-6);
    }

    /// O fator nasce do tamanho no documento novo, fica com ELE ao trocar de imagem e atravessa o disco.
    #[test]
    fn o_fator_nasce_do_tamanho_e_viaja_com_o_documento() {
        let mut t = PainterTool::default();
        assert_eq!(t.fator_aparencia(), 1.0, "sem documento: o motor de sempre");
        // 2160 de lado curto num documento estreito (a RAM do teste fica em 34 MB).
        let (w, h) = (2160u32, 4000u32);
        t.bind_document(1, vec![255; (w * h * 4) as usize], w, h);
        assert_eq!(t.fator_aparencia(), 2.0, "documento novo grande");
        // Uma marca para o documento não ser descartável (o stash é que guarda o fator).
        let camada = t.layers.active().expect("uma camada");
        t.tintas_molhadas.insert(camada);
        t.bind_document(2, vec![255; 16 * 16 * 4], 16, 16);
        assert_eq!(t.fator_aparencia(), 1.0, "a outra imagem é pequena");
        t.bind_document(1, Vec::new(), w, h);
        assert_eq!(
            t.fator_aparencia(),
            2.0,
            "voltar devolve o fator do documento"
        );

        let doc = t.documento_vivo(5);
        assert_eq!(doc.fator_aparencia, 2.0);
        let bytes = postcard::to_allocvec(&doc).expect("grava");
        let lido: crate::tool::persist::PaintedDocument = postcard::from_bytes(&bytes).expect("lê");
        assert_eq!(lido.fator_aparencia, 2.0, "o fator atravessa o disco");
        let mut u = PainterTool::default();
        u.install_document(9, lido);
        u.bind_document(9, Vec::new(), w, h);
        assert_eq!(u.fator_aparencia(), 2.0, "e volta ao documento");
    }

    /// ⛔ BUGS_painter #115 (dono 07/10: só o Wet Paint evolui): fora do Wet Paint o papel lê-se numa leitura, a de
    /// `main` 4f46a3573c; as duas leituras do B33.1 só no Wet Paint — no motor e na prévia do cartão.
    #[test]
    fn fora_do_wet_paint_o_papel_e_a_leitura_unica_de_antes() {
        use crate::PaintMedia;
        for meio in [
            PaintMedia::Digital,
            PaintMedia::Watercolor,
            PaintMedia::Impasto,
            PaintMedia::WetPaint,
        ] {
            let mut t = PainterTool::default();
            t.set_paint_media(meio);
            assert_eq!(t.paint_media(), meio);
            let wet = meio == PaintMedia::WetPaint;
            assert_eq!(
                t.papel_na_escala(TextureSettings::default())
                    .lei
                    .leitura_dupla,
                wet,
                "{meio:?}: o motor"
            );
            assert_eq!(
                t.brush_settings()
                    .papel_como_o_motor_o_le()
                    .lei
                    .leitura_dupla,
                wet,
                "{meio:?}: a prévia"
            );
        }
    }
}
