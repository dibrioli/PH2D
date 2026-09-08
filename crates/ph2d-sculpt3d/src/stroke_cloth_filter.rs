//! ⭐⭐⭐ **O FILTRO DE TECIDO** — o solver do pano aplicado à peça INTEIRA, sem
//! pincel (espec §7).
//!
//! Filho (`#[path]`) de [`super`] pelo mesmo motivo do [`super::stroke_filter`]:
//! ele lê os planos congelados do traço (`touched`, `base_pos`) e escreve a lista
//! `moved`.
//!
//! # O que este ficheiro é, e o que ele NÃO é
//!
//! ⭐ **Não há lei nova aqui, e não há kernel novo em lado nenhum.** As cinco leis
//! vivem na `ph2d-cloth` — três delas desde o pincel, duas escritas para isto — e
//! o que mora neste ficheiro é *o que a mão na tela quer dizer*: montar a
//! simulação sobre a malha toda, traduzir o arrasto no escalar da espec §7, e
//! escrever o resultado.
//!
//! # ⚠️⚠️ A diferença de GESTO que separa este filtro do de malha
//!
//! O [`super::stroke_filter`] **repõe a pose congelada a cada passo** e reaplica
//! a lei com a força de agora — é um passo só, e voltar com o dedo desfaz. Este
//! **não**: a espec §7 manda *a cada movimento do rato: guardar estado → forças →
//! activar todas → passo de simulação*, ou seja **a simulação corre e acumula**.
//! ⛔ Repor a pose aqui apagaria a memória do tecido (velocidade, plasticidade,
//! desvio de repouso) e o pano deixaria de cair — seria o filtro de malha a usar
//! um solver caro para nada.
//!
//! ⇒ *voltar com o dedo não desfaz; ele aplica a força ao contrário*, que é o que
//! o alvo faz e o que uma simulação pode fazer. O que devolve a peça é o `Ctrl+Z`,
//! e ele existe porque o [`super::SculptStroke::cloth_filter_begin`] congela a
//! malha inteira pela MESMA porta do filtro de malha.

use super::{Brush, SculptStroke};
use crate::ClothFilterKind;
use ph2d_cloth::V3;
use ph2d_cloth::verlet::Solver;
use ph2d_cloth::verlet_gesto::{
    Accionamento, Area, Curva, Passo, Pincel, PincelTecido, Referencial,
};
use ph2d_mesh::{Face, Mesh};

fn v3(p: [f32; 3]) -> V3 {
    [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]
}

/// **O que UM passo do filtro recebe da tela.**
///
/// ⚠️ **Os três campos de referencial chegam JÁ RESOLVIDOS em coordenadas de
/// mundo**, e é o shell que os resolve: nem esta crate nem a `ph2d-cloth` sabem
/// o que é uma câmera, e o caso especial da orientação *View* (onde o «baixo» da
/// gravidade é o eixo do ecrã, não a profundidade — espec §7) vive lá.
#[derive(Clone, Copy, Debug)]
pub struct ClothFilterStep {
    /// **`S`, o escalar do arrasto** (espec §7), com sinal: arrastar para a
    /// direita é positivo.
    ///
    /// ⚠️ **Ele é uma RECTA, não um quadrado.** O `B` do traço é
    /// `10 · força² · flip`, e por isso um traço não consegue carregá-lo — está
    /// medido em `ph2d-cloth/tests/mede_a_composicao_do_filtro.rs`.
    pub s: f32,
    /// A direcção de mundo do «baixo» da gravidade, unitária.
    pub gravity_axis: [f32; 3],
    /// Os três eixos do referencial, em mundo.
    pub frame: [[f32; 3]; 3],
    /// Quais deles a [`ClothFilterKind::Scale`] usa.
    pub axes: [bool; 3],
    /// A direcção da superfície para o OLHO.
    ///
    /// ⚠️ **Nenhum dos cinco tipos a lê**, e ela chega na mesma porque a lei
    /// calcula a normal da área antes de escolher o modo. *Um campo inerte com o
    /// motivo escrito é mais barato que um ramo que salta a fase.*
    pub eye: [f32; 3],
}

