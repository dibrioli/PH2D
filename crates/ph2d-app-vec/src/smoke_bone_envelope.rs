//! ⭐⭐⭐ **A CENA DO ENVELOPE** — `PH2D_VEC_BONE_SMOKE=2`, a cena que o dono pediu por escrito
//! (2026-09-18: *«Testei aqui e não vi em nenhum dos casos o envelope fazer diferença na
//! deformação. […] Melhor montar uma cena específica para me mostrar isso»*).
//!
//! # ⛔⛔⛔ O report estava CERTO, e mais certo do que a minha resposta lhe deu
//!
//! Eu respondi-lhe que *«numa forma vectorial o envelope manda como sempre»*, e a medição derruba-o:
//! o `ph2d_vec_skin::pesos::pesos_do_caminho` resolve os mesmos *Bounded Biharmonic Weights* sobre o
//! INTERIOR de um contorno fechado desde 2026-09-15 — logo o alcance é inerte ali também. Medido
//! pela porta do produto, variando o `strength` do osso do meio de `0,1` a `8,0`
//! ([`ph2d_skeleton_live`], `sonda_do_envelope_no_vector_tests`):
//!
//! | forma | fechada? | amplitude da deformação |
//! |---|---|---:|
//! | `Rectangle` · `Ellipse` · `Star` · `Polygon` | fechada | **`0,000000`** |
//! | `Line` · `Arc` · `Spiral` | ABERTA | `2,03` · `4,25` · `2,05` |
//!
//! ⇒ **a mídia nunca foi a pergunta certa.** O envelope manda onde o padrão-ouro **não resolveu**, e
//! isso acontece num sítio só: um caminho **ABERTO** não tem interior, logo não tem domínio para a
//! energia, logo não tem pesos guardados — e a deformação cai na lei euclidiana, que é a que lê o
//! alcance.
//!
//! # O que a cena monta, e por que cada peça está lá
//!
//! Três fileiras, a MESMA corda, a MESMA corrente de três ossos, a MESMA dobra. Só o que está
//! escrito na coluna da direita muda:
//!
//! | fileira | o que ela é | o envelope |
//! |---|---|---|
//! | **`Corda (alcance 1)`** | um traço ABERTO, com o alcance de fábrica | VIVO — mancha e alça à vista |
//! | **`Corda (alcance 4)`** | o MESMO traço, com o alcance do osso do meio em `4` | VIVO — e a corda sai **noutro sítio** |
//! | **`Barra preenchida`** | a mesma faixa, FECHADA | INERTE — sem mancha e sem alça |
//!
//! ⭐⭐ **As duas primeiras são um par, e é ele que responde ao report:** elas diferem num número só,
//! logo *a diferença que se vê entre as duas É o envelope*. ⛔ Uma cena com uma corda só obrigaria o
//! dono a confiar na memória da pose anterior, que é exactamente o que falhou no smoke dele.
//!
//! ⚠️ **A terceira fileira é a metade NEGATIVA, e vale tanto como as outras duas:** ela é onde a
//! segunda queixa dele (*«o gizmo do envelope fica sempre visível mesmo quando não é usado?»*) se
//! lê curada — ali o osso do meio não tem mancha nenhuma, porque ali o alcance não decide nada.
//!
//! # ⚠️ Os números desta cena são DERIVADOS, e a tabela está no código
//!
//! A dobra, o comprimento da corda e o alcance forte saem de uma varredura de legibilidade, não de
//! gosto — ver [`DOBRA`], [`CORDA_M`] e [`ALCANCE_FORTE`].

use ph2d_ecs::{Entity, SimWorld};
use ph2d_vec_scene::{ShapeKind, VecScene, cook_tinted as shape};

/// ⭐ **Quantos ossos por corda.**
///
/// ⚠️ **Três, e o número é MEDIDO** (a varredura vive no doc de [`DOBRA`]): com `2` o efeito é maior
/// (`23,7 %` contra `18,4 %` a alcance `4`) e a corda tem **um vinco só**, logo não há curva para o
/// olho julgar; com `6` o alcance de fábrica já cobre tão pouco que a `2` o efeito cai a `1,7 %`.
const OSSOS: u32 = 3;

