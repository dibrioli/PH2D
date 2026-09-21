//! **OS GATES DA MATÉRIA** — filho (`#[path]`) da [`super`], compilado só sob `cfg(test)`.
//!
//! ⭐ **Os quatro primeiros são PUROS e correm SEMPRE** — sem adapter, sem ambiente, sem `#[ignore]`.
//! É para isso que a [`super::decide`] existe como função: a lei que esta wave tem de afirmar
//! (*a matéria é lida na MUDANÇA e não por quadro*) não tem um pixel dentro.

use super::{Decisao, decide};
use ph2d_mesh_render::Lighting;

const A: u64 = 7;
const B: u64 = 9;

/// ⭐⭐⭐ **A MATÉRIA É LIDA UMA VEZ, E NÃO POR QUADRO.**
///
/// ⛔ **O recurso é o RELÓGIO DO DEVICE** (o cabeçalho da [`super::decide`] tem os bytes): sem esta
/// lei, cada quadro do visor pagaria um `readback` para responder a uma pergunta cuja resposta só
/// muda quando o artista escolhe outro objecto.
///
/// ⚠️ **O CONTROLO está dentro:** a primeira chamada TEM de ler. Sem ele, uma implementação que
/// nunca lesse nada passaria — e seria exactamente o defeito que o dono reportou três vezes.
#[test]
fn a_materia_e_lida_uma_vez_e_nao_por_quadro() {
    let mut memo = None;
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Pbr, false),
        Decisao::Le(A),
        "o CONTROLO: a primeira vez tem de ler"
    );
    for quadro in 0..5 {
        assert_eq!(
            decide(&mut memo, Some(A), Lighting::Pbr, false),
            Decisao::Nada,
            "o quadro {quadro} voltou a pedir a leitura"
        );
    }
}

/// ⭐⭐ **ASSAR TROCA A MATÉRIA, E A MEMÓRIA VÊ A TROCA.**
///
/// Depois do primeiro bake a sprite passa a ser `base × luz` e a tabela dos assados passa a ter o
/// `base` — que é a matéria de verdade. Sem a segunda metade da chave o visor continuaria a mostrar
/// a leitura de antes do gesto, **para sempre**.
#[test]
fn assar_troca_a_materia_e_a_memoria_ve_a_troca() {
    let mut memo = None;
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Pbr, false),
        Decisao::Le(A)
    );
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Pbr, true),
        Decisao::Le(A),
        "a sprite foi assada e o visor ficou com a matéria de antes"
    );
    // E a nova também é memoizada — senão a cura de um laço abriria outro.
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Pbr, true),
        Decisao::Nada
    );
}

/// ⭐⭐⭐ **SÓ O MODO QUE LÊ A MATÉRIA A PEDE — e a lista é DERIVADA.**
///
/// ⚠️ **`Lighting::todos()` e não uma lista escrita à mão:** a irmã escrita à mão noutra crate ficou
/// com `2` fixos quando o terceiro modo chegou, e o gate da ponte acusou `12` contra `13`. Com a
/// tabela como fonte, o modo seguinte entra aqui sozinho.
///
/// ⚠️⚠️ **As DUAS metades, e a primeira sozinha mente:** de memória vazia um modo que não lê a
/// matéria devolve `Nada` — *não há nada a esquecer* —, e um gate que só olhasse isso ficaria verde
/// sobre uma implementação que nunca limpa a fonte. A metade que importa é a do artista que **já
/// tinha a matéria posta e TROCA de modo**: ali a resposta tem de ser `Esquece`, senão o visor
/// pinta a sprite com uma luz que não é a que assa.
#[test]
fn so_o_modo_que_le_a_materia_a_pede() {
    let mut pediram = 0;
    let mut esqueceram = 0;
    for modo in Lighting::todos() {
        // (a) De memória vazia: só a lei que assa manda LER.
        let mut memo = None;
        let d = decide(&mut memo, Some(A), modo, false);
        if modo == Lighting::Pbr {
            assert_eq!(d, Decisao::Le(A), "a lei que assa tem de pedir a matéria");
            pediram += 1;
        } else {
            assert_eq!(
                d,
                Decisao::Nada,
                "o modo {modo:?} não lê a matéria e mandou lê-la — é um `readback` por nada"
            );
        }
        // (b) Com a matéria JÁ posta: trocar para um modo que não a lê tem de a esquecer.
        let mut posto = Some((A, false));
        let d = decide(&mut posto, Some(A), modo, false);
        if modo == Lighting::Pbr {
            assert_eq!(d, Decisao::Nada, "a memória deixou de segurar a leitura");
        } else {
            assert_eq!(
                d,
                Decisao::Esquece,
                "o modo {modo:?} ficou a pintar a matéria de uma lei que ele não corre"
            );
            esqueceram += 1;
        }
    }
    assert_eq!(pediram, 1, "a tabela dos modos não tem a lei que assa");
    assert!(
        esqueceram >= 3,
        "a tabela dos modos encolheu: só {esqueceram} não leem a matéria"
    );
}

