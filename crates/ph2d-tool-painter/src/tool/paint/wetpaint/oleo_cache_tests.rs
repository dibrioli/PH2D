//! **O cache do campo de altura por dependência** (`ph2d_wet_paint::oleo_real::CacheCampo`, filho de
//! [`super`]): a varredura do cartão com o campo guardado = a recomposição inteira de raiz, ao byte — para o que a
//! chave deixa reusar (Specular, fenda, material sem Subsurface, Transparency) e para o que a obriga a refazer
//! (cada campo da chave: a mutação que o esquece fica vermelha aqui).

use super::super::oleo_plano::sem_cache_do_campo;
use super::*;
use ph2d_wet_paint::oleo_real::OleoReal;

/// O cartão `o` pela VARREDURA (o caminho do arrasto); devolve quantos campos se calcularam de raiz.
fn varre_com(t: &mut PainterTool, o: OleoReal) -> usize {
    t.paint.wetpaint.oleo_real_valores = o;
    t.paint.wetpaint.oleo_real = Some(o);
    let _ = sem_cache_do_campo::take_calculados();
    t.wet_recompoe_no_quadro();
    varre(t);
    sem_cache_do_campo::take_calculados()
}

/// A tela do cartão `o` recomposta inteira e de raiz.
fn de_raiz(mut t: PainterTool, o: OleoReal) -> Vec<u8> {
    sem_cache_do_campo::set(true);
    t.set_wet_oleo_real(Some(o));
    t.wet_recomposite_full();
    sem_cache_do_campo::set(false);
    t.canvas_rgba.to_vec()
}

#[test]
fn o_cache_do_campo_e_a_recomposicao_de_raiz_ao_byte() {
    let cena = || tela(160, 320);
    let base = OleoReal::default();
    let com_sss = OleoReal {
        material: ph2d_wet_paint::oleo_real::MaterialCartao {
            subsuperficie: 0.8,
            ..base.material
        },
        ..base
    };
    let muda = |o: OleoReal, f: &dyn Fn(&mut OleoReal)| {
        let mut o = o;
        f(&mut o);
        o
    };
    // (nome, antes, depois, o cache SERVE?)
    let casos: [(&str, OleoReal, OleoReal, bool); 12] = [
        ("Specular", base, muda(base, &|o| o.brilho = 0.3), true),
        ("Crevice", base, muda(base, &|o| o.fenda = 2.0), true),
        (
            "Metallic",
            base,
            muda(base, &|o| o.material.metal = 0.7),
            true,
        ),
        (
            "Coat",
            base,
            muda(base, &|o| o.material.verniz = 0.5),
            true,
        ),
        ("Depth", base, muda(base, &|o| o.profundidade = 1.6), false),
        (
            "af.raio_suave",
            base,
            muda(base, &|o| o.af.raio_suave = 5),
            false,
        ),
        ("Shadow 0", base, muda(base, &|o| o.sombra = 0.0), false),
        (
            "SSS: luz",
            com_sss,
            muda(com_sss, &|o| o.altitude = 0.4),
            false,
        ),
        (
            "SSS: Crevice",
            com_sss,
            muda(com_sss, &|o| o.fenda = 2.0),
            false,
        ),
        (
            "SSS: ambiente",
            com_sss,
            muda(com_sss, &|o| o.forca_ambiente = 0.2),
            false,
        ),
        (
            "SSS: raio",
            com_sss,
            muda(com_sss, &|o| o.material.raio_subsuperficie = 9.0),
            false,
        ),
        (
            "SSS: Specular",
            com_sss,
            muda(com_sss, &|o| o.brilho = 0.3),
            true,
        ),
    ];
    oleo_no_quadro::faixa_fixa(Some(24));
    for (nome, antes, depois, serve) in casos {
        let mut a = cena();
        a.set_wet_oleo_real(Some(antes));
        a.wet_recomposite_full();
        let calculados = varre_com(&mut a, depois);
        let b = de_raiz(cena(), depois);
        let difs = a.canvas_rgba.iter().zip(&b).filter(|(x, y)| x != y).count();
        assert_eq!(
            difs, 0,
            "{nome}: {difs} bytes entre o cache e a recomposição de raiz"
        );
        assert_eq!(
            calculados == 0,
            serve,
            "{nome}: {calculados} campos calculados de raiz na varredura"
        );
    }
    oleo_no_quadro::faixa_fixa(None);
}

/// Um traço novo (as grades mudam) depois de o cache estar cheio: a varredura do Specular a seguir não pode servir o
/// campo de antes do traço (a [`ph2d_wet_paint::painter::Engine::revisao`] mudou).
#[test]
fn o_cache_do_campo_nao_serve_grades_que_mudaram() {
    let traco = |t: &mut PainterTool| {
        t.set_brush_color_srgb8([20, 160, 60]);
        let p = |k: f32, phase| CanvasPointer {
            pos: [20.0 + 120.0 * k, 250.0 - 60.0 * k],
            pressure: 0.9,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(p(0.0, PointerPhase::Down));
        for i in 1..=20 {
            t.on_canvas_pointer(p(i as f32 / 20.0, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(p(1.0, PointerPhase::Up));
        t.on_tick(0.0);
    };
    let depois = OleoReal {
        brilho: 0.3,
        ..OleoReal::default()
    };
    oleo_no_quadro::faixa_fixa(Some(24));
    let mut a = tela(160, 320);
    a.wet_recomposite_full();
    traco(&mut a);
    varre_com(&mut a, depois);
    let mut b = tela(160, 320);
    traco(&mut b);
    let b = de_raiz(b, depois);
    oleo_no_quadro::faixa_fixa(None);
    let difs = a.canvas_rgba.iter().zip(&b).filter(|(x, y)| x != y).count();
    assert_eq!(
        difs, 0,
        "{difs} bytes: o cache serviu o campo de antes do traço"
    );
}

/// **A sonda do cache**: duas voltas da varredura do Specular (a 1.ª monta o cache, a 2.ª é o arrasto que
/// continua), com e sem o cache intercalados no mesmo processo, à tela do smoke (1024²) e a 4K.
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_cache_do_campo -- --ignored --nocapture`
#[test]
#[ignore = "sonda de custo — corre-se à mão no perfil smoke"]
fn sonda_cache_do_campo() {
    for (w, h) in [(1024, 1024), (3840, 2160)] {
        let mut t = tela(w, h);
        for ronda in 0..6 {
            let sem = ronda % 2 == 0;
            sem_cache_do_campo::set(sem);
            arrasta(&mut t, 4, 0.1 + 0.03 * ronda as f64);
            varre(&mut t);
            arrasta(&mut t, 4, 0.5 + 0.03 * ronda as f64);
            let _ = (
                contagem::take(),
                contagem::take_tempos(),
                contagem::take_campo(),
                sem_cache_do_campo::take_calculados(),
            );
            let t0 = std::time::Instant::now();
            let quadros = varre(&mut t);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            let tc = contagem::take_campo();
            let (tp, ts) = contagem::take_tempos();
            let calc = sem_cache_do_campo::take_calculados();
            let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
            println!(
                "[cache] {w}x{h} {}: volta {ms:.1} ms em {quadros} quadros · campo {tc:.1} ({calc} de raiz) · \
                 pigmento {tp:.1} · sombra {ts:.1} ms · load {}",
                if sem { "SEM" } else { "COM" },
                load.trim()
            );
        }
        sem_cache_do_campo::set(false);
    }
}
