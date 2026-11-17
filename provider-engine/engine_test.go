package main

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"
)

type transportFunc func(*http.Request) (*http.Response, error)

func TestHetznerCreateMissing(t *testing.T) {
	for _, scenario := range []struct {
		name, mode, code                   string
		missing, create, dryRun, wantWrite bool
	}{
		{name: "confirmed", mode: "primary", missing: true, create: true, wantWrite: true},
		{name: "ordinary check", mode: "primary", missing: true, code: "not_found"},
		{name: "already exists", mode: "primary", create: true, code: "record_exists"},
		{name: "dry run", mode: "primary", missing: true, create: true, dryRun: true},
		{name: "secondary", mode: "secondary", missing: true, create: true, code: "unsupported"},
	} {
		t.Run(scenario.name, func(t *testing.T) {
			in := baseRequest()
			in.Type, in.Desired, in.Managed = "AAAA", "2606:4700:4700:0:0:0:0:1111", ""
			in.CreateMissing, in.DryRun = scenario.create, scenario.dryRun
			zonePath := "/v1/zones/" + in.ZoneID
			setPath := zonePath + "/rrsets/" + in.Name + "/AAAA"
			writes := 0
			client := &http.Client{Transport: transportFunc(func(req *http.Request) (*http.Response, error) {
				switch {
				case req.Method == "GET" && req.URL.Path == zonePath:
					return response(`{"zone":{"name":"` + in.ZoneName + `","mode":"` + scenario.mode + `"}}`), nil
				case req.Method == "GET" && req.URL.Path == setPath:
					if scenario.missing && writes == 0 {
						out := response(`{}`)
						out.StatusCode = 404
						return out, nil
					}
					return response(`{"rrset":{"records":[{"value":"2606:4700:4700::1111"}]}}`), nil
				case req.Method == "POST" && req.URL.Path == zonePath+"/rrsets":
					writes++
					var body struct {
						Name    string
						Type    string
						Records []struct{ Value string }
					}
					if err := json.NewDecoder(req.Body).Decode(&body); err != nil || body.Name != in.Name ||
						body.Type != "AAAA" || len(body.Records) != 1 || body.Records[0].Value != in.Desired {
						t.Fatalf("invalid create payload: %+v (%v)", body, err)
					}
					out := response(`{"action":{"id":42,"status":"success"}}`)
					out.StatusCode = 201
					return out, nil
				default:
					t.Fatalf("unexpected request: %s %s", req.Method, req.URL.Path)
					return nil, nil
				}
			})}
			out, err := reconcile(context.Background(), client, in)
			if scenario.code != "" {
				if err == nil || errorCode(err).Code != scenario.code {
					t.Fatalf("expected %s, got %v", scenario.code, err)
				}
			} else if err != nil {
				t.Fatal(err)
			}
			if writes > 1 || (writes == 1) != scenario.wantWrite ||
				out.Changed != scenario.wantWrite || out.Verified != scenario.wantWrite {
				t.Fatalf("unexpected write count/result: %d %+v", writes, out)
			}
		})
	}
}

func (f transportFunc) RoundTrip(r *http.Request) (*http.Response, error) { return f(r) }
func response(body string) *http.Response {
	return &http.Response{StatusCode: 200, Body: io.NopCloser(strings.NewReader(body)), Header: make(http.Header)}
}

func TestClassificationAndSecretRedaction(t *testing.T) {
	for _, test := range []struct {
		status int
		code   string
	}{{401, "auth"}, {403, "permission"}, {404, "not_found"}, {429, "rate_limit"}, {500, "unavailable"}} {
		client := &http.Client{Transport: transportFunc(func(*http.Request) (*http.Response, error) {
			r := response(`secret-provider-response`)
			r.StatusCode = test.status
			r.Header.Set("Retry-After", "600")
			return r, nil
		})}
		_, err := reconcile(context.Background(), client, baseRequest())
		if err == nil || errorCode(err).Code != test.code || strings.Contains(err.Error(), "secret") {
			t.Fatalf("status=%d err=%v", test.status, err)
		}
		if test.status == 429 && errorCode(err).RetryAfter != 600 {
			t.Fatal("Retry-After lost")
		}
	}
}

