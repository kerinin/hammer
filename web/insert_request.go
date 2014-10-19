package web

import (
	"math/big"
)

type InsertRequest struct {
	Scalars []big.Int `json:"Scalars" binding:"required"`
	// Vectors [][]big.Int
}
