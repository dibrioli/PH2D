//! ⭐ **O tutorial «fazer» na família MACROS** (`docs/Macros/03_plano_W6.md`, W6a): com uma família de
//! brinquedo que ensina três etapas — o balão pede primeiro o modo Edit, depois a 1.ª etapa por
//! cumprir; «Mostra-me» chega à família com a etapa corrente; o estado que volta (o Ctrl+Z) volta o
//! balão; «Recomeçar» chega à família; uma `Macro` que não está em `Fazer` não tem balão.

use ph2d_a11y::NodeId;
use ph2d_app_macros::{Cena, Familias, Macros};
use ph2d_ecs::{Macro, ModoDaMacro, SimWorld, StableId};
use ph2d_editor_core::macro_edits::{
    CampoDaEtapa, ErroDaMacroView, EtapaView, FamiliaDaMacro, MacroRequest,
};
use ph2d_editor_core::macro_etapas::{EtapasDaFamilia, erros};
use ph2d_editor_core::macro_tutorial::{
    BalaoDoTutorial, EtapaDoTutorial, PedidoDoTutorial, TutorialDaFamilia, balao,
};
use ph2d_macro::{Etapa, Event, LeitorDaFamilia, MacroRecorder, MacroTime, RefRect, Track};
use ph2d_timeline::{TimelineIntent, TimelineState};

const DONO: u64 = 5;
const KS: [usize; 3] = [10, 20, 30];
const DICA: &str = "O vento só sopra dentro da caixa.";

/// O aluno de brinquedo: quantas etapas fez; o que lhe pediram.
#[derive(Default)]
struct Aluno {
    feitas: usize,
    mostradas: Vec<usize>,
    recomecos: usize,
    recusa: bool,
}

impl ph2d_macro::Gravadora for Aluno {
    fn abre(&mut self, rec: &mut MacroRecorder) -> Result<ph2d_macro::TrackId, &'static str> {
        Ok(rec.open_track("motion", DONO, Vec::new()))
    }
    fn passos(&mut self) -> Vec<Vec<u8>> {
        Vec::new()
    }
    fn para(&mut self) -> Vec<Vec<u8>> {
        Vec::new()
    }
}

impl TutorialDaFamilia for Aluno {
    fn avalia(
        &mut self,
        dono: u64,
        _: &Track,
        _: u64,
    ) -> Result<Vec<EtapaDoTutorial>, ErroDaMacroView> {
        assert_eq!(dono, DONO, "o dono é o StableId do objecto");
        Ok(KS
            .iter()
            .enumerate()
            .map(|(i, &k)| EtapaDoTutorial {
                k,
                falta: (i >= self.feitas).then(|| PedidoDoTutorial::Acrescenta {
                    tipo: format!("no{i}"),
                }),
                alvos: vec![NodeId(k as u64)],
                area: None,
            })
            .collect())
    }
    fn mostra(&mut self, _: u64, _: &Track, _: u64, k: usize) -> Result<(), ErroDaMacroView> {
        if self.recusa {
            return Err(erros::erro(erros::ALVO));
        }
        self.mostradas.push(k);
        self.feitas += 1;
        Ok(())
    }
    fn recomeca(&mut self, _: u64, _: &Track) -> Result<(), ErroDaMacroView> {
        self.recomecos += 1;
        self.feitas = 0;
        Ok(())
    }
}

/// O leitor e as etapas do Motion de brinquedo: a família MACROS só lhes pede a chave e a família.
struct Leitor;
impl LeitorDaFamilia for Leitor {
    fn etapas(&self, ev: &[Event]) -> Vec<Etapa> {
        (0..ev.len()).map(|i| Etapa { inicio: i, fim: i }).collect()
    }
    fn e_relogio(&self, _: &[u8]) -> bool {
        false
    }
    fn espera(&self, _: &[Event], _: usize, _: MacroTime, _: u64, _: i64) -> Vec<Event> {
        Vec::new()
    }
    fn desloca(&self, _: &[u8], _: i64) -> Option<Vec<u8>> {
        None
    }
    fn sai_da_gravacao(&self) -> Option<Vec<u8>> {
        None
    }
}
struct EtapasDoBrinquedo;
impl EtapasDaFamilia for EtapasDoBrinquedo {
    fn familia(&self) -> FamiliaDaMacro {
        FamiliaDaMacro::Motion
    }
    fn chave(&self) -> &'static str {
        "motion"
    }
    fn leitor(&self) -> &'static dyn LeitorDaFamilia {
        &Leitor
    }
    fn vistas(&self, _: usize, _: &Track) -> Result<Vec<EtapaView>, ErroDaMacroView> {
        Ok(Vec::new())
    }
    fn edita(&self, _: &mut Track, _: usize, _: &CampoDaEtapa) -> Result<(), ErroDaMacroView> {
        Err(erros::sem_etapa())
    }
}

