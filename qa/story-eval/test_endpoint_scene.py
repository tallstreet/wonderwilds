"""Current endpoint payload and deterministic scene selection."""
import json
import unittest
import endpoint_scene as endpoint


class EndpointSceneTests(unittest.TestCase):
    def test_live_prompt_schema_and_privacy(self):
        row = {"scenarioId": "synthetic", "checkpoint": "NEVER_SEND", "reference": "NO_ANSWER",
               "input": {"mission": None, "context": {"locationId": "bridge_bank"},
                         "here": {"characters": [{"id": "pip"}]}},
               "presentation": {"sceneId": "scene", "requiredEntries": []}}
        request = endpoint.request_for(row)
        supplied = json.loads(request["messages"][1]["content"])
        self.assertEqual(supplied["allowedSpeakerIds"], ["pip"])
        self.assertEqual(request["response_format"], {"type": "json_schema", "json_schema": {
            "name": "StoryScene", "strict": True, "schema": supplied["outputSchema"]}})
        self.assertNotIn("NEVER_SEND", json.dumps(request))
        self.assertNotIn("NO_ANSWER", json.dumps(request))
        other = json.loads(json.dumps(row))
        other["presentation"]["requiredEntries"] = [{"entryId": "greeting"}]
        self.assertEqual(endpoint.select([row, row, other]), [row, other])


if __name__ == '__main__':
    unittest.main()
