//! ⭐⭐⭐ **O BALDE na shell** (plano 40) — a costura entre o ponteiro e a lei ([`ph2d_vec_fill`]).
//!
//! # As três coisas que esta camada decide
//!
//! 1. **QUE contornos são PAREDE.** Os visíveis do documento, cozidos e assados no MUNDO — **menos
//!    os preenchimentos** ([`ph2d_ecs::VecBucketFill`]). ⛔ Um preenchimento tem por fronteira os
//!    mesmos arcos que as linhas; deixá-lo voltar à rede punha lá arestas **coincidentes**, com
//!    direcção de saída idêntica, e o passeio de faces passava a escolher entre duas meias-arestas
//!    indistinguíveis: *"ao usar o balde nas áreas coloridas, ele para de funcionar nas áreas não
//!    coloridas"* (Enio, 2026-09-01).
//! 2. **QUANDO a rede é reconstruída.** ⚠️⚠️ Medido:
//!
//!    | contornos | arcos | montar a rede | achar a face |
//!    |---|---|---|---|
//!    | 4 | 8 | `0,06 ms` | `0,01 ms` |
//!    | 20 | 280 | **`3,80 ms`** | `0,08 ms` |
//!    | 80 | 1293 | **`188 ms`** | `0,35 ms` |
//!
//!    ⇒ montar por quadro está refutado; achar a face por quadro é de graça. A rede é **guardada**,
//!    com a chave a ser o **CONTEÚDO** (âncoras e alças) e não a contagem de caminhos.
//!    ⭐ **E o upkeep inteiro não corre se não houver preenchimento nenhum e o balde não estiver na
//!    mão** — quem não usa a ferramenta não paga nada.
//! 3. **QUANDO os preenchimentos são RE-COZIDOS.** Sempre que a rede muda, e **em qualquer
//!    ferramenta**: o artista arrasta um nó com a seta branca e a área tem de acompanhar.

use ph2d_ecs::{Entity, SimWorld, VecBucketFill};
use ph2d_vec_fill::Rede;
use ph2d_vec_scene::{VecPath, VecScene, VecVertex, VecXforms, trim_tool};

/// A rede guardada, com a chave do documento que a produziu.
pub(crate) struct BucketCache {
    chave: u64,
    rede: Rede,
}

/// A região sob o cursor neste quadro: a área **e** o ponto que a nomeia.
#[derive(Clone, Debug)]
pub(crate) struct BucketHit {
    /// A geometria, em MUNDO — o que o realce desenha e o que o clique deposita.
    pub(crate) face: VecPath,
    /// **O ponto apontado.** É ele que vira a receita ([`VecBucketFill::seed`]), e não a área.
    pub(crate) seed: [f64; 2],
}

/// **Os contornos que são PAREDE**, no MUNDO.
///
/// ⛔ Fora ficam os escondidos (uma linha que não se vê não cerca nada) e os **preenchimentos** (a
/// nota do cabeçalho).
fn contornos_mundo(
    scene: &VecScene,
    xforms: &VecXforms,
    fora: &dyn Fn(u64) -> bool,
) -> Vec<(Vec<VecVertex>, bool)> {
    let mut out = Vec::new();
    for p in scene.paths() {
        if fora(p.id) {
            continue;
        }
        let mut cozido = p.cooked().into_owned();
        ph2d_vec_scene::bake_xform(&mut cozido, &ph2d_vec_scene::xform_of(xforms, p.id));
        for c in trim_tool::contours_of(&cozido) {
            if c.verts.len() >= 2 {
                out.push((c.verts.clone(), c.closed));
            }
        }
    }
    out
}

/// ⭐⭐⭐ **A REGIÃO QUE ESTE PREENCHIMENTO PINTOU**, achatada e em MUNDO — a receita com que ele
/// reclama as faces de hoje ([`crate::vec_bucket_claim`]).
///
/// ⚠️ **Sai da mesma porta que as paredes** (`cooked` + `bake_xform` + `detection_polyline`): quem
/// pergunta *"esta face era minha?"* tem de falar da mesma curva amostrada que decidiu onde a face
/// começa, senão a resposta muda por erro de amostragem na fronteira.
fn regiao_mundo(scene: &VecScene, xforms: &VecXforms, id: u64) -> Vec<Vec<[f64; 2]>> {
    let Some(p) = scene.path(id) else {
        return Vec::new();
    };
    let mut cozido = p.cooked().into_owned();
    ph2d_vec_scene::bake_xform(&mut cozido, &ph2d_vec_scene::xform_of(xforms, id));
    trim_tool::contours_of(&cozido)
        .filter(|c| c.closed && c.verts.len() >= 3)
        .map(|c| ph2d_vec_scene::detection_polyline(&c.verts, true))
        .collect()
}

