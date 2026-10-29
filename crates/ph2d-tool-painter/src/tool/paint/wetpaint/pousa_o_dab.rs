//! **O DAB que entra na água** — o corpo do laço de [`super::dab_route`] (filho de [`super`], LOC cap).
//!
//! A rota apura uma vez o que vale para o LOTE inteiro ([`LoteDoDab`]); aqui cada dab do lote vira lane,
//! silhueta, grão, cor e a porta do motor que o pousa.

use super::*;

/// O que [`super::dab_route`] apurou para o lote inteiro e cada dab lê (os nomes são os da rota).
pub(super) struct LoteDoDab<'a> {
    pub(super) brush: &'a BrushSpec,
    pub(super) erasing: bool,
    pub(super) opacidade_da_pressao: Option<f32>,
    pub(super) dual: Option<super::super::dual::RecorteNoWet>,
    pub(super) ratio: u8,
    pub(super) wet_tool: WetTool,
    pub(super) rampa: Option<super::super::watercolor_accum_cor::RampaDaAguada>,
    pub(super) padrao_vira_cor: bool,
    pub(super) grao_e_cor: bool,
    pub(super) shape_image: Option<ph2d_painter_brush::ImageMask<'a>>,
    pub(super) shape_ramp_lut: Option<&'a [f32]>,
    pub(super) cerdas: bool,
    pub(super) marca_do_spray: bool,
    pub(super) filme_liso: bool,
    pub(super) fios: ph2d_wet_paint::bristles_fios::Fios,
    pub(super) relevo: ph2d_wet_paint::bristles_relevo::Relevo,
    pub(super) rebordo: f64,
    pub(super) lamina: bool,
    pub(super) passo_do_pigmento: f32,
    pub(super) capsulas: bool,
    pub(super) shape_active: bool,
    pub(super) grain_image: Option<ph2d_painter_brush::ImageMask<'a>>,
    pub(super) grain_active: bool,
    pub(super) groups: &'a [u32],
    pub(super) canvas_wh: [f32; 2],
    pub(super) strength: f32,
    pub(super) fino: f32,
    pub(super) esmaece_fino: f64,
    pub(super) escala_tex: f32,
    pub(super) grao_e_o_papel: bool,
    pub(super) pigmento_da_marca: f32,
    pub(super) escala_cerdas: [f32; 2],
    pub(super) copia: &'a [u32],
    pub(super) pontas: bool,
    pub(super) alem: f32,
    pub(super) esbate_e: f32,
    pub(super) virtual_: bool,
    pub(super) lote: super::tampa::Lote,
    pub(super) gota: bool,
}

