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

/// ⭐⭐⭐ A borda que ferve, medida — ver [`borda_sondas`].
#[cfg(test)]
#[path = "borda_sondas.rs"]
mod borda_sondas;
/// ⭐⭐⭐ A borda que ferve, afirmada — ver [`borda_tests`].
#[cfg(test)]
#[path = "borda_tests.rs"]
mod borda_tests;
/// ⭐⭐⭐ **O divisor depois do dispositivo** — ver [`device_tests`].
#[cfg(test)]
#[path = "preview_device_tests.rs"]
mod device_tests;
/// ⭐⭐⭐ O rebordo claro de um pixel, medido — ver [`premultiplicado_sondas`].
#[cfg(test)]
#[path = "premultiplicado_sondas.rs"]
mod premultiplicado_sondas;
/// ⭐⭐⭐ O vaso sem facetas no modo MODEL — ver [`vaso_sem_facetas_tests`].
#[cfg(test)]
#[path = "vaso_sem_facetas_tests.rs"]
mod vaso_sem_facetas_tests;

use ph2d_field::FieldDoc;
use ph2d_field_render::Orbit;

/// O que o último traçado custou. ⚠️ **Os dois números juntos** — um tempo sem o tamanho a que foi
/// medido não prevê nada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measured {
    pub pixels: u64,
    pub millis: f32,
}

/// ⭐⭐⭐⭐ **As DUAS últimas medições de movimento** — e o divisor decide pela mais BARATA (por pixel).
///
/// ⛔⛔ **Um quadro só decidia a resolução do seguinte, e o 1.º quadro de cada gesto é o mais caro**
/// (report do dono, 2026-09-30: *«a resolução ainda cai»*). Medido no nó de toro a girar, a tela
/// cheia: o 1.º quadro depois de parar custa `18`–`19 ms` (a oclusão no tempo enche as células que o
/// quadro parado não tocou) e os seguintes descem a `13`–`16` — o laço via o pico e baixava o
/// quadro SEGUINTE para metade dos píxeis por um custo que já tinha passado. ⇒ um pico ISOLADO não
/// mexe na resolução; dois quadros seguidos acima do orçamento mexem, que é o laço de sempre.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Medicoes {
    pub ultima: Option<Measured>,
    pub antes: Option<Measured>,
}

impl Medicoes {
    /// Guarda uma medição nova; a anterior passa a `antes`.
    pub fn regista(&mut self, m: Measured) {
        self.antes = self.ultima;
        self.ultima = Some(m);
    }

    /// ⭐ **A que o divisor lê** — a de menor custo por pixel das duas.
    #[must_use]
    pub fn para_o_divisor(&self) -> Option<Measured> {
        let por_pixel = |m: &Measured| f64::from(m.millis) / (m.pixels.max(1) as f64);
        match (self.ultima, self.antes) {
            (Some(a), Some(b)) => Some(if por_pixel(&b) < por_pixel(&a) { b } else { a }),
            (a, b) => a.or(b),
        }
    }
}

/// ⭐ As duas medições — ver [`preview_medicoes_tests`].
#[cfg(test)]
#[path = "preview_medicoes_tests.rs"]
mod preview_medicoes_tests;

/// ⭐⭐⭐⭐ **O que o ASSENTAR lembra entre quadros** — o travão ao girar (report do dono de
/// 2026-09-24: *«travamentos ao rotacionar a tela continuam»*).
///
/// Um quadro assente pintado pela PLACA **não se cancela**: quando a mão volta a mexer, o quadro de
/// movimento seguinte espera por ele (o traçador é um só). Com as sondas guardadas
/// (`ph2d_field_gpu::sondas_na_placa`) o degrau do tamanho do movimento custa o que um quadro de
/// movimento custa — e o de tela CHEIA custa o que o movimento custaria nesse tamanho: `~84 ms` no nó
/// de toro da cena `=28`, `15`–`20` nas outras.
#[derive(Clone, Copy, Debug)]
pub struct Assentar {
    /// Desde quando a câmera e o documento não mudam — reposto a cada pedido de MOVIMENTO.
    pub quieto_desde: std::time::Instant,
    /// O custo do último quadro ASSENTE, com os pixels — o que prevê o próximo degrau.
    pub medido: Option<Measured>,
    /// `true` quando esse quadro foi pintado pela placa — o único caminho que não se cancela.
    pub pela_placa: bool,
}

