//! §12 Physics Joint + §13 Pulley Wheel — os snapshots e os enums de edição
//! **do objeto-joint** (W3/W8/W-Pulley, extraído em W-JointCopy).
//!
//! Irmão do `inspector_model_physics`, que ficou com a §11 (o CORPO), separado
//! dele quando o copy/paste passou o cap de 700 LOC. O corte é o mesmo que o
//! painel já desenha em `sections/physics.rs` × `sections/joint.rs` ×
//! `sections/wheel.rs`, e o mesmo que a §12 desenha na tela: *o que este corpo
//! É* × *entre quais dois isto está, e o que a restrição faz*.
//!
//! Re-exportado por `screens::hero` junto com o irmão, então o caminho de
//! importação de todo consumidor é o mesmo de antes.
//!
//! Primitivos puros (tags + floats + `String`) — o `editor-core` segue
//! desacoplado do `ph2d-ecs` e da crate de física, e a shell mapeia tags ↔ enums
//! na fronteira.

/// §12 Physics Joint snapshot (W3) — the selected **joint object**.
///
/// A joint is an entity, so this is the section that describes it: what kind
/// of constraint it is, which two bodies it names, and the parameters that
/// kind actually uses. Not `Copy`, unlike its siblings, because it carries the
/// two bodies' NAMES — the joint stores name hashes, and a hash is not
/// something to show a person.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorJointInfo {
    pub entity_bits: u64,
    /// `0` Pin · `1` Spring · `2` Rope · `3` Weld · `4` Slider · `5` Rod ·
    /// `6` Wheel · `7` Pulley — a ordem de `JointKind::ALL`, que é a ordem dos
    /// chips que a §12 pinta.
    pub kind_tag: u8,
    /// The bodies, resolved for display. Empty means the name no longer
    /// matches any body in the scene — deleted or renamed.
    pub body_a_name: String,
    pub body_b_name: String,
    /// Are BOTH bodies present right now? The section says so out loud: a
    /// joint whose body was renamed is dormant, not broken, and silently
    /// showing its parameters as if it were live would be a lie.
    ///
    /// ⚠️ **Num pino de MUNDO isto é verdade com UM corpo só** (W-JointWorld) —
    /// o cenário não é um objeto que possa estar ausente. Perguntá-lo pelos DOIS
    /// nomes chamaria de quebrado um joint que está segurando.
    pub bound: bool,
    /// **O lado B é o MUNDO** — um ponto do cenário, não um objeto
    /// (`JointWorldAnchor`). A row *Body B* diz *World* e não oferece
    /// conta-gotas: não há o que apontar, e um ícone apagado que ainda despacha é
    /// exatamente o que esta linha proíbe.
    pub world_anchored: bool,
    pub limits_enabled: bool,
    /// The limit range **in the KIND's own unit** — degrees for a Pin's angular
    /// range, metres for a Slider's stroke.
    ///
    /// ⚠️ Named `_ui` and not `_deg` because it is not always degrees, and an
    /// identifier that promises one unit while carrying two is the same defect as
    /// a label that does (the field was `limit_min_deg` until the Slider arrived).
    /// The component stores radians for a Pin — converted at this boundary, as
    /// `rotation_rad` is — and metres for a Slider, converted not at all. The
    /// single door is `JointKind::limits_in_metres`.
    pub limit_min_ui: f32,
    pub limit_max_ui: f32,
    pub motor_enabled: bool,
    /// `0` Velocity · `1` Position — what the motor is aiming at (W-J6).
    pub motor_mode_tag: u8,
    /// The motor's target RATE, **in the kind's own unit**: degrees per second
    /// on a hinge, metres per second on a rail or a winch.
    ///
    /// ⚠️ Named `_ui` and not `_deg` for the reason [`Self::limit_min_ui`] gives
    /// at length — an identifier that promises one unit while carrying two is
    /// the same defect as a label that does. The door is
    /// `JointKind::motor_in_metres`, which is deliberately NOT the same question
    /// as `limits_in_metres`: a Rope has no limits and still has a linear motor.
    pub motor_speed_ui: f32,
    /// The servo's target PLACE, likewise in the kind's own unit — degrees on a
    /// hinge, metres on a rail or a winch.
    pub motor_target_ui: f32,
    pub motor_max_force: f32,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub max_length: f32,
    /// Which body slot has an ARMED canvas pick right now: `0` none, `1` A,
    /// `2` B. The §12 draws that slot's eyedropper pressed (accent) so the
    /// artist sees the picker is waiting for a click on a body. The shell owns
    /// the armed state (`App.joint_body_pick`); this mirrors it for the paint.
    pub pick_armed: u8,
    /// **Quantas roldanas esta corda atravessa** (W-Pulley W1).
    ///
    /// A §12 o mostra ao lado do botão que acrescenta uma — sem ele o artista
    /// clicaria e não teria como saber que algo aconteceu, porque a roldana nova
    /// nasce SOBRE a corda (para não dar um puxão) e o desenho quase não muda.
    pub wheel_count: u32,
    /// Can this joint be torn apart? (W-J7.) Gates the two thresholds below —
    /// the `∞ = off` of P7, expressed as the checkbox the section already uses
    /// for limits and for the motor.
    pub break_enabled: bool,
    /// The linear reaction, **newtons**, above which it gives way. No unit
    /// conversion at this boundary: a newton is a newton in both worlds, unlike
    /// the angle rows above.
    pub break_force: f32,
    /// The angular reaction, **newton-metres**.
    pub break_torque: f32,
    /// Does the torque threshold have any chance of firing on this kind?
    ///
    /// ⚠️ Only a Pin does: rapier reports the reaction of a limited or motorised
    /// angular axis and NOTHING for a locked one (measured — a Weld cantilever
    /// holds 4.905 N·m and reads `0.0000`). The section paints the torque row
    /// only when this is true, because a threshold that can never be reached is
    /// a control in name only.
    pub breaks_on_torque: bool,
    /// **Is the constraint in force?** (W-J8.) Off leaves the object, its
    /// parameters and its anchors exactly where they are and stops it holding —
    /// the thing Delete cannot do.
    pub active: bool,
    /// **Esta solda CEDE?** (W-SoftWeld — `Weld` apenas.)
    ///
    /// Marcada, o ângulo vira mola e a seção passa a oferecer Stiffness/Damping —
    /// os MESMOS dois campos e os MESMOS dois ids que a Spring e o Wheel usam,
    /// porque é a mesma coisa física noutro eixo.
    pub soft: bool,
    /// **Do the two jointed bodies still collide with each other?** (W-J8.)
    /// Default off, and the default is measured: a chain link overlaps its
    /// neighbour at the pin by construction.
    pub collide_connected: bool,
    /// **Quantos joints um Paste atingiria agora** — `0` = a área de
    /// transferência está vazia (W-JointCopy).
    ///
    /// Um número só, porque as duas perguntas colapsam: a §12 só existe com um
    /// joint selecionado, logo *há o que colar* implica *há pelo menos um alvo*.
    /// `0` some com o botão (um Paste vazio não muda nada na tela e leria como
    /// *"o paste está quebrado"*); `> 1` entra no RÓTULO, porque o fan-out é o
    /// que o gesto tem de valioso e um clique que toca dez objetos tem de dizer
    /// isso ANTES — a mesma lei do `Bake 5.0s to Timeline` e do
    /// `Add Wheel (N on this rope)`.
    pub paste_targets: usize,
}

