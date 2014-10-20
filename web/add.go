package web

import (
	"fmt"
	"log"
	"strconv"

	"math/big"
	"encoding/json"

	"github.com/go-martini/martini"
	"github.com/kerinin/hammer/db"
)

type AddRequest struct {
	Scalars []big.Int `json:"Scalars" binding:"required"`
}

type ScalarAddResult struct {
	Scalar big.Int
	Added bool
}

type AddResponse struct {
	Scalars []ScalarAddResult
}

func addHandler(request AddRequest, params martini.Params, logger *log.Logger) (int, string) {
	logger.Printf("Handling Add with request:%v, params:%v", request, params)

	bits, err := strconv.ParseUint(params["bits"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	tolerance, err := strconv.ParseUint(params["tolerance"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	namespace := params["namespace"]
	add_db_key := fmt.Sprintf("%v/%v/%v", bits, tolerance, namespace)
	response := AddResponse{}

	databases_mutex.RLock()
	add_db, ok := databases[add_db_key]
	databases_mutex.RUnlock()
	if !ok {
		logger.Printf("Initializing db %v", add_db_key)
		add_db = db.NewPartitioning(uint(bits), uint(tolerance))
		databases_mutex.Lock()
		databases[add_db_key] = add_db
		databases_mutex.Unlock()
	}

	scalars := request.Scalars
	response.Scalars = make([]ScalarAddResult, 0, len(scalars))

	for _, scalar := range request.Scalars {
		safe_scalar := big.NewInt(0)
		safe_scalar.SetBytes(scalar.Bytes())

		added, err := add_db.Insert(safe_scalar)
		if err != nil {
			return 500, err.Error()
		}

		logger.Printf("Added into %v %016b: %v", add_db_key, safe_scalar, added)

		response.Scalars = append(response.Scalars, ScalarAddResult{Scalar: *safe_scalar, Added: added})
	}

	json_bytes, err := json.Marshal(response)
	if err != nil {
		return 500, err.Error()
	}

	return 200, string(json_bytes)
}
