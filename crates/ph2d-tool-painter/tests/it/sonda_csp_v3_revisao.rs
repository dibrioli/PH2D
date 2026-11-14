//! Sonda (08/11, revisão v3): a cena partilhada com o Clip Studio (2400×880: vazio · branco · blocos) pintada por 22
//! pincéis de fábrica pela RECEITA, no tamanho de fábrica do CSP (sem tecto). `PH2D_CSP_CENA=<pasta com base.rgba,
//! tracos.txt, pinceis.tsv>`; RGBA cru em `<pasta>/out_ph2d/<nome>.rgba`. `PH2D_V3_SO="A,B"` filtra. Corre à mão.

use crate::familias::FAMILIAS;
use crate::presets_de_fabrica::{confere_receita_do_meio, preset_do_meio};
use crate::receitas_de_fabrica::ReceitaMeio;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;
use std::path::PathBuf;

const W: usize = 2400;
const H: usize = 880;

fn receita(nome: &str) -> &'static ReceitaMeio {
    FAMILIAS
        .iter()
        .flat_map(|f| f.meios.iter())
        .find(|r| r.nome == nome)
        .unwrap_or_else(|| panic!("receita {nome} não existe"))
}

// Um ponto só = toque: Down, um tique, Up. `rampa`: a pressão sobe de 0,1 a 1,0 ao longo do traço (senão 1,0).
fn traca(t: &mut PainterTool, s: &[[f32; 2]], rampa: bool) {
    let n = s.len().saturating_sub(1).max(1) as f32;
    let pr = |i: usize| if rampa { 0.1 + 0.9 * i as f32 / n } else { 1.0 };
    let cp = |pos, pressure, phase| CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(s[0], pr(0), PointerPhase::Down));
    for (i, &p) in s.iter().enumerate().skip(1) {
        t.on_canvas_pointer(cp(p, pr(i), PointerPhase::Move));
        t.on_tick(8.0);
    }
    if s.len() == 1 {
        t.on_tick(8.0);
    }
    let fim = s.len() - 1;
    t.on_canvas_pointer(cp(s[fim], pr(fim), PointerPhase::Up));
    t.on_tick(8.0);
}

