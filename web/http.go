package web

import (
	"fmt"
	"regexp"
	"strconv"

	"io/ioutil"
	"encoding/json"
	"math/big"
	"net/http"

	"github.com/kerinin/hammer/db"
)

var databases map[string]db.Partitioning
var findRegex = regexp.MustCompile(`\/db\/(\d+)\/(\d+)\/(.*)\/find`)
var insertRegex = regexp.MustCompile(`\/db\/(\d+)\/(\d+)\/(.*)\/insert`)

func Server(listen string) {
	databases = make(map[string]db.Partitioning)

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		http.NotFound(w, r)
	})

	http.HandleFunc("/db/", func(w http.ResponseWriter, r *http.Request) {
		findMatches := findRegex.FindStringSubmatch(r.URL.Path)
		if findMatches != nil {
			findHandler(w, r, findMatches)
			return
		}

		insertMatches := insertRegex.FindStringSubmatch(r.URL.Path)
		if insertMatches != nil {
			insertHandler(w, r, insertMatches)
			return
		}

		http.NotFound(w, r)
	})

	http.ListenAndServe(listen, nil)
}

func findHandler(w http.ResponseWriter, r *http.Request, routeMatches []string) {
	bits, err := strconv.ParseUint(routeMatches[1], 0, 64)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}
	tolerance, err := strconv.ParseUint(routeMatches[2], 0, 64)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}
	namespace := routeMatches[3]
	find_db_key := fmt.Sprintf("%v/%v/%v", bits, tolerance, namespace)
	find_request := FindRequest{}
	find_response := FindResponse{}

	find_db, ok := databases[find_db_key]
	if !ok {
		find_db = db.NewPartitioning(uint(bits), uint(tolerance))
		databases[find_db_key] = find_db
	}

	body, err := ioutil.ReadAll(r.Body)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}
	if err := json.Unmarshal(body, find_request); err != nil {
		http.Error(w, err.Error(), 500)
		return
	}

	scalars := find_request.Scalars()
	find_response.Scalar = make([]ScalarFindResult, 0, len(scalars))

	for _, scalar := range find_request.Scalars() {
		found_map, err := find_db.Find(&scalar)
		found := make([]big.Int, 0, len(found_map))

		if err != nil {
			http.Error(w, err.Error(), 500)
			return
		}
		for i, _ := range found_map {
			found[len(found)] = *i
		}

		find_response.Scalar = append(find_response.Scalar, ScalarFindResult{Scalar: scalar, Found: found})
	}

	json_bytes, err := json.Marshal(find_response)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}

	w.WriteHeader(http.StatusOK)
	w.Write(json_bytes)
}

func insertHandler(w http.ResponseWriter, r *http.Request, routeMatches []string) {
	bits, err := strconv.ParseUint(routeMatches[1], 0, 64)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}
	tolerance, err := strconv.ParseUint(routeMatches[2], 0, 64)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}
	namespace := routeMatches[3]
	insert_db_key := fmt.Sprintf("%v/%v/%v", bits, tolerance, namespace)
	insert_request := InsertRequest{}
	insert_response := InsertResponse{}

	insert_db, ok := databases[insert_db_key]
	if !ok {
		insert_db = db.NewPartitioning(uint(bits), uint(tolerance))
		databases[insert_db_key] = insert_db
	}

	body, err := ioutil.ReadAll(r.Body)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}
	if err := json.Unmarshal(body, insert_request); err != nil {
		http.Error(w, err.Error(), 500)
		return
	}

	scalars := insert_request.Scalars()
	insert_response.Scalar = make([]ScalarInsertResult, 0, len(scalars))

	for _, scalar := range insert_request.Scalars() {
		inserted, err := insert_db.Insert(&scalar)
		if err != nil {
			http.Error(w, err.Error(), 500)
			return
		}

		insert_response.Scalar = append(insert_response.Scalar, ScalarInsertResult{Scalar: scalar, Inserted: inserted})
	}

	json_bytes, err := json.Marshal(insert_response)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}

	w.WriteHeader(http.StatusOK)
	w.Write(json_bytes)
}
