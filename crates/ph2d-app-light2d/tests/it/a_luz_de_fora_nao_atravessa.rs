//! ⭐ O bug 6 (`docs/Luz2D/BUGS_luz2d.md` §6; handoff W6g §32–§33): a luz de FORA da grade da vista entra
//! pela BORDA dela e os opacos dela tapam-na — nunca atravessa um opaco que só a vista resolve. Antes, a c0
//! do entorno somava-se na saída sem direção: atrás do L 0,047 (24 % da luz livre), atrás de uma parede de
//! 0,03 m 0,166 (90 %), e a cena 5 na troca de oitava +69 % / +141 % na sombra contra a grade esticada.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_app_light2d::{
    DEGRAU_PADRAO, LuzDoQuadro, config_da_vista, config_do_entorno, grade_do_quadro,
    regiao_da_janela, smoke,
};
use ph2d_ecs::{Name, PresentWorld};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_gpu::{Grade, Solver};
use ph2d_render::Camera2d;

use crate::comum::{fluencia_lida, gpu, renderer};

const FHD: (u32, u32) = (1920, 1080);
/// Atrás de um opaco, com a luz de fora a 0,12–0,14 sem ele: o teto (1,5 % dela). Medido 0,0000.
const TETO_ATRAS: f64 = 0.002;
/// Sem o opaco a luz de fora CHEGA (o controlo): ao gabarito ±5 %. Medido 0,1719 / 0,1711 (+0,5 %).
const TOL_LIVRE: f64 = 0.05;

/// A cena 5 só com o L (sem a lâmpada, o ambiente e os refletores) e um corpo que brilha fora da grade.
fn cena_minima() -> CenaDeLuz {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    for n in [
        "Disco",
        "Elipse",
        "Losango",
        "Barra",
        "Cruz — braço",
        "Cruz — tronco",
        "Luz ambiente",
        "Lâmpada",
    ] {
        let e = w
            .query::<(Entity, &Name)>()
            .iter(&w)
            .find(|(_, m)| m.0 == n)
            .map(|(e, _)| e)
            .expect(n);
        w.despawn(e);
    }
    let mut cena = cena_de_luz(&w);
    cena.emissores.push(Emissor::Poligono {
        vertices: vec![[3.0, -2.5], [4.0, -2.5], [4.0, -1.5], [3.0, -1.5]],
        radiancia: [1.0, 0.3, 0.3],
        alcance_m: f32::INFINITY,
    });
    cena.luzes += 1;
    cena
}

/// A fluência (vermelho) nos `pontos` com a vista e o entorno do produto, 60 quadros.
fn vista_mais_entorno(
    gpu: &ph2d_gpu::GpuContext,
    cena: &CenaDeLuz,
    pontos: &[[f64; 2]],
) -> Vec<f64> {
    let d = DEGRAU_PADRAO;
    let a = Camera2d::new([-4.2, -1.8], 3.8).world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
    let g = grade_do_quadro(a, FHD, d).expect("grade");
    let mut s = Solver::novo(gpu, config_da_vista(&gpu.device, g, a, d)).expect("vista");
    let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
    s.regiao_fina(i, f);
    let ce = config_do_entorno(&gpu.device, a, FHD, d).expect("o entorno");
    assert!(
        ce.arquivo_da_borda.is_some(),
        "o produto lê o entorno pela borda"
    );
    let mut e = Solver::novo(gpu, ce).expect("entorno");
    e.regiao_fina([0; 2], [0; 2]);
    assert!(e.mascarar_miolo(
        gpu,
        Some([
            g.origem_m[0],
            g.origem_m[1],
            g.origem_m[0] + f64::from(g.largura) * g.texel_m,
            g.origem_m[1] + f64::from(g.altura) * g.texel_m,
        ])
    ));
    for _ in 0..60 {
        e.quadro(gpu, cena);
        assert!(s.somar_entorno(gpu, Some(&e)));
        s.quadro(gpu, cena);
    }
    let fl = fluencia_lida(gpu, &s);
    pontos.iter().map(|&p| fl.amostra_canal(p, 0)).collect()
}

