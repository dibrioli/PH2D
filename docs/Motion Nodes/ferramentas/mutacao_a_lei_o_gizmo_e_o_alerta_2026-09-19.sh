#!/usr/bin/env bash
# Provas de mutação do AVISO DE VISIBILIDADE (ordem do dono, 2026-09-19:
# «coloque um alerta de que se não forem usados com duplicator e um objeto a ser copiado,
# são invisíveis»).
#
# A corrente atravessa SEIS elos, e cada bloco mata um:
#
#   manifesto  →  a REGRA (marca_as_fontes_de_posicoes)      · os gates do app-motion
#              →  a BANDEIRA no registo                       · idem
#              →  quem VESTE, e o alcance a jusante           · a_lei_chega_ao_sistema_de_alerta
#              →  o SISTEMA DE ALERTA (diagnose)              · idem + os do diagnosticador
#              →  a LEI que retira o quadrado, nas TRES rotas · lower (x2) + o DISPOSITIVO
#              →  o GIZMO que poe os pontos no lugar dele     · ponto_gizmo + o pintor
#
# ⚠️⚠️ **Os elos 3 e 4 mudaram em 2026-09-19, por ordem do dono** (*«o módulo tem um sistema de
#    alerta. não era para colocar a mensagem no próprio nó»*): a nota permanente SAIU do cartão e
#    a mensagem passou a viajar pelo `ph2d-motion-diagnose`. Os blocos que mediam a vista do
#    cartão e a tinta da fileira **foram apagados com o sujeito deles**, não afrouxados.
# ⛔⛔ **E o elo 5 foi RE-ESCRITO no mesmo dia, pela mesma razão, um nível acima:** ele media uma
#    MARCA desenhada no canal do conteúdo (`PONTO_DO_TAMANHO`, `omissoes_da_marca`), que foi
#    construída, medida e REVERTIDA — *uma marca no canal do conteúdo herda os controlos do
#    conteúdo*, e multiplicar o `size` por `0,15` apagava a cena dos campos. O que ela mede hoje é
#    a lei a CORTAR (três rotas) e o GIZMO a pôr os pontos, que é chrome.
# ⚠️ **Um script de mutação cujo sujeito foi apagado NÃO fica verde — ele aborta na âncora**, e é
#    por isso que o `muta` conta as ocorrências: *um bloco que não encontra o que vai mutar
#    lê-se, num relatório, exactamente como um que sangrou.*
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
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_a_lei_o_gizmo_e_o_alerta_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

REG=crates/ph2d-node-registry/src/fontes_de_posicoes.rs
INIT=crates/ph2d-node-registry-init/src/lib.rs
DUP=crates/ph2d-node-motion-duplicator/src/lib.rs
DIAG=crates/ph2d-motion-diagnose/src/lib.rs
HEAL=crates/ph2d-app-motion/src/motion_bridge_heal.rs
ORDEM=crates/ph2d-eval-motion/src/sink_style.rs
LOWER=crates/ph2d-eval-motion/src/lower.rs
DEVICE=crates/ph2d-gpu-cook/src/instances.rs
GIZMO=crates/ph2d-app-motion/src/ponto_gizmo.rs
PINTOR=crates/ph2d-app-motion/src/ponto_gizmo_overlay.rs
ESTADO=crates/ph2d-app-motion/src/motion_state.rs

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
echo "=== ELO 5 — a LEI que retira o quadrado, nas TRES rotas ==="

# ⛔⛔ **As tres tem de sangrar em separado.** A do DISPOSITIVO e' a que o produto de facto corre
#    (o cozimento e' GPU-resident por omissao), e ate' 2026-09-19 ela **nao existia**: a lei
#    estava escrita so' na CPU, logo o report do dono — *«o grid continua desenhando quadrados»* —
#    reproduzia-se com os gates da CPU todos verdes. *Uma lei escrita numa rota so' e' uma lei que
#    o produto nao tem.*

bloco "a rota das SPRITES deixa de cortar (o quadrado volta na CPU)" \
  ph2d-eval-motion "lower" "$LOWER" 1 \
  "    if style.so_com_forma && !tem_aparencia(stream) {
        return;
    }
    let n = stream.count();" \
  "    let n = stream.count();"

