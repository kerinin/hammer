package web

import (
	"fmt"
	"log"
	"regexp"
	"strconv"

	"encoding/json"
	"math/big"

	"github.com/go-martini/martini"
	"github.com/martini-contrib/binding"
	"github.com/kerinin/hammer/db"
)

var databases map[string]db.Partitioning
var findRegex = regexp.MustCompile(`\/db\/(\d+)\/(\d+)\/(.*)\/find`)
var insertRegex = regexp.MustCompile(`\/db\/(\d+)\/(\d+)\/(.*)\/insert`)

func Server() {
	databases = make(map[string]db.Partitioning)

	m := martini.Classic()

	m.Post("/db/(?P<bits>\\d+)/(?P<tolerance>\\d+)/(?P<namespace>.*)/insert$", binding.Json(InsertRequest{}), insertHandler)
	m.Post("/db/(?P<bits>\\d+)/(?P<tolerance>\\d+)/(?P<namespace>.*)/find$", binding.Json(FindRequest{}), findHandler)

	m.Run()
}

func findHandler(request FindRequest, params martini.Params, logger *log.Logger) (int, string) {
	logger.Printf("Handling Find with request:%v, params:%v", request, params)

	bits, err := strconv.ParseUint(params["bits"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	tolerance, err := strconv.ParseUint(params["tolerance"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	namespace := params["namespace"]
	find_db_key := fmt.Sprintf("%v/%v/%v", bits, tolerance, namespace)
	response := FindResponse{}

	find_db, ok := databases[find_db_key]
	if !ok {
		logger.Printf("Initializing db %v", find_db_key)
		find_db = db.NewPartitioning(uint(bits), uint(tolerance))
		databases[find_db_key] = find_db
	}

	scalars := request.Scalars
	response.Scalars = make([]ScalarFindResult, 0, len(scalars))

	for _, scalar := range request.Scalars {
		safe_scalar := big.NewInt(0)
		safe_scalar.SetBytes(scalar.Bytes())

		found_map, err := find_db.Find(safe_scalar)
		if err != nil {
			return 500, err.Error()
		}

		found := make([]big.Int, 0, len(found_map))
		for i, _ := range found_map {
			found = append(found, *i)
		}

		logger.Printf("Queried %v with %016b: %v", find_db_key, safe_scalar, found_map)
		response.Scalars = append(response.Scalars, ScalarFindResult{Scalar: *safe_scalar, Found: found})
	}

	json_bytes, err := json.Marshal(response)
	if err != nil {
		return 500, err.Error()
	}

	return 200, string(json_bytes)
}

func insertHandler(request InsertRequest, params martini.Params, logger *log.Logger) (int, string) {
	logger.Printf("Handling Insert with request:%v, params:%v", request, params)

	bits, err := strconv.ParseUint(params["bits"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	tolerance, err := strconv.ParseUint(params["tolerance"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	namespace := params["namespace"]
	insert_db_key := fmt.Sprintf("%v/%v/%v", bits, tolerance, namespace)
	response := InsertResponse{}

	insert_db, ok := databases[insert_db_key]
	if !ok {
		logger.Printf("Initializing db %v", insert_db_key)
		insert_db = db.NewPartitioning(uint(bits), uint(tolerance))
		databases[insert_db_key] = insert_db
	}

	scalars := request.Scalars
	response.Scalars = make([]ScalarInsertResult, 0, len(scalars))

	for _, scalar := range request.Scalars {
		safe_scalar := big.NewInt(0)
		safe_scalar.SetBytes(scalar.Bytes())

		inserted, err := insert_db.Insert(safe_scalar)
		if err != nil {
			return 500, err.Error()
		}

		logger.Printf("Inserted into %v %016b: %v", insert_db_key, safe_scalar, inserted)

		response.Scalars = append(response.Scalars, ScalarInsertResult{Scalar: *safe_scalar, Inserted: inserted})
	}

	json_bytes, err := json.Marshal(response)
	if err != nil {
		return 500, err.Error()
	}

	return 200, string(json_bytes)
}