/// ⭐⭐⭐ **A DOBRA POR JUNTA — o número que torna o envelope VISÍVEL.**
///
/// Varrido pela porta do produto (o desvio entre a corda a alcance `1` e a mesma corda a alcance
/// `N`, em percentagem do comprimento dela):
///
/// | dobra/junta | alcance `2` | `3` | **`4`** | `6` | `8` |
/// |---:|---:|---:|---:|---:|---:|
/// | `15°` | `3,1 %` | `6,9 %` | `8,9 %` | `10,2 %` | `10,7 %` |
/// | `25°` | `5,1 %` | `10,9 %` | `13,8 %` | `15,8 %` | `16,5 %` |
/// | **`40°`** | `7,9 %` | `14,9 %` | **`18,4 %`** | `20,7 %` | `21,5 %` |
/// | `60°` | `12,0 %` | `22,6 %` | `27,7 %` | `31,2 %` | `32,4 %` |
///
/// ⚠️ **Maior que os `25°` da família das mídias, e de propósito:** ali a pergunta é *«as três mídias
/// dobram igual?»*, que se lê numa curva suave; aqui é *«este número muda a deformação?»*, e a
/// resposta cresce com a dobra. ⛔ Os `60°` ficam fora porque a ponta sobe `0,80 ×` o comprimento da
/// corda e as três fileiras deixam de caber no ecrã — *uma cena que sai do enquadramento ensina o
/// contrário do que diz*.
const DOBRA: f32 = 40.0;

/// ⭐⭐⭐ **O ALCANCE DA SEGUNDA CORDA.**
///
/// ⚠️⚠️ **Abaixo de `1` o envelope é INERTE, e isso está medido:** `0,4 → 1` dá desvio
/// **`0,0000`**, porque com o alcance curto cada ponto é dominado pelo osso dele e não há disputa
/// nenhuma. *O alcance só decide quando DOIS ossos chegam ao mesmo ponto* — ⇒ a segunda corda tem de
/// levar um alcance MAIOR que o de fábrica, nunca menor.
///
/// `4` e não `8`: da tabela da [`DOBRA`], `4` compra `18,4 %` e `8` compra `21,5 %` — o joelho está
/// em `4`, e uma mancha de alcance `8` mede `8 ×` o osso e cobre a cena inteira, que é onde o gizmo
/// deixa de dizer alguma coisa.
const ALCANCE_FORTE: f64 = 4.0;

/// ⭐⭐ **E A ORDEM É ERRO DE COMPILAÇÃO, não um `assert!` de teste.**
///
/// ⛔ Um `assert!` sobre duas CONSTANTES é dobrado pelo compilador antes de correr, e o clippy
/// di-lo em voz alta (`this assertion has a constant value`) — foi ele que apanhou a 1.ª redacção
/// deste gate, exactamente como na cena do personagem. *Uma afirmação que o compilador já resolveu
/// não é um teste; num `const` ela é uma PROPRIEDADE.*
///
/// ⚠️ **E ela guarda uma MEDIÇÃO:** abaixo de `1` o envelope é inerte (`0,4 → 1` dá desvio
/// `0,0000`), porque com o alcance curto cada ponto é dominado pelo osso dele e não há disputa
/// nenhuma. Uma cena que pusesse a segunda corda num alcance MENOR mostraria duas cordas idênticas
/// — que é, à letra, o report do dono.
const _: () = assert!(ALCANCE_FORTE > 1.0);

/// **O comprimento de uma corda, em metros de mundo.**
///
/// ⚠️ **Derivado do ENQUADRAMENTO, não escolhido.** Esta cena não pede *Frame All* (a razão medida
/// vive no `Prologo::enquadrar` da família das mídias: com um painel aberto ele corta sempre), logo
/// ela abre na câmera de omissão — `Camera2d::default().height_world`. A ponta de uma corrente de
/// `OSSOS` ossos dobrada [`DOBRA`] por junta sobe `(L/OSSOS) · Σ sin(k·θ)`, e as três fileiras mais
/// os vãos têm de caber na altura que sobra depois dos docks (~`2/3`). Ver
/// [`tests::as_tres_fileiras_cabem_na_camera_de_omissao`], que faz a conta.
const CORDA_M: f64 = 3.0;

/// A espessura da faixa preenchida, e a folga vertical da corda aberta.
const ESPESSURA_M: f64 = 0.24;