/// A single editable §12 joint field, dispatched as
/// [`EditorAction::InspectorJointEdit`](crate::action_bus::EditorAction).
///
/// Angles arrive in **degrees** and the shell converts, so the panel never
/// holds a radian and the component never holds a degree.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum JointFieldEdit {
    /// `JointKind` tag — a ordem de `JointKind::ALL`, a mesma de
    /// [`InspectorJointInfo::kind_tag`].
    Kind(u8),
    LimitsEnabled(bool),
    /// The limit range in the kind's own unit — see [`InspectorJointInfo::limit_min_ui`].
    LimitMin(f32),
    LimitMax(f32),
    MotorEnabled(bool),
    /// `0` Velocity · `1` Position.
    MotorMode(u8),
    /// The target rate, in the kind's own unit — see
    /// [`InspectorJointInfo::motor_speed_ui`].
    MotorSpeed(f32),
    /// The servo's target place, in the kind's own unit.
    MotorTarget(f32),
    MotorMaxForce(f32),
    RestLength(f32),
    Stiffness(f32),
    Damping(f32),
    MaxLength(f32),
    BreakEnabled(bool),
    /// Newtons — no conversion (W-J7).
    BreakForce(f32),
    /// Newton-metres.
    BreakTorque(f32),
    /// ARM a canvas pick for slot A (§12 eyedropper): the next click on a body
    /// re-binds this end. Carries no operand — the shell holds the armed state
    /// and the body is chosen by the click, with nothing pre-selected.
    PickBodyA,
    /// ARM a canvas pick for slot B — the sibling of
    /// [`PickBodyA`](JointFieldEdit::PickBodyA).
    PickBodyB,
    /// **Is this joint in force?** (W-J8.) The authored twin of a break: both
    /// write the same rapier flag, and only this one rides in the descriptor —
    /// so a Reset brings an inactive joint back inactive and a broken one back
    /// holding.
    /// A solda CEDE? (W-SoftWeld.) Só um `Weld` a oferece; a ponte pergunta a
    /// `JointKind::can_be_soft` antes de entregar ao solver, então um `soft`
    /// deixado por uma troca de tipo não segue em vigor.
    Soft(bool),
    Active(bool),
    /// **Do the two jointed bodies collide with each other?** (W-J8.)
    CollideConnected(bool),
    /// **Copiar as propriedades deste joint** para a área de transferência da
    /// shell (W-JointCopy). Como os conta-gotas, não carrega operando e não
    /// escreve componente nenhum: quem guarda é a shell.
    CopyProperties,
    /// **Colar as propriedades copiadas neste joint.** Sem operando pela mesma
    /// razão — a fonte é a área de transferência da shell, e pô-la aqui faria
    /// esta crate depender de `ph2d-physics-ecs`, que é exatamente o
    /// acoplamento que o resto da §12 evita falando em tags e floats.
    ///
    /// ⚠️ **A única edição da §12 que FAZ FAN-OUT** sobre a seleção. As duas
    /// irmãs estruturais recusam o fan-out por motivos que não valem aqui: um
    /// `Join` espalhado criaria N joints entre o mesmo par, e um `Bake`
    /// espalhado re-simularia a cena inteira N vezes pelos MESMOS números. Um
    /// paste espalhado faz exatamente o que o artista pediu, e sem ele o gesto
    /// é *digitar quinze campos, dez vezes*.
    PasteProperties,
    /// **Exchange the two ends.** Behaviour-preserving by construction
    /// (`PhysicsJoint::swapped`): the anchors travel with their bodies and every
    /// signed quantity measured between them is negated, so the joint does
    /// exactly what it did and only the labelling changes.
    Swap,
    /// **Acrescentar uma roldana a esta corda** (W-Pulley W1) — o pedido (4) do
    /// artista. Estrutural, como o `Remove`: a shell SPAWNA um objeto, e o undo
    /// global por-diff o captura como captura qualquer outro.
    AddWheel,
    /// **Prender o lado B ao MUNDO (`true`) ou a um objeto (`false`)**
    /// (W-JointWorld).
    ///
    /// ⚠️ Estrutural como o `AddWheel`: acrescenta/remove o marcador
    /// `JointWorldAnchor`, e por isso NÃO passa pelo funil de campos que os
    /// outros variants usam — não há campo de `PhysicsJoint` a escrever.
    AnchorToWorld(bool),
    /// Delete the joint object.
    Remove,
}

