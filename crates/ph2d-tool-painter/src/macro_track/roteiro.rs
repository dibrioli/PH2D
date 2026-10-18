//! **A macro do «Splash»** (`PH2D_MACRO_SMOKE=1`, `docs/Macros/01_plano_onda1.md` §6): quatro
//! pinceladas de Wet Paint numa tela branca, gravadas por script pelas portas de fora do tool — a cor e
//! o tamanho entre traços, o ponteiro, o tique — e drenando a pré-visualização a cada quadro como o
//! editor. O meio e o relógio fixo da água escolhem-se ANTES de armar (o estado inicial guarda-os).

use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};
use ph2d_macro::{MacroRecorder, MacroTime, RefRect};

pub(crate) const DT_MS: f32 = 1000.0 / 60.0;
/// Quadros de cada pincelada.
pub(crate) const TRACO: u32 = 24;
/// Quadros entre duas pinceladas.
pub(crate) const ENTRE: u32 = 12;
/// Quadros depois da última (a água a assentar).
pub(crate) const CAUDA: u32 = 120;

#[derive(Clone, Copy)]
pub(crate) struct Pincelada {
    pub(crate) cor: [u8; 3],
    pub(crate) tamanho_px: f32,
    /// Início e fim, em fracções da tela; `curva` empurra o meio na perpendicular.
    pub(crate) de: [f32; 2],
    pub(crate) ate: [f32; 2],
    pub(crate) curva: f32,
    /// A pressão no início e no fim (rampa linear); a do rato é `[1, 1]`.
    pub(crate) pressao: [f32; 2],
}

// LITERAL-COLOR-OK: as cores de uma pincelada GRAVADA (o artista escolheu-as), não da UI.
pub(crate) const SPLASH: [Pincelada; 4] = [
    Pincelada {
        cor: [30, 90, 200],
        tamanho_px: 70.0,
        de: [0.18, 0.30],
        ate: [0.80, 0.42],
        curva: 0.06,
        pressao: [1.0, 1.0],
    },
    Pincelada {
        cor: [200, 40, 60],
        tamanho_px: 60.0,
        de: [0.22, 0.62],
        ate: [0.78, 0.52],
        curva: -0.08,
        pressao: [1.0, 1.0],
    },
    Pincelada {
        cor: [220, 160, 30],
        tamanho_px: 55.0,
        de: [0.50, 0.15],
        ate: [0.55, 0.85],
        curva: 0.05,
        pressao: [1.0, 1.0],
    },
    Pincelada {
        cor: [30, 90, 200],
        tamanho_px: 45.0,
        de: [0.30, 0.82],
        ate: [0.74, 0.70],
        curva: 0.04,
        pressao: [1.0, 1.0],
    },
];

pub(crate) fn ponto(p: &Pincelada, f: f32, lado: f32) -> [f32; 2] {
    let (dx, dy) = (p.ate[0] - p.de[0], p.ate[1] - p.de[1]);
    let arco = p.curva * (std::f32::consts::PI * f).sin();
    [
        lado * (p.de[0] + dx * f - dy * arco),
        lado * (p.de[1] + dy * f + dx * arco),
    ]
}

/// **O «Splash» gravado** numa tela branca `lado`², com a trilha a apontar a sprite `alvo` (`StableId`;
/// `0` = a entidade do próprio `Macro`): os bytes do `Macro.doc` e a duração em segundos. `None` = o
/// tool recusou armar.
#[must_use]
pub fn grava_o_splash(lado: u32, alvo: u64) -> Option<(Vec<u8>, f64)> {
    grava_pinceladas(lado, alvo, &SPLASH, &[0; 4])
}

/// **`n` pinceladas (as do Splash, em ciclo) numa sprite branca aberta pela porta do PRODUTO**
/// (`abre_a_sprite`: branca = papel branco sem tinta, a composição completa — o que o artista grava no
/// editor).
#[must_use]
pub fn grava_no_papel(lado: u32, alvo: u64, n: usize) -> Option<(Vec<u8>, f64)> {
    let ps: Vec<Pincelada> = SPLASH.iter().copied().cycle().take(n).collect();
    grava(lado, alvo, &ps, &[], true, &[PaintMedia::WetPaint])
}