#[test]
fn a_luz_de_fora_nao_atravessa_um_opaco_da_vista() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — saltado");
        return;
    };
    let base = cena_minima();
    let albedo = base.oclusores.first().map_or([0.5; 3], |o| o.albedo);
    let parede = CenaDeLuz {
        oclusores: vec![Oclusor {
            vertices: vec![[-3.0, -6.0], [-2.97, -6.0], [-2.97, 2.0], [-3.0, 2.0]],
            opacidade: 1.0,
            albedo,
        }],
        sprites: Vec::new(),
        ..base.clone()
    };
    let livre = CenaDeLuz {
        oclusores: Vec::new(),
        sprites: Vec::new(),
        ..base.clone()
    };
    // (cena, pontos atrás do opaco) — o L da cena 5 e uma parede de 0,03 m (2 texels da vista, ¼ do entorno).
    let casos: [(&str, &CenaDeLuz, Vec<[f64; 2]>); 2] = [
        ("o L", &base, vec![[-4.2, -2.0], [-5.2, -2.0], [-4.6, -2.4]]),
        (
            "a parede de 0,03 m",
            &parede,
            vec![[-3.1, -2.0], [-3.6, -2.0], [-4.2, -2.0], [-5.2, -2.0]],
        ),
    ];
    for (nome, cena, pontos) in &casos {
        let atras = vista_mais_entorno(&gpu, cena, pontos);
        let controlo = vista_mais_entorno(&gpu, &livre, pontos);
        for ((p, v), c) in pontos.iter().zip(&atras).zip(&controlo) {
            let gab = ph2d_light2d_ref::fluencia(&livre, *p)[0];
            eprintln!(
                "{nome} {p:?}: atrás {v:.4} · sem o opaco {c:.4} · gabarito sem ele {gab:.4}"
            );
            assert!(
                *v <= TETO_ATRAS,
                "{nome} {p:?}: a luz de fora atravessou o opaco ({v:.4})"
            );
            assert!(
                (c / gab - 1.0).abs() <= TOL_LIVRE,
                "{nome} {p:?}: sem o opaco a luz de fora não chega ({c:.4} contra {gab:.4})"
            );
        }
    }
}

/// A cena 5 inteira no caminho do app, dos dois lados da troca de oitava do texel (0,0312 → 0,0156: os
/// refletores saem da grade e o rebote deles passa ao entorno), contra a MESMA vista com a grade esticada
/// sobre a sala. Medido (handoff W6g §35, §37): com o entorno 4× da Baixa o pior −1,4 % (o entorno 8×,
/// −4,0…−4,7 %; 1× ±0,4 %); a sombra à esquerda da haste é o raio rasante na quina e oscila sem ordem
/// com a grossura (+1,4 · −5,1 %: a tolerância dela à parte). Com a 1.ª passagem do entorno a mascarar
/// a vista, −13,7 % (o laranja do Disco a 67–76 %); a soma da W5d dava +69 % / +141 % na sombra.
#[test]
fn a_cena_5_na_troca_de_oitava_segue_a_grade_esticada() {
    const TOL: f64 = 0.025;
    const TOL_QUINA: f64 = 0.07;
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — saltado");
        return;
    };
    let r = renderer(&gpu);
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let cena = cena_de_luz(&w);
    let pontos: [(&str, [f64; 2]); 4] = [
        ("sombra à esquerda da haste", [-4.2, -2.0]),
        ("sombra abaixo da base", [-2.6, -2.7]),
        ("face de cima da haste", [-3.4, -0.9]),
        ("face de cima da base", [-2.2, -2.1]),
    ];
    let mut texels = Vec::new();
    for altura in [4.27_f32, 3.8] {
        let a = Camera2d::new([-4.2, -1.8], altura)
            .world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
        let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
        let mut present = PresentWorld::new();
        for _ in 0..120 {
            assert!(l.preparar(&w, a, FHD));
            l.gravar(&gpu, &mut present, &r);
        }
        let gv = l.grade().expect("grade");
        let t = gv.texel_m;
        texels.push(t);
        let ge = Grade {
            origem_m: [(-6.5 / t).floor() * t, (-5.0 / t).floor() * t],
            texel_m: t,
            largura: (13.0 / t).ceil() as u32,
            altura: (10.0 / t).ceil() as u32,
        };
        let mut s = Solver::novo(&gpu, config_da_vista(&gpu.device, ge, a, DEGRAU_PADRAO))
            .expect("a esticada");
        let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
        s.regiao_fina(i, f);
        for _ in 0..120 {
            s.quadro(&gpu, &cena);
        }
        let fv = fluencia_lida(&gpu, l.solver().expect("solver"));
        let fe = fluencia_lida(&gpu, &s);
        for (k, (nome, p)) in pontos.into_iter().enumerate() {
            let tol = if k == 0 { TOL_QUINA } else { TOL };
            for c in 0..3 {
                let (v, e) = (fv.amostra_canal(p, c), fe.amostra_canal(p, c));
                eprintln!(
                    "h={altura} texel={t:.4} {nome} canal {c}: vista {v:.4} · esticada {e:.4} ({:+.1} %)",
                    100.0 * (v / e - 1.0)
                );
                assert!(
                    (v / e - 1.0).abs() <= tol,
                    "h={altura} {nome} canal {c}: a vista {v:.4} contra a esticada {e:.4}"
                );
            }
        }
    }
    assert!(
        texels[0] > texels[1],
        "as duas vistas ficam dos dois lados da oitava: {texels:?}"
    );
}

