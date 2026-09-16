//! A ponte dos scripts contra o LEDGER de verdade — o `Ctrl+Z` e o rebobinar são o que ela decide,
//! então os gates andam com a `settle` a correr a cada quadro, como no produto.

use std::sync::atomic::{AtomicU32, Ordering};

use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_preview_drive::{Driver, PreviewDrive};
use ph2d_script::{LuauScript, ScriptHost};

use super::{frame, rewind};

fn script_file(source: &str) -> String {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!("ph2d_script_bridge_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp");
    let p = dir.join(format!("s{}.luau", N.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&p, source).expect("escreve");
    p.to_string_lossy().into_owned()
}

/// Anda `+1` por tique durante `limite` tiques, e depois PÁRA — o condutor que acaba de mexer.
const ANDA_E_PARA: &str = r#"
ph2d.property("limite", 3)
function init(self) self.n = 0 end
function update(self, dt)
  if self.n < self.limite then
    self.n = self.n + 1
    ph2d.set(self.id, "x", ph2d.get(self.id, "x") + 1)
  end
end
"#;

struct Cena {
    sim: SimWorld,
    host: ScriptHost,
    drive: PreviewDrive,
    e: ph2d_ecs::Entity,
}

fn cena(source: &str) -> Cena {
    let mut sim = SimWorld::new();
    let mut t = Transform::IDENTITY;
    t.translation.x = 10.0;
    let e = sim
        .world_mut()
        .spawn((t, StableId(1), LuauScript::at(script_file(source))))
        .id();
    Cena {
        sim,
        host: ScriptHost::new().expect("vm"),
        drive: PreviewDrive::default(),
        e,
    }
}

impl Cena {
    /// Um quadro do produto: a ponte, e depois a `settle` do `post_frame_undo`.
    fn quadro(&mut self, playing: bool) -> super::ScriptFrame {
        let f = frame(
            &mut self.host,
            &mut self.sim,
            &mut self.drive,
            playing,
            1,
            1.0 / 60.0,
            &[],
        );
        self.drive.settle();
        f
    }

    fn x(&self) -> f32 {
        self.sim
            .world()
            .get::<Transform>(self.e)
            .expect("pose")
            .translation
            .x
    }

    /// O `x` que a CAPTURA (undo + save) vê — o autorado.
    fn x_documento(&mut self) -> f32 {
        let live = self.drive.substitute_authored(&mut self.sim);
        let x = self.x();
        PreviewDrive::restore_live(&mut self.sim, &live);
        x
    }
}

#[test]
fn parado_nada_corre_e_nada_e_conduzido() {
    let mut c = cena(ANDA_E_PARA);
    let f = c.quadro(false);
    assert_eq!(f.calls, 0);
    assert_eq!(c.x(), 10.0);
    assert!(c.drive.is_empty());
    assert_eq!(c.host.scene().live_count(), 0);
}

/// ⭐⭐⭐ **A corrida vê-se e não se guarda; rebobinar devolve a pose do artista.**
#[test]
fn a_corrida_move_o_objecto_e_o_documento_fica_onde_o_artista_o_pos() {
    let mut c = cena(ANDA_E_PARA);
    c.quadro(true);
    c.quadro(true);
    assert_eq!(c.x(), 12.0, "o mundo mostra a corrida");
    assert_eq!(c.x_documento(), 10.0, "a captura vê o autorado");
    assert_eq!(rewind(&mut c.host, &mut c.sim, &mut c.drive), 1);
    assert_eq!(c.x(), 10.0, "rebobinar devolve a pose");
    assert!(c.drive.is_empty());
    c.quadro(true);
    assert_eq!(
        c.x(),
        11.0,
        "e a corrida seguinte parte do zero — o self renasceu"
    );
}

/// ⛔⛔ **A linha do meio da tabela:** um script que PAROU de mexer continua a conduzir. Sem ela a
/// `settle` promovia a pose da corrida a documento, e o rebobinar ficava sem destino.
#[test]
fn um_script_que_parou_de_mexer_nao_vira_documento() {
    let mut c = cena(ANDA_E_PARA);
    for _ in 0..10 {
        c.quadro(true);
    }
    assert_eq!(c.x(), 13.0, "andou três e parou");
    assert_eq!(
        c.x_documento(),
        10.0,
        "sete quadros parado e o documento continua o do artista"
    );
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.x(), 10.0);
}

#[test]
fn pausado_a_meio_a_pose_fica_e_o_rebobinar_ainda_a_devolve() {
    let mut c = cena(ANDA_E_PARA);
    c.quadro(true);
    for _ in 0..5 {
        let f = c.quadro(false);
        assert_eq!(f.calls, 0);
    }
    assert_eq!(c.x(), 11.0, "congelado");
    assert_eq!(c.x_documento(), 10.0);
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.x(), 10.0);
}

/// ⚠️ **Um arrasto feito na PAUSA é a outra mão**, e fica como o novo autorado.
#[test]
fn arrastar_na_pausa_muda_para_onde_o_rebobinar_volta() {
    let mut c = cena(ANDA_E_PARA);
    c.quadro(true);
    c.quadro(false);
    c.sim
        .world_mut()
        .get_mut::<Transform>(c.e)
        .expect("pose")
        .translation
        .x = 50.0;
    c.quadro(false);
    assert_eq!(c.x_documento(), 50.0, "o arrasto é documento");
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.x(), 50.0, "e é para lá que se volta");
}

#[test]
fn um_script_que_nunca_mexe_nao_abre_conducao_nenhuma() {
    let mut c = cena("function update(self, dt) end");
    c.quadro(true);
    c.quadro(true);
    assert!(
        !c.drive
            .drivers_of(c.e.to_bits())
            .contains(&Driver::ScriptPose),
        "`still_driving` nunca cria uma entrada"
    );
}

#[test]
fn os_sinais_emitidos_saem_com_quem_os_emitiu() {
    let mut c = cena(r#"function update(self, dt) ph2d.emit("bip") end"#);
    let f = c.quadro(true);
    assert_eq!(f.emitted, vec![(c.e.to_bits(), "bip".to_owned())]);
}

#[test]
fn a_falha_sai_uma_vez_com_a_mensagem() {
    let mut c = cena(r#"function update(self, dt) error("partiu") end"#);
    let f = c.quadro(true);
    assert_eq!(f.failed.len(), 1);
    assert!(f.failed[0].1.contains("partiu"), "{}", f.failed[0].1);
    assert!(
        c.quadro(true).failed.is_empty(),
        "parado, não repete a queixa"
    );
}

#[test]
fn dois_tiques_num_quadro_sao_duas_chamadas() {
    let mut c = cena(ANDA_E_PARA);
    let f = frame(
        &mut c.host,
        &mut c.sim,
        &mut c.drive,
        true,
        2,
        1.0 / 60.0,
        &[],
    );
    assert_eq!(f.calls, 3, "init + dois update");
    assert_eq!(c.x(), 12.0);
}