/// Como [`grava_no_papel`], com `entrada` quadros parados antes da 1.ª pincelada — o Gravar antes do 1.º
/// toque, como o artista grava: o instante 0 é a tela de partida (em [`grava_no_papel`] o instante 0 já
/// traz o 1.º toque do pincel).
#[must_use]
pub fn grava_no_papel_com_entrada(
    lado: u32,
    alvo: u64,
    n: usize,
    entrada: u32,
) -> Option<(Vec<u8>, f64)> {
    let ps: Vec<Pincelada> = SPLASH.iter().copied().cycle().take(n).collect();
    grava(lado, alvo, &ps, &[entrada], true, &[PaintMedia::WetPaint])
}

/// [`grava_no_papel`] no Painter do editor LIGADO à sprite `entidade` (bits) — como o artista grava:
/// escolhe a sprite, depois grava — e esse tool, parado no Parar.
#[must_use]
pub fn grava_no_papel_ligado(
    lado: u32,
    entidade: u64,
    n: usize,
) -> Option<(PainterTool, Vec<u8>, f64)> {
    let ps: Vec<Pincelada> = SPLASH.iter().copied().cycle().take(n).collect();
    grava_com(
        lado,
        0,
        &ps,
        &[],
        Tela::Ligada(entidade),
        &[PaintMedia::WetPaint],
        Relogio::Fixo,
    )
}

/// [`grava_no_papel_ligado`] com a água no WORKER de verdade (o ritmo da parede), como o editor grava — e
/// a água continua nele depois do Parar.
#[must_use]
pub fn grava_no_papel_ligado_com_o_worker(
    lado: u32,
    entidade: u64,
    n: usize,
) -> Option<(PainterTool, Vec<u8>, f64)> {
    let ps: Vec<Pincelada> = SPLASH.iter().copied().cycle().take(n).collect();
    grava_com(
        lado,
        0,
        &ps,
        &[],
        Tela::Ligada(entidade),
        &[PaintMedia::WetPaint],
        Relogio::Worker,
    )
}

// LITERAL-COLOR-OK: as pinceladas GRAVADAS da fixtura (o preto do artista), não da UI.
const CRUZADAS: [Pincelada; 2] = [
    Pincelada {
        cor: [0, 0, 0],
        tamanho_px: 10.0,
        de: [0.08, 0.35],
        ate: [0.92, 0.35],
        curva: 0.0,
        pressao: [1.0, 1.0],
    },
    Pincelada {
        cor: [0, 0, 0],
        tamanho_px: 10.0,
        de: [0.08, 0.65],
        ate: [0.92, 0.65],
        curva: 0.0,
        pressao: [1.0, 1.0],
    },
];

/// **Duas horizontais pretas de Digital** (`v = 0,35` e `0,65`) no Painter LIGADO à sprite `entidade`,
/// com 12 quadros parados antes da 1.ª (a tela branca até `0,2 s`; a 2.ª nasce em `≈ 0,82 s`) — a
/// fixtura dos cruzamentos (W10, smoke do dono 19/10) — e o tool, parado no Parar.
#[must_use]
pub fn grava_duas_horizontais_ligado(
    lado: u32,
    entidade: u64,
) -> Option<(PainterTool, Vec<u8>, f64)> {
    grava_duas_horizontais_ligado_em(lado, entidade, PaintMedia::Digital)
}

/// [`grava_duas_horizontais_ligado`] no meio `meio` (W12: os cruzamentos em Wet Paint, Watercolor e
/// Impasto), com o relógio fixo da água.
#[must_use]
pub fn grava_duas_horizontais_ligado_em(
    lado: u32,
    entidade: u64,
    meio: PaintMedia,
) -> Option<(PainterTool, Vec<u8>, f64)> {
    grava_com(
        lado,
        0,
        &CRUZADAS,
        &[12],
        Tela::Ligada(entidade),
        &[meio],
        Relogio::Fixo,
    )
}

/// Um ColorDrop do C&F gravado (W13): a cor solta em `onde` (fracções da tela) e confirmada no Done do
/// modal — com o limiar que o tool tem, ou `limiar` afinado no slider do modal antes do Done.
#[derive(Clone, Copy)]
pub(crate) struct Gota {
    pub(crate) cor: [u8; 3],
    pub(crate) onde: [f32; 2],
    pub(crate) limiar: Option<f32>,
}

