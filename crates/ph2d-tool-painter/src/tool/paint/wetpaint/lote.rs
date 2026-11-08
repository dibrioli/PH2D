//! **O LOTE** — o que um lote de dabs do Wet Paint acerta antes de pousar (filho de [`super`], LOC cap, a partir de
//! `dab_route.rs`): o papel da sessão e o sub-passo da cápsula.

use super::*;
use ph2d_det_math::DetMath as _;

impl PainterTool {
    /// O papel do artista dá o dente ao motor (W2.7), reconciliado por lote pela [`PaperKey`].
    pub(super) fn reconcilia_o_papel(&self, sess: &mut WetSession, brush: &BrushSpec, ratio: u8) {
        // RECONCILED per batch, not seeded once: paper is substrate under
        // live water, but the SLOT is authored state and the session spans
        // strokes — a key of everything the seed reads decides. Key moved =
        // re-seed; slot disarmed = the engine re-bakes its own preset
        // (`rebake_paper`); unchanged = one small struct compare. The law is
        // the painter's own canvas-fixed paper sampling (the NEUTRAL door
        // `sample_tiled_rot_wrapped`, the same law the watercolor substrate
        // reads — one paper, never a second system). Known v1 gap: the
        // bitmap Size seam-snap under sprite Tiling (`snap_slot_size`) is
        // not applied — procedurals wrap exactly; a bitmap paper may seam.
        let chave = self.chave_do_papel(brush);
        let (period, tooth) = (chave.period, chave.tooth);
        // O Tooth (BUGS #32) mora no MOTOR, que o aplica nas duas portas que escrevem o papel: o assado
        // dele re-assa no `reconcile_facts` abaixo (papel sujo); um papel do artista re-semeia pela chave.
        sess.engine.set_paper_tooth(tooth);
        // ⚠️ **UM papel, o do PAINEL (W1e, F2; dono 10/10).** Com um Kind armado, o do artista; SEM Kind,
        // um papel SEM DESENHO (escolha do dono, 10/10 — `prancha_w1g_sem_papel.png`): as alturas do papel
        // de fábrica, ao acaso célula a célula. A água comporta-se como num papel (o limiar do depósito
        // por célula é o que a faz sangrar no molhado); o desenho do papel — as manchas — some. Nunca o
        // papel procedural do MOTOR: a chave existe nos dois casos, então o motor não re-assa o dele.
        let want = Some(chave);
        if sess.paper_key == want {
            return;
        }
        let sem_kind = !brush.paper.is_active();
        // O Tooth esculpe um papel; sem papel não age (as alturas entram inteiras, `dente(h, 1) = h`).
        if sem_kind {
            sess.engine.set_paper_tooth(1.0);
        }
        match sess.papel_pronto.take() {
            // O plano que o preparo já semeou com esta chave ([`super::preparo`]).
            Some((k, plano)) if Some(k) == want => {
                sess.engine.instala_papel(plano);
                super::preparo::conta_o_papel();
            }
            _ => {
                let lei = super::preparo::lei_do_papel(
                    brush.paper,
                    self.paint.paper_image.as_ref(),
                    period,
                    ratio,
                );
                sess.engine.seed_paper_with(&*lei);
            }
        }
        if sem_kind {
            sess.engine.set_paper_tooth(tooth);
        }
        sess.paper_key = want;
    }
}

