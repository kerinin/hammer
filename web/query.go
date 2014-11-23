package web

import (
	"github.com/kerinin/hammer/db"
)

type QueryRequest struct {
	Scalars []db.Key `json:"Scalars" binding:"required"`
}

type ScalarQueryResult struct {
	Scalar db.Key
	Found []db.Key
}

type QueryResponse struct {
	Scalars []ScalarQueryResult
}

