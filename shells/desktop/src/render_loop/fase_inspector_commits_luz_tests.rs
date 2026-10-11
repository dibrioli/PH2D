//! ⭐⭐ **G8 da shell — a fila da LUZ chega ao MUNDO** (Luz 2D, W4): a porta que o quadro usa para
//! aplicar as secções (`top20_commits::aplicar`) escreve-a na `Luz`. O dreno do barramento que
//! enche a fila tem o gate dele ao lado (`fase_bus_inspector_luz_tests.rs`).

use ph2d_editor_core::luz_edits::LuzFieldEdit;

#[test]
fn a_fila_da_luz_chega_ao_mundo() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let bits = ph2d_app_light2d::object_add::add(ph2d_app_light2d::object_add::LUZ, &mut sim)
        .expect("é da luz");
    let edit = LuzFieldEdit::Intensidade(1.5);

    let pd_luz = vec![(bits, edit)];
    // A aplicação: a porta das secções escreve na `Luz` e diz que mudou.
    let mudou = super::top20_commits::aplicar(
        &mut sim,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &pd_luz,
        &[],
        &[],
        &[],
        &[],
        &[],
    );
    assert!(mudou, "a aplicação das secções não tocou no mundo");
    let luz = sim
        .world()
        .get::<ph2d_light2d_ecs::Luz>(ph2d_ecs::Entity::from_bits(bits))
        .copied()
        .expect("a luz existe");
    assert_eq!(luz.intensidade, 1.5);
}
