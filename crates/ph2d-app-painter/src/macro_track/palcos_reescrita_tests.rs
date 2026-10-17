//! W8 item 2b, `docs/Macros/05_plano_W8.md` §2b — **editar uma etapa reescreve a imagem do artista**:
//! onde a imagem ainda é o fim antigo da gravação passa ao fim novo; o resto é do artista; um número de
//! transação liga o passo do Painter (ou a escrita da sprite) ao passo global da edição.

use super::*;
use crate::macro_track::EscreveSprite;
use std::cell::{Cell, RefCell};

impl Clipe {
    /// Um quadro em `t` e as reescritas dele (a sprite por `le`/`escreve`, ou o Painter `t`).
    fn quadro_e_reescreve(
        &mut self,
        q: u32,
        painter: Option<&mut PainterTool>,
        le: &mut LeSprite<'_>,
        escreve: &mut EscreveSprite<'_, Placa>,
    ) -> Option<u64> {
        let mut p = painter;
        self.quadro(
            f64::from(q) / 60.0,
            Atuais {
                painter: p.as_deref(),
                le_sprite: Some(&mut *le),
            },
        );
        let t = self.palcos.reescreve(p.as_deref_mut(), le, escreve, &mut self.placa);
        // O passo global regista-se e leva-a (a shell, no `post_frame_undo`).
        assert_eq!(self.palcos.toma_a_transacao(), t, "a transação espera o passo global");
        t
    }

    /// Troca o `Macro.doc` e corre quadros até a reescrita acabar; devolve as transações.
    fn edita_e_espera(
        &mut self,
        doc: &[u8],
        mut painter: Option<&mut PainterTool>,
        le: &mut LeSprite<'_>,
        escreve: &mut EscreveSprite<'_, Placa>,
    ) -> Vec<u64> {
        self.world.entity_mut(self.e).insert(Macro {
            doc: doc.to_vec(),
            modo: ModoDaMacro::Fiel,
        });
        let t0 = std::time::Instant::now();
        let mut transacoes = Vec::new();
        let mut q = 1;
        loop {
            transacoes.extend(self.quadro_e_reescreve(q, painter.as_deref_mut(), le, escreve));
            if q > 1 && !self.palcos.a_reescrever() {
                return transacoes;
            }
            assert!(t0.elapsed().as_secs() < 120, "a reescrita não acabou em 120 s");
            std::thread::sleep(std::time::Duration::from_millis(2));
            q += 1;
        }
    }
}

/// A imagem que a lei espera: `atual` onde ainda é `de` passa a `para`.
fn reescrita(atual: &[u8], de: &[u8], para: &[u8]) -> Vec<u8> {
    atual
        .as_chunks::<4>()
        .0
        .iter()
        .zip(de.as_chunks::<4>().0.iter().zip(para.as_chunks::<4>().0))
        .flat_map(|(a, (d, p))| if a == d { p } else { a }.to_vec())
        .collect()
}

/// ⭐ **A sprite sem o Painter**: editar a cor da 2.ª pincelada reescreve, uma vez, a imagem onde ela
/// ainda é a gravação (a pintura depois do Parar fica) e devolve a transação; editar de volta reescreve de
/// volta; com a imagem já devolvida (o desfazer global devolve as duas coisas), editar de volta não
/// escreve nada. ⛔ VERMELHO sem a reescrita: a imagem não muda e o último quadro salta ao parar.
#[test]
fn editar_uma_etapa_reescreve_a_sprite_e_devolve_a_transacao() {
    let (doc, _, fim) = gravada_no_papel();
    let editada = com_a_etapa_editada(&doc);
    let fim_editado = fim_de(&editada);
    let atual = pintada_depois(&fim);
    let img = RefCell::new(atual.clone());
    let escritas = Cell::new(0);
    let mut le = |_: u64, _: &mut Placa| Some((img.borrow().clone(), LADO, LADO));
    let mut escreve = |_: u64, rgba: Vec<u8>, _: u32, _: u32, _: &mut Placa| {
        escritas.set(escritas.get() + 1);
        *img.borrow_mut() = rgba;
    };
    let mut c = Clipe::novo(&doc);
    c.espera_o_fim(0.0, None, &mut le);

    let t = c.edita_e_espera(&editada, None, &mut le, &mut escreve);
    assert_eq!((t.len(), escritas.get()), (1, 1), "uma escrita, uma transação");
    let depois = img.borrow().clone();
    assert!(
        depois == reescrita(&atual, &fim, &fim_editado),
        "a sprite não foi reescrita onde era a gravação"
    );
    assert_ne!(depois, atual, "controlo: a edição mudou a imagem");

    // Editar de volta (outra edição, a imagem ficou): reescreve de volta.
    let t2 = c.edita_e_espera(&doc, None, &mut le, &mut escreve);
    assert_eq!((t2.len(), escritas.get()), (1, 2), "editar de volta reescreve");
    assert!(*img.borrow() == atual, "editar de volta não devolveu a imagem");
    assert_ne!(t2[0], t[0], "a transação de cada reescrita é outra");

    // O desfazer global: a imagem e o documento voltam juntos — nada a escrever.
    c.edita_e_espera(&editada, None, &mut le, &mut escreve);
    *img.borrow_mut() = atual.clone();
    let t3 = c.edita_e_espera(&doc, None, &mut le, &mut escreve);
    assert!(t3.is_empty(), "com a imagem já devolvida, a reescrita escreveu");
    assert_eq!(escritas.get(), 3, "com a imagem já devolvida, a reescrita escreveu");
}

