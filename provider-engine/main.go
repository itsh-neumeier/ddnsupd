package main

import (
	"context"
	"crypto/subtle"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"os"
	"time"
)

func handler(serviceToken string, client *http.Client) http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /v1/providers", func(w http.ResponseWriter, r *http.Request) {
		if subtle.ConstantTimeCompare([]byte(r.Header.Get("Authorization")), []byte("Bearer "+serviceToken)) != 1 {
			w.WriteHeader(401)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(registryJSON)
	})
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(200) })
	mux.HandleFunc("POST /v1/discover", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		if subtle.ConstantTimeCompare([]byte(r.Header.Get("Authorization")), []byte("Bearer "+serviceToken)) != 1 {
			w.WriteHeader(401)
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, 32768)
		decoder := json.NewDecoder(r.Body)
		decoder.DisallowUnknownFields()
		var in DiscoveryRequest
		if decoder.Decode(&in) != nil || decoder.Decode(&struct{}{}) != io.EOF {
			w.WriteHeader(400)
			_ = json.NewEncoder(w).Encode(EngineError{Code: "invalid_input"})
			return
		}
		ctx, cancel := context.WithTimeout(r.Context(), 35*time.Second)
		defer cancel()
		result, err := discover(ctx, client, in)
		if err != nil {
			w.WriteHeader(422)
			_ = json.NewEncoder(w).Encode(errorCode(err))
			return
		}
		_ = json.NewEncoder(w).Encode(result)
	})
	mux.HandleFunc("POST /v1/reconcile", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		if subtle.ConstantTimeCompare([]byte(r.Header.Get("Authorization")), []byte("Bearer "+serviceToken)) != 1 {
			w.WriteHeader(401)
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, 32768)
		decoder := json.NewDecoder(r.Body)
		decoder.DisallowUnknownFields()
		var in Request
		if decoder.Decode(&in) != nil || decoder.Decode(&struct{}{}) != io.EOF {
			w.WriteHeader(400)
			_ = json.NewEncoder(w).Encode(EngineError{Code: "invalid_input"})
			return
		}
		ctx, cancel := context.WithTimeout(r.Context(), 35*time.Second)
		defer cancel()
		result, err := reconcile(ctx, client, in)
		if err != nil {
			w.WriteHeader(422)
			_ = json.NewEncoder(w).Encode(errorCode(err))
			return
		}
		_ = json.NewEncoder(w).Encode(result)
	})
	return mux
}
func main() {
	if len(os.Args) > 1 && os.Args[1] == "healthcheck" {
		c := http.Client{Timeout: 3 * time.Second}
		r, err := c.Get("http://127.0.0.1:8090/healthz")
		if err != nil {
			os.Exit(1)
		}
		defer r.Body.Close()
		if r.StatusCode != 200 {
			os.Exit(1)
		}
		return
	}
	token := os.Getenv("DDNSUPD_ENGINE_TOKEN")
	if len(token) < 32 {
		log.Fatal("engine service token must have at least 32 characters")
	}
	client := &http.Client{Timeout: 30 * time.Second, CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }}
	server := http.Server{Addr: ":8090", Handler: handler(token, client), ReadHeaderTimeout: 5 * time.Second, ReadTimeout: 10 * time.Second, WriteTimeout: 40 * time.Second, IdleTimeout: 30 * time.Second, MaxHeaderBytes: 16384}
	log.Print("provider engine listening on :8090")
	log.Fatal(server.ListenAndServe())
}
