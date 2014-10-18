package main

import (
	"math/big"
)

/*
 * Converts a string to a big.Int, like "0011" => 3
 */
func binary(s string) *big.Int {
	bigint := big.NewInt(0)

	for i, r := range s {
		if r == '1' {
			mask := big.NewInt(1)
			mask.Lsh(mask, uint(i))
			bigint.Or(bigint, mask)
		}
	}

	return bigint
}
