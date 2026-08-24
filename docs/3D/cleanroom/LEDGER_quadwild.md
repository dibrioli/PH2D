# LEDGER de proveniência — clean-room do quad remesh (família `quadwild`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_quadwild.md` (append cego).

---

## Alvo

| campo | valor |
|---|---|
| Nome | `quadwild` / `quadwild-bimdf` (umbrella; a família inclui os satélites abaixo) |
| Repo | clone local em `/home/enio/Documentos/Projetos/ph2d-quadbench/oracle/` (fora da árvore da engine) |
| Papers | Pietroni et al., *Reliable Feature-Line Driven Quad-Remeshing* (SIGGRAPH 2021) · Heistermann et al., *Min-Deviation-Flow in Bi-directed Graphs for T-Mesh Quantization* (SIGGRAPH 2023) |
| Licença do umbrella | **GPL-3.0** (`oracle/LICENSE`, verificado byte a byte em 2026-08-24) |
| Precedente da casa | [ADR-0162](../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md) — oráculo fora da árvore, invocado por CLI, nunca linkado |

### A concessão relevante (GPLv3 §2), transcrita

> *"You may make, run and propagate covered works that you do not convey, without
> conditions so long as your license otherwise remains in force."*

⇒ Ler, compilar, rodar, modificar e instrumentar em privado é **licenciado**, não tolerado.
Nenhum ato deste ledger envolve *convey*. Não é AGPL ⇒ o §13 não se aplica; ainda assim o
oráculo roda **local**.

---

## §2 — Triagem: a escada de portas

⚠️ **A licença do umbrella não é a licença de cada fase.** O primeiro achado da triagem é
que a família é um mosaico, e o copyleft entra por **três** submódulos, não pelo todo.

### Tabela de licenças MEDIDA (2026-08-24, lida de cada `LICENSE`/`COPYING` no clone)

| dependência | licença | degrau | fase que ela serve |
|---|---|---|---|
| `vcglib` | **GPL-3.0** | T2 | malha, I/O, utilitários geométricos |
| `xfield_tracer` | **GPL-3.0** | T2 | **o traçado de separatrizes (o nosso F3)** |
| `CoMISo` | **GPL-3.0** | T2 | solver misto-inteiro (o *mixed-integer* do MIQ) |
| `libigl` | MPL-2.0, **com `include/igl/copyleft/` GPL** | T0½ / T2 | campo, parametrização |
| `libsatsuma` | **MIT** | **T0** | **quantização Bi-MDF (o nosso F4)** |
| `lemon` | **Boost** | **T0** | fluxo de custo mínimo (backend do Bi-MDF) |
| `OpenMesh` | **BSD-3** | **T0** | estrutura de malha half-edge |
| `libTimekeeper` | **MIT** | T0 | instrumentação de tempo |
| `nlohmann/json` | **MIT** | T0 | serialização |
| `eigen` | **MPL-2.0** | T0½ | álgebra linear |
| `blossom5-cmake` | Unlicense (o *wrapper*) | ⚠️ | ⚠️ o **blossom5** empacotado tem licença própria — a conferir |
| `quadretopology` | ⚠️ **sem arquivo de licença no topo, sem cabeçalho nos fontes** | ⚠️ | preenchimento por padrões (o nosso F5) |
| `lpsolve` | ⚠️ a conferir (tipicamente LGPL) | ⚠️ | solver linear |
| `glew` | BSD-3 | T0 | só visualização |

⇒ **Registrado antes de qualquer leitura de fonte.** A caçada T1 (irmão permissivo por
fase) começa daqui, e o veredito por fase vai abaixo.

---

## Patente (§8.1) — checkpoint incondicional, CUMPRIDO

- **Buscado em:** 2026-08-24
- **Termos:** `quad mesh extraction` · `cross field` · `integer grid map` ·
  `quadrilateral remeshing` · `global parametrization`, cruzados com Autodesk, Pixologic,
  Maxon, Adobe, Dassault, Siemens, Ansys e universidades; mais os autores dos papers.
- **Resultado:** ⭐ **nenhuma patente viva bloqueia o caminho campo → mapa de grade inteira
  → extracção.** Três achados, com veredito:

