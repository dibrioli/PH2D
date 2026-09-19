//! ⭐⭐ **AS ALAVANCAS QUE AS SONDAS VARREM** — as portas por onde um teste troca
//! um número do passe sem recompilar o produto.
//!
//! Filho (`#[path]`, `cfg(test)`) do [`super`], e o corte é por
//! responsabilidade: ali mora *o passe*, aqui *o que uma sonda pode mexer nele*.
//! ⚠️ Ele saiu de lá por **TECTO DE LOC** (`731` contra `700`) no dia em que a
//! memória do traço entrou.
//!
//! ⛔ **Em produção cada porta devolve a constante medida, AO BIT** — o gémeo
//! `#[cfg(not(test))]` vive no pai, ao lado da declaração deste módulo, para que
//! a ausência de um braço seja erro de compilação e não um número diferente.

thread_local! {
    pub(crate) static RONDAS_DO_TESTE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static ALTERNANCIAS_DO_TESTE: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
    pub(crate) static RAIO_DO_TESTE: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// ⭐⭐ **O CONTROLO da memória do traço, e ele vive DENTRO do gate.**
    ///
    /// ⚠️ A lei que a memória muda é um PARÂMETRO, e o lado que a contradiz tem
    /// de correr no mesmo arnês — senão o «sem memória» é uma corrida de outro
    /// programa. É a mesma forma que o §54 desta linha pagou quando um
    /// `env VAR=` pôs o controlo a correr a lei que ele devia contradizer.
    pub(crate) static SEM_MEMORIA_NO_TESTE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
    /// Quantos anéis a memória transborda — `usize::MAX` = o valor que shipa.
    pub(crate) static ANEIS_NO_TESTE: std::cell::Cell<usize> =
        const { std::cell::Cell::new(usize::MAX) };
    /// A orientação é lembrada — `0` = o valor que shipa, `1` = sim, `2` = não.
    pub(crate) static ORIENTACAO_NO_TESTE: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

/// A memória que o arnês do teste entrega ao pente — `None` quando o CONTROLO
/// está armado. Ver [`SEM_MEMORIA_NO_TESTE`].
pub(crate) fn campo_do_teste(
    campo: &mut ph2d_quadflow::regiao::CampoDoTraco,
) -> Option<&mut ph2d_quadflow::regiao::CampoDoTraco> {
    let aneis = ANEIS_NO_TESTE.with(std::cell::Cell::get);
    if aneis != usize::MAX {
        campo.aneis = aneis;
    }
    match ORIENTACAO_NO_TESTE.with(std::cell::Cell::get) {
        1 => campo.lembra_orientacao = true,
        2 => campo.lembra_orientacao = false,
        _ => {}
    }
    (!SEM_MEMORIA_NO_TESTE.with(std::cell::Cell::get)).then_some(campo)
}

pub(super) fn rondas_da_grelha() -> usize {
    #[cfg(test)]
    {
        let n = RONDAS_DO_TESTE.with(std::cell::Cell::get);
        if n > 0 {
            return n;
        }
    }
    super::RONDAS_DA_GRELHA
}

pub(super) fn raio_do_pente() -> f32 {
    #[cfg(test)]
    {
        let r = RAIO_DO_TESTE.with(std::cell::Cell::get);
        if r > 0.0 {
            return r;
        }
    }
    1.0
}

pub(super) fn alternancias() -> usize {
    #[cfg(test)]
    {
        let n = ALTERNANCIAS_DO_TESTE.with(std::cell::Cell::get);
        if n > 0 {
            return n;
        }
    }
    super::ALTERNANCIAS
}
