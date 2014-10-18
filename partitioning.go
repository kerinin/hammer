package main

import (
	"math"
	"math/big"
)

type Partitioning struct {
	bits            uint
	tolerance       uint
	partition_count uint
	partitions      []Partition
}

/*
 * bits:      The bit size of the keys to be stored
 * tolerance: Queries will return all keys whose hamming distance to the query
 *            is equal to or less than this value
 */
func NewPartitioning(bits uint, tolerance uint) Partitioning {
	var partition_count uint

	switch {
	case tolerance == 0:
		partition_count = 1
	case tolerance > bits:
		partition_count = (bits + 3) / 2
	case true:
		partition_count = (tolerance + 3) / 2
	}

	head_width := uint(math.Ceil(float64(bits) / float64(partition_count)))
	tail_width := uint(math.Floor(float64(bits) / float64(partition_count)))

	head_count := bits % partition_count
	tail_count := partition_count - head_count

	partitions := make([]Partition, partition_count, partition_count)

	for i := uint(0); i < head_count; i++ {
		shift := i * head_width
		mask := head_width

		partitions[i] = NewPartition(shift, mask)
	}

	for i := uint(0); i < tail_count; i++ {
		shift := (head_count * head_width) + (i * tail_width)
		mask := tail_width

		partitions[head_count+i] = NewPartition(shift, mask)
	}

	return Partitioning{bits: bits, tolerance: tolerance, partition_count: partition_count, partitions: partitions}
}

/*
 * Returns a map from keys to their hamming distance to the query
 */
func (p *Partitioning) Find(key *big.Int) (map[*big.Int]uint, error) {
	candidates := make(map[*big.Int][2]uint)
	matches := make(map[*big.Int]uint)

	for _, partition := range p.partitions {
		p_candidates, err := partition.Find(key)
		if err != nil {
			return matches, err
		}

		for k, v := range p_candidates {
			current := candidates[k]

			if v == 0 {
				// Exact match
				current[0] = current[0] + 1
			} else {
				// 1-match
				current[1] = current[1] + 1
			}
			candidates[k] = current
		}
	}

	if p.tolerance%2 == 0 {
		for k, v := range candidates {
			// "If k is an even number, S must have at least one exact-matching
			// partition, or two 1-matching partitions"
			if v[0] >= 1 || v[1] >= 2 {
				if h := hamming(k, key); h <= p.tolerance {
					matches[k] = h
				}
			}
		}
	} else {
		for k, v := range candidates {
			// "If k is an odd number, S must have at least two matching partitions
			// where at least one of the matches should be an exact match, or S
			// must have at least three 1-matching partitions"
			if (v[0] >= 1 && (v[0]+v[1]) >= 2) || v[1] >= 3 {
				if h := hamming(k, key); h <= p.tolerance {
					matches[k] = h
				}
			}
		}
	}

	return matches, nil
}

func (p *Partitioning) Insert(key *big.Int) (bool, error) {
	any_inserted := false

	for _, partition := range p.partitions {
		added, err := partition.Insert(key)
		if err != nil {
			return any_inserted, err
		}

		any_inserted = any_inserted || added
	}

	return any_inserted, nil
}

func (p *Partitioning) Remove(key *big.Int) (bool, error) {
	any_removed := false

	for _, partition := range p.partitions {
		removed, err := partition.Remove(key)
		if err != nil {
			return any_removed, err
		}

		any_removed = any_removed || removed
	}

	return any_removed, nil
}
