//! SONDA W14 (`docs/Macros/handoffs/PROMPT_W14_a_agua_depois_do_parar_2026-10-20.md`): o salto da água
//! no fim do clipe. O último quadro do clipe é a tela no Parar; a imagem do artista é a água que correu
//! depois. Mede a diferença entre os dois em vários instantes depois do Parar, ao lado do que a água muda
//! num quadro (o movimento que o olho já vê a correr).

use super::em_cena::MacroEmCena;
use super::roteiro::grava_no_papel_ligado;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::Tool;

fn tique(t: &mut PainterTool) {
    drop(t.take_preview_arc());
    t.on_tick(16.667);
}

/// `(texels que diferem, > 2 níveis, > 8 níveis, > 32 níveis, o maior nível)`.
fn salto(a: &[u8], b: &[u8]) -> (usize, usize, usize, usize, u8) {
    let mut r = (0, 0, 0, 0, 0u8);
    for (x, y) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
        let d = x.iter().zip(y).map(|(p, q)| p.abs_diff(*q)).max().unwrap_or(0);
        if d > 0 {
            r.0 += 1;
        }
        if d > 2 {
            r.1 += 1;
        }
        if d > 8 {
            r.2 += 1;
        }
        if d > 32 {
            r.3 += 1;
        }
        r.4 = r.4.max(d);
    }
    r
}

fn pintados(a: &[u8]) -> usize {
    a.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..3] != [255, 255, 255])
        .count()
}

#[test]
#[ignore = "sonda W14: corre-se por nome (--ignored --nocapture)"]
fn sonda_o_salto_no_fim_do_clipe() {
    for (lado, n) in [(1024u32, 2usize), (1024, 4)] {
        let (mut t, bytes, dur) = grava_no_papel_ligado(lado, 1, n).expect("grava");
        // A espera de uma etapa editada (W8 2b): o fio da imagem do fim nova.
        let track = ph2d_macro::codec::read(&bytes).expect("lê").tracks.remove(0);
        let t0 = std::time::Instant::now();
        let _ = super::etapas::imagens_da_gravacao(&track, &std::sync::atomic::AtomicBool::new(false));
        println!(
            "[w14] {lado}² {n} pinceladas: a imagem do fim nova leva {:.1} s (load {})",
            t0.elapsed().as_secs_f64(),
            std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim()
        );
        let mut palco = MacroEmCena::de(&bytes).expect("lê").expect("trilha");
        palco.segue(dur).expect("toca");
        let ultimo = palco.toma_a_tela().expect("tela").0.to_vec();
        let no_parar = t.imagem_aplicada().expect("doc").0;
        println!(
            "[w14] {lado}² {n} pinceladas, clipe {dur:.2} s: pintados={} | último quadro ≠ tela no Parar: {:?}",
            pintados(&no_parar),
            salto(&ultimo, &no_parar)
        );
        let marcos = [60u32, 120, 312, 600, 1200, 2400];
        let mut anterior = no_parar.clone();
        let (mut q, mut maior_quadro, mut soma_quadro) = (0u32, (0usize, 0u8), 0usize);
        while !t.wet_assentada() && q < 40_000 {
            tique(&mut t);
            q += 1;
            let agora = t.imagem_aplicada().expect("doc").0;
            let d = salto(&anterior, &agora);
            soma_quadro += d.0;
            if d.4 > maior_quadro.1 || (d.4 == maior_quadro.1 && d.0 > maior_quadro.0) {
                maior_quadro = (d.0, d.4);
            }
            if marcos.contains(&q) {
                println!(
                    "[w14]   +{:.1} s: último quadro ≠ editor {:?} | a água num quadro aqui {:?}",
                    f64::from(q) / 60.0,
                    salto(&ultimo, &agora),
                    d
                );
            }
            anterior = agora;
        }
        let fim = t.imagem_aplicada().expect("doc").0;
        println!(
            "[w14]   assentada em {q} quadros ({:.1} s): último quadro ≠ assentada {:?} | a água num \
             quadro: o maior {maior_quadro:?}, média {} texels",
            f64::from(q) / 60.0,
            salto(&ultimo, &fim),
            soma_quadro / q.max(1) as usize
        );
    }
}
