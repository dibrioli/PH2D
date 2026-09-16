//! ⭐⭐ **O PASSO RESOLVE O CONTACTO ENTRE PEÇAS** — doc 109, ordem do dono (2026-09-13):
//! *«colidem sozinhas»*.
//!
//! Todo motor faz num `step` duas coisas: integra e resolve contatos. Este nó fazia a primeira;
//! a segunda passa a correr aqui, **depois** da integração, sobre as peças que declaram colisor
//! (as colunas `collider` / `collider_box`, escritas por quem desenha). Sem nenhuma das duas nada
//! aqui corre, e o passo é o de sempre.
//!
//! ## A posição: a lei da folha `ph2d-contact`
//!
//! Cada peça é o colisor que DECLAROU — um disco ou uma caixa, escalado pelo `size` e girado pelo
//! `rot` dela (doc 109 §5) —, lido pela porta `ph2d_contact::colisores`; e o peso é o `inv_mass`,
//! para um pino ser obstáculo.
//!
//! ## E a ROTAÇÃO (doc 109 §6)
//!
//! Report do dono: *«precisa destravar a rot»*. A correcção reparte-se entre mover e rodar pela
//! massa efectiva no PONTO do contacto, e o quanto cada peça rodou volta daqui em GRAUS para o
//! [`super::step`] somar à coluna `rot`. Quem trava é a coluna `inv_inertia` a `0` — o botão
//! `Lock Rotation` do cartão da forma.
//!
//! ## A velocidade: o IMPULSO DO PAR (doc 111 §5.12)
//!
//! ⚠️ **Sem resposta de velocidade a pilha RESPIRA:** a velocidade continuaria a empurrar a peça
//! para dentro da vizinha a cada tique, a posição seria corrigida outra vez, e o monte nunca
//! assentaria (é o `93 %` do vão que a cena `=114` mede com o `motion.collide`, que só mexe em `P`).
//!
//! ⛔⛔ **E a lei que aqui esteve até 2026-09-15 era POR PEÇA, sobre a velocidade ABSOLUTA de cada
//! uma** — report do dono: *«quando aumento bounciness… é como se uma fosse muito mais pesada que a
//! outra?»*. Ele leu-o exactamente: a peça PARADA tinha `vn = 0`, o guarda *«só se responde a quem
//! se aproxima»* disparava, e **ela nunca recebia velocidade nenhuma**. Nunca havia troca de
//! momento, e quem era atingido comportava-se como uma PAREDE — com o momento a **inverter-se**
//! (`+1,00` antes do choque, `−1,00` depois, com salto máximo).
//!
//! Hoje a resposta é o impulso clássico sobre a velocidade **RELATIVA**, repartido pelas massas,
//! mais o **atrito de Coulomb** na tangente — ver [`ph2d_contact::impulsos`], que tem as duas
//! tabelas. ⭐ As três leis de antes continuam a valer **por construção**: quem nasce sobreposto e
//! parado não ganha velocidade (`vrel = 0`), só se responde a quem se aproxima (`vrel > 0`), e um
//! obstáculo devolve o salto inteiro (`w = 0`).
//!
//! ## ⭐⭐⭐ E a ROTAÇÃO PERSISTE (doc 111 §9, 2026-09-16)
//!
//! Report do dono: *«umas caixas rodam, outras parecem não rotacionar»*. A rotação deixou de ser
//! uma projecção de posição e passou a ser **velocidade angular**, que o `sim.step` integra: uma
//! caixa atingida fora do centro gira a `101,9°/s` e **continua a girar** depois de se separarem,
//! contra `0,5341°` uma vez e congelar.
//!
//! ⛔⛔ **E a metade que faltava à 1.ª tentativa não era angular: era DESLIGAR a outra rotação.** A
//! separação de posição também roda (doc 109 §6) e acumula esse ângulo **sem lhe dar velocidade**,
//! logo nenhum atrito o pode travar. Com as duas a correr o rodopio da `=114` lê `42`; com uma só,
//! `2,6..3,1`. Ver [`ph2d_contact::Leis::EM_VIGOR`], que tem a tabela das cinco células.

use ph2d_nodegraph::attr::Stream;

/// **Varreduras por passo.** É o default que o `motion.collide` shipa, medido na tabela dele
/// (`measure_packing_and_order_dependence`: a folga mínima de uma nuvem apertada sobe de `0,050`
/// com Gauss–Seidel para `0,270` com este esquema a 8); e os `substeps` da zona multiplicam-no sem
/// mais nenhum knob.
pub(crate) const VARREDURAS: usize = 8;

/// **AS LEIS DO SOLVER DE VELOCIDADE** — uma escolha do produto, e o único sítio onde ela se faz.
///
/// ⚠️ Ela é um `const` e **não** uma variável de ambiente de propósito: uma bandeira global lida
/// dentro do solver alcançaria todo chamador dele (o defeito que o `remesh_with` da `line/sculpt3d`
/// pagou por escrito). Quem varre as células é a bancada da `ph2d-contact`, que chama a porta com
/// as leis na mão; aqui escolhe-se **uma**, com a tabela que a escolheu ao lado — doc 111 §9.
const LEIS: ph2d_contact::Leis = ph2d_contact::Leis::EM_VIGOR;