/// ⛔⛔⛔ **LARGAR A SELECÇÃO NÃO LARGA A MATÉRIA — e até 2026-09-21 este gate afirmava o
/// CONTRÁRIO.**
///
/// A redacção anterior chamava-se `largar_a_seleccao_devolve_o_barro_do_shader` e exigia
/// [`Decisao::Esquece`] no quadro em que a selecção largasse a sprite. ⚠️ **Ela estava a pinar o
/// defeito que o dono reportou:** *«se seleciono o objeto 3d, ele muda a aparência»* — e a selecção
/// larga a sprite exactamente quando ele pega na PEÇA para esculpir.
///
/// ⭐ *Um gate verde pode pinar um defeito de produto*, e este pinou-o por um dia.
///
/// # O que se afirma agora
///
/// A matéria é da PEÇA: largar a selecção dá [`Decisao::Nada`] e a fonte fica onde estava.
///
/// ⚠️ **As DUAS metades que sobram são o CONTROLO**, e sem elas isto seria «nunca mais lê nada»:
/// escolher **outra** sprite lê outra vez, e **voltar** à mesma não re-lê (o `readback` de `4 MiB`
/// que a memória existe para evitar).
#[test]
fn largar_a_seleccao_mantem_a_materia_da_peca() {
    let mut memo = None;
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Pbr, false),
        Decisao::Le(A)
    );
    // ⭐ O report do dono, em duas linhas: pegar na peça não pode trocar a aparência dela.
    assert_eq!(decide(&mut memo, None, Lighting::Pbr, false), Decisao::Nada);
    assert_eq!(decide(&mut memo, None, Lighting::Pbr, false), Decisao::Nada);
    // CONTROLO 1 — voltar à MESMA sprite não paga um `readback`.
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Pbr, false),
        Decisao::Nada
    );
    // CONTROLO 2 — escolher OUTRA sprite lê outra vez; a chave não é um booleano.
    assert_eq!(
        decide(&mut memo, Some(B), Lighting::Pbr, false),
        Decisao::Le(B)
    );
}

/// ⛔ **E o `Esquece` sobra para UMA coisa só: sair da lei que lê a matéria** — e **uma vez só**.
///
/// ⚠️ Sem a metade do *«uma vez só»* o visor limparia a fonte em todo quadro fora do PBR — barato,
/// mas é a mesma forma do defeito que a irmã de cima mede, e a forma é o que envelhece.
#[test]
fn sair_da_lei_que_le_a_materia_devolve_o_barro_uma_vez_so() {
    let mut memo = None;
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Pbr, false),
        Decisao::Le(A)
    );
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Flat, false),
        Decisao::Esquece
    );
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Flat, false),
        Decisao::Nada
    );
    // E voltar ao PBR lê outra vez — senão o barro ficava para sempre.
    assert_eq!(
        decide(&mut memo, Some(A), Lighting::Pbr, false),
        Decisao::Le(A)
    );
}

