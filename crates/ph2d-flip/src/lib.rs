#![forbid(unsafe_code)]
//! `ph2d-flip` — o modelo de documento do **Flip**, o meio de animação desenhada
//! quadro-a-quadro do PH2D (o "Grease Pencil" do engine; ver `docs/Flip/` +
//! ADR-0114).
//!
//! Modelo **puro e serializável**, sem render nem ECS — testável headless. A
//! hierarquia:
//!
//! ```text
//! FlipDoc (cena)                 // uma por projeto; guardada no ProjectState
//!  └─ FlipObject (id estável)    // vira UMA entidade ECS (FlipObjectRef)
//!      ├─ FlipLayer (id estável) // blend/opacity/visible/lock/mask (idioma Painter)
//!      │   └─ frames: BTreeMap<Frame, FlipFrame>   // chave = quadro de início; hold implícito
//!      ├─ drawings: Vec<FlipDrawing>               // refcontado; DrawingId é o índice (posicional)
//!      └─ onion: OnionSettings                     // Ghost Frames
//! ```
//!
//! **Clean-room do Grease Pencil 5.2** (só comportamento; nunca código GPL). As
//! ops de frame (hold, end-sentinel, refcount, compactação+remap) espelham
//! `blenkernel/intern/grease_pencil.cc`; a referência comentada com `arquivo:linha`
//! está em `docs/Flip/02_referencia_algoritmos_blender_5.2.md`.
//!
//! Foundational-isolada (ADR-0107): crate própria, pontos de extensão por módulo
//! irmão. Deps só de `ph2d-core` (Vec2), `ph2d-painter-effects` (BlendMode
//! canônico de 22 modos), serde e postcard.

mod autokey;
mod color;
mod cycle;
mod doc;
mod drawing;
mod expose;
mod frame;
mod ids;
mod layer;
mod layer_time;
mod object;
mod onion;
mod pose;
mod segment;
mod stroke;
mod tween;
mod tween_flip;
mod tween_match;
mod tween_phase;
mod tween_spiral;

pub use autokey::AutokeyPolicy;
pub use color::Rgba;
pub use cycle::{CycleMode, LayerCycle, map_frame};
pub use doc::{FlipDoc, SceneSample};
pub use drawing::FlipDrawing;
pub use frame::{FlipFrame, Hold, KeyKind};
pub use ids::{DrawingId, FlipObjectId, Frame, LayerId, MaterialId};
pub use layer::{FlipLayer, LayerMask};
pub use object::{DEFAULT_FPS, DupMode, FlipObject};
pub use onion::{GHOST_MIN_ALPHA, Ghost, OnionMode, OnionSettings, ghosts};
pub use pose::Pose;
pub use segment::{Cutter, cuts, piece_of_point, probe_point};
pub use stroke::{
    Cap, DEFAULT_DOT_SPACING, DEFAULT_HARDNESS, DEFAULT_OPACITY, DEFAULT_WIDTH, Fill, FlipStroke,
    Point, StrokeTip,
};
pub use tween::{TweenOptions, TweenRequest, tween_drawing};
pub use tween_match::{PAIR_REJECT_COST, StrokeFeatures, TweenPlan, features};

/// A curva de easing do tween — o MESMO enum que a timeline/graph editor usam
/// (`ph2d-anim`), re-exportado para o painel/shell não dependerem da crate de
/// animação só por causa de um preset.
pub use ph2d_anim::Interp;

/// Os presets nomeados da mesma curva — o shell monta `Interp::Eased(..)` a partir do
/// chip de easing da barra sem depender do `ph2d-anim` por causa de um enum.
pub use ph2d_anim::{Easing, EasingFamily, EasingMode};

/// O compositor de blend do Painter, re-exportado para os consumidores do Flip
/// (render/painel) casarem o tipo sem depender de `ph2d-painter-effects`
/// diretamente. É o MESMO `BlendMode` guardado em [`FlipLayer::blend`].
pub use ph2d_painter_effects::BlendMode;

