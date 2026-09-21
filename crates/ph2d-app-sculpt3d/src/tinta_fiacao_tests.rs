//! ⭐⭐⭐⭐ **O CENSO DA FIAÇÃO DA TINTA FINA** — os NOVE elos que a cura desta
//! wave precisa de ter LIGADOS: os três consumidores da porta
//! [`crate::tinta_da_peca::o_gesto_muda_a_topologia`], a metade da porta que lê
//! a tinta **EMPRESTADA**, e o `close_stroke` do gesto que **erra** a peça.
//!
//! ⛔⛔ **Porque é um censo de TEXTO e não um gate de comportamento:** os
//! consumidores são métodos de [`crate::Sculpt3dScene`], e construir uma cena
//! pede um `wgpu::Device` — os quatro testes do `dyntopo_tests.rs` são **todos**
//! `#[ignore]` por isso, e nem a suíte da família nem o CI correm um
//! `#[ignore]`. A prova de COMPORTAMENTO existe e vive no
//! `tinta_no_produto_tests.rs`, na placa; o que ESTE ficheiro defende é o elo
//! que ela não alcança em quem não tem placa — *a lei certa numa porta que
//! ninguém chama lê-se exactamente como a lei ausente* (a 5.ª ocorrência desta
//! forma nesta casa).
//!
//! ⚠️⚠️ **Ele foi escrito por CINCO MUTAÇÕES SOBREVIVENTES** — as M19, M20,
//! M21, M22 e a M25 de [`docs/3D/ferramentas/muta_a_metade_visivel.sh`], em
//! 2026-09-21. As quatro apagam a cura desta wave (um pincel de cor volta a
//! partir faces, o interruptor volta a triangular, o pen-down volta a
//! fotografar, a tinta EMPRESTADA deixa de contar, e o gesto que ERRA a peça
//! morre com o plano dentro) e a suíte inteira ficava **VERDE**. *Vinte de
//! vinte e cinco sangravam, e as cinco que não sangravam eram o controlo mais
//! quatro destas; a quinta nasceu com a cura do report que sobrou.*
//!
//! ⭐ **O SEXTO (`M26`) é o único que NÃO nasceu de uma sobrevivente: ele
//! nasceu PREVENIDO.** A cura que ele defende (*um pincel de cor pinta mesmo
//! errando o pen-down*, 21/09) só tem prova de comportamento num gate
//! `#[ignore]` + placa, e este ficheiro existe exactamente porque essa
//! população não é corrida nem pelo arnês nem pelo CI — *saber a forma da
//! armadilha vale o mesmo que a pagar outra vez, e custa menos*.
//!
//! ⛔ **A prosa é CORTADA antes de se medir, e isso é a metade que importa:** um
//! doc-comment que EXPLICA a cura contém o nome da porta, e um censo ingénuo
//! lê-o como se fosse a chamada — foi assim que a régua do fio da Fase B da
//! física acusou a própria cura. O [`so_a_prosa`] é o CONTROLO disso: cada
//! agulha tem de estar **ausente** da metade comentada.

