// Provider HTTP workflows adapted from qdm12/ddns-updater, MIT, Copyright
// (c) 2019 Quentin McGaw, commit 7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2.
// Changes: private typed contract, origin verification, explicit record binding,
// sibling/comment preservation, safe errors and no implicit record creation.
// Full original notice: LICENSE.upstream.
package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/netip"
	"regexp"
	"strings"
	"time"
)

type Request struct {
	Provider      string `json:"provider"`
	Token         string `json:"token"`
	ZoneID        string `json:"zone_id"`
	ZoneName      string `json:"zone_name"`
	Name          string `json:"name"`
	RecordID      string `json:"record_id"`
	Type          string `json:"record_type"`
	Desired       string `json:"desired"`
	Managed       string `json:"managed"`
	DryRun        bool   `json:"dry_run"`
	CreateMissing bool   `json:"create_missing"`
}
type Result struct {
	Before   string `json:"before"`
	After    string `json:"after"`
	Changed  bool   `json:"changed"`
	Verified bool   `json:"verified"`
}
type EngineError struct {
	Code       string `json:"code"`
	RetryAfter uint64 `json:"retry_after,omitempty"`
}

func (e *EngineError) Error() string { return e.Code }
func fail(code string) error         { return &EngineError{Code: code} }

var identifier = regexp.MustCompile(`^[A-Za-z0-9_-]{1,128}$`)
var hostname = regexp.MustCompile(`^(\*\.)?[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*$`)
var blockedPrefixes = func() []netip.Prefix {
	values := []string{"0.0.0.0/8", "10.0.0.0/8", "100.64.0.0/10", "127.0.0.0/8", "169.254.0.0/16", "172.16.0.0/12", "192.0.0.0/24", "192.0.2.0/24", "192.168.0.0/16", "198.18.0.0/15", "198.51.100.0/24", "203.0.113.0/24", "224.0.0.0/3", "2001:db8::/32", "2001::/23", "2002::/16", "3fff::/20"}
	out := make([]netip.Prefix, 0, len(values))
	for _, v := range values {
		out = append(out, netip.MustParsePrefix(v))
	}
	return out
}()

func publicIP(raw string, typ string) bool {
	ip, err := netip.ParseAddr(raw)
	if err != nil || ip.Is4In6() || !ip.IsGlobalUnicast() || ip.IsPrivate() || (typ == "A") != ip.Is4() {
		return false
	}
	if ip.Is6() && !netip.MustParsePrefix("2000::/3").Contains(ip) {
		return false
	}
	for _, prefix := range blockedPrefixes {
		if prefix.Contains(ip) {
			return false
		}
	}
	return true
}
func validate(in Request) error {
	if in.Token == "" || len(in.Token) > 8192 || !identifier.MatchString(in.ZoneID) || len(in.ZoneName) > 253 || !hostname.MatchString(in.ZoneName) || strings.Contains(in.ZoneName, "*") {
		return fail("invalid_input")
	}
	if in.Type != "A" && in.Type != "AAAA" {
		return fail("invalid_input")
	}
	if in.Name != "@" && in.Name != "*" && (!hostname.MatchString(in.Name) || len(in.Name) > 253) {
		return fail("invalid_input")
	}
	if !publicIP(in.Desired, in.Type) {
		return fail("invalid_input")
	}
	if in.Managed != "" {
		if _, err := netip.ParseAddr(in.Managed); err != nil {
			return fail("invalid_input")
		}
	}
	if in.Provider == "cloudflare" && !identifier.MatchString(in.RecordID) {
		return fail("invalid_input")
	}
	if _, ok := adapters[in.Provider]; !ok {
		return fail("unsupported")
	}
	if in.CreateMissing && in.Provider != "hetznercloud" {
		return fail("unsupported")
	}
	return nil
}

