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

/// ⭐⭐⭐ **SOMAR AS FATIAS É A SEQUÊNCIA INTEIRA** — e é este gate que autoriza o quadro assente a
/// refinar em passagens em vez de pagar `32` raios de uma vez.
///
/// ⚠️ **Ao BIT**, e não «perto»: as duas rotas somam os mesmos `f32` pela mesma ordem por pixel
/// (cada fatia acrescenta os seus raios no mesmo lugar do acumulador). Uma barra de tolerância aqui
/// esconderia exactamente o defeito que ele mede — uma fatia que estratifica em si própria em vez
/// de no total.
#[test]
fn somar_as_fatias_da_a_sequencia_inteira() {
    let doc = cruz();
    let (reg, cam, g, _) = cena(&doc, 96, 54);
    const TOTAL: u32 = 8;

    let inteira = crate::occlusion(&doc, &reg, &cam, &g, TOTAL);

    // Oito passagens de um raio, como o quadro assente as vai correr.
    let mut acc = vec![0.0f32; g.hit.len()];
    for k in 0..TOTAL {
        let fatia = crate::occlusion_slice(&doc, &reg, &cam, &g, k, 1, TOTAL);
        for (a, f) in acc.iter_mut().zip(&fatia) {
            *a += f;
        }
    }

    let peca = (0..g.hit.len()).filter(|i| g.hit[*i]).count();
    assert!(peca > 500, "a fixtura não desenhou peça: {peca} px");
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / TOTAL as f32;
    for i in 0..g.hit.len() {
        if !g.hit[i] {
            continue;
        }
        assert_eq!(
            acc[i] * inv,
            inteira[i],
            "o pixel {i} diverge entre acumular e pagar tudo de uma vez — a fatia está a \
             estratificar em SI PRÓPRIA em vez de no total"
        );
    }
}

/// ⭐⭐ **Uma peça CONVEXA quase não se oclui, e uma peça de partes cruzadas oclui-se.**
///
/// ⚠️ **Não é `== 0` como o gate da sombra**, e a diferença é a lei: a oclusão mede quanto do
/// HEMISFÉRIO está tapado, e numa esfera a própria curvatura tapa uma parte — é isso que faz uma
/// esfera ter sombreado de contacto nenhum mas ambiente ligeiramente menor que `1`. *A barra sai da
/// medição da esfera, e o que o gate afirma é a ORDEM entre as duas peças.*
#[test]
fn a_oclusao_distingue_uma_esfera_de_uma_cruz() {
    // ⚠️⚠️ **A RÉGUA É A CAUDA, e a 1.ª redacção usou a MÉDIA.** Numa cruz de cilindros a fenda é
    // `~5 %` dos pixels visíveis: uma oclusão de `94 %` no fundo dela move a média de `1,000` para
    // `0,965`, e o gate lia isso como *«o passe devolve a mesma coisa para tudo»*. *Uma média sobre
    // a peça é o «extremo global» pelo lado de dentro.*
    let cauda = |doc: &ph2d_field::FieldDoc| -> (f32, f64) {
        let (reg, cam, g, _) = cena(doc, 160, 90);
        let oc = crate::occlusion(doc, &reg, &cam, &g, 16);
        let peca: Vec<usize> = (0..g.hit.len()).filter(|i| g.hit[*i]).collect();
        assert!(peca.len() > 1_000, "fixtura vazia");
        let min = peca.iter().fold(1.0f32, |m, i| m.min(oc[*i]));
        let escuros = peca.iter().filter(|i| oc[**i] < 0.8).count();
        (min, 100.0 * escuros as f64 / peca.len() as f64)
    };
    let (esf_min, esf_pct) = cauda(&esfera());
    let (cruz_min, cruz_pct) = cauda(&cruz());

    assert!(
        cruz_min < 0.35,
        "a fenda de tres cilindros cruzados tem de ficar FUNDA: o pixel mais ocluido le \
         {cruz_min:.3} (medido `0,062`)"
    );
    assert!(
        cruz_pct > 3.0,
        "so {cruz_pct:.1} % da cruz esta abaixo de 0,8 — a fenda tem de ser uma POPULACAO, nao um \
         pixel solto (medido `5,8 %`)"
    );
    // ⭐ O controlo, e é ele que apanha um estimador que oclui por CURVATURA: uma esfera é convexa,
    // logo **nenhum** raio dela pode bater na própria esfera.
    assert_eq!(
        esf_min, 1.0,
        "uma esfera e convexa e o pixel mais ocluido dela le {esf_min:.3} — o passe esta a ler a \
         curvatura da propria superficie como oclusor (ver a nota do `hardness` em `shadow.rs`)"
    );
    assert_eq!(esf_pct, 0.0, "e nenhum pixel dela pode estar abaixo de 0,8");
}

