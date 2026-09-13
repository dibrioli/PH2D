#!/usr/bin/env python3
"""Gera o `open_pbr_surface` com os TRÊS geradores GLSL do MaterialX instalado.

Medido em 2026-09-13 (docs/Render3d/04 §2): MaterialX 1.39.5 (Apache-2.0).

    python3 docs/Render3d/ferramentas/gerar_openpbr.py <dir_saida>

Escreve `opbr_{glsl,vk,wgsl}.{vert,frag}` e imprime, por gerador e estágio, as linhas e a
`#version`. ⚠️ O `WgslShaderGenerator` vive em `PyMaterialXGenGlsl` (não num módulo `GenWgsl`):
um censo por MÓDULO não o vê — foi assim que o doc 00 §3 concluiu «não há gerador de WGSL».
"""

import pathlib
import sys

import MaterialX as mx
from MaterialX import PyMaterialXGenGlsl as gg
from MaterialX import PyMaterialXGenShader as gs

out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
out.mkdir(parents=True, exist_ok=True)

sp = mx.FileSearchPath("/usr/share/materialx")
lib = mx.createDocument()
mx.loadLibraries(mx.getDefaultDataLibraryFolders(), sp, lib)
doc = mx.createDocument()
doc.importLibrary(lib)
shader = doc.addNode("open_pbr_surface", "SR_opbr", "surfaceshader")
material = doc.addMaterialNode("M_opbr", shader)
ok, msg = doc.validate()
assert ok, msg
print("MaterialX", mx.getVersionString())

for name, cls in [
    ("glsl", gg.GlslShaderGenerator),
    ("vk", gg.VkShaderGenerator),
    ("wgsl", gg.WgslShaderGenerator),
]:
    gen = cls.create()
    ctx = gs.GenContext(gen)
    ctx.registerSourceCodeSearchPath(sp)
    s = gen.generate("opbr_" + name, material, ctx)
    for stage, ext in [("vertex", "vert"), ("pixel", "frag")]:
        src = s.getSourceCode(stage)
        (out / f"opbr_{name}.{ext}").write_text(src)
        lines = src.splitlines()
        version = next((line for line in lines if line.startswith("#version")), "?")
        print(f"{name:5s} {stage:6s} linhas={len(lines):5d} {version}")
