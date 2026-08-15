//! **UM TETO DIGITÁVEL NÃO PODE PASSAR DO QUE O KERNEL HONRA.**
//!
//! A §14 regista as faixas dos números dela em
//! `ph2d-panel-inspector/src/populate_physics.rs`, e a lei do player clampa
//! alguns deles em constantes **MEDIDAS** (`RideConfig::MAX_SPRING_STRENGTH`,
//! `RideConfig::MAX_DAMPING`). Quando a faixa oferecida passa do clamp, o
//! artista arrasta o slider até ao fim, **não vê nada mudar** e não tem como
//! saber por quê: a caixa *aceita e mente*.
//!
//! ⚠️ **Foi assim que este defeito nasceu, e é o §0 a morder:** o comentário da
//! row da rigidez dizia *"sem teto medido — o que aperta a rigidez é a
//! estabilidade do passo, e o amortecimento é quem a governa"*, e era **verdade
//! no dia em que foi escrita**. A `W-Landing` (2026-08-07) mediu o teto em
//! `1/dt²` = **3600** e pôs o clamp na lei; ninguém reconferiu a nota, e o
//! slider continuou a oferecer **100 000** — vinte e sete vezes o que o kernel
//! honra. *Quem move o número que tornava algo inalcançável tem de reconferir a
//! nota.*
//!
//! # ⚠️ Por que este gate mora na SHELL
//!
//! A `ph2d-panel-inspector` **não depende** da `ph2d-platformer` nem da
//! `ph2d-physics-ecs` (ela é o desenho, não o modelo), então nem ela consegue
//! afirmar isto sobre si mesma. A shell é a única árvore que vê os dois lados —
//! o precedente exato do gate do `MAX_FX_KINDS` da `line/Vector`.
//!
//! ⚠️ **E o teto é lido da LEI, ao vivo** — nunca de um literal copiado para cá.
//! Um número escrito à mão neste arquivo seria a terceira cópia do mesmo fato, e
//! a que ninguém atualizaria quando a medição se movesse.

use ph2d_physics_ecs::RideConfig;
use std::path::{Path, PathBuf};

/// A tabela: **o id da row, e a constante da lei que a clampa.**
///
/// ⚠️ Só entram aqui os números que a lei de facto CLAMPA. Um teto que é
/// conforto de arrasto (a `Speed`, a `Accel`) não pertence a esta lista — a
/// caixa de texto passar dele é a feature do slider dual, e um gate que os
/// misturasse pediria tetos onde não há recurso nenhum (§0).
fn clamped_rows() -> Vec<(&'static str, f32)> {
    vec![
        ("INSP_PLAYER_STIFFNESS", RideConfig::MAX_SPRING_STRENGTH),
        ("INSP_PLAYER_DAMPING", RideConfig::MAX_DAMPING),
    ]
}

fn populate_src() -> (PathBuf, String) {
    // `CARGO_MANIFEST_DIR` é `<root>/shells/desktop`.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a raiz da workspace")
        .to_path_buf();
    let p = root.join("crates/ph2d-panel-inspector/src/populate_physics.rs");
    let s = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("o registro das faixas da §14 tem de ser legivel: {p:?}: {e}"));
    (p, s)
}

/// **O DEFAULT registado, e o que o gesto `Add` de facto escreve.**
///
/// ⚠️ A tabela do painel é uma **segunda cópia** dos números que o
/// `PlatformPlayer::default()` semeia, e ela já divergiu em dois campos: a
/// rigidez registava `400` (o valor do `bevy-tnua`, o que shipava antes da
/// `W-Landing`) contra os `2000` que o botão escreve, e o amortecimento `0,5`
/// contra `1,0`. É invisível quase sempre — o `sync_physics` sobrescreve o
/// store ao selecionar —, e é exatamente por isso que ninguém a via envelhecer.
fn seeded_rows() -> Vec<(&'static str, f32)> {
    let seed = ph2d_physics_ecs::PlatformPlayer::default();
    vec![
        ("INSP_PLAYER_FLOAT", seed.float_height),
        ("INSP_PLAYER_CLING", seed.cling_distance),
        ("INSP_PLAYER_STIFFNESS", seed.spring_strength),
        ("INSP_PLAYER_DAMPING", seed.spring_damping),
        ("INSP_PLAYER_SPEED", seed.speed),
        ("INSP_PLAYER_ACCEL", seed.acceleration),
    ]
}

