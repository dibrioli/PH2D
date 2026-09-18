//! ⏱️ **O TESTE NULO DAS TRÊS MÍDIAS** — filho do [`super`] para herdar as fixturas dele.
//!
//! ⭐⭐⭐ **A pergunta é OUTRA e por isso o ficheiro é outro.** O pai mede *que arte cada mídia
//! desenha*; aqui mede-se *se as três a dobram IGUAL* — a mesma silhueta, o mesmo tamanho, o mesmo
//! esqueleto, e o certo é **não haver diferença nenhuma**. ⚠️ *Uma régua cujo valor bom é zero é a
//! mais dura que existe, porque não há nada para interpretar.*
//!
//! ⚠️ Ele é filho e não irmão porque as fixturas de que precisa — `pedacos_de`, `espalha`,
//! `malhas_em_repouso` — são do pai, e **duplicá-las tornaria o teste nulo uma segunda resposta à
//! mesma pergunta**.

use super::*;

/// A dobra por junta que este teste nulo usa, em graus.
///
/// ⚠️ **Ela NÃO espelha a constante da cena, de propósito** — a `ph2d-app-vec` depende desta crate e
/// não o contrário, logo um espelho aqui seria um número escrito à mão a envelhecer no dia em que o
/// smoke mudasse (e ele mudou: `13 → 25` em 2026-09-18). O que este teste precisa é de **um ângulo
/// que dobre de verdade**, e a lei que governa quanto ele pode dobrar tem gate próprio na
/// [`super::sonda_da_dobra`].
const DOBRA_DA_CENA: f32 = 25.0;

/// O braço do teste nulo: `240 × 60 px`, opaco — a mesma silhueta para as três mídias.
fn braco_px() -> ([u32; 2], Vec<u8>) {
    let (w, h) = (240u32, 60u32);
    let mut rgba = vec![0u8; (w * h) as usize * 4];
    for p in rgba.as_chunks_mut::<4>().0 {
        p[3] = 255;
    }
    ([w, h], rgba)
}

/// A folha de 4 braços iguais — a MESMA silhueta em cada célula.
fn folha_de_bracos() -> ([u32; 2], Vec<u8>) {
    let ([w, h], _) = braco_px();
    let total = w * 4;
    let mut rgba = vec![0u8; (total * h) as usize * 4];
    for p in rgba.as_chunks_mut::<4>().0 {
        p[3] = 255;
    }
    ([total, h], rgba)
}

