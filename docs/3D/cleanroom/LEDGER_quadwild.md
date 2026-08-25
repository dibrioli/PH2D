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

### ⚠️ Vassoura (§7.1) — ⛔ **corrigido pelo R-pré em 2026-08-24: esta secção estava ERRADA**

> **O que ela dizia:** *«NÃO gerada, de propósito — gerar ≥20 identificadores idiossincráticos
> exige ler o fonte do alvo GPL.»*
> ⛔ **E o arquivo existe**, com **21 entradas**, commitado, e é o que varre todos os artefactos
> desta pasta. A [`TRIAGEM §7.4`](TRIAGEM_quad_remesh.md) já registava a verdade; o ledger é que
> não tinha sido alinhado a ela. *Uma afirmação falsa no ledger custa mais do que o que esconde.*

**O estado real:**

- ⭐ **Existe uma vassoura PARCIAL** (`VASSOURA_quadwild.txt`, 21 entradas em base64), montada
  **só do que a janela E de facto viu** — nomes de campo colhidos ao conferir licenças e
  manifestos, e os identificadores internos que o próprio **repositório** já citava
  ([`ACHADO`](ACHADO_proveniencia_por_nome_interno.md): 13 linhas da família do quad remesh).
  ⇒ proveniência **lícita**, e nenhuma delas exigiu ler algoritmo GPL.
- ⚠️ **Ela é suficiente para o que se varre hoje** e foi provada por **controle positivo**
  (§R-pré.5): semeada num arquivo, o sweep sai **vermelho**.
- ⛔ **A vassoura COMPLETA da travessia continua por gerar**, e continua a ser o **primeiro acto
  da Rota B**, se a Rota B abrir — porque *essa* exige ler o fonte do alvo GPL.

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

### ⚠️⚠️ INVENTÁRIO DE EXPOSIÇÃO, fase a fase — **uma pergunta para o R decidir**

⛔ **A janela E NÃO se auto-certifica** (§6.2: *«na dúvida, R decide — nunca a própria janela
interessada»*). Este bloco existe para o **R-pré** poder decidir com o inventário na mão, em
vez de herdar um «queimada» sem alcance definido.

**O que esta janela de facto teve no contexto, por artefacto:**

| o que li | grau | o que vi |
|---|---|---|
| ⛔ **integração + arredondamento** da biblioteca (MPL-2.0) | **T0½** | ⛔ **o LAÇO de arredondamento e a estrutura de dados de opções** — é uma implementação directa da **SPEC §5/§5.1** |
| extração da biblioteca (MPL-2.0) | T0½ | ⭐ **apenas** os `#include` do topo, o banner de licença e uma busca que não devolveu nada. ⛔ **Nenhuma linha de algoritmo.** |
| assinaturas públicas e `struct`s de opções dos módulos de extração | T0½ | interface pública (§4.1.13) |
| tutoriais da biblioteca | T0½ | a **sequência de chamadas** da API pública — não algoritmo |
| banners de licença de ~10 cabeçalhos | T0½ | só o texto legal |
| quantização por retalhos (**GPL-3.0**) | ⛔ **T2** | uma `struct` de avaliação + nomes de campo, colhidos ao conferir licença. ⛔ **Nenhum algoritmo**, e pertence à fase **F4**, que **já está construída** e **não faz parte desta obra** |
| ⭐ os *papers* (QEx 2013, MIQ 2009) | público | **a fonte real da espec** — lícita para **todos** os papéis |

**A pergunta, posta com todas as letras:**

> A parede é do **T2/T3** (o próprio §3 o diz: *«onde um agente só já era permitido sem parede
> nenhuma: T0/T0½ … A parede é só do T2/T3»*). O único T2 que esta janela leu **não é
> algoritmo** e pertence a uma fase **fora desta obra**. ⇒ **esta janela está queimada para
> qual escopo?**

**A leitura desta janela (⛔ não é veredito — é o material para o R):**

| escopo | leitura de E |
|---|---|
| ⛔ **SPEC §5/§5.1 — o arredondamento** | **QUEIMADA, e sem dúvida.** Vi o laço equivalente. Escrevê-lo aqui arriscaria convergência de expressão **e** converteria em silêncio a rota do [ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md) (*clean-room dos papers*) na rota **rejeitada** (porte). |
| ⚠️ **SPEC §2–§6 — a extração** | **em aberto.** O insumo que li foi o *paper* — a mesma fonte que o Implementador usaria. Nenhuma implementação de extração entrou neste contexto. |

⛔⛔ **Enquanto o R não decidir, vale o mais restritivo: esta janela não escreve produto
nenhum deste módulo.** É a postura que o ledger inteiro sustenta, e afrouxá-la por conta
própria destruiria o valor probatório dele.

⚠️ **Modo SOLO não resolve isto:** a janela SOLO **nasce** sob as regras do BLOCO-I e delega a
leitura a subagentes. Esta nasceu como E e leu. *SOLO é para a próxima janela, não para esta.*

---

## Papel I — Implementador

| campo | valor |
|---|---|
| session-id | `186ce13e-479b-467a-904c-0ff087ab76c9` |
| declarado em | `INBOX_quadwild.md`, por **append cego** no Passo 0, 2026-08-24 |
| árvore | `Worktrees/line-quadextract`, branch `line/quadextract` (fork de `line/sculpt3d`) |
| entrega | 5 commits; HEAD `4ddf2abaa` |

### Declaração do §6 (transcrita pelo R-pós do canal de ida)

> *"Nenhum conteúdo do fonte do alvo entrou no CONTEXTO desta janela (incluindo reports de
> subagentes e compactação); exposição via pesos do modelo não é atestável por construção —
> mitigada §7.3."*

### ⭐⭐ E ela foi **VERIFICADA**, não acreditada (R-pós, 2026-08-24)

⚠️ **Uma declaração de I é auto-relato; o transcript é o instrumento.** Medido sobre
`~/.claude/projects/…/186ce13e-….jsonl` (2,3 MB):

| pergunta | resposta medida |
|---|---|
| ferramentas usadas | ⭐ **`Bash` × 177, e MAIS NENHUMA** — zero `Read`, zero `WebSearch`/`WebFetch`, zero `Agent` (subagente), zero `SendMessage` |
| caminhos absolutos fora da árvore do PH2D em **qualquer** chamada | ⭐ **três ocorrências, e as três são as próprias entradas da denylist**, escritas (não lidas): duas no `cat > .claude/settings.local.json` do Passo 0, uma no texto do `NEXT_R-POS.md` |
| leitura de `~/Referencias/**` ou do clone GPL | ⛔ **nenhuma** |
| leitura do `LEDGER_*` / `VASSOURA_*` / `TRIAGEM_*` / `ACHADO_*` | ⛔ **nenhuma.** Os quatro nomes aparecem **uma vez**, num `ls` da pasta — *nomes de ficheiro, sem uma linha de conteúdo* |
| sweep da vassoura sobre o transcript inteiro | ✓ **limpo, exit 0** |

⇒ ⭐ **A parede aguentou, e é auditável de fora.**

### ⛔⛔ ACHADO DE PROTOCOLO (não é violação desta janela — é do MOLDE)

O Passo 0 do BLOCO-I promete que *«a parede vira permissão do harness, e não lembrança do
agente»*. **Ela não vira.** Os quatro `deny` são matchers de **`Read(…)`** — e esta janela
fez **177 chamadas `Bash` e ZERO `Read`**. Um `cat`/`sed -n` sobre um caminho proibido
**não é alcançado** por um matcher de `Read`, e o modo de operação desta casa manda
explicitamente preferir o `Bash` onde ele resolve.

⇒ *A parede desta janela foi de DISCIPLINA, não de mecanismo* — e o mecanismo só existe se
o `deny` cobrir também a ferramenta que a janela de facto usa. **Emenda devida à
SKILL_Cleanroom §3.I** (Passo 0), não a esta linha. ⚠️ E a verificação por transcript acima
é o que faz a diferença entre *«a regra existia»* e *«a regra funcionou»*.

---

## Papel R — Revisor

| campo | valor |
|---|---|
| session-id (modo PRÉ) | `23c68c7a-90db-4316-9d14-a4efcda6af7f` |
| data | 2026-08-24 |
| janela | ⭐ **≠ a janela E** (`edbb014f-…`) — a exigência do §3.R para o modo PRÉ está cumprida |
| árvore auditada | `Worktrees/line-sculpt3d` (⚠️ **não** o primário — a pasta `cleanroom/` não existe no `main`) |

| session-id (modo PÓS) | `49c94a84-e903-48a9-bd7f-b14685d71061` |
| data | 2026-08-24 |
| janela | ⭐ **≠ a janela I** (`186ce13e-…`) e ≠ a janela E (`edbb014f-…`) |

