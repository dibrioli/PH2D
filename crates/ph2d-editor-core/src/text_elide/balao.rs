//! ⭐⭐⭐ **O BALÃO DO CORTE — onde uma palavra cortada volta a poder ser LIDA.**
//!
//! # ⛔⛔ O defeito
//!
//! A varredura das elisões mediu, no degrau em que o dono trabalha (as colunas no mínimo),
//! **128 rótulos cortados** — `80` deles no Inspector. Um rótulo cortado não é um defeito de
//! pintura: *é uma palavra que o artista não consegue ler de maneira nenhuma*, porque a coluna
//! não cresce e o nome não cabe. E parte deles nunca pode ser curada por encurtar: o nome de uma
//! âncora, de um objecto, de uma propriedade de script são texto que o **ARTISTA** escreveu.
//!
//! ⇒ **ordem do dono (2026-09-19): «encurtar · balão ao passar o rato»** — as duas metades. Esta
//! é a segunda, e é a que vale para o texto dele.
//!
//! # ⭐⭐⭐ A porta é a LEI DO CORTE, nunca os pintores
//!
//! Este app tem ~40 sítios que pintam um rótulo e **um** sítio que decide que ele não cabe
//! ([`super::elide`]). Pôr o registo nos pintores seria uma lista que alguém esquece; pô-lo na
//! lei faz um pintor novo herdar o balão **por usar a porta**. O que a lei não sabe é *ONDE* o
//! texto vai parar — e isso chega por um ÂMBITO ([`na_area`]), no molde do
//! `HitIndex::push_clip`: quem tem um rect embrulha a pintura nele, e **o neutro é não chamar**.
//!
//! # ⭐⭐ O preço é UMA comparação por corte, e não uma `String`
//!
//! O cabeçalho do [`super::elisao`] recusa, por escrito, registar cada corte **por quadro**: seria
//! uma `String` por rótulo cortado, sessenta vezes por segundo. Esta lista não paga isso porque
//! pergunta **primeiro** se o ponteiro está lá dentro: o caminho do produto gasta um
//! [`Rect::contains`] por corte e aloca **no máximo uma** `String` por quadro — a que está debaixo
//! do rato. *Uma lista que só guarda o que alguém está a olhar não é um censo, é uma resposta.*
//!
//! ⚠️ O censo ([`medindo`]) desliga essa guarda, porque um gate quer TODOS — e é nessa forma que
//! a varredura das elisões prova que todo corte tem balão.

use crate::zones::Rect;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Onde o rato está, publicado uma vez por quadro pelo [`novo_quadro`].
    static PONTEIRO: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
    /// O rect do pintor em curso — `None` fora de um [`na_area`].
    static AREA: Cell<Option<Rect>> = const { Cell::new(None) };
    /// ⚠️ O censo quer **todos** os cortes, não só o que está sob o rato.
    static TUDO: Cell<bool> = const { Cell::new(false) };
    /// O que foi cortado e é alcançável — em ordem de pintura, logo o ÚLTIMO está por cima.
    static ACHADOS: RefCell<Vec<(Rect, String)>> = const { RefCell::new(Vec::new()) };
}

/// ⭐⭐⭐ **Onde o rato está** — escrito pelo despacho do ponteiro, lido por este módulo e por
/// mais ninguém.
///
/// ⛔⛔ **Ele NÃO é um campo do `WidgetStore`, e a ausência é a decisão** (2026-09-19): aquele
/// guarda o estado de widgets REGISTADOS, e a maior parte do texto deste app não é um widget (o
/// nome de uma linha de propriedade, o item de uma lista, uma frase de estado). *Um facto que só
/// um módulo consome vive nesse módulo* — e a 1.ª redacção pô-lo no store, onde custou uma linha
/// de um ficheiro que já estava no tecto de LOC.
///
/// ⚠️ **O `hot_id` não responde a isto:** ele nomeia o widget debaixo do ponteiro, e este módulo
/// precisa da COORDENADA para decidir se vale a pena guardar o rótulo que acabou de ser cortado.
/// ⚠️ **`Option` e não um par cru:** o estado *«ainda não houve rato»* é real (o arranque, e um
/// ecrã tocado) e tem de ser alcançável — sem ele um gate não consegue medir o silêncio, que é
/// metade do que este módulo promete.
pub fn onde_esta_o_rato(ponteiro: Option<(f32, f32)>) {
    PONTEIRO.set(ponteiro);
}