/// ⭐ The dab-CAPSULE (Continuous, [`ph2d_painter_brush::capsula`]) reaches the engine as its
/// sub-dabs, oldest first, at least one engine cell apart: the deposit gates each against the
/// paper (`trail/deposit.rs::cell`), so only the sub-dabs themselves lay the dense stroke —
/// a capsule composed host-side either dilutes under the gate or widens past it (measured).
/// O sub-passo é o do raio do PINCEL — o do dab traz a pressão e a caneta deitada (×3,3), e o traçador sempre
/// espaçou pelo do pincel —, um inteiro de células ([`ph2d_painter_brush::capsula::sub_passo_sem_batimento`]).
/// O sub-passo da cápsula ANTES do arredondamento a células inteiras: a fracção `capsuleSpacing` do diâmetro, com o
/// tecto `capsuleMaxPx`. É o passo a que a receita do pincel foi afinada — [`super::pousa_o_dab`] pesa cada sub-dab
/// por `caminho ÷ este`, para o arredondamento (até 2× mais sub-dabs por célula, que cresciam o grumo do óleo entre
/// raio 24 e 40) não mudar a tinta por área (gate `o_impasto_nao_encaroca_em_tamanho_nenhum`, 31/10).
pub(super) fn sub_passo_natural(tuning: &ph2d_wet_paint::tuning::Tuning, brush: &BrushSpec) -> f32 {
    let sub_da_capsula = tuning.get(ph2d_wet_paint::tuning::Knob::CapsuleSpacing) as f32;
    let tecto_da_capsula = tuning.get(ph2d_wet_paint::tuning::Knob::CapsuleMaxPx) as f32;
    let n = ph2d_painter_brush::capsula::sub_passo_px_a(brush.clamped_radius(), sub_da_capsula);
    if tecto_da_capsula > 0.0 {
        n.min(tecto_da_capsula)
    } else {
        n
    }
}

/// O passo pelo qual [`super::pousa_o_dab`] pesa cada sub-dab (`caminho ÷ este`): o [`sub_passo_natural`], nunca
/// menor que uma CÉLULA — a grade não vê sub-dabs mais densos que um por célula e a tinta por área já é por célula.
/// Sem o piso, a 2 px por célula e sub-passo natural ~1 px cada sub-dab pesava 2× (o Oil Bristle de 40 px: corpo
/// 0,274 contra 0,254 a 1 px; o desfiado da ponta 0,87 → 2,14, gate `a_ponta_desfiada_nao_depende_da_grade`).
pub(super) fn sub_passo_do_peso(
    tuning: &ph2d_wet_paint::tuning::Tuning,
    brush: &BrushSpec,
    ratio: u8,
) -> f32 {
    sub_passo_natural(tuning, brush).max(f32::from(ratio).max(1.0))
}

pub(super) fn sub_passo_do_lote(
    tuning: &ph2d_wet_paint::tuning::Tuning,
    brush: &BrushSpec,
    ratio: u8,
) -> f32 {
    let mais_perto = tuning.get(ph2d_wet_paint::tuning::Knob::CapsuleRoundNearest) > 0.5;
    let n = sub_passo_natural(tuning, brush);
    let cel = f32::from(ratio).max(1.0);
    if mais_perto {
        ph2d_painter_brush::capsula::sub_passo_mais_perto(n, cel)
    } else {
        ph2d_painter_brush::capsula::sub_passo_sem_batimento(n, cel)
    }
}

/// ⭐ **O traço FINO pesa por área o que o largo pesa** (J2, `docs/Painter/Pinceis/03_MOTORES.md` §7 b): cada célula
/// recebe `2r ÷ passo` dabs; o passo é uma fracção do diâmetro, mas nunca menos de UMA célula (o piso do caminhador
/// e da cápsula). Abaixo dele a contagem por célula cai com o diâmetro, e com ela o pigmento e a água por área: o
/// recto de 5 px do 8B cobria 0,38 contra 0,81 a 40 px (o Rebelle 0,80 nos dois). Cada dab pesa então a célula ÷ o
/// passo natural a [`FRACAO_DO_FINO`] do diâmetro — a contagem por área de um traço de ~29 px —; `1` daí para cima, ao
/// bit. ⭐ Com o TECTO da lei física: o fino pousa `max(2r ÷ célula, 1)` dabs por célula — um dab menor que a célula
/// pousa nela UMA vez —, e o pigmento por área nunca passa o do largo (`1 ÷ FRACAO_DO_FINO` dabs): a 2 px por célula
/// e r 0,5 o peso lia 57× num só dab (`peso_tests`).
pub(super) fn peso_do_passo_fino(brush: &BrushSpec, ratio: u8) -> f32 {
    let natural = FRACAO_DO_FINO * 2.0 * brush.clamped_radius();
    let cel = f32::from(ratio).max(1.0);
    if natural > 0.0 && natural < cel {
        (cel / natural).min(1.0 / FRACAO_DO_FINO)
    } else {
        1.0
    }
}