/// O outro lado da mesma lei: a luz de fora que a vista deixa PASSAR chega. Uma parede junto à borda da
/// grade, de 0,25 m (2 texels do entorno), com uma fenda de 0,06 m (4 texels da vista, ½ do entorno): o
/// entorno, grosso, fecha-a — se o
/// arquivo dele trouxesse a parede grossa (o miolo não vácuo na 2.ª passagem), a luz pela fenda sumia.
#[test]
fn a_luz_de_fora_passa_a_fenda_perto_da_borda() {
    const MIN_DO_GABARITO: f64 = 0.5;
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — saltado");
        return;
    };
    let base = cena_minima();
    let albedo = base.oclusores.first().map_or([0.5; 3], |o| o.albedo);
    // A parede a 0,5 m da borda direita da grade da vista: entre ela e a coluna de sondas que a vista lê.
    let a = Camera2d::new([-4.2, -1.8], 3.8).world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
    let g = grade_do_quadro(a, FHD, DEGRAU_PADRAO).expect("grade");
    let x0 = (g.origem_m[0] + f64::from(g.largura) * g.texel_m - 0.5) as f32;
    eprintln!(
        "a borda direita da vista em x = {:.3}; a parede em x = {x0:.3}",
        x0 + 0.5
    );
    let placa = |y0: f32, y1: f32| Oclusor {
        vertices: vec![[x0, y0], [x0 + 0.25, y0], [x0 + 0.25, y1], [x0, y1]],
        opacidade: 1.0,
        albedo,
    };
    let fenda = CenaDeLuz {
        oclusores: vec![placa(-6.0, -2.03), placa(-1.97, 2.0)],
        sprites: Vec::new(),
        ..base
    };
    let pontos = [[-1.0, -2.0], [-2.0, -2.0], [-3.0, -2.0]];
    let v = vista_mais_entorno(&gpu, &fenda, &pontos);
    for (p, v) in pontos.iter().zip(&v) {
        let gab = ph2d_light2d_ref::fluencia(&fenda, *p)[0];
        eprintln!("a fenda {p:?}: vista {v:.4} · gabarito {gab:.4}");
        assert!(gab > 0.0, "a fenda deixa passar no gabarito");
        assert!(
            *v >= MIN_DO_GABARITO * gab,
            "{p:?}: a luz pela fenda não chegou ({v:.4} contra {gab:.4})"
        );
    }
}

