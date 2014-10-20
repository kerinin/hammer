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

type DeleteRequest struct {
	Scalars []big.Int `json:"Scalars" binding:"required"`
	// Vector [][]big.Int
}

type ScalarDeleteResult struct {
	Scalar big.Int
	Deleted bool
}

// type VectorInsertResult struct {
// 	Vector []big.Int
// 	Inserted bool
// }

type DeleteResponse struct {
	Scalars []ScalarDeleteResult
	// Vectors []VectorInsertResult
}

func deleteHandler(request DeleteRequest, params martini.Params, logger *log.Logger) (int, string) {
	logger.Printf("Handling Delete with request:%v, params:%v", request, params)

	bits, err := strconv.ParseUint(params["bits"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	tolerance, err := strconv.ParseUint(params["tolerance"], 0, 64)
	if err != nil {
		return 500, err.Error()
	}
	namespace := params["namespace"]
	delete_db_key := fmt.Sprintf("%v/%v/%v", bits, tolerance, namespace)
	response := DeleteResponse{}

	databases_mutex.RLock()
	delete_db, ok := databases[delete_db_key]
	databases_mutex.RUnlock()
	if !ok {
		logger.Printf("Initializing db %v", delete_db_key)
		delete_db = db.NewPartitioning(uint(bits), uint(tolerance))
		databases_mutex.Lock()
		databases[delete_db_key] = delete_db
		databases_mutex.Unlock()
	}

	scalars := request.Scalars
	response.Scalars = make([]ScalarDeleteResult, 0, len(scalars))

	for _, scalar := range request.Scalars {
		safe_scalar := big.NewInt(0)
		safe_scalar.SetBytes(scalar.Bytes())

		deleted, err := delete_db.Remove(safe_scalar)
		if err != nil {
			return 500, err.Error()
		}

		logger.Printf("Deleted from %v %016b: %v", delete_db_key, safe_scalar, deleted)

		response.Scalars = append(response.Scalars, ScalarDeleteResult{Scalar: *safe_scalar, Deleted: deleted})
	}

	json_bytes, err := json.Marshal(response)
	if err != nil {
		return 500, err.Error()
	}

	return 200, string(json_bytes)
}
