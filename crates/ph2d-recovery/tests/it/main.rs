//! As provas da recuperação (`docs/PixelArt/P12_plano.md` §4.3).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ph2d_recovery::atomic::{Fault, write_atomic, write_atomic_with};
use ph2d_recovery::{Autosave, Choice, Meta, Offer, Period, Prompt, Session, scan};

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "ph2d-recovery-{tag}-{}-{}",
        std::process::id(),
        fastrand::u64(..)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("tmp");
    d
}

fn meta(saved: u64) -> Meta {
    Meta {
        name: "Hero.ph2dproj".into(),
        path: "/home/x/Hero.ph2dproj".into(),
        saved,
    }
}

fn dead(_: u32) -> bool {
    false
}

/// ⛔ Morte em cada ponto da escrita: reabrir dá a cópia anterior inteira ou a nova, nunca meia.
#[test]
fn matar_a_escrita_em_cada_passo_deixa_a_anterior_ou_a_nova() {
    let d = tmpdir("kill");
    let path = d.join("project.ph2dproj");
    for round in 0..40 {
        let old: Vec<u8> = (0..fastrand::usize(1..300))
            .map(|_| fastrand::u8(..))
            .collect();
        let new: Vec<u8> = (0..fastrand::usize(1..300))
            .map(|_| fastrand::u8(..))
            .collect();
        write_atomic(&path, &old).expect("old");
        let mut faults: Vec<Fault> = (0..=new.len()).map(Fault::StopAfter).collect();
        faults.push(Fault::BeforeRename);
        for f in faults {
            write_atomic(&path, &old).expect("reset");
            assert!(write_atomic_with(&path, &new, f).is_err(), "{f:?}");
            let got = std::fs::read(&path).expect("read");
            assert_eq!(
                got, old,
                "round {round}, {f:?}: o ficheiro final mudou sem rename"
            );
        }
        write_atomic_with(&path, &new, Fault::None).expect("new");
        assert_eq!(std::fs::read(&path).expect("read"), new);
        // E a varredura apaga o temporário que a escrita morta deixou.
        let _ = write_atomic_with(&path, &old, Fault::StopAfter(1));
        assert!(ph2d_recovery::atomic::tmp_of(&path).exists());
    }
    let _ = std::fs::remove_dir_all(d);
}

/// Só se escreve quando os bytes mudam; o Save também conta como «a última».
#[test]
fn so_grava_quando_os_bytes_mudaram() {
    let root = tmpdir("dirty");
    let s = Session::start(&root, 1, 4242).expect("session");
    let t0 = Instant::now();
    let mut a = Autosave::new(Period::default(), t0);
    assert!(!a.due(t0), "antes do período não é hora");
    let t1 = t0 + Duration::from_secs(120);
    assert!(a.due(t1));
    a.offer(b"A".to_vec(), meta(1), &s, t1);
    assert_eq!(a.finish(), Some(Offer::Written));
    assert_eq!(
        std::fs::read(s.dir().join("project.ph2dproj")).expect("copy"),
        b"A"
    );
    let t2 = t1 + Duration::from_secs(120);
    a.offer(b"A".to_vec(), meta(2), &s, t2);
    assert_eq!(a.finish(), Some(Offer::Unchanged));
    // O Save apaga a cópia e lembra os bytes gravados: os mesmos bytes não voltam a escrever.
    a.mark_saved(b"B", &s, t2);
    assert!(!s.dir().join("project.ph2dproj").exists());
    a.offer(b"B".to_vec(), meta(3), &s, t2);
    assert_eq!(a.finish(), Some(Offer::Unchanged));
    a.offer(b"C".to_vec(), meta(4), &s, t2);
    assert_eq!(a.finish(), Some(Offer::Written));
    let _ = std::fs::remove_dir_all(root);
}

/// Sessão viva não se oferece; morta sim; o fecho normal não deixa nada.
#[test]
fn a_sessao_morta_e_oferecida_e_a_viva_nao() {
    let root = tmpdir("dead");
    let now = 1_000_000;
    let morta = Session::start(&root, 10, 111).expect("a");
    morta.write_copy(b"morta", &meta(now)).expect("w");
    let viva = Session::start(&root, 11, 222).expect("b");
    viva.write_copy(b"viva", &meta(now)).expect("w");
    let limpa = Session::start(&root, 12, 333).expect("c");
    limpa.write_copy(b"limpa", &meta(now)).expect("w");
    limpa.close();
    let eu = Session::start(&root, 13, 444).expect("d");
    let found = scan(&root, now, Some(eu.dir()), &|pid| pid == 222);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(std::fs::read(&found[0].copy).expect("r"), b"morta");
    assert_eq!(found[0].size, 5);
    assert_eq!(found[0].meta, meta(now));
    // O pid deste processo vive de verdade.
    assert!(ph2d_recovery::is_alive(std::process::id()));
    let _ = std::fs::remove_dir_all(root);
}

