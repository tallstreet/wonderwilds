"""Offline validation for the disabled authored personal-quest development catalog."""
from __future__ import annotations

import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
CATALOG = ROOT / "content/quests/catalog.v1.json"
ID = re.compile(r"^[a-z][a-z0-9-]{0,119}$")
COMPANIONS = {"pip": ("M06", "missing_quantity"), "tansy": ("M05", "pattern_sort"), "nib": ("M08", "equal_sharing"), "flutter": ("M09", "pattern_sort")}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def keys(value: object, names: set[str], label: str) -> dict:
    require(isinstance(value, dict) and set(value) == names, f"{label}: closed object fields required")
    return value


def text(value: object, maximum: int, label: str, identifier: bool = False) -> None:
    require(isinstance(value, str) and 0 < len(value) <= maximum and value.strip() == value, f"{label}: invalid bounded text")
    require(not any(ord(c) < 32 for c in value), f"{label}: control characters forbidden")
    if identifier:
        require(bool(ID.fullmatch(value)), f"{label}: invalid identifier")


def identifiers(value: object, minimum: int, maximum: int, label: str) -> None:
    require(isinstance(value, list) and minimum <= len(value) <= maximum, f"{label}: invalid list bound")
    for item in value:
        text(item, 120, label, True)
    require(len(set(value)) == len(value), f"{label}: duplicate identifiers")


def validate_document(document: dict, activity_slots: dict[str, dict]) -> None:
    keys(document, {"schemaVersion", "questId", "documentVersion", "title", "companionId", "regionId", "interestTags", "entryNodeId", "nodes"}, "QuestDocument")
    require(document["schemaVersion"] == "1.0", "unsupported quest schema")
    for key in ("questId", "companionId", "regionId", "entryNodeId"):
        text(document[key], 120, key, True)
    text(document["documentVersion"], 40, "documentVersion")
    text(document["title"], 120, "title")
    require(document["companionId"] in COMPANIONS, "unsupported development companion")
    require(document["regionId"] == "meadow", "development region unavailable")
    identifiers(document["interestTags"], 1, 8, "interestTags")
    require(isinstance(document["nodes"], list) and 3 <= len(document["nodes"]) <= 16, "quest node bounds")
    nodes, edges = {}, {}
    for node in document["nodes"]:
        require(isinstance(node, dict), "node must be an object")
        kind = node.get("type")
        base = {"id", "type", "text"}
        expected = {"choice": base | {"choices"}, "activity": base | {"activitySlotId", "nextNodeId"}, "handoff": base | {"continuationTags"}}
        require(kind in expected, "unsupported node type")
        keys(node, expected[kind], "QuestNode")
        text(node["id"], 120, "nodeId", True)
        text(node["text"], 600, "node text")
        require(node["id"] not in nodes, "duplicate node ID")
        nodes[node["id"]] = node
        outgoing = []
        if kind == "choice":
            require(isinstance(node["choices"], list) and 2 <= len(node["choices"]) <= 3, "choice count")
            choices, labels = set(), set()
            for choice in node["choices"]:
                keys(choice, {"id", "label", "nextNodeId"}, "QuestChoice")
                text(choice["id"], 120, "choiceId", True)
                text(choice["label"], 120, "choice label")
                text(choice["nextNodeId"], 120, "choice target", True)
                require(choice["id"] not in choices and choice["label"].casefold() not in labels, "duplicate choice")
                choices.add(choice["id"])
                labels.add(choice["label"].casefold())
                outgoing.append(choice["nextNodeId"])
        elif kind == "activity":
            slot = activity_slots.get(node["activitySlotId"])
            require(slot is not None, "unknown activity slot")
            require((slot["skillId"], slot["templateId"]) == COMPANIONS[document["companionId"]], "companion activity binding mismatch")
            text(node["nextNodeId"], 120, "activity target", True)
            outgoing.append(node["nextNodeId"])
        else:
            identifiers(node["continuationTags"], 1, 5, "continuationTags")
        edges[node["id"]] = outgoing
    require(document["entryNodeId"] in nodes, "missing entry node")
    require(nodes[document["entryNodeId"]]["type"] == "choice", "entry must offer a choice")
    require(all(target in nodes for targets in edges.values() for target in targets), "dangling node reference")
    visited, visiting = set(), set()

    def visit(node_id: str) -> None:
        require(node_id not in visiting, "quest graph cycle")
        if node_id in visited:
            return
        visiting.add(node_id)
        for target in edges[node_id]:
            visit(target)
        visiting.remove(node_id)
        visited.add(node_id)

    visit(document["entryNodeId"])
    require(visited == set(nodes), "unreachable node")
    checked = set()

    def check_paths(node_id: str, activity_seen: bool) -> None:
        state = (node_id, activity_seen)
        if state in checked:
            return
        checked.add(state)
        node = nodes[node_id]
        activity_seen = activity_seen or node["type"] == "activity"
        if node["type"] == "handoff":
            require(activity_seen, "handoff bypasses verified activity")
        for target in edges[node_id]:
            check_paths(target, activity_seen)

    check_paths(document["entryNodeId"], False)
    require(any(node["type"] == "handoff" for node in nodes.values()), "missing continuation handoff")