impl Default for Assentar {
    fn default() -> Self {
        Self {
            quieto_desde: std::time::Instant::now(),
            medido: None,
            pela_placa: false,
        }
    }
}

impl Assentar {
    /// ⭐⭐⭐⭐ **O degrau assente de `w×h` pode começar AGORA?**
    ///
    /// ⭐ **A regra é a do aluguer de esquis** (*ski rental*, o problema clássico de decisão em
    /// linha): um trabalho que **não se cancela** e dura `L` só começa depois de a mão estar parada
    /// há `L`. Qualquer pausa mais curta nunca o paga, e o pior caso — a mão volta mesmo depois de
    /// ele começar — espera **no máximo o dobro** do óptimo que alguém que soubesse o futuro
    /// conseguiria. ⇒ *o atraso não é um número escolhido: é o custo MEDIDO do degrau.*
    ///
    /// ⚠️ **Só vale para a placa.** O caminho de CPU cancela-se a meio (`trace_cancellable`), e ali
    /// adiar custaria nitidez sem comprar nada — sem medição, ou com o último assente pela CPU, a
    /// resposta é sempre *«pode»*.
    #[must_use]
    pub fn pode_comecar(&self, w: u32, h: u32) -> bool {
        self.pode_comecar_parado(self.quieto_desde.elapsed().as_secs_f64() * 1000.0, w, h)
    }

    /// O mesmo, com o tempo PARADO dado — a porta pela qual a lei se testa sem relógio.
    #[must_use]
    pub fn pode_comecar_parado(&self, parado_ms: f64, w: u32, h: u32) -> bool {
        if !self.pela_placa {
            return true;
        }
        let Some(m) = self.medido else {
            return true;
        };
        if m.pixels == 0 {
            return true;
        }
        #[allow(clippy::cast_precision_loss)]
        let previsto = f64::from(m.millis) * (f64::from(w) * f64::from(h)) / m.pixels as f64;
        parado_ms >= previsto
    }
}

/// **O orçamento de um traçado em movimento: um quadro a 60 Hz.**
///
/// ⚠️ O número é do **monitor**, não uma preferência: um traçado que cabe num quadro faz a imagem
/// nunca ficar mais de um quadro atrás da câmera, que é a definição de *acompanhar a mão*. Ele não é
/// um teto de quadro (o traçado corre noutra thread e a janela continua a 60 fps de qualquer forma)
/// — é o alvo da **taxa de atualização da imagem**.
pub const PREVIEW_BUDGET_MS: f32 = 16.7;

/// **Quão grosso o preview pode ficar.** Ver a nota do módulo: a D=3 a cena mais pesada medida
/// (escultura + booleana, 1920×1080) custa 16,6 ms — já dentro do orçamento.
pub const MAX_PREVIEW_DIVISOR: u32 = 3;

