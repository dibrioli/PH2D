//! ⭐⭐ **AS OPERAÇÕES SOBRE A TABELA INTEIRA de estados** — irmão de [`super`] pelo teto de 600
//! LOC (HR-18), e o corte é por assunto: ali *o que UMA pose é* (capturar, instalar, o verbo de um
//! clique); aqui *o que acontece à tabela toda quando o mundo muda por baixo dela*.
//!
//! ⚠️ **As três respondem à mesma pergunta em tempos diferentes:** o objecto **moveu-se**
//! ([`shift_host_in_all_states`]), o objecto **saiu** ([`forget_object_in_all_states`]), ou a forma
//! que uma pose nomeia **deixou de existir como estado**
//! ([`replace_morph_shape_in_all_states`]). ⛔ Espalhadas pelo arquivo grande, as três eram lidas
//! como detalhes de outras funções — e a do meio chegou a ficar **dentro do doc-comment** da
//! vizinha (achado ao cortar).

use ph2d_ui_state::{StateRole, StateSets};
use ph2d_vec_scene::VecPathId;

/// ⛔⛔ **UMA FORMA QUE SAI DA SUB-ÁRVORE LEVA AS POSES DELA** (plano 32 W11d).
///
/// Um estado grava a **sub-árvore** ([`members`]), com a pose **LOCAL** de cada filho. Quando o
/// artista tira um filho de lá — o ⊘ *Desconectar* de um conjunto de Morph States é o gesto que o
/// faz num clique —, a pose antiga fica na tabela e o `install` do próximo Show **reescreve-lhe o
/// `Transform`**: a forma solta **salta para a origem do hospedeiro**, no meio de uma animação que
/// já não é sobre ela.
///
/// ⚠️ É o mesmo argumento do `retain_hosts` da [`crate::render_loop::ui_state_bridge`], um nível
/// abaixo — ali *"uma forma apagada leva os estados dela"*, aqui *"uma forma que sai leva as poses
/// dela"*.
///
/// ⛔ **O Dissolve não passa por aqui**, e não precisa: ele apaga o path do conjunto, e o
/// `retain_hosts` deixa cair a tabela inteira do hospedeiro no mesmo quadro.
///
/// Devolve `true` se alguma pose saiu.
pub(crate) fn forget_object_in_all_states(
    states: &mut StateSets,
    host: VecPathId,
    id: VecPathId,
) -> bool {
    let mut dropped = false;
    for role in StateRole::ALL {
        let Some(mut st) = states.role(host, role).cloned() else {
            continue;
        };
        let before = st.objects.len();
        st.objects.retain(|p| p.id != id);
        // ⚠️ **A escrita é POR ESTADO e condicional**, a mesma lei da irmã abaixo: re-escrever um
        // estado que não continha a forma é inócuo hoje e é a forma exacta de um defeito no dia em
        // que o `set` ganhar um efeito colateral.
        if st.objects.len() != before {
            states.set(host, st);
            dropped = true;
        }
    }
    dropped
}

