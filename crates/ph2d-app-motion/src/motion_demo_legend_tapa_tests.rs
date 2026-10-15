//! ⭐⭐⭐ **NENHUMA FICHA TAPA A FIGURA QUE ELA EXPLICA** — em TODAS as cenas do roteador, em todo
//! quadro de duas voltas do ciclo.
//!
//! A ficha da casa TAPA o que está por baixo (lei do `paint_chip`), logo uma legenda pousada sobre a
//! figura esconde justamente o que ela manda ver — achado na foto da `=142` (15/10), curado só ali; a
//! 1.ª medição desta régua achou 95 das 192 fichas a tapar.
//!
//! ## A régua (o que se mede, e o que se DECLARA)
//!
//! - **A figura é o que o canvas PINTA, cozido:** a cena monta-se pela porta dos testes, o quadro
//!   corre como na ponte (`publish_all` + a bomba) e lê-se o que a bomba BAIXOU — `instances` (quad de
//!   sprite: `P + basis·(anchor + [±½]·size)`) e `vector_instances` (forma: a caixa que o caminho
//!   PINTA, `path_visual_bounds`, sob a MESMA `instance_pose` do `encode`; imagem: o quad `[±½]²`) —
//!   e, sob a lei do dono, o GIZMO de posições dos sinks sem aparência ([`gizmo`]). Cada peça entra
//!   pela caixa alinhada aos eixos — conservadora para uma peça girada. ⛔ Não entra o que não sai
//!   daí (o halo do glow, as guias de edição).
//! - **A ficha é a do produto:** largura `readout::chip_width` (o Inter embutido — o `paint_chip`
//!   mede o latim com ele) e altura `CHIP_H_PX`, px de ecrã levados ao mundo pela câmara de arranque
//!   da cena: `Camera2d::default()` (10 m de altura) na janela da cena sob o split que a ferramenta
//!   Motion põe ao entrar (`scene_window_wh`), numa janela de [`JANELA`] (a foto do smoke; o monitor
//!   do dono é 2560×1080). Dá 57,2 px/m — conferido na foto (`antes_82.png`: 2 m = 112–114 px).
//! - **Só conta o que se VÊ** ([`CANVAS_VISTO_PX`]): uma ficha (ou um pedaço dela) por baixo da barra
//!   de ferramentas ou do grafo não esconde nada que o artista veja. ⚠️ Ficha fora do canvas visto é
//!   OUTRO defeito, que a sonda lista (`FORA-DA-VISTA`).
//! - **O cenário não conta:** uma peça que atravessa o canvas visto de lado a lado ou de cima a baixo
//!   é o fundo — não há onde pousar fora dela.
//! - **O tempo:** `2 · CICLO` segundos a 60 Hz, o quadro inteiro (`publish_all` + bomba) em TODO
//!   tique ([`PASSO`]).

use ph2d_editor_core::readout::{CHIP_H_PX, chip_width};
use ph2d_editor_core::screens::layout::CenterSplit;
use ph2d_host::WindowSize;
use ph2d_render::camera_frame::{Area, View};
use ph2d_vector::Affine;

use super::Caption;
use crate::motion_state::MotionState;

/// A janela da foto do smoke (`fotografa_cena.sh … 1930 1040`).
pub(super) const JANELA: (u32, u32) = (1930, 1040);
/// O que se VÊ do canvas nessa janela (foto `antes_82.png`, 15/10), px: entre o painel Hierarchy e o
/// Inspector, abaixo da barra de ferramentas (que tapa os 96 px de cima da cena) e acima do grafo.
pub(super) const CANVAS_VISTO_PX: [f32; 4] = [332.0, 96.0, 1622.0, 566.0];
/// De quantos em quantos tiques se lê a figura — TODOS: as entradas publicam-se por quadro, e a 2
/// a régua perdia os gatilhos ímpares (a `=139` media a galáxia com 3,5 m de raio; a foto, 4,1).
const PASSO: usize = 1;
/// Folga entre uma ficha curada e a figura, px de ecrã.
const AR_PX: f32 = 4.0;

