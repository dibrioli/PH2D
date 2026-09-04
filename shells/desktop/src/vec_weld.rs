//! ⭐⭐⭐ **SOLDAR na shell** (plano 39) — a costura entre a selecção e a lei
//! ([`ph2d_vec_scene::weld`]).
//!
//! Segue a convenção das outras operações destrutivas do módulo (`apply_vec_boolean`): os
//! operandos podem ter poses diferentes, então cada um é **assado no MUNDO** e o que nasce é
//! world-space na identidade — a rede aparece exactamente onde os traços estavam.
//!
//! ⚠️ **Decisão do Enio (2026-08-31): soldar CONSOME os traços originais.** Não há vínculo vivo,
//! não há original escondido: o que sobra é a rede, e o caminho de volta é o desfazer.

use ph2d_vec_scene::{Contour, VecPath, VecScene, VecVertex, VecXforms, trim_tool, weld};

/// Um contorno como o motor de corte o vê: os vértices e se ele fecha.
type Arco = (Vec<VecVertex>, bool);

/// Os contornos de um caminho, já no MUNDO.
fn contornos_mundo(scene: &VecScene, xforms: &VecXforms, id: u64) -> Vec<Contour> {
    let Some(p) = scene.path(id) else {
        return Vec::new();
    };
    let mut cozido = p.cooked().into_owned();
    ph2d_vec_scene::bake_xform(&mut cozido, &ph2d_vec_scene::xform_of(xforms, id));
    trim_tool::contours_of(&cozido).collect()
}

