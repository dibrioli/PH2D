#!/usr/bin/env bash
# Provas de mutação do AVISO DE VISIBILIDADE (ordem do dono, 2026-09-19:
# «coloque um alerta de que se não forem usados com duplicator e um objeto a ser copiado,
# são invisíveis»).
#
# A corrente que o aviso atravessa tem QUATRO elos, e cada bloco mata um:
#
#   manifesto  →  a REGRA (marca_as_fontes_de_posicoes)      · os gates do app-motion
#              →  a BANDEIRA no registo                       · idem
#              →  quem VESTE, e o alcance a jusante           · a_lei_chega_ao_sistema_de_alerta
#              →  o SISTEMA DE ALERTA (diagnose)              · idem + os do diagnosticador
#
# ⚠️⚠️ **Os elos 3 e 4 mudaram em 2026-09-19, por ordem do dono** (*«o módulo tem um sistema de
#    alerta. não era para colocar a mensagem no próprio nó»*): a nota permanente SAIU do cartão e
#    a mensagem passou a viajar pelo `ph2d-motion-diagnose`. Os blocos que mediam a vista do
#    cartão e a tinta da fileira **foram apagados com o sujeito deles**, não afrouxados.
#
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o MUTADO).
# ⚠️⚠️ CONTROLO sobre o próprio FILTRO — um filtro que casa ZERO testes sai VERDE e lê-se
#    exactamente como «SOBREVIVEU».
# ⚠️ Toda troca é por `muta`, que ABORTA se a âncora não aparecer o número esperado de vezes.
# ⛔⛔ **NUNCA DUAS CORRIDAS DESTE SCRIPT AO MESMO TEMPO — medido em 2026-09-19.** Duas corridas
#    em paralelo partilham a árvore e não o `$TMP`: uma muta um ficheiro, a outra guarda o estado
#    JÁ MUTADO como «original», e o restauro dela grava a mutação **permanentemente**. Foi assim
#    que a cláusula do `Dim::Vec2` ficou apagada no disco, com todos os testes a passarem — *o
#    modo de falha é mudo, e o que o denuncia é um `git diff` que ninguém mandou existir.*
# ⚠️ E o `cargo fmt` MOVE as âncoras: uma expressão que passou a caber em duas linhas faz o
#    `muta` abortar (bem) — releia o fonte antes de corrigir a âncora, nunca a escreva de memória.
#
# Corra-o pela porta de recursos:
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_a_marca_e_o_alerta_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

REG=crates/ph2d-node-registry/src/fontes_de_posicoes.rs
INIT=crates/ph2d-node-registry-init/src/lib.rs
DUP=crates/ph2d-node-motion-duplicator/src/lib.rs
DIAG=crates/ph2d-motion-diagnose/src/lib.rs
HEAL=crates/ph2d-app-motion/src/motion_bridge_heal.rs
STYLE=crates/ph2d-render/src/sink_style.rs
ORDEM=crates/ph2d-eval-motion/src/sink_style.rs
DOT=crates/ph2d-render/src/atlas/dot.rs
SHELL_INIT=shells/desktop/src/init.rs

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").orig"; }
restaura() { cp "$TMP/$(basename "$1").orig" "$1"; touch "$1"; }

muta() { # ficheiro vezes antigo novo
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
p, n, old, new = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
s = open(p).read()
c = s.count(old)
if c != n:
    sys.exit(f"  ⛔ ANCORA: {old!r} aparece {c} vezes em {p} (esperado {n})")
open(p, "w").write(s.replace(old, new))
PY
}

