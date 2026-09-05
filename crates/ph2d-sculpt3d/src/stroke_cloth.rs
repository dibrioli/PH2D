//! ⭐⭐⭐ **O TECIDO SOB O PINCEL** — a região que simula, o anel pregado, e o
//! primeiro verbo deste módulo cujo ESTADO sobrevive ao evento.
//!
//! # ⚠️ Por que ele BIFURCA em vez de virar mais um alvo
//!
//! Os 23 verbos anteriores respondem `alvo = f(pre_congelado, dab)` e o
//! aplicador interpola — são função pura do gesto, e é isso que torna o undo
//! trivial. Uma simulação **não é função do gesto**: ela tem velocidade, e o
//! resultado do evento *N* é a entrada do *N+1*.
//!
//! ⚠️ **Este módulo já encontrou esta forma uma vez, e ali ela era um DEFEITO:**
//! na W9a as leis de anel liam a malha viva, e num filtro isso fazia duas
//! chamadas na mesma força **comporem** — *o desenho passava a depender de
//! quantos eventos o rato mandou*. Aqui a composição **é** a feature, e a
//! diferença entre as duas situações é o **relógio**: o filtro não tem nenhum, e
//! o tecido corre em sub-passos determinísticos.
//!
//! ⇒ o `dab` desvia para cá antes de tudo, e este arquivo é dono da própria
//! expansão de simetria — cada cópia tem a **sua** região e a sua sessão.
//!
//! # ⚠️ O solver não vive aqui
//!
//! A lei é a [`ph2d_cloth`] (Vertex Block Descent), que não sabe o que é uma
//! malha nem um pincel. O que este arquivo faz é a **tradução**: escolher a
//! região, dizer quem está pregado, converter o gesto em força e devolver as
//! posições à malha com a escrituração que o undo e o upload já esperam.

use crate::{Brush, Dab, SculptStroke, Symmetry};
use ph2d_cloth::{ClothMaterial, ClothRest, ClothState, ClothTopology, StepConfig, V3};
use ph2d_mesh::Mesh;

/// **Quantos raios de pincel a região que simula tem.**
///
/// ⚠️ **Ela é MAIOR que a pegada de propósito**, e é isso que dá ao pano onde
/// responder: uma prega nasce porque o tecido em volta do dedo é puxado junto.
/// Com região = pegada, o que está fora da pegada estaria pregado, e o gesto
/// viraria um Grab com bordas duras.
pub const CLOTH_SIM_LIMIT: f32 = 2.0;

/// **Quanto o pincel pode andar antes de a região o seguir**, em fração do raio
/// dela.
///
/// ⚠️ Ele não é um teto: é a distância a partir da qual reconstruir sai mais
/// barato que empurrar de longe. Perto de `0` a região é refeita a cada dab
/// (caro, e o repouso re-medido a cada passo apaga a memória do gesto); perto de
/// `1` o pincel chega ao anel pregado antes de a região o seguir, que é o arco
/// escuro do report.
pub const CLOTH_FOLLOW: f32 = 0.25;

/// Sub-passos por evento de ponteiro.
///
/// ⚠️ **O orçamento é gasto em SUB-PASSOS e não em iterações**, que é o achado do
/// *Small Steps* (Macklin et al. 2019): `n` sub-passos de uma iteração batem um
/// passo de `n` iterações. O VBD é estável nos dois.
pub const CLOTH_SUBSTEPS: u32 = 4;

/// Iterações de VBD por sub-passo.
pub const CLOTH_ITERATIONS: u32 = 1;

/// O relógio de um evento de ponteiro.
///
/// ⚠️ **FIXO, e não o relógio de parede.** Um passo derivado do tempo real
/// tornaria o resultado função da taxa de quadros — a mesma pincelada daria
/// pregas diferentes num dia de máquina carregada, e o replay desta casa não o
/// reproduziria. *O tecido responde ao GESTO, não ao relógio.*
pub const CLOTH_DT: f64 = 1.0 / 60.0;

// ⛔⛔ **AQUI MORAVA UMA CONSTANTE DE GANHO, E ELA FOI MEDIDA E APAGADA.**
//
// A 1.ª versão convertia *quanto a mão andou* em FORÇA por um fator escolhido
// (`30`), e o gate mediu o resultado: com o gesto a percorrer `0,24`, o pano
// respondia **`5,6e-4`** — `0,2 %` do que a mão fez. O número não nomeava
// recurso nenhum (CLAUDE.md §0.0), e afiná-lo até «parecer certo» seria calibrar
// um pincel contra uma fixtura.
//
// ⭐ **A forma certa não tem constante:** sob o dedo o pano SEGUE a mão — a
// posição recebe o deslocamento do gesto, pesado pela curva do pincel, e a
// velocidade recebe esse deslocamento por unidade de tempo. O que faz a PREGA é
// o solver arrastar a vizinhança por membrana e dobra, e o `Strength` volta a ser
// o que ele é em todo verbo deste módulo: *quanto do gesto chega*.

