//! **O GESTO DO FILTRO** — o verbo corrente na malha INTEIRA, com o arrasto
//! horizontal a dar a força.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados; o corte é o
//! do [`super::transform`], o vizinho de que este módulo é a cópia estrutural: a
//! LEI mora no kernel ([`ph2d_sculpt3d::SculptStroke::filter`]) e o que mora
//! aqui é *o que a mão na tela quer dizer*.
//!
//! # O gesto é o da referência, e o número também
//!
//! **Comportamento medido:** o arrasto para a **DIREITA é positivo**, e a régua
//! é [`FILTER_DRAG_PER_PX`] = `0,001` de força por pixel — mil pixels de arrasto
//! valem força `1,0`.
//!
//! ⚠️ **Divergência declarada:** aqui a força é o arrasto e **nada mais**; a
//! referência multiplica-a ainda por uma força inicial da ferramenta, e nós não
//! (o doc do driver diz porquê).
//!
//! Quem mede: `shells/desktop/src/sculpt3d_filter_tests.rs` (o SINAL do arrasto
//! e a régua) e `crates/ph2d-sculpt3d/src/stroke_filter_tests.rs` (`1000 px` de
//! arrasto ⇒ força `1,0`).
//!
//! # Por que não há `close_filter`
//!
//! Porque o [`super::Sculpt3dScene::close_stroke`] **já é** a porta. O
//! [`ph2d_sculpt3d::SculptStroke::filter_begin`] preenche exactamente os dois
//! arrays que o fecho de um traço grava (`touched` e `base_positions`), então
//! uma porta própria seria a segunda resposta a *"como se desfaz um punhado de
//! vértices deslocados"* — o mesmo argumento que fez o transform reusar a
//! [`super::StrokeUndo::Stroke`] em vez de inventar um variant.
//!
//! ⚠️ E o braço da MÁSCARA do `close_stroke` é inerte aqui por construção: ele
//! pergunta `verb.paints_mask()`, e nenhum verbo que filtra pinta máscara.

use super::{FILTER_DRAG_PER_PX, Sculpt3dScene};
use ph2d_sculpt3d::ClothFilterOrientation;

/// ⭐⭐⭐ **O QUE O ARTISTA AFINOU NO FILTRO DE TECIDO.**
///
/// ⚠️ **GLOBAL, e não por-verbo**: o `slots` do painel guarda o pincel de cada
/// ferramenta porque *afinar a força do Smooth não é afinar a do Clay* — e um
/// filtro não pertence a ferramenta nenhuma. Três dos cinco tipos não têm verbo.
///
/// ⛔⛔ **E as PROPRIEDADES são DELE, não do pincel** (pergunta do dono,
/// 2026-09-08). Ver [`ph2d_sculpt3d::ClothFilterProps`] para o que estava errado
/// e porquê.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Tecido {
    /// **O REFERENCIAL do filtro de tecido** (espec §7) — ele decide os eixos da
    /// Escala e a direcção do «baixo» da Gravidade.
    ///
    /// ⚠️ Estado de FERRAMENTA, como o [`Self::filter_law`]: ele é do artista e
    /// sobrevive a trocar de verbo.
    pub(super) orientation: ph2d_sculpt3d::ClothFilterOrientation,
    /// ⭐⭐ **OS QUATRO NÚMEROS DO FILTRO** — dele, e não do pincel.
    pub(super) props: ph2d_sculpt3d::ClothFilterProps,
    /// ⭐⭐ ***Force Axis*** — quais eixos a Escala usa (espec §7).
    ///
    /// ⛔ **Era um controlo que NÃO EXISTIA**, e a distinção com um knob morto é
    /// a cura: a de um morto é ligar o braço, a de um ausente é criá-lo. O motor
    /// já o honrava (gate `escala_eixox` a `0,009151`).
    pub(super) axes: [bool; 3],
    /// **O `x` do último evento de ponteiro do filtro de TECIDO, por drenar.**
    ///
    /// ⚠️ **Um evento regista, o QUADRO corre** — ver
    /// [`Sculpt3dScene::flush_cloth_filter`]. Irmão do `pending_grab`, e pela
    /// mesma razão: sem isto, quantos passos a simulação avança seria função da
    /// taxa de amostragem do rato.
    pub(super) pending: Option<f32>,
}

