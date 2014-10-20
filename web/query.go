package web

import (
	"fmt"
	"log"
	"strconv"

	"encoding/json"
	"math/big"

	"github.com/go-martini/martini"
	"github.com/kerinin/hammer/db"
)

type QueryRequest struct {
	Scalars []big.Int `json:"Scalars" binding:"required"`
	// Vector [][]big.Int
}

type ScalarQueryResult struct {
	Scalar big.Int
	Found []big.Int
}

type QueryResponse struct {
	Scalars []ScalarQueryResult
}

func queryHandler(request QueryRequest, params martini.Params, logger *log.Logger) (int, string) {
	logger.Printf("Handling Query with request:%v, params:%v", request, params)

	bits, err := strconv.ParseUint(params["bits"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	tolerance, err := strconv.ParseUint(params["tolerance"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	namespace := params["namespace"]
	query_db_key := fmt.Sprintf("%v/%v/%v", bits, tolerance, namespace)
	response := QueryResponse{}

	databases_mutex.RLock()
	query_db, ok := databases[query_db_key]
	databases_mutex.RUnlock()
	if !ok {
		logger.Printf("Initializing db %v", query_db_key)
		query_db = db.NewPartitioning(uint(bits), uint(tolerance))
		databases_mutex.Lock()
		databases[query_db_key] = query_db
		databases_mutex.Unlock()
	}

	scalars := request.Scalars
	response.Scalars = make([]ScalarQueryResult, 0, len(scalars))

	for _, scalar := range request.Scalars {
		safe_scalar := big.NewInt(0)
		safe_scalar.SetBytes(scalar.Bytes())

		found_map, err := query_db.Find(safe_scalar)
		if err != nil {
			return 500, err.Error()
		}

		found := make([]big.Int, 0, len(found_map))
		for i, _ := range found_map {
			found = append(found, *i)
		}

		logger.Printf("Queried %v with %016b: %v", query_db_key, safe_scalar, found_map)
		response.Scalars = append(response.Scalars, ScalarQueryResult{Scalar: *safe_scalar, Found: found})
	}

	json_bytes, err := json.Marshal(response)
	if err != nil {
		return 500, err.Error()
	}

	return 200, string(json_bytes)
}
