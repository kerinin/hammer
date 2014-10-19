package web

import (
	"math/big"
)

type ScalarInsertResult struct {
	Scalar big.Int
	Inserted bool
}

type VectorInsertResult struct {
	Vector []big.Int
	Inserted bool
}

type InsertResponse struct {
	Scalar []ScalarInsertResult
	Vector []VectorInsertResult
}
