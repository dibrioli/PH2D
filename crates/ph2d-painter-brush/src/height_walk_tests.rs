//! **A banda não move um byte** — o gate que a divisão em linhas do laço de altura precisa ter.
//!
//! A wave de 2026-08-10 dividiu o [`super::walk_dab_rows`] em bandas de linhas disjuntas porque a
//! auditoria mediu o passe de altura em **4,67 ms/move a raio 100** contra **0,30 ms** do depósito de
//! cor, que faz o mesmo caminho com um carimbo em cache. A divisão é o primitivo que a cor já usa
//! ([`crate::dab::band_count`]), e o argumento de identidade é estrutural — linhas disjuntas, um
//! escritor por texel, nenhum texel lê o vizinho.
//!
//! ⚠️ **Um argumento estrutural não é uma prova, e o modo de falha aqui é ARITMÉTICO**: os cinco
//! planos que uma banda possui começam na linha dela, então o índice deles é LOCAL, enquanto o gate e
//! a mordida são planos da tela e continuam GLOBAIS. Trocar um pelo outro compila, não avisa, e
//! escreve o relevo em linhas erradas — deslocado por `band_y0`, ou seja **invisível na primeira
//! banda**, que é onde toda fixture pequena olha.
//!
//! ⚠️ **E o gate carrega o próprio CONTROLE:** sem afirmar que a fixture de fato CRUZA o piso, ele
//! compararia a rota serial com ela mesma e passaria por vácuo — que é exactamente o que aconteceria
//! com o raio 20 do smoke (2 025 visitas contra um piso de 3 232).

use crate::height::{HeightDab, HeightFields, accumulate_dab_height};
use crate::{BrushSpec, Falloff, TextureKind, ablate};

/// Os cinco planos canvas-shaped que o depósito escreve.
struct Planes {
    height: Vec<f32>,
    paint: Vec<f32>,
    grain: Vec<u8>,
    film: Vec<u8>,
    radius: Vec<f32>,
}

impl Planes {
    fn new(n: usize) -> Self {
        Self {
            height: vec![0.0; n],
            paint: vec![0.0; n],
            grain: vec![crate::height::NO_GRAIN; n],
            film: vec![0; n],
            radius: vec![0.0; n],
        }
    }

    fn fields(&mut self) -> HeightFields<'_> {
        HeightFields {
            height: &mut self.height,
            paint: &mut self.paint,
            grain: &mut self.grain,
            film: &mut self.film,
            radius: &mut self.radius,
            gate: None,
        }
    }
}

/// Um pincel que exercita **todo ramo do laço**: Shape procedural (a silhueta sai da amostra, não do
/// falloff), Grain (a cauda samplea e quantiza), impasto com Smooth Edges (o AA do filme e a LUT) e um
/// filme de substrato (o termo que a wave do Relief da Shape acrescentou ao `derive_height`).
fn loaded(radius: f32) -> BrushSpec {
    let mut s = BrushSpec {
        radius_px: radius,
        falloff: Falloff::Smooth,
        impasto: true,
        impasto_depth: 0.5,
        impasto_smooth_edges: true,
        film_depth: 0.04,
        ..Default::default()
    };
    s.shape.kind = TextureKind::Stripes;
    s.shape.size = [0.35, 0.35];
    s.texture.kind = TextureKind::Noise;
    s.texture.size = [0.5, 0.5];
    s
}

/// Um dab VARRIDO — a corda que faz o `t` da altura ser o da cápsula e não o do disco, e portanto o
/// único fixture em que banda, calota e straddle correm juntos.
fn swept(s: &BrushSpec, radius: f32, centre: [f32; 2]) -> HeightDab<'static> {
    let back = radius * 0.2;
    HeightDab {
        center: centre,
        radius,
        coverage: 1.0,
        footprint: s.dab_footprint([1.0, 0.0]),
        prev_center: Some([centre[0] + back * 0.891, centre[1] + back * 0.454]),
        shape: None,
        grain: None,
        grain_image: None,
    }
}

/// Deposita 3 dabs ao longo de uma linha, com a máscara de ablação armada, e devolve os planos **e os
/// retângulos sujos** — o `touched` de uma banda tem de chegar a quem a abriu, e um `fold` que o perde
/// deixa os cinco planos idênticos com o preview sem saber que algo mudou.
fn deposit(mask: u32, radius: f32, side: u32) -> (Planes, Vec<Option<crate::dab::DirtyRect>>) {
    let s = loaded(radius);
    let mut planes = Planes::new((side as usize) * (side as usize));
    let mut rects = Vec::new();
    ablate::with(mask, || {
        for k in 0..3u8 {
            let cx = f32::from(k).mul_add(radius * 0.2, side as f32 * 0.5);
            let dab = swept(&s, radius, [cx, side as f32 * 0.5]);
            rects.push(accumulate_dab_height(
                &mut planes.fields(),
                side,
                side,
                &s,
                &dab,
                None,
            ));
        }
    });
    (planes, rects)
}

