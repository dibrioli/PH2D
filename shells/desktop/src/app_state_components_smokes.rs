//! **Os latches das cenas de smoke da família das INSTÂNCIAS** — irmão de [`super`] por tecto de
//! LOC (HR-18).
//!
//! ⚠️ **O corte é por RESPONSABILIDADE:** o pai declara o que a `App` É; isto é o estado de UM
//! assunto dentro dela.
//!
//! ⚠️ **Eles ficam na SHELL e não na crate da família**, e a razão está escrita no
//! `components_scenes`: um latch dentro dela seria a família a ter opinião sobre **quando** o
//! quadro a chama.

/// ⭐⭐⭐ **O estado da família das INSTÂNCIAS do lado da SHELL** — os latches das cenas **e** o que
/// o HUD precisa de lembrar entre quadros.
///
/// ⚠️⚠️ **Porque é UMA struct e não campos soltos na `App`:** a catraca
/// `the_app_only_sheds_fields` está **no número exacto de hoje**, logo *qualquer* campo novo a
/// reprova — e a mensagem dela diz a cura por escrito: *«um campo novo tem DONO: ponha-o no estado
/// da família do assunto dele»*. O HUD (TOP-20 #20) é desta família, e por isso o estado dele entra
/// AQUI em vez de somar dois campos ao topo.
#[derive(Default)]
pub(crate) struct ComponentsShell {
    /// Os latches das cenas de smoke.
    pub(crate) smokes: ComponentsSmokeLatches,
    /// O estado vivo do HUD.
    pub(crate) hud: HudShell,
    /// ⭐ O que a cena da CUTSCENE mede sobre si mesma — ver [`SequenceShell`].
    pub(crate) sequence: SequenceShell,
    /// ⭐ O que a cena da VIGIA mede sobre si mesma — ver [`CounterWatchShell`].
    pub(crate) counter_watch: CounterWatchShell,
}

/// ⭐⭐⭐ **A AUTO-CONFERÊNCIA da cena da cutscene** (TOP-20 #19) — o que ela mede sobre si mesma
/// enquanto corre.
///
/// ⚠️ **Ela existe porque a FOTO não decide esta pergunta:** uma imagem parada mostra a porta num
/// instante, e *«ela mexeu-se?»* é uma propriedade de um INTERVALO. Sem isto, a única prova do
/// lado pintado seria comparar duas fotos à mão — que é exactamente o tipo de verificação que
/// envelhece sem ninguém reparar.
#[derive(Default)]
pub(crate) struct SequenceShell {
    /// Quantos quadros ainda faltam amostrar. `0` = a conferência já saiu.
    pub(crate) resta: u32,
    /// A entidade que TEM o componente, e a que não tem.
    pub(crate) porta: u64,
    /// Ver [`Self::porta`].
    pub(crate) controlo: u64,
    /// A excursão vertical de cada uma, em unidades de mundo: `(min, max)`.
    pub(crate) faixa_porta: (f32, f32),
    /// Ver [`Self::faixa_porta`].
    pub(crate) faixa_controlo: (f32, f32),
}

/// ⭐⭐⭐ **A AUTO-CONFERÊNCIA da cena da vigia** — o que ela mede sobre si mesma enquanto corre.
///
/// ⚠️ **Ela existe pela mesma razão da irmã** ([`SequenceShell`]): uma foto mostra um instante, e
/// *«sumiu ao chegar a zero»* é uma propriedade de um INTERVALO.
#[derive(Default)]
pub(crate) struct CounterWatchShell {
    /// Quantos quadros ainda faltam amostrar. `0` = a conferência já saiu.
    pub(crate) resta: u32,
    /// A entidade que TEM a vigia, e a que não tem.
    pub(crate) heroi: u64,
    /// Ver [`Self::heroi`].
    pub(crate) controlo: u64,
    /// Ele ficou invisível em algum quadro?
    pub(crate) heroi_sumiu: bool,
    /// Ver [`Self::heroi_sumiu`] — ⚠️ **tem de ficar `false`**, e é essa metade que faz da outra
    /// uma prova.
    pub(crate) controlo_sumiu: bool,
    /// ⭐ Quantas LUZES de vida se apagaram de cada lado — a metade que o dono de facto VÊ.
    pub(crate) luzes_heroi: usize,
    /// Ver [`Self::luzes_heroi`] — ⚠️ **tem de ficar `0`**.
    pub(crate) luzes_controlo: usize,
    /// O valor mais baixo que o contador de cada um atingiu.
    pub(crate) min_heroi: i64,
    /// Ver [`Self::min_heroi`].
    pub(crate) min_controlo: i64,
}

/// ⭐ **O que o HUD lembra entre quadros** — nada disto é documento.
#[derive(Default)]
pub(crate) struct HudShell {
    /// **O botão em que o dedo POUSOU** (TOP-20 #20) — a memória de um gesto, e a razão de ela
    /// existir é a lei do oráculo: um botão dispara ao LARGAR, e só se o largar cair no MESMO
    /// botão em que se carregou.
    pub(crate) press: Option<ph2d_ecs::Entity>,
    /// A última vista já impressa pelo `PH2D_HUD_LOG` — para a linha sair **uma vez por mudança**
    /// em vez de sessenta vezes por segundo.
    pub(crate) log: Option<String>,
}