func TestHandlerAuthenticationAndStrictBody(t *testing.T) {
	client := &http.Client{Transport: transportFunc(func(*http.Request) (*http.Response, error) { t.Fatal("unexpected network"); return nil, nil })}
	h := handler("service-test-token", client)
	for _, test := range []struct {
		body, auth string
		status     int
	}{{`{}`, "", 401}, {`{"unknown":true}`, "Bearer service-test-token", 400}, {`{} {}`, "Bearer service-test-token", 400}} {
		r := httptest.NewRequest("POST", "/v1/reconcile", strings.NewReader(test.body))
		r.Header.Set("Authorization", test.auth)
		w := httptest.NewRecorder()
		h.ServeHTTP(w, r)
		if w.Code != test.status {
			t.Fatalf("status=%d", w.Code)
		}
	}
}

func TestDesiredSiblingDoesNotStealBinding(t *testing.T) {
	client := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) {
		if r.Method != "GET" {
			t.Fatal("unexpected write")
		}
		if r.URL.Path == "/v1/zones/123" {
			return response(`{"zone":{"name":"example.org"}}`), nil
		}
		return response(`{"rrset":{"records":[{"value":"1.1.1.1"},{"value":"8.8.8.8"}]}}`), nil
	})}
	_, err := reconcile(context.Background(), client, baseRequest())
	if err == nil || errorCode(err).Code != "conflict" {
		t.Fatal("management binding moved to sibling")
	}
}

func TestRegistryHasImplementedAdapters(t *testing.T) {
	var registry struct {
		Items []struct {
			ID string `json:"id"`
		} `json:"items"`
	}
	if err := json.Unmarshal(registryJSON, &registry); err != nil {
		t.Fatal(err)
	}
	if len(registry.Items) != len(adapters) {
		t.Fatal("registry mismatch")
	}
	for _, item := range registry.Items {
		if adapters[item.ID] == nil {
			t.Fatal("unimplemented adapter")
		}
	}
}

func TestBoundedResponseAndDeadline(t *testing.T) {
	large := &http.Client{Transport: transportFunc(func(*http.Request) (*http.Response, error) { return response(strings.Repeat("x", 1024*1024+1)), nil })}
	if _, err := reconcile(context.Background(), large, baseRequest()); err == nil || errorCode(err).Code != "unavailable" {
		t.Fatal("oversized provider response accepted")
	}
	stalled := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) { <-r.Context().Done(); return nil, r.Context().Err() })}
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Millisecond)
	defer cancel()
	if _, err := reconcile(ctx, stalled, baseRequest()); err == nil {
		t.Fatal("deadline ignored")
	}
}

func TestCloudflareIPv6(t *testing.T) {
	changed := false
	client := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) {
		if r.URL.Path == "/client/v4/zones/123" {
			return response(`{"success":true,"result":{"name":"example.org"}}`), nil
		}
		if r.Method == "PATCH" {
			changed = true
			return response(`{"success":true}`), nil
		}
		value := "2606:4700:4700::1001"
		if changed {
			value = "2606:4700:4700::1111"
		}
		return response(`{"success":true,"result":{"name":"home.example.org","type":"AAAA","content":"` + value + `"}}`), nil
	})}
	in := baseRequest()
	in.Provider = "cloudflare"
	in.RecordID = "abc"
	in.Type = "AAAA"
	in.Managed = "2606:4700:4700::1001"
	in.Desired = "2606:4700:4700::1111"
	out, err := reconcile(context.Background(), client, in)
	if err != nil || !out.Verified || !out.Changed {
		t.Fatalf("IPv6 reconcile failed: %+v %v", out, err)
	}
}
func baseRequest() Request {
	return Request{Provider: "hetznercloud", Token: "test-secret", ZoneID: "123", ZoneName: "example.org", Name: "home", Type: "A", Desired: "8.8.8.8", Managed: "1.1.1.1"}
}

