//! ⭐⭐⭐ **OS EFEITOS de uma forma presa** — a pilha cozida em repouso, o campo do contorno cozido
//! (com o solver em fundo, F50-j) e a união do contacto, num irmão do [`super`] pelo tecto de LOC
//! (2026-10-03). Desde a mesma data o Bind COZE os efeitos (`skin_live::bind`) ⇒ isto só desenha
//! uma forma presa que já tinha efeitos vivos (um projecto anterior).

use super::*;

/// ⭐⭐⭐ **A fonte com a PILHA DE EFEITOS viva cozida no REPOUSO** (quinas vivas incluídas, na
/// ordem do `cooked`), a tabela de pesos dos nós dela e o campo do contorno cozido.
///
/// ⚠️ O `FxCtx` de cada efeito (caixa, centro, `ref_size`) sai assim da forma EM REPOUSO, e o
/// tamanho do efeito deixa de depender da pose. ⛔ Sem campo não há de onde amostrar a tabela ⇒
/// `None`, e a forma desenha-se pela lei antiga.
fn cozido_com_efeitos(
    g: &SkinnedPath,
    pilha: &[FxEntry],
    campo: Option<Rc<CampoFx>>,
) -> Option<CozidoFx> {
    let da_fonte = g.campo.as_ref()?;
    let caminho = geometria_cozida(g, pilha);
    let tabela =
        ph2d_vec_skin::pesos::pesos_dos_pontos(&caminho, campo.as_ref().map_or(da_fonte, |c| &c.0));
    // ⚠️ Pela PERGUNTA, sem correr a união: ela é cara (um *Repeater* de centenas de cópias) e
    // pode falhar, e uma falha lida como «neutra» tentá-la-ia outra vez a cada quadro.
    let contacto = !ph2d_vec_boolean::overlaps_itself(&so_os_fechados(&caminho));
    Some(CozidoFx {
        pilha: pilha.to_vec(),
        caminho,
        tabela,
        campo,
        contacto,
    })
}

/// A fonte com a pilha cozida em repouso, as voltas apertadas já em nós.
fn geometria_cozida(g: &SkinnedPath, pilha: &[FxEntry]) -> VecPath {
    let mut fonte = g.path.clone();
    fonte.effects = pilha.to_vec();
    // ⭐⭐ F50-h: as voltas apertadas do efeito viram NÓS — ver [`crate::skin_desenho_voltas`].
    crate::skin_desenho_voltas::parte_nas_voltas(fonte.cooked().into_owned())
}

/// O solver do campo sobre o contorno cozido — o que corre FORA do quadro (F50-j).
fn resolve_o_campo(caminho: &VecPath, eixos: &[Handle], ossos: usize) -> Option<CampoFx> {
    ph2d_vec_skin::pesos::campo_do_caminho(caminho, eixos)
        .filter(|c| c.ossos() == ossos)
        .map(|c| {
            let i = IndiceDoCampo::novo(&c.malha);
            (c, i)
        })
}