/// O ficheiro sem uma única linha de comentário — o que sobra é CÓDIGO.
fn sem_prosa(f: &str) -> String {
    f.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Só as linhas de comentário — a metade contra a qual cada agulha é o CONTROLO.
fn so_a_prosa(f: &str) -> String {
    f.lines()
        .filter(|l| l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

const DYNTOPO: &str = include_str!("dyntopo.rs");
const PEN_DOWN: &str = include_str!("history_dyntopo.rs");
const PORTA: &str = include_str!("tinta_da_peca.rs");
const INPUT_DOWN: &str = include_str!("input_down.rs");
/// ⚠️ **Caminho relativo para FORA da crate, e é de propósito:** estes dois
/// elos vivem no MOTOR (`ph2d-sculpt3d`) e o censo vive na FAMÍLIA, porque é a
/// família que tem a cena e os gates de produto. Um `git mv` do motor faz isto
/// **falhar a COMPILAR**, que é a metade barata da família (HOWTO §2).
const DAB_CORE: &str = include_str!("../../ph2d-sculpt3d/src/stroke_dab_core.rs");
const TINTA_FINA: &str = include_str!("../../ph2d-sculpt3d/src/tinta_fina.rs");

/// Cada elo: o ficheiro, a agulha, e o nome da mutação que ela mata.
fn elos() -> Vec<(&'static str, &'static str, String, &'static str)> {
    vec![
        (
            "dyntopo.rs",
            "M20 refine_for_dab volta a perguntar só ao verbo",
            "        if !self.o_gesto_em_maos_muda_a_topologia(verbo) {".to_string(),
            DYNTOPO,
        ),
        (
            "dyntopo.rs",
            "M21 ligar o interruptor volta a triangular com o plano armado",
            [
                "        if self.tinta_fina_armada() {",
                "            return (true, 0);",
                "        }",
            ]
            .join("\n"),
            DYNTOPO,
        ),
        (
            "history_dyntopo.rs",
            "M22 o pen-down volta a fotografar/triangular para quem não mexe na topologia",
            [
                "        if !self.o_gesto_em_maos_muda_a_topologia(self.brush.verb) {",
                "            self.dyn_before = None;",
                "            return;",
                "        }",
            ]
            .join("\n"),
            PEN_DOWN,
        ),
        (
            "tinta_da_peca.rs",
            "M19 a tinta EMPRESTADA deixa de contar",
            [
                "        self.stroke.tinta_fina.is_some()",
                "            || self",
            ]
            .join("\n"),
            PORTA,
        ),
        // ⭐⭐⭐⭐ **O 5.º elo: o gesto que ERRA a peça tem de FECHAR.**
        //
        // ⛔ O pen-down empresta o plano por um `take` ANTES de saber se o
        // gesto pega (o 1.º dab precisa dele). Se o raio erra, isto vira
        // `Drag::Orbit` e o `close_stroke` — que é quem devolve — nunca corre.
        // ⚠️ A agulha leva as TRÊS linhas juntas de propósito: depois de cortar
        // a prosa elas ficam contíguas, e é isso que prova que o fecho está
        // **neste braço** e não noutro sítio qualquer do ficheiro.
        (
            "input_down.rs",
            "M25 o gesto que erra a peça volta a morrer com o plano dentro",
            [
                "                scene.brush.verb = verb;",
                "                scene.close_stroke();",
                "                scene.drag = Some(Drag::Orbit);",
            ]
            .join("\n"),
            INPUT_DOWN,
        ),
        // ⭐⭐⭐⭐ **O 6.º elo: um pincel de COR pinta mesmo errando o pen-down.**
        //
        // ⛔ Ordem do dono (21/09): *«permita pintar mesmo se [não] tocar um
        // vertex»*. Sem o segundo braço desta condição um traço de cor que
        // começa fora da peça vira ÓRBITA e **não pinta uma única amostra** —
        // e o report lê-se como *«a pintura sumiu»*, que é o mesmo texto do
        // defeito do plano emprestado, com outra causa.
        // ⚠️ A prova de comportamento é `#[ignore]` + placa
        // (`um_traco_de_cor_que_comeca_fora_da_peca_pinta`), logo sem este elo
        // a mutação SOBREVIVE — exactamente como as M19-M22.
        (
            "input_down.rs",
            "M26 um traço de cor que começa fora da peça volta a virar órbita",
            "            if took || scene.brush.verb.paints_color() {".to_string(),
            INPUT_DOWN,
        ),
        // ⭐⭐⭐⭐ **Os TRÊS elos do 3.º report de 21/09** — *«a tinta só é
        // depositada se o pincel está sobre um vertex»*. As duas cercas do dab
        // contam VÉRTICES, e a unidade que a tinta fina escreve é a AMOSTRA;
        // a terceira é a lei da folha, transplantada para essa unidade.
        //
        // ⚠️ Os três têm gate de comportamento em `tinta_no_produto_tests.rs`,
        // e os três são `#[ignore]` + placa — que é exactamente a população
        // que nem o arnês de mutação nem o CI correm.
        (
            "stroke_dab_core.rs",
            "M27 a pegada VAZIA volta a matar o dab de cor fina",
            "        if pegada_ja_vazia && !so_amostras {".to_string(),
            DAB_CORE,
        ),
        (
            "stroke_dab_core.rs",
            "M28 a máscara volta a decidir sem ter um único vértice sobre que julgar",
            "            if self.footprint.is_empty() && !(so_amostras && pegada_ja_vazia) {"
                .to_string(),
            DAB_CORE,
        ),
        (
            "tinta_fina.rs",
            "M29 a lei da folha deixa de valer para a AMOSTRA",
            "            if corta_a_folha && dot_olho(a.nrm) > crate::dab_alcance::NORMAL_LIMIAR {"
                .to_string(),
            TINTA_FINA,
        ),
    ]
}

/// ⭐⭐⭐ **Os consumidores CONSULTAM a porta, e a porta lê os dois sítios.**
///
/// ⚠️ **O `piso` não é decoração:** um `include_str!` que apontasse para o
/// ficheiro errado, ou um `sem_prosa` partido que devolvesse vazio, fariam a
/// busca falhar em voz alta — mas um que devolvesse **tudo** faria a prosa
/// satisfazer a agulha, e é isso que o [`so_a_prosa`] recusa.
#[test]
fn a_cura_da_tinta_fina_esta_ligada_nos_nove_sitios() {
    let elos = elos();
    assert_eq!(elos.len(), 9, "a população deste censo são os nove elos");

    for (ficheiro, mutacao, agulha, fonte) in elos {
        let codigo = sem_prosa(fonte);
        let prosa = so_a_prosa(fonte);

        assert!(
            prosa.len() > 200,
            "{ficheiro}: sem prosa não há controlo — o corte de comentários mediu nada"
        );
        assert!(
            codigo.contains(&agulha),
            "{ficheiro}: o elo da tinta fina SUMIU do código.\n\
             A mutação que isto existe para matar é a «{mutacao}».\n\
             Esperava, textualmente:\n{agulha}"
        );
        assert!(
            !prosa.contains(&agulha),
            "{ficheiro}: CONTROLO — a agulha do elo «{mutacao}» foi encontrada na PROSA.\n\
             Um censo satisfeito por um doc-comment não mede fiação nenhuma."
        );
    }
}

/// ⛔ **E o braço que a M20 põe no lugar NÃO pode voltar a existir.**
///
/// ⚠️ Ela não apaga a chamada: **substitui-a** pela pergunta antiga (só ao
/// verbo). O gate irmão vê a ausência da agulha certa; este vê a presença da
/// errada, e as duas metades são precisas porque *um ficheiro pode conter as
/// duas* — foi assim que a lei do projectar sobreviveu numa segunda cópia.
#[test]
fn ninguem_volta_a_perguntar_so_ao_verbo() {
    let codigo = sem_prosa(DYNTOPO);
    let antiga = "if !verbo.refina_no_dyntopo() && !verbo.colapsa_no_dyntopo() {";
    assert!(
        !codigo.contains(antiga),
        "dyntopo.rs: a pergunta ANTIGA voltou ao código.\n\
         Com o plano de tinta fina armado, um pincel de COR não muda topologia —\n\
         quem responde isso é a `tinta_da_peca::o_gesto_muda_a_topologia`."
    );
}
