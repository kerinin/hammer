package web

import (
	"math/big"
)

type ScalarFindResult struct {
	Scalar big.Int
	Found []big.Int
}

type VectorFindResult struct {
	Vector []big.Int
	Found [][]big.Int
}

type FindResponse struct {
	Scalar []ScalarFindResult
	Vector []VectorFindResult
}