/// **A rota em BANDA escreve exactamente o que a serial escreve, nos cinco planos.**
///
/// **Mutação que tem de sangrar:** trocar o índice local dos planos pelo global (`let i = gi;` em
/// [`super::walk_band`]) — as bandas ≥ 1 passam a escrever fora da própria fatia e o `assert` de
/// `height` acusa. Medido ao escrever este gate: **1 banda contra 7**, e o produto errado só é
/// visível a partir da segunda.
#[test]
fn the_banded_walk_writes_what_the_serial_walk_writes() {
    const SIDE: u32 = 320;
    const RADIUS: f32 = 100.0;

    // CONTROLE: a fixture TEM de cruzar o piso, senão as duas colunas são a mesma rota.
    let rows = (2.0 * RADIUS) as usize;
    let bands = crate::dab::band_count(rows * rows, rows, crate::dab::PARALLEL_MIN_AREA);
    assert!(
        bands > 1,
        "a fixture não cruza o piso ({bands} banda) — o gate compararia serial com serial"
    );

    let (a, ra) = deposit(ablate::SERIAL, RADIUS, SIDE);
    let (b, rb) = deposit(0, RADIUS, SIDE);

    // CONTROLE 2: houve depósito. Dois conjuntos de planos vazios são trivialmente iguais.
    let laid = b.film.iter().filter(|&&v| v > 0).count();
    assert!(laid > 10_000, "a fixture não depositou nada: {laid} texels");

    assert_eq!(a.height, b.height, "o plano de ALTURA divergiu");
    assert_eq!(a.paint, b.paint, "o plano de CARGA divergiu");
    assert_eq!(a.grain, b.grain, "o plano de GRAIN divergiu");
    assert_eq!(a.film, b.film, "o plano de FILME divergiu");
    assert_eq!(a.radius, b.radius, "o plano de RAIO divergiu");
    // ⚠️ E o retângulo SUJO: um `fold` que perdesse o `touched` de uma banda deixaria os cinco planos
    // idênticos (as escritas aconteceram) e devolveria `None` — o preview nunca saberia que a tinta
    // mudou. Os planos não podem provar isto; só o valor de retorno pode.
    assert!(
        ra.iter().all(Option::is_some),
        "a rota serial não sujou nada"
    );
    assert_eq!(ra, rb, "o retângulo sujo divergiu entre as duas rotas");
}

/// **A MORDIDA do bow wave mantém a rota serial** — e isto é uma decisão, não um esquecimento.
///
/// `PushBite::displaced` é um escalar em `f32` somado texel a texel; somas parciais por banda dariam
/// outra ordem de adição, logo outros bits, logo outro desenho de onda — o que o Enio aprovou olhando.
///
/// **Mutação que tem de sangrar, e ela é PIOR do que eu escrevi da primeira vez.** A 1ª versão deste
/// doc dizia *"o compilador recusa"* — **medido, não recusa**: com a mordida decidindo a rota por uma
/// condição no `threads`, tirar a condição compila limpo e a rota em banda entrega `None` a cada
/// banda, ou seja **a onda de proa simplesmente não acontece**, em silêncio, com a suíte inteira verde
/// menos este gate (`displaced 0`). A cura foi mover a decisão para um `if let Some(b) = bite` que
/// **consome** a mordida — daí em diante nenhuma linha da rota em banda a alcança —, e este gate é o
/// que prova que ela continua sendo tomada.
#[test]
fn the_bow_waves_bite_keeps_the_serial_road() {
    const SIDE: u32 = 320;
    const RADIUS: f32 = 100.0;
    let s = loaded(RADIUS);
    let n = (SIDE as usize) * (SIDE as usize);
    let mut planes = Planes::new(n);
    let ground = vec![0.3f32; n];
    let mut plane = vec![0.0f32; n];
    let mut bite = crate::height_push::PushBite {
        ground: &ground,
        plane: &mut plane,
        displaced: 0.0,
    };
    let dab = swept(&s, RADIUS, [SIDE as f32 * 0.5, SIDE as f32 * 0.5]);
    let _ = accumulate_dab_height(&mut planes.fields(), SIDE, SIDE, &s, &dab, Some(&mut bite));
    assert!(
        bite.displaced > 0.0,
        "a fixture não exercita a mordida: displaced {}",
        bite.displaced
    );
}
