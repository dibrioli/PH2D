//! Gates da tabela de slots, **sem device**.
//!
//! Uma [`super::Sculpt3dScene`] não nasce sem um `wgpu::Device`, e é por isso
//! que a decisão é uma função livre: o que pode estar errado aqui é *quem mora
//! em cada slot*, e essa pergunta não tem nada a ver com uma GPU.

use super::{ObjectId, PieceState, SlotJob, plan_slots};

fn piece(piece: usize, id: u32, uploaded: bool, dirty: bool) -> PieceState {
    PieceState {
        piece,
        id: ObjectId(id),
        uploaded,
        dirty,
    }
}

fn jobs(slots: &[u32], visible: &[PieceState]) -> Vec<SlotJob> {
    let slots: Vec<ObjectId> = slots.iter().copied().map(ObjectId).collect();
    plan_slots(&slots, visible).iter().map(|l| l.job).collect()
}

/// **O SLOT QUE HERDA MUDA DE DONO, E ELE TEM DE RECEBER A MALHA INTEIRA.**
///
/// ⚠️ Este é o gate do bug VIVO que a wave achou: apagar a peça 0 de três
/// desloca as sobreviventes para os slots 0 e 1, as duas com `uploaded == true`.
/// A guarda antiga perguntava *"o slot existe?"* — e existe. O device continuava
/// com a geometria da peça morta no slot 0, desenhada na pose da sobrevivente,
/// **sem erro nenhum**.
#[test]
fn a_slot_that_changed_owner_gets_the_whole_mesh() {
    // O device tem as peças 7, 8, 9 nos slots 0, 1, 2. A peça 7 foi apagada.
    let visible = [piece(0, 8, true, false), piece(1, 9, true, false)];
    assert_eq!(
        jobs(&[7, 8, 9], &visible),
        vec![SlotJob::Full, SlotJob::Full],
        "as duas sobreviventes deslocaram de slot: as duas sobem inteiras"
    );
}

/// **E O SLOT QUE NÃO MUDOU DE DONO NÃO RECEBE NADA.**
///
/// ⚠️ O par do gate acima, e ele é o que impede a cura de virar *"suba tudo
/// todo frame"*: com quatro peças de 100k vértices isso é a malha inteira da
/// cena por quadro, e o gesto que este módulo mede em milissegundos (o dab)
/// passaria a pagar por peças que ninguém tocou.
#[test]
fn a_settled_slot_receives_nothing() {
    let visible = [piece(0, 7, true, false), piece(1, 8, true, false)];
    assert_eq!(
        jobs(&[7, 8], &visible),
        vec![SlotJob::Skip, SlotJob::Skip],
        "o device já tem as duas, e elas não mudaram"
    );
}

/// **O DAB SOBE SÓ A REGIÃO** — o caminho quente, e ele sobrevive à testemunha.
#[test]
fn a_dab_uploads_only_its_region() {
    let visible = [piece(0, 7, true, true)];
    assert_eq!(jobs(&[7], &visible), vec![SlotJob::Region]);
}

/// **UMA PEÇA QUE A CPU RECONSTRUIU SOBE INTEIRA, mesmo no slot dela.**
///
/// É a outra metade da pergunta: a testemunha responde *o device tem esta
/// peça?*, e o `uploaded` responde *a malha dela é a mesma?*. As duas são
/// necessárias — um `Ctrl+Z` reconstrói a malha sem mover ninguém de slot.
#[test]
fn a_rebuilt_piece_uploads_in_full_even_in_its_own_slot() {
    let visible = [piece(0, 7, false, false)];
    assert_eq!(jobs(&[7], &visible), vec![SlotJob::Full]);
}

/// **O SLOT QUE AINDA NÃO EXISTE recebe a malha inteira** — a peça nova, e o
/// primeiro frame de toda cena.
#[test]
fn a_slot_that_does_not_exist_yet_gets_the_whole_mesh() {
    let visible = [piece(0, 7, true, false), piece(1, 8, true, false)];
    assert_eq!(
        jobs(&[7], &visible),
        vec![SlotJob::Skip, SlotJob::Full],
        "a segunda peça nunca esteve no device"
    );
}

/// **O ISOLAMENTO COMPACTA, e a peça isolada pode cair num slot alheio.**
///
/// ⚠️ Isolar a peça 2 de três a leva do slot 2 para o slot 0 — que tem a
/// geometria da peça 7. Sem a testemunha, o artista isolaria uma peça e veria
/// **outra**: o defeito do delete, pela porta que esta wave abriu.
#[test]
fn isolating_a_piece_moves_it_to_slot_zero_and_it_uploads_there() {
    // Só a peça 9 está à vista, e ela é o índice 2 da lista da cena.
    let visible = [piece(2, 9, true, false)];
    let plan = plan_slots(&[ObjectId(7), ObjectId(8), ObjectId(9)], &visible);
    assert_eq!(plan.len(), 1, "um slot só, porque só uma peça está à vista");
    assert_eq!(plan[0].piece, 2, "e ele desenha a peça 2 da CENA");
    assert_eq!(
        plan[0].job,
        SlotJob::Full,
        "no slot 0, que era de outra peça: sobe inteira"
    );
}

/// **A SUBIDA DO CANAL DE PREVIEW NÃO MORA DENTRO DO `match` DO PLANO.**
///
/// ⚠️ **Arch-gate sobre o fonte porque nenhum teste de CPU alcança isto:** o
/// `sync_mesh` exige um `wgpu::Device` e uma `Queue`, e o defeito é de POSIÇÃO —
/// com a chave do padrão mudada e nenhum vértice movido, o plano do slot cai em
/// [`SlotJob::Skip`] e um upload escrito dentro do braço `Region` nunca roda no
/// caso exato que ele existe para cobrir. O sintoma seria o barro com o padrão
/// ANTERIOR enquanto o quadro do painel já mostra o novo — dois previews
/// discordando, que é pior que preview nenhum.
///
/// A propriedade afirmada é a RELAÇÃO (a chamada vem DEPOIS do `}` que fecha o
/// `match`), nunca uma distância em bytes: um proxy de distância expira na
/// primeira linha que alguém acrescenta no meio.
#[test]
fn the_whole_preview_upload_lives_outside_the_slot_job_match() {
    let src = std::fs::read_to_string("src/sculpt3d_slots.rs")
        .expect("o roteador de slots é legível a partir do pacote");
    let call = src
        .find("upload_preview_at")
        .expect("a porta do canal inteiro sumiu do `sync_mesh`");
    let m = src
        .find("match line.job {")
        .expect("o `match` do plano sumiu");
    assert!(call > m, "o upload do canal inteiro precede o `match`");

    // O `}` que FECHA o match: a linha de fechamento na indentação do `match`.
    let close = src[m..]
        .find("\n            }\n")
        .map(|o| m + o)
        .expect("o `match` do plano não fecha na indentação esperada");
    assert!(
        call > close,
        "o upload do canal inteiro está DENTRO de um braço do `match` — com a \
         chave do padrão mudada e nada sujo o plano cai em `Skip`, e ele nunca roda"
    );
}
