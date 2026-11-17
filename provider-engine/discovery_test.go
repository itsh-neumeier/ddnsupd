package main

import (
	"context"
	"net/http"
	"testing"
)

func TestDiscoveryCloudflarePaginationAndScope(t *testing.T) {
	client := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) {
		if r.Method != "GET" {
			t.Fatal("discovery wrote provider data")
		}
		switch r.URL.Path {
		case "/client/v4/zones":
			if r.URL.Query().Get("page") != "2" || r.URL.Query().Get("per_page") != "50" {
				t.Fatal(r.URL)
			}
			return response(`{"success":true,"result":[{"id":"zone1","name":"example.org"}],"result_info":{"page":2,"total_pages":3}}`), nil
		case "/client/v4/zones/zone1":
			return response(`{"success":true,"result":{"name":"example.org"}}`), nil
		default:
			return response(`{"success":true,"result":[{"id":"record1","name":"vpn.example.org","type":"A","content":"8.8.8.8","ttl":300,"proxied":true},{"id":"mx","name":"example.org","type":"MX","content":"mail.example.org"}],"result_info":{"page":1,"total_pages":1}}`), nil
		}
	})}
	result, err := discover(context.Background(), client, DiscoveryRequest{Provider: "cloudflare", Token: "test", Operation: "zones", Page: 2})
	if err != nil || len(result.Zones) != 1 || result.NextPage == nil || *result.NextPage != 3 {
		t.Fatalf("%+v %v", result, err)
	}
	records, err := discover(context.Background(), client, DiscoveryRequest{Provider: "cloudflare", Token: "test", Operation: "records", ZoneID: "zone1", ZoneName: "example.org", Page: 1})
	if err != nil || len(records.Records) != 1 || records.Records[0].Name != "vpn" || !records.Records[0].Proxied || !records.Records[0].Bindable {
		t.Fatalf("%+v %v", records, err)
	}
}

func TestDiscoveryHetznerRRSetsAndUnusableValues(t *testing.T) {
	client := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) {
		if r.Method != "GET" {
			t.Fatal("unexpected write")
		}
		if r.URL.Path == "/v1/zones/123" {
			return response(`{"zone":{"name":"example.org"}}`), nil
		}
		return response(`{"rrsets":[{"name":"@","type":"A","ttl":60,"records":[{"value":"8.8.8.8"},{"value":"10.0.0.1"}]},{"name":"vpn","type":"AAAA","ttl":300,"records":[{"value":"2606:4700:4700::1111"}]}],"meta":{"pagination":{"page":1,"next_page":2}}}`), nil
	})}
	result, err := discover(context.Background(), client, DiscoveryRequest{Provider: "hetznercloud", Token: "test", Operation: "records", ZoneID: "123", ZoneName: "example.org", Page: 1})
	if err != nil || len(result.Records) != 3 || !result.Records[0].Bindable || result.Records[1].Bindable || result.NextPage == nil {
		t.Fatalf("%+v %v", result, err)
	}
}

func TestDiscoveryRejectsScopeAndUnboundedPagination(t *testing.T) {
	client := &http.Client{Transport: transportFunc(func(*http.Request) (*http.Response, error) { t.Fatal("unexpected network"); return nil, nil })}
	for _, in := range []DiscoveryRequest{
		{Provider: "custom", Token: "test", Operation: "zones", Page: 1},
		{Provider: "cloudflare", Token: "test", Operation: "zones", Page: 10001},
		{Provider: "cloudflare", Token: "test", Operation: "records", Page: 1, ZoneID: "../escape", ZoneName: "example.org"},
	} {
		if _, err := discover(context.Background(), client, in); err == nil {
			t.Fatal("invalid discovery accepted")
		}
	}
	scoped := &http.Client{Transport: transportFunc(func(r *http.Request) (*http.Response, error) {
		if r.URL.Path != "/client/v4/zones/123" {
			t.Fatal("cross-zone record query")
		}
		return response(`{"success":true,"result":{"name":"other.org"}}`), nil
	})}
	if _, err := discover(context.Background(), scoped, DiscoveryRequest{Provider: "cloudflare", Token: "test", Operation: "records", Page: 1, ZoneID: "123", ZoneName: "example.org"}); err == nil {
		t.Fatal("wrong zone accepted")
	}
}
