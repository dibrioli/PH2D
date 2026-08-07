//! **O FILL no Impasto: além da cor, o CORPO — e a borda da seleção com o perfil do Falloff**
//! (Enio, 2026-08-07: *"no modo Impasto vamos implementar o Fill exatamente como já funciona para o
//! digital só que além da cor ele preenche com relevo e as bordas da seleção devem ficar como o Falloff
//! que está selecionado no painter"*).
//!
//! ## As duas metades, e por que são duas
//!
//! **A COR** já funcionava: os dois fills (o balde e o *Color Fill* da seleção) compõem a cor do pincel
//! pela cobertura da seleção. O que faltava era o **CORPO** — no meio cuja razão de existir é a tinta ter
//! espessura, encher uma região deixava um decalque perfeitamente plano.
//!
//! **A BORDA** é a outra metade, e é onde o Falloff entra. Uma seleção tem cobertura *binária* (ou a
//! rampa linear do Feather), então o corpo depositado por ela seria um platô com um penhasco na
//! fronteira — a luz lê `∇h`, e um penhasco lê como recorte de papel, não como tinta. O Falloff é
//! exatamente o perfil que este app já usa para *como a tinta termina*, e o artista o tem escolhido na
//! tela ao lado. Então a rampa não é um knob novo: ela é **o perfil que já está em mãos**, pousado na
//! distância de cada texel até a borda da seleção.
//!
//! ## As três decisões
//!
//! 1. **A régua da rampa é o RAIO do pincel.** O Falloff *é* um perfil sobre o raio — o preview do card o
//!    desenha assim —, então medir a rampa em qualquer outro número obrigaria dois controles a
//!    concordar. Um fill com pincel grande tem borda larga; com pincel pequeno, borda estreita.
//! 2. **Só a borda da SELEÇÃO ganha o perfil.** No balde, a fronteira do próprio flood é a fronteira da
//!    ARTE (onde a cor mudou), e amaciá-la seria a ferramenta desfazendo o contorno contra o qual o
//!    artista despejou. O que ganha rampa é o que a seleção recorta.
//! 3. **Digital fica BYTE-IDÊNTICO.** Sem impasto no pincel, [`PainterTool::fill_selection_keep`]
//!    devolve a máscara verbatim (um `Arc` clonado, custo zero) e nenhum plano de relevo é tocado.
//!
//! ## Por que isto é pequeno
//!
//! Nada aqui carimba um dab. O commit do relevo (`impasto_live::commit_stroke_height`) já recebe uma
//! região através de CINCO planos — `paint` / `grain` / `film` / `radius` + a bbox — e faz o resto:
//! deriva a altura, funde a cobertura por `max`, compõe o MATERIAL por `over`, assenta, empurra, guarda
//! os ingredientes para o card Body seguir vivo e entrega a janela ao undo. Um fill é **um dab do
//! tamanho da região**, e escrever aqueles cinco planos é dizer isso ao motor que já sabe o resto.

use super::PainterTool;
use super::Region;
use ph2d_painter_brush::height::{NO_GRAIN, derive_height};
use std::sync::Arc;

/// ⚠️ **A pergunta é `deposits_height()`, NÃO `impasto_applies()`.** A segunda pergunta *"o card Body
/// se aplica a este MODO?"* e exige `PaintMode::Paint` — e um fill nunca está em `Paint` (ele roda em
/// `Fill` ou em `Selection`), então usá-la deixaria o corpo do fill morto em todo caminho, com a suíte
/// inteira verde. A pergunta de um fill é sobre o PINCEL: *este pincel deposita corpo?* — que é também
/// o que dá de graça o `Depth = 0` e o `DrawTo::Color` como no-ops.
///
/// O limiar de "estou dentro da região" para a distância à borda. Baixo de propósito: a cauda macia de
/// uma seleção com Feather é PARTE da região, e um limiar alto a arrancaria antes de o perfil pousar.
const INSIDE_THRESHOLD: u8 = 8;

