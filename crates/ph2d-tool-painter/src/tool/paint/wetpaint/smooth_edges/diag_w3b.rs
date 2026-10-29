use crate::{PaintMedia, PainterTool};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

const W: u32 = 700;
const H: u32 = 200;

/// Um traço quase horizontal (y 90 → 102) de 40 px do pincel de fábrica `nome`, Óleo Real ligado, Smooth Edges 0,
/// numa tela 700 × 200 — e a água parada.
fn um_traco(nome: &str, cold: bool) -> PainterTool {
    let (w, h) = (W, H);
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    let p = crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato(nome);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    if cold {
        t.set_brush_paper_kind(1);
        t.set_substrate_depth(0.6);
    }
    t.set_wet_oleo_real_ligado(true);
    t.set_wet_smooth_edges(0.0);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([30, 70, 210]);
    let (a, z) = ([60.0f32, 90.0f32], [640.0f32, 102.0f32]);
    let n = ((z[0] - a[0]).hypot(z[1] - a[1]) / 3.0).ceil() as u32;
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    for k in 1..=n {
        let f = k as f32 / n as f32;
        t.on_canvas_pointer(cp(
            [a[0] + (z[0] - a[0]) * f, a[1] + (z[1] - a[1]) * f],
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(z, PointerPhase::Up));
    for _ in 0..600 {
        t.on_tick(1000.0 / 60.0);
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        if !s.engine.active_grid().has_fluid {
            break;
        }
    }
    t
}

#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_planos_da_borda_w3b() {
    let d = std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>");
    let nome = std::env::var("PH2D_OLEO_PINCEL").unwrap_or_else(|_| "Dabber".into());
    let (w, h) = (W, H);
    let mut t = um_traco(&nome, std::env::var("PH2D_OLEO_COLD").is_ok());
    let mut px = t.run_full().0;
    let sem_luz = px.clone();
    t.apply_impasto_light(&mut px, crate::Region { x: 0, y: 0, w, h });
    let s = t.paint.wetpaint.session.as_ref().expect("sessão");
    let ratio = s.ratio;
    let gw = s.pigment.len() / 4 / (h as usize / usize::from(ratio)).max(1);
    println!(
        "[planos] ratio {ratio} · grade {gw} · sombra {} · grao {}",
        s.sombra.len(),
        s.grao.len()
    );
    std::fs::write(format!("{d}/tela.rgba"), &px).expect("grava");
    std::fs::write(format!("{d}/sem_luz.rgba"), &sem_luz).expect("grava");
    std::fs::write(format!("{d}/pigmento.rgba"), &s.pigment).expect("grava");
    std::fs::write(format!("{d}/sombra.u8"), &s.sombra).expect("grava");
    std::fs::write(format!("{d}/grao.u8"), &s.grao).expect("grava");
    // A massa EFECTIVA (× o factor da espessura) da grade, `u16`, e os fios: a fixtura do gate da orla.
    let g = s.engine.active_grid();
    let mut m = Vec::with_capacity(g.w * g.h * 2);
    for y in 1..=g.h {
        for x in 1..=g.w {
            let i = x + y * g.s;
            let v = (f64::from(g.sett[i]) + f64::from(g.susp[i]))
                * ph2d_wet_paint::grid::espessura::factor(g, i);
            m.extend_from_slice(&(v.round().clamp(0.0, 65_535.0) as u16).to_le_bytes());
        }
    }
    println!(
        "[planos] grade {}x{} · altura {} · crista {} · material {}",
        g.w,
        g.h,
        g.altura.len(),
        g.crista.len(),
        g.material.len()
    );
    std::fs::write(format!("{d}/massa.u16"), &m).expect("grava");
    let fios: Vec<u8> = (1..=g.h)
        .flat_map(|y| (1..=g.w).map(move |x| x + y * g.s))
        .flat_map(|i| g.altura.get(i).copied().unwrap_or(0.0).to_le_bytes())
        .collect();
    std::fs::write(format!("{d}/fios.f32"), &fios).expect("grava");
}

/// ⭐ §BUG-bordas-W3b (foto do dono `smoke_27_10/7.png`): **a orla do traço do Dabber não tem pontinhos escuros
/// soltos NA TELA** — o composite inteiro (o contorno, o entalhe cheio, a sombra do papel lida sem furos), pela
/// régua [`ph2d_wet_paint::oleo_real::pontinhos_escuros`] com a cobertura pela cor (`(B − R)/120`, a de
/// `oraculo/pontinhos_regua.py`). Antes: 22 soltos neste traço.
#[test]
fn a_orla_do_dabber_nao_tem_pontinhos_escuros_na_tela() {
    let mut t = um_traco("Dabber", false);
    let px = t.run_full().0;
    let (w, h) = (W as usize, H as usize);
    let mut tela = vec![[255.0f64, 0.0]; (w + 2) * (h + 2)];
    for y in 0..h {
        for x in 0..w {
            let p = &px[(x + y * w) * 4..(x + y * w) * 4 + 3];
            let (r, g, b) = (f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
            tela[(x + 1) + (y + 1) * (w + 2)] = [
                0.299 * r + 0.587 * g + 0.114 * b,
                ((b - r) / 120.0).clamp(0.0, 1.0),
            ];
        }
    }
    let (soltos, todos) = ph2d_wet_paint::oleo_real::pontinhos_escuros(&tela, (w, h));
    println!("[pontinhos na tela] soltos na orla {soltos} · em toda a parte {todos}");
    assert_eq!(
        soltos, 0,
        "{soltos} pontinhos escuros soltos na orla do Dabber"
    );
}

/// A varredura do tick até fechar (a do painel, `oleo_no_quadro`).
fn varre(t: &mut PainterTool) -> usize {
    let mut q = 0;
    while t.oleo_por_compor() {
        t.on_tick(0.0);
        q += 1;
        assert!(q < 10_000, "a varredura não fecha");
    }
    q
}

fn mexe(t: &mut PainterTool, v: f64) {
    t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SetValue(
        crate::ids::PAINTER_WETPAINT_SMOOTH_EDGES,
        v,
    ));
}

/// ⭐ D1 W3b (dono 27/10: «Smooth Edges deve ser para todo o papel, de todos os traços já pintados ou futuros»):
/// **o Smooth Edges é da TELA** — mexer nele muda a tinta JÁ pousada, com e sem o Óleo Real; os eventos do
/// arrasto não recompõem (a varredura corre no tick, 1×/quadro); voltar a `0` é a tela de antes, ao bit (o
/// raio está na chave do campo guardado).
#[test]
fn o_smooth_edges_e_da_tela_muda_a_tinta_antiga_no_quadro_e_volta_ao_bit() {
    for oleo in [true, false] {
        let mut t = um_traco("Dabber", false);
        if !oleo {
            t.set_wet_oleo_real_ligado(false);
        }
        let a = t.canvas_rgba.to_vec();
        for v in [0.3, 0.6, 1.0] {
            mexe(&mut t, v);
        }
        assert_eq!(
            t.canvas_rgba.to_vec(),
            a,
            "óleo {oleo}: o evento recompôs a tela"
        );
        assert!(
            t.oleo_por_compor(),
            "óleo {oleo}: o Smooth Edges não pediu a varredura"
        );
        let quadros = varre(&mut t);
        let mudou = t.canvas_rgba.iter().zip(&a).filter(|(x, y)| x != y).count();
        println!(
            "[smooth da tela] óleo {oleo}: {mudou} bytes da tinta antiga em {quadros} quadros"
        );
        assert!(
            mudou > 0,
            "óleo {oleo}: o Smooth Edges não mudou a tinta já pousada"
        );
        mexe(&mut t, 0.0);
        varre(&mut t);
        let resto = t.canvas_rgba.iter().zip(&a).filter(|(x, y)| x != y).count();
        assert_eq!(
            resto, 0,
            "óleo {oleo}: voltar a 0 deixou {resto} bytes diferentes"
        );
    }
}

/// ⭐ D1 W3b: **com o Óleo Real o Smooth Edges age no RELEVO** — a sombra que a tinta projecta no papel (que só o
/// campo de altura decide) muda com ele. Controlo: a lei de antes (`so_na_cor`, só a cor) não a muda.
#[test]
fn com_o_oleo_o_smooth_edges_muda_a_sombra_no_papel() {
    for so_na_cor in [false, true] {
        super::so_na_cor::set(so_na_cor);
        let mut t = um_traco("Dabber", false);
        let sombra = |t: &PainterTool| {
            t.paint
                .wetpaint
                .session
                .as_ref()
                .expect("sessão")
                .sombra
                .clone()
        };
        let s0 = sombra(&t);
        t.set_wet_smooth_edges(1.0);
        let n = sombra(&t).iter().zip(&s0).filter(|(x, y)| x != y).count();
        super::so_na_cor::set(false);
        println!(
            "[smooth no relevo] só na cor {so_na_cor}: {n} células da sombra no papel mudaram"
        );
        if so_na_cor {
            assert_eq!(n, 0, "controlo: a lei só da cor mexeu na sombra");
        } else {
            assert!(
                n > 0,
                "o Smooth Edges não chegou ao relevo (a sombra no papel)"
            );
        }
    }
}

/// Os VÃOS CLAROS do halo: píxeis de papel (azul `(B − R)/120 < 0,1`) a ≤ 6 px da tinta (azul ≥ ½) mais de 12
/// níveis acima da média 5×5 — o papel branco num buraco da orla amaciada (folha D1, `oraculo/README.md`).
fn vaos_claros(px: &[u8], (w, h): (usize, usize)) -> usize {
    let at = |x: usize, y: usize| {
        let p = &px[(x + y * w) * 4..(x + y * w) * 4 + 3];
        let (r, g, b) = (f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
        (((b - r) / 120.0).clamp(0.0, 1.0), (r + g + b) / 3.0)
    };
    let mut n = 0;
    for y in 8..h - 8 {
        for x in 8..w - 8 {
            let (azul, lum) = at(x, y);
            if azul >= 0.1 {
                continue;
            }
            let perto = (y - 6..=y + 6).any(|yy| (x - 6..=x + 6).any(|xx| at(xx, yy).0 >= 0.5));
            if !perto {
                continue;
            }
            let media = (y - 2..=y + 2)
                .flat_map(|yy| (x - 2..=x + 2).map(move |xx| (xx, yy)))
                .map(|(xx, yy)| at(xx, yy).1)
                .sum::<f64>()
                / 25.0;
            n += usize::from(lum > media + 12.0);
        }
    }
    n
}

/// ⭐ D1 W3b: **com o Óleo Real o halo do Smooth Edges não tem VÃOS BRANCOS** — o campo amacia a silhueta da tinta
/// (cobertura e relevo), e o píxel sem pigmento da orla (a metade de fora da tenda, os vãos da orla rasgada) recebe
/// a tinta desfocada, sem relevo. Sem o enchimento (1.ª ronda) a folha mostrava 1 166 píxeis claros a Smooth 1
/// contra 5 da lei só da cor.
#[test]
fn com_o_oleo_o_halo_do_smooth_edges_nao_tem_vaos_brancos() {
    let mut n = [0usize; 2];
    for (k, so) in [false, true].into_iter().enumerate() {
        super::so_na_cor::set(so);
        let mut t = um_traco("Dabber", false);
        t.set_wet_smooth_edges(1.0);
        // A tela que se vê (com a luz do relevo): é nela que os vãos saltam.
        let mut px = t.run_full().0;
        t.apply_impasto_light(
            &mut px,
            crate::Region {
                x: 0,
                y: 0,
                w: W,
                h: H,
            },
        );
        n[k] = vaos_claros(&px, (W as usize, H as usize));
    }
    super::so_na_cor::set(false);
    println!("[vãos do halo] campo {} · só na cor {}", n[0], n[1]);
    assert!(
        n[0] <= n[1] + 10,
        "o halo com o relevo tem {} vãos claros (a lei só da cor {})",
        n[0],
        n[1]
    );
}

/// ⭐ D1 W3b: **com o Óleo Real a orla DA TINTA é a do campo** — o píxel com pigmento ao pé da orla (as 3×3 células
/// com alfa ≥ `SS0`, uma célula sem tinta a ≤ 2) mostra a cobertura que o campo lhe deu, a mesma do relevo: o
/// `Suave` dos píxeis não a amacia outra vez (só enche o papel e os degraus de cor longe da orla).
#[test]
fn com_o_oleo_a_orla_da_tinta_e_a_do_campo() {
    let corre = |sem: bool| {
        super::sem_suave::set(sem);
        let mut t = um_traco("Dabber", false);
        t.set_wet_smooth_edges(1.0);
        super::sem_suave::set(false);
        t
    };
    let (a, b) = (corre(false), corre(true));
    let s = a.paint.wetpaint.session.as_ref().expect("sessão");
    let (rr, (gw, gh)) = (usize::from(s.ratio), s.grid);
    let alfa = |cx: isize, cy: isize| {
        let (cx, cy) = (
            cx.clamp(0, gw as isize - 1) as usize,
            cy.clamp(0, gh as isize - 1) as usize,
        );
        s.pigment[(cy * gw + cx) * 4 + 3]
    };
    let ss0 = (crate::tool::paint::watercolor_render::SS0 * 255.0).ceil() as u8;
    let (mut orla, mut mexidos) = (0, 0);
    for y in 0..H as usize {
        for x in 0..W as usize {
            let (cx, cy) = ((x / rr) as isize, (y / rr) as isize);
            let tinta = (-1..=1).all(|dy| (-1..=1).all(|dx| alfa(cx + dx, cy + dy) >= ss0));
            let perto = (-2..=2).any(|dy| (-2..=2).any(|dx| alfa(cx + dx, cy + dy) == 0));
            if tinta && perto {
                orla += 1;
                let o = (x + y * W as usize) * 4;
                mexidos += usize::from(a.canvas_rgba[o..o + 4] != b.canvas_rgba[o..o + 4]);
            }
        }
    }
    println!("[orla da tinta] {orla} píxeis com tinta ao pé da orla · {mexidos} amaciados de novo");
    assert!(orla > 100, "controlo: a régua não achou a orla ({orla})");
    assert_eq!(
        mexidos, 0,
        "o Suave amaciou de novo {mexidos} píxeis da orla do campo"
    );
}

/// **Pegar um pincel não muda o Smooth Edges da tela** (D1 W3b: não viaja no preset de pincel).
#[test]
fn escolher_um_pincel_nao_muda_o_smooth_edges_da_tela() {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; 64 * 64 * 4], 64, 64);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_smooth_edges(0.7);
    for nome in ["Dabber", "Knife Oily"] {
        let p = crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato(nome);
        assert!(t.wet_preset_aplicar(&p).is_empty());
        assert_eq!(
            t.wet_smooth_edges(),
            0.7,
            "o pincel {nome} trouxe o Smooth Edges"
        );
    }
}

/// A FOLHA D1 W3b: o traço do Dabber com o Óleo Real e o Smooth Edges 0 · 0,5 · 1, antes (só a cor) | depois (a
/// silhueta no campo). `PH2D_OLEO_PNG=<pasta>` ⇒ `se_<antes|depois>_<s>.ppm`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_smooth_edges_relevo_folha() {
    let d = std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>");
    let (w, h) = (W, H);
    for (nome, so) in [("antes", true), ("depois", false)] {
        super::so_na_cor::set(so);
        let mut t = um_traco("Dabber", false);
        for s in [0.0f32, 0.5, 1.0] {
            t.set_wet_smooth_edges(s);
            let mut px = t.run_full().0;
            t.apply_impasto_light(&mut px, crate::Region { x: 0, y: 0, w, h });
            let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
            for p in px.as_chunks::<4>().0 {
                ppm.extend_from_slice(&p[..3]);
            }
            std::fs::write(format!("{d}/se_{nome}_{s}.ppm"), ppm).expect("grava");
        }
    }
    super::so_na_cor::set(false);
}