impl SculptStroke {
    /// **Congela a peça e monta a simulação** — o pen-down do filtro.
    ///
    /// ⚠️ **A ordem de visita é derivada AQUI e uma vez só** (espec §3.1-bis):
    /// ela fixa a ordem da lista de restrições, a lista resolve-se em
    /// Gauss-Seidel, e Gauss-Seidel não comuta. É a mesma lei do traço, e a
    /// mesma porta.
    ///
    /// ⚠️ **`ponto` é o alvo congelado do [`ClothFilterKind::Pinch`]** — o que
    /// estava sob o cursor ao carregar. Os outros quatro tipos não o lêem, e o
    /// chamador que não tenha acerto pode dar o centro da peça: há gate a
    /// afirmar que os quatro não mudam com ele.
    pub fn cloth_filter_begin(
        &mut self,
        mesh: &Mesh,
        brush: &Brush,
        kind: ClothFilterKind,
        ponto: [f32; 3],
    ) {
        self.filter_begin(mesh);
        let pos: Vec<V3> = mesh.positions().iter().map(|p| v3(*p)).collect();
        let caras: Vec<&[u32]> = mesh.faces().iter().map(Face::verts).collect();
        let ordem = ph2d_cloth::particao::ordem_de_visita(&pos, &caras);
        let mut tecido =
            PincelTecido::pen_down(pincel_do_filtro(brush, kind), &pos, v3(ponto), ordem);
        // A máscara é mais um peso por-vértice, como no traço — e ela entra uma
        // vez, porque um filtro não tem carimbo que ande.
        if let Some(livre) = mesh.masks() {
            tecido.mascara = livre
                .iter()
                .map(|m| 1.0 - f64::from(crate::mask_ops::free_weight(*m)))
                .collect();
        }
        // ⭐⭐⭐ **AS RESTRIÇÕES NASCEM AO CARREGAR, e não no primeiro movimento do
        // rato** (espec §7: *«restrições construídas UMA vez, para TODOS os
        // vértices, ao carregar»*, e a lista de fases do passo dela começa na
        // **fase 2**, saltando a 0 e a 1).
        //
        // ⚠️⚠️ **A fase 0 do TRAÇO não se aplica aqui, e a espec §1 diz porquê com
        // o motivo dentro:** *«no 1.º passo de uma passagem o deslocamento do
        // cursor é zero, e os modos que dele tiram direcção, referencial ou alvo
        // não têm resposta definida sem ele»*. **Nenhum dos cinco tipos do filtro
        // tira nada de um deslocamento de cursor** — a gravidade vem do
        // chamador, o Inflate das normais, o Expand e a Escala do repouso, e o
        // aperto de um ponto congelado. ⇒ herdar o salto faria o primeiro
        // movimento do artista não fazer nada, que foi o que este gate apanhou.
        //
        // ⚠️ **Corre-se um passo de força ZERO**, e é ele a «carga» da espec: a
        // lei constrói as restrições, devolve `false` (não simulou) e deixa a
        // sessão pronta. ⛔ Não se mexe no `primeiro` à mão — isso seria escrever
        // a lei a partir de fora dela.
        let anel = |v: u32| anel_da_malha(mesh, v);
        let normais = vec![[0.0, 0.0, 1.0]; pos.len()];
        let carga = Passo {
            cursor: v3(ponto),
            delta: [0.0; 3],
            delta_3d: [0.0; 3],
            parado: false,
            vista: [0.0, 0.0, 1.0],
            normais: &normais,
            pressao: 1.0,
        };
        let simulou = tecido.passo(&pos, &anel, &carga);
        debug_assert!(!simulou, "a carga do filtro nao pode simular");
        self.cloth_filter = Some(tecido);
    }

