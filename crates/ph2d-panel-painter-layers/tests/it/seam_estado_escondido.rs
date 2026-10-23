//! **Estado escondido ou desligado não pinta** (captura do dono, 22/10): três manchas na Aquarela — a
//! 1.ª azul normal; a 2.ª com Voronoi + Grain Colors; depois «Same as Paper» marcado e a 3.ª azul, com
//! as MESMAS configurações visíveis da 1.ª, saía com o miolo quase branco e só a orla azul.
//!
//! A régua é a SEQUÊNCIA de cliques do painel (a [`Bancada`]) e a tela da ponte: a mancha pintada
//! depois de um interruptor que neutraliza tem de ser byte-igual à do pincel limpo na mesma sequência,
//! e o preview do pincel também. ⭐ A família inteira é a tabela [`INTERRUPTORES`]: interruptor novo que
//! esconde ou desliga algo entra nela.

use super::censo_dos_controlos::{Bancada, Entrega, Gesto, diferenca, em_paralelo, tique, traco};
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};
use ph2d_tool_painter::ids::*;
use ph2d_tool_painter::{LINHAS_DA_PONTA, LinhaDaPonta, PaintMedia, SrgbRgba};

const W: u32 = 256;
const H: u32 = 128;
/// Os centros das três manchas, longe o bastante para a borda de uma não chegar à vizinha.
const CENTROS: [f32; 3] = [44.0, 128.0, 212.0];
const AZUL: [u8; 3] = [40, 110, 220];
const VERMELHO: [u8; 3] = [220, 40, 40];
const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Watercolor,
    PaintMedia::Digital,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// Um passo do artista: um gesto pelo painel, um evento pela porta do contrato (o que a shell manda)
/// ou uma imagem escolhida no seletor da shell.
#[derive(Clone, Debug)]
enum Passo {
    G(Gesto),
    E(PanelEvent),
    /// A imagem da Shape (a ponta de uma imagem).
    Ponta,
    /// `n` imagens a mais na ponta (a ponta de N imagens).
    Mangueira(usize),
    /// A imagem da 2.ª ponta (o dual brush).
    ImagemDoDual,
}

/// Uma imagem 16² de tom `c` com riscas diagonais (a silhueta tem textura).
fn img_bytes(c: u8) -> Vec<u8> {
    (0..16 * 16)
        .flat_map(|i| {
            let v = if (i % 16 + i / 16) % 3 == 0 { c } else { 255 };
            [v, v, v, 255]
        })
        .collect()
}

fn img(c: u8) -> Vec<SrgbRgba> {
    SrgbRgba::iter_byte_slice(&img_bytes(c)).collect()
}

fn aplica(b: &mut Bancada, passos: &[Passo]) {
    for p in passos {
        match p {
            Passo::G(g) => {
                let mut e = Entrega::default();
                b.aplica(g, &mut e);
                assert!(e.ao_tool > 0, "o gesto {g:?} não chegou à ferramenta");
            }
            Passo::E(ev) => b.tool.handle_panel_event(ev.clone()),
            Passo::Ponta => {
                b.tool
                    .set_brush_shape_image_rgba(&img_bytes(90), 16, 16, Some(7));
            }
            Passo::Mangueira(n) => {
                for k in 0..*n {
                    b.tool
                        .add_brush_shape_hose_image_rgba(&img(40 + 50 * k as u8), 16, 16);
                }
            }
            Passo::ImagemDoDual => b.tool.set_brush_dual_image_rgba(&img(30), 16, 16),
        }
        b.aplica(&Gesto::Nenhum, &mut Entrega::default());
    }
}

/// Uma mancha: um rabisco curto em `cx`.
fn mancha(b: &mut Bancada, cx: f32, cor: [u8; 3], ultima: &mut Option<Vec<u8>>) {
    b.tool.set_brush_color_srgb8(cor);
    traco(
        &mut b.tool,
        ultima,
        [cx - 14.0, 56.0],
        [cx + 14.0, 70.0],
        [1.0, 1.0],
        [0.0, 0.0],
    );
    for _ in 0..4 {
        tique(&mut b.tool, ultima);
    }
}

