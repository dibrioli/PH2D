//! ⭐ **A RESOLUÇÃO DO PREVIEW É DERIVADA DO RELÓGIO** — grosso enquanto se mexe, nítido quando
//! assenta ([ADR-0161], plano W2: *"grosso ao mexer, fino ao parar"*, que ali estava escrito no
//! idioma da malha e pertence, afinal, ao **traçado**).
//!
//! # O defeito, com o número do Enio ao lado
//!
//! Enio, no smoke da cena 6 (22/08): *"lento"*. Ele estava a ver isto (medido nesta workstation,
//! release, máquina calma, área de 1920×1080):
//!
//! | cena | traçado cheio | o que o artista vê |
//! |---|---:|---|
//! | 1 — três cilindros com filete | 46,0 ms | 21 quadros por segundo |
//! | 2 — cubo arredondado | 32,5 ms | 30 fps |
//! | 6 — escultura com furo | **121,0 ms** | **8 fps** |
//!
//! ⚠️ **A janela nunca trava** (o traçado corre noutra thread) — o que fica lento é a **imagem**, e
//! é ela que o artista usa para saber onde a peça está.
//!
//! # ⭐ A lei: o divisor sai da MEDIÇÃO, não de uma constante
//!
//! Um traçado devolve quantos milissegundos custou; dividindo pelos pixels dele sai um custo por
//! pixel **desta máquina, desta peça, deste momento**. O pedido seguinte escolhe o maior tamanho
//! cujo custo previsto cabe no orçamento. É um laço fechado: se a previsão errar, a medição
//! seguinte corrige-a — e é por isso que ele não precisa de saber nada sobre a máquina em que corre.
//!
//! ⚠️ **O erro do modelo é conhecido e é na direção segura.** O custo por pixel **sobe** quando a
//! imagem encolhe (o anti-serrilhado corre sobre as arestas, e a fração de pixels de aresta é maior
//! numa imagem pequena), então prever um traçado grosso a partir de um cheio é **otimista**. O laço
//! apanha isso no quadro seguinte e desce mais um degrau. Convergência medida, com os números
//! reais acima e orçamento de 16,7 ms:
//!
//! | cena | cheio | 1ª escolha | 2ª | assenta em | custo final |
//! |---|---:|---:|---:|---:|---:|
//! | 1 | 46,0 ms | D=2 (17,8) | D=3 | **D=3** | **11,0 ms** |
//! | 6 | 121,0 ms | D=3 (16,6) | D=3 | **D=3** | **16,6 ms** |
//!
//! …ou seja **4,2× e 7,3×** mais depressa enquanto a mão está a mexer, e o número que muda é o que
//! a máquina disse, não um que alguém escolheu.
//!
//! # ⚠️ O piso do divisor é o ORÇAMENTO, e é isso que o mantém honesto
//!
//! A sonda `probe_how_coarse_a_preview_can_be` mediu a deriva da silhueta até D=8 e ela **não
//! existe** (0,15 % no pior caso) — a métrica não contém o fenómeno, e dizer *"o piso é onde a forma
//! muda"* seria dressar um palpite de medição. O piso real é outro e é verificável:
//! [`MAX_PREVIEW_DIVISOR`] `= 3` porque **a D=3 a cena mais pesada já cabe no orçamento** (16,6 ms
//! de 16,7). Descer mais não compra nada que o orçamento peça, e custa nitidez num módulo cuja razão
//! de existir é a aresta. Se um dia uma peça não couber a D=3, o laço fica **preso no piso e a
//! imagem fica lenta em vez de virar papa** — a direção conservadora para este módulo.
//!
//! # ⚠️ O primeiro traçado é sempre CHEIO
//!
//! Sem medição não há previsão, e o primeiro traçado **é** a medição. Isso tem um efeito de produto
//! que não é acidente: a primeira coisa que se vê é a peça **nítida**; a suavização só aparece
//! depois, em movimento, que é onde ela não se nota.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use ph2d_field::FieldDoc;
use ph2d_field_render::Orbit;

/// O que o último traçado custou. ⚠️ **Os dois números juntos** — um tempo sem o tamanho a que foi
/// medido não prevê nada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Measured {
    pub pixels: u64,
    pub millis: f32,
}