/// **O GESTO, EM ACELERAÇÃO** — e este número é DERIVADO, não escolhido.
///
/// ⛔⛔⛔ **A 1.ª versão somava o passo à POSIÇÃO, e isso é um arrasto rígido —
/// «zero física», nas palavras do report.** O `walk` emite muitos dabs por
/// evento, então um vértice sob o pincel recebia o passo a cada um deles e era
/// levado o traço INTEIRO: medido, `0,67` de deslocamento numa esfera de raio
/// `1`, vizinhos a diferir `0,35` (as rachaduras em arco da foto) e uma aresta a
/// esticar **`2,71×`**. Escrever a posição não deixa o material discordar.
///
/// ⭐⭐ **A forma certa: o gesto propõe onde a INÉRCIA quer ir, e o solver
/// arbitra.** Uma aceleração externa `a` desloca a previsão de inércia em
/// `h²·a` por sub-passo; para que o evento inteiro proponha exatamente `passo·w`,
///
/// ```text
/// Δx = ½·a·dt²   ⇒   a = 2·passo·w / dt²
/// ```
///
/// ⇒ **nenhum número inventado**: ele sai da cinemática do evento. Quando o
/// material não resiste, o vértice chega ao que a mão propôs; quando resiste, ele
/// fica pelo caminho — e é **essa diferença** que é a física.
///
/// ⛔⛔ **E a 3.ª versão dividia pelos SUB-PASSOS, o que fazia o solver piorar
/// ao receber mais orçamento** — medido: `4 → 8 → 16` sub-passos levavam o
/// esticão de `2,3×` a `5,7×` e a `10,7×`. *Um solver que fica pior com mais
/// orçamento não está a convergir: há um termo que depende do orçamento.* Ele
/// era o MOMENTO — uma aceleração aplicada durante o evento inteiro também
/// injeta `a·dt` de velocidade, e eu tinha somado só os `h²·a` de posição. A
/// forma acima é a cinemática completa, e é **independente dos sub-passos** por
/// construção (há gate).
///
/// ⚠️ **E a 2.ª versão INVENTOU um ganho (`30`) e ficou 32× fraca** — o valor
/// correto é `sub-passos/dt²`, e a distância entre um palpite e uma derivação foi
/// exatamente esse fator.
///
/// ⚠️⚠️ **Ela é uma FUNÇÃO do orçamento, e o parâmetro é o gate.** Enquanto era
/// uma `const`, o gate `o_gesto_nao_depende_do_orcamento_do_solver` estava verde
/// **por vácuo**: ele varia os sub-passos em execução e a constante era de
/// compilação, então nenhuma fórmula errada podia ser vista — a mutação que
/// repunha a divisão pelo orçamento **SOBREVIVEU**. Recebendo o `StepConfig`, a
/// lei correta usa só o `dt` e ignora o `substeps`; uma que o leia diverge, e a
/// mutação passa a sangrar.
fn gesto_para_aceleracao(cfg: &StepConfig) -> f64 {
    2.0 / (cfg.dt * cfg.dt)
}

/// **A SESSÃO de tecido de UMA cópia de simetria, dentro de UM traço.**
///
/// ⚠️ **Ela nasce no primeiro dab e morre no pen-up.** Tudo o que depende do
/// repouso — a topologia, a coloração, as áreas, os ângulos, as massas — é
/// medido **uma vez**; por evento sobra o passo do solver e a escrita.
#[derive(Clone, Debug)]
pub(super) struct ClothSession {
    topo: ClothTopology,
    rest: ClothRest,
    state: ClothState,
    /// Índice local → vértice da malha. **Ordenado**, e a ordenação é o que
    /// torna a região função da malha e não da ordem em que a consulta a devolveu.
    verts: Vec<u32>,
    pinned: Vec<bool>,
    /// A aceleração que o gesto propõe, reusada entre eventos.
    ext: Vec<V3>,
    /// Onde a região foi centrada, e qual o raio dela.
    em: [f32; 3],
    raio: f32,
}

