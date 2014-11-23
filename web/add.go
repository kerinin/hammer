package web

import (
	"github.com/kerinin/hammer/db"
)

type AddRequest struct {
	Scalars []db.Key `json:"Scalars" binding:"required"`
}

type ScalarAddResult struct {
	Scalar db.Key
	Added bool
}

type AddResponse struct {
	Scalars []ScalarAddResult
}

