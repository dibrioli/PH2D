//! **A cena pronta para o smoke do WIDTH TOOL** — `PH2D_BUILD_SMOKE=42` (plano 25 §5).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **A cena NÃO arma perfil nenhum e NÃO entra no modo Width** — o gesto que o smoke tem de
//! provar começa no pill do painel. Armar por baixo da mesa pularia exactamente a costura que ele
//! existe para exercer.
//!
//! O que ela dá é o MATERIAL: um traço reto (onde a normal é constante e a alça é fácil de ler),
//! uma curva (onde a alça tem de seguir a tangente) e um traço que **já tem perfil** — a
//! referência de como a coisa fica, para o artista comparar com o que a mão dele produz.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, WidthProfile};

/// A largura dos traços — grossa o bastante para a fita ser legível de longe.
const STROKE_W: f64 = 0.18;

fn stroked(verts: Vec<VecVertex>, rgb: [u8; 3]) -> VecPath {
    let mut p = VecPath {
        verts,
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(
        Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
        STROKE_W,
    ));
    p
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
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    // (1) Uma RETA: a normal é constante, e é onde a alça se lê sem esforço.
    let line = stroked(
        [[-3.0, 1.6], [3.0, 1.6]].map(VecVertex::corner).to_vec(),
        [70, 150, 220],
    );
    gfx.vec_scene.push_path(line);
    // (2) Uma CURVA: aqui a alça tem de seguir a tangente. Se ela ficar presa a uma direção fixa,
    // aparece na hora — as alças pousam fora da fita nas partes viradas.
    let wave: Vec<VecVertex> = (0..13)
        .map(|i| {
            let t = f64::from(i) / 12.0;
            VecVertex::corner([
                -3.0 + 6.0 * t,
                -0.2 + 0.9 * (t * std::f64::consts::TAU).sin(),
            ])
        })
        .collect();
    gfx.vec_scene.push_path(stroked(wave, [120, 190, 120]));
    // (3) A REFERÊNCIA: o mesmo traço reto, já com um afinamento nas duas pontas. O artista
    // compara — e pode agarrar as alças DESTE para ver que elas descrevem o que já está na tela.
    let ref_line = stroked(
        [[-3.0, -1.9], [3.0, -1.9]].map(VecVertex::corner).to_vec(),
        [220, 150, 90],
    );
    let rid = gfx.vec_scene.push_path(ref_line);
    app.vec_width_ref = Some(rid);
}

fn announce(app: &mut crate::App) {
    // O perfil da referência é armado AQUI, depois do `sync` do frame 3 ter dado entidade ao
    // caminho — antes disso o componente não teria onde pousar.
    if let Some(rid) = app.vec_width_ref.take()
        && let Some(gfx) = app.gfx.as_mut()
    {
        let stops = WidthProfile {
            start: 0.25,
            mid: 1.6,
            end: 0.25,
            position: 0.5,
        }
        .to_stops();
        crate::profile_live::arm(&mut gfx.sim, &app.vec_entities, &[rid], &stops);
    }
    eprintln!(
        "[smoke] width tool (plano 25 §5): tres tracos -- AZUL reto, VERDE ondulado, LARANJA reto \
         com um afinamento JA' armado (a referencia). (1) na fileira TOOL clique **Width** -- se o \
         pill nao existir ou nao acender, PARE; (2) clique o LARANJA: tres alcas aparecem SOBRE A \
         CURVA, cada uma com uma HASTE que sai ate' a borda da fita -- a haste do meio e' mais \
         longa porque ali o traco e' grosso; (3) clique o AZUL: duas alcas com hastes curtas -- \
         ele nao tem perfil, e o que se ve e' o neutro; (4) ARRASTE uma alca do azul para FORA: o traco engrossa ali, ao vivo; \
         para DENTRO: afina; (5) arraste-a AO LONGO do traco: o ponto grosso anda com o dedo; \
         (6) ARRASTE a partir de um ponto qualquer da curva onde nao ha alca: nasce uma alca ali \
         e ela ja' segue o dedo; (7) CLIQUE (sem arrastar) num ponto da curva e solte: NADA pode \
         mudar -- nem uma alca nova, nem a espessura; (8) BOTAO DIREITO sobre uma alca: ela some \
         (com duas restantes o traco volta ao uniforme); (9) faca o mesmo no VERDE e confira que \
         as HASTES seguem a TANGENTE -- elas tem de sair sempre perpendiculares a' tinta, nas \
         subidas e nas descidas; (10) Ctrl+Z desfaz UM gesto por vez. \
         [o report de 30/07] (11) desenhe um GRAMPO (uma linha que volta quase por cima de si \
         mesma, os dois bracos bem juntos) ou um X, e clique UM dos bracos: tem de nascer UMA \
         alca SO', e ela tem de ficar sobre o braco que voce clicou -- nunca sobre o vizinho; \
         (12) engrosse muito essa alca (a haste atravessa o outro braco, e isso e' honesto: a \
         fita de facto chega la') e clique DE NOVO no mesmo braco: tem de AGARRAR a alca que ja' \
         esta' la', sem criar uma segunda; (13) num traco NOVO, sem perfil nenhum, arraste a \
         partir do MEIO da curva: as duas alcas das pontas tem de continuar onde estavam -- a \
         nova entra entre elas, e o fim do traco nao pode mudar de sitio."
    );
}