/// O material do pano, derivado do pincel.
///
/// ⚠️ **Os números não são do solver, são do PANO** — e enquanto o painel da
/// W10c não existe eles são a tabela de fábrica, com o `Strength` do pincel a
/// entrar pela FORÇA e não pela rigidez. *Um slider que mudasse a rigidez faria
/// o mesmo gesto dar pregas de tamanhos diferentes conforme a pressão.*
fn material() -> ClothMaterial {
    ClothMaterial {
        density: 1.0,
        young: 40.0,
        poisson: 0.3,
        bending: 2.0e-3,
        damping: 0.05,
    }
}

impl SculptStroke {
    /// **O DAB do tecido** — a porta que o [`SculptStroke::dab`] desvia para cá.
    ///
    /// Ela é dona da própria expansão de simetria: cada cópia tem a sua região,
    /// porque duas regiões do outro lado da peça não partilham vértice nenhum e
    /// juntá-las numa só faria o solver resolver um sistema desconexo.
    pub(super) fn cloth_dab(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        dab: &Dab,
        sym: Symmetry,
    ) -> usize {
        let (signs, n) = sym.signs();
        self.moved.clear();
        for (copy, s) in signs.iter().take(n).enumerate() {
            let center = [
                dab.center[0] * s[0],
                dab.center[1] * s[1],
                dab.center[2] * s[2],
            ];
            let path = [
                f64::from(dab.path[0] * s[0]),
                f64::from(dab.path[1] * s[1]),
                f64::from(dab.path[2] * s[2]),
            ];
            self.cloth_copy(mesh, brush, dab, center, path, copy);
        }
        // ⚠️⚠️ **ESCRITA, e não herdada.** O [`SculptStroke::last_gpu_dirty`]
        // escolhe a janela de upload por esta bandeira, e o `begin` NÃO a
        // reinicia — sem esta linha um traço de tecido logo depois de um traço
        // de MÁSCARA subiria a janela errada, e o defeito seria *«a malha mudou
        // e a tela não»*, com todos os gates de CPU verdes. O tecido move
        // geometria, ponto.
        self.last_paints_mask = false;
        if self.moved.is_empty() {
            return 0;
        }
        mesh.refresh_region(&self.moved, &mut self.region);
        self.moved.len()
    }

    /// Uma cópia: garante a sessão, aplica a força, avança e escreve.
    fn cloth_copy(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        dab: &Dab,
        center: [f32; 3],
        path: V3,
        copy: usize,
    ) {
        if self.cloth.len() <= copy {
            self.cloth.resize_with(copy + 1, || None);
        }
        // ⛔⛔ **A REGIÃO ACOMPANHA O PINCEL, e a 1.ª versão a congelava no
        // pen-down.** O traço do report atravessa a peça: com a região presa no
        // primeiro toque, o pincel SAI dela e passa a empurrar quem já não está
        // sob o dedo — e o anel pregado, imóvel, vira uma parede. É o arco
        // escuro da foto. A referência diz a mesma coisa por outras palavras:
        // *«a área de simulação acompanha o pincel, limitada por um raio fixo»*.
        //
        // ⚠️ **Refazer CARREGA a velocidade dos vértices que ficam**, senão cada
        // reconstrução seria uma paragem brusca no meio do gesto — e o repouso é
        // re-medido na malha de AGORA, que é o que torna a prega já feita
        // permanente (é um pincel de escultura, não um simulador de roupa).
        let refaz = self.cloth[copy].as_ref().is_none_or(|s| {
            let d = [
                center[0] - s.em[0],
                center[1] - s.em[1],
                center[2] - s.em[2],
            ];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() > s.raio * CLOTH_FOLLOW
        });
        if refaz {
            let velha = self.cloth[copy].take();
            let Some(mut nova) = self.build_cloth(mesh, center, dab.radius) else {
                return;
            };
            if let Some(v) = velha {
                nova.herda(&v);
            }
            self.cloth[copy] = Some(nova);
        }
        // ⚠️ **A sessão sai do vetor durante o passo**, porque o solver precisa
        // dela por `&mut` enquanto a malha também é `&mut` — e as duas vivem no
        // mesmo `self`. Ela volta no fim, sempre.
        let Some(mut ses) = self.cloth[copy].take() else {
            return;
        };
        // ⚠️ **UM `StepConfig`, dois consumidores** — o solver e a lei do gesto.
        // Dois orçamentos escritos em sítios diferentes divergiriam no dia em que
        // alguém mexesse num, e é exatamente por eles serem o mesmo que o gate da
        // independência pode existir.
        let cfg = StepConfig {
            dt: CLOTH_DT,
            #[cfg(test)]
            substeps: self.cloth_substeps_override.unwrap_or(CLOTH_SUBSTEPS),
            #[cfg(not(test))]
            substeps: CLOTH_SUBSTEPS,
            iterations: CLOTH_ITERATIONS,
            gravity: [0.0; 3],
        };
        self.cloth_drive(&mut ses, brush, dab, center, path, &cfg);
        ph2d_cloth::step(
            &ses.topo,
            &ses.rest,
            &material(),
            &ses.pinned,
            &ses.ext,
            &cfg,
            &mut ses.state,
        );
        let out = mesh.positions_mut();
        for (i, v) in ses.verts.iter().enumerate() {
            if ses.pinned[i] {
                continue;
            }
            let p = ses.state.x[i];
            let (vi, novo) = (*v as usize, [p[0] as f32, p[1] as f32, p[2] as f32]);
            if out[vi] != novo {
                out[vi] = novo;
                self.moved.push(*v);
            }
        }
        self.cloth[copy] = Some(ses);
    }

