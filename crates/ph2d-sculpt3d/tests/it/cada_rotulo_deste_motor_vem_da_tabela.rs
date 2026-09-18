//! ⭐⭐⭐ **NENHUM RÓTULO DESTE MOTOR É ESCRITO NO FONTE** — o HR-15 na fronteira dos MOTORES.
//!
//! ⛔⛔ **Esta crate nunca teve régua, e o painel que a pinta fecha VERDE.** O
//! `ph2d-panel-sculpt3d` tem o `every_word_this_panel_shows_comes_from_the_string_table` e ele já
//! estava a zero quando lá chegou — enquanto os **35 verbos**, as **12 quedas**, os **10 alfas** e
//! os **9 filtros** que ele pinta eram literais AQUI. *Um censo cuja crate não é DONA do texto que
//! ela pinta fica verde sobre texto cru.*
//!
//! ⚠️⚠️ **E nenhuma das 30 réguas lexicais o podia ver:** elas varrem os painéis, as `ph2d-app-*`,
//! a `ph2d-editor-core` e a shell — o motor não está na lista, e uma lista de crates a varrer é
//! exactamente onde a próxima fronteira se esconde. Quem o achou foi o **idioma de teste**, na 2.ª
//! fotografia do dono (*«vários botões de sculpt»*).
//!
//! # ⭐ O que este gate afirma, e porquê cada metade
//!
//! | metade | o defeito que ela apanha |
//! |---|---|
//! | a chave é BEM FORMADA (`sculpt3d.<enum>.<variante>`) | uma cópia-e-cola que deixa texto inglês no `label_key` |
//! | a chave está DECLARADA na tabela | o `tr` faria `leak_key` e o chip pintaria `sculpt3d.verb.draw` cru |
//! | duas variantes nunca partilham chave | dois chips com a mesma palavra na mesma fileira |
//! | o inglês NÃO é a chave | ⭐ a metade que torna a segunda observável: sem ela, um `leak_key` lê-se como sucesso |
//!
//! ⛔ **O que ele NÃO pode afirmar:** a derivação exacta, porque o nome do enum não existe em
//! runtime (`Verb::Draw` não sabe dizer-se `verb`). Ele afirma a FORMA, a UNICIDADE e a PRESENÇA
//! na tabela — que é onde uma cópia-e-cola erra.

use ph2d_sculpt3d::{
    Alpha, BoundaryModo, BoundaryQueda, ClothArea, ClothFilterKind, ClothFilterOrientation,
    ClothForceFalloff, ClothMode, Falloff, FilterKind, PlanoInversao, PoseDeformacao, PoseModo,
    ProjectMode, RefMode, SmearMode, TransformKind, TrimForma, Verb,
};
// ⚠️ A `Scales` do kelvinlet não é re-exportada na raiz (o `lib.rs` só levanta a constante do
// alcance) — o caminho longo é o que existe, e escrevê-lo aqui é mais honesto do que abrir a
// superfície pública da crate por causa de um teste.
use ph2d_sculpt3d::kelvinlet::Scales;

/// ⚠️ **Piso de população, por família.** Uma varredura que passasse a colher zero variantes
/// ficaria trivialmente verde — e o `Verb::ALL` já encolheu uma vez neste repo sem ninguém ver.
const PISO: usize = 100;

