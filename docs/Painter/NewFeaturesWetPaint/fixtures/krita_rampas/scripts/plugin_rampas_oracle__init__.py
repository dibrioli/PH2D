import os, traceback
from krita import Krita, Extension
from PyQt6.QtCore import QTimer

def _run():
    log = os.environ.get('RAMPAS_LOG', '/dev/null')
    try:
        src = open(os.environ['RAMPAS_SCRIPT']).read()
        g = {'__name__': '__rampas__', 'LOG': log}
        exec(compile(src, os.environ['RAMPAS_SCRIPT'], 'exec'), g)
    except Exception:
        with open(log, 'a') as f: f.write('DRIVER EXC\n' + traceback.format_exc())
    finally:
        with open(log + '.done', 'w') as f: f.write('done\n')

class RampasExt(Extension):
    def setup(self): pass
    def createActions(self, window):
        QTimer.singleShot(int(os.environ.get('RAMPAS_DELAY_MS', '6000')), _run)

Krita.instance().addExtension(RampasExt(Krita.instance()))
