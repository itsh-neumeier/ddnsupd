"""Discovery and preview/confirm integration tests. Use only the isolated QA stack."""
import concurrent.futures
import http.cookiejar
import json
import os
from pathlib import Path
import secrets
import subprocess
import time
import urllib.error
import urllib.request

fixture=json.loads(Path(".local/qa-credentials.json").read_text())
base=fixture["url"]
compose=["docker","compose","--env-file",".local/qa.env","-p",os.environ.get("DDNSUPD_QA_PROJECT","ddnsupd-qa"),"-f","docker-compose.yml","-f","docker-compose.dev.yml","-f","docker-compose.test.yml"]
checks=0
def expect(condition,message):
    global checks
    assert condition,message
    checks+=1;print("PASS",message,flush=True)
def sql(statement):
    return subprocess.check_output(compose+["exec","-T","postgres","psql","-U","ddnsupd","-d","ddnsupd","-tAc",statement],text=True).strip()
class Client:
    def __init__(self):
        self.opener=urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()));self.csrf=""
    def request(self,path,method="GET",data=None,headers=None):
        hdr={"Origin":base,"X-CSRF-Token":self.csrf,"Content-Type":"application/json"};hdr.update(headers or {})
        request=urllib.request.Request(base.replace("localhost","127.0.0.1")+"/api/v1"+path,method=method,headers=hdr,data=json.dumps(data).encode() if data is not None else None)
        try:response=self.opener.open(request,timeout=40)
        except urllib.error.HTTPError as error:response=error
        raw=response.read()
        try:value=json.loads(raw) if raw else None
        except ValueError:value=raw.decode()
        return response.status,value
    def login(self,username,password):
        status,data=self.request("/auth/login","POST",{"username":username,"password":password});assert status==200,(status,data);self.csrf=data["csrf_token"]
admin=Client();admin.login(fixture["username"],fixture["password"])
suffix=secrets.token_hex(4)
status,data=admin.request("/providers","POST",{"kind":"cloudflare","name":"Management QA "+suffix,"token":"fixture-management-token"});assert status==201
provider=data["id"]
def preview(names=("imported",),**overrides):
    config={"settings":[{"provider":"cloudflare","domain":name+".example.org","ip_version":"ipv4"} for name in names]}
    payload={"provider_id":provider,"zone_id":"testzone","zone_name":"example.org","config":config}
    payload.update(overrides)
    return admin.request("/imports/preview","POST",payload)
def confirm(draft,indices=(0,),**overrides):
    data={"digest":draft["digest"],"selected_indices":list(indices),"accept_warnings":True};data.update(overrides)
    return admin.request("/imports/"+draft["id"]+"/confirm","POST",data)

expect(admin.request(f"/providers/{provider}/discover","POST",{"operation":"zones"},headers={"X-CSRF-Token":""})[0]==403,"discovery requires CSRF")
status,data=admin.request(f"/providers/{provider}/discover","POST",{"operation":"zones"})
expect(status==200 and data["permission_verification"]=="read_only" and data["next_page"]==2,"read-only access test reports pagination without claiming write permission")
status,page2=admin.request(f"/providers/{provider}/discover","POST",{"operation":"zones","page":2})
expect(status==200 and page2["zones"][0]["name"]=="other.org","zone pagination is explicit")
expect(admin.request(f"/providers/{provider}/discover","POST",{"operation":"records","zone_id":"../escape","zone_name":"example.org"})[0]==400,"discovery rejects path injection")
status,data=admin.request(f"/providers/{provider}/discover","POST",{"operation":"records","zone_id":"testzone","zone_name":"example.org"})
expect(status==200 and data["records"][0]["record_id"]=="mock-home","discovery returns origin records and stable remote IDs")

source_secret="source-secret-"+secrets.token_hex(12)
config={"settings":[{"provider":"cloudflare","domain":"example.org","host":"imported","ip_version":"ipv4","token":source_secret}]}
status,draft=preview(config=config)
expect(status==201 and not draft["plan"]["items"][0]["errors"],"preview resolves legacy host against actual paginated provider records")
expect(source_secret not in json.dumps(draft),"preview never echoes source credentials")
expect(sql("SELECT count(*) FROM import_previews WHERE plan::text LIKE '%"+source_secret+"%'")=="0","source credentials are not persisted in drafts")
expect(sql("SELECT count(*) FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id WHERE z.provider_id='"+provider+"'")=="0","preview does not create bindings or DNS jobs")
expect(confirm(draft,digest="forged")[0]==409,"confirmation rejects a modified digest")
expect(confirm(draft,accept_warnings=False)[0]==400,"confirmation requires explicit review acknowledgement")
status,result=confirm(draft)
expect(status==201 and result["active"] is False,"confirmation creates only paused bindings")
record=result["created"][0]["id"]
time.sleep(2)
expect(sql("SELECT count(*) FROM update_jobs WHERE record_id='"+record+"'")=="0","imported bindings never schedule writes before activation")
expect(confirm(draft)[0]==409,"preview can be consumed only once")
expect(preview()[1]["plan"]["items"][0]["errors"]==["already_managed"],"preview detects an existing local binding")