/// Um corpo que BRILHA encostado à borda de cima da vista (a cavalo e todo fora dela), lido pela borda,
/// contra a MESMA vista com a grade esticada sobre a sala. Medido (handoff W6g §37): com o entorno 4× da
/// Baixa −1,6…+1,1 %; a 8× +1,5…+11,0 % (a grossura do arquivo); 1× ±0,4 %.
#[test]
fn o_emissor_encostado_a_borda_segue_a_grade_esticada() {
    const TOL: f64 = 0.03;
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — saltado");
        return;
    };
    let mut so_o_l = cena_minima();
    so_o_l.emissores.pop();
    so_o_l.luzes -= 1;
    let pontos = [
        [-4.2, -2.0],
        [-2.4, -1.2],
        [-3.4, -0.9],
        [-2.2, -2.1],
        [-5.0, -0.4],
    ];
    let d = DEGRAU_PADRAO;
    let a = Camera2d::new([-4.2, -1.8], 3.8).world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
    let t = grade_do_quadro(a, FHD, d).expect("grade").texel_m;
    let ge = Grade {
        origem_m: [(-6.5 / t).floor() * t, (-5.0 / t).floor() * t],
        texel_m: t,
        largura: (13.0 / t).ceil() as u32,
        altura: (10.0 / t).ceil() as u32,
    };
    for (rotulo, dy) in [("a cavalo", 0.0_f32), ("todo fora", 0.6)] {
        let mut cena = so_o_l.clone();
        cena.emissores.push(Emissor::Poligono {
            vertices: (0..48)
                .map(|k| {
                    let t = std::f32::consts::TAU * k as f32 / 48.0;
                    [-3.0 + 0.7 * t.cos(), 1.7 + dy + 0.7 * t.sin()]
                })
                .collect(),
            radiancia: [1.0, 0.3, 0.2],
            alcance_m: f32::INFINITY,
        });
        cena.luzes += 1;
        let mut x = Solver::novo(&gpu, config_da_vista(&gpu.device, ge, a, d)).expect("esticada");
        let (i, f) = regiao_da_janela(x.grade_da_fluencia(), a, FHD);
        x.regiao_fina(i, f);
        for _ in 0..60 {
            x.quadro(&gpu, &cena);
        }
        let fx = fluencia_lida(&gpu, &x);
        let vista = vista_mais_entorno(&gpu, &cena, &pontos);
        for (p, v) in pontos.iter().zip(vista) {
            let e = fx.amostra_canal(*p, 0);
            eprintln!(
                "o emissor {rotulo} {p:?}: vista {v:.4} · esticada {e:.4} ({:+.1} %)",
                100.0 * (v / e - 1.0)
            );
            assert!(
                (v / e - 1.0).abs() <= TOL,
                "o emissor {rotulo} {p:?}: a vista {v:.4} contra a esticada {e:.4}"
            );
        }
    }
}

