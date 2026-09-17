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
/// ⚠️ **Ao BIT para fatias de UMA direcção**, que é a forma que o refinamento tem: as duas rotas
/// somam os mesmos `f32` pela mesma ordem por pixel, logo ficam com a mesma **dobra à esquerda**.
/// Uma barra de tolerância nessa metade esconderia exactamente o defeito que ele mede — uma fatia
/// que estratifica em si própria em vez de no total.
///
/// ⛔⛔ **E uma partição DESIGUAL já não é igual ao bit — medido em 2026-09-17, aqui e na irmã do
/// ricochete.** `(a₀+a₁+a₂) + (a₃+…+a₇)` é uma dobra em ÁRVORE e a sequência inteira é uma dobra à
/// ESQUERDA; a diferença é só associatividade, e a barra dessa metade é **derivada do número de
/// parcelas** (`(total − 1) · f32::EPSILON`). *A frase que estava aqui — e a do
/// [`crate::occlusion_slice`] — dizia «somar todas as fatias dá EXACTAMENTE», o que era verdade
/// sobre a única partição que este gate corria e falso sobre uma partição qualquer.*
#[test]
fn somar_as_fatias_da_a_sequencia_inteira() {
    let doc = cruz();
    let (reg, cam, g, _) = cena(&doc, 96, 54);
    const TOTAL: u32 = 8;

    let inteira = crate::occlusion(&doc, &reg, &cam, &g, TOTAL);

    // Oito passagens de um cone, como o quadro assente as vai correr.
    let mut acc = crate::ConeSlice {
        sum: vec![0.0f32; g.hit.len()],
        weight: vec![0.0f32; g.hit.len()],
    };
    for k in 0..TOTAL {
        acc.add(&crate::occlusion_slice(&doc, &reg, &cam, &g, k, 1, TOTAL));
    }
    let somadas = acc.average(&g.hit);

    let peca = (0..g.hit.len()).filter(|i| g.hit[*i]).count();
    assert!(peca > 500, "a fixtura não desenhou peça: {peca} px");
    for i in 0..g.hit.len() {
        if !g.hit[i] {
            continue;
        }
        assert_eq!(
            somadas[i], inteira[i],
            "o pixel {i} diverge entre acumular e pagar tudo de uma vez — a fatia está a \
             estratificar em SI PRÓPRIA em vez de no total"
        );
    }

    // ── a outra metade: uma partição DESIGUAL, onde só a associatividade pode divergir ─────────
    let mut desigual = crate::ConeSlice {
        sum: vec![0.0f32; g.hit.len()],
        weight: vec![0.0f32; g.hit.len()],
    };
    desigual.add(&crate::occlusion_slice(&doc, &reg, &cam, &g, 0, 3, TOTAL));
    desigual.add(&crate::occlusion_slice(&doc, &reg, &cam, &g, 3, 5, TOTAL));
    let tres_e_cinco = desigual.average(&g.hit);
    let mut pior_rel = 0.0f32;
    for i in 0..g.hit.len() {
        if !g.hit[i] {
            continue;
        }
        let m = tres_e_cinco[i].abs().max(inteira[i].abs());
        if m > 1e-6 {
            pior_rel = pior_rel.max((tres_e_cinco[i] - inteira[i]).abs() / m);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let associatividade = (TOTAL - 1) as f32 * f32::EPSILON;
    println!(
        "  3+5 contra 8: pior relativo {pior_rel:.3e} (a associatividade permite \
         {associatividade:.3e})"
    );
    assert!(
        pior_rel <= associatividade,
        "3 + 5 cones divergem de 8 por {pior_rel:.3e}, mais do que a associatividade da soma em \
         f32 explica ({associatividade:.3e}) — isso já não é arredondamento, é outra amostra"
    );
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
    // ⚠️⚠️ **AS BARRAS FORAM RE-CALIBRADAS na §31, e a versão anterior delas media o DEFEITO.**
    // Com o leque plano a fenda lia `mín 0,062` e `5,8 %` abaixo de `0,8` — números que vinham do
    // enviesamento, não da peça. Com o hemisfério varrido e a suavização, os verdadeiros são
    // `mín 0,750` e `12,2 %` abaixo de `0,9`. *Uma barra calibrada sobre um estimador enviesado
    // defende o enviesamento.*
    let cauda = |doc: &ph2d_field::FieldDoc| -> (f32, f64) {
        let (reg, cam, g, _) = cena(doc, 160, 90);
        let cru = crate::occlusion(doc, &reg, &cam, &g, crate::OCCLUSION_PASSES);
        let oc = crate::blur_occlusion(&g, &cru);
        let peca: Vec<usize> = (0..g.hit.len()).filter(|i| g.hit[*i]).collect();
        assert!(peca.len() > 1_000, "fixtura vazia");
        let min = peca.iter().fold(1.0f32, |m, i| m.min(oc[*i]));
        let escuros = peca.iter().filter(|i| oc[**i] < 0.9).count();
        (min, 100.0 * escuros as f64 / peca.len() as f64)
    };
    let (esf_min, esf_pct) = cauda(&esfera());
    let (cruz_min, cruz_pct) = cauda(&cruz());

    assert!(
        cruz_min < 0.85,
        "a fenda de tres cilindros cruzados tem de escurecer: o pixel mais ocluido le \
         {cruz_min:.3} (medido `0,750`)"
    );
    assert!(
        cruz_pct > 6.0,
        "so {cruz_pct:.1} % da cruz esta abaixo de 0,9 — a fenda tem de ser uma POPULACAO, nao um \
         pixel solto (medido `12,2 %`)"
    );
    // ⭐ O controlo, e é ele que apanha um estimador que oclui por CURVATURA: uma esfera é convexa,
    // logo **nenhum** raio dela pode bater na própria esfera.
    assert_eq!(
        esf_min, 1.0,
        "uma esfera e convexa e o pixel mais ocluido dela le {esf_min:.3} — o passe esta a ler a \
         curvatura da propria superficie como oclusor (ver a nota do `hardness` em `shadow.rs`)"
    );
    assert_eq!(esf_pct, 0.0, "e nenhum pixel dela pode estar abaixo de 0,9");
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
    let cru = crate::occlusion(&doc, &reg, &cam, &g, crate::OCCLUSION_PASSES);
    let oc = crate::blur_occlusion(&g, &cru);
    // ⚠️ **`0,85` e não `0,6`**: com o amostrador honesto (§31) a fenda mais funda desta peça lê
    // `0,750`. A barra antiga saía do leque plano, e depois da cura ela não achava pixel nenhum —
    // *uma fixtura calibrada sobre um defeito deixa de ter sujeito quando ele é curado.*
    let tapado = (0..g.hit.len())
        .find(|i| g.hit[*i] && oc[*i] < 0.85)
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
    com.set_ambient(oc);

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

/// ⭐⭐⭐ **O REFINAMENTO: [`crate::OCCLUSION_PASSES`] passagens, uma por vez, e a última é a
/// sequência inteira.** ⚠️ A contagem **deriva** da const — um número escrito aqui tornar-se-ia a
/// segunda resposta à pergunta de quantas passagens existem.
#[test]
fn o_refinamento_entrega_as_passagens_e_acaba_na_sequencia_inteira() {
    let doc = cruz();
    let (reg, cam, g, _) = cena(&doc, 96, 54);

    let mut sh = crate::Shadows::default();
    let mut vistos = Vec::new();
    let mut ultimo = Vec::new();
    let correu =
        crate::refine_hemisphere(&doc, &reg, &cam, &g, &sem_cena(), &[], &mut sh, |sh, k| {
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
    //
    // ⚠️ **Com a SUAVIZAÇÃO por cima dos dois lados**: ela é aplicada no publicar, logo faz parte
    // do que o refinamento entrega. Comparar com o cru acusaria a suavização de ser uma divergência.
    let inteira = crate::blur_occlusion(
        &g,
        &crate::occlusion(&doc, &reg, &cam, &g, crate::OCCLUSION_PASSES),
    );
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
    let correu =
        crate::refine_hemisphere(&doc, &reg, &cam, &g, &sem_cena(), &[], &mut sh, |_, k| {
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
    crate::refine_hemisphere(&doc, &reg, &cam, &g, &sem_cena(), &[], &mut sh, |sh, _| {
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

/// ⭐⭐⭐ **A LISTRA VERTICAL do report do dono — medida, não vista** (`docs/Render3d/05` §31).
///
/// ⛔ O defeito era o azimute sair só do pixel, com o bit `0` dele no bit mais significativo: as
/// colunas PARES apontavam para um lado e as ÍMPARES para o oposto. ⚠️ **Nenhuma régua desta linha
/// o via** — a média da imagem é a mesma nas duas paridades quando se somam as duas, e a sonda da
/// convergência media o estimador contra ele próprio.
///
/// A régua que o apanha é a **paridade da coluna**: numa peça lisa, colunas vizinhas têm de ler
/// quase o mesmo. ⚠️ A barra é contra a diferença que a própria GEOMETRIA produz entre linhas
/// vizinhas (o controlo), e não um número escolhido: *uma listra é a horizontal a destoar da
/// vertical.*
#[test]
fn a_oclusao_nao_faz_listras_entre_colunas_vizinhas() {
    let doc = cruz();
    let (reg, cam, g, _) = cena(&doc, 192, 108);
    let oc = crate::occlusion(&doc, &reg, &cam, &g, 8);
    let (w, h) = (g.width as usize, g.height as usize);

    // A diferença média entre vizinhos, em cada eixo, só onde os DOIS são peça.
    let mut dx = (0.0f64, 0usize);
    let mut dy = (0.0f64, 0usize);
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !g.hit[i] {
                continue;
            }
            if x + 1 < w && g.hit[i + 1] {
                dx.0 += f64::from((oc[i] - oc[i + 1]).abs());
                dx.1 += 1;
            }
            if y + 1 < h && g.hit[i + w] {
                dy.0 += f64::from((oc[i] - oc[i + w]).abs());
                dy.1 += 1;
            }
        }
    }
    assert!(
        dx.1 > 2_000 && dy.1 > 2_000,
        "fixtura pequena demais: {dx:?} {dy:?}"
    );
    let (hx, hy) = (dx.0 / dx.1 as f64, dy.0 / dy.1 as f64);
    assert!(
        hx < hy * 1.6,
        "as colunas destoam das linhas: vizinho horizontal {hx:.4} contra vertical {hy:.4} \
         ({:.2}×) — é a assinatura de uma LISTRA vertical, e foi assim que ela chegou à foto do \
         dono sem nenhum gate a acusar",
        hx / hy
    );

    // ⭐ E a metade direta: as colunas PARES e as ÍMPARES têm de ler a mesma coisa.
    let media = |p: usize| -> f64 {
        let v: Vec<f64> = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|(x, y)| x % 2 == p && g.hit[y * w + x])
            .map(|(x, y)| f64::from(oc[y * w + x]))
            .collect();
        v.iter().sum::<f64>() / v.len() as f64
    };
    let (par, impar) = (media(0), media(1));
    assert!(
        (par - impar).abs() < 0.01,
        "as colunas pares leem {par:.4} de céu e as ímpares {impar:.4} — a paridade do pixel está \
         a escolher o AZIMUTE, que é o defeito da §31"
    );
}

/// ⭐⭐⭐ **A OCLUSÃO DE UM PONTO É FUNÇÃO DO PONTO — não do pixel em que ele cai.**
///
/// ⛔⛔ **É o gate que a lei de Monte Carlo NÃO podia passar, e é por isso que ele nasce agora.** O
/// amostrador que shipou até 2026-09-15 semeava o sorteio no **índice do pixel** (`sample_uv(i, …)`),
/// logo o mesmo ponto da peça recebia uma estimativa diferente conforme onde aterrava — e a
/// consequência que o dono via não era «grão»: era a imagem **mudar** ao reenquadrar.
///
/// ⚠️ **A igualdade é EXACTA, ao bit.** Não há epsilon porque não há nada a promediar: a mesma
/// entrada percorre as mesmas `OCCLUSION_PASSES` direcções pela mesma ordem.
#[test]
fn a_oclusao_de_um_ponto_nao_depende_do_pixel_em_que_ele_cai() {
    let doc = cruz();
    let (reg, cam, g, _) = cena(&doc, 96, 54);

    // ⭐ O sujeito tem de estar OCLUÍDO — num ponto de céu aberto todo estimador concorda, e o
    // gate passaria sobre a lei que ele existe para proibir.
    let cru = crate::occlusion(&doc, &reg, &cam, &g, crate::OCCLUSION_PASSES);
    let fonte = (0..g.hit.len())
        .find(|i| g.hit[*i] && cru[*i] < 0.8)
        .expect("nenhum pixel ocluído na fixtura");
    assert!(
        cru[fonte] > 0.0,
        "o sujeito está totalmente tapado — um `0` seria igual por saturação, não por lei"
    );

    // O MESMO ponto e a MESMA normal, repetidos em oito índices diferentes.
    const N: usize = 8;
    let repetido = Gbuffer {
        width: N as u32,
        height: 1,
        hit: vec![true; N],
        normal: vec![g.normal[fonte]; N],
        point: vec![g.point[fonte]; N],
        edges: Vec::new(),
    };
    let oc = crate::occlusion(&doc, &reg, &cam, &repetido, crate::OCCLUSION_PASSES);
    for i in 1..N {
        assert_eq!(
            oc[i], oc[0],
            "o índice {i} lê {:.6} e o índice 0 lê {:.6} sobre o MESMO ponto — a oclusão está \
             semeada no pixel",
            oc[i], oc[0]
        );
    }
}

/// ⭐⭐⭐ **E ELA NÃO DEPENDE DA CÂMERA** — rodar a vista não repinta o sombreado de contacto.
///
/// ⚠️ **O conjunto de direcções é de MUNDO** ([`crate::cone_dir`]), e é isso que este gate afirma.
/// Um referencial TANGENTE construído a partir da normal daria o mesmo aqui — o que ele não daria
/// é continuidade na costura da base, que é a outra metade da razão.
///
/// ⚠️ A barra não é o bit: as duas corridas convertem a normal por bases diferentes, logo os `f32`
/// chegam ao campo por caminhos diferentes. `1e-5` é folga de representação, e um estimador semeado
/// na vista erraria por **décimas**.
#[test]
fn a_oclusao_de_um_ponto_nao_depende_da_camera() {
    let doc = cruz();
    let (reg, cam_a, g, _) = cena(&doc, 96, 54);
    let cru = crate::occlusion(&doc, &reg, &cam_a, &g, crate::OCCLUSION_PASSES);
    let fonte = (0..g.hit.len())
        .find(|i| g.hit[*i] && cru[*i] < 0.8)
        .expect("nenhum pixel ocluído na fixtura");

    // O ponto e a normal em MUNDO — o que a peça de facto tem.
    let (ra, ua, fa) = cam_a.basis();
    let nv = g.normal[fonte];
    let n_mundo = [0, 1, 2].map(|c| nv[0] * ra[c] + nv[1] * ua[c] + nv[2] * fa[c]);
    let ponto = g.point[fonte];

    // ⚠️ **Só a ROTAÇÃO muda.** O `half_extent` entra na tolerância de acerto e no alcance, logo
    // mexer nele mediria outra coisa — e essa dependência é da LEI, não do amostrador.
    let de = |cam: &Orbit| -> f32 {
        let (r, u, f) = cam.basis();
        let em_vista = [
            n_mundo[0] * r[0] + n_mundo[1] * r[1] + n_mundo[2] * r[2],
            n_mundo[0] * u[0] + n_mundo[1] * u[1] + n_mundo[2] * u[2],
            n_mundo[0] * f[0] + n_mundo[1] * f[1] + n_mundo[2] * f[2],
        ];
        let um = Gbuffer {
            width: 1,
            height: 1,
            hit: vec![true],
            normal: vec![em_vista],
            point: vec![ponto],
            edges: Vec::new(),
        };
        crate::occlusion(&doc, &reg, cam, &um, crate::OCCLUSION_PASSES)[0]
    };

    let a = de(&cam_a);
    assert!(a < 0.8, "o sujeito deixou de estar ocluído ({a:.4})");
    for (yaw, pitch) in [
        (0.72_f32, 0.52_f32),
        (2.10, -0.30),
        (-1.40, 0.95),
        (3.90, 0.10),
    ] {
        let cam_b = Orbit {
            rotation: Orbit::from_yaw_pitch(yaw, pitch).rotation,
            ..cam_a
        };
        let b = de(&cam_b);
        assert!(
            (a - b).abs() < 1e-5,
            "o mesmo ponto lê {a:.6} de uma câmera e {b:.6} de outra ({yaw}, {pitch}) — a oclusão \
             está ancorada na VISTA e ferve ao reenquadrar"
        );
    }
}

/// ⭐⭐⭐ **A SUAVIZAÇÃO NÃO ATRAVESSA UMA QUINA.**
///
/// ⛔ É a única coisa que a torna legítima: a oclusão é de baixa frequência **dentro** de uma
/// superfície, e borrar através da fronteira entre duas seria inventar resposta onde a verdadeira
/// salta. O gate põe duas metades com normais opostas e um degrau perfeito entre elas.
#[test]
fn a_suavizacao_nao_atravessa_uma_quina() {
    let (w, h) = (16u32, 8u32);
    let n = (w * h) as usize;
    let mut g = crate::Gbuffer {
        width: w,
        height: h,
        hit: vec![true; n],
        normal: vec![[0.0, 0.0, 1.0]; n],
        point: vec![[0.0; 3]; n],
        edges: Vec::new(),
    };
    let mut oc = vec![0.0f32; n];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = y * w as usize + x;
            // Metade esquerda: normal para um lado e céu cheio. Direita: o oposto, e tapada.
            if x < 8 {
                g.normal[i] = [0.0, 0.0, 1.0];
                oc[i] = 1.0;
            } else {
                g.normal[i] = [0.0, 0.0, -1.0];
                oc[i] = 0.0;
            }
        }
    }
    let out = crate::blur_occlusion(&g, &oc);
    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = y * w as usize + x;
            let esperado = if x < 8 { 1.0 } else { 0.0 };
            assert_eq!(
                out[i], esperado,
                "o pixel ({x},{y}) sangrou através da quina: {} em vez de {esperado}",
                out[i]
            );
        }
    }
}

/// ⭐⭐ **E DENTRO de uma superfície ela suaviza mesmo** — senão o gate de cima passaria por ela
/// não fazer nada.
#[test]
fn a_suavizacao_apaga_o_ruido_dentro_de_uma_superficie() {
    let (w, h) = (16u32, 8u32);
    let n = (w * h) as usize;
    let g = crate::Gbuffer {
        width: w,
        height: h,
        hit: vec![true; n],
        normal: vec![[0.0, 0.0, 1.0]; n],
        point: vec![[0.0; 3]; n],
        edges: Vec::new(),
    };
    // Um tabuleiro de xadrez: a média verdadeira é `0,5` em todo o lado.
    let oc: Vec<f32> = (0..n)
        .map(|i| {
            let (x, y) = (i % w as usize, i / w as usize);
            f32::from(u8::from((x + y) % 2 == 0))
        })
        .collect();
    let out = crate::blur_occlusion(&g, &oc);
    let miolo: Vec<f32> = (1..h as usize - 1)
        .flat_map(|y| (1..w as usize - 1).map(move |x| (x, y)))
        .map(|(x, y)| out[y * w as usize + x])
        .collect();
    let pior = miolo.iter().fold(0.0f32, |m, v| m.max((v - 0.5).abs()));
    assert!(
        pior < 0.12,
        "o ruído sobreviveu à suavização: o pior desvio de 0,5 é {pior:.3} (o cru é 0,5)"
    );
}

/// Sonda: as barras dos gates, com o amostrador e a suavização finais.
#[test]
#[ignore = "sonda"]
fn probe_as_barras_da_oclusao() {
    for (nome, doc) in [("esfera", esfera()), ("cruz", cruz())] {
        let (reg, cam, g, _) = cena(&doc, 160, 90);
        let cru = crate::occlusion(&doc, &reg, &cam, &g, crate::OCCLUSION_PASSES);
        let oc = crate::blur_occlusion(&g, &cru);
        let peca: Vec<usize> = (0..g.hit.len()).filter(|i| g.hit[*i]).collect();
        let mut v: Vec<f32> = peca.iter().map(|i| oc[*i]).collect();
        v.sort_by(f32::total_cmp);
        let q = |f: f64| v[((v.len() - 1) as f64 * f) as usize];
        println!(
            "{nome:7}: n={} · mín {:.3} · p01 {:.3} · p05 {:.3} · mediana {:.3} · <0,9: {:.1} % · <0,8: {:.1} %",
            v.len(),
            v[0],
            q(0.01),
            q(0.05),
            q(0.50),
            100.0 * v.iter().filter(|x| **x < 0.9).count() as f64 / v.len() as f64,
            100.0 * v.iter().filter(|x| **x < 0.8).count() as f64 / v.len() as f64,
        );
    }
}

/// ⭐⭐⭐ **O borrão do RICOCHETE é a MESMA lei do borrão do CÉU, canal a canal e ao bit.**
///
/// ⚠️⚠️ **Este gate existe porque o irmão dele é CEGO à cópia:** o
/// `o_refinamento_avanca_as_duas_metades_com_o_mesmo_k` compara a saída do refinamento com
/// `blur_bounce(bounce_pass(..))` — as duas passam pela MESMA função, logo mutá-la muda os dois
/// lados e ele fica **verde**. *Uma igualdade entre duas rotas que partilham uma porta não afirma
/// nada sobre essa porta.*
///
/// ⇒ a régua é o **céu**: cada canal do ricochete tem de sair exactamente como o
/// [`crate::blur_occlusion`] o devolveria sozinho. É isso que torna a partilha da
/// [`crate::occlusion::para_cada_vizinhanca`] uma LEI e não uma economia — se alguém afinar o
/// [`crate::OCCLUSION_BLUR_COS`] numa das duas, este gate diz.
#[test]
fn o_borrao_do_ricochete_e_o_do_ceu_canal_a_canal() {
    // ⚠️ **Em CONSTANTE e não num `assert!` de corpo:** o compilador dobra um `assert!` sobre duas
    // constantes antes de ele correr, e o clippy di-lo em voz alta. *Aqui a lei é de COMPILAÇÃO.*
    const _: () = assert!(crate::BOUNCE_BLUR_PASSES > 1);

    let doc = cruz();
    // ⚠️ Só o G-buffer interessa: esta lei é sobre a VIZINHANÇA (a máscara e as normais), e não
    // sobre o campo — pôr a câmera e o registo em jogo seria medir outra coisa.
    let (_reg, _cam, g, _) = cena(&doc, 96, 54);
    let peca = (0..g.hit.len()).filter(|i| g.hit[*i]).count();
    assert!(peca > 500, "a fixtura não desenhou peça: {peca} px");

    // ⚠️ Três canais DIFERENTES de propósito: com os três iguais, um borrão que trocasse os canais
    // entre si ficaria verde.
    #[allow(clippy::cast_precision_loss)]
    let canal: Vec<[f32; 3]> = (0..g.hit.len())
        .map(|i| {
            let t = i as f32;
            [
                (t * 0.013).sin().abs(),
                (t * 0.031).cos().abs(),
                (t % 7.0) / 7.0,
            ]
        })
        .collect();

    // ⛔⛔ **A PREMISSA DESTE GATE MORREU em 2026-09-17, e a morte está aqui no diff.**
    //
    // Ele dizia `blur_bounce == blur_occlusion` e isso era verdade enquanto os dois levassem UMA
    // passagem. O report do dono (*«como se fosse muitas sombras duras»*) mediu que o ricochete
    // precisa de **duas** ([`crate::BOUNCE_BLUR_PASSES`], com a tabela e o joelho no doc dela) ⇒ a
    // igualdade passa a ser sobre **UMA passagem**, que é onde a lei de facto é partilhada.
    //
    // ⭐ **O que o gate protege continua a ser o mesmo, e é o que interessa:** que não existe uma
    // SEGUNDA cópia da guarda da normal. As duas leis partilham a
    // [`crate::occlusion::para_cada_vizinhanca`], e duas cópias divergiriam no dia em que alguém
    // afinasse o [`crate::OCCLUSION_BLUR_COS`] numa delas.
    let uma = crate::bounce::blur_bounce_uma_vez(&g, &canal);
    for c in 0..3 {
        let sozinho: Vec<f32> = canal.iter().map(|v| v[c]).collect();
        let esperado = crate::blur_occlusion(&g, &sozinho);
        let obtido: Vec<f32> = uma.iter().map(|v| v[c]).collect();
        assert_eq!(
            obtido, esperado,
            "o canal {c} de UMA passagem do borrão do ricochete diverge do borrão do céu — as \
             duas leis separaram-se"
        );
    }

    // ⚠️ **E a metade que impede a reversão silenciosa:** o que o produto aplica são
    // [`crate::BOUNCE_BLUR_PASSES`] passagens, e elas TÊM de se notar. Sem esta asserção, pôr o
    // número de volta a `1` deixaria o gate acima verde e a wave do report desfeita **em silêncio**.
    let ship = crate::blur_bounce(&g, &canal);
    let diferentes = (0..g.hit.len())
        .filter(|&i| g.hit[i] && (0..3).any(|c| ship[i][c] != uma[i][c]))
        .count();
    assert!(
        diferentes > peca / 10,
        "só {diferentes} de {peca} pixels distinguem UMA passagem das \
         {} que o produto aplica — o borrão do ricochete voltou a ser o do céu",
        crate::BOUNCE_BLUR_PASSES
    );
}

/// ⭐ **A cena SEM materiais e SEM lâmpadas** — com ela o ricochete degenera para o canal vazio e a
/// [`crate::refine_hemisphere`] entrega exactamente o que a metade do céu entregava sozinha.
///
/// ⚠️ Estes gates medem o CÉU, e é por isso que eles não passam uma cena: *um gate que medisse as
/// duas metades ao mesmo tempo deixaria de dizer qual delas reprovou.*
fn sem_cena() -> crate::Surfaces<'static> {
    crate::Surfaces {
        all: &[],
        owners: None,
    }
}