/// ⭐ **As fichas que TAPAM e não têm para onde ir** — medido: nem na coluna delas (dentro do canvas
/// visto) nem por fora da metade delas há um vão do tamanho da ficha em algum quadro. ⛔ O gate
/// RE-MEDE cada uma: se um lugar aparecer, ou se ela deixar de tapar, a isenção reprova.
const SEM_LUGAR: &[(&str, &str, &str)] = &[
    (
        "95",
        "ATTRACTOR as before",
        "as forças levam as plumas a x −42…61 · y −39…35 m: a coluna e a faixa inteiras",
    ),
    (
        "95",
        "ATTRACTOR with PROFILE",
        "as forças levam as plumas a x −42…61 · y −39…35 m: a coluna e a faixa inteiras",
    ),
    (
        "95",
        "WIND Force",
        "as forças levam as plumas a x −42…61 · y −39…35 m: a coluna e a faixa inteiras",
    ),
    (
        "95",
        "WIND Target Velocity",
        "as forças levam as plumas a x −42…61 · y −39…35 m: a coluna e a faixa inteiras",
    ),
    (
        "95",
        "VORTEX Target Velocity",
        "as forças levam as plumas a x −42…61 · y −39…35 m: a coluna e a faixa inteiras",
    ),
    (
        "95",
        "SEA of ONE wave",
        "as forças levam as plumas a x −42…61 · y −39…35 m: a coluna e a faixa inteiras",
    ),
];

/// Uma caixa de mundo `[x0, y0, x1, y1]`.
pub(super) type Caixa = [f32; 4];

/// A janela da CENA sob o split que a ferramenta Motion põe ao entrar.
pub(super) fn janela_da_cena() -> WindowSize {
    crate::field_gizmo::scene_camera_window(
        CenterSplit::Horizontal {
            t: CenterSplit::T_DEFAULT,
        },
        WindowSize::new(JANELA.0, JANELA.1),
    )
}

