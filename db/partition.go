package db

import (
	"fmt"
	"math/big"
)

type Partition struct {
	shift uint
	mask  uint
	zero_kv    map[interface{}][]*big.Int
	one_kv    map[interface{}][]*big.Int
}

func NewPartition(shift uint, mask uint) Partition {
	zero_kv := make(map[interface{}][]*big.Int)
	one_kv := make(map[interface{}][]*big.Int)

	return Partition{shift: shift, mask: mask, zero_kv: zero_kv, one_kv: one_kv}
}

func (p *Partition) Coords() (uint, uint) {
	return p.shift, p.mask
}

func (p *Partition) Find(key *big.Int) (map[*big.Int]uint, error) {
	transformed_key := p.transformKey(key)
	permutations := p.permuteKey(transformed_key)
	found_keys := make(map[*big.Int]uint)

	for _, permuted_key := range permutations {
		permuted_key_int, err := p.toInt(permuted_key)
		if err != nil {
			return found_keys, err
		}
		source_keys, ok := p.one_kv[permuted_key_int]
		if ok {
			for _, source_key := range source_keys {
				found_keys[source_key] = 1
			}
		}
	}

	transformed_key_int, err := p.toInt(transformed_key)
	if err != nil {
		return found_keys, err
	}
	source_keys, ok := p.zero_kv[transformed_key_int]
	if ok {
		for _, source_key := range source_keys {
			found_keys[source_key] = 0
		}
	}

	return found_keys, nil
}

func (p *Partition) Insert(key *big.Int) (bool, error) {
	transformed_key := p.transformKey(key)
	transformed_key_int, err := p.toInt(transformed_key)
	if err != nil {
		return false, err
	}

	if insertKey(&p.zero_kv, transformed_key_int, key) {
		permuted_keys := p.permuteKey(transformed_key)
		for _, permuted_key := range permuted_keys {
			permuted_key_int, err := p.toInt(permuted_key)
			if err != nil {
				return false, err
			}

			insertKey(&p.one_kv, permuted_key_int, key)
		}
		return true, nil
	}

	return false, nil
}

func insertKey(kv *map[interface{}][]*big.Int, key interface{}, value *big.Int) bool {
	found_values, ok := (*kv)[key]

	if ok {
		for _, found_value := range found_values {
			if found_value.Cmp(value) == 0 {
				return false
			}		
		}

		(*kv)[key] = append(found_values, value)
	} else {
		(*kv)[key] = []*big.Int{value}
	}

	return true
}

func (p *Partition) Remove(key *big.Int) (bool, error) {
	transformed_key := p.transformKey(key)
	transformed_key_int, err := p.toInt(transformed_key)
	if err != nil {
		return false, err
	}

	if removeKey(&p.zero_kv, transformed_key_int, key) {
		permuted_keys := p.permuteKey(transformed_key)
		for _, permuted_key := range permuted_keys {
			permuted_key_int, err := p.toInt(permuted_key)
			if err != nil {
				return false, err
			}

			removeKey(&p.one_kv, permuted_key_int, key)
		}
		return true, nil
	}

	return false, nil
}

func removeKey(kv *map[interface{}][]*big.Int, key interface{}, value *big.Int) bool {
	found_values, ok := (*kv)[key]

	if ok {
		for i, found_value := range found_values {
			if found_value.Cmp(value) == 0 {
				// Seriously, THIS is how I have to delete elements in Go?!?!?!
				copy(found_values[i:], found_values[i+1:])
				found_values[len(found_values)-1] = nil
				(*kv)[key] = found_values[:len(found_values)-1]

				return true
			}		
		}
	}

	return false
}

func (p *Partition) transformKey(key *big.Int) *big.Int {
	transformed := big.NewInt(0)
	transformed.SetBytes(key.Bytes())

	transformed.Rsh(transformed, p.shift)
	transformed.And(transformed, p.maskBytes())

	return transformed
}

func (p *Partition) permuteKey(key *big.Int) []*big.Int {
	permutations := make([]*big.Int, 0, 0)

	for i := 0; i < key.BitLen(); i++ {
		permutation := big.NewInt(0)
		permutation.Or(permutation, key)

		if key.Bit(i) == 0 {
			permutation.SetBit(permutation, i, 1)
		} else {
			permutation.SetBit(permutation, i, 0)
		}

		permutations = append(permutations, permutation)
	}

	return permutations
}

func (p *Partition) toInt(key *big.Int) (interface{}, error) {
	switch {
	case p.mask <= 8:
		return uint8(key.Int64()), nil
	case p.mask <= 16:
		return uint16(key.Int64()), nil
	case p.mask <= 32:
		return uint32(key.Int64()), nil
	case p.mask <= 64:
		return uint64(key.Int64()), nil
	}
	return nil, fmt.Errorf("Mask too long - consider decreasing key size or increasing max Hamming distance")
}

func (p *Partition) maskBytes() *big.Int {
	mask := big.NewInt(0)

	for i := 0; i < int(p.mask); i++ {
		mask.SetBit(mask, i, 1)
	}

	return mask
}
