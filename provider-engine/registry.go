package main

import (
	"context"
	_ "embed"
	"net/http"
)

//go:embed registry.json
var registryJSON []byte

type Adapter interface {
	Reconcile(context.Context, *http.Client, Request) (Result, error)
}
type adapterFunc func(context.Context, *http.Client, Request) (Result, error)

func (f adapterFunc) Reconcile(ctx context.Context, client *http.Client, in Request) (Result, error) {
	return f(ctx, client, in)
}

var adapters = map[string]Adapter{
	"hetznercloud": adapterFunc(reconcileHetzner),
	"cloudflare":   adapterFunc(reconcileCloudflare),
}