/// ⭐ §40.2 (handoff W6g) — a descida da 2.ª passagem do entorno só onde o arquivo depende NÃO muda um bit
/// da luz da vista: a cena 5 inteira (os refletores fora da grade), duas câmeras na Baixa e uma na Alta (os
/// níveis de cima), o recorte ligado × a passagem inteira.
#[test]
fn o_recorte_do_arquivo_nao_muda_um_bit() {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let cena = cena_de_luz(&w);
    for (centro, nivel) in [
        ([-4.2, -1.8], NivelDaLuz::Baixa),
        ([-1.0, 0.5], NivelDaLuz::Baixa),
        ([-4.2, -1.8], NivelDaLuz::Alta),
    ] {
        let d = degrau_do(nivel);
        let a = Camera2d::new(centro, 3.8).world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
        let g = grade_do_quadro(a, FHD, d).expect("grade");
        let luz = |recorta: bool| {
            let mut s = Solver::novo(&gpu, config_da_vista(&gpu.device, g, a, d)).expect("vista");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
            s.regiao_fina(i, f);
            let mut ce = config_do_entorno(&gpu.device, a, FHD, d).expect("o entorno");
            assert!(
                ce.arquivo_da_borda.is_some(),
                "o produto lê o entorno pela borda"
            );
            ce.hrc_recorta_o_arquivo = recorta;
            let mut e = Solver::novo(&gpu, ce).expect("entorno");
            e.regiao_fina([0; 2], [0; 2]);
            assert!(e.mascarar_miolo(
                &gpu,
                Some([
                    g.origem_m[0],
                    g.origem_m[1],
                    g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                    g.origem_m[1] + f64::from(g.altura) * g.texel_m,
                ])
            ));
            for _ in 0..30 {
                e.quadro(&gpu, &cena);
                assert!(s.somar_entorno(&gpu, Some(&e)));
                s.quadro(&gpu, &cena);
            }
            fluencia_lida(&gpu, &s).v
        };
        let (com, sem) = (luz(true), luz(false));
        let diferentes = com.iter().zip(&sem).filter(|(a, b)| a != b).count();
        eprintln!(
            "{centro:?} {nivel:?}: {diferentes} de {} texels diferentes",
            com.len()
        );
        assert!(
            com.len() == sem.len() && diferentes == 0,
            "{centro:?} {nivel:?}: o recorte mudou {diferentes} texels"
        );
    }
}

/// ⭐ §44 (handoff W6g) — os 4 quadrantes da holográfica num despacho por etapa NÃO mudam um bit da luz da vista:
/// a cena 5 inteira, a Baixa (o único nível que os junta), duas câmeras deitadas e uma janela em pé (o quadrante
/// com mais níveis troca de eixo: o topo de um não é o do outro), a vista e o entorno juntos × um depois do outro.
#[test]
fn os_quadrantes_juntos_nao_mudam_um_bit() {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let cena = cena_de_luz(&w);
    for nivel in [NivelDaLuz::Media, NivelDaLuz::Alta] {
        assert!(
            !degrau_do(nivel).quadrantes_juntos,
            "{nivel:?}: a memória ×3,7 só na Baixa"
        );
    }
    let d = degrau_do(NivelDaLuz::Baixa);
    // A 2160p a vista da Baixa (1280×768) já enche a placa e fica um a um; o entorno dela (992×576) junta.
    let a4k = Camera2d::new([-4.2, -1.8], 3.8).world_to_screen_affine(WindowSize::new(3840, 2160));
    let g4k = grade_do_quadro(a4k, (3840, 2160), d).expect("grade");
    assert!(
        !config_da_vista(&gpu.device, g4k, a4k, d).hrc_juntos(),
        "a vista de 2160p passa o JUNTOS_ATE"
    );
    assert!(
        config_do_entorno(&gpu.device, a4k, (3840, 2160), d)
            .expect("o entorno")
            .hrc_juntos()
    );
    for (centro, alvo) in [
        ([-4.2, -1.8], FHD),
        ([-1.0, 0.5], FHD),
        ([-4.2, -1.8], (1080, 1920)),
    ] {
        let a = Camera2d::new(centro, 3.8).world_to_screen_affine(WindowSize::new(alvo.0, alvo.1));
        let g = grade_do_quadro(a, alvo, d).expect("grade");
        let luz = |juntos: bool| {
            let mut cv = config_da_vista(&gpu.device, g, a, d);
            let mut ce = config_do_entorno(&gpu.device, a, alvo, d).expect("o entorno");
            assert!(
                cv.hrc_juntos() && ce.hrc_juntos(),
                "o produto junta os quadrantes na Baixa"
            );
            cv.hrc_quadrantes_juntos = juntos;
            ce.hrc_quadrantes_juntos = juntos;
            let mut s = Solver::novo(&gpu, cv).expect("vista");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
            s.regiao_fina(i, f);
            let mut e = Solver::novo(&gpu, ce).expect("entorno");
            e.regiao_fina([0; 2], [0; 2]);
            assert!(e.mascarar_miolo(
                &gpu,
                Some([
                    g.origem_m[0],
                    g.origem_m[1],
                    g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                    g.origem_m[1] + f64::from(g.altura) * g.texel_m,
                ])
            ));
            for _ in 0..30 {
                e.quadro(&gpu, &cena);
                assert!(s.somar_entorno(&gpu, Some(&e)));
                s.quadro(&gpu, &cena);
            }
            fluencia_lida(&gpu, &s).v
        };
        let (com, sem) = (luz(true), luz(false));
        let diferentes = com.iter().zip(&sem).filter(|(a, b)| a != b).count();
        let acesos = sem.iter().filter(|v| v.iter().any(|c| *c > 0.0)).count();
        eprintln!(
            "{centro:?} {alvo:?}: {diferentes} de {} texels diferentes ({acesos} acesos)",
            com.len()
        );
        assert!(
            acesos > 0,
            "{centro:?} {alvo:?}: a vista às escuras não prova nada"
        );
        assert!(
            com.len() == sem.len() && diferentes == 0,
            "{centro:?} {alvo:?}: juntar os quadrantes mudou {diferentes} texels"
        );
    }
}

