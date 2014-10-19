package web

import (
	"math/big"
	"encoding/base64"
)

type InsertRequest struct {
	ScalarBase64 []string
	VectorBase64 [][]string

	Scalar []big.Int
	Vector [][]big.Int
}

func (r *InsertRequest) Scalars() []big.Int {
	scalars := make([]big.Int, 0, len(r.ScalarBase64))

	for _, s := range r.ScalarBase64 {
		data, err := base64.StdEncoding.DecodeString(s)

		//  Going to just ignore decoding errors for now...
		if err == nil {
			scalar := big.NewInt(0)
			scalar.SetBytes(data)
			scalars[len(scalars)] = *scalar
		}
	}

	for _, s := range r.Scalar {
		scalars[len(scalars)] = s
	}

	return scalars
}