/// As três manchas e o preview do pincel no fim: a 1.ª com o pincel armado, `entre_1_2` (os cliques que
/// sujam), a 2.ª vermelha, `entre_2_3` (o interruptor que neutraliza), a 3.ª azul.
fn tres_manchas(
    meio: PaintMedia,
    armar: &[Passo],
    entre_1_2: &[Passo],
    entre_2_3: &[Passo],
) -> (Vec<u8>, Vec<u8>) {
    let mut b = Bancada::nova(meio, &[]);
    b.tool.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    aplica(&mut b, armar);
    let mut ultima = None;
    mancha(&mut b, CENTROS[0], AZUL, &mut ultima);
    aplica(&mut b, entre_1_2);
    mancha(&mut b, CENTROS[1], VERMELHO, &mut ultima);
    aplica(&mut b, entre_2_3);
    // ⚠️ **O Wet Paint acopla manchas LONGE umas das outras pela cadência de secagem**, que é GLOBAL: o
    // `ph2d_wet_paint::sim` escolhe `dry_every`/`evap_scale` pelo `vmax` da grade inteira. Medido (22/10): a 3.ª
    // mancha difere em `175` texels (Δ5) quando a 2.ª foi pintada com Voronoi — a cor, a pressão e a posição da
    // 2.ª não mudam nada; alternar o grão SEM pintar não muda nada; e o estado do pincel depois do interruptor é
    // o mesmo (o `macro_estado` inteiro igual). Deixar a 2.ª assentar não basta: o escorrido dela continua a
    // correr (6000 quadros: 776 texels). É física da SESSÃO, não estado escondido do pincel ⇒ no Wet Paint a 3.ª
    // mancha pinta-se numa tela nova (a sessão acaba), depois de TODA a sequência suja pintada.
    if meio == PaintMedia::WetPaint {
        b.tool.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    }
    mancha(&mut b, CENTROS[2], AZUL, &mut ultima);
    for _ in 0..30 {
        tique(&mut b.tool, &mut ultima);
    }
    let mut preview = Vec::new();
    ph2d_tool_painter::pintar_preview(&b.tool.wet_preview_cena(), &|| false, &mut |px, _, _| {
        preview = px.to_vec();
    });
    (
        ultima.expect("as manchas sujaram a pré-visualização"),
        preview,
    )
}

/// Os texels da faixa da 3.ª mancha (`x ≥ 170`).
fn faixa_3(px: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    for y in 0..H {
        let i = ((y * W + 170) * 4) as usize;
        let f = ((y * W + W) * 4) as usize;
        v.extend_from_slice(&px[i..f]);
    }
    v
}

/// A luminância média do miolo da 3.ª mancha (o centro, 9×9).
fn miolo_3(px: &[u8]) -> f32 {
    let (cx, cy) = (CENTROS[2] as u32, 63u32);
    let mut s = 0.0;
    for y in cy - 4..=cy + 4 {
        for x in cx - 4..=cx + 4 {
            let i = ((y * W + x) * 4) as usize;
            s += 0.2126 * f32::from(px[i])
                + 0.7152 * f32::from(px[i + 1])
                + 0.0722 * f32::from(px[i + 2]);
        }
    }
    s / 81.0
}

/// Grava a tela em PNG quando `VAZAMENTO_FOTOS=<pasta>` (as fotos do handoff).
fn foto(nome: &str, px: &[u8]) {
    if let Ok(dir) = std::env::var("VAZAMENTO_FOTOS") {
        let p = std::path::Path::new(&dir).join(format!("{nome}.png"));
        image::save_buffer(&p, px, W, H, image::ExtendedColorType::Rgba8).expect("png gravado");
    }
}

fn g(id: ph2d_a11y::NodeId) -> Passo {
    Passo::G(Gesto::Clique(id))
}