/// O canvas visto como a `Area` do enquadramento (px da janela da cena).
pub(super) fn area() -> Area {
    let [x0, y0, x1, y1] = CANVAS_VISTO_PX;
    Area {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

/// A margem do enquadramento de arranque — a do produto, a do *Enquadrar tudo*.
pub(super) fn margem() -> f32 {
    ph2d_editor_core::screens::hero::float_bar::frame_margin()
}

/// ⭐ **A vista com que a cena abre** — a lei do produto (`motion_demo_enquadra::vista`) sobre a
/// figura da tabela e as fichas `legendas` (já no mundo). Campo ou cena fora da tabela: a omissão.
pub(super) fn vista_da_cena(
    level: &str,
    legendas: &[Caption],
    ts: &mut ph2d_text::TextSystem,
) -> View {
    use crate::motion_demo_enquadra as e;
    let fichas = e::fichas(legendas, ts);
    e::vista_de(
        e::extensao(Some(level)),
        &fichas,
        area(),
        janela_da_cena(),
        margem(),
    )
}

/// Px de ecrã por metro sob a vista `v`, e a caixa de mundo do canvas visto.
pub(super) fn camara(v: View) -> (f32, Caixa) {
    let j = janela_da_cena();
    #[expect(clippy::cast_precision_loss, reason = "px de janela")]
    let (w, h) = (j.width as f32, j.height as f32);
    let k = h / v.height_world;
    let [x0, y0, x1, y1] = CANVAS_VISTO_PX;
    let mundo = |px: f32, py: f32| {
        [
            v.center[0] + (px - 0.5 * w) / k,
            v.center[1] + (0.5 * h - py) / k,
        ]
    };
    let ([a, b], [c, d]) = (mundo(x0, y1), mundo(x1, y0));
    (k, [a, b, c, d])
}

/// A caixa de mundo da ficha `c` (centro = `c.world`).
fn caixa_da_ficha(ts: &mut ph2d_text::TextSystem, c: &Caption, k: f32) -> Caixa {
    let hw = 0.5 * chip_width(ts, &c.text, 0.0) / k;
    let hh = 0.5 * CHIP_H_PX / k;
    [
        c.world[0] - hw,
        c.world[1] - hh,
        c.world[0] + hw,
        c.world[1] + hh,
    ]
}

fn caixa_dos_cantos(cantos: impl IntoIterator<Item = [f32; 2]>) -> Caixa {
    let mut b = [
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ];
    for p in cantos {
        b = [
            b[0].min(p[0]),
            b[1].min(p[1]),
            b[2].max(p[0]),
            b[3].max(p[1]),
        ];
    }
    b
}

/// O que o canvas pinta neste quadro, uma caixa por peça.
pub(super) fn figura(m: &MotionState, k: f32, fora: &mut Vec<Caixa>) {
    fora.clear();
    gizmo(m, k, fora);
    for i in &m.pump.instances {
        if i.tint[3] <= 0.0 {
            continue;
        }
        let [b0, b1, b2, b3] = i.basis;
        fora.push(caixa_dos_cantos(
            [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]].map(|q| {
                let l = [
                    i.anchor[0] + q[0] * i.size[0],
                    i.anchor[1] + q[1] * i.size[1],
                ];
                [
                    i.world_pos[0] + b0 * l[0] + b2 * l[1],
                    i.world_pos[1] + b1 * l[0] + b3 * l[1],
                ]
            }),
        ));
    }
    for v in &m.pump.vector_instances {
        if v.tint[3] <= 0.0 {
            continue;
        }
        let local = if v.geometry_id > 0 {
            let Some(r) = m
                .shape_store
                .get(v.geometry_id)
                .and_then(|p| ph2d_vec_render::path_visual_bounds(p, Affine::IDENTITY))
            else {
                continue;
            };
            [r.x0, r.y0, r.x1, r.y1]
        } else {
            [-0.5, -0.5, 0.5, 0.5]
        };
        let pose = crate::motion_shape_gen::instance_pose(v, Affine::IDENTITY);
        #[expect(clippy::cast_possible_truncation, reason = "metros de cena")]
        fora.push(caixa_dos_cantos(
            [
                (local[0], local[1]),
                (local[2], local[1]),
                (local[2], local[3]),
                (local[0], local[3]),
            ]
            .map(|q| {
                let p = pose * ph2d_vector::Point::new(q.0, q.1);
                [p.x as f32, p.y as f32]
            }),
        ));
    }
}

/// ⭐ **O GIZMO DE POSIÇÕES** (`ponto_gizmo`): sob a lei do dono, um sink sem aparência pinta uma
/// cruz por posição com a ferramenta Motion na mão — é a figura das cenas de conferência. Braço
/// `0,5 · size` m (a agulha, `1 · size`, quando há `rot`) mais meio traço com o casing, em px. Todas
/// as posições entram (o gizmo desenha uma amostra delas).
fn gizmo(m: &MotionState, k: f32, fora: &mut Vec<Caixa>) {
    use ph2d_nodegraph::attr::Column;
    if !m.pump.a_lei() {
        return;
    }
    #[expect(clippy::cast_possible_truncation, reason = "px de chrome")]
    let traco = ((crate::warp_overlay::OUTLINE_PX * 0.5 + crate::warp_overlay::CASE_PX) as f32) / k;
    for &sink in &m.sinks {
        let Some(s) = m
            .pump
            .cook
            .peek(sink)
            .and_then(|v| v.first())
            .map(|v| v.as_stream())
        else {
            continue;
        };
        if ph2d_eval_motion::tem_aparencia(s) {
            continue;
        }
        let Some(Column::Vec2(p)) = s.get("P") else {
            continue;
        };
        let braco = if s.get("rot").is_some() { 1.0 } else { 0.5 };
        for (i, q) in p.iter().enumerate() {
            let e = match s.get("size") {
                Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
                Some(Column::Vec2(v)) => v.get(i).map_or(1.0, |e| (e[0] + e[1]) * 0.5),
                _ => 1.0,
            };
            let r = braco * e.abs() + traco;
            fora.push([q[0] - r, q[1] - r, q[0] + r, q[1] + r]);
        }
    }
}

/// **O cenário:** uma peça que atravessa o canvas visto de lado a lado ou de cima a baixo (o fundo de
/// 14 m das `=140`/`=142`).
fn e_cenario(p: Caixa, vista: Caixa) -> bool {
    (p[1] <= vista[1] && p[3] >= vista[3]) || (p[0] <= vista[0] && p[2] >= vista[2])
}

/// A sobreposição de duas caixas, `[largura, altura]` — `None` quando não se tocam.
fn sobrepoe(a: Caixa, b: Caixa) -> Option<[f32; 2]> {
    let w = a[2].min(b[2]) - a[0].max(b[0]);
    let h = a[3].min(b[3]) - a[1].max(b[1]);
    (w > 0.0 && h > 0.0).then_some([w, h])
}

/// Quantas fatias de 1 px cabem em `[a, b]`.
fn fatias(k: f32, [a, b]: [f32; 2]) -> usize {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "px"
    )]
    let n = ((b - a) * k).ceil() as usize;
    n
}

