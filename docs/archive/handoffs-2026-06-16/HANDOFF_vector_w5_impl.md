═══════════════════════════════════════════════════════════════════
HANDOFF → IMPLEMENTADOR Vector · W5 — variable-width stroke + SDF Hybrid full
Autor: Coordenador (2026-06-04, pós W4 fechado) · ⚠️ LEIA §1 — W5 NÃO é fan-out limpo
═══════════════════════════════════════════════════════════════════

## §0 — ANTES DE TUDO
**Baseline = HEAD LOCAL** (~38 commits não-pushados; W1-W4 + SDF + tokens). NÃO rebase pra
origin. Sanity: `git log --oneline -3` + `git status` (só docs/`.vscode` alheios) +
`cargo check -p ph2d-tool-vector-pencil` verde.
**Git:** commits SCOPED (`git add -- <teus paths>` · `--no-verify`); NUNCA `-A`/`-a`/`stash`;
`M`/`??` alheio → reporte. Você NÃO pusha. `rustfmt <arquivos>` (não `fmt -p`).

## §1 — ⚠️ ESTRUTURA DO W5: Coord-heavy (diferente do W4)

W5 (plano §8) = T5.1 variable-width stroke + T5.2 SDF Hybrid full + T5.3 audit. **A maior
parte é foundational/Coord** — leia antes de começar pra não bater em parede:

| Peça | Dono | Por quê |
|---|---|---|
| Render de stroke variable-width em `ph2d-vector` | **COORD** | `vector_network.rs` hoje usa largura CONSTANTE; Vello stroke-expansion = foundational |
| Modelo de dados de largura (WidthProfile na rede/estilo) | **COORD (+ talvez ADR)** | pode tocar o contrato Vector congelado (`Vertex`/`Segment`, ADR-0056) |
| **T5.2 SDF Hybrid full** (N-ops + ativação asset/tool + gate `vector_sdf_real_time`) | **COORD** | SDF core JÁ fechado (`ph2d-vector-sdf`+bridge, meus); o resto é bridge/gate |

**NÃO toque** `ph2d-vector`, `ph2d-vector-doc` (contrato), `ph2d-vector-sdf`, o bridge,
`render_loop`, nem crie o gate. Isso é meu — entrego o hook de integração (veja §3).

## §2 — TUA FATIA (começa JÁ, tool/node-local, sem meu scaffold)

1. **Captura de pressão no Pencil** — `crates/ph2d-tool-vector-pencil/` (TEU). O pencil já
   captura input; estenda pra **gravar pressão por amostra** (`PointerEvent.pressure`, ADR-0064)
   → uma largura-por-vértice ao longo do traço, guardada no modelo do tool. Isso PRODUZ o dado
   que o render variable-width (meu) vai consumir. Tool-local, isolado, não-bloqueado.
2. **Riqueza do nó `vector.width-profile`** — `crates/ph2d-node-vector-width-profile/` (TEU). v1
   é taper linear (`width_start`/`width_end` → banda preenchida). Estenda os **eixos de perfil**
   (taper/contrast/jitter — abordagem GEOMÉTRICA de banda, que já funciona e renderiza hoje, sem
   depender do meu render). Golden bit-idêntico. `Effect::Pure`. Caps intactos.

Ambos: `cargo check -p <crate>` inner loop; teste/clippy 1× no fim; golden determinístico
(snap Q16.16 + PRNG inteiro semeado p/ jitter). Reporte cada peça quando fechar.

## §3 — PONTO DE INTEGRAÇÃO (eu entrego, você liga)

Eu (Coord) vou: (a) decidir o modelo WidthProfile (e se exige ADR de contrato) + entregá-lo;
(b) implementar o render variable-width em `ph2d-vector`; (c) fazer T5.2 SDF-full + o gate.
**Quando eu landar o modelo WidthProfile + o render**, você liga a pressão capturada no Pencil
(§2.1) nele — esse é o handoff de volta. Até lá, sua pressão fica no modelo do tool; não tente
renderizar variable-width você mesmo (é foundational). Se precisar do meu hook antes de eu
landar, **PARE e reporte** — eu sequencio.

## §4 — Caps congelados (gate `architecture_vector_contract_surface`)
`Vertex`SmallVec32 / `Segment`64 / `Region.segments`16 / `VectorOp≤16` / `NodeOp=2`/`OpResolver=1`/
`NodeManifest=8` **intactos**. **Largura-por-vértice NÃO pode inchar `Vertex`/`Segment`** — se sua
captura de pressão precisar disso, é contrato (Coord+ADR) → reporte, não force. (Guarde a largura
no modelo do TOOL por enquanto, não no `VectorNetwork`.)

## §5 — Onboarding
1. CLAUDE.md §0 + §6 (contrato Vector). 2. ADR-0059 (renderer pipeline) + ADR-0064 (input/pressão).
3. Plano §8 (W5). 4. Código: `ph2d-tool-vector-pencil` (teu), `ph2d-node-vector-width-profile`
(teu, v1), `ph2d-vector/src/vector_network.rs` (render atual constante — REFERÊNCIA, não edite).
═══════════════════════════════════════════════════════════════════
