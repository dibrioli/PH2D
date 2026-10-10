//! O aperto de mão por dentro: as chaves dos dois lados casam, e o anfitrião não confia no convidado
//! (um convidado que ignora a confirmação e manda a sua com o código errado é recusado).

use std::net::{TcpListener, TcpStream};
use std::thread;

use super::*;

fn pair() -> (TcpStream, TcpStream) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let a = TcpStream::connect(l.local_addr().unwrap()).unwrap();
    let (b, _) = l.accept().unwrap();
    (a, b)
}

fn run_host(mut s: TcpStream, code: Code) -> thread::JoinHandle<(HostShake, Gate)> {
    thread::spawn(move || {
        let gate = Mutex::new(Gate::default());
        let r = host(&mut s, &code, &gate, &Instant::now);
        (r, gate.into_inner().unwrap())
    })
}

#[test]
fn codigo_certo_as_chaves_casam_nos_dois_sentidos() {
    let code = Code::parse("ABC-DEF").unwrap();
    let (mut g, h) = pair();
    let ht = run_host(h, code.clone());
    let mut gk = guest(&mut g, &code).unwrap();
    let (HostShake::Joined(mut hk), _) = ht.join().unwrap() else {
        panic!("o anfitrião devia aceitar");
    };
    let w = gk.seal.seal(b"do convidado").unwrap();
    assert_eq!(
        hk.open.read(&mut std::io::Cursor::new(w)).unwrap(),
        b"do convidado"
    );
    let w = hk.seal.seal(b"do anfitriao").unwrap();
    assert_eq!(
        gk.open.read(&mut std::io::Cursor::new(w)).unwrap(),
        b"do anfitriao"
    );
    // As direcções têm chaves diferentes: o próprio eco não abre.
    let w = gk.seal.seal(b"x").unwrap();
    assert!(gk.open.read(&mut std::io::Cursor::new(w)).is_err());
}

#[test]
fn o_anfitriao_recusa_a_confirmacao_de_um_convidado_que_nao_sabe_o_codigo() {
    let (mut g, h) = pair();
    let ht = run_host(h, Code::parse("ABC-DEF").unwrap());
    // Um convidado desonesto: código errado, ignora a confirmação do anfitrião e manda a sua.
    let wrong = Code::parse("ABC-DEG").unwrap();
    let (sp, msg_a) = Spake2::<Ed25519Group>::start_a(
        &password(&wrong),
        &Identity::new(ID_GUEST),
        &Identity::new(ID_HOST),
    );
    let mut hello = MAGIC.to_vec();
    hello.push(VERSION);
    hello.extend_from_slice(&msg_a);
    g.write_all(&frame(&hello)).unwrap();
    let reply = read_frame(&mut g, MAX_SHAKE).unwrap();
    assert_eq!(reply[0], GO);
    let msg_b = &reply[1..=MSG];
    let k = sp.finish(msg_b).unwrap();
    let d = derive(&k);
    let transcript = [&msg_a[..], msg_b].concat();
    let mac_g = mac(&d.conf_g, &transcript).finalize().into_bytes();
    g.write_all(&frame(&mac_g)).unwrap();
    let (r, gate) = ht.join().unwrap();
    assert!(matches!(r, HostShake::Wrong));
    assert_eq!(gate.fails(), 1, "conta como palpite");
}

#[test]
fn codigo_errado_o_convidado_ve_e_o_anfitriao_conta() {
    let (mut g, h) = pair();
    let ht = run_host(h, Code::parse("ABC-DEF").unwrap());
    let r = guest(&mut g, &Code::parse("ABC-DEG").unwrap());
    assert_eq!(r.err(), Some(JoinError::WrongCode));
    drop(g);
    let (r, gate) = ht.join().unwrap();
    assert!(matches!(r, HostShake::Wrong));
    assert_eq!(gate.fails(), 1);
}

#[test]
fn outra_versao_e_recusada_sem_contar_palpite() {
    let (mut g, h) = pair();
    let ht = run_host(h, Code::parse("ABC-DEF").unwrap());
    let mut hello = MAGIC.to_vec();
    hello.push(VERSION + 1);
    hello.extend_from_slice(&[0; MSG]);
    g.write_all(&frame(&hello)).unwrap();
    assert_eq!(read_frame(&mut g, MAX_SHAKE).unwrap(), [OTHER, VERSION]);
    let (r, gate) = ht.join().unwrap();
    assert!(matches!(r, HostShake::Refused));
    assert_eq!(gate.fails(), 0);
}
