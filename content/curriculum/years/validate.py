"""Validate Years 1-6 curriculum graph integrity and cross-pack prerequisites."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CURRICULUM_ROOT = ROOT.parent

def load_bundles():
    # Load Foundation seed to resolve foundation prerequisites
    foundation_seed = json.loads((CURRICULUM_ROOT / "seed-v1.json").read_text())
    foundation_ids = {n["id"]: n for n in foundation_seed["nodes"]}

    years_seed = json.loads((ROOT / "seed-years1to6-v1.json").read_text())
    years_authoring = json.loads((ROOT / "authoring-years1to6-v1.json").read_text())
    years_reviews = json.loads((ROOT / "reviews-years1to6-v1.json").read_text())

    # Policies from root
    policies = json.loads((CURRICULUM_ROOT / "evidence-policies-v1.json").read_text())["policies"]

    return foundation_ids, years_seed, years_authoring, years_reviews, policies

def validate():
    foundation_map, years_seed, years_authoring, years_reviews, policies = load_bundles()
    errors = []

    nodes = years_seed["nodes"]
    node_map = {n["id"]: n for n in nodes}
    if len(node_map) != len(nodes):
        errors.append("Duplicate skill IDs in years1to6 seed")

    # Authoring & Review maps
    author_map = {item["skillId"]: item for item in years_authoring["skills"]}
    review_map = {item["skillId"]: item for item in years_reviews["reviews"]}

    if set(author_map) != set(node_map):
        errors.append(f"Authoring mismatch: {set(node_map) ^ set(author_map)}")
    if set(review_map) != set(node_map):
        errors.append(f"Reviews mismatch: {set(node_map) ^ set(review_map)}")

    policy_map = {p["id"]: p for p in policies}

    # Combined map for prerequisite cycle and existence checking
    all_nodes = {**foundation_map, **node_map}

    visiting, visited = set(), set()
    def check_cycle(skill_id):
        if skill_id in visiting:
            errors.append(f"Prerequisite cycle detected at {skill_id}")
            return
        if skill_id in visited or skill_id not in all_nodes:
            return
        visiting.add(skill_id)
        for pre in all_nodes[skill_id]["prerequisites"]:
            check_cycle(pre)
        visiting.remove(skill_id)
        visited.add(skill_id)

    expected_bands = {f"year-{i}" for i in range(1, 7)}

    for node in nodes:
        skill_id = node["id"]
        if node["band"] not in expected_bands:
            errors.append(f"{skill_id}: unexpected band {node['band']}")
        if node["subject"] not in {"maths", "english"}:
            errors.append(f"{skill_id}: invalid subject {node['subject']}")
        if (node["subject"] == "maths" and not skill_id.startswith("M")) or \
           (node["subject"] == "english" and not skill_id.startswith("E")):
            errors.append(f"{skill_id}: ID prefix does not match subject")

        # Check prerequisites
        for pre in node["prerequisites"]:
            if pre not in all_nodes:
                errors.append(f"{skill_id}: unknown prerequisite {pre}")
        if len(node["prerequisites"]) != len(set(node["prerequisites"])):
            errors.append(f"{skill_id}: duplicate prerequisites")

        if node["evidencePolicyId"] not in policy_map:
            errors.append(f"{skill_id}: unknown evidence policy {node['evidencePolicyId']}")

        if len(node.get("hintLadder", [])) != 3:
            errors.append(f"{skill_id}: hintLadder must have exactly 3 rungs")

        # Bounds
        for b_name, b_val in node["parameterBounds"].items():
            if not (0 <= b_val["min"] <= b_val["max"] <= 1000000):
                errors.append(f"{skill_id}: parameter bound {b_name} invalid")

        # Difficulty ladder in authoring
        auth = author_map.get(skill_id)
        if auth:
            rungs = auth.get("difficultyLadder", [])
            if len(rungs) != 3:
                errors.append(f"{skill_id}: difficultyLadder must have 3 rungs")
            ctx_ids = {c["id"] for c in auth.get("contexts", [])}
            if len(ctx_ids) < 2:
                errors.append(f"{skill_id}: requires at least 2 distinct contexts")
            for rung in rungs:
                if rung["contextId"] not in ctx_ids:
                    errors.append(f"{skill_id}: rung references unknown context {rung['contextId']}")
                if rung["representation"] not in node["allowedRepresentations"]:
                    errors.append(f"{skill_id}: rung representation {rung['representation']} not in allowedRepresentations")
                if rung.get("supportAvailable") is not True:
                    errors.append(f"{skill_id}: supportAvailable must be true")

        check_cycle(skill_id)

    # Check entry points exist
    entry_nodes = [n["id"] for n in nodes if not n["prerequisites"]]
    math_entries = [eid for eid in entry_nodes if eid.startswith("M")]
    eng_entries = [eid for eid in entry_nodes if eid.startswith("E")]
    print(f"Validated {len(nodes)} Year 1-6 nodes.")
    print(f"Math entry points: {math_entries}")
    print(f"English entry points: {eng_entries}")

    return errors

if __name__ == "__main__":
    errs = validate()
    if errs:
        print(f"FAILED with {len(errs)} errors:")
        for e in errs:
            print(f" - {e}")
        exit(1)
    else:
        print("Years 1-6 curriculum graph validation PASSED successfully.")