impl Default for Tecido {
    fn default() -> Self {
        Self {
            orientation: ph2d_sculpt3d::ClothFilterOrientation::default(),
            props: ph2d_sculpt3d::ClothFilterProps::default(),
            // ⚠️ Os três LIGADOS: é a omissão do alvo, e é a única que faz a
            // Escala escalar em todas as direcções como sempre escalou.
            axes: [true; 3],
            pending: None,
        }
    }
}

impl Sculpt3dScene {
    /// **O filtro está armado?**
    ///
    /// ⚠️ **A PREMISSA MUDOU com a W9b, e a metade do VERBO saiu.** Ela existia
    /// porque a lei era DERIVADA do verbo em mãos: sem um verbo que filtrasse
    /// não havia lei, então um arm que sobrevivesse a pegar o Draw ficava
    /// **aceso e invisível** — o botão esquerdo parava de esculpir sem nada na
    /// tela dizendo por quê. Com a lei ESCOLHIDA ([`Sculpt3dScene::filter_kind`])
    /// ela existe sempre, a row é sempre pintada, e *visível ⇔ vivo* passa a
    /// valer por outra via: **um arm aceso é sempre visível**, então a
    /// preocupação some e o mecanismo que a resolvia sai com ela.
    pub(crate) fn filter_arm(&self) -> bool {
        self.filter_arm
    }

    /// **Arma ou desarma** — clicar o aceso desliga, a lei de todo toggle deste
    /// app.
    ///
    /// ⚠️ **Ele DESARMA o transform**, e a exclusão mora nas duas portas de
    /// armar em vez de num `enum` de modo: os dois reivindicam o MESMO botão
    /// esquerdo, e um gesto que significasse as duas coisas não teria como
    /// escolher. Um `enum` seria a resposta mais limpa e obrigaria a reescrever
    /// os cinco leitores do `transform_arm`; a exclusão por porta custa duas
    /// linhas e tem gate.
    pub(crate) fn arm_filter(&mut self) -> bool {
        self.filter_arm = !self.filter_arm;
        if self.filter_arm {
            self.transform_arm = None;
            // ⚠️ **O verbo SEMEIA a escolha, e só ao ARMAR.** Quem pega o Smooth
            // e liga o filtro quer alisar — fazer o artista escolher de novo o
            // que a ferramenta na mão dele já diz seria um passo a mais em todo
            // gesto. E é semente e não amarra: trocar de verbo com o filtro já
            // aceso **não** re-escreve a lei, senão a escolha do selector seria
            // apagada por um gesto que não fala sobre ela. Um verbo sem lei
            // própria (Grab, Twist, …) deixa a última escolha de pé.
            // ⚠️ **Só um verbo de MALHA semeia**, e a assimetria é a verdade: os
            // cinco tipos de tecido não têm verbo que os semeie (não existe
            // pincel de gravidade), e o [`Verb::Cloth`] é um TRAÇO, não um
            // filtro. ⛔ Fazê-lo semear `Cloth(Gravity)` trocaria a escolha do
            // artista por um palpite sobre um gesto que ele não fez.
            if let Some(kind) = self.brush.verb.filter_kind() {
                self.filter_law = ph2d_sculpt3d::FilterLaw::Mesh(kind);
            }
        }
        self.filter_arm
    }

