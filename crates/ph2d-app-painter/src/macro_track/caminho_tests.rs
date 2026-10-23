//! Os gates de [`super`] — o roteiro de cada variante do caminho do dono e o condutor (`#[path]`, fora do
//! tecto de linhas do pai).

use super::{Caminho, Gesto, Variante, roteiro};

/// Cada variante grava, pinta duas pinceladas fechadas na tela, pára e só então toca — por ordem.
#[test]
fn o_roteiro_de_cada_variante_e_o_caminho_do_dono_por_ordem() {
    for v in [
        Variante::Base,
        Variante::Humano,
        Variante::Meio,
        Variante::Splash,
        Variante::SemRewind,
        Variante::Topo,
        Variante::Arrasta,
        Variante::Parar,
        Variante::Cruza,
        Variante::CruzaMolhada,
        Variante::Fim,
        Variante::Etapa,
    ] {
        let r = roteiro(v);
        assert!(
            r.windows(2).all(|w| w[0].0 <= w[1].0),
            "{v:?}: fora de ordem"
        );
        let pos = |g: Gesto| r.iter().position(|(_, x)| *x == g);
        let (grava, para) = (pos(Gesto::Grava).unwrap(), pos(Gesto::Para).unwrap());
        let desce: Vec<usize> = (0..r.len())
            .filter(|&i| matches!(r[i].1, Gesto::Desce(_)))
            .collect();
        let sobe: Vec<usize> = (0..r.len()).filter(|&i| r[i].1 == Gesto::Sobe).collect();
        let cruza = matches!(v, Variante::Cruza | Variante::CruzaMolhada);
        let n = if cruza { 3 } else { 2 };
        assert_eq!((desce.len(), sobe.len()), (n, n), "{v:?}: as pinceladas");
        assert!(grava < desce[0] && desce[0] < sobe[0] && sobe[0] < desce[1]);
        assert!(sobe[1] < para, "{v:?}: Parar depois da 2.ª pincelada");
        if cruza {
            let rw = pos(Gesto::Rewind).unwrap();
            assert!(
                para < desce[2] && sobe[2] < rw,
                "{v:?}: a vertical entre o Parar e o ⏮"
            );
        }
        let arrasta = r.iter().filter(|(_, g)| matches!(g, Gesto::Arrasta(_)));
        let fim = if v == Variante::Arrasta {
            // Saltos para trás depois do Parar, todos dentro do clipe.
            assert!(
                arrasta
                    .clone()
                    .all(|(t, g)| *t > r[para].0 && (0.0..=4.2).contains(&g.segundos()))
            );
            let s: Vec<f64> = arrasta.map(|(_, g)| g.segundos()).collect();
            assert!(
                s.windows(2).filter(|w| w[1] < w[0]).count() > 10,
                "{v:?}: arrasta para trás"
            );
            matches!(r.last().unwrap().1, Gesto::Arrasta(_))
        } else if v == Variante::Fim {
            // Um arrasto, depois do Parar, para lá do fim do clipe.
            let s: Vec<(u64, f64)> = arrasta.map(|(t, g)| (*t, g.segundos())).collect();
            assert!(
                s.len() == 1 && s[0].0 > r[para].0 && s[0].1 > 4.2,
                "{v:?}: {s:?}"
            );
            matches!(r.last().unwrap().1, Gesto::Arrasta(_))
        } else if v == Variante::Etapa {
            // Depois do Parar, as chaves e o clique no losango da 2.ª etapa.
            assert_eq!(arrasta.count(), 0);
            let ch = pos(Gesto::Chaves).unwrap();
            ch > para
                && r.last()
                    .is_some_and(|(t, g)| *t > r[ch].0 && *g == Gesto::Losango(1))
        } else {
            assert_eq!(arrasta.count(), 0);
            matches!(r.last().unwrap().1, Gesto::Play | Gesto::PlayDeCima)
        };
        assert!(fim, "{v:?}: o Play (ou o arrasto) fecha o caminho");
        let sem = matches!(
            v,
            Variante::SemRewind
                | Variante::Topo
                | Variante::Arrasta
                | Variante::Fim
                | Variante::Etapa
        );
        assert_eq!(pos(Gesto::Rewind).is_some(), !sem);
        assert!(r.iter().all(|(_, g)| match g {
            Gesto::Desce([u, v]) | Gesto::Move([u, v]) =>
                (0.0..=1.0).contains(u) && (0.0..=1.0).contains(v),
            _ => true,
        }));
    }
}

/// O condutor dá cada gesto uma vez, os vencidos de uma vez.
#[test]
fn o_condutor_da_cada_gesto_uma_vez() {
    let mut c = Caminho::de(Variante::Base);
    let todos = roteiro(Variante::Base).len();
    let mut n = c.gestos_ate(0).len();
    for ms in (0..20_000).step_by(37) {
        n += c.gestos_ate(ms).len();
    }
    assert_eq!(n, todos);
    assert!(c.gestos_ate(u64::MAX).is_empty());
}

/// W16 — a `sobrepor`: a 2.ª pincelada noutra cor, Parar, o losango dela recuado para ela começar a meio
/// da 1.ª, e só depois ⏮ e ▷.
#[test]
fn a_sobrepor_recua_o_losango_da_segunda_depois_de_parar_e_antes_de_tocar() {
    let r = roteiro(Variante::Sobrepor);
    assert!(r.windows(2).all(|w| w[0].0 <= w[1].0), "por ordem");
    let pos = |f: &dyn Fn(&Gesto) -> bool| r.iter().position(|(_, g)| f(g)).unwrap();
    let para = pos(&|g| *g == Gesto::Para);
    let larga = pos(&|g| matches!(g, Gesto::ArrastaLosango(1, d) if *d < 0.0));
    let cor = pos(&|g| matches!(g, Gesto::EscolheCor(_)));
    let (rw, play) = (pos(&|g| *g == Gesto::Rewind), pos(&|g| *g == Gesto::Play));
    let desce: Vec<usize> = (0..r.len())
        .filter(|&i| matches!(r[i].1, Gesto::Desce(_)))
        .collect();
    assert_eq!(desce.len(), 2);
    assert!(desce[0] < cor && cor < desce[1], "a 2.ª noutra cor");
    assert!(para < larga && larga < rw && rw < play);
}