    /// **A REGIÃO** — quem simula, quem está pregado, e o repouso de tudo isso.
    ///
    /// ⚠️ **Todo vértice da região é CAPTURADO**, pregado incluído: o `pre` é o
    /// que o undo devolve, e um vértice que a simulação move sem ter sido
    /// capturado é um vértice que o `Ctrl+Z` não sabe repor.
    fn build_cloth(&mut self, mesh: &Mesh, center: [f32; 3], radius: f32) -> Option<ClothSession> {
        let limit = radius * CLOTH_SIM_LIMIT;
        mesh.verts_in_sphere(center, limit, &mut self.query, &mut self.footprint);
        if self.footprint.len() < 4 {
            return None;
        }
        let mut verts = self.footprint.clone();
        verts.sort_unstable();
        verts.dedup();
        let dentro = |v: u32| verts.binary_search(&v).is_ok();

        // As faces cujos TRÊS cantos estão na região. Uma face é vista uma vez
        // por canto, então ela é recolhida e depois deduplicada — juntar por
        // `HashSet` daria uma ordem que não é função da malha.
        let adj = mesh.adjacency();
        let mut faces: Vec<u32> = Vec::new();
        for v in &verts {
            faces.extend_from_slice(adj.vert_faces.neighbours(*v as usize));
        }
        faces.sort_unstable();
        faces.dedup();

        let mut tris: Vec<[u32; 3]> = Vec::new();
        // ⚠️ **Uma face de fronteira NÃO entra, e ela PREGA os cantos dela.** É
        // assim que a região se cola ao resto da escultura: o pano acaba onde a
        // malha continua, e ali ele não pode andar.
        let mut borda = vec![false; verts.len()];
        for f in &faces {
            let face = &mesh.faces()[*f as usize];
            let todos = face.verts().iter().all(|v| dentro(*v));
            if !todos {
                for v in face.verts() {
                    if let Ok(i) = verts.binary_search(v) {
                        borda[i] = true;
                    }
                }
                continue;
            }
            for k in 0..face.tri_count() {
                let t = face.tri_at(k);
                tris.push([
                    local(&verts, t[0]),
                    local(&verts, t[1]),
                    local(&verts, t[2]),
                ]);
            }
        }
        if tris.is_empty() {
            return None;
        }

        let pos = mesh.positions();
        let x: Vec<V3> = verts
            .iter()
            .map(|v| {
                let p = pos[*v as usize];
                [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]
            })
            .collect();
        // ⛔⛔ **PREGA SÓ A FRONTEIRA TOPOLÓGICA, e a banda por DISTÂNCIA foi
        // MEDIDA E APAGADA.** Ela existia como o *«lock vertices in the
        // simulation falloff area»* da referência, a `0,7` do raio da região — e
        // varrida de `0,7` a `0,95` e depois REMOVIDA, ela move o produto na 4.ª
        // casa decimal, em TODOS os materiais (`young` de `100` a `15`):
        //
        // | material | com a banda | só a fronteira |
        // |---|---|---|
        // | `young 40` | `0,0522` / `0,0176` | `0,0524` / `0,0177` |
        //
        // ⇒ ela é morta **porque a deformação nunca chega até ela**: com o pano a
        // responder `~17 %` do raio do pincel, o que está a `0,7` da região não se
        // mexe, pregado ou não. *Um limite que só diz «por segurança» é um palpite
        // esperando um smoke*, e este smoke aconteceu. Quem PRECISA de estar
        // pregado é a fronteira, que é onde o pano encontra o resto da peça.
        let pinned: Vec<bool> = borda;

        for v in &verts {
            self.capture(mesh, *v);
        }
        let topo = ClothTopology::build(&tris, verts.len());
        let rest = ClothRest::measure(&topo, &x, &material());
        Some(ClothSession {
            state: ClothState::at_rest(&x),
            ext: vec![[0.0; 3]; verts.len()],
            em: center,
            raio: limit,
            topo,
            rest,
            verts,
            pinned,
        })
    }
}