    /// **Congela a foto e abre o gesto.** `false` quando o verbo em mãos não
    /// filtra — e a recusa é REPORTADA pelo chamador, como a do transform: um
    /// gesto que não faz nada e não diz nada é indistinguível de um botão que
    /// não chegou.
    pub(super) fn begin_filter(&mut self, x: f32, y: f32) -> bool {
        if !self.filter_arm() {
            return false;
        }
        let mesh = self.mesh().clone();
        if let Some(kind) = self.filter_law.cloth() {
            // ⭐ **O pen-down do tecido faz as DUAS coisas do `filter_begin`** (a
            // foto congelada para o undo) **mais a carga da simulação** — ver
            // [`ph2d_sculpt3d::SculptStroke::cloth_filter_begin`].
            //
            // ⚠️ **O ponto do aperto é o do ACERTO** (espec §7: *o filtro aperta
            // para onde o cursor estava quando ele começou*), e o `aim` do
            // chamador já correu — é isso que faz o [`Self::hit_point`] descrever
            // este clique. Sem acerto (o artista carregou fora da peça) fica o
            // centro da caixa, que é a única resposta honesta.
            let ponto = self.filter_pinch_anchor(x, y);
            // ⭐⭐ **A FOTOGRAFIA DOS COLISORES** (espec §7) — a mesma lei do
            // traço: a lista é tirada no pen-down, e uma peça que se mova
            // durante o gesto **não se move para o pano**.
            //
            // ⚠️ **A pose de cada peça entra na cópia**: a lei recebe posições em
            // espaço de MUNDO, e o `Multires::mesh` está em espaço local.
            self.stroke.cloth_colliders.clear();
            if self.tecido.props.collisions {
                let activo = self.active;
                for (i, o) in self.objects.iter().enumerate() {
                    if i != activo {
                        self.stroke
                            .cloth_colliders
                            .push((o.stack.mesh().clone(), o.pose));
                    }
                }
            }
            self.stroke
                .cloth_filter_begin(&mesh, self.tecido.props, kind, ponto);
        } else {
            self.stroke.filter_begin(&mesh);
        }
        self.filter_from_x = x;
        true
    }

    /// ⭐⭐⭐ **O QUADRO DRENA UM PASSO DO FILTRO DE TECIDO** — a metade que faz do
    /// evento um registo e do quadro o relógio.
    ///
    /// ⚠️ **Ela roda uma vez por quadro, com ou sem arrasto**, e o `take()` é a
    /// guarda inteira: sem evento pendente não há nada a fazer, e com dez
    /// pendentes só o ÚLTIMO conta — que é a posição de agora, e a lei já é
    /// função do arrasto TOTAL desde o pen-down.
    ///
    /// ⚠️ **E ela é chamada TAMBÉM no pen-up**, antes do fecho, pelo mesmo motivo
    /// que o `flush_pending_grab`: o último movimento do dedo chega como evento e
    /// fica pendente — *sem isso o gesto perde a ponta*, e o pano pararia onde o
    /// último QUADRO o deixou.
    pub(crate) fn flush_cloth_filter(&mut self) {
        let Some(x) = self.tecido.pending.take() else {
            return;
        };
        let Some(kind) = self.filter_law.cloth() else {
            return;
        };
        let amount = (x - self.filter_from_x) * FILTER_DRAG_PER_PX;
        let passo = self.cloth_filter_step_of(amount);
        let mesh = self.objects[self.active].stack.mesh_mut();
        if self.stroke.cloth_filter_step(mesh, kind, &passo) == 0 {
            return;
        }
        let touched = self.stroke.touched().to_vec();
        Self::mesh_changed(
            &mut self.objects[self.active].dirty,
            &mut self.edits,
            &touched,
        );
    }

    /// **Larga a sessão do filtro de tecido** — chamado no pen-up.
    pub(crate) fn end_cloth_filter(&mut self) {
        self.stroke.cloth_filter_end();
        // ⚠️ O pendente morre com o gesto: deixá-lo vivo faria o primeiro quadro
        // depois do pen-up correr um passo sobre uma pose já gravada como undo.
        self.tecido.pending = None;
    }