#[test]
#[ignore = "sonda"]
fn sonda_csp_v3_revisao() {
    let d = PathBuf::from(std::env::var("PH2D_CSP_CENA").expect("PH2D_CSP_CENA"));
    let o = d.join("out_ph2d");
    std::fs::create_dir_all(&o).expect("out_ph2d");
    let base = std::fs::read(d.join("base.rgba")).expect("base");
    assert_eq!(base.len(), W * H * 4);
    let tracos: Vec<Vec<[f32; 2]>> = std::fs::read_to_string(d.join("tracos.txt"))
        .expect("tracos")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            l.split_whitespace()
                .map(|p| {
                    let (x, y) = p.split_once(',').expect("x,y");
                    [x.parse().expect("x"), y.parse().expect("y")]
                })
                .collect()
        })
        .collect();
    assert_eq!(tracos.len(), 12, "12 traços");
    let so = std::env::var("PH2D_V3_SO").ok();
    // `PH2D_V3_PRESSAO=rampa` (pressão 0,1→1,0 ao longo de cada traço) e `PH2D_V3_PASSADAS=<n>` (a cena inteira
    // repete-se n vezes por cima); sem elas, o de sempre. A saída leva `__rampa`/`__x<n>` no nome.
    let rampa = std::env::var("PH2D_V3_PRESSAO").is_ok_and(|v| v == "rampa");
    let passadas: usize =
        std::env::var("PH2D_V3_PASSADAS").map_or(1, |v| v.parse().expect("passadas"));
    let cond = format!(
        "{}{}",
        if rampa { "__rampa" } else { "" },
        if passadas > 1 {
            format!("__x{passadas}")
        } else {
            String::new()
        }
    );
    let tsv = std::fs::read_to_string(d.join("pinceis.tsv")).expect("pinceis");
    let diam_de = |nome: &str| -> f32 {
        let l = tsv.lines().find(|l| l.split('\t').nth(2) == Some(nome));
        l.and_then(|l| l.split('\t').nth(1))
            .expect("no tsv")
            .parse()
            .expect("diâmetro")
    };
    // `PH2D_V3_VARIANTES=<ficheiro>`: por linha `Nome\tsufixo\tcampo=json\t…` (o campo do pincel; `mistura.<k>` dentro
    // da mistura opaca; `size` = o diâmetro) — afinar receitas sem regravar presets. Saída `<nome>__<sufixo>.rgba`.
    type Trabalho = (String, String, Vec<(String, String)>);
    let trabalhos: Vec<Trabalho> = match std::env::var("PH2D_V3_VARIANTES") {
        Ok(f) => std::fs::read_to_string(f)
            .expect("variantes")
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .map(|l| {
                let c: Vec<&str> = l.split('\t').collect();
                let aj = c[2..]
                    .iter()
                    .map(|a| {
                        let (k, v) = a.split_once('=').expect("campo=json");
                        (k.to_owned(), v.to_owned())
                    })
                    .collect();
                (c[0].to_owned(), format!("__{}", c[1]), aj)
            })
            .collect(),
        Err(_) => tsv
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                (
                    l.split('\t').nth(2).expect("nome").to_owned(),
                    String::new(),
                    Vec::new(),
                )
            })
            .collect(),
    };
    for (nome, sufixo, ajustes) in &trabalhos {
        let nome = nome.as_str();
        if so
            .as_deref()
            .is_some_and(|s| !s.split(',').any(|n| n == nome))
        {
            continue;
        }
        let t0 = std::time::Instant::now();
        // `base=<Nome>`: parte da receita de outro pincel (trocar o meio na afinação).
        let de = ajustes
            .iter()
            .find(|(k, _)| k == "base")
            .map_or(nome, |(_, v)| v.as_str());
        let r = receita(de);
        let mut p = preset_do_meio(r);
        confere_receita_do_meio(r, &p);
        let mut diam = diam_de(nome);
        for (k, v) in ajustes {
            if k == "base" {
                continue;
            }
            if let Some(c) = k.strip_prefix('-') {
                p.parametros.brush.remove(c);
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(v).expect("json do ajuste");
            if k == "opacity" {
                p.parametros.volumes.opacity = v.as_f64();
            } else if let Some(m) = k.strip_prefix("mao.") {
                p.parametros
                    .engine
                    .insert(m.to_owned(), v.as_f64().expect("mão"));
            } else if k == "size" {
                diam = v.as_f64().expect("size") as f32;
            } else if let Some(m) = k.strip_prefix("mistura.") {
                let o = p
                    .parametros
                    .brush
                    .get_mut("mistura_opaca")
                    .expect("sem mistura_opaca");
                let mut obj: serde_json::Value = match &*o {
                    serde_json::Value::String(s) => serde_json::from_str(s).expect("mistura"),
                    x => x.clone(),
                };
                obj[m] = v;
                *o = obj;
            } else {
                p.parametros.brush.insert(k.clone(), v);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(base.clone(), W as u32, H as u32);
        assert_eq!(t.preset_aplicar(&p), Ok(Vec::new()));
        t.sonda_raio(diam / 2.0); // raio; o diâmetro de fábrica do CSP
        t.set_brush_color_srgb8([0, 160, 60]);
        for _ in 0..passadas {
            for s in &tracos {
                traca(&mut t, s, rampa);
            }
        }
        let slug = format!("{}{sufixo}{cond}", nome.replace(' ', "_"));
        std::fs::write(o.join(format!("{slug}.rgba")), t.run_full().0).expect("grava");
        eprintln!("[v3] {nome} ok {:.1}s", t0.elapsed().as_secs_f64());
    }
}
