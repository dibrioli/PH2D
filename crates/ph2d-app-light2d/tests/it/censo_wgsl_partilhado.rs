//! **Um texto WGSL, vários consumidores** (molde do censo do G12): não há `#include` em WGSL, e duas
//! cópias de uma conta divergem em silêncio. Cada função partilhada da luz 2D é DEFINIDA num só
//! ficheiro `.wgsl` do repositório inteiro, e cada módulo que a usa CONTÉM esse texto (concatenado).
//!
//! - `luz_em` / `luz_receber` (`receber.wgsl`) — o sprite e a colagem do vetor (G7: a mesma conta);
//! - `tonemap_mapear` (`tonemap_conta.wgsl`) — o passe de tonemap e a colagem do vetor aceso;
//! - `c0_alvos` (`c0_dono.wgsl`) — a C0 das formas e a dos sprites por texel.

use std::fs;
use std::path::{Path, PathBuf};

fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("a raiz da workspace")
        .to_path_buf()
}

fn wgsl(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target" || n == ".git") {
                continue;
            }
            wgsl(&p, out);
        } else if p.extension().is_some_and(|x| x == "wgsl")
            && let Ok(s) = fs::read_to_string(&p)
        {
            out.push((p, s));
        }
    }
}

#[test]
fn cada_conta_partilhada_da_luz_e_definida_uma_vez() {
    let mut todos = Vec::new();
    for d in ["crates", "shells", "tools"] {
        wgsl(&raiz().join(d), &mut todos);
    }
    // Controlo positivo: a varredura acha os shaders da casa (senão o censo mede o nada).
    assert!(
        todos.len() > 30,
        "a varredura achou só {} .wgsl",
        todos.len()
    );
    for (funcao, dono) in [
        ("fn luz_em(", "receber.wgsl"),
        ("fn luz_receber(", "receber.wgsl"),
        ("fn tonemap_mapear(", "tonemap_conta.wgsl"),
        ("fn c0_alvos(", "c0_dono.wgsl"),
    ] {
        let definem: Vec<&PathBuf> = todos
            .iter()
            .filter(|(_, s)| s.contains(funcao))
            .map(|(p, _)| p)
            .collect();
        assert_eq!(
            definem.len(),
            1,
            "`{funcao}` tem de existir num só .wgsl — achado em {definem:?}"
        );
        assert!(
            definem[0].ends_with(dono),
            "`{funcao}` vive em `{dono}`, não em {:?}",
            definem[0]
        );
    }
}

#[test]
fn cada_modulo_consumidor_contem_o_texto_partilhado() {
    let receber = ph2d_render::RECEBER_WGSL;
    let tonemap = ph2d_render::tonemap::TONEMAP_CONTA_WGSL;
    for (nome, modulo, partes) in [
        ("sprite", ph2d_render::pipeline::SPRITE_WGSL, vec![receber]),
        (
            "colagem do vetor",
            ph2d_render::band_blit::BAND_BLIT_WGSL,
            vec![receber, tonemap],
        ),
        ("tonemap", ph2d_render::tonemap::TONEMAP_WGSL, vec![tonemap]),
    ] {
        for parte in partes {
            assert!(
                modulo.contains(parte),
                "o módulo do {nome} deixou de concatenar o texto partilhado"
            );
        }
    }
    // A lei do dono da C0: as duas capturas CHAMAM-na e nenhuma a redefine (o primeiro teste).
    let dono = ph2d_light2d_gpu::C0_DONO_WGSL;
    assert!(dono.contains("fn c0_alvos("));
    for f in ["c0_captura.wgsl", "c0_sprite.wgsl"] {
        let s = fs::read_to_string(raiz().join("crates/ph2d-light2d-gpu/src/wgsl").join(f))
            .expect("o shader da C0");
        assert!(
            s.contains("return c0_alvos("),
            "`{f}` deixou de passar pela lei única do dono"
        );
    }
}