struct Banca {
    sim: SimWorld,
    aluno: Aluno,
    em_edicao: Option<u64>,
    tl: TimelineState,
    intents: Vec<TimelineIntent>,
    m: Macros,
    bits: u64,
}

impl Banca {
    fn nova(modo: ModoDaMacro) -> Self {
        let mut rec = MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, 0);
        let t = rec.open_track("motion", DONO, Vec::new());
        for i in 0..=KS[2] as u64 {
            rec.push(t, i * 1_000, vec![1]).expect("tempo");
        }
        let mut doc = rec.finish();
        // O autor escreveu o texto do balão da 2.ª etapa do tutorial (k = 20 na gravação).
        doc.tracks[0].edicoes.push(ph2d_macro::EdicaoDeEtapa {
            ancora: KS[1],
            dica: Some(DICA.into()),
            ..Default::default()
        });
        let doc = ph2d_macro::codec::write(&doc);
        let mut sim = SimWorld::new();
        let bits = sim
            .world_mut()
            .spawn((StableId(DONO), Macro { doc, modo }))
            .id()
            .to_bits();
        Self {
            sim,
            aluno: Aluno::default(),
            em_edicao: Some(DONO),
            tl: TimelineState::default(),
            intents: Vec::new(),
            m: Macros::default(),
            bits,
        }
    }

    fn quadro(&mut self) -> Option<BalaoDoTutorial> {
        let mut f = Familias {
            painter: None,
            painter_activo: false,
            motion: &mut self.aluno,
            motion_em_edicao: self.em_edicao,
            motion_na_mao: self.em_edicao.is_some(),
            etapas: &[&EtapasDoBrinquedo],
        };
        let mut c = Cena {
            sim: &mut self.sim,
            timeline: &mut self.tl,
            intents: &mut self.intents,
            partidas: None,
        };
        self.m.quadro(&mut f, &mut c, 0);
        balao()
    }
}

#[test]
fn o_balao_pede_o_edit_depois_a_primeira_etapa_por_cumprir() {
    let mut b = Banca::nova(ModoDaMacro::Fazer);
    b.em_edicao = None;
    let x = b.quadro().expect("balão");
    assert_eq!(
        (x.passo, x.total, x.pedido),
        (0, 3, Some(PedidoDoTutorial::Edita))
    );
    b.em_edicao = Some(DONO);
    let x = b.quadro().expect("balão");
    assert_eq!(x.entity_bits, b.bits);
    assert_eq!((x.passo, x.alvos.clone()), (1, vec![NodeId(10)]));
    assert_eq!(
        x.pedido,
        Some(PedidoDoTutorial::Acrescenta { tipo: "no0".into() })
    );
}

/// ⭐ «Mostra-me» leva a etapa CORRENTE à família; o estado que volta (o desfazer) volta o balão; e
/// «Recomeçar» chega à família.
#[test]
fn mostra_me_desfazer_e_recomecar() {
    let mut b = Banca::nova(ModoDaMacro::Fazer);
    b.quadro();
    b.m.pede(MacroRequest::MostraMe {
        entity_bits: b.bits,
    });
    let x = b.quadro().expect("balão");
    assert_eq!(
        (x.passo, x.dica.as_deref()),
        (2, Some(DICA)),
        "o texto do autor na etapa dele"
    );
    b.m.pede(MacroRequest::MostraMe {
        entity_bits: b.bits,
    });
    assert_eq!(b.quadro().expect("balão").passo, 3);
    assert_eq!(
        b.aluno.mostradas,
        [10, 20],
        "a etapa corrente, pela k da gravação"
    );

    b.aluno.feitas = 1; // o Ctrl+Z do aluno
    assert_eq!(b.quadro().expect("balão").passo, 2);

    b.aluno.feitas = 3;
    let fim = b.quadro().expect("balão");
    assert_eq!((fim.passo, fim.pedido), (3, None), "feito");

    b.m.pede(MacroRequest::Recomeca {
        entity_bits: b.bits,
    });
    assert_eq!(b.quadro().expect("balão").passo, 1);
    assert_eq!(b.aluno.recomecos, 1);
}

#[test]
fn a_recusa_de_um_botao_aparece_no_balao() {
    let mut b = Banca::nova(ModoDaMacro::Fazer);
    b.aluno.recusa = true;
    b.m.pede(MacroRequest::MostraMe {
        entity_bits: b.bits,
    });
    let x = b.quadro().expect("balão");
    assert_eq!(x.erro.map(|e| e.chave), Some(erros::ALVO));
    assert_eq!(x.passo, 1);
}

#[test]
fn fora_do_modo_fazer_nao_ha_balao() {
    for modo in [ModoDaMacro::Fiel, ModoDaMacro::Explicativo] {
        let mut b = Banca::nova(modo);
        assert_eq!(b.quadro(), None, "{modo:?}");
    }
}
