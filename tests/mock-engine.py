"""Isolated contract mock. Never calls a DNS provider."""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os

values = {}
counts = {}
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass
    def do_GET(self):
        self.send_response(200)
        self.end_headers()
        self.wfile.write(json.dumps({"counts": counts}).encode())
    def do_POST(self):
        if self.headers.get("Authorization") != "Bearer " + os.environ["DDNSUPD_ENGINE_TOKEN"]:
            self.send_response(401); self.end_headers(); return
        data = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        if self.path == "/v1/discover":
            if data["token"].startswith("invalid-"):
                result = {"code":"auth"}; status = 422
            elif data["token"].startswith("forbidden-"):
                result = {"code":"permission"}; status = 422
            elif data["token"].startswith("ratelimit-"):
                result = {"code":"rate_limit","retry_after":600}; status = 422
            else:
                result = {"zones":[],"records":[],"page":data["page"],"next_page":None,"permission_verification":"read_only"}; status=200
                if data["operation"] == "zones":
                    if data["page"] == 1:
                        result["zones"] = [{"id":"testzone","name":"example.org","mode":"primary"}]
                        result["next_page"] = 2
                    else:
                        result["zones"] = [{"id":"zone2","name":"other.org","mode":"primary"}]
                else:
                    names = ["home","imported","no-family","expired","credential-change","race-a","import-a","import-b","race-b","same-preview","browser-import","protected"]
                    records = [{"name":name,"record_type":"A","record_id":"mock-"+name if data["provider"]=="cloudflare" else "","value":"8.8.8.8","ttl":300,"proxied":False,"bindable":True} for name in names]
                    if data["page"] == 1:
                        result["records"] = records[:6]; result["next_page"]=2
                    else:
                        result["records"] = records[6:]
            self.send_response(status); self.send_header("Content-Type","application/json"); self.end_headers(); self.wfile.write(json.dumps(result).encode()); return
        key = data["zone_id"] + ":" + data["name"] + ":" + data["record_type"]
        counts[key] = counts.get(key, 0) + 1
        if data["token"].startswith("invalid-"):
            result = {"code": "auth"}; status = 422
        elif data["name"] == "rate-limited":
            result = {"code": "rate_limit", "retry_after": 600}; status = 422
        elif data.get("create_missing"):
            if key in values:
                result = {"code": "record_exists"}; status = 422
            else:
                values[key] = data["desired"]
                result = {"before": "", "after": data["desired"], "changed": True, "verified": True}; status = 200
        elif data["name"] == "missing-aaaa" and key not in values:
            result = {"code": "not_found"}; status = 422
        else:
            before = values.get(key, data["managed"] or "1.1.1.1")
            values[key] = data["desired"]
            result = {"before": before, "after": data["desired"], "changed": before != data["desired"], "verified": True}; status = 200
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(result).encode())

ThreadingHTTPServer(("0.0.0.0", 8090), Handler).serve_forever()