/// O que o motor lê do LOTE inteiro: a água do dab fino ([`peso_do_passo_fino`]: o pigmento pesa, a água fica a do
/// dab) e ⭐ **o relevo pelo diâmetro** (J2, `docs/Painter/Pinceis/03_MOTORES.md` §7 c): o `Emboss` que a célula
/// guarda vezes `(ref ÷ D)^γ` ([`ph2d_wet_paint::tuning::Knob::EmbossSizeExp`]), com `D` o diâmetro do PINCEL (o do
/// traço — o do dab traz a pressão e mudaria o relevo dentro de uma rampa). `γ = 0` = `1`, ao bit.
pub(super) fn acerta_o_lote(
    engine: &mut ph2d_wet_paint::painter::Engine,
    brush: &BrushSpec,
    fino: f32,
    ratio: u8,
) {
    use ph2d_wet_paint::tuning::Knob;
    engine.agua_do_dab = f64::from(fino).recip();
    engine.sim.px_por_celula = f64::from(ratio.max(1));
    let gama = engine.tuning.get(Knob::EmbossSizeExp);
    engine.sim.relevo_do_tamanho = if gama > 0.0 {
        let d = 2.0 * f64::from(brush.clamped_radius()).max(0.5);
        (engine.tuning.get(Knob::EmbossSizeRef) / d).det_powf(gama)
    } else {
        1.0
    };
}

/// ⭐ **A textura acompanha o TAMANHO do pincel** (W3 K2, `docs/Painter/Pinceis/W3_K2_textura.md`): o factor
/// `(D ÷ ref)^γ` ([`ph2d_wet_paint::tuning::Knob::TextureSizeExp`]) por que crescem o grão `Tiled` e o pó solto, com `D`
/// o diâmetro do PINCEL (o do traço: o grão preso à tela re-faseava a cada oscilação da pressão). `γ = 0` = `1`, ao bit.
pub(super) fn escala_da_textura(
    engine: &ph2d_wet_paint::painter::Engine,
    brush: &BrushSpec,
) -> f32 {
    use ph2d_wet_paint::tuning::Knob;
    let gama = engine.tuning.get(Knob::TextureSizeExp);
    if gama > 0.0 {
        let d = 2.0 * f64::from(brush.clamped_radius()).max(0.5);
        (d / engine.tuning.get(Knob::TextureSizeRef).max(1.0)).det_powf(gama) as f32
    } else {
        1.0
    }
}

/// ⭐ **Os FIOS crescem com o pincel** (J6 cerdas, [`ph2d_wet_paint::tuning::Knob::BristleLaneSizeExp`]): o factor
/// `(D ÷ ref)^γ` dos comprimentos dos fios ([`ph2d_wet_paint::bristles_fios::Fios::escalado`]), com `D` o diâmetro do
/// pincel e `ref` o [`ph2d_wet_paint::tuning::Knob::TextureSizeRef`]. `γ = 0` = `1`, ao bit.
pub(super) fn escala_dos_fios(engine: &ph2d_wet_paint::painter::Engine, brush: &BrushSpec) -> f64 {
    use ph2d_wet_paint::tuning::Knob;
    let gama = engine.tuning.get(Knob::BristleLaneSizeExp);
    if gama > 0.0 {
        let d = 2.0 * f64::from(brush.clamped_radius()).max(0.5);
        (d / engine.tuning.get(Knob::TextureSizeRef).max(1.0)).det_powf(gama)
    } else {
        1.0
    }
}