thread_local! {
    /// ⚠️ Só os gates mexem nisto: nos testes da crate o solver corre no quadro (síncrono), e o
    /// gate do solver em fundo liga-o aqui.
    static EM_FUNDO_NO_TESTE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// O solver de um campo NOVO corre numa thread? — no produto sim; nos testes da crate só quando
/// o gate o pede ([`EM_FUNDO_NO_TESTE`]).
fn em_fundo() -> bool {
    !cfg!(test) || EM_FUNDO_NO_TESTE.with(std::cell::Cell::get)
}

/// Liga o solver em fundo nesta thread de teste.
#[cfg(test)]
pub(crate) fn solver_em_fundo_no_teste(v: bool) {
    EM_FUNDO_NO_TESTE.with(|c| c.set(v));
}

/// ⭐⭐⭐ **O COZIDO desta pilha, com o melhor campo que há** (F50-j).
///
/// O campo do contorno cozido custa um solver (`20`–`100 ms`) e arrastar o controlo de um efeito
/// muda a pilha a cada quadro. ⇒ a 1.ª vez (nenhum campo resolvido) o solver corre no quadro — a
/// forma nunca aparece rasgada ao abrir o projecto —; daí em diante corre numa THREAD, UM de cada
/// vez, e entretanto fica à vista o último desenho RESOLVIDO (pilha e campo dela). Quando o campo
/// chega, o cozido refaz-se com ele e o quadro também ([`Ultimo::fx`]).
pub(super) fn efeitos_da_gaveta(
    g: &mut Gaveta,
    guardado: &SkinnedPath,
    pilha: &[FxEntry],
    eixos: &dyn Fn() -> Vec<Handle>,
) -> Option<Rc<CozidoFx>> {
    let ossos = guardado.campo.as_ref()?.ossos();
    if let Some((pedida, rx)) = &g.a_caminho {
        match rx.try_recv() {
            Ok(res) => {
                g.resolvido = Some((pedida.clone(), res.map(Rc::new)));
                g.a_caminho = None;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => g.a_caminho = None,
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
    }
    let exacto = g.resolvido.as_ref().is_some_and(|(p, _)| p == pilha);
    if !exacto {
        let pedida = g.a_caminho.as_ref().is_some_and(|(p, _)| p == pilha);
        if g.resolvido.is_none() || !em_fundo() {
            let caminho = geometria_cozida(guardado, pilha);
            let res = resolve_o_campo(&caminho, &eixos(), ossos).map(Rc::new);
            g.resolvido = Some((pilha.to_vec(), res));
        } else if g.a_caminho.is_none() && !pedida {
            let caminho = geometria_cozida(guardado, pilha);
            let eixos = eixos();
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(resolve_o_campo(&caminho, &eixos, ossos));
            });
            g.a_caminho = Some((pilha.to_vec(), rx));
        }
    }
    // ⛔⛔ **O que se mostra é SEMPRE um par exacto: a pilha resolvida com o campo DELA** (report do
    // dono de 2026-10-03: *«quanto mais veloz se arrasta o valor de twist mais deformações
    // bizarras»*). A 1.ª redacção desenhava a geometria NOVA com o campo de uma pilha anterior —
    // num *Twist* rápido a forma nova sai do domínio velho e rasga. ⇒ enquanto o campo da pilha
    // pedida não chega, fica à vista o último desenho resolvido: o arrasto anda aos degraus do
    // solver, nunca por uma forma que nenhuma pilha desenha.
    let (pilha_r, campo) = g.resolvido.clone()?;
    let actual = g.efeitos.as_ref().is_some_and(|c| {
        c.pilha == pilha_r
            && match (&c.campo, &campo) {
                (None, None) => true,
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                _ => false,
            }
    });
    if !actual {
        g.efeitos = cozido_com_efeitos(guardado, &pilha_r, campo).map(Rc::new);
    }
    g.efeitos.clone()
}

/// ⭐⭐ **A UNIÃO só dos contornos FECHADOS**, com os abertos devolvidos como estavam — as riscas de
/// um *Hatch* são contornos abertos (sem interior), e a [`ph2d_vec_boolean::resolve_overlap`]
/// recusa a forma inteira por causa delas: o contorno da barra cruzava-se por dentro de uma dobra
/// forte (FOTOGRAFADO a `110°`, F50-f). `None` quando nada se cruza.
pub(super) fn uniao_dos_fechados(d: &VecPath) -> Option<VecPath> {
    let mut u = ph2d_vec_boolean::resolve_overlap(&so_os_fechados(d))?;
    u.subpaths.extend(d.subpaths.iter().filter(|c| !c.closed).cloned());
    Some(u)
}

/// O caminho sem os subcontornos ABERTOS.
fn so_os_fechados(d: &VecPath) -> VecPath {
    let mut so = d.clone();
    so.subpaths.retain(|c| c.closed);
    so
}
