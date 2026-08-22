//! **A cena da MOLDURA** — `PH2D_BUILD_SMOKE=49`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não arma modo nenhum** — a cicatriz que o `impasto_smoke` prega: um
//! smoke que arma o estado por baixo do pano pula justamente a costura que existe para provar. As
//! molduras desta cena são construídas com o componente porque o que se julga é o RECORTE, não o
//! gesto de criar; o gesto tem o passo 5 do roteiro, e ali é o artista quem pega a ferramenta.
//!
//! # A pergunta desta cena é UMA, e é de olho
//!
//! *O que transborda de uma moldura que recorta some — e o fundo dela fica ATRÁS do conteúdo.*
//!
//! O que ela monta, e por quê:
//! - **duas molduras IDÊNTICAS**, lado a lado, com o mesmo conteúdo: a da esquerda recorta, a da
//!   direita não. É o par CONTROLE deste repo — a resposta é visível sem tocar num controle, e
//!   uma diferença que aparece nas duas não é da moldura;
//! - **três filhos** em cada, e o terceiro TRANSBORDA pela borda direita (é ele que a de fora
//!   mostra inteiro e a de dentro corta ao meio);
//! - um **vizinho SOLTO** entre as duas, que não é filho de nenhuma: o recorte não pode alcançar
//!   quem não está dentro.

use ph2d_ecs::{ChildOf, Entity, VecClipContent, VecFrame};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, ellipse, rectangle};

/// O centro de cada moldura, em `x`. A largura do telefone é ~3,70 (o preset `Phone` com o lado
/// maior em `LONG_SIDE = 8`), então 3 unidades de folga entre os centros deixa um corredor.
const FRAME_X: [f64; 2] = [-3.4, 3.4];
/// A meia-largura e a meia-altura da moldura — o aspecto do telefone, no tamanho que o preset dá.
const HALF: [f64; 2] = [1.85, 4.0];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O parentesco e o componente só depois do `sync` — é ele que dá entidade a cada caminho.
        6 => adopt(app),
        7 => announce(app),
        _ => {}
    }
}

/// Os caminhos. A ORDEM é a da pilha de z que a árvore vai ditar: os filhos primeiro (eles
/// desenham ao fundo), a moldura por último — o DFS lista o pai antes e a pilha é o inverso.
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;
    for (i, &cx) in FRAME_X.iter().enumerate() {
        // Um cabeçalho no topo, um disco no meio, e a BARRA que transborda pela direita.
        s.push_path(tint(
            rectangle([cx - HALF[0], HALF[1] - 1.2], [cx + HALF[0], HALF[1] - 0.2]),
            [90, 140, 210],
        ));
        s.push_path(tint(ellipse([cx, 0.4], 1.1, 1.1), [235, 200, 120]));
        s.push_path(tint(
            rectangle([cx - 0.6, -2.6], [cx + HALF[0] + 1.6, -1.6]),
            [225, 110, 110],
        ));
        // A moldura: o fundo do card. Ela é o ÚLTIMO da sub-árvore na pilha, e o renderer a
        // antecipa para o fundo — é isso que o passo 2 do roteiro julga.
        let shade = if i == 0 { 55 } else { 40 };
        s.push_path(tint(
            rectangle([cx - HALF[0], -HALF[1]], [cx + HALF[0], HALF[1]]),
            [shade, shade, shade + 8],
        ));
    }
    // O vizinho SOLTO, entre as duas: não é filho de moldura nenhuma.
    s.push_path(tint(ellipse([0.0, 2.6], 0.5, 0.5), [140, 220, 150]));
}

/// Pendura os três filhos em cada moldura e marca a da esquerda como recortante.
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    // Os ids saem da ORDEM em que foram empurrados: 4 caminhos por moldura, a moldura no fim.
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < 9 {
        return;
    }
    for i in 0..2 {
        let base = i * 4;
        let Some(&fb) = app.vec_entities.get(&ids[base + 3]) else {
            continue;
        };
        let frame = Entity::from_bits(fb);
        // ⚠️ A da direita é o CONTROLE: ela É moldura (a seção Frame aparece nela) e NÃO recorta.
        // Desde 2026-08-21 o contraste é a PRESENÇA do `VecClipContent` — as duas são molduras
        // por igual, e é só o recorte que as separa. É o que a cena existe para mostrar.
        if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(frame) {
            e.insert(VecFrame);
            if i == 0 {
                e.insert(VecClipContent);
            }
        }
        for k in 0..3 {
            let Some(&kb) = app.vec_entities.get(&ids[base + k]) else {
                continue;
            };
            if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(Entity::from_bits(kb)) {
                e.insert(ChildOf(frame));
            }
        }
    }
}

fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    // A contagem sai do MAPA de caminhos: é a mesma porta que a shell usa para ir de um
    // `VecPathId` à entidade, e o que o smoke afirma é sobre as molduras que a cena montou.
    let frames = app
        .vec_entities
        .values()
        .filter(|&&b| {
            gfx.sim
                .world()
                .get::<VecFrame>(Entity::from_bits(b))
                .is_some()
        })
        .count();
    eprintln!(
        "[frame] cena montada: {} formas, {frames} moldura(s) — a da ESQUERDA recorta, a da \
         DIREITA e' o controle.",
        gfx.vec_scene.paths().len()
    );
    eprintln!("[frame] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  0. ⚠️ **AS ETIQUETAS**, antes de clicar em nada: cada moldura traz o NOME dela");
    eprintln!("     acima do canto superior-esquerdo, em tamanho fixo. De zoom e pan: a etiqueta");
    eprintln!("     ACOMPANHA a moldura e NAO cresce. A da moldura selecionada acende.");
    eprintln!("  1. ⚠️ A pergunta da wave, sem tocar em nada: a BARRA vermelha transborda pela");
    eprintln!("     direita nas duas molduras. Na da ESQUERDA ela e' CORTADA na borda; na da");
    eprintln!("     DIREITA ela sai inteira. A bolinha verde entre as duas nao e' filha de");
    eprintln!("     nenhuma — nenhum recorte pode alcanca-la.");
    eprintln!("  2. ⚠️ O FUNDO: o retangulo escuro da moldura fica ATRAS do conteudo, nao por");
    eprintln!("     cima. Na arvore ele e' o PAI, e um pai desenha na frente dos filhos — e' a");
    eprintln!("     moldura que e' a excecao, porque o preenchimento dela e' o fundo do card.");
    eprintln!("  3. Selecione a moldura da ESQUERDA (clique no fundo escuro dela). Aparecem DUAS");
    eprintln!("     secoes: **Clip** e **Frame**. Na **Clip**, ponha 'Clip content' em Off: a");
    eprintln!("     barra sai inteira. Ligue de volta: ela e' cortada outra vez.");
    eprintln!("     ⚠️ Sao duas secoes de proposito: a **Clip** aparece em QUALQUER forma");
    eprintln!("     fechada (veja o passo 8), a **Frame** so' numa moldura.");
    eprintln!("  4. Ainda com ela selecionada, clique **Desktop**. ⚠️ A moldura muda de forma e o");
    eprintln!("     CONTEUDO NAO SE MEXE — nao ha layout ainda, e e' isso que torna a W2 visivel.");
    eprintln!("     Os campos W/H da secao Transform mostram os numeros novos: o preset escreve");
    eprintln!("     por ali, nao por uma porta propria.");
    eprintln!("  5. Agora o GESTO: pegue o 14o pill (**Frame**) e arraste no vazio. Nasce uma");
    eprintln!("     moldura nova, ja' recortando. Arraste uma forma para dentro dela na");
    eprintln!("     Hierarquia — ela passa a ser recortada.");
    eprintln!("  6. Ctrl+Z depois de cada passo: o chip, o preset e a moldura nova desfazem.");
    eprintln!("  7. ⚠️ **O WIDTH pela CAIXA**: selecione a barra vermelha e DIGITE um numero na");
    eprintln!("     caixa ao lado do slider Width (Enter). O traco tem de engrossar na hora —");
    eprintln!("     era esse o bug. Depois escolha uma COR: a largura NAO pode mudar junto.");
    eprintln!(
        "  8. ⚠️ **O RECORTE SEM MOLDURA** (2026-08-21): pegue a ferramenta de forma, escolha"
    );
    eprintln!("     a ESTRELA no catalogo e desenhe uma grande no vazio. Com ela selecionada,");
    eprintln!("     aparece a secao **Clip** — e NAO a **Frame** (a estrela nao e' contentor: sem");
    eprintln!("     etiqueta com nome, sem presets de telefone). Ligue 'Clip content'. Agora");
    eprintln!("     arraste uma forma para DENTRO da estrela na Hierarquia: ela passa a ser");
    eprintln!("     recortada pela silhueta da estrela, pontas e tudo.");
    eprintln!("  9. Na mesma estrela, desenhe uma LINHA (forma aberta) e selecione-a: a secao");
    eprintln!("     **Clip** NAO aparece. Uma linha nao tem 'dentro' — nao ha' o que recortar.");
    eprintln!(" 10. ⚠️ **A MOLDURA ARREDONDAVEL**: pegue o pill **Frame**, arraste uma moldura");
    eprintln!("     nova e olhe a secao de parametros da forma: ha' 'Radius' (e mais quatro).");
    eprintln!("     Suba o Radius — as quinas arredondam. Ela nasce com quina VIVA, como antes;");
    eprintln!("     o que mudou e' que agora da' para arredondar.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena contém o fenômeno que o roteiro pede.** Um roteiro que manda olhar um transbordo
    /// que a geometria não tem engana exactamente quem o corre.
    #[test]
    fn the_third_child_overflows_the_frame() {
        let cx = FRAME_X[0];
        let bar = rectangle([cx - 0.6, -2.6], [cx + HALF[0] + 1.6, -1.6]);
        let right = bar
            .verts
            .iter()
            .map(|v| v.anchor[0])
            .fold(f64::MIN, f64::max);
        assert!(
            right > cx + HALF[0],
            "a barra tem de passar da borda direita da moldura ({right} <= {})",
            cx + HALF[0]
        );
    }

    /// E os outros dois NÃO transbordam — senão o passo 1 não distinguiria recorte de nada.
    #[test]
    fn the_other_children_stay_inside() {
        let cx = FRAME_X[0];
        for p in [
            rectangle([cx - HALF[0], HALF[1] - 1.2], [cx + HALF[0], HALF[1] - 0.2]),
            ellipse([cx, 0.4], 1.1, 1.1),
        ] {
            for v in &p.verts {
                assert!(
                    v.anchor[0] <= cx + HALF[0] + 1e-9 && v.anchor[0] >= cx - HALF[0] - 1e-9,
                    "esta forma ja' transborda: {:?}",
                    v.anchor
                );
            }
        }
    }

    /// A moldura tem o aspecto do preset `Phone` — a cena mostra o que o botão entrega.
    #[test]
    fn the_frame_matches_the_phone_preset() {
        let phone = ph2d_tool_vector::frames::DEVICE_PRESETS[0];
        let (w, h) = phone.size();
        assert!((w - HALF[0] * 2.0).abs() < 0.02, "largura {w}");
        assert!((h - HALF[1] * 2.0).abs() < 0.02, "altura {h}");
    }
}