/// ⭐⭐ **Os latches das cenas de smoke da família das INSTÂNCIAS** — um por roteador.
///
/// ⚠️ **Uma struct e não cinco campos soltos** (2026-09-14): a catraca `the_app_only_sheds_fields`
/// diz, no próprio texto da falha, que *«um campo novo tem DONO: ponha-o no estado da família do
/// assunto dele»* — e o dono destes é o assunto, não a `App`. ⛔ Subir o número é a última saída.
///
/// ⚠️ **O `game_camera` é lido FORA do prólogo** (`render_loop::fase_game_camera` decide mover o
/// herói da cena), e é por isso que ele não podia ser um `thread_local` da crate — ao contrário do
/// que os outros quatro poderiam ser, se a lei do latch não fosse a que é.
#[derive(Default)]
pub(crate) struct ComponentsSmokeLatches {
    /// ⭐⭐⭐ **O HUD** (TOP-20 #20) — `PH2D_HUD_SMOKE=1`. ⚠️ **`u8` e não `bool`**, sozinho entre
    /// os irmãos: esta cena tem TRÊS estados (por montar · formas montadas · componentes vestidos),
    /// porque o `sync` do render loop só dá entidade a um `VecPath` no quadro SEGUINTE — e sem
    /// entidade não há onde pendurar um `UiLabel`.
    pub(crate) hud: u8,
    /// Quantos quadros ainda trazem o Inspector à frente na cena do HUD.
    pub(crate) hud_raise: u8,
    /// ⭐⭐⭐ O smoke do `Timer` (TOP-20 #2). `PH2D_TIMER_SMOKE=1`.
    pub(crate) timer: bool,
    /// ⭐ A cena do `SignalActions` (TOP-20 #5).
    pub(crate) signal_action: bool,
    /// ⭐ A cena do SOM DE CENA (TOP-20 #4).
    pub(crate) audio_2d: bool,
    /// ⭐ A cena da CÂMERA DE JOGO (TOP-20 #7).
    pub(crate) game_camera: bool,
    /// ⭐ As duas cenas das TAGS (TOP-20 #9). `PH2D_TAGS_SMOKE=1|2`.
    pub(crate) tags: bool,
    /// ⛔⛔ **Quantos quadros falta ainda trazer o Inspector à frente na cena das TAGS** — o irmão
    /// exacto do [`Self::particles_raise`], e pela mesma razão medida. ⚠️ Ele existe porque o smoke
    /// daquela cena mandava ler uma secção do Inspector **sobre um ecrã sem objecto escolhido**
    /// (report do dono, 2026-09-19): sem selecção não há um único chip, e sem a subida o painel que
    /// o `~/.ph2d/layout.txt` deixou aberto fica por cima.
    pub(crate) tags_raise: u8,
    /// ⭐ A FÁBRICA e o CICLO DE VIDA (TOP-20 #11 e #12) — `PH2D_FACTORY_SMOKE`.
    pub(crate) factory: bool,
    /// ⭐ O MOVER DE VISTA DE CIMA (TOP-20 #13) — `PH2D_TOPDOWN_SMOKE=1|2`.
    pub(crate) topdown: bool,
    /// ⭐ O CÉREBRO AUTORÁVEL (TOP-20 #15) — `PH2D_STATEMACHINE_SMOKE`.
    pub(crate) statemachine: bool,
    /// ⭐ O SCRIPT DO ARTISTA (TOP-20 #16) — `PH2D_SCRIPT_SMOKE`.
    pub(crate) script: bool,
    /// ⭐ O PROJÉCTIL (TOP-20 #14) — `PH2D_PROJECTILE_SMOKE=1|2`.
    pub(crate) projectile: bool,
    /// ⭐ O EMISSOR DE PARTÍCULAS (TOP-20 #18) — `PH2D_PARTICLES_SMOKE=1|2`.
    pub(crate) particles: bool,
    /// ⛔⛔ **Quantos quadros falta ainda trazer o Inspector à frente** — e ele não é um contador
    /// defensivo, é a cura de uma ordem MEDIDA numa foto: o `reconcile_z` acrescenta, no início de
    /// cada quadro, os painéis que ainda não estão na ordem z, logo um `bump` feito no quadro em
    /// que a cena monta fica **por baixo** dos que chegam a seguir. ⚠️ Ele PÁRA — passados estes
    /// quadros a aba é do dono, e uma subida por quadro roubar-lhe-ia o painel que ele escolhesse.
    pub(crate) particles_raise: u8,
    /// ⭐ A CUTSCENE (TOP-20 #19) — `PH2D_SEQUENCE_SMOKE=1`.
    pub(crate) sequence: bool,
    /// ⭐ A VIGIA DO CONTADOR — `PH2D_COUNTERWATCH_SMOKE=1`.
    pub(crate) counter_watch: bool,
    /// ⭐ O GATILHO (suplente #24) — `PH2D_TRIGGER_SMOKE=1`.
    pub(crate) trigger: bool,
    /// ⭐ O GOLPE (suplente #24) — `PH2D_DANO_SMOKE=1`.
    pub(crate) dano: bool,
    /// Quantos quadros ainda trazem o Inspector à frente na cena da vigia — ver `sequence_raise`.
    pub(crate) counter_watch_raise: u8,
    /// Quantos quadros ainda trazem o Inspector à frente na cena da cutscene — ver o irmão
    /// `particles_raise`, que é a mesma cura da mesma ordem medida.
    pub(crate) sequence_raise: u8,
    /// O ragdoll instanciado 3× (ADR-0164 F4). `PH2D_INSTANCE_SMOKE=1..7`.
    pub(crate) instance: bool,
}
