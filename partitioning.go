package main

import (
	"math"
	"math/big"

	"gopkg.in/fatih/set.v0"
)

type Partitioning struct {
	partitions []Partition
}

func NewPartitioning(bits uint, max_hamming_distance uint) Partitioning {
	var partition_count uint

	switch {
	case max_hamming_distance == 0:
		partition_count = 1
	case max_hamming_distance > bits:
		partition_count = bits
	case true:
		partition_count = max_hamming_distance
	}

	head_width := uint(math.Ceil(float64(bits) / float64(partition_count)))
	tail_width := uint(math.Floor(float64(bits) / float64(partition_count)))

	head_count := bits % partition_count
	tail_count := partition_count - head_count

	partitions := make([]Partition, 0, 0)

	for i := uint(0); i < head_count; i++ {
		shift := i * head_width
		mask := head_width

		partitions[i] = NewPartition(shift, mask).(Partition)
	}

	for i := uint(0); i < tail_count; i++ {
		shift := (head_count * head_width) + (i * tail_width)
		mask := tail_width;

		partitions[head_count + i] = NewPartition(shift, mask).(Partition)
	}

	return Partitioning{partitions: partitions}
}

// NOTE: This isn't actually correct - see paper for pointers
func (p *Partitioning) Find(key big.Int) (set.Set, error) {
	results := set.New()

	for _, partition := range(p.partitions) {
		keys, err := partition.Find(&key)
		if err != nil {
			return *results, err
		}

		for {
			found_key := keys.Pop()
			if found_key != nil {
				results.Add(found_key)
			} else {
				break
			}
		}
	}

	return *results, nil
}

func (p *Partitioning) Insert(key big.Int) (bool, error) {
	any_inserted := false

	for _, partition := range(p.partitions) {
		added, err := partition.Insert(&key)
		if err != nil {
			return any_inserted, err
		}

		any_inserted = any_inserted || added
	}

	return any_inserted, nil
}

func (p *Partitioning) Remove(key big.Int) (bool, error) {
	any_removed := false

	for _, partition := range(p.partitions) {
		removed, err := partition.Remove(&key)
		if err != nil {
			return any_removed, err
		}

		any_removed = any_removed || removed
	}

	return any_removed, nil
}
