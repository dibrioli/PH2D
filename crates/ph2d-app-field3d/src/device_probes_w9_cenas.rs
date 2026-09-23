//! ⏱️⭐⭐⭐ **AS SONDAS DA `W9` QUE MEDEM UMA CENA** — o que ela custa, e o que a cache do chão lhe
//! poupa.
//!
//! ⚠️ **Elas saíram do [`super::sondas_w9`] por um tecto de LOC** (2026-09-23, `748` contra `700`),
//! e a fronteira é o SUJEITO: o irmão mede o que a **COMPILAÇÃO** custa e o que a fita inerte lhe
//! tirou; estas medem o que uma **CENA** custa — o relógio dela, as peças dela, e o que a cache do
//! chão lhe poupa a orbitar.

use super::super::*;

/// ⏱️ **Sonda: de quanto a cache do campo do chão mente na IMAGEM, em bytes?**
///
/// A cache exclui a ORIENTAÇÃO da chave, e os gates de
/// `ph2d-field-render/src/tests/chao_ricochete.rs` medem que orbitar move o campo `6e-9` — `3` ULP,
/// `50 000×` abaixo de um byte de radiância. ⚠️ **Mas um byte de IMAGEM é outra grandeza**: o campo
/// entra como luz somada e passa pelo sRGB, e um pixel que esteja exactamente na fronteira de
/// arredondamento pode virar. ⇒ *a barra do gate sai desta sonda, não de um palpite.*
#[test]
#[ignore = "sonda de diagnóstico: mede a barra do gate da cache do chão"]
fn diag_a_cache_do_chao_na_imagem() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let doc = crate::smoke::scene(1);
    let reg = crate::smoke::sampled_registry();
    let chao = Some(ph2d_field_render::Ground { height: -1.0 });
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pinta = |azimute: f32, chao_em_cache: bool| {
        let cam = ph2d_field_render::Orbit {
            rotation: ph2d_field_render::Orbit::from_yaw_pitch(azimute, 0.5).rotation,
            ..ph2d_field_render::Orbit::default()
        };
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            chao,
            LW,
            LH,
            true,
            crate::gpu_frame::Sonda {
                chao_em_cache,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor")
    };
    // ⚠️ A lâmpada segue a câmera (`tests_lampada`), logo trocar o azimute troca a LUZ e a cache
    // faltaria de qualquer maneira. Esta sonda mede o que interessa: o MESMO enquadramento, com a
    // cache cheia de um azimute anterior contra uma assadura fresca.
    let _ = pinta(0.0, true);
    let com = pinta(1.0, true);
    let sem = pinta(1.0, false);
    let mut piores = [0u32; 4];
    let mut diferentes = 0usize;
    for (a, b) in com
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(sem.rgba.as_chunks::<4>().0)
    {
        if a != b {
            diferentes += 1;
        }
        for k in 0..4 {
            piores[k] = piores[k].max(u32::from(a[k].abs_diff(b[k])));
        }
    }
    println!(
        "\n  píxeis diferentes: {diferentes} de {} · pior |Δ| por canal: {piores:?}\n",
        com.rgba.len() / 4
    );
}

