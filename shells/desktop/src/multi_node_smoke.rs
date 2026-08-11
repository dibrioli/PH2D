//! **A cena dos NÓS DE VÁRIAS FORMAS** — `PH2D_BUILD_SMOKE=70`.
//!
//! O plano 25 §6 nomeava editar nós de várias formas como ausência **POR CONSTRUÇÃO**: a seleção
//! de nós era uma lista de índices PLANOS dentro de um caminho único, então dois índices de formas
//! diferentes eram indistinguíveis. Medido antes da wave, com duas formas lado a lado:
//!
//! | gesto | apanhava | devia |
//! |---|---|---|
//! | caixa sobre AS DUAS | **4 de 8** | 8 |
//! | somar B a A (Shift) | **4** (trocava de alvo) | 8 |
//! | Shift+clique em A, depois em B | **1** (o 2º substituía) | 2 |
//! | nudge depois da caixa | movia **4 de 8** | 8 |
//!
//! # A cena tem TRÊS formas, e a do meio é ESCALADA — a premissa que a torna capaz de reprovar
//!
//! ⚠️ A metade visível (*os nós das três acendem*) falha de forma óbvia. A metade que **compila,
//! roda e deforma em silêncio** é o ESPAÇO: a conversão mundo→local é por forma (ADR-0111), e um
//! único delta serviria a todas — a forma escalada andaria o dobro e a seleção se desmontaria sob
//! o dedo, com a contagem certa o tempo todo. Com as três na mesma escala o olho não distingue as
//! duas leis; por isso a do meio nasce **2×**, e o [`announce`] IMPRIME a escala que encontrou.
//!
//! # O que esta cena NÃO arma
//!
//! Nada da seleção. A lei que o `impasto_smoke` prega — *um smoke que arma o estado por baixo do
//! pano pula justamente a costura que existe para provar* — vale inteira aqui: o gesto de apanhar
//! nós de três formas É a wave.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// Os centros das três formas, em X.
const CX: [f64; 3] = [-2.6, 0.0, 2.6];
/// Meia-largura e meia-altura de cada quadrado (na geometria LOCAL).
const HALF: f64 = 0.7;
/// A escala da forma do MEIO — a premissa que torna a cena capaz de separar as duas leis do espaço.
const MIDDLE_SCALE: f32 = 2.0;
/// Uma cor por forma, para o artista dizer *"o nó daquela ali"* sem contar.
const RGB: [[u8; 3]; 3] = [[86, 132, 214], [214, 132, 86], [120, 196, 128]];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // A pose só depois do `sync` (é ele que dá entidade a cada caminho) e do `settle_origins`,
        // que assenta o pivô no centro de cada forma — escrever a escala antes seria escrevê-la
        // num `Transform` que o assentamento reescreve.
        7 => scale_the_middle_one(app),
        8 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (k, cx) in CX.iter().enumerate() {
        // A do meio é desenhada MENOR na geometria local, para que a escala 2× a deixe do tamanho
        // aparente das irmãs — a cena compara gestos, não tamanhos.
        let h = if k == 1 {
            HALF / f64::from(MIDDLE_SCALE)
        } else {
            HALF
        };
        gfx.vec_scene
            .push_path(tint(rectangle([cx - h, -h], [cx + h, h]), RGB[k]));
    }
    app.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Node);
}

/// Escala a forma do MEIO — a premissa do §"a cena tem três formas".
fn scale_the_middle_one(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    let Some(&bits) = ids.get(1).and_then(|id| app.vec_entities.get(id)) else {
        return;
    };
    let e = Entity::from_bits(bits);
    let keep = gfx
        .sim
        .world()
        .get::<Transform>(e)
        .map_or(Vec2::ZERO, |t| t.translation);
    if let Ok(mut em) = gfx.sim.world_mut().get_entity_mut(e) {
        em.insert(Transform {
            translation: keep,
            scale: Vec2::new(MIDDLE_SCALE, MIDDLE_SCALE),
            ..Transform::IDENTITY
        });
    }
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let n = gfx.vec_scene.paths().len();
    let nodes: usize = gfx.vec_scene.paths().iter().map(|p| p.total_verts()).sum();
    let mid_scale = gfx
        .vec_scene
        .paths()
        .get(1)
        .and_then(|p| app.vec_entities.get(&p.id))
        .and_then(|&b| gfx.sim.world().get::<Transform>(Entity::from_bits(b)))
        .map_or(1.0, |t| t.scale.x);
    eprintln!("[multi-node-smoke] cena montada: {n} formas, {nodes} nos, modo NODE.");
    eprintln!("[multi-node-smoke] escala da forma do MEIO: {mid_scale:.2}");
    eprintln!(
        "[multi-node-smoke] (!) se as formas nao forem 3, os nos nao forem 12, ou a escala do \
         meio nao for {MIDDLE_SCALE:.2}, PARE: a cena perdeu a premissa e o passo 2 deixa de \
         distinguir a lei certa da errada."
    );
    eprintln!("[multi-node-smoke] o roteiro (a ferramenta VECTOR ja' esta' no modo Node):");
    eprintln!("  1. Arraste uma CAIXA sobre as TRES formas. Os 12 nos tem de acender em CIANO.");
    eprintln!(
        "     (antes desta wave acendiam so' os 4 de UMA delas -- e o resto da caixa nao fazia nada)"
    );
    eprintln!(
        "  2. Agarre UM no' e arraste. As tres formas acompanham, e a do MEIO -- que e' escalada \
         {MIDDLE_SCALE:.0}x -- anda a MESMA distancia na tela que as outras."
    );
    eprintln!(
        "     (!) se ela andar o dobro, ou a selecao se desmontar, o delta esta' a ser convertido \
         no espaco da forma errada."
    );
    eprintln!("  3. Ctrl+Z. Agora **SHIFT+clique** um no' da 1a forma, e depois um no' da 3a.");
    eprintln!("     Os DOIS ficam acesos -- antes, o segundo clique APAGAVA o primeiro.");
    eprintln!(
        "  4. Com nos de duas formas escolhidos, aperte Delete: os nos somem das DUAS, e as formas \
         que sobrevivem continuam selecionadas."
    );
    eprintln!(
        "  5. O CONTROLE: desfaca tudo, e trabalhe UMA forma so' -- caixa, arrasto, Tab, Delete. \
         Tem de estar exatamente como sempre esteve."
    );
}
