# Fixturas do `ph2d-timeline`

| ficheiro | o que é | gerado por |
|---|---|---|
| `timeline_doc_v18.bin` | um `TimelineDoc` **v18** (o último sem dono por clipe) em postcard: 2 clipes (`Main` com loop, `andar` com ping-pong e expressão), 2 bindings com `wire_id` 42/43, marcador `hit`, uma lane `Base` com strip, o container `Porta` com uma lane e um strip do clipe 1 | `TimelineDoc::to_bytes` no commit `b8418ba63` (antes da v19), por um teste `#[ignore]` de uma corrida só, apagado a seguir |
| `timeline_doc_v18.json` | o MESMO documento em `serde_json` — a régua legível da migração v18 → v19 | idem |

⛔ Não regenere: depois da v19 o código já não escreve v18. A migração (`doc_v18.rs`) é provada
contra estes bytes em `tests/it/clip_owner.rs`.
