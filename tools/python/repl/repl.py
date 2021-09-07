import code
import readline
import rlcompleter
import sys

# https://docs.python.org/3/library/code.html

vs = globals()
vs.update(locals())
readline.set_completer(rlcompleter.Completer(vs).complete)
if sys.platform.startswith('linux'):
    print("use linux...")
    readline.parse_and_bind('tab: complete')
elif sys.platform == 'darwin':
    print("use darwin...")
    readline.parse_and_bind('bind ^I rl_complete')

code.InteractiveConsole(vs).interact()
