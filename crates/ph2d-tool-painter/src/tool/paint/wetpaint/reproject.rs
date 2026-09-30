//! ⭐⭐ **A SESSÃO DA ÁGUA MUDA DE VISTA** — a metade do host da porta
//! [`ph2d_wet_paint::grid::reproject_grid`] (report do dono, 29/09: *«rotacionar
//! e pintar em seguida está pausando a simulação»*).
//!
//! A tela da vista 3D ([`super::super::super::screen_canvas`]) é o ECRÃ, e o
//! traço dado depois de rodar a peça tem de começar numa tela que mostra a
//! peça como ela se vê AGORA. Semear essa tela passava pelo `set_source`, que
//! mata a sessão (o guarda de identidade vê a tela trocada) ⇒ a água do traço
//! anterior parava. Aqui a sessão SOBREVIVE: a grade, a base congelada e a tela
//! são refeitas na vista nova, ponto da superfície a ponto da superfície.
//!
//! ⚠️ **Quem sabe onde cada píxel estava é quem chamou** (a escultura): o
//! Painter não sabe o que é uma malha, e continua a não saber. Ele recebe, píxel
//! a píxel da tela nova, a coordenada CONTÍNUA na tela de antes do mesmo ponto
//! da superfície — ou `None` onde a vista de antes não o via —, e o retrato da
//! peça SEM a água na vista nova, que passa a ser a base congelada inteira.
//!
//! ⭐⭐⭐ **A base é a semente, em todo píxel** (report do dono, 30/09, sobre a
//! cura de 29/09: *«melhorou mas não curou perfeitamente»*). A 1.ª cura levava
//! a base de ANTES debaixo da água (amostrada pelo píxel mais perto) e punha o
//! retrato fora dela; as duas metades eram tentativas de adivinhar, daqui, qual
//! é a peça sem a água. Quem sabe isso é quem chama — ele guarda a cor de antes
//! de cada amostra que a água tocou —, e o contrato passa a ser dele: a
//! `semente` já É a peça sem a água. Ela é desenhada na vista nova, logo não há
//! amostragem pelo píxel mais perto em lado nenhum.

use super::*;

impl PainterTool {
    /// **Refaz a sessão da água na vista nova.** `semente` é o retrato da peça
    /// SEM a água na vista nova (RGBA8, o tamanho da tela); `origem` é, por píxel da tela
    /// nova, o ponto na tela de ANTES que vê o mesmo sítio da superfície.
    ///
    /// Devolve `false` sem sessão viva, com um traço aberto, ou com tamanhos
    /// que não são os da tela — e aí nada mudou: quem chama semeia como antes.
    pub(crate) fn wetpaint_reproject(
        &mut self,
        semente: &[u8], // COLOR-RAW-OK: the new view's portrait bytes, same contract as `seed_screen_canvas`
        origem: &[Option<[f32; 2]>],
    ) -> bool {
        self.wetpaint_guard();
        let (wu, hu) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if wu == 0 || hu == 0 || semente.len() != wu * hu * 4 || origem.len() != wu * hu {
            return false;
        }
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return false;
        };
        if sess.stroke_open || sess.base.len() != wu * hu * 4 {
            return false;
        }
        sess.bring_home();
        // O píxel da tela de antes, ou `None` — a porta não confia em quem
        // chama para os limites (um `NaN` também cai aqui: `NaN >= 0` é falso).
        let pixel_de_antes = |p: usize| -> Option<usize> {
            let [x, y] = origem[p]?;
            if !(x >= 0.0 && y >= 0.0) {
                return None;
            }
            let (qx, qy) = (x as usize, y as usize);
            (qx < wu && qy < hu).then_some(qy * wu + qx)
        };
        // ⭐ A grade: a célula nova pergunta pelo píxel do CENTRO dela, e a
        // resposta volta a ser uma célula da grade de antes (a mesma razão).
        let r = usize::from(sess.ratio);
        let celula = |cx: i32, cy: i32| -> Option<(i32, i32)> {
            let px = ((cx - 1) as usize * r + r / 2).min(wu - 1);
            let py = ((cy - 1) as usize * r + r / 2).min(hu - 1);
            let q = pixel_de_antes(py * wu + px)?;
            Some(((q % wu / r) as i32 + 1, (q / wu / r) as i32 + 1))
        };
        for camada in &mut sess.engine.layers {
            ph2d_wet_paint::grid::reproject_grid(&mut camada.grid, celula);
        }
        // ⭐ A base congelada é a SEMENTE inteira — a peça sem a água, vista
        // daqui (ver o cabeçalho): o composite põe a água levada por cima dela.
        let veu = self.paint.wetpaint.show_wet;
        self.wetpaint_rebase(semente.to_vec(), veu);
        true
    }

    /// Troca a base congelada da sessão e refaz a tela inteira por cima dela —
    /// a mesma forma do nascimento da sessão (`ensure_wet_session`), com o
    /// guarda re-armado para a tela nova. `veu` é o do composite.
    fn wetpaint_rebase(&mut self, base: Vec<u8>, veu: bool) {
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        let base = Arc::new(base);
        sess.base = Arc::clone(&base);
        sess.preview = None;
        sess.lanes.clear();
        sess.engine.mark_dirty_full();
        self.replace_canvas(base);
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            sess.canvas = Arc::downgrade(&self.canvas_rgba);
        }
        self.wetpaint_composite_veiled(veu);
    }
}