/// §13 Pulley Wheel snapshot (W-Pulley W1) — a **roldana** selecionada.
///
/// Seção própria porque uma roldana é uma ENTIDADE: ela é o objeto selecionado
/// quando estas rows importam, e a §12 ao lado só existe com a CORDA
/// selecionada. Não é `Copy`, como a irmã §12 e pelo mesmo motivo: carrega o
/// NOME da corda, e o componente guarda um hash — que não é coisa de mostrar a
/// uma pessoa.
///
/// ⚠️ **A posição não está aqui.** O centro da roldana É o
/// `Transform.translation`, que a §2 já pinta para toda entidade — uma row de
/// posição aqui seria a segunda porta para o mesmo fato.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorWheelInfo {
    pub entity_bits: u64,
    /// A corda a que ela pertence, resolvida para exibição. Vazio = o nome não
    /// casa com nenhuma corda na cena (apagada ou renomeada).
    pub rope_name: String,
    /// Essa corda existe agora? A seção o diz em voz alta: uma roldana órfã é
    /// inerte, não quebrada, e mostrar os parâmetros dela como se ela estivesse
    /// na rota seria mentira — a mesma escolha do `bound` da §12.
    pub bound: bool,
    /// O raio, metros. A alça do aro no canvas edita o MESMO campo.
    pub radius: f32,
    /// **O segundo diâmetro** (W4), metros — `0` é uma roldana comum.
    ///
    /// Com ele a roldana vira um **tambor diferencial** e a corda ganha vantagem
    /// mecânica `radius / radius_out`. O número não é digitado: ele CAI das duas
    /// circunferências, e as duas são desenhadas.
    pub radius_out: f32,
    /// **Este eixo de dois diâmetros é uma talha de WESTON?** (W-Weston.)
    ///
    /// `false` é o **TAMBOR**: a corda troca de diâmetro no MESMO nó, e a vantagem
    /// a jusante é `R/r`. `true` faz os dois contatos ABRAÇAREM o resto da rota —
    /// a corda sai pelo grande, dá a volta no que houver no meio, e volta pelo
    /// pequeno —, e o quociente passa a ser `R/(R−r)`.
    ///
    /// Só significa algo com [`Self::radius_out`] positivo, e é isso que a row
    /// pergunta antes de se oferecer.
    pub weston: bool,
    /// **O que as duas circunferências COMPRARAM** — o quociente que o solver usa,
    /// `R/r` num tambor e `R/(R−r)` numa Weston; `1` sem segundo diâmetro.
    ///
    /// ⚠️ **Vem da porta do MOTOR** (`rope_route::weston_gear` /
    /// `RopeWheel::gear`), nunca de uma conta escrita no painel: um readout com
    /// aritmética própria mostraria um número e o solver usaria outro, que é
    /// exatamente o defeito do `ratio` DIGITADO que o W4 aposentou.
    ///
    /// Ele existe porque a medição do `crossing_gear` **recusou um teto**: a
    /// máquina degrada para *não se move* em vez de explodir, e o remédio honesto
    /// para dois diâmetros quase iguais é o artista VER que desenhou 512:1 — não um
    /// cap que contradiz o desenho.
    pub gear: f32,
    /// A posição ao longo da corda, **1-based** — 1º nó, 2º nó, …
    ///
    /// ⚠️ Sufixo `_ui` pela razão que o `limit_min_ui` da §12 dá por extenso: o
    /// componente conta de zero e a pessoa conta de um, e um identificador que
    /// promete uma convenção carregando outra é o mesmo defeito de um rótulo que
    /// faz isso. A conversão mora nas duas fronteiras, uma vez cada.
    pub order_ui: u32,
    /// `0` Auto · `1` Over · `2` Under — a ordem de `WrapSide::ALL`, que é a
    /// ordem dos chips que a §13 pinta.
    pub wrap_tag: u8,
    /// A velocidade do tambor **em GRAUS por segundo** — a unidade da row, já
    /// convertida pela shell. `0` é uma roldana livre, e não há um segundo
    /// booleano *"tem motor?"* para discordar do número.
    pub motor_deg_per_s: f32,
    /// **Este eixo pode ceder?** (W2.) Gateia o limiar abaixo — o `∞ = off` do
    /// P7, na forma de checkbox que a §12 já usa para limites, motor e ruptura.
    pub break_enabled: bool,
    /// **O que ele aguenta**, newtons. Sem conversão nesta fronteira: um newton
    /// é um newton nos dois mundos, ao contrário das rows de ângulo.
    pub break_force: f32,
    /// **Em que CORPO o eixo está montado**, resolvido para exibição (W3). Vazio
    /// = roldana pregada no CENÁRIO, que é o que toda roldana é até alguém dizer
    /// o contrário.
    ///
    /// Montada num corpo que se move, ela é a *cadernal móvel* de uma talha, e o
    /// corpo passa a ser sustentado por DOIS ramos da corda.
    pub mount_name: String,
    /// O pick de canvas desta roldana está ARMADO? O eyedropper pinta `Pressed`
    /// enquanto espera o clique no corpo — a mesma máquina do re-pick de corpo do
    /// joint (W-JointAuthoring), agora para o eixo.
    pub mount_pick_armed: bool,
    /// O pick da CORDA está armado? O eyedropper da row Rope pinta `Pressed`
    /// enquanto espera o clique — e o alvo dele é a ROTA desenhada, não um sprite
    /// (a entidade-corda não tem nenhum).
    pub rope_pick_armed: bool,
}