fn textura(k: u8) -> Passo {
    Passo::G(Gesto::Opcao(
        PAINTER_BRUSH_TEXTURE_KIND,
        painter_brush_texture_kind_option_id(k),
        0,
        0,
    ))
}

fn clique(id: ph2d_a11y::NodeId) -> Passo {
    Passo::E(PanelEvent::Click(id))
}

fn ponta(l: LinhaDaPonta, k: u8) -> Passo {
    let i = LINHAS_DA_PONTA
        .iter()
        .position(|x| *x == l)
        .expect("a linha da ponta");
    Passo::E(PanelEvent::SelectOption(
        PAINTER_PONTA_FIELDS[i],
        k.to_string(),
    ))
}

fn forma(k: u8) -> Passo {
    Passo::E(PanelEvent::SelectOption(PAINTER_SHAPE_KIND, k.to_string()))
}

fn meio_evento(m: PaintMedia) -> Passo {
    let k = match m {
        PaintMedia::Digital => 0,
        PaintMedia::Watercolor => 1,
        PaintMedia::Impasto => 2,
        PaintMedia::WetPaint => 3,
    };
    Passo::E(PanelEvent::SelectOption(PAINTER_BRUSH_MEDIA, k.to_string()))
}

/// «Same as Paper» desligado (a Aquarela de fábrica o tem ligado; só ela o tem).
fn grao_proprio(m: PaintMedia) -> Vec<Passo> {
    if m == PaintMedia::Watercolor {
        vec![g(PAINTER_WATERCOLOR_GRAN_SAME)]
    } else {
        Vec::new()
    }
}

/// O grão do artista: Voronoi + Grain Colors (com o grão próprio na Aquarela).
fn grao_com_cores(m: PaintMedia) -> Vec<Passo> {
    let mut v = grao_proprio(m);
    v.extend([textura(3), g(PAINTER_BRUSH_TEXTURE_RAMP_ENABLE)]);
    v
}

/// Um interruptor da família: em `meio`, `armar` (os dois lados), `sujar` (só o lado sujo, entre a 1.ª e
/// a 2.ª mancha), `neutraliza` (entre a 2.ª e a 3.ª) e `limpo` (o lado limpo, com o que fica VISÍVEL).
struct Interruptor {
    nome: &'static str,
    meios: &'static [PaintMedia],
    /// O controlo de não-vácuo se aplica (o sujo sem o interruptor pinta outra coisa).
    controlo: bool,
    caso: fn(PaintMedia) -> Caso,
}

struct Caso {
    armar: Vec<Passo>,
    sujar: Vec<Passo>,
    neutraliza: Vec<Passo>,
    limpo: Vec<Passo>,
}

const TODOS: &[PaintMedia] = &MEIOS;