def validate_catalog(catalog: dict, documents: dict[str, dict]) -> None:
    keys(catalog, {"schemaVersion", "contentVersion", "reviewStatus", "releaseEnabled", "activitySlots", "quests"}, "QuestCatalog")
    require(catalog["schemaVersion"] == "1.0" and catalog["contentVersion"] == "personal-quests-1.0.0", "unsupported catalog version")
    require(catalog["reviewStatus"] == "pending" and catalog["releaseEnabled"] is False, "development content must remain disabled and pending review")
    require(isinstance(catalog["activitySlots"], list) and len(catalog["activitySlots"]) == 4, "four authored activity slots required")
    slots = {}
    for slot in catalog["activitySlots"]:
        keys(slot, {"id", "skillId", "templateId"}, "QuestActivitySlot")
        text(slot["id"], 120, "slotId", True)
        require(slot["id"] not in slots, "duplicate activity slot")
        require((slot["skillId"], slot["templateId"]) in COMPANIONS.values(), "unsupported slot binding")
        slots[slot["id"]] = slot
    require(isinstance(catalog["quests"], list) and len(catalog["quests"]) == 4, "four personal quests required")
    ids, rewards, companions, paths = set(), set(), set(), set()
    for entry in catalog["quests"]:
        keys(entry, {"questId", "documentPath", "minimumCharacterLevel", "rewardItemId"}, "QuestCatalogEntry")
        text(entry["questId"], 120, "catalog questId", True)
        text(entry["rewardItemId"], 120, "rewardItemId", True)
        require(entry["minimumCharacterLevel"] == 4, "personal quests unlock at character level four")
        expected = f'content/quests/{entry["questId"]}.v1.json'
        require(entry["documentPath"] == expected, "unsafe or mismatched document path")
        require(entry["questId"] not in ids and entry["rewardItemId"] not in rewards and expected not in paths, "duplicate catalog entry/reward")
        ids.add(entry["questId"]); rewards.add(entry["rewardItemId"]); paths.add(expected)
        require(expected in documents, "missing quest document")
        document = documents[expected]
        validate_document(document, slots)
        require(document["questId"] == entry["questId"], "catalog/document identity mismatch")
        require(document["companionId"] not in companions, "duplicate companion personal quest")
        companions.add(document["companionId"])
    require(companions == set(COMPANIONS), "incomplete Meadow personal quests")
    require(paths == set(documents), "uncatalogued quest documents")


def load() -> tuple[dict, dict[str, dict]]:
    catalog = json.loads(CATALOG.read_text())
    documents = {entry["documentPath"]: json.loads((ROOT / entry["documentPath"]).read_text()) for entry in catalog["quests"]}
    return catalog, documents


if __name__ == "__main__":
    validate_catalog(*load())
    print("Personal quests valid: four branching episodes with continuing handoffs; release remains disabled.")