func TestHetznerPreservesSiblingsAndComments(t *testing.T) {
	updated := false
	client := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) {
		if r.Method == "POST" {
			var payload struct {
				Records []map[string]any `json:"records"`
			}
			if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
				t.Fatal(err)
			}
			if len(payload.Records) != 2 || payload.Records[0]["value"] != "8.8.8.8" || payload.Records[0]["comment"] != "managed" || payload.Records[1]["value"] != "9.9.9.9" {
				t.Fatalf("destructive payload: %#v", payload)
			}
			updated = true
			return response(`{"action":{"id":1,"status":"success"}}`), nil
		}
		if r.URL.Path == "/v1/zones/123" {
			return response(`{"zone":{"id":123,"name":"example.org"}}`), nil
		}
		if updated {
			return response(`{"rrset":{"records":[{"value":"8.8.8.8","comment":"managed"},{"value":"9.9.9.9"}]}}`), nil
		}
		return response(`{"rrset":{"records":[{"value":"1.1.1.1","comment":"managed"},{"value":"9.9.9.9"}]}}`), nil
	})}
	result, err := reconcile(context.Background(), client, baseRequest())
	if err != nil || !result.Changed || !result.Verified || !updated {
		t.Fatalf("result=%+v err=%v", result, err)
	}
}

func TestNoChangeAndDryRunNeverWrite(t *testing.T) {
	for _, dry := range []bool{false, true} {
		in := baseRequest()
		in.DryRun = dry
		client := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) {
			if r.Method != "GET" {
				t.Fatal("unexpected write")
			}
			if r.URL.Path == "/v1/zones/123" {
				return response(`{"zone":{"name":"example.org"}}`), nil
			}
			if !dry {
				return response(`{"rrset":{"records":[{"value":"8.8.8.8"}]}}`), nil
			}
			return response(`{"rrset":{"records":[{"value":"1.1.1.1"}]}}`), nil
		})}
		out, err := reconcile(context.Background(), client, in)
		if err != nil || out.Changed {
			t.Fatalf("%+v %v", out, err)
		}
	}
}

func TestRejectsUnsafeIPAndScopeBeforeNetwork(t *testing.T) {
	client := &http.Client{Transport: transportFunc(func(*http.Request) (*http.Response, error) { t.Fatal("network called"); return nil, nil })}
	for _, ip := range []string{"127.0.0.1", "100.64.0.1", "192.0.2.1", "10.0.0.1", "::1", "2001:db8::1", "garbage", "::ffff:10.0.0.1"} {
		in := baseRequest()
		in.Desired = ip
		if _, err := reconcile(context.Background(), client, in); err == nil {
			t.Fatalf("accepted %s", ip)
		}
	}
	in := baseRequest()
	in.Name = "../other"
	if _, err := reconcile(context.Background(), client, in); err == nil {
		t.Fatal("accepted path")
	}
}

func TestCloudflareOriginAndPatch(t *testing.T) {
	written := false
	client := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) {
		if r.Method == "PATCH" {
			var fields map[string]any
			if err := json.NewDecoder(r.Body).Decode(&fields); err != nil {
				t.Fatal(err)
			}
			if len(fields) != 1 || fields["content"] != "8.8.8.8" {
				t.Fatal(fields)
			}
			written = true
			return response(`{"success":true}`), nil
		}
		if r.URL.Path == "/client/v4/zones/123" {
			return response(`{"success":true,"result":{"name":"example.org"}}`), nil
		}
		content := "1.1.1.1"
		if written {
			content = "8.8.8.8"
		}
		return response(`{"success":true,"result":{"name":"home.example.org","type":"A","content":"` + content + `","proxied":true}}`), nil
	})}
	in := baseRequest()
	in.Provider = "cloudflare"
	in.RecordID = "abc"
	out, err := reconcile(context.Background(), client, in)
	if err != nil || !out.Verified || !written {
		t.Fatalf("%+v %v", out, err)
	}
}

func TestConflictingManagedValueNeverWrites(t *testing.T) {
	client := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) {
		if r.Method != "GET" {
			t.Fatal("unexpected write")
		}
		if r.URL.Path == "/v1/zones/123" {
			return response(`{"zone":{"name":"example.org"}}`), nil
		}
		return response(`{"rrset":{"records":[{"value":"4.4.4.4"}]}}`), nil
	})}
	if _, err := reconcile(context.Background(), client, baseRequest()); err == nil {
		t.Fatal("accepted external drift without expected binding")
	}
}
