//! ⭐⭐⭐ **O PAINTER PINTA A PEÇA** — a costura entre o módulo Painter e a
//! escultura (ordem do dono, 2026-09-24: *«a integração total do módulo Painter
//! já existente para que consiga pintar com os mesmos features na malha 3d»*).
//!
//! # A corrente, e quem é dono de cada elo
//!
//! 1. **O Painter pinta uma tela do tamanho da vista** — o motor dele inteiro,
//!    sem uma linha nova (`ph2d_tool_painter::PainterTool::bind_screen_canvas`).
//! 2. **A ponta de cada gesto** chega aqui por [`entrega`], em coordenadas de
//!    janela, e sai para o Painter em coordenadas da tela (a vista activa).
//! 3. **A cada quadro e no fim do traço**, o que a tela mudou é pousado na peça
//!    ([`quadro`], [`ph2d_sculpt3d::tela_na_malha`]) — só onde a superfície está à
//!    vista, com a máscara, na tinta fina quando ela está armada.
//! 4. **O traço fecha pela porta de sempre** (`close_stroke`), logo o `Ctrl+Z`
//!    desfaz uma pincelada do Painter como desfaz uma do pincel de pintura.
//!
//! # ⚠️ A tela limpa-se a cada traço
//!
//! A peça guarda a tinta; a tela é só o traço em voo. Deixá-la cheia faria o
//! traço seguinte compor o anterior outra vez por cima dele.
//!
//! # ⚠️ Os limites desta primeira etapa, ditos
//!
//! Pinta-se o lado que se VÊ (o de trás espera que o artista rode a peça), e a
//! tela começa TRANSPARENTE — os modos que misturam com o que já está pintado
//! (borrar, esfregar, multiplicar) precisam da cor da peça na tela e são a
//! etapa seguinte.

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_sculpt3d::tela_na_malha::{Tela, TelaNaMalha, Vista};
use ph2d_tool_painter::{PainterTool, ScreenCanvasFrame};

use crate::Sculpt3dScene;

/// ⭐ **Um quadro da costura** — prende (ou solta) a tela do Painter conforme o
/// barro está ou não no ecrã, e pousa na peça o que a tela mudou.
///
/// `painter` é `None` quando a ferramenta em mãos não é o Painter: aí um traço
/// aberto fecha, porque ninguém o vai terminar.
pub fn quadro(scene: Option<&mut Sculpt3dScene>, painter: Option<&mut PainterTool>) {
    let Some(painter) = painter else {
        if let Some(s) = scene {
            s.painter_fecha();
            s.painter_raio_px = None;
        }
        return;
    };
    match scene {
        Some(s) if s.tela_do_painter().is_some() => {
            // Pousa ANTES de redimensionar: a tinta em voo é da tela de antes.
            if let Some(f) = painter.take_screen_canvas() {
                s.painter_pousa(&f);
            }
            if let Some((w, h)) = s.tela_do_painter() {
                painter.bind_screen_canvas(w, h);
            }
            s.painter_raio_px = Some(painter.dab_footprint_px());
        }
        Some(s) => {
            s.painter_fecha();
            s.painter_raio_px = None;
            painter.release_screen_canvas();
        }
        None => painter.release_screen_canvas(),
    }
}

/// ⭐ **Um ponto do gesto**, em coordenadas de JANELA. Devolve se o Painter o
/// consumiu (para o gesto não descer ao pan nem à selecção).
///
/// ⚠️ Só arma com a tela presa ([`PainterTool::on_screen_canvas`]); fora disso
/// quem responde é a ponte da sprite.
pub fn entrega(
    scene: &mut Sculpt3dScene,
    painter: &mut PainterTool,
    x: f32,
    y: f32,
    pressure: f32,
    phase: PointerPhase,
) -> bool {
    if !painter.on_screen_canvas() {
        return false;
    }
    let (vx, vy) = scene.to_view(x, y);
    if phase == PointerPhase::Down {
        let (w, h) = scene.viewport();
        if vx < 0.0 || vy < 0.0 || vx >= w as f32 || vy >= h as f32 {
            return false;
        }
        if !scene.painter_abre(x, y) {
            return false;
        }
        // ⭐⭐ Os modos que lêem a cor debaixo do pincel começam com o RETRATO
        // da peça na tela, e a lei passa a ser a diferença (etapa 2).
        if painter.screen_canvas_reads_the_piece() {
            scene.painter_semeia(painter);
        }
    }
    let consumed = painter.on_canvas_pointer(CanvasPointer {
        pos: [vx, vy],
        pressure,
        tilt: [0.0, 0.0],
        phase,
    });
    if phase == PointerPhase::Up {
        if let Some(f) = painter.take_screen_canvas() {
            scene.painter_pousa(&f);
        }
        scene.painter_fecha();
        painter.clear_screen_canvas();
    }
    consumed
}

