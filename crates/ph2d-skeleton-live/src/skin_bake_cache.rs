//! ⭐⭐⭐ **A MALHA ASSADA, GUARDADA POR BIND** — a metade da W2 da F9 que não precisa da placa
//! (`docs/Skeleton/01_a_fila.md`).
//!
//! # A pergunta que este memo responde: ONDE a malha assada vive
//!
//! A W1 provou que a densidade pode sair do QUADRO e ir para o BIND — assar uma vez, em repouso,
//! onde o campo de pesos curva ([`crate::skin_bake`]). Faltava decidir onde o resultado mora, e a
//! diferença entre as duas casas é de **PRODUTO**, não de relógio:
//!
//! - **No DOCUMENTO** (a 1.ª redacção da W1b, dentro do [`crate::skin_live::bind_image`]): a malha
//!   assada SUBSTITUI a do bind nos bytes guardados. ⛔ Três consequências que nenhum número
//!   desculpa — o `Fast` deixa de ser barato (ele passa a desenhar a malha fina, `5,76×` maior na
//!   arte do dono), a escolha `Fast`/`Smooth` do painel **COLAPSA** (as duas desenham exactamente a
//!   mesma malha), e a densidade fica **congelada no ficheiro**, onde o artista não a desfaz sem
//!   re-prender a arte.
//! - **Num MEMO por bind** (esta folha): a malha assada é **DERIVADA**, e o painel continua a
//!   escolher — `Fast` desenha a do bind, `Smooth` desenha a assada. *Estado derivado guardado no
//!   documento é o que envenena o undo*, que é a lei que o `FieldProfileSource` do campo implícito
//!   já escreve por extenso.
//!
//! ⭐⭐ **E é o mesmo memo que a placa vai querer**: quando o *vertex shader* posar, o que sobe uma
//! vez por bind é exactamente esta malha (repouso + tabela de pesos). *A casa é a mesma; muda quem
//! a lê.*
//!
//! # ⚠️ A chave é a ENTIDADE e a prova é o CONTEÚDO
//!
//! ⛔ **`Entity::to_bits()` sozinho não serve como identidade durável** — é a lei que este módulo já
//! pagou (`degrau 122` da escada: o undo re-spawna e os bits são ids de alocação). Aqui ele é só o
//! **endereço** da gaveta; quem diz se o conteúdo ainda serve é a **igualdade byte a byte** da
//! fonte. ⇒ bits reciclados por outra arte dão uma falha de comparação e uma assadura nova, nunca a
//! malha errada.
//!
//! ⚠️ **E a comparação é barata ao lado do que ela evita:** os bytes do bind da arte do dono medem
//! `~100 KiB` (memcmp a `~10 GB/s` ⇒ `~10 µs`) contra a assadura, que é `O(peças)` de refinamento.
//! ⛔ Uma função de dispersão criptográfica (o `DefaultHasher` do `std` é SipHash, `~1 GB/s`) seria
//! **dez vezes mais cara que a prova exacta** — *uma chave derivada só compensa quando comparar o
//! original é caro, e aqui não é.*

use crate::skinned_mesh::SkinnedMesh;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// ⚠️ **Quantos binds distintos o memo guarda** — o recurso é MEMÓRIA, e o número é derivado dela:
/// a malha assada da arte do dono ocupa `~280 KiB` (`13 996` triângulos × 12 B + `7 k` vértices ×
/// 16 B + a tabela de pesos), logo `64` entradas são `~18 MiB` no pior caso de uma cena em que
/// **todas** as imagens presas fossem daquele tamanho. Uma cena real tem partes mais pequenas.
///
/// ⛔ Não é um tecto de *quantas imagens o produto suporta*: passar dele custa uma assadura nova na
/// imagem menos usada, nunca um desenho errado.
const ENTRADAS_MAX: usize = 64;

/// Uma gaveta: a fonte que a produziu, a assada, e quando foi usada pela última vez.
struct Entrada {
    fonte: Vec<u8>,
    assada: Option<Rc<SkinnedMesh>>,
    visto: u64,
}

