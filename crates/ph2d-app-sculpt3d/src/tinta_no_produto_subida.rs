//! 🔎 **SONDA (não é gate) — o relógio da subida INCREMENTAL com relevo** (doc
//! [`29`](../../../docs/3D/29_plano_o_relevo_do_impasto_na_peca.md) §4: *«se a
//! subida incremental das alturas custar mais de `1 ms` por quadro durante um
//! traço na peça de fábrica, a W2 não fecha nessa forma»*). Filho (`#[path]`)
//! do arnês do Painter, pelo tecto de LOC.
//!
//! Mede a PORTA (`upload_tinta_amostras_at`) e não o quadro: o quadro inteiro já
//! tem a sonda dele (`diag_o_preco_de_pintar_em_cada_degrau`), e o que o
//! critério pergunta é o preço que o RELEVO acrescenta à subida. Duas partes:
//! as escritas na fila (CPU) e o `submit` + espera que as leva à placa.

use super::super::cena_52;
use super::painter_vermelho;
use crate::painter_na_malha::{entrega, quadro};
use ph2d_editor_core::tool::PointerPhase;
use ph2d_tool_painter::PaintMedia;
use std::time::Instant;

#[test]
#[ignore = "sonda: precisa de adaptador e imprime a tabela"]
fn diag_o_relogio_da_subida_com_relevo() {
    let gpu = gpu_or_skip!();
    for (nome, meio) in [
        ("digital", PaintMedia::Digital),
        ("impasto", PaintMedia::Impasto),
    ] {
        let mut s = cena_52(&gpu.device);
        s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1400.0, 900.0));
        s.sync_mesh(&gpu);
        let mut p = painter_vermelho();
        p.set_paint_media(meio);
        p.set_brush_size_px(28.0);
        quadro(Some(&mut s), Some(&mut p));
        assert!(entrega(
            &mut s,
            &mut p,
            420.0,
            330.0,
            1.0,
            PointerPhase::Down
        ));
        let mut fila = Vec::new();
        let mut poe = Vec::new();
        let mut sujas_por_quadro = Vec::new();
        for j in 1..=60u16 {
            let t = f32::from(j) / 60.0;
            let (x, y) = (420.0 + 300.0 * t, 330.0 + 40.0 * (t * 5.0).sin());
            entrega(&mut s, &mut p, x, y, 1.0, PointerPhase::Move);
            quadro(Some(&mut s), Some(&mut p));
            let mut sujas = Vec::new();
            let subiu = match s.stroke.tinta_fina.as_mut() {
                Some(fina) => {
                    fina.drena_sujas(&mut sujas);
                    let n = sujas.len();
                    let t0 = Instant::now();
                    let ok = s.renderer.upload_tinta_amostras_at(
                        &gpu.queue,
                        0,
                        s.objects[s.active].stack.mesh(),
                        fina.tinta(),
                        &mut sujas,
                    );
                    let escrever = t0.elapsed();
                    let t1 = Instant::now();
                    gpu.queue.submit([]);
                    gpu.device
                        .poll(wgpu::PollType::wait_indefinitely())
                        .expect("poll");
                    ok.then(|| (n, escrever, t1.elapsed()))
                }
                None => None,
            };
            // A 1.ª vez que o relevo aparece pede a subida INTEIRA (a porta
            // devolve `false`): essa fica fora da régua e o `sync_mesh` a faz.
            if let Some((n, e, q)) = subiu {
                sujas_por_quadro.push(n);
                fila.push(e.as_secs_f64() * 1e3);
                poe.push(q.as_secs_f64() * 1e3);
            } else {
                s.objects[s.active].tinta_suja = true;
            }
            s.sync_mesh(&gpu);
        }
        entrega(&mut s, &mut p, 720.0, 330.0, 1.0, PointerPhase::Up);
        let relevo = s.objects[s.active]
            .tinta
            .as_ref()
            .is_some_and(|t| t.tem_relevo());
        let ord = |v: &mut Vec<f64>| v.sort_by(f64::total_cmp);
        ord(&mut fila);
        ord(&mut poe);
        let q = |v: &[f64], f: f64| v[((v.len() - 1) as f64 * f) as usize];
        let n_total = s.objects[s.active]
            .tinta
            .as_ref()
            .map_or(0, |t| t.amostras().len());
        eprintln!(
            "{nome}: relevo={relevo} · {n_total} amostras · {} quadros medidos · sujas/quadro mediana {} máx {} · \
             escrever na fila mediana {:.3} ms p95 {:.3} máx {:.3} · submit+espera mediana {:.3} ms p95 {:.3} máx {:.3}",
            fila.len(),
            {
                let mut v = sujas_por_quadro.clone();
                v.sort_unstable();
                v[v.len() / 2]
            },
            sujas_por_quadro.iter().max().copied().unwrap_or(0),
            q(&fila, 0.5),
            q(&fila, 0.95),
            q(&fila, 1.0),
            q(&poe, 0.5),
            q(&poe, 0.95),
            q(&poe, 1.0),
        );
    }
}