/// ⭐⭐⭐⭐ **A LEI DA MATÉRIA TEM UMA PORTA E DOIS LEITORES — e este é o censo.**
///
/// ⛔⛔ **A ordem é a lei** (a tabela dos assados primeiro, o device depois) e ela tinha **um**
/// leitor até 2026-09-21. O visor é o segundo, e *uma segunda cópia da ordem divergiria exactamente
/// no caso que ela existe para impedir*: uma sprite já assada devolveria `base × luz` a quem pediu
/// `base`, e o objecto escureceria a cada gesto.
///
/// ⚠️ **As DUAS metades**: a porta tem os dois chamadores **e** o gesto de assar não guarda uma
/// segunda cópia da leitura. Cada uma sozinha mente — a primeira ficaria verde com o `bake.rs` a
/// reler a fonte ao lado, e a segunda com a porta órfã.
///
/// ⚠️⚠️ **As agulhas são medidas com o ESPAÇO COLAPSADO**, e isso não é conforto: o `rustfmt`
/// quebra uma chamada que passe a largura da linha, e um censo que casasse o texto CRU reprovaria
/// produto correcto no dia em que um nome crescesse um caractere. *Um gate que se cura
/// reformatando o ficheiro que ele lê não mede a propriedade que diz medir.*
#[test]
fn a_lei_da_materia_tem_uma_porta_e_dois_leitores() {
    /// O ficheiro com cada corrida de espaços reduzida a um — ver o doc.
    fn espremido(src: &str) -> String {
        src.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    let bake = espremido(include_str!("bake.rs"));
    let albedo = espremido(include_str!("albedo.rs"));
    let (bake_src, albedo_src) = (bake.as_str(), albedo.as_str());

    assert_eq!(
        bake_src.matches("albedo::materia_para(").count(),
        1,
        "o gesto de assar deixou de ler a matéria pela porta"
    );
    assert_eq!(
        albedo_src
            .matches("materia_para(forms, bits, &mut || ler_fonte(sim, renderer))")
            .count(),
        1,
        "a porta perdeu o leitor do VISOR — ela voltou a ter um consumidor só"
    );
    // A metade que a primeira não cobre: nenhuma SEGUNDA leitura da fonte vive no gesto.
    assert_eq!(
        bake_src.matches("into_straight()").count(),
        0,
        "o gesto de assar voltou a converter a fonte por conta própria"
    );
    assert_eq!(
        albedo_src.matches("into_straight()").count(),
        1,
        "o CONTROLO da extracção: a conversão tem de viver na porta"
    );
}

/// ⭐⭐⭐⭐ **RE-ASSAR NÃO LÊ A TELA DE VOLTA — e agora há quem o afirme.**
///
/// ⛔⛔ **É a ORDEM dentro da [`super::materia_para`], e ela nunca tinha tido gate.** Depois do
/// primeiro bake os pixels da sprite são `base × luz`; lê-los como fonte faria a segunda acendida
/// acender o que já está aceso, e **o objecto escureceria a cada gesto**. Foi por isso que o
/// [`ph2d_form_donation::baked_form::BakedForm`] guarda o `base` desde que existe — a lei estava
/// escrita em prosa, com um leitor só e nenhuma régua.
///
/// ⚠️ **Ela passou a ter DOIS leitores em 2026-09-21** (o gesto e o VISOR), e é isso que a torna
/// urgente: uma segunda cópia da ordem divergiria exactamente no caso que ela existe para impedir.
///
/// ⚠️ **A régua é o fecho NÃO SER CHAMADO**, e não o valor devolvido: uma implementação que lesse a
/// tela e depois deitasse fora a leitura devolveria o `base` certo e pagaria um `readback` por
/// gesto — e, pior, seria a cópia de que a ordem já não importa.
///
/// ⚠️ **O CONTROLO é a metade de baixo:** com a tabela VAZIA o fecho tem de ser chamado. Sem ele,
/// uma porta que nunca lesse nada passaria na metade de cima.
#[test]
fn re_assar_nao_le_a_tela_de_volta() {
    use ph2d_form_donation::baked_form::BakedForm;
    use std::collections::BTreeMap;

    let mut forms: BTreeMap<u64, BakedForm> = BTreeMap::new();
    forms.insert(
        A,
        BakedForm {
            size: (2, 1),
            base: vec![10, 20, 30, 255, 40, 50, 60, 255],
            form: Vec::new(),
            form_occ: Vec::new(),
            texture_id: 0,
            rig: ph2d_light::LightRig::default(),
            lit_with: None,
            lei: ph2d_form_donation::lei_da_luz::Lei::default(),
        },
    );

    let mut leituras = 0u32;
    let got = super::materia_para(&forms, A, &mut || {
        leituras += 1;
        None
    })
    .expect("a matéria de uma sprite assada é o `base` que ela guarda");
    assert_eq!(got.1, (2, 1));
    assert_eq!(
        got.0, forms[&A].base,
        "a matéria não é o `base` do objecto assado"
    );
    assert_eq!(
        leituras, 0,
        "uma sprite JÁ ASSADA voltou a ler a tela — a próxima acendida acende o que já está aceso"
    );

    // O CONTROLO: quem ainda não foi assado TEM de ler a fonte.
    let vazio: BTreeMap<u64, BakedForm> = BTreeMap::new();
    let mut leituras = 0u32;
    let _ = super::materia_para(&vazio, A, &mut || {
        leituras += 1;
        None
    });
    assert_eq!(
        leituras, 1,
        "o CONTROLO: sem `base` guardado a fonte tem de ser lida"
    );
}

/// ⛔⛔⛔ **O BARRO TEM UM CAMINHO DE VOLTA, E É O `Esquece` — a porta das traseiras não existe.**
///
/// # Porque este gate nasceu, e ele nasceu de uma MUTAÇÃO SOBREVIVENTE
///
/// O report do dono de 2026-09-21 (*«se seleciono o objeto 3d, ele muda a aparência»*) tinha **duas**
/// metades, e a [`largar_a_seleccao_mantem_a_materia_da_peca`] só cobre uma. A outra é que escolher
/// o objecto 3D **é uma selecção que não tem pixels** — ela não passa pela [`decide`], passa pela
/// leitura, que falha. Repor ali o `clear_albedo_source` devolve o `CLAY` com o mesmo sintoma, e a
/// mutação que o fazia **SOBREVIVEU a tudo**.
///
/// ⚠️ **E ela não é gateável pelo caminho normal:** a decisão vive na [`super::sincroniza`], que pede
/// um `Device`, um `SimWorld` e um `SpriteRenderer` — *quando um gate precisa de um device para
/// medir uma decisão que não tem pixel nenhum, a lei está no sítio errado*, e aqui ela não pode sair
/// de lá porque o efeito É o device.
///
/// ⇒ a régua é o CENSO: há **exactamente um** caminho de volta ao barro em todo o ficheiro, e ele é
/// o braço do [`Decisao::Esquece`].
///
/// ⚠️ **A metade que impede o vácuo** é a segunda: sem ela um ficheiro que perdesse o `Esquece`
/// inteiro leria `0` e passaria — *um censo que só conta para cima aprova a ausência*.
#[test]
fn o_barro_tem_um_caminho_de_volta_e_e_o_esquece() {
    let src = include_str!("albedo.rs");
    let espremido = src.split_whitespace().collect::<Vec<_>>().join(" ");

    // ⚠️ Conta-se no CÓDIGO e não no ficheiro: a prosa deste módulo nomeia a porta várias vezes, e
    // um censo que contasse os comentários mediria quanto alguém escreveu sobre ela.
    let chamadas = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//") && !l.trim_start().starts_with("///"))
        .filter(|l| l.contains("clear_albedo_source"))
        .count();
    assert_eq!(
        chamadas, 1,
        "há {chamadas} caminhos de volta ao barro — o segundo devolve o `CLAY` pela porta das \
         traseiras, que é o report do dono de 2026-09-21"
    );

    // A metade que impede o vácuo: o caminho que existe é o do `Esquece`.
    assert!(
        espremido.contains("Decisao::Esquece => { scene.renderer.clear_albedo_source(); return; }"),
        "o único caminho de volta ao barro deixou de ser o braço do `Esquece`"
    );
}