impl PainterTool {
    /// **A cobertura que a seleção concede a um FILL.**
    ///
    /// Sem impasto (ou sem seleção) é a máscara verbatim — o mesmo `Arc`, sem cópia, e o digital não se
    /// move um byte. Com impasto, a borda da seleção passa a ter o perfil do Falloff em mãos, medido
    /// sobre o RAIO do pincel a partir da fronteira para dentro.
    #[must_use]
    pub(super) fn fill_selection_keep(&self) -> Arc<Vec<u8>> {
        let mask = Arc::clone(&self.paint.selection_mask);
        if !self.paint.brush.deposits_height() || !self.paint.selection_active {
            return mask;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || mask.len() != w * h {
            return mask;
        }
        let ramp = self.paint.brush.clamped_radius();
        if ramp <= 0.5 {
            return mask; // um pincel de meio texel não tem perfil que caiba numa borda
        }
        let full = Region {
            x: 0,
            y: 0,
            w: self.source_size.0,
            h: self.source_size.1,
        };
        let dist = super::sculpt_close::distance_inside(&mask, w, full, INSIDE_THRESHOLD);
        if dist.len() != w * h {
            return mask;
        }
        // ⚠️ **`falloff_weight` recebe `t = distância / raio` a partir do CENTRO do dab** (0 no miolo, 1
        // no aro) — não a profundidade. A grandeza que temos é a distância à BORDA, que é o complemento
        // dela, então a conversão acontece aqui, na entrada, e uma vez. Passar a profundidade crua
        // inverte o perfil inteiro: o miolo sai transparente e o aro sai sólido — foi exatamente o que a
        // primeira versão fez, e o fill não pintava nada.
        //
        // E a `hardness` entra junto porque a porta a compõe: ela é a outra metade de *como esta tinta
        // termina*, e um pincel de borda dura (`hardness >= 1`) devolve um disco — a borda da seleção
        // fica reta, que é o que aquele pincel significa.
        let out: Vec<u8> = dist
            .iter()
            .zip(mask.iter())
            .map(|(&d, &m)| {
                if m <= INSIDE_THRESHOLD {
                    return 0;
                }
                let t = 1.0 - (d / ramp).clamp(0.0, 1.0);
                let wgt = self.paint.brush.falloff_weight(t).clamp(0.0, 1.0);
                // ⚠️ O perfil MULTIPLICA a máscara, nunca a substitui: uma seleção com Feather já
                // carrega uma cauda autorada, e trocá-la pelo perfil apagaria o que o artista pediu no
                // slider. Dentro, longe da borda, `wgt = 1` e o produto devolve a máscara intacta.
                (f32::from(m) * wgt).round().clamp(0.0, 255.0) as u8
            })
            .collect();
        Arc::new(out)
    }

    /// **Deposita CORPO na região que `cov` cobre** — escreve os cinco planos que um traço escreve e
    /// deixa o commit do impasto fazer o resto.
    ///
    /// ⚠️ **O raio é o do PINCEL, o mesmo que mediu a rampa.** A altura de um depósito escala com o raio
    /// do dab que o fez (`IMPASTO_REFERENCE_RADIUS_PX`), então um fill com pincel grande é um empaste
    /// grosso de borda larga e um com pincel pequeno é um filete — os dois números saem do mesmo lugar,
    /// que é o que impede a borda e a espessura de discordarem sobre "que pincel foi este".
    ///
    /// No-op sem impasto no pincel: um fill digital não tem corpo a depositar.
    pub(super) fn deposit_fill_body(&mut self, cov: &[u8]) {
        if self.arm_fill_body(cov) {
            self.commit_stroke_height();
            self.sync_relief_flags();
        }
    }

    /// **ARMA o corpo do fill sem commitar** — escreve os cinco planos VIVOS e a bbox, e para aí.
    ///
    /// As duas metades existem porque os dois fills têm ciclos de vida diferentes: o *Color Fill* é um
    /// disparo só (arma e entrega no mesmo gesto), e o BALDE re-deriva a cada tique do slider de
    /// tolerância e só fecha no Done — ali o commit é o `close_stroke`, que já sabe fazê-lo. Um único
    /// `deposit` obrigaria o balde a fundir na camada uma vez por tique de slider.
    ///
    /// `true` quando algo foi armado.
    pub(super) fn arm_fill_body(&mut self, cov: &[u8]) -> bool {
        if !self.paint.brush.deposits_height() {
            return false;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let n = w * h;
        if n == 0 || cov.len() != n {
            return false;
        }
        let radius = self.paint.brush.clamped_radius();
        let spec = self.paint.brush;
        let mut paint = vec![0.0f32; n];
        let mut film = vec![0u8; n];
        let mut rad = vec![0.0f32; n];
        let (mut minx, mut miny, mut maxx, mut maxy) = (usize::MAX, usize::MAX, 0usize, 0usize);
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let c = cov[i];
                if c == 0 {
                    continue;
                }
                paint[i] = f32::from(c) / 255.0;
                film[i] = c;
                rad[i] = radius;
                minx = minx.min(x);
                miny = miny.min(y);
                maxx = maxx.max(x);
                maxy = maxy.max(y);
            }
        }
        if minx > maxx {
            return false; // nada coberto
        }
        // A altura é DERIVADA dos ingredientes, exatamente como no traço — o commit a re-deriva, mas o
        // plano vivo é o que a luz lê antes dele.
        let height: Vec<f32> = paint
            .iter()
            .map(|&p| derive_height(&spec, p, f32::from(NO_GRAIN) / 255.0))
            .collect();
        self.paint.relief.stroke_paint = paint;
        self.paint.relief.stroke_grain = vec![NO_GRAIN; n];
        self.paint.relief.stroke_film = film;
        self.paint.relief.stroke_radius = rad;
        self.paint.relief.stroke_height = height;
        self.paint.relief.stroke_relief_bbox = Some(Region {
            x: minx as u32,
            y: miny as u32,
            w: (maxx - minx + 1) as u32,
            h: (maxy - miny + 1) as u32,
        });
        true
    }
}