impl Sculpt3dScene {
    /// O tamanho da tela que o Painter deve segurar — a vista activa —, ou
    /// `None` quando não há barro no ecrã a receber tinta.
    pub(crate) fn tela_do_painter(&self) -> Option<(u32, u32)> {
        (self.clay_on_screen() && self.obj().is_some()).then(|| self.viewport())
    }

    /// A vista da peça ACTIVA, de espaço local para píxeis da tela.
    fn vista_do_painter(&self) -> Option<Vista> {
        let o = self.obj()?;
        let (w, h) = self.viewport();
        // ⚠️ A pose é translação + escala UNIFORME (`ph2d_mesh::Pose`), logo
        // `VP · modelo` fecha-se coluna a coluna sem uma matriz de modelo.
        let s = o.pose.scale();
        let t = o.pose.translation;
        let vp = self
            .camera
            .view_proj(w as f32 / h.max(1) as f32)
            .to_cols_array();
        let mut m = [0.0f32; 16];
        for r in 0..4 {
            for c in 0..3 {
                m[4 * c + r] = vp[4 * c + r] * s;
            }
            m[12 + r] = vp[r] * t[0] + vp[4 + r] * t[1] + vp[8 + r] * t[2] + vp[12 + r];
        }
        let olho = o.pose.point_to_local(self.camera.eye().into());
        Some(Vista::nova(m, (w, h), olho))
    }

    /// **Abre a pincelada do Painter na peça** — a mesma abertura de um traço de
    /// pintura: mira a peça sob o cursor, congela o traço e empresta o plano de
    /// tinta fina. Devolve `false` sem peça nenhuma.
    pub(crate) fn painter_abre(&mut self, x: f32, y: f32) -> bool {
        self.painter_fecha();
        // ⚠️ Errar a peça NÃO recusa o traço: o pincel pinta a tela, e a tinta
        // aterra onde a peça estiver quando o traço lá chegar (a lei do pincel
        // de cor desta casa, 21/09). A mira só muda a peça activa quando acerta.
        self.aim(x, y);
        // A vista ANTES do empréstimo: um `return` depois dele deixaria o plano
        // preso num traço que ninguém fecha.
        let Some(vista) = self.vista_do_painter() else {
            return false;
        };
        self.stroke.begin(self.objects[self.active].stack.mesh());
        let dono = self.objects[self.active].id;
        self.stroke.tinta_fina =
            crate::tinta_da_peca::empresta(&mut self.objects[self.active].tinta, dono);
        let mesh = self.objects[self.active].stack.mesh();
        let destino = self
            .stroke
            .tinta_fina
            .as_ref()
            .map_or(mesh.vert_count(), |t| t.tinta().amostras().len());
        self.painter_tela = Some(TelaNaMalha::nova(mesh, vista, destino));
        true
    }

    /// ⭐⭐ **A tela começa com o retrato da peça** — os MESMOS bytes vão para
    /// o Painter e para a sessão, senão o que o pincel não tocou deixa de se
    /// anular na diferença.
    ///
    /// ⚠️ **E a drenagem do retrato deita-se FORA:** semear a tela marca-a
    /// inteira como mudada, e o retrato não é uma mudança — pousá-lo varreria a
    /// peça inteira no quadro seguinte para concluir que nada mudou.
    pub(crate) fn painter_semeia(&mut self, painter: &mut PainterTool) {
        let Some(sessao) = self.painter_tela.as_mut() else {
            return;
        };
        let tinta = self.stroke.tinta_fina.as_ref().map(|t| t.tinta());
        let mesh = self.objects[self.active].stack.mesh();
        let retrato = ph2d_sculpt3d::tela_semente::semente(mesh, tinta, sessao.vista());
        if painter.seed_screen_canvas(retrato.clone()) {
            let _ = painter.take_screen_canvas();
            sessao.com_semente(retrato);
        }
    }

    /// **Pousa na peça o que a tela mudou.**
    pub(crate) fn painter_pousa(&mut self, f: &ScreenCanvasFrame) {
        let Some(sessao) = self.painter_tela.as_mut() else {
            return;
        };
        let rect = f.rect.map_or([0, 0, f.w, f.h], |(x, y, w, h)| [x, y, w, h]);
        let tela = Tela {
            rgba: &f.rgba,
            largura: f.w,
            altura: f.h,
        };
        let active = self.active;
        let Some(o) = self.objects.get_mut(active) else {
            return;
        };
        let (vertices, _) = self
            .stroke
            .pousa_a_tela(o.stack.mesh_mut(), sessao, &tela, rect);
        if !vertices.is_empty() {
            Self::mesh_changed(&mut o.dirty, &mut self.edits, &vertices);
        }
    }

    /// **Fecha a pincelada** — o desfazer é gravado pela porta de todo traço.
    pub(crate) fn painter_fecha(&mut self) {
        if self.painter_tela.take().is_some() {
            self.close_stroke();
        }
    }
}