/// O grão do dab sob [`escala_da_textura`]: só o mapeamento `Tiled` (preso à tela) — o View/Random já é relativo ao
/// dab, o Flow ao traço e o Stencil ao gizmo. O `Size` MULTIPLICA a coordenada, logo dividi-lo cresce o padrão.
pub(super) fn escala_o_grao(t: &mut ph2d_painter_brush::texture::TextureSettings, escala: f32) {
    if escala != 1.0 && t.mapping == ph2d_painter_brush::texture::TextureMapping::Tiled && !t.flow {
        t.size = t.size.map(|s| s / escala);
    }
}

/// ⭐ **As cerdas do Shape crescem com o pincel** (W3 K3b, `docs/Painter/Pinceis/W3_K3b_cerda.md`): o tipo `Bristles` é
/// uma lei em PÍXEIS (1 texel = 1 px a todo tamanho), então os fios do leque de 200 px eram os de 40 — o mesmo
/// [`escala_da_textura`] do grão `Tiled`, em qualquer mapeamento (View e Flow incluídos: o tapete nunca seguia o raio).
/// ⭐ Por EIXO (W3 K5, `docs/Painter/Pinceis/W3_K5.md`): `escala = [u, v]` — o `v` (através do traço, com Flow/Rake) pelo
/// [`ph2d_wet_paint::tuning::Knob::TextureSizeExpAcross`] ([`escala_atraves`]).
pub(super) fn escala_as_cerdas(
    t: &mut ph2d_painter_brush::texture::TextureSettings,
    escala: [f32; 2],
) {
    if escala != [1.0, 1.0] && t.kind == ph2d_painter_brush::TextureKind::Bristles {
        t.size = [t.size[0] / escala[0], t.size[1] / escala[1]];
    }
}

/// O factor das cerdas ATRAVÉS do traço: `(D ÷ ref)^γ⊥` com `γ⊥` = [`ph2d_wet_paint::tuning::Knob::TextureSizeExpAcross`];
/// com [`ph2d_wet_paint::tuning::Knob::TextureAcrossOwn`] desligado (o default), o `escala` do [`escala_da_textura`], ao bit.
pub(super) fn escala_atraves(
    engine: &ph2d_wet_paint::painter::Engine,
    brush: &BrushSpec,
    escala: f32,
) -> f32 {
    use ph2d_wet_paint::tuning::Knob;
    if engine.tuning.get(Knob::TextureAcrossOwn) <= 0.0 {
        return escala;
    }
    let gama = engine.tuning.get(Knob::TextureSizeExpAcross);
    let d = 2.0 * f64::from(brush.clamped_radius()).max(0.5);
    (d / engine.tuning.get(Knob::TextureSizeRef).max(1.0)).det_powf(gama) as f32
}

/// A média do grão preso à tela, numa grelha fixa da tela (não depende de onde o traço passa nem de como os dabs se
/// agrupam em lotes): a régua que deixa o papel desenhar sem mudar a quantidade (Bug #61).
pub(in crate::tool::paint) fn media_do_grao(
    brush: &BrushSpec,
    imagem: Option<&ph2d_painter_brush::ImageMask<'_>>,
    canvas_wh: [f32; 2],
) -> f64 {
    const LADO: i64 = 32;
    const PASSO: i64 = 13;
    let mut rng = 0u64;
    let base = ph2d_painter_brush::texture::dab_basis(
        &brush.texture,
        &mut rng,
        canvas_wh,
        ph2d_painter_brush::footprint::FootprintDeform::identity(),
    );
    let mut soma = 0.0;
    for y in 0..LADO {
        for x in 0..LADO {
            soma += f64::from(ph2d_painter_brush::dab::grain_at_na_ponta(
                brush,
                &base,
                imagem,
                x * PASSO,
                y * PASSO,
                [0.0, 0.0],
                brush.clamped_radius(),
                1.0,
            ));
        }
    }
    (soma / (LADO * LADO) as f64).max(1e-3)
}

/// O que decide a média de um grão: um lote só amostra de novo quando isto muda (o custo: BUGS #61).
#[derive(Clone, Copy, PartialEq)]
struct ChaveDaMedia {
    grao: ph2d_painter_brush::texture::TextureSettings,
    profundidade: f32,
    granulacao: f32,
    raio: f32,
    versao_da_imagem: u64,
    canvas_wh: [f32; 2],
}