| patente | dono | estado | lê sobre nós? |
|---|---|---|---|
| US 8.531.456 (remalhamento por grade 2D em género g) | Technion R&D | **EXPIRADA** | ⇒ espec de graça (§1.5.4) |
| US 11.017.597 (redução de singularidades) | concedida 2021 | **VIVA** | ⛔ não — pós-processa malha quad **existente** por gabaritos. ⚠️ cerca nomeada |
| US 9.349.216 (quad por **esboço**) | ETH Zurich + Disney | **VIVA até 2034** | ⛔ não — rede de curvas **autorada**. ⚠️ cerca nomeada: os autores são os do paper de padrões de retalhos de n lados (família do nosso F5) |

⇒ Detalhe e mecanismo: [`TRIAGEM_quad_remesh.md` §3](TRIAGEM_quad_remesh.md).

---

## Papel E — Especificador

| campo | valor |
|---|---|
| session-id | `edbb014f-4ffb-40ff-bd89-2200158288ca` |
| transcript (⛔ **zona contaminada** — I nunca lê) | `/home/enio/.claude/projects/-home-enio-Documentos-Projetos-PH2D/edbb014f-4ffb-40ff-bd89-2200158288ca.jsonl` |
| aberto em | 2026-08-24 |
| ⚠️ nota de papel | Esta janela **escreveu produto** desta linha antes de assumir E (a cadeia F1–F5 + `ph2d-gridmap` G1–G4). O custo foi declarado ao Enio e a ordem foi mantida: a janela **muda de papel** e fica **queimada para I** no módulo. |

### Cobertura da travessia (§3.E)

⚠️ **A travessia integral do alvo GPL NÃO foi iniciada, de propósito** — o passo 1 do
BLOCO-E (triagem) devolveu uma porta mais barata **antes** dela, e a ordem embutida no
passo 1 é *«PARE e reporte»*. Ler o fonte GPL agora seria pagar contaminação por uma rota
que pode nunca abrir.

**Lido até aqui (2026-08-24), e SÓ isto:**

| o quê | natureza | por quê |
|---|---|---|
| `LICENSE`/`COPYING` de 14 dependências do umbrella | texto de licença | triagem §2 |
| `.gitmodules`, `CMakeLists.txt` do umbrella, `CMakeLists.txt` de um submódulo | manifesto de build | descobrir **qual** dependência traz o copyleft |
| `README.md` público do umbrella | prosa pública | lícita a todos os papéis (§3.I) |
| ⛔ **fonte de algoritmo do alvo GPL** | — | **NÃO LIDO** |
| `Directional` (**MPL-2.0**): cabeçalhos de licença + listagem de módulos + a estrutura de opções da integração + o laço de arredondamento | fonte **permissivo** | ⚠️ **não é alvo copyleft** — nenhuma parede se aplica; lido para responder se a porta T0½ alcança o nosso bloqueador |

### ⚠️ Consequência de papel, registrada

Esta janela leu **fonte MPL-2.0** (Directional) e **metadados** do alvo GPL. Ela **não** leu
algoritmo GPL. Ainda assim, e por disciplina do §2 da skill, **quem executar um porte fiel
T0½ deve ser OUTRA janela** — porte fiel não se mistura com quem percorreu a triagem de um
alvo copyleft.

### ⛔ Vassoura (§7.1): NÃO gerada, de propósito

Gerar ≥20 identificadores idiossincráticos **exige ler o fonte do alvo GPL**. Ela é o
**primeiro acto da Rota B**, se a Rota B abrir — e não antes.

---

### Literatura lida (a fonte REAL da espec)

⭐ **A rota escolhida ([ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md))
é clean-room dos *papers*** — logo a travessia integral do fonte copyleft **não é o insumo**,
e não foi feita. O insumo é público:

| fonte | uso | onde |
|---|---|---|
| QEx (SIGGRAPH Asia 2013) | §2–§6 da espec | público; ⚠️ PDF de **imagem**, extraído com `pdftotext -layout` |
| Mixed-Integer Quadrangulation (SIGGRAPH 2009) | §5 e **§5.1** da espec | público — ⭐ **corrigiu** a 1ª redacção, que dizia «re-resolva» onde a receita é uma **escada adaptativa** |
| Integer-Grid Maps (SIGGRAPH 2013) · Ray (arXiv 2025) | contexto | públicos |