fn bem_formada(k: &str) -> bool {
    let p: Vec<&str> = k.split('.').collect();
    p.len() == 3
        && p[0] == "sculpt3d"
        && p[1..].iter().all(|s| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
}

/// Todos os pares `(chave, inglês)` que o motor oferece à interface.
fn pares() -> Vec<(&'static str, &'static str)> {
    let mut v: Vec<(&'static str, &'static str)> = Vec::new();
    // ⭐ A chave sai do MOTOR e o inglês sai da TABELA — nunca de um `label()`. As quatro das
    //   crates de lei não têm `label()` nenhum (a lib delas declara-se sem dependência nenhuma),
    //   e ler as vinte famílias pela mesma porta é o que torna este censo uma régua só.
    macro_rules! colhe {
        ($($t:ty),* $(,)?) => {$(
            for x in <$t>::ALL {
                let k = x.label_key();
                v.push((k, ph2d_i18n::tr_em(ph2d_i18n::Idioma::Ingles, k)));
            }
        )*};
    }
    colhe!(
        Verb,
        Falloff,
        Alpha,
        FilterKind,
        ClothMode,
        ClothArea,
        ClothFilterKind,
        ClothFilterOrientation,
        ClothForceFalloff,
        TransformKind,
        Scales,
        PlanoInversao,
        ProjectMode,
        RefMode,
        SmearMode,
        TrimForma,
        // ⚠️ As quatro das crates de LEI (`ph2d-boundary`, `ph2d-pose`) entram por esta porta: elas
        //    são re-exportadas aqui e NÃO têm o acessório em inglês (a lib delas declara-se sem
        //    dependência nenhuma), logo o `label()` que este censo lê é o da re-exportação.
        BoundaryModo,
        BoundaryQueda,
        PoseDeformacao,
        PoseModo,
    );
    v
}

#[test]
fn cada_rotulo_deste_motor_vem_da_tabela() {
    let pares = pares();
    assert!(
        pares.len() >= PISO,
        "o censo colheu {} rótulos e o piso é {PISO} — uma varredura que encolhe fica verde a \
         medir nada",
        pares.len()
    );
    let mut vistas = std::collections::BTreeMap::new();
    for (chave, ingles) in &pares {
        assert!(
            bem_formada(chave),
            "{chave:?} não tem a forma `sculpt3d.<enum>.<variante>`"
        );
        // ⭐ A metade que torna a presença na tabela OBSERVÁVEL: o `tr` de uma chave desconhecida
        //   devolve a própria chave (*missing-key passthrough*), logo `ingles == chave` é
        //   exactamente «esta entrada não existe em `ph2d-i18n/src/sculpt_engine.rs`».
        assert_ne!(
            ingles, chave,
            "a chave {chave:?} não está declarada em `ph2d-i18n/src/sculpt_engine.rs` — o chip \
             pintaria o identificador cru na tela"
        );
        assert!(
            !ingles.is_empty(),
            "{chave:?} traduz para texto vazio — o chip fica sem nome"
        );
        if let Some(outra) = vistas.insert(*chave, *ingles) {
            panic!("a chave {chave:?} é devolvida por duas variantes ({outra:?} e {ingles:?})");
        }
    }
}

/// ⭐ **Duas leis com o MESMO nome na mesma fileira são indistinguíveis para o artista** — e é o
/// que uma chave copiada de um irmão produz. ⚠️ Rótulos repetidos entre FAMÍLIAS são legítimos e
/// estão medidos (*Inflate* e *Scale* existem na malha e no pano, e são leis diferentes: é a
/// fileira que as separa — está escrito no `FilterLaw::label`).
#[test]
fn duas_variantes_da_mesma_fileira_nunca_mostram_a_mesma_palavra() {
    let mut por_familia: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    for (chave, ingles) in pares() {
        let familia = chave.split('.').nth(1).expect("a forma já foi afirmada");
        por_familia.entry(familia).or_default().push(ingles);
    }
    // ⛔ Controlo de vacuidade: sem isto uma colheita vazia dá zero famílias e concorda sempre.
    assert!(
        por_familia.len() >= 20,
        "só {} famílias — a colheita encolheu?",
        por_familia.len()
    );
    for (familia, mut rotulos) in por_familia {
        let antes = rotulos.len();
        rotulos.sort_unstable();
        rotulos.dedup();
        assert_eq!(
            rotulos.len(),
            antes,
            "a família `{familia}` mostra a mesma palavra em duas variantes: {rotulos:?}"
        );
    }
}

/// ⭐⭐ **O `label()` do motor é um ACESSÓRIO DERIVADO da tabela, nunca uma segunda cópia.**
///
/// ⚠️ Esta é a metade que paga a decisão de desenho: 216 sítios chamam `.label()` e continuam a
/// ler inglês — entre eles as tabelas de proveniência do dyntopo, que carregam o veredito do dono
/// escrito ao lado do nome do verbo. *Se alguém voltar a escrever o inglês num `match` do motor,
/// as duas cópias divergem no dia seguinte e ninguém vê.*
#[test]
fn o_ingles_do_motor_e_lido_da_tabela_e_nao_de_um_segundo_match() {
    let mut n = 0usize;
    macro_rules! confere {
        ($($t:ty),* $(,)?) => {$(
            for x in <$t>::ALL {
                assert_eq!(
                    x.label(),
                    ph2d_i18n::tr_em(ph2d_i18n::Idioma::Ingles, x.label_key()),
                    "`{}::label()` deixou de vir da tabela",
                    stringify!($t)
                );
                n += 1;
            }
        )*};
    }
    confere!(
        Verb,
        Falloff,
        Alpha,
        FilterKind,
        ClothMode,
        ClothArea,
        ClothFilterKind,
        ClothFilterOrientation,
        ClothForceFalloff,
        TransformKind,
        Scales,
        PlanoInversao,
        ProjectMode,
        RefMode,
        SmearMode,
        TrimForma,
    );
    // ⛔ Controlo de vacuidade: um `ALL` vazio faria o laço não correr e o gate concordar.
    assert!(n >= PISO, "só {n} variantes conferidas, o piso é {PISO}");
}