/// As médias dos últimos grãos (dois por lote: o aplicado e o de agora), na sessão.
#[derive(Clone, Copy, Default)]
pub(super) struct MediasDoGrao {
    fila: [Option<(ChaveDaMedia, f64)>; 4],
    proxima: usize,
    tapete: Option<([u32; 4], f64)>,
}

impl MediasDoGrao {
    /// A média do tapete de cerdas destes sliders (o `texv` de um pincel de cerdas sem Grain).
    fn do_tapete(&mut self, cerdas: &ph2d_painter_brush::texture::TextureSettings) -> f64 {
        let chave = [0, 1, 2, 3].map(|i| cerdas.params[i].to_bits());
        if let Some((_, m)) = self.tapete.filter(|(c, _)| *c == chave) {
            return m;
        }
        let tile = ph2d_painter_brush::texture::bristle_tile(&cerdas.params);
        let soma: f64 = tile.iter().map(|v| f64::from(*v)).sum();
        let m = (soma / tile.len().max(1) as f64).max(1e-3);
        self.tapete = Some((chave, m));
        m
    }

    pub(super) fn media(
        &mut self,
        brush: &BrushSpec,
        (imagem, versao_da_imagem): GraoComImagem<'_, '_>,
        canvas_wh: [f32; 2],
    ) -> f64 {
        let chave = ChaveDaMedia {
            grao: brush.texture,
            profundidade: brush.grain_depth(),
            granulacao: brush.effective_granulation(),
            raio: brush.clamped_radius(),
            versao_da_imagem,
            canvas_wh,
        };
        if let Some((_, m)) = self.fila.iter().flatten().find(|(c, _)| *c == chave) {
            return *m;
        }
        let m = media_do_grao(brush, imagem, canvas_wh);
        self.fila[self.proxima] = Some((chave, m));
        self.proxima = (self.proxima + 1) % self.fila.len();
        m
    }
}

/// A imagem de um slot de grão e a versão dela.
pub(super) type GraoComImagem<'a, 'b> = (Option<&'a ph2d_painter_brush::ImageMask<'b>>, u64);

/// Como o Grain de agora entra na tinta, medido contra o Grain com que o pincel foi aplicado.
pub(super) struct ReguaDoGrao {
    /// A ponta (as cerdas, ou o `texv = 1` da ponta lisa) continua a ser o `texv`, e o grão desenha depois do portão.
    pub(super) ponta_sob_o_grao: bool,
    /// A média que divide o grão depois do portão (ausente nas Grain Colors: o grão só dá a cor).
    pub(super) media_depois_do_portao: Option<f64>,
    /// `média(aplicado) ÷ média(agora)` quando o Grain tomou o lugar do aplicado; `1`, ao bit, quando é ele.
    pub(super) ganho_do_grao: f64,
    /// O Grain aplicado saiu: `[média dele, média do tapete]`. O `texv` fica à média dele, e as cerdas desenham depois do
    /// portão, de média 1.
    pub(super) sem_grao: Option<[f64; 2]>,
}

