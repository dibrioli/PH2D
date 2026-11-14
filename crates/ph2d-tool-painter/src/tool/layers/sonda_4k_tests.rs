//! B18.1–B18.3: as sondas da troca, da duplicação e da coleta para o disco a 3840×2160, pelas portas
//! públicas que o painel de camadas usa. `--ignored --nocapture`, `--profile smoke`.

use super::super::*;

const W: u32 = 3840;
const H: u32 = 2160;

fn ms(t0: std::time::Instant) -> f64 {
    t0.elapsed().as_secs_f64() * 1e3
}

fn resumo(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

fn ruido(semente: u32) -> Vec<u8> {
    let mut s = semente | 1;
    (0..(W * H * 4) as usize)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            (s >> 24) as u8
        })
        .collect()
}

fn documento() -> (PainterTool, RtLayerId, RtLayerId) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    let a = t.add_raster_layer_with_pixels("A", ruido(7)).expect("A");
    let b = t.add_raster_layer_with_pixels("B", ruido(11)).expect("B");
    let _ = t.take_preview_arc();
    (t, a, b)
}

/// Trocar A↔B (a chamada e a recomposição do quadro seguinte), duplicar a activa, e o documento vivo que
/// a gravação colhe — mín./mediana de 9 repetições intercaladas.
#[test]
#[ignore = "sonda de tempo"]
fn sonda_trocar_duplicar_e_salvar_camada_a_4k() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    let (mut t, a, b) = documento();
    let (mut troca, mut quadro, mut dup, mut salva) = (vec![], vec![], vec![], vec![]);
    for i in 0..9 {
        let _ = i;
        // A activa é sempre B aqui (a duplicata apagada devolve-a): A e depois B são duas trocas.
        for alvo in [a, b] {
            assert_ne!(t.layers.active(), Some(alvo));
            let t0 = std::time::Instant::now();
            t.select_layer(alvo);
            troca.push(ms(t0));
            let t0 = std::time::Instant::now();
            let _ = t.take_preview_arc();
            quadro.push(ms(t0));
        }
        let alvo = b;
        let t0 = std::time::Instant::now();
        let nova = t.duplicate_layer(alvo).expect("dup");
        dup.push(ms(t0));
        let _ = t.take_preview_arc();
        assert!(t.delete_layer(nova));
        let _ = t.take_preview_arc();
        let t0 = std::time::Instant::now();
        let d = t.documento_vivo(0);
        salva.push(ms(t0));
        drop(d);
    }
    let (tm, tmed) = resumo(&mut troca);
    let (qm, qmed) = resumo(&mut quadro);
    let (dm, dmed) = resumo(&mut dup);
    let (sm, smed) = resumo(&mut salva);
    println!(
        "[B18 4K] trocar {tm:.2}/{tmed:.2} ms | quadro depois da troca {qm:.2}/{qmed:.2} ms | duplicar {dm:.2}/{dmed:.2} ms | documento vivo {sm:.2}/{smed:.2} ms (mín./mediana)"
    );
}

/// B18.3 A/B no mesmo processo: a troca (a chamada + o quadro seguinte) recompondo sempre (antes) e sem
/// recompor quando a figura é provada igual (agora), intercaladas — mín./mediana de 9 de cada.
#[test]
#[ignore = "sonda de tempo"]
fn sonda_troca_recompoe_contra_mantem_a_4k() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    use super::cache::TROCA_RECOMPOE_SEMPRE;
    let (mut t, a, b) = documento();
    let mut m = [[vec![], vec![]], [vec![], vec![]]];
    for i in 0..18 {
        let v = (i + i / 2) % 2; // ABBA: a ordem roda entre os blocos
        TROCA_RECOMPOE_SEMPRE.with(|c| c.set(v == 0));
        let alvo = if t.layers.active() == Some(a) { b } else { a };
        let t0 = std::time::Instant::now();
        t.select_layer(alvo);
        m[v][0].push(ms(t0));
        let t0 = std::time::Instant::now();
        let _ = t.take_preview_arc();
        m[v][1].push(ms(t0));
    }
    TROCA_RECOMPOE_SEMPRE.with(|c| c.set(false));
    for (v, nome) in [(0, "recompõe (antes)"), (1, "mantém (agora)")] {
        let (tm, tmed) = resumo(&mut m[v][0]);
        let (qm, qmed) = resumo(&mut m[v][1]);
        println!(
            "[B18.3 4K] {nome}: trocar {tm:.2}/{tmed:.2} ms | quadro seguinte {qm:.2}/{qmed:.2} ms (mín./mediana)"
        );
    }
}