/// ⭐ **A FAMÍLIA** — cada interruptor que esconde ou desliga algo do pincel.
const INTERRUPTORES: &[Interruptor] = &[
    Interruptor {
        nome: "Same as Paper",
        meios: &[PaintMedia::Watercolor],
        controlo: true,
        caso: |m| Caso {
            armar: vec![],
            sujar: grao_com_cores(m),
            neutraliza: vec![g(PAINTER_WATERCOLOR_GRAN_SAME)],
            limpo: vec![],
        },
    },
    Interruptor {
        nome: "Grain None",
        meios: TODOS,
        controlo: true,
        caso: |m| Caso {
            armar: vec![],
            sujar: grao_com_cores(m),
            neutraliza: vec![textura(0)],
            limpo: grao_proprio(m),
        },
    },
    Interruptor {
        nome: "Use Color Ramp off",
        meios: TODOS,
        controlo: true,
        caso: |m| Caso {
            armar: vec![],
            sujar: grao_com_cores(m),
            neutraliza: vec![g(PAINTER_BRUSH_TEXTURE_RAMP_ENABLE)],
            limpo: {
                let mut v = grao_proprio(m);
                v.push(textura(3));
                v
            },
        },
    },
    Interruptor {
        nome: "Paint Mode ida e volta",
        meios: TODOS,
        // O visível é o MESMO nos dois lados por construção: não há o que o controlo medir.
        controlo: false,
        caso: |m| Caso {
            armar: vec![],
            sujar: grao_com_cores(m),
            neutraliza: vec![
                meio_evento(if m == PaintMedia::Digital {
                    PaintMedia::Watercolor
                } else {
                    PaintMedia::Digital
                }),
                meio_evento(m),
            ],
            limpo: grao_com_cores(m),
        },
    },
    Interruptor {
        nome: "Dual brush off",
        // A Aquarela e o Impasto não leem a 2.ª ponta (o censo dos controlos responde por ela lá).
        meios: &[PaintMedia::Digital, PaintMedia::WetPaint],
        controlo: true,
        caso: |_| Caso {
            armar: vec![],
            sujar: vec![ponta(LinhaDaPonta::DualMode, 4), Passo::ImagemDoDual],
            neutraliza: vec![ponta(LinhaDaPonta::DualMode, 0)],
            limpo: vec![],
        },
    },
    Interruptor {
        nome: "N imagens -> 1",
        // A aguada carimba a silhueta de UMA imagem.
        meios: &[
            PaintMedia::Digital,
            PaintMedia::Impasto,
            PaintMedia::WetPaint,
        ],
        controlo: true,
        caso: |_| Caso {
            armar: vec![Passo::Ponta],
            sujar: vec![Passo::Mangueira(2)],
            neutraliza: vec![clique(PAINTER_SHAPE_HOSE_CLEAR)],
            limpo: vec![],
        },
    },
    Interruptor {
        nome: "Composite off",
        // A pilha de camadas do traço é do carimbo (Digital, Impasto).
        meios: &[PaintMedia::Digital, PaintMedia::Impasto],
        controlo: true,
        caso: |_| Caso {
            armar: vec![],
            sujar: vec![
                clique(PAINTER_BRUSH_COMPOSITE_ENABLE),
                clique(PAINTER_BRUSH_COMPOSITE_ADD_OPTION[2]),
                clique(PAINTER_BRUSH_COMPOSITE_ADD),
            ],
            neutraliza: vec![clique(PAINTER_BRUSH_COMPOSITE_ENABLE)],
            limpo: vec![],
        },
    },
    Interruptor {
        nome: "Shape Color off",
        meios: TODOS,
        controlo: true,
        caso: |_| Caso {
            armar: vec![Passo::Ponta],
            sujar: vec![clique(PAINTER_SHAPE_RAMP_ENABLE)],
            neutraliza: vec![clique(PAINTER_SHAPE_RAMP_ENABLE)],
            limpo: vec![],
        },
    },
    Interruptor {
        nome: "Reset do Grain",
        meios: TODOS,
        controlo: true,
        caso: |m| Caso {
            armar: vec![],
            sujar: grao_com_cores(m),
            neutraliza: vec![clique(PAINTER_BRUSH_TEXTURE_RESET)],
            limpo: {
                let mut v = grao_proprio(m);
                v.push(clique(PAINTER_BRUSH_TEXTURE_RESET));
                v
            },
        },
    },
    Interruptor {
        nome: "Reset da Shape",
        meios: &[
            PaintMedia::Digital,
            PaintMedia::Impasto,
            PaintMedia::WetPaint,
        ],
        controlo: true,
        caso: |_| Caso {
            armar: vec![],
            sujar: vec![Passo::Ponta, Passo::Mangueira(2)],
            neutraliza: vec![clique(PAINTER_SHAPE_RESET)],
            limpo: vec![clique(PAINTER_SHAPE_RESET)],
        },
    },
    Interruptor {
        nome: "Forma fora de Image",
        meios: &[
            PaintMedia::Digital,
            PaintMedia::Impasto,
            PaintMedia::WetPaint,
        ],
        controlo: true,
        caso: |_| Caso {
            armar: vec![Passo::Ponta],
            sujar: vec![Passo::Mangueira(2)],
            neutraliza: vec![forma(0)],
            limpo: vec![forma(0)],
        },
    },
    Interruptor {
        nome: "Reset da Shape Color",
        meios: TODOS,
        controlo: true,
        caso: |_| Caso {
            armar: vec![Passo::Ponta],
            sujar: vec![
                clique(PAINTER_SHAPE_RAMP_ENABLE),
                clique(PAINTER_SHAPE_TEXTURE_COLOR),
            ],
            neutraliza: vec![clique(PAINTER_SHAPE_RAMP_RESET)],
            limpo: vec![clique(PAINTER_SHAPE_RAMP_RESET)],
        },
    },
    Interruptor {
        nome: "Falloff Color off",
        // Com a ponta de imagem, a Falloff Color só muda a tinta na aguada (medido: 0 texels nos outros três).
        meios: &[PaintMedia::Watercolor],
        controlo: true,
        caso: |_| Caso {
            armar: vec![Passo::Ponta],
            sujar: vec![
                clique(PAINTER_SHAPE_RAMP_ENABLE),
                clique(PAINTER_SHAPE_FALLOFF_COLOR),
            ],
            neutraliza: vec![clique(PAINTER_SHAPE_FALLOFF_COLOR)],
            limpo: vec![clique(PAINTER_SHAPE_RAMP_ENABLE)],
        },
    },
];

