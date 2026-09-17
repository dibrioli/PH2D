//! A secção PARTICLES: o instantâneo que o painel lê e o dreno que ele devolve.

use ph2d_ecs::{EmissionShape, ParticleEmitter, ParticleSpace, SimWorld, Transform};
use ph2d_editor_core::particles_edits::{
    PARTICLES_NUMBERS, PARTICLES_TEXTS, ParticlesFieldEdit as E, ParticlesNumber as N,
    ParticlesText as T,
};

use super::{apply, build_info};

fn cena(cfg: ParticleEmitter) -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn((Transform::IDENTITY, cfg)).id();
    (sim, e.to_bits())
}

/// ⚠️ **Quem não tem o componente não tem secção** — ADR-0166. Sem isto o Inspector mostrava um
/// emissor a toda a gente, com os números do `Default` a fingir que são do objecto.
#[test]
fn quem_nao_tem_emissor_nao_tem_seccao() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(Transform::IDENTITY).id();
    assert!(build_info(&sim, e.to_bits(), 1, true, 0).is_none());
}

/// ⭐⭐ **Cada NÚMERO da tabela vai ao campo dele, e a ida-e-volta fecha.**
///
/// ⚠️ **A prova é por PAR e não por campo**: escrevo um valor distinto em cada número, releio o
/// instantâneo e exijo que cada um leia o que foi escrito. Um par trocado — o defeito que uma
/// segunda lista escrita à mão produz — reprova aqui e não em mais lado nenhum.
#[test]
fn cada_numero_vai_ao_campo_dele_e_volta() {
    let (mut sim, bits) = cena(ParticleEmitter::default());
    for (i, n) in PARTICLES_NUMBERS.into_iter().enumerate() {
        // ⚠️ Valores DISTINTOS e nenhum igual a um default: com dois iguais, dois campos trocados
        // leem-se certos.
        #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
        let v = 3.0 + i as f32;
        assert!(apply(&mut sim, bits, &E::Number(n, v)), "{n:?} não mexeu");
    }
    let info = build_info(&sim, bits, 1, true, 0).expect("tem emissor");
    for (i, n) in PARTICLES_NUMBERS.into_iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
        let v = 3.0 + i as f32;
        assert!(
            (info.number(n) - v).abs() < 1e-6,
            "{n:?}: leu {} e escreveu {v}",
            info.number(n)
        );
    }
}

/// ⭐ **Os quatro NOMES de sinal também são um por campo.**
#[test]
fn cada_texto_vai_ao_campo_dele_e_volta() {
    let (mut sim, bits) = cena(ParticleEmitter::default());
    for (i, t) in PARTICLES_TEXTS.into_iter().enumerate() {
        assert!(apply(&mut sim, bits, &E::Text(t, format!("sinal{i}"))));
    }
    let info = build_info(&sim, bits, 1, true, 0).expect("tem emissor");
    for (i, t) in PARTICLES_TEXTS.into_iter().enumerate() {
        assert_eq!(info.text(t), format!("sinal{i}"), "{t:?}");
    }
}

/// ⭐⭐⭐ **Escrever o MESMO valor não muda o documento** — e isto não é higiene de barramento: a
/// ponte da corrida compara o config para decidir se o emissor RENASCE, logo um componente tocado
/// por nada apagaria o penacho a cada quadro em que o rato passa por cima de um campo.
#[test]
fn escrever_o_mesmo_valor_nao_mexe_no_documento() {
    let (mut sim, bits) = cena(ParticleEmitter::default());
    let d = ParticleEmitter::default();
    assert!(!apply(&mut sim, bits, &E::Number(N::Life, d.life)));
    assert!(!apply(&mut sim, bits, &E::Emitting(d.emitting)));
    assert!(!apply(&mut sim, bits, &E::Shape(d.shape.index())));
    assert!(!apply(&mut sim, bits, &E::Color(false, d.color)));
    assert!(!apply(
        &mut sim,
        bits,
        &E::Text(T::StartOn, d.start_on.clone())
    ));
    assert!(apply(&mut sim, bits, &E::Number(N::Life, d.life + 1.0)));
}

/// ⚠️ **Uma forma que não existe não escreve nada** — o índice vem de uma fileira de botões, e um
/// botão a mais no painel não pode pôr o componente num estado que o `EmissionShape` não tem.
#[test]
fn uma_forma_fora_da_lista_e_recusada() {
    // ⚠️⚠️ **A fixtura NÃO começa no `Point`**, e a prova de mutação é que o disse: uma cura falsa
    // que caísse no `Point` escreveria exactamente o que já lá estava, e *«recusou»* e *«escreveu o
    // mesmo»* devolvem os dois `false`. Começando no `Ring`, só a recusa deixa a forma quieta.
    let (mut sim, bits) = cena(ParticleEmitter {
        shape: EmissionShape::Ring,
        ..ParticleEmitter::default()
    });
    assert!(!apply(&mut sim, bits, &E::Shape(9)));
    assert_eq!(
        build_info(&sim, bits, 1, true, 0).expect("tem").shape,
        EmissionShape::Ring.index(),
        "um índice fora da lista mexeu na forma"
    );
    assert!(apply(&mut sim, bits, &E::Shape(3)));
    let info = build_info(&sim, bits, 1, true, 0).expect("tem emissor");
    assert_eq!(info.shape, EmissionShape::Rect.index());
}

/// ⚠️ **O espaço é um par, e o painel manda o ÍNDICE** — `0` mundo, o resto local.
#[test]
fn o_espaco_le_se_do_indice() {
    let (mut sim, bits) = cena(ParticleEmitter::default());
    assert!(apply(&mut sim, bits, &E::Space(1)));
    assert_eq!(
        sim.world()
            .get::<ParticleEmitter>(ph2d_ecs::Entity::from_bits(bits))
            .map(|c| c.space),
        Some(ParticleSpace::Local)
    );
    assert_eq!(build_info(&sim, bits, 1, true, 0).expect("tem").space, 1);
}

/// ⭐⭐ **As duas grandezas da CORRIDA chegam de fora e aparecem no instantâneo** — sem elas o
/// painel não consegue dizer a diferença entre *«não configurei»* e *«o relógio está parado»*, que
/// é a metade da secção que um campo sozinho nunca diz.
#[test]
fn o_relogio_e_as_vivas_chegam_ao_painel() {
    let (sim, bits) = cena(ParticleEmitter::default());
    let parado = build_info(&sim, bits, 2, false, 0).expect("tem emissor");
    assert!(!parado.clock_playing);
    assert_eq!(parado.alive, 0);
    assert_eq!(parado.selected_count, 2);
    let a_correr = build_info(&sim, bits, 1, true, 17).expect("tem emissor");
    assert!(a_correr.clock_playing);
    assert_eq!(a_correr.alive, 17);
}

/// ⚠️ **Um campo INTEIRO arredonda e nunca fica negativo** — o campo de número do painel entrega um
/// `f32` e o componente guarda um `u32`; um `-1` truncado daria `4 294 967 295` partículas.
#[test]
fn um_campo_inteiro_arredonda_e_nao_fica_negativo() {
    let (mut sim, bits) = cena(ParticleEmitter::default());
    apply(&mut sim, bits, &E::Number(N::Amount, -3.0));
    assert_eq!(build_info(&sim, bits, 1, true, 0).expect("tem").amount, 0.0);
    apply(&mut sim, bits, &E::Number(N::Amount, 12.6));
    assert_eq!(
        build_info(&sim, bits, 1, true, 0).expect("tem").amount,
        13.0
    );
}
