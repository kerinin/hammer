package main

import (
	"bytes"
	"time"

	"encoding/json"
	"math/rand"
	"net/http"
)

type Request struct {
	Scalars *[]uint `json:"Scalars" binding:"required"`
}

func main() {
	// ~10,000 requests total
	concurrency := 33
	repetitions := 10000
	sleep := 0 * time.Millisecond

	done := make(chan bool)

	for i := 0; i < concurrency; i ++ {
		go func() {
			for j := 0; j < repetitions; j ++ {
				doRequests()
				time.Sleep(sleep)
			}
			done <- true
		}()
	}

	for i := 0; i < concurrency; i ++ {
		<-done
	}
}

func doRequests() {
	keys := make([]uint, 10, 10)

	for j := 0; j < 10; j ++ {
		keys[j] = uint(rand.Int63())
	}
	body := &Request{Scalars: &keys}
	json_bytes, _ := json.Marshal(body)

	http.Post("http://localhost:3000/add", "application/json", bytes.NewReader(json_bytes))
	http.Post("http://localhost:3000/query", "application/json", bytes.NewReader(json_bytes))
	http.Post("http://localhost:3000/delete", "application/json", bytes.NewReader(json_bytes))
}