/// ⭐⭐⭐ **A ÁREA RE-COZIDA DESCE AO ESPAÇO DO CAMINHO.**
///
/// Report do Enio (2026-09-01, com foto): *"o preenchimento está nascendo deslocado para fora do
/// stroke"*.
///
/// ⚠️⚠️ **A rede fala MUNDO e o documento guarda LOCAL** — a regra-mãe do módulo. A forma nasce
/// certa porque uma entidade nova está na identidade; mas no quadro seguinte o `settle_origins`
/// muda a **origem** dela para o centro da própria caixa (e recua a geometria o mesmo tanto), e a
/// partir daí escrever mundo naquele `VecPath` desloca-o **pelo centro dele**. Era por isso que
/// cada área saía com um desvio DIFERENTE — o desvio era o centro de cada uma.
///
/// ⛔ **Não é o `apply_bucket` que estava errado**: ali a entidade ainda nem existe. É o
/// RE-COZIMENTO, que só corre depois de a pose ter sido assentada — e é por isso que o defeito
/// aparecia *"ao nascer"*: o primeiro re-cozimento acontece no quadro seguinte ao clique.
fn para_local(verts: Vec<VecVertex>, xf: &ph2d_vec_scene::Xform) -> Option<Vec<VecVertex>> {
    if xf.is_identity() {
        return Some(verts);
    }
    let inv = xf.inverse()?;
    let mut p = VecPath {
        verts,
        ..VecPath::default()
    };
    ph2d_vec_scene::bake_xform(&mut p, &inv);
    Some(p.verts)
}

/// ⭐⭐⭐ **A FORMA DE UM PREENCHIMENTO, no espaço do caminho dele** — o contorno primário e os
/// `subpaths`, a partir das faces que ele ganhou.
///
/// ⚠️⚠️ **UMA REGIÃO QUE PARTIU CONTINUA A SER UM OBJECTO**: os pedaços são contornos do MESMO
/// caminho — o mesmo substrato que a rede soldada usa desde 2026-09-02. ⛔ Um objecto por pedaço
/// encheria a Hierarquia à primeira travessia de nó, que é o report do mesmo dia pela outra porta.
///
/// ⚠️ **A área desce ao espaço do CAMINHO** — ver [`para_local`]. Escrever mundo num caminho já
/// assentado desloca-o pelo centro dele.
///
/// ⚠️ **Existe como função, e não como fecho no sítio da chamada, porque o `bucket_upkeep` precisa
/// de uma `App` viva e não é alcançável de um teste** — um fecho ali levaria esta lei com ele.
fn geometria_local(
    rede: &ph2d_vec_fill::Rede,
    faces: &[ph2d_vec_fill::Face],
    meus: &[usize],
    xfp: &ph2d_vec_scene::Xform,
) -> Option<(Vec<VecVertex>, Vec<ph2d_vec_scene::Contour>)> {
    let mut cs = meus.iter().filter_map(|&i| {
        let g = rede.geometria(faces.get(i)?);
        (g.len() >= 2).then(|| para_local(g, xfp))?
    });
    let primeiro = cs.next()?;
    let subs = cs
        .map(|verts| ph2d_vec_scene::Contour {
            verts,
            closed: true,
        })
        .collect();
    Some((primeiro, subs))
}

/// ⭐⭐⭐ **AS DUAS RAZÕES para um contorno não ser PAREDE**, numa porta só.
///
/// ⚠️ **Ela existe porque um fecho escrito no sítio da chamada não é gateável**: a 1.ª redacção
/// tinha a regra dentro do `bucket_upkeep`, e o gate media o fecho que o **teste** construía — uma
/// mutação que apagava o termo do preenchimento **sobreviveu**. *Um gate que constrói o seu próprio
/// predicado testa o predicado dele.*
///
/// - **escondido**: uma linha que não se vê não pode cercar uma região que o artista aponta;
/// - **preenchimento**: ele tem por fronteira os mesmos arcos que as linhas, e de volta à rede põe
///   lá arestas coincidentes que envenenam o passeio de faces (o report de 2026-09-01).
fn fora_da_rede(escondido: bool, e_preenchimento: bool) -> bool {
    escondido || e_preenchimento
}