/// A caixa (W13): quatro traços pretos que fecham um quadrado, depois o azul solto lá dentro pelo C&F,
/// depois um traço amarelo por cima do azul.
// LITERAL-COLOR-OK: as cores GRAVADAS da fixtura (o artista escolheu-as), não da UI.
const CAIXA: [Pincelada; 5] = {
    const fn lado(de: [f32; 2], ate: [f32; 2]) -> Pincelada {
        Pincelada {
            cor: [30, 30, 30],
            tamanho_px: 16.0,
            de,
            ate,
            curva: 0.0,
            pressao: [1.0, 1.0],
        }
    }
    [
        lado([0.22, 0.25], [0.78, 0.25]),
        lado([0.75, 0.22], [0.75, 0.78]),
        lado([0.78, 0.75], [0.22, 0.75]),
        lado([0.25, 0.78], [0.25, 0.22]),
        Pincelada {
            cor: [230, 190, 40],
            tamanho_px: 24.0,
            de: [0.35, 0.62],
            ate: [0.65, 0.38],
            curva: 0.05,
            pressao: [1.0, 1.0],
        },
    ]
};
// LITERAL-COLOR-OK: a cor GRAVADA do Fill da caixa.
const AZUL_DA_CAIXA: Gota = Gota {
    cor: [40, 110, 210],
    onde: [0.45, 0.55],
    limiar: None,
};

/// Um traço preto, o Fill que o cobre (o limiar no máximo: a tela inteira) e o MESMO traço outra vez,
/// em Digital — a 2.ª passagem só tem critério contra a cor do Fill (W13, a cobertura que o Fill apaga).
#[cfg(test)]
pub(crate) fn grava_o_traco_sob_o_fill_e_por_cima(lado: u32) -> Option<(Vec<u8>, f64)> {
    // LITERAL-COLOR-OK: a cor GRAVADA do Fill da fixtura.
    let gota = Gota {
        cor: [200, 60, 40],
        onde: [0.5, 0.8],
        limiar: Some(1.0),
    };
    let traco = CRUZADAS[0];
    grava_com_gotas(
        lado,
        0,
        &[traco, traco],
        &[],
        &[(1, gota)],
        Tela::Papel,
        &[PaintMedia::Digital],
        Relogio::Fixo,
    )
    .map(|(_, b, s)| (b, s))
}

/// **A caixa com um Fill** (W13, a cena do tutorial `PH2D_MACRO_SMOKE=4` com
/// `PH2D_MACRO_CAMINHO=fill`): quatro traços que fecham um quadrado, o azul do C&F solto dentro dele, um
/// traço por cima — numa sprite branca aberta pela porta do produto, no meio `meio`.
#[must_use]
pub fn grava_a_caixa_com_um_fill(lado: u32, alvo: u64, meio: PaintMedia) -> Option<(Vec<u8>, f64)> {
    grava_com_gotas(
        lado,
        alvo,
        &CAIXA,
        &[],
        &[(4, AZUL_DA_CAIXA)],
        Tela::Papel,
        &[meio],
        Relogio::Fixo,
    )
    .map(|(_, b, s)| (b, s))
}

/// As pinceladas `ps` gravadas como o Splash, com `antes[k]` quadros parados a mais antes da `k`.
pub(crate) fn grava_pinceladas(
    lado: u32,
    alvo: u64,
    ps: &[Pincelada],
    antes: &[u32],
) -> Option<(Vec<u8>, f64)> {
    grava(lado, alvo, ps, antes, false, &[PaintMedia::WetPaint])
}

/// As pinceladas `ps` gravadas como o Splash, a `k` no meio `meios[k]` (o último repete-se): o 1.º
/// escolhe-se antes de armar, cada troca seguinte grava-se pelo seletor do painel antes da pincelada.
#[cfg(test)]
pub(crate) fn grava_nos_meios(
    lado: u32,
    ps: &[Pincelada],
    meios: &[PaintMedia],
) -> Option<(Vec<u8>, f64)> {
    grava(lado, 0, ps, &[], false, meios)
}