/// A linha da tabela de um interruptor num meio.
struct Linha {
    nome: &'static str,
    meio: PaintMedia,
    /// Texels da faixa da 3.ª mancha diferentes (sujo+neutralizado × limpo) e o Δ máximo.
    tela: (usize, u8),
    /// O mesmo no preview do pincel.
    preview: (usize, u8),
    /// O CONTROLO: o sujo SEM o interruptor difere do limpo (o caso não é vácuo).
    vivo: usize,
    controlo: bool,
}

fn mede(i: &Interruptor, m: PaintMedia) -> Linha {
    let c = (i.caso)(m);
    let (sujo, pv_sujo) = tres_manchas(m, &c.armar, &c.sujar, &c.neutraliza);
    let (limpo, pv_limpo) = tres_manchas(m, &c.armar, &c.limpo, &[]);
    let (vivo_px, _) = tres_manchas(m, &c.armar, &c.sujar, &[]);
    let d = diferenca(&faixa_3(&sujo), &faixa_3(&limpo));
    let p = diferenca(&pv_sujo, &pv_limpo);
    let nome = format!("{}_{m:?}", i.nome.replace([' ', '>', '-'], ""));
    if d.0 > 0 {
        foto(&format!("vazamento_{nome}_sujo"), &sujo);
        foto(&format!("vazamento_{nome}_limpo"), &limpo);
    }
    Linha {
        nome: i.nome,
        meio: m,
        tela: (d.0, d.2),
        preview: (p.0, p.2),
        vivo: diferenca(&faixa_3(&vivo_px), &faixa_3(&limpo)).0,
        controlo: i.controlo,
    }
}

/// A captura do dono: Voronoi + Grain Colors (Same as Paper desligado) → a 2.ª mancha → Same as Paper
/// marcado → a 3.ª, que tem de ser a do pincel limpo (antes da cura: miolo `247,6` contra `208,2`).
#[test]
fn a_mancha_depois_de_same_as_paper_e_a_do_pincel_limpo() {
    let m = PaintMedia::Watercolor;
    let (com, _) = tres_manchas(
        m,
        &[],
        &grao_com_cores(m),
        &[g(PAINTER_WATERCOLOR_GRAN_SAME)],
    );
    let (limpo, _) = tres_manchas(m, &[], &[], &[]);
    foto("vazamento_same_as_paper", &com);
    foto("vazamento_pincel_limpo", &limpo);
    let (n, _, max) = diferenca(&faixa_3(&com), &faixa_3(&limpo));
    let (mc, ml) = (miolo_3(&com), miolo_3(&limpo));
    assert_eq!(
        n, 0,
        "a 3.ª mancha difere da do pincel limpo em {n} texels (Δmax {max}); miolo {mc:.1} vs {ml:.1}"
    );
}