/// **O campo `n` da tupla `(id, default, min, max, step)`** — a porta ÚNICA que
/// os dois gates deste arquivo usam.
///
/// ⚠️ **Ela nasceu DUPLICADA, e o arquivo que a condena é este:** a primeira
/// versão tinha um `registered_max` a ler o campo 3 e um `field(_, n)` genérico,
/// os dois com o mesmo parser da mesma tupla. Duas cópias de *"como se lê esta
/// linha"* é exatamente a doença que este gate persegue, escrita dentro dele.
///
/// ⚠️ Devolve `None` quando a row não é achada, e os dois chamadores tratam isso
/// como FALHA ALTA: uma varredura vazia é indistinguível de um gate verde.
fn field(src: &str, id: &str, n: usize) -> Option<f32> {
    let needle = format!("ids::{id},");
    let line = src.lines().find(|l| l.contains(&needle))?;
    let open = line.find('(')?;
    let close = line[open..].find(')')? + open;
    let fields: Vec<&str> = line[open + 1..close].split(',').map(str::trim).collect();
    fields.get(n)?.replace('_', "").parse::<f32>().ok()
}

/// **O que o painel SEMEIA tem de ser o que o gesto escreve.**
#[test]
fn the_registered_default_is_what_the_add_gesture_writes() {
    let (path, src) = populate_src();
    let mut bad = Vec::new();
    for (id, written) in seeded_rows() {
        let Some(seeded) = field(&src, id, 1) else {
            panic!("a row {id} nao foi achada em {path:?} -- varredura vazia e' verde por vacuo");
        };
        if (seeded - written).abs() > 1.0e-6 {
            bad.push(format!(
                "{id}: o painel semeia {seeded} e o `Add` escreve {written}"
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "a tabela do painel e' a SEGUNDA copia dos defaults da lei, e divergiu:\n  {}",
        bad.join("\n  ")
    );
}

/// ⚠️ **O CONTROLE POSITIVO, e ele vem primeiro** — um extrator partido acha
/// zero rows e o gate fica verde sobre qualquer defeito.
#[test]
fn the_extractor_finds_what_it_is_looking_for() {
    let sample = "        (ids::INSP_PLAYER_STIFFNESS, 400.0, 0.0, 100_000.0, 1.0), // comment\n";
    assert_eq!(
        field(sample, "INSP_PLAYER_STIFFNESS", 3),
        Some(100_000.0),
        "o extrator tem de ler o QUARTO campo da tupla"
    );
    assert_eq!(
        field(sample, "INSP_PLAYER_NOT_A_ROW", 3),
        None,
        "e tem de dizer NAO quando a row nao existe, em vez de devolver lixo"
    );
}

#[test]
fn a_typable_ceiling_never_passes_what_the_law_honours() {
    let (path, src) = populate_src();
    let mut bad = Vec::new();
    for (id, honoured) in clamped_rows() {
        let Some(offered) = field(&src, id, 3) else {
            panic!(
                "a row {id} nao foi achada em {path:?} -- ou ela mudou de casa, ou \
                 este gate esta' a varrer o arquivo errado, e nos dois casos ele \
                 estaria VERDE sobre um teto que ninguem confere"
            );
        };
        if offered > honoured {
            bad.push(format!(
                "{id}: o painel oferece {offered} e a lei honra {honoured} \
                 ({:.1}x) -- o artista arrasta ate' ao fim e nada muda",
                offered / honoured
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "faixa que passa do clamp da lei -- a caixa ACEITA E MENTE:\n  {}",
        bad.join("\n  ")
    );
}
