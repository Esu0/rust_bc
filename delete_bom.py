import os
import glob

def __main__():
    for p in glob.iglob("assets/org/unit/*/*/*.imgcut"):
        with open(p, mode="r", encoding="utf-8_sig") as f:
            buf = f.read()
        with open(p, mode="w", encoding="utf-8") as f:
            f.write(buf)
    for p in glob.iglob("assets/org/unit/*/*/*.mamodel"):
        with open(p, mode="r", encoding="utf-8_sig") as f:
            buf = f.read()
        with open(p, mode="w", encoding="utf-8") as f:
            f.write(buf)

if __name__ == "__main__":
    __main__()