func call(ctx context.Context, client *http.Client, in Request, method, path string, payload any, out any) error {
	base := "https://api.hetzner.cloud/v1"
	if in.Provider == "cloudflare" {
		base = "https://api.cloudflare.com/client/v4"
	}
	var body io.Reader
	if payload != nil {
		b, err := json.Marshal(payload)
		if err != nil {
			return fail("internal")
		}
		body = bytes.NewReader(b)
	}
	req, err := http.NewRequestWithContext(ctx, method, base+path, body)
	if err != nil {
		return fail("invalid_input")
	}
	req.Header.Set("Authorization", "Bearer "+in.Token)
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("User-Agent", "DDNSupd/0.1.0")
	resp, err := client.Do(req)
	if err != nil {
		if method != "GET" {
			return fail("uncertain")
		}
		return fail("unavailable")
	}
	defer resp.Body.Close()
	if resp.StatusCode == 401 {
		return fail("auth")
	}
	if resp.StatusCode == 403 {
		return fail("permission")
	}
	if resp.StatusCode == 404 {
		return fail("not_found")
	}
	if resp.StatusCode == 429 {
		return &EngineError{Code: "rate_limit", RetryAfter: retryAfter(resp.Header.Get("Retry-After"))}
	}
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		if method != "GET" {
			return fail("uncertain")
		}
		return fail("unavailable")
	}
	if out == nil {
		return nil
	}
	b, err := io.ReadAll(io.LimitReader(resp.Body, 1024*1024+1))
	if err != nil || len(b) > 1024*1024 || json.Unmarshal(b, out) != nil {
		if method != "GET" {
			return fail("uncertain")
		}
		return fail("unavailable")
	}
	return nil
}
func retryAfter(raw string) uint64 {
	var seconds uint64
	if _, err := fmt.Sscanf(raw, "%d", &seconds); err == nil {
		return min(max(seconds, 1), 86400)
	}
	if date, err := http.ParseTime(raw); err == nil {
		return uint64(min(max(time.Until(date).Seconds(), 1), 86400))
	}
	return 300
}

func reconcile(ctx context.Context, client *http.Client, in Request) (Result, error) {
	if err := validate(in); err != nil {
		return Result{}, err
	}
	return adapters[in.Provider].Reconcile(ctx, client, in)
}
func fqdn(in Request) string {
	if in.Name == "@" {
		return in.ZoneName
	}
	return in.Name + "." + in.ZoneName
}
func checkZone(ctx context.Context, client *http.Client, in Request) error {
	var z struct {
		Zone struct {
			Name string `json:"name"`
			Mode string `json:"mode"`
		} `json:"zone"`
		Success bool `json:"success"`
		Result  struct {
			Name string `json:"name"`
		} `json:"result"`
	}
	if err := call(ctx, client, in, "GET", "/zones/"+in.ZoneID, nil, &z); err != nil {
		return err
	}
	if in.CreateMissing && z.Zone.Mode != "primary" {
		return fail("unsupported")
	}
	name := z.Zone.Name
	if in.Provider == "cloudflare" {
		if !z.Success {
			return fail("unavailable")
		}
		name = z.Result.Name
	}
	if name != in.ZoneName {
		return fail("conflict")
	}
	return nil
}
func reconcileCloudflare(ctx context.Context, client *http.Client, in Request) (Result, error) {
	if err := checkZone(ctx, client, in); err != nil {
		return Result{}, err
	}
	path := "/zones/" + in.ZoneID + "/dns_records/" + in.RecordID
	read := func() (string, error) {
		var r struct {
			Success bool `json:"success"`
			Result  struct {
				Name    string `json:"name"`
				Type    string `json:"type"`
				Content string `json:"content"`
			} `json:"result"`
		}
		if err := call(ctx, client, in, "GET", path, nil, &r); err != nil {
			return "", err
		}
		if !r.Success {
			return "", fail("unavailable")
		}
		if r.Result.Name != fqdn(in) || r.Result.Type != in.Type {
			return "", fail("conflict")
		}
		return r.Result.Content, nil
	}
	before, err := read()
	if err != nil {
		return Result{}, err
	}
	out := Result{Before: before, After: before, Verified: before == in.Desired}
	if before == in.Desired || in.DryRun {
		return out, nil
	}
	if in.Managed != "" && before != in.Managed {
		return out, fail("conflict")
	}
	var result struct {
		Success bool `json:"success"`
	}
	if err := call(ctx, client, in, "PATCH", path, map[string]string{"content": in.Desired}, &result); err != nil {
		return out, err
	}
	if !result.Success {
		return out, fail("uncertain")
	}
	after, err := read()
	if err != nil {
		return out, fail("uncertain")
	}
	out.After = after
	out.Changed = true
	out.Verified = after == in.Desired
	if !out.Verified {
		return out, fail("uncertain")
	}
	return out, nil
}
func reconcileHetzner(ctx context.Context, client *http.Client, in Request) (Result, error) {
	if err := checkZone(ctx, client, in); err != nil {
		return Result{}, err
	}
	path := "/zones/" + in.ZoneID + "/rrsets/" + in.Name + "/" + in.Type
	read := func() ([]map[string]any, error) {
		var r struct {
			RRSet struct {
				Records []map[string]any `json:"records"`
			} `json:"rrset"`
		}
		if err := call(ctx, client, in, "GET", path, nil, &r); err != nil {
			return nil, err
		}
		return r.RRSet.Records, nil
	}
	records, err := read()
	if in.CreateMissing {
		if err == nil {
			return Result{}, fail("record_exists")
		}
		if errorCode(err).Code != "not_found" {
			return Result{}, err
		}
		if in.DryRun {
			return Result{}, nil
		}
		return writeHetzner(ctx, client, in, "/zones/"+in.ZoneID+"/rrsets", map[string]any{
			"name": in.Name, "type": in.Type, "records": []map[string]string{{"value": in.Desired}},
		}, Result{}, read)
	}
	if err != nil {
		return Result{}, err
	}
	if len(records) == 0 {
		return Result{}, fail("not_found")
	}
	index := -1
	before := ""
	desiredPresent := false
	for i, r := range records {
		value, ok := r["value"].(string)
		if !ok {
			return Result{}, fail("unavailable")
		}
		if sameAddress(value, in.Desired) {
			desiredPresent = true
		}
		if sameAddress(value, in.Managed) {
			if index >= 0 {
				return Result{}, fail("conflict")
			}
			index = i
			before = value
		}
	}
	if desiredPresent {
		// Do not move the management binding to an unrelated sibling value.
		if (index >= 0 && !sameAddress(in.Managed, in.Desired)) || (in.Managed == "" && len(records) > 1) {
			return Result{}, fail("conflict")
		}
		return Result{Before: in.Desired, After: in.Desired, Verified: true}, nil
	}
	if index < 0 && in.Managed == "" && len(records) == 1 {
		index = 0
		before, _ = records[0]["value"].(string)
	}
	if index < 0 {
		return Result{}, fail("conflict")
	}
	out := Result{Before: before, After: before}
	if in.DryRun {
		return out, nil
	}
	records[index]["value"] = in.Desired
	return writeHetzner(ctx, client, in, path+"/actions/set_records", map[string]any{"records": records}, out, read)
}