/// ⭐ **Começa um quadro:** esvazia o que o quadro anterior deixou.
///
/// ⚠️ **Esvaziar é a função toda.** Sem isso a lista cresce sem fim — que é exactamente o
/// vazamento que o cabeçalho do [`super::elisao`] nomeia —, e o balão passaria a mostrar o que
/// estava debaixo do rato há dez segundos.
pub fn novo_quadro() {
    ACHADOS.with_borrow_mut(Vec::clear);
}

/// ⭐⭐⭐ **O ÂMBITO: «o que for cortado aqui dentro mora NESTE rect».**
///
/// ⛔ **Guarda e repõe**, nunca limpa. Um pintor de rótulo chama outro pintor de texto por dentro
/// (o rótulo de propriedade delega no [`crate::paint::paint_text_elided`]), e um âmbito que
/// limpasse ao sair apagaria o do pai a meio da pintura dele. É a mesma aritmética do
/// `push_clip`/`pop_clip`.
pub fn na_area<R>(area: Rect, f: impl FnOnce() -> R) -> R {
    let _guarda = Ambito::nova(area);
    f()
}

/// ⭐⭐ **O MESMO âmbito para quem não pode embrulhar o corpo num fecho** — um pintor com saídas
/// antecipadas.
///
/// ⛔ **Não são duas leis: a [`na_area`] é UMA CHAMADA desta.** Duas implementações da mesma
/// aritmética divergiriam no dia em que o âmbito aprendesse a fazer mais alguma coisa, que é o
/// defeito que esta casa já pagou em `stroke_uniform` e na fileira de param do editor de áudio.
pub struct Ambito {
    anterior: Option<Rect>,
}

impl Ambito {
    #[must_use]
    pub fn nova(area: Rect) -> Self {
        Self {
            anterior: AREA.replace(Some(area)),
        }
    }
}

impl Drop for Ambito {
    fn drop(&mut self) {
        AREA.set(self.anterior);
    }
}

/// ⭐ **O que está debaixo do rato, cortado** — o último a ser pintado, que é o que está por cima.
#[must_use]
pub fn sob() -> Option<(Rect, String)> {
    ACHADOS.with_borrow(|v| v.last().cloned())
}

/// ⭐⭐ **A PORTA de um gate: liga o «conta todos», corre, desliga, devolve.**
///
/// ⚠️ Ela **também** limpa à entrada: um gate que meça o que o vizinho deixou mede outro programa
/// — a lição que o [`super::elisao::arma`] já pagou neste mesmo ficheiro.
pub fn medindo<R>(f: impl FnOnce() -> R) -> (R, Vec<(Rect, String)>) {
    TUDO.set(true);
    ACHADOS.with_borrow_mut(Vec::clear);
    let r = f();
    let out = ACHADOS.with_borrow(Clone::clone);
    TUDO.set(false);
    (r, out)
}

/// ⭐⭐⭐ **Um corte aconteceu** — chamado pela lei, com o texto INTEIRO.
///
/// ⚠️ **O texto inteiro e não o que saiu:** o balão existe para mostrar o que o corte levou. E um
/// texto vazio fica de fora pela mesma razão que fica no censo — um espaçador não é um rótulo.
pub(super) fn corte(texto: &str) {
    // ⚠️ Um quadro de AQUECIMENTO não é visto por ninguém — ver [`crate::aquecimento`].
    if texto.is_empty() || crate::aquecimento::aquecendo() {
        return;
    }
    let Some(area) = AREA.get() else {
        // ⚠️ **Sem âmbito não há balão, e isso é uma RESPOSTA e não uma falha**: há cortes fora de
        //    um pintor (uma medição de largura, um gate). Quem tem rect embrulha; o censo
        //    `todo_corte_do_app_tem_balao` é que decide se algum pintor se esqueceu.
        return;
    };
    let alcancavel = TUDO.get() || PONTEIRO.get().is_some_and(|(px, py)| area.contains(px, py));
    if !alcancavel {
        return;
    }
    ACHADOS.with_borrow_mut(|v| v.push((area, texto.to_string())));
}