/// Os nomes que a Hierarquia mostra — ⚠️ **o roteiro cita-os, logo eles são uma AFIRMAÇÃO sobre o
/// que está na tela** e vivem num sítio só.
const NOMES: [&str; 3] = ["Corda (alcance 1)", "Corda (alcance 4)", "Barra preenchida"];

/// A subida da ponta de uma corrente dobrada, em metros — a lei fechada que dimensiona a cena.
fn subida(corda: f64) -> f64 {
    let osso = corda / f64::from(OSSOS);
    let t = f64::from(DOBRA).to_radians();
    (1..OSSOS).map(|k| (f64::from(k) * t).sin()).sum::<f64>() * osso
}

/// O passo entre duas fileiras: a banda de uma corda dobrada mais um quarto de folga.
fn passo() -> f64 {
    let banda = ESPESSURA_M + subida(CORDA_M);
    banda + ESPESSURA_M
}

/// ⭐⭐⭐ **O PRÓLOGO DESTA CENA — e a DECISÃO é da cena, o EFEITO é da shell.**
///
/// ⛔⛔⛔ **A FOTO de 2026-09-19 obrigou-o.** Sem ele a cena abre na câmera de omissão com a
/// arrumação que o dono tem em `~/.ph2d/layout.txt` — e ali a **timeline está aberta**, come um
/// terço da altura, e a terceira fileira (a barra preenchida, que é metade do report) ficava
/// **cortada pela borda de baixo**. *Uma cena que sai do enquadramento ensina o contrário do que
/// diz*, e esta família já o pagou três vezes.
///
/// ⚠️ **FECHAR a timeline e SÓ DEPOIS pedir o *Frame All*, nesta ordem.** A família das mídias
/// mediu que o `All` ajusta ao rectângulo da **JANELA** enquanto os painéis são desenhados por
/// cima: com a timeline aberta (~`1/3` da altura) **nenhum tamanho de cena sobrevive**. ⇒ aqui a
/// cena fecha-a — ela não anima nada, logo não tem o que fazer com ela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prologo {
    /// A timeline tem de estar **fechada**: com ela aberta o *Frame All* corta sempre.
    pub timeline_fechada: bool,
    /// Pedir o *Frame All* — ⚠️ **depois** de a fechar.
    pub enquadrar: bool,
    /// ⭐⭐⭐ **Abrir o painel de ossos** — a FOTO de 2026-09-19 obrigou-o.
    ///
    /// ⛔⛔ O passo (3) do roteiro manda ler a fileira `Deform By` *«no painel Bones»*, e ele
    /// **nasce fechado** (`DEFAULT_VISIBLE = false`): ele só se abre sozinho quando um OSSO é
    /// escolhido, e ali o artista está a escolher um DESENHO. ⇒ *um passo que nomeia uma linha de
    /// painel afirma que ela está lá, e o dono aprova o smoke com o passo impossível dentro.*
    pub painel_do_osso: bool,
}

/// A lei do [`Prologo`], **por NÍVEL**. ⚠️ **Pura aqui e efeito na shell**, como o molde da família
/// das mídias: abrir e fechar um painel é coisa da `App`, e a DECISÃO é gateável sem janela nenhuma.
///
/// ⭐⭐⭐ **Ela recebe o nível em vez de a ponte perguntar `== 2`, e isso é uma escolha MEDIDA.**
/// Uma prova de mutação (`V9`) trocou o guarda da ponte por `if false` e **nenhum gate acusou** —
/// *um gate de texto afirma que o código EXISTE, nunca que ele CORRE*. ⇒ a ponte deixa de ter um
/// literal e passa a ter **uma chamada só, incondicional**, e a inércia do `=1` vira uma lei PURA
/// que um gate mede.
///
/// ⚠️ **O `=1` tem de sair INERTE**, e não é conforto: aquela é a cena que o dono já aprovou, e
/// fechar-lhe a timeline ou reenquadrá-la seria mudá-la por baixo da mesa.
#[must_use]
pub const fn prologo_do_nivel(n: u32) -> Prologo {
    Prologo {
        timeline_fechada: n == 2,
        enquadrar: n == 2,
        painel_do_osso: n == 2,
    }
}