/// ⭐⭐⭐ **A OCLUSÃO ESCURECE O AMBIENTE E NÃO TOCA NA LÂMPADA.**
///
/// ⚠️ É o que separa oclusão de sujidade: um pixel numa fenda continua a receber a luz direta que
/// o alcança, e é isso que impede a peça de ficar acinzentada.
#[test]
fn a_oclusao_escurece_o_ceu_e_deixa_a_lampada() {
    use crate::{Lighting, PointLamp, Shadows, Surfaces, shade_render};

    let doc = cruz();
    let (reg, cam, g, luz) = cena(&doc, 96, 54);
    let oc = crate::occlusion(&doc, &reg, &cam, &g, 8);
    let tapado = (0..g.hit.len())
        .find(|i| g.hit[*i] && oc[*i] < 0.6)
        .expect("nenhum pixel ocluído na fixtura");

    let so = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = Surfaces {
        all: &so,
        owners: None,
    };
    let lamps: [crate::Lamp; 0] = [];
    // ⚠️ **A lâmpada é FRACA de propósito.** Com `40` o pixel SATURAVA (`765` de `765`) e a
    // oclusão não tinha onde se ver — *um gate cuja fixtura satura mede o TECTO DO BYTE, não o
    // produto*, e ele lia «não escureceu nada» sobre um passe que funcionava.
    let points = [PointLamp {
        world: luz,
        radiance_at_one: [0.6, 0.6, 0.6],
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
        u32::from(rgba[tapado * 4])
            + u32::from(rgba[tapado * 4 + 1])
            + u32::from(rgba[tapado * 4 + 2])
    };
    let mut com = Shadows::default();
    com.set_ambient(oc.clone());

    let claro = pinta(None);
    let escuro = pinta(Some(&com));
    assert!(
        escuro < claro,
        "a oclusão não escureceu nada: {claro} -> {escuro}"
    );

    // ⭐ O CONTROLO: sem céu nenhum, a oclusão não pode mudar um único byte — o que sobra é a
    // lâmpada, e ela não é dela. *Sem esta metade o gate acima passaria por a oclusão escurecer
    // o pixel INTEIRO, que é o defeito.*
    let sem_ceu = |sh: Option<&Shadows>| {
        shade_render(
            &g,
            &cam,
            &surfaces,
            &Lighting {
                lamps: &lamps,
                points: &points,
                sky: &CeuUniforme(0.0),
                shadows: sh,
            },
            ph2d_view_transform::Look::default(),
            [0, 0, 0, 0],
        )
    };
    assert_eq!(
        sem_ceu(None),
        sem_ceu(Some(&com)),
        "com o céu apagado a oclusão mudou pixels — ela está a multiplicar o RESULTADO em vez do \
         ambiente, e isso apaga a luz direta de dentro de uma fenda"
    );
}

/// ⭐⭐⭐ **O REFINAMENTO: `32` passagens, uma por vez, e a última é a sequência inteira.**
#[test]
fn o_refinamento_entrega_32_passagens_e_acaba_na_sequencia_inteira() {
    let doc = cruz();
    let (reg, cam, g, _) = cena(&doc, 96, 54);

    let mut sh = crate::Shadows::default();
    let mut vistos = Vec::new();
    let mut ultimo = Vec::new();
    let correu = crate::refine_occlusion(&doc, &reg, &cam, &g, &mut sh, |sh, k| {
        vistos.push(k);
        ultimo = (0..g.hit.len()).map(|i| sh.ambient_at(i)).collect();
        true
    });

    assert_eq!(correu, crate::OCCLUSION_PASSES);
    assert_eq!(
        vistos,
        (1..=crate::OCCLUSION_PASSES).collect::<Vec<_>>(),
        "as passagens têm de chegar em ordem e sem buracos — quem as conta é quem publica"
    );

    // ⭐ **E a última é EXACTAMENTE o que pagar tudo de uma vez daria** — é isto que torna o
    // refinamento uma forma de pagar, e não um resultado diferente.
    let inteira = crate::occlusion(&doc, &reg, &cam, &g, crate::OCCLUSION_PASSES);
    for i in 0..g.hit.len() {
        if g.hit[i] {
            assert_eq!(
                ultimo[i], inteira[i],
                "o pixel {i} diverge no fim do refinamento"
            );
        }
    }
}

/// ⚠️ **Em tempo de COMPILAÇÃO**: sem passagens por correr o gate abaixo não afirma nada — e um
/// `assert!` sobre dois `const` é uma asserção de valor constante, que o clippy recusa com razão.
const _: () = assert!(crate::OCCLUSION_PASSES > 3);

/// ⭐⭐⭐ **PARAR quando a mão volta a mexer** — sem isto o refinamento queima um núcleo por uma
/// imagem que já não se vê.
#[test]
fn o_refinamento_para_quando_lhe_dizem_para_parar() {
    let doc = cruz();
    let (reg, cam, g, _) = cena(&doc, 96, 54);

    let mut sh = crate::Shadows::default();
    let mut n = 0;
    let correu = crate::refine_occlusion(&doc, &reg, &cam, &g, &mut sh, |_, k| {
        n += 1;
        k < 3
    });
    assert_eq!(correu, 3, "ele continuou depois de lhe dizerem para parar");
    assert_eq!(n, 3, "e publicou passagens a mais");
}

/// ⭐⭐ **A imagem abre CLARA e vai escurecendo onde há oclusão** — nunca o contrário.
///
/// ⚠️ A média é sobre as passagens **já corridas**, e não sobre o total. Dividir pelo total faria a
/// peça abrir preta e clarear, que é o oposto do que uma acumulação deve parecer — e é o defeito
/// mais fácil de escrever aqui.
#[test]
fn o_refinamento_nunca_abre_a_peca_preta() {
    let doc = cruz();
    let (reg, cam, g, _) = cena(&doc, 96, 54);
    let peca: Vec<usize> = (0..g.hit.len()).filter(|i| g.hit[*i]).collect();

    let mut sh = crate::Shadows::default();
    let mut medias = Vec::new();
    crate::refine_occlusion(&doc, &reg, &cam, &g, &mut sh, |sh, _| {
        let m: f64 = peca
            .iter()
            .map(|i| f64::from(sh.ambient_at(*i)))
            .sum::<f64>()
            / peca.len() as f64;
        medias.push(m);
        true
    });
    assert!(
        medias[0] > 0.8,
        "a 1.ª passagem abriu a peça a {:.3} de céu — ela está a dividir pelo TOTAL em vez das \
         passagens corridas, e a peça abre preta",
        medias[0]
    );
    let fim = *medias.last().expect("passagens");
    assert!(
        (fim - medias[0]).abs() < 0.2,
        "o céu médio saltou de {:.3} para {fim:.3} ao longo do refinamento — ele deve AFINAR, \
         não mudar de nível",
        medias[0]
    );
}