/// Separa as peças com colisor, troca o momento delas pelo IMPULSO do par, e devolve **quanto cada
/// uma rodou**, em graus (vazio quando ninguém declara colisor).
///
/// `antes_do_passo` é onde cada peça estava **antes de a integração a mover** — é dele que sai o
/// DESLIZE que o atrito opõe (doc 109 §7), e `girou` é o que o `spin` já rodou neste mesmo passo.
///
/// ⭐⭐ **Ele já não recebe o `dt`, e a ausência é o achado:** a lei antiga precisava dele para pôr
/// tecto (`|Δp| / dt`) a uma velocidade que ela própria inventava a partir da correcção de posição.
/// Um IMPULSO não precisa de tecto nenhum — ele é limitado pela velocidade RELATIVA que de facto
/// existe. *Um parâmetro que deixa de ser preciso é a medida de quanto a lei nova sabe a mais.*
pub(crate) fn resolve(
    state: &Stream,
    p: &mut [[f32; 2]],
    vel: &mut [[f32; 2]],
    pesos: &[f32],
    antes_do_passo: &[[f32; 2]],
    girou: &[f32],
    dt: impl Fn(usize) -> f32,
) -> (Vec<f32>, Vec<f32>) {
    let n = p.len();
    let Some(colisores) = ph2d_contact::colisores(state) else {
        return (Vec::new(), Vec::new());
    };
    if colisores.len() != n || antes_do_passo.len() != n || girou.len() != n {
        return (Vec::new(), Vec::new());
    }
    let inv_inercia = ph2d_contact::inv_inercias(state, &colisores, pesos);
    // ⭐ O MATERIAL de cada peça (doc 109 §7). Sem as colunas ele é `LISO` para todas, `μ = 0`, e
    // o solver devolve a lei de antes do §7 **ao bit** — é isso que dispensa migração nenhuma.
    let material =
        ph2d_contact::materiais(state).unwrap_or_else(|| vec![ph2d_contact::Material::LISO; n]);
    let (mut giro, mut salto) = (vec![0.0_f32; n], vec![0.0_f32; n]);
    let antes = p.to_vec();
    let pecas = ph2d_contact::Pecas {
        colisores: &colisores,
        pesos,
        inv_inercia: &inv_inercia,
        deslize: Some(ph2d_contact::Deslize {
            antes: antes_do_passo,
            girou_antes: girou,
            material: &material,
        }),
    };
    ph2d_contact::separate(
        p,
        &mut ph2d_contact::Saida {
            giro: &mut giro,
            salto: &mut salto,
        },
        &pecas,
        VARREDURAS,
    );
    // ⭐⭐⭐ **A VELOCIDADE responde pelo IMPULSO DO PAR** — report do dono (2026-09-15): *«quando
    // aumento bounciness… é como se uma fosse muito mais pesada que a outra?»*. Ver o cabeçalho de
    // [`ph2d_contact::impulsos`], que tem a tabela do defeito.
    //
    // ⚠️⚠️ **A lei que estava aqui era POR PEÇA, sobre a velocidade ABSOLUTA de cada uma**, na
    // direcção em que a correcção de posição a empurrara: a peça PARADA lia `vn = 0`, o guarda
    // «só se responde a quem se aproxima» disparava, e **ela nunca recebia velocidade nenhuma**.
    // Nunca havia troca de momento ⇒ quem era atingido comportava-se como uma PAREDE.
    //
    // ⛔ **E ela era medida sobre `p − antes`, que é a correcção TOTAL da peça** — a soma do que
    // todos os vizinhos lhe pediram. Isso não é a normal de contacto nenhum: numa pilha apertada
    // aponta para onde a peça calhou de ser espremida.
    //
    // ⚠️ O impulso corre sobre `antes` — as posições em que os contactos DE FACTO aconteceram.
    // Depois da separação as peças já não se sobrepõem, e ali não haveria par nenhum a encontrar.
    // ⚠️ **A rotação POSICIONAL, quando as leis a dispensam** — ver `Leis::giro_posicional`. Ela é
    // a projecção de despenetração (doc 109 §6) e corre a par da velocidade angular; zerá-la aqui é
    // o que permite medir qual das duas é que a pilha precisa.
    if !LEIS.giro_posicional {
        giro.iter_mut().for_each(|g| *g = 0.0);
    }
    let mut spin = vec![0.0_f32; n];
    ph2d_contact::impulsos(
        &antes,
        &mut ph2d_contact::Movimento {
            vel,
            giro: &mut giro,
            spin: &mut spin,
        },
        &pecas,
        &dt,
        LEIS,
    );
    (giro, spin)
}

#[cfg(test)]
#[path = "contact_tests.rs"]
mod contact_tests;

/// As SONDAS — irmã dos gates pelo tecto de LOC e por responsabilidade; ver o cabeçalho delas.
#[cfg(test)]
#[path = "contact_probes.rs"]
mod contact_probes;
