//! Os gates da **sombra** — ver [`crate::shadow`] para o preço e para as três cercas.

use crate::{Gbuffer, Orbit, Shadows, shadow_pass, trace};
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;

/// Três cilindros cruzados: as partes tapam-se umas às outras. ⚠️ **Uma esfera é CONVEXA** e não se
/// auto-sombreia — medir nela responderia «não há sombra» sobre um caso em que não há nada a fazer.
fn cruz() -> FieldDoc {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let cyl = |rot: [f32; 4]| {
        ph2d_field_eval::leaf(
            Primitive::Cylinder {
                radius: 0.22,
                half_height: 0.78,
                round: 0.05,
                chamfer: 0.0,
            },
            Xform {
                rotation: rot,
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            cyl([0.0, 0.0, 0.0, 1.0]),
            cyl([s, 0.0, 0.0, s]),
            cyl([0.0, 0.0, s, s]),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(ph2d_field::Blend::Exact { radius: 0.12 }),
                    children: vec![NodeId(0), NodeId(1), NodeId(2)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(3),
    )
    .expect("a cruz")
}

/// Um céu **uniforme** — a fixtura precisa de que o pixel tapado tenha de onde continuar a
/// receber luz, e um céu constante é o mais simples que o exprime.
struct CeuUniforme(f32);

impl ph2d_material::Environment for CeuUniforme {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        [self.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        [self.0; 3]
    }
}

fn esfera() -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.8 },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("a esfera")
}

/// A lâmpada onde a wave da §25 a põe, e o G-buffer daquela peça.
fn cena(doc: &FieldDoc, w: u32, h: u32) -> (Registry, Orbit, Gbuffer, [f32; 3]) {
    let reg = Registry::new();
    let cam = Orbit::default();
    let g = trace(doc, &reg, &cam, w, h);
    let (right, up, toward_eye) = cam.basis();
    let ecra = [-0.5566703_f32, 0.6634139, 0.5];
    let r = 2.0 * cam.half_extent;
    let luz = [0, 1, 2].map(|i| {
        cam.target[i] + r * (ecra[0] * right[i] + ecra[1] * up[i] + ecra[2] * toward_eye[i])
    });
    (reg, cam, g, luz)
}

/// ⭐⭐⭐ **A peça tapa-se a si própria** — que é a razão de a wave existir.
///
/// ⚠️ **A barra é uma FRACÇÃO da peça, não um pixel**: um gate que se contentasse com *«algum pixel
/// escureceu»* passaria com um único pixel de ruído numérico na silhueta.
#[test]
fn uma_peca_que_se_tapa_a_si_propria_tem_pixels_tapados() {
    let doc = cruz();
    let (reg, cam, g, luz) = cena(&doc, 320, 180);
    let s = shadow_pass(&doc, &reg, &cam, &g, &[luz]);

    // ⚠️⚠️ **O DENOMINADOR é a PEÇA, e a sonda do relógio usa outro** — ela divide pela população
    // que VÊ a luz, que é `44,8 %` da peça. Os mesmos `1 777` pixels leem-se `11,7 %` aqui e
    // `27,3 %` lá, e a 1.ª redacção deste gate copiou o número da sonda para o denominador errado.
    // *Uma fracção sem o denominador escrito ao lado dela não é um número.*
    let peca = (0..g.hit.len()).filter(|i| g.hit[*i]).count();
    assert!(
        peca > 5_000,
        "a fixtura não desenhou peça nenhuma: {peca} px"
    );
    let tapados = (0..g.hit.len())
        .filter(|i| g.hit[*i] && s.at(0, *i) < 0.99)
        .count();
    let frac = 100.0 * tapados as f64 / peca as f64;
    assert!(
        (5.0..20.0).contains(&frac),
        "os pixels com sombra são {frac:.1} % da PEÇA ({tapados} de {peca}) — a medição de \
         2026-09-14 dá `10,4 %`, e a barra é larga porque a marcha sai do documento e não de um \
         golden"
    );
}

/// ⭐⭐ **Uma peça CONVEXA não se sombreia** — o controlo que impede o gate de cima de passar por
/// uma sombra que o passe inventa em todo lado.
#[test]
fn uma_esfera_nao_se_tapa_a_si_propria() {
    let doc = esfera();
    let (reg, cam, g, luz) = cena(&doc, 320, 180);
    let s = shadow_pass(&doc, &reg, &cam, &g, &[luz]);

    let peca = (0..g.hit.len()).filter(|i| g.hit[*i]).count();
    assert!(
        peca > 1_000,
        "a fixtura não desenhou peça nenhuma: {peca} px"
    );
    let tapados = (0..g.hit.len())
        .filter(|i| g.hit[*i] && s.at(0, *i) < 0.99)
        .count();
    // ⭐⭐⭐ **EXACTAMENTE zero, e não «quase».** Não é uma barra escolhida: é o que a geometria
    // exige — um corpo convexo não tem por onde se tapar a si próprio. É este gate que apanha a
    // ACNE, e ele foi VERMELHO a `10,5 %` antes de o raio passar a erguer-se pela normal
    // ([`crate::shadow`]). ⛔ Uma tolerância aqui apagaria exactamente o defeito que ele mede.
    assert_eq!(
        tapados, 0,
        "uma esfera é convexa e {tapados} dos {peca} pixels dela vieram sombreados — o passe está \
         a inventar sombra, e a sonda `probe_de_onde_vem_a_sombra_da_esfera` diz se é acerto DURO \
         (acne: o raio parte rasante e volta a entrar na peça) ou penumbra MOLE"
    );
}

/// ⭐⭐⭐ **A SOMBRA MULTIPLICA A LUZ QUE CHEGA, e não o resultado.**
///
/// ⚠️ Um pixel inteiramente tapado **não fica preto**: ele continua a reflectir o céu. Este gate é o
/// que separa a conta certa (dentro do `chega`) da conta fácil (um multiplicador no fim), e as duas
/// dão a MESMA imagem numa cena sem céu — é por isso que a fixtura tem céu.
#[test]
fn um_pixel_tapado_continua_a_reflectir_o_ceu() {
    use crate::{Lighting, PointLamp, Surfaces, shade_render};

    let doc = cruz();
    let (reg, cam, g, luz) = cena(&doc, 160, 90);
    let s = shadow_pass(&doc, &reg, &cam, &g, &[luz]);
    // ⚠️⚠️ **`== 0.0`, e não `< 0,05`** — e a diferença é o gate inteiro. Com a 1.ª redacção, a
    // mutação que move a visibilidade para o RESULTADO **SOBREVIVEU**: num pixel a `vis = 0,03`,
    // `0,03 × (céu + lâmpada)` ainda é maior que zero, e a asserção *«não ficou preto»* passava
    // sobre o defeito que ela existe para apanhar. *Só um pixel INTEIRAMENTE tapado separa
    // «multiplicar a luz que chega» de «multiplicar o resultado»* — ali um dos dois dá preto.
    let tapado = (0..g.hit.len())
        .find(|i| g.hit[*i] && s.at(0, *i) == 0.0)
        .expect("nenhum pixel inteiramente tapado na fixtura");

    let so = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = Surfaces {
        all: &so,
        owners: None,
    };
    let lamps: [crate::Lamp; 0] = [];
    let points = [PointLamp {
        world: luz,
        radiance_at_one: [40.0, 40.0, 40.0],
    }];
    let pinta = |sh: Option<&Shadows>| {
        let rgba = shade_render(
            &g,
            &cam,
            &surfaces,
            &Lighting {
                lamps: &lamps,
                points: &points,
                sky: &CeuUniforme(0.35),
                shadows: sh,
            },
            ph2d_view_transform::Look::default(),
            [0, 0, 0, 0],
        );
        [rgba[tapado * 4], rgba[tapado * 4 + 1], rgba[tapado * 4 + 2]]
    };
    let claro = pinta(None);
    let escuro = pinta(Some(&s));
    let soma = |c: [u8; 3]| u32::from(c[0]) + u32::from(c[1]) + u32::from(c[2]);
    assert!(
        soma(escuro) < soma(claro),
        "a sombra não escureceu o pixel tapado: {claro:?} -> {escuro:?}"
    );
    assert!(
        soma(escuro) > 0,
        "o pixel tapado saiu PRETO ({escuro:?}) — a visibilidade foi aplicada ao resultado em vez \
         de à luz que chega, e apagou também o céu"
    );
}

/// ⭐⭐ **Sem o passe, a imagem é a de sempre — AO BIT.**
#[test]
fn sem_o_passe_a_imagem_e_byte_identica() {
    use crate::{Lighting, PointLamp, Surfaces, shade_render};

    let doc = cruz();
    let (reg, cam, g, luz) = cena(&doc, 96, 54);
    let _ = reg;
    let so = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = Surfaces {
        all: &so,
        owners: None,
    };
    let lamps: [crate::Lamp; 0] = [];
    let points = [PointLamp {
        world: luz,
        radiance_at_one: [40.0, 40.0, 40.0],
    }];
    let vazio = Shadows::default();
    let pinta = |sh: Option<&Shadows>| {
        shade_render(
            &g,
            &cam,
            &surfaces,
            &Lighting {
                lamps: &lamps,
                points: &points,
                sky: &CeuUniforme(0.35),
                shadows: sh,
            },
            ph2d_view_transform::Look::default(),
            [0, 0, 0, 0],
        )
    };
    assert_eq!(
        pinta(None),
        pinta(Some(&vazio)),
        "um passe VAZIO tem de ser indistinguível de não haver passe — o `Shadows::at` devolve \
         `1,0` fora de alcance exactamente para isto"
    );
}

/// Sonda: numa esfera, a sombra que aparece é ACERTO ou é o termo de penumbra?
#[test]
#[ignore = "sonda"]
fn probe_de_onde_vem_a_sombra_da_esfera() {
    for (nome, doc) in [("esfera", esfera()), ("cruz", cruz())] {
        let (reg, cam, g, luz) = cena(&doc, 320, 180);
        let s = shadow_pass(&doc, &reg, &cam, &g, &[luz]);
        let peca: Vec<usize> = (0..g.hit.len()).filter(|i| g.hit[*i]).collect();
        let duro = peca.iter().filter(|i| s.at(0, **i) <= 0.0).count();
        let mole = peca
            .iter()
            .filter(|i| s.at(0, **i) > 0.0 && s.at(0, **i) < 0.99)
            .count();
        println!(
            "{nome:7}: peça {} · acerto DURO {duro} ({:.1} %) · penumbra MOLE {mole} ({:.1} %)",
            peca.len(),
            100.0 * duro as f64 / peca.len() as f64,
            100.0 * mole as f64 / peca.len() as f64,
        );
    }
}