    /// **O ALVO CONGELADO do aperto** — onde o raio do pen-down bateu na peça.
    ///
    /// ⚠️ **Sem acerto fica o centro da caixa da peça**, e é a única resposta
    /// honesta: o artista carregou fora do barro, e apertar *para o canto do
    /// ecrã* seria inventar um ponto que ele não apontou. ⛔ Recusar o gesto
    /// seria pior — os outros quatro tipos não têm ponto nenhum e funcionariam.
    ///
    /// ⭐⭐⭐ **E ele ENCOSTA NO VÉRTICE, não fica no ponto da face** (espec §7: o
    /// alvo do aperto é o **vértice activo** no instante da abertura). ⚠️ **A
    /// diferença foi MEDIDA e não é decoração:** na bancada do oráculo
    /// (`ph2d-cloth/tests/oraculo_do_filtro.rs`) apertar para o ponto solto do
    /// cursor em vez do vértice dá erro `0,696942`; para o vértice, `0,010755` —
    /// **65× melhor**. *A diferença entre apertar para um ponto da malha e para
    /// um ponto que não pertence a ela.*
    ///
    /// ⛔⛔ **O ponto chega por ARGUMENTO, e não de `self.last`** — a 1.ª redacção
    /// lia o campo, e ele está ERRADO exactamente aqui: o `sculpt3d_pointer_down`
    /// escreve `scene.last` **depois** de chamar o `begin_filter`, e o
    /// `sculpt3d_pointer_move` só o actualiza **com um arrasto em curso**. ⇒ no
    /// pen-down ele ainda guarda *onde o gesto ANTERIOR acabou*, e o aperto
    /// puxaria para lá. Quem o apanhou foi o gate do vértice, que leu a âncora no
    /// centro da caixa (o raio nem sequer acertava na peça).
    pub(super) fn filter_pinch_anchor(&self, x: f32, y: f32) -> [f32; 3] {
        let Some(o) = self.obj() else {
            return [0.0; 3];
        };
        let mesh = o.stack.mesh();
        let Some(hit) = self.pick_active(x, y) else {
            return mesh.bounds().center();
        };
        // O vértice mais próximo DA FACE ACERTADA — três candidatos, e não uma
        // varredura da malha: o raio já disse onde bateu.
        let Some(face) = mesh.faces().get(hit.face as usize) else {
            return hit.point;
        };
        let d2 = |v: u32| {
            let p = mesh.positions()[v as usize];
            let d = [
                p[0] - hit.point[0],
                p[1] - hit.point[1],
                p[2] - hit.point[2],
            ];
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
        };
        face.verts()
            .iter()
            .copied()
            .min_by(|a, b| d2(*a).total_cmp(&d2(*b)))
            .map_or(hit.point, |v| mesh.positions()[v as usize])
    }

    /// **O que um passo do filtro de tecido lê da CÂMERA e da peça.**
    ///
    /// ⚠️ **A simulação corre no espaço LOCAL da peça activa** (é `mesh.positions()`
    /// que entra na lei), então todo eixo tem de chegar lá — e a
    /// [`ph2d_mesh::Pose`] desta casa não tem rotação, o que faz a conversão de
    /// direcção ser a identidade. É por isso que o *World* não é oferecido
    /// ([`ClothFilterOrientation::offered`]).
    ///
    /// ⚠️⚠️ **O caso especial da vista é da ESPEC §7 e vive aqui**, que é o único
    /// sítio com uma matriz de câmera: na orientação *View* o «baixo» da
    /// gravidade é o eixo **vertical do ecrã**, e não a profundidade.
    ///
    /// Medido por `na_vista_o_baixo_e_o_vertical_do_ecra_e_nao_a_profundidade`
    /// (`shells/desktop/src/sculpt3d_filter_cloth_tests.rs`), com o ecrã
    /// **inclinado 90°** — ali *cima do
    /// mundo* e *profundidade* são a MESMA resposta errada, e só a vertical do
    /// ecrã se separa das duas. ⛔ Com o ecrã alinhado ao mundo os dois braços
    /// coincidem ao bit e um gate seria verde por vácuo.
    pub(super) fn cloth_filter_step_of(&self, amount: f32) -> ph2d_sculpt3d::ClothFilterStep {
        let v = self.camera.view();
        // As LINHAS de uma matriz de vista ortonormal são os eixos do ECRÃ em
        // coordenadas de mundo — a mesma leitura que a `Camera3d::pan` faz.
        let ecra = [0, 1, 2].map(|r| {
            let c = v.row(r);
            [c.x, c.y, c.z]
        });
        let (frame, gravity) = referencial_e_gravidade(self.tecido.orientation, ecra);
        let eye = ecra[2];
        ph2d_sculpt3d::ClothFilterStep {
            s: amount,
            gravity_axis: gravity,
            frame,
            // ⭐⭐ **O *Force Axis*** (espec §7), que só a Escala lê — e desde
            // 2026-09-08 ele EXISTE. A nota que vivia aqui dizia *«é um controlo
            // que não existe, e a cura é criá-lo, não ligar um braço»*: foi
            // criado.
            axes: self.tecido.axes,
            eye,
        }
    }