bloco "a rota VECTORIAL deixa de cortar (a mesma posicao ganha DUAS pecas)" \
  ph2d-eval-motion "lower" "$LOWER" 1 \
  "    if style.so_com_forma && !tem_aparencia(stream) {
        return;
    }
    // ⚠️ **A saída cedo desapareceu de propósito:**" \
  "    // ⚠️ **A saída cedo desapareceu de propósito:**"

# ⚠️ **O filtro é `a_lei_do_dono` e NÃO `lower`** — e a troca nasceu de duas mutações que
#    SOBREVIVERAM: o `lower` casa 14 testes desta crate e **nenhum** deles nomeia a lei. *Um
#    filtro largo que casa muitos testes lê-se como cobertura e pode não cobrir nada.*

bloco_it "o DISPOSITIVO ignora a lei (a rota de OMISSAO volta ao report do dono)" \
  ph2d-gpu-cook "a_lei_do_dono" "$DEVICE" 1 \
  "    (!(style.so_com_forma && !tem_ladrilho), style)" \
  "    let _ = tem_ladrilho;
    (true, style)"

bloco_it "o dispositivo deixa de ver GEOMETRIA VIVA (toda forma do grafo desaparece)" \
  ph2d-gpu-cook "a_lei_do_dono" "$DEVICE" 1 \
  "    let style = if style.so_com_forma && tem_geometria {" \
  "    let style = if false {"

bloco "a lei volta a shipar DESLIGADA (o grid volta aos quadrados)" \
  ph2d-eval-motion "a_porta" "$ORDEM" 1 \
  '    !matches!(v, Some("0"))' \
  '    matches!(v, Some(x) if !x.is_empty() && x != "0")'

bloco "a lei EVAPORA ao carregar um ficheiro (o arnes passa a medir outro programa)" \
  ph2d-app-motion "lei_da_aparencia" "$ESTADO" 1 \
  "        let lei = self.pump.a_lei();
        self.pump = MotionCookPump::new();
        self.pump.define_a_lei(lei);" \
  "        self.pump = MotionCookPump::new();"

echo
echo "=== ELO 6 — o GIZMO que poe os pontos no lugar do quadrado ==="

# ⚠️ **Ele e' CHROME e nao conteudo**, e e' isso que o separa da MARCA que esta jornada construiu,
#    mediu e REVERTEU: uma marca desenhada no mesmo canal do conteudo herda os controlos do
#    conteudo (multiplicar o `size` por `0,15` apagava a cena dos campos, medido). O gizmo mede-se
#    em pixeis de ecra e nenhum no' do grafo lhe pode mexer no tamanho.

bloco "o gizmo nunca pede TOMADAS (nao ha' retrato: o ecra fica vazio)" \
  ph2d-app-motion "ponto_gizmo" "$GIZMO" 1 \
  "    if !so_com_forma {
        return Vec::new();
    }
    motion.sinks.clone()" \
  "    let _ = so_com_forma;
    Vec::new()"

bloco "a amostra vira PREFIXO (o `gap_y` volta a ler-se como uma faixa a voar)" \
  ph2d-app-motion "a_amostra_varre_a_nuvem" "$GIZMO" 1 \
  "        k * self.alcance / self.n" \
  "        k"

bloco "o tecto deixa de cortar (o custo do quadro passa a seguir a cena)" \
  ph2d-app-motion "o_tecto_corta" "$GIZMO" 1 \
  "            n: total.min(MAX_PONTOS)," \
  "            n: total,"

bloco "a CRUZ vira um anel (o report do dono — *«apenas pontos»* — reabre)" \
  ph2d-app-motion "uma_cruz_e_nao_um_anel" "$PINTOR" 1 \
  "        let braco = glifo_px(CRUZ_DA_PECA, pegada);" \
  "        let braco = 0.0;"

echo
echo "── TOTAL: $TOTAL provas · $FALHAS falha(s)"
[ "$FALHAS" = 0 ]
