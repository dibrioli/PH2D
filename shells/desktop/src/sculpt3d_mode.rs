//! **ENTRAR E SAIR do modo escultura** — o que o pill SCULPT faz, e o que ele MOSTRA (ADR-0150).
//!
//! Filho (`#[path]`) de [`super`] para alcançar o papel da forma, que é privado; o corte é *o que
//! a mão faz com o barro* (os irmãos `input`/`keys`) contra *quem é dono da tela* (aqui).
//!
//! ## A pergunta que o pill faz, e a que ele NÃO faz
//!
//! O barro na tela decide **duas** coisas de uma vez — o passe de cor desenha, e o ponteiro é da
//! cena ([`super::donation::FormRole::draws_clay`]). Enquanto ele estiver lá, todo clique no canvas
//! é da escultura, então **o artista não consegue selecionar um sprite** — que é o gesto inteiro da
//! W17 (Enio, 2026-08-09: *"não consigo configurar a textura da sprite já que não posso sair do
//! modo escultura"*). Antes desta wave a única saída era a tecla `D`.
//!
//! O pill é **binário** (*estou esculpindo?*); a forma tem **três** posições (barro / luz /
//! desligada), e quem as percorre continua sendo o `D`. Duas perguntas, dois controles.
//!
//! ⚠️ **E a SAÍDA reusa a ordem que o ciclo já declara** (`role.next()` a partir do barro = a LUZ),
//! em vez de inventar um destino. Escolher `Off` seria o pill ter opinião sobre a OUTRA pergunta —
//! *a forma acende a tinta?* — que ninguém expressou ao clicar num botão escrito "SCULPT"; e é a
//! opinião que DESTRÓI trabalho, porque quem estava na luz a perderia. Sair para a luz é a ordem do
//! trabalho: acabei de esculpir, agora quero ver a tinta acesa pela forma.

use crate::app_state::App;
use crate::sculpt3d::Sculpt3dScene;
use crate::sculpt3d::donation::FormRole;

impl App {
    /// Cumpre o pedido do pill: põe o barro na tela, ou tira-o.
    ///
    /// ⚠️ **Sem cena, ENTRAR CRIA uma** — e é isso que tira o módulo de trás da variável de
    /// ambiente. Sem esta metade o pill seria um botão que não faz nada em todo run normal, que é o
    /// apodrecimento que este codebase recusa por regra; com ela, a escultura passa a ser
    /// alcançável no produto pelo mesmo gesto que a fecha.
    ///
    /// ⚠️ **SAIR nunca destrói a cena.** O papel muda e a escultura fica — largar o `Option` seria
    /// apagar o trabalho do artista num botão cujo nome não promete isso, e o `D` (que faz a mesma
    /// travessia) também não apaga nada.
    pub(crate) fn sculpt3d_apply_toggle(&mut self) {
        if !std::mem::replace(&mut self.sculpt3d_toggle_request, false) {
            return;
        }
        if let Some(scene) = self.gfx.as_mut().and_then(|g| g.sculpt3d.as_mut()) {
            let label = scene.toggle_clay();
            eprintln!("[sculpt3d] modo: {label}");
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            // Sem GPU não há cena a criar. O pedido foi consumido acima de propósito: guardá-lo
            // faria a escultura nascer sozinha no frame em que a janela aparecesse, muito depois
            // do clique que ninguém lembra ter dado.
            return;
        };
        let size = gfx.surface.size();
        let aspect = size.width as f32 / size.height.max(1) as f32;
        let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
        // ⚠️ **A MESMA peça que o verbo de acrescentar cria.** Uma malha própria aqui seria a
        // segunda resposta a *com que forma uma escultura começa*.
        //
        // ⚠️ **E ela já vem ESCULPÍVEL** — o `Primitive::mesh` refina a base pela régua do módulo
        // (`refine_until_sculptable`). A versão anterior desta linha dizia que a peça era *"a
        // primitiva de BLOCAGEM, grossa de propósito — o `K` subdivide"*, e a frase custou o
        // report *"a escultura não funciona"* (Enio, 2026-08-09): na aresta de blocagem um dab do
        // pincel padrão movia **um** vértice em toda resolução. *Blocagem* pressupõe que as formas
        // grandes se deixem esculpir, e elas não se deixavam — a densidade é o que o `K` ajusta a
        // partir de uma peça usável, não o que separa uma peça usável de uma inerte.
        let mesh = crate::sculpt3d::Primitive::Sphere.mesh();
        let scene = Sculpt3dScene::new(&device, mesh, aspect);
        gfx.sculpt3d = Some(scene);
        eprintln!("[sculpt3d] modo: BARRO (esculpir) -- cena nova");
    }
}