/// ⭐⭐ **A família**: cada interruptor de [`INTERRUPTORES`] em cada meio onde age — sequência suja →
/// o interruptor → a mancha (e o preview) == as do pincel limpo; e o controlo: sem o interruptor, o
/// sujo pinta OUTRA coisa (senão a igualdade é vácua). `TABELA` imprime a linha de cada um.
#[test]
fn cada_interruptor_que_neutraliza_pinta_como_o_pincel_limpo() {
    let pares: Vec<(usize, PaintMedia)> = INTERRUPTORES
        .iter()
        .enumerate()
        .flat_map(|(k, i)| i.meios.iter().map(move |m| (k, *m)))
        .collect();
    let linhas = em_paralelo(&pares, |(k, m)| mede(&INTERRUPTORES[*k], *m));
    let mut falhas = Vec::new();
    for l in &linhas {
        eprintln!(
            "TABELA\t{}\t{:?}\ttela n={} Δ{}\tpreview n={} Δ{}\tvivo={}",
            l.nome, l.meio, l.tela.0, l.tela.1, l.preview.0, l.preview.1, l.vivo
        );
        if l.tela.0 > 0 || l.preview.0 > 0 || (l.controlo && l.vivo == 0) {
            falhas.push(format!(
                "{} {:?}: tela {:?} preview {:?} vivo {}",
                l.nome, l.meio, l.tela, l.preview, l.vivo
            ));
        }
    }
    assert!(falhas.is_empty(), "vazam (ou vácuos): {falhas:#?}");
}