/// Marca as fatias que o intervalo `p` cobre dentro do intervalo `eixo`.
fn marca(ocupado: &mut [bool], p: [f32; 2], k: f32, eixo: [f32; 2]) {
    let n = i64::try_from(ocupado.len()).unwrap_or(i64::MAX);
    #[expect(clippy::cast_possible_truncation, reason = "px")]
    let a = (((p[0] - eixo[0]) * k).floor() as i64).clamp(0, n);
    #[expect(clippy::cast_possible_truncation, reason = "px")]
    let b = (((p[1] - eixo[0]) * k).ceil() as i64).clamp(0, n);
    #[expect(clippy::cast_sign_loss, reason = "já presos a 0..n")]
    for f in &mut ocupado[a as usize..b as usize] {
        *f = true;
    }
}

/// ⭐ **A CURA DERIVADA**: o centro mais perto de `c0` em que uma ficha de `tamanho_px` cabe, com
/// [`AR_PX`] de ar de cada lado, sem tocar nenhuma fatia ocupada das faixas dadas, dentro do intervalo
/// `eixo` do canvas visto e dentro de `lado`. `None` = não há lugar.
fn lugar_livre(
    faixas: &[&[bool]],
    c0: f32,
    tamanho_px: f32,
    k: f32,
    eixo: [f32; 2],
    lado: [f32; 2],
) -> Option<f32> {
    let n = faixas.first()?.len();
    let livre = |f: usize| faixas.iter().all(|c| !c[f]);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "px"
    )]
    let largo = (tamanho_px + 2.0 * AR_PX).ceil() as usize;
    #[expect(clippy::cast_precision_loss, reason = "px")]
    (0..=n.saturating_sub(largo))
        .filter(|&a| (a..a + largo).all(livre))
        .map(|a| eixo[0] + (a as f32 + 0.5 * largo as f32) / k)
        .filter(|c| (lado[0]..=lado[1]).contains(c))
        .min_by(|a, b| (a - c0).abs().total_cmp(&(b - c0).abs()))
}

/// O veredito de uma ficha.
struct Medida {
    texto: String,
    world: [f32; 2],
    caixa: Caixa,
    /// A pior sobreposição VISÍVEL com uma peça (a de maior área) e o tique em que se deu.
    tapa: Option<([f32; 2], u64)>,
    /// A ficha não cabe inteira no canvas visto.
    fora_da_vista: bool,
    /// A COLUNA da ficha (as peças cuja faixa de x cruza a dela) em fatias de 1 px sobre a altura do
    /// canvas visto: `true` = alguma peça passou ali em algum quadro.
    ocupado_y: Vec<bool>,
    /// A FAIXA da ficha (as peças cuja faixa de y cruza a dela) em fatias de 1 px sobre a largura.
    ocupado_x: Vec<bool>,
}

/// O que se mediu numa cena: as fichas, e quantas peças se leram ao todo.
struct Cena {
    medidas: Vec<Medida>,
    pecas_lidas: usize,
    /// Px/m e o canvas visto (mundo) da vista com que a cena ABRE.
    k: f32,
    vista: Caixa,
}