/// A chave do documento: o conteúdo das âncoras e das alças, e não a contagem.
fn chave(contornos: &[(Vec<VecVertex>, bool)]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (verts, closed) in contornos {
        h = h.wrapping_mul(0x0100_0000_01b3) ^ u64::from(*closed);
        for v in verts {
            // ⭐⭐⭐ **AS DUAS ALÇAS, e a lei é do Enio** (2026-09-01): *"o nó de uma solda é um só
            // para todas as linhas; as alças daquele nó devem servir simultaneamente para o stroke
            // e para os preenchimentos, senão é impossível que sejam transformados juntos."*
            //
            // ⛔ A 1.ª redacção lia só a de SAÍDA. Arrastar a alça de ENTRADA de um nó mudava o
            // traço e a chave **não via** — a rede não era refeita, e a área ficava com a curva de
            // antes. *As duas alças de um vértice são dois graus de liberdade, e ler um é medir
            // metade da curva.*
            for x in [
                v.anchor[0],
                v.anchor[1],
                v.in_handle[0],
                v.in_handle[1],
                v.out_handle[0],
                v.out_handle[1],
            ] {
                h = h.wrapping_mul(0x0100_0000_01b3) ^ x.to_bits();
            }
        }
    }
    h
}

/// **Os preenchimentos do documento** — `(caminho, entidade, semente em MUNDO)`.
fn preenchimentos(
    sim: &SimWorld,
    map: &crate::vec_entities::VecEntityMap,
) -> Vec<(u64, Entity, [f64; 2])> {
    let mut out = Vec::new();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if let Ok(er) = sim.world().get_entity(e)
            && let Some(f) = er.get::<VecBucketFill>()
        {
            out.push((id, e, [f64::from(f.seed[0]), f64::from(f.seed[1])]));
        }
    }
    out
}

/// ⭐⭐⭐ **Prende a receita à forma recém-nascida e manda-a para o FUNDO.**
///
/// Corre **logo depois do `vec_entities::sync`**, que é quem cria a entidade: no clique ela ainda
/// não existe.
///
/// ⚠️⚠️ **O `insert_path(0, …)` NÃO põe a forma no fundo, e foi assim que ela nasceu por cima das
/// linhas** (report do Enio, 2026-09-01: *"o preenchimento está acima do stroke, mas deveria estar
/// abaixo"*). Quem manda no desenho é o `RootOrder` da ENTIDADE, e o `sync` dá a toda entidade nova
/// **o maior** — ou seja, a frente. *O índice na cena e a ordem de desenho são duas listas, e a que
/// o olho vê é a segunda.*
pub(crate) fn arm_new_fills(
    sim: &mut SimWorld,
    map: &crate::vec_entities::VecEntityMap,
    pendentes: &mut Vec<(u64, [f32; 2])>,
) {
    for (id, seed) in pendentes.drain(..) {
        let Some(&bits) = map.get(&id) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_ok() {
            sim.world_mut()
                .entity_mut(e)
                .insert(VecBucketFill::new(seed));
        }
        crate::vec_entities::zorder::reorder(sim, map, id, ph2d_vec_scene::ZOrder::ToBack);
    }
}

