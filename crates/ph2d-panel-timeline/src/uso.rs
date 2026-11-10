//! **Onde um container é usado** — «used 2×» / «not placed», a mesma frase no chip da barra e na linha da
//! lista (onda 3, `docs/Timeline/16_onda3_containers_arrange.md`). O número vem de
//! [`ph2d_timeline::ContainerView::uses`]; esta é a única porta que o escreve.

/// A frase do uso de um container que está posto `uses` vezes.
#[must_use]
pub(crate) fn rotulo(uses: usize) -> String {
    if uses == 0 {
        ph2d_i18n::tr("panel.timeline.card.not_placed").to_string()
    } else {
        ph2d_i18n::tr_with("panel.timeline.card.used_times", &[("n", &uses)])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn o_uso_diz_quantas_vezes_ou_que_nao_esta_posto() {
        assert_eq!(super::rotulo(0), "not placed");
        assert_eq!(super::rotulo(2), "used 2×");
    }
}