ambiguous={"settings":[{"provider":"cloudflare","domain":"no-family.example.org"}]}
status,result=preview(config=ambiguous)
expect(status==201 and "family_choice_required" in result["plan"]["items"][0]["errors"],"ambiguous upstream family never silently becomes dual stack")
expect(confirm(result)[0]==400,"conflicted entries cannot be confirmed")
status,result=preview(config=ambiguous,ambiguous_family="A")
expect(status==201 and not result["plan"]["items"][0]["errors"],"explicit IP-family choice resolves ambiguity")
unsafe={"settings":[{"provider":"custom","domain":"home.example.org","url":"http://127.0.0.1/secret"}]}
status,result=preview(config=unsafe)
expect(status==201 and "unsupported_option" in result["plan"]["items"][0]["errors"] and "127.0.0.1" not in json.dumps(result),"unsupported URL options are rejected without fetching or echoing them")

status,expired=preview(("expired",));assert status==201
sql("UPDATE import_previews SET expires_at=now()-interval '1 second' WHERE id='"+expired["id"]+"'")
expect(confirm(expired)[0]==409,"expired preview is rejected")
status,changed=preview(("credential-change",));assert status==201
admin.request(f"/providers/{provider}","PATCH",{"name":"Management QA rotated "+suffix,"enabled":True,"token":"fixture-management-new-token"})
expect(confirm(changed)[0]==409,"credential rotation invalidates staged preview")

status,race=preview(("race-a","race-b"));assert status==201
admin.request("/records","POST",{"provider_id":provider,"zone_remote_id":"testzone","zone_name":"example.org","name":"race-b","record_type":"A","remote_record_id":"mock-race-b","managed_ip":"8.8.8.8","active":False})
expect(confirm(race,(0,1))[0]==409,"new database conflict rejects entire confirmation")
expect(sql("SELECT count(*) FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id WHERE z.provider_id='"+provider+"' AND r.name='race-a'")=="0","conflicting bulk import rolls back prior inserted rows")

status,single=preview(("same-preview",));assert status==201
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    results=list(pool.map(lambda _:confirm(single)[0],range(2)))
expect(sorted(results)==[201,409],"concurrent confirmation creates exactly one set of bindings")

viewer_pass=secrets.token_urlsafe(20)
sql("UPDATE provider_discovery_limits SET requests=0,window_start=now() WHERE provider_id='"+provider+"'")
viewer_id=admin.request("/users","POST",{"username":"management-viewer-"+suffix,"password":viewer_pass,"role_id":"00000000-0000-0000-0000-000000000004"})[1]["id"]
viewer=Client();viewer.login("management-viewer-"+suffix,viewer_pass)
expect(viewer.request(f"/providers/{provider}/discover","POST",{"operation":"zones"})[0]==404,"provider discovery does not leak ungranted accounts")
admin.request("/grants","POST",{"user_id":viewer_id,"provider_id":provider,"enabled":True})
expect(viewer.request(f"/providers/{provider}/discover","POST",{"operation":"zones"})[0]==200,"provider read grant permits non-mutating discovery")
expect(viewer.request("/imports/preview","POST",{"provider_id":provider,"zone_id":"testzone","zone_name":"example.org","config":config})[0]==403,"viewer cannot preview an import mutation")

other_pass=secrets.token_urlsafe(20)
admin.request("/users","POST",{"username":"management-admin-"+suffix,"password":other_pass,"role_id":"00000000-0000-0000-0000-000000000001"})
other=Client();other.login("management-admin-"+suffix,other_pass)
status,private=preview(("protected",));assert status==201
expect(other.request(f"/imports/{private['id']}/confirm","POST",{"digest":private["digest"],"selected_indices":[0],"accept_warnings":True})[0]==404,"even another administrator cannot consume a user's draft")

sql("UPDATE provider_discovery_limits SET requests=30,window_start=now() WHERE provider_id='"+provider+"'")
expect(admin.request(f"/providers/{provider}/discover","POST",{"operation":"zones"})[0]==429,"repeated discovery calls have a persistent provider budget")
expect(source_secret not in json.dumps(admin.request("/audit")[1]),"import audit never contains source credentials")
print(f"{checks} management checks passed",flush=True)
