//! **O catálogo de componentes que as fixturas desta família montam** — e o gate que o obriga a
//! ser o MESMO que o produto monta.
//!
//! # Por que ele existe
//!
//! As fixturas desta família precisam de um [`ComponentRegistry`] para tudo o que medem: a cópia
//! profunda copia **bytes de componentes registados**, e um componente que não esteja no registo é
//! descartado **em silêncio** — que é precisamente o defeito que os gates desta família existem
//! para apanhar. Até 2026-09-12 elas pediam-no a `crate::init::build_component_registry`, e essa
//! chamada era a **âncora** que prendia a família à shell: **33 usos em 29 ficheiros, todos
//! testes** — `91 %` do fecho da família, medido pelo `scripts/fecho-da-familia.py`.
//!
//! ⛔ **Uma crate nunca pode chamar o `bin`.** A shell é um binário e não pode ser dependência de
//! ninguém (HOWTO §4), então «continuar a chamar o `init.rs`» não é uma opção deste lado da
//! fronteira — a pergunta é só onde o catálogo passa a ser montado.
//!
//! # ⚠️ A tentação, e porque ela está REFUTADA por medição
//!
//! O precedente na árvore é a `ph2d-app-physics`: **seis** ficheiros de teste dela compõem o
//! registo com o subconjunto que cada um usa (`register_ecs` + `register_physics`). Para aquela
//! família chega, porque as fixturas dela só põem componentes daquelas duas crates.
//!
//! ⛔ **Aqui não chega, e o motivo é o assunto desta família.** Medido em 12/09, as fixturas
//! usam `ph2d_render::Sprite` (**54** ocorrências), `ph2d_physics_ecs::{Collider, RigidBody,
//! ColliderShape, PlatformPlayer}` (**23** ficheiros) e `ph2d_field_ecs::FieldObject` — que são
//! exactamente os componentes cujo **descarte silencioso** estes gates medem. *Um registo montado
//! com «o que este teste põe» não pode nunca acusar «o produto deixou de registar isto»: ele é a
//! mesma resposta escrita duas vezes, e as duas envelhecem juntas na direcção errada.*
//!
//! ⇒ Este registo é montado com **as mesmas cinco chamadas, na mesma ordem** que o
//! `build_component_registry` do produto — incluindo o `ph2d_skeleton_ecs`, que **nenhum ficheiro
//! desta família usa hoje**. ⭐ Registá-lo custa uma linha de `Cargo.toml` e **remove a lista de
//! isenções inteira**: sem isenções não há catraca para apodrecer, e o gate abaixo pode exigir
//! **igualdade exacta** em vez de inclusão.
//!
//! ⚠️ *É a lei do §5.0 do `CLAUDE.md` levada um passo à frente — «uma catraca sem censo de
//! obsolescência vira LICENÇA». A tolerância que não existe é a única que não precisa de censo.*

use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};

/// **O catálogo, montado como o produto o monta.**
///
/// ⚠️ Toda alteração aqui tem de ser a mesma alteração que o `build_component_registry` do
/// produto sofreu — e o gate [`o_registo_das_fixturas_e_o_do_produto`] reprova se não for.
pub(crate) fn registo() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_render::register_render_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    ph2d_field_ecs::register_field_components(&mut reg);
    ph2d_skeleton_ecs::register_skeleton_components(&mut reg);
    reg
}

#[cfg(test)]
mod tests {
    /// As chamadas `register_*_components(` que o corpo de `fn <nome>` faz, ordenadas e sem
    /// repetições.
    ///
    /// ⛔ **Descasca comentários antes de contar** — um censo textual que lê PROSA como código
    /// mente nos dois sentidos (HOWTO §2.12), e este ficheiro tem os nomes escritos no
    /// doc-comment logo acima da função que ele mede.
    fn chamadas_de(fonte: &str, nome: &str) -> Vec<String> {
        let corpo = fonte
            .split_once(nome)
            .unwrap_or_else(|| panic!("`{nome}` desapareceu — reancore este censo"))
            .1
            .split_once("\n}")
            .expect("e ela fecha")
            .0;
        let mut v: Vec<String> = corpo
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .flat_map(|l| l.match_indices("register_").map(move |(i, _)| &l[i..]))
            .filter_map(|s| s.split_once('(').map(|(n, _)| n))
            .filter(|n| n.ends_with("_components"))
            .map(str::to_string)
            .collect();
        v.sort();
        v.dedup();
        v
    }