/// Um dab do lote (`didx` o índice na mão, `sub_k` o sub-dab da cápsula, `passo` o sub-passo) pousa na água.
pub(super) fn pousa_o_dab(
    l: &LoteDoDab<'_>,
    sess: &mut WetSession,
    dab_rng: &mut super::super::tiling::DabRng,
    didx: usize,
    sub_k: u64,
    passo: Option<f32>,
    d: &Dab,
) {
    let &LoteDoDab {
        brush,
        erasing,
        opacidade_da_pressao,
        ref dual,
        ratio,
        wet_tool,
        ref rampa,
        padrao_vira_cor,
        grao_e_cor,
        shape_image,
        shape_ramp_lut,
        cerdas,
        marca_do_spray,
        filme_liso,
        fios,
        relevo,
        rebordo,
        lamina,
        passo_do_pigmento,
        capsulas,
        shape_active,
        grain_image,
        grain_active,
        groups,
        canvas_wh,
        strength,
        fino,
        esmaece_fino,
        escala_tex,
        grao_e_o_papel,
        pigmento_da_marca,
        escala_cerdas,
        copia,
        pontas,
        alem,
        esbate_e,
        virtual_,
        lote,
        gota,
    } = l;
    let [x, y] = d.center;
    // A lane do dab ([`super::rota_da_lane::casa_a_lane`]); a borracha não tem lanes.
    let (li, lane_born) = if erasing {
        (0, false)
    } else if let Some(&li) = sess.fecho.get(didx) {
        // Um dab virtual das pontas rectas: a lane é a que ele continua ([`super::tampa`]).
        let p = sess.lanes[li].pos;
        let corda = (x - p[0]).hypot(y - p[1]);
        let caminho = super::tampa::passo_virtual(&sess.lanes[li], brush);
        super::rota_da_lane::anda_a_lane(sess, li, corda, caminho, wet_tool);
        (li, false)
    } else {
        super::rota_da_lane::casa_a_lane(sess, d, brush, copia[didx], wet_tool, gota)
    };
    // The lane's PREVIOUS centre (before this dab updates it) — the
    // direct tools' displacement source; a born lane has none, and
    // the eraser has no lanes at all.
    let lane_prev: Option<[f32; 2]> = (!erasing && !lane_born).then(|| sess.lanes[li].pos);
    // `Some(None)` = o dab sem rumo com as pontas rectas: não pousa (a lane anda na mesma).
    // O caminho desde o dab anterior da lane: o de um dab virtual é o passo dela (a abertura anda para trás).
    let caminho = if lane_prev.is_none() {
        0.0
    } else if virtual_ {
        super::tampa::passo_virtual(&sess.lanes[li], brush)
    } else {
        (d.arc_len - sess.lanes[li].arco).max(0.0)
    };
    let tampa = pontas.then(|| super::tampa::Tampa::do_dab(d, &mut sess.lanes[li], lote, alem));
    let corte = tampa.flatten();
    let esbate = (esbate_e > 0.0).then(|| {
        super::esbate::Esbate::do_dab(d, &mut sess.lanes[li], lote, esbate_e, pontas, filme_liso)
    });
    let peso_esbate = esbate.flatten().filter(|w| !w.trivial());
    // Per-dab fresh ink (Randomize): reload the lane's trail — a
    // brush dipped in new paint (see `Trail::set_base_color`).
    if !erasing && wet_tool == WetTool::Paint && d.color != sess.lanes[li].ink {
        sess.engine
            .set_stroke_color(li, super::rota_da_lane::tinta(d.color));
        sess.lanes[li].ink = d.color;
    }
    let (b, opacidade) =
        super::rota_da_lane::b_e_opacidade(d, brush, strength, opacidade_da_pressao, wet_tool);
    // Per-dab silhouette closure — the impasto walk's exact recipe
    // (spec at the dab's radius → rotor → footprint → Shape basis in
    // the stroke frame), evaluated per engine cell (cell − 1 = px).
    let mut spec = brush.at_dab(d);
    // O papel não cresce com o pincel: o «Same as Paper» fica à escala do papel visível.
    if !grao_e_o_papel {
        super::lote::escala_o_grao(&mut spec.texture, escala_tex);
    }
    super::lote::escala_as_cerdas(&mut spec.shape, escala_cerdas);
    let peso = lane_prev
        .filter(|_| passo_do_pigmento > 0.0)
        .map_or(1.0, |_| {
            let w = caminho / brush.passo_ao_longo_px(passo_do_pigmento, d.dir);
            // O passo do PRÓPRIO Spacing de afinação é o dab inteiro, ao byte (a corda sai dos centros em f32).
            if (w - 1.0).abs() < 1e-3 {
                1.0
            } else if capsulas {
                w
            } else {
                w.min(1.0)
            }
        });
    // O pente do relevo também é por comprimento: as células de caminho desde o dab anterior da lane (a
    // borracha, sem lanes, pelo sub-passo da cápsula).
    let passos_do_pente = lane_prev.map_or(1.0, |_| {
        let n = f64::from(caminho) / f64::from(ratio);
        if (n - 1.0).abs() < 1e-3 { 1.0 } else { n }
    });
    let fp = spec.dab_footprint_of(d);
    let dab_index = didx;
    let tex_rng = dab_rng.enter(&groups, dab_index);
    // Cada sub-dab da cápsula tira o SEU sorteio (a posição da textura, o pó) do do dab: repetido, os `n`
    // sub-dabs pousavam a mesma textura e o traço ganhava o período do Spacing (doc 51 §3, `os_fios`). O
    // `k = 0` é o próprio dab, o último: o fluxo segue para o próximo dab como sem a cápsula.
    if sub_k > 0 {
        *tex_rng = super::rota_da_lane::sorteio_do_sub_dab(*tex_rng, sub_k);
    }
    let shape_basis = (shape_active || cerdas).then(|| {
        ph2d_painter_brush::texture::shape_basis(
            &spec.shape,
            &mut *tex_rng,
            canvas_wh,
            fp,
            ph2d_painter_brush::texture::ShapeFrame::Stroke {
                arc_len: d.arc_len,
                unit_px: d.stroke_radius_px,
            },
        )
    });
    let shape_input =
        shape_basis
            .as_ref()
            .filter(|_| shape_active)
            .map(|sb| ph2d_painter_brush::ShapeInput {
                basis: sb,
                image: shape_image.as_ref(),
                ramp_lut: shape_ramp_lut,
            });
    // O alcance da ponta de imagem (Bug #52): o motor percorre `r · reach` células, e a
    // janela do rasto cabe o máximo do traço (com rotação por dab, o disco de qualquer ângulo).
    // A ponta reta (prompt 44): o canto do quadrado rodado sai do disco — a caixa é a da pegada
    // DESTE dab (`fp`, com a rotação dele); a janela só precisa do canto em QUALQUER ângulo quando a
    // pegada roda por dab (o Follow, o Jitter Rotate) — fixa, a do dab basta (medido: 1,21× o custo).
    let mut reach = spec.dab_tip_reach(shape_basis.as_ref());
    if spec.dab_quina > 0.0 {
        let e = fp.extent();
        reach = reach.max(e[0].max(e[1]));
    }
    let quina_roda = spec.dab_quina > 0.0 && (spec.follows_stroke() || spec.has_per_dab_rotation());
    let janela =
        if spec.shape_has_per_dab_rotation(shape_image.is_some()) || quina_roda || d.tilt.is_some()
        {
            spec.tip_reach().max(reach)
        } else {
            reach
        };
    sess.engine.silhouette_reach = [f64::from(reach), f64::from(janela)];
    // Grain basis AFTER the Shape basis, from the same RNG stream —
    // the colour route's resolve order (Shape before Grain), so the
    // random draws land where every other route puts them.
    let grain_basis = grain_active.then(|| {
        ph2d_painter_brush::texture::dab_basis(&spec.texture, &mut *tex_rng, canvas_wh, fp)
    });
    // O PÓ (prompt 43): a semente das partículas deste dab, um passo do MESMO fluxo (o splitmix64 do
    // pincel) — só com o pó ligado, para que nenhum outro pincel veja o fluxo mudar.
    if sess.engine.tuning.get(ph2d_wet_paint::tuning::Knob::Dust) > 0.0 {
        *tex_rng = tex_rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let z = (*tex_rng ^ (*tex_rng >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        sess.engine.sim.semente_do_po = (z ^ (z >> 31)) as u32;
    }
    let inv_r = 1.0 / d.radius_px.max(0.01);
    // ⚠️ A silhueta é avaliada em espaço de CANVAS, sempre — só
    // AMOSTRADA por célula. É por isso que a razão da grade não muda a
    // forma do pincel, apenas quão fino o fluido a resolve: o `t` sai
    // da distância em pixels ao centro do dab, e o Shape/Grain leem o
    // texel de canvas onde o sub-ponto caiu.
    //
    // ⚠️ **E ela é amostrada por COBERTURA, não no centro** — o AA do
    // depósito (`grid_map::cell_subsamples`): com a razão > 1 uma
    // decisão de um ponto por célula quantizava a borda do pincel em
    // degraus de `ratio` px, e nenhuma interpolação da SAÍDA recupera
    // isso. Em razão 1 são `n = 1` sub-pontos e o corpo abaixo é o de
    // sempre, ao bit.
    let (aa_n, aa_step) = grid_map::cell_subsamples(ratio);
    let aa_inv = 1.0 / f32::from(u16::from(aa_n) * u16::from(aa_n));
    // ⚠️ **Um carimbo REDONDO não roda** (W1e): rodar `(u, v)` e medir o comprimento muda o
    // último bit do raio, e com a água a correr no traço esse bit vira píxeis (o censo mediu o
    // Jitter Rotate e o ângulo a «mudar a tinta» de um carimbo sem ângulo). O Shape e o Grain
    // seguem a pegada rodada (`fp`): neles o ângulo é real.
    let deitada = d.tilt.is_some_and(|t| t.stretches());
    let fp_raio = if !spec.pegada_anisotropica() && spec.dab_curve.is_flat() && !deitada {
        ph2d_painter_brush::FootprintDeform::new(0.0, 0)
    } else {
        fp
    };
    // ⭐ **Lei B** (plano 27 W3): o padrão PROCEDURAL do Shape não entra na silhueta que o dente do
    // papel corta — vai ao motor no canal `padrao`, que multiplica DEPOIS do portão. A ponta de
    // imagem fica na silhueta (ela é o contorno), e a borracha apaga pela silhueta inteira.
    let padrao_a_parte = shape_input.is_some()
        && spec.shape.kind != ph2d_painter_brush::TextureKind::Image
        && !erasing;
    let sil_input = shape_input.filter(|_| !padrao_a_parte);
    let sil = |cx: i32, cy: i32| -> f64 {
        let mut acc = 0.0f32;
        for j in 0..aa_n {
            let sy = grid_map::cell_subsample_px(cy, ratio, j, aa_step);
            for i in 0..aa_n {
                let sx = grid_map::cell_subsample_px(cx, ratio, i, aa_step);
                let t = fp_raio.falloff_t((sx - d.center[0]) * inv_r, (sy - d.center[1]) * inv_r);
                let v = ph2d_painter_brush::dab::silhouette_at(
                    &spec,
                    sil_input,
                    t,
                    sx.floor() as i64,
                    sy.floor() as i64,
                    d.center,
                    d.radius_px,
                );
                acc += match corte {
                    Some(c) => v * c.cobre(sx, sy),
                    None => v,
                };
            }
        }
        f64::from(acc * aa_inv)
    };
    // Per-dab grain closure (armed slot only): replaces the bristle
    // sample inside the shaped stamp, cell − 1 = px like `sil`.
    let spec_ref = &spec;
    // O valor do Shape numa célula, pela cobertura (os sub-pontos do AA) — o padrão e a cor leem-no.
    let valor_da_forma = |si: ph2d_painter_brush::ShapeInput<'_>, cx: i32, cy: i32| -> f32 {
        let mut acc = 0.0f32;
        for j in 0..aa_n {
            let sy = grid_map::cell_subsample_px(cy, ratio, j, aa_step);
            for i in 0..aa_n {
                let sx = grid_map::cell_subsample_px(cx, ratio, i, aa_step);
                acc += ph2d_painter_brush::dab::shape_value_at(
                    spec_ref,
                    si,
                    sx.floor() as i64,
                    sy.floor() as i64,
                    d.center,
                    d.radius_px,
                );
            }
        }
        acc * aa_inv
    };
    let padrao = shape_input
        .filter(|_| padrao_a_parte && !padrao_vira_cor)
        .map(|si| move |cx: i32, cy: i32| -> f64 { f64::from(valor_da_forma(si, cx, cy)) });
    let padrao_arg = padrao
        .as_ref()
        .map(|p| p as ph2d_wet_paint::brush::CellFn<'_>);
    // A cor da célula: a Texture Color lê o valor do Shape (as cerdas incluídas — é a textura que a
    // prévia mostra), a Falloff Color a silhueta; sRGB `0..1` → os `0..255` do motor.
    let forma_da_cor = shape_basis
        .as_ref()
        .map(|sb| ph2d_painter_brush::ShapeInput {
            basis: sb,
            image: shape_image.as_ref(),
            ramp_lut: None,
        });
    let (centro_x, centro_y) = (
        grid_map::px_to_cell(f64::from(x), ratio),
        grid_map::px_to_cell(f64::from(y), ratio),
    );
    // Os FIOS (W2): a pressão de cada fio e a oscilação, no comprimento de caminho do dab, em px.
    let (arco, rumo) = (f64::from(d.arc_len), [d.dir[0], d.dir[1]].map(f64::from));
    let contacto = f64::from(d.radius_px / d.stroke_radius_px.max(1e-3));
    let fios_dab = fios.no_dab(arco, rumo, contacto);
    // O RELEVO de cada fio (W3): as faixas dos fios, mesmo sem pressão nem cor, e a altura de cada uma.
    let relevo_dab = (!relevo.neutro())
        .then(|| fios_dab.unwrap_or_else(|| fios.resolvido(arco, rumo, contacto)));
    let px_cel = f64::from(ratio);
    let cell_x = grid_map::px_to_cell(f64::from(x), ratio);
    let cell_y = grid_map::px_to_cell(f64::from(y), ratio);
    let cell_r = grid_map::px_len_to_cell(f64::from(d.radius_px), ratio);
    // A caixa do relevo (W3): a da PEGADA da silhueta (a extensão de `fp_raio` × o alcance do Shape).
    let meia = {
        let (e, alc) = (fp_raio.extent(), sess.engine.silhouette_reach[0].max(1.0));
        [0, 1].map(|i| cell_r * alc * f64::from(e[i]))
    };
    // A PONTA DESFIADA (J2 §7 c): cada fio com o seu desvio ao longo do traço ([`super::ponta_das_cerdas`]).
    let rho = sess
        .engine
        .tuning
        .get(ph2d_wet_paint::tuning::Knob::BristleTipRagged);
    let alc = meia[0].max(meia[1]) * px_cel;
    let ponta = super::ponta_das_cerdas::PontaDasCerdas::nova(
        relevo_dab.or(fios_dab),
        rho,
        [centro_x, centro_y],
        px_cel,
        alc,
        &sil,
    );
    // A ponta que ESFARELA (W3 K3b): com um grão PRESO À TELA armado, o desvio de cada célula é o do grão, não o
    // do fio — a ponta acaba em grumos. Sem grão `Tiled`, a desfiada de sempre, ao bit.
    let esfarela = grain_basis.as_ref().filter(|_| {
        spec.texture.mapping == ph2d_painter_brush::texture::TextureMapping::Tiled
            && !spec.texture.flow
    });
    let grao_img_ponta = grain_image.as_ref();
    let sil = |cx: i32, cy: i32| match (ponta.as_ref(), esfarela) {
        (Some(p), Some(gb)) => {
            let u = ph2d_painter_brush::texture::sample(
                &spec.texture,
                gb,
                grid_map::cell_center_texel(cx, ratio),
                grid_map::cell_center_texel(cy, ratio),
                d.center,
                d.radius_px,
                grao_img_ponta,
            );
            p.recorta_pelo_grao(&sil, cx, cy, f64::from(u))
        }
        (Some(p), None) => p.recorta(&sil, cx, cy),
        (None, _) => sil(cx, cy),
    };
    // O depósito, o pente e o grão que lê a ponta leem a MESMA silhueta (com a ponta), avaliada UMA vez por célula
    // e guardada no dab ([`super::sil_memo`]), ao byte.
    let memo =
        (relevo_dab.is_some() || lamina || grain_active && spec.texture.lei.mode.le_a_ponta())
            .then(|| super::sil_memo::SilMemo::nova(cell_x, cell_y, meia));
    let sil = |cx: i32, cy: i32| -> f64 {
        memo.as_ref()
            .map_or_else(|| sil(cx, cy), |m| m.ou(cx, cy, sil))
    };
    let grao_da_cor = grain_basis.as_ref().filter(|_| grao_e_cor);
    let grao_img = grain_image.as_ref();
    let cor = rampa.as_ref().map(|r| {
        move |cx: i32, cy: i32, queda: f64| -> [f32; 4] {
            let v = forma_da_cor.map(|si| valor_da_forma(si, cx, cy));
            let g = grao_da_cor.map(|gb| {
                ph2d_painter_brush::texture::sample(
                    &spec_ref.texture,
                    gb,
                    grid_map::cell_center_texel(cx, ratio),
                    grid_map::cell_center_texel(cy, ratio),
                    d.center,
                    d.radius_px,
                    grao_img,
                )
            });
            let c = r.cor_do_dab(v, g, queda as f32, d.color_shift);
            [c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, c[3]]
        }
    });
    // O que é de cada FIO, uma vez por dab (o traço rápido pagava os Perlin e o HSLuv por célula).
    let tabela = fios_dab.map(|f| f.tabela(f64::from(d.radius_px) * 2.0 + 4.0 * px_cel));
    let tabela = tabela.as_ref();
    // A cor de cada FIO (W2, o ruído HSLuv) sobre a da Shape Color ou a do dab.
    let cor_fios = fios_dab.filter(|_| fios.tem_cor()).map(|f| {
        let (cor, base) = (cor.as_ref(), d.color.map(|c| c * 255.0));
        let t = tabela.expect("a tabela dos fios do dab");
        // Sem Shape Color a cor de base é a do dab: a de cada fio sai UMA vez (o HSLuv era por célula).
        let (k0, k1) = t.faixa();
        let do_fio: Vec<[f32; 3]> = if cor.is_none() {
            (k0..=k1).map(|k| f.cor_do_fio_em(base, k, t)).collect()
        } else {
            Vec::new()
        };
        move |cx: i32, cy: i32, queda: f64| -> [f32; 4] {
            let c = cor.map_or([base[0], base[1], base[2], 1.0], |c| c(cx, cy, queda));
            let dc = [f64::from(cx) - centro_x, f64::from(cy) - centro_y];
            let fio = f.no_ponto_em(dc.map(|v| v * px_cel), t).fio;
            let rgb = usize::try_from(fio - k0)
                .ok()
                .and_then(|i| do_fio.get(i).copied())
                .unwrap_or_else(|| f.cor_do_fio_em([c[0], c[1], c[2]], fio, t));
            [rgb[0], rgb[1], rgb[2], c[3]]
        }
    });
    let cor_arg = match (cor_fios.as_ref(), cor.as_ref()) {
        (Some(c), _) => Some(c as ph2d_wet_paint::brush::ColorFn<'_>),
        (None, Some(c)) => Some(c as ph2d_wet_paint::brush::ColorFn<'_>),
        (None, None) => None,
    };
    let gimg = grain_image.as_ref();
    // O modo do grão que lê a PONTA (Subtract, Darken, Color Burn — L9) morde a orla pela silhueta da célula.
    let na_ponta = spec_ref.texture.lei.mode.le_a_ponta();
    let grain = grain_basis.as_ref().map(|gb| {
        move |cx: i32, cy: i32| -> f64 {
            let px = grid_map::cell_center_texel(cx, ratio);
            let py = grid_map::cell_center_texel(cy, ratio);
            let w = if na_ponta { sil(cx, cy) as f32 } else { 1.0 };
            f64::from(ph2d_painter_brush::dab::grain_at_na_ponta(
                spec_ref,
                gb,
                gimg,
                px,
                py,
                d.center,
                d.radius_px,
                w,
            ))
        }
    });
    // As cerdas do Shape, pela MESMA porta do Grain e no referencial do motor: o desvio em células ao
    // centro do dab (`x − cx`, como `brush.rs::stamp_row_shaped`) — no neutro, o `sample_bristle` do
    // motor ao bit. Um Grain armado continua a tomar o lugar delas (W2.4; D7 medido no 27 §6.7).
    // Com o FILME do traço (`Knob::StrokeFilm`, J4 aguarela) o tapete fica de fora: a lavagem é lisa.
    let cerdas_sb = shape_basis
        .as_ref()
        .filter(|_| cerdas && grain.is_none() && !filme_liso && !marca_do_spray);
    let cerdas_fn = cerdas_sb.map(|sb| {
        let forma = &spec_ref.shape;
        let px_por_celula = f64::from(ratio);
        let raio = d.radius_px;
        move |cx: i32, cy: i32| -> f64 {
            let (fx, fy) = (f64::from(cx), f64::from(cy));
            let dc = [fx - centro_x, fy - centro_y];
            let tapete = |q: [f64; 2]| {
                ph2d_painter_brush::texture::bristles_at(
                    forma,
                    sb,
                    q,
                    [fx, fy],
                    px_por_celula,
                    raio,
                )
            };
            let Some(f) = fios_dab else {
                return tapete(dc);
            };
            let p = match tabela {
                Some(t) => f.no_ponto_em(dc.map(|v| v * px_por_celula), t),
                None => f.no_ponto(dc.map(|v| v * px_por_celula)),
            };
            p.ganho * tapete(p.q.map(|v| v / px_por_celula))
        }
    });
    let lisa = |_: i32, _: i32| -> f64 { 1.0 };
    let grain_arg: Option<ph2d_wet_paint::brush::CellFn<'_>> =
        Some(match (grain.as_ref(), cerdas_fn.as_ref()) {
            (Some(_), _) if grao_e_cor => &lisa,
            (Some(g), _) => g,
            (None, Some(c)) => c,
            (None, None) => &lisa,
        });
    // O ponto e o raio, em CÉLULAS — as duas coisas que o motor mede
    // na sua própria grade. O `d.radius_px` continua sendo o raio da
    // silhueta (em pixels), e é assim que o pincel não muda de tamanho
    // quando a razão muda: o mesmo disco, resolvido mais grosso.
    // O dual brush (L6) recorta o que POUSA pelo canal do padrão (depois do portão do papel, `RecorteNoWet`).
    let pad = dual.as_ref().map(|r| {
        let pegada = super::super::dual::PegadaDaPrimeira::do_dab(&spec, d);
        r.padrao(padrao_arg, &sil, ratio, pegada)
    });
    let padrao_arg = pad
        .as_ref()
        .map_or(padrao_arg, |p| Some(p as ph2d_wet_paint::brush::CellFn<'_>));
    // As pontas rectas: metade dos dabs cobre cada célula — a soma volta pelo padrão (depois do portão), e a
    // cor da queda lê a de sempre. O filme (o máximo) não soma: fica.
    let k_tampa = super::tampa::COMPENSA;
    let compensa = corte.is_some() && !filme_liso;
    let padrao_tampa = compensa.then_some(move |cx: i32, cy: i32| -> f64 {
        k_tampa * padrao_arg.map_or(1.0, |p| p(cx, cy))
    });
    let padrao_arg = padrao_tampa
        .as_ref()
        .map_or(padrao_arg, |p| Some(p as ph2d_wet_paint::brush::CellFn<'_>));
    let cor_tampa = cor_arg
        .filter(|_| compensa)
        .map(|c| move |cx: i32, cy: i32, queda: f64| -> [f32; 4] { c(cx, cy, queda / k_tampa) });
    let cor_arg = cor_tampa
        .as_ref()
        .map_or(cor_arg, |c| Some(c as ph2d_wet_paint::brush::ColorFn<'_>));
    // As pontas que esbatem: o peso ao comprido, depois do portão (a cor da queda lê a de sempre).
    let padrao_esbate = peso_esbate.map(|w| {
        move |cx: i32, cy: i32| -> f64 {
            let x = grid_map::cell_subsample_px(cx, ratio, 0, f32::from(ratio));
            let y = grid_map::cell_subsample_px(cy, ratio, 0, f32::from(ratio));
            f64::from(w.peso(x, y)) * padrao_arg.map_or(1.0, |p| p(cx, cy))
        }
    });
    let padrao_arg = padrao_esbate
        .as_ref()
        .map_or(padrao_arg, |p| Some(p as ph2d_wet_paint::brush::CellFn<'_>));
    let cor_esbate = cor_arg.zip(peso_esbate).map(|(c, w)| {
        move |cx: i32, cy: i32, queda: f64| -> [f32; 4] {
            let x = grid_map::cell_subsample_px(cx, ratio, 0, f32::from(ratio));
            let y = grid_map::cell_subsample_px(cy, ratio, 0, f32::from(ratio));
            let k = f64::from(w.peso(x, y));
            c(cx, cy, if k > 1e-6 { queda / k } else { queda })
        }
    });
    let cor_arg = cor_esbate
        .as_ref()
        .map_or(cor_arg, |c| Some(c as ph2d_wet_paint::brush::ColorFn<'_>));
    if matches!(tampa, Some(None)) || matches!(esbate, Some(None)) {
        sess.lanes[li].pos = d.center;
        sess.lanes[li].arco = d.arc_len;
        if gota {
            super::rota_da_lane::solta_a_gota(sess, li);
        }
        return;
    }
    if erasing {
        // W2.6: the same §9 mapping, routed to the engine's ERASER —
        // silhouette and grain shape the erase exactly as they shape
        // the deposit. O Flow (a taxa por dab, à vista na Borracha do Wet) escala a força
        // do Erase Strength; a Strength escondida já saiu do `b` (`coverage / strength`).
        sess.engine.dispatch_pressure_dab_erase(
            cell_x,
            cell_y,
            b,
            f64::from(brush.flow.clamp(0.0, 1.0)),
            cell_r,
            Some(&sil),
            grain_arg,
        );
        // A borracha tira também o relevo que houver (sem o campo, nada a tirar).
        if !sess.engine.active_grid().altura.is_empty() {
            let passos = passo.map_or(1.0, |s| f64::from(s) / px_cel);
            sess.engine
                .pentear_relevo(cell_x, cell_y, meia, &sil, &|_, _| 0.0, passos);
        }
        return;
    }
    match wet_tool {
        WetTool::Paint => {
            sess.engine.dispatch_pressure_dab_lane_cor(
                li,
                cell_x,
                cell_y,
                b,
                cell_r,
                // O FILME não pesa por comprimento (o máximo de um dab já é a camada): com o peso, o 1.º dab
                // da lane (peso 1) ficava acima dos seguintes (4 ÷ 4,775 a 190 px) — o disco do 1.º toque.
                opacidade
                    * f64::from(if filme_liso { fino } else { peso * fino })
                    * esmaece_fino
                    * f64::from(pigmento_da_marca),
                Some(&sil),
                grain_arg,
                padrao_arg,
                cor_arg,
            );
            if let Some(f) = relevo_dab {
                let contacto = f64::from(d.radius_px / d.stroke_radius_px.max(1e-3));
                super::lote::penteia_o_relevo_do_dab(
                    &mut sess.engine,
                    &f,
                    &relevo,
                    contacto,
                    [centro_x, centro_y],
                    meia,
                    px_cel,
                    &sil,
                    passos_do_pente,
                );
            }
            if lamina {
                sess.engine.pentear_rebordo(
                    centro_x,
                    centro_y,
                    meia,
                    &sil,
                    rumo,
                    rebordo / px_cel,
                    passos_do_pente,
                );
            }
        }
        // Blend keeps the engine's own fixed-hardness stamp (the
        // model's tools ignore the brush silhouette on purpose) — o Grain armado modula a máscara dele
        // (J4 mistura, L-2; sem Grain, a de sempre ao bit).
        WetTool::Blend => sess.engine.dispatch_pressure_dab_lane_blend_grao(
            li,
            cell_x,
            cell_y,
            b,
            cell_r,
            grain
                .as_ref()
                .map(|g| g as ph2d_wet_paint::brush::CellFn<'_>),
        ),
        // Direct tools: per-dab grid ops; the displacement source is
        // THIS lane's previous centre (host-tracked per copy).
        WetTool::Wet | WetTool::Dry | WetTool::Blow | WetTool::Smear => {
            sess.engine.dispatch_pressure_dab_tool(
                wet_tool.engine(),
                cell_x,
                cell_y,
                b,
                cell_r,
                lane_prev.map(|p| {
                    [
                        grid_map::px_to_cell(f64::from(p[0]), ratio),
                        grid_map::px_to_cell(f64::from(p[1]), ratio),
                    ]
                }),
            );
        }
    }
    sess.lanes[li].pos = d.center;
    sess.lanes[li].arco = d.arc_len;
    if gota {
        super::rota_da_lane::solta_a_gota(sess, li);
    }
}
