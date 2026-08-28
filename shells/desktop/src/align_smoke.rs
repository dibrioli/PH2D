//! **A cena do ALINHAMENTO do traço** — `PH2D_BUILD_SMOKE=47`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não arma modo nenhum** — a cicatriz que o `impasto_smoke` do Painter
//! prega: um smoke que arma o estado por baixo do pano pula justamente a costura que existe para
//! provar. Todas as formas nascem em **Centre**, que é o default do produto, e é o artista quem
//! escolhe Inner/Outer no painel.
//!
//! # A pergunta desta cena é de OLHO, e é UMA
//!
//! *Engrossar um traço INTERNO não pode fazer a forma crescer.* Tudo o mais é consequência: a
//! silhueta parada é a razão de a feature existir, e é a única coisa que uma screenshot mostra
//! sem ninguém ler um número.
//!
//! O que ela monta:
//! - **três quadrados IDÊNTICOS** lado a lado, com o MESMO traço grosso — o trio de comparação.
//!   Eles nascem os três em Centre, e é o artista que põe o do meio em Inner e o da direita em
//!   Outer. Quadrados porque a diferença é medível a olho contra a régua das linhas-guia;
//! - a **ROSQUINHA** (um quadrado com furo, EvenOdd) — a forma que separa *"recortei contra o
//!   contorno de fora"* de *"recortei contra a REGIÃO"*: no Inner a faixa tem de abraçar o FURO
//!   também, e no Outer ela tem de entrar nele;
//! - uma **LINHA ABERTA**, o CONTROLE. Ela não tem interior, então os três chips não podem mudar
//!   um pixel dela — e o painel diz isso antes de o artista descobrir por acidente.

