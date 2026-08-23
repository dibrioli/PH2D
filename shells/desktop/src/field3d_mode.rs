//! ⭐ **QUEM É DONO DO CANVAS** — e o modo de modelagem a ceder quando outro o toma (W40).
//!
//! # O defeito, com as palavras do Enio
//!
//! *"o modo Modelagem nunca é desativado e não consigo usar nenhum outro modo do app. Se eu entro
//! no modo sculpt ou vector ou qualquer outro, o Modelagem deve ceder. Não consigo esculpir nada
//! pois o modo de modelagem permanece interferindo."* (2026-08-22)
//!
//! # ⚠️ É o MESMO report de 2026-08-17, um nível acima
//!
//! O módulo de escultura já pagou este exato defeito duas vezes, e as duas estão escritas no
//! `input_dispatch`:
//!
//! - *"não consigo configurar a textura da sprite já que não posso sair do modo escultura"* (09/08)
//! - *"depois de abrir outros módulos como Sculpt, o Motion não consegue usar os atalhos"* (17/08),
//!   cuja lição foi: **o ponteiro já cedia, o teclado não** — *"uma assimetria entre duas portas que
//!   respondem à MESMA pergunta"*.
//!
//! Aqui a assimetria é maior: **nenhuma das duas cede.** O módulo de modelagem é armado pela
//! visibilidade do painel (`set_armed_by_panel`), e nada no app fecha esse painel. Enquanto ele
//! estiver aberto, o traçado desenha por cima do canvas e o ponteiro é dele.
//!
//! # ⭐ A lei: tomar o canvas LIBERTA quem o tinha
//!
//! Não é *"o modelador desliga-se sozinho"* — é que o canvas tem **um** dono, e pegar nele é um
//! gesto que solta os outros. Duas metades simétricas:
//!
//! | quem entra | quem cede |
//! |---|---|
//! | uma **ferramenta** é pegada no rail, ou o **barro** aparece na tela | o painel MODEL **fecha** |
//! | o pill **MODEL** é aberto | o **barro** sai da tela (pela porta do próprio módulo de escultura) |
//!
//! ⚠️ **Fechar o painel, e não só desarmar em silêncio:** o pill *é* o interruptor do módulo
//! (`set_armed_by_panel`), então um desarme invisível deixaria o botão aceso a mentir sobre o
//! estado. *O artista tem de ver por que o modelador saiu.*
//!
//! # ⚠️ Por que a lei é de BORDA e não contínua
//!
//! Uma regra contínua — *"MODEL cede enquanto houver ferramenta em mãos"* — é mais simples e cria um
//! impasse: uma ferramenta pegada **fica** em mãos (o `set_active` no mesmo id é no-op, e não há
//! gesto de largar), então o modelador nunca mais abriria. A borda diz o que o Enio pediu (*"se eu
//! entro noutro modo"*) sem tirar o caminho de volta.

use ph2d_editor::ToolId;

/// **Quem tem o canvas neste quadro.**
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Owner {
    /// A ferramenta em mãos, se houver.
    pub(crate) tool: Option<ToolId>,
    /// O barro da escultura está na tela?
    pub(crate) clay: bool,
}

/// ⭐ **Alguém TOMOU o canvas entre estes dois quadros?**
///
/// ⚠️ Função pura, e é ela que carrega a decisão inteira — por isso é gateável sem janela nenhuma.
///
/// | transição | tomou? | porquê |
/// |---|---|---|
/// | ferramenta `None` → `Some(x)` | **sim** | pegou-se uma ferramenta |
/// | ferramenta `Some(x)` → `Some(y)` | **sim** | trocou-se de ferramenta, e a nova quer o canvas |
/// | ferramenta `Some(x)` → `Some(x)` | não | nada aconteceu; ceder aqui seria fechar o painel a cada quadro |
/// | ferramenta `Some(x)` → `None` | não | **largar** não é tomar |
/// | barro `false` → `true` | **sim** | entrou-se no modo escultura |
/// | barro `true` → `true` | não | mesma razão da ferramenta inalterada |
pub(crate) fn took_the_canvas(prev: &Owner, now: &Owner) -> bool {
    let tool_taken = now.tool.is_some() && now.tool != prev.tool;
    let clay_entered = now.clay && !prev.clay;
    tool_taken || clay_entered
}

thread_local! {
    /// O dono do canvas no quadro anterior — a metade que transforma um estado numa BORDA.
    static LAST: std::cell::RefCell<Owner> = const { std::cell::RefCell::new(Owner {
        tool: None,
        clay: false,
    }) };
}

/// **Compara com o quadro anterior e devolve se alguém tomou o canvas**, guardando o novo dono.
pub(crate) fn note_owner(now: Owner) -> bool {
    LAST.with(|last| {
        let took = took_the_canvas(&last.borrow(), &now);
        *last.borrow_mut() = now;
        took
    })
}

/// ⭐ **O painel MODEL acabou de ABRIR?** — a outra borda, e a metade simétrica da lei.
///
/// ⚠️ Ela é separada do [`note_owner`] de propósito: são **duas** transições independentes no mesmo
/// quadro (alguém tomou o canvas · o modelador tomou-o), e juntá-las num estado só faria uma
/// mascarar a outra no quadro em que ambas acontecem.
pub(crate) fn model_just_opened(open: bool) -> bool {
    MODEL_WAS_OPEN.with(|was| !was.replace(open) && open)
}

thread_local! {
    static MODEL_WAS_OPEN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// ⚠️ Só para gates: esquece o dono anterior, para que dois gates no mesmo processo não se
/// contaminem pela ordem em que correram.
#[cfg(test)]
pub(crate) fn forget_owner() {
    LAST.with(|last| *last.borrow_mut() = Owner::default());
    MODEL_WAS_OPEN.with(|w| w.set(false));
}

#[cfg(test)]
#[path = "field3d_mode_tests.rs"]
mod tests;