/// **Uma pincelada do Splash por meio de `meios`** (em ciclo), numa sprite branca aberta pela porta do
/// produto como [`grava_no_papel`]: a troca de meio a meio da gravação é um passo do seletor do painel.
#[must_use]
pub fn grava_no_papel_nos_meios(
    lado: u32,
    alvo: u64,
    meios: &[PaintMedia],
) -> Option<(Vec<u8>, f64)> {
    let ps: Vec<Pincelada> = SPLASH.iter().copied().cycle().take(meios.len()).collect();
    grava(lado, alvo, &ps, &[], true, meios)
}

/// As pinceladas `ps` gravadas como o Splash, com a água no worker ao ritmo `ritmo` e o quadro `dts[q]`
/// (em ciclo) — o relógio irregular de uma máquina, e não o fixo dos outros roteiros (W8, a sonda da
/// corrida partilhada).
#[cfg(test)]
pub(crate) fn grava_pinceladas_como_a_maquina(
    lado: u32,
    ps: &[Pincelada],
    ritmo: &crate::macro_track::RitmoDaAgua,
    dts: &[f32],
) -> Option<(Vec<u8>, f64)> {
    grava_como_a_maquina_e_o_tool(lado, ps, ritmo, dts).map(|(_, b, s)| (b, s))
}

/// As pinceladas `ps` gravadas nos `meios` (como [`grava_nos_meios`]) e o tool que gravou, parado no
/// Parar (o editor depois dele).
#[cfg(test)]
pub(crate) fn grava_nos_meios_e_o_tool(
    lado: u32,
    ps: &[Pincelada],
    meios: &[PaintMedia],
) -> Option<(PainterTool, Vec<u8>)> {
    grava_com(lado, 0, ps, &[], Tela::Fonte, meios, Relogio::Fixo).map(|(t, b, _)| (t, b))
}

/// [`grava_pinceladas_como_a_maquina`] e o tool que gravou, parado no Parar (o editor depois dele).
#[cfg(test)]
pub(crate) fn grava_como_a_maquina_e_o_tool(
    lado: u32,
    ps: &[Pincelada],
    ritmo: &crate::macro_track::RitmoDaAgua,
    dts: &[f32],
) -> Option<(PainterTool, Vec<u8>, f64)> {
    grava_com(
        lado,
        0,
        ps,
        &[],
        Tela::Fonte,
        &[PaintMedia::WetPaint],
        Relogio::Maquina((ritmo, dts)),
    )
}

/// O ColorDrop `g` como a shell o entrega: a cor no seletor, o C&F a entrar em Fill antes do toque, o
/// soltar na tela e o Done do modal — um quadro por passo.
fn solta(
    t: &mut PainterTool,
    g: &Gota,
    l: f32,
    quadro: &mut dyn FnMut(&mut PainterTool) -> Option<()>,
) -> Option<()> {
    t.set_brush_color_srgb8(g.cor);
    quadro(t)?;
    let volta = t.active_paint_mode_id();
    t.begin_colordrop_fill(volta);
    let pos = [g.onde[0] * l, g.onde[1] * l];
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        t.on_canvas_pointer(CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        quadro(t)?;
    }
    if let Some(v) = g.limiar {
        t.handle_panel_event(PanelEvent::SetValue(
            ph2d_editor_core::ids::PAINTER_FILL_MODAL_SLIDER,
            f64::from(v),
        ));
        quadro(t)?;
    }
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_FILL_MODAL_DONE,
    ));
    quadro(t)
}

/// [`grava`], e o tool que pintou AO VIVO (a tela dele é a régua de quem toca a gravação).
pub(crate) fn grava_vivo(
    lado: u32,
    alvo: u64,
    ps: &[Pincelada],
    antes: &[u32],
    papel: bool,
    meios: &[PaintMedia],
) -> Option<(Vec<u8>, f64, PainterTool)> {
    let tela = if papel { Tela::Papel } else { Tela::Fonte };
    grava_com(lado, alvo, ps, antes, tela, meios, Relogio::Fixo).map(|(t, b, s)| (b, s, t))
}

fn grava(
    lado: u32,
    alvo: u64,
    ps: &[Pincelada],
    antes: &[u32],
    papel: bool,
    meios: &[PaintMedia],
) -> Option<(Vec<u8>, f64)> {
    grava_vivo(lado, alvo, ps, antes, papel, meios).map(|(doc, s, _)| (doc, s))
}

