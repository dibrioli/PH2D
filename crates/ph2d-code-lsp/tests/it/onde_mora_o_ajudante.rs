//! Where the pinned helper is found (`Pin::resolve_from`): env var → next to the executable → the
//! cache. `scripts/pinned-helpers-next-to-exe.sh` fills the SECOND place, so a clean home folder still
//! has the assistant (crítica 08/11: sem o cache, «not installed» e nada de sugestões nem ajuda).

use ph2d_code_lsp::{BinarySource, LUAU_LSP, ResolveError};
use std::path::{Path, PathBuf};

fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ph2d-onde-mora-{}-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn com_binario(dir: &Path) -> PathBuf {
    let p = dir.join(LUAU_LSP.binary_file_name());
    std::fs::write(&p, b"").unwrap();
    p
}

#[test]
fn ao_lado_do_executavel_vence_o_cache() {
    let (exe, cache) = (pasta("exe-a"), pasta("cache-a"));
    let ao_lado = com_binario(&exe);
    com_binario(&cache);
    assert_eq!(
        LUAU_LSP.resolve_from(None, Some(exe), Some(cache)),
        Ok((ao_lado, BinarySource::NextToExe))
    );
}

#[test]
fn sem_nada_ao_lado_o_cache_responde() {
    let (exe, cache) = (pasta("exe-b"), pasta("cache-b"));
    let no_cache = com_binario(&cache);
    assert_eq!(
        LUAU_LSP.resolve_from(None, Some(exe), Some(cache)),
        Ok((no_cache, BinarySource::Cache))
    );
}

#[test]
fn a_variavel_vence_tudo_e_nunca_e_saltada() {
    let (exe, cache) = (pasta("exe-c"), pasta("cache-c"));
    com_binario(&exe);
    let dir = pasta("env-c");
    let env = com_binario(&dir);
    assert_eq!(
        LUAU_LSP.resolve_from(
            Some(env.clone().into()),
            Some(exe.clone()),
            Some(cache.clone())
        ),
        Ok((env, BinarySource::Env))
    );
    let nenhures = dir.join("nao-existe");
    assert_eq!(
        LUAU_LSP.resolve_from(Some(nenhures.clone().into()), Some(exe), Some(cache)),
        Err(ResolveError::EnvPointsNowhere {
            pin: LUAU_LSP,
            path: nenhures
        })
    );
}

#[test]
fn sem_programa_o_erro_diz_os_dois_sitios_e_o_comando() {
    let (exe, cache) = (pasta("exe-d"), pasta("cache-d"));
    let e = LUAU_LSP
        .resolve_from(Some("".into()), Some(exe.clone()), Some(cache.clone()))
        .unwrap_err();
    let nome = LUAU_LSP.binary_file_name();
    assert_eq!(
        e,
        ResolveError::NotFound {
            pin: LUAU_LSP,
            tried: vec![exe.join(&nome), cache.join(&nome)]
        }
    );
    assert!(e.to_string().contains(LUAU_LSP.fetch), "{e}");
}

/// O script que põe o ajudante ao lado do executável chama os DOIS fetch fixados (o do `luau-lsp`
/// é o mesmo que o `Pin` cita) — renomear um deixaria a pasta do build sem ele, em silêncio.
#[test]
fn o_script_do_build_chama_os_fetch_fixados() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let s = std::fs::read_to_string(raiz.join("scripts/pinned-helpers-next-to-exe.sh")).unwrap();
    let luau = Path::new(LUAU_LSP.fetch)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    for fetch in [luau, "stylua-fetch.sh"] {
        assert!(s.contains(fetch), "o script não chama {fetch}");
        assert!(
            raiz.join("scripts").join(fetch).is_file(),
            "{fetch} não existe"
        );
    }
}
