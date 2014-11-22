package web

import (
	"math/big"
)

type DeleteRequest struct {
	Scalars []big.Int `json:"Scalars" binding:"required"`
}

type ScalarDeleteResult struct {
	Scalar big.Int
	Deleted bool
}

type DeleteResponse struct {
	Scalars []ScalarDeleteResult
}

