//! ⭐ **Os campos do painel são os do cartão do Inspector** (dono 15/11, smoke da W4j): a caixa aceita o que
//! se escreve (pela tecla e pelo `Commit` do IME), o número acompanha a alça arrastada no desenho, e
//! arrastar a LETRA do campo muda o número com a lei do `field_scrub`. Cada teste dirige a porta real do
//! evento e lê o número que a caixa PINTOU (o valor acessível do campo, `doc.shown`).

use bumpalo::Bump;
use ph2d_app_joint_editor::Hit;
use ph2d_app_joint_editor::doc::{Field, Handle};
use ph2d_editor_core::doc_surface::PointerInput;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_host::{Modifiers, PointerButton, PointerKind};

use super::Mesa;

impl Mesa {
    /// O texto que a caixa cujo nome começa por `label` («Speed (°/s)») pintou, lido da árvore de
    /// acessibilidade da aba.
    fn painted(&mut self, label: &str) -> String {
        let (f, id) = self.hero.documents.active_surface_mut().expect("aba");
        f.a11y(id)
            .into_iter()
            .find(|(_, n)| {
                n.role() == ph2d_a11y::Role::SpinButton
                    && n.label().is_some_and(|l| l.starts_with(label))
            })
            .and_then(|(_, n)| n.value().map(str::to_owned))
            .unwrap_or_else(|| panic!("a caixa {label} sem valor pintado"))
    }

    /// Um evento do rato com teclas, pela porta da aba.
    fn ptr_mods(&mut self, kind: PointerKind, x: f32, y: f32, mods: Modifiers) {
        let ts = &mut self.ts;
        if let Some((f, id)) = self.hero.documents.active_surface_mut() {
            let input = PointerInput {
                text: ts,
                mods,
                now_ns: 0,
                primary_held: kind != PointerKind::Up,
            };
            f.pointer(id, input, kind, PointerButton::Primary, x, y);
        }
        self.frame();
    }

    /// Uma tecla premida pela porta do teclado da shell (`board_keys::key`).
    fn tecla(&mut self, k: BoardKey, text: Option<&str>) {
        let _ = board_keys::key(
            &mut self.hero,
            &mut self.ts,
            k,
            Modifiers::default(),
            true,
            text,
            None,
        );
        self.frame();
    }

    /// Um carácter pela porta do texto (o `Commit` do IME e o caminho do texto da shell).
    fn escreve(&mut self, s: &str) {
        let arena = Bump::new();
        for ch in s.chars() {
            let _ = self.hero.handle_text_input(ch, &arena);
        }
        self.frame();
    }

    fn centre(&self, h: &Hit) -> (f32, f32) {
        let r = self.rect(h);
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }
}

fn graus(rad: f32) -> f32 {
    rad.to_degrees()
}

/// **A.** Clicar na caixa e escrever muda o número: pelo `Commit` do IME (o caminho da máquina do dono) e
/// pela tecla com texto; o Enter grava na cena.
/// **Mutações que sangram:** `compose` a devolver `false` · `type_char` sem empurrar o carácter.
#[test]
fn a_caixa_aceita_o_que_se_escreve_pelo_ime_e_pela_tecla() {
    let mut m = Mesa::carro();
    m.click(&Hit::Field(Field::MotorSpeed));
    m.escreve("-120");
    assert_eq!(
        m.painted("Speed"),
        "-120",
        "a caixa mostra o que se escreve"
    );
    m.tecla(BoardKey::Enter, None);
    let v = graus(m.joints()[0].motor_speed);
    assert!((v + 120.0).abs() < 1e-3, "o Enter grava: {v}");
    assert_eq!(m.painted("Speed"), "-120");

    m.click(&Hit::Field(Field::MotorSpeed));
    m.tecla(BoardKey::Char('9'), Some("9"));
    m.tecla(BoardKey::Char('0'), Some("0"));
    assert_eq!(m.painted("Speed"), "90", "a tecla com texto também escreve");
    m.tecla(BoardKey::Enter, None);
    assert!((graus(m.joints()[0].motor_speed) - 90.0).abs() < 1e-3);
}

/// **B.** Arrastar a alça do motor no desenho mostra o número VIVO na caixa antes do Up; a cena só muda no Up.
/// **Mutações que sangram:** a caixa a ler só `doc.value` (o guardado) · a alça sem o grão do campo.
#[test]
fn a_alca_arrastada_mostra_o_numero_vivo_na_caixa() {
    let mut m = Mesa::carro();
    let before = m.painted("Speed");
    let stored = m.joints()[0].motor_speed;
    let (x, y) = m.centre(&Hit::Handle(Handle::MotorSpeed));
    m.ptr_mods(PointerKind::Down, x, y, Modifiers::default());
    m.ptr_mods(PointerKind::Move, x, y - 12.0, Modifiers::default());
    let mid = m.painted("Speed");
    assert_ne!(mid, before, "a caixa acompanha a alça");
    assert!(
        !mid.contains('.'),
        "a alça anda no grão do campo (1 °/s): {mid}"
    );
    assert_eq!(m.joints()[0].motor_speed, stored, "a cena espera pelo Up");
    m.ptr_mods(PointerKind::Up, x, y - 12.0, Modifiers::default());
    assert_eq!(m.painted("Speed"), mid, "o número largado é o que se via");
    assert_ne!(m.joints()[0].motor_speed, stored);
}