use ph2d_vec_scene::{Contour, FillRule, Paint, Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Largura do traço, em unidades de mundo. **Grossa de propósito**: com um fio fino a diferença
/// entre os três alinhamentos é sub-pixel, e a cena não provaria nada.
const STROKE_W: f64 = 0.28;

/// O lado do quadrado de comparação.
const SIDE: f64 = 1.4;

/// Os três centros do trio, em `x`. A distância entre eles é folgada o bastante para o Outer de
/// um não encostar no vizinho.
const TRIO_X: [f64; 3] = [-2.6, 0.0, 2.6];

fn vertex(a: [f64; 2]) -> VecVertex {
    VecVertex {
        anchor: a,
        in_handle: a,
        out_handle: a,
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn stroke(rgb: [u8; 3]) -> StrokeSpec {
    // ⚠️ Sem `align:` — a forma nasce em Centre, o default. Semear Inner aqui seria o smoke a
    // armar o estado por baixo do pano, que é exatamente o que este cabeçalho recusa.
    StrokeSpec::new(Rgba8::new(rgb[0], rgb[1], rgb[2], 255), STROKE_W)
}

/// Um quadrado centrado em `(cx, cy)`, preenchido (o miolo é o que torna *dentro* visível).
fn square(cx: f64, cy: f64, side: f64, rgb: [u8; 3]) -> VecPath {
    let h = side * 0.5;
    VecPath {
        verts: [
            [cx - h, cy - h],
            [cx + h, cy - h],
            [cx + h, cy + h],
            [cx - h, cy + h],
        ]
        .iter()
        .map(|p| vertex(*p))
        .collect(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(70, 80, 95, 255))),
        stroke: Some(stroke(rgb)),
        ..VecPath::default()
    }
}

/// A rosquinha: o mesmo quadrado com um FURO. `EvenOdd` é o que faz o contorno de dentro ser
/// buraco, e é o furo que separa as duas leis de recorte possíveis.
fn donut(cx: f64, cy: f64) -> VecPath {
    let mut p = square(cx, cy, SIDE * 1.7, [200, 180, 120]);
    let h = SIDE * 0.34;
    p.subpaths = vec![Contour::new_closed(
        [
            [cx - h, cy - h],
            [cx + h, cy - h],
            [cx + h, cy + h],
            [cx - h, cy + h],
        ]
        .iter()
        .map(|q| vertex(*q))
        .collect(),
    )];
    p.fill_rule = FillRule::EvenOdd;
    p
}

/// O CONTROLE: uma linha aberta, sem interior.
fn open_line(cx: f64, cy: f64) -> VecPath {
    VecPath {
        verts: [[cx - 1.2, cy], [cx, cy + 0.5], [cx + 1.2, cy]]
            .iter()
            .map(|p| vertex(*p))
            .collect(),
        closed: false,
        stroke: Some(stroke([130, 200, 170])),
        ..VecPath::default()
    }
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, cx) in TRIO_X.iter().enumerate() {
        // Os três são IDÊNTICOS de propósito — a cor só os nomeia no roteiro.
        let rgb = [[235, 120, 120], [120, 200, 235], [235, 200, 120]][i];
        gfx.vec_scene.push_path(square(*cx, 1.4, SIDE, rgb));
    }
    gfx.vec_scene.push_path(donut(-1.6, -1.6));
    gfx.vec_scene.push_path(open_line(2.0, -1.6));
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    // O que o artista vai medir a olho: o quadrado mede `SIDE`, e um traço CENTRADO faz a
    // silhueta medir `SIDE + w`. Inner tem de a devolver a `SIDE`; Outer leva-a a `SIDE + 2w`.
    let (c, i, o) = (SIDE + STROKE_W, SIDE, SIDE + 2.0 * STROKE_W);
    eprintln!(
        "[align] cena montada: {} formas — o TRIO de quadrados iguais (lado {SIDE:.2}, traço \
         {STROKE_W:.2}), a ROSQUINHA e a LINHA ABERTA (o controle).",
        gfx.vec_scene.paths().len()
    );
    eprintln!(
        "[align] TODAS nascem em Centre — o default. A silhueta de um quadrado mede {c:.2} em \
         Centre, {i:.2} em Inner (o lado exacto) e {o:.2} em Outer."
    );
    eprintln!("[align] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Selecione o quadrado do MEIO. Na seção Stroke, a row 'Align': escolha Inner.");
    eprintln!("     ⚠️ A SILHUETA NÃO PODE CRESCER — a tinta entra para dentro do quadrado, e a");
    eprintln!("     borda externa fica exactamente onde o quadrado vermelho da esquerda a tem.");
    eprintln!("  2. Selecione o da DIREITA e escolha Outer. Agora ele é o MAIOR dos três, e o");
    eprintln!("     miolo escuro dele é o maior também: a tinta saiu toda para fora.");
    eprintln!("  3. Arraste o slider Width com o do meio (Inner) selecionado. ⚠️ Engrossar NÃO");
    eprintln!("     move a borda externa — é a promessa inteira da feature, e é de olho.");
    eprintln!("  4. A ROSQUINHA, em Inner: a faixa tem de abraçar o FURO também (uma coroa em");
    eprintln!("     volta dele), não só a borda de fora. Em Outer ela ENTRA no furo.");
    eprintln!("  5. A LINHA ABERTA é o CONTROLE: selecione-a e clique Inner e Outer. ⚠️ Ela não");
    eprintln!("     pode mudar um pixel — sem interior, 'dentro' e 'fora' não significam nada.");
    eprintln!("  6. Volte tudo a Centre. ⚠️ As três formas têm de ficar EXACTAMENTE como");
    eprintln!("     nasceram — o alinhamento é desenho derivado, e nunca tocou o documento.");
    eprintln!("  7. Ctrl+Z depois de cada escolha: a row volta ao valor anterior.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena nasce toda em Centre.**
    ///
    /// ⚠️ É o gate que impede o smoke de provar a coisa errada. Uma cena que já entregasse a
    /// forma em Inner pularia a costura inteira — o chip, a rota até o tool, o re-cozimento —
    /// e mostraria uma imagem bonita sobre um caminho que nunca correu.
    #[test]
    fn every_shape_is_born_centred() {
        use ph2d_vec_scene::StrokeAlign;
        let shapes = [
            square(TRIO_X[0], 1.4, SIDE, [1, 2, 3]),
            donut(-1.6, -1.6),
            open_line(2.0, -1.6),
        ];
        for s in &shapes {
            assert_eq!(
                s.stroke
                    .as_ref()
                    .expect("toda forma da cena tem traço")
                    .align,
                StrokeAlign::Centre,
                "a cena semeou um alinhamento por baixo do pano"
            );
        }
    }

    /// **Os três números que a mensagem anuncia são os que o artista vai MEDIR.**
    ///
    /// ⚠️ A `announce` deriva `SIDE + w` / `SIDE` / `SIDE + 2w` da aritmética; isto mede a
    /// SILHUETA que o motor de facto produz sobre a geometria desta cena. Uma cena que afirma um
    /// número que o produto não entrega engana exactamente quem a corre — e o roteiro pede que o
    /// artista compare as bordas a olho, então o número tem de ser o do desenho.
    #[test]
    fn the_announced_silhouettes_are_the_ones_the_engine_draws() {
        use ph2d_vec_scene::StrokeAlign;
        let width = |align: StrokeAlign| -> f64 {
            let mut sq = square(0.0, 0.0, SIDE, [1, 2, 3]);
            if let Some(st) = sq.stroke.as_mut() {
                st.align = align;
            }
            // A silhueta DESENHADA: o preenchimento unido à faixa (em Centre não há faixa
            // derivada — o traço é pintado como traço, e cobre `w/2` para cada lado).
            match ph2d_vec_boolean::aligned_stroke(&sq) {
                Some(band) => {
                    let xs: Vec<f64> = band
                        .iter()
                        .chain(std::iter::once(&sq))
                        .flat_map(|p| p.verts.iter().map(|v| v.anchor[0]))
                        .collect();
                    let (lo, hi) = xs
                        .iter()
                        .fold((f64::MAX, f64::MIN), |(l, h), &x| (l.min(x), h.max(x)));
                    hi - lo
                }
                None => SIDE + STROKE_W, // Centre: o traço cobre meia largura de cada lado.
            }
        };
        let tol = 0.02;
        assert!((width(StrokeAlign::Centre) - (SIDE + STROKE_W)).abs() < tol);
        assert!(
            (width(StrokeAlign::Inner) - SIDE).abs() < tol,
            "Inner mediu {:.3}, e a mensagem promete {SIDE:.2}",
            width(StrokeAlign::Inner)
        );
        assert!(
            (width(StrokeAlign::Outer) - (SIDE + 2.0 * STROKE_W)).abs() < tol,
            "Outer mediu {:.3}, e a mensagem promete {:.2}",
            width(StrokeAlign::Outer),
            SIDE + 2.0 * STROKE_W
        );
    }

    /// **A rosquinha tem FURO, e a linha é ABERTA** — sem isso os passos 4 e 5 do roteiro não
    /// provam nada, e a mensagem estaria a afirmar o que a geometria não tem.
    #[test]
    fn the_fixture_contains_the_phenomena_the_script_asks_about() {
        assert_eq!(
            donut(0.0, 0.0).subpaths.len(),
            1,
            "a rosquinha perdeu o furo"
        );
        assert_eq!(donut(0.0, 0.0).fill_rule, FillRule::EvenOdd);
        assert!(
            !open_line(0.0, 0.0).closed,
            "o controle deixou de ser aberto"
        );
    }
}