thread_local! {
    static MEMO: RefCell<BTreeMap<u64, Entrada>> = const { RefCell::new(BTreeMap::new()) };
    /// O relógio do memo — conta CONSULTAS, não quadros: ele só precisa de ordenar quem foi usado
    /// há mais tempo, e uma consulta é a única coisa que esta folha vê acontecer.
    static RELOGIO: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// ⭐⭐⭐ **A MALHA ASSADA DESTE BIND** — do memo, ou assada agora e guardada.
///
/// `bits` é o endereço (a entidade da arte) e `fonte` são os bytes do `SkinBind::source` que a
/// provam. `crua` é a malha já descodificada, que o chamador tem na mão.
///
/// `None` quando a assadura não parte nada (o campo de pesos já é linear em toda aresta, ou a porta
/// está fechada) — e aí o chamador desenha a do bind, que é o caminho de omissão byte-idêntico.
/// ⚠️ **O `None` também é GUARDADO**: sem isso, uma arte que a assadura recusa pagaria a tentativa
/// a cada quadro, que é o caso mais caro possível para o resultado mais barato possível.
///
/// ⚠️⚠️ **`assar` é PARÂMETRO, e a razão é a porta ser um `OnceLock`:** o `PH2D_SKIN_BAKE` é lido
/// uma vez por processo, logo um gate que a escrevesse mediria o que o teste vizinho já tinha
/// fixado — a armadilha que esta família já pagou (*«`env VAR=` define a variável VAZIA, e o
/// CONTROLO passou a correr a mesma lei que devia contradizer»*). ⇒ *a lei é parâmetro e a porta é
/// um `if` de uma linha, em quem chama.*
#[must_use]
pub fn assada_do_bind(
    bits: u64,
    fonte: &[u8],
    crua: &SkinnedMesh,
    assar: impl FnOnce(&SkinnedMesh) -> Option<SkinnedMesh>,
) -> Option<Rc<SkinnedMesh>> {
    let agora = RELOGIO.with(|c| {
        let t = c.get() + 1;
        c.set(t);
        t
    });
    MEMO.with(|m| {
        let mut m = m.borrow_mut();
        if let Some(e) = m.get_mut(&bits)
            && e.fonte == fonte
        {
            e.visto = agora;
            return e.assada.clone();
        }
        let assada = assar(crua).map(Rc::new);
        m.insert(
            bits,
            Entrada {
                fonte: fonte.to_vec(),
                assada: assada.clone(),
                visto: agora,
            },
        );
        // A gaveta menos recentemente USADA sai.
        //
        // ⚠️⚠️ **O que protege a recém-assada é o RELÓGIO dela ser o maior, e não um `filter` a
        // saltá-la** — a 1.ª redacção tinha o `filter`, e a mutação que o apagava **SOBREVIVEU**:
        // com `visto = agora` a entrada nova nunca pode ser o mínimo. *Uma linha que a mutação não
        // consegue matar não é lei, é comentário com sintaxe de código.* ⇒ a mutação que morde é
        // pôr a nova a nascer com o relógio a zero, e é essa que o gate mede.
        while m.len() > ENTRADAS_MAX {
            let Some(velha) = m.iter().min_by_key(|(_, e)| e.visto).map(|(k, _)| *k) else {
                break;
            };
            m.remove(&velha);
        }
        assada
    })
}

/// A malha que o `Smooth` desenha: a **ASSADA** deste bind quando ela existe, a do bind quando não.
///
/// ⚠️ **A cópia é o preço de a porta que posa receber a malha POR VALOR** (ela move os triângulos
/// para dentro do [`SpriteMesh`]), e é `~280 KiB` de memcpy por imagem por quadro na arte do dono —
/// ao lado dos `~500 µs` que a deformação dela custa. *É mais uma coisa que o caminho da placa
/// remove, e não uma razão para o memo não existir:* sem ele pagava-se a assadura inteira.
///
/// `None` = *«desenhe a do bind»*, e ele tem duas causas legítimas: a porta está fechada, ou o
/// campo de pesos desta arte já é linear em toda aresta.
///
/// ⛔ Sem `SkinBind` não há fonte que prove a gaveta ⇒ `None`. Aquele componente é a razão de a
/// entidade estar nesta lista, logo o caso não acontece; o que ele faz é impedir que a identidade do
/// memo venha de outro sítio que não a fonte.
#[must_use]
pub fn assada_da_arte(
    sim: &ph2d_ecs::SimWorld,
    e: ph2d_ecs::Entity,
    crua: &SkinnedMesh,
) -> Option<SkinnedMesh> {
    let skin = sim.world().get::<ph2d_skeleton_ecs::SkinBind>(e)?;
    // ⚠️ **A PORTA está AQUI** (`assar_no_bind`, `PH2D_SKIN_BAKE=1`) e não dentro do memo: ele
    // guarda o que a porta responde, e com ela fechada a resposta é `None` em toda a arte — o
    // caminho de omissão continua byte-idêntico, e a gaveta que fica a dizer *«esta não tem
    // assada»* é o que evita voltar a perguntar no quadro seguinte.
    let assada = assada_do_bind(e.to_bits(), &skin.source, crua, |m| {
        crate::skin_bake::assar_no_bind(&m.mesh, &m.pesos, m.ossos())
            .map(|(mesh, pesos)| SkinnedMesh { mesh, pesos })
    })?;
    Some((*assada).clone())
}

/// Esvazia o memo — para os gates, e para quem quiser medir a assadura a frio.
pub fn esquece_tudo() {
    MEMO.with(|m| m.borrow_mut().clear());
}

/// Quantas gavetas o memo tem agora — só os gates perguntam.
#[must_use]
pub fn gavetas() -> usize {
    MEMO.with(|m| m.borrow().len())
}

#[cfg(test)]
#[path = "skin_bake_cache_tests.rs"]
mod tests;
