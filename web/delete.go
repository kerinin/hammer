package web

import (
	"github.com/kerinin/hammer/db"
)

type DeleteRequest struct {
	Scalars []db.Key `json:"Scalars" binding:"required"`
}

type ScalarDeleteResult struct {
	Scalar db.Key
	Deleted bool
}

type DeleteResponse struct {
	Scalars []ScalarDeleteResult
}