    /// ⭐⭐ **O registo das fixturas é o do PRODUTO — o mesmo conjunto, sem isenção nenhuma.**
    ///
    /// # O que ele mede, e o que aconteceria sem ele
    ///
    /// O dia em que alguém acrescentar uma sexta crate de componentes ao
    /// `build_component_registry` da shell, as fixturas desta família continuariam a montar cinco
    /// — e **toda** medição de «a cópia não descarta nada» passaria a correr sobre um catálogo
    /// mais pobre que o do produto, **em silêncio e a verde**. Não há mais nada no repo que
    /// pergunte isto: os espelhos do `ph2d-render`/`ph2d-script` contam o registo do **ECS**, não
    /// as chamadas de composição da shell (e foi por essa cegueira que eles ficaram vermelhos
    /// durante três waves — §6 do handoff de 10/09).
    ///
    /// # ⛔⛔ A 1.ª redacção deste gate era DECORATIVA, e só a mutação o disse
    ///
    /// Ela comparava o `init.rs` com uma **constante escrita à mão** (`AS_MINHAS`) que listava os
    /// cinco nomes. Apagar a linha do `ph2d_skeleton_ecs` de [`super::registo`] deixava o gate
    /// **VERDE**: os dois lados que ele comparava continuavam a ter cinco, e nenhum deles era a
    /// função. *Uma lista escrita à mão ao lado da coisa que ela descreve é uma TERCEIRA resposta
    /// à mesma pergunta, e é sempre a que não é executada.* ⇒ hoje os dois lados são **extraídos
    /// do corpo das duas funções** pelo mesmo extractor.
    ///
    /// # ⚠️ Porque é `include_str!` e não `read_to_string`
    ///
    /// HOWTO §2.6: o `include_str!` falha **a compilar** quando o ficheiro se move; o gémeo em
    /// runtime só falha **se o teste correr**, e um `#[ignore]` ou um filtro apagam-no. Esta
    /// agulha aponta para **fora** da crate, que é legítimo e tem precedente (os ~53 gates de
    /// arquitectura do `ph2d-editor-core` varrem `shells/desktop/src` de fora).
    ///
    /// # ⚠️ E porque a agulha é `register_…_components(` e não o nome da função da shell
    ///
    /// HOWTO §2.13: uma agulha ancora na **LEI**, nunca na visibilidade nem no endereço de quem
    /// chama. `build_component_registry` pode mudar de módulo; o que não pode mudar sem isto
    /// reprovar é **o conjunto de crates cujos componentes o produto regista**.
    ///
    /// (Mutação, verificada: tirar a linha do `ph2d_skeleton_ecs` de [`super::registo`] ⇒ RED.)
    #[test]
    fn o_registo_das_fixturas_e_o_do_produto() {
        let do_produto = chamadas_de(
            include_str!("../../../shells/desktop/src/init.rs"),
            "fn build_component_registry()",
        );
        let as_minhas = chamadas_de(
            include_str!("component_registry_for_tests.rs"),
            "fn registo()",
        );

        // ⛔ **O piso de população** (HOWTO §2.7): sem ele, um `split_once` que deixasse de casar
        // daria DUAS listas vazias, e `vazio == vazio` é trivialmente verdadeiro. *Um censo que
        // varre nada fica verde a medir nada.*
        assert!(
            do_produto.len() >= 4 && as_minhas.len() >= 4,
            "este censo leu {} chamadas no produto e {} nas fixturas, e esperava >= 4 de cada \
             lado — perdeu o sujeito",
            do_produto.len(),
            as_minhas.len()
        );

        assert_eq!(
            do_produto, as_minhas,
            "o catalogo do produto e o das fixturas desta familia divergiram.\n  \
             produto: {do_produto:?}\n  fixturas: {as_minhas:?}\n\
             ⛔ a cura e' igualar `component_registry_for_tests::registo`, NUNCA acrescentar uma \
             isencao: sem isencoes nao ha catraca para apodrecer."
        );
    }
}