prova() { # nome pacote filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(cargo test -p "$2" --lib -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto MUTADO"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

bloco() { # nome pacote filtro ficheiro vezes antigo novo
  guarda "$4"
  if muta "$4" "$5" "$6" "$7"; then prova "$1" "$2" "$3"; else FALHAS=$((FALHAS+1)); fi
  restaura "$4"
}

# ⚠️ O irmão para os gates que vivem em `tests/it/`: um `--lib` ali casa ZERO testes, e o
# controlo do filtro leria isso como «filtro vazio» — certo, e inútil.
prova_it() { # nome pacote filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(cargo test -p "$2" --test it -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto MUTADO"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

# ⭐⭐⭐ A forma MAIS FORTE de sangrar: a mutação nem chega a compilar.
#
# ⚠️ **Ela precisa de bloco próprio porque o `prova` a leria como «filtro vazio»** — e um
# «filtro vazio» é um alarme sobre o ARNÊS, não sobre o produto. Aqui o que se afirma é que o
# COMPILADOR guarda a lei, que é o que um `const _: () = assert!(…)` compra.
prova_nao_compila() { # nome pacote
  TOTAL=$((TOTAL+1))
  echo "── $1"
  if cargo check -p "$2" --all-targets >/dev/null 2>&1; then
    echo "  ⛔⛔ SOBREVIVEU — o produto MUTADO compila; a lei nao esta' no compilador"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou no COMPILADOR (a forma mais forte)"
  fi
}

bloco_nao_compila() { # nome pacote ficheiro vezes antigo novo
  guarda "$3"
  if muta "$3" "$4" "$5" "$6"; then prova_nao_compila "$1" "$2"; else FALHAS=$((FALHAS+1)); fi
  restaura "$3"
}

bloco_it() { # nome pacote filtro ficheiro vezes antigo novo
  guarda "$4"
  if muta "$4" "$5" "$6" "$7"; then prova_it "$1" "$2" "$3"; else FALHAS=$((FALHAS+1)); fi
  restaura "$4"
}

echo "=== ELO 1 — a REGRA das tres clausulas ==="

bloco "a clausula do Vec2 cai: os nos de VALOR ganham o aviso" \
  ph2d-app-motion "lei_da_aparencia" "$REG" 1 \
  "        let emite_posicoes =
            |p: &PortSpec| p.ty.domain == Domain::Instances && p.ty.dim == Dim::Vec2;" \
  "        let emite_posicoes = |p: &PortSpec| p.ty.domain == Domain::Instances;"

bloco "a clausula da ORIGEM DE APARENCIA cai: o aviso mente no source.object" \
  ph2d-app-motion "lei_da_aparencia" "$REG" 1 \
  "            .filter(|id| !self.is_object_source(*id) && !self.is_live_vector_source(*id))" \
  "            .filter(|id| { let _ = id; true })"

bloco "o `!` do input inverte: toda PASSAGEM ganha o aviso" \
  ph2d-app-motion "lei_da_aparencia" "$REG" 1 \
  "            .filter(|m| m.outputs.iter().any(emite_posicoes) && !m.inputs.iter().any(recebe))" \
  "            .filter(|m| m.outputs.iter().any(emite_posicoes) && m.inputs.iter().any(recebe))"

bloco "a passagem deixa de marcar ninguem (o piso de populacao)" \
  ph2d-app-motion "lei_da_aparencia" "$REG" 1 \
  "        for id in fontes {
            self.so_posicoes.insert(id);
        }" \
  "        let _ = fontes;"

echo
echo "=== ELO 2 — a ORDEM em que ela corre ==="

bloco "a classificacao passa a ser ADITIVA sem cerca (a ordem deixa de decidir)" \
  ph2d-node-registry "a_classificacao_das_fontes" "$REG" 1 \
  "            .filter(|id| !self.is_object_source(*id) && !self.is_live_vector_source(*id))" \
  "            .filter(|id| { let _ = id; true })"

echo
echo "=== ELO 3 — quem VESTE, e o alcance a jusante ==="

bloco "o duplicador deixa de se declarar (nada veste: o aviso acende no grafo certo)" \
  ph2d-app-motion "a_lei_chega_ao_sistema_de_alerta" "$DUP" 1 \
  "    reg.register_veste_as_posicoes(MANIFEST.id);" \
  "    let _ = &reg;"

bloco "o alcance responde SEMPRE sim (o aviso nunca acende)" \
  ph2d-app-motion "a_lei_chega_ao_sistema_de_alerta" "$DIAG" 1 \
  "    let mut seen = BTreeSet::new();
    seen.insert(from);
    let mut stack = vec![from];
    while let Some(n) = stack.pop() {
        for e in graph.edges() {
            if e.from.0 == n && !e.delayed && seen.insert(e.to.0) {
                if carrega.contains(&e.to.0) {
                    return true;
                }
                stack.push(e.to.0);
            }
        }
    }
    false" \
  "    let _ = (graph, from, &carrega);
    true"

bloco "o FECHO para a frente cai (a juncao com um IRMAO passa a acusar)" \
  ph2d-app-motion "a_lei_chega_ao_sistema_de_alerta" "$DIAG" 1 \
  "    let mut cresceu = true;
    while cresceu {
        cresceu = false;
        for e in graph.edges() {
            if !e.delayed && carrega.contains(&e.from.0) && carrega.insert(e.to.0) {
                cresceu = true;
            }
        }
    }" \
  ""

echo
echo "=== ELO 4 — o SISTEMA DE ALERTA ==="

bloco "o deficit nunca e' emitido (a ordem do dono some sem um erro)" \
  ph2d-app-motion "a_lei_chega_ao_sistema_de_alerta" "$DIAG" 1 \
  "        if reg.so_posicoes(ty) && !arte_alcancavel(graph, reg, inst.id) {" \
  "        if false && reg.so_posicoes(ty) && !arte_alcancavel(graph, reg, inst.id) {"

bloco_it "o censo de SETUP deixa de filtrar (107 cenas voltam a ler-se como buracos)" \
  ph2d-motion-diagnose "a_nota_das_posicoes_nao_e_um_buraco" "$DIAG" 1 \
  "        .filter(|d| d.deficit != Deficit::SemQuemVista)" \
  "        .filter(|d| { let _ = d; true })"

bloco "a frase do toast cai no catch-all (o artista le' outro defeito)" \
  ph2d-app-motion "every_deficit_has_its_own_advisory" "$HEAL" 1 \
  "        (Deficit::SemQuemVista, _) => ph2d_i18n::tr(" \
  "        (Deficit::SemQuemVista, _) if false => ph2d_i18n::tr("

echo
echo "=== ELO 5 — a MARCA que se desenha no lugar do quadrado ==="

bloco_nao_compila "a marca volta a medir uma COPIA (o report do dono reabre)" \
  ph2d-render "$STYLE" 1 \
  "pub const PONTO_DO_TAMANHO: [f32; 2] = [0.15, 0.15];" \
  "pub const PONTO_DO_TAMANHO: [f32; 2] = [1.0, 1.0];"

bloco "a porta da marca nunca troca nada (o quad branco volta)" \
  ph2d-render "a_marca_troca_as_omissoes" "$STYLE" 1 \
  "    if style.so_com_forma && !tem_aparencia {
        (style.ponto_uv, PONTO_DO_TAMANHO)
    } else {
        (uv, size)
    }" \
  "    let _ = (style, tem_aparencia);
    (uv, size)"