func writeHetzner(ctx context.Context, client *http.Client, in Request, path string, payload map[string]any, out Result, read func() ([]map[string]any, error)) (Result, error) {
	var action struct {
		Action struct {
			ID     uint64 `json:"id"`
			Status string `json:"status"`
		} `json:"action"`
	}
	if err := call(ctx, client, in, "POST", path, payload, &action); err != nil {
		return out, err
	}
	for attempts := 0; action.Action.Status == "running" && attempts < 15; attempts++ {
		if action.Action.ID == 0 {
			return out, fail("uncertain")
		}
		select {
		case <-ctx.Done():
			return out, fail("uncertain")
		case <-time.After(time.Second):
		}
		if err := call(ctx, client, in, "GET", fmt.Sprintf("/zones/actions/%d", action.Action.ID), nil, &action); err != nil {
			return out, fail("uncertain")
		}
	}
	if action.Action.Status != "success" {
		return out, fail("uncertain")
	}
	after, err := read()
	if err != nil {
		return out, fail("uncertain")
	}
	found := false
	for _, r := range after {
		if value, ok := r["value"].(string); ok && sameAddress(value, in.Desired) {
			found = true
		}
	}
	if !found {
		return out, fail("uncertain")
	}
	out.After = in.Desired
	out.Changed = true
	out.Verified = true
	return out, nil
}

func sameAddress(a, b string) bool {
	left, leftErr := netip.ParseAddr(a)
	right, rightErr := netip.ParseAddr(b)
	return leftErr == nil && rightErr == nil && left == right
}

func errorCode(err error) EngineError {
	var e *EngineError
	if errors.As(err, &e) {
		return *e
	}
	return EngineError{Code: "internal"}
}
