//! **O DAB que entra na água** — o corpo do laço de [`super::dab_route`] (filho de [`super`], LOC cap).
//!
//! A rota apura uma vez o que vale para o LOTE inteiro ([`LoteDoDab`]); aqui cada dab do lote vira lane,
//! silhueta, grão, cor e a porta do motor que o pousa. A lane e o peso vivem em [`a_lane`]; a porta em
//! [`despacho`].

use super::*;
use oraculos::{com_corte_fora, com_memo_base};

mod a_lane;
mod corte_das_linhas;
mod despacho;
#[cfg(test)]
#[path = "pousa_o_dab_memo_tests.rs"]
mod memo_tests;
mod oraculos;

pub(super) use super::lote_do_dab::LoteDoDab;

/// Um dab do lote (`didx` o índice na mão, `sub_k` o sub-dab da cápsula, `passo` o sub-passo) pousa na água.
pub(super) fn pousa_o_dab(
    l: &LoteDoDab,
    sess: &mut WetSession,
    dab_rng: &mut super::super::tiling::DabRng,
    didx: usize,
    sub_k: u64,
    passo: Option<f32>,
    d: &Dab,
) {
    let &LoteDoDab {
        ref brush,
        erasing,
        opacidade_da_pressao,
        ref dual,
        ratio,
        wet_tool,
        ref rampa,
        padrao_vira_cor,
        grao_e_cor,
        cerdas,
        marca_do_spray,
        filme_liso,
        fios,
        relevo,
        lamina,
        shape_active,
        grain_active,
        ref groups,
        canvas_wh,
        strength,
        escala_tex,
        grao_e_o_papel,
        media_depois_do_portao,
        ponta_sob_o_grao,
        ganho_do_grao,
        sem_grao,
        escala_cerdas,
        pontas,
        alem,
        esbate_e,
        lote,
        ..
    } = l;
    let (shape_image, grain_image) = (l.shape_mask(), l.grain_mask());
    let shape_ramp_lut = l.shape_ramp_lut.as_deref();
    let [x, y] = d.center;
    let lane = a_lane::casa(l, sess, didx, d);
    let (li, lane_prev) = (lane.li, lane.lane_prev);
    // `Some(None)` = o dab sem rumo com as pontas rectas: não pousa (a lane anda na mesma).
    let tampa = pontas.then(|| super::tampa::Tampa::do_dab(d, &mut sess.lanes[li], lote, alem));
    let corte = tampa.flatten();
    let esbate = (esbate_e != 0.0).then(|| {
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
    let (peso, passos_do_pente) = a_lane::pesos(l, sess, &lane, d);
    let fp = spec.dab_footprint_of(d);
    let dab_index = didx;
    let tex_rng = dab_rng.enter(groups, dab_index);
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
    // A contagem da sonda (B14c), lida 1× por dab: desligada, as leituras da silhueta não pagam a carga por célula.
    let conta = crate::wet_diag::CONTA_SIL.load(std::sync::atomic::Ordering::Relaxed);
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
    // B14c-cont3: sem ponta de imagem e com o falloff nulo além de `t = 1`, a célula inteira (o quadrado dela mais
    // `folga` px por eixo) com `t ≥ 1` dá `0` sem as sub-amostras — o `t` minorado pelo Lipschitz da pegada, ao bit.
    let lado = aa_step * f32::from(aa_n);
    let lips = fp_raio
        .falloff_t_lipschitz()
        .filter(|_| com_corte_fora() && sil_input.is_none() && spec.falloff_weight(1.0) == 0.0);
    // `desliza`: o que um deslocamento conhecido soma a `t` (a métrica da pegada na direcção dele — `falloff_t` é a
    // norma de `apply`, linear com a curva plana).
    let t_do_centro = move |cx: i32, cy: i32| -> f32 {
        let sx = grid_map::cell_subsample_px(cx, ratio, 0, lado);
        let sy = grid_map::cell_subsample_px(cy, ratio, 0, lado);
        fp_raio.falloff_t((sx - d.center[0]) * inv_r, (sy - d.center[1]) * inv_r)
    };
    // A margem de quem corta com `folga` px por eixo, contada 1× por dab (era por célula): `t − margem − desliza`.
    let margem =
        move |folga: f32| lips.map(|l| l * (0.5 * lado + folga) * std::f32::consts::SQRT_2 * inv_r);
    let fora_t = move |t: f32, margem: Option<f32>, desliza: f32| -> bool {
        margem.is_some_and(|a| t - a - desliza >= 1.001)
    };
    let fora = move |cx: i32, cy: i32, margem: Option<f32>, desliza: f32| -> bool {
        margem.is_some() && fora_t(t_do_centro(cx, cy), margem, desliza)
    };
    let margem0 = margem(0.0);
    // Um sub-ponto do AA em `(sx, sy)` com o seu `t`.
    let amostra = |sx: f32, sy: f32, t: f32| -> f32 {
        let v = ph2d_painter_brush::dab::silhouette_at(
            &spec,
            sil_input,
            t,
            sx.floor() as i64,
            sy.floor() as i64,
            d.center,
            d.radius_px,
        );
        match corte {
            Some(c) => v * c.cobre(sx, sy),
            None => v,
        }
    };
    let sil = |cx: i32, cy: i32| -> f64 {
        let conta_aa = || {
            if conta {
                crate::wet_diag::conta_sil(2, 1);
                crate::wet_diag::conta_sil(3, u64::from(aa_n) * u64::from(aa_n));
            }
        };
        let mut acc = 0.0f32;
        if aa_n == 1 {
            // O único sub-ponto do AA é o centro de `fora` (`lado = aa_step`): o mesmo ponto e o mesmo `t`, contados 1×.
            let sx = grid_map::cell_subsample_px(cx, ratio, 0, aa_step);
            let sy = grid_map::cell_subsample_px(cy, ratio, 0, aa_step);
            let t = fp_raio.falloff_t((sx - d.center[0]) * inv_r, (sy - d.center[1]) * inv_r);
            if fora_t(t, margem0, 0.0) {
                return 0.0;
            }
            conta_aa();
            acc += amostra(sx, sy, t);
        } else {
            if fora(cx, cy, margem0, 0.0) {
                return 0.0;
            }
            conta_aa();
            for j in 0..aa_n {
                let sy = grid_map::cell_subsample_px(cy, ratio, j, aa_step);
                for i in 0..aa_n {
                    let sx = grid_map::cell_subsample_px(cx, ratio, i, aa_step);
                    let t =
                        fp_raio.falloff_t((sx - d.center[0]) * inv_r, (sy - d.center[1]) * inv_r);
                    acc += amostra(sx, sy, t);
                }
            }
        }
        if conta && acc == 0.0 {
            crate::wet_diag::conta_sil(5, 1);
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
    // O fio de cada célula, achado 1× e relido pela ponta, pelas cerdas e pela cor dos fios (a mesma `FiosNoDab`: com
    // `fios_dab`, a da ponta é ela). Por linha, a última célula ([`super::sil_memo::FioPorLinha`]), ao bit.
    let fio_linha = fios_dab.map(|_| {
        let m =
            meia[0].max(meia[1]) + ponta.as_ref().map_or(0.0, |p| p.alcance_px()) / px_cel + 2.0;
        super::sil_memo::FioPorLinha::nova(cell_y, m)
    });
    let fio_linha = fio_linha.as_ref();
    // A bilinear da ponta lê a silhueta BASE em quatro células por célula: guardada à parte (B14c), cada célula da
    // janela do motor deslocada do maior desvio é avaliada UMA vez, ao byte.
    let memo_base = ponta.as_ref().filter(|_| com_memo_base()).map(|p| {
        let m = cell_r * sess.engine.silhouette_reach[0].max(1.0) + p.alcance_px() / px_cel + 2.0;
        super::sil_memo::SilMemo::nova_base(cell_x, cell_y, [m, m])
    });
    let sil_base = |cx: i32, cy: i32| -> f64 {
        memo_base
            .as_ref()
            .map_or_else(|| sil(cx, cy), |m| m.ou(cx, cy, sil))
    };
    // A ponta que ESFARELA (W3 K3b): com um grão PRESO À TELA armado, o desvio de cada célula é o do grão, não o
    // do fio — a ponta acaba em grumos. Sem grão `Tiled`, a desfiada de sempre, ao bit.
    let esfarela = grain_basis.as_ref().filter(|_| {
        spec.texture.mapping == ph2d_painter_brush::texture::TextureMapping::Tiled
            && !spec.texture.flow
    });
    let grao_img_ponta = grain_image.as_ref();
    let sil_da_ponta = |cx: i32, cy: i32| match (ponta.as_ref(), esfarela) {
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
            p.recorta_pelo_grao(&sil_base, cx, cy, f64::from(u))
        }
        (Some(p), None) => match fio_linha {
            Some(m) => {
                p.recorta_no_fio(&sil_base, cx, cy, m.ou(cx, cy, || p.fio_da_celula(cx, cy)))
            }
            None => p.recorta(&sil_base, cx, cy),
        },
        (None, _) => sil(cx, cy),
    };
    let sil = |cx: i32, cy: i32| {
        if conta {
            crate::wet_diag::conta_sil(1, 1);
        }
        sil_da_ponta(cx, cy)
    };
    // O depósito, o pente e o grão que lê a ponta leem a MESMA silhueta (com a ponta), avaliada UMA vez por célula
    // e guardada no dab ([`super::sil_memo`]), ao byte.
    let memo =
        (relevo_dab.is_some() || lamina || grain_active && spec.texture.lei.mode.le_a_ponta())
            .then(|| super::sil_memo::SilMemo::nova(cell_x, cell_y, meia));
    // A ponta lê a base no ponto deslocado `|s| ≤ alcance` AO LONGO do traço e, pela bilinear, até 1 célula por eixo
    // dele: a folga isotrópica é essa célula, o deslocamento entra pela métrica da pegada nessa direcção (a fita do
    // Flat 240 deitada no traço: o Lipschitz isotrópico × alcance cobria a caixa inteira e o corte nunca disparava).
    let (folga_ponta, desliza) = ponta.as_ref().map_or((0.0, 0.0), |p| {
        let n = relevo_dab.or(fios_dab).expect("a ponta tem fios").normal();
        let a = p.alcance_px() as f32 * 1.001 * inv_r;
        (
            lado,
            fp_raio.falloff_t(n[1] as f32 * a, -n[0] as f32 * a) * 1.001,
        )
    });
    // O corte do motor por LINHA ([`corte_das_linhas`]): fora do intervalo e no miolo, sem o `falloff_t`, ao bit.
    let corte_motor = lips.map(|l| {
        let a = f64::from(l)
            * (0.5 * f64::from(lado) + f64::from(folga_ponta))
            * std::f64::consts::SQRT_2;
        let limiar = 1.001 + f64::from(desliza) + a * f64::from(inv_r);
        corte_das_linhas::CorteDasLinhas::nova(&fp_raio, limiar, d.center, inv_r, ratio, lado)
    });
    let margem_ponta = margem(folga_ponta);
    let fora_motor = |cx: i32, cy: i32| -> bool {
        corte_motor
            .as_ref()
            .and_then(|c| c.decide(cx, cy))
            .unwrap_or_else(|| fora(cx, cy, margem_ponta, desliza))
    };
    // Com a memo, ela guarda também o `0` de `fora`: a 2.ª leitura da célula (o depósito, o pente, o grão) não paga o
    // corte. A contagem do motor continua a das leituras não cortadas (`v ≠ 0` só sai de quem passou o corte).
    let sil = |cx: i32, cy: i32| -> f64 {
        let Some(m) = memo.as_ref() else {
            if fora_motor(cx, cy) {
                return 0.0;
            }
            if conta {
                crate::wet_diag::conta_sil(0, 1);
            }
            return sil(cx, cy);
        };
        let v = m.ou(
            cx,
            cy,
            |cx, cy| {
                if fora_motor(cx, cy) { 0.0 } else { sil(cx, cy) }
            },
        );
        if conta && (v != 0.0 || !fora_motor(cx, cy)) {
            crate::wet_diag::conta_sil(0, 1);
        }
        v
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
            let fio = |q: [f64; 2]| f.fio_no_ponto(q.map(|v| v * px_cel));
            let fio = fio_linha.map_or_else(|| fio(dc), |m| m.ou(cx, cy, || fio(dc)));
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
            ganho_do_grao
                * f64::from(ph2d_painter_brush::dab::grain_at_na_ponta(
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
    let cerdas_sb = shape_basis.as_ref().filter(|_| {
        cerdas && (grain.is_none() || ponta_sob_o_grao) && !filme_liso && !marca_do_spray
    });
    let cerdas_fn = cerdas_sb.map(|sb| {
        let forma = &spec_ref.shape;
        let px_por_celula = f64::from(ratio);
        // O que é do dab sai 1× (o `bristles_at` refazia-o por célula, com o azulejo num `thread_local`).
        let cd =
            ph2d_painter_brush::texture::CerdasDoDab::nova(forma, sb, px_por_celula, d.radius_px);
        move |cx: i32, cy: i32| -> f64 {
            let (fx, fy) = (f64::from(cx), f64::from(cy));
            let dc = [fx - centro_x, fy - centro_y];
            let tapete = |q: [f64; 2]| cd.em(q, [fx, fy]);
            let Some(f) = fios_dab.as_ref() else {
                return tapete(dc);
            };
            let q = [dc[0] * px_por_celula, dc[1] * px_por_celula];
            let p = match (tabela, fio_linha) {
                (Some(t), Some(m)) => f.no_ponto_no_fio(q, m.ou(cx, cy, || f.fio_no_ponto(q)), t),
                (Some(t), None) => f.no_ponto_em(q, t),
                (None, _) => f.no_ponto(q),
            };
            p.ganho * tapete([p.q[0] / px_por_celula, p.q[1] / px_por_celula])
        }
    });
    let texv_liso = sem_grao.map_or(1.0, |[g, _]| g);
    let lisa = move |_: i32, _: i32| -> f64 { texv_liso };
    // Filme liso: o grão continua a ser o `texv`, como o de sempre.
    let ponta_sob_o_grao = ponta_sob_o_grao && !filme_liso;
    let grain_arg: Option<ph2d_wet_paint::brush::CellFn<'_>> =
        Some(match (grain.as_ref(), cerdas_fn.as_ref()) {
            (None, Some(_)) if sem_grao.is_some() => &lisa,
            (Some(_), Some(c)) => c,
            (Some(_), _) if grao_e_cor || ponta_sob_o_grao => &lisa,
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
    // Grão (de escolha ou o papel) sobre a ponta aplicada sem Grain (Bug #61): a ponta continua a dar a quantidade e o
    // grão desenha depois do portão, de média 1 — o tapete tem média ~0,02 e trocá-lo pelo grão pousava até 20× a tinta.
    let grao_depois = grain
        .as_ref()
        .filter(|_| ponta_sob_o_grao)
        .zip(media_depois_do_portao)
        .map(|(g, media)| {
            move |cx: i32, cy: i32| -> f64 {
                g(cx, cy) / media * padrao_arg.map_or(1.0, |p| p(cx, cy))
            }
        });
    let padrao_arg = grao_depois
        .as_ref()
        .map_or(padrao_arg, |p| Some(p as ph2d_wet_paint::brush::CellFn<'_>));
    // O Grain tirado: as cerdas desenham depois do portão, de média 1, sobre o `texv` à média dele.
    let cerdas_depois = cerdas_fn.as_ref().zip(sem_grao).map(|(c, [_, media])| {
        move |cx: i32, cy: i32| -> f64 { c(cx, cy) / media * padrao_arg.map_or(1.0, |p| p(cx, cy)) }
    });
    let padrao_arg = cerdas_depois
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
    // O ARCO dos tufos (o leque, `bristles_arco`): no referencial da pegada (a rotação da ponta), depois do portão.
    let tun = &sess.engine.tuning;
    let arco = ph2d_wet_paint::bristles_arco::Arco {
        curvatura: tun.get(ph2d_wet_paint::tuning::Knob::TuftArc),
    };
    let pressao_arco = ph2d_wet_paint::bristles_arco::Arco::pressao_do_contacto(
        contacto,
        tun.get(ph2d_wet_paint::tuning::Knob::PressureSize),
    );
    let padrao_arco = (!arco.neutro()).then_some(move |cx: i32, cy: i32| -> f64 {
        let k = px_cel / f64::from(d.radius_px.max(0.01));
        let u = [f64::from(cx) - centro_x, f64::from(cy) - centro_y].map(|v| (v * k) as f32);
        let f = fp.apply(u).map(f64::from);
        arco.no_ponto(f, pressao_arco) * padrao_arg.map_or(1.0, |p| p(cx, cy))
    });
    let padrao_arg = padrao_arco
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
        despacho::anda_a_lane(l, sess, li, d);
        return;
    }
    despacho::despacha(
        l,
        sess,
        d,
        despacho::Despacho {
            li,
            cell: [cell_x, cell_y],
            b,
            cell_r,
            opacidade,
            peso,
            sil: &sil,
            grain_arg,
            padrao_arg,
            cor_arg,
            grain: grain.as_ref(),
            cerdas_fn: cerdas_fn.as_ref(),
            ponta_sob_o_grao,
            relevo_dab,
            meia,
            px_cel,
            passos_do_pente,
            rumo,
            lane_prev,
            passo,
        },
    );
}
