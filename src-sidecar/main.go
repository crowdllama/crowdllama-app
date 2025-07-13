package main

import (
	"fmt"
	"log"
	"net/http"
)

func main() {
	port := 8181

	log.Printf("Starting HTTP server on port %d", port)

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "hello world")
	})

	addr := fmt.Sprintf(":%d", port)
	log.Fatal(http.ListenAndServe(addr, nil))
}