/// ⭐⭐⭐ **UMA FORMA QUE SAI É SUBSTITUÍDA POR OUTRA DO CONJUNTO** (plano 32 W11h).
///
/// Enio, 2026-08-26, 4.º report: *"com um morph states com 3 shapes dentro de uma animação de
/// States, desconectei uma shape do morph state e quebrou a animação do state. (…) se o usuário
/// desconectar uma shape, coloque outra shape do conjunto em seu lugar de modo a não quebrar as
/// anims."*
///
/// # ⛔⛔ O que quebrava, medido
///
/// A pose do hospedeiro guarda **qual forma o conjunto mostra** (`morph_shape`). Ao tirar essa
/// forma do conjunto, a pose continuava a nomeá-la: medido, com `Default = 0` e `Hover = 1`, tirar
/// a `0` deixava a tabela em `Default = Some(0)` com os membros já em `[1, 2]`, e o
/// `Transition::morph_steps` publicava **`from: 0`** — o motor a cozer a partir de uma forma que
/// saiu do conjunto e cujo `Transform` já é de MUNDO, não do referencial dele.
///
/// # A escolha, e por que não é «a primeira que sobrar»
///
/// ⭐ **Prefere-se uma sobrevivente que NENHUM outro estado nomeie.** Com `Default = wide` e
/// `Hover = tall`, tirar o `wide` e pôr `tall` no lugar deixaria os dois estados na MESMA forma —
/// a animação sobreviveria ao ficheiro e **morreria na tela**, que é o defeito com outro nome. Com
/// três formas há sempre uma livre, e é ela que entra.
///
/// ⚠️ **Uma substituição por FORMA, não por estado:** dois estados que nomeavam a mesma forma
/// continuam a nomear a mesma. Escolher por estado partiria uma igualdade que o artista autorou.
///
/// Devolve a forma escolhida, ou `None` se nada nomeava a que saiu (ou se não sobrou candidata).
pub(crate) fn replace_morph_shape_in_all_states(
    states: &mut StateSets,
    host: VecPathId,
    gone: VecPathId,
    candidates: &[VecPathId],
) -> Option<VecPathId> {
    // ⚠️ **A leitura vem toda ANTES da escrita**: o `set` reescreve o papel inteiro, e decidir a
    // meio faria a preferência ler uma tabela já meio mudada.
    let mut named: Vec<VecPathId> = Vec::new();
    let mut touches = false;
    for role in StateRole::ALL {
        let Some(st) = states.role(host, role) else {
            continue;
        };
        for p in &st.objects {
            match p.morph_shape {
                Some(s) if s == gone => touches = true,
                Some(s) => named.push(s),
                None => {}
            }
        }
    }
    if !touches {
        return None;
    }
    let to = candidates
        .iter()
        .find(|c| !named.contains(c))
        .or_else(|| candidates.first())
        .copied()?;
    for role in StateRole::ALL {
        let Some(mut st) = states.role(host, role).cloned() else {
            continue;
        };
        let mut here = false;
        for p in &mut st.objects {
            if p.morph_shape == Some(gone) {
                p.morph_shape = Some(to);
                here = true;
            }
        }
        if here {
            states.set(host, st);
        }
    }
    Some(to)
}

/// **Move o widget inteiro, carregando TODOS os estados** (Enio, 2026-08-07).
///
/// Desloca a pose do **HOSPEDEIRO** por `delta` em cada estado gravado dele. Devolve `true` se
/// alguma pose se moveu.
///
/// # ⚠️ Só o HOSPEDEIRO, e é isso que torna a operação correta
///
/// As poses dos filhos são **LOCAIS ao hospedeiro** ([`capture`]), então mover o `Transform` dele
/// já os leva junto na tela. Deslocá-los também moveria tudo **duas vezes** — e destruiria
/// exactamente o que o artista quer preservar: *a coreografia interna do widget*.
///
/// # Por que ela precisa de existir
///
/// Um estado grava a sub-árvore, e o hospedeiro está nela sempre que ele próprio é uma forma
/// desenhada. Então a translação ABSOLUTA dele fica congelada em cada estado, e relocar o widget
/// deixa de funcionar: mostrar um estado **devolve a forma ao lugar antigo**. ⚠️ Um hospedeiro que
/// seja um GRUPO puro nunca teve o problema (o `members` não o inclui — ele não tem forma), e é
/// por isso que o defeito só aparece depois de o artista gravar um estado que move a própria
/// forma-hospedeiro.
pub(crate) fn shift_host_in_all_states(
    states: &mut StateSets,
    host: VecPathId,
    delta: [f64; 2],
) -> bool {
    if delta == [0.0, 0.0] {
        return false;
    }
    let mut moved = false;
    for role in StateRole::ALL {
        let Some(mut st) = states.role(host, role).cloned() else {
            continue;
        };
        // ⚠️ **A flag é POR ESTADO**, e não do laço: com uma flag acumulada, o primeiro estado
        // que se move faz TODOS os seguintes serem re-escritos, inclusive os que não contêm o
        // hospedeiro. É inócuo hoje (re-escrever o mesmo valor), e é a forma exacta de um defeito
        // que só aparece no dia em que o `set` ganhar um efeito colateral.
        let mut here = false;
        for pose in &mut st.objects {
            if pose.id == host {
                pose.translation[0] += delta[0];
                pose.translation[1] += delta[1];
                here = true;
            }
        }
        if here {
            states.set(host, st);
            moved = true;
        }
    }
    moved
}
