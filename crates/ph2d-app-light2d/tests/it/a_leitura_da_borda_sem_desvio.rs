//! ⭐ W6h §13 — a leitura do arquivo da borda SEM desvio (o `de_fora` lê sempre e escolhe depois) NÃO muda um bit:
//! o produto × a leitura de antes (dentro do `if`, pelo `Config::modulo_rc`), a vista com o entorno, a cena 5. As
//! vistas do portão do T nulo (as que apanharam as formas que desfaziam a FMA, W6g §53) e a 2160p Alta (onde a
//! cura paga: −3,0 ms de `c2`).

use bevy_ecs::world::World;
use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::teto::{Tetos, fina_que_cabe};
use ph2d_app_light2d::{
    config_da_vista, config_do_entorno, grade_do_quadro, regiao_da_janela, smoke,
};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Config, Solver};
use ph2d_render::Camera2d;

use crate::comum::{fluencia_lida, gpu};

/// A leitura do produto e a de antes (W6g §32).
pub(crate) const SEM_DESVIO: &str = "        var i = FORA_DA_JANELA;
        if s >= 0.0 && s < xe.y {
            i = indice_na_janela(m, u32(coluna), u32(s), pai);
        }
        let ok = i != FORA_DA_JANELA;
        let lido = desempacotar(arquivo_dos_r[hrcs[iq()].fora_base[m / 4u][m % 4u] + select(0u, i, ok)]).r;
        let r = select(ceu, lido, ok);";
pub(crate) const COM_DESVIO: &str = "        var r = ceu;
        if s >= 0.0 && s < xe.y {
            let i = indice_na_janela(m, u32(coluna), u32(s), pai);
            if i != FORA_DA_JANELA {
                r = desempacotar(arquivo_dos_r[hrcs[iq()].fora_base[m / 4u][m % 4u] + i]).r;
            }
        }";

/// O módulo do produto com `de` trocado por `para` (uma vez).
pub(crate) fn trocado(c: Config, de: &str, para: &str) -> Config {
    let fonte =
        ph2d_light2d_gpu::fonte_do_rc(c.aritmetica, c.armazenamento, c.hrc_niveis_em_textura());
    assert_eq!(fonte.matches(de).count(), 1, "a agulha: {de}");
    Config {
        modulo_rc: Some(Box::leak(fonte.replacen(de, para, 1).into_boxed_str())),
        ..c
    }
}

#[test]
fn a_leitura_da_borda_sem_desvio_nao_muda_um_bit() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — saltado");
        return;
    };
    // `PH2D_SEM_DESVIO_MUTA=<de>⇒<para>`: a prova de mutação (troca no PRODUTO, a de antes fica).
    let muta = std::env::var("PH2D_SEM_DESVIO_MUTA").ok();
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let cena = cena_de_luz(&w);
    let casos = [
        (NivelDaLuz::Baixa, [-4.2, -1.8], (1920, 1080)),
        (NivelDaLuz::Alta, [-4.2, -1.8], (1920, 1080)),
        (NivelDaLuz::Media, [-3.2, -1.8], (2560, 1440)),
        (NivelDaLuz::Baixa, [-3.2, -1.8], (3840, 2160)),
        (NivelDaLuz::Alta, [-4.2, -1.8], (3840, 2160)),
    ];
    for (nivel, centro, alvo) in casos {
        let a = Camera2d::new(centro, 3.8).world_to_screen_affine(WindowSize::new(alvo.0, alvo.1));
        let tetos = Tetos {
            lado: gpu.device.limits().max_texture_dimension_2d,
            recusou: None,
        };
        let k = fina_que_cabe(a, alvo, degrau_do(nivel), tetos).expect("cabe");
        let d = degrau_do(nivel).com_fina_engrossada(k);
        let g = grade_do_quadro(a, alvo, d).expect("grade");
        let luz = |antes: bool| {
            let forma = |c: Config| match (&muta, antes) {
                (_, true) => trocado(c, SEM_DESVIO, COM_DESVIO),
                (Some(m), false) => {
                    let (de, para) = m.split_once('⇒').expect("<de>⇒<para>");
                    trocado(c, de, para)
                }
                (None, false) => c,
            };
            let cv = forma(config_da_vista(&gpu.device, g, a, d));
            let ce = forma(config_do_entorno(&gpu.device, a, alvo, d).expect("o entorno"));
            assert!(ce.arquivo_da_borda.is_some(), "sem o arquivo não há borda");
            let mut s = Solver::novo(&gpu, cv).expect("vista");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
            s.regiao_fina(i, f);
            let mut e = Solver::novo(&gpu, ce).expect("entorno");
            e.regiao_fina([0; 2], [0; 2]);
            assert!(e.mascarar_miolo(
                &gpu,
                Some([
                    g.origem_m[0],
                    g.origem_m[1],
                    g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                    g.origem_m[1] + f64::from(g.altura) * g.texel_m,
                ])
            ));
            for _ in 0..30 {
                e.quadro(&gpu, &cena);
                assert!(s.somar_entorno(&gpu, Some(&e)));
                s.quadro(&gpu, &cena);
            }
            fluencia_lida(&gpu, &s).v
        };
        let (agora, antes) = (luz(false), luz(true));
        let acesos = agora.iter().filter(|v| v.iter().any(|c| *c > 0.0)).count();
        assert!(
            acesos > 0,
            "{nivel:?} {centro:?} {alvo:?}: a vista às escuras não prova nada"
        );
        let diferentes = agora.iter().zip(&antes).filter(|(x, y)| x != y).count();
        eprintln!(
            "{nivel:?} {centro:?} {alvo:?} (k {k}): {diferentes} de {} texels diferentes ({acesos} acesos)",
            agora.len()
        );
        assert!(
            agora.len() == antes.len() && diferentes == 0,
            "{nivel:?} {centro:?} {alvo:?}: a leitura sem desvio mudou {diferentes} texels"
        );
    }
}
