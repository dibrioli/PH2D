//! A lei contra o oráculo (Excalidraw 0.18.1, `saidas/w18/w18_sondas_offline.json`): cada link colado vira
//! vídeo onde ele faz um `embeddable` do YouTube, e texto onde ele faz texto — com as correcções NOMEADAS.

use super::*;

const ORACLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w18/w18_sondas_offline.json"
));

const ID: &str = "dQw4w9WgXcQ";

/// O que o oráculo fez com cada link: `Some(src do iframe)` se nasceu um `embeddable`, `None` se texto.
fn oracle() -> Vec<(String, String, Option<String>)> {
    let d: serde_json::Value = serde_json::from_str(ORACLE).unwrap();
    d["sondas"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["sonda"].as_str().unwrap().starts_with('Y'))
        .map(|s| {
            let p = s["passos"].as_array().unwrap().last().unwrap();
            let link = p["colado"].as_str().unwrap().to_owned();
            let embed = p["elementos"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["type"] == "embeddable")
                .map(|_| p["iframes"][0]["src"].as_str().unwrap().to_owned());
            (s["sonda"].as_str().unwrap().to_owned(), link, embed)
        })
        .collect()
}

/// As divergências DELIBERADAS (o porquê no ADR-0185 e no README do oráculo §W18): o nome da sonda e o
/// que a nossa lei dá.
fn corrected(sonda: &str) -> Option<Option<(&'static str, u32, bool)>> {
    Some(match sonda {
        // O oráculo parte estes (`embed/live`, `embed/watch`) ou não os reconhece: são vídeos.
        "Y05_live" | "Y18_maiusculas" | "Y21_watch_v_depois" => Some((ID, 0, false)),
        // Tocadores do YouTube que o oráculo deixa em texto.
        "Y13_m_youtube" | "Y14_nocookie" | "Y15_music" | "Y19_sem_esquema" => Some((ID, 0, false)),
        // Não são UM vídeo (o oráculo faz um embeddable genérico ou uma lista): ficam texto.
        "Y11_playlist" | "Y12_canal" | "Y24_vimeo" => None,
        // Um id que não tem os 11 caracteres não é um vídeo.
        "Y20_id_curto" => None,
        _ => return None,
    })
}

/// ⭐⭐ Cada um dos 24 links do oráculo: o mesmo veredicto (vídeo/texto), o mesmo id e o começo em
/// SEGUNDOS — ou a correcção nomeada.
#[test]
fn every_oracle_link_gets_the_oracle_verdict_or_a_named_correction() {
    let all = oracle();
    assert_eq!(all.len(), 24, "as 24 sondas Y");
    for (sonda, link, embed) in all {
        let got = youtube(&link).map(|v| match v {
            Video::YouTube {
                id,
                start,
                vertical,
            } => (id, start, vertical),
        });
        if let Some(want) = corrected(&sonda) {
            assert_eq!(
                got,
                want.map(|(i, s, v)| (i.to_owned(), s, v)),
                "{sonda}: {link}"
            );
            continue;
        }
        let Some(src) = embed else {
            assert_eq!(got, None, "{sonda}: o oráculo deixa em texto «{link}»");
            continue;
        };
        let (id, start, vertical) =
            got.unwrap_or_else(|| panic!("{sonda}: o oráculo faz vídeo de «{link}»"));
        assert!(src.contains(&format!("/embed/{id}")), "{sonda}: {src}");
        assert_eq!(vertical, sonda.contains("shorts"), "{sonda}");
        // O oráculo passa o `t` CRU (`start=90s`, `start=1m30s`): nós em segundos.
        let raw = src
            .split("start=")
            .nth(1)
            .map(|s| s.split('&').next().unwrap());
        let want = raw.map_or(0, seconds);
        assert_eq!(start, want, "{sonda}: {src}");
    }
}

#[test]
fn start_reads_seconds_minutes_and_hours() {
    assert_eq!(
        [
            seconds("90"),
            seconds("90s"),
            seconds("1m30s"),
            seconds("1h2m3s")
        ],
        [90, 90, 90, 3723]
    );
    assert_eq!(seconds("x"), 0);
}

/// O tamanho de nascença é o do oráculo (560×315; `/shorts/` 315×560) e o link canónico volta ao mesmo
/// vídeo.
#[test]
fn natural_size_and_canonical_link_round_trip() {
    let v = youtube(&format!("https://youtu.be/{ID}?t=42")).unwrap();
    assert_eq!(natural_size(&v), [560.0, 315.0]);
    assert_eq!(youtube(&canonical_link(&v)), Some(v.clone()));
    assert!(player_url(&v).starts_with("https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ?"));
    assert!(player_url(&v).contains("start=42"));
    let s = youtube(&format!("https://www.youtube.com/shorts/{ID}")).unwrap();
    assert_eq!(natural_size(&s), [315.0, 560.0]);
    assert_eq!(youtube(&canonical_link(&s)), Some(s));
}

/// ⭐ As regras do YouTube: vivo só descoberto, sem rodar e com ≥ 200×200 px VISÍVEIS (W19: meio fora da
/// vista não pausa).
#[test]
fn the_player_is_live_only_where_youtube_allows_it() {
    let ok = OnScreen {
        size: [560.0, 315.0],
        angle: 0.0,
        covered: false,
        visible: [560.0, 315.0],
    };
    assert!(live_allowed(ok));
    assert!(
        !live_allowed(OnScreen {
            covered: true,
            ..ok
        }),
        "tapado"
    );
    assert!(!live_allowed(OnScreen { angle: 0.1, ..ok }), "rodado");
    assert!(
        live_allowed(OnScreen {
            size: [1600.0, 900.0],
            visible: [700.0, 200.0],
            ..ok
        }),
        "meio fora da vista com 700×200 à vista (o zoom do smoke do dono)"
    );
    assert!(
        !live_allowed(OnScreen {
            size: [1600.0, 900.0],
            visible: [700.0, 199.9],
            ..ok
        }),
        "menos de 200 px à vista"
    );
    assert!(
        !live_allowed(OnScreen {
            size: [356.0, 199.9],
            visible: [356.0, 199.9],
            ..ok
        }),
        "abaixo de 200 px"
    );
    assert!(
        live_allowed(OnScreen {
            size: [200.0, 200.0],
            visible: [200.0, 200.0],
            ..ok
        }),
        "o mínimo"
    );
}

/// W19 — o tocador vem SEM os controles dele (os nossos ficam por baixo): cada parâmetro oficial medido.
#[test]
fn the_player_comes_without_its_own_controls() {
    let v = youtube(&format!("https://youtu.be/{ID}?t=42")).unwrap();
    let u = player_url(&v);
    for p in ["controls=0", "fs=0", "disablekb=1", "rel=0", "iv_load_policy=3", "enablejsapi=1"] {
        assert!(u.split(['?', '&']).any(|q| q == p), "falta {p}: {u}");
    }
    assert!(player_url_at(&v, 130).ends_with("start=130"));
    // «Abrir no YouTube» continua onde o quadro parou — e o link volta ao mesmo vídeo, nesse ponto.
    let at = youtube(&link_at(&v, 75)).unwrap();
    assert_eq!(at, Video::YouTube { id: ID.into(), start: 75, vertical: false });
    assert_eq!(link_at(&v, 0), format!("https://www.youtube.com/watch?v={ID}"));
}
