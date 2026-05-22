# Briefing — Implementador de node-crate (fan-out)

**Para:** uma sessão Claude Code isolada, uma por nó, no fan-out pós-freeze.
**Modelo:** DIRETRIZ §1.3 (3 obrigações) + ADR-0031 (nó = caixa-preta FBP).
**Pré-requisito:** o contrato `ph2d-nodegraph` está **congelado** (W2.T4). Se ainda não, pare — você está no neck, não no fan-out.

Cole o bloco abaixo numa sessão nova, preenchendo `<domínio>`/`<slug>`/spec.

---
```
═══════════════════════════════════════════════════════════════════
BRIEFING — node-crate · domínio: <domínio> · slug: <slug>
═══════════════════════════════════════════════════════════════════

PASTA EXCLUSIVA: crates/ph2d-node-<domínio>-<slug>/
(criada por você; o glob de workspace.members a inclui automaticamente —
NÃO edite o Cargo.toml raiz.)

O QUE VOCÊ FAZ (só dentro da sua pasta):
1. Cargo.toml: deps mínimas — ph2d-nodegraph, ph2d-node-registry,
   e ph2d-expr se usar math por-elemento.
2. src/lib.rs:
   - pub const MANIFEST: NodeManifest { id (NodeTypeId::of("<dom>.<slug>")),
     name, inputs/outputs (PortSpec com PortType = domínio+dim+CLOCK),
     effect (Pure | Temporal | Stateful), clock, params (&[ParamSpec]),
     lowerings (&[LoweringKind::Cpu] e/ou Wgsl) }
   - impl NodeOp { manifest(); eval(ctx) PURO }
   - pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError>
3. Teste golden input→output (ADR-0031 §3): construa um grafo source→seu-nó,
   registre num NodeRegistry, g.validate(&ops), cook, asserte a saída.

EXEMPLO CANÔNICO A COPIAR:
- crates/ph2d-node-debug-wave/  ← input + param + Temporal + ph2d-expr + golden
- crates/ph2d-node-debug-const/ ← gerador Pure trivial

CONTRATO (leia, não edite): crates/ph2d-nodegraph/src/{node,port,effect,attr,cook}.rs
- Você só vê (portas tipadas + efeito + clock + params + lowerings).
- A membrana: se seu nó NÃO escreve estado simulado, é Pure/Temporal (lado pull).
  Stateful é só gameplay. Pure/Temporal são isentos de HR-5.
- Math por-elemento: use ph2d-expr (Expr + eval_column sobre o Stream de entrada).
- PARAMS: leia SEMPRE via `ctx.param("nome")` no eval — resolve o override
  por-instância do grafo se houver, senão o default do manifest. NUNCA leia
  `MANIFEST.params[..].default` direto (ignora overrides) nem `unwrap_or(0.0)`
  (falha silenciosa). Um nome não-declarado no manifest faz `ctx.param` dar
  panic — pego pelo golden test, nunca silencioso. Param que vira contagem/
  alocação: passe por `ph2d_nodegraph::node::param_as_count(valor, max)` e cape
  o produto (vide motion.grid/clone) — um override hostil (NaN/∞/gigante) não
  pode estourar a alocação.

O QUE VOCÊ NÃO TOCA:
- Qualquer arquivo fora da sua pasta.
- ph2d-nodegraph, ph2d-expr, ph2d-node-registry (foundational, congelados).
- crates/ph2d-node-registry-init/ (GERADO — vide abaixo).
- Cargo.toml raiz (glob cobre você).

WIRING (sem colisão, sem edição central):
- Depois de criar a pasta, rode:  cargo run -p ph2d-node-sync
  (regenera register_all_nodes + deps de registry-init a partir do scan).
- Valide o wiring localmente (segundos, fecha o gate antes do CI):
      cargo test -p ph2d-node-registry-init
  (o staleness gate falha se você esqueceu o sync; a compilação de
   registry-init falha se seu `register` tem assinatura errada.)

VALIDAÇÃO (codificação rápida):
- cargo check -p ph2d-node-<domínio>-<slug>   (durante editing)
- cargo test  -p ph2d-node-<domínio>-<slug>   (golden)
- cargo clippy -p ... --all-targets -- -D warnings
- cargo fmt -p ...

NOMES (gates ativos):
- type name canônico = "<domínio>.<slug>", único cross-crate (colisão de id é
  pega no boot por RegistryError::Collision).
- atributos de stream e params: identificadores simples (sem espaço/ponto) —
  o lowering WGSL sanitiza, mas mantenha-os limpos.

QUANDO TERMINAR, reporte ao Enio:
  "Node <dom>.<slug> pronto. Commit local: <sha>. cargo test -p
   ph2d-node-<dom>-<slug> e -p ph2d-node-registry-init verdes."
═══════════════════════════════════════════════════════════════════
```
---

## Por que isto é sem-colisão (a garantia)

Dois agentes adicionando dois nós diferentes **não tocam nenhum arquivo em comum**:
cada um cria sua pasta; o `workspace.members` é glob (zero edit); o
`register_all_nodes` + deps de `registry-init` são **gerados** por `ph2d-node-sync`
(rodado no integração, ou por cada agente — o resultado é determinístico e o
staleness gate garante consistência). É o isolamento FBP do ADR-0031 levado à
prática: o contrato (portas + efeito) é o único acoplamento, e ele está congelado.

## Checklist do revisor (Coordenador)

- [ ] `MANIFEST` completo (params + lowerings preenchidos).
- [ ] `eval` é puro (sem estado global, sem IO); efeito declarado bate (Stateful só se escreve sim).
- [ ] teste golden presente e verde.
- [ ] `cargo run -p ph2d-node-sync` rodado; `cargo test -p ph2d-node-registry-init` verde.
- [ ] clippy `--all-targets` + fmt limpos.
- [ ] nome canônico único; sem dep fora do contrato.
