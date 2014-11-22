package web

import (
	"math/big"
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