bloco "a porta ignora a APARENCIA (toda sprite perde o seu ladrilho)" \
  ph2d-eval-motion "lower_tests" "$STYLE" 1 \
  "    if style.so_com_forma && !tem_aparencia {" \
  "    if style.so_com_forma {"

bloco "a lei volta a shipar DESLIGADA (o grid volta aos quadrados)" \
  ph2d-eval-motion "a_porta" "$ORDEM" 1 \
  '    !matches!(v, Some("0"))' \
  '    matches!(v, Some(x) if !x.is_empty() && x != "0")'

bloco_it "a shell esquece de dar o ladrilho a' bomba (a marca vira o atlas INTEIRO)" \
  ph2d-host-desktop "o_ladrilho_da_marca" "$SHELL_INIT" 1 \
  "            m.pump.define_o_ladrilho_do_ponto(motion_ponto_uv);" \
  "            let _ = motion_ponto_uv;"

bloco "o ladrilho do ponto vira um QUADRADO cheio" \
  ph2d-render "o_ponto_e_redondo" "$DOT" 1 \
  "            let cobertura = ((raio - d) / RAMPA_TEXELS).clamp(0.0, 1.0);" \
  "            let cobertura = 1.0_f32.min((raio - d.min(0.0)) / RAMPA_TEXELS).clamp(0.0, 1.0);"

echo
echo "── TOTAL: $TOTAL provas · $FALHAS falha(s)"
[ "$FALHAS" = 0 ]
