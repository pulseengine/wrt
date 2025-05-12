import os, tomli, pathlib
from .base_dag import workflow

HERE = pathlib.Path(__file__).resolve().parent
CFG  = tomli.loads((HERE.parent / "config/ai.toml").read_text())
PROFILE = "component"  # change in the other flow files
cfg = CFG[PROFILE]

state = {"ticket": os.getenv("TICKET", "local"), **cfg}
workflow.invoke(state) 