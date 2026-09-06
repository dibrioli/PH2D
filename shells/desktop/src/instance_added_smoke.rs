//! ⭐⭐⭐ **`PH2D_INSTANCE_SMOKE=6` — DAR UMA PEÇA NOVA AO COMPONENTE** (ADR-0164 / plano F5.11).
//!
//! # O que ela põe na tela
//!
//! **A mesma cena da `=5`** — um *Robot* que é receita e três cópias dele, cada uma com um corpo
//! azul e um braço laranja. ⚠️ **A montagem é literalmente a mesma função**
//! ([`crate::instance_removed_smoke::spawn_robot_scene`]): as duas cenas ensinam as duas metades da
//! mesma pergunta — *o que esta cópia tem de diferente da receita* —, e duas montagens divergiriam
//! no dia em que uma delas ganhasse uma peça.
//!
//! # ⚠️ Ela é o ESPELHO da `=5`, e é isso que a torna curta
//!
//! Lá o artista **tira** uma peça de uma cópia e a devolve. Aqui ele **acrescenta** uma e a dá à
//! receita — e as outras duas cópias recebem-na. *A `=5` mostra o que uma cópia perde; esta mostra o
//! que o componente ganha.*
//!
//! ⚠️ **O gesto que cria a peça é *Duplicate*, e não *Add Child*:** os dois deixam uma entidade sem
//! elo (que é o que uma peça acrescentada É), mas o *Add Child* deixa um objecto **vazio** — um anel
//! fino que o dono tem de procurar. Duplicar o braço deixa uma **barra laranja**, visível a dois
//! metros do ecrã. *O sujeito de um passo tem de se ver.*
//!
//! ⚠️ **O rótulo do botão é DERIVADO do modelo** ([`AddedRow::label`]), nunca escrito à mão aqui:
//! um smoke que promete uma frase que o painel já não pinta manda o dono procurar um botão que não
//! existe — e há gate a proibir o literal neste ficheiro.

use ph2d_editor::screens::hero::AddedRow;

impl crate::App {
    /// Cena 6 — ver o cabeçalho do módulo.
    ///
    /// ⚠️ **Cada passo é UM gesto**, e o passo diz **onde** ele acontece (canvas ou lista) — a lei
    /// que o report de 2026-09-06 comprou.
    pub(crate) fn instance_smoke_added(&mut self) {
        let vec_entities = &mut self.vec_entities;
        let gfx = self.gfx.as_mut().expect("gfx");
        let mut docs = crate::instance_docs::OwnedDocs {
            vec_scene: &mut gfx.vec_scene,
            vec_entities,
        };
        let (_master, copies) = crate::instance_removed_smoke::spawn_robot_scene(
            &mut gfx.sim,
            &gfx.component_registry,
            &mut docs,
        );
        // ⚠️ **A frase sai do MODELO.** O nome `Arm (1)` é o que o *Duplicate* produz (o `unique_name`
        // acrescenta o sufixo porque `Arm` já está em uso), e `Robot` é a receita — mas quem monta
        // a frase é a mesma função que o botão usa.
        let button = AddedRow {
            piece_id: 0,
            name: "Arm (1)".to_string(),
            master_name: "Robot".to_string(),
        }
        .label();
        println!(
            "[instance smoke 6] montado: {} robos iguais, todos do componente 'Robot'",
            copies.len()
        );
        println!(
            "[instance smoke 6] (na lista da esquerda: 'Robot' e' o COMPONENTE; os tres da tela \
             chamam-se 'Robot (1)', 'Robot (2)' e 'Robot (3)')"
        );
        println!(
            "[instance smoke 6] PASSO 1 (na TELA): clique na barra LARANJA do robo do MEIO — e' o \
             braco dele"
        );
        println!(
            "[instance smoke 6] PASSO 2 (na LISTA da esquerda): botao direito na linha 'Arm' que \
             esta' acesa > 'Duplicate'"
        );
        println!(
            "[instance smoke 6] => aparece um SEGUNDO braco laranja, so' nesse robo. Os outros \
             dois continuam com um so'"
        );
        println!(
            "[instance smoke 6] PASSO 3 (no cartao do topo do Inspector): aparece o botao \
             '{button}' — clique nele"
        );
        println!(
            "[instance smoke 6] => os TRES robos ficam com DOIS bracos. A peca passou a ser do \
             componente"
        );
        println!(
            "[instance smoke 6] (deu errado se: o botao nao aparecer · so' o robo do meio ficar \
             com dois bracos · ou o robo do meio ficar com TRES)"
        );
    }
}

#[cfg(test)]
#[path = "instance_added_smoke_tests.rs"]
mod tests;
