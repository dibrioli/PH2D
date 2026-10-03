# HANDOFF DE CONTINUAÇÃO — `line/motion-value`, 2026-10-03 (A VARIANTE ENXUTA · A ABLAÇÃO DO DESENHO)

> **Para a próxima janela da MESMA linha** (não é handoff de integração: a linha continua aberta).
> Assuma pelo [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
> (`cd` + `pwd` + `git branch --show-current` ANTES de ler). Mecanismo e números: [doc 121
> §9.10–§9.11](../121_as_formas_na_placa.md). O anterior: [02/10](HANDOFF_CONTINUACAO_line_motion_value_2026-10-02.md)
> (o §3 dele — o que um leitor do diff entende ao contrário — continua válido).
> ⭐ **A próxima janela começa pelo §4** (o buffer de acumulação, com o kill-criterion já escrito).

## §0 — Identidade

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |
| ramo | `line/motion-value` |
| base | `main` @ `1ad60a1ce` (o `rebase` desta jornada foi no-op) |
| commits desta jornada | `8cb0ab9e1` (a variante enxuta + gate + instrumentos) · `a9fbbf385` (doc 121 §9.10) · o doc §9.11 + este handoff |

## §1 — O que esta jornada fechou

- ⛔ **A premissa do §6 de 02/10 caiu.** Não era o `EixoItem` `56 → 72 B`: o §9.9 tinha posto o ramo do
  tracejado INLINE no fragmento e no `cs_escreve`, e os registos de TODA a cena dobraram na iGPU
  (fragmento `56 → 128` VGPRs, `18 → 8` ondas/SIMD). Sonda intercalada: esticadas `1,74 → 2,45 ms`,
  conformes `~1,0 → 1,37`, densas `1,71 → 2,28`. Na RTX quase nada (`0,33 → 0,36`) — por isso o §9.9 não
  o viu.
- ✅ **Cura** (`8cb0ab9e1`): `override TRACEJADO` no `tracejado(it)` do `shape.wgsl`; o passe compila as
  duas variantes (desenho, `cs_conta`, `cs_escreve`) e escolhe pelo eixo carregado
  (`EixoItem::tracejado`). A enxuta é byte a byte o fragmento de antes. Tabela calma: a cura = antes do
  §9.9 nos três arranjos, nas duas placas (doc 121 §9.11).
- ⛔ **Recusa medida:** a origem das células como interpolante `flat` (`0,81 → 0,80 ms`, critério `≥ 10 %`).
- ⭐ **A ablação do desenho** (§9.11): das `0,81 ms` do desenho das esticadas, `0,68` são o LAÇO DAS
  LISTAS; o piso do hardware é `0,09`, o registo `0,04`.

## §2 — Gates e provas (corridos nesta árvore, RTX)

- `ph2d-shape-gpu --test it -- --ignored`: **10/10** (inclui o novo `so_um_eixo_tracejado_pede_a_variante_completa`).
- `ph2d-app-motion --lib motion_shape_placa::gpu_tests -- --ignored`: **5/5** + a sonda.
- **Mutação `8` de `9`** ([arnês](../ferramentas/mutacao_a_variante_enxuta_2026-10-03.py)): a V9 (o
  `override` fora do predicado do WGSL) **sobrevive por desenho** — só o `registos_dos_shaders.sh` a vê.
- O arnês do tracejado (`21` mutações) NÃO foi re-corrido: a cura não toca no percurso, só no predicado,
  e as `21` passam pela variante completa. ⚠️ Re-corra-o no fecho da linha (DIRETIVA §5).

## §3 — Instrumentos novos (versionados)

- [`registos_dos_shaders.sh`](../ferramentas/registos_dos_shaders.sh) — VGPRs, ondas/SIMD e código de cada
  shader na iGPU (RADV, caches desligados), **sem relógio** — vale com a máquina carregada. Corra-o depois
  de mexer em qualquer ramo raro de um shader quente.
- [`mede_sonda_das_estrelas.sh`](../ferramentas/mede_sonda_das_estrelas.sh) aceita VÁRIOS binários
  (intercalados na mesma janela calma) e `PERFIL=1` (o relógio por passe).
- ⚠️ A máquina partilhada ficou com `load 20–30` durante horas (as sondas da `line-PainterWatercolor`): a
  tabela calma levou ~2 h de espera. Construa as ablações ANTES de lançar a medição, para medir tudo na
  mesma janela.

## §4 — ⭐ O PRÓXIMO ITEM: o buffer de ACUMULAÇÃO (doc 121 §9.11)

Tirar do fragmento a SOMA das listas e deixar-lhe só a mistura: cada aresta deposita, por atómicos em
ponto fixo, a área do pixel que cruza e o resto no seguinte; um fio por célula faz o prefixo (o fundo da
célula + os depósitos) e grava a cobertura acabada; o fragmento faz UMA leitura. Não pede ordem entre
cópias — a mistura fica no hardware. O desenho, as guardas a recontar e o **kill-criterion** (esticadas
iGPU ≤ `1,0 ms` de soma dos passes depois da 2.ª tentativa; conformes ≤ `0,76`, densas ≤ `1,50`; RTX
nenhum `> +10 %`) estão no §9.11. Medir: `PERFIL=1 PLACAS=igpu` com o binário de `8cb0ab9e1` intercalado.

⚠️ **Não quebre:** os `10` gates GPU da crate, os `5` do produto, a mutação `14/14` das listas (§9.8 — as
guardas da cena que muda e das fileiras que não cabem têm de ganhar equivalentes) e a `21/21` do tracejado.

## §5 — O que fica aberto (sem mudança desde 02/10)

A variante COMPLETA (cena com tracejado) continua a `128` VGPRs — encolher o ramo do tracejado é item
próprio · a mordida do traço rente (divergência declarada, §9.9) · `M6`/`S6`/`S8` e o `fx.glow` que lê o
`pump` anterior · `fk.rs` duplicado em seis crates (bug #11).
