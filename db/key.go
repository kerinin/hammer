package db

import (
	"fmt"
	"math/big"
	"github.com/cznic/mathutil"
)

type Key struct {
	value *big.Int
}

func NewKey(value *big.Int) Key {
	return Key{value: value}
}

/*
 * Converts a string to a Key, like "0011" => 3
 */
func NewKeyFromBinaryString(value string) Key {
	bigint := big.NewInt(0)

	for i, r := range value {
		if r == '1' {
			mask := big.NewInt(1)
			mask.Lsh(mask, uint(i))
			bigint.Or(bigint, mask)
		}
	}

	return Key{value: bigint}
}

func (k Key) String() string {
	return fmt.Sprintf("%d", k.value)
}

func (k Key) Cmp(other Key) int {
	return k.value.Cmp(other.value)
}

func (k Key) Hamming(other Key) uint {
	z := big.NewInt(0)
	z.Xor(k.value, other.value)
	return uint(mathutil.PopCountBigInt(z))
}

func (k Key) Int(size uint) interface{} {
	switch {
	case size <= 8:
		return uint8(k.value.Int64())
	case size <= 16:
		return uint16(k.value.Int64())
	case size <= 32:
		return uint32(k.value.Int64())
	}
	return uint64(k.value.Int64())
}

func (k Key) Bytes() []byte {
	bytes := k.value.Bytes()
	if len(bytes) == 0 {
		bytes = make([]byte, 1, 1)
	}
	return bytes
}

func (k Key) Permutations(count uint) []Key {
	permutations := make([]Key, count, count)

	for i := 0; i < int(count); i++ {
		permutation := big.NewInt(0)
		permutation.SetBytes(k.value.Bytes())

		if k.value.Bit(i) == 0 {
			permutation.SetBit(permutation, i, 1)
		} else {
			permutation.SetBit(permutation, i, 0)
		}

		permutations[i] = Key{value: permutation}
	}

	return permutations
}


func (k Key) Transform(shift uint, mask *big.Int) Key {
	transformed := big.NewInt(0)
	transformed.SetBytes(k.value.Bytes())

	transformed.Rsh(transformed, shift)
	transformed.And(transformed, mask)

	return Key{value: transformed}
}
