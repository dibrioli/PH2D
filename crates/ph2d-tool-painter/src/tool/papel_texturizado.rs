//! **O «Textured Paper» do New Image** (dono 30/10): a imagem nasce com o cartão Paper do dono
//! (foto `docs/Painter/Pinceis/pranchas/J9_smoke_dono_papel_texturizado.png`).

use super::{PainterTool, RasterEditTool};
use ph2d_painter_brush::TextureKind;

/// O cartão Paper de uma imagem nascida «Textured Paper».
pub struct PapelPreset {
    /// A Color do papel (sRGB8).
    pub cor: [u8; 3],
    pub kind: TextureKind,
    pub relief: f32,
    pub roughness: f32,
    pub angle_deg: f32,
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub tooth: f32,
    /// Os params do padrão (Cold Press: Contrast, Brightness).
    pub params: [f32; 2],
}

/// Os valores do dono (30/10): Cold Press bege `#E6D9CC`.
pub const PAPEL_TEXTURIZADO: PapelPreset = PapelPreset {
    cor: [0xE6, 0xD9, 0xCC], // LITERAL-COLOR-OK: a cor do papel escolhida pelo dono (dado, não UI)
    kind: TextureKind::PaperCold,
    relief: 0.5,
    roughness: 0.5,
    angle_deg: 0.0,
    offset: [0.0, 0.0],
    size: [1.0, 1.0],
    tooth: 1.0,
    params: [0.5, 0.5],
};

impl PainterTool {
    /// ⭐ **Os píxeis com que a SPRITE de uma imagem «Textured Paper» `w`×`h` nasce** — o assado
    /// (`run_full`) do documento que o Draw abre para ela. O Object mostra a sprite: nascida branca, o
    /// papel só se via no Draw (BUG dono 31/10, `docs/Painter/Arquitetura/BUG_papel_so_no_draw.md`).
    /// Bytes crus de propósito: saem do `run_full` do contrato congelado `RasterEditTool` e entram
    /// verbatim no `set_source`/`bind_document` e no `spawn_rgba` — todos `Vec<u8>`.
    #[must_use]
    pub fn pixeis_do_papel_texturizado(w: u32, h: u32) -> Vec<u8> /* COLOR-RAW-OK: plano RGBA8 cru da sprite */
    {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; (w as usize) * (h as usize) * 4], w, h);
        t.aplica_papel_preset(&PAPEL_TEXTURIZADO);
        let (px, _, _) = (&mut t as &mut dyn RasterEditTool).run_full();
        nascida::lembra(w, h, &px);
        px
    }

    /// **Abre a sprite de um documento novo** — e reconhece a nascida «Textured Paper» que ninguém pintou
    /// ([`Self::pixeis_do_papel_texturizado`]): ela abre como o documento de que é o assado (fundo vazio
    /// + o papel do dono), nunca com o dente assado na camada e outro por cima.
    pub(crate) fn abre_a_sprite_nova(
        &mut self,
        entity: u64,
        mut rgba: Vec<u8>, // COLOR-RAW-OK: a fonte crua do `set_source`, entregue ao `abre_a_sprite`
        w: u32,
        h: u32,
    ) {
        if nascida::e_ela(&rgba, w, h) {
            rgba.fill(255);
            self.papel_texturizado_para = Some(entity);
        }
        self.abre_a_sprite(rgba, w, h);
        self.serve_papel_texturizado(entity);
    }

    /// A imagem `entity` acabou de nascer pelo *New Image…* com «Textured Paper»: quando ela vira
    /// documento, o papel dela é o [`PAPEL_TEXTURIZADO`].
    pub fn agenda_papel_texturizado(&mut self, entity: u64) {
        self.macro_recusa("agenda_papel_texturizado");
        self.papel_texturizado_para = Some(entity);
    }

    /// Chamada quando `entity` abre como documento novo: serve o papel agendado, se é dela.
    pub(crate) fn serve_papel_texturizado(&mut self, entity: u64) {
        if self.papel_texturizado_para != Some(entity) || self.papel.is_none() {
            return;
        }
        self.papel_texturizado_para = None;
        self.aplica_papel_preset(&PAPEL_TEXTURIZADO);
    }

    /// Escreve o cartão Paper inteiro pelas portas dos controlos dele.
    fn aplica_papel_preset(&mut self, p: &PapelPreset) {
        let [r, g, b] = p.cor;
        self.set_paper_color_rgb8(r, g, b);
        self.papel = Some(p.cor);
        self.set_brush_paper_kind(p.kind.to_u8());
        self.set_substrate_depth(p.relief);
        self.set_substrate_roughness(p.roughness);
        self.set_brush_paper_angle(p.angle_deg);
        for axis in 0..2 {
            self.set_brush_paper_offset(axis, p.offset[axis]);
            self.set_brush_paper_size(axis, p.size[axis]);
        }
        self.set_brush_paper_depth(p.tooth);
        for (slot, v) in p.params.iter().enumerate() {
            self.set_brush_paper_param(slot, *v);
        }
        self.invalidate_composite();
    }
}

/// O reconhecimento da sprite nascida: o hash do assado por tamanho (um documento novo de outro tamanho
/// nunca o casa; o filtro barato — opaca e o 1.º píxel do assado — poupa o hash às outras imagens).
mod nascida {
    use std::collections::BTreeMap;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::sync::Mutex;

    /// (largura, altura) → (o papel, o hash dos píxeis).
    type Assados = BTreeMap<(u32, u32), ([u8; 4], u64)>;
    static ASSADOS: Mutex<Assados> = Mutex::new(BTreeMap::new());

    fn hash(px: &[u8]) -> u64 {
        let mut h = DefaultHasher::new();
        px.hash(&mut h);
        h.finish()
    }

    pub(super) fn lembra(w: u32, h: u32, px: &[u8]) {
        let Some(p0) = px.first_chunk::<4>() else {
            return;
        };
        if let Ok(mut m) = ASSADOS.lock() {
            m.insert((w, h), (*p0, hash(px)));
        }
    }

    pub(super) fn e_ela(rgba: &[u8], w: u32, h: u32) -> bool {
        let Some(p0) = rgba.first_chunk::<4>() else {
            return false;
        };
        // Branco puro é o fundo de sempre; o papel do dono nunca é branco.
        if p0[3] != u8::MAX || p0[..3] == [u8::MAX; 3] {
            return false;
        }
        let memo = ASSADOS.lock().ok().and_then(|m| m.get(&(w, h)).copied());
        let (q0, hq) = match memo {
            Some(v) => v,
            // Sem memória deste tamanho (outra sessão): só assa para comparar se o 1.º píxel é do tom do papel.
            None if perto_do_papel(p0) => {
                let px = super::PainterTool::pixeis_do_papel_texturizado(w, h);
                (*px.first_chunk::<4>().unwrap_or(&[0; 4]), hash(&px))
            }
            None => return false,
        };
        *p0 == q0 && hash(rgba) == hq
    }

    fn perto_do_papel(p: &[u8; 4]) -> bool {
        const FOLGA: u8 = 48;
        p.iter()
            .zip(super::PAPEL_TEXTURIZADO.cor)
            .all(|(a, b)| a.abs_diff(b) <= FOLGA)
    }
}