/// Mede as fichas do nível `level`, com `poe` a mexer nelas antes (os controlos) — `None` quando ele
/// não publica legenda.
fn mede(level: &str, ts: &mut ph2d_text::TextSystem, poe: impl Fn(&mut [Caption])) -> Option<Cena> {
    let mut m = MotionState::new();
    let (sinks, mut caps) = super::monta(level, &mut m.doc, &m.registry);
    if caps.is_empty() {
        return None;
    }
    poe(&mut caps);
    m.sinks = sinks;
    let no_mundo = super::no_mundo_de(&caps, &m);
    let (k, vista) = camara(vista_da_cena(level, &no_mundo, ts));
    let mut medidas: Vec<Medida> = no_mundo
        .iter()
        .map(|c| {
            let caixa = caixa_da_ficha(ts, c, k);
            Medida {
                texto: c.text.clone(),
                world: c.world,
                caixa,
                tapa: None,
                fora_da_vista: caixa[0] < vista[0]
                    || caixa[1] < vista[1]
                    || caixa[2] > vista[2]
                    || caixa[3] > vista[3],
                ocupado_y: vec![false; fatias(k, [vista[1], vista[3]])],
                ocupado_x: vec![false; fatias(k, [vista[0], vista[2]])],
            }
        })
        .collect();
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "segundos inteiros"
    )]
    let tiques = (2.0 * crate::motion_state::ritmo_demo::CICLO * 60.0) as u64;
    let mut pecas = Vec::new();
    let mut pecas_lidas = 0usize;
    for t in (0..=tiques).step_by(PASSO) {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
        figura(&m, k, &mut pecas);
        pecas_lidas += pecas.len();
        for p in pecas.iter().filter(|p| !e_cenario(**p, vista)) {
            for md in &mut medidas {
                if p[0] < md.caixa[2] && p[2] > md.caixa[0] {
                    marca(&mut md.ocupado_y, [p[1], p[3]], k, [vista[1], vista[3]]);
                }
                if p[1] < md.caixa[3] && p[3] > md.caixa[1] {
                    marca(&mut md.ocupado_x, [p[0], p[2]], k, [vista[0], vista[2]]);
                }
                let Some(vis) = sobrepoe(md.caixa, vista).map(|_| {
                    [
                        md.caixa[0].max(vista[0]),
                        md.caixa[1].max(vista[1]),
                        md.caixa[2].min(vista[2]),
                        md.caixa[3].min(vista[3]),
                    ]
                }) else {
                    continue;
                };
                if let Some(s) = sobrepoe(vis, *p)
                    && md.tapa.is_none_or(|(o, _)| s[0] * s[1] > o[0] * o[1])
                {
                    md.tapa = Some((s, t));
                }
            }
        }
    }
    Some(Cena {
        medidas,
        pecas_lidas,
        k,
        vista,
    })
}

/// As duas curas de `md`: `(y` da FILEIRA dela — as fichas à mesma altura mudam juntas, a regra das
/// metades —, `x` por FORA da metade dela, à mesma altura`)`.
fn curas(md: &Medida, todas: &[Medida], k: f32, vista: Caixa) -> (Option<f32>, Option<f32>) {
    let fileira: Vec<&[bool]> = todas
        .iter()
        .filter(|o| o.world[1] == md.world[1])
        .map(|o| o.ocupado_y.as_slice())
        .collect();
    let y = lugar_livre(
        &fileira,
        md.world[1],
        CHIP_H_PX,
        k,
        [vista[1], vista[3]],
        [f32::MIN, f32::MAX],
    );
    let lado = match md.world[0] {
        x if x < 0.0 => [f32::MIN, x],
        x if x > 0.0 => [x, f32::MAX],
        _ => [f32::MIN, f32::MAX],
    };
    let x = lugar_livre(
        &[md.ocupado_x.as_slice()],
        md.world[0],
        (md.caixa[2] - md.caixa[0]) * k,
        k,
        [vista[0], vista[2]],
        lado,
    );
    (y, x)
}

/// ⭐⭐⭐ **O GATE**: nenhuma ficha de nenhuma cena tapa o que o canvas visto mostra, salvo as
/// [`SEM_LUGAR`] — re-medidas.
#[test]
fn nenhuma_ficha_tapa_a_figura_que_explica() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let (mut cenas, mut fichas) = (0usize, 0usize);
    let mut maus = Vec::new();
    let mut isentas_vistas = Vec::new();
    for n in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let level = n.to_string();
        let Some(cena) = mede(&level, &mut ts, |_| ()) else {
            continue;
        };
        cenas += 1;
        fichas += cena.medidas.len();
        if cena.pecas_lidas == 0 {
            maus.push(format!("={level}: a régua não leu peça nenhuma da figura"));
        }
        for md in &cena.medidas {
            let isenta = SEM_LUGAR
                .iter()
                .find(|(l, t, _)| *l == level && *t == md.texto);
            match (md.tapa, isenta) {
                (Some((s, t)), None) => maus.push(format!(
                    "={level} {:?} em ({:.2}, {:.2}) tapa {:.2}x{:.2} m da figura no tique {t}",
                    md.texto, md.world[0], md.world[1], s[0], s[1]
                )),
                (Some(_), Some(_)) => {
                    isentas_vistas.push((level.clone(), md.texto.clone()));
                    let (y, x) = curas(md, &cena.medidas, cena.k, cena.vista);
                    if y.is_some() || x.is_some() {
                        maus.push(format!(
                            "={level} {:?} está em SEM_LUGAR, mas já há lugar: y = {y:?} · x = {x:?}",
                            md.texto
                        ));
                    }
                }
                (None, Some(_)) => maus.push(format!(
                    "={level} {:?} está em SEM_LUGAR e já não tapa — retire a isenção",
                    md.texto
                )),
                (None, None) => {}
            }
        }
    }
    for (l, t, _) in SEM_LUGAR {
        if !isentas_vistas.iter().any(|(a, b)| a == l && b == t)
            && !maus.iter().any(|m| m.contains(t))
        {
            maus.push(format!(
                "={l} {t:?} está em SEM_LUGAR e a cena já não a publica"
            ));
        }
    }
    // ⛔ PISOS (15/10: 42 cenas com legenda, 192 fichas): um leitor que deixe de montar lê zero e passa.
    assert!(
        cenas >= 42 && fichas >= 192,
        "a régua leu {cenas} cenas com legenda e {fichas} fichas — ou as cenas mudaram, ou a montagem partiu-se"
    );
    assert!(
        maus.is_empty(),
        "{} ficha(s) tapam a figura que explicam (régua: motion_demo_legend_tapa_tests; a sonda \
         `a_tabela_das_fichas_que_tapam` diz para onde ir):\n  {}",
        maus.len(),
        maus.join("\n  ")
    );
}

