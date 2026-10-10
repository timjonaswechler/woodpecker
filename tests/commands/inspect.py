"""Resource and entity queries must inspect a real world without advancing it."""
from tests.catalog import scenario

COMMAND = "inspect.query"


@scenario("inspect", "resource_projections_and_missing", commands=(COMMAND,))
def resource_projections_and_missing(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        result = driver.command(COMMAND, {
            "source": "resources", "selector": {"kind": "type", "type_path": ctx.app.type_path},
            "projection": {"kind": "metadata"}})
        assert result == {"items": [{"kind": "resource", "result": {
            "kind": "metadata", "type_path": ctx.app.type_path}}]}, result
        all_resources = driver.command(COMMAND, {
            "source": "resources", "selector": {"kind": "all"}, "projection": {"kind": "metadata"}})
        assert ctx.app.type_path in {i["result"]["type_path"] for i in all_resources["items"]}
        driver.command(COMMAND, {
            "source": "resources", "selector": {"kind": "type", "type_path": "suite::DoesNotExist"},
            "projection": {"kind": "value"}}, expected_error="unknown_type_path")
        assert ctx.probe(driver) == before


@scenario("inspect", "entity_projections", commands=(COMMAND,), capabilities=("state", "entities"))
def entity_projections(ctx):
    with ctx.session() as driver:
        driver.command("tick.warp.start", {"ticks": 1})
        before = ctx.probe(driver)
        query = {"source": "entities", "entity": None, "with": [], "without": [],
                 "projection": {"kind": "summary"}}
        items = driver.command(COMMAND, query)["items"]
        entity_name = ctx.app.data["entity_name"]
        matches = [item for item in items if item["result"].get("name") == entity_name]
        assert len(matches) == 1, (entity_name, items)
        query["entity"] = matches[0]["entity"]
        query["projection"] = {"kind": "component_names"}
        result = driver.command(COMMAND, query)["items"]
        assert len(result) == 1 and result[0]["entity"] == query["entity"], result
        names = {c["type_path"] for c in result[0]["result"]["components"]}
        assert "bevy_ecs::name::Name" in names, names
        query["with"] = ["bevy_ecs::name::Name"]
        assert len(driver.command(COMMAND, query)["items"]) == 1
        query["without"] = ["bevy_ecs::name::Name"]
        assert driver.command(COMMAND, query)["items"] == []
        query["with"], query["without"] = [], []
        query["projection"] = {"kind": "hierarchy", "depth": 2}
        root = driver.command(COMMAND, query)["items"][0]["result"]["root"]
        assert root["entity"] == query["entity"] and root["name"] == entity_name, root
        query["projection"] = {"kind": "components", "selection": {
            "kind": "listed", "type_paths": ["bevy_ecs::name::Name"]}}
        components = driver.command(COMMAND, query)["items"][0]["result"]["components"]
        assert len(components) == 1 and components[0]["component"]["type_path"] == "bevy_ecs::name::Name", components
        assert components[0]["value"]["status"] == "readable", components
        query["projection"] = {"kind": "components", "selection": {"kind": "all"}}
        all_components = driver.command(COMMAND, query)["items"][0]["result"]["components"]
        assert any(c["component"]["type_path"] == "bevy_ecs::name::Name" for c in all_components)
        assert ctx.probe(driver) == before