/// As fases do `set_active_layer` a 4K, uma a uma (mín. de 9 trocas): onde está o custo da troca.
#[test]
#[ignore = "sonda de tempo"]
fn sonda_fases_da_troca_de_camada_a_4k() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    let (mut t, a, b) = documento();
    let mut f = [vec![], vec![], vec![], vec![], vec![], vec![], vec![]];
    for i in 0..10 {
        let alvo = if i % 2 == 0 { a } else { b };
        let t0 = std::time::Instant::now();
        t.fecha_a_forma_aberta();
        let antes = t.snapshot_model();
        f[0].push(ms(t0));
        let t0 = std::time::Instant::now();
        t.flush_active_to_images();
        f[1].push(ms(t0));
        // `carrega_a_tela_da_activa` aberta em duas: a posse do plano e a troca da tela (o diário).
        let t0 = std::time::Instant::now();
        let img = t.images.remove(&alvo).expect("a outra camada tem plano");
        let buf = crate::tool::internal::own_image(img);
        f[2].push(ms(t0));
        let t0 = std::time::Instant::now();
        t.instala_a_tela_do_restauro(buf); // a porta da troca, sem o diário (B18.3)
        t.layers.set_active(alvo);
        f[6].push(ms(t0));
        let t0 = std::time::Instant::now();
        t.commit_structural_edit(antes);
        f[3].push(ms(t0));
        let t0 = std::time::Instant::now();
        t.wet_acorda_da_camada();
        f[4].push(ms(t0));
        let t0 = std::time::Instant::now();
        t.troca_sem_mudar_a_figura();
        f[5].push(ms(t0));
        let _ = t.take_preview_arc();
    }
    let m: Vec<String> = f
        .iter_mut()
        .zip([
            "snapshot", "flush", "posse", "commit", "acorda", "publica", "tela",
        ])
        .map(|(v, n)| {
            let (mi, me) = resumo(v);
            format!("{n} {mi:.2}/{me:.2}")
        })
        .collect();
    println!("[B18.3 fases 4K] {}", m.join(" | "));
}

/// A/B no mesmo processo: a posse de um plano partilhado (o caso da troca: o undo segura o `Arc`) pela
/// cópia serial de antes e pela em blocos, intercaladas — mín./mediana de 15, a UHD e a 4096².
#[test]
#[ignore = "sonda de tempo"]
fn sonda_copia_serial_contra_blocos_a_4k() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    for (w, h) in [(W, H), (4096, 4096)] {
        let img = Arc::new(crate::compositor::LayerImage {
            width: w,
            height: h,
            rgba8: vec![77u8; (w * h * 4) as usize].into(),
        });
        let (mut serie, mut blocos) = (vec![], vec![]);
        for _ in 0..15 {
            let t0 = std::time::Instant::now();
            let c = img.rgba8.as_ref().clone();
            serie.push(ms(t0));
            drop(c);
            let t0 = std::time::Instant::now();
            let c = crate::plane_copy::par_clone(&img.rgba8);
            blocos.push(ms(t0));
            drop(c);
        }
        let (sm, smed) = resumo(&mut serie);
        let (bm, bmed) = resumo(&mut blocos);
        println!(
            "[B18.1 cópia {w}x{h}] serial {sm:.2}/{smed:.2} ms | blocos {bm:.2}/{bmed:.2} ms (mín./mediana)"
        );
    }
}

/// B18.2 A/B no mesmo processo: colher o documento a 4K com relevo, cobertura, material e vidro — o corpo
/// de antes (`par_clone` de cada plano) contra o de agora (`documento_vivo`, partilha), intercalados ABAB,
/// mín./mediana de 9 de cada.
#[test]
#[ignore = "sonda de tempo"]
fn sonda_colher_documento_copia_contra_partilha_a_4k() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    use crate::plane_copy::par_clone;
    let (mut t, a, b) = documento();
    let n = (W * H) as usize;
    t.heights.insert(a, Arc::new(vec![0.5f32; n]));
    t.covers.insert(a, Arc::new(vec![200u8; n]));
    t.mats.insert(a, Arc::new(vec![[3u8, 1, 4, 1, 5, 9, 2]; n]));
    t.vidros
        .insert(b, Arc::new(vec![[2u8, 7, 1, 8, 2, 8, 1]; n]));
    let antes = |t: &PainterTool| {
        par_clone(&t.canvas_rgba).len()
            + t.heights
                .values()
                .map(|v| par_clone(v).len())
                .sum::<usize>()
            + t.covers.values().map(|v| par_clone(v).len()).sum::<usize>()
            + t.mats.values().map(|v| par_clone(v).len()).sum::<usize>()
            + t.vidros.values().map(|v| par_clone(v).len()).sum::<usize>()
    };
    let (mut va, mut vb) = (vec![], vec![]);
    for _ in 0..9 {
        let t0 = std::time::Instant::now();
        std::hint::black_box(antes(&t));
        va.push(ms(t0));
        let t0 = std::time::Instant::now();
        std::hint::black_box(t.documento_vivo(0));
        vb.push(ms(t0));
    }
    let (am, amed) = resumo(&mut va);
    let (bm, bmed) = resumo(&mut vb);
    println!(
        "[B18.2 4K] colher: cópia (antes) {am:.2}/{amed:.2} ms | partilha (agora) {bm:.3}/{bmed:.3} ms (mín./mediana)"
    );
}