/// ⭐ §46.2 (handoff W6g) — o nível 0 da descida por UM cone (`hrc::DESCIDA_UM_CONE`) NÃO muda um bit: o produto
/// contra todos os níveis pelos cones (o antes) e contra todos por um cone, a vista com o entorno; a cena 5, a
/// Baixa (os quadrantes juntos) nas duas câmeras deitadas e na janela em pé, a Média e a Alta.
#[test]
fn o_nivel_0_por_um_cone_nao_muda_um_bit() {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let cena = cena_de_luz(&w);
    let casos = [
        (NivelDaLuz::Baixa, [-4.2, -1.8], FHD),
        (NivelDaLuz::Baixa, [-1.0, 0.5], FHD),
        (NivelDaLuz::Baixa, [-4.2, -1.8], (1080, 1920)),
        (NivelDaLuz::Media, [-4.2, -1.8], FHD),
        (NivelDaLuz::Alta, [-4.2, -1.8], FHD),
    ];
    for (nivel, centro, alvo) in casos {
        let d = degrau_do(nivel);
        let a = Camera2d::new(centro, 3.8).world_to_screen_affine(WindowSize::new(alvo.0, alvo.1));
        let g = grade_do_quadro(a, alvo, d).expect("grade");
        let luz = |um: Option<u32>| {
            let cv = config_da_vista(&gpu.device, g, a, d);
            let ce = config_do_entorno(&gpu.device, a, alvo, d).expect("o entorno");
            assert!(
                cv.hrc_cones_por_fio > 1,
                "{nivel:?}: o produto faz cones por fio"
            );
            let mut s = Solver::novo(&gpu, cv).expect("vista");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
            s.regiao_fina(i, f);
            let mut e = Solver::novo(&gpu, ce).expect("entorno");
            e.regiao_fina([0; 2], [0; 2]);
            assert!(e.mascarar_miolo(
                &gpu,
                Some([
                    g.origem_m[0],
                    g.origem_m[1],
                    g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                    g.origem_m[1] + f64::from(g.altura) * g.texel_m,
                ])
            ));
            if let Some(um) = um {
                s.diag_cones_um(&gpu, um);
                e.diag_cones_um(&gpu, um);
            }
            for _ in 0..30 {
                e.quadro(&gpu, &cena);
                assert!(s.somar_entorno(&gpu, Some(&e)));
                s.quadro(&gpu, &cena);
            }
            fluencia_lida(&gpu, &s).v
        };
        let produto = luz(None);
        let acesos = produto
            .iter()
            .filter(|v| v.iter().any(|c| *c > 0.0))
            .count();
        assert!(
            acesos > 0,
            "{nivel:?} {centro:?} {alvo:?}: a vista às escuras não prova nada"
        );
        for (nome, um) in [("todos pelos cones", 0), ("todos por um cone", u32::MAX)] {
            let outra = luz(Some(um));
            let diferentes = produto.iter().zip(&outra).filter(|(x, y)| x != y).count();
            eprintln!(
                "{nivel:?} {centro:?} {alvo:?} × {nome}: {diferentes} de {} texels diferentes ({acesos} acesos)",
                produto.len()
            );
            assert!(
                produto.len() == outra.len() && diferentes == 0,
                "{nivel:?} {centro:?} {alvo:?}: o produto × {nome} mudou {diferentes} texels"
            );
        }
    }
}

