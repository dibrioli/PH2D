//! ⭐ **O que esta sonda DESENHA** — tudo o que produz uma IMAGEM.
//!
//! ⚠️ Os tres desenhos respondem a perguntas diferentes: o [`desenha`] mostra a
//! ARRUMACAO (uma cor por ilha, o duplicado a branco), o [`desenha_densidade`] mostra
//! a resolucao no QUADRADO, e o [`desenha_na_peca`] mostra-a na SUPERFICIE. ⛔ O do
//! meio e' estruturalmente incapaz de mostrar a igualacao de densidade, e o doc dele
//! diz porque' — *e' pesado pela area em `(u, v)` e o artista e' pesado pela
//! superficie*.

use super::medidas::{area3, conta, densidade, uv_area2, varre};
use ph2d_mesh::Mesh;

/// ⭐ **DESENHA O ATLAS** num `.ppm` — uma cor por ilha, os triângulos preenchidos.
///
/// ⚠️ É a única forma de o dono ver o que a arrumação fez. *Uma tabela de aproveitamento
/// não diz se as ilhas ficaram legíveis* — e esta linha já leu duas imagens ao contrário
/// por decidir por tabela.
pub(crate) fn desenha(atlas: &ph2d_uv_atlas::Atlas, mesh: &Mesh, caminho: &str, lado: usize) {
    let mut px = vec![[24u8, 24, 28]; lado * lado];
    let cor = |i: u32| -> [u8; 3] {
        // Uma roda de matizes: ilhas vizinhas nunca saem parecidas.
        let h = f32::from(u16::try_from(i % 12).unwrap_or(0)) / 12.0 * 6.0;
        let f = h - h.floor();
        let (a, b) = ((255.0 * f) as u8, (255.0 * (1.0 - f)) as u8);
        match h as u32 {
            0 => [255, a, 40],
            1 => [b, 255, 40],
            2 => [40, 255, a],
            3 => [40, b, 255],
            4 => [a, 40, 255],
            _ => [255, 40, b],
        }
    };
    // ⭐ O leque vem da PORTA — ver [`ph2d_uv_atlas::topo::triangulos`]. Esta sonda
    // escrevia-o duas vezes (aqui e no contador) e as duas concordavam por acaso.
    let tris = ph2d_uv_atlas::topo::triangulos(mesh);
    let mut n = vec![0u16; lado * lado];
    for t in &tris {
        let z = [
            atlas.uv[t[0] as usize],
            atlas.uv[t[1] as usize],
            atlas.uv[t[2] as usize],
        ];
        preenche(&mut px, lado, z, cor(atlas.ilha[t[0] as usize]));
        conta(&mut n, lado, z);
    }
    // ⛔⛔⛔ **O QUE FOI PINTADO DUAS VEZES SAI A BRANCO, e sem isto a imagem MENTE.**
    //
    // A 1.ª redacção desta sonda desenhava uma cor por ilha e mais nada — e uma ilha que
    // se pinta duas vezes desenha a MESMA cor por cima de si mesma, logo o defeito é
    // **invisível**: a peça do dono saía um bloco vermelho impecável com `34 %` da área
    // pintada em duplicado. *Uma imagem de smoke que não contém o fenómeno ensina que
    // não há fenómeno nenhum.*
    //
    // ⭐ E ela é o MESMO percurso que a linha `dobra` conta (o [`varre`]), logo a imagem e
    // o número são o mesmo facto — não duas medições que podem discordar.
    for (i, &c) in n.iter().enumerate() {
        if c > 1 {
            px[i] = [255, 255, 255];
        }
    }
    let mut out = format!("P6\n{lado} {lado}\n255\n").into_bytes();
    for p in &px {
        out.extend_from_slice(p);
    }
    if let Err(e) = std::fs::write(caminho, out) {
        println!("   (nao consegui escrever {caminho}: {e})");
    } else {
        println!("   desenho  {caminho}");
    }
}

/// Um triângulo cheio, em coordenadas `[0,1]²`.
pub(crate) fn preenche(px: &mut [[u8; 3]], lado: usize, t: [[f32; 2]; 3], c: [u8; 3]) {
    varre(lado, t, &mut |i| px[i] = c);
}