/// Onde mora cada controlo que ESCONDE linhas do painel: o nome do interruptor em [`INTERRUPTORES`], outro portão,
/// ou `fora:` com o motivo.
enum Mora {
    Aqui(&'static str),
    Portao(&'static str),
    Fora(&'static str),
}

fn linha(l: LinhaDaPonta) -> ph2d_a11y::NodeId {
    PAINTER_PONTA_FIELDS[LINHAS_DA_PONTA
        .iter()
        .position(|x| *x == l)
        .expect("a linha da ponta")]
}

/// ⭐ **O CENSO** — cada controlo que, no pincel com tudo armado ([`tudo_armado`]), faz linhas SUMIREM do painel
/// (medido pela [`sonda_quem_esconde`], 22/10, nos meios Digital · Impasto · Watercolor). Controlo novo que esconde
/// tem de entrar aqui — e a sonda reprova se aparecer um que não está.
fn censo() -> Vec<(ph2d_a11y::NodeId, Mora)> {
    use Mora::*;
    let mut v = vec![
        (PAINTER_WATERCOLOR_GRAN_SAME, Aqui("Same as Paper")),
        (PAINTER_BRUSH_TEXTURE_KIND, Aqui("Grain None")),
        (
            PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
            Aqui("Use Color Ramp off"),
        ),
        (PAINTER_BRUSH_MEDIA, Aqui("Paint Mode ida e volta")),
        (linha(LinhaDaPonta::DualMode), Aqui("Dual brush off")),
        (PAINTER_SHAPE_HOSE_CLEAR, Aqui("N imagens -> 1")),
        (PAINTER_BRUSH_COMPOSITE_ENABLE, Aqui("Composite off")),
        (PAINTER_SHAPE_RAMP_ENABLE, Aqui("Shape Color off")),
        (PAINTER_BRUSH_TEXTURE_RESET, Aqui("Reset do Grain")),
        (PAINTER_SHAPE_RESET, Aqui("Reset da Shape")),
        (PAINTER_SHAPE_KIND, Aqui("Forma fora de Image")),
        (PAINTER_SHAPE_RAMP_RESET, Aqui("Reset da Shape Color")),
        (PAINTER_SHAPE_FALLOFF_COLOR, Aqui("Falloff Color off")),
        (
            PAINTER_BRUSH_PRESET,
            Portao("ph2d-tool-painter::o_preset_nao_herda"),
        ),
        (
            PAINTER_SHAPE_TEXTURE_COLOR,
            Fora(
                "a camada sem cor é a da rampa ligada: a Reset da Shape Color a cobre; o toggle sozinho mede 0 texels",
            ),
        ),
        (
            PAINTER_SHAPE_RAMP_REMOVE,
            Fora("tira um stop da rampa VISÍVEL: o que fica é o que se vê"),
        ),
        (
            PAINTER_BRUSH_COLOR_RAMP_RESET,
            Fora("repõe a rampa VISÍVEL das Grain Colors: o que fica é o que se vê"),
        ),
        (
            PAINTER_BRUSH_TEXTURE_MAPPING,
            Fora(
                "troca o referencial do grão; as linhas que saem são do outro — o censo dos controlos mede-as por estado",
            ),
        ),
        (
            PAINTER_IMPASTO_TOOL_KNIFE,
            Fora("outra ferramenta, outro pincel (`brush_by_mode`)"),
        ),
        (
            PAINTER_LINE_SOLID,
            Fora("o estilo do traço muda as linhas do próprio estilo"),
        ),
        (
            PAINTER_SCULPT_MODE_CHISEL,
            Fora("modo da escultura: outra ferramenta"),
        ),
        (
            PAINTER_SCULPT_MODE_FILL,
            Fora("modo da escultura: outra ferramenta"),
        ),
        (
            PAINTER_SCULPT_MODE_FLATTEN,
            Fora("modo da escultura: outra ferramenta"),
        ),
        (
            PAINTER_SCULPT_MODE_INFLATE,
            Fora("modo da escultura: outra ferramenta"),
        ),
        (
            PAINTER_SCULPT_MODE_LAYER,
            Fora("modo da escultura: outra ferramenta"),
        ),
        (
            PAINTER_SCULPT_MODE_SCRAPE,
            Fora("modo da escultura: outra ferramenta"),
        ),
        (
            PAINTER_SCULPT_MODE_SHARPEN,
            Fora("modo da escultura: outra ferramenta"),
        ),
        (
            PAINTER_SCULPT_MODE_SMOOTH,
            Fora("modo da escultura: outra ferramenta"),
        ),
        (
            PAINTER_SHAPE_PER_LAYER_COLOR,
            Portao("seam_color_inativo (Bug #51)"),
        ),
        (
            PAINTER_BRUSH_TEXTURE_RAMP_REMOVE,
            Fora("tira um stop da rampa VISÍVEL: o que fica é o que se vê"),
        ),
        (
            PAINTER_SHAPE_FALLOFF_RAMP_REMOVE,
            Fora("tira um stop da rampa VISÍVEL: o que fica é o que se vê"),
        ),
        (
            ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_METHOD,
            Fora(
                "outro método do traço, com as linhas dele — o censo dos controlos as mede por estado",
            ),
        ),
        (
            linha(LinhaDaPonta::HoseMode),
            Fora("a ordem das N imagens: só o sorteio lê os pesos que saem"),
        ),
        (
            linha(LinhaDaPonta::DualTip),
            Fora("a forma da 2.ª ponta escolhe a fonte: a imagem só se lê com Image"),
        ),
    ];
    v.extend(
        PAINTER_BRUSH_COMPOSITE_REMOVE
            .iter()
            .map(|id| (*id, Fora("retira a camada: as linhas dela saem com ela"))),
    );
    v
}

/// O censo não aponta para um interruptor que não existe, nem repete um controlo.
#[test]
fn o_censo_dos_que_escondem_aponta_para_interruptores_que_existem() {
    let c = censo();
    for (k, (id, mora)) in c.iter().enumerate() {
        assert!(
            !c[..k].iter().any(|(j, _)| j == id),
            "{id:?} repetido no censo"
        );
        if let Mora::Aqui(n) = mora {
            assert!(
                INTERRUPTORES.iter().any(|i| i.nome == *n),
                "o censo aponta para «{n}», que não está em INTERRUPTORES"
            );
        }
        if let Mora::Fora(m) | Mora::Portao(m) = mora {
            assert!(!m.is_empty());
        }
    }
    for i in INTERRUPTORES {
        assert!(
            c.iter()
                .any(|(_, m)| matches!(m, Mora::Aqui(n) if *n == i.nome)),
            "«{}» não tem o controlo no censo",
            i.nome
        );
    }
}

/// O pincel com TUDO o que a família liga: o grão com cores, o dual com imagem, a ponta de N imagens, a
/// pilha do Composite e a Shape Color.
fn tudo_armado(m: PaintMedia) -> Vec<Passo> {
    let mut v = grao_com_cores(m);
    v.extend([
        ponta(LinhaDaPonta::DualMode, 4),
        Passo::ImagemDoDual,
        Passo::Ponta,
        Passo::Mangueira(2),
        clique(PAINTER_BRUSH_COMPOSITE_ENABLE),
        clique(PAINTER_BRUSH_COMPOSITE_ADD_OPTION[2]),
        clique(PAINTER_BRUSH_COMPOSITE_ADD),
        clique(PAINTER_SHAPE_RAMP_ENABLE),
    ]);
    v
}

fn armada(m: PaintMedia) -> Bancada {
    let mut b = Bancada::nova(m, &[]);
    aplica(&mut b, &tudo_armado(m));
    b
}

/// A régua do [`censo`]: cada gesto do painel (sem números nem sliders, que mudam valores) no pincel com tudo
/// armado; o que faz linhas SUMIREM tem de estar no censo. ⚠️ Lenta (12 min+ com o Wet Paint: cada gesto
/// refaz a bancada armada) — por isso `#[ignore]`; `CENSO_MEIOS=Digital,Watercolor` restringe.
#[test]
#[ignore = "sonda lenta: o censo dos controlos que escondem"]
fn sonda_quem_esconde() {
    use super::censo_dos_controlos::{gestos_da, nome, nomes};
    let nomes = nomes();
    let conhecidos: Vec<ph2d_a11y::NodeId> = censo().iter().map(|(i, _)| *i).collect();
    let pedidos = std::env::var("CENSO_MEIOS").unwrap_or_default();
    let mut novos = Vec::new();
    for m in MEIOS
        .into_iter()
        .filter(|m| pedidos.is_empty() || pedidos.contains(&format!("{m:?}")))
    {
        let antes: Vec<ph2d_a11y::NodeId> = armada(m).rects().iter().map(|(i, _)| *i).collect();
        let gs: Vec<Gesto> = gestos_da(armada(m))
            .into_iter()
            .filter(|g| matches!(g, Gesto::Clique(_) | Gesto::Opcao(..)))
            .collect();
        let r = em_paralelo(&gs, |g| {
            let mut b = armada(m);
            let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                b.aplica(g, &mut Entrega::default());
            }))
            .is_ok();
            let depois: Vec<ph2d_a11y::NodeId> = b.rects().iter().map(|(i, _)| *i).collect();
            ok.then(|| antes.iter().filter(|i| !depois.contains(i)).count())
        });
        for (g, n) in gs.iter().zip(r) {
            let dono = match g {
                Gesto::Opcao(menu, ..) => *menu,
                _ => g.id().unwrap_or(ph2d_a11y::NodeId(0)),
            };
            let secao = nome(&nomes, dono).contains("SECTION");
            if n.is_some_and(|n| n > 0) && !secao && !conhecidos.contains(&dono) {
                novos.push(format!("{m:?} {} {g:?}", nome(&nomes, dono)));
            }
        }
    }
    assert!(
        novos.is_empty(),
        "controlos que escondem e não estão no censo: {novos:#?}"
    );
}
