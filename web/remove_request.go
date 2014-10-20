package web

import (
	"math/big"
)

type RemoveRequest struct {
	Scalars []big.Int `json:"Scalars" binding:"required"`
	// Vector [][]big.Int
}