impl crate::App {
    /// ⭐⭐⭐ **O UPKEEP do balde** — a rede e os preenchimentos vivos, uma vez por quadro.
    ///
    /// ⚠️ **Corre em QUALQUER ferramenta**: o artista arrasta um nó com a seta branca, e a área
    /// preenchida tem de acompanhar. ⛔ Pô-lo dentro do modo Balde era o defeito que o report de
    /// 2026-09-01 nomeia (*"se movo os nós da linha, o preenchimento não acompanha"*).
    ///
    /// ⭐ **Sai de graça quando não há nada a fazer**: sem preenchimento nenhum e sem o balde na
    /// mão, ele nem chega a cozer os contornos.
    pub(crate) fn bucket_upkeep(&mut self) {
        let armado = self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Bucket;
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let fills = preenchimentos(&gfx.sim, &self.vec_entities);
        if fills.is_empty() && !armado {
            self.vec_bucket_cache = None;
            self.vec_bucket_face = None;
            return;
        }
        let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
        let vista = crate::vec_entities::view_state(&gfx.sim, &self.vec_entities);
        let so_fill: std::collections::BTreeSet<u64> = fills.iter().map(|(id, _, _)| *id).collect();
        let contornos = contornos_mundo(&gfx.vec_scene, &xf, &|id| {
            fora_da_rede(vista.is_hidden(id), so_fill.contains(&id))
        });
        let k = chave(&contornos);
        if self.vec_bucket_cache.as_ref().is_some_and(|c| c.chave == k) {
            return; // nada mudou: nem rede nova, nem re-cozedura
        }
        let rede = ph2d_vec_fill::rede(&contornos);
        // ⛔⛔ **Acima do tecto de amostragem a rede RECUSA**, e o artista tem de saber porquê: sem
        // isto ele vê o balde parar de acender e conclui que a ferramenta se partiu. ⚠️ A linha sai
        // uma vez por reconstrução (a rede é guardada), não por quadro.
        if rede.recusada {
            eprintln!(
                "[ph2d-vec] balde: o desenho tem linhas de mais para achar as regioes ({} \
                 contornos) — os preenchimentos ficam como estao",
                contornos.len()
            );
        }
        // ⭐⭐⭐ **RE-COZER os preenchimentos**: a receita é a REGIÃO que cada um pintou, e as faces
        // de hoje herdam-na pela votação do [`crate::vec_bucket_claim`].
        //
        // Report do Enio (2026-09-02): *"quando atravessamos uma linha com um nó, os preenchimentos
        // se quebram"*. Com uma semente por preenchimento, partir uma região dava a tinta a **uma**
        // das metades (a outra ficava por pintar) e fundir duas punha **as duas sementes na mesma
        // face** — duas tintas empilhadas, e uma região sem dono.
        //
        // ⚠️ **Um preenchimento que não ganhou face nenhuma CONGELA a forma onde ela está**, em vez
        // de sumir — a mesma escolha do conector e do morph, e a única que preserva o trabalho do
        // artista. ⭐⭐ E aqui ela vale mais do que preservar: a forma congelada **é** a receita,
        // então desfazer o gesto com o próprio dedo (arrastar o nó de volta) faz a tinta voltar
        // sozinha. *No Live Paint do Illustrator ela não volta.*
        //
        // ⚠️⚠️ **E CUSTA MAIS do que custava — medido, contra o que eu tinha escrito aqui antes de
        // medir.** A 1.ª redacção desta nota dizia *"é mais barato: o `face_em` reconstruía as
        // faces uma vez por preenchimento"*; a construção das faces de facto deixou de se repetir,
        // mas a votação é `faces × amostras × regiões` e paga muito mais do que isso poupa.
        //
        // Grelha de linhas, 8 preenchimentos, `--release`:
        //
        // | faces | montar a rede | 1 `face_em` por fill | a votação |
        // |---|---|---|---|
        // | 9 | `0,15 ms` | `0,053 ms` | `0,35 ms` |
        // | 25 | `0,30 ms` | `0,119 ms` | `0,45 ms` |
        // | 49 | `0,56 ms` | `0,224 ms` | **`0,64 ms`** |
        //
        // ⇒ ~2,9× o que a semente custava, e **3,8% de um quadro** de 16,7 ms no pior caso medido —
        // que é o preço de a tinta sobreviver a uma travessia de nó. ⭐ Sem a rejeição por caixa do
        // `vec_bucket_claim` o mesmo caso custava **4,53 ms** (30% do quadro): 7,1× mais.
        let faces: Vec<ph2d_vec_fill::Face> =
            rede.faces().into_iter().filter(|f| f.area > 0.0).collect();
        let regioes: Vec<crate::vec_bucket_claim::Regiao> = fills
            .iter()
            .map(|(id, _, seed)| crate::vec_bucket_claim::Regiao {
                poligonos: regiao_mundo(&gfx.vec_scene, &xf, *id),
                semente: *seed,
            })
            .collect();
        let minhas = crate::vec_bucket_claim::por_preenchimento(
            &faces,
            &crate::vec_bucket_claim::donos(&rede, &faces, &regioes),
            fills.len(),
        );
        #[allow(clippy::cast_possible_truncation)]
        type Forma = (Vec<VecVertex>, Vec<ph2d_vec_scene::Contour>);
        let novos: Vec<(u64, Entity, Forma, [f32; 2])> = fills
            .iter()
            .enumerate()
            .filter_map(|(k, (id, e, seed))| {
                let meus = &minhas[k];
                let xfp = ph2d_vec_scene::xform_of(&xf, *id);
                let (primeiro, subs) = geometria_local(&rede, &faces, meus, &xfp)?;
                // ⭐⭐⭐ **A semente RE-SEMEIA-SE no ponto mais FUNDO da maior face.** O clique cai
                // onde o dedo caiu, e isso pode ser encostado à borda: arrastar essa parede por
                // cima do ponto fazia a face deixar de o conter (medido). Fundo, a parede tem de
                // varrer o miolo inteiro para o perder.
                let novo = rede.interior_point(&faces[meus[0]]).unwrap_or(*seed);
                Some((*id, *e, (primeiro, subs), [novo[0] as f32, novo[1] as f32]))
            })
            .collect();
        if let Some(gfx) = self.gfx.as_mut() {
            for (id, e, (primeiro, subs), seed) in novos {
                if let Some(p) = gfx.vec_scene.path_mut(id) {
                    if p.verts != primeiro {
                        p.verts = primeiro;
                    }
                    if p.subpaths != subs {
                        p.subpaths = subs;
                    }
                }
                // ⚠️ Só quando MUDA: uma escrita por quadro sobre o mesmo valor é ruído no diff do
                // undo.
                if let Ok(mut er) = gfx.sim.world_mut().get_entity_mut(e)
                    && er.get::<VecBucketFill>().is_some_and(|f| f.seed != seed)
                {
                    er.insert(VecBucketFill::new(seed));
                }
            }
        }
        self.vec_bucket_cache = Some(BucketCache { chave: k, rede });
    }