/// Um campo editável da §13, despachado como
/// [`EditorAction::InspectorWheelEdit`](crate::action_bus::EditorAction).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WheelFieldEdit {
    /// Metros.
    Radius(f32),
    /// O SEGUNDO diâmetro, metros (W4). `0` volta a roldana a ser comum.
    RadiusOut(f32),
    /// **Este eixo de dois diâmetros é uma talha de WESTON?** (W-Weston.)
    ///
    /// Ao contrário dos irmãos, ele **não escreve na `PulleyWheel`** — a presença do
    /// marcador `WestonAxle` É o booleano (sem bump de `PROJECT_SCHEMA`), então a
    /// shell o roteia pela porta de anexar/desanexar em vez do funil de campo.
    ///
    /// ⚠️ **E ele MUDA A ROTA**, o que nenhum outro toggle desta seção faz: armá-lo
    /// acrescenta um nó (o contato de retorno) e re-pesa a corda, então o `L0` tem de
    /// ser re-derivado — e o `route_differs`, que compara campos da `PulleyWheel`,
    /// **não consegue ver isso**.
    Weston(bool),
    /// **1-based**, como a row mostra — ver [`InspectorWheelInfo::order_ui`].
    Order(u32),
    /// `0` Auto · `1` Over · `2` Under.
    Wrap(u8),
    /// Graus por segundo, como a row fala — a shell converte para radianos.
    MotorDegPerS(f32),
    /// O switch de ruptura do EIXO.
    BreakEnabled(bool),
    /// O limiar dele, newtons.
    BreakForce(f32),
    /// **ARMA** o pick de canvas do corpo em que esta roldana se monta — sem
    /// operando, como o `PickBodyA` do joint: o clique escolhe o alvo, não este
    /// evento.
    PickMountBody,
    /// **Armar o pick da CORDA** (W1) — sem operando: o alvo vem do clique no
    /// canvas, resolvido pela ROTA. Fecha o item *"a corda de uma roldana é
    /// escolhida só na CRIAÇÃO"*, que deixava uma roldana órfã sem volta a não ser
    /// apagar e refazer.
    PickRope,
    /// **DESMONTA** a roldana: ela volta a ser um ponto do cenário.
    ///
    /// Existe porque montar é um gesto e desmontar tem de ser outro — sem ele a
    /// única saída seria apagar a roldana e refazê-la, que é a mesma queixa que
    /// abriu esta wave (o pedido 4 do artista).
    Unmount,
}
