# Fixturas do `ph2d-timeline`

| ficheiro | o que é | gerado por |
|---|---|---|
| `timeline_doc_v18.bin` | um `TimelineDoc` **v18** (o último sem dono por clipe) em postcard: 2 clipes (`Main` com loop, `andar` com ping-pong e expressão), 2 bindings com `wire_id` 42/43, marcador `hit`, uma lane `Base` com strip, o container `Porta` com uma lane e um strip do clipe 1 | `TimelineDoc::to_bytes` no commit `b8418ba63` (antes da v19), por um teste `#[ignore]` de uma corrida só, apagado a seguir |
| `timeline_doc_v18.json` | o MESMO documento em `serde_json` — a régua legível da migração v18 → v19 | idem |
| `timeline_doc_v19.bin` | um `TimelineDoc` **v19** (o último sem clipe de entrada): o v18 acima migrado + dois clipes do dono `0x1_0000002a` (`andar`, `pular`) | `TimelineDoc::to_bytes` no commit `74960e98c` (antes da v20), por um teste `#[ignore]` de uma corrida só, apagado a seguir |
| `timeline_doc_v19.json` | o MESMO documento em `serde_json` — a régua da migração v19 → v20 | idem |
| `timeline_doc_v20.bin` | um `TimelineDoc` **v20** (o último sem `PropKind::Field`): o v19 acima migrado, com o Autoplay do 1.º clipe (Cena) ligado | `TimelineDoc::to_bytes` sobre `a45c8ee8ef` (antes da v21), por um teste `#[ignore]` de uma corrida só, apagado a seguir |
| `timeline_doc_v20.json` | o MESMO documento em `serde_json` — a régua da migração v20 → v21 | idem |

⛔ Não regenere: depois da v19 o código já não escreve v18, e depois da v20 já não escreve v19, e depois da v21 já não escreve v20. As
migrações (`doc_v18.rs`, `doc_v19.rs`) são provadas contra estes bytes em `tests/it/clip_owner.rs` e
`tests/it/clip_entry.rs`.