/// **SOLDA a selecção**: cada contorno parte-se em arcos nos pontos onde encontra os outros, e as
/// pontas vizinhas caem no mesmo sítio.
///
/// ⚠️ **Um caminho que não encontra ninguém NÃO é tocado** — nem o objecto, nem o id, nem os
/// subpaths. É o que faz soldar uma selecção grande não dissolver os compostos que estavam só a
/// passar por lá.
///
/// ⛔ **Um caminho que É cortado dissolve-se por inteiro**, e um composto perde o buraco: depois da
/// solda os contornos são arcos de uma rede, e um arco não tem dentro. É o preço declarado de
/// *"consome os originais"*.
pub(crate) fn apply_vec_weld(
    scene: &mut VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &mut ph2d_vec_edit::PenTool,
    xforms: &VecXforms,
    ligacao: f64,
) {
    let sel: Vec<u64> = pen.selected_paths().to_vec();
    if sel.is_empty() {
        eprintln!("[ph2d-vec] soldar: selecione os tracos a soldar (Shift+clique)");
        return;
    }
    // Todos os contornos de todos os seleccionados, no mundo — o universo que se cruza.
    let por_caminho: Vec<(u64, Vec<Contour>)> = sel
        .iter()
        .map(|&id| (id, contornos_mundo(scene, xforms, id)))
        .collect();
    let escala = escala_da_selecao(&por_caminho);

    // ── FASE 1: CORTAR. Por caminho, os arcos de cada contorno dele.
    let mut cortados: Vec<(u64, Vec<Arco>)> = Vec::new();
    for (id, meus) in &por_caminho {
        let mut arcos = Vec::new();
        let mut mexeu = false;
        for (k, c) in meus.iter().enumerate() {
            // Os OUTROS: todo contorno que não seja este. ⚠️ Os do MESMO caminho entram — dois
            // subpaths que se cruzam são uma rede tanto quanto dois caminhos.
            let mut outros: Vec<Arco> = Vec::new();
            for (oid, cs) in &por_caminho {
                for (ok, o) in cs.iter().enumerate() {
                    if oid == id && ok == k {
                        continue; // ele próprio: o auto-cruzamento já sai do `crossings_against`
                    }
                    outros.push((o.verts.clone(), o.closed));
                }
            }
            let xings = trim_tool::crossings_against(&c.verts, c.closed, &outros, escala);
            let pedacos = weld::split_at(&c.verts, c.closed, &xings);
            mexeu |= pedacos.len() > 1 || (c.closed && pedacos.first().is_some_and(|a| !a.1));
            arcos.extend(pedacos);
        }
        if mexeu {
            cortados.push((*id, arcos));
        }
    }

    // ── A TOLERÂNCIA, que tem DOIS pisos e uma razão para cada um.
    //
    // ⚠️⚠️ **DUAS vezes a flecha, e não uma.** As duas pontas de um cruzamento vêm de contornos
    // diferentes, e **cada um erra a SUA flecha, em direcções opostas** — a separação de pior caso
    // é a SOMA. Medido em dois círculos de raio 100: as pontas ficaram a `0,1376` com uma flecha de
    // `0,12`, e com a folga de uma flecha só a solda **não pegava**.
    //
    // ⭐⭐ **E a `ligacao`, que é o ímã que o artista já sente.** Ela vem do chamador e é a MESMA
    // régua do encaixe (`SNAP_PX` convertido pelo zoom): *o app já tem uma resposta para «estas
    // duas coisas estão no mesmo sítio», e soldar reusa-a em vez de inventar uma segunda.*
    let flecha = 2.0
        * por_caminho
            .iter()
            .flat_map(|(_, cs)| cs.iter())
            .map(|c| trim_tool::sampling_error(&c.verts, c.closed))
            .fold(0.0_f64, f64::max);
    let tol = flecha.max(ligacao.max(0.0));

    // ── FASE 2: AS PONTAS. ⭐⭐⭐ Cortar não é soldar, e **cruzar não é a única forma de se
    // encontrar**: duas curvas que acabam no mesmo sítio partilham um nó tanto quanto duas que se
    // atravessam. Report do Enio (2026-09-01): *"ainda não consegue conectar as duas curvas … as
    // linhas não compartilham o mesmo nó"* — e medido, o comando recusava-se (*"nada se cruza"*)
    // sobre duas curvas ponta-com-ponta a `0,36` de distância.
    //
    // ⚠️ As pontas vêm de DOIS substratos, e é por isso que quem as agrupa é uma porta só
    // (`weld::cluster_endpoints`): as de um arco recém-cortado vivem num vector; as de um caminho
    // que ninguém cortou vivem na cena, com pose própria e um id a preservar.
    let mut todos: Vec<Arco> = Vec::new();
    let mut donos: Vec<(u64, usize)> = Vec::new(); // (caminho, quantos arcos dele)
    for (id, arcos) in &cortados {
        donos.push((*id, arcos.len()));
        todos.extend(arcos.iter().cloned());
    }
    let mut slots: Vec<Slot> = Vec::new();
    let mut pontos: Vec<[f64; 2]> = Vec::new();
    for (i, (verts, closed)) in todos.iter().enumerate() {
        if *closed || verts.len() < 2 {
            continue;
        }
        for v in [0, verts.len() - 1] {
            slots.push(Slot::Arco(i, v));
            pontos.push(verts[v].anchor);
        }
    }
    for (id, _) in &por_caminho {
        if cortados.iter().any(|(c, _)| c == id) {
            continue; // já entrou como arcos
        }
        let Some(p) = scene.path(*id).filter(|p| editavel_no_sitio(p)) else {
            continue;
        };
        let x = ph2d_vec_scene::xform_of(xforms, *id);
        // ⚠️ **TODO contorno aberto empresta as suas duas pontas**, em índice PLANO. Antes eram
        // `[0, verts.len() - 1]`, que numa rede já soldada — hoje um caminho COMPOSTO — nomeava só
        // o primeiro arco: soldar uma linha nova à ponta de uma rede existente não pegava.
        for v in pontas_planas(p) {
            let Some(vt) = p.vert(v) else { continue };
            slots.push(Slot::Caminho(*id, v));
            pontos.push(x.apply(vt.anchor));
        }
    }
    let (de_quem, nos) = weld::cluster_endpoints(&pontos, tol);
    let ligou = de_quem
        .iter()
        .zip(&slots)
        .any(|(n, s)| n.is_some() && matches!(s, Slot::Caminho(..)));
    if cortados.is_empty() && !ligou {
        eprintln!("[ph2d-vec] soldar: nada se cruza nem se encontra na selecao — nada a soldar");
        return;
    }

    let pre = scene.clone();
    // As pontas dos ARCOS mudam-se no mundo; as dos caminhos INTACTOS descem à pose deles, que é o
    // que lhes preserva o id, o estilo e a pilha de efeitos.
    for (k, &n) in de_quem.iter().enumerate() {
        let Some(n) = n else { continue };
        match slots[k] {
            Slot::Arco(a, v) => weld::mover_ponta(&mut todos[a].0[v], nos[n]),
            Slot::Caminho(id, v) => {
                let Some(inv) = ph2d_vec_scene::xform_of(xforms, id).inverse() else {
                    continue;
                };
                let alvo = inv.apply(nos[n]);
                if let Some(p) = scene.path_mut(id)
                    && let Some(vt) = p.vert_mut(v)
                {
                    weld::mover_ponta(vt, alvo);
                }
            }
        }
    }
    let mut it = todos.into_iter();
    let cortados: Vec<(u64, Vec<Arco>)> = donos
        .into_iter()
        .map(|(id, n)| (id, it.by_ref().take(n).collect()))
        .collect();
    let ids_cortados: Vec<u64> = cortados.iter().map(|(id, _)| *id).collect();
    let n_arcos: usize = cortados.iter().map(|(_, a)| a.len()).sum();

    // ── FASE 3: ⭐⭐⭐ **UM OBJECTO**, e não um por arco.
    //
    // Report do Enio (2026-09-02): *"o weld cria uma grande quantidade de path na hierarquia
    // quando na verdade deveria criar apenas 1. No canvas também deve ser transformado como se
    // fosse um único objeto e o seu gizmo deve ser apenas 1"*.
    //
    // ⚠️ **Uma rede é UMA coisa.** Enquanto cada arco era um `VecPath`, cada arco era uma entidade
    // ECS (ADR-0110) ⇒ uma linha na Hierarquia, uma pose própria e um gizmo próprio — e mover um
    // deles **rasgava** a rede, que é precisamente o que soldar promete que não acontece. O
    // `VecPath` já sabe carregar vários contornos (`subpaths`), e o Trim já emite contornos
    // ABERTOS por essa porta (`trim_tool::sever`): a rede é um **caminho composto** cujos
    // contornos são os arcos.
    //
    // ⚠️⚠️ **E escreve-se NO LUGAR do participante mais ao fundo**, como o Trim faz — *o id, a
    // ordem em z e a entidade ECS que o representa têm de sobreviver à operação*. Um `insert_path`
    // daria um objecto novo, com nome de fábrica, e o artista perderia o que tinha baptizado.
    //
    // ⛔ **O preço, declarado:** a rede tem **um** estilo (o do anfitrião) e **uma** pose. Os arcos
    // que vieram de outros traços perdem cor, largura e pose próprias — é o que "um objecto"
    // significa, e é a mesma conta que o `make_compound` já cobra.
    let anfitriao = cortados
        .iter()
        .filter_map(|(id, _)| {
            scene
                .paths()
                .iter()
                .position(|p| p.id == *id)
                .map(|z| (z, *id))
        })
        .min();
    let mut rede: Option<u64> = None;
    if let Some((_, host)) = anfitriao {
        // ⚠️ **Os arcos falam MUNDO e o documento guarda LOCAL** (a regra-mãe do módulo): eles
        // descem ao espaço do anfitrião pelo inverso da pose DELE. Com a pose na identidade — o
        // caso comum — o inverso é a identidade e nada se mexe.
        let inv = ph2d_vec_scene::xform_of(xforms, host).inverse();
        let mut contornos: Vec<Contour> = cortados
            .into_iter()
            .flat_map(|(_, a)| a)
            .map(|(verts, closed)| Contour {
                verts: match inv.as_ref() {
                    Some(i) => para_local(verts, i),
                    None => verts,
                },
                closed,
            })
            .filter(|c| c.verts.len() >= 2)
            .collect();
        // ⚠️ **Escreve NO LUGAR** (`path_mut`), e não apaga-e-insere: é o id, a fatia de z e a
        // entidade ECS do anfitrião que sobrevivem — a mesma lei que o Trim já cumpre.
        // ⚠️ A pilha de efeitos SAI: ela descreve um caminho que deixou de existir, e os arcos já
        // saíram da geometria COZIDA por ela — mantê-la aplicá-la-ia uma segunda vez.
        // ⛔ **Sem arco nenhum a rede não se escreve — e mesmo assim NÃO se sai por aqui**: a fase 2
        // já mudou pontas na cena, e uma saída antecipada deixaria essa mudança sem Ctrl+Z.
        if !contornos.is_empty()
            && let Some(slot) = scene.path_mut(host)
        {
            let primeiro = contornos.remove(0);
            slot.verts = primeiro.verts;
            slot.closed = primeiro.closed;
            slot.subpaths = contornos;
            slot.effects = Vec::new();
            // ⭐⭐⭐ **A TAMPA DA REDE É REDONDA, e é geometria, não gosto.**
            //
            // Report do Enio (2026-09-02, com foto): *"com stroke muito largo, o stroke se quebra
            // na peça com weld"*. ⚠️ **Estrutural, e prova-se sem medir**: o traço é aplicado ao
            // caminho inteiro, e a kurbo põe **TAMPA** — nunca junta — na ponta de cada
            // sub-caminho. Cada arco da rede é um sub-caminho, então **todo nó é um par de
            // tampas**, e com tampa recta sobra uma cunha vazia de profundidade `(w/2)·tan(θ/2)` do
            // lado de fora da curva: invisível num traço fino, um rasgo num traço largo.
            //
            // ⭐ Uma tampa REDONDA é o meio-disco de raio `w/2` centrado na ponta. No nó partilhado
            // a união dos meios-discos dos arcos que ali chegam **cobre o disco inteiro** — logo
            // cobre a cunha para **qualquer** ângulo, exactamente. ⛔ Não é uma aproximação que
            // melhora com o zoom: é a mesma tinta que uma junta redonda poria.
            //
            // ⚠️ **O preço, declarado:** as pontas SOLTAS da rede também ficam redondas — um
            // `StrokeSpec` tem **uma** tampa para o caminho todo, e não há como dizer *"junta no nó,
            // a do artista na ponta livre"* sem um segundo caminho.
            if let Some(st) = slot.stroke.as_mut() {
                st.cap = ph2d_vec_scene::LineCap::Round;
            }
            rede = Some(host);
        }
        if rede.is_some() {
            // Os OUTROS participantes são consumidos — o preço declarado de "soldar consome os
            // traços". ⚠️ Só depois de a rede existir: apagá-los primeiro perderia a geometria se a
            // escrita não acontecesse.
            for id in ids_cortados {
                if id != host {
                    scene.remove_path(id);
                }
            }
        }
    }
    history.push_undo(pre);
    // ⚠️ **A selecção final é a rede**: o objecto novo MAIS os traços que sobreviveram inteiros.
    // Limpá-la para a rede só faria o artista perder de vista as curvas que ele acabou de ligar.
    let fica: Vec<u64> = sel
        .iter()
        .copied()
        .filter(|id| scene.path(*id).is_some())
        .collect();
    pen.select_many(&fica);
    eprintln!(
        "[ph2d-vec] soldar: ok ({} rede[s] de {n_arcos} arco[s], {} no[s], folga {tol:.4})",
        usize::from(rede.is_some()),
        nos.len()
    );
}

