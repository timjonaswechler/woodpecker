#!/usr/bin/env python3
"""CPU-only verification of committed evidence or a separately authorized local run."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess

from png_oracle import decode

SRC = Path(__file__).resolve().parent
ROOT = SRC.parents[2]
OUT = None


def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def events(p):
    rows=[]
    for line in p.read_text().splitlines():
        if line.startswith("{"):
            try:row=json.loads(line)
            except json.JSONDecodeError:continue
            if "event" in row and "data" in row:rows.append(row)
    return rows

def one(rows,name):
    a=[x for x in rows if x["event"]==name];assert len(a)==1,(name,len(a));return a[0]

def verify_run():
    ledger=json.loads((OUT/"PROCESSES.json").read_text());assert len(ledger)==15
    assert [x["number"] for x in ledger]==list(range(1,16))
    assert all(x["exit_code"]==0 and not x["timeout"] and x["reaped"] and x["seconds"]<120 for x in ledger)
    results={};all_pngs=[];committed=[];chains=[]
    selected=("valid_shader_module_accepted","actual_output_validation","actual_failed_map_callback","actual_map_failure_verified",
              "publisher_accepted","publisher_stale_rejected","publisher_stage_barrier","publisher_final_check",
              "publisher_terminal","publisher_observed_fatal","publisher_success","publisher_no_success_proven",
              "publisher_io_error","partial_write_fault_injection","fault_api_enter","backend_error_callback_log",
              "public_render_error_handler","public_pre_extract_enter","public_pre_extract_return")
    for item in ledger:
        log=ROOT/item["log"];folder=log.parent;rows=events(log)
        assert [r["seq"] for r in rows]==sorted(set(r["seq"] for r in rows))
        assert [r["t_ns"] for r in rows]==sorted(r["t_ns"] for r in rows)
        wire=[]
        for line in log.read_text().splitlines():
            if line.startswith('{"status":'):wire.append(json.loads(line))
        assert [w for w in wire if w["status"]=="completed"]==[{"status":"completed","request_id":1,"command":"tick.warp.start","output":{"executed_ticks":1,"outcome":"completed","requested_ticks":1}}]
        ready=[w for w in wire if w["status"]=="ready"];assert len(ready)==1 and ready[0]["capabilities"]["screenshot"] is False
        end=one(rows,"publication_case_complete")["data"];s=end["snapshot"]
        assert s["ticks"]==s["updates"]==s["fixed"]==s["startup"]==1
        assert s["real_ns"]==s["virtual_ns"]==s["real_delta_ns"]==16_666_667
        assert s["pending_input"]==7 and s["consumed_input"]==0
        assert end["render"]["startup"]==1 and end["pumps"]<=64
        assert end["chain"]["original"]==end["chain"]["extracts"]==end["chain"]["renders"]==end["chain"]["tracked_clones"]
        if item["number"] in (1,2):
            h=one(rows,"healthy_window_verified")["data"];a,b=h["before"],h["after"]
            for key in ("original","renders","extracts","tracked_clones","image_modified"):assert b[key]-a[key]==3
            assert b["explicit_hook"]-a["explicit_hook"]==(3 if item["number"]==2 else 0)
            assert a["entities"][0][:2]==b["entities"][0][:2] and b["entities"][0][2]==3
            chains.append(h)
        if item["number"]==3:
            one(rows,"valid_shader_module_accepted");e=one(rows,"actual_output_validation")["data"]["error"]
            assert "create_render_pipeline" in e and "Uint32x4" in e and "Float32x4" in e and "parsing error" not in e
        if item["number"]==4:assert "Render pipeline must be set" in one(rows,"actual_output_validation")["data"]["error"]
        if item["number"]==5:
            e=one(rows,"actual_output_validation")["data"]["error"];assert "24836 bytes" in e and "size 16" in e
        if item["number"]==6:
            e=one(rows,"actual_map_failure_verified")["data"];assert e["result"]=="Err(BufferAsyncError)" and not e["cpu_fault_injection"] and "has been destroyed" in e["scoped_error"]
        successes=[r for r in rows if r["event"]=="publisher_success"]
        if item["number"] in (7,10,11):
            assert len(successes)==1
            success=successes[0]["data"];destination=Path(success["destination"]);r=success["receipt"]
            assert destination.is_dir() and set(p.name for p in destination.iterdir())=={"frame.png","receipt.json"}
            assert json.loads((destination/"receipt.json").read_text())==r
            pixels=decode(destination/"frame.png")
            assert hashlib.sha256(pixels).hexdigest()==r["pixels_sha256"] and sha(destination/"frame.png")==r["png_sha256"]
            accepted=one(rows,"publisher_accepted")["data"]
            assert r["request"]==accepted["request"] and r["generation"]==accepted["generation"] and r["frame"]>=accepted["minimum_frame"] and r["target"]==accepted["target"]
            matching=[x for x in rows if x["event"]=="mapped_candidate_NOT_receipt" and x["data"]["token"]=={k:r[k] for k in ("request","generation","frame","tick","state")}]
            assert len(matching)==1 and matching[0]["data"]["sha256"]==r["pixels_sha256"] and matching[0]["data"]["queue_callback"]
            copies=[x for x in rows if x["event"]=="graph_final_copy_submitted" and x["data"]["token"]["frame"]==r["frame"]]
            assert len(copies)==1 and copies[0]["data"]["submission"]==r["submission"]
            assert r["map_result"]=="Ok" and r["tick"]==1 and r["width"]==65 and r["height"]==49 and r["stride"]==512
            committed.append(str(destination.relative_to(ROOT)))
        else:
            assert not successes
            if item["number"]>2:one(rows,"publisher_no_success_proven")
        if item["number"]==10:
            assert [x["data"]["reason"] for x in rows if x["event"]=="publisher_stale_rejected"]==["request","generation"]
        if item["number"]==11:assert one(rows,"publisher_stale_rejected")["data"]["reason"]=="frame"
        if item["number"]==12:
            assert one(rows,"publisher_stage_barrier")["seq"]<one(rows,"public_render_error_handler")["seq"]<one(rows,"publisher_observed_fatal")["seq"]<one(rows,"publisher_no_success_proven")["seq"]
            assert one(rows,"public_render_error_handler")["data"]["type"]=="DeviceLost"
        if item["number"]==13:
            assert one(rows,"publisher_stage_barrier")["seq"]<one(rows,"publisher_terminal")["seq"]<one(rows,"publisher_no_success_proven")["seq"]
        if item["number"]==14:
            partial=folder/"publisher/.staging-800-1/frame.png";assert partial.read_bytes()==b"\x89PNG\r\n\x1a\n"
            one(rows,"partial_write_fault_injection")
        if item["number"]==15:
            assert (folder/"publisher/result-800-1").read_bytes()==b"OWN_SENTINEL_DO_NOT_REPLACE"
            assert one(rows,"publisher_io_error")["data"]["operation"]=="rename"
        for path in folder.rglob("*.png"):
            if "source" in path.relative_to(folder).parts:continue
            if item["number"]==14 and ".staging" in str(path):
                all_pngs.append({"path":str(path.relative_to(ROOT)),"kind":"intentionally partial nonfinal PNG prefix","bytes":8,"sha256":sha(path)});continue
            pixels=decode(path);colors=Counter(zip(*[iter(pixels)]*4));assert all(c[3]==255 for c in colors)
            if path.name!="first-candidate.png":
                assert colors[(255,0,0,255)]==320
                for c in ((0,0,255,255),(255,255,0,255),(0,255,255,255),(255,0,255,255)):assert colors[c]==64
            all_pngs.append({"path":str(path.relative_to(ROOT)),"kind":"decoded actual PNG","sha256":sha(path),"pixels_sha256":hashlib.sha256(pixels).hexdigest()})
        for name in ("fixture.rs","study_base.rs","publication.rs","cases.rs","main.rs"):
            assert (folder/"source/src"/name).read_bytes()==(SRC/"src"/name).read_bytes(),"executed source changed"
        results[item["case"]]={"snapshot":s,"render":end["render"],"chain":end["chain"],"success_count":len(successes),
            "events":[x for x in rows if x["event"] in selected]}
    actual=sorted(str(p.relative_to(ROOT)) for p in OUT.glob("*/publisher/result-*") if p.is_dir())
    assert actual==sorted(committed) and len(committed)==3
    assert len(chains)==2
    summary={"processes":15,"all_exit0":True,"retries":0,"case_limit":15,"execution_source_snapshots_match":True,
        "decoded_pngs":sum(x["kind"]=="decoded actual PNG" for x in all_pngs),"intentional_partial_nonfinal_pngs":1,
        "committed_bundles":committed,"healthy_windows":chains,"general_headless_PASS":False}
    (OUT/"SUMMARY.json").write_text(json.dumps(summary,indent=2)+"\n")
    (OUT/"RESULTS.json").write_text(json.dumps({"summary":summary,"results":results,"images":all_pngs},indent=2)+"\n")
    provenance={"sources":{},"product_and_dependencies":{}}
    for p in SRC.rglob("*"):
        if p.is_file():provenance["sources"][str(p.relative_to(ROOT))]=sha(p)
    for name in ("Cargo.toml","Cargo.lock","src/session/plugin.rs","src/session/plugin/render.rs","src/session/screenshot/destination.rs"):
        provenance["product_and_dependencies"][name]=sha(ROOT/name)
    for dep,files in {"bevy_app-0.20.0":("src/app.rs",),"bevy_extract-0.20.0":("src/extract_plugin.rs","src/extract_component.rs"),
        "bevy_render-0.20.0":("src/error_handler.rs",),"wgpu-30.0.1":("src/api/buffer.rs","src/api/device.rs"),
        "wgpu-core-30.0.1":("src/device/queue.rs",)}.items():
        base=next((Path.home()/".cargo/registry/src").glob("*/"+dep))
        for name in files:provenance["product_and_dependencies"][str(base/name)]=sha(base/name)
    graph=subprocess.check_output(["cargo","tree","--offline","--locked","--manifest-path",str(SRC/"Cargo.toml"),"--prefix","none"],cwd=ROOT,text=True)
    assert not any(x.startswith(("winit ","bevy_clipboard ","arboard ")) for x in graph.splitlines())
    (OUT/"FEATURE_GRAPH.txt").write_text(graph)
    provenance["feature_graph_sha256"]=sha(OUT/"FEATURE_GRAPH.txt")
    provenance["binary_sha256"]=ledger[0]["binary_sha256"];assert all(x["binary_sha256"]==provenance["binary_sha256"] for x in ledger)
    for cmd in (("rustc","--version"),("cargo","--version"),("sw_vers",)):
        provenance[cmd[0]]=subprocess.check_output(cmd,text=True).strip()
    provenance["head"]=subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()
    (OUT/"PROVENANCE.json").write_text(json.dumps(provenance,indent=2)+"\n")
    print(json.dumps(summary,indent=2))


def verify_recorded():
    root = SRC / "evidence"
    record = json.loads((root / "recorded.json").read_text())
    assert record["schema"] == 1
    for name, digest in record["publication_source_sha256"].items():
        assert sha(SRC / name) == digest, f"published execution source changed: {name}"
        original = (SRC / name).read_bytes() + b"\n" * record["removed_trailing_newlines"].get(name, 0)
        assert hashlib.sha256(original).hexdigest() == record["execution_source_sha256"][name], f"original source not reconstructible: {name}"
    for name, digest in record["sample_file_sha256"].items():
        assert sha(root / name) == digest, f"sample changed: {name}"
    outcomes = record["process_outcomes"]
    assert len(outcomes) == 15
    completion = {"status": "completed", "request_id": 1, "command": "tick.warp.start",
                  "output": {"executed_ticks": 1, "outcome": "completed", "requested_ticks": 1}}
    assert all(r["exit_code"] == 0 and not r["timeout"] and r["reaped"]
               and r["warp_completions"] == [completion] for r in outcomes)
    summary = record["latest"]["summary"]
    assert summary["processes"] == 15 and summary["decoded_pngs"] == 52
    assert summary["general_headless_PASS"] is False
    results = record["latest"]["results"]
    assert len(results) == 15
    pipeline = one(results["pipeline_validation"]["events"], "actual_output_validation")["data"]["error"]
    assert "create_render_pipeline" in pipeline and "Uint32x4" in pipeline and "parsing error" not in pipeline
    assert "Render pipeline must be set" in one(results["draw_submit_validation"]["events"], "actual_output_validation")["data"]["error"]
    assert "24836 bytes" in one(results["copy_validation"]["events"], "actual_output_validation")["data"]["error"]
    assert one(results["map_destroyed_buffer"]["events"], "actual_map_failure_verified")["data"]["result"] == "Err(BufferAsyncError)"
    successful = {name for name, result in results.items() if result["success_count"]}
    assert successful == {"publisher_healthy", "stale_request_generation", "stale_frame"}
    assert [x["data"]["reason"] for x in results["stale_request_generation"]["events"] if x["event"] == "publisher_stale_rejected"] == ["request", "generation"]
    assert one(results["stale_frame"]["events"], "publisher_stale_rejected")["data"]["reason"] == "frame"
    fatal_rows = results["observed_device_fatal_before_commit"]["events"]
    assert one(fatal_rows, "public_render_error_handler")["seq"] < one(fatal_rows, "publisher_observed_fatal")["seq"] < one(fatal_rows, "publisher_no_success_proven")["seq"]
    for h in summary["healthy_windows"]:
        before, after = h["before"], h["after"]
        for key in ("original", "renders", "extracts", "tracked_clones", "image_modified"):
            assert after[key] - before[key] == 3
        assert after["startup"] == 1 and h["simulation"]["updates"] == 1
    a, b, paused = (decode(root / name) for name in ("a.png", "b.png", "b-paused.png"))
    assert a != b and b == paused
    assert sum(a[i:i+4] != b[i:i+4] for i in range(0, len(a), 4)) == 640
    assert set(zip(*[iter(decode(root / "early-black.png"))] * 4)) == {(0, 0, 0, 255)}
    published = decode(root / "published-frame.png")
    receipt = json.loads((root / "published-receipt.json").read_text())
    assert hashlib.sha256(published).hexdigest() == receipt["pixels_sha256"]
    assert sha(root / "published-frame.png") == receipt["png_sha256"]
    assert receipt["request"] == 800 and receipt["generation"] == 1 and receipt["tick"] == 1
    assert (root / "partial-staging-prefix.bin").read_bytes() == b"\x89PNG\r\n\x1a\n"
    # The recorded marker identifies export/packaging, not a fresh GPU qualification.
    assert "not a new run" in record["scope"]
    return {"recorded_cases": 15, "verified_sample_files": len(record["sample_file_sha256"]),
            "historical_png_audit_count": 52, "new_gpu_execution": False}


def cli(argv=None):
    global OUT
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, help="local run directory below this checkout's target/")
    args = parser.parse_args(argv)
    if args.run is None:
        print(json.dumps(verify_recorded(), indent=2))
    else:
        OUT = args.run.resolve()
        if not OUT.is_relative_to((ROOT / "target").resolve()):
            parser.error("--run must be below this checkout's target/")
        verify_run()


if __name__ == "__main__":
    cli()
