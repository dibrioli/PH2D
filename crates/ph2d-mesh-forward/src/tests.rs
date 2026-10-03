//! Os gates do desenhista NA PLACA (ignorados por omissão: precisam de um aparelho) —
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-mesh-forward -- --ignored`.

use crate::{Ambiente, Camera, Cena, Forward, Instancia, Malha};

/// Um céu CHAPADO de radiância `L`, sem caixa — a fixtura em que a lei do material tem resposta
/// conhecida na CPU (`Surface::indirect` com o mesmo ambiente).
const L: f32 = 0.8;

const CEU_CHAPADO: &str = r#"
struct Ceu { l: vec4<f32>, };
fn ceu_radiance_sem_caixa(dir: vec3<f32>, shrink: f32) -> vec3<f32> { return vec3<f32>(ceu.l.x); }
fn ceu_radiance_da_caixa(dir: vec3<f32>, alpha: f32) -> vec3<f32> { return vec3<f32>(0.0); }
fn ceu_irradiance_sem_caixa(n: vec3<f32>) -> vec3<f32> { return vec3<f32>(ceu.l.x); }
fn ceu_irradiance_da_caixa(n: vec3<f32>) -> vec3<f32> { return vec3<f32>(0.0); }
"#;

struct Chapado;
impl ph2d_material::Environment for Chapado {
    fn radiance(&self, _: [f32; 3], _: f32) -> [f32; 3] {
        [L; 3]
    }
    fn irradiance(&self, _: [f32; 3]) -> [f32; 3] {
        [L; 3]
    }
}

fn ambiente() -> Ambiente<'static> {
    Ambiente {
        wgsl: CEU_CHAPADO,
        constantes: &[L, 0.0, 0.0, 0.0],
        tabela: &[0.0],
        piso_luz: 0.05,
    }
}

/// Uma esfera UV de raio `r`.
fn esfera(r: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let (anel, gomo) = (48u32, 96u32);
    let (mut p, mut n, mut idx) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..=anel {
        let t = std::f32::consts::PI * i as f32 / anel as f32;
        for j in 0..=gomo {
            let f = 2.0 * std::f32::consts::PI * j as f32 / gomo as f32;
            let d = [t.sin() * f.cos(), t.cos(), t.sin() * f.sin()];
            n.push(d);
            p.push(d.map(|c| c * r));
        }
    }
    let w = gomo + 1;
    for i in 0..anel {
        for j in 0..gomo {
            let (a, b, c, d) = (
                i * w + j,
                i * w + j + 1,
                (i + 1) * w + j,
                (i + 1) * w + j + 1,
            );
            idx.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    (p, n, idx)
}

const ID: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Ortográfica a olhar para `−z`, meia-largura `s`, rodada `a` radianos em torno de `y`.
fn camera(s: f32, a: f32) -> Camera {
    let (c, si) = (a.cos(), a.sin());
    // vista: roda o mundo por −a em y; recorte: x/s, y/s, z = 0,5 − 0,1·z_vista.
    let vp = [
        [c / s, 0.0, -(-si) * -0.1, 0.0],
        [0.0, 1.0 / s, 0.0, 0.0],
        [-si / s, 0.0, -c * 0.1, 0.0],
        [0.0, 0.0, 0.5, 1.0],
    ];
    Camera {
        view_proj: vp,
        olho: [0.0; 3],
        perspectiva: false,
        dir_vista: [-si, 0.0, -c],
    }
}

fn material_cinza() -> [f32; ph2d_material::wgsl::PACKED] {
    let s = ph2d_material::OpenPbr::default().prepare();
    ph2d_material::wgsl::pack(&s, ph2d_material::wgsl::EnvLobe::of(&s))
}

fn cena<'a>(objs: &'a [Instancia], mats: &'a [[f32; 48]], cam: Camera) -> Cena<'a> {
    Cena {
        objetos: objs,
        materiais: mats,
        camera: cam,
        luzes: &[],
        chao: None,
        caixa_tan: None,
        exposicao: 0.0,
        vista: 0,
        tamanho: (96, 96),
        brilho: ph2d_bloom::Bloom::default(),
        estilo: ph2d_style::Style::default(),
        raio_da_peca: 0.5,
    }
}

fn desenhista_com_esfera() -> Option<Forward> {
    let Some(mut fw) = Forward::no_aparelho(&ambiente()) else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return None;
    };
    let (p, n, idx) = esfera(0.5);
    let ao = vec![1.0; p.len()];
    let mat = vec![0u32; p.len()];
    fw.sobe(
        1,
        &Malha {
            posicoes: &p,
            normais: &n,
            ao: &ao,
            material: &mat,
            indices: &idx,
        },
    );
    Some(fw)
}

/// ⭐ **Cabe no celular**: o desenhista nasce com `Features::empty()` e os limites do WebGL2, o
/// shader valida, e o quadro tem a peça opaca no meio e o fundo transparente nos cantos.
#[test]
#[ignore = "precisa de aparelho"]
fn cabe_no_celular() {
    let Some(mut fw) = desenhista_com_esfera() else {
        return;
    };
    assert_eq!(fw.device().features(), wgpu::Features::empty());
    let objs = [Instancia {
        malha: 1,
        modelo: ID,
    }];
    let mats = [material_cinza()];
    let img = fw
        .quadro(&cena(&objs, &mats, camera(1.0, 0.0)))
        .expect("quadro");
    let px = |x: usize, y: usize| &img[(y * 96 + x) * 4..(y * 96 + x) * 4 + 4];
    assert_eq!(px(48, 48)[3], 255, "o meio é peça");
    assert_eq!(px(2, 2)[3], 0, "o canto é fundo");
}

/// ⭐⭐ **O quadro está pronto na hora** — os quatro sintomas do dono (borrado, engasgo, granulado,
/// espera) têm uma raiz: um quadro que depende dos ANTERIORES. Aqui: a mesma câmara depois de um
/// giro dá o MESMO quadro, ao byte.
#[test]
#[ignore = "precisa de aparelho"]
fn quadro_pronto_na_hora() {
    let Some(mut fw) = desenhista_com_esfera() else {
        return;
    };
    let objs = [Instancia {
        malha: 1,
        modelo: ID,
    }];
    let mats = [material_cinza()];
    // ⚠️ E com o BRILHO ligado: a cadeia é refeita de raiz a cada quadro, nada vem do anterior.
    for brilho in [ph2d_bloom::Bloom::default(), brilho_aceso()] {
        let com = |a| Cena {
            brilho,
            ..cena(&objs, &mats, camera(1.0, a))
        };
        let a = fw.quadro(&com(0.0)).expect("a");
        let girado = fw.quadro(&com(0.7)).expect("girado");
        let b = fw.quadro(&com(0.0)).expect("b");
        assert_eq!(
            a, b,
            "a mesma câmara depois de um giro tem de dar o mesmo quadro (brilho {})",
            brilho.enabled
        );
        assert_ne!(a, girado, "o controlo: a câmara girada é outra imagem");
    }
}

/// O brilho de um gate: a esfera cinzenta sob o céu chapado (`~0,6` de cena-linear) passa o limiar.
fn brilho_aceso() -> ph2d_bloom::Bloom {
    ph2d_bloom::Bloom {
        enabled: true,
        params: ph2d_bloom::BloomParams {
            threshold: 0.2,
            knee: 0.0,
            intensity: 1.0,
            ..ph2d_bloom::BloomParams::default()
        },
    }
}

/// ⭐⭐⭐ **O brilho acende FORA da peça, e nunca escurece** — o halo derrama-se no fundo (que a peça
/// não cobre) e leva cobertura consigo; desligado, o quadro é AO BYTE o de sempre.
#[test]
#[ignore = "precisa de aparelho"]
fn o_brilho_acende_fora_da_peca() {
    let Some(mut fw) = desenhista_com_esfera() else {
        return;
    };
    if !fw.tem_brilho() {
        eprintln!("esta placa não desenha Rgba16Float com 4× — sem brilho, o gate não corre");
        return;
    }
    let objs = [Instancia {
        malha: 1,
        modelo: ID,
    }];
    let mats = [material_cinza()];
    let base = cena(&objs, &mats, camera(1.0, 0.0));
    let sem = fw.quadro(&base).expect("sem");
    let com = fw
        .quadro(&Cena {
            brilho: brilho_aceso(),
            ..base
        })
        .expect("com");
    let mut desligado = brilho_aceso();
    desligado.enabled = false;
    let off = fw
        .quadro(&Cena {
            brilho: desligado,
            ..base
        })
        .expect("desligado");
    assert_eq!(
        sem, off,
        "o controlo: desligado é o quadro de sempre, ao byte"
    );
    let mut acesos = 0;
    for (a, b) in sem.as_chunks::<4>().0.iter().zip(com.as_chunks::<4>().0) {
        for k in 0..4 {
            assert!(b[k] >= a[k], "o brilho escureceu um canal: {a:?} → {b:?}");
        }
        if a[3] == 0 && b[3] > 0 {
            acesos += 1;
        }
    }
    // A esfera tem `24` px de raio num quadro de `96`: o fundo que a rodeia é `~7 400` px.
    assert!(acesos > 1000, "o halo mal chegou ao fundo: {acesos} píxeis");
    let px = |img: &[u8], x: usize, y: usize| img[(y * 96 + x) * 4 + 3];
    assert_eq!(px(&sem, 48, 16), 0, "a 8 px da silhueta é fundo");
    assert!(
        px(&com, 48, 16) > 0,
        "a 8 px da silhueta o halo tem de chegar"
    );
}

/// ⭐⭐ **Nada compila ao editar** — mover, mudar a cor e acrescentar um objeto não criam pipeline.
#[test]
#[ignore = "precisa de aparelho"]
fn nada_compila_ao_editar() {
    let Some(mut fw) = desenhista_com_esfera() else {
        return;
    };
    let antes = fw.pipelines_compilados();
    let mut mats = vec![material_cinza()];
    let mut objs = vec![Instancia {
        malha: 1,
        modelo: ID,
    }];
    let _ = fw.quadro(&cena(&objs, &mats, camera(1.0, 0.0)));
    let vermelho = ph2d_material::OpenPbr {
        base_color: [0.9, 0.1, 0.1],
        ..ph2d_material::OpenPbr::default()
    };
    let s = vermelho.prepare();
    mats.push(ph2d_material::wgsl::pack(
        &s,
        ph2d_material::wgsl::EnvLobe::of(&s),
    ));
    let mut movido = ID;
    movido[3][0] = 0.3;
    objs.push(Instancia {
        malha: 1,
        modelo: movido,
    });
    let _ = fw.quadro(&cena(&objs, &mats, camera(1.0, 0.0)));
    assert_eq!(fw.pipelines_compilados(), antes);
    // ⭐ E o BRILHO: ligar, mexer no raio, no limiar e na tinta, e desligar — nada compila.
    let mut b = brilho_aceso();
    for passo in 0..4 {
        b.params.radius = 1.0 + passo as f32;
        b.params.threshold = 0.1 * passo as f32;
        b.params.tint = [1.0, 0.5, 0.25 * passo as f32, 1.0];
        b.enabled = passo != 3;
        let _ = fw.quadro(&Cena {
            brilho: b,
            ..cena(&objs, &mats, camera(1.0, 0.0))
        });
    }
    assert_eq!(fw.pipelines_compilados(), antes);
}

/// ⭐⭐⭐ **A cor é a da lei da casa**: o pixel do meio da esfera sob um céu chapado é o
/// `Surface::indirect` da CPU, pelo mesmo olhar, codificado como o `para_ecra` — a diferença máxima
/// é a de arredondamento.
#[test]
#[ignore = "precisa de aparelho"]
fn a_cor_e_a_lei_da_casa() {
    let Some(mut fw) = desenhista_com_esfera() else {
        return;
    };
    let objs = [Instancia {
        malha: 1,
        modelo: ID,
    }];
    let mats = [material_cinza()];
    // ⭐ E o TOM DE CÂMARA: a exposição e a vista entram pela mesma lei do `to_display`.
    let olhares = [
        (0.0, ph2d_view_transform::ViewTransform::Standard),
        (1.0, ph2d_view_transform::ViewTransform::Neutral),
        (-1.0, ph2d_view_transform::ViewTransform::Standard),
        (2.0, ph2d_view_transform::ViewTransform::Neutral),
    ];
    let i = (48 * 96 + 48) * 4;
    // No meio da esfera a normal e a vista são `+z`.
    let s = ph2d_material::OpenPbr::default().prepare();
    let c = s.indirect([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], &Chapado);
    let mut vistos = Vec::new();
    for (exposicao, vista) in olhares {
        let img = fw
            .quadro(&Cena {
                exposicao,
                vista: ph2d_view_transform::wgsl::view_code(vista),
                ..cena(&objs, &mats, camera(1.0, 0.0))
            })
            .expect("quadro");
        let d = ph2d_view_transform::to_display(c, exposicao, vista);
        for k in 0..3 {
            let esperado = (srgb(d[k]) * 255.0 + 0.5).floor();
            let lido = f32::from(img[i + k]);
            assert!(
                (lido - esperado).abs() <= 2.0,
                "{vista:?}{exposicao:+}, canal {k}: placa {lido} contra CPU {esperado}"
            );
        }
        vistos.push(img[i]);
    }
    vistos.dedup();
    assert!(
        vistos.len() > 2,
        "o controlo: os olhares têm de dar cores diferentes ({vistos:?})"
    );
}

/// ⭐⭐⭐ **O estilo é a lei da casa** — o pixel do meio da esfera com a tinta de ARESTA, as zonas e
/// a saturação do indirecto, contra a CPU: `saturate_indirect` sobre o indirecto, depois
/// `Style::apply` com `|N·V| = 1` e a curvatura `H · raio` (esfera de raio `0,5`: `H = 2`, peça de
/// raio `0,5` ⇒ `1`). A curvatura chega pela porta própria ([`Forward::sobe_curvatura`]).
///
/// ⚠️ O controlo: a MESMA cena com a curvatura a zero é OUTRA cor (a tinta lê a curvatura subida).
#[test]
#[ignore = "precisa de aparelho"]
fn o_estilo_e_a_lei_da_casa() {
    let Some(mut fw) = desenhista_com_esfera() else {
        return;
    };
    let objs = [Instancia {
        malha: 1,
        modelo: ID,
    }];
    // ⚠️ Um material COLORIDO: sob o céu cinzento um cinzento não tem croma, e a saturação do
    // indirecto seria invisível (uma mutação que a apagava SOBREVIVEU com a fixtura cinzenta).
    let pbr = ph2d_material::OpenPbr {
        base_color: [0.8, 0.3, 0.15],
        ..ph2d_material::OpenPbr::default()
    };
    let s = pbr.prepare();
    let mats = [ph2d_material::wgsl::pack(
        &s,
        ph2d_material::wgsl::EnvLobe::of(&s),
    )];
    let mut estilo = ph2d_style::Style::default();
    estilo.curvature.convex = [1.0, 0.35, 0.2];
    estilo.curvature.edge_sharpness = 0.5;
    estilo.zones.shadow = [0.6, 0.7, 1.0];
    estilo.zones.highlight = [1.0, 0.95, 0.8];
    estilo.indirect_saturation = 0.4;
    let com = |fw: &mut Forward| {
        fw.quadro(&Cena {
            estilo,
            ..cena(&objs, &mats, camera(1.0, 0.0))
        })
        .expect("quadro")
    };
    let n = esfera(0.5).0.len();
    assert!(fw.sobe_curvatura(1, &vec![[0.0, 0.0]; n]));
    let plano = com(&mut fw);
    assert!(fw.sobe_curvatura(1, &vec![[0.0, 2.0]; n]));
    assert!(
        !fw.sobe_curvatura(1, &[[0.0, 2.0]]),
        "um tamanho que não bate é recusado"
    );
    let img = com(&mut fw);
    let i = (48 * 96 + 48) * 4;
    let c = s.indirect([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], &Chapado);
    let c = estilo.sanitized().saturate_indirect(c);
    let c = estilo.sanitized().apply(
        c,
        ph2d_style::Point {
            facing: 1.0,
            curvature: 1.0,
        },
    );
    let d = ph2d_view_transform::to_display(c, 0.0, ph2d_view_transform::ViewTransform::Standard);
    for k in 0..3 {
        let esperado = (srgb(d[k]) * 255.0 + 0.5).floor();
        let lido = f32::from(img[i + k]);
        assert!(
            (lido - esperado).abs() <= 2.0,
            "canal {k}: placa {lido} contra CPU {esperado}"
        );
    }
    assert_ne!(
        plano[i..i + 3],
        img[i..i + 3],
        "o controlo: a curvatura subida tem de mudar a tinta"
    );
}

/// ⭐⭐ **A subsuperfície MACIÇA lê a curvatura do material** — a mesma esfera sob uma lâmpada, com a
/// curvatura do material a zero e a `2` (`1/raio`): o pixel tem de mudar. Antes deste porte o canal
/// ia sempre a zero (o `pack` deixa-o vazio de propósito: é do PIXEL), e a peça translúcida lia-se
/// como plana.
#[test]
#[ignore = "precisa de aparelho"]
fn a_subsuperficie_le_a_curvatura_do_material() {
    let Some(mut fw) = desenhista_com_esfera() else {
        return;
    };
    let objs = [Instancia {
        malha: 1,
        modelo: ID,
    }];
    let pbr = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        subsurface_color: [0.9, 0.4, 0.3],
        ..ph2d_material::OpenPbr::default()
    };
    let s = pbr.prepare();
    let mats = [ph2d_material::wgsl::pack(
        &s,
        ph2d_material::wgsl::EnvLobe::of(&s),
    )];
    let luzes = [crate::Luz {
        posicao: [0.6, 0.4, 1.5],
        radiancia_a_um: [3.0, 3.0, 3.0],
    }];
    let n = esfera(0.5).0.len();
    let mut com = |k: f32| {
        assert!(fw.sobe_curvatura(1, &vec![[k, 0.0]; n]));
        fw.quadro(&Cena {
            luzes: &luzes,
            ..cena(&objs, &mats, camera(1.0, 0.0))
        })
        .expect("quadro")
    };
    let plano = com(0.0);
    let curvo = com(2.0);
    let i = (48 * 96 + 60) * 4;
    assert_ne!(
        plano[i..i + 3],
        curvo[i..i + 3],
        "a curvatura do material não chegou à subsuperfície"
    );
}

fn srgb(x: f32) -> f32 {
    let c = x.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// ⭐ **O shader valida SEM aparelho, com capacidades VAZIAS** — corre em todo lado, CI incluída.
#[test]
fn o_shader_valida_sem_capacidades() {
    for (nome, src) in [
        ("forward", crate::fonte(&ambiente())),
        ("ecra", crate::fonte::ecra()),
        ("brilho", crate::fonte::brilho()),
    ] {
        let module = naga::front::wgsl::parse_str(&src)
            .unwrap_or_else(|e| panic!("{nome}: nao parsa: {}", e.emit_to_string(&src)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("{nome}: nao valida: {}", e.emit_to_string(&src)));
    }
}

/// Nenhuma ranhura fica por preencher.
#[test]
fn nenhuma_ranhura_fica_por_preencher() {
    let src = crate::fonte(&ambiente());
    for marca in [
        "{MATERIAL}",
        "{AMBIENTE}",
        "{OLHAR}",
        "{ESTILO}",
        "{MAX_LUZES",
        "{TAB_W}",
        "{PISO_LUZ}",
        "{ENV}",
    ] {
        assert!(!src.contains(marca), "a ranhura {marca} ficou no shader");
    }
}

/// ⭐ **Cabe no GLES** — o backend do WebGL2 e dos Androids sem Vulkan. A mesma imagem do
/// `cabe_no_celular`, ao byte no meio da peça, e o fundo transparente.
#[test]
#[ignore = "precisa de aparelho GL"]
fn cabe_no_gles() {
    let Some(mut gl) = Forward::no_backend(wgpu::Backends::GL, &ambiente()) else {
        eprintln!("sem adaptador GL nesta máquina — o gate não corre aqui");
        return;
    };
    let Some(mut nativo) = desenhista_com_esfera() else {
        return;
    };
    let (p, n, idx) = esfera(0.5);
    let ao = vec![1.0; p.len()];
    let mat = vec![0u32; p.len()];
    gl.sobe(
        1,
        &Malha {
            posicoes: &p,
            normais: &n,
            ao: &ao,
            material: &mat,
            indices: &idx,
        },
    );
    let objs = [Instancia {
        malha: 1,
        modelo: ID,
    }];
    let mats = [material_cinza()];
    let a = gl
        .quadro(&cena(&objs, &mats, camera(1.0, 0.0)))
        .expect("gl");
    let b = nativo
        .quadro(&cena(&objs, &mats, camera(1.0, 0.0)))
        .expect("nativo");
    let i = (48 * 96 + 48) * 4;
    eprintln!(
        "GLES formato {:?}: meio {:?} · nativo {:?}",
        gl.formato(),
        &a[i..i + 4],
        &b[i..i + 4]
    );
    for k in 0..4 {
        assert!(
            a[i + k].abs_diff(b[i + k]) <= 2,
            "GLES {:?} contra nativo {:?}",
            &a[i..i + 4],
            &b[i..i + 4]
        );
    }
    assert_eq!(a[3], 0, "o canto é fundo no GLES");
}

/// Um céu com CAIXA: a parte sem caixa vale `L`, a da caixa vale `C` vinda de cima (`+y`).
const CEU_COM_CAIXA: &str = r#"
struct Ceu { l: vec4<f32>, };
fn ceu_radiance_sem_caixa(dir: vec3<f32>, shrink: f32) -> vec3<f32> { return vec3<f32>(ceu.l.x); }
fn ceu_radiance_da_caixa(dir: vec3<f32>, alpha: f32) -> vec3<f32> { return vec3<f32>(ceu.l.y * max(dir.y, 0.0)); }
fn ceu_irradiance_sem_caixa(n: vec3<f32>) -> vec3<f32> { return vec3<f32>(ceu.l.x); }
fn ceu_irradiance_da_caixa(n: vec3<f32>) -> vec3<f32> { return vec3<f32>(ceu.l.y * max(n.y, 0.0)); }
"#;

/// ⭐⭐ **A sombra da caixa POUSA no chão** — uma esfera a `0,1` acima do chão escurece o chão logo
/// por baixo dela (vista de cima, o meio da esfera tapa; o anel à volta é a sombra), e o chão longe
/// dela fica transparente (o chão que só recebe não aparece onde nada o tapa).
#[test]
#[ignore = "precisa de aparelho"]
fn a_sombra_pousa_no_chao() {
    let amb = Ambiente {
        wgsl: CEU_COM_CAIXA,
        constantes: &[0.2, 1.0, 0.0, 0.0],
        tabela: &[0.0],
        piso_luz: 0.05,
    };
    let Some(mut fw) = Forward::no_aparelho(&amb) else {
        eprintln!("sem aparelho — saltado");
        return;
    };
    let (p, n, idx) = esfera(0.3);
    let ao = vec![1.0; p.len()];
    let mat = vec![0u32; p.len()];
    fw.sobe(
        1,
        &Malha {
            posicoes: &p,
            normais: &n,
            ao: &ao,
            material: &mat,
            indices: &idx,
        },
    );
    // A esfera centrada a y = 0,4: o chão a y = 0 fica 0,1 abaixo dela.
    let mut m = ID;
    m[3][1] = 0.4;
    let objs = [Instancia {
        malha: 1,
        modelo: m,
    }];
    let mats = [material_cinza()];
    // A câmara olha de CIMA (−y): x → x, z → −y do ecrã.
    let s = 1.0f32;
    let cam = Camera {
        view_proj: [
            [1.0 / s, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.1, 0.0],
            [0.0, -1.0 / s, 0.0, 0.0],
            [0.0, 0.0, 0.5, 1.0],
        ],
        olho: [0.0; 3],
        perspectiva: false,
        dir_vista: [0.0, -1.0, 0.0],
    };
    let mut c = cena(&objs, &mats, cam);
    c.chao = Some(0.0);
    c.caixa_tan = Some(0.47);
    let img = fw.quadro(&c).expect("quadro");
    let alfa = |x: usize, y: usize| img[(y * 96 + x) * 4 + 3];
    // O raio da esfera em pixels é 0,3·48 = 14,4: a 18 px do meio já é chão, e ele tem de escurecer.
    let anel = alfa(48 + 18, 48);
    let longe = alfa(2, 2);
    eprintln!("chão: anel alfa {anel} · longe alfa {longe}");
    assert!(
        anel > 20,
        "a sombra não pousou: alfa {anel} ao lado da esfera"
    );
    assert!(
        longe < 3,
        "o chão longe tem de ficar transparente: alfa {longe}"
    );
}