    /// **Um passo do filtro.** Devolve quantos vértices se moveram.
    ///
    /// ⚠️ **A guarda é DERIVADA** (há sessão? a captura cobre a malha?) e não um
    /// flag — a mesma escolha, e o mesmo motivo, do [`Self::filter`]: dois campos
    /// a dizerem *«estou em modo filtro»* podem discordar.
    pub fn cloth_filter_step(
        &mut self,
        mesh: &mut Mesh,
        kind: ClothFilterKind,
        passo: &ClothFilterStep,
    ) -> usize {
        if self.touched.len() != mesh.vert_count() {
            return 0;
        }
        let Some(mut ses) = self.cloth_filter.take() else {
            return 0;
        };
        if ses.sim.len() != mesh.vert_count() {
            return 0;
        }
        // ⭐ O arrasto de AGORA — a única coisa que muda de passo para passo.
        ses.pincel.accionamento = Accionamento::Filtro {
            s: f64::from(passo.s),
        };
        ses.pincel.eixo_da_gravidade = v3(passo.gravity_axis);
        ses.pincel.referencial = Referencial {
            eixos: [v3(passo.frame[0]), v3(passo.frame[1]), v3(passo.frame[2])],
            activo: passo.axes,
        };

        let pos: Vec<V3> = mesh.positions().iter().map(|p| v3(*p)).collect();
        // ⭐⭐⭐ **AS NORMAIS SÃO AS DE AGORA, e é aqui que este filtro se separa do
        // traço** (espec §4.2-ter contra §7): o traço lê a fotografia da
        // superfície que ENCONTROU, o filtro repete a preparação da peça a cada
        // passo. Medido: `30,30 %` de divergência ao fim de seis passos, e
        // `0,000` num só — *a mesma palavra nomeia duas leis, e o chamador é que
        // escolhe qual*.
        //
        // ⚠️ Só o `Inflate` as lê, e por isso só ele paga a travessia.
        let normais: Vec<V3> = if kind.le_as_normais() {
            mesh.normals().iter().map(|n| v3(*n)).collect()
        } else {
            vec![[0.0, 0.0, 1.0]; pos.len()]
        };
        let anel = |v: u32| anel_da_malha(mesh, v);
        let p = Passo {
            // ⚠️ O ponto do `Pinch` é o do pen-down e **não segue o rato**
            // (espec §7) — ele vive no `inicio` da sessão desde o
            // [`Self::cloth_filter_begin`], e é de lá que sai.
            cursor: ses.inicio,
            // ⚠️⚠️ **O filtro não tem movimento de cursor, e `parado` continua
            // `false`.** Aquele guarda é do TRAÇO — ele pergunta *«o rato mexeu-se
            // no ecrã?»* para não aplicar força num dab repetido. Aqui a força
            // vive no `s`, e um `δ` nulo é a verdade: nenhum dos cinco tipos o lê
            // (a direcção do arrasto é do Drag, que o filtro não tem).
            delta: [0.0; 3],
            delta_3d: [0.0; 3],
            parado: false,
            vista: v3(passo.eye),
            normais: &normais,
            pressao: 1.0,
        };
        // ⚠️ **A lista é do PASSO, não do gesto** — a mesma lei dos cinco
        // vizinhos que escrevem posições (`dab_core`, os dois de tecido, o
        // filtro de malha, o sharpen). Sem isto ela cresceria a cada movimento do
        // rato e o refresco relia o gesto inteiro por quadro.
        self.moved.clear();
        let simulou = ses.passo(&pos, &anel, &p);
        let mut movidos = 0;
        if simulou {
            // Todo vértice activo é capturado antes de ser escrito — o `pre` é o
            // que o undo devolve. ⚠️ O [`Self::filter_begin`] já capturou a malha
            // inteira, e recapturar é um no-op por construção; a chamada fica
            // porque a lei *«capture antes de escrever»* é de quem escreve.
            let out = mesh.positions_mut();
            for (v, act) in ses.sim.activo.iter().enumerate() {
                if !*act {
                    continue;
                }
                let q = ses.sim.x[v];
                let novo = [q[0] as f32, q[1] as f32, q[2] as f32];
                if out[v] != novo {
                    out[v] = novo;
                    self.moved.push(u32::try_from(v).unwrap_or(u32::MAX));
                    movidos += 1;
                }
            }
        }
        // ⭐⭐⭐ **AS NORMAIS SEGUEM A FORMA, e sem esta linha o render MENTE.**
        //
        // ⛔⛔ **Ela faltava, e foi o report do dono que a apanhou** (07/09, com
        // foto: *«o render fica muito estranho, como se tivesse feito o bake de
        // uma textura»*). É exactamente isso: o passo escrevia as posições novas
        // e a malha ficava com as normais VELHAS, então a janela de upload subia
        // geometria nova com sombreamento antigo — *o relevo deixa de ser forma e
        // passa a ser um desenho colado por cima dela*.
        //
        // ⚠️ **Este era o ÚNICO escritor de posições da crate sem o refresco.**
        // Os outros cinco — o carimbo, os dois de tecido, o filtro de malha e o
        // sharpen — todos terminam aqui, e o `transform` também. *Uma família de
        // seis com um membro em falta não se vê a ler o ficheiro novo: vê-se a
        // contar a família.*
        if movidos > 0 {
            mesh.refresh_region(&self.moved, &mut self.region);
        }
        self.cloth_filter = Some(ses);
        movidos
    }

