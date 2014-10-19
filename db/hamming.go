package db

import (
	"math/big"

	"github.com/cznic/mathutil"
)

func hamming(x *big.Int, y *big.Int) uint {
	z := big.NewInt(0)
	z.Xor(x, y)
	return uint(mathutil.PopCountBigInt(z))
}
