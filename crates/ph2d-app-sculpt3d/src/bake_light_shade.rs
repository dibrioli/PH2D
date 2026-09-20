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
        lighting: ph2d_mesh_render::Lighting::Rig,
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
        // ⚠️ **O material só é lido pelo `Lighting::Pbr`** — aqui a lei em prova é a do BARRO, e
        // ele ignora este campo. Ele está escrito por nome e não por `..Default::default()` de
        // propósito: um campo novo no `Shade` tem de ser uma decisão desta sonda, não uma herança.
        material: ph2d_material::OpenPbr::default(),
        // ⚠️ **NEUTRO aqui, e é uma decisão:** o olhar só é lido pelo `Lighting::Pbr`, e a lei em
        // prova nesta vista é a do BARRO, que escreve uma resposta RELATIVA (dividida pela de uma
        // superfície plana sob a mesma luz) — ela não tem unidade de cena para expor. O irmão
        // [`pbr_law_shade`] é que escreve o olhar com que esta casa assa.
        look: ph2d_mesh_render::Look::default(),
        // Uma vista, não uma lei.
        wireframe: false,
    }
}

/// ⭐⭐⭐ **A VISTA em que o visor acende com a LEI QUE ASSA** — o modo `Pbr`, com toda ajuda de
/// edição desligada.
///
/// ⚠️ **As quatro que ficam a zero não são cosmética:** a cavidade, os dois AOs e o espalhamento são
/// leituras de FORMA que o visor derruba por cima da lei, e o sprite recebe a oclusão dele por um
/// canal separado. Deixá-los vivos aqui poria a mesma sombra num lado só, e a diferença leria-se
/// como *«as duas leis divergem»* quando o que divergiu foram as ajudas.
///
/// ⛔ **O `env` fica a zero e é inerte neste modo**, e dizê-lo evita a leitura errada: ele é o knob
/// do ambiente com direcção do modelo de ARGILA. No modo `Pbr` o ambiente entra pela ranhura da lei,
/// com o céu que o rig produz — *são dois caminhos para a mesma palavra, e só um corre aqui*.
pub(super) fn pbr_law_shade() -> Shade {
    Shade {
        lighting: ph2d_mesh_render::Lighting::Pbr,
        // ⭐⭐⭐ **O OLHAR com que esta casa assa, e sem ele esta sonda mede OUTRA COISA.**
        //
        // A [`ph2d_form_pbr::acende_texel`] recebe-o como argumento e devolve já display-referred;
        // o visor sem ele entrega a radiância CRUA. Medido antes desta linha existir, sobre a mesma
        // esfera e com o mesmo albedo: `0,128` de média viva contra `0,539` de assada — `4,3×`, que
        // é exactamente o `2^2,1` do [`OLHAR_DA_FORMA`]. *Uma comparação entre duas implementações
        // da mesma lei que só corre metade de uma delas mede a metade que falta.*
        look: ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA,
        ..shared_law_shade()
    }
}