/// ⭐ **O tamanho a traçar**, dado o tamanho cheio e o que a última medição disse.
///
/// Sem medição devolve o cheio — o primeiro traçado é a medição.
pub fn preview_size(
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
    let cheio = per_pixel * f64::from(full.0) * f64::from(full.1);
    if cheio <= f64::from(budget_ms) {
        return full;
    }
    // ⭐⭐⭐⭐ **A ESCALA É CONTÍNUA** (2026-10-01, report do dono: *«se aproximar do objeto ainda
    // fica lento e perde resolução»*). Eram três degraus (`1`, `½`, `⅓` da largura — `¼` e `1/9`
    // dos píxeis): um quadro a `20 ms` caía para METADE da largura para poupar `17 %`. Hoje a
    // largura desce só o que o orçamento pede, em passos de `1/ESCALA_PASSOS` arredondados para
    // BAIXO (a direcção segura do laço), com o mesmo piso de `1/MAX_PREVIEW_DIVISOR`; e a imagem
    // sobe ao tamanho cheio NA PLACA ([`ph2d_field_gpu::amplia`]), não esticada no ecrã.
    let piso = 1.0 / f64::from(MAX_PREVIEW_DIVISOR);
    let quantiza = |orcamento: f64| {
        (((orcamento / cheio).sqrt() * ESCALA_PASSOS).floor() / ESCALA_PASSOS).clamp(piso, 1.0)
    };
    // ⚠️ **A HISTERESE: subir pede FOLGA, descer não** — sem ela um laço contínuo oscila um passo
    // para cima e outro para baixo em quadros alternados (o custo por pixel sobe quando a imagem
    // encolhe, e a previsão de um tamanho maior a partir de um menor é otimista).
    #[allow(clippy::cast_precision_loss)]
    let agora = (m.pixels as f64 / (f64::from(full.0) * f64::from(full.1))).sqrt();
    let mut escala = quantiza(f64::from(budget_ms));
    if escala > agora + 0.5 / ESCALA_PASSOS {
        escala = quantiza(f64::from(budget_ms) * SUBIR_COM_FOLGA).max(agora.min(escala));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lado = |v: u32| ((f64::from(v) * escala).round() as u32).max(min);
    (lado(full.0), lado(full.1))
}

/// ⭐ Em quantos passos a largura do quadro de movimento desce — ver [`preview_size`]. Um passo é
/// `~3 %` da largura: fino o bastante para não deitar fora píxeis que cabiam, grosso o bastante
/// para o tamanho não mudar a cada quadro por ruído do relógio.
pub const ESCALA_PASSOS: f64 = 32.0;

/// ⭐ A fracção do orçamento que um tamanho MAIOR tem de caber para o laço subir — ver
/// [`preview_size`].
pub const SUBIR_COM_FOLGA: f64 = 0.85;

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
pub fn next_trace(
    requested: Option<(&Orbit, u32, u32, &FieldDoc, bool)>,
    cam: &Orbit,
    doc: &FieldDoc,
    full: (u32, u32),
    measured: Option<Measured>,
    has_frame: bool,
    min: u32,
) -> Option<(u32, u32, bool)> {
    let Some((rcam, rw, rh, rdoc, rcoarse)) = requested else {
        return Some((full.0, full.1, false));
    };
    if !has_frame {
        return Some((full.0, full.1, false));
    }
    if rcam != cam || rdoc != doc {
        let (w, h) = preview_size(full, measured, PREVIEW_BUDGET_MS, min);
        return Some((w, h, true));
    }
    // ⭐⭐⭐ **ASSENTOU — e o primeiro degrau é ALISAR, não aumentar** (W73).
    //
    // ⛔ **O report do Enio:** *«ao parar ficou mais lento para alisar (500 ms)»*. O que ele espera
    // ao parar é a peça **lisa** — e a W72 mudou onde o alisamento vive: ele deixou de estar no
    // quadro de movimento e passou a existir só no assente, que corre no tamanho **cheio** e custa
    // `~500 ms` num contorno denso. *Uma cura pode mudar o sítio de uma espera em vez de a cortar.*
    //
    // ⇒ o assentar passa a ser uma **escada de dois degraus**: primeiro o MESMO tamanho, agora com
    // o contorno inteiro e o anti-serrilhado (`131 ms` a `640×360` com 672 arestas), e só depois o
    // tamanho cheio (`504 ms`). *O que ele espera chega `3,8×` mais cedo, e o degrau caro deixa de
    // estar no caminho dele.*
    //
    // ⚠️ **O preço está nomeado:** o total sobe `~26 %`, porque o degrau do meio é trabalho a mais.
    // É a troca clássica do refinamento progressivo — **latência percebida contra trabalho total**
    // —, e aqui ela é decidida pelo que a mão faz a seguir: se ela voltar a mexer, o degrau caro
    // nunca chega a correr (`cancels_the_inflight`), e o trabalho a mais foi zero.
    if rcoarse {
        return Some((rw, rh, false));
    }
    // Só há mais trabalho se o que está na tela ainda não é o tamanho cheio.
    ((rw, rh) != full).then_some((full.0, full.1, false))
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
///
/// # ⚠️⚠️ A pergunta era feita ao TAMANHO, e desde a W73 o tamanho já não a responde (W89)
///
/// A 1.ª versão perguntava *«o que está em voo é o tamanho CHEIO?»* — e isso **era** a definição de
/// refinamento… até a W73 partir o assentar em dois degraus e pôr o primeiro deles no tamanho
/// **grosso**. A partir daí um refinamento e um traçado de movimento partilhavam o tamanho, o
/// predicado deixava de os distinguir, e o degrau do meio **nunca era abandonado**.
///
/// *Um predicado que identifica uma ESPÉCIE por uma GRANDEZA fica errado no dia em que duas
/// espécies partilham a grandeza* — e o `InFlight` já dizia, em comentário, que sabia a espécie.
///
/// Medido (`measure_the_stall_a_hesitating_hand_pays`, erro angular máximo da imagem contra a mão,
/// arrasto a `90°/s` com uma hesitação no meio):
///
/// | pausa da mão | pelo tamanho | **pela espécie** |
/// |---|---:|---:|
/// | `0 ms` | `1,50°` | `1,50°` |
/// | **`17 ms`** | **`2,97°`** | **`1,50°`** |
/// | `34`–`136 ms` | `1,50°` | `1,50°` |
///
/// ⇒ o defeito aparece na hesitação do tamanho de **um quadro**, que é a que uma mão faz sem dar
/// por ela, e ali ele **dobra** o atraso.
pub fn cancels_the_inflight(inflight_refinement: bool, asked_moving: bool) -> bool {
    inflight_refinement && asked_moving
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "preview_cost_tests.rs"]
mod cost_tests;

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
/// ⭐⭐⭐ **E desde a W85 ela vale TAMBÉM ao parar** — com outro orçamento de erro, não com outra
/// lei. Ver [`MOVING_NORMAL_ERR_DEG`] e [`SETTLED_NORMAL_ERR_DEG`]: o que muda entre os dois quadros
/// é **quanto erro de sombreado se tolera**, e a contagem de arestas sai da forma da peça.
pub fn coarse_doc(doc: &FieldDoc, moving: bool) -> Option<FieldDoc> {
    let erro = if moving {
        MOVING_NORMAL_ERR_DEG
    } else {
        SETTLED_NORMAL_ERR_DEG
    }
    .to_radians();
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
                let thin = ph2d_field::coarsen_to_normal_error(profile, erro);
                // ⛔⛔ **O custo da marcha é o `prim_count`, não o `segment_count`** (2026-09-16).
                // A decimação devolve uma POLILINHA — os arcos da decomposição exacta ficam para trás
                // —, e a comparação pela polilinha trocava `24` arcos por `168`–`392` segmentos no vaso
                // da cena 5 (e `4` por `332` num círculo) assim que o `Resolution` subia: mais caro,
                // e facetado. Um arco já tem a normal exacta; engrossá-lo só vale se ficar MAIS BARATO.
                if thin.prim_count() < profile.prim_count() {
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

/// ⭐⭐⭐ **Quanto erro de NORMAL o quadro de movimento tolera** (W85).
///
/// ⚠️ **Era uma contagem de arestas (`PREVIEW_MAX_EDGES = 168`) e passou a ser um ERRO**, porque a
/// decimação por giro (W84) tornou a contagem uma consequência: o que o orçamento fixa é o erro da
/// normal, que é o que a luz mostra. *Pedir um erro é pedir a coisa que se vê; pedir uma contagem é
/// pedir um número que só a esperança liga ao que se vê.*
///
/// ⭐ **`1,0°` reproduz o que já shipava**: num círculo, `168` arestas dão `1,056°` de erro p99
/// (medido, `measure_how_many_contour_edges_are_visible`) — *a constante nova entra sem mudar o
/// quadro de movimento de nenhuma peça de omissão.*
///
/// ⭐⭐ E ela é **adaptativa à forma de graça**: uma estrela gira muito mais que um círculo, então
/// recebe mais arestas — porque tem mais direcção para gastar.
pub const MOVING_NORMAL_ERR_DEG: f32 = 1.0;

/// ⭐⭐⭐ **Quanto erro de NORMAL o quadro ASSENTE tolera** (W85) — e de que recurso ele é.
///
/// Medido (`measure_how_many_contour_edges_are_visible`, a régua é o **pixel sombreado** em níveis
/// de 8 bits, contra um contorno de `2048`):
///
/// | erro de normal p99 | arestas (círculo) | pixel p99 | pixel máx |
/// |---:|---:|---:|---:|
/// | `0,266°` | `672` | `1` | `1`–`2` |
/// | **`0,529°`** | **`336`** | **`1`** | **`2`–`3`** |
/// | `1,056°` | `168` | `3` | `4` |
///
/// ⭐⭐⭐ **`0,5°` é onde a imagem para de mudar:** de `336` para `672` arestas o pixel muda **um
/// nível em 255**, e o traçado custa o **dobro** (o custo é linear nas arestas). ⇒ *subir o
/// `Resolution` acima disto compra `≤3/255` e paga o preço inteiro.*
///
/// ⛔ **E o tecto NÃO se deriva do tamanho do pixel** — os mesmos números a `640×360` e a
/// `1600×900`. O erro que se vê é **angular**, e um ângulo não encolhe com a resolução da tela.
///
/// ⚠️ **Isto NÃO toca a exportação.** O `Resolution` continua a governar a malha que sai para o
/// arquivo, que é onde ele não é desperdício — ver o gate
/// `the_export_never_goes_through_the_preview_coarsening`.
pub const SETTLED_NORMAL_ERR_DEG: f32 = 0.5;

/// ⭐⭐⭐⭐ **A GRADE DE LONGE: a resolução dela, e a porta que a bisecta** — ver
/// [`ph2d_field_gpu::longe`].
///
/// `PH2D_FIELD_LONGE=off` devolve a marcha de sempre; `=0` só recorta o raio pela caixa da peça;
/// `=<n>` recorta e salta pela grade de `n` células no lado maior. ⚠️ O valor de omissão é o
/// [`LONGE_RES`], e a tabela que o escolhe vive no doc dele.
#[must_use]
pub fn a_grade_de_longe() -> Option<u32> {
    static RES: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *RES.get_or_init(|| match std::env::var("PH2D_FIELD_LONGE").as_deref() {
        Ok("off") => None,
        Ok(v) => v.parse().ok().or(LONGE_RES),
        Err(_) => LONGE_RES,
    })
}

/// ⭐⭐⭐⭐ **O recorte e a grade de longe de omissão: SÓ O RECORTE** — ver [`a_grade_de_longe`].
///
/// ⛔⛔ **A grade de longe foi construída, medida e RECUSADA (2026-09-24).** Ela salta o vazio por um
/// limite inferior provado (`s·f ≥ s·f̃ − (√3/2)·h`, zero violações na CPU **e** na placa, nas oito
/// cenas) e não compra nada: o custo mora PERTO da superfície (os passos finais, a normal, as bordas
/// re-amostradas), e o vazio já o corta o RECORTE sozinho. Duas corridas a `1920×1080`, matcap, a
/// `92 %` e `70 %` de CPU ociosa, a placa partilhada com outra janela do app (logo `±10 %`):
///
/// | cena | árvore ms | só recorte | recorte + grade `64` |
/// |---|---|---|---|
/// | `=5`  | `16,4` / `16,0` | `1,31×` / `1,12×` | `1,05×` / `1,07×` |
/// | `=28` | `103,6` / `103,1` | `1,19×` / `1,16×` | `1,19×` / `1,18×` |
/// | `=1`  | `10,4` / `9,9` | `1,15×` / `0,95×` | `0,99×` / `0,94×` |
/// | `=11` | `18,9` / `17,9` | `1,21×` / `1,17×` | `1,07×` / `1,09×` |
/// | `=26` | `5,5` / `4,9` | `1,26×` / `1,10×` | `1,18×` / `1,05×` |
/// | `=27` | `14,8` / `15,0` | `1,19×` / `1,13×` | `1,21×` / `1,21×` |
/// | `=29` | `10,0` / `10,2` | `1,10×` / `1,11×` | `1,03×` / `1,08×` |
/// | `=30` | `15,5` / `15,2` | `1,04×` / `1,09×` | `1,04×` / `1,02×` |
///
/// ⭐ **E o recorte é também a PARIDADE:** é o que a CPU faz, e com ele a silhueta do dispositivo
/// discorda da dela em `0` pixels nas oito cenas (sem ele, `8` na `=29`). ⚠️ A grade continua
/// alcançável por `PH2D_FIELD_LONGE=<n>`, e as sondas da recusa vivem em
/// `device_probes_w9_longe.rs`.
pub const LONGE_RES: Option<u32> = Some(0);

/// ⭐⭐⭐⭐ **Abaixo de quantas células da superfície o raio deixa a grade** — ver
/// [`ph2d_field_gpu::longe::Longe::perto`].
pub const LONGE_PERTO: f32 = 1.0;

#[must_use]
pub fn re_amostra_a_silhueta() -> bool {
    static LIGADO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIGADO.get_or_init(|| std::env::var("PH2D_FIELD_BORDA").as_deref() != Ok("0"))
}

