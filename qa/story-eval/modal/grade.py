"""Grade downloaded evaluation results in a new local directory, without a GPU."""
import argparse
from pathlib import Path
import shutil

from eval_io import DEFAULT_RUN, grade_result

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--result", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--run", type=Path, default=Path(DEFAULT_RUN))
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False)
    for name in ("suite.json", "result.json"):
        shutil.copyfile(args.result / name, args.out / name)
    grade_result(args.out, args.run)