/// **O 1.º tempo: desenha as três peças e monta as três correntes.**
///
/// ⚠️ **Prender fica para o 2.º tempo** porque a entidade de uma forma é criada pelo
/// `vec_entities::sync`, que corre DEPOIS do prólogo — é a mesma máquina da cena `=1`.
pub(crate) fn build(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    let p = passo();
    let mut pecas = Vec::new();
    for k in 0..NOMES.len() {
        let y = p * (1.0 - k as f64);
        // ⭐⭐ **As duas primeiras são um ARCO ABERTO e a terceira é uma faixa FECHADA** — a única
        // diferença que a cena tem a mostrar. ⚠️ A cor separa o par (as duas cordas) do controlo.
        // ⭐⭐⭐ **O CONTROLO é o MESMO ARCO, FECHADO — e a FOTO impôs essa escolha.**
        //
        // ⛔⛔ A 1.ª redacção pôs ali um `RoundRect`, e na foto ele saiu como uma **barra recta com
        // um vinco**: a deformação de uma forma vectorial é aplicada aos PONTOS DE CONTROLO, e um
        // rectângulo arredondado tem oito — logo a curva entre eles fica esticada. *Uma peça de
        // controlo que se lê como partida ensina que o app está partido.*
        //
        // ⭐ O [`ShapeKind::Segment`] é o arco fechado pela CORDA: a MESMA curva das duas cordas,
        // com os mesmos pontos, só que preenchida. ⚠️ E ele é **INERTE ao alcance**, medido ao lado
        // das outras formas fechadas (`amplitude 0,000000` numa faixa de `80 ×`).
        let (kind, cor) = if k < 2 {
            (ShapeKind::Arc, [235, 170, 90])
        } else {
            (ShapeKind::Segment, [120, 170, 235])
        };
        // ⚠️ **A peça fechada leva uma caixa mais ALTA**, e não é gosto: a lente entre o arco e a
        // corda dele tem a altura da caixa — com a espessura de uma corda ela sairia um fio, e o
        // controlo seria invisível exactamente onde se quer ler que ele não tem mancha.
        let meia = if k < 2 {
            ESPESSURA_M / 2.0
        } else {
            ESPESSURA_M * 2.0
        };
        let id = scene.push_path(shape(
            kind,
            [-CORDA_M / 2.0, y - meia],
            [CORDA_M / 2.0, y + meia],
            &[],
            cor,
        ));
        // ⛔⛔ **UM CAMINHO ABERTO NÃO SE VÊ PELO PREENCHIMENTO, e a FOTO é que o disse.** A 1.ª
        // redacção desta cena pintava as três com `fill`, e as duas cordas saíram como um fio fino
        // da cor de omissão do traço — *a peça que a cena existe para mostrar era a menos visível
        // do ecrã*. ⇒ um caminho aberto leva um TRAÇO grosso, que é o que um artista desenha quando
        // desenha uma corda.
        if kind == ShapeKind::Arc
            && let Some(path) = scene.path_mut(id)
        {
            path.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
                ph2d_vec_scene::Rgba8::new(cor[0], cor[1], cor[2], 255),
                ESPESSURA_M,
            ));
            // ⚠️ E o preenchimento SAI: um arco aberto preenchido desenha a corda e a CORDA da
            // corda (o fecho implícito entre as pontas), e a segunda lê-se como um defeito.
            path.fill = None;
        }
        // ⭐ A corrente de cada fileira, com o alcance do osso do MEIO escrito só na segunda.
        let raiz = corrente(sim, y, if k == 1 { ALCANCE_FORTE } else { 1.0 });
        pecas.push((id, raiz));
    }
    st.bone_smoke_pend = Some(pecas);
    st.bone_smoke_step = 1;
}

/// Uma corrente de [`OSSOS`] ossos deitada em `y`, com o do MEIO a levar `alcance`.
///
/// ⚠️ **Só o do meio**, e é isso que faz a diferença medida ser do ENVELOPE: mudar a corrente
/// inteira mudaria também quanto cada ponta alcança para fora da arte.
fn corrente(sim: &mut SimWorld, y: f64, alcance: f64) -> Option<Entity> {
    let passo_do_osso = CORDA_M / f64::from(OSSOS);
    let x0 = -CORDA_M / 2.0;
    let mut pai: Option<Entity> = None;
    let mut raiz = None;
    for k in 0..OSSOS {
        let a = [x0 + passo_do_osso * f64::from(k), y];
        let b = [x0 + passo_do_osso * f64::from(k + 1), y];
        let bits = ph2d_skeleton_live::bone::create(sim, pai, a, b)?;
        let e = Entity::try_from_bits(bits)?;
        if k == OSSOS / 2
            && let Some(mut osso) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(e)
        {
            osso.strength = alcance;
        }
        raiz.get_or_insert(e);
        pai = Some(e);
    }
    raiz
}