/// ⭐⭐⭐ **DESENHA A DENSIDADE** — a imagem em que «a resolução é a mesma em toda a
/// parte» se lê de uma vez.
///
/// Cada triângulo é pintado pela razão entre a densidade dele e a MEDIANA da peça:
/// cinzento é a mediana, azul é mais grosso, laranja é mais fino, e a saturação satura ao
/// **dobro / metade**. ⭐ *Uma peça uniforme sai CINZENTA de ponta a ponta* — e é por isso
/// que esta imagem responde à pergunta que a das ilhas não responde: aquela pinta uma cor
/// por ilha, logo ela é colorida por construção e não diz nada sobre resolução.
pub(crate) fn desenha_densidade(
    atlas: &ph2d_uv_atlas::Atlas,
    mesh: &Mesh,
    caminho: &str,
    lado: usize,
) {
    let Some(d) = densidade(atlas, mesh) else {
        return;
    };
    let mut px = vec![[24u8, 24, 28]; lado * lado];
    let vert = ph2d_uv_atlas::topo::vertices_dos_cantos(mesh);
    let pos = mesh.positions();
    for t in &ph2d_uv_atlas::topo::triangulos(mesh) {
        let z = [
            atlas.uv[t[0] as usize],
            atlas.uv[t[1] as usize],
            atlas.uv[t[2] as usize],
        ];
        let mundo = area3(
            pos[vert[t[0] as usize] as usize],
            pos[vert[t[1] as usize] as usize],
            pos[vert[t[2] as usize] as usize],
        );
        let plano = uv_area2(z[0], z[1], z[2]).abs();
        if mundo <= 0.0 || plano <= 0.0 {
            continue;
        }
        let razao = (plano / mundo).sqrt() / d.p50.max(1.0e-12);
        let l = razao.log2().clamp(-1.0, 1.0);
        let alvo: [f64; 3] = if l < 0.0 {
            [50.0, 90.0, 235.0]
        } else {
            [255.0, 150.0, 40.0]
        };
        let k = l.abs();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let c = [
            (170.0 + (alvo[0] - 170.0) * k) as u8,
            (170.0 + (alvo[1] - 170.0) * k) as u8,
            (170.0 + (alvo[2] - 170.0) * k) as u8,
        ];
        preenche(&mut px, lado, z, c);
    }
    let mut out = format!("P6\n{lado} {lado}\n255\n").into_bytes();
    for p in &px {
        out.extend_from_slice(p);
    }
    if let Err(e) = std::fs::write(caminho, out) {
        println!("   (nao consegui escrever {caminho}: {e})");
    } else {
        println!("   desenho  {caminho}");
    }
}

