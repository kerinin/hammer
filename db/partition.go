package db

import (
	"fmt"

	"math/big"
)

type Partition struct {
	shift      uint
	mask       uint
	zero_kv    KV
	one_kv     KV
}

func (p Partition) String() string {
	return fmt.Sprintf("[%v,%v]", p.shift, p.mask)
}

func (p *Partition) Coords() (uint, uint) {
	return p.shift, p.mask
}

func (p *Partition) Find(key Key) (map[Key]uint, error) {
	logger.Info("Tring to find %v in partition %v", key, p)

	transformed_key := key.Transform(p.shift, p.maskBytes())
	found_keys := make(map[Key]uint)

	for _, permuted_key := range transformed_key.Permutations(p.mask) {
		source_keys, ok := p.one_kv.Get(permuted_key.Int(p.mask))
		if ok {
			for _, source_key := range source_keys {
				logger.Debug("Found partial match %v for %v in partition %v", source_key, key, p)
				found_keys[source_key] = 1
			}
		}
	}

	source_keys, ok := p.zero_kv.Get(transformed_key.Int(p.mask))
	if ok {
		for _, source_key := range source_keys {
			logger.Debug("Found exact match %v for %v in partition %v", source_key, key, p)
			found_keys[source_key] = 0
		}
	}

	return found_keys, nil
}

func (p *Partition) Insert(key Key) (bool, error) {
	logger.Info("Trying to insert %v in partition %v", key, p)

	transformed_key := key.Transform(p.shift, p.maskBytes())

	if p.zero_kv.Add(transformed_key.Int(p.mask), key) {
		// NOTE: That second bit should be monitonically increasing

		logger.Debug("Inserted exact match %v in partition %v", transformed_key.Int(p.mask), p)

		for _, permuted_key := range transformed_key.Permutations(p.mask) {
			logger.Debug("Inserted partial match %v in partition %v", permuted_key.Int(p.mask), p)
			p.one_kv.Add(permuted_key.Int(p.mask), key)
		}

		return true, nil
	} else {
	}

	logger.Debug("Found %v in partition %v, not inserting", key, p)
	return false, nil
}

func (p *Partition) Remove(key Key) (bool, error) {
	transformed_key := key.Transform(p.shift, p.maskBytes())

	if p.zero_kv.Remove(transformed_key.Int(p.mask), key) {
		// NOTE: That second bit should match the value on insertion

		for _, permuted_key := range transformed_key.Permutations(p.mask) {
			p.one_kv.Remove(permuted_key.Int(p.mask), key)
		}

		return true, nil
	}

	return false, nil
}

func (p *Partition) maskBytes() *big.Int {
	mask := big.NewInt(0)

	for i := 0; i < int(p.mask); i++ {
		mask.SetBit(mask, i, 1)
	}

	return mask
}