/// **O 2.º tempo: prende as três e dobra as três.**
///
/// ⚠️ **A ordem é LEI:** prender fotografa a pose de repouso, e com a corrente já dobrada as três
/// peças nasciam presas à pose torta e a cena abria imóvel.
pub(crate) fn bind(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    st.bone_smoke_step = 2;
    let Some(pecas) = st.bone_smoke_pend.take() else {
        return;
    };
    let map = &st.entities;
    let mut presas = 0;
    for (k, (id, raiz)) in pecas.iter().enumerate() {
        presas += ph2d_skeleton_live::skin_live::bind(sim, scene, map, &[*id], *raiz);
        // ⭐⭐ **O NOME é escrito AQUI e não no 1.º tempo**, e não é arrumação: quem cria a entidade
        // de uma forma é o `vec_entities::sync`, que corre DEPOIS do prólogo — no 1.º tempo não há
        // onde o escrever. ⚠️ E ele importa: o roteiro manda clicar numa linha da Hierarquia, logo
        // *um passo que nomeia uma linha AFIRMA que ela está lá*, e «Path 7» não afirma nada.
        if let Some(e) = map.get(id).and_then(|b| Entity::try_from_bits(*b))
            && let Some(nome) = NOMES.get(k)
        {
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_ecs::Name::new(*nome));
        }
    }
    for (_, raiz) in &pecas {
        let Some(raiz) = *raiz else { continue };
        let mut e = raiz;
        while let Some(f) = sim
            .world()
            .get::<ph2d_ecs::Children>(e)
            .and_then(|c| c.iter().next().copied())
        {
            e = f;
            if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
                t.rotation += DOBRA.to_radians();
            }
        }
    }
    anuncia(presas);
}