/// ⏱️⭐⭐⭐⭐ **O QUE A CACHE DO CHÃO COMPRA NO RELÓGIO — o A/B intercalado, a ORBITAR.**
///
/// A dívida nomeada é **`+4,98 ms` por quadro assente** ([`09` §6](../../../docs/Render3d/09_a_cor_que_a_peca_devolve_ao_chao.md))
/// e o gesto que a paga é **orbitar**, porque a assadura corria a cada quadro sobre um campo que
/// não depende de para onde a câmera olha.
///
/// ⚠️ **A régua é a da `W9`**: o mesmo processo, INTERCALADO, mínimo de [`QUADROS_MEDIDOS`], com a
/// ociosidade ao lado — *subtrair dois relógios de corridas separadas dá a soma dos ruídos*. E a
/// lâmpada é **FIXA em mundo**, senão orbitar troca a luz e a cache falta de qualquer maneira (foi
/// o furo da 1.ª redacção do gate irmão).
///
/// ⭐ **O lado COM a cache pinta uma vez antes de ser cronometrado** — o que se quer medir é o
/// quadro que REAPROVEITA, não o que assa.
#[test]
#[ignore = "sonda de diagnóstico: mede o que a cache do chão compra no relógio"]
fn diag_o_relogio_da_cache_do_chao() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    const LUZ: [ph2d_field_render::PointLamp; 1] = [ph2d_field_render::PointLamp {
        world: [1.6, 2.4, 1.2],
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let reg = crate::smoke::sampled_registry();
    let chao = Some(ph2d_field_render::Ground { height: -1.0 });
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    println!("\n  {}", contexto());
    println!("  cena ·  com a cache ·  sem a cache ·  razão");
    for n in [1u32, 2, 30] {
        let doc = crate::smoke::scene(n);
        let pinta = |azimute: f32, chao_em_cache: bool| -> f32 {
            let cam = ph2d_field_render::Orbit {
                rotation: ph2d_field_render::Orbit::from_yaw_pitch(azimute, 0.5).rotation,
                ..ph2d_field_render::Orbit::default()
            };
            let t0 = std::time::Instant::now();
            let saiu = crate::gpu_frame::paint_com(
                t,
                &doc,
                &reg,
                &cam,
                &LUZ,
                &surfaces,
                &ph2d_field_render::Presentation::of(olhar),
                BG,
                chao,
                LW,
                LH,
                true,
                crate::gpu_frame::Sonda {
                    chao_em_cache,
                    ..crate::gpu_frame::Sonda::default()
                },
            );
            assert!(saiu.is_some(), "o pintor recusou a cena {n}");
            #[allow(clippy::cast_possible_truncation)]
            let ms = t0.elapsed().as_secs_f32() * 1e3;
            ms
        };
        // ⚠️ A ORBITAR: cada leitura num azimute diferente, que é o gesto do artista.
        #[allow(clippy::cast_precision_loss)]
        let minimo = |chao_em_cache: bool| {
            if chao_em_cache {
                let _ = pinta(0.0, true);
            }
            (0..QUADROS_MEDIDOS)
                .map(|k| pinta(0.3 + k as f32 * 0.2, chao_em_cache))
                .fold(f32::INFINITY, f32::min)
        };
        let com = minimo(true);
        let sem = minimo(false);
        println!(
            "  {n:>4} · {com:9.2} ms · {sem:9.2} ms · {:5.2}x",
            sem / com.max(1e-3)
        );
    }
    println!();
}

/// ⏱️⭐⭐⭐⭐ **A CENA `28` É A PIOR DO CORPUS — mas ela são QUATRO peças. Uma sozinha custa quanto?**
///
/// Com o vermelho resolvido (`14 de 22`), a `28` é a única cena longe do orçamento: `63,69 ms`
/// contra `16,7`, com `724` instruções, `28` transcendentes, `44` raízes, `95` valores vivos e
/// **`410` passos por acerto** — o dobro da segunda pior.
///
/// ⛔⛔ **Antes de atacar a lei do nó, a pergunta é se o artista alcança este caso:** a cena põe
/// **quatro** nós de toro lado a lado para mostrar que `(p,q)` e `(q,p)` são peças diferentes, e
/// quem modela trabalha **uma** peça. *Uma cena de vitrina não é um caso de produto, e curar a lei
/// pela vitrina é optimizar o que ninguém faz.*
///
/// ⇒ esta sonda mede cada um dos quatro nós **sozinho**, e a cena inteira ao lado.
#[test]
#[ignore = "sonda de diagnóstico: mede um nó de toro sozinho contra a cena de quatro"]
fn diag_um_no_de_toro_sozinho() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let reg = crate::smoke::sampled_registry();
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let um = |winds: u32, loops: u32| {
        let (radius, tube) = (0.20_f32, 0.085_f32);
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::TorusKnot {
                    radius,
                    tube,
                    cord: ph2d_field::knot_cord_ceiling(radius, tube, winds, loops) * 0.85,
                    winds,
                    loops,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("o nó")
    };
    let mede = |doc: &FieldDoc, nome: &str| {
        let cam = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        let minimo = (0..QUADROS_MEDIDOS)
            .map(|_| {
                let t0 = std::time::Instant::now();
                let saiu = crate::gpu_frame::paint(
                    t,
                    doc,
                    &reg,
                    &cam,
                    &luz,
                    &surfaces,
                    &ph2d_field_render::Presentation::of(olhar),
                    BG,
                    None,
                    LW,
                    LH,
                    false,
                );
                assert!(saiu.is_some(), "o pintor recusou {nome}");
                #[allow(clippy::cast_possible_truncation)]
                let ms = t0.elapsed().as_secs_f32() * 1e3;
                ms
            })
            .fold(f32::INFINITY, f32::min);
        // ⭐ Os passos por acerto vêm da CPU, que é quem tem o contador — a mesma régua do gate.
        use std::sync::atomic::Ordering;
        ph2d_field_render::STEP_SAMPLES.store(0, Ordering::Relaxed);
        let g = ph2d_field_render::trace(doc, &reg, &cam, 96, 54);
        let acertos = g.hit.iter().filter(|h| **h).count().max(1);
        #[allow(clippy::cast_precision_loss)]
        let por_acerto =
            ph2d_field_render::STEP_SAMPLES.load(Ordering::Relaxed) as f32 / acertos as f32;
        let campo = ph2d_field_eval::device::DeviceField::new(doc, &reg).expect("a peça");
        let fita = campo.tape_wgsl().expect("a fita");
        println!(
            "  {nome:<22} {minimo:8.2} ms · {:>5} instr · {por_acerto:7.1} passos/acerto",
            fita.source.lines().count()
        );
    };
    println!("\n  {}", contexto());
    for (p, q) in [(2u32, 3u32), (3, 2), (2, 5), (5, 2)] {
        mede(&um(p, q), &format!("o nó ({p},{q}) sozinho"));
    }
    mede(&crate::smoke::scene(28), "a cena 28 inteira");
    println!();
}

/// ⏱️⭐⭐⭐⭐ **QUANTAS PEÇAS TEM CADA CENA? — a pergunta que reenquadra o vermelho inteiro.**
///
/// Com o vermelho verde a `14 de 22`, as `8` cenas que sobram são as de fita grande. ⛔⛔ **E uma
/// cena de smoke deste módulo mostra QUATRO a SEIS variantes lado a lado de propósito** — é assim
/// que ela prova que `(p,q)` e `(q,p)` são peças diferentes. *Quem modela trabalha UMA peça.*
///
/// ⇒ esta sonda conta as FOLHAS de cada cena ao lado da fita dela, e marca as que o gate acusa.
/// Ela é de CPU e não lê relógio: o que ela mede é a POPULAÇÃO.
#[test]
#[ignore = "sonda de diagnóstico: conta as peças de cada cena"]
fn diag_quantas_pecas_tem_cada_cena() {
    let reg = crate::smoke::sampled_registry();
    // As cenas que o gate acusou na leitura calma de 2026-09-22.
    const ACUSADAS: &[u32] = &[5, 6, 11, 14, 27, 28, 30, 32];
    println!("\n  cena · folhas ·  instr · acusada");
    let (mut folhas_ok, mut folhas_ma, mut n_ok, mut n_ma) = (0usize, 0usize, 0usize, 0usize);
    for n in 0..crate::smoke::scenes::CENAS {
        if crate::smoke::scenes::PODADAS.contains(&n) {
            continue;
        }
        let doc = crate::smoke::scene(n);
        let folhas = doc
            .nodes()
            .iter()
            .filter(|no| matches!(no.kind, ph2d_field::NodeKind::Leaf(_)))
            .count();
        let instr = ph2d_field_eval::device::DeviceField::new(&doc, &reg)
            .and_then(|c| c.tape_wgsl())
            .map_or(0, |f| f.source.lines().count());
        let acusada = ACUSADAS.contains(&n);
        if acusada {
            folhas_ma += folhas;
            n_ma += 1;
        } else {
            folhas_ok += folhas;
            n_ok += 1;
        }
        println!(
            "  {n:>4} · {folhas:>6} · {instr:>6} · {}",
            if acusada { "⛔" } else { "" }
        );
    }
    #[allow(clippy::cast_precision_loss)]
    let media = |soma: usize, n: usize| soma as f32 / n.max(1) as f32;
    println!(
        "\n  ⇒ as {n_ma} ACUSADAS têm em média {:.1} folhas · as {n_ok} nítidas têm {:.1}\n",
        media(folhas_ma, n_ma),
        media(folhas_ok, n_ok)
    );
}