/// Um contorno em MUNDO desce ao espaço LOCAL de quem o vai guardar.
///
/// ⚠️ Passa pelo `bake_xform`, que é a MESMA porta que subiu a geometria ao mundo na fase 1 — as
/// alças de um vértice transformam-se com a âncora, e uma conta escrita à mão aqui deixaria as
/// tangentes por trás numa pose com rotação.
fn para_local(verts: Vec<VecVertex>, inv: &ph2d_vec_scene::Xform) -> Vec<VecVertex> {
    let mut p = VecPath {
        verts,
        ..VecPath::default()
    };
    ph2d_vec_scene::bake_xform(&mut p, inv);
    p.verts
}

/// De onde vem cada ponta que entra no agrupamento.
#[derive(Clone, Copy, Debug)]
enum Slot {
    /// Um arco recém-cortado: `(índice em `todos`, índice do vértice)`.
    Arco(usize, usize),
    /// Um caminho que ninguém cortou e que fica com o id: `(caminho, índice do vértice)`.
    Caminho(u64, usize),
}

/// **Este caminho pode receber a ponta no sítio, sem se dissolver?**
///
/// ⚠️ A pergunta não é de gosto: um caminho com **efeitos** tem a geometria que se vê COZIDA, e os
/// vértices autorados já não são as pontas que o cruzamento mediu — mover o primeiro seria mover
/// outra coisa. Fora disso, mover a primeira ou a última âncora de um contorno ABERTO é exactamente
/// mover a ponta, e o objecto mantém id, estilo e pose.
///
/// ⚠️ **Um COMPOSTO passou a entrar** (2026-09-02): a rede que a solda produz É um composto, e sem
/// isto soldar uma linha nova à ponta de uma rede existente não pegava — a rede era recusada à
/// porta. Quem enumera as pontas dela é a [`pontas_planas`].
fn editavel_no_sitio(p: &VecPath) -> bool {
    p.effects.is_empty()
}

