//! **Smoke das Buffer Curves** (joia da coroa §5). `PH2D_BUFFER_SMOKE=1`:
//!
//! UM objeto com uma track de X de curva reconhecível (uma ondinha de 5 keys),
//! timeline ABERTA e pausada. As Buffer Curves são o A/B da curva do Unreal:
//! guarda-se a curva atual (Store), edita-se, e volta-se à guardada (Swap) — com
//! a versão guardada desenhada como um FANTASMA por baixo enquanto você edita.
//!
//! Os botões **Store**/**Swap** vivem no canto superior-direito da BANDA do graph
//! editor, então a track precisa estar EXPANDIDA: clique no ▸ (twirl) na label da
//! track para abrir o graph. Store aparece sempre; Swap só aparece na track que
//! detém o buffer (a mesma condição que desenha o fantasma).
//!
//! O que provar na tela:
//! 1. Expanda a track (twirl ▸) — a banda do graph abre; **Store** no canto sup-dir.
//! 2. Clique **Store** — nada muda ainda, mas agora existe um buffer (surge **Swap**).
//! 3. Arraste uma âncora da curva (mude a forma) — o **fantasma** cinza da curva
//!    guardada aparece por baixo da curva viva.
//! 4. Clique **Swap** — a curva viva vira a guardada e o fantasma vira a editada
//!    (o A/B). Clique **Swap** de novo — volta. Um Ctrl+Z desfaz o swap.
//! 5. Nenhum strip/fade se mexe (Buffer Curves é edit de KEY, via edit/settle).
//!
//! ⚠️ Se a linha `[buffer-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, TimelineDoc};

/// A distinctive 5-key wiggle so Store -> edit -> Swap is visible: any anchor you
/// drag changes the shape, and the ghost of THIS curve stays put underneath.
fn author_wiggle(doc: &mut TimelineDoc, bits: u64) {
    let s = RationalTime::from_seconds;
    for (t, v) in [
        (0.0_f64, 0.0_f32),
        (0.5, 4.0),
        (1.0, -2.0),
        (1.5, 3.0),
        (2.0, 0.0),
    ] {
        doc.insert_key(
            bits,
            PropKind::TranslationX,
            s(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
}

impl crate::App {
    /// In the frame prologue, once. No-op without the env.
    pub(crate) fn buffer_smoke(&mut self) {
        if self.buffer_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_BUFFER_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // no world yet; try next frame
        }
        self.buffer_smoke_done = true;

        let bits = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, 0.0)),
                    Sprite::atlas(0, [1.4, 0.4], [0.4, 0.7, 1.0, 1.0]),
                    Name::new("Wiggle"),
                ))
                .id()
                .to_bits()
        };
        author_wiggle(&mut self.timeline.doc, bits);

        // Open the timeline (the chips live in the graph band) and park at 0, paused —
        // Buffer Curves is authoring, not playback. The track opens COLLAPSED: the
        // artist expands it with the twirl (the panel owns that view state).
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(bits));
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "[buffer-smoke] 1 track (X, ondinha de 5 keys), timeline aberta e pausada. \
             EXPANDA a track (twirl na label) para ver a banda do graph; STORE fica no \
             canto sup-dir. Store -> arraste uma ancora (surge o FANTASMA cinza da curva \
             guardada) -> SWAP faz o A/B. Nenhum strip/fade se mexe."
        );
    }
}