/// **C.** Arrastar a LETRA do campo muda o número como no Inspector: um passo por pixel depois do limiar, o
/// Shift ×10, a caixa e a alça do desenho acompanham, o Up grava; um clique na letra abre a escrita.
/// **Mutações que sangram:** sem `Hit::Scrub` · o passo sem `step_mul` · a alça a não ler `live`.
#[test]
fn arrastar_a_letra_muda_o_numero_como_no_inspector() {
    let mut m = Mesa::carro();
    assert_eq!(m.painted("Speed"), "-360");
    let tip0 = m.rect(&Hit::Handle(Handle::MotorSpeed));
    let (x, y) = m.centre(&Hit::Scrub(Field::MotorSpeed));
    let no = Modifiers::default();
    m.ptr_mods(PointerKind::Down, x, y, no);
    m.ptr_mods(PointerKind::Move, x + 10.0, y, no); // passa o limiar: ancora sem salto
    assert_eq!(m.painted("Speed"), "-360");
    m.ptr_mods(PointerKind::Move, x + 30.0, y + 7.0, no); // só a horizontal conta
    assert_eq!(m.painted("Speed"), "-340", "um grau por pixel");
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    m.ptr_mods(PointerKind::Move, x + 32.0, y, shift);
    assert_eq!(m.painted("Speed"), "-320", "Shift ×10");
    assert_ne!(
        m.rect(&Hit::Handle(Handle::MotorSpeed)),
        tip0,
        "a ponta da seta acompanha o campo"
    );
    assert!(
        (graus(m.joints()[0].motor_speed) + 360.0).abs() < 1e-3,
        "a cena espera pelo Up"
    );
    m.ptr_mods(PointerKind::Up, x + 32.0, y, shift);
    assert!(
        (graus(m.joints()[0].motor_speed) + 320.0).abs() < 1e-3,
        "o Up grava"
    );

    m.ptr_mods(PointerKind::Down, x, y, no);
    m.ptr_mods(PointerKind::Up, x, y, no);
    m.escreve("5");
    assert_eq!(m.painted("Speed"), "5", "um clique na letra abre a escrita");
}

/// **A espiral do motor não tem tecto** (dono 15/11): arrastar a ponta à volta da roda, no sentido do giro,
/// passa da meia volta (−180°, o antigo limite de −720°/s) e chega a três quartos (−1080°/s); a espiral abre
/// o raio a cada volta, e a ponta fica mais longe do eixo do que a do quarto de volta.
/// **Mutações que sangram:** o arrasto sem contar voltas (`value_at` no lugar de `value_near`) · a espiral sem
/// abrir o raio · o giro preso a ±π.
#[test]
fn a_espiral_do_motor_passa_da_meia_volta() {
    let mut m = Mesa::carro();
    let (cx, cy) = m.centre(&Hit::Shape(1, 0));
    let (tx, ty) = m.centre(&Hit::Handle(Handle::MotorSpeed));
    let r0 = libm::hypotf(tx - cx, ty - cy);
    let no = Modifiers::default();
    m.ptr_mods(PointerKind::Down, tx, ty, no);
    // De −90° (a ponta de fábrica, à direita) até −270° (à esquerda), por baixo, em passos de 15°.
    let mut last = (tx, ty);
    for k in 1..=12 {
        let a = (90.0 + 15.0 * k as f32).to_radians(); // a contar do topo, no sentido horário
        last = (cx + r0 * libm::sinf(a), cy - r0 * libm::cosf(a));
        m.ptr_mods(PointerKind::Move, last.0, last.1, no);
    }
    assert_eq!(
        m.painted("Speed"),
        "-1080",
        "três quartos de volta em 0,25 s"
    );
    m.ptr_mods(PointerKind::Up, last.0, last.1, no);
    assert!((graus(m.joints()[0].motor_speed) + 1080.0).abs() < 1e-2);
    let (ex, ey) = m.centre(&Hit::Handle(Handle::MotorSpeed));
    assert!(ex < cx, "a ponta à esquerda do eixo: {ex} ≥ {cx}");
    assert!(
        libm::hypotf(ex - cx, ey - cy) > r0 * 1.05,
        "a espiral abriu: {} ≤ {r0}",
        libm::hypotf(ex - cx, ey - cy)
    );
}