    /// **Recalcula a região sob o cursor** — só com o balde na mão, e sobre a rede JÁ guardada.
    ///
    /// ⚠️ **Fora do modo ele é LIMPO**, e não apenas não-actualizado: uma região a arder depois de
    /// trocar de ferramenta prometeria um preenchimento que nenhum clique faria.
    pub(crate) fn refresh_bucket_hover(&mut self, pointer: (f32, f32)) {
        if self.vec_draw_config.mode != ph2d_tool_vector::DrawMode::Bucket {
            self.vec_bucket_face = None;
            return;
        }
        let Some(world) = self.vec_world_at(pointer) else {
            self.vec_bucket_face = None;
            return;
        };
        let Some(cache) = self.vec_bucket_cache.as_ref() else {
            self.vec_bucket_face = None;
            return;
        };
        self.vec_bucket_face = cache
            .rede
            .face_em(world)
            .map(|f| cache.rede.geometria(&f))
            .filter(|g| g.len() >= 2)
            .map(|verts| BucketHit {
                face: VecPath {
                    verts,
                    closed: true,
                    ..VecPath::default()
                },
                seed: world,
            });
    }

    /// **A tinta que o balde deposita** — a corrente da ferramenta.
    ///
    /// ⚠️ **`alpha == 0` significa SEM preenchimento** neste app (a convenção que a ferramenta de
    /// forma usa ao fechar), e um balde que a ignorasse depositaria formas invisíveis.
    pub(crate) fn bucket_paint(&self) -> Option<ph2d_vec_scene::Rgba8> {
        let f = self.vec_pen.style().fill;
        (f.a != 0).then_some(f)
    }

    /// ⭐⭐⭐ **DEPOSITA a região que está acesa.** `true` se algo nasceu.
    ///
    /// ⚠️ **A geometria vem do ESTADO DO QUADRO** (`vec_bucket_face`): o que o artista vê aceso é
    /// literalmente o que fica.
    ///
    /// ⚠️ **A receita (a semente) é armada DEPOIS do `sync`** — no clique a entidade ainda não
    /// existe —, e é lá que a forma também vai para o fundo.
    pub(crate) fn apply_bucket(&mut self) -> bool {
        let Some(tinta) = self.bucket_paint() else {
            eprintln!(
                "[ph2d-vec] balde: o preenchimento corrente e' transparente — escolha uma cor"
            );
            return false;
        };
        let Some(hit) = self.vec_bucket_face.clone() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let nova = VecPath {
            fill: Some(ph2d_vec_scene::Paint::solid(tinta)),
            ..hit.face
        };
        let id = gfx.vec_scene.insert_path(0, nova);
        #[allow(clippy::cast_possible_truncation)]
        self.vec_bucket_new
            .push((id, [hit.seed[0] as f32, hit.seed[1] as f32]));
        self.vec_pen.select(Some(id));
        // A rede não muda (um preenchimento não é parede), mas o `upkeep` tem de reconhecer o
        // caminho novo como fill — o cache cai e o próximo quadro reconstrói com ele de fora.
        self.vec_bucket_cache = None;
        self.vec_bucket_face = None;
        true
    }
}

#[cfg(test)]
#[path = "vec_bucket_tests.rs"]
mod tests;
