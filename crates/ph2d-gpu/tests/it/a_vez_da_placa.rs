//! A VEZ da placa (`ph2d_gpu::vez`, `docs/DevOps/A_VEZ_DA_PLACA.md`): todo pedido de adaptador do
//! repo passa por ela, e o cliente Rust cumpre o protocolo do porteiro do `scripts/ph2d-run.sh`.

use std::path::{Path, PathBuf};

fn fontes(dir: &Path, saida: &mut Vec<PathBuf>) {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n != "target" && n != ".git") {
                fontes(&p, saida);
            }
        } else if p.extension().is_some_and(|x| x == "rs") {
            saida.push(p);
        }
    }
}

/// ⛔ Um pedido de adaptador sem a vez à frente põe o processo na placa a meio da medição de outra
/// linha (17/10: os `it-*` e o smoke de outra linha estragaram uma rodada inteira da Luz 2D).
/// A agulha monta-se por partes para este ficheiro não se contar a si próprio.
#[test]
fn todo_pedido_de_adaptador_passa_pela_vez() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut ficheiros = Vec::new();
    for d in ["crates", "shells", "spike"] {
        fontes(&raiz.join(d), &mut ficheiros);
    }
    let agulhas = [
        concat!("request_", "adapter("),
        concat!("enumerate_", "adapters("),
    ];
    let vez = concat!("vez_da_", "placa(");
    let (mut vistos, mut sem_vez, mut no_contexto) = (0, Vec::new(), false);
    for f in &ficheiros {
        let Ok(texto) = std::fs::read_to_string(f) else {
            continue;
        };
        let fonte = ph2d_label_census::sem_comentarios(&texto);
        for agulha in agulhas {
            for (i, _) in fonte.match_indices(agulha) {
                vistos += 1;
                let rel = f.strip_prefix(&raiz).unwrap_or(f).display().to_string();
                no_contexto |= rel.ends_with("ph2d-gpu/src/context.rs");
                // Da `fn` que o contém até ao pedido.
                let inicio = fonte[..i].rfind("fn ").unwrap_or(0);
                if !fonte[inicio..i].contains(vez) {
                    let linha = fonte[..i].lines().count();
                    sem_vez.push(format!("{rel}:{linha}"));
                }
            }
        }
    }
    // Controlo da população: um censo que não vê o pedido do próprio `GpuContext` está cego.
    assert!(
        no_contexto && vistos >= 2,
        "o censo não viu os pedidos conhecidos (vistos {vistos}) — a régua está cega"
    );
    assert!(
        sem_vez.is_empty(),
        "pedido de adaptador sem `ph2d_gpu::vez_da_placa()` antes, na mesma fn:\n  {}\n\
         Chame-a antes (ou use o `GpuContext::new`, que já a chama).",
        sem_vez.join("\n  ")
    );
}

#[cfg(target_os = "linux")]
mod protocolo {
    use ph2d_gpu::vez::pede_em;
    use std::io::{BufRead, BufReader};
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    fn pasta(nome: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ph2d-vez-teste-{}-{nome}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// Um porteiro de mentira: o pid dele é `pid` e a porta fica aberta (como a do porteiro real).
    fn porteiro(d: &Path, pid: u32) -> std::fs::File {
        let ok = std::process::Command::new("mkfifo")
            .arg(d.join("porta"))
            .status()
            .unwrap();
        assert!(ok.success());
        std::fs::write(d.join("porteiro"), format!("{pid}\n")).unwrap();
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(d.join("porta"))
            .unwrap()
    }

    #[test]
    fn com_a_vez_ja_na_arvore_nao_pede_nada() {
        let d = pasta("pronto");
        std::fs::write(d.join("pronto"), "").unwrap();
        assert_eq!(pede_em(&d), Ok(()));
        assert!(!d.join("porta").exists());
    }

    #[test]
    fn pede_pela_porta_e_espera_o_pronto() {
        let d = pasta("espera");
        let porta = porteiro(&d, std::process::id());
        let d2 = d.clone();
        let atende = std::thread::spawn(move || {
            let mut pedido = String::new();
            BufReader::new(porta).read_line(&mut pedido).unwrap();
            std::thread::sleep(Duration::from_millis(300));
            std::fs::write(d2.join("pronto"), "").unwrap();
            pedido
        });
        let t0 = Instant::now();
        assert_eq!(pede_em(&d), Ok(()));
        assert!(
            t0.elapsed() >= Duration::from_millis(300),
            "não esperou o porteiro"
        );
        let pedido = atende.join().unwrap();
        assert!(
            pedido.starts_with(&format!("{} ", std::process::id())),
            "o pedido diz quem pede: {pedido:?}"
        );
    }

    #[test]
    fn a_recusa_do_porteiro_e_um_erro_com_o_porque() {
        let d = pasta("recusa");
        let _porta = porteiro(&d, std::process::id());
        std::fs::write(
            d.join("recusa"),
            "a placa está com outra árvore há mais de 5s",
        )
        .unwrap();
        assert_eq!(
            pede_em(&d),
            Err("a placa está com outra árvore há mais de 5s".into())
        );
    }

    #[test]
    fn sem_porteiro_vivo_segue_sem_esperar() {
        let d = pasta("morto");
        let mut filho = std::process::Command::new("true").spawn().unwrap();
        let morto = filho.id();
        filho.wait().unwrap();
        let _porta = porteiro(&d, morto);
        let t0 = Instant::now();
        assert_eq!(pede_em(&d), Ok(()));
        assert!(t0.elapsed() < Duration::from_secs(2));
    }
}
