package db

import (
	"os"
	"fmt"
	"github.com/bitly/dablooms/godablooms"
	"math/big"
	"sync"
)

type Partition struct {
	shift      uint
	mask       uint
	zero_mutex sync.RWMutex
	one_mutex  sync.RWMutex
	zero_bloom *dablooms.ScalingBloom
	one_bloom  *dablooms.ScalingBloom
	zero_kv    map[interface{}][]Key
	one_kv     map[interface{}][]Key
}

func NewPartition(shift uint, mask uint) Partition {
	os.Remove("zero_bloom.dat")
	zero_bloom := dablooms.NewScalingBloom(100000, 0.5, "zero_bloom.dat")
	if zero_bloom == nil {
		logger.Fatal("Failed to create zero bloom")
	}
	os.Remove("one_bloom.dat")
	one_bloom := dablooms.NewScalingBloom(100000, 0.5, "one_bloom.dat")
	if one_bloom == nil {
		logger.Fatal("Failed to create one bloom")
	}

	zero_kv := make(map[interface{}][]Key)
	one_kv := make(map[interface{}][]Key)

	zero_mutex := sync.RWMutex{}
	one_mutex := sync.RWMutex{}

	return Partition{
		shift:      shift,
		mask:       mask,
		zero_bloom: zero_bloom,
		one_bloom:  one_bloom,
		zero_kv:    zero_kv,
		one_kv:     one_kv,
		zero_mutex: zero_mutex,
		one_mutex:  one_mutex,
	}
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
		p.one_mutex.RLock()
		if p.one_bloom.Check(permuted_key.Bytes()) {
			source_keys, ok := p.one_kv[permuted_key.Int(p.mask)]
			if ok {
				for _, source_key := range source_keys {
					logger.Debug("Found partial match %v for %v in partition %v", source_key, key, p)
					found_keys[source_key] = 1
				}
			}
		}
		p.one_mutex.RUnlock()
	}

	p.zero_mutex.RLock()
	if p.zero_bloom.Check(transformed_key.Bytes()) {
		source_keys, ok := p.zero_kv[transformed_key.Int(p.mask)]
		if ok {
			for _, source_key := range source_keys {
				logger.Debug("Found exact match %v for %v in partition %v", source_key, key, p)
				found_keys[source_key] = 0
			}
		}
	}
	p.zero_mutex.RUnlock()

	return found_keys, nil
}

func (p *Partition) Insert(key Key) (bool, error) {
	logger.Info("Trying to insert %v in partition %v", key, p)

	transformed_key := key.Transform(p.shift, p.maskBytes())

	p.zero_mutex.Lock()
	if insertKey(&p.zero_kv, transformed_key.Int(p.mask), key) {
		// NOTE: That second bit should be monitonically increasing
		p.zero_bloom.Add(transformed_key.Bytes(), 1)
		p.zero_mutex.Unlock()

		logger.Debug("Inserted exact match %v in partition %v", transformed_key.Int(p.mask), p)

		p.one_mutex.Lock()
		for _, permuted_key := range transformed_key.Permutations(p.mask) {
			logger.Debug("Inserted partial match %v in partition %v", permuted_key.Int(p.mask), p)
			if insertKey(&p.one_kv, permuted_key.Int(p.mask), key) {
				// NOTE: That second bit should be monitonically increasing
				p.one_bloom.Add(permuted_key.Bytes(), 1)
			}
		}
		p.one_mutex.Unlock()

		return true, nil
	} else {
		p.zero_mutex.Unlock()
	}

	logger.Debug("Found %v in partition %v, not inserting", key, p)
	return false, nil
}

func insertKey(kv *map[interface{}][]Key, key interface{}, value Key) bool {
	found_values, ok := (*kv)[key]

	if ok {
		for _, found_value := range found_values {
			if found_value.Cmp(value) == 0 {
				return false
			}
		}
	} else {
		(*kv)[key] = make([]Key, 0, 1)
	}

	(*kv)[key] = append(found_values, value)

	return true
}

func (p *Partition) Remove(key Key) (bool, error) {
	transformed_key := key.Transform(p.shift, p.maskBytes())

	p.zero_mutex.Lock()
	if removeKey(&p.zero_kv, transformed_key.Int(p.mask), key) {
		// NOTE: That second bit should match the value on insertion
		p.zero_bloom.Remove(transformed_key.Bytes(), 1)
		p.zero_mutex.Unlock()

		p.one_mutex.Lock()
		for _, permuted_key := range transformed_key.Permutations(p.mask) {
			if removeKey(&p.one_kv, permuted_key.Int(p.mask), key) {
				// NOTE: That second bit should match the value on insertion
				p.one_bloom.Remove(permuted_key.Bytes(), 1)
			}
		}
		p.one_mutex.Unlock()

		return true, nil
	} else {
		p.zero_mutex.Unlock()
	}

	return false, nil
}

func removeKey(kv *map[interface{}][]Key, key interface{}, value Key) bool {
	found_values, ok := (*kv)[key]

	if ok {
		for i, found_value := range found_values {
			if found_value.Cmp(value) == 0 {
				// Seriously, THIS is how I have to delete elements in Go?!?!?!
				copy(found_values[i:], found_values[i+1:])
				(*kv)[key] = found_values[:len(found_values)-1]

				return true
			}
		}
	}

	return false
}

func (p *Partition) maskBytes() *big.Int {
	mask := big.NewInt(0)

	for i := 0; i < int(p.mask); i++ {
		mask.SetBit(mask, i, 1)
	}

	return mask
}