    /// **Largou o dedo** — a sessão morre e a peça fica como está.
    ///
    /// ⚠️ O undo é o do traço: o [`Self::close_stroke`] grava `touched` +
    /// `base_positions`, que é *um passo por uso do filtro* (espec §7). ⛔ Uma
    /// porta de undo própria seria a segunda resposta a *«como se desfaz um
    /// punhado de vértices deslocados»*.
    pub fn cloth_filter_end(&mut self) {
        self.cloth_filter = None;
    }

    /// **Há um filtro de tecido a correr?** — derivado, sem flag.
    #[must_use]
    pub fn cloth_filter_running(&self) -> bool {
        self.cloth_filter.is_some()
    }
}

/// O anel-1 de um vértice — ⭐ **a MESMA porta do traço**
/// ([`super::stroke_cloth_ref::anel_de`]), e não uma segunda.
///
/// ⛔⛔ **A 1.ª redacção deste ficheiro escreveu um anel próprio, e ele estava
/// errado das DUAS maneiras que importam** (report do dono, 07/09: *«péssima
/// performance, impossíveis de usar»*):
///
/// 1. **QUADRÁTICO.** Ele varria **todas as faces** por cada vértice — `O(V·F)` —
///    onde a porta do traço lê a tabela pronta de `mesh.adjacency()` em `O(1)`.
///    Medido: o pen-down crescia com `V^1,77` (`50,5 ms` a `6 836` vértices), o
///    que na malha do smoke (**`98 306`**) é **~5,6 segundos** de paragem ao
///    carregar o botão.
/// 2. **NA ORDEM ERRADA.** Ele fazia `sort_unstable()`, e a espec §3.1 diz que a
///    ordem do anel é a das **FACES** à volta do vértice — ela fixa a ordem em
///    que as restrições entram na lista, e Gauss-Seidel **não comuta**. *Um
///    `sort` é uma ordem NOSSA a substituir a do alvo*, e a bancada do pincel já
///    tinha esse aviso escrito.
///
/// ⇒ *uma segunda resposta a uma pergunta que a casa já tinha respondido, e o
/// preço dela foi um expoente e uma lei.*
fn anel_da_malha(mesh: &Mesh, v: u32) -> Vec<u32> {
    super::stroke_cloth_ref::anel_de(mesh, v)
}

/// **O [`Pincel`] do filtro** (espec §7): área *Global*, sem banda, sem pino,
/// sem curva, e o accionamento a nascer em zero.
///
/// ⚠️ **A área *Global* é o que a espec pede sem ter de inventar nada:** ela
/// devolve `w ≡ 1` por porta, põe toda a malha na lista de activos e faz a
/// construção das restrições correr **uma vez** ([`Pincel::construcoes`]) — as
/// três cláusulas da §7, já escritas e já gateadas pelas 86 fixtures do traço.
///
/// ⛔ **DIVERGÊNCIA DECLARADA:** o alvo dá ao filtro números PRÓPRIOS (massa `1`,
/// amortecimento **`0`**, colisões desligadas) e aqui ele partilha os do pincel.
/// A razão é de produto: *«as definições do tecido»* é uma coisa só para quem usa
/// o app, e duplicar cinco controlos para os separar produziria dois sítios onde
/// mexer na mesma ideia. ⚠️ O que muda de facto é o **piso do amortecimento** —
/// o traço clampa em `0,01` e a §7 admite `0` —, e aqui vale o do filtro.
fn pincel_do_filtro(brush: &Brush, kind: ClothFilterKind) -> Pincel {
    Pincel {
        modo: kind.modo(),
        area: Area::Global,
        // Sem pincel não há curva; a constante deixa a intenção explícita para
        // quem ler, e o `factor_accionado` do filtro nem chega a consultá-la.
        curva: Curva::Constante,
        forca: 1.0,
        dureza: 0.0,
        pino: false,
        accionamento: Accionamento::Filtro { s: 0.0 },
        solver: Solver {
            massa: f64::from(brush.cloth_mass.clamp(0.01, 2.0)),
            amortecimento: f64::from(brush.cloth_damping.clamp(0.0, 1.0)),
            plasticidade: f64::from(brush.cloth_plasticity.clamp(0.0, 1.0)),
            ..Solver::default()
        },
        ..Pincel::default()
    }
}

/// ⚠️ Nomeado para o gate: um tipo que não seja de âncora não pode escrever `σ`.
#[cfg(test)]
pub(crate) fn e_de_ancora(kind: ClothFilterKind) -> bool {
    use ph2d_cloth::verlet_gesto::Modo;
    matches!(kind.modo(), Modo::Escala | Modo::Agarrar | Modo::Gancho)
}
