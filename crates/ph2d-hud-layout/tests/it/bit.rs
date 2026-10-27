//! O mesmo bit a cada quadro; quem não tem relação é a sua caixa; índice mau ignorado.

use ph2d_hud_layout::{Arranjo, Eixo, Forca, Lado, Relacao, resolve};

use crate::{bits, caixa, el};

fn cena() -> (Vec<ph2d_hud_layout::Elemento>, Vec<(Relacao, Forca)>) {
    let els = vec![
        el(16.0, 312.0, 100.0, 32.0, Some(61.3)),
        el(200.0, 316.0, 24.0, 24.0, None),
        el(504.0, 240.0, 120.0, 104.0, Some(120.0)),
        el(120.0, 100.0, 400.0, 200.0, None),
        el(0.0, 260.0, 180.0, 30.0, Some(96.7)),
    ];
    let rel = vec![
        (Relacao::LarguraDoConteudo { a: 0 }, Forca::Media),
        (Relacao::AoLado { a: 1, b: 0, lado: Lado::Direita, folga: 8.1 }, Forca::Requerida),
        (Relacao::LarguraDoConteudo { a: 2 }, Forca::Forte),
        (Relacao::NaoInvade { a: 1, b: 2, lado: Lado::Direita, folga: 7.7 }, Forca::Requerida),
        (Relacao::Centrado { a: 4, b: 3, eixo: Eixo::X }, Forca::Forte),
        (Relacao::LarguraDoConteudo { a: 4 }, Forca::Forte),
    ];
    (els, rel)
}

#[test]
fn the_layout_is_the_same_bit_every_frame() {
    let (els, rel) = cena();
    let conteudos: Vec<_> = els.iter().map(|e| e.conteudo).collect();
    let tela = caixa(0.0, 0.0, 640.3, 360.0);
    let mut arr = Arranjo::new(tela, &els, &rel);
    let primeiro = bits(&arr.atualiza(tela, &conteudos));
    assert_eq!(primeiro.len(), els.len(), "uma caixa por elemento");
    assert_ne!(primeiro[1], bits(&[els[1].caixa])[0], "a moeda tinha de ter andado (o solver agiu)");
    for _ in 0..100 {
        assert_eq!(bits(&arr.atualiza(tela, &conteudos)), primeiro, "o mesmo quadro mudou de bit");
    }
    // Mudar (tela estreita, texto maior, texto que some) e voltar devolve os primeiros bits.
    let mut outro = conteudos.clone();
    outro[0] = Some(377.9);
    outro[4] = None;
    for _ in 0..10 {
        arr.atualiza(caixa(0.0, 0.0, 333.3, 300.0), &outro);
        assert_eq!(bits(&arr.atualiza(tela, &conteudos)), primeiro, "ida e volta mudou o bit");
    }
    // Outra instância (outras variáveis, outras sementes de hash) dá os mesmos bits.
    for _ in 0..50 {
        assert_eq!(bits(&Arranjo::new(tela, &els, &rel).atualiza(tela, &conteudos)), primeiro, "outra instância");
    }
}

#[test]
fn an_element_without_relations_is_its_own_box_bit_for_bit() {
    let (mut els, rel) = cena();
    // Fora da tela e com largura negativa: sem relação, nem a tela o toca.
    let solto = el(-0.1, 1e300, -3.3, f64::MIN_POSITIVE, Some(9.0));
    els.push(solto);
    let tela = caixa(0.0, 0.0, 640.0, 360.0);
    let r = resolve(tela, &els, &rel);
    assert_eq!(r.len(), els.len());
    assert_eq!(bits(&r[5..]), bits(&[solto.caixa]));
    // Sem nenhuma relação, o solver nem nasce: tudo volta ao bit.
    let r = resolve(tela, &els, &[]);
    assert_eq!(bits(&r), bits(&els.iter().map(|e| e.caixa).collect::<Vec<_>>()));
}

#[test]
fn a_bad_index_is_ignored() {
    let els = [el(16.0, 312.0, 100.0, 32.0, Some(50.0)), el(200.0, 316.0, 24.0, 24.0, None)];
    let tela = caixa(0.0, 0.0, 640.0, 360.0);
    let mas = [
        (Relacao::AoLado { a: 1, b: 9, lado: Lado::Direita, folga: 8.0 }, Forca::Requerida),
        (Relacao::Centrado { a: 9, b: 0, eixo: Eixo::Y }, Forca::Requerida),
        (Relacao::NaoInvade { a: 0, b: 0, lado: Lado::Esquerda, folga: 1.0 }, Forca::Requerida),
        (Relacao::LarguraDoConteudo { a: usize::MAX }, Forca::Forte),
    ];
    assert_eq!(bits(&resolve(tela, &els, &mas)), bits(&[els[0].caixa, els[1].caixa]), "relações más são inertes");
    // Misturadas com uma boa, só a boa conta; conteúdos a menos também não rebentam.
    let mut rel = mas.to_vec();
    rel.push((Relacao::AoLado { a: 1, b: 0, lado: Lado::Direita, folga: 8.0 }, Forca::Requerida));
    let mut arr = Arranjo::new(tela, &els, &rel);
    let r = arr.atualiza(tela, &[]);
    assert_eq!(r[1].x, r[0].x + r[0].w + 8.0);
    assert_eq!(r.len(), 2);
}
