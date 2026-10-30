"""Graph analysis and progression simulator for adaptive curriculum."""
import json
from collections import defaultdict, deque
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CURRICULUM_ROOT = ROOT.parent

def build_graph():
    foundation = json.loads((CURRICULUM_ROOT / "seed-v1.json").read_text())["nodes"]
    years = json.loads((ROOT / "seed-years1to6-v1.json").read_text())["nodes"]
    all_nodes = {n["id"]: n for n in foundation + years}

    # adjacency
    children = defaultdict(list)
    parents = defaultdict(list)
    for nid, node in all_nodes.items():
        for pre in node["prerequisites"]:
            children[pre].append(nid)
            parents[nid].append(pre)

    return all_nodes, children, parents

def simulate_progression(subject="maths"):
    all_nodes, children, parents = build_graph()
    subject_nodes = {nid: n for nid, n in all_nodes.items() if n["subject"] == subject}

    # entry points (no prerequisites in the subject or in general)
    mastered = set()
    available = set()

    for nid, node in subject_nodes.items():
        if not node["prerequisites"]:
            available.add(nid)

    print(f"=== Adaptive Graph Progression for {subject.upper()} ===")
    print(f"Total nodes: {len(subject_nodes)}")
    print(f"Starting entry points: {sorted(list(available))}")

    step = 1
    while available:
        # simulate mastering current available tier
        tier = sorted(list(available))
        print(f"Step {step}: Available to learn -> {tier}")
        # master all current tier
        for n in tier:
            mastered.add(n)
        available.clear()

        # find newly unlocked
        for nid, node in subject_nodes.items():
            if nid not in mastered:
                if all(p in mastered for p in node["prerequisites"]):
                    available.add(nid)
        step += 1

    unreached = set(subject_nodes) - mastered
    if unreached:
        print(f"WARNING: Unreached nodes: {unreached}")
    else:
        print(f"All {len(subject_nodes)} nodes fully reachable across {step-1} learning progression waves!")

if __name__ == "__main__":
    simulate_progression("maths")
    print()
    simulate_progression("english")
