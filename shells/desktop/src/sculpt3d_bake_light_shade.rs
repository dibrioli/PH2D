//! **A PREMISSA DA SONDA — a vista em que as duas luzes descrevem a MESMA LEI.**
//!
//! Irmão (`#[path]`) do [`super`], e o corte é de ASSUNTO: lá mora *o que a sonda
//! MEDE* (as duas luzes, os baldes de distância, os gates); aqui *com o que ela é
//! MONTADA*. As duas metades já se separaram sozinhas no dia em que a premissa
//! quebrou — o `DEFAULT_MATCAP` mudou e a medição passou a comparar dois modelos
//! —, e um arquivo só as deixava confundir-se outra vez.
//!
//! ⚠️ **E o corte foi FORÇADO pelo teto de LOC do shell (HR-18, 600):** o pai
//! chegou a **631**. A cura de um teto é um corte para o IRMÃO, nunca uma
//! allowlist — extrair dentro do mesmo arquivo cura um número e estoura o outro.

use ph2d_mesh_render::{Shade, SssParams};

/// **A VISTA EM QUE AS DUAS LUZES DESCREVEM A MESMA LEI** — a premissa desta
/// sonda, escrita por NOME em vez de herdada.
///
/// Esta comparação só significa alguma coisa entre **duas implementações da
/// mesma lei**: o barro aceso pelo rig do documento contra o passe de tinta
/// aceso pelo mesmo rig. Todo termo que **só o barro tem** é ruído aqui, e cada
/// um está zerado abaixo com o motivo ao lado.
///
/// ⚠️ **ELA VINHA DE `Shade::default()`, E ISSO CUSTOU DEZ DIAS DE GATE
/// VERMELHO NO `main`.** O doc que vivia no sítio da chamada já dizia *"o matcap
/// é outra luz inteira — ligá-lo aqui faria a comparação medir a diferença entre
/// dois MODELOS"*; em 2026-08-09 o `DEFAULT_MATCAP` passou a `Some(0)` por
/// **decisão de produto** (o barro abre aceso pela luz do OLHO, como no
/// SculptGL), e a premissa quebrou **em silêncio**. O gate passou a medir
/// *matcap contra rig* e a acusar 0,3370 sobre um produto correto.
///
/// ⚠️ **É a lei do `CLAUDE.md` §0.0 na direção inversa:** quem move o número que
/// sustentava uma nota tem de reconferir a nota — e um default COMPARTILHADO é
/// exatamente o número que ninguém sabe que sustenta alguma coisa.
///
/// ⚠️ **Os SETE campos estão escritos, e nenhum vem de `..Default::default()`.**
/// Não é verbosidade: com o literal completo, **um campo NOVO na [`Shade`] é um
/// erro de COMPILAÇÃO aqui** — quem o acrescentar é obrigado a dizer se ele é da
/// lei partilhada ou só do barro. Com `..Default::default()` o próximo termo
/// entraria mudo, que é precisamente como este entrou.
pub(super) fn shared_law_shade() -> Shade {
    Shade {
        // ⭐ **O rig do documento, e não a luz do olho.** É o único modelo que a
        // tinta também implementa — comparar contra um matcap é comparar duas
        // leis diferentes e chamar à diferença um defeito.
        matcap: None,
        // A cavidade é leitura de FORMA e só o barro a tem no caminho desta
        // sonda: a tinta a recebe assada, por outro canal.
        cavity: 0.0,
        // O AO assado entra na tinta pelo `form_occlusion`, que é um plano
        // separado — deixá-lo vivo aqui poria a mesma sombra num lado só.
        ao: 0.0,
        // O AO de TELA é medido por-vista e não existe no passe de tinta.
        ssao: 0.0,
        // O espalhamento é um MATERIAL do barro; a tinta não o modela.
        sss: SssParams {
            strength: 0.0,
            ..SssParams::default()
        },
        // ⚠️ O ambiente com DIREÇÃO ainda não chegou à tinta — o
        // `DEFAULT_ENV = 0.0` diz isso, e o doc dele nomeia a adoção como
        // follow-up. Zerar aqui é honrar essa fronteira em vez de a atravessar.
        env: 0.0,
        // Uma vista, não uma lei.
        wireframe: false,
    }
}