/// ⭐ **O Painter ligado à sprite**: a gravação feita nele (como o artista grava), a água parada; editar a
/// cor reescreve o documento num passo do Painter com a MESMA transação que a reescrita devolve à shell; o
/// desfazer do Painter devolve o fim antigo.
#[test]
fn editar_uma_etapa_reescreve_o_painter_ligado_num_passo_com_a_transacao() {
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, _) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado(LADO, e.to_bits(), 2)
            .expect("grava");
    assert_eq!(t.bound_document(), Some(e.to_bits()), "controlo: o Painter está ligado");
    let t0 = std::time::Instant::now();
    while !t.agua_parada() {
        assert!(t0.elapsed().as_secs() < 120, "a água do editor não parou");
        drop(t.take_preview_arc());
        t.on_tick(16.0);
    }
    let fim = fim_de(&doc);
    assert_eq!(t.imagem_aplicada().expect("documento").0, fim, "controlo: o editor é o fim");
    world.entity_mut(e).insert(Macro {
        doc: doc.clone(),
        modo: ModoDaMacro::Fiel,
    });
    let mut c = Clipe {
        world,
        e,
        palcos: PalcosDoPainter::default(),
        placa: Placa::default(),
        verdade: ph2d_tool_painter::macro_track::em_cena::MacroEmCena::de(&doc)
            .expect("lê-se")
            .expect("trilha"),
        palco: Vec::new(),
    };
    let mut nunca = |_: u64, _: &mut Placa| -> Option<(Vec<u8>, u32, u32)> {
        panic!("com o Painter ligado não se lê a sprite")
    };
    let mut nao_escreve = |_: u64, _: Vec<u8>, _: u32, _: u32, _: &mut Placa| {
        panic!("com o Painter ligado não se escreve a sprite")
    };
    // O palco nasce com a gravação (o editor está no fim dela).
    assert_eq!(c.quadro_e_reescreve(0, Some(&mut t), &mut nunca, &mut nao_escreve), None);
    let editada = com_a_etapa_editada(&doc);
    let transacoes = c.edita_e_espera(&editada, Some(&mut t), &mut nunca, &mut nao_escreve);
    assert_eq!(transacoes.len(), 1, "uma transação");
    assert_eq!(
        t.transacao_do_topo(false),
        Some(transacoes[0]),
        "o passo do Painter não leva a transação devolvida"
    );
    assert_eq!(
        t.imagem_aplicada().expect("documento").0,
        fim_de(&editada),
        "o Painter não foi reescrito para o fim editado"
    );
    assert!(t.undo_last(), "o desfazer");
    assert_eq!(t.imagem_aplicada().expect("documento").0, fim, "o desfazer não devolveu o fim");
}

/// O cálculo do fim do produto, devagar (as duas edições chegam antes de a 1.ª reescrever).
fn devagar(
    t: &ph2d_macro::Track,
    p: &std::sync::atomic::AtomicBool,
) -> super::super::super::fim::Resultado {
    std::thread::sleep(std::time::Duration::from_millis(300));
    super::super::super::fim::calcula(t, p)
}