/// **Armar, trocar ou tirar um Grain não muda a quantidade de tinta** (Bug #61, dono 04/11: «consertar»). Aplicado sem
/// Grain, a ponta fica e o grão desenha de média 1; aplicado com G, outro grão entra à média de G, e sem nenhum a ponta
/// fica à média de G. Same as Paper é trocar.
/// `flags` = `[grain_active, grao_e_o_papel, grao_e_cor]`; `brush` é o pincel do lote e já traz o grão que pinta.
pub(super) fn regua_do_grao(
    brush: &BrushSpec,
    [de_agora, proprio]: [GraoComImagem<'_, '_>; 2],
    [grain_active, grao_e_o_papel, grao_e_cor]: [bool; 3],
    canvas_wh: [f32; 2],
    escala_tex: f32,
    medias: &mut MediasDoGrao,
) -> ReguaDoGrao {
    let g = brush.grao_aplicado;
    let mut do_aplicado = *brush;
    do_aplicado.texture = g.grao;
    let aplicado_amostra = g.media.is_some() || do_aplicado.grain_samples(proprio.0.is_some());
    let ponta_sob_o_grao = grain_active && !aplicado_amostra;
    let trocou = grain_active && aplicado_amostra && (grao_e_o_papel || brush.grao_trocado());
    let (usa_media, usa_ganho) = (ponta_sob_o_grao && !grao_e_cor, trocou && !grao_e_cor);
    let tirou = !grain_active && aplicado_amostra && brush.grao_trocado();
    if !usa_media && !usa_ganho && !tirou {
        return ReguaDoGrao {
            ponta_sob_o_grao,
            media_depois_do_portao: None,
            ganho_do_grao: 1.0,
            sem_grao: None,
        };
    }
    // À escala em que o dab o lê (o papel não cresce com o pincel, `pousa_o_dab`).
    escala_o_grao(&mut do_aplicado.texture, escala_tex);
    if tirou {
        let m = g
            .media
            .map_or_else(|| medias.media(&do_aplicado, proprio, canvas_wh), f64::from);
        return ReguaDoGrao {
            ponta_sob_o_grao,
            media_depois_do_portao: None,
            ganho_do_grao: 1.0,
            sem_grao: Some([m, medias.do_tapete(&brush.shape)]),
        };
    }
    let mut que_pinta = *brush;
    if !grao_e_o_papel {
        escala_o_grao(&mut que_pinta.texture, escala_tex);
    }
    let media = medias.media(&que_pinta, de_agora, canvas_wh);
    ReguaDoGrao {
        ponta_sob_o_grao,
        media_depois_do_portao: usa_media.then_some(media),
        sem_grao: None,
        ganho_do_grao: if usa_ganho {
            g.media
                .map_or_else(|| medias.media(&do_aplicado, proprio, canvas_wh), f64::from)
                / media
        } else {
            1.0
        },
    }
}

/// ⭐ **A nuvem do spray pousa o pigmento de UM dab** (W3 K5, `docs/Painter/Pinceis/W3_K5.md`): o depósito do Wet é
/// por CÉLULA, então `n` marcas de diâmetro `p·D` pousavam, juntas, só `n·p²` do pigmento de um dab do pincel — com
/// `particle` 0,1 e 4 marcas, 4 % — e liam-se DISCOS PÁLIDOS a todo tamanho (o Rebelle: pontos pequenos e escuros).
/// Cada marca menor que o pincel leva `1 ÷ (n·p²)` do pigmento por célula (a área do dab repartida pelas marcas);
/// sem spray, ou com marcas do tamanho do pincel, `1`, ao bit. `p` tem o piso de 0,5 px de raio do `dab_from`.
pub(super) fn pigmento_da_marca(brush: &BrushSpec) -> f32 {
    let r = brush.clamped_radius();
    let n = brush.dab_jitter.conta_no(brush.spray_count, r);
    let p = brush.dab_jitter.particula_no(brush.spray_count, r);
    if n <= 1 || p >= 1.0 {
        return 1.0;
    }
    let p = p.max(0.5 / r.max(0.5));
    1.0 / (n as f32 * p * p)
}

/// A fracção do diâmetro de [`peso_do_passo_fino`] (o diâmetro de referência `1 ÷ (2 · 0,035)` ≈ 29 px). MEDIDA
/// (03_MOTORES §7 b): 0,025 (a cápsula, 40 px) repunha tudo mas tirava do vale as mídias afinadas a 17–18 px (Soft
/// Pastel, Marker Liner, Dabber) e a tabela da largura colapsava a 1–2 px; 0,05 (20 px) deixava 8 pincéis abaixo
/// da barra (o 8B 0,82 do recto de 40); 0,035 passa os 26 com origem sem tirar nenhuma cena de fábrica do lugar.
pub(super) const FRACAO_DO_FINO: f32 = 0.035;