/// 🔎 **SONDA (não é gate) — o preço de refazer TODAS as inclinações** (doc 29
/// §9): é o que a subida inteira paga, e a subida inteira corre em todo quadro
/// em que a peça muda de forma com relevo. Por degrau da tinta, na peça da
/// cena `=52`, com um relevo em todas as amostras.
#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_o_preco_de_refazer_as_inclinacoes() {
    let gpu = gpu_or_skip!();
    for k in [3u8, 4, 5, 6] {
        let mut s = cena_52(&gpu.device);
        s.tinta_nivel = Some(k);
        s.sync_mesh(&gpu);
        let o = &mut s.objects[s.active];
        let mesh = o.stack.mesh().clone();
        let Some(t) = o.tinta.as_mut() else { continue };
        for (i, r) in t.relevo_mut().iter_mut().enumerate() {
            *r = [(i % 7) as f32 * 1e-3, 1.0];
        }
        let n = t.amostras().len();
        let mut pior = 0.0f64;
        let mut soma = 0.0f64;
        for _ in 0..5 {
            let t0 = Instant::now();
            let inc = ph2d_mesh_colors::Inclinacoes::nova(
                t,
                |f| mesh.faces()[f].verts(),
                mesh.positions(),
            );
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            std::hint::black_box(inc);
            pior = pior.max(ms);
            soma += ms;
        }
        // A subida INTEIRA pelo renderizador: a 1.ª refaz tudo (o relevo é
        // novo para o device), a 2.ª reaproveita a foto sem nada mudado, e a
        // 3.ª depois de um «dab» que moveu 40 vértices — o quadro de esculpir.
        let plano = t.clone();
        // Dois «dabs»: um de pincel (os vértices a menos de `0,3` de um ponto
        // do equador) e um GRANDE (os 40 primeiros, o polo e os anéis dele).
        let perto: Vec<usize> = (0..mesh.positions().len())
            .filter(|&v| {
                let p = mesh.positions()[v];
                (p[0] - 1.0).powi(2) + p[1].powi(2) + p[2].powi(2) < 0.09
            })
            .collect();
        let mexe = |vs: &mut dyn Iterator<Item = usize>| {
            let mut m = mesh.clone();
            for v in vs {
                m.positions_mut()[v][2] += 0.01;
            }
            m
        };
        let pincel = mexe(&mut perto.iter().copied());
        let movida = mexe(&mut (0..40));
        let mut sobe = |m: &ph2d_mesh::Mesh| {
            let t0 = Instant::now();
            s.renderer
                .upload_tinta_at(&gpu.device, &gpu.queue, 0, m, Some(&plano));
            t0.elapsed().as_secs_f64() * 1e3
        };
        let (nova, igual, dab, grande) = (sobe(&mesh), sobe(&mesh), sobe(&pincel), sobe(&movida));
        // A referência: a MESMA subida de um plano sem relevo — o que o quadro
        // de esculpir já pagava antes das inclinações existirem.
        let mut liso = plano.clone();
        liso.com_relevo(None);
        let t0 = Instant::now();
        s.renderer
            .upload_tinta_at(&gpu.device, &gpu.queue, 0, &pincel, Some(&liso));
        let sem_relevo = t0.elapsed().as_secs_f64() * 1e3;
        let media = soma / 5.0;
        let nv = perto.len();
        eprintln!(
            "k={k} ({}x): {n} amostras · refazer as inclinações média {media:.2} ms · pior {pior:.2} ms · \
             subida inteira: 1.ª {nova:.2} ms · sem mudança {igual:.2} ms · dab de pincel ({nv} vértices) {dab:.2} ms · \
             dab grande (40) {grande:.2} ms · a mesma subida SEM relevo {sem_relevo:.2} ms",
            1u32 << k,
        );
    }
}
