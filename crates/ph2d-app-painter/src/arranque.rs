//! **O arranque liga as páginas grandes do malloc** (B43.7, missão 4K N10).
//!
//! O glibc só lê `GLIBC_TUNABLES` antes do `main`, logo a única forma de ligar
//! `glibc.malloc.hugetlb=1` sem tocar o ambiente do dono é o processo re-`exec` a si mesmo com a
//! variável acrescentada. Medido (F6f, `medicoes/N10_build.md`): faltas de página 2 955 k → 65 k,
//! processo −14 %, o mesmo hash, +50 MB de pico. `PH2D_HUGETLB=0` desliga; a sentinela impede laço.

use std::ffi::{OsStr, OsString};

/// A tunable que o arranque acrescenta.
pub const TUNABLE: &str = "glibc.malloc.hugetlb=1";
/// Marca do processo já re-executado (nunca dois `exec`).
pub const SENTINELA: &str = "PH2D_HUGETLB_REEXEC";

/// A decisão, pura: dado o ambiente, o valor novo de `GLIBC_TUNABLES` (re-`exec`) ou `None`.
/// `thp_never` = o kernel tem as páginas grandes desligadas (`[never]`): o `madvise` seria nulo.
pub fn decide(
    tunables: Option<&OsStr>,
    chave: Option<&OsStr>,
    sentinela: Option<&OsStr>,
    thp_never: bool,
) -> Option<OsString> {
    if sentinela.is_some() || chave == Some(OsStr::new("0")) || thp_never {
        return None;
    }
    let atual = tunables
        .map(|t| t.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ja_tem = atual
        .split(':')
        .any(|e| e.split('=').next() == Some("glibc.malloc.hugetlb"));
    if ja_tem {
        return None; // a escolha de quem lançou manda
    }
    Some(if atual.is_empty() {
        OsString::from(TUNABLE)
    } else {
        OsString::from(format!("{atual}:{TUNABLE}"))
    })
}

/// Chama-se na 1.ª linha do `main`. Só em Linux/glibc; noutro sítio é nada.
pub fn liga_paginas_grandes() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        use std::os::unix::process::CommandExt;
        let thp_never = std::fs::read_to_string("/sys/kernel/mm/transparent_hugepage/enabled")
            .map(|s| s.contains("[never]"))
            .unwrap_or(false);
        let Some(novo) = decide(
            std::env::var_os("GLIBC_TUNABLES").as_deref(),
            std::env::var_os("PH2D_HUGETLB").as_deref(),
            std::env::var_os(SENTINELA).as_deref(),
            thp_never,
        ) else {
            return;
        };
        let Ok(exe) = std::env::current_exe() else {
            return;
        };
        let mut args = std::env::args_os();
        let arg0 = args.next().unwrap_or_else(|| exe.clone().into_os_string());
        let erro = std::process::Command::new(&exe)
            .arg0(arg0)
            .args(args)
            .env("GLIBC_TUNABLES", novo)
            .env(SENTINELA, "1")
            .exec();
        eprintln!("[ph2d] re-exec com {TUNABLE} falhou ({erro}); segue sem");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(t: Option<&str>, c: Option<&str>, s: Option<&str>, never: bool) -> Option<String> {
        decide(
            t.map(OsStr::new),
            c.map(OsStr::new),
            s.map(OsStr::new),
            never,
        )
        .map(|v| v.into_string().unwrap())
    }

    #[test]
    fn o_arranque_decide_o_reexec_das_paginas_grandes() {
        assert_eq!(d(None, None, None, false).as_deref(), Some(TUNABLE));
        assert_eq!(d(Some(""), None, None, false).as_deref(), Some(TUNABLE));
        // as tunables do dono ficam, a nossa vai no fim
        assert_eq!(
            d(Some("glibc.malloc.arena_max=2"), None, None, false).as_deref(),
            Some("glibc.malloc.arena_max=2:glibc.malloc.hugetlb=1")
        );
        // quem lançou já escolheu (até 0): não se mexe
        assert_eq!(d(Some("glibc.malloc.hugetlb=0"), None, None, false), None);
        assert_eq!(
            d(Some("a.b=1:glibc.malloc.hugetlb=2"), None, None, false),
            None
        );
        // nome parecido não conta
        assert!(d(Some("glibc.malloc.hugetlbx=1"), None, None, false).is_some());
        // chave, sentinela e kernel sem THP
        assert_eq!(d(None, Some("0"), None, false), None);
        assert!(d(None, Some("1"), None, false).is_some());
        assert_eq!(d(None, None, Some("1"), false), None);
        assert_eq!(d(None, None, None, true), None);
    }
}