/// A tela em que se grava: a fonte do tool, a sprite aberta no papel, ou o Painter LIGADO à sprite
/// (bits) — como o artista grava.
#[derive(Clone, Copy)]
enum Tela {
    Fonte,
    Papel,
    Ligada(u64),
}

/// A máquina de uma gravação: o ritmo da água no worker e a duração de cada quadro, em ms.
type Maquina<'a> = (&'a crate::macro_track::RitmoDaAgua, &'a [f32]);

/// O relógio da água a gravar: o fixo (a régua), o ritmo escrito de uma máquina, ou o worker real.
#[derive(Clone, Copy)]
enum Relogio<'a> {
    Fixo,
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "só os gates gravam ao ritmo de uma máquina")
    )]
    Maquina(Maquina<'a>),
    Worker,
}

fn grava_com(
    lado: u32,
    alvo: u64,
    ps: &[Pincelada],
    antes: &[u32],
    tela: Tela,
    meios: &[PaintMedia],
    relogio: Relogio<'_>,
) -> Option<(PainterTool, Vec<u8>, f64)> {
    grava_com_gotas(lado, alvo, ps, antes, &[], tela, meios, relogio)
}

/// [`grava_com`] com os ColorDrops `gotas`: `(k, g)` solta `g` antes da pincelada `k`.
#[expect(
    clippy::too_many_arguments,
    reason = "a gravação inteira de uma fixtura, cada parte com o seu nome"
)]
fn grava_com_gotas(
    lado: u32,
    alvo: u64,
    ps: &[Pincelada],
    antes: &[u32],
    gotas: &[(usize, Gota)],
    tela: Tela,
    meios: &[PaintMedia],
    relogio: Relogio<'_>,
) -> Option<(PainterTool, Vec<u8>, f64)> {
    let meio = |k: usize| meios.get(k).or(meios.last()).copied();
    let mut t = PainterTool::default();
    let n = usize::try_from(lado).ok()?;
    let branca = vec![255u8; n * n * 4];
    match tela {
        Tela::Ligada(e) => t.bind_document(e, branca, lado, lado),
        Tela::Papel => t.abre_a_sprite(branca, lado, lado),
        Tela::Fonte => t.set_source(branca, lado, lado),
    }
    t.set_paint_media(meio(0)?);
    match relogio {
        Relogio::Maquina((r, _)) => t.set_wet_ritmo_de_teste(Some(r.clone())),
        Relogio::Fixo => t.set_wet_relogio_fixo(true),
        Relogio::Worker => {}
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "o lado de uma tela cabe num f32"
    )]
    let l = lado as f32;
    let mut rec = MacroRecorder::new(RefRect { w: l, h: l }, 0);
    let trilha = t.macro_abre(&mut rec, alvo).ok()?;
    let (mut q, mut s): (u32, f64) = (0, 0.0);
    let mut quadro = |t: &mut PainterTool, rec: &mut MacroRecorder| -> Option<()> {
        drop(t.take_preview_arc());
        let agora = MacroTime::from_secs_f64(s).0;
        // O relógio fixo grava `q / 60` exactos (os bytes das gravações dos gates não mudam).
        match relogio {
            Relogio::Maquina((_, d)) => {
                let dt = d[q as usize % d.len()];
                t.on_tick(dt);
                s += f64::from(dt) / 1000.0;
            }
            Relogio::Fixo | Relogio::Worker => {
                t.on_tick(DT_MS);
                s = f64::from(q + 1) / 60.0;
            }
        }
        for c in t.macro_passos() {
            rec.push(trilha, agora, c).ok()?;
        }
        q += 1;
        Some(())
    };
    for (k, p) in ps.iter().enumerate() {
        for _ in 0..antes.get(k).copied().unwrap_or(0) {
            quadro(&mut t, &mut rec)?;
        }
        for (_, g) in gotas.iter().filter(|(j, _)| *j == k) {
            solta(&mut t, g, l, &mut |t| quadro(t, &mut rec))?;
        }
        let m = meio(k)?;
        if t.paint_media() != m {
            t.handle_panel_event(PanelEvent::SelectOption(
                crate::ids::PAINTER_BRUSH_MEDIA,
                m.to_u8().to_string(),
            ));
        }
        t.set_brush_color_srgb8(p.cor);
        t.set_brush_size_px(p.tamanho_px);
        for i in 0..=TRACO {
            #[expect(clippy::cast_precision_loss, reason = "poucos quadros")]
            let f = i as f32 / TRACO as f32;
            let phase = match i {
                0 => PointerPhase::Down,
                i if i == TRACO => PointerPhase::Up,
                _ => PointerPhase::Move,
            };
            t.on_canvas_pointer(CanvasPointer {
                pos: ponto(p, f, l),
                pressure: p.pressao[0] + (p.pressao[1] - p.pressao[0]) * f,
                tilt: [0.0, 0.0],
                phase,
            });
            quadro(&mut t, &mut rec)?;
        }
        for _ in 0..ENTRE {
            quadro(&mut t, &mut rec)?;
        }
    }
    for _ in 0..CAUDA {
        quadro(&mut t, &mut rec)?;
    }
    let fim = MacroTime::from_secs_f64(s).0;
    for c in t.macro_para() {
        rec.push(trilha, fim, c).ok()?;
    }
    Some((t, ph2d_macro::codec::write(&rec.finish()), s))
}