/// **O orçamento de um traçado em movimento: um quadro a 60 Hz.**
///
/// ⚠️ O número é do **monitor**, não uma preferência: um traçado que cabe num quadro faz a imagem
/// nunca ficar mais de um quadro atrás da câmera, que é a definição de *acompanhar a mão*. Ele não é
/// um teto de quadro (o traçado corre noutra thread e a janela continua a 60 fps de qualquer forma)
/// — é o alvo da **taxa de atualização da imagem**.
pub(crate) const PREVIEW_BUDGET_MS: f32 = 16.7;

/// **Quão grosso o preview pode ficar.** Ver a nota do módulo: a D=3 a cena mais pesada medida
/// (escultura + booleana, 1920×1080) custa 16,6 ms — já dentro do orçamento.
pub(crate) const MAX_PREVIEW_DIVISOR: u32 = 3;

/// ⭐ **O tamanho a traçar**, dado o tamanho cheio e o que a última medição disse.
///
/// Sem medição devolve o cheio — o primeiro traçado é a medição.
pub(crate) fn preview_size(
    full: (u32, u32),
    measured: Option<Measured>,
    budget_ms: f32,
    min: u32,
) -> (u32, u32) {
    let Some(m) = measured else {
        return full;
    };
    if m.pixels == 0 || !m.millis.is_finite() || m.millis <= 0.0 {
        return full;
    }
    let per_pixel = f64::from(m.millis) / m.pixels as f64;
    let mut chosen = full;
    for d in 1..=MAX_PREVIEW_DIVISOR {
        let size = ((full.0 / d).max(min), (full.1 / d).max(min));
        chosen = size;
        let predicted = per_pixel * f64::from(size.0) * f64::from(size.1);
        if predicted <= f64::from(budget_ms) {
            break;
        }
    }
    chosen
}

/// ⭐ **O que pedir a seguir** — `None` quando não há nada a fazer.
///
/// As três respostas, e cada uma é uma regra:
///
/// | situação | pedido | porquê |
/// |---|---|---|
/// | ainda não há quadro nenhum | **cheio** | o primeiro traçado é a medição |
/// | a câmera ou o documento MUDARAM | **grosso** (o que couber) | é aqui que a mão espera |
/// | nada mudou e o último foi grosso | **cheio** | assentou: refina |
/// | nada mudou e o último já era cheio | `None` | re-traçar seria queimar um núcleo por nada |
///
/// ⚠️ **A área ter mudado de tamanho cai no terceiro caso** e é o comportamento certo: o quadro
/// anterior é esticado enquanto o novo não chega, e o novo sai nítido.
pub(crate) fn next_trace(
    requested: Option<(&Orbit, u32, u32, &FieldDoc)>,
    cam: &Orbit,
    doc: &FieldDoc,
    full: (u32, u32),
    measured: Option<Measured>,
    has_frame: bool,
    min: u32,
) -> Option<(u32, u32)> {
    let Some((rcam, rw, rh, rdoc)) = requested else {
        return Some(full);
    };
    if !has_frame {
        return Some(full);
    }
    if rcam != cam || rdoc != doc {
        return Some(preview_size(full, measured, PREVIEW_BUDGET_MS, min));
    }
    // Assentou. Só há trabalho se o que está na tela ainda não é o tamanho cheio.
    ((rw, rh) != full).then_some(full)
}

/// ⭐ **Vale a pena ABANDONAR o traçado que está em voo?** (W32)
///
/// # A latência que isto fecha, com o número medido
///
/// A W24 deixou-o escrito: *"se a mão recomeça a mexer no meio de um refinamento cheio, a resposta
/// espera por ele — até **121 ms** medidos na cena mais pesada"*. O refinamento é o único traçado
/// que corre **depois** de a cena assentar, e é exactamente o que está no caminho quando a mão volta.
///
/// # ⛔ Cancelar TUDO faria a imagem nunca chegar
///
/// A regra óbvia — *"mudou? abandona o que está a correr"* — tem um modo de falha que a mata: numa
/// órbita contínua a câmera muda **a cada quadro**, e um traçado grosso que leve mais do que um
/// quadro seria cancelado antes de acabar, **sempre**. O artista arrastaria o rato contra uma imagem
/// congelada, e o defeito seria muito pior do que a espera que se queria curar.
///
/// A regra que sobrevive é a que nomeia o caso medido: **um REFINAMENTO cede à mão; um traçado de
/// movimento corre até ao fim.** Um refinamento só começa quando nada está a mudar, então ele nunca
/// está no caminho de si mesmo.
pub(crate) fn cancels_the_inflight(
    inflight: (u32, u32),
    asked: (u32, u32),
    full: (u32, u32),
) -> bool {
    // Em voo está o CHEIO (um refinamento)…
    inflight == full
        // …e o que se pede agora é mais grosso (a mão voltou a mexer).
        && (asked.0 < full.0 || asked.1 < full.1)
}