/// O roteiro.
///
/// ⚠️ **Ele nomeia o que se vê NA TELA** (`Deform By`, `Bone Reach`, os nomes das três fileiras) —
/// *um passo que manda clicar numa linha de painel AFIRMA que ela está lá*, e o dono aprova o smoke
/// com o passo impossível dentro.
fn anuncia(presas: usize) {
    if presas < NOMES.len() {
        eprintln!(
            "[vec-bone-smoke] PARE: prendi so' {presas} de {} pecas",
            NOMES.len()
        );
    }
    println!(
        "[vec-bone-smoke] O ENVELOPE: tres desenhos, a MESMA corrente de {OSSOS} ossos e a MESMA \
         dobra ({DOBRA}° por junta). So' muda o ALCANCE do osso do meio.\n\
         [vec-bone-smoke] 1) Olhe as DUAS cordas cor de laranja («{}» em cima e «{}» no meio): e' o \
         MESMO desenho com o MESMO esqueleto, e elas acabam em sitios DIFERENTES. Essa diferenca e' \
         o envelope — e ela vale {ALCANCE_FORTE:.0}x o alcance de fabrica\n\
         [vec-bone-smoke] 2) Na Hierarquia clique no osso do MEIO da corda de cima: no canvas \
         aparece uma MANCHA a' volta dele (ate' onde ele alcanca) com uma ALCA na borda. Arraste \
         essa alca para FORA: a corda muda de forma enquanto arrasta\n\
         [vec-bone-smoke] 3) Agora a ESCOLHA: na Hierarquia clique na «{}» (a peca AZUL, em baixo). \
         No painel Bones aparece a fileira «Deform By» com dois botoes — ela esta' em «Artwork», e \
         por isso a peca azul nao tem mancha nenhuma\n\
         [vec-bone-smoke] 4) Carregue em «Bone Reach»: a peca azul MUDA DE FORMA na hora, e o osso \
         do meio dela ganha a mancha e a alca. Arraste a alca — ela obedece como as cordas\n\
         [vec-bone-smoke] 5) Carregue em «Artwork» outra vez: ela volta EXACTAMENTE ao que era, \
         sem esperar nada. A conta boa ficou guardada desde que voce^ prendeu\n\
         [vec-bone-smoke] ⇒ e' essa a escolha, e ela e' POR DESENHO: cada peca tem a sua, e mexer \
         numa nao mexe nas outras\n\
         [vec-bone-smoke] Se carregar em «Bone Reach» e a peca azul nao mudar, PARE e diga",
        NOMES[0], NOMES[1], NOMES[2]
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐⭐ **O BLOCO TEM DE SER QUASE QUADRADO** — a lei de enquadramento desta casa, MEDIDA.
    ///
    /// ⛔⛔ **A premissa anterior deste gate MORREU na foto de 2026-09-19, e a morte fica aqui.**
    /// Ele media se o bloco cabia na **câmera de omissão**, porque a cena não pedia *Frame All* —
    /// e a foto mostrou a terceira fileira **cortada pela borda de baixo**, porque a arrumação do
    /// dono abre a timeline e ela come um terço da altura. ⇒ a cena passou a **fechar a timeline e
    /// a pedir o `All`** ([`prologo`]), e a grandeza que decide deixou de ser a altura absoluta.
    ///
    /// ⚠️ **O que decide agora é a FORMA do bloco**, e a lei é a que a família das mídias mediu: o
    /// `All` ajusta a altura a `max(span_y, span_x / aspecto_da_JANELA) × 1,1`, e os painéis tapam
    /// `~37 %` da largura ⇒ a largura VISÍVEL é `altura × aspecto_do_CANVAS`. Com o `span_x` a
    /// mandar isso dá `span_x × 0,81 < span_x` **para qualquer tamanho** — *uma cena mais larga do
    /// que alta nunca cabe, e encolhê-la não ajuda.*
    #[test]
    fn o_bloco_e_quase_quadrado_senao_o_frame_all_corta() {
        let alto = passo() * (NOMES.len() as f64 - 1.0) + ESPESSURA_M + subida(CORDA_M);
        assert!(
            alto >= CORDA_M,
            "o bloco mede {alto:.3} m de alto por {CORDA_M:.3} m de largo — com o span_x a mandar, \
             o Frame All entrega {:.3} m de largura visivel e as pontas ficam debaixo de um dock",
            CORDA_M * 0.81
        );
        // ⭐ E a metade de cima: um bloco muito mais alto que largo desperdiça a largura e as
        // cordas ficam pequenas no ecrã, que é onde a diferença entre elas deixa de se ver.
        assert!(
            alto <= CORDA_M * 2.5,
            "o bloco mede {alto:.3} m de alto para {CORDA_M:.3} m de largo: as cordas ficam \
             pequenas no ecra' e a diferenca entre as duas, que e' uma FRACCAO do comprimento \
             delas, deixa de se ver"
        );
    }

    /// ⭐⭐ **E O PRÓLOGO FECHA A TIMELINE ANTES DE ENQUADRAR** — as duas metades de uma decisão só.
    ///
    /// ⛔ Sem a primeira o `All` corta (medido na foto); sem a segunda a cena abre na câmera de
    /// omissão, onde o bloco fica descentrado (a área do canvas não é a da janela).
    #[test]
    fn o_prologo_fecha_a_timeline_e_enquadra() {
        let p = prologo_do_nivel(2);
        assert!(
            p.timeline_fechada && p.enquadrar && p.painel_do_osso,
            "o prologo desta cena deixou de fazer as duas coisas: {p:?} — com a timeline aberta o \
             Frame All corta sempre, e sem ele o bloco abre descentrado"
        );
        // ⭐⭐⭐ **E a metade NEGATIVA vale tanto como a outra**, porque a ponte chama isto SEM
        // guarda: a cena `=1` é a que o dono já aprovou, e um prólogo que armasse ali fechar-lhe-ia
        // a timeline e reenquadrá-la-ia por baixo da mesa.
        let um = prologo_do_nivel(1);
        assert!(
            !um.timeline_fechada && !um.enquadrar && !um.painel_do_osso,
            "o prologo passou a armar a cena =1, que o dono ja' aprovou sem ele: {um:?}"
        );
    }
}

#[cfg(test)]
#[path = "smoke_bone_envelope_texto_tests.rs"]
mod smoke_bone_envelope_texto_tests;