    /// O dedo andou: re-roda a lei sobre a pose CONGELADA com a força de agora.
    ///
    /// ⚠️ **A força é o arrasto TOTAL desde o pen-down, nunca um incremento**, e
    /// é o que faz do gesto um passo só: o driver repõe a pose congelada a cada
    /// chamada, então voltar com o dedo desfaz — e um filtro que compusesse
    /// incrementos dependeria de quantos eventos o rato mandou, a lei que esta
    /// casa já pagou quatro vezes no relevo do Painter.
    pub(super) fn filter_at(&mut self, x: f32) {
        if !self.filter_arm() {
            return;
        }
        // ⭐⭐⭐ **UM EVENTO DE PONTEIRO NÃO É UM PASSO DE SIMULAÇÃO.**
        //
        // ⛔⛔ O braço vizinho deste mesmo `match` já escreve a lei — *«um evento
        // de ponteiro NÃO é um dab»* — e o filtro de tecido nasceu a violá-la: ele
        // corria **um passo de solver por evento do sistema**, e um rato de
        // `1000 Hz` entrega dezasseis por quadro. ⇒ *quantos passos a simulação
        // avança passava a ser função da TAXA DE AMOSTRAGEM do rato*, que é a lei
        // que esta casa já pagou **seis vezes** no relevo do Painter.
        //
        // ⚠️ **Ela é de CORRECÇÃO antes de ser de relógio:** dois artistas com
        // ratos diferentes obtinham panos diferentes do mesmo gesto. Que também
        // cure o report de performance é consequência, não a razão.
        //
        // ⇒ o evento **regista** e o QUADRO drena ([`Self::flush_cloth_filter`]),
        // exactamente como o `flush_pending_grab` que já vive ao lado.
        //
        // ⛔ **As leis de MALHA continuam imediatas, e é deliberado:** elas repõem
        // a pose congelada e reaplicam, logo são **idempotentes** no mesmo `x` —
        // um evento a mais é trabalho repetido, nunca uma resposta diferente.
        if self.filter_law.is_cloth() {
            self.tecido.pending = Some(x);
            return;
        }
        let amount = (x - self.filter_from_x) * FILTER_DRAG_PER_PX;
        // ⚠️ **O pincel é o AUTORADO, e não o [`Sculpt3dScene::armed_brush`]**:
        // aquele resolve o raio contra o ACERTO e carimba o estêncil da vista, e
        // um filtro não tem cursor nem pegada. O que a lei lê daqui é o verbo, a
        // referência e os dois `hc_*` — todos autorados —, então pedir um pincel
        // ancorado num ponto seria inventar o ponto que o gesto não tem.
        // ⚠️ **A LEI é a ESCOLHIDA, nunca a do verbo em mãos** — o selector do
        // painel é a autoridade, e o verbo apenas a semeou no
        // [`Self::arm_filter`]. Ler o verbo aqui reinstalaria a derivação e
        // tornaria as três leis sem verbo (`Scale`, `Sphere`, `Random`)
        // inalcançáveis outra vez, com o chip aceso a mentir sobre qual delas
        // corre.
        let law = self.filter_law;
        // ⚠️ **Só a lei de MALHA o lê** — desde 2026-09-08 o filtro de TECIDO
        // recebe as propriedades DELE e não vê o pincel (ver
        // `ph2d_sculpt3d::ClothFilterProps`).
        let brush = self.brush.clone();
        // ⚠️ **O referencial é resolvido AQUI porque é aqui que a câmera existe**
        // — nem a `ph2d-sculpt3d` nem a `ph2d-cloth` sabem o que é uma vista, e
        // a espec §7 põe o caso especial (na orientação *View* o «baixo» da
        // gravidade é o eixo do ECRÃ, não a profundidade) exactamente do lado de
        // quem tem a matriz.
        let passo = self.cloth_filter_step_of(amount);
        let mesh = self.objects[self.active].stack.mesh_mut();
        let moved = match law {
            ph2d_sculpt3d::FilterLaw::Mesh(kind) => self.stroke.filter(mesh, &brush, kind, amount),
            ph2d_sculpt3d::FilterLaw::Cloth(kind) => {
                self.stroke.cloth_filter_step(mesh, kind, &passo)
            }
        };
        if moved == 0 {
            return;
        }
        let touched = self.stroke.touched().to_vec();
        Self::mesh_changed(
            &mut self.objects[self.active].dirty,
            &mut self.edits,
            &touched,
        );
    }
}

