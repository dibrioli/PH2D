//! ⭐⭐⭐ **A LISTA DE VIEWPORTS** (W90) — abrir, fechar, e de quem é um ponto.
//!
//! ⚠️ É o **único** sítio do módulo que escreve a lista inteira, e é por isso que a invariante
//! *«nunca vazia»* se defende aqui (ver [`ensure_viewports`]) — o censo
//! `nothing_can_empty_the_viewport_list` nomeia esta função, e nenhuma outra.

use super::Smoke;

/// ⭐⭐ **Qual viewport está debaixo deste ponto da JANELA** (W90).
///
/// ⚠️ Lê as áreas **guardadas nos viewports**, e não o layout: o ponteiro corre fora do quadro e não
/// tem a área do app na mão. Um viewport que ainda não desenhou tem `area: None` e recebe um
/// retângulo vazio, que nunca ganha — *«ainda não desenhei» e «o ponto não é meu» são a mesma
/// resposta para quem pergunta de quem é um clique.*
pub(crate) fn viewport_at(s: &Smoke, pos: (f32, f32)) -> Option<usize> {
    crate::field3d_layout::hit(
        s.vps.iter().map(|v| {
            v.area
                .unwrap_or(ph2d_editor::zones::Rect::new(0.0, 0.0, 0.0, 0.0))
        }),
        [pos.0, pos.1],
    )
}

/// ⭐⭐ **O retângulo do CANVAS inteiro** — a união dos viewports (W92).
///
/// ⚠️ **É derivado e não guardado**, e os retângulos ladrilham a área exactamente (há gate), então a
/// união é a área. *Guardá-lo seria uma segunda resposta a «onde está o canvas?», e a que
/// envelheceria seria a guardada.* `None` enquanto nenhuma vista tiver desenhado.
pub(crate) fn canvas_area(s: &Smoke) -> Option<ph2d_editor::zones::Rect> {
    let mut it = s.vps.iter().filter_map(|v| v.area);
    let first = it.next()?;
    let (mut x0, mut y0) = (first.x, first.y);
    let (mut x1, mut y1) = (first.x + first.w, first.y + first.h);
    for r in it {
        x0 = x0.min(r.x);
        y0 = y0.min(r.y);
        x1 = x1.max(r.x + r.w);
        y1 = y1.max(r.y + r.h);
    }
    Some(ph2d_editor::zones::Rect::new(x0, y0, x1 - x0, y1 - y0))
}

/// ⭐⭐⭐ **ABRE E FECHA A DIVISÃO** (W90).
///
/// ⚠️ **Ao fechar, a vista que fica é a ACTIVA** — não «a primeira». É a lei do Blender e é a certa:
/// o artista fecha a divisão a olhar para o quadrante que lhe interessa, e ficar com outro seria
/// desfazer-lhe o gesto. A [`ensure_viewports`] já lê `Smoke::vp()`, então isto sai de graça.
pub(crate) fn toggle_split(s: &mut Smoke) {
    use crate::field3d_layout::Split;
    s.split = match s.split {
        Split::One => Split::quad(),
        Split::Quad { .. } => Split::One,
    };
    // ⚠️ A lista é reconciliada no desenho ([`ensure_viewports`]), que é quem sabe a área — mas o
    // **número** já é conhecido aqui, e adiá-lo faria o quadro seguinte perguntar «de quem é este
    // clique?» a uma lista que ainda não existe.
    ensure_viewports(s, s.split.count());
}

/// ⭐⭐⭐ **A LISTA DE VIEWPORTS SEGUE A DIVISÃO** (W90) — e este é o **único** sítio que a encolhe.
///
/// # ⚠️ Porque ela é reconstruída, e não crescida
///
/// Abrir a divisão em quatro não é *«acrescentar três»*: a vista do artista muda de **quadrante**
/// (no Blender ela é a de baixo à direita, que é onde a mão dele já está), e as outras três nascem
/// **nomeadas**. Uma lista crescida por `push` poria a perspectiva no canto de cima à esquerda, onde
/// o Blender põe o *Top*.
///
/// ⚠️ **Só quando a CONTAGEM muda.** Reconstruir todo quadro deitaria fora o quadro pronto, o
/// traçado em voo e a cache de fitas de cada viewport — a cada 16 ms. A guarda é a primeira linha.
///
/// ⭐ **As vistas nomeadas nascem `manual: true`**, e isso não é um extra: o `manual` é *«o prato já
/// foi tocado»*, e um `Top` que girasse sozinho deixaria de ser o Top no quadro seguinte. *A lei que
/// já existia responde à pergunta nova sem um campo a mais.*
pub(crate) fn ensure_viewports(smoke: &mut Smoke, n: usize) {
    // ⚠️⚠️ **O piso é UM, e ele mora aqui.** Este é o único sítio do módulo que escreve a lista
    // inteira, então é aqui que a invariante *«nunca vazia»* se defende — não em cada chamador. Um
    // `n = 0` que chegasse (uma divisão nova mal contada) faria o `Smoke::vp` fazer `len() - 1` em
    // `usize` e o módulo entrar em pânico no caminho mais quente que tem.
    let n = n.max(1);
    if smoke.vps.len() == n {
        return;
    }
    // A câmera que o artista tem AGORA — é ela que sobrevive à mudança de divisão, porque é a única
    // que ele autorou.
    let (cam_artista, manual_artista) = (smoke.vp().cam, smoke.vp().manual);
    let split = smoke.split;
    let mut novos: Vec<crate::field3d_smoke::state::Viewport> = Vec::with_capacity(n);
    let mut do_artista = 0usize;
    for i in 0..n {
        match split.named(i) {
            Some(v) => {
                let mut cam = cam_artista;
                cam.rotation = v.rotation();
                // ⚠️ **Ortográfica**, como no Blender: uma vista nomeada existe para MEDIR, e a
                // perspectiva estraga exactamente isso.
                cam.lens = ph2d_field_render::Lens::Ortho;
                novos.push(crate::field3d_smoke::state::Viewport::new(cam, true));
            }
            None => {
                do_artista = i;
                novos.push(crate::field3d_smoke::state::Viewport::new(
                    cam_artista,
                    manual_artista,
                ));
            }
        }
    }
    smoke.vps = novos;
    smoke.active = do_artista;
}