/// O índice local de um vértice que já se sabe estar na região.
fn local(verts: &[u32], v: u32) -> u32 {
    u32::try_from(verts.binary_search(&v).unwrap_or(0)).unwrap_or(0)
}

impl ClothSession {
    /// **CARREGA a velocidade da região anterior** para os vértices que ficam.
    ///
    /// ⚠️ Sem isto, cada vez que a região segue o pincel o pano PARA — e o
    /// artista vê o gesto a engasgar em intervalos regulares, que é pior que não
    /// seguir. Os dois `verts` são ordenados, então a interseção é uma passagem
    /// só.
    fn herda(&mut self, velha: &Self) {
        let (mut a, mut b) = (0usize, 0usize);
        while a < self.verts.len() && b < velha.verts.len() {
            match self.verts[a].cmp(&velha.verts[b]) {
                core::cmp::Ordering::Less => a += 1,
                core::cmp::Ordering::Greater => b += 1,
                core::cmp::Ordering::Equal => {
                    if !self.pinned[a] {
                        self.state.v[a] = velha.state.v[b];
                    }
                    a += 1;
                    b += 1;
                }
            }
        }
    }
}

impl SculptStroke {
    /// **O GESTO ENTRA NO PANO** — sob o dedo ele SEGUE a mão; em volta, o solver.
    ///
    /// ⭐⭐ **Sem constante de conversão, e é isso que a torna correta:** o
    /// deslocamento do gesto é somado à posição (pesado pela curva do pincel) e à
    /// velocidade (o mesmo, por unidade de tempo). Debaixo do dedo com peso `1` o
    /// pano acompanha a mão exatamente; a prega nasce do que o solver faz com a
    /// VIZINHANÇA, que não recebe gesto nenhum e é arrastada por membrana e dobra.
    ///
    /// ⚠️ **A velocidade entra junto de propósito.** Só a posição daria um pano
    /// que para no instante em que a mão para; com o momento, ele continua e
    /// assenta — que é o que faz uma prega parecer pano e não borracha.
    ///
    /// ⚠️⚠️ **A MÁSCARA e o ALPHA entram aqui, pelas MESMAS portas do laço
    /// normal** (`mask_ops::free_weight` e `Brush::alpha_weight`), lidos no `pre`
    /// CONGELADO. A lei deste módulo é que *o alpha é mais um peso por-vértice,
    /// como a máscara* — escrita para o filtro na W9, vale aqui pela mesma razão.
    /// Um pincel de tecido que ignorasse a máscara destruiria a região que o
    /// artista protegeu.
    fn cloth_drive(
        &self,
        ses: &mut ClothSession,
        brush: &Brush,
        dab: &Dab,
        center: [f32; 3],
        path: V3,
        cfg: &StepConfig,
    ) {
        let frame = brush.alpha_frame();
        let escala = gesto_para_aceleracao(cfg);
        let ganho = f64::from(brush.weight() * dab.pressure.clamp(0.0, 1.0));
        let inv_r = 1.0 / dab.radius;
        for i in 0..ses.verts.len() {
            ses.ext[i] = [0.0; 3];
            if ses.pinned[i] {
                continue;
            }
            let v = ses.verts[i] as usize;
            let p = ses.state.x[i];
            let d = [
                p[0] - f64::from(center[0]),
                p[1] - f64::from(center[1]),
                p[2] - f64::from(center[2]),
            ];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let t = (dist * f64::from(inv_r)) as f32;
            if t >= 1.0 {
                continue;
            }
            // ⚠️ **A curva do pincel, pela PORTA do pincel** — não uma segunda
            // lei de queda escrita aqui. Duas respostas para *«quanto este
            // vértice sente»* divergiriam no dia em que o artista mexesse na
            // dureza.
            let s = self.slot[v] as usize;
            let base = self.base_pos[s];
            let w = ganho
                * f64::from(
                    brush.falloff.weight(brush.shaped_distance(t))
                        * brush.alpha_weight(base, &frame)
                        * crate::mask_ops::free_weight(self.base_mask[s]),
                );
            for (c, andou) in path.iter().enumerate() {
                ses.ext[i][c] = andou * w * escala;
            }
        }
    }
}