/// O `y` de uma ficha da `=142` medida com ela posta em `world`.
fn tapa_a_142_em(world: [f32; 2]) -> bool {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let cena = mede("142", &mut ts, |caps| caps[0].world = world).expect("a =142 publica legenda");
    assert_eq!(cena.medidas.len(), 1, "uma ficha na =142");
    cena.medidas[0].tapa.is_some()
}

/// ⭐ **CONTROLO da régua contra o achado da foto** (15/10): a ficha da `=142` a `-3,2` (a herdada da
/// `=140`) tapava a ponta de baixo do losango — tem de REPROVAR; a `-4,2` (a cura) passa.
#[test]
fn a_regua_reprova_a_142_a_menos_3_2_e_aprova_a_menos_4_2() {
    assert!(
        tapa_a_142_em([0.0, -3.2]),
        "a -3,2 a ficha tapava a galáxia (foto de 15/10)"
    );
    assert!(
        !tapa_a_142_em([0.0, -4.2]),
        "a -4,2 ela pousa abaixo da galáxia"
    );
}

/// ⭐ **CONTROLO POSITIVO feito à mão**: uma ficha posta no meio da galáxia da `=142` tapa.
#[test]
fn a_regua_reprova_uma_ficha_posta_sobre_a_figura() {
    assert!(
        tapa_a_142_em([0.0, 0.0]),
        "no centro da galáxia a ficha tapa-a"
    );
}

/// ⭐ **A SONDA** — a tabela cena → ficha → tapa/livre, com a sobreposição e as duas curas medidas.
///
/// `cargo test -p ph2d-app-motion --lib a_tabela_das_fichas_que_tapam -- --ignored --nocapture`
/// (`PH2D_LAYOUT_LEVEL=<n>` mede uma cena só).
#[test]
#[ignore = "sonda: imprime a tabela, não afirma"]
fn a_tabela_das_fichas_que_tapam() {
    let only = std::env::var("PH2D_LAYOUT_LEVEL").ok();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for n in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let level = n.to_string();
        if only.as_deref().is_some_and(|w| w != level) {
            continue;
        }
        let Some(cena) = mede(&level, &mut ts, |_| ()) else {
            continue;
        };
        let (k, v) = (cena.k, cena.vista);
        println!(
            "={level}: {k:.2} px/m · canvas visto x [{:.2}..{:.2}] y [{:.2}..{:.2}] m",
            v[0], v[2], v[1], v[3]
        );
        for md in &cena.medidas {
            let estado = match md.tapa {
                Some((s, t)) => {
                    let (y, x) = curas(md, &cena.medidas, k, v);
                    format!(
                        "TAPA {:.2}x{:.2} m (t={t}) CURA y={} | x={}",
                        s[0],
                        s[1],
                        y.map_or("-".into(), |y| format!("{y:.2}")),
                        x.map_or("-".into(), |x| format!("{x:.2}"))
                    )
                }
                None => "livre".into(),
            };
            println!(
                "={level:<4} ({:>6.2},{:>6.2}) {estado}{} · {:?} · {} peças lidas",
                md.world[0],
                md.world[1],
                if md.fora_da_vista {
                    " FORA-DA-VISTA"
                } else {
                    ""
                },
                md.texto,
                cena.pecas_lidas
            );
        }
    }
}