/// ⚠️ Só para o censo das elisões: quantos cortes esta thread já viu com âmbito.
///
/// ⛔ Existe para o gate poder cruzar as DUAS listas — a do [`super::elisao`] (todo corte) e esta
/// (todo corte **alcançável**) — sem reabrir o armazém de nenhuma delas.
#[must_use]
pub fn achados() -> Vec<(Rect, String)> {
    ACHADOS.with_borrow(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA_DO_ROTULO: Rect = Rect {
        x: 10.0,
        y: 20.0,
        w: 80.0,
        h: 16.0,
    };

    /// ⭐ Um corte debaixo do rato é alcançável.
    #[test]
    fn um_corte_sob_o_rato_entra() {
        onde_esta_o_rato(Some((50.0, 25.0)));
        novo_quadro();
        na_area(AREA_DO_ROTULO, || corte("Non-Spatialized Radius"));
        assert_eq!(
            sob().map(|(_, t)| t),
            Some("Non-Spatialized Radius".to_string())
        );
    }

    /// ⭐⭐ **O CONTROLO, e é ele que compra o preço:** um corte LONGE do rato não aloca nada.
    ///
    /// ⚠️ Sem esta metade, a lista seria um censo por quadro — a `String` por corte que o
    /// cabeçalho do [`super::elisao`] recusa por escrito.
    #[test]
    fn um_corte_longe_do_rato_nao_entra() {
        onde_esta_o_rato(Some((500.0, 500.0)));
        novo_quadro();
        na_area(AREA_DO_ROTULO, || corte("Non-Spatialized Radius"));
        assert_eq!(sob(), None);
    }

    /// ⛔ Sem âmbito não há balão — uma medição fora de um pintor não inventa um rect.
    #[test]
    fn um_corte_sem_area_nao_entra() {
        onde_esta_o_rato(Some((50.0, 25.0)));
        novo_quadro();
        corte("Non-Spatialized Radius");
        assert_eq!(sob(), None);
    }

    /// ⭐⭐⭐ **O âmbito REPÕE o anterior** — um pintor que delega noutro não apaga o rect do pai.
    ///
    /// ⚠️ É o defeito que um `AREA.set(None)` à saída daria: o rótulo de propriedade chama o
    /// pintor de texto por dentro, e o corte do PAI acontece antes disso mas o do irmão a seguir
    /// ficaria órfão.
    #[test]
    fn o_ambito_repoe_o_pai_em_vez_de_o_apagar() {
        onde_esta_o_rato(Some((50.0, 25.0)));
        novo_quadro();
        let interior = Rect::new(0.0, 0.0, 1.0, 1.0);
        na_area(AREA_DO_ROTULO, || {
            na_area(interior, || {});
            corte("depois do filho");
        });
        assert_eq!(
            sob(),
            Some((AREA_DO_ROTULO, "depois do filho".to_string())),
            "o ambito do filho sobreviveu ao fim dele"
        );
    }

    /// ⭐ O último pintado é o que está por cima.
    #[test]
    fn o_ultimo_pintado_ganha() {
        onde_esta_o_rato(Some((50.0, 25.0)));
        novo_quadro();
        na_area(AREA_DO_ROTULO, || {
            corte("por baixo");
            corte("por cima");
        });
        assert_eq!(sob().map(|(_, t)| t), Some("por cima".to_string()));
    }

    /// ⭐⭐ O censo vê o que o rato não vê — é o que torna o gate possível.
    #[test]
    fn o_censo_ve_todos_os_cortes() {
        let (_, achados) = medindo(|| {
            onde_esta_o_rato(None);
            novo_quadro();
            na_area(AREA_DO_ROTULO, || corte("longe de todo rato"));
        });
        assert_eq!(achados.len(), 1);
    }

    /// ⛔ Um quadro novo esvazia — senão o balão mostra o que estava lá há dez segundos.
    #[test]
    fn um_quadro_novo_esvazia() {
        onde_esta_o_rato(Some((50.0, 25.0)));
        novo_quadro();
        na_area(AREA_DO_ROTULO, || corte("do quadro anterior"));
        onde_esta_o_rato(Some((50.0, 25.0)));
        novo_quadro();
        assert_eq!(sob(), None);
    }

    /// ⚠️ Um texto vazio não é um rótulo.
    #[test]
    fn um_texto_vazio_nao_e_um_rotulo() {
        onde_esta_o_rato(Some((50.0, 25.0)));
        novo_quadro();
        na_area(AREA_DO_ROTULO, || corte(""));
        assert_eq!(sob(), None);
    }
}