/// Onde a malha `m` põe o ponto de UV `uv` do QUAD DA SPRITE, em metros locais.
///
/// ⚠️ **`frac` leva a UV do pedaço para a UV da SPRITE**: num 9-slice cada pedaço tem a UV dele a ir
/// de `0` a `1` sobre o quad DELE, e sem esta conversão os nove viveriam em nove espaços diferentes.
fn onde_poe(m: &SpriteMesh, frac: [f32; 4], uv: [f32; 2]) -> Option<[f32; 2]> {
    let (du, dv) = (frac[2] - frac[0], frac[3] - frac[1]);
    m.tris.iter().find_map(|t| {
        let [a, b, c] = t.map(|i| {
            let u = m.uv[i as usize];
            [frac[0] + u[0] * du, frac[1] + u[1] * dv]
        });
        let det = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if det == 0.0 {
            return None;
        }
        let l1 = ((uv[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (uv[1] - a[1])) / det;
        let l2 = ((b[0] - a[0]) * (uv[1] - a[1]) - (uv[0] - a[0]) * (b[1] - a[1])) / det;
        let l0 = 1.0 - l1 - l2;
        if l0 < -1e-5 || l1 < -1e-5 || l2 < -1e-5 {
            return None;
        }
        let [pa, pb, pc] = t.map(|i| m.local[i as usize]);
        Some([
            l0 * pa[0] + l1 * pb[0] + l2 * pc[0],
            l0 * pa[1] + l1 * pb[1] + l2 * pc[1],
        ])
    })
}

/// Monta um braço da mídia pedida, dobra a corrente, e devolve as malhas desenhadas com a fracção
/// de cada uma.
///
/// `qual`: `0` simples · `1` folha `4×1` no quadro `1` · `2` 9-slice no tamanho INTRÍNSECO.
fn braco_desenhado(qual: u8, graus: f32) -> Vec<(SpriteMesh, [f32; 4])> {
    let ([w, h], arte) = if qual == 1 {
        folha_de_bracos()
    } else {
        braco_px()
    };
    let celula = [f32::from(240u16) / PPM, f32::from(60u16) / PPM];
    let s = sprite(celula[0], celula[1], 0.0, 0.0);
    let mut sim = SimWorld::default();
    // ⚠️ Três ossos a cobrir o braço, como a cena faz — uma corrente que não o cobre prende meia arte.
    let mut pai = None;
    let mut ossos = Vec::new();
    for k in 0..3 {
        let passo = f64::from(celula[0]) / 3.0;
        let x0 = -f64::from(celula[0]) / 2.0;
        let osso = crate::bone::create(
            &mut sim,
            pai,
            [x0 + passo * f64::from(k), 0.0],
            [x0 + passo * f64::from(k + 1), 0.0],
        )
        .expect("osso");
        let e = Entity::from_bits(osso);
        ossos.push(e);
        pai = Some(e);
    }
    let e = sim.world_mut().spawn((Transform::IDENTITY, s)).id();
    if qual == 1 {
        sim.world_mut().entity_mut(e).insert(ph2d_ecs::SpriteGrid {
            hframes: 4,
            vframes: 1,
            frame: 1,
        });
    }
    assert!(crate::skin_live::bind_image(
        &mut sim,
        e,
        &arte,
        [w, h],
        PPM,
        ph2d_poly2d::GridOptions::default(),
        ossos.first().copied(),
    ));
    for osso in ossos.iter().skip(1) {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(*osso) {
            t.rotation += graus.to_radians();
        }
    }
    let mut present = PresentWorld::new();
    let base = instancia_de(&s);
    let mut ps: Vec<(Entity, [f32; 4])> = Vec::new();
    if qual == 2 {
        for (k, (ri, f)) in pedacos_de(&s, base, [24.0, 12.0, 24.0, 12.0], celula, [240.0, 60.0])
            .into_iter()
            .enumerate()
        {
            ps.push((espalha(present.world_mut(), e, ri, f, k), f.frac));
        }
    } else {
        let mut ri = base;
        if qual == 1 {
            ri.atlas_uv = [0.25, 0.0, 0.5, 1.0];
        }
        ps.push((
            present.world_mut().spawn((SimRef(e), ri)).id(),
            [0.0, 0.0, 1.0, 1.0],
        ));
    }
    attach_skin_meshes(&sim, &mut present, PPM, &[]);
    ps.iter()
        .filter_map(|(p, f)| {
            present
                .world()
                .get::<SpriteMesh>(*p)
                .cloned()
                .map(|m| (m, *f))
        })
        .collect()
}

/// ⭐⭐⭐ **AS TRÊS MÍDIAS PÕEM CADA TEXEL NO MESMO SÍTIO** — o TESTE NULO que o dono pediu
/// (2026-09-18: *«do mesmo tamanho e mesma largura, para eu ver se se dobram igual uma a outra»*).
///
/// Mesma arte, mesmo tamanho, mesmo esqueleto, mesma dobra: a sprite simples, a folha `4×1` no
/// quadro `1` e o 9-slice **no tamanho INTRÍNSECO** (onde ele é a identidade geométrica). ⇒ para
/// cada ponto da arte, as três TÊM de o pôr no mesmo sítio.
///
/// ⚠️ **A barra é UM PIXEL de arte** (`1/240` do braço). Ela não é zero de propósito: as três malhas
/// são traçadas e refinadas por caminhos diferentes (a da folha vem da UNIÃO dos quadros, a do
/// 9-slice é CORTADA em nove), e um afim por triângulo sobre malhas diferentes não dá os mesmos
/// bits. *O que se afirma é que a diferença é invisível, não que ela não existe.*
///
/// ⛔ **A régua é o PRODUTO** (`attach_skin_meshes` pela porta de sempre), e o espaço comum é a UV
/// da SPRITE — sem a conversão pela fracção, os nove pedaços viveriam em nove espaços diferentes.
#[test]
fn as_tres_midias_dobram_igual() {
    let casos: Vec<Vec<(SpriteMesh, [f32; 4])>> =
        (0..3).map(|q| braco_desenhado(q, DOBRA_DA_CENA)).collect();
    for (k, c) in casos.iter().enumerate() {
        assert!(!c.is_empty(), "a midia {k} nao desenhou nada");
    }
    let mut pior = 0.0_f32;
    let mut medidos = 0;
    // ⚠️ Amostras no INTERIOR: a fronteira de um pedaço pertence a dois, e ali um empate de último
    // bit escolhe um ou outro — o que se mede é a arte, não o desempate.
    for iu in 1..40 {
        for iv in 1..10 {
            let uv = [iu as f32 / 40.0, iv as f32 / 10.0];
            let onde: Vec<[f32; 2]> = casos
                .iter()
                .filter_map(|c| c.iter().find_map(|(m, f)| onde_poe(m, *f, uv)))
                .collect();
            if onde.len() < 3 {
                continue;
            }
            medidos += 1;
            for o in &onde[1..] {
                pior = pior.max((o[0] - onde[0][0]).hypot(o[1] - onde[0][1]));
            }
        }
    }
    assert!(medidos > 300, "so' {medidos} amostras: a regua mede pouco");
    let um_pixel = 240.0 / PPM / 240.0;
    assert!(
        pior < um_pixel,
        "as tres midias poem o mesmo texel a ate' {pior:.6} m de distancia (um pixel de arte mede \
         {um_pixel:.6} m) — elas NAO dobram igual"
    );
    println!(
        "{medidos} amostras | pior desvio {pior:.6} m = {:.2} pixels de arte",
        pior / um_pixel
    );
}

#[path = "sonda_da_dobra_tests.rs"]
mod sonda_da_dobra;

#[path = "sonda_do_rig_partilhado_tests.rs"]
mod sonda_do_rig_partilhado;
