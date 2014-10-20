package web

import (
	"math/big"
)

type ScalarRemoveResult struct {
	Scalar big.Int
	Removed bool
}

// type VectorInsertResult struct {
// 	Vector []big.Int
// 	Inserted bool
// }

type RemoveResponse struct {
	Scalars []ScalarRemoveResult
	// Vectors []VectorInsertResult
}
