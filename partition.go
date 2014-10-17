package main

import (
	"fmt"
	"math/big"

	"gopkg.in/fatih/set.v0"
)

type Partition struct {
	shift uint
	mask uint
	kv map[interface{}]big.Int
}

func NewPartition(shift uint, mask uint) Partition {
	return Partition{shift: shift, mask: mask, kv: make(map[interface{}]big.Int)}
}

func (p *Partition) Coords() (uint, uint) {
	return p.shift, p.mask
}

func (p *Partition) Find(key big.Int) (set.Set, error) {
	transformed_key := p.transformKey(key)
	permutations := p.permuteKey(transformed_key)

	found_keys := set.New()

	transformed_key_int, err := p.toInt(transformed_key)
	if err != nil {
		return *found_keys, err
	}
	source_key, ok := p.kv[transformed_key_int]
	if ok {
		found_keys.Add(&source_key)
	}

	for _, permuted_key := range(permutations) {
		permuted_key_int, err := p.toInt(permuted_key)
		if err != nil {
			return *found_keys, err
		}
		source_key, ok := p.kv[permuted_key_int]
		if ok {
			found_keys.Add(&source_key)
		}
	}

	return *found_keys, nil
}

func (p *Partition) Insert(key big.Int) (bool, error) {
	transformed_key := p.transformKey(key)
	permuted_keys := p.permuteKey(transformed_key)

	transformed_key_int, err := p.toInt(transformed_key)
	if err != nil {
		return false, err
	}
	_, found := p.kv[transformed_key_int]
	p.kv[transformed_key_int] = key

	for _, permuted_key := range(permuted_keys) {
		permuted_key_int, err := p.toInt(permuted_key)
		if err != nil {
			return false, err
		}
		p.kv[permuted_key_int] = key
	}

	return !found, nil
}

func (p *Partition) Remove(key big.Int) (bool, error) {
	transformed_key := p.transformKey(key)
	permuted_keys := p.permuteKey(transformed_key)

	transformed_key_int, err := p.toInt(transformed_key)
	if err != nil {
		return false, err
	}

	_, found := p.kv[transformed_key_int]
	delete(p.kv, transformed_key_int)

	for _, permuted_key := range(permuted_keys) {
		permuted_key_int, err := p.toInt(permuted_key)
		if err != nil {
			return false, err
		}
		delete(p.kv, permuted_key_int)
	}

	return found, nil
}

func (p *Partition) transformKey(key big.Int) big.Int {
	transformed := big.NewInt(0)

	transformed.Or(transformed, &key)
	transformed.Lsh(transformed, p.shift)
	transformed.Or(transformed, p.maskBytes())

	return *transformed
}

func (p *Partition) permuteKey(key big.Int) []big.Int {
	permutations := make([]big.Int, 0, 0)

	for i := 0; i < key.BitLen(); i++ {
		permutation := big.NewInt(0)
		permutation.Or(permutation, &key)

		if key.Bit(i) == 0 {
			permutation.SetBit(permutation, i, 1)
		} else {
			permutation.SetBit(permutation, i, 0)
		}

		permutations = append(permutations, *permutation)
	}

	return permutations
}

func (p *Partition) toInt(key big.Int) (interface{}, error) {
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