impl Sculpt3dScene {
    /// Entra no barro, ou sai dele. Devolve o rótulo do papel novo (o mesmo vocabulário que o `D`
    /// imprime — um nome, uma fonte).
    /// ⚠️ `pub(crate)` desde a W40 do módulo de modelagem, e pela mesma razão que a
    /// [`Self::clay_on_screen`] ao lado já o era: **quem toma o canvas liberta quem o tinha**, e
    /// abrir o pill MODEL tem de tirar o barro da tela. A alternativa era o modelador escrever a
    /// própria saída do modo escultura — uma segunda resposta a *"como se sai do barro"*, e a que
    /// não conhece a ordem do ciclo que o cabeçalho deste arquivo defende.
    pub(crate) fn toggle_clay(&mut self) -> &'static str {
        self.role = if self.role.draws_clay() {
            // Ver o cabeçalho: a saída é a próxima posição do ciclo, não um destino escolhido aqui.
            self.role.next()
        } else {
            FormRole::Clay
        };
        self.role.label()
    }

    /// **O barro está na tela?** — a pergunta que o pill MOSTRA.
    ///
    /// ⚠️ Ela é a mesma do [`Self::shows_clay`], e existe separada só porque aquela é
    /// `pub(super)` do módulo do gesto. Um segundo campo aqui seria o bool que passa a discordar do
    /// `D`.
    pub(crate) fn clay_on_screen(&self) -> bool {
        self.role.draws_clay()
    }

    /// **O barro ACABOU de entrar, ou de sair?** — `Some(entrou)` na borda, `None` entre elas.
    ///
    /// ⚠️ **É uma TESTEMUNHA, não uma enumeração de quem move o papel.** Hoje quem o move são o
    /// pill e a tecla `D`; amanhã pode ser um terceiro. Uma lista de chamadores que avisam o painel
    /// nasce incompleta no dia em que ela cresce — comparar com o valor anterior não.
    ///
    /// ⚠️ **E ela CONSOME a borda** (o nome diz `take`): um segundo leitor receberia `None` e
    /// concluiria que nada mudou. O leitor é o `sculpt3d_panel_bridge`, e é um só.
    pub(crate) fn take_clay_edge(&mut self) -> Option<bool> {
        let now = self.clay_on_screen();
        (now != std::mem::replace(&mut self.clay_was_on, now)).then_some(now)
    }
}

/// **O pill DIZ o que a forma É** — escrito a cada frame, nunca guardado.
///
/// ⚠️ **Por frame, e não no clique.** O `D` move o papel sem passar pelo pill, então um estado
/// escrito só no clique ficaria dizendo *fora* sobre uma cena que está na tela — a falha de duas
/// portas que o `physics_toggle` evita derivando do `panel_visibility`, e que o rádio do rail do
/// Painter evita derivando do modo publicado.
///
/// Sem cena o pill fica solto (`Normal`): não há barro na tela, e é o estado honesto de *entrar*.
pub(crate) fn sync_pill(
    hero: &mut ph2d_editor::screens::hero::HeroScreen,
    scene: Option<&Sculpt3dScene>,
) {
    use ph2d_editor::interaction::InteractiveState;
    use ph2d_editor::widget::ButtonState;
    let on = scene.is_some_and(Sculpt3dScene::clay_on_screen);
    if let Some(InteractiveState::Button { state }) = hero
        .store
        .get_mut(ph2d_editor::screens::hero::ids::TOPBAR_SCULPT3D)
    {
        *state = if on {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
    }
}

#[cfg(test)]
#[path = "sculpt3d_mode_tests.rs"]
mod tests;