/// ⭐⭐⭐ **DESENHA A DENSIDADE NA PEÇA** — e não no quadrado.
///
/// ⛔⛔⛔ **A imagem do ATLAS é estruturalmente incapaz de mostrar esta cura, e isso foi
/// MEDIDO.** Um desenho do quadrado é pesado pela área em `(u, v)`; o que o artista sente
/// é pesado pela área da SUPERFÍCIE — e a igualação existe precisamente para mudar a
/// relação entre as duas. Medido pixel a pixel sobre o par com/sem: o desvio MÉDIO à cor
/// neutra melhora (`12,2 → 11,4`) e a fracção de píxeis fortemente coloridos **PIORA**
/// (`3,7 → 6,1 %`), porque uma peça grossa ocupa pouco quadrado, a cura fá-la CRESCER, e
/// o que ela ainda tenha de errado passa a cobrir nove vezes mais píxeis. *A imagem não
/// mente sobre a lei: ela mede a grandeza errada.*
///
/// Esta desenha a escultura numa vista de três quartos, uma cor por triângulo pela mesma
/// rampa (cinzento = a mediana, azul = mais grosso, laranja = mais fino, a saturar no
/// dobro/metade), com o relevo a entrar só no BRILHO — *a matiz continua a ser a
/// medição, e o sombreado é o que faz a forma ler-se.*
pub(crate) fn desenha_na_peca(
    atlas: &ph2d_uv_atlas::Atlas,
    mesh: &Mesh,
    caminho: &str,
    lado: usize,
) {
    let Some(d) = densidade(atlas, mesh) else {
        return;
    };
    let vert = ph2d_uv_atlas::topo::vertices_dos_cantos(mesh);
    let pos = mesh.positions();
    // A vista: guinada de 35° e arfagem de 25°, ortográfica.
    let (cy, sy) = (35.0f64.to_radians().cos(), 35.0f64.to_radians().sin());
    let (cp, sp) = (25.0f64.to_radians().cos(), 25.0f64.to_radians().sin());
    let mut centro = [0.0f64; 3];
    for p in pos {
        for k in 0..3 {
            centro[k] += f64::from(p[k]);
        }
    }
    for c in &mut centro {
        *c /= pos.len().max(1) as f64;
    }
    let ve = |p: [f32; 3]| -> [f64; 3] {
        let q = [
            f64::from(p[0]) - centro[0],
            f64::from(p[1]) - centro[1],
            f64::from(p[2]) - centro[2],
        ];
        let (x, z) = (cy * q[0] + sy * q[2], -sy * q[0] + cy * q[2]);
        let (y, z) = (cp * q[1] - sp * z, sp * q[1] + cp * z);
        [x, y, z]
    };
    let mut raio = 1.0e-9f64;
    for p in pos {
        let v = ve(*p);
        raio = raio.max(v[0].abs()).max(v[1].abs());
    }
    let escala = lado as f64 * 0.46 / raio;
    let meio = lado as f64 * 0.5;

    let mut px = vec![[24u8, 24, 28]; lado * lado];
    let mut zb = vec![f64::MIN; lado * lado];
    for t in &ph2d_uv_atlas::topo::triangulos(mesh) {
        let p3: Vec<[f64; 3]> = t
            .iter()
            .map(|&c| ve(pos[vert[c as usize] as usize]))
            .collect();
        let mundo = area3(
            pos[vert[t[0] as usize] as usize],
            pos[vert[t[1] as usize] as usize],
            pos[vert[t[2] as usize] as usize],
        );
        let plano = uv_area2(
            atlas.uv[t[0] as usize],
            atlas.uv[t[1] as usize],
            atlas.uv[t[2] as usize],
        )
        .abs();
        if mundo <= 0.0 || plano <= 0.0 {
            continue;
        }
        let razao = (plano / mundo).sqrt() / d.p50.max(1.0e-12);
        let l = razao.log2().clamp(-1.0, 1.0);
        let alvo: [f64; 3] = if l < 0.0 {
            [50.0, 90.0, 235.0]
        } else {
            [255.0, 150.0, 40.0]
        };
        let k = l.abs();
        // A normal da face na vista, só para o brilho.
        let u = [
            p3[1][0] - p3[0][0],
            p3[1][1] - p3[0][1],
            p3[1][2] - p3[0][2],
        ];
        let v = [
            p3[2][0] - p3[0][0],
            p3[2][1] - p3[0][1],
            p3[2][2] - p3[0][2],
        ];
        let n = [
            u[1].mul_add(v[2], -(u[2] * v[1])),
            u[2].mul_add(v[0], -(u[0] * v[2])),
            u[0].mul_add(v[1], -(u[1] * v[0])),
        ];
        let len = n[0]
            .mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2]))
            .sqrt()
            .max(1.0e-18);
        let luz = (0.45 + 0.55 * (n[2] / len).abs()).clamp(0.0, 1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let cor = [
            ((170.0 + (alvo[0] - 170.0) * k) * luz) as u8,
            ((170.0 + (alvo[1] - 170.0) * k) * luz) as u8,
            ((170.0 + (alvo[2] - 170.0) * k) * luz) as u8,
        ];
        let e: Vec<[f64; 3]> = p3
            .iter()
            .map(|q| [meio + q[0] * escala, meio - q[1] * escala, q[2]])
            .collect();
        preenche_z(&mut px, &mut zb, lado, [e[0], e[1], e[2]], cor);
    }
    let mut out = format!("P6\n{lado} {lado}\n255\n").into_bytes();
    for p in &px {
        out.extend_from_slice(p);
    }
    if let Err(e) = std::fs::write(caminho, out) {
        println!("   (nao consegui escrever {caminho}: {e})");
    } else {
        println!("   desenho  {caminho}");
    }
}

/// Um triângulo com profundidade, pelas baricêntricas.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub(crate) fn preenche_z(
    px: &mut [[u8; 3]],
    zb: &mut [f64],
    lado: usize,
    t: [[f64; 3]; 3],
    c: [u8; 3],
) {
    let (x0, x1) = (
        t.iter().fold(f64::MAX, |a, p| a.min(p[0])).floor().max(0.0) as usize,
        (t.iter().fold(f64::MIN, |a, p| a.max(p[0])).ceil() as usize).min(lado - 1),
    );
    let (y0, y1) = (
        t.iter().fold(f64::MAX, |a, p| a.min(p[1])).floor().max(0.0) as usize,
        (t.iter().fold(f64::MIN, |a, p| a.max(p[1])).ceil() as usize).min(lado - 1),
    );
    let den = (t[1][0] - t[0][0]) * (t[2][1] - t[0][1]) - (t[2][0] - t[0][0]) * (t[1][1] - t[0][1]);
    if den.abs() < 1.0e-12 {
        return;
    }
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
            let a = ((t[1][0] - fx) * (t[2][1] - fy) - (t[2][0] - fx) * (t[1][1] - fy)) / den;
            let b = ((t[2][0] - fx) * (t[0][1] - fy) - (t[0][0] - fx) * (t[2][1] - fy)) / den;
            let g = 1.0 - a - b;
            if a < 0.0 || b < 0.0 || g < 0.0 {
                continue;
            }
            let z = a * t[0][2] + b * t[1][2] + g * t[2][2];
            let i = y * lado + x;
            if z > zb[i] {
                zb[i] = z;
                px[i] = c;
            }
        }
    }
}