/// ⭐ §52 (handoff W6g) — a descida que não lê o R de cima onde T = 0 (`Config::hrc_salta_o_t_nulo`) NÃO muda um
/// bit: o produto × a descida que lê sempre, a vista com o entorno, a cena 5. As vistas são as que apanharam as
/// formas erradas da mesma cura (1440p Média e 2160p Baixa: o termo de longe a 0 desfazia a FMA; 1080p Alta: o
/// termo unificado sem o salto).
#[test]
fn a_descida_sem_o_t_nulo_nao_muda_um_bit() {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_app_light2d::teto::{Tetos, fina_que_cabe};
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let cena = cena_de_luz(&w);
    let casos = [
        (NivelDaLuz::Baixa, [-4.2, -1.8], FHD),
        (NivelDaLuz::Alta, [-4.2, -1.8], FHD),
        (NivelDaLuz::Media, [-3.2, -1.8], (2560, 1440)),
        (NivelDaLuz::Baixa, [-3.2, -1.8], (3840, 2160)),
    ];
    for (nivel, centro, alvo) in casos {
        let a = Camera2d::new(centro, 3.8).world_to_screen_affine(WindowSize::new(alvo.0, alvo.1));
        let k = fina_que_cabe(
            a,
            alvo,
            degrau_do(nivel),
            Tetos {
                lado: 8192,
                recusou: None,
            },
        )
        .expect("cabe");
        let d = degrau_do(nivel).com_fina_engrossada(k);
        let g = grade_do_quadro(a, alvo, d).expect("grade");
        let luz = |salta: bool| {
            let mut cv = config_da_vista(&gpu.device, g, a, d);
            let mut ce = config_do_entorno(&gpu.device, a, alvo, d).expect("o entorno");
            assert!(
                cv.hrc_salta_o_t_nulo && ce.hrc_salta_o_t_nulo,
                "o produto salta o T nulo"
            );
            cv.hrc_salta_o_t_nulo = salta;
            ce.hrc_salta_o_t_nulo = salta;
            let mut s = Solver::novo(&gpu, cv).expect("vista");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
            s.regiao_fina(i, f);
            let mut e = Solver::novo(&gpu, ce).expect("entorno");
            e.regiao_fina([0; 2], [0; 2]);
            assert!(e.mascarar_miolo(
                &gpu,
                Some([
                    g.origem_m[0],
                    g.origem_m[1],
                    g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                    g.origem_m[1] + f64::from(g.altura) * g.texel_m,
                ])
            ));
            for _ in 0..30 {
                e.quadro(&gpu, &cena);
                assert!(s.somar_entorno(&gpu, Some(&e)));
                s.quadro(&gpu, &cena);
            }
            fluencia_lida(&gpu, &s).v
        };
        let (com, sem) = (luz(true), luz(false));
        let acesos = com.iter().filter(|v| v.iter().any(|c| *c > 0.0)).count();
        assert!(
            acesos > 0,
            "{nivel:?} {centro:?} {alvo:?}: a vista às escuras não prova nada"
        );
        let diferentes = com.iter().zip(&sem).filter(|(x, y)| x != y).count();
        eprintln!(
            "{nivel:?} {centro:?} {alvo:?}: {diferentes} de {} texels diferentes ({acesos} acesos)",
            com.len()
        );
        assert!(
            com.len() == sem.len() && diferentes == 0,
            "{nivel:?} {centro:?} {alvo:?}: saltar o T nulo mudou {diferentes} texels"
        );
    }
}