/// Mais de uma semana: sai sem se oferecer. Morta sem cópia: sai.
#[test]
fn a_copia_de_mais_de_uma_semana_e_podada() {
    let root = tmpdir("prune");
    let now = 10_000_000;
    let velha = Session::start(&root, 1, 1).expect("v");
    velha
        .write_copy(b"v", &meta(now - ph2d_recovery::KEEP_SECS - 1))
        .expect("w");
    let quase = Session::start(&root, 2, 2).expect("q");
    quase
        .write_copy(b"q", &meta(now - ph2d_recovery::KEEP_SECS))
        .expect("w");
    let vazia = Session::start(&root, 3, 3).expect("e");
    let (vd, vz) = (velha.dir().to_path_buf(), vazia.dir().to_path_buf());
    let found = scan(&root, now, None, &dead);
    assert_eq!(found.len(), 1);
    assert_eq!(std::fs::read(&found[0].copy).expect("r"), b"q");
    assert!(!vd.exists() && !vz.exists(), "a velha e a vazia saíram");
    let _ = std::fs::remove_dir_all(root);
}

/// `Restore` devolve os bytes da cópia, byte a byte, e a cópia passa à sessão nova; `Discard` apaga;
/// `Later` deixa no disco.
#[test]
fn restaurar_devolve_os_bytes_da_copia() {
    let root = tmpdir("restore");
    let bytes: Vec<u8> = (0..4096).map(|_| fastrand::u8(..)).collect();
    for (i, b) in [&bytes[..], b"discard", b"later"].iter().enumerate() {
        let s = Session::start(&root, i as u64, 900 + i as u32).expect("s");
        s.write_copy(b, &meta(100 + i as u64)).expect("w");
    }
    let eu = Session::start(&root, 99, 999).expect("eu");
    let mut p = Prompt {
        entries: scan(&root, 200, Some(eu.dir()), &dead),
    };
    assert!(p.is_open());
    assert_eq!(p.entries.len(), 3);
    // A mais nova primeiro: later (102), discard (101), a grande (100).
    let (later, _) = p.answer(0, Choice::Later).expect("later");
    let (disc, _) = p.answer(0, Choice::Discard).expect("discard");
    let (rest, c) = p.answer(0, Choice::Restore).expect("restore");
    assert_eq!(c, Choice::Restore);
    assert!(!p.is_open());
    assert_eq!(eu.adopt(&rest).expect("adopt"), bytes);
    assert_eq!(
        std::fs::read(eu.dir().join("project.ph2dproj")).expect("r"),
        bytes
    );
    assert!(!rest.dir.exists() && !disc.dir.exists() && later.dir.exists());
    let again = scan(&root, 200, Some(eu.dir()), &dead);
    assert_eq!(again.len(), 1, "só a do Later volta");
    assert_eq!(rest.meta.path, "/home/x/Hero.ph2dproj");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn o_intervalo_vai_a_opcao_do_aseprite_mais_proxima() {
    assert_eq!(Period::default().duration(), Duration::from_secs(120));
    assert_eq!(
        Period::from_minutes("0.166700").duration(),
        Duration::from_secs(10)
    );
    assert_eq!(
        Period::from_minutes("5").duration(),
        Duration::from_secs(300)
    );
    assert_eq!(
        Period::from_minutes("8").duration(),
        Duration::from_secs(600)
    );
    assert_eq!(Period::from_minutes("lixo"), Period::default());
    assert_eq!(Period::from_minutes("-1"), Period::default());
    assert!(Period::from_minutes("0").is_off(), "0 = Off");
    assert_eq!(Period::from_minutes(&Period::OFF.to_minutes()), Period::OFF);
    assert_eq!(Period::from_secs(0), Period::OFF);
    for s in Period::OPTIONS_SECS {
        assert_eq!(Period::from_secs(s).duration().as_secs(), s);
    }
    for s in Period::OPTIONS_SECS {
        let p = Period::from_minutes(&format!("{}", s as f64 / 60.0));
        assert_eq!(Period::from_minutes(&p.to_minutes()), p, "{s}");
    }
}

/// A medida que decidiu o fio (o número vai no doc do `autosave`).
#[test]
fn a_escrita_de_um_projeto_grande_custa() {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join("recovery-medida");
    std::fs::create_dir_all(&d).expect("dir");
    let path: &Path = &d.join("big.ph2dproj");
    let bytes = vec![7u8; 64 << 20];
    let mut best = Duration::MAX;
    for _ in 0..3 {
        let t = Instant::now();
        write_atomic(path, &bytes).expect("w");
        best = best.min(t.elapsed());
    }
    eprintln!("[medida] write_atomic 64 MB: {best:?} (mínimo de 3)");
    let _ = std::fs::remove_dir_all(d);
}

/// A medida da serialização, à hora, de um projecto GRANDE: 16 Pixel Sprites 512² RGBA × 8 quadros
/// (128 MiB de cels, fundo branco opaco): o codec do Pixel, uma cópia (o `postcard` do
/// `ProjectFile` copia cada blob uma vez) e a impressão. Corre com `--profile smoke`.
#[test]
fn a_serializacao_de_um_projeto_grande_custa() {
    use ph2d_pixel_core::frames::{NewFrame, new_frame};
    use ph2d_pixel_core::{Background, NewSprite, Sprite};
    let spec = NewSprite {
        width: 512,
        height: 512,
        background: Background::White,
        ..NewSprite::default()
    };
    let mut lib = std::collections::BTreeMap::new();
    for k in 0..16u64 {
        let mut s = Sprite::new(spec).expect("sprite");
        for _ in 1..8 {
            let last = s.frames().last().expect("quadro").id;
            let (op, _) = new_frame(&mut s, last, NewFrame::Copy).expect("quadro novo");
            ph2d_pixel_core::apply(&mut s, op).expect("aplica");
        }
        lib.insert(k, s);
    }
    let min3 = |f: &mut dyn FnMut()| {
        (0..3)
            .map(|_| {
                let t = Instant::now();
                f();
                t.elapsed()
            })
            .min()
            .unwrap_or_default()
    };
    let mut bytes = Vec::new();
    let codec = min3(&mut || bytes = ph2d_pixel_core::codec::write_library(&lib));
    let copy = min3(&mut || drop(std::hint::black_box(bytes.to_vec())));
    let mut p = 0;
    let print = min3(&mut || p = ph2d_recovery::print_of(&bytes));
    eprintln!(
        "[medida] {} MiB: codec {codec:?} · cópia {copy:?} · impressão {print:?} ({p:x})",
        bytes.len() >> 20
    );
    assert!(bytes.len() >= 128 << 20, "as cels estão lá");
}

/// A janela fala inglês da tabela (nunca a chave crua): nome, há quanto tempo, tamanho.
#[test]
fn a_janela_diz_nome_hora_e_tamanho() {
    let o = ph2d_recovery::Orphan {
        dir: PathBuf::new(),
        copy: PathBuf::new(),
        meta: meta(1_000),
        size: 3 << 20,
    };
    let b = ph2d_recovery::body(&o, 1_000 + 5 * 60);
    assert!(
        b.contains("Hero.ph2dproj") && b.contains("5 minutes ago") && b.contains("3 MB"),
        "{b}"
    );
    assert_eq!(ph2d_i18n::tr("recovery.restore"), "Restore");
    assert_eq!(ph2d_recovery::size(1), "1 KB");
}

mod card;

/// ⭐ Sem soluço a meio do gesto: com a hora vencida, um traço em curso (mesmo parado) e a entrada a
/// chegar ADIAM a serialização; o quadro calmo depois do fim do traço ou 1 s sem entrada liberta-a;
/// sem pausa nenhuma, o prazo duro de 2 × o intervalo grava na mesma. `Off` nunca é hora.
#[test]
fn a_hora_espera_a_pausa_do_artista_com_prazo_duro() {
    let root = tmpdir("idle");
    let mut r = ph2d_recovery::Recovery::start(Some(&root), Period::default());
    let p = r.autosave.period.duration();
    let t0 = Instant::now();
    let hora = t0 + p + Duration::from_millis(1);
    // A entrada chega em todos os quadros: a hora venceu, mas o artista não parou.
    r.input(true, hora);
    assert!(!r.due(hora), "entrada a chegar adia");
    assert!(r.due(hora + ph2d_recovery::IDLE), "1 s sem entrada é pausa");
    // Um traço premido e PARADO (sem eventos) também não é pausa.
    r.pointer(true);
    r.input(false, hora + Duration::from_millis(10));
    assert!(
        !r.due(hora + Duration::from_secs(5)),
        "o traço em curso adia"
    );
    // O traço acaba: o quadro seguinte sem entrada já é pausa (não espera o segundo).
    r.pointer(false);
    let fim = hora + Duration::from_secs(6);
    r.input(true, fim);
    assert!(!r.due(fim), "o quadro do Up ainda tem entrada");
    r.input(false, fim + Duration::from_millis(16));
    assert!(
        r.due(fim + Duration::from_millis(16)),
        "o quadro calmo depois do traço"
    );
    // Sem pausa nenhuma: o prazo duro (2 × o intervalo desde a última hora) grava na mesma.
    r.pointer(true);
    r.input(true, t0 + 2 * p);
    assert!(!r.due(t0 + 2 * p - Duration::from_millis(1)));
    assert!(r.due(t0 + 2 * p + Duration::from_millis(1)), "o prazo duro");
    // Off: nunca é hora; mudar o intervalo vale já (a próxima hora conta de agora).
    r.follow(0, t0);
    assert!(!r.due(t0 + Duration::from_secs(10_000)), "Off");
    r.follow(10, t0);
    r.pointer(false);
    r.input(false, t0);
    assert!(!r.due(t0 + Duration::from_secs(9)));
    assert!(r.due(t0 + Duration::from_secs(11)), "10 s vale já");
    r.close();
    let _ = std::fs::remove_dir_all(root);
}

/// ⭐ A MEDIDA do que fica no fio da janela à hora: antes (impressão SipHash no fio + entrega) ×
/// depois (só a entrega; o fio tira a impressão e escreve), 128 MiB, MESMO processo, intercalado,
/// mínimo de 5 (`docs/DevOps/MEDIR_VELOCIDADE.md`). Corre com `--profile smoke`.
#[test]
fn o_fio_da_janela_so_entrega_os_bytes() {
    let root = tmpdir("entrega");
    let s = Session::start(&root, 1, 4243).expect("session");
    let mut a = Autosave::new(Period::default(), Instant::now());
    let (mut antes, mut depois) = (Duration::MAX, Duration::MAX);
    for i in 0..10u8 {
        let bytes = vec![i; 128 << 20];
        if i % 2 == 0 {
            // O caminho de antes: a impressão no fio da janela, depois o fio só escreve.
            let t = Instant::now();
            let p = ph2d_recovery::print_of(&bytes);
            let dir = s.dir().to_path_buf();
            let h = std::thread::spawn(move || {
                ph2d_recovery::atomic::write_atomic(&dir.join("x"), &bytes).map(|()| p)
            });
            antes = antes.min(t.elapsed());
            h.join().expect("fio").expect("escrita");
        } else {
            let t = Instant::now();
            a.offer(bytes, meta(u64::from(i)), &s, Instant::now());
            depois = depois.min(t.elapsed());
            assert_eq!(a.finish(), Some(Offer::Written));
        }
    }
    eprintln!(
        "[medida] 128 MiB no fio da janela: antes {antes:?} · depois {depois:?} (mínimo de 5)"
    );
    assert!(depois < antes, "a entrega custa menos que a impressão");
    let _ = std::fs::remove_dir_all(root);
}

/// P12 S: os bytes JUNTAM-SE no fio da escrita (`offer_with`): a janela entrega um fecho; o que ele
/// devolve grava-se igual ao `offer`, e um `None` (serialização recusada) não escreve nada.
#[test]
fn a_serializacao_corre_no_fio_da_escrita() {
    let root = tmpdir("with");
    let s = Session::start(&root, 1, 4243).expect("session");
    let t0 = Instant::now();
    let mut a = Autosave::new(Period::default(), t0);
    let janela = std::thread::current().id();
    let t1 = t0 + Duration::from_secs(120);
    a.offer_with(
        move || {
            assert_ne!(std::thread::current().id(), janela, "juntou no fio da janela");
            Some(b"AB".to_vec())
        },
        meta(1),
        &s,
        t1,
    );
    assert_eq!(a.finish(), Some(Offer::Written));
    let copia = s.dir().join("project.ph2dproj");
    assert_eq!(std::fs::read(&copia).expect("copy"), b"AB");
    a.offer_with(|| None, meta(2), &s, t1);
    assert_eq!(a.finish(), None, "recusado: nada gravado");
    assert_eq!(std::fs::read(&copia).expect("copy"), b"AB", "a anterior fica");
    a.offer(b"AB".to_vec(), meta(3), &s, t1);
    assert_eq!(a.finish(), Some(Offer::Written), "a falha esqueceu a impressão");
    let _ = std::fs::remove_dir_all(root);
}
