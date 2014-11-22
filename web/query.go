package web

import (
	"math/big"
)

type QueryRequest struct {
	Scalars []big.Int `json:"Scalars" binding:"required"`
}

type ScalarQueryResult struct {
	Scalar big.Int
	Found []big.Int
}

type QueryResponse struct {
	Scalars []ScalarQueryResult
}

