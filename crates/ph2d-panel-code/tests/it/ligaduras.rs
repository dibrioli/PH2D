//! ⛔ O código mostra os caracteres ESCRITOS: o `=` de `==` é o mesmo glifo do `=` sozinho (a JetBrains
//! Mono juntaria `==`/`>=` por `calt`; o dono, 15/10, confundiu-se com um `=`).

use ph2d_panel_code::LineCache;
use ph2d_text::{PositionedLayoutItem, TextSystem};
use ph2d_tokens::TypeToken;

fn glyphs(cache: &mut LineCache, ts: &mut TextSystem, s: &str) -> Vec<u32> {
    let layout = cache.get(ts, s, &[], TypeToken::Base.px());
    let mut out = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(run) = item {
                out.extend(run.glyphs().map(|g| g.id));
            }
        }
    }
    out
}

#[test]
fn igual_igual_e_maior_igual_sao_os_caracteres_escritos() {
    let mut ts = TextSystem::new();
    let mut cache = LineCache::default();
    cache.begin_frame();
    let eq = glyphs(&mut cache, &mut ts, "=")[0];
    let gt = glyphs(&mut cache, &mut ts, ">")[0];
    let til = glyphs(&mut cache, &mut ts, "~")[0];
    assert_eq!(glyphs(&mut cache, &mut ts, "=="), vec![eq, eq], "`==` virou ligadura");
    assert_eq!(glyphs(&mut cache, &mut ts, ">="), vec![gt, eq], "`>=` virou ligadura");
    assert_eq!(glyphs(&mut cache, &mut ts, "~="), vec![til, eq], "`~=` virou ligadura");
}
