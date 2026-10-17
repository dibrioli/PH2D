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
}

// LITERAL-COLOR-OK: as cores de uma pincelada GRAVADA (o artista escolheu-as), não da UI.
pub(crate) const SPLASH: [Pincelada; 4] = [
    Pincelada {
        cor: [30, 90, 200],
        tamanho_px: 70.0,
        de: [0.18, 0.30],
        ate: [0.80, 0.42],
        curva: 0.06,
    },
    Pincelada {
        cor: [200, 40, 60],
        tamanho_px: 60.0,
        de: [0.22, 0.62],
        ate: [0.78, 0.52],
        curva: -0.08,
    },
    Pincelada {
        cor: [220, 160, 30],
        tamanho_px: 55.0,
        de: [0.50, 0.15],
        ate: [0.55, 0.85],
        curva: 0.05,
    },
    Pincelada {
        cor: [30, 90, 200],
        tamanho_px: 45.0,
        de: [0.30, 0.82],
        ate: [0.74, 0.70],
        curva: 0.04,
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

/// [`grava_pinceladas_como_a_maquina`] e o tool que gravou, parado no Parar (o editor depois dele).
#[cfg(test)]
pub(crate) fn grava_como_a_maquina_e_o_tool(
    lado: u32,
    ps: &[Pincelada],
    ritmo: &crate::macro_track::RitmoDaAgua,
    dts: &[f32],
) -> Option<(PainterTool, Vec<u8>, f64)> {
    grava_com(lado, 0, ps, &[], false, &[PaintMedia::WetPaint], Some((ritmo, dts)))
}

fn grava(
    lado: u32,
    alvo: u64,
    ps: &[Pincelada],
    antes: &[u32],
    papel: bool,
    meios: &[PaintMedia],
) -> Option<(Vec<u8>, f64)> {
    grava_com(lado, alvo, ps, antes, papel, meios, None).map(|(_, b, s)| (b, s))
}

/// A máquina de uma gravação: o ritmo da água no worker e a duração de cada quadro, em ms.
type Maquina<'a> = (&'a crate::macro_track::RitmoDaAgua, &'a [f32]);

fn grava_com(
    lado: u32,
    alvo: u64,
    ps: &[Pincelada],
    antes: &[u32],
    papel: bool,
    meios: &[PaintMedia],
    maquina: Option<Maquina<'_>>,
) -> Option<(PainterTool, Vec<u8>, f64)> {
    let meio = |k: usize| meios.get(k).or(meios.last()).copied();
    let mut t = PainterTool::default();
    let n = usize::try_from(lado).ok()?;
    if papel {
        t.abre_a_sprite(vec![255u8; n * n * 4], lado, lado);
    } else {
        t.set_source(vec![255u8; n * n * 4], lado, lado);
    }
    t.set_paint_media(meio(0)?);
    match maquina {
        Some((r, _)) => t.set_wet_ritmo_de_teste(Some(r.clone())),
        None => t.set_wet_relogio_fixo(true),
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
        match maquina {
            Some((_, d)) => {
                let dt = d[q as usize % d.len()];
                t.on_tick(dt);
                s += f64::from(dt) / 1000.0;
            }
            None => {
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
                pressure: 1.0,
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
    drop(quadro);
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
