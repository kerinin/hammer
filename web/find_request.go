package web

import (
	"math/big"
)

type FindRequest struct {
	Scalars []big.Int `json:"Scalars" binding:"required"`
	// Vector [][]big.Int
}
