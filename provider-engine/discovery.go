package main

import (
	"context"
	"fmt"
	"net/http"
	"strconv"
	"strings"
)

type DiscoveryRequest struct {
	Provider  string `json:"provider"`
	Token     string `json:"token"`
	Operation string `json:"operation"`
	ZoneID    string `json:"zone_id"`
	ZoneName  string `json:"zone_name"`
	Page      int    `json:"page"`
}
type Zone struct {
	ID   string `json:"id"`
	Name string `json:"name"`
	Mode string `json:"mode"`
}
type DiscoveredRecord struct {
	Name     string `json:"name"`
	Type     string `json:"record_type"`
	ID       string `json:"record_id"`
	Value    string `json:"value"`
	TTL      uint32 `json:"ttl"`
	Proxied  bool   `json:"proxied"`
	Bindable bool   `json:"bindable"`
}
type DiscoveryResult struct {
	Zones                  []Zone             `json:"zones"`
	Records                []DiscoveredRecord `json:"records"`
	Page                   int                `json:"page"`
	NextPage               *int               `json:"next_page"`
	PermissionVerification string             `json:"permission_verification"`
}

func relativeName(fqdn, zone string) (string, error) {
	name := strings.ToLower(strings.TrimSuffix(fqdn, "."))
	if name == zone {
		return "@", nil
	}
	if !strings.HasSuffix(name, "."+zone) {
		return "", fail("conflict")
	}
	name = strings.TrimSuffix(name, "."+zone)
	if name != "*" && !hostname.MatchString(name) {
		return "", fail("unavailable")
	}
	return name, nil
}
func discover(ctx context.Context, client *http.Client, in DiscoveryRequest) (DiscoveryResult, error) {
	result := DiscoveryResult{Zones: []Zone{}, Records: []DiscoveredRecord{}, Page: in.Page, PermissionVerification: "read_only"}
	if _, ok := adapters[in.Provider]; !ok {
		return result, fail("unsupported")
	}
	if in.Page < 1 || in.Page > 10000 || in.Token == "" || len(in.Token) > 8192 || (in.Operation != "zones" && in.Operation != "records") {
		return result, fail("invalid_input")
	}
	request := Request{Provider: in.Provider, Token: in.Token, ZoneID: in.ZoneID, ZoneName: in.ZoneName}
	path := "/zones"
	if in.Operation == "records" {
		if !identifier.MatchString(in.ZoneID) || len(in.ZoneName) > 253 || !hostname.MatchString(in.ZoneName) || strings.Contains(in.ZoneName, "*") {
			return result, fail("invalid_input")
		}
		if err := checkZone(ctx, client, request); err != nil {
			return result, err
		}
		path += "/" + in.ZoneID
		if in.Provider == "cloudflare" {
			path += "/dns_records"
		} else {
			path += "/rrsets"
		}
	}
	path += fmt.Sprintf("?page=%d&per_page=50", in.Page)
	if in.Provider == "cloudflare" {
		var data struct {
			Success bool `json:"success"`
			Result  []struct {
				ID      string `json:"id"`
				Name    string `json:"name"`
				Type    string `json:"type"`
				Content string `json:"content"`
				TTL     uint32 `json:"ttl"`
				Proxied bool   `json:"proxied"`
			} `json:"result"`
			Info struct {
				Page       int `json:"page"`
				TotalPages int `json:"total_pages"`
			} `json:"result_info"`
		}
		if err := call(ctx, client, request, "GET", path, nil, &data); err != nil {
			return result, err
		}
		if !data.Success || data.Result == nil || len(data.Result) > 50 || data.Info.Page != in.Page || data.Info.TotalPages < 0 {
			return result, fail("unavailable")
		}
		if data.Info.TotalPages > in.Page {
			next := in.Page + 1
			result.NextPage = &next
		}
		for _, item := range data.Result {
			if in.Operation == "zones" {
				if !identifier.MatchString(item.ID) || !hostname.MatchString(item.Name) || strings.Contains(item.Name, "*") {
					return result, fail("unavailable")
				}
				result.Zones = append(result.Zones, Zone{ID: item.ID, Name: item.Name, Mode: "primary"})
			} else if item.Type == "A" || item.Type == "AAAA" {
				name, err := relativeName(item.Name, in.ZoneName)
				if err != nil {
					return result, err
				}
				if !identifier.MatchString(item.ID) {
					return result, fail("unavailable")
				}
				result.Records = append(result.Records, DiscoveredRecord{Name: name, Type: item.Type, ID: item.ID, Value: item.Content, TTL: item.TTL, Proxied: item.Proxied, Bindable: publicIP(item.Content, item.Type)})
			}
		}
	} else {
		var data struct {
			Zones []struct {
				ID   uint64 `json:"id"`
				Name string `json:"name"`
				Mode string `json:"mode"`
			} `json:"zones"`
			RRSets []struct {
				Name    string `json:"name"`
				Type    string `json:"type"`
				TTL     uint32 `json:"ttl"`
				Records []struct {
					Value string `json:"value"`
				} `json:"records"`
			} `json:"rrsets"`
			Meta struct {
				Pagination struct {
					Page     int  `json:"page"`
					NextPage *int `json:"next_page"`
				} `json:"pagination"`
			} `json:"meta"`
		}
		if err := call(ctx, client, request, "GET", path, nil, &data); err != nil {
			return result, err
		}
		if (in.Operation == "zones" && data.Zones == nil) || (in.Operation == "records" && data.RRSets == nil) || len(data.Zones) > 50 || len(data.RRSets) > 50 {
			return result, fail("unavailable")
		}
		if data.Meta.Pagination.NextPage != nil {
			if *data.Meta.Pagination.NextPage != in.Page+1 {
				return result, fail("unavailable")
			}
			result.NextPage = data.Meta.Pagination.NextPage
		}
		for _, zone := range data.Zones {
			if zone.ID == 0 || !hostname.MatchString(zone.Name) || strings.Contains(zone.Name, "*") {
				return result, fail("unavailable")
			}
			result.Zones = append(result.Zones, Zone{ID: strconv.FormatUint(zone.ID, 10), Name: zone.Name, Mode: zone.Mode})
		}
		for _, set := range data.RRSets {
			if set.Type != "A" && set.Type != "AAAA" {
				continue
			}
			if set.Name != "@" && set.Name != "*" && !hostname.MatchString(set.Name) {
				return result, fail("unavailable")
			}
			for _, record := range set.Records {
				if len(result.Records) >= 1000 {
					return result, fail("unavailable")
				}
				result.Records = append(result.Records, DiscoveredRecord{Name: set.Name, Type: set.Type, Value: record.Value, TTL: set.TTL, Bindable: publicIP(record.Value, set.Type)})
			}
		}
	}
	return result, nil
}