/// **As pontas de um caminho, em índice PLANO** — uma por extremo de cada contorno ABERTO.
///
/// ⚠️ Espelha a lei do [`ph2d_vec_edit::PenTool::welded_with`] (que a usa para decidir *quem anda
/// junto com isto*). São duas crates, e por isso duas funções; ⛔ mas é **uma lei**, e o gate
/// `the_mark_and_the_drag_answer_the_same_question` continua a cosê-las pelo comportamento.
fn pontas_planas(p: &VecPath) -> Vec<usize> {
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((verts, closed)) = p.contour(c) else {
            continue;
        };
        if closed || verts.len() < 2 {
            continue;
        }
        let n = verts.len();
        out.extend(
            [p.flat_vert(c, 0), p.flat_vert(c, n - 1)]
                .into_iter()
                .flatten(),
        );
    }
    out
}

/// A diagonal da caixa de tudo o que entra — a régua com que duas travessias quase-coincidentes
/// são a mesma. ⛔ Um número fixo trataria uma peça de 5 unidades e uma de 5 000 igual.
fn escala_da_selecao(por_caminho: &[(u64, Vec<Contour>)]) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for (_, cs) in por_caminho {
        for c in cs {
            for v in &c.verts {
                lo = [lo[0].min(v.anchor[0]), lo[1].min(v.anchor[1])];
                hi = [hi[0].max(v.anchor[0]), hi[1].max(v.anchor[1])];
            }
        }
    }
    if !lo[0].is_finite() {
        return 1.0;
    }
    ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2))
        .sqrt()
        .max(1e-6)
}

#[cfg(test)]
#[path = "vec_weld_tests.rs"]
mod tests;

/// Os gates do que se VÊ — irmão pelo tecto de LOC (HR-18), cortado por responsabilidade.
#[cfg(test)]
#[path = "vec_weld_look_tests.rs"]
mod look_tests;