#[cfg(test)]
#[path = "field3d_preview_tests.rs"]
mod tests;

/// ⭐⭐⭐ **O CONTORNO TAMBÉM ENGROSSA ENQUANTO A MÃO MEXE** (2026-08-26).
///
/// # ⛔ O buraco que ela fecha, com o número
///
/// A lei deste módulo baixava a **resolução da tela** e deixava as **arestas do contorno**
/// intactas. Medido: o traçado custa **`0,22 ms` por aresta** e esse custo é **cego aos pixels** —
/// de `D=3` para `D=6` são 4× menos pixels e o tempo cai `1,3×`. ⇒ subir o `Resolution` custava fps
/// enquanto a mão mexia, que foi o report do Enio.
///
/// | arestas | traçado `D=1` | piso (`D=6`) |
/// |---:|---:|---:|
/// | 168 | 214 ms | 39,3 ms |
/// | 940 | 1 090 ms | 212,5 ms |
///
/// ⭐ ⇒ *grosso a mexer, nítido ao assentar* — **aplicado ao contorno**, que é onde o custo estava.
/// O que o artista pediu em detalhe aparece quando ele **pára**, que é quando ele olha.
///
/// # ⚠️ O tecto é a resolução de OMISSÃO, e não um número novo
///
/// Enquanto a mão mexe, a peça é traçada com o contorno que ela teria **antes** de alguém tocar no
/// `Resolution` — [`ph2d_field::DEFAULT_PROFILE_RESOLUTION`], `168` arestas no círculo medido.
/// *Subir o knob deixa de ter preço em movimento; ele passa a ser inteiramente sobre o que se vê ao
/// parar.*
///
/// ⚠️ **A substituição é SÓ no que vai para o traçador.** O documento que o laço compara
/// ([`next_trace`]) continua a ser o real — trocar o comparado faria a cena parecer mudada a cada
/// alternância entre grosso e fino, e o laço re-traçaria para sempre.
pub(crate) fn coarse_doc(doc: &FieldDoc, asked: (u32, u32), full: (u32, u32)) -> Option<FieldDoc> {
    if asked == full {
        return None;
    }
    let mut mexeu = false;
    let nodes: Vec<ph2d_field::Node> = doc
        .nodes()
        .iter()
        .map(|node| {
            let mut node = node.clone();
            if let ph2d_field::NodeKind::Leaf(
                ph2d_field::Primitive::Extrude { profile, .. }
                | ph2d_field::Primitive::Revolve { profile },
            ) = &mut node.kind
            {
                let thin = ph2d_field::coarsen(profile, PREVIEW_MAX_EDGES);
                if thin.segment_count() < profile.segment_count() {
                    *profile = thin;
                    mexeu = true;
                }
            }
            node
        })
        .collect();
    if !mexeu {
        return None;
    }
    // ⚠️ Uma raiz que o `FieldDoc::new` recuse devolve `None` — a pré-visualização volta ao
    // documento real, que é a resposta segura. *Um preview que não nasce não pode partir a peça.*
    FieldDoc::new(nodes, doc.root()).ok()
}

/// **Quantas arestas o contorno leva enquanto a mão mexe.**
///
/// ⚠️ Não é um número novo: é o que o contorno tem na resolução de **omissão** (medido, `168` no
/// círculo do doc do [`ph2d_field::MAX_PROFILE_RESOLUTION`]) — *o preview é exactamente tão nítido
/// como era antes de o knob existir.*
pub(crate) const PREVIEW_MAX_EDGES: usize = 168;
