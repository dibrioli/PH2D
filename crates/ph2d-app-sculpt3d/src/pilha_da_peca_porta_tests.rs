//! Gates da porta do painel sobre a pilha (`docs/3D/30` §4, W3).

use super::super::tests::pilha_rica;
use super::*;

const N: usize = 1_100;

/// Uma operação e o passo de desfazer dela — e as duas direcções do passo devolvem a pilha AO BIT.
fn ida_e_volta(
    p: &mut PilhaDaPeca,
    quando: &str,
    op: impl FnOnce(&mut PilhaDaPeca) -> TrocaDaPilha,
) {
    let antes = p.clone();
    let passo = op(p);
    assert!(p.sincronizada(), "{quando}: a operação dessincronizou");
    let depois = p.clone();
    assert_ne!(
        &depois, &antes,
        "{quando}: o CONTROLO — a operação mudou a pilha"
    );
    let inversa = p.troca_estrutura(passo).expect("desfaz");
    assert_eq!(
        p, &antes,
        "{quando}: o desfazer não devolveu a pilha ao bit"
    );
    p.troca_estrutura(inversa).expect("refaz");
    assert_eq!(
        p, &depois,
        "{quando}: o refazer não devolveu a pilha ao bit"
    );
}

/// ⭐⭐⭐ **GATE — todo passo do painel desfaz e refaz AO BIT** (pilha + planos): nova, máscara,
/// ajuste, duplica, apaga (o plano tirado volta) e o metadado.
#[test]
fn cada_passo_do_painel_desfaz_e_refaz_ao_bit() {
    let (mut p, [_, mult, over, _]) = pilha_rica(N);
    let nada = BTreeMap::new;
    ida_e_volta(&mut p, "nova", |p| {
        let a = p.pilha().clone();
        p.nova_camada("nova").expect("nova");
        TrocaDaPilha::de(a, nada())
    });
    ida_e_volta(&mut p, "máscara", |p| {
        let a = p.pilha().clone();
        p.nova_mascara(mult).expect("máscara");
        TrocaDaPilha::de(a, nada())
    });
    ida_e_volta(&mut p, "ajuste", |p| {
        let a = p.pilha().clone();
        p.novo_ajuste(AdjustmentKind::Invert).expect("ajuste");
        TrocaDaPilha::de(a, nada())
    });
    ida_e_volta(&mut p, "duplica", |p| {
        let a = p.pilha().clone();
        p.duplica(mult).expect("duplica");
        TrocaDaPilha::de(a, nada())
    });
    ida_e_volta(&mut p, "apaga (com a máscara)", |p| {
        let a = p.pilha().clone();
        let tirados = p.apaga(over).expect("apaga");
        assert_eq!(tirados.len(), 2, "a camada e a máscara dela");
        TrocaDaPilha::de(a, tirados)
    });
    ida_e_volta(&mut p, "metadado", |p| {
        let mut nova = p.pilha().clone();
        nova.set_opacity(mult, 0.25);
        nova.set_blend_mode(mult, BlendMode::Screen);
        let velha = p.troca_metadado(nova).expect("metadado");
        TrocaDaPilha::de(velha, nada())
    });
}

/// ⭐⭐⭐ **GATE — o metadado não muda a ESTRUTURA, não tira a base do fundo, e espera o traço.**
#[test]
fn o_metadado_recusa_estrutura_base_e_traco_aberto() {
    let (mut p, [base, mult, ..]) = pilha_rica(N);
    let antes = p.clone();

    let mut com_mais = p.pilha().clone();
    com_mais.add_raster("intrusa", 1024, 2);
    assert_eq!(
        p.troca_metadado(com_mais),
        Err(RecusaDaPilha::MudaAEstrutura)
    );

    let mut relevo = p.pilha().clone();
    if let Some(c) = relevo.get_mut(mult) {
        c.has_relief = true;
    }
    assert_eq!(p.troca_metadado(relevo), Err(RecusaDaPilha::MudaAEstrutura));

    let mut base_acima = p.pilha().clone();
    base_acima.move_up(base);
    assert_eq!(p.troca_metadado(base_acima), Err(RecusaDaPilha::ABase));
    assert_eq!(p, antes, "as recusas deixam a pilha como estava");

    // O pen-down regista a camada emprestada (`trabalho_da_activa`).
    p.em_traco = Some(mult);
    let mut opaca = p.pilha().clone();
    opaca.set_opacity(mult, 0.1);
    assert_eq!(
        p.troca_metadado(opaca.clone()),
        Err(RecusaDaPilha::TracoAberto)
    );
    assert_eq!(p.nova_camada("x"), Err(RecusaDaPilha::TracoAberto));
    p.fim_do_traco();
    // ⛔ CONTROLO: a mesma mudança, sem traço, entra.
    assert!(p.troca_metadado(opaca).is_ok());
}

/// ⭐⭐ **GATE — a BASE fica**: não se apaga, e a cópia dela não leva o relevo (até à W4 ele é só
/// da base: duas camadas com relevo partiriam o `relevo_composto`).
#[test]
fn a_base_fica_e_a_copia_nao_leva_o_relevo() {
    let (mut p, [base, ..]) = pilha_rica(N);
    assert_eq!(p.apaga(base), Err(RecusaDaPilha::ABase));
    if let Some(pl) = p.planos.get_mut(&base) {
        pl.relevo = Some(vec![[0.5, 1.0]; N]);
    }
    if let Some(c) = p.pilha.get_mut(base) {
        c.has_relief = true;
    }
    let copia = p.duplica(base).expect("duplica");
    assert!(p.plano(copia).is_some_and(|c| c.relevo().is_none()));
    assert!(!p.pilha().get(copia).is_some_and(|c| c.has_relief));
    assert_eq!(
        p.plano(copia).map(|c| c.rgba8(N)),
        p.plano(base).map(|c| c.rgba8(N)),
        "a cor vem"
    );
    assert!(p.sincronizada());
    assert!(p.relevo_composto().is_some(), "o relevo continua o da base");
}

/// ⭐ **GATE — um ajuste novo nasce como o do 2D e não rouba a activa** (um ajuste não se pinta).
#[test]
fn um_ajuste_novo_nasce_como_o_do_2d_e_nao_rouba_a_activa() {
    let (mut p, _) = pilha_rica(N);
    let activa = p.pilha().active();
    let curvas = p.novo_ajuste(AdjustmentKind::Curves).expect("curvas");
    assert_eq!(p.pilha().active(), activa);
    let mut esperado = AdjustmentParams::Curves(Default::default());
    ph2d_tool_painter::seed_user_adjustment(&mut esperado);
    assert_eq!(
        p.pilha.adjustment_mut(curvas).map(|a| a.params.clone()),
        Some(esperado)
    );
}