- **Modo PRÉ:** ✅ **VERDE em 2026-08-24.** Veredito, método e achados abaixo.
- **Modo PÓS:** ✅ **fechado em 2026-08-24** — paridade, sweeps, revisão estrutural e o
  achado que ela devolveu vivem no [**Fechamento R**](#fechamento-r), no fim deste ficheiro.

### §R-pré.1 — O veredito do §4.2, item a item

⛔ **A auditoria não foi feita lendo a espec contra a memória** — foi feita contra os *papers*
citados no mapa de leitura, com `grep` sobre o texto extraído deles. É o que a torna refutável.

| item do §4.2 | veredito | como foi conferido |
|---|---|---|
| texto de código, trechos, diffs | ✅ **ausente** | leitura integral + sweep verde; os únicos blocos cercados são **matemática** (a forma da transição) e o **nosso** formato de fixture |
| nomes internos do alvo | ✅ **ausente** | o glossário do §0 usa vocabulário **do *paper***, não do fonte: os cinco termos ocorrem **40 · 42 · 13 · 14 · 55** vezes no texto publicado do QEx 2013 ⇒ §4.1.13 (nome público) + §4.1.10, e a espec ainda os põe **entre parênteses**, mapeados a nomes desta casa. A vassoura de 21 identificadores **internos** reais não casou uma vez |
| comentários do original | ✅ **ausente** | — |
| wording de manual/paper verbatim ou quase | ✅ **dentro da medida** | a espec é re-descrição em português; a única passagem atribuída (*«os autores registam que esta escada adaptativa é mais eficiente…»*) é §4.1.12 com proveniência, e é **mais curta e menos específica** que a fonte |
| organização arquivo-a-arquivo / função-a-função | ✅ **ausente** | a espec organiza-se por **fases funcionais** em ordem de dependência de dados (§4.1.1 permite-o explicitamente). Não é a decomposição do fonte — é a ordem de exposição do *paper*, e ela **não é arbitrária**: não há saídas antes de nós, nem traçado antes de saídas |
| pseudo-código espelhando o original linha a linha | ✅ **ausente** | o único procedimento detalhado (§5.1, degrau 1) corresponde ao **Algorithm 1 publicado** do MIQ 2009 ⇒ §4.1.10 («se pudesse estar num paper, pode estar na espec» — aqui **está**), e a espec re-estrutura-o como escada de três degraus em prosa |
| tabela grande afinada à mão, verbatim | ✅ **ausente** | toda tabela é medição **nossa** ou análise de casos em forma fechada (4 linhas, §2.4) |

⇒ ⭐ **§4.2: VERDE. Atestado no cabeçalho da espec.**

### §R-pré.2 — ⭐⭐⭐ A pergunta que o E deixou em aberto: o ESCOPO da queima

O ledger pediu ao R que decidisse (§6.2: *«na dúvida, R decide — nunca a própria janela
interessada»*). **Decidido, e o eixo da pergunta estava errado.**

> ⛔⛔ **EMENDA DO R-PÓS (2026-08-24): a frase seguinte é VERDADEIRA para tudo o que ela
> ENUMERA, e FALSA como afirmação sobre o §5 inteiro.** Duas linhas daquele parágrafo — as
> *«duas modalidades»* e o caso de canto do género > 0 — **não estão no *paper***, e a sua
> proveniência está provada no transcript da janela E. Régua §6.2: **relance**; cura:
> documental, já aplicada na espec. Mecanismo e veredito: [§R-pós.4](#r-pós4).
> ⚠️ Fica aqui, e não corrigida em silêncio, pela razão que o próprio [§R-pré.7](#r-pré7)
> deu: *uma afirmação falsa dentro do ledger custa mais do que o que ela esconde* — e esta
> era, outra vez, **a favor de quem a escreveu**.

**O achado que resolve:** ⭐ **tudo o que a espec §5/§5.1 diz está PUBLICADO no *paper* de
2009**, conferido linha a linha contra o texto extraído — a escolha gulosa da variável de menor
erro, a premissa de que um erro pequeno tem impacto pequeno, a fila dos não-zeros da linha, o
resíduo, a actualização, a escada Gauss-Seidel ⇒ gradiente conjugado ⇒ factorização directa, e
a eliminação de uma variável por restrição. O paper publica-o como **`Algorithm 1`** e como
prosa da §2.1 dele.

⭐⭐ **E a prova de que a espec descende do *paper* e não de uma implementação é ela ser MENOS
específica que ele:** o *paper* dá a tolerância concreta; **a espec não a copia** — fala em
«tolerância» e «tecto» e manda **medir** a fracção que fica no degrau 1. *Uma tradução de
código herda as constantes; uma descrição herda a lei.* A espec também regista que a 1ª
redacção dizia «re-resolva» e foi **corrigida contra o paper** — o paper é o insumo operante.

| escopo | leitura do E | ⚖️ **veredito do R** |
|---|---|---|
| SPEC §5/§5.1 (o arredondamento) | «QUEIMADA, e sem dúvida» | ⚠️ **confirmado, mas por OUTRO mecanismo** — e o escopo é **escrever produto**, não a espec |
| SPEC §2–§6 (a extração) | «em aberto» | ✅ **não queimado** — nenhuma implementação de extração entrou naquele contexto (a própria travessia diz: só `#include`s e o banner) |
| a ESPEC como artefacto | — | ✅ **não é obra derivada.** Descende do *paper*, e o §R-pré.2 acima é a demonstração |

⛔⛔ **A correcção de eixo, e é o que importa:** a exposição do E **não foi ao alvo GPL** — ao
GPL ele viu licenças, manifestos e uma `struct` de campos, que pela régua do §6.2 é **relance**
e pertence a uma fase fora desta obra. O que ele viu foi a **biblioteca MPL-2.0**. Isso não é
violação de parede nenhuma (a MPL é permissiva e aceite pelo `deny.toml`); o risco é **outro**:
escrever o §5 com aquele laço na memória produziria plausivelmente **obra derivada de um
arquivo MPL**, cujo custo é o arquivo ficar **permanentemente público** — exactamente a Rota A
que o [ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)
**rejeitou**, tomada em silêncio.

⇒ ⭐ **E por isso «qual janela está queimada» é a pergunta errada.** A resposta certa é *«qual
FONTE tem de ficar fora de quem escrever o §5»* — e assim a resposta deixa de depender de
identidade de janela. É o que a correcção de parede do §R-pré.3 faz.

**Consequência operacional:** a janela I pode escrever **§2–§6 e §5**, na mesma janela, desde
que a parede corrigida esteja de pé. A janela E mantém-se fora de escrever produto neste
módulo — ⚠️ e isso já era verdade **por decisão registada no ADR-0164**, sem precisar deste
veredito.

### §R-pré.3 — ⛔⛔ ACHADO DE PAREDE (a razão de o R-pré existir), CURADO

⛔ **A espec mandava o Implementador fazer exactamente o que o BLOCO-I dele proíbe.**

Em **dois** sítios (a nota de fecho do §9 e a linha «Regenerar» do README dos fixtures) a espec
dava ao I o caminho do arnês em `~/Referencias/`. Mas o **Passo 0 do BLOCO-I** cria um deny de
`Read(~/Referencias/**)`, e o §3.I conta *porte/fork do alvo em qualquer linguagem e sob
qualquer licença* como **código do alvo**. ⇒ um I obediente à espec violava o próprio passo 0;
um I obediente ao passo 0 não conseguia executar o que a espec lhe pedia.

⚠️ **E o mecanismo torna-o concreto, não teórico:** o arnês é um consumidor **header-only** —
`control.cpp` inclui o cabeçalho onde a implementação vive (**361 linhas**, com o laço que
queimou o E). ⛔ *Um erro de compilação num consumidor header-only despeja o cabeçalho no
terminal.* A exposição involuntária conta na mesma (§6), e chegaria pela porta que a própria
espec abriu.

**Curado pelo R-pré, nos dois sítios + no cabeçalho:**

1. a nota do §9 passa a dizer que correr o arnês é **acto de E** (ou de wrapper de E que entregue
   só dados) — que é a lei do **§5 da skill**, de que a espec se tinha afastado em silêncio;
2. a linha «Regenerar» do README dos fixtures idem, com o mecanismo nomeado;
3. o cabeçalho ganha uma **denylist de CAMINHOS** ao lado da de URLs — ⚠️ *a de URLs não bastava,
   porque as duas implementações estão **neste disco***. Ela nomeia `~/Referencias/**` e
   `ph2d-quadbench/oracle/**`, ⚠️ este último **irmão de `ph2d-quadbench/corpus/`**, que é nosso
   e lícito: o corpus e o clone GPL vivem lado a lado, e `/home/enio/Documentos/Projetos` é
   directório de trabalho por omissão das janelas desta casa.

⛔ **Nada de algoritmo foi acrescentado à espec por este R** — as três correcções **retiram** uma
instrução e nomeiam um caminho proibido. O §3.R proíbe ao R escrever ou ditar produto, e nenhuma
delas o faz.

### §R-pré.4 — Achado de instrumento: o gate nº4 era CEGO ao que a §2.1 prevê

O verificador de mapa (declarado *«o gate nº4 da espec, executável»*) saltava por `continue`
**mudo** toda aresta cuja imagem no domínio é degenerada. Ele imprimia `arestas_interiores` e um
`n` menor, **sem nunca dizer que a diferença existia**.

⚠️ **E a diferença é exactamente o fenómeno que a espec §2.1 manda colapsar antes de tudo** — um
gate cego ao caso que o seu próprio insumo prevê. Medido agora: **`1` aresta em CADA uma das duas
peças** (`6 144 → 6 143`, `10 152 → 10 151`), o que também explica dois números que a espec e o
README davam como «arestas interiores» e eram, na verdade, **arestas medidas**.

⇒ **Curado:** o verificador conta e imprime `degeneradas_no_dominio`, e as duas tabelas passam a
mostrar as duas colunas. *É a lei da casa — ponha a contagem ao lado — aplicada ao instrumento que
gateia os fixtures.* ⛔ Nenhum número de resíduo mudou; o veredito dos dois fixtures continua ✓.

### §R-pré.5 — Sweep (§7.1), com CONTROLE POSITIVO

⚠️ **O sweep foi invocado pelo caminho ABSOLUTO do primário** — `scripts/cleanroom-sweep.sh`
**não existe nesta worktree** (§R-pré.6).

| alvo do sweep | resultado |
|---|---|
| ⭐ **controle positivo** (arquivo semeado com uma entrada decodificada da vassoura) | ✅ **✗ exit 1** — o instrumento **funciona**, e foi provado antes de se acreditar num verde |
| espec + `fixtures/` + README + `NEXT_R-PRE.md` + INBOX (os artefactos que cruzam a parede) | ✓ limpo, exit 0 |
| ledger + triagem + achado (zona E/R) | ✓ limpo, exit 0 |
| `crates/` + `shells/` + `docs/3D/` (árvore rastreada) | ✓ limpo, exit 0 |
| `--git-history` sobre `docs/3D/cleanroom` (mensagens **e** patches) | ✓ limpo, exit 0 |

### §R-pré.6 — ⏳ Os dois BLOQUEIOS que ficam, e nenhum é do R resolver

1. ⛔ **A pasta `cleanroom/` não existe no `main`, e o bloco de abertura de linha nasce do `main`.**
   O E já o reportou («Bloqueio operacional a montante»). ⇒ **uma worktree I aberta pelo MODELO
   padrão não veria a espec.** ⭐ **Saída que não exige ordem de integração:** abrir a worktree do I
   a partir de **`line/sculpt3d`** em vez de `main` — é abertura de linha, não integração, e está
   dentro do Modo L. O `NEXT_I.md` já sai com essa alteração **explícita** no passo 4.
   *A alternativa é o Enio ordenar a integração desta pasta primeiro; as duas servem, e a escolha é dele.*
2. ⚠️ **`scripts/cleanroom-sweep.sh` não é rastreado por git em árvore nenhuma** — ele existe só
   como arquivo solto no primário. ⛔ *Uma ferramenta fora do repo não existe nas outras máquinas,
   e um script novo não existe nas árvores que nasceram antes dele* (CLAUDE.md §2). Não bloqueia o
   **I** (I não roda o sweep), mas bloqueia o **R-pós** de qualquer janela que não seja esta, e a
   reprodutibilidade da prova. ⇒ **tem de ser commitado junto com a pasta `cleanroom/`.**

### §R-pré.7 — Reconciliações do registo (o ledger contradizia-se a si próprio)

⚠️ **O ledger é a prova (§6); uma afirmação falsa dentro dele custa mais do que o que ela
esconde.** Duas foram corrigidas, e **as duas eram a favor de quem as escreveu**, o que é
precisamente o padrão que uma auditoria independente existe para apanhar:

1. a secção da **vassoura** dizia *«NÃO gerada, de propósito»* — e o arquivo **existe, com 21
   entradas, e é o que varre tudo**. A [`TRIAGEM §7.4`](TRIAGEM_quad_remesh.md) já dizia a verdade
   (*«a que existe foi montada só do que esta janela de facto viu»*). O ledger foi alinhado à
   triagem: **existe uma vassoura parcial, de proveniência lícita; a completa é o 1º acto da Rota B.**
2. o **«Veredito da triagem»** no pé do ledger dizia *«T0½ ⇒ PARADO e REPORTADO ao Enio»* — estado
   **anterior** ao [ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md),
   que está **Accepted** e escolheu a rota dos *papers*. Um leitor que parasse no pé do ledger
   concluiria que a decisão ainda não foi tomada. Marcado como **superado**, com o ADR ao lado.

---

## Espec entregue

| versão | caminho | `sha256` (16) |
|---|---|---|
| 1 (2026-08-24) | [`SPEC_extracao_de_malha_quad.md`](SPEC_extracao_de_malha_quad.md) | `4455ee56e1ae6ae5` |
| 2 (2026-08-24, **pós-implementação**) | idem — a correcção do [§R-pós.4](#r-pós4) no §5 + a linha de atestado no cabeçalho | `9086a1dd766e53d6` |
| ⭐ **obra seguinte**, 1 (2026-08-24) | [`SPEC_restricoes_por_eliminacao.md`](SPEC_restricoes_por_eliminacao.md) — a costura e as linhas de feição, **um mecanismo só** | `f56aad2648c4086b` |

⛔ **A versão 2 é POSTERIOR à obra e não a alimentou** — ela **retira** expressão emprestada
e põe no lugar a derivação que a `ph2d-gridmap` já tinha escrito. ⚠️ *Nenhum algoritmo foi
acrescentado por um R*, nas duas passagens (§3.R).

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

## Handoff da corrente (§10)

| passo | estado |
|---|---|
| **E ⇒ R-pré** | ⭐ **entregue** em [`NEXT_R-PRE.md`](NEXT_R-PRE.md), 2026-08-24, **sweep verde sobre o próprio handoff** |
| **R-pré ⇒ I** | ⭐ **entregue** em [`NEXT_I.md`](NEXT_I.md), 2026-08-24, **sweep verde sobre o próprio handoff** (as DUAS mensagens do Modo L) |
| I ⇒ R-pós | ⭐ **entregue e FECHADO** em 2026-08-24 — ver [Fechamento R](#fechamento-r) |

### ⭐ A corrente da OBRA SEGUINTE (as restrições por eliminação)

| passo | estado |
|---|---|
| **E ⇒ R-pré** | ⭐ **entregue** em [`NEXT_R-PRE_eliminacao.md`](NEXT_R-PRE_eliminacao.md), 2026-08-24, **sweep verde sobre os dois artefactos, com controlo positivo antes** |
| R-pré ⇒ I | ⏳ pendente — ⛔ **e é o que bloqueia a obra**: sem o atestado do §4.2 no cabeçalho da espec, a janela I não abre |
| I ⇒ R-pós | ⏳ pendente |

⛔ **O R-pré tem de ser uma janela que não seja o E** (§3.R) — logo **não** a
`49c94a84-…`, que escreveu esta espec.

⛔ **Bloqueio operacional a montante, e não é do E resolvê-lo:** o passo 9 do BLOCO-E manda
commitar espec+ledger+vassoura+README **no `main` do primário antes de a linha I abrir**.
⚠️ Nesta casa isso é **integração**, e o [`CLAUDE.md` §0.7](../../../CLAUDE.md) reserva-a a uma
**ordem explícita do Enio** por um agente integrador dedicado. ⇒ **reportado, não executado.**
*Hoje `main` não vê um único arquivo desta pasta, e uma worktree nova nasce dele.*

---

## Fechamento R

> Modo **PÓS**, 2026-08-24, janela `49c94a84-e903-48a9-bd7f-b14685d71061`.
> Roteiro: [SKILL_Cleanroom §7.2](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **Regra que governa TODO este bloco (§6.1): descreve, nunca reproduz.** Onde a
> identificação exacta importa, o registo traz o `sha256` do trecho — nunca o trecho.

### §R-pós.1 — Paridade

| gate | corrida | resultado |
|---|---|---|
| `ph2d-quadextract` (15) | `cargo test -p ph2d-quadextract --all-targets` | ⭐ **15/15 ✓** (`gates_exact` 6 · `gates_fixtures` 4 · `gates_precision` 4 · `measure_quad_shape` 1) |
| `ph2d-gridmap` (3 + 2 sondas) | idem, `-p ph2d-gridmap` | ⭐ **19/19 ✓**, 13 `#[ignore]` — as 2 sondas desta linha entre elas, rotuladas `"sonda — …"` |
| shell (2) | `cargo test -p ph2d-host-desktop --bins` | ⭐ **2/2 ✓** |

- **Barra derivada, conferida:** o gate nº5 (`o_predicado_e_exacto_onde_o_f64_ja_nao_e`)
  ⭐ **traz um controlo positivo SOBRE O PRÓPRIO CONTROLO** — ele exige que a rota `f64`
  **erre na maioria** dos casos, e regista que a 1ª redacção usava quase-colinearidade
  diagonal, onde o `f64` acertava nos 36 casos. *Um controlo que nunca falha não é um
  controlo*, e este gate mede a si próprio antes de medir o predicado. É a forma mais
  forte de barra derivada que este ledger viu.
- **Fase a fase (§9 da espec):** ⭐ **feita, e é ela que produz o achado da linha.** Os
  dumps existem (`fixtures/`, mapas de referência verificados), e a comparação por fase
  está no [§8 do handoff](../handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md):
  sobre os mapas de referência a extracção fecha (`100 %` quads, `χ` preservado nas duas
  peças); sobre a cadeia da casa a **forma** entra na barra e a **topologia** não, e a
  causa está medida **a montante** (o G3). ⇒ *a obra desta linha não é o bloqueador.*
- ⚠️ **Nota de ambiente:** a máquina esteve a `load ≈ 13` durante a corrida. Nenhum dos 20
  gates novos mede relógio ou razão de relógios, então a leitura é válida — mas o **gate
  batched** do handoff (§6) foi corrido pela janela I sob fan-out, e os 2 ✗ que ele reporta
  são da família de flakes de recurso do `CLAUDE.md` §5.0, verdes sozinhos, sem uma linha
  do diff a tocá-los.

### §R-pós.2 — Sweep total (§7.2.2), com CONTROLE POSITIVO

⚠️ Invocado pelo **caminho absoluto do primário** — o script continua a não existir nesta
worktree (§R-pré.6.2 segue **aberto**, ver §R-pós.6).

| alvo | resultado |
|---|---|
| ⭐ **controle positivo** (ficheiro semeado com a 1ª entrada decodificada; `sha256(16) = d535c85cc0eac159`) | ✅ **✗ exit 1** — o instrumento funciona, provado **antes** de se acreditar num verde |
| **árvore rastreada**: `crates` `shells` `docs` `scripts` `tests` `tools` `runtime` `metrics` `spikes` `project-memory` `assets` `CLAUDE.md` `Cargo.lock` `Cargo.toml` `deny.toml` `clippy.toml` `.claude` `.github` | ✓ **limpo, exit 0** |
| `--git-history -- docs/3D/cleanroom` | ✓ limpo |
| `--git-history -- project-memory` | ✓ limpo |
| ⭐ `--git-history` **restrito aos 5 commits desta linha** (`line/sculpt3d..line/quadextract`, mensagens **e** patches) | ✓ **limpo — esta linha não traz um único hit** |
| **transcript da janela I** (§7.2.2, opcional e recomendado) | ✓ **limpo, exit 0** |
| ⛔ `--git-history` **do repositório inteiro** (`--all`) | ⛔ **✗ exit 1** — ver abaixo |

#### ⛔ O único vermelho, atribuído até ao commit

O sweep de histórico `--all` acha **um nome de ficheiro interno** da vassoura (uma
assinatura de função e dois números de linha ao lado dele), em mensagens de commit **e**
em patches. ⛔ Descrito, não reproduzido. **Dois commits, e nenhum é desta linha:**

| commit | data | onde vive | o que é |
|---|---|---|---|
| `fe61596fc` | 2026-08-21 | ⛔ **já em `main`** e em 7 branches | quem **introduziu** as notas — anterior a esta linha, a este alvo e ao uso da skill aqui |
| `6d00c7e10` | 2026-08-24 | `line/sculpt3d` | ⭐ o commit da **CURA**, que as retira da árvore |

⇒ **É o incidente já registado** em
[`ACHADO_proveniencia_por_nome_interno.md`](ACHADO_proveniencia_por_nome_interno.md), cuja
régua do §6.2 é **relance** (assinatura/nome isolado) ⇒ **nenhuma janela é queimada**.

⚠️⚠️ **E ele traz um mecanismo que vale a pena escrever, porque não é óbvio:** *a
proveniência apagada da ÁRVORE fica GRAVADA no HISTÓRICO pelo próprio commit que a apagou*
— um `git log -p` reimprime a linha removida. ⇒ **um sweep de árvore verde não implica um
sweep de histórico verde**, e a única cura seria reescrever o histórico de `main`, que
⛔ **não é decisão do R nem desta linha**. Fica **NOMEADO e ABERTO** aqui, como facto do
repositório.

### §R-pós.3 — Revisão estrutural (§7.2.3): convergência de EXPRESSÃO

⚠️ **Comportamento igual não é achado — é o objectivo.** O que se procura é decomposição
arbitrária igual, ordem não-forçada igual, nomes traduzidos, truques de escrita.

#### O que está e o que **não** está neste disco (medido antes de comparar)

| a obra | há implementação local para comparar? |
|---|---|
| **§2–§6, a extracção** | ⛔ **NÃO.** A família do *paper* não existe em árvore nenhuma desta máquina: `grep` do vocabulário dela sobre o clone GPL inteiro devolve **0** ocorrências, e a biblioteca MPL resolve o mesmo problema por **outra família** (arranjo de segmentos por triângulo + DCEL + unificação de vértices + emparelhamento de meias-arestas), que não tem nós/saídas/traço/células |
| **§5, o arredondamento** | ⭐ **SIM** — o laço da biblioteca MPL, que é o trecho que queimou o E |

⇒ Para a obra 2 **não há com o que convergir localmente**; sobra o risco de **convergência
de treino** (§7.3), que se avalia pelo idioma e pelos detalhes-além-da-espec (abaixo).

#### A comparação do §5, lado a lado — **DIVERGENTE**

| eixo | a implementação de referência | ⭐ o nosso `round.rs` |
|---|---|---|
| a actualização após cada arredondamento | **re-solve completo** do sistema KKT (fatoração esparsa) a cada variável pregada | ⭐ **a escada do *paper* de 2009**: Gauss–Seidel local ⇒ varreduras globais orçamentadas. ⛔ *Nunca* re-resolve |
| decomposição | **um laço só**, que reconstrói matrizes, resolve, escolhe e prega | **quatro fases sequenciais** nomeadas: calibre ⇒ singularidades ⇒ costuras ⇒ propagação |
| o passo do **calibre** | ⛔ **não existe lá** | ⭐ **nosso, e é a resposta a *quais* variáveis são inteiras**: as de árvore vão a zero de graça, sobram `E − V + c` |
| o caso de canto das costuras | um `if` **dentro** do laço, que muta a máscara | uma **fase 3 própria**, com `switched_to_seams` a contá-la |
| a escolha gulosa `min │x − round(x)│` | igual | igual — ⚠️ **e é a LEI**, publicada como `Algorithm 1` do *paper* de 2009. Comportamento igual é o objectivo |

⇒ ⭐ **Nenhuma convergência de expressão no §5.**

#### A obra 2 — os sinais que se podem medir sem um alvo local

- ⭐⭐⭐ **A divergência nº1 é o sinal mais forte que este ledger podia colher.** A espec
  **sugeria** a rota de precisão múltipla com filtro em vírgula flutuante — que é
  exactamente a rota da biblioteca de referência (ela carrega números exactos apoiados numa
  biblioteca de inteiros grandes). A implementação foi por **outro lado**: truncagem numa
  grade **global**, domínio em `i64`, orientação num determinante `i128` — e a crate shipa
  com **UMA dependência, interna** (`ph2d-mesh`), zero externas. *Um implementador a
  convergir com o que existe teria chegado ao que a espec já lhe oferecia.*
- **Idioma:** ficheiros e tipos são desta casa (`nodes`/`ports`/`walk`/`cells`/`fan`/
  `sanitize`/`ingest`/`exact`/`mapa`), com `*Stats`/`*Report` **auto-medidos** em cada fase
  (`ring_len`, `port_step`, `contested`, `collapsed_fans`) — uma forma que nenhuma
  implementação em C++ desta área tem, porque ela existe para alimentar gates.
- **Detalhes além da espec, conferidos um a um** — todos com origem declarada e verificável
  no próprio código: `MAX_SIDES = 64` (⭐ medido, com a distribuição ao lado e o relato do
  tecto anterior que **apagava** células), `MAX_STEPS = 256` (tecto de sanidade, com o
  raciocínio), `COORD_MAX`/`Q_HEADROOM` (**identidades**, não medições), `contested`
  (defeito próprio, apanhado por medição, com o mecanismo escrito). ⇒ **nenhum tripwire de
  recall por tratar.**
- ⚠️ **Conferido e LIMPO, para o próximo leitor não o reabrir:** o módulo chama-se `ports` e
  o tipo `Port`, que é o termo **público do *paper*** e não o termo de casa do glossário
  ("saída", usado na prosa). §4.1.13 admite nome público ⇒ **não é achado**.

### §R-pós.4 — ⛔⛔ O ACHADO: duas linhas do §5 da espec **não** descendem do *paper*

⚠️ **Este é o achado que o modo PÓS existe para produzir, e ele corrige o próprio R-pré.**

O §R-pré.2 afirma: *«tudo o que a espec §5/§5.1 diz está PUBLICADO no paper de 2009,
conferido linha a linha»*. ⭐ **Reconferi, e para tudo o que ele ENUMERA a afirmação
sustenta-se** — a escolha gulosa, a premissa do impacto pequeno, a fila dos não-zeros da
linha, o resíduo, a actualização `x_k ← x_k − r_k/A_kk`, a escada de três degraus e a
eliminação de uma variável por restrição estão no `Algorithm 1` e na §2.1 publicados; e a
espec é de facto **menos específica** (o *paper* dá a tolerância concreta, a espec manda
**medir**).

⛔ **Mas duas linhas do §5 não estão na lista dele, e não estão no *paper*:**

| a linha da espec | onde ela está **mesmo** |
|---|---|
| *«Duas modalidades … arredondar as COSTURAS **ou** as SINGULARIDADES»* | ⛔ o *paper* de 2009 arredonda as variáveis inteiras da transição (`j_e, k_e`) e **não tem** modalidade de singularidades. É um **campo booleano de opções** da biblioteca MPL, com o seu doc-comment — `sha256(16) = 00fc1b34114bbdf0` |
| *«Caso de canto medido: quando todas as singularidades já foram pregadas mas ainda restam costuras por arredondar (acontece em peças com alça…)»* | ⛔ **um comentário de bloco** daquela biblioteca — `sha256(16) = 372f54e21780afaf`. ⚠️ A frase da espec segue-lhe os **três elementos na mesma ordem**, parêntese incluído |

⭐ **A proveniência está PROVADA, não suposta:** o transcript da janela E regista o `curl`
que trouxe aquele ficheiro e o `grep` que imprimiu **as duas linhas** no contexto dela,
antes de a espec ser escrita. ⚠️ E a [`TRIAGEM §…`](TRIAGEM_quad_remesh.md) já registava
honestamente o campo de opções e o que ele faz — *o E viu, e anotou onde viu*. **O que
falhou foi a espec ter atravessado a parede sem o rótulo**, e o §4.2 do R-pré não ter
coberto estas duas linhas.

#### ⚖️ Veredito do R (§6.2 — *na dúvida, R decide, nunca a janela interessada*)

| pergunta | veredito |
|---|---|
| é «substancial» (⇒ queima)? | ⛔ **NÃO — é relance.** O que atravessou foi **uma ideia** (há duas famílias de variável para arredondar) e **um facto de topologia** (num género > 0 as costuras que fecham ciclo são independentes das singularidades), que o §1.2 põe no piso do que **nunca** é protegível. Da *expressão*, atravessou o esqueleto de **uma frase** |
| a janela I ficou exposta? | ⛔ **não.** Ela nunca viu o comentário — recebeu uma frase em português com uma justificação **diferente** |
| há quarentena a comparar? | ⛔ **não é preciso**, e eu comparei na mesma: a região correspondente (`RoundOptions::pin_singularities` + a fase 3) **diverge** do original em decomposição (fase sequencial × `if` dentro do laço) e traz uma justificação **medida por nós** que o original não tem (*sem pregar a singularidade, o ponto fixo da holonomia cai num meio-inteiro e a malha rasga-se ali; na esfera fina saíam 4 nós onde eram precisos 8*) |
| ⇒ re-derivação do código (§7.3.d)? | ⛔ **não prescrita** |

#### ⏳ O que fica DEVIDO (documentação, e é do E — não do I, não desta linha)

1. **A espec §5 tem de perder o esqueleto emprestado.** ⭐ A cura não exige inventar nada:
   a casa **já escreveu a mesma verdade melhor e sozinha**, no doc-comment do
   [`round.rs`](../../../crates/ph2d-gridmap/src/round.rs) — *a translação de uma costura é
   grandeza de calibre; numa árvore de expansão vão todas a zero de graça, e os inteiros a
   escolher são as `E − V + componentes` costuras que **fecham ciclo***. Numa peça de género
   0 esse número é zero; num toro não é. ⇒ **substituir a frase pela derivação de calibre,
   que é nossa, e nomear a proveniência da observação.**
2. **O §R-pré.2 do ledger tem de ser emendado**, pela razão que o próprio §R-pré.7 deu:
   *uma afirmação falsa dentro do ledger custa mais do que o que ela esconde*. ⚠️ E ela é,
   outra vez, **a favor de quem a escreveu**.

### §R-pós.5 — Incidentes (§7.2, item 4)

| origem | estado |
|---|---|
| `INBOX_quadwild.md` | ⭐ **uma única linha, e é a declaração de sessão do Passo 0.** Transcrita para o [Papel I](#papel-i--implementador). ⛔ **ZERO relances, ZERO tripwires de recall, ZERO dúvidas de espec** — e a auditoria de transcript do §R-pós.1 confirma que não havia nada a declarar |
| histórico do repositório | ⛔ o vermelho de `--git-history` (§R-pós.2), **pré-existente e já registado**, régua = relance |
| ⛔ **novo, aberto por este R** | a proveniência das duas linhas do §5 (§R-pós.4), régua = **relance**, cura = documental |
| ⚠️ **de MOLDE, não desta linha** | o `deny` do Passo 0 não alcança a ferramenta que a janela usa ([Papel I](#papel-i--implementador)) |

### §R-pós.6 — Session-ids (§7.2, item 4)

| papel | session-id | fora de {E, queimadas}? |
|---|---|---|
| E | `edbb014f-4ffb-40ff-bd89-2200158288ca` | — (queimada por decisão registada) |
| R-pré | `23c68c7a-90db-4316-9d14-a4efcda6af7f` | — |
| ⭐ **I** | `186ce13e-479b-467a-904c-0ff087ab76c9` | ✅ **SIM** — distinta das duas, e o transcript dela prova-o por comportamento |
| R-pós | `49c94a84-e903-48a9-bd7f-b14685d71061` | ✅ ≠ I |

### §R-pós.7 — ⏳ O que continua ABERTO (nenhum é do R resolver)

1. ⛔ **`scripts/cleanroom-sweep.sh` continua NÃO RASTREADO** — só existe como ficheiro
   solto no primário, e **não existe nesta worktree**. O §R-pré.6.2 pediu-o e ele não veio.
   ⇒ *a prova deste ledger não é reproduzível noutra máquina, nem por outra janela.* Tem de
   ser commitado junto com a pasta `cleanroom/`.
2. ⛔ **A pasta `cleanroom/` continua ausente do `main`** (§R-pré.6.1). Não bloqueou esta
   linha, porque ela nasceu de `line/sculpt3d`.
3. ⚠️ **Colisão de número de ADR `0164`**, escrita por duas linhas com títulos diferentes
   (a desta corrente, versionada; e uma **não versionada** na árvore primária, mais uma
   `0165` idem). ⛔ **Não é desta linha** e ela **passa muda** — quem integrar conta,
   escolhe, e regenera o índice (`bash scripts/adr-index.sh`).
4. ⚠️ **A emenda de espec que a janela I devolveu** (o G3 penaliza a costura em vez de
   **eliminar** a variável, e a espec §5.1 já nomeia a cura): é pergunta para o **E**, via
   Enio. ⛔ A janela I não foi olhar, e fez bem.

---

### ⭐ FECHAMENTO (§6)

| item do §6 | estado |
|---|---|
| **Paridade** | ✅ **20/20 gates novos verdes** (15 + 3 + 2), barra derivada conferida, comparação **fase a fase** feita sobre os dumps de referência — §R-pós.1 |
| **Sweep de árvore** | ✅ **verde**, com controlo positivo vermelho provado antes |
| **Sweep de histórico** | ✅ verde nos **5 commits desta linha**, em `docs/3D/cleanroom` e em `project-memory`; ⛔ **vermelho no repositório inteiro**, atribuído a **dois commits pré-existentes** (um deles já em `main`), régua §6.2 = **relance** — §R-pós.2 |
| **Sweep de memória** | ✅ verde (`project-memory`, árvore e histórico) |
| **Sweep do transcript de I** | ✅ verde |
| **Similaridade** | ✅ **sem convergência de expressão no código.** O §5 diverge do único alvo local em decomposição e em método de actualização; a obra 2 **não tem alvo local** com que convergir, e o idioma e a rota escolhida (⭐ zero dependências, `i64`/`i128` em vez da precisão múltipla que a própria espec oferecia) apontam para o contrário de convergência — §R-pós.3 |
| **Incidentes** | ✅ todos transcritos e tratados; **um novo**, aberto por este R, classificado **relance**, cura **documental** e devida pelo E — §R-pós.4/5 |
| **Session-id de I** | ✅ fora de {janelas E, queimadas}, e **verificado por transcript**, não por auto-relato — §R-pós.6 |

⇒ ⭐⭐⭐ **LEDGER FECHADO. O MÓDULO ESTÁ APTO A INTEGRAR.**

⚠️ **Com duas coisas ditas com todas as letras, porque um fechamento que as calasse valeria
menos que nenhum:**

- ⛔ **Nada do que fica devido é bloqueador de integração** — as duas dívidas do §R-pós.4
  são **de documentação** (a espec e o §R-pré.2 deste ledger), não de código, e o código foi
  comparado contra o alvo e diverge. O que fica no `main` é obra da casa.
- ⛔ **O ship é do Enio, e o smoke também** (`CLAUDE.md` §0.7). O caminho novo shipa
  **desligado**, com gate a contar a bifurcação única.

---

## ⚠️ ADENDO PÓS-FECHAMENTO — mudança de papel por ordem do dono (§6.5)

**2026-08-24, mesma janela `49c94a84-…`, depois de o fechamento acima estar assinado.**

O Enio ordenou *«implemente até o smoke ser possível»*. O custo foi explicado numa frase
antes de qualquer edição, como o §6.5 manda, e a ordem manteve-se. ⇒ **esta janela deixou
de ser só R e escreveu produto.** Fica registado o que ela escreveu **e o que recusou
escrever**, porque é a fronteira que dá valor ao resto do ledger.

| | |
|---|---|
| ⛔ **NÃO escrito, e a recusa é o ponto** | a eliminação da variável de costura no G3 / qualquer linha de `solve.rs` ou `round.rs`. ⚠️ **Esta janela leu o laço de arredondamento da referência** para a revisão estrutural do [§R-pós.3](#r-pós3) — escrevê-lo aqui converteria em silêncio a rota do ADR-0164 na que ele rejeitou. ⇒ **janela I nova** ([handoff §8-bis](../handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md)) |
| ⭐ **escrito** | [`sculpt3d_scenes_quad.rs`](../../../shells/desktop/src/sculpt3d_scenes_quad.rs) — o **roteiro de smoke** da cena `=35` para o caminho novo, mais dois gates. ⛔ **Zero linhas de algoritmo**, em crate nenhuma da cadeia; é texto de terminal e a bifurcação que o escolhe |
| a razão de ser produto e não doc | o roteiro **existente** manda *"PARE"* diante de uma casca esburacada — correcto no caminho de sempre, **falso** no novo, onde o buraco está medido. *Um smoke que manda reportar como regressão o que já está medido gasta o dono do produto duas vezes.* |

⚠️ **A parede não foi atravessada:** o ficheiro tocado não pertence ao alvo funcional da
espec (§2–§6, §5) nem a qualquer crate da cadeia — o sweep continua verde sobre a árvore, e
o §R-pós.3 (similaridade) não é afectado por ele.

⛔ **O que esta janela NÃO pode voltar a ser:** a janela I de qualquer obra do §5 ou da
extracção. Ela entra no conjunto **{janelas queimadas}** para este módulo, ao lado da E.

---

## ⭐ OBRA SEGUINTE — a mesma janela assume o papel **E** (§3.E)

**2026-08-24, ordem do Enio: *«siga ao estado da arte»*.**

⚠️ **«Estado da arte» tem endereço medido**, e são as duas queixas do smoke: a casca não
fecha (`~1 %` de células rasgadas) e a grade não encosta aos vincos. ⭐⭐⭐ **E a leitura
do *paper* público mostrou que são o MESMO mecanismo em falta** — *uma restrição linear
entra eliminando uma variável*. A costura é uma; a aresta de feição é outra. ⇒ **uma obra,
dois pagamentos**, e é isso que dimensiona o trabalho.

| | |
|---|---|
| papel | ⭐ **E — Especificador.** §3.E: *«contaminado por definição, e tudo bem»* — **é precisamente por já ter visto que esta janela pode especificar** |
| ⛔ o que ela continua a NÃO poder ser | a **I** desta obra, nem de nenhuma outra deste módulo |
| espec entregue | [`SPEC_restricoes_por_eliminacao.md`](SPEC_restricoes_por_eliminacao.md) |
| handoff da corrente | [`NEXT_R-PRE_eliminacao.md`](NEXT_R-PRE_eliminacao.md) |
| sweep | ✓ limpo sobre os dois, **com controlo positivo vermelho antes** |

⛔⛔ **A contra-medida que esta espec carrega, e que o R-pré tem de cobrar:** o risco dela
**não é o de sempre**. O de sempre é o E filtrar mal a travessia; aqui é **convergência de
expressão a entrar pelo próprio E**, num ponto onde ele viu a resposta (a montagem de
restrições da biblioteca MPL). ⇒ a espec foi escrita **sem receita de montagem** — diz o
que tem de ser **verdade** e qual é a **lei publicada**, e recusa estrutura de dados,
decomposição e ordem. ⚠️ *Isso é uma afirmação do E sobre o próprio trabalho, e o
[`NEXT_R-PRE_eliminacao.md`](NEXT_R-PRE_eliminacao.md) põe-na como item nº1 da auditoria —
precisamente para não ser aceite de graça.*

⭐ **E a espec traz a prova de que descende do *paper* e não de uma implementação, pelo
mesmo teste que valeu para a anterior:** o *paper* dá os quatro coeficientes concretos da
detecção de feição, e ⛔ **a espec não os copia** — manda medi-los no nosso corpus.
*Quem traduz código herda as constantes; quem descreve herda a lei.*

---

## ⭐ OBRA 2 — Papel **R, modo PRÉ** (as restrições por eliminação)

| campo | valor |
|---|---|
| session-id | `6ce7cd70-b800-48d7-91c7-b18f17bc7bc1` |
| data | 2026-08-24 |
| ≠ janela E? | ✅ **sim** — a E desta espec é `49c94a84-…`; esta janela é outra, e nunca conteve fonte de alvo antes de assumir R |
| espec auditada | [`SPEC_restricoes_por_eliminacao.md`](SPEC_restricoes_por_eliminacao.md) |
| handoff produzido | [`NEXT_I_eliminacao.md`](NEXT_I_eliminacao.md) |

### §R-pré2.1 — ⭐ O veredito do §4.2: **VERDE**, e a contra-medida do E sustenta-se

⚠️ **O item nº1 desta auditoria não era filtragem, era CONVERGÊNCIA vinda do próprio E** —
a janela que escreveu esta espec leu, no mesmo dia, o laço de arredondamento **e a montagem
de restrições** de uma implementação de referência. Ela declarou ter escrito a espec **sem
receita de montagem**. ⛔ *Uma afirmação do E sobre o próprio trabalho é exactamente o que o
R existe para não aceitar de graça.*

**Conferido por varredura de alarme sobre a espec inteira** (estrutura de matriz · ordem de
eliminação · decomposição · factorização · permutação · esparsidade · nomes de ficheiro,
função ou tipo):

| onde | o que a varredura devolveu |
|---|---|
| §1 (a lei) e §2.3 (o requisito) | ⭐ **ZERO** — nenhuma das palavras de alarme. Os quatro pontos do §2.3 são requisitos (*o que tem de ser verdade*), e o parágrafo final recusa explicitamente estrutura, decomposição e ordem |
| §3 (as feições) | ZERO |
| únicos hits em toda a espec | o **próprio aviso** do E (linhas 30/35), a palavra *«decomposição»* a designar o **dump** do oráculo (§5) e *«esparsas»*, que é a **cerca do *paper*** |

⭐ **E a lei que a espec afirma está PUBLICADA**, conferida no *paper* de 2009 (fim do §2
dele, imediatamente antes do §3): os autores dizem, em uma frase, que tratam restrições
lineares **eliminando internamente uma variável por restrição independente**. ⚠️ E o *paper*
**também não diz como** — a espec está ao nível dele ou abaixo. A segunda metade da lei da
espec (*«nunca como termo de energia»*) é **nossa**, e vem da medição do `SEAM_WEIGHT`.

⭐ Idem para o §3: *«uma coordenada constante e inteira ao longo da aresta ⇒ uma variável
eliminada»* e o **bónus do bordo** (a mesma maquinaria preserva o bordo e evita bordo
serrilhado) estão os dois publicados no §5.2 do *paper*. ⚠️ **Falta-lhes a citação da
secção** — a espec diz *«publicado»* sem dizer onde (§4.1.12 pede o link). Emenda menor.

⭐⭐ **E a prova de descendência do *paper* e não de uma implementação sustenta-se**: os
coeficientes concretos da detecção de feição **não** foram copiados. O *paper* dá cinco
símbolos com valor; a espec dá **zero** e manda medir.

### §R-pré2.2 — ⛔⛔ ACHADO DE PAREDE nº1: a vassoura não cobria o que o E leu

**O sweep do E saiu verde sobre uma vassoura de 21 entradas — e as 21 são todas da família
GPL da obra anterior.** A implementação que o E desta espec de facto leu é **outra
biblioteca** (a permissiva, tratada como alvo por decisão do [ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)
e pelo §3.I: *porte/fork em qualquer linguagem e sob qualquer licença conta como código do
alvo*). ⛔ **Nenhum dos identificadores dela estava na rede.**

⇒ ⭐⭐⭐ *Um controlo positivo prova o INSTRUMENTO; ele não prova a COBERTURA.* O verde do E
era verde sobre uma rede sem os buracos do peixe que se caçava — e o controlo positivo dele,
correcto e vermelho, não podia dizê-lo.

**Curado por este R:** a vassoura passa de **21 para 56 entradas** (+35), colhidas dos
módulos de integração/arredondamento e do esboço de extracção que o inventário de exposição
do E nomeia — nomes de tipo, nomes de função e **os campos da estrutura de opções**, que é a
superfície de maior risco desta obra. ⛔ Tudo em base64, uma entrada por linha, como o §7.1
manda; nada em claro toca a árvore.

⚠️ **A vassoura não aceita comentários** — o sweep exige que *toda* linha não-vazia decodifique
para base64 não-vazio, e sai `2` se não. Registado para quem a alargar a seguir.

### §R-pré2.3 — ⛔⛔ ACHADO DE PAREDE nº2: a `TRIAGEM` estava do lado errado da parede

⭐ **É o achado que só a vassoura alargada podia produzir**, e apareceu no primeiro sweep
depois dela:

| ficheiro | estatuto ANTES | o que ele carrega |
|---|---|---|
| `LEDGER_quadwild.md` | ⛔ marcado, e **negado** pelo Passo 0 | 1 hit — esperado, é a zona contaminada |
| ⛔⛔ `TRIAGEM_quad_remesh.md` | **não marcado, não negado, e a README convidava a lê-lo** (*«quem decide»*) | ⛔ **3 hits: nomes de ficheiro internos e os campos da estrutura de opções do alvo, com a glosa do que cada um faz** |

⚠️ E um deles é precisamente o campo de opções cuja **ideia** o [§R-pós.4](#r-pós4) já
apanhou a atravessar a parede na espec anterior. ⇒ *O mesmo identificador estava, ao mesmo
tempo, registado como incidente numa página e oferecido em lista de leitura na página ao
lado.*

⛔ **A regra já existia e não estava em mecanismo nenhum:** o §3.I diz *«dentro de
`cleanroom/`, SÓ `SPEC_*`»*, e o `deny` do Passo 0 cobria `LEDGER_*` e `VASSOURA_*` — dois de
seis. **Curado:** a README marca ⛔ a `TRIAGEM`, o `ACHADO` e os `NEXT_*`, transcreve a regra
do §3.I, e o `deny` do novo `NEXT_I_eliminacao.md` cobre a pasta inteira por padrão.

### §R-pré2.4 — ⛔⛔⛔ ACHADO DE PAREDE nº3: o `deny` do Passo 0 **nunca foi exercido**, e corre sob `bypassPermissions`

O handoff do E pedia um `deny` que cobrisse **Bash** e não só `Read` (medido: a janela I
anterior fez **177** chamadas `Bash` e **ZERO** `Read`). ⭐ **Reconferi a medição no
transcript daquela janela e ela está exacta** — `{'Bash': 177}`, mais nada.

⚠️ **Mas a causa é uma camada acima, e o `deny` de Bash não a alcançaria.** Medido nesta
máquina, em 2026-08-24:

| onde | o que diz |
|---|---|
| `~/.claude/settings.json` | `defaultMode: bypassPermissions` · `deny: []` · allow de `Bash` e `Read(//**)` **nus** |
| `<repo>/.claude/settings.local.json` | `defaultMode: bypassPermissions` · `deny: []` · `Bash(*)` no allow |
| `~/.config/Code/User/settings.json` | `claudeCode.allowDangerouslySkipPermissions: true` + `claudeCode.initialPermissionMode: "bypassPermissions"` — a cura aplicada em **2026-08-24** para os prompts de edição ([memória](../../../project-memory/reference_vscode_extension_refuses_bypass_and_edits_always_prompt_in_default.md)) |

⇒ **A máquina está configurada para NÃO consultar a camada de permissões, e a parede do
Passo 0 vive inteiramente dentro dela.**

⭐ **E o transcript prova que ela nunca foi testada:** das 177 chamadas da janela I, **duas**
citam os caminhos proibidos — e as duas são a própria janela a **escrever** o ficheiro de
`deny` e a escrever o handoff. **ZERO tentativas de leitura.** Os dois «resultados com cara de
recusa» são falsos positivos (o texto da skill e o eco do ficheiro). ⇒ *a parede aguentou
porque ninguém lhe encostou; não há uma única observação de que ela pare alguém.*

⛔ **O que este R NÃO faz:** afirmar se um `deny` é ou não consultado sob `bypassPermissions`.
Não é atestável de dentro de uma sessão, e um ledger que o afirmasse repetiria o defeito que
o §R-pré.7 já corrigiu duas vezes — uma afirmação confortável e não medida.

⭐⭐⭐ **A cura, e ela não depende da resposta:** o Passo 0 do `NEXT_I_eliminacao.md` termina
com um **CONTROLO POSITIVO sobre a própria parede** — a janela I tenta ler
`~/Referencias/CANARIO_do_passo_zero.txt`, criado por este R **precisamente para poder ser
lido sem contaminar ninguém** (não contém expressão de alvo nenhum, e diz isso mesmo no
corpo). Recusado ⇒ a parede está de pé. Devolvido ⇒ **a parede está em baixo, a janela PARA e
reporta antes de escrever uma linha de produto.** *Um muro que ninguém empurrou é um muro que
ninguém tem — e empurrá-lo tinha de deixar de custar a contaminação que ele protege.*

### §R-pré2.5 — Achados de PROVENIÊNCIA (§4.3.2) — nenhum é de parede, todos são de rigor

⚠️ **Nenhuma tabela desta espec vem do alvo** — todas rastreiam para código ou handoff
nossos, e conferi uma a uma. Mas três não sobrevivem ao teste *«escreva o número que a
medição deu, com a tabela ao lado»*:

1. ⛔⛔ **§1, a tabela do `SEAM_WEIGHT` — a coluna que carrega o argumento mistura DUAS
   estatísticas e DUAS medições.** O doc-comment de `SEAM_WEIGHT` tem **duas** tabelas: uma na
   esfera `24×36` (com colunas `p50` e `max` separadas) e outra na esfera fina `96×144` com o
   G4/F5 a montar. A espec toma a linha `8` da segunda (`2,9°` · `0,23`, que é a coluna
   **max**) e as linhas `64` e `512` da primeira (`0,004` e `0,0006`, que são a coluna
   **p50**) — e escreve o ângulo de `512` como um intervalo `13,0°–16,8°` que atravessa as
   duas. ⇒ *a queda de `0,23` para `0,0006` mistura max com p50 e malha com malha.* Lida de
   forma consistente, a queda é `0,23 → 0,05 → 0,01` (fina) ou `0,90 → 0,146 → 0,017`
   (`24×36`). ⭐ **O veredito de §7 — não afinar o peso — SOBREVIVE**, e sobrevive melhor pela
   terceira coluna da segunda tabela (o enviesamento dos quads: `17° → 19° → 22°`), que é
   produto e não resíduo. **Emenda: uma tabela, com as condições dela ao lado.**
2. ⚠️ **§2.1 e §2.2** rastreiam **exactos** para o [handoff da linha](../handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md)
   (`1,0834` · `18 282` · `0,4913` · `0,2348`). ⭐ Proveniência nossa, confirmada. Falta só
   **nomeá-la** na espec.
3. ⚠️ **§0** não nomeia o instrumento das duas linhas medidas (é o `chain_info`, que o §4 já
   nomeia noutro sítio). Emenda de uma linha.

### §R-pré2.6 — ⛔ O gate nº1 REPROVA a referência de que diz descender

⭐ **Medido por mim, correndo o verificador de `fixtures/` sobre as duas peças:**

| peça | resíduo de translação **max** | rotação max | arestas |
|---|---|---|---|
| `torus_64x32` | **`3,553e-15`** | `4,959e-15` | `6 143` (1 degenerada) |
| `sculpt_hooked` | **`3,553e-15`** | `4,652e-14` | `10 151` (1 degenerada) |

⛔ A espec escreve a barra do gate nº1 como **`3,5e-15`**, e `3,553e-15 > 3,5e-15` ⇒ **os
próprios mapas de referência falhariam o gate.** *Uma barra copiada com um dígito a menos
inverte-se de «tão bom como a referência» para «melhor que a referência».*

⚠️ **E há uma segunda metade:** o §1 promete resíduo **zero por construção**, *«não «pequeno»,
não «abaixo de uma tolerância»»* — e o gate nº1 é uma tolerância. As duas afirmações
reconciliam-se (depois da eliminação o resíduo é o erro de **avaliação** da substituição em
vírgula flutuante, que é representação e não folga), mas a espec **não as reconcilia**, e um
implementador que leve o §1 à letra escreverá `== 0.0`.

⇒ **Emenda devida pelo E** (registada no cabeçalho da espec, que é o único sítio que o I lê).
⛔ Este R **não** fixa a barra: é conteúdo funcional da espec, e o §3.R diz *achado → E
reescreve*.

### §R-pré2.7 — ⛔ A OBRA B perdeu a janela de estabilidade (e é a única emenda que BLOQUEIA)

O §3.1 manda **medir** os quatro coeficientes — ⭐ e a recusa de os copiar é correcta e é a
prova de descendência do *paper*. ⛔ **Mas ele descreve o papel de três.**

O que a lei publicada faz, e a espec não diz: a estimativa é feita numa **faixa** de raios, e
à volta de **cada** raio candidato há uma **janela** — é *dentro dessa janela* que os dois
limiares (anisotropia e piso de curvatura média) têm de valer **em toda ela** para o candidato
ser válido, e é *dentro dela* que se mede a variação de direcção que elege o mais estável. A
espec funde a janela na faixa e escreve *«a de menor variação de direcção dentro da faixa»*.

⇒ ⛔ **Como está, ela especifica outra regra** — o desvio sobre a faixa inteira —, e manda
medir um quarto coeficiente cujo papel nunca nomeia. *Uma espec pode ser menos específica que
o paper nos NÚMEROS; ela não pode ser menos específica na LEI, que é a metade que não tem dono.*

⭐ **Não bloqueia a obra:** o §6 da própria espec manda fazer **a costura primeiro** e proíbe
misturar as duas numa wave. ⇒ a janela I abre na OBRA A e a emenda chega a tempo.

### §R-pré2.8 — Cabeçalho (§4): quatro campos em falta, preenchidos

| campo do §4 | antes | agora |
|---|---|---|
| Ledger · Patente · Denylists · a frase final | ✅ | ✅ |
| **Mapa de leitura da literatura** | ⛔ **ausente** — e a espec manda o I seguir um mapa que não existia | ✅ os dois *papers*, com as secções que interessam e o apêndice a pular |
| **Filtragem §4.3 · Sweep** | ⛔ ausentes | ✅ com data |
| **Auditoria §4.2 (R-pré)** | ⛔ ausente (é a que faltava para a janela I poder abrir) | ✅ |
| ⛔⛔ **as duas emendas devidas** | — | ✅ escritas **no cabeçalho**, que é o único sítio da pasta que o I lê |

⛔⛔ **E um achado que só apareceu ao preencher o mapa de leitura:** os *papers* estavam em
`~/Referencias/papers/`, **dentro da árvore que o Passo 0 nega inteira**. ⇒ *a espec mandava o
Implementador ler uma fonte lícita guardada atrás da parede que o proíbe de a alcançar* — e
teria de escolher entre desobedecer à espec e desobedecer ao Passo 0. **Curado:** literatura
pública passa a viver em `~/Literatura/`, fora de qualquer denylist, com README a dizer porquê.

### §R-pré2.9 — Sweep (§7.1), com **DOIS** controlos positivos

⚠️ Corrido pela cópia **rastreada** do script (ver §R-pré2.10), não pela do primário.

| alvo do sweep | resultado |
|---|---|
| ⭐ controlo positivo **A** — entrada ANTIGA semeada | ✅ **✗ exit 1** — o script funciona |
| ⭐⭐ controlo positivo **B** — entrada **NOVA** semeada | ✅ **✗ exit 1** — a **cobertura nova** funciona (é o controlo que faltava) |
| espec + `NEXT_R-PRE_eliminacao.md` + README + INBOX + `fixtures/` (o que cruza a parede) | ✓ **limpo, exit 0** — com **56** entradas |
| árvore rastreada do produto (`crates` `shells` `scripts` `CLAUDE.md`) | ✓ limpo, exit 0 |
| zona E/R (`LEDGER` · `TRIAGEM` · `ACHADO` · `NEXT_*` antigos) | ⛔ **✗ exit 1 — 4 hits**, e é o §R-pré2.3 |
| `NEXT_I_eliminacao.md` (antes de ser salvo) | ✓ limpo |

### §R-pré2.10 — Os dois «passam mudas» do handoff: um curado, um NOMEADO

1. ✅ **`scripts/cleanroom-sweep.sh` deixa de ser não-rastreado** — pedido pelo §R-pré.6.2,
   por vir no §R-pós.7.1, e agora commitado nesta branch. ⇒ a worktree do I (que nasce daqui)
   passa a tê-lo, e a prova deste ledger passa a ser reproduzível noutra máquina. ⭐ Conferido
   pela cópia local, com controlo positivo vermelho antes.
2. ⛔ **A colisão do ADR `0164` continua, e continua a passar muda** — medido hoje:
   `line/quadextract` tem `0164-quad-extraction-is-clean-room-from-papers…` (versionado) e a
   árvore primária tem `0164-instances-are-real-entities…` **não versionado**, mais um `0165`
   idem. **Não é do R resolver**: quem integrar conta, escolhe e regenera o índice
   (`bash scripts/adr-index.sh`). ⚠️ *Um número que soma entre linhas conta-se, e a colisão de
   dois literais iguais funde MUDA.*

### §R-pré2.11 — ⭐ Veredito

| pergunta | veredito |
|---|---|
| §4.2 (o que o modo PRÉ existe para responder) | ⭐ **VERDE** — atestado no cabeçalho da espec |
| a contra-medida declarada pelo E | ⭐ **sustenta-se**, conferida por varredura e por leitura |
| convergência de expressão vinda do E | ⛔ **nenhuma encontrada na espec.** ⚠️ E a rede que o diz é agora a rede certa (§R-pré2.2) |
| a janela I pode abrir? | ⭐ **SIM, na OBRA A (a costura)** — que é a ordem que o §6 da espec já impunha. A OBRA B espera a emenda do §3.1 |
| session-id do R-pré ≠ E? | ✅ `6ce7cd70-…` ≠ `49c94a84-…` (E) e ≠ `edbb014f-…` / `23c68c7a-…` / `186ce13e-…` |

---

## Veredito da triagem (§2) — 2026-08-24 · ⚠️ **SUPERADO no mesmo dia (nota do R-pré)**

> ⛔ **Leia o parágrafo seguinte antes deste bloco.** Este veredito é o estado **anterior** à
> decisão, e quem parasse aqui concluiria que ela ainda não foi tomada.
> ⭐ **A decisão foi tomada e está `Accepted`:**
> [ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)
> — **clean-room dos *papers*; a biblioteca MPL-2.0 fica FORA, como oráculo.** A Rota A (porte
> fiel T0½) foi **rejeitada com motivo medido**: arquivos permanentemente públicos no subsistema
> mais valioso, descarte da cadeia própria, e ⛔ **falha em 3 de 7 peças do nosso corpus**.
> ⚠️ O degrau T0½ **não foi queimado** — só não foi tomado primeiro.

**O que este bloco registava, e continua verdadeiro como FACTO de triagem:**

⭐⭐⭐ **A porta mais barata era T0½, não T2.** As duas fases que bloqueiam o produto hoje — o
**arredondamento inteiro** das translações de costura e a **extracção** a partir do mapa de grade
inteira — existem sob **MPL-2.0**, licença **já aceite** pelo `deny.toml` desta casa.

⇒ **PARADO e REPORTADO ao Enio**, conforme a ordem embutida no passo 1 do BLOCO-E — e foi
dessa paragem que saiu o ADR-0164.
As três rotas, com preço medido: [`TRIAGEM_quad_remesh.md` §5](TRIAGEM_quad_remesh.md).