⚠️ Cópias locais em `~/Referencias/papers/` (zona contaminada por convenção; ⛔ o
Implementador busca-as pelos **URLs** do cabeçalho da espec, não por lá).

### Instrumentação do oráculo (§5 da skill)

| artefacto | onde | estatuto |
|---|---|---|
| biblioteca **MPL-2.0** clonada | `~/Referencias/directional/` | permissiva; corrida como oráculo |
| arnês C++ (leitor OBJ/OFF, despejo do mapa, modo `so-mapa`, `lengthRatio`) | `~/Referencias/directional-bench/` | escrito por E |
| exportador do **nosso** campo + **fase zero** (`ph2d-remesh-iso`) + régua de **curl** | `~/Referencias/directional-bench/rustfield/` | escrito por E; depende das nossas crates por caminho |
| régua por-face (espelho do `QuadShape`) e verificador do mapa | idem, e **o verificador entrou no repo** | dados/harness |
| ⭐ **fixtures** publicados a I | `docs/3D/cleanroom/fixtures/` | **dados** — entrada nossa, campo nosso |

⛔ **Nada do oráculo entrou no repositório** além de **saídas** (mapas verificados) e do
verificador escrito por E.

---

## Papel I — Implementador

_(a preencher quando a janela I abrir; declaração do §6 exigida)_
⇒ o canal de ida é [`INBOX_quadwild.md`](INBOX_quadwild.md) (append **cego**).
⇒ o bloco pronto a colar está em [`BLOCOS_para_colar.md`](BLOCOS_para_colar.md).

---

## Papel R — Revisor

- **Modo PRÉ:** ⏳ **PENDENTE — e é condição de abertura da janela I.**
  ⚠️ Exige janela que **NÃO** seja esta (autofiltragem não se audita). Bloco pronto em
  [`BLOCOS_para_colar.md`](BLOCOS_para_colar.md) §1.
- **Modo PÓS:** ⏳ pendente (após paridade verde).

---

## Espec entregue

| versão | caminho | `sha256` (16) |
|---|---|---|
| 1 (2026-08-24) | [`SPEC_extracao_de_malha_quad.md`](SPEC_extracao_de_malha_quad.md) | `4455ee56e1ae6ae5` |

⚠️ **A espec foi corrigida três vezes no dia da entrega**, sempre por medição — as correcções
estão no corpo dela e na [`TRIAGEM §5-bis`](TRIAGEM_quad_remesh.md). A auditoria R-pré incide
sobre **esta** versão.

---

## Incidentes

⚠️ **Um, e é do repositório, não da janela.** O sweep de abertura achou **~460 notas** no repo
**inteiro** a citar arquivo de fonte interno de alvo restrito, **25 com transcrição** —
anterior a esta linha e à própria skill. ⛔ **DESCRITO, nunca reproduzido**, em
[`ACHADO_proveniencia_por_nome_interno.md`](ACHADO_proveniencia_por_nome_interno.md).

- **Régua do §6.2:** todas são *assinatura/nome isolado* ⇒ **relance**, não substancial.
  ⇒ **nenhuma janela é queimada por elas.**
- **Curado:** a família do quad remesh está a **ZERO na árvore rastreada** (com controlo
  positivo vermelho), e a memória contaminada foi re-expressa.
- ⏳ **Aberto:** a família Blender (~420 de Classe A, ~21 de Classe B) — exige vassoura
  própria, que **não existe**.

---

## Fechamento R

_(pendente — modo PÓS)_

---

## Veredito da triagem (§2) — 2026-08-24

⭐⭐⭐ **T0½, não T2.** As duas fases que bloqueiam o produto hoje — o **arredondamento
inteiro** das translações de costura e a **extracção** a partir do mapa de grade inteira —
existem sob **MPL-2.0**, licença **já aceite** pelo `deny.toml` desta casa.

⇒ **PARADO e REPORTADO ao Enio**, conforme a ordem embutida no passo 1 do BLOCO-E.
As três rotas, com preço medido: [`TRIAGEM_quad_remesh.md` §5](TRIAGEM_quad_remesh.md).