#[cfg(test)]
mod tests {
    use super::{grava_no_papel, grava_no_papel_com_entrada, grava_o_splash};
    use crate::macro_track::em_cena::MacroEmCena;

    /// Texels não brancos da tela do palco de `doc` no instante `t`.
    fn pintados_em(doc: &[u8], t: f64) -> usize {
        let mut m = MacroEmCena::de(doc).expect("lê-se").expect("trilha");
        m.segue(t).expect("toca");
        let (tela, _, _) = m.toma_a_tela().expect("a tela mostra-se");
        tela.as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[..3] != [255; 3])
            .count()
    }

    /// A cena `PH2D_MACRO_SMOKE=5` abre com o clipe parado no instante 0: a gravação com ENTRADA mostra
    /// aí a tela branca e pinta depois. ⛔ O controlo: no papel sem entrada o instante 0 já tem o 1.º
    /// toque (a mancha da 1.ª foto da `=5`, 17/10).
    #[test]
    fn com_entrada_o_instante_0_e_a_tela_branca() {
        let (sem, _) = grava_no_papel(128, 0, 1).expect("arma");
        assert!(
            pintados_em(&sem, 0.0) > 0,
            "controlo: sem entrada, o instante 0 traz o 1.º toque"
        );
        let (com, d) = grava_no_papel_com_entrada(128, 0, 1, 30).expect("arma");
        assert_eq!(
            pintados_em(&com, 0.0),
            0,
            "com entrada, o instante 0 é a tela branca"
        );
        assert!(
            pintados_em(&com, d) > 128 * 128 / 50,
            "controlo: a gravação pinta"
        );
    }

    /// A cena `PH2D_MACRO_SMOKE=1` ensina o que acontece: o Splash tocado até ao fim pinta as quatro
    /// cores na tela branca, e o início é a tela branca.
    #[test]
    fn o_splash_tocado_pinta_a_tela_e_o_inicio_e_branco() {
        let (bytes, d) = grava_o_splash(128, 0).expect("arma no Wet Paint");
        let mut m = MacroEmCena::de(&bytes)
            .expect("lê-se")
            .expect("tem trilha do Painter");
        // Antes de qualquer passo (o instante 0 já traz o 1.º toque do pincel).
        let (inicio, _, _) = m.toma_a_tela().expect("a tela inicial mostra-se");
        assert!(inicio.iter().all(|&b| b == 255), "o início é a tela branca");
        m.segue(d).expect("toca até ao fim");
        assert!(m.erro().is_none(), "{:?}", m.erro());
        let (fim, _, _) = m.toma_a_tela().expect("a tela mudou");
        let pintados = fim
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[..3] != [255; 3])
            .count();
        let azuis = fim
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| u16::from(p[2]) > u16::from(p[0]) + 20)
            .count();
        let vermelhos = fim
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| u16::from(p[0]) > u16::from(p[2]) + 20)
            .count();
        assert!(pintados > 128 * 128 / 5, "pintou {pintados} texels");
        assert!(
            azuis > 0 && vermelhos > 0,
            "azul {azuis}, vermelho {vermelhos}"
        );
    }
}