/// ⭐⭐⭐ **O REFERENCIAL E O «BAIXO» DE UM PASSO DO FILTRO** — lei pura, sem
/// cena e sem device, para que um gate a possa dirigir.
///
/// ⛔⛔⛔ **REPORT DO ENIO, 2026-09-08:** *«parece que a gravidade está em z mas
/// neste app deve ser em y»*. Ele tem razão, e o defeito é de PROVENIÊNCIA: o
/// `[0, 0, −1]` que estava escrito aqui é a convenção do **alvo** da espec §7
/// (que é `Z` para cima), e esta casa é **`Y` para cima** — a
/// [`ph2d_mesh_render::Camera3d`] roda o `yaw` em torno do `+Y` e o
/// `field3d_navball` já o diz por escrito para o módulo vizinho.
///
/// ⚠️ **A cura NÃO é trocar um literal por outro** — é o «baixo» passar a ser
/// **derivado** do cima da câmera ([`ph2d_mesh_render::Camera3d::UP`]). Um
/// segundo literal, mesmo certo hoje, é a segunda resposta à pergunta *«para
/// que lado é baixo?»*, e seria ela a envelhecer.
///
/// ⚠️⚠️ **O braço da VISTA já estava certo, e é por isso que o report diz
/// «parece»:** ali o baixo é o `−cima do ECRÃ`, que num enquadramento típico
/// aponta mesmo para baixo na tela. *Metade de uma lei correcta esconde a
/// outra metade errada — o defeito só aparece na orientação que ninguém
/// escolhe primeiro.*
///
/// `ecra` são os três eixos do ECRÃ em coordenadas de mundo — direita, cima e
/// para-o-olho —, que é o que as linhas da matriz de vista dão. ⚠️ **Ela recebe
/// os eixos e não a câmera**: assim a lei não depende de `glam` nem de um
/// `Device`, e um gate dirige-a com três vectores escritos à mão.
fn referencial_e_gravidade(
    orientation: ClothFilterOrientation,
    ecra: [[f32; 3]; 3],
) -> ([[f32; 3]; 3], [f32; 3]) {
    match orientation {
        ClothFilterOrientation::View => {
            let up = ecra[1];
            (ecra, [-up[0], -up[1], -up[2]])
        }
        // *Local* — e o *World* coincide com ele nesta casa, medido
        // ([`ClothFilterOrientation::offered`]).
        _ => {
            let up = ph2d_mesh_render::Camera3d::UP;
            let right = [1.0, 0.0, 0.0];
            let back = [0.0, 0.0, 1.0];
            ([right, [up.x, up.y, up.z], back], [-up.x, -up.y, -up.z])
        }
    }
}

#[cfg(test)]
#[path = "sculpt3d_filter_tests.rs"]
mod tests;

/// **Os gates do FILTRO DE TECIDO**, irmãos dos de cima pelo tecto de LOC — e o
/// corte é por sujeito, não por tamanho. Ver o cabeçalho deles.
#[cfg(test)]
#[path = "sculpt3d_filter_cloth_tests.rs"]
mod cloth_tests;