/// **Duas edições antes de a 1.ª reescrever**: a imagem ainda tem o fim ORIGINAL, e é dele que se
/// reescreve para o fim da 2.ª. ⛔ VERMELHO se a 2.ª edição trocasse o «de» pelo fim da 1.ª (que a imagem
/// nunca teve): nada casava e a imagem ficava velha.
#[test]
fn duas_edicoes_seguidas_reescrevem_do_fim_que_a_imagem_tem() {
    let (doc, _, fim) = gravada_no_papel();
    let editada = com_a_etapa_editada(&doc);
    let mut lido = ph2d_macro::codec::read(&editada).expect("lê-se");
    ph2d_tool_painter::macro_track::etapas::edita_cor(&mut lido.tracks[0], 0, [200, 40, 40])
        .expect("a etapa edita-se");
    let duas = ph2d_macro::codec::write(&lido);
    let fim_das_duas = fim_de(&duas);
    let atual = pintada_depois(&fim);
    let img = RefCell::new(atual.clone());
    let mut le = |_: u64, _: &mut Placa| Some((img.borrow().clone(), LADO, LADO));
    let mut escreve =
        |_: u64, rgba: Vec<u8>, _: u32, _: u32, _: &mut Placa| *img.borrow_mut() = rgba;
    let mut c = Clipe::novo(&doc);
    c.espera_o_fim(0.0, None, &mut le);
    c.palcos.calcula_o_fim = Some(devagar);
    c.world.entity_mut(c.e).insert(Macro {
        doc: editada,
        modo: ModoDaMacro::Fiel,
    });
    assert_eq!(c.quadro_e_reescreve(1, None, &mut le, &mut escreve), None);
    assert!(c.palcos.a_reescrever(), "controlo: a 1.ª ainda espera");
    let t = c.edita_e_espera(&duas, None, &mut le, &mut escreve);
    assert_eq!(t.len(), 1, "uma reescrita para as duas edições");
    assert!(
        *img.borrow() == reescrita(&atual, &fim, &fim_das_duas),
        "a imagem não foi reescrita do fim original para o das duas edições"
    );
}

/// **A água do editor ainda corre** quando a etapa se edita: a reescrita espera que ela pare (a água a
/// correr não é nem o fim antigo nem o novo) e depois a imagem é, ao byte, o fim editado. ⛔ VERMELHO sem
/// a espera: reescrevia só o que já tinha assentado, e o resto da água assentava na cor velha.
#[test]
fn a_reescrita_espera_a_agua_do_editor_parar() {
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, _) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado(LADO, e.to_bits(), 2)
            .expect("grava");
    assert!(!t.agua_parada(), "controlo: a água ainda corre no Parar");
    world.entity_mut(e).insert(Macro {
        doc: doc.clone(),
        modo: ModoDaMacro::Fiel,
    });
    let mut c = Clipe {
        world,
        e,
        palcos: PalcosDoPainter::default(),
        placa: Placa::default(),
        verdade: ph2d_tool_painter::macro_track::em_cena::MacroEmCena::de(&doc)
            .expect("lê-se")
            .expect("trilha"),
        palco: Vec::new(),
    };
    let mut nunca = |_: u64, _: &mut Placa| -> Option<(Vec<u8>, u32, u32)> { None };
    let mut nao_escreve = |_: u64, _: Vec<u8>, _: u32, _: u32, _: &mut Placa| {
        panic!("com o Painter ligado não se escreve a sprite")
    };
    assert_eq!(c.quadro_e_reescreve(0, Some(&mut t), &mut nunca, &mut nao_escreve), None);
    let editada = com_a_etapa_editada(&doc);
    c.world.entity_mut(c.e).insert(Macro {
        doc: editada.clone(),
        modo: ModoDaMacro::Fiel,
    });
    let t0 = std::time::Instant::now();
    let mut q = 1;
    let transacao = loop {
        let parada = t.agua_parada();
        let r = c.quadro_e_reescreve(q, Some(&mut t), &mut nunca, &mut nao_escreve);
        if let Some(x) = r {
            assert!(parada, "a reescrita correu com a água do editor a correr (quadro {q})");
            break x;
        }
        assert!(t0.elapsed().as_secs() < 120, "a reescrita não acabou em 120 s");
        drop(t.take_preview_arc());
        t.on_tick(16.0);
        q += 1;
    };
    assert_eq!(t.transacao_do_topo(false), Some(transacao));
    assert_eq!(
        t.imagem_aplicada().expect("documento").0,
        fim_de(&editada),
        "a imagem do editor não é o fim editado"
    );
}