/// ⭐ **O RELEVO de cada fio num dab** (W3/W3b, `ph2d_wet_paint::bristles_relevo`; a partir de `dab_route.rs`, LOC
/// cap): o alvo de cada fio tabelado UMA vez por dab (o ruído de cada fio custava dois Perlin por célula — doc
/// 49_medicao §5), contínuo através dos fios (o degrau por fio era a escada do traço inclinado), penteado na silhueta
/// `sil` do dab (`cel` = o centro em células, `meia` = a meia-caixa, `passos` = o pente por comprimento).
#[allow(clippy::too_many_arguments)]
pub(super) fn penteia_o_relevo_do_dab(
    engine: &mut ph2d_wet_paint::painter::Engine,
    f: &ph2d_wet_paint::bristles_fios::FiosNoDab,
    relevo: &ph2d_wet_paint::bristles_relevo::Relevo,
    contacto: f64,
    cel: [f64; 2],
    meia: [f64; 2],
    px_cel: f64,
    sil: ph2d_wet_paint::brush::CellFn<'_>,
    passos: f64,
) {
    let (k0, k1) = f.faixa((meia[0].det_hypot(meia[1]) + 2.0) * px_cel);
    let (k0, k1) = (k0 - 1, k1 + 1);
    let tabela: Vec<f64> = (k0..=k1)
        .map(|k| relevo.alvo_do_fio(k, f.ganho_do_fio(k), contacto))
        .collect();
    let do_fio = |k: i32| -> f64 {
        usize::try_from(k - k0)
            .ok()
            .and_then(|i| tabela.get(i).copied())
            .unwrap_or_else(|| relevo.alvo_do_fio(k, f.ganho_do_fio(k), contacto))
    };
    let alvo = |cx: i32, cy: i32| -> f64 {
        let dc = [f64::from(cx) - cel[0], f64::from(cy) - cel[1]];
        let ([a, b], t) = f.entre_fios(dc.map(|v| v * px_cel));
        ph2d_wet_paint::bristles_relevo::entre(do_fio(a), do_fio(b), t) / px_cel
    };
    engine.pentear_relevo(cel[0], cel[1], meia, sil, &alvo, passos);
}

#[cfg(test)]
mod peso_tests {
    /// ⭐ **O traço fino nunca pousa mais pigmento por área que o largo** (o tecto de [`super::peso_do_passo_fino`]): o
    /// largo pousa `1 ÷ FRACAO_DO_FINO` dabs por célula; o fino de passo de UMA célula pousa `max(2r ÷ célula, 1)` —
    /// o peso é a razão, e nunca passa `1 ÷ FRACAO_DO_FINO` (a 2 px por célula e r 0,5 lia 57×; a 4, 114×). Acima do
    /// diâmetro natural, 1.
    #[test]
    fn o_fino_nao_passa_o_pigmento_por_area_do_largo() {
        let tecto = 1.0 / super::FRACAO_DO_FINO;
        for ratio in [1u8, 2, 3, 4] {
            for r in [
                0.5f32, 0.75, 1.0, 1.5, 2.0, 3.0, 5.0, 10.0, 14.0, 20.0, 60.0,
            ] {
                let b = ph2d_painter_brush::BrushSpec {
                    radius_px: r,
                    ..Default::default()
                };
                let p = super::peso_do_passo_fino(&b, ratio);
                let cel = f32::from(ratio);
                let por_celula = (2.0 * r / cel).max(1.0);
                let esperado = if 2.0 * super::FRACAO_DO_FINO * r < cel {
                    (tecto / por_celula).max(1.0)
                } else {
                    1.0
                };
                assert!(
                    p <= tecto + 1e-3,
                    "r {r} grelha {ratio}: peso {p} > o tecto {tecto}"
                );
                assert!(
                    (p - esperado).abs() < 1e-3 * esperado,
                    "r {r} grelha {ratio}: {p} ≠ {esperado}"
                );
            }
        }
    }
}