/// Versão do schema de serialização da [`FlipDoc`]. Bump ⇒ migração ou hard-break
/// (postcard é posicional — mudança de campo quebra saves antigos, HR-14).
///
/// v2 (W3): a camada ganhou `cycle` (pre/post behavior) + `use_onion`, e o
/// `OnionSettings` ganhou `kind_filter`.
/// v3 (W4): o traço ganhou `holes` (os buracos do preenchimento) + `hide_stroke`.
/// v4 (W6): o traço ganhou `selected` — o atributo `.selection` do GP no domínio
/// **Curve** (o Edit Mode). É atributo de documento (e não estado do shell) porque a
/// identidade de um traço é a posição dele na lista, e o balde insere no MEIO dela.
/// v5 (W7.2): a CHAVE ganhou `offset` — a **pose do quadro**. É o que faz uma
/// instância (duas chaves, um desenho) ser mais do que um hold: a arte é
/// compartilhada e o LUGAR é de cada quadro.
/// v6 (W7.5): a pose virou AFIM (`Pose([f32; 6])`, era `Vec2`) — rotate/escala por chave.
/// v7 (W8): o traço ganhou `point_sel` — o `.selection` do GP no domínio **Point**
/// (vetor paralelo aos pontos; vazio = a seleção vive no Curve).
/// v8 (§4.C.6): **a UNIDADE do `Point.width` mudou** — era px de TELA (brush absoluto,
/// 2026-07-11), virou unidade de MUNDO (`size_to_world`, `SIZE_PX_PER_WORLD = 100`).
/// ⚠️ O layout NÃO mudou (segue um `f32`), e é justamente por isso que este bump é
/// obrigatório: sem ele o postcard lê um `width: 6.0` antigo **com sucesso** e o
/// interpreta como 6 unidades de mundo — ~100× a espessura pretendida, arte destruída
/// em SILÊNCIO. Um quebra de layout falha alto; uma troca de unidade, não.
/// v9 (03 §8): o traço ganhou `tip` ([`StrokeTip`]) + `dot_spacing` — o pincel pontilhado.
/// Postcard é POSICIONAL, e os campos entraram no MEIO do `FlipStroke` (após `hardness`) ⇒
/// todo `FlipStroke` salvo em v8 lê os campos seguintes deslocados ⇒ bump obrigatório.
/// v10 (2.5D multiplane, ADR-0114 §Decisão 3): a `FlipLayer` ganhou `depth` (a fração de
/// paralaxe da câmera). Campo apendado ⇒ um doc v9 lê `depth` sobre o fim do buffer ⇒ bump.
/// v11 (Self Overlap, 03 §8): o traço ganhou `self_overlap` (auto-sobreposição com acúmulo).
/// Postcard é POSICIONAL, e o campo entrou no MEIO do `FlipStroke` (após `dot_spacing`) ⇒ todo
/// `FlipStroke` salvo em v10 lê os campos seguintes deslocados ⇒ bump obrigatório.
pub const FLIP_SCHEMA_VERSION: u32 = 11;

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::{Playhead, Vec2};

    /// Constrói um documento rico (2 camadas, um par de desenhos-chave com
    /// traços e cores), "anima" o playhead e assere o desenho amostrado — depois
    /// prova o round-trip serde. É o teste de integração headless do W0.
    #[test]
    fn build_animate_sample_and_round_trip() {
        let mut doc = FlipDoc::new();
        let oid = doc.push_object("Character");
        let (bg, fg, d_bg, d_fg0, d_fg6);
        {
            let obj = doc.object_mut(oid).unwrap();
            obj.fps = 24.0;
            // Camada de fundo: um único desenho segurando a animação inteira.
            bg = obj.add_layer("BG");
            d_bg = obj
                .insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe)
                .unwrap();
            {
                let mut s = FlipStroke::new();
                s.push_default(Vec2::new(0.0, 0.0));
                s.push_default(Vec2::new(10.0, 0.0));
                s.closed = true;
                s.fill = Some(Fill {
                    color: Rgba::new(0.2, 0.2, 0.9, 1.0),
                    opacity: 1.0,
                });
                obj.drawing_mut(d_bg).unwrap().strokes.push(s);
            }
            // Camada de frente: dois desenhos-chave (quadros 0 e 6).
            fg = obj.add_layer("FG");
            d_fg0 = obj
                .insert_frame(fg, 0, Hold::Implicit, KeyKind::Keyframe)
                .unwrap();
            d_fg6 = obj
                .insert_frame(fg, 6, Hold::Implicit, KeyKind::Keyframe)
                .unwrap();
            {
                let mut s = FlipStroke::new();
                s.push_point(Point {
                    pos: Vec2::new(1.0, 1.0),
                    width: 2.0,
                    opacity: 1.0,
                    color: Rgba::BLACK,
                });
                obj.drawing_mut(d_fg6).unwrap().strokes.push(s);
            }
        }

        // Amostra em t=0: BG=d_bg, FG=d_fg0.
        let mut ph = Playhead::new(1.0 / 24.0);
        ph.seek(0.0);
        let s0 = doc.sample(&ph);
        assert_eq!(s0.len(), 1, "um objeto");
        assert_eq!(s0[0].0, oid);
        assert_eq!(
            s0[0].1,
            vec![(bg, Some(d_bg)), (fg, Some(d_fg0))],
            "t=0: BG e FG(0)"
        );

        // Avança para t=0.25s = quadro 6: FG troca para d_fg6, BG segura.
        ph.seek(0.25);
        let s6 = doc.sample(&ph);
        assert_eq!(
            s6[0].1,
            vec![(bg, Some(d_bg)), (fg, Some(d_fg6))],
            "t=0.25: FG virou o 2º desenho, BG holdou"
        );

        // Round-trip serde: a cena inteira volta idêntica.
        let bytes = doc.to_bytes().unwrap();
        let back = FlipDoc::from_bytes(&bytes).unwrap();
        assert_eq!(back, doc, "round-trip postcard preserva o documento");

        // Rejeita schema alheio.
        let mut bad = postcard::to_allocvec(&(FLIP_SCHEMA_VERSION + 1, &doc)).unwrap();
        assert!(FlipDoc::from_bytes(&bad).is_err());
        bad.clear();
    }
}